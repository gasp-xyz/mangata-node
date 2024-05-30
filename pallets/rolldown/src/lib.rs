#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, Get, StorageVersion},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use messages::{to_eth_u256, Origin, RequestId, UpdateType};
use scale_info::prelude::{format, string::String};
use sp_runtime::traits::{One, SaturatedConversion, Saturating};
use sp_std::{collections::btree_map::BTreeMap, iter::Iterator};

use alloy_sol_types::SolValue;
use frame_support::traits::WithdrawReasons;
use itertools::Itertools;
use mangata_support::traits::{
	AssetRegistryProviderTrait, GetMaintenanceStatusTrait, RolldownProviderTrait,
	SequencerStakingProviderTrait,
};
use mangata_types::assets::L1Asset;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use sha3::{Digest, Keccak256};
use sp_core::{H256, U256};
use sp_runtime::{
	serde::Serialize,
	traits::{Convert, MaybeConvert},
};
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

pub mod messages;

use crate::messages::L1Update;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use crate::messages::UpdateType;

	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
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

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
	pub struct Cancel<AccountId> {
		pub requestId: RequestId,
		pub updater: AccountId,
		pub canceler: AccountId,
		pub range: messages::Range,
		pub hash: H256,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo)]
	pub struct RequestResult {
		pub requestId: RequestId,
		pub originRequestId: u128,
		pub status: bool,
		pub updateType: UpdateType,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Default)]
	pub struct Withdrawal {
		pub requestId: RequestId,
		pub withdrawalRecipient: [u8; 20],
		pub tokenAddress: [u8; 20],
		pub amount: U256,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo)]
	pub enum L2Request<AccountId> {
		RequestResult(RequestResult),
		Cancel(Cancel<AccountId>),
		Withdrawal(Withdrawal),
	}

	#[pallet::storage]
	#[pallet::getter(fn get_last_processed_request_on_l2)]
	// Id of the last request originating on other chain that has been executed
	pub type LastProcessedRequestOnL2<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ChainId, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_l2_origin_updates_counter)]
	// Id of the next request that will originate on this chain
	pub type L2OriginRequestId<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::ChainId,
		u128,
		ValueQuery,
		frame_support::traits::ConstU128<1>,
	>;

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
		L2Request<T::AccountId>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_awaiting_cancel_resolution)]
	pub type AwaitingCancelResolution<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::ChainId, T::AccountId), BTreeSet<u128>, ValueQuery>;

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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// (seuquencer, end_of_dispute_period, lastAcceptedRequestOnL1, lastProccessedRequestOnL1)
		L1ReadStored((T::ChainId, T::AccountId, u128, messages::Range, H256)),
		// Chain, request id
		RequestProcessedOnL2(T::ChainId, u128),
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
		type MaintenanceStatusProvider: GetMaintenanceStatusTrait;
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

			let hash_of_pending_request = Self::calculate_hash_of_pending_requests(request.clone());

			let l2_request_id = L2OriginRequestId::<T>::get(chain);
			L2OriginRequestId::<T>::mutate(chain, |request_id| request_id.saturating_inc());

			let cancel_request = Cancel {
				requestId: RequestId { origin: Origin::L2, id: l2_request_id },
				updater: submitter.clone(),
				canceler: canceler.clone(),
				range: request.range().ok_or(Error::<T>::InvalidUpdate)?,
				hash: hash_of_pending_request,
			};

			AwaitingCancelResolution::<T>::mutate((chain, submitter), |v| v.insert(l2_request_id));
			AwaitingCancelResolution::<T>::mutate((chain, canceler), |v| v.insert(l2_request_id));

			L2Requests::<T>::insert(
				chain,
				RequestId::from((Origin::L2, l2_request_id)),
				L2Request::Cancel(cancel_request),
			);

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

			let l2_request_id = L2OriginRequestId::<T>::get(chain);
			L2OriginRequestId::<T>::mutate(chain, |request_id| request_id.saturating_inc());

			let request_id = RequestId { origin: Origin::L2, id: l2_request_id };
			let withdrawal_update = Withdrawal {
				requestId: request_id.clone(),
				withdrawalRecipient: recipient,
				tokenAddress: token_address,
				amount: U256::from(amount),
			};
			// add cancel request to pending updates
			L2Requests::<T>::insert(chain, request_id, L2Request::Withdrawal(withdrawal_update));

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
			let calculated_hash = Self::calculate_hash_of_pending_requests(l1_update);
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
						rights.read_rights = 1;
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

		let id = L2OriginRequestId::<T>::get(l1);
		L2OriginRequestId::<T>::mutate(l1, |request_id| {
			request_id.saturating_inc();
		});

		let l2_request_id = RequestId { origin: Origin::L2, id };

		let (status, request_type) = match request.clone() {
			messages::L1UpdateRequest::Deposit(deposit) =>
				(Self::process_deposit(&deposit).is_ok(), UpdateType::DEPOSIT),
			messages::L1UpdateRequest::CancelResolution(cancel) => (
				Self::process_cancel_resolution(l1, &cancel).is_ok(),
				UpdateType::CANCEL_RESOLUTION,
			),
			messages::L1UpdateRequest::WithdrawalResolution(withdrawal) => (
				Self::process_withdrawal_resolution(l1, &withdrawal).is_ok(),
				UpdateType::WITHDRAWAL_RESOLUTION,
			),
			messages::L1UpdateRequest::Remove(remove) =>
				(Self::process_l2_updates_to_remove(l1, &remove).is_ok(), UpdateType::INDEX_UPDATE),
		};

		L2Requests::<T>::insert(
			l1,
			request.request_id(),
			L2Request::RequestResult(RequestResult {
				requestId: l2_request_id,
				originRequestId: request.id(),
				status,
				updateType: request_type,
			}),
		);

		Pallet::<T>::deposit_event(Event::RequestProcessedOnL2(l1, request.id()));

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
			update.pendingWithdrawalResolutions.iter().map(|r| r.requestId.id).max(),
			update.pendingCancelResolutions.iter().map(|r| r.requestId.id).max(),
			update.pendingL2UpdatesToRemove.iter().map(|r| r.requestId.id).max(),
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

	fn process_deposit(deposit_request_details: &messages::Deposit) -> Result<(), &'static str> {
		let account: T::AccountId =
			T::AddressConverter::convert(deposit_request_details.depositRecipient);

		let amount: u128 =
			deposit_request_details.amount.try_into().or(Err(Error::<T>::BalanceOverflow))?;

		// check ferried

		let eth_asset = L1Asset::Ethereum(deposit_request_details.tokenAddress);
		let asset_id = match T::AssetRegistryProvider::get_l1_asset_id(eth_asset.clone()) {
			Some(id) => id,
			None => T::AssetRegistryProvider::create_l1_asset(eth_asset)
				.or(Err(Error::<T>::L1AssetCreationFailed))?,
		};

		// ADD tokens: mint tokens for user
		T::Tokens::mint(
			asset_id,
			&account,
			amount.try_into().or(Err(Error::<T>::BalanceOverflow))?,
		)?;

		TotalNumberOfDeposits::<T>::mutate(|v| *v = v.saturating_add(One::one()));
		log!(debug, "Deposit processed successfully: {:?}", deposit_request_details);

		Ok(())
	}

	fn process_withdrawal_resolution(
		l1: T::ChainId,
		withdrawal_resolution: &messages::WithdrawalResolution,
	) -> Result<(), &'static str> {
		L2Requests::<T>::remove(
			l1,
			RequestId::from((Origin::L2, withdrawal_resolution.l2RequestId)),
		);
		TotalNumberOfWithdrawals::<T>::mutate(|v| *v = v.saturating_add(One::one()));
		//TODO: handle sending tokens back
		log!(debug, "Withdrawal resolution processed successfully: {:?}", withdrawal_resolution);
		Ok(())
	}

	fn process_cancel_resolution(
		l1: T::ChainId,
		cancel_resolution: &messages::CancelResolution,
	) -> Result<(), &'static str> {
		let cancel_request_id = cancel_resolution.l2RequestId;
		let cancel_justified = cancel_resolution.cancelJustified;

		let cancel_update =
			match L2Requests::<T>::get(l1, RequestId::from((Origin::L2, cancel_request_id))) {
				Some(L2Request::Cancel(cancel)) => Some(cancel),
				_ => None,
			}
			.ok_or("NoCancelRequest")?;

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

		L2Requests::<T>::remove(l1, RequestId::from((Origin::L2, cancel_request_id)));

		AwaitingCancelResolution::<T>::mutate((l1, &updater), |v| v.remove(&cancel_request_id));
		AwaitingCancelResolution::<T>::mutate((l1, &canceler), |v| v.remove(&cancel_request_id));

		// slash is after adding rights, since slash can reduce stake below required level and remove all rights
		T::SequencerStakingProvider::slash_sequencer(l1, &to_be_slashed, to_reward.as_ref())?;

		log!(debug, "SLASH for: {:?}, rewarded: {:?}", to_be_slashed, to_reward);

		log!(debug, "Cancel resolutiuon processed successfully: {:?}", cancel_resolution);
		// additional checks
		Ok(())
	}

	fn process_l2_updates_to_remove(
		l1: T::ChainId,
		updates_to_remove_request_details: &messages::L2UpdatesToRemove,
	) -> Result<(), &'static str> {
		for requestId in updates_to_remove_request_details.l2UpdatesToRemove.iter() {
			L2Requests::<T>::remove(l1, RequestId { origin: Origin::L1, id: *requestId });
		}

		log!(
			debug,
			"Update removal processed successfully, removed: {:?}",
			updates_to_remove_request_details
		);
		//additional checks

		Ok(())
	}

	fn to_eth_cancel(cancel: Cancel<T::AccountId>) -> messages::eth_abi::Cancel {
		messages::eth_abi::Cancel {
			requestId: cancel.requestId.into(),
			range: cancel.range.into(),
			hash: alloy_primitives::FixedBytes::<32>::from_slice(&cancel.hash[..]),
		}
	}

	fn to_eth_request_result(request: RequestResult) -> messages::eth_abi::RequestResult {
		messages::eth_abi::RequestResult {
			requestId: request.requestId.into(),
			originRequestId: messages::to_eth_u256(request.originRequestId.into()),
			updateType: request.updateType.into(),
			status: request.status.into(),
		}
	}

	fn to_eth_withdrawal(withdrawal: Withdrawal) -> messages::eth_abi::Withdrawal {
		messages::eth_abi::Withdrawal {
			requestId: withdrawal.requestId.into(),
			withdrawalRecipient: withdrawal.withdrawalRecipient.into(),
			tokenAddress: withdrawal.tokenAddress.into(),
			amount: to_eth_u256(withdrawal.amount),
		}
	}

	fn calculate_hash_of_pending_requests(update: messages::L1Update) -> H256 {
		let update: messages::eth_abi::L1Update = update.into();
		let hash: [u8; 32] = Keccak256::digest(&update.abi_encode()[..]).into();
		H256::from(hash)
	}

	fn get_l2_update(l1: T::ChainId) -> messages::eth_abi::L2Update {
		let mut update = messages::eth_abi::L2Update {
			results: Vec::new(),
			cancels: Vec::new(),
			withdrawals: Vec::new(),
		};

		for (request_id, req) in L2Requests::<T>::iter_prefix(l1) {
			match req {
				L2Request::RequestResult(result) =>
					update.results.push(Self::to_eth_request_result(result)),
				L2Request::Cancel(cancel) => {
					update.cancels.push(Self::to_eth_cancel(cancel));
				},
				L2Request::Withdrawal(withdrawal) => {
					update.withdrawals.push(Self::to_eth_withdrawal(withdrawal));
				},
			};
		}

		update.results.sort_by(|a, b| a.requestId.id.cmp(&b.requestId.id));
		update.cancels.sort_by(|a, b| a.requestId.id.cmp(&b.requestId.id));

		update.withdrawals.sort_by(|a, b| a.requestId.id.cmp(&b.requestId.id));

		update
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

	pub fn pending_l2_requests_proof(chain: T::ChainId) -> sp_core::H256 {
		let hash: [u8; 32] = Keccak256::digest(Self::l2_update_encoded(chain).as_slice()).into();
		hash.into()
	}

	pub fn l2_update_encoded(chain: T::ChainId) -> Vec<u8> {
		let update = Pallet::<T>::get_l2_update(chain);
		update.abi_encode()
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
			!update.pendingDeposits.is_empty() ||
				!update.pendingCancelResolutions.is_empty() ||
				!update.pendingL2UpdatesToRemove.is_empty(),
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
		ensure!(
			update
				.pendingL2UpdatesToRemove
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

		ensure!(
			update
				.pendingL2UpdatesToRemove
				.iter()
				.map(|v| v.requestId.id)
				.into_iter()
				.tuple_windows()
				.all(|(a, b)| a < b),
			Error::<T>::InvalidUpdate
		);

		ensure!(
			update
				.pendingWithdrawalResolutions
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
			update.pendingWithdrawalResolutions.first().map(|v| v.requestId.id),
			update.pendingL2UpdatesToRemove.first().map(|v| v.requestId.id),
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
			(update.pendingWithdrawalResolutions.len() as u128) +
			(update.pendingCancelResolutions.len() as u128) +
			(update.pendingL2UpdatesToRemove.len() as u128);

		ensure!(last_id > LastProcessedRequestOnL2::<T>::get(l1), Error::<T>::WrongRequestId);

		let mut deposit_it = update.pendingDeposits.iter();
		let mut withdrawal_it = update.pendingWithdrawalResolutions.iter();
		let mut cancel_it = update.pendingCancelResolutions.iter();
		let mut updates_it = update.pendingL2UpdatesToRemove.iter();
		let mut withdrawal = withdrawal_it.next();

		let mut deposit = deposit_it.next();
		let mut cancel = cancel_it.next();
		let mut update = updates_it.next();

		for id in (lowest_id..last_id).into_iter() {
			match (deposit, cancel, update, withdrawal) {
				(Some(d), _, _, _) if d.requestId.id == id => {
					deposit = deposit_it.next();
				},
				(_, Some(c), _, _) if c.requestId.id == id => {
					cancel = cancel_it.next();
				},
				(_, _, Some(u), _) if u.requestId.id == id => {
					update = updates_it.next();
				},
				(_, _, _, Some(w)) if w.requestId.id == id => {
					withdrawal = withdrawal_it.next();
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

		Pallet::<T>::deposit_event(Event::L1ReadStored((
			l1,
			sequencer,
			dispute_period_end,
			requests_range,
			H256::from_slice(request_hash.as_slice()),
		)));

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
					read_rights: T::RightsMultiplier::get(),
					cancel_rights: count.saturating_mul(T::RightsMultiplier::get()),
				},
			);

			for (_, rights) in sequencer_set.iter_mut().filter(|(s, _)| *s != sequencer) {
				rights.cancel_rights.saturating_accrue(T::RightsMultiplier::get())
			}
		});
	}

	fn sequencer_unstaking(chain: T::ChainId, sequencer: &AccountIdOf<T>) -> DispatchResult {
		ensure!(
			LastUpdateBySequencer::<T>::get((chain, sequencer))
				.saturating_add(T::DisputePeriodLength::get()) <
				<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
			Error::<T>::SequencerLastUpdateStillInDisputePeriod
		);
		ensure!(
			AwaitingCancelResolution::<T>::get((chain, &sequencer)).is_empty(),
			Error::<T>::SequencerAwaitingCancelResolution
		);

		LastUpdateBySequencer::<T>::remove((chain, &sequencer));
		AwaitingCancelResolution::<T>::remove((chain, &sequencer));

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
