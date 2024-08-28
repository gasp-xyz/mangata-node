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
	SequencerStakingProviderTrait, SetMaintenanceModeOn,
};
use mangata_types::assets::L1Asset;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use sha3::{Digest, Keccak256};
use sp_core::{H256, U256};
use sp_runtime::traits::{AccountIdConversion, Convert, Zero};
use sp_std::{collections::btree_set::BTreeSet, convert::TryInto, prelude::*, vec::Vec};

pub type CurrencyIdOf<T> = <<T as Config>::Tokens as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

pub type BalanceOf<T> =
	<<T as Config>::Tokens as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance;
pub type ChainIdOf<T> = <T as pallet::Config>::ChainId;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

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

#[derive(Clone)]
pub struct Keccak256Hasher {}

impl Hasher for Keccak256Hasher {
	type Hash = [u8; 32];

	fn hash(data: &[u8]) -> [u8; 32] {
		let mut output = [0u8; 32];
		let hash = Keccak256::digest(&data[..]);
		output.copy_from_slice(&hash[..]);
		output
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
			let batch_size = Self::automatic_batch_size();
			let batch_period: BlockNumberFor<T> = Self::automatic_batch_period().saturated_into();

			if T::MaintenanceStatusProvider::is_maintenance() {
				return T::DbWeight::get().reads_writes(10, 20);
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
								batches.insert(
									chain.clone(),
									(now, batch_id, (range_start, range_end)),
								);
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

			Self::end_dispute_period();
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
	/// stores id of the failed depoisit, so it can be  refunded using [`Pallet::refund_failed_deposit`]
	pub type FailedL1Deposits<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::ChainId, u128), (), OptionQuery>;

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
		(T::AccountId, messages::L1Update),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	// queue of all updates that went through dispute period and are ready to be processed
	pub type UpdatesExecutionQueue<T: Config> =
		StorageMap<_, Blake2_128Concat, u128, (T::ChainId, messages::L1Update), OptionQuery>;

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
		(T::ChainId, T::AccountId),
		BTreeSet<(u128, DisputeRole)>,
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
	pub type TotalNumberOfDeposits<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_number_of_withdrawals)]
	pub type TotalNumberOfWithdrawals<T: Config> = StorageValue<_, u32, ValueQuery>;

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
		},
		ManualBatchExtraFeeSet(BalanceOf<T>),
		DepositRefundCreated {
			chain: ChainIdOf<T>,
			refunded_request_id: RequestId,
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
		FailedDepositDoesExists,
		EmptyBatch,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type SequencerStakingProvider: SequencerStakingProviderTrait<
			Self::AccountId,
			BalanceOf<Self>,
			ChainIdOf<Self>,
		>;
		type AddressConverter: Convert<[u8; 20], Self::AccountId>;
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
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn update_l2_from_l1(
			origin: OriginFor<T>,
			requests: messages::L1Update,
		) -> DispatchResult {
			let sequencer = ensure_signed(origin)?;
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
			Self::schedule_requests(chain, update.into());
			Self::process_requests();
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

			let (submitter, request) =
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

			AwaitingCancelResolution::<T>::mutate((chain, submitter), |v| {
				v.insert((l2_request_id, DisputeRole::Submitter))
			});
			AwaitingCancelResolution::<T>::mutate((chain, canceler), |v| {
				v.insert((l2_request_id, DisputeRole::Canceler))
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
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::BlockedByMaintenanceMode
			);

			let eth_asset = T::AssetAddressConverter::convert((chain, token_address));
			let asset_id = T::AssetRegistryProvider::get_l1_asset_id(eth_asset)
				.ok_or(Error::<T>::MathOverflow)?;

			// fail will occur if user has not enough balance
			<T as Config>::Tokens::ensure_can_withdraw(
				asset_id.into(),
				&account,
				amount.try_into().or(Err(Error::<T>::BalanceOverflow))?,
				WithdrawReasons::all(),
				Default::default(),
			)
			.or(Err(Error::<T>::NotEnoughAssets))?;

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

			let (submitter, _request) =
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
			let now: BlockNumberFor<T> = <frame_system::Pallet<T>>::block_number();

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::BlockedByMaintenanceMode
			);


			let asignee = if let Some(sequencer) = sequencer_account {
				ensure!(
					T::SequencerStakingProvider::is_active_sequencer_alias(
						chain, &sequencer, &sender
					),
					Error::<T>::UnknownAliasAccount
				);
				sequencer
			} else {
				sender.clone()
			};

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

			let (last_batch_id, last_request_id) = L2RequestsBatchLast::<T>::get()
				.get(&chain)
				.cloned()
				.map(|(_block_number, batch_id, range)| (batch_id, range.1))
				.unwrap_or_default();

			let batch_id = last_batch_id.saturating_add(1u128);
			let range_start = last_request_id.saturating_add(1u128);

			ensure!(
				L2Requests::<T>::contains_key(chain, RequestId{origin: Origin::L2, id: range_start} ),
				Error::<T>::EmptyBatch
			);
			let range_end = Self::get_latest_l2_request_id(chain).ok_or(Error::<T>::EmptyBatch)?;
			ensure!(range_end >= range_start, Error::<T>::InvalidRange);

			L2RequestsBatch::<T>::insert((chain, batch_id), (now, (range_start, range_end), asignee.clone()));
			L2RequestsBatchLast::<T>::mutate(|batches| {
				batches.insert(chain.clone(), (now, batch_id, (range_start, range_end)));
			});

			Pallet::<T>::deposit_event(Event::TxBatchCreated {
				chain,
				source: BatchSource::Manual,
				assignee: asignee.clone(),
				batch_id,
				range: (range_start, range_end),
			});

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
		pub fn refund_failed_deposit(
			origin: OriginFor<T>,
			chain: T::ChainId,
			request_id: u128,
		) -> DispatchResult {
			let _ = ensure_root(origin)?;

			// NOTE: failed deposits are not reachable at this point
			let _ = FailedL1Deposits::<T>::take((chain, request_id))
				.ok_or(Error::<T>::FailedDepositDoesExists)?;

			let l2_request_id = Self::acquire_l2_request_id(chain);

			let failed_deposit_resolution = FailedDepositResolution {
				requestId: RequestId { origin: Origin::L2, id: l2_request_id },
				originRequestId: request_id,
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
			});

			Ok(().into())
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
		if let Some((_, l1_update)) = pending_requests_to_process {
			let calculated_hash = Self::calculate_hash_of_sequencer_update(l1_update);
			Some(hash == calculated_hash)
		} else {
			None
		}
	}

	// should run each block, check if dispute period ended, if yes, process pending requests
	fn end_dispute_period() {
		let block_number = <frame_system::Pallet<T>>::block_number().saturated_into::<u128>();

		for (l1, pending_requests_to_process) in
			PendingSequencerUpdates::<T>::iter_prefix(block_number)
		{
			log!(debug, "dispute end {:?}", block_number);

			let sequencer = &pending_requests_to_process.0;
			let requests = pending_requests_to_process.1.clone();

			if T::SequencerStakingProvider::is_active_sequencer(l1, sequencer) {
				SequencersRights::<T>::mutate(l1, |sequencers| {
					if let Some(rights) = sequencers.get_mut(sequencer) {
						rights.read_rights.saturating_accrue(T::RightsMultiplier::get());
					}
				});
			}

			Self::schedule_requests(l1, requests.clone());
		}

		let _ = PendingSequencerUpdates::<T>::clear_prefix(
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
			u32::MAX,
			None,
		);
		Self::process_requests();
	}

	fn process_single_request(l1: T::ChainId, request: messages::L1UpdateRequest) {
		if request.id() <= LastProcessedRequestOnL2::<T>::get(l1) {
			return
		}

		let status = match request.clone() {
			messages::L1UpdateRequest::Deposit(deposit) => Self::process_deposit(l1, &deposit)
				.or_else(|err| {
					FailedL1Deposits::<T>::insert((l1, deposit.requestId.id), ());
					Err(err)
				}),
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

	fn process_requests() {
		let mut limit = Self::get_max_requests_per_block();

		loop {
			if limit == 0 {
				return
			}
			if let Some((l1, r)) =
				UpdatesExecutionQueue::<T>::get(UpdatesExecutionQueueNextId::<T>::get())
			{
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

	fn schedule_requests(chain: T::ChainId, update: messages::L1Update) {
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

		let id = LastScheduledUpdateIdInExecutionQueue::<T>::get();
		LastScheduledUpdateIdInExecutionQueue::<T>::put(id + 1);
		UpdatesExecutionQueue::<T>::insert(id + 1, (chain, update));
	}

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// NOTE: This function is not transactional, so even if it fails at some point that DOES NOT
	/// REVERT PREVIOUS CHANGES TO STORAGE, whoever is modifying it should take that into account!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	fn process_deposit(
		l1: T::ChainId,
		deposit_request_details: &messages::Deposit,
	) -> Result<(), L1RequestProcessingError> {
		let account: T::AccountId =
			T::AddressConverter::convert(deposit_request_details.depositRecipient);

		let amount = TryInto::<u128>::try_into(deposit_request_details.amount)
			.map_err(|_| L1RequestProcessingError::Overflow)?
			.try_into()
			.map_err(|_| L1RequestProcessingError::Overflow)?;

		let eth_asset =
			T::AssetAddressConverter::convert((l1, deposit_request_details.tokenAddress));

		let asset_id = match T::AssetRegistryProvider::get_l1_asset_id(eth_asset.clone()) {
			Some(id) => id,
			None => T::AssetRegistryProvider::create_l1_asset(eth_asset)
				.map_err(|_| L1RequestProcessingError::AssetRegistrationProblem)?,
		};

		T::Tokens::mint(asset_id, &account, amount)
			.map_err(|_| L1RequestProcessingError::MintError)?;

		TotalNumberOfDeposits::<T>::mutate(|v| *v = v.saturating_add(One::one()));
		log!(debug, "Deposit processed successfully: {:?}", deposit_request_details);

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

		AwaitingCancelResolution::<T>::mutate((l1, &updater), |v| {
			v.remove(&(cancel_request_id, DisputeRole::Submitter))
		});
		AwaitingCancelResolution::<T>::mutate((l1, &canceler), |v| {
			v.remove(&(cancel_request_id, DisputeRole::Canceler))
		});

		// slash is after adding rights, since slash can reduce stake below required level and remove all rights
		T::SequencerStakingProvider::slash_sequencer(l1, &to_be_slashed, to_reward.as_ref())
			.map_err(|_| L1RequestProcessingError::SequencerNotSlashed)?;

		Ok(())
	}

	fn calculate_hash_of_sequencer_update(update: messages::L1Update) -> H256 {
		let update: messages::eth_abi::L1Update = update.into();
		let hash: [u8; 32] = Keccak256::digest(&update.abi_encode()[..]).into();
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

		// insert pending_requests
		PendingSequencerUpdates::<T>::insert(
			dispute_period_end,
			l1,
			(sequencer.clone(), read.clone()),
		);

		let update: messages::eth_abi::L1Update = read.clone().into();
		let request_hash = Keccak256::digest(&update.abi_encode());

		LastUpdateBySequencer::<T>::insert((l1, &sequencer), current_block_number);

		let requests_range = read.range().ok_or(Error::<T>::InvalidUpdate)?;

		Pallet::<T>::deposit_event(Event::L1ReadStored {
			chain: l1,
			sequencer,
			dispute_period_end,
			range: requests_range,
			hash: H256::from_slice(request_hash.as_slice()),
		});

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
			AwaitingCancelResolution::<T>::get((chain, sequencer))
				.iter()
				.filter(|(_, role)| *role == DisputeRole::Submitter)
				.count() as u128,
		);

		read_rights
	}

	fn count_of_cancel_rights_under_dispute(
		chain: ChainIdOf<T>,
		sequencer: &AccountIdOf<T>,
	) -> usize {
		AwaitingCancelResolution::<T>::get((chain, sequencer))
			.iter()
			.filter(|(_, role)| *role == DisputeRole::Canceler)
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

	pub(crate) fn get_next_l2_request_id(chain: ChainIdOf<T>) -> u128 {
		L2OriginRequestId::<T>::get().get(&chain).cloned().unwrap_or(1u128)
	}

	fn get_latest_l2_request_id(chain: ChainIdOf<T>) -> Option<u128> {
		L2OriginRequestId::<T>::get()
			.get(&chain)
			.cloned()
			.map(|v| v.saturating_sub(1))
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

	fn treasury_account_id() -> T::AccountId {
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

			for (_, rights) in sequencer_set.iter_mut().filter(|(s, _)| *s != sequencer) {
				rights.cancel_rights.saturating_accrue(T::RightsMultiplier::get())
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
