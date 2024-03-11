#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, Get, StorageVersion},
	StorageHasher,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use messages::{to_eth_u256, PendingRequestType, UpdateType};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::SaturatedConversion;

use alloy_sol_types::SolValue;
use frame_support::traits::WithdrawReasons;
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

impl Convert<[u8; 20], sp_runtime::AccountId32>
	for EthereumAddressConverter<sp_runtime::AccountId32>
{
	fn convert(eth_addr: [u8; 20]) -> sp_runtime::AccountId32 {
		Blake2_256::hash(eth_addr.as_ref()).into()
	}
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

mod messages;

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

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
	pub struct Cancel<AccountId> {
		pub l2RequestId: U256,
		pub updater: AccountId,
		pub canceler: AccountId,
		pub lastProccessedRequestOnL1: U256,
		pub lastAcceptedRequestOnL1: U256,
		pub hash: H256,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
	pub struct Withdrawal {
		pub l2RequestId: U256,
		pub withdrawalRecipient: [u8; 20],
		pub tokenAddress: [u8; 20],
		pub amount: U256,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo)]
	pub enum PendingUpdate<AccountId> {
		RequestResult((bool, UpdateType)),
		Cancel(Cancel<AccountId>),
		Withdrawal(Withdrawal),
	}

	#[pallet::storage]
	#[pallet::getter(fn get_sequencer_count)]
	pub type sequencer_count<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_last_processed_request_on_l2)]
	pub type last_processed_request_on_l2<T: Config> = StorageValue<_, U256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_l2_origin_updates_counter)]
	pub type l2_origin_updates_counter<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_requests)]
	pub type pending_requests<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, (T::AccountId, messages::L1Update), OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub type request_to_execute<T: Config> =
		StorageMap<_, Blake2_128Concat, u128, (u128, messages::L1Update), ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub type request_to_execute_cnt<T: Config> = StorageValue<_, u128, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_sequencer_rights)]
	pub type sequencer_rights<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SequencerRights, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_updates)]
	pub type pending_updates<T: Config> =
		StorageMap<_, Blake2_128Concat, sp_core::U256, PendingUpdate<T::AccountId>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// (seuquencer, end_of_dispute_period, lastAcceptedRequestOnL1, lastProccessedRequestOnL1)
		PendingRequestStored((T::AccountId, u128, sp_core::U256, sp_core::U256, H256)),
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
			l2_origin_updates_counter::<T>::put(u128::MAX / 2);
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
			Self::validate_l1_update(&requests)?;

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

			// insert pending_requests
			pending_requests::<T>::insert(
				U256::from(dispute_period_end),
				(sequencer.clone(), requests.clone()),
			);

			let update: messages::eth_abi::L1Update = requests.clone().into();
			let request_hash = Keccak256::digest(&update.abi_encode());

			Pallet::<T>::deposit_event(Event::PendingRequestStored((
				sequencer,
				dispute_period_end,
				requests.lastAcceptedRequestOnL1,
				requests.lastProccessedRequestOnL1,
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
			Self::validate_l1_update(&update)?;
			Self::schedule_requests(update.into());
			Self::process_requests();
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		//EXTRINSIC2 (who canceled, dispute_period_end(u32-blocknum)))
		pub fn cancel_requests_from_l1(
			origin: OriginFor<T>,
			requests_to_cancel: U256,
		) -> DispatchResultWithPostInfo {
			let canceler = ensure_signed(origin)?;

			sequencer_rights::<T>::try_mutate_exists(canceler.clone(), |maybe_sequencer| {
				if let Some(ref mut sequencer) = maybe_sequencer {
					sequencer.cancelRights -= 1;
					Ok(())
				} else {
					Err(Error::<T>::ReadRightsExhausted)
				}
			})?;

			let (submitter, request) = pending_requests::<T>::take(requests_to_cancel)
				.ok_or(Error::<T>::RequestDoesNotExist)?;

			let hash_of_pending_request = Self::calculate_hash_of_pending_requests(request.clone());

			let l2RequestId = U256::from(Self::get_l2_origin_updates_counter());
			// create cancel request
			let cancel_request = Cancel {
				l2RequestId,
				updater: submitter,
				canceler,
				lastProccessedRequestOnL1: request.lastProccessedRequestOnL1,
				lastAcceptedRequestOnL1: request.lastAcceptedRequestOnL1,
				hash: hash_of_pending_request,
			};
			// add cancel request to pending updates
			pending_updates::<T>::insert(l2RequestId, PendingUpdate::Cancel(cancel_request));

			let l2_origin_updates_counter = Self::get_l2_origin_updates_counter()
				.checked_add(1)
				.ok_or(Error::<T>::MathOverflow)?;
			l2_origin_updates_counter::<T>::put(l2_origin_updates_counter);

			// remove whole l1l2 update (read) from pending requests
			pending_requests::<T>::remove(&requests_to_cancel);

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

			let l2RequestId = U256::from(Self::get_l2_origin_updates_counter());

			let withdrawal_update = Withdrawal {
				l2RequestId,
				withdrawalRecipient,
				tokenAddress,
				amount: U256::from(amount),
			};
			// add cancel request to pending updates
			pending_updates::<T>::insert(l2RequestId, PendingUpdate::Withdrawal(withdrawal_update));

			let l2_origin_updates_counter = Self::get_l2_origin_updates_counter()
				.checked_add(1)
				.ok_or(Error::<T>::MathOverflow)?;
			// increase counter for updates originating on l2
			l2_origin_updates_counter::<T>::put(l2_origin_updates_counter);
			Ok(().into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn force_cancel_requests_from_l1(
			origin: OriginFor<T>,
			requests_to_cancel: U256,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;

			let (submitter, request) = pending_requests::<T>::take(requests_to_cancel)
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
		let pending_requests_to_process = pending_requests::<T>::get::<U256>(request_id.into());
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
		if let Some(pending_requests_to_process) =
			pending_requests::<T>::get(U256::from(block_number))
		{
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

			Self::schedule_requests(requests.clone());
		}
		pending_requests::<T>::remove(U256::from(
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
		));
		Self::process_requests();
	}

	fn process_single_request(request: (sp_core::U256, messages::L1UpdateRequest)) {
		let (request_id, request_details) = request;

		if request_id <= last_processed_request_on_l2::<T>::get().into() {
			return
		}

		match request_details {
			messages::L1UpdateRequest::Deposit(deposit) => pending_updates::<T>::insert(
				request_id,
				PendingUpdate::RequestResult((
					Self::process_deposit(&deposit).is_ok(),
					UpdateType::DEPOSIT,
				)),
			),
			messages::L1UpdateRequest::Cancel(cancel) => pending_updates::<T>::insert(
				request_id,
				PendingUpdate::RequestResult((
					Self::process_cancel_resolution(&cancel.into()).is_ok(),
					UpdateType::CANCEL_RESOLUTION,
				)),
			),
			messages::L1UpdateRequest::Remove(remove) => pending_updates::<T>::insert(
				request_id,
				PendingUpdate::RequestResult((
					Self::process_l2_updates_to_remove(&remove).is_ok(),
					UpdateType::INDEX_UPDATE,
				)),
			),
		};

		last_processed_request_on_l2::<T>::put(request_id);
	}

	fn process_requests() {
		if let Some(end_dispute_period) = request_to_execute_cnt::<T>::get() {
			request_to_execute::<T>::mutate_exists(end_dispute_period, |req| {
				if let Some((pos, update)) = req.clone() {
					let requests_count = update.order.len() as u128;
					let requests = update.clone().into_requests();
					let requests_to_process = requests
						.into_iter()
						.skip(pos as usize)
						.enumerate()
						.filter(|(_, (request_id, _update))| {
							*request_id > last_processed_request_on_l2::<T>::get().into()
						})
						.take(T::RequestsPerBlock::get() as usize);

					for (i, r) in requests_to_process {
						Self::process_single_request(r);
						if (pos + i as u128) >= requests_count {
							*req = None;
							break
						} else {
							*req = Some((pos + (i as u128), update.clone()));
						}
					}
				}
			});
		}
	}

	fn schedule_requests(update: messages::L1Update) {
		if let Some(id) = request_to_execute_cnt::<T>::get() {
			request_to_execute_cnt::<T>::put(id + 1);
			request_to_execute::<T>::insert(id + 1, (0u128, update));
		} else {
			request_to_execute_cnt::<T>::put(0);
			request_to_execute::<T>::insert(0u128, (0u128, update));
		}
	}

	fn process_deposit(deposit_request_details: &messages::Deposit) -> Result<(), &'static str> {
		let account: T::AccountId =
			T::AddressConverter::convert(deposit_request_details.depositRecipient);

		let amount: u128 = deposit_request_details.amount.try_into().unwrap();

		// check ferried

		// translate to token id
		// Add check if token exists, if not create one

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

	fn process_cancel_resolution(
		cancel_resolution: &messages::CancelResolution,
	) -> Result<(), &'static str> {
		let cancel_request_id = cancel_resolution.l2RequestId;
		let cancel_justified = cancel_resolution.cancelJustified;

		let cancel_update = match pending_updates::<T>::get(cancel_request_id) {
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

		pending_updates::<T>::remove(cancel_request_id);

		// slash is after adding rights, since slash can reduce stake below required level and remove all rights
		Self::slash(&to_be_slashed);

		log!(debug, "Cancel resolutiuon processed successfully: {:?}", cancel_resolution);
		// additional checks
		Ok(())
	}

	fn process_l2_updates_to_remove(
		updates_to_remove_request_details: &messages::L2UpdatesToRemove,
	) -> Result<(), &'static str> {
		for requestId in updates_to_remove_request_details.l2UpdatesToRemove.iter() {
			pending_updates::<T>::remove(requestId);
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

	fn to_eth_cancel(request_id: U256, cancel: Cancel<T::AccountId>) -> messages::eth_abi::Cancel {
		messages::eth_abi::Cancel {
			l2RequestId: to_eth_u256(request_id),
			lastProccessedRequestOnL1: to_eth_u256(cancel.lastProccessedRequestOnL1),
			lastAcceptedRequestOnL1: to_eth_u256(cancel.lastAcceptedRequestOnL1),
			hash: alloy_primitives::FixedBytes::<32>::from_slice(&cancel.hash[..]),
		}
	}

	fn to_eth_withdrawal(
		l2RequestId: U256,
		withdrawal: Withdrawal,
	) -> messages::eth_abi::Withdrawal {
		messages::eth_abi::Withdrawal {
			l2RequestId: to_eth_u256(l2RequestId),
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

	fn get_l2_update() -> messages::eth_abi::L2Update {
		let mut update = messages::eth_abi::L2Update {
			results: Vec::new(),
			cancels: Vec::new(),
			withdrawals: Vec::new(),
		};

		for (request_id, req) in pending_updates::<T>::iter() {
			match req {
				PendingUpdate::RequestResult((status, request_type)) =>
					update.results.push(messages::eth_abi::RequestResult {
						requestId: to_eth_u256(request_id),
						updateType: request_type,
						status,
					}),
				PendingUpdate::Cancel(cancel) => {
					update.cancels.push(Self::to_eth_cancel(request_id, cancel));
				},
				PendingUpdate::Withdrawal(withdrawal) => {
					update.withdrawals.push(Self::to_eth_withdrawal(request_id, withdrawal));
				},
			};
		}

		update.results.sort_by(|a, b| a.requestId.partial_cmp(&b.requestId).unwrap());

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
		let update = Pallet::<T>::get_l2_update();
		update.abi_encode()
	}

	pub fn validate_l1_update(update: &messages::L1Update) -> DispatchResult {
		ensure!(!update.order.is_empty(), Error::<T>::EmptyUpdate);
		ensure!(
			update.offset.saturated_into::<u128>() + (update.order.len() as u128) >
				last_processed_request_on_l2::<T>::get().saturated_into::<u128>(),
			Error::<T>::InvalidUpdate
		);
		// if there are no requests on l1 there is no need for update
		ensure!(update.lastAcceptedRequestOnL1 > 0u128.into(), Error::<T>::EmptyUpdate);
		ensure!(
			update.lastAcceptedRequestOnL1 > update.lastProccessedRequestOnL1,
			Error::<T>::InvalidUpdate
		);
		ensure!(update.offset > update.lastProccessedRequestOnL1, Error::<T>::WrongRequestId);
		ensure!(update.offset <= update.lastAcceptedRequestOnL1, Error::<T>::WrongRequestId);

		let deposits_count = update.pendingDeposits.len();
		let cancels_count = update.pendingCancelResultions.len();
		let l2_updates_count = update.pendingL2UpdatesToRemove.len();

		ensure!(
			update.order.iter().filter(|e| **e == PendingRequestType::DEPOSIT).count() ==
				deposits_count,
			Error::<T>::InvalidUpdate
		);
		ensure!(
			update
				.order
				.iter()
				.filter(|e| **e == PendingRequestType::CANCEL_RESOLUTION)
				.count() == cancels_count,
			Error::<T>::InvalidUpdate
		);
		ensure!(
			update
				.order
				.iter()
				.filter(|e| **e == PendingRequestType::L2_UPDATES_TO_REMOVE)
				.count() == l2_updates_count,
			Error::<T>::InvalidUpdate
		);
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
