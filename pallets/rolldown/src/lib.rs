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
	pub type last_processed_request_on_l2<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_prev_last_processed_request_on_l2)]
	pub type prev_last_processed_request_on_l2<T: Config> = StorageValue<_, u128, ValueQuery>;

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
		PendingRequestStored((T::AccountId, H256)),
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
		// sequencer read
		//fn update_l2_from_l1(origin: String, l1_pending_requests_json: &'static str, current_block_number: u32) {
		pub fn update_l2_from_l1(
			origin: OriginFor<T>,
			requests: messages::L1Update,
		) -> DispatchResultWithPostInfo {
			let sequencer = ensure_signed(origin)?;

			ensure!(!requests.order.is_empty(), Error::<T>::EmptyUpdate);
			ensure!(requests.order.len() <= 10, Error::<T>::TooManyRequests);

			let deposits_count = requests.pendingDeposits.len();
			let cancels_count = requests.pendingCancelResultions.len();
			let l2_updates_count = requests.pendingL2UpdatesToRemove.len();

			ensure!(
				requests.order.iter().filter(|e| **e == PendingRequestType::DEPOSIT).count() ==
					deposits_count,
				Error::<T>::InvalidUpdate
			);
			ensure!(
				requests
					.order
					.iter()
					.filter(|e| **e == PendingRequestType::CANCEL_RESOLUTION)
					.count() == cancels_count,
				Error::<T>::InvalidUpdate
			);
			ensure!(
				requests
					.order
					.iter()
					.filter(|e| **e == PendingRequestType::L2_UPDATES_TO_REMOVE)
					.count() == l2_updates_count,
				Error::<T>::InvalidUpdate
			);

			// check json length to prevent big data spam, maybe not necessary as it will be checked later and slashed
			let current_block_number =
				<frame_system::Pallet<T>>::block_number().saturated_into::<u128>();
			let dispute_period_end: u128 = current_block_number + (DISPUTE_PERIOD_LENGTH as u128);

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
				H256::from_slice(request_hash.as_slice()),
			)));

			Ok(().into())
		}

		#[pallet::call_index(1)]
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

			log!(debug, "Pending Updates:");
			for (request_id, update) in pending_requests::<T>::iter() {
				log!(debug, "request_id: {:?}:  {:?} ", request_id, update);
			}
			log!(debug, "Pending requests:");
			for (request_id, update) in pending_updates::<T>::iter() {
				log!(debug, "request_id: {:?}:  {:?} ", request_id, update);
			}

			Ok(().into())
		}

		#[pallet::call_index(3)]
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

		// no checks if this read is correct, can put counters out of sync
		// #[pallet::call_index(2)]
		// #[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		// pub fn council_update_l2_from_l1(
		// 	origin: OriginFor<T>,
		// 	input_json: String,
		// ) -> DispatchResultWithPostInfo {
		// 	let _ = ensure_root(origin)?;
		//
		// 	match Self::deserialize_json(&input_json) {
		// 		Ok(deserialized) => {
		// 			log!(debug, "Deserialized struct: {:?}", deserialized);
		//
		// 			// Add a new variable of type Requests
		// 			let requests: Requests = deserialized;
		//
		// 			if requests.lastProccessedRequestOnL1 <
		// 				Self::get_prev_last_processed_request_on_l2()
		// 			{
		// 				log!(debug, "lastProccessedRequestOnL1 is less than prev_last_processed_request_on_l2");
		// 			} else {
		// 				for request_id in (requests.lastProccessedRequestOnL1 + 1_u128)..=
		// 					requests.lastAcceptedRequestOnL1
		// 				{
		// 					if let Some(request) = requests.requests.get(&request_id) {
		// 						// ignore if already processed
		// 						if request_id > Self::get_last_processed_request_on_l2() {
		// 							log!(debug, "EXECUTING: ");
		// 							let mut success = true;
		// 							match request {
		// 								Request::Deposit(deposit_request_details) => {
		// 									log!(debug, "Deposit: {:?}", deposit_request_details);
		// 									match Self::process_deposit(deposit_request_details) {
		// 										Ok(_) => (),
		// 										Err(_) => success = false,
		// 									};
		// 									PENDING_UPDATES::<T>::insert(
		// 										request_id,
		// 										Update::DepositUpdate(success),
		// 									);
		// 								},
		// 								Request::Withdraw(withdraw_request_details) => {
		// 									log!(debug, "Withdraw: {:?}", withdraw_request_details);
		// 									match Self::process_withdraw(withdraw_request_details) {
		// 										Ok(_) => (),
		// 										Err(_) => success = false,
		// 									};
		// 									PENDING_UPDATES::<T>::insert(
		// 										request_id,
		// 										Update::WithdrawUpdate(success),
		// 									);
		// 								},
		// 								Request::CancelResolution(
		// 									cancel_resolution_request_details,
		// 								) => {
		// 									log!(
		// 										info,
		// 										"CancelResolution: {:?}",
		// 										cancel_resolution_request_details
		// 									);
		// 									match Self::process_cancel_resolution(
		// 										cancel_resolution_request_details,
		// 									) {
		// 										Ok(_) => (),
		// 										Err(_) => success = false,
		// 									};
		// 									PENDING_UPDATES::<T>::insert(
		// 										request_id,
		// 										Update::ProcessedOnlyInfoUpdate(success),
		// 									);
		// 								},
		//
		// 								Request::L2UpdatesToRemove(
		// 									updates_to_remove_request_details,
		// 								) => {
		// 									log!(
		// 										info,
		// 										"L2UpdatesToRemove: {:?}",
		// 										updates_to_remove_request_details
		// 									);
		// 									match Self::process_l2_updates_to_remove(
		// 										updates_to_remove_request_details,
		// 									) {
		// 										Ok(_) => (),
		// 										Err(_) => success = false,
		// 									};
		// 									PENDING_UPDATES::<T>::insert(
		// 										request_id,
		// 										Update::ProcessedOnlyInfoUpdate(success),
		// 									);
		// 								},
		// 							}
		// 							// if success, increase last_processed_request_on_l2
		// 							last_processed_request_on_l2::<T>::put(request_id);
		// 						}
		// 					} else {
		// 						log!(debug, "No request found for request_id: {:?}", request_id);
		// 					}
		// 				}
		//
		// 				log!(debug, "Pending Updates:");
		// 				for (request_id, update) in PENDING_UPDATES::<T>::iter() {
		// 					log!(debug, "request_id: {:?}:  {:?} ", request_id, update);
		// 				}
		// 			}
		// 		},
		// 		Err(e) => {
		// 			log!(debug, "Error deserializing JSON: {:?}", e);
		// 		},
		// 	}
		// 	Ok(().into())
		// }
	}
}

