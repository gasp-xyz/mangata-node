#![cfg_attr(not(feature = "std"), no_std)]

use messages::{EthAbi, EthAbiHash};
pub mod messages;

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, ExistenceRequirement, Get, StorageVersion},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use messages::{Cancel, FailedDepositResolution, Origin, RequestId, Withdrawal};
use rs_merkle::{Hasher, MerkleProof, MerkleTree};
use scale_info::prelude::{format, string::String};

use sp_runtime::traits::{One, SaturatedConversion, Saturating};
use sp_std::{collections::btree_map::BTreeMap, iter::Iterator};

use alloy_sol_types::SolValue;
use frame_support::{traits::WithdrawReasons, PalletId};
use itertools::Itertools;
use mangata_support::traits::{
	AssetRegistryProviderTrait, GetMaintenanceStatusTrait, RolldownProviderTrait,
	SequencerStakingProviderTrait, SequencerStakingRewardsTrait, SetMaintenanceModeOn,
};
use mangata_types::assets::L1Asset;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use sp_core::{H256, U256};
use sp_crypto_hashing::keccak_256;
use sp_runtime::traits::{AccountIdConversion, Convert, ConvertBack, Zero};
use sp_std::{collections::btree_set::BTreeSet, convert::TryInto, prelude::*, vec::Vec};

pub type CurrencyIdOf<T> = <<T as Config>::Tokens as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

pub type BalanceOf<T> =
	<<T as Config>::Tokens as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance;
pub type ChainIdOf<T> = <T as pallet::Config>::ChainId;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type RoundIndex = u32;

pub(crate) const LOG_TARGET: &'static str = "rolldown";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

#[derive(Debug, PartialEq)]
pub struct EthereumAddressConverter<AccountId>(sp_std::marker::PhantomData<AccountId>);

impl Convert<[u8; 20], sp_runtime::AccountId20>
	for EthereumAddressConverter<sp_runtime::AccountId20>
{
	fn convert(eth_addr: [u8; 20]) -> sp_runtime::AccountId20 {
		eth_addr.into()
	}
}

impl ConvertBack<[u8; 20], sp_runtime::AccountId20>
	for EthereumAddressConverter<sp_runtime::AccountId20>
{
	fn convert_back(eth_addr: sp_runtime::AccountId20) -> [u8; 20] {
		eth_addr.into()
	}
}

#[derive(Clone)]
pub struct Keccak256Hasher {}

impl Hasher for Keccak256Hasher {
	type Hash = [u8; 32];

	fn hash(data: &[u8]) -> [u8; 32] {
		let mut output = [0u8; 32];
		let hash = keccak_256(&data[..]);
		output.copy_from_slice(&hash[..]);
		output
	}
}

#[derive(PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum L1DepositProcessingError {
	Overflow,
	AssetRegistrationProblem,
	MintError,
}

impl<T> From<L1DepositProcessingError> for Error<T> {
	fn from(e: L1DepositProcessingError) -> Self {
		match e {
			L1DepositProcessingError::Overflow => Error::<T>::BalanceOverflow,
			L1DepositProcessingError::AssetRegistrationProblem => Error::<T>::L1AssetCreationFailed,
			L1DepositProcessingError::MintError => Error::<T>::MintError,
		}
	}
}

#[derive(PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum L1RequestProcessingError {
	Overflow,
	AssetRegistrationProblem,
	MintError,
	NotEnoughtCancelRights,
	WrongCancelRequestId,
	SequencerNotSlashed,
}

impl From<L1DepositProcessingError> for L1RequestProcessingError {
	fn from(err: L1DepositProcessingError) -> Self {
		match err {
			L1DepositProcessingError::Overflow => Self::Overflow,
			L1DepositProcessingError::AssetRegistrationProblem => Self::AssetRegistrationProblem,
			L1DepositProcessingError::MintError => Self::MintError,
		}
	}
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

use crate::messages::L1Update;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			if T::MaintenanceStatusProvider::is_maintenance() {
				LastMaintananceMode::<T>::put(now.saturated_into::<u128>());
			}

