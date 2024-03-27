#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, Get, StorageVersion},
	StorageHasher,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use messages::{to_eth_u256, Origin, RequestId, UpdateType, L1};
use scale_info::prelude::{format, string::String};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::SaturatedConversion;

use alloy_sol_types::SolValue;
use frame_support::traits::WithdrawReasons;
use itertools::Itertools;
use mangata_support::traits::{
	AssetRegistryProviderTrait, RolldownProviderTrait, SequencerStakingProviderTrait,
};
use mangata_types::assets::L1Asset;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use sha3::{Digest, Keccak256};
use sp_core::{H256, U256};
use sp_runtime::{serde::Serialize, traits::Convert};
use sp_std::{convert::TryInto, prelude::*, vec::Vec};

pub type CurrencyIdOf<T> = <<T as Config>::Tokens as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

pub type BalanceOf<T> =
	<<T as Config>::Tokens as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

const DISPUTE_PERIOD_LENGTH: u128 = 5;
const RIGHTS_MULTIPLIER: u128 = 1;

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
		pub readRights: u128,
		pub cancelRights: u128,
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

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
	pub struct Withdrawal {
		pub requestId: RequestId,
		pub withdrawalRecipient: [u8; 20],
		pub tokenAddress: [u8; 20],
		pub amount: U256,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo)]
	pub enum PendingUpdate<AccountId> {
		RequestResult(RequestResult),
		Cancel(Cancel<AccountId>),
		Withdrawal(Withdrawal),
	}

	#[pallet::storage]
	#[pallet::getter(fn get_sequencer_count)]
	pub type sequencer_count<T: Config> = StorageValue<_, u128, ValueQuery>;

	//TODO: multi L1
	#[pallet::storage]
	#[pallet::getter(fn get_last_processed_request_on_l2)]
	pub type last_processed_request_on_l2<T: Config> =
		StorageMap<_, Blake2_128Concat, L1, u128, ValueQuery>;

	//TODO: multi L1
	#[pallet::storage]
	#[pallet::getter(fn get_l2_origin_updates_counter)]
	pub type l2_origin_request_id<T: Config> =
		StorageMap<_, Blake2_128Concat, L1, u128, ValueQuery>;

	//TODO: multi L1
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_requests)]
	pub type pending_requests<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u128,
		Blake2_128Concat,
		L1,
		(T::AccountId, messages::L1Update),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub type request_to_execute<T: Config> =
		StorageMap<_, Blake2_128Concat, u128, (L1, messages::L1Update), OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub type request_to_execute_cnt<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub type request_to_execute_last<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_sequencer_rights)]
	pub type sequencer_rights<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SequencerRights, OptionQuery>;

	//maps L1 and !!!! request origin id!!! to pending update
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_updates)]
	pub type pending_updates<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		L1,
		Blake2_128Concat,
		RequestId,
		PendingUpdate<T::AccountId>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// (seuquencer, end_of_dispute_period, lastAcceptedRequestOnL1, lastProccessedRequestOnL1)
		L1ReadStored((T::AccountId, u128, messages::Range, H256)),
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		OperationFailed,
		ReadRightsExhausted,
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
		MultipleUpdatesInSingleBlock,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type SequencerStakingProvider: SequencerStakingProviderTrait<
			Self::AccountId,
			BalanceOf<Self>,
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
		type RequestsPerBlock: Get<u128>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub sequencers: Vec<T::AccountId>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { sequencers: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			for s in self.sequencers.iter() {
				sequencer_rights::<T>::insert(
					s.clone(),
					SequencerRights {
						readRights: 1,
						cancelRights: (self.sequencers.len() - 1) as u128,
					},
				);
			}
			l2_origin_request_id::<T>::insert(L1::Ethereum, 1);
		}
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
			Self::validate_l1_update(L1::Ethereum, &requests)?;

			// check json length to prevent big data spam, maybe not necessary as it will be checked later and slashed
			let current_block_number =
				<frame_system::Pallet<T>>::block_number().saturated_into::<u128>();
			let dispute_period_length = Self::get_dispute_period();
			let dispute_period_end: u128 = current_block_number + dispute_period_length;

			// ensure sequencer has rights to update
			if let Some(sequencer) = sequencer_rights::<T>::get(&sequencer) {
				if sequencer.readRights == 0 {
					log!(debug, "{:?} does not have sufficient readRights", sequencer);
					return Err(Error::<T>::OperationFailed.into())
				}
			} else {
				log!(debug, "{:?} not a sequencer, CHEEKY BASTARD!", sequencer);
				return Err(Error::<T>::OperationFailed.into())
			}

			// // Decrease readRights by 1
			sequencer_rights::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
				if let Some(ref mut sequencer) = maybe_sequencer {
					sequencer.readRights -= 1;
				}
			});

			ensure!(
				!pending_requests::<T>::contains_key(dispute_period_end, L1::Ethereum),
				Error::<T>::MultipleUpdatesInSingleBlock
			);

			// insert pending_requests
			pending_requests::<T>::insert(
				dispute_period_end,
				L1::Ethereum,
				(sequencer.clone(), requests.clone()),
			);

			let update: messages::eth_abi::L1Update = requests.clone().into();
			let request_hash = Keccak256::digest(&update.abi_encode());

			Pallet::<T>::deposit_event(Event::L1ReadStored((
				sequencer,
				dispute_period_end,
				requests.range().ok_or(Error::<T>::InvalidUpdate)?,
				H256::from_slice(request_hash.as_slice()),
			)));

			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn force_update_l2_from_l1(
			origin: OriginFor<T>,
			update: messages::L1Update,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;
			let l1 = L1::Ethereum;
			Self::validate_l1_update(L1::Ethereum, &update)?;
			Self::schedule_requests(l1, update.into());
			Self::process_requests();
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		//EXTRINSIC2 (who canceled, dispute_period_end(u32-blocknum)))
		pub fn cancel_requests_from_l1(
			origin: OriginFor<T>,
			requests_to_cancel: u128,
		) -> DispatchResultWithPostInfo {
			let canceler = ensure_signed(origin)?;
			let l1 = L1::Ethereum;

			sequencer_rights::<T>::try_mutate_exists(canceler.clone(), |maybe_sequencer| {
				if let Some(ref mut sequencer) = maybe_sequencer {
					sequencer.cancelRights -= 1;
					Ok(())
				} else {
					Err(Error::<T>::ReadRightsExhausted)
				}
			})?;

			let (submitter, request) = pending_requests::<T>::take(requests_to_cancel, l1)
				.ok_or(Error::<T>::RequestDoesNotExist)?;

			let hash_of_pending_request = Self::calculate_hash_of_pending_requests(request.clone());

			let l2_request_id = l2_origin_request_id::<T>::mutate(l1, |request_id| {
				let current = request_id.clone();
				// TODO: safe math
				*request_id += 1;
				current
			});
			// create cancel request
			let cancel_request = Cancel {
				requestId: RequestId { origin: Origin::L2, id: l2_request_id },
				updater: submitter,
				canceler,
				range: request.range().ok_or(Error::<T>::InvalidUpdate)?,
				hash: hash_of_pending_request,
			};

			pending_updates::<T>::insert(
				l1,
				RequestId::from((Origin::L2, l2_request_id)),
				PendingUpdate::Cancel(cancel_request),
			);

			Ok(().into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn withdraw(
			origin: OriginFor<T>,
			withdrawalRecipient: [u8; 20],
			tokenAddress: [u8; 20],
			amount: u128,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;
			let l1 = L1::Ethereum;

			let eth_asset = L1Asset::Ethereum(tokenAddress);
			let asset_id = T::AssetRegistryProvider::get_l1_asset_id(eth_asset.clone())
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

			let l2_request_id = l2_origin_request_id::<T>::mutate(l1, |request_id| {
				let current = request_id.clone();
				// TODO: safe math
				*request_id += 1;
				current
			});

			let request_id = RequestId { origin: Origin::L2, id: l2_request_id };
			let withdrawal_update = Withdrawal {
				requestId: request_id.clone(),
				withdrawalRecipient,
				tokenAddress,
				amount: U256::from(amount),
			};
			// add cancel request to pending updates
			pending_updates::<T>::insert(
				l1,
				request_id,
				PendingUpdate::Withdrawal(withdrawal_update),
			);

			Ok(().into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn force_cancel_requests_from_l1(
			origin: OriginFor<T>,
			requests_to_cancel: u128,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;

			let (submitter, request) =
				pending_requests::<T>::take(requests_to_cancel, L1::Ethereum)
					.ok_or(Error::<T>::RequestDoesNotExist)?;

			sequencer_rights::<T>::mutate_exists(submitter.clone(), |maybe_sequencer| {
				match maybe_sequencer {
					&mut Some(ref mut sequencer_rights)
						if T::SequencerStakingProvider::is_active_sequencer(submitter.clone()) =>
					{
						sequencer_rights.readRights = 1;
					},
					_ => {},
				}
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

	pub fn verify_pending_requests(hash: H256, request_id: u128) -> Option<bool> {
		let pending_requests_to_process = pending_requests::<T>::get(request_id, L1::Ethereum);
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

		for (l1, pending_requests_to_process) in pending_requests::<T>::iter_prefix(block_number) {
			log!(debug, "dispute end {:?}", block_number);

			let sequencer = &pending_requests_to_process.0;
			let requests = pending_requests_to_process.1.clone();

			sequencer_rights::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
				match maybe_sequencer {
					&mut Some(ref mut sequencer_rights)
						if T::SequencerStakingProvider::is_active_sequencer(sequencer.clone()) =>
					{
						sequencer_rights.readRights += 1;
					},
					_ => {},
				}
			});

			Self::schedule_requests(l1, requests.clone());
		}

		let _ = pending_requests::<T>::clear_prefix(
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
			u32::MAX,
			None,
		);
		Self::process_requests();
	}

	fn process_single_request(l1: L1, request: messages::L1UpdateRequest) {
		if request.id() <= last_processed_request_on_l2::<T>::get(l1) {
			return
		}

		let id = l2_origin_request_id::<T>::mutate(l1, |request_id| {
			let current = request_id.clone();
			// TODO: safe math
			*request_id += 1;
			current
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

		pending_updates::<T>::insert(
			l1,
			request.request_id(),
			PendingUpdate::RequestResult(RequestResult {
				requestId: l2_request_id,
				originRequestId: request.id(),
				status,
				updateType: request_type,
			}),
		);

		last_processed_request_on_l2::<T>::insert(l1, request.id());
	}

	fn process_requests() {
		let mut limit = Self::get_max_requests_per_block();

		loop {
			if limit == 0 {
				return
			}
			if let Some((l1, r)) = request_to_execute::<T>::get(request_to_execute_cnt::<T>::get())
			{
				for req in r
					.into_requests()
					.into_iter()
					.filter(|request| request.id() > last_processed_request_on_l2::<T>::get(l1))
					.map(|val| Some(val))
					.chain(sp_std::iter::repeat(None))
					.take(limit.try_into().unwrap())
				{
					if let Some(request) = req {
						Self::process_single_request(l1, request);
						limit -= 1;
					} else {
						request_to_execute::<T>::remove(request_to_execute_cnt::<T>::get());
						request_to_execute_cnt::<T>::mutate(|v| *v += 1);
						break
					}
				}
			} else {
				if request_to_execute::<T>::contains_key(request_to_execute_cnt::<T>::get() + 1) {
					request_to_execute_cnt::<T>::mutate(|v| *v += 1);
				} else {
					break
				}
			}
		}
	}

	fn schedule_requests(l1: L1, update: messages::L1Update) {
		let id = request_to_execute_last::<T>::get();
		request_to_execute_last::<T>::put(id + 1);
		request_to_execute::<T>::insert(id + 1, (l1, update));
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
		log!(debug, "Deposit processed successfully: {:?}", deposit_request_details);

		// ADD tokens: mint tokens for user
		T::Tokens::mint(
			asset_id,
			&account,
			amount.try_into().or(Err(Error::<T>::BalanceOverflow))?,
		)?;
		Ok(())
	}

	fn process_withdrawal_resolution(
		l1: L1,
		withdrawal_resolution: &messages::WithdrawalResolution,
	) -> Result<(), &'static str> {
		pending_updates::<T>::remove(
			l1,
			RequestId::from((Origin::L2, withdrawal_resolution.l2RequestId)),
		);
		//TODO: handle sending tokens back
		log!(debug, "Withdrawal resolution processed successfully: {:?}", withdrawal_resolution);
		Ok(())
	}

	fn process_cancel_resolution(
		l1: L1,
		cancel_resolution: &messages::CancelResolution,
	) -> Result<(), &'static str> {
		let cancel_request_id = cancel_resolution.l2RequestId;
		let cancel_justified = cancel_resolution.cancelJustified;

		let cancel_update =
			match pending_updates::<T>::get(l1, RequestId::from((Origin::L2, cancel_request_id))) {
				Some(PendingUpdate::Cancel(cancel)) => Some(cancel),
				_ => None,
			}
			.ok_or("NoCancelRequest")?;

		let updater = cancel_update.updater;
		let canceler = cancel_update.canceler;
		let to_be_slashed = if cancel_justified { updater.clone() } else { canceler.clone() };

		// return rights to canceler and updater
		// only if active sequencer
		sequencer_rights::<T>::mutate_exists(updater.clone(), |maybe_sequencer| {
			match maybe_sequencer {
				&mut Some(ref mut sequencer)
					if T::SequencerStakingProvider::is_active_sequencer(updater) =>
				{
					sequencer.readRights += 1;
				},
				_ => {},
			}
		});
		sequencer_rights::<T>::mutate_exists(canceler.clone(), |maybe_sequencer| {
			match maybe_sequencer {
				&mut Some(ref mut sequencer)
					if T::SequencerStakingProvider::is_active_sequencer(canceler) =>
				{
					sequencer.cancelRights += 1;
				},
				_ => {},
			}
		});

		pending_updates::<T>::remove(l1, RequestId::from((Origin::L2, cancel_request_id)));

		// slash is after adding rights, since slash can reduce stake below required level and remove all rights
		Self::slash(&to_be_slashed);

		log!(debug, "Cancel resolutiuon processed successfully: {:?}", cancel_resolution);
		// additional checks
		Ok(())
	}

	fn process_l2_updates_to_remove(
		l1: L1,
		updates_to_remove_request_details: &messages::L2UpdatesToRemove,
	) -> Result<(), &'static str> {
		for requestId in updates_to_remove_request_details.l2UpdatesToRemove.iter() {
			pending_updates::<T>::remove(l1, RequestId { origin: Origin::L1, id: *requestId });
		}

		log!(
			debug,
			"Update removal processed successfully, removed: {:?}",
			updates_to_remove_request_details
		);
		//additional checks

		Ok(())
	}

	fn slash(sequencer: &T::AccountId) -> Result<(), &'static str> {
		// check if sequencer is active
		let is_active_sequencer_before =
			T::SequencerStakingProvider::is_active_sequencer(sequencer.clone());
		// slash sequencer
		T::SequencerStakingProvider::slash_sequencer(sequencer.clone())?;
		// check if sequencer is active
		let is_active_sequencer_after =
			T::SequencerStakingProvider::is_active_sequencer(sequencer.clone());

		// if sequencer was active and is not active anymore, remove rights
		if is_active_sequencer_before && !is_active_sequencer_after {
			Self::handle_sequencer_deactivation(sequencer.clone());
		}

		log!(debug, "SLASH for: {:?}", sequencer);

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

	fn get_l2_update(l1: L1) -> messages::eth_abi::L2Update {
		let mut update = messages::eth_abi::L2Update {
			results: Vec::new(),
			cancels: Vec::new(),
			withdrawals: Vec::new(),
		};

		for (request_id, req) in pending_updates::<T>::iter_prefix(l1) {
			match req {
				PendingUpdate::RequestResult(result) =>
					update.results.push(Self::to_eth_request_result(result)),
				PendingUpdate::Cancel(cancel) => {
					update.cancels.push(Self::to_eth_cancel(cancel));
				},
				PendingUpdate::Withdrawal(withdrawal) => {
					update.withdrawals.push(Self::to_eth_withdrawal(withdrawal));
				},
			};
		}

		update
			.results
			.sort_by(|a, b| a.requestId.id.partial_cmp(&b.requestId.id).unwrap());
		update
	}

	fn handle_sequencer_deactivation(deactivated_sequencer: T::AccountId) {
		// lower sequencer count
		sequencer_count::<T>::put(Self::get_sequencer_count() - 1);
		// remove all rights of deactivated sequencer
		sequencer_rights::<T>::remove(deactivated_sequencer.clone());
		// remove 1 cancel right of all sequencers
		for (sequencer, sequencer_rights) in sequencer_rights::<T>::iter() {
			if sequencer_rights.cancelRights > 0 {
				sequencer_rights::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
					if let Some(ref mut sequencer) = maybe_sequencer {
						sequencer.cancelRights -= RIGHTS_MULTIPLIER;
					}
				});
			}
		}
	}

	pub fn pending_updates_proof() -> sp_core::H256 {
		let hash: [u8; 32] = Keccak256::digest(Self::l2_update_encoded().as_slice()).into();
		hash.into()
	}

	pub fn l2_update_encoded() -> Vec<u8> {
		let update = Pallet::<T>::get_l2_update(L1::Ethereum);
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

	pub fn validate_l1_update(l1: L1, update: &messages::L1Update) -> DispatchResult {
		ensure!(
			!update.pendingDeposits.is_empty() ||
				!update.pendingCancelResultions.is_empty() ||
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
				.pendingCancelResultions
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
				.pendingCancelResultions
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
			update.pendingCancelResultions.first().map(|v| v.requestId.id),
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
			lowest_id <= last_processed_request_on_l2::<T>::get(l1) + 1,
			Error::<T>::WrongRequestId
		);

		let last_id = lowest_id +
			(update.pendingDeposits.len() as u128) +
			(update.pendingWithdrawalResolutions.len() as u128) +
			(update.pendingCancelResultions.len() as u128) +
			(update.pendingL2UpdatesToRemove.len() as u128);

		ensure!(last_id > last_processed_request_on_l2::<T>::get(l1), Error::<T>::WrongRequestId);

		let mut deposit_it = update.pendingDeposits.iter();
		let mut withdrawal_it = update.pendingWithdrawalResolutions.iter();
		let mut cancel_it = update.pendingCancelResultions.iter();
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
}

impl<T: Config> RolldownProviderTrait<AccountIdOf<T>> for Pallet<T> {
	fn new_sequencer_active(sequencer: AccountIdOf<T>) {
		// raise sequencer count
		sequencer_count::<T>::put(Self::get_sequencer_count() + 1);
		// add rights to new sequencer
		sequencer_rights::<T>::insert(
			sequencer.clone(),
			SequencerRights {
				readRights: RIGHTS_MULTIPLIER,
				cancelRights: RIGHTS_MULTIPLIER * (sequencer_count::<T>::get() - 1),
			},
		);

		// add 1 cancel right of all sequencers
		for (sequencer, sequencer_rights) in sequencer_rights::<T>::iter() {
			if sequencer_rights.cancelRights > 0 {
				sequencer_rights::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
					if let Some(ref mut sequencer) = maybe_sequencer {
						sequencer.cancelRights += RIGHTS_MULTIPLIER;
					}
				});
			}
		}
	}
}