impl<T: Config> Pallet<T> {
	// should run each block, check if dispute period ended, if yes, process pending requests
	fn end_dispute_period() {
		if let Some(pending_requests_to_process) = pending_requests::<T>::get(U256::from(
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
		)) {
			log!(debug, "dispute end ",);

			let sequencer = &pending_requests_to_process.0;
			let requests = pending_requests_to_process.1.clone();

			if requests.lastProccessedRequestOnL1 <
				sp_core::U256::from(Self::get_prev_last_processed_request_on_l2())
			{
				log!(
					debug,
					"lastProccessedRequestOnL1 is less than prev_last_processed_request_on_l2"
				);

				// TODO: SLASH sequencer for bringing unnecessary past requests, to be tested
				// Self::slash(sequencer);
			}

			// return readRights to sequencer
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
			Self::process_requests(sequencer, requests.clone());
		}
		pending_requests::<T>::remove(U256::from(
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
		));
		// TODO: prev_last_processed_request_on_l2 update goes here?
	}

	fn process_requests(_sequencer: &T::AccountId, update: messages::L1Update) {
		// TODO: make sure not missing any request, 1st reqid in "read" == last_processed_request_on_l2 + 1
		// TODO: check if not missing any request, not processing requests. This is double check, first should be done by sequencers and requests with missing request should be canceled
		// for key in (requests.lastProccessedRequestOnL1 + 1_u128)..=requests.lastAcceptedRequestOnL1
		// {
		// 	if let Some(_request) = requests.requests.get(&key) {
		// 	} else {
		// 		log!(debug, "No request found for key: {:?}", key);
		// 		// SLASH sequencer for missing request
		// 		Self::slash(sequencer);
		// 		return
		// 	}
		// }

		for (request_id, request_details) in update.into_requests() {
			if pending_updates::<T>::contains_key(request_id) {
				log!(debug, "Request already processed: {:?}", request_id);
				continue
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
				_ => {},
			};
			// if success, increase last_processed_request_on_l2
			let request_id: u128 = request_id.try_into().unwrap();
			last_processed_request_on_l2::<T>::put(request_id);
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

		// slash is after adding rights, since slash can reduce stake below required level and remove all rights
		Self::slash(&to_be_slashed);

		log!(debug, "Cancel resolutiuon processed successfully: {:?}", cancel_resolution);
		// additional checks
		Ok(())
	}

	fn process_l2_updates_to_remove(
		updates_to_remove_request_details: &messages::L2UpdatesToRemove,
	) -> Result<(), &'static str> {
		log!(debug, "XXXXXXXXXXXXXXXXXXXXXX ");
		for requestId in updates_to_remove_request_details.l2UpdatesToRemove.iter() {
			pending_updates::<T>::remove(requestId);
			log!(debug, "Aaaa {:?}", requestId);
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