			Self::maybe_create_batch(now);
			Self::schedule_request_for_execution_if_dispute_period_has_passsed(now);
			Self::execute_requests_from_execute_queue(now);
			T::DbWeight::get().reads_writes(20, 20)
		}
	}

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo, Default,
	)]
	pub struct SequencerRights {
		pub read_rights: u128,
		pub cancel_rights: u128,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Copy, Encode, Decode, TypeInfo)]
	pub enum L2Request<AccountId: Clone> {
		FailedDepositResolution(FailedDepositResolution),
		Cancel(Cancel<AccountId>),
		Withdrawal(Withdrawal),
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo)]
	pub enum BatchSource {
		Manual,
		AutomaticSizeReached,
		PeriodReached,
	}

	#[derive(
		PartialOrd, Ord, Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo,
	)]
	pub enum DisputeRole {
		Canceler,
		Submitter,
	}

	#[pallet::storage]
	pub type FerriedDeposits<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::ChainId, H256), T::AccountId, OptionQuery>;

	#[pallet::storage]
	/// stores id of the failed depoisit, so it can be  refunded using [`Pallet::refund_failed_deposit`]
	pub type FailedL1Deposits<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::ChainId, u128), (T::AccountId, H256), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_last_processed_request_on_l2)]
	// Id of the last request originating on other chain that has been executed
	pub type LastProcessedRequestOnL2<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ChainId, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	// Id of the next request that will originate on this chain
	pub type L2OriginRequestId<T: Config> = StorageValue<_, BTreeMap<T::ChainId, u128>, ValueQuery>;

	#[pallet::storage]
	pub type ManualBatchExtraFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_requests)]
	// Stores requests brought by sequencer that are under dispute period.
	pub type PendingSequencerUpdates<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u128,
		Blake2_128Concat,
		T::ChainId,
		(T::AccountId, messages::L1Update, H256),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	// queue of all updates that went through dispute period and are ready to be processed
	pub type UpdatesExecutionQueue<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u128,
		(BlockNumberFor<T>, T::ChainId, messages::L1Update),
		OptionQuery,
	>;

	#[pallet::storage]
	pub type LastMaintananceMode<T: Config> = StorageValue<_, u128, OptionQuery>;

	#[pallet::storage]
	// Id of the next update to be executed
	pub type UpdatesExecutionQueueNextId<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	// Id of the last update that has been executed
	pub type LastScheduledUpdateIdInExecutionQueue<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub type SequencersRights<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::ChainId,
		BTreeMap<T::AccountId, SequencerRights>,
		ValueQuery,
	>;

	//maps Chain and !!!! request origin id!!! to pending update
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_l2_request)]
	pub type L2Requests<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::ChainId,
		Blake2_128Concat,
		RequestId,
		(L2Request<T::AccountId>, H256),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_awaiting_cancel_resolution)]
	pub type AwaitingCancelResolution<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::ChainId,
		BTreeSet<(T::AccountId, u128, DisputeRole)>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_last_update_by_sequencer)]
	pub type LastUpdateBySequencer<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::ChainId, T::AccountId), u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_max_accepted_request_id_on_l2)]
	pub type MaxAcceptedRequestIdOnl2<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ChainId, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_number_of_deposits)]
	pub type TotalNumberOfDeposits<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_number_of_withdrawals)]
	pub type TotalNumberOfWithdrawals<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	pub type L2RequestsBatch<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(ChainIdOf<T>, u128),
		(BlockNumberFor<T>, (u128, u128), AccountIdOf<T>),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	/// For each supported chain stores:
	/// - last batch id
	/// - range of the reqeusts in last batch
	pub type L2RequestsBatchLast<T: Config> =
		StorageValue<_, BTreeMap<T::ChainId, (BlockNumberFor<T>, u128, (u128, u128))>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		L1ReadStored {
			chain: T::ChainId,
			sequencer: T::AccountId,
			dispute_period_end: u128,
			range: messages::Range,
			hash: H256,
		},
		RequestProcessedOnL2 {
			chain: T::ChainId,
			request_id: u128,
			status: Result<(), L1RequestProcessingError>,
		},
		L1ReadCanceled {
			chain: T::ChainId,
			canceled_sequencer_update: u128,
			assigned_id: RequestId,
		},
		TxBatchCreated {
			chain: T::ChainId,
			source: BatchSource,
			assignee: T::AccountId,
			batch_id: u128,
			range: (u128, u128),
		},
		WithdrawalRequestCreated {
			chain: T::ChainId,
			request_id: RequestId,
			recipient: [u8; 20],
			token_address: [u8; 20],
			amount: u128,
			hash: H256,
			ferry_tip: u128,
		},
		ManualBatchExtraFeeSet(BalanceOf<T>),
		DepositRefundCreated {
			chain: ChainIdOf<T>,
			refunded_request_id: RequestId,
			ferry: Option<AccountIdOf<T>>,
		},
		L1ReadScheduledForExecution {
			chain: T::ChainId,
			hash: H256,
		},
		L1ReadIgnoredBecauseOfMaintenanceMode {
			chain: T::ChainId,
			hash: H256,
		},
		DepositFerried {
			chain: T::ChainId,
			deposit: messages::Deposit,
			deposit_hash: H256,
		},
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		OperationFailed,
		ReadRightsExhausted,
		CancelRightsExhausted,
		EmptyUpdate,
		AddressDeserializationFailure,
		RequestDoesNotExist,
		NotEnoughAssets,
		NotEnoughAssetsForFee,
		BalanceOverflow,
		L1AssetCreationFailed,
		MathOverflow,
		TooManyRequests,
		InvalidUpdate,
		L1AssetNotFound,
		WrongRequestId,
		OnlySelectedSequencerisAllowedToUpdate,
		SequencerLastUpdateStillInDisputePeriod,
		SequencerAwaitingCancelResolution,
		MultipleUpdatesInSingleBlock,
		BlockedByMaintenanceMode,
		UnsupportedAsset,
		InvalidRange,
		NonExistingRequestId,
		UnknownAliasAccount,
		FailedDepositDoesNotExist,
		EmptyBatch,
		TokenDoesNotExist,
		NotEligibleForRefund,
		FerryHashMismatch,
		MintError,
		AssetRegistrationProblem,
		UpdateHashMishmatch,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type SequencerStakingProvider: SequencerStakingProviderTrait<
			Self::AccountId,
			BalanceOf<Self>,
			ChainIdOf<Self>,
		>;
		type SequencerStakingRewards: SequencerStakingRewardsTrait<Self::AccountId, RoundIndex>;
		type AddressConverter: ConvertBack<[u8; 20], Self::AccountId>;
		// Dummy so that we can have the BalanceOf type here for the SequencerStakingProviderTrait
		type Tokens: MultiTokenCurrency<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>
			+ MultiTokenCurrencyExtended<Self::AccountId>;
		type AssetRegistryProvider: AssetRegistryProviderTrait<CurrencyIdOf<Self>>;
		#[pallet::constant]
		type DisputePeriodLength: Get<u128>;
		#[pallet::constant]
		type RightsMultiplier: Get<u128>;
		#[pallet::constant]
		type RequestsPerBlock: Get<u128>;
		type MaintenanceStatusProvider: GetMaintenanceStatusTrait + SetMaintenanceModeOn;
		type ChainId: From<messages::Chain>
			+ Parameter
			+ Member
			+ Default
			+ TypeInfo
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ PartialOrd
			+ codec::Decode
			+ codec::Encode
			+ Ord
			+ Copy;
		type AssetAddressConverter: Convert<(ChainIdOf<Self>, [u8; 20]), L1Asset>;
		// How many L2 requests needs to be created so they are grouped and assigned
		// to active sequencer
		#[pallet::constant]
		type MerkleRootAutomaticBatchSize: Get<u128>;
		// How many blocks since last batch needs to be create so the batch is created and assigned to
		// active sequencer
		#[pallet::constant]
		type MerkleRootAutomaticBatchPeriod: Get<u128>;
		type TreasuryPalletId: Get<PalletId>;
		type NativeCurrencyId: Get<CurrencyIdOf<Self>>;
		/// Withdrawals flat fee
		type WithdrawFee: Convert<ChainIdOf<Self>, BalanceOf<Self>>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub _phantom: PhantomData<T>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(3, 3).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn update_l2_from_l1(
			origin: OriginFor<T>,
			requests: messages::L1Update,
			update_hash: H256,
		) -> DispatchResult {
			let sequencer = ensure_signed(origin)?;

			let hash = requests.abi_encode_hash();
			ensure!(update_hash == hash, Error::<T>::UpdateHashMishmatch);

			Self::update_impl(sequencer, requests)
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn force_update_l2_from_l1(
			origin: OriginFor<T>,
			update: messages::L1Update,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;
			let chain: T::ChainId = update.chain.into();

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::BlockedByMaintenanceMode
			);

			Self::validate_l1_update(chain, &update)?;
			let now = <frame_system::Pallet<T>>::block_number();
			Self::schedule_requests(now, chain, update.into());
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		//EXTRINSIC2 (who canceled, dispute_period_end(u32-blocknum)))
		pub fn cancel_requests_from_l1(
			origin: OriginFor<T>,
			chain: T::ChainId,
			requests_to_cancel: u128,
		) -> DispatchResultWithPostInfo {
			let canceler = ensure_signed(origin)?;

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::BlockedByMaintenanceMode
			);

			SequencersRights::<T>::try_mutate(chain, |sequencers| {
				let rights =
					sequencers.get_mut(&canceler).ok_or(Error::<T>::CancelRightsExhausted)?;
				rights.cancel_rights =
					rights.cancel_rights.checked_sub(1).ok_or(Error::<T>::CancelRightsExhausted)?;
				Ok::<_, Error<T>>(())
			})?;

			let (submitter, request, _) =
				PendingSequencerUpdates::<T>::take(requests_to_cancel, chain)
					.ok_or(Error::<T>::RequestDoesNotExist)?;

			let hash_of_pending_request = Self::calculate_hash_of_sequencer_update(request.clone());

			let l2_request_id = Self::acquire_l2_request_id(chain);

			let cancel_request = Cancel {
				requestId: RequestId { origin: Origin::L2, id: l2_request_id },
				updater: submitter.clone(),
				canceler: canceler.clone(),
				range: request.range().ok_or(Error::<T>::InvalidUpdate)?,
				hash: hash_of_pending_request,
			};

			AwaitingCancelResolution::<T>::mutate(chain, |v| {
				v.insert((submitter.clone(), l2_request_id, DisputeRole::Submitter))
			});
			AwaitingCancelResolution::<T>::mutate(chain, |v| {
				v.insert((canceler, l2_request_id, DisputeRole::Canceler))
			});

			L2Requests::<T>::insert(
				chain,
				RequestId::from((Origin::L2, l2_request_id)),
				(L2Request::Cancel(cancel_request.clone()), cancel_request.abi_encode_hash()),
			);

			Pallet::<T>::deposit_event(Event::L1ReadCanceled {
				canceled_sequencer_update: requests_to_cancel,
				chain,
				assigned_id: RequestId { origin: Origin::L2, id: l2_request_id },
			});

			Ok(().into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn withdraw(
			origin: OriginFor<T>,
			chain: T::ChainId,
			recipient: [u8; 20],
			token_address: [u8; 20],
			amount: u128,
			ferry_tip: Option<u128>,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::BlockedByMaintenanceMode
			);

			let eth_asset = T::AssetAddressConverter::convert((chain, token_address));
			let asset_id = T::AssetRegistryProvider::get_l1_asset_id(eth_asset)
				.ok_or(Error::<T>::TokenDoesNotExist)?;

			// fail will occur if user has not enough balance
			<T as Config>::Tokens::ensure_can_withdraw(
				asset_id.into(),
				&account,
				amount.try_into().or(Err(Error::<T>::BalanceOverflow))?,
				WithdrawReasons::all(),
				Default::default(),
			)
			.or(Err(Error::<T>::NotEnoughAssets))?;

			let extra_fee = <<T as Config>::WithdrawFee>::convert(chain);
			if !extra_fee.is_zero() {
				<T as Config>::Tokens::ensure_can_withdraw(
					Self::native_token_id(),
					&account,
					extra_fee,
					WithdrawReasons::all(),
					Default::default(),
				)
				.or(Err(Error::<T>::NotEnoughAssetsForFee))?;

				<T as Config>::Tokens::transfer(
					Self::native_token_id(),
					&account,
					&Self::treasury_account_id(),
					extra_fee,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			// burn tokes for user
			T::Tokens::burn_and_settle(
				asset_id,
				&account,
				amount.try_into().or(Err(Error::<T>::BalanceOverflow))?,
			)?;

			let l2_request_id = Self::acquire_l2_request_id(chain);

			let request_id = RequestId { origin: Origin::L2, id: l2_request_id };
			let withdrawal_update = Withdrawal {
				requestId: request_id.clone(),
				withdrawalRecipient: recipient.clone(),
				tokenAddress: token_address.clone(),
				amount: U256::from(amount),
				ferryTip: U256::from(ferry_tip.unwrap_or_default()),
			};
			// add cancel request to pending updates
			L2Requests::<T>::insert(
				chain,
				request_id.clone(),
				(
					L2Request::Withdrawal(withdrawal_update.clone()),
					withdrawal_update.abi_encode_hash(),
				),
			);

			Pallet::<T>::deposit_event(Event::WithdrawalRequestCreated {
				chain,
				request_id,
				recipient,
				token_address,
				amount,
				hash: withdrawal_update.abi_encode_hash(),
				ferry_tip: ferry_tip.unwrap_or_default(),
			});
			TotalNumberOfWithdrawals::<T>::mutate(|v| *v = v.saturating_add(One::one()));

			Ok(().into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn force_cancel_requests_from_l1(
			origin: OriginFor<T>,
			chain: T::ChainId,
			requests_to_cancel: u128,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::BlockedByMaintenanceMode
			);

			let (submitter, _request, _hash) =
				PendingSequencerUpdates::<T>::take(requests_to_cancel, chain)
					.ok_or(Error::<T>::RequestDoesNotExist)?;

			if T::SequencerStakingProvider::is_active_sequencer(chain, &submitter) {
				SequencersRights::<T>::mutate(chain, |sequencers| {
					if let Some(rights) = sequencers.get_mut(&submitter) {
						rights.read_rights = 1;
					}
				});
			}

			Ok(().into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn create_batch(
			origin: OriginFor<T>,
			chain: T::ChainId,
			sequencer_account: Option<T::AccountId>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::BlockedByMaintenanceMode
			);

			let asignee = Self::get_batch_asignee(chain, &sender, sequencer_account)?;
			let extra_fee = ManualBatchExtraFee::<T>::get();
			if !extra_fee.is_zero() {
				<T as Config>::Tokens::transfer(
					Self::native_token_id(),
					&sender,
					&Self::treasury_account_id(),
					extra_fee,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			let range = Self::get_batch_range_from_available_requests(chain)?;
			Self::persist_batch_and_deposit_event(chain, range, asignee);
			Ok(().into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn set_manual_batch_extra_fee(
			origin: OriginFor<T>,
			balance: BalanceOf<T>,
		) -> DispatchResult {
			let _ = ensure_root(origin)?;
			ManualBatchExtraFee::<T>::set(balance);
			Pallet::<T>::deposit_event(Event::ManualBatchExtraFeeSet(balance));
			Ok(().into())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		/// only deposit recipient can initiate refund failed deposit
		pub fn refund_failed_deposit(
			origin: OriginFor<T>,
			chain: T::ChainId,
			request_id: u128,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// NOTE: failed deposits are not reachable at this point
			let (author, deposit_hash) = FailedL1Deposits::<T>::take((chain, request_id))
				.ok_or(Error::<T>::FailedDepositDoesNotExist)?;

			let ferry = FerriedDeposits::<T>::get((chain, deposit_hash));
			let eligible_for_refund = ferry.clone().unwrap_or(author.clone());
			ensure!(eligible_for_refund == sender, Error::<T>::NotEligibleForRefund);

			let l2_request_id = Self::acquire_l2_request_id(chain);

			let failed_deposit_resolution = FailedDepositResolution {
				requestId: RequestId { origin: Origin::L2, id: l2_request_id },
				originRequestId: request_id,
				ferry: ferry.clone().map(T::AddressConverter::convert_back).unwrap_or([0u8; 20]),
			};

			L2Requests::<T>::insert(
				chain,
				RequestId::from((Origin::L2, l2_request_id)),
				(
					L2Request::FailedDepositResolution(failed_deposit_resolution),
					failed_deposit_resolution.abi_encode_hash(),
				),
			);

			Self::deposit_event(Event::DepositRefundCreated {
				refunded_request_id: RequestId { origin: Origin::L1, id: request_id },
				chain,
				ferry,
			});

			Ok(().into())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		/// Froce create batch and assigns it to provided sequencer
		/// provided requests range must exists - otherwise `[Error::InvalidRange]` error will be returned
		pub fn force_create_batch(
			origin: OriginFor<T>,
			chain: T::ChainId,
			range: (u128, u128),
			sequencer_account: AccountIdOf<T>,
		) -> DispatchResult {
			let _ = ensure_root(origin)?;

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::BlockedByMaintenanceMode
			);

			ensure!(
				L2Requests::<T>::contains_key(chain, RequestId { origin: Origin::L2, id: range.0 }),
				Error::<T>::InvalidRange
			);

			ensure!(
				L2Requests::<T>::contains_key(chain, RequestId { origin: Origin::L2, id: range.1 }),
				Error::<T>::InvalidRange
			);

			Self::persist_batch_and_deposit_event(chain, range, sequencer_account);
			Ok(().into())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn ferry_deposit(
			origin: OriginFor<T>,
			chain: T::ChainId,
			request_id: RequestId,
			deposit_recipient: [u8; 20],
			token_address: [u8; 20],
			amount: u128,
			timestamp: u128,
			ferry_tip: u128,
			deposit_hash: H256,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let deposit = messages::Deposit {
				depositRecipient: deposit_recipient,
				requestId: request_id,
				tokenAddress: token_address,
				amount: amount.into(),
				timeStamp: timestamp.into(),
				ferryTip: ferry_tip.into(),
			};

			ensure!(deposit.abi_encode_hash() == deposit_hash, Error::<T>::FerryHashMismatch);
			Self::ferry_desposit_impl(sender, chain, deposit)?;

			Ok(().into())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn ferry_deposit_unsafe(
			origin: OriginFor<T>,
			chain: T::ChainId,
			request_id: RequestId,
			deposit_recipient: [u8; 20],
			token_address: [u8; 20],
			amount: u128,
			timestamp: u128,
			ferry_tip: u128,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let deposit = messages::Deposit {
				depositRecipient: deposit_recipient,
				requestId: request_id,
				tokenAddress: token_address,
				amount: amount.into(),
				timeStamp: timestamp.into(),
				ferryTip: ferry_tip.into(),
			};

			Self::ferry_desposit_impl(sender, chain, deposit)?;

			Ok(().into())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(T::DbWeight::get().reads_writes(3, 3).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn update_l2_from_l1_unsafe(
			origin: OriginFor<T>,
			requests: messages::L1Update,
		) -> DispatchResult {
			let sequencer = ensure_signed(origin)?;
			Self::update_impl(sequencer, requests)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_dispute_period() -> u128 {
		T::DisputePeriodLength::get()
	}

	fn get_max_requests_per_block() -> u128 {
		T::RequestsPerBlock::get()
	}

	pub fn verify_sequencer_update(
		chain: T::ChainId,
		hash: H256,
		request_id: u128,
	) -> Option<bool> {
		let pending_requests_to_process = PendingSequencerUpdates::<T>::get(request_id, chain);
		if let Some((_, l1_update, _hash)) = pending_requests_to_process {
			let calculated_hash = Self::calculate_hash_of_sequencer_update(l1_update);
			Some(hash == calculated_hash)
		} else {
			None
		}
	}

	fn maybe_create_batch(now: BlockNumberFor<T>) {
		let batch_size = Self::automatic_batch_size();
		let batch_period: BlockNumberFor<T> = Self::automatic_batch_period().saturated_into();

		if T::MaintenanceStatusProvider::is_maintenance() {
			return
		}

		for (chain, next_id) in L2OriginRequestId::<T>::get().iter() {
			let last_id = next_id.saturating_sub(1);

			let (last_batch_block_number, last_batch_id, last_id_in_batch) =
				L2RequestsBatchLast::<T>::get()
					.get(&chain)
					.cloned()
					.map(|(block_number, batch_id, (_, last_reqeust_id))| {
						(block_number, batch_id, last_reqeust_id)
					})
					.unwrap_or_default();

			let trigger = if last_id >= last_id_in_batch + batch_size {
				Some(BatchSource::AutomaticSizeReached)
			} else if now >= last_batch_block_number + batch_period {
				Some(BatchSource::PeriodReached)
			} else {
				None
			};

			if let Some(trigger) = trigger {
				if let Some(updater) = T::SequencerStakingProvider::selected_sequencer(*chain) {
					let batch_id = last_batch_id.saturating_add(1);
					let range_start = last_id_in_batch.saturating_add(1);
					let range_end = sp_std::cmp::min(
						range_start.saturating_add(batch_size.saturating_sub(1)),
						last_id,
					);
					if range_end >= range_start {
						L2RequestsBatch::<T>::insert(
							(chain, batch_id),
							(now, (range_start, range_end), updater.clone()),
						);
						L2RequestsBatchLast::<T>::mutate(|batches| {
							batches
								.insert(chain.clone(), (now, batch_id, (range_start, range_end)));
						});
						Pallet::<T>::deposit_event(Event::TxBatchCreated {
							chain: *chain,
							source: trigger,
							assignee: updater,
							batch_id,
							range: (range_start, range_end),
						});
						break
					}
				} else {
					continue
				}
			}
		}
	}

	fn schedule_request_for_execution_if_dispute_period_has_passsed(now: BlockNumberFor<T>) {
		let block_number = <frame_system::Pallet<T>>::block_number().saturated_into::<u128>();

		for (l1, (sequencer, requests, l1_read_hash)) in
			PendingSequencerUpdates::<T>::iter_prefix(block_number)
		{
			if T::SequencerStakingProvider::is_active_sequencer(l1, &sequencer) {
				SequencersRights::<T>::mutate(l1, |sequencers| {
					if let Some(rights) = sequencers.get_mut(&sequencer) {
						rights.read_rights.saturating_accrue(T::RightsMultiplier::get());
					}
				});
			}

			let update_creation_block = block_number.saturating_sub(Self::get_dispute_period());
			match LastMaintananceMode::<T>::get() {
				Some(last_maintanance_mode) if update_creation_block < last_maintanance_mode => {
					Self::deposit_event(Event::L1ReadIgnoredBecauseOfMaintenanceMode {
						chain: l1,
						hash: l1_read_hash,
					});
				},
				_ => {
					Self::schedule_requests(now, l1, requests.clone());
					Self::deposit_event(Event::L1ReadScheduledForExecution {
						chain: l1,
						hash: l1_read_hash,
					});
				},
			}
		}

		let _ = PendingSequencerUpdates::<T>::clear_prefix(
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
			u32::MAX,
			None,
		);
	}

	fn process_single_request(l1: T::ChainId, request: messages::L1UpdateRequest) {
		if request.id() <= LastProcessedRequestOnL2::<T>::get(l1) {
			return
		}

		let status = match request.clone() {
			messages::L1UpdateRequest::Deposit(deposit) => {
				let deposit_status = Self::process_deposit(l1, &deposit);
				TotalNumberOfDeposits::<T>::mutate(|v| *v = v.saturating_add(One::one()));
				deposit_status.or_else(|err| {
					let who: T::AccountId = T::AddressConverter::convert(deposit.depositRecipient);
					FailedL1Deposits::<T>::insert(
						(l1, deposit.requestId.id),
						(who, deposit.abi_encode_hash()),
					);
					Err(err.into())
				})
			},
			messages::L1UpdateRequest::CancelResolution(cancel) =>
				Self::process_cancel_resolution(l1, &cancel).or_else(|err| {
					T::MaintenanceStatusProvider::trigger_maintanance_mode();
					Err(err)
				}),
		};

		Pallet::<T>::deposit_event(Event::RequestProcessedOnL2 {
			chain: l1,
			request_id: request.id(),
			status,
		});

		LastProcessedRequestOnL2::<T>::insert(l1, request.id());
	}

	fn execute_requests_from_execute_queue(now: BlockNumberFor<T>) {
		if T::MaintenanceStatusProvider::is_maintenance() &&
			UpdatesExecutionQueue::<T>::get(UpdatesExecutionQueueNextId::<T>::get()).is_some()
		{
			let new_id: u128 = LastScheduledUpdateIdInExecutionQueue::<T>::mutate(|v| {
				v.saturating_inc();
				*v
			});
			UpdatesExecutionQueueNextId::<T>::put(new_id);
			return
		}

		let mut limit = Self::get_max_requests_per_block();
		loop {
			if limit == 0 {
				return
			}
			if let Some((scheduled_at, l1, r)) =
				UpdatesExecutionQueue::<T>::get(UpdatesExecutionQueueNextId::<T>::get())
			{
				if scheduled_at == now {
					return
				}

				for req in r
					.into_requests()
					.into_iter()
					.filter(|request| request.id() > LastProcessedRequestOnL2::<T>::get(l1))
					.map(|val| Some(val))
					.chain(sp_std::iter::repeat(None))
					.take(limit.try_into().unwrap())
				{
					if let Some(request) = req {
						Self::process_single_request(l1, request);
						limit -= 1;
					} else {
						UpdatesExecutionQueue::<T>::remove(UpdatesExecutionQueueNextId::<T>::get());
						UpdatesExecutionQueueNextId::<T>::mutate(|v| *v += 1);
						break
					}
				}
			} else {
				if UpdatesExecutionQueue::<T>::contains_key(
					UpdatesExecutionQueueNextId::<T>::get() + 1,
				) {
					UpdatesExecutionQueueNextId::<T>::mutate(|v| *v += 1);
				} else {
					break
				}
			}
		}
	}

	fn schedule_requests(now: BlockNumberFor<T>, chain: T::ChainId, update: messages::L1Update) {
		let max_id = [
			update.pendingDeposits.iter().map(|r| r.requestId.id).max(),
			update.pendingCancelResolutions.iter().map(|r| r.requestId.id).max(),
		]
		.iter()
		.filter_map(|elem| elem.clone())
		.max();

		if let Some(max_id) = max_id {
			MaxAcceptedRequestIdOnl2::<T>::mutate(chain, |cnt| {
				*cnt = sp_std::cmp::max(*cnt, max_id)
			});
		}

		let id = LastScheduledUpdateIdInExecutionQueue::<T>::mutate(|id| {
			id.saturating_inc();
			*id
		});
		UpdatesExecutionQueue::<T>::insert(id, (now, chain, update));
	}

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// NOTE: This function is not transactional, so even if it fails at some point that DOES NOT
	/// REVERT PREVIOUS CHANGES TO STORAGE, whoever is modifying it should take that into account!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	fn process_deposit(
		l1: T::ChainId,
		deposit: &messages::Deposit,
	) -> Result<(), L1DepositProcessingError> {
		let amount = TryInto::<u128>::try_into(deposit.amount)
			.map_err(|_| L1DepositProcessingError::Overflow)?
			.try_into()
			.map_err(|_| L1DepositProcessingError::Overflow)?;

		let eth_asset = T::AssetAddressConverter::convert((l1, deposit.tokenAddress));

		let asset_id = match T::AssetRegistryProvider::get_l1_asset_id(eth_asset.clone()) {
			Some(id) => id,
			None => T::AssetRegistryProvider::create_l1_asset(eth_asset)
				.map_err(|_| L1DepositProcessingError::AssetRegistrationProblem)?,
		};

		let account = FerriedDeposits::<T>::get((l1, deposit.abi_encode_hash()))
			.unwrap_or(T::AddressConverter::convert(deposit.depositRecipient));

		T::Tokens::mint(asset_id, &account, amount)
			.map_err(|_| L1DepositProcessingError::MintError)?;

		Ok(())
	}

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// NOTE: This function is not transactional, so even if it fails at some point that DOES NOT
	/// REVERT PREVIOUS CHANGES TO STORAGE, whoever is modifying it should take that into account!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	fn process_cancel_resolution(
		l1: T::ChainId,
		cancel_resolution: &messages::CancelResolution,
	) -> Result<(), L1RequestProcessingError> {
		let cancel_request_id = cancel_resolution.l2RequestId;
		let cancel_justified = cancel_resolution.cancelJustified;

		let cancel_update =
			match L2Requests::<T>::get(l1, RequestId::from((Origin::L2, cancel_request_id))) {
				Some((L2Request::Cancel(cancel), _)) => Some(cancel),
				_ => None,
			}
			.ok_or(L1RequestProcessingError::WrongCancelRequestId)?;

		let updater = cancel_update.updater;
		let canceler = cancel_update.canceler;
		let (to_be_slashed, to_reward) = if cancel_justified {
			(updater.clone(), Some(canceler.clone()))
		} else {
			(canceler.clone(), None)
		};

		if T::SequencerStakingProvider::is_active_sequencer(l1, &updater) {
			SequencersRights::<T>::mutate(l1, |sequencers| {
				if let Some(rights) = sequencers.get_mut(&updater) {
					rights.read_rights.saturating_inc();
				}
			});
		}
		if T::SequencerStakingProvider::is_active_sequencer(l1, &canceler) {
			SequencersRights::<T>::mutate(l1, |sequencers| {
				if let Some(rights) = sequencers.get_mut(&canceler) {
					rights.cancel_rights.saturating_inc();
				}
			});
		}

		AwaitingCancelResolution::<T>::mutate(l1, |v| {
			v.remove(&(updater, cancel_request_id, DisputeRole::Submitter))
		});
		AwaitingCancelResolution::<T>::mutate(l1, |v| {
			v.remove(&(canceler, cancel_request_id, DisputeRole::Canceler))
		});

		// slash is after adding rights, since slash can reduce stake below required level and remove all rights
		T::SequencerStakingProvider::slash_sequencer(l1, &to_be_slashed, to_reward.as_ref())
			.map_err(|_| L1RequestProcessingError::SequencerNotSlashed)?;

		Ok(())
	}

	fn calculate_hash_of_sequencer_update(update: messages::L1Update) -> H256 {
		let update: messages::eth_abi::L1Update = update.into();
		let hash: [u8; 32] = keccak_256(&update.abi_encode()[..]).into();
		H256::from(hash)
	}

	fn handle_sequencer_deactivation(
		chain: T::ChainId,
		deactivated_sequencers: BTreeSet<T::AccountId>,
	) {
		SequencersRights::<T>::mutate(chain, |sequencers_set| {
			let mut removed: usize = 0;
			for seq in deactivated_sequencers.iter() {
				if sequencers_set.remove(seq).is_some() {
					removed.saturating_inc();
				}
			}

			for (_, rights) in sequencers_set.iter_mut() {
				rights.cancel_rights = rights
					.cancel_rights
					.saturating_sub(T::RightsMultiplier::get().saturating_mul(removed as u128));
			}
		});
	}

	pub fn convert_eth_l1update_to_substrate_l1update(
		payload: Vec<u8>,
	) -> Result<L1Update, String> {
		messages::eth_abi::L1Update::abi_decode(payload.as_ref(), true)
			.map_err(|err| format!("Failed to decode L1Update: {}", err))
			.and_then(|update| {
				update.try_into().map_err(|err| format!("Failed to convert L1Update: {}", err))
			})
	}

	pub fn validate_l1_update(l1: T::ChainId, update: &messages::L1Update) -> DispatchResult {
		ensure!(
			!update.pendingDeposits.is_empty() || !update.pendingCancelResolutions.is_empty(),
			Error::<T>::EmptyUpdate
		);

		ensure!(
			update
				.pendingDeposits
				.iter()
				.map(|v| v.requestId.origin)
				.all(|v| v == Origin::L1),
			Error::<T>::InvalidUpdate
		);
		ensure!(
			update
				.pendingCancelResolutions
				.iter()
				.map(|v| v.requestId.origin)
				.all(|v| v == Origin::L1),
			Error::<T>::InvalidUpdate
		);

		// check that consecutive id
		ensure!(
			update
				.pendingDeposits
				.iter()
				.map(|v| v.requestId.id)
				.into_iter()
				.tuple_windows()
				.all(|(a, b)| a < b),
			Error::<T>::InvalidUpdate
		);

		ensure!(
			update
				.pendingCancelResolutions
				.iter()
				.map(|v| v.requestId.id)
				.into_iter()
				.tuple_windows()
				.all(|(a, b)| a < b),
			Error::<T>::InvalidUpdate
		);

		let lowest_id = [
			update.pendingDeposits.first().map(|v| v.requestId.id),
			update.pendingCancelResolutions.first().map(|v| v.requestId.id),
		]
		.iter()
		.filter_map(|v| v.clone())
		.into_iter()
		.min()
		.ok_or(Error::<T>::InvalidUpdate)?;

		ensure!(lowest_id > 0u128, Error::<T>::WrongRequestId);

		ensure!(
			lowest_id <= LastProcessedRequestOnL2::<T>::get(l1) + 1,
			Error::<T>::WrongRequestId
		);

		let last_id = lowest_id +
			(update.pendingDeposits.len() as u128) +
			(update.pendingCancelResolutions.len() as u128);

		ensure!(last_id > LastProcessedRequestOnL2::<T>::get(l1), Error::<T>::WrongRequestId);

		let mut deposit_it = update.pendingDeposits.iter();
		let mut cancel_it = update.pendingCancelResolutions.iter();

		let mut deposit = deposit_it.next();
		let mut cancel = cancel_it.next();

		for id in (lowest_id..last_id).into_iter() {
			match (deposit, cancel) {
				(Some(d), _) if d.requestId.id == id => {
					deposit = deposit_it.next();
				},
				(_, Some(c)) if c.requestId.id == id => {
					cancel = cancel_it.next();
				},
				_ => return Err(Error::<T>::InvalidUpdate.into()),
			}
		}

		Ok(().into())
	}

	pub fn update_impl(sequencer: T::AccountId, read: messages::L1Update) -> DispatchResult {
		// let l1 = read.chain;
		let l1 = read.chain.into();
		ensure!(
			!T::MaintenanceStatusProvider::is_maintenance(),
			Error::<T>::BlockedByMaintenanceMode
		);

		ensure!(
			T::SequencerStakingProvider::is_selected_sequencer(l1, &sequencer),
			Error::<T>::OnlySelectedSequencerisAllowedToUpdate
		);
		Self::validate_l1_update(l1, &read)?;

		// check json length to prevent big data spam, maybe not necessary as it will be checked later and slashed
		let current_block_number =
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>();
		let dispute_period_length = Self::get_dispute_period();
		let dispute_period_end: u128 = current_block_number + dispute_period_length;

		// ensure sequencer has rights to update
		if let Some(rights) = SequencersRights::<T>::get(&l1).get(&sequencer) {
			if rights.read_rights == 0u128 {
				log!(debug, "{:?} does not have sufficient read_rights", sequencer);
				return Err(Error::<T>::OperationFailed.into())
			}
		} else {
			log!(debug, "{:?} not a sequencer, CHEEKY BASTARD!", sequencer);
			return Err(Error::<T>::OperationFailed.into())
		}

		// // Decrease read_rights by 1
		SequencersRights::<T>::mutate(l1, |sequencers_set| {
			if let Some(rights) = sequencers_set.get_mut(&sequencer) {
				rights.read_rights -= 1;
			}
		});

		ensure!(
			!PendingSequencerUpdates::<T>::contains_key(dispute_period_end, l1),
			Error::<T>::MultipleUpdatesInSingleBlock
		);

		let update: messages::eth_abi::L1Update = read.clone().into();
		let request_hash = keccak_256(&update.abi_encode());
		let l1_read_hash = H256::from_slice(request_hash.as_slice());

		PendingSequencerUpdates::<T>::insert(
			dispute_period_end,
			l1,
			(sequencer.clone(), read.clone(), l1_read_hash),
		);

		LastUpdateBySequencer::<T>::insert((l1, &sequencer), current_block_number);

		let requests_range = read.range().ok_or(Error::<T>::InvalidUpdate)?;

		Pallet::<T>::deposit_event(Event::L1ReadStored {
			chain: l1,
			sequencer: sequencer.clone(),
			dispute_period_end,
			range: requests_range,
			hash: l1_read_hash,
		});

		// 2 storage reads & writes in seqs pallet
		T::SequencerStakingRewards::note_update_author(&sequencer);

		Ok(().into())
	}

	fn count_of_read_rights_under_dispute(chain: ChainIdOf<T>, sequencer: &AccountIdOf<T>) -> u128 {
		let mut read_rights = 0u128;
		let last_update = LastUpdateBySequencer::<T>::get((chain, sequencer));

		if last_update != 0 &&
			last_update.saturating_add(T::DisputePeriodLength::get()) >=
				<frame_system::Pallet<T>>::block_number().saturated_into::<u128>()
		{
			read_rights += 1;
		}

		read_rights.saturating_accrue(
			AwaitingCancelResolution::<T>::get(chain)
				.iter()
				.filter(|(acc, _, role)| acc == sequencer && *role == DisputeRole::Submitter)
				.count() as u128,
		);

		read_rights
	}

	fn count_of_cancel_rights_under_dispute(
		chain: ChainIdOf<T>,
		sequencer: &AccountIdOf<T>,
	) -> usize {
		AwaitingCancelResolution::<T>::get(chain)
			.iter()
			.filter(|(acc, _, role)| acc == sequencer && *role == DisputeRole::Canceler)
			.count()
	}

	pub fn create_merkle_tree(
		chain: ChainIdOf<T>,
		range: (u128, u128),
	) -> Option<MerkleTree<Keccak256Hasher>> {
		let l2_requests = (range.0..=range.1)
			.into_iter()
			.map(|id| match L2Requests::<T>::get(chain, RequestId { origin: Origin::L2, id }) {
				Some((_, hash)) => Some(hash.into()),
				None => None,
			})
			.collect::<Option<Vec<_>>>();

		l2_requests.map(|txs| MerkleTree::<Keccak256Hasher>::from_leaves(txs.as_ref()))
	}

	pub fn get_merkle_root(chain: ChainIdOf<T>, range: (u128, u128)) -> H256 {
		if let Some(tree) = Self::create_merkle_tree(chain, range) {
			H256::from(tree.root().unwrap_or_default())
		} else {
			H256::from([0u8; 32])
		}
	}

	pub fn get_merkle_proof_for_tx(
		chain: ChainIdOf<T>,
		range: (u128, u128),
		tx_id: u128,
	) -> Vec<H256> {
		if tx_id < range.0 || tx_id > range.1 {
			return Default::default()
		}

		let tree = Self::create_merkle_tree(chain, range);
		if let Some(merkle_tree) = tree {
			let idx = tx_id as usize - range.0 as usize;
			let indices_to_prove = vec![idx];
			let merkle_proof = merkle_tree.proof(&indices_to_prove);
			merkle_proof.proof_hashes().iter().map(|hash| H256::from(hash)).collect()
		} else {
			Default::default()
		}
	}

	pub fn max_id(chain: ChainIdOf<T>, range: (u128, u128)) -> u128 {
		let mut max_id = 0u128;
		for id in range.0..=range.1 {
			if let Some((L2Request::Withdrawal(withdrawal), _)) =
				L2Requests::<T>::get(chain, RequestId { origin: Origin::L2, id })
			{
				if withdrawal.requestId.id > max_id {
					max_id = withdrawal.requestId.id;
				}
			}
		}
		max_id
	}

	pub(crate) fn automatic_batch_size() -> u128 {
		<T as Config>::MerkleRootAutomaticBatchSize::get()
	}

	pub(crate) fn automatic_batch_period() -> u128 {
		<T as Config>::MerkleRootAutomaticBatchPeriod::get()
	}

	fn acquire_l2_request_id(chain: ChainIdOf<T>) -> u128 {
		L2OriginRequestId::<T>::mutate(|val| {
			// request ids start from id == 1
			val.entry(chain).or_insert(1u128);
			let id = val[&chain];
			val.entry(chain).and_modify(|v| v.saturating_inc());
			id
		})
	}

	#[cfg(test)]
	fn get_next_l2_request_id(chain: ChainIdOf<T>) -> u128 {
		L2OriginRequestId::<T>::get().get(&chain).cloned().unwrap_or(1u128)
	}

	fn get_latest_l2_request_id(chain: ChainIdOf<T>) -> Option<u128> {
		L2OriginRequestId::<T>::get().get(&chain).cloned().map(|v| v.saturating_sub(1))
	}

	pub fn verify_merkle_proof_for_tx(
		chain: ChainIdOf<T>,
		range: (u128, u128),
		root_hash: H256,
		tx_id: u128,
		proof: Vec<H256>,
	) -> bool {
		let proof =
			MerkleProof::<Keccak256Hasher>::new(proof.into_iter().map(Into::into).collect());

		let inclusive_range = range.0..=range.1;
		if !inclusive_range.contains(&tx_id) {
			return false
		}

		let pos = inclusive_range.clone().position(|elem| elem == tx_id);
		let request = L2Requests::<T>::get(chain, RequestId { origin: Origin::L2, id: tx_id });
		if let (Some((req, _)), Some(pos)) = (request, pos) {
			proof.verify(
				root_hash.into(),
				&[pos],
				&[req.abi_encode_hash().into()],
				inclusive_range.count(),
			)
		} else {
			false
		}
	}

	pub(crate) fn treasury_account_id() -> T::AccountId {
		T::TreasuryPalletId::get().into_account_truncating()
	}

	fn native_token_id() -> CurrencyIdOf<T> {
		<T as Config>::NativeCurrencyId::get()
	}

	pub fn get_abi_encoded_l2_request(chain: ChainIdOf<T>, request_id: u128) -> Vec<u8> {
		L2Requests::<T>::get(chain, RequestId::from((Origin::L2, request_id)))
			.map(|(req, _hash)| req.abi_encode())
			.unwrap_or_default()
	}

	fn get_batch_range_from_available_requests(
		chain: ChainIdOf<T>,
	) -> Result<(u128, u128), Error<T>> {
		let last_request_id = L2RequestsBatchLast::<T>::get()
			.get(&chain)
			.cloned()
			.map(|(_block_number, _batch_id, range)| range.1)
			.unwrap_or_default();
		let range_start = last_request_id.saturating_add(1u128);
		let range_end = Self::get_latest_l2_request_id(chain).ok_or(Error::<T>::EmptyBatch)?;

		if L2Requests::<T>::contains_key(chain, RequestId { origin: Origin::L2, id: range_start }) {
			Ok((range_start, range_end))
		} else {
			Err(Error::<T>::EmptyBatch)
		}
	}

	fn get_next_batch_id(chain: ChainIdOf<T>) -> u128 {
		let last_batch_id = L2RequestsBatchLast::<T>::get()
			.get(&chain)
			.cloned()
			.map(|(_block_number, batch_id, _range)| batch_id)
			.unwrap_or_default();
		last_batch_id.saturating_add(1u128)
	}

	fn persist_batch_and_deposit_event(
		chain: ChainIdOf<T>,
		range: (u128, u128),
		asignee: T::AccountId,
	) {
		let now: BlockNumberFor<T> = <frame_system::Pallet<T>>::block_number();
		let batch_id = Self::get_next_batch_id(chain);

		L2RequestsBatch::<T>::insert((chain, batch_id), (now, (range.0, range.1), asignee.clone()));

		L2RequestsBatchLast::<T>::mutate(|batches| {
			batches.insert(chain.clone(), (now, batch_id, (range.0, range.1)));
		});

		Pallet::<T>::deposit_event(Event::TxBatchCreated {
			chain,
			source: BatchSource::Manual,
			assignee: asignee.clone(),
			batch_id,
			range: (range.0, range.1),
		});
	}

	/// Deduces account that batch should be assigened to
	fn get_batch_asignee(
		chain: ChainIdOf<T>,
		sender: &T::AccountId,
		sequencer_account: Option<T::AccountId>,
	) -> Result<T::AccountId, Error<T>> {
		if let Some(sequencer) = sequencer_account {
			if T::SequencerStakingProvider::is_active_sequencer_alias(chain, &sequencer, sender) {
				Ok(sequencer)
			} else {
				Err(Error::<T>::UnknownAliasAccount)
			}
		} else {
			Ok(sender.clone())
		}
	}

	fn ferry_desposit_impl(
		sender: T::AccountId,
		chain: T::ChainId,
		deposit: messages::Deposit,
	) -> Result<(), Error<T>> {
		let deposit_hash = deposit.abi_encode_hash();

		let amount = deposit
			.amount
			.checked_sub(deposit.ferryTip)
			.and_then(|v| TryInto::<u128>::try_into(v).ok())
			.and_then(|v| TryInto::<BalanceOf<T>>::try_into(v).ok())
			.ok_or(Error::<T>::MathOverflow)?;

		let eth_asset = T::AssetAddressConverter::convert((chain, deposit.tokenAddress));
		let asset_id = match T::AssetRegistryProvider::get_l1_asset_id(eth_asset.clone()) {
			Some(id) => id,
			None => T::AssetRegistryProvider::create_l1_asset(eth_asset)
				.map_err(|_| Error::<T>::AssetRegistrationProblem)?,
		};

		let account = T::AddressConverter::convert(deposit.depositRecipient);

		T::Tokens::transfer(asset_id, &sender, &account, amount, ExistenceRequirement::KeepAlive)
			.map_err(|_| Error::<T>::NotEnoughAssets)?;
		FerriedDeposits::<T>::insert((chain, deposit_hash), sender);

		Self::deposit_event(Event::DepositFerried { chain, deposit, deposit_hash });

		Ok(().into())
	}
}

impl<T: Config> RolldownProviderTrait<ChainIdOf<T>, AccountIdOf<T>> for Pallet<T> {
	fn new_sequencer_active(chain: T::ChainId, sequencer: &AccountIdOf<T>) {
		SequencersRights::<T>::mutate(chain, |sequencer_set| {
			let count = sequencer_set.len() as u128;

			sequencer_set.insert(
				sequencer.clone(),
				SequencerRights {
					read_rights: T::RightsMultiplier::get().saturating_sub(
						Pallet::<T>::count_of_read_rights_under_dispute(chain, sequencer),
					),
					cancel_rights: count.saturating_mul(T::RightsMultiplier::get()).saturating_sub(
						Pallet::<T>::count_of_cancel_rights_under_dispute(chain, sequencer) as u128,
					),
				},
			);

			let sequencer_count = (sequencer_set.len() as u128).saturating_sub(1u128);

			for (s, rights) in sequencer_set.iter_mut().filter(|(s, _)| *s != sequencer) {
				rights.cancel_rights =
					T::RightsMultiplier::get().saturating_mul(sequencer_count).saturating_sub(
						Pallet::<T>::count_of_cancel_rights_under_dispute(chain, s) as u128,
					)
			}
		});
	}

	fn sequencer_unstaking(chain: T::ChainId, sequencer: &AccountIdOf<T>) -> DispatchResult {
		ensure!(
			Pallet::<T>::count_of_read_rights_under_dispute(chain, sequencer).is_zero(),
			Error::<T>::SequencerLastUpdateStillInDisputePeriod
		);

		ensure!(
			Pallet::<T>::count_of_cancel_rights_under_dispute(chain, sequencer).is_zero(),
			Error::<T>::SequencerAwaitingCancelResolution
		);

		LastUpdateBySequencer::<T>::remove((chain, &sequencer));

		Ok(())
	}

	fn handle_sequencer_deactivations(
		chain: T::ChainId,
		deactivated_sequencers: Vec<T::AccountId>,
	) {
		Pallet::<T>::handle_sequencer_deactivation(
			chain,
			deactivated_sequencers.into_iter().collect(),
		);
	}
}

pub struct MultiEvmChainAddressConverter;
impl Convert<(messages::Chain, [u8; 20]), L1Asset> for MultiEvmChainAddressConverter {
	fn convert((chain, address): (messages::Chain, [u8; 20])) -> L1Asset {
		match chain {
			messages::Chain::Ethereum => L1Asset::Ethereum(address),
			messages::Chain::Arbitrum => L1Asset::Arbitrum(address),
		}
	}
}
