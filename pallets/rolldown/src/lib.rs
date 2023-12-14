#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	StorageHasher,
	traits::{Get, StorageVersion, tokens::currency::MultiTokenCurrency},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::traits::BlakeTwo256;
use sp_std::collections::btree_map::BTreeMap;

use sp_std::{convert::TryInto, prelude::*};
use mangata_support::traits::SequencerStakingProviderTrait;
use codec::alloc::string::{String, ToString};
use scale_info::prelude::format;
use sha3::{Digest, Keccak256};
use sp_core::U256;
use sp_runtime::{
	serde::{Deserialize, Serialize},
	traits::TryConvert,
};
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};

pub type BalanceOf<T> = <<T as Config>::Tokens as MultiTokenCurrency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;
	
const DISPUTE_PERIOD_LENGTH: u128 = 5;

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
impl TryConvert<String, sp_runtime::AccountId32>
	for EthereumAddressConverter<sp_runtime::AccountId32>
{
	fn try_convert(value: String) -> Result<sp_runtime::AccountId32, String> {
		let eth_addr: [u8; 20] = array_bytes::hex2array(value.clone()).or(Err(value))?;
		Ok(Blake2_256::hash(eth_addr.as_ref()).into())
	}
}

#[cfg(test)]
mod tests;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {

	use sp_runtime::traits::TryConvert;

	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			Self::end_dispute_period();
			T::DbWeight::get().reads_writes(20, 20)
		}

		fn on_finalize(n: BlockNumberFor<T>) {
			let pending_updates_u256_array = Self::get_pending_updates_as_u256_array();
			PendingUpdatesU256Array::<T>::insert(n, pending_updates_u256_array.clone());
			let hash_of_pending_updates_u256_array =
				Self::calculate_hash_of_u256_array(pending_updates_u256_array);
			HashPendingUpdatesU256Array::<T>::insert(n,hash_of_pending_updates_u256_array);
		}
	}

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo, Default,
	)]
	pub struct SequencerRights {
		pub readRights: u128,
		pub cancelRights: u128,
	}

	//L1 incomming request structs
	//L1 incomming request structs
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize,
	)]
	pub struct Requests {
		pub requests: BTreeMap<u128, Request>,
		pub lastProccessedRequestOnL1: u128,
		pub lastAcceptedRequestOnL1: u128,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Deserialize)]
	pub enum Request {
		Deposit(DepositRequestDetails),
		Withdraw(WithdrawRequestDetails),
		L2UpdatesToRemove(UpdatesToRemoveRequestDetails),
		CancelResolution(CancelResolutionRequestDetails),
	}

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize,
	)]
	pub struct DepositRequestDetails {
		pub depositRecipient: String,
		pub tokenAddress: String,
		pub amount: u128,
	}

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize,
	)]
	pub struct WithdrawRequestDetails {
		pub withdrawRecipient: String,
		pub tokenAddress: String,
		pub amount: u128,
	}

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize,
	)]
	pub struct CancelResolutionRequestDetails {
		pub l2RequestId: u128,
		pub cancelJustified: bool,
	}

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize,
	)]
	pub struct UpdatesToRemoveRequestDetails {
		pub updates: Vec<u128>,
	}
	//L1 incomming request structs
	//L1 incomming request structs

	//L2 outgoing updates structs
	//L2 outgoing updates structs
	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
	pub enum Update<AccountId> {
		DepositUpdate(bool),
		WithdrawUpdate(bool),
		Cancel(Cancel<AccountId>),
		ProcessedOnlyInfoUpdate(bool),
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
	pub struct Cancel<AccountId> {
		pub updater: AccountId,
		pub canceler: AccountId,
		pub lastProccessedRequestOnL1: u128,
		pub lastAcceptedRequestOnL1: u128,
		pub hash: U256,
	}

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
	#[pallet::getter(fn get_pending_updates_json)]
	pub type PendingUpdatesJson<T: Config> = StorageValue<_, String, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_hash_pending_updates_json)]
	pub type HashPendingUpdatesJson<T: Config> = StorageValue<_, String, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_updates_u256_array)]
	pub type PendingUpdatesU256Array<T: Config> = StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, Vec<U256>, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_hash_pending_updates_u256_array)]
	pub type HashPendingUpdatesU256Array<T: Config> = StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, U256, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_requests)]
	pub type PENDING_REQUESTS<T: Config> =
		StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, (T::AccountId, String), OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_sequencer_rights)]
	pub type SEQUENCER_RIGHTS<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SequencerRights, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_updates)]
	pub type PENDING_UPDATES<T: Config> =
		StorageMap<_, Blake2_128Concat, u128, Update<T::AccountId>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		OperationFailed,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AddressConverter: TryConvert<String, Self::AccountId>;
		type SequencerStakingProvider: SequencerStakingProviderTrait<Self::AccountId, BalanceOf<Self>>;
		// Dummy so that we can have the BalanceOf type here for the SequencerStakingProviderTrait
		type Tokens: MultiTokenCurrency<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>
			+ MultiTokenCurrencyExtended<Self::AccountId>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		// sequencer read
		//fn update_l2_from_l1(origin: String, l1_pending_requests_json: &'static str, current_block_number: u32) {
		pub fn update_l2_from_l1(
			origin: OriginFor<T>,
			input_json: String,
		) -> DispatchResultWithPostInfo {
			let sequencer = ensure_signed(origin)?;
			// check json length to prevent big data spam, maybe not necessary as it will be checked later and slashed
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let dispute_period_end = current_block_number + (DISPUTE_PERIOD_LENGTH as u32).into();
						
			let l1_pending_requests_json = input_json.trim().to_string();

			// ensure sequencer has rights to update
			if let Some(sequencer) = SEQUENCER_RIGHTS::<T>::get(&sequencer) {
				if sequencer.readRights == 0 {
					log!(info, "{:?} does not have sufficient readRights", sequencer);
					return Err(Error::<T>::OperationFailed.into())
				}
			} else {
				log!(info, "{:?} not a sequencer, CHEEKY BASTARD!", sequencer);
				return Err(Error::<T>::OperationFailed.into())
			}

			// // Decrease readRights by 1
			SEQUENCER_RIGHTS::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
				if let Some(ref mut sequencer) = maybe_sequencer {
					sequencer.readRights -= 1;
				}
			});

			// insert pending_requests
			PENDING_REQUESTS::<T>::insert(
				dispute_period_end,
				(sequencer.clone(), l1_pending_requests_json),
			);

			
			log!(info, "Pending Requests:");
			for (dispute_period_end, (origin, json)) in PENDING_REQUESTS::<T>::iter() {
				log!(
					info,
					"dispute period end: {:?}: Origin: {:?}, JSON: {:?}",
					dispute_period_end,
					origin,
					json
				);
			}
			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		//EXTRINSIC2 (who canceled, dispute_period_end(u32-blocknum)))
		pub fn cancel_requests_from_l1(
			origin: OriginFor<T>,
			input_2: String,
		) -> DispatchResultWithPostInfo {
			let canceler = ensure_signed(origin)?;
			let requests_to_cancel: BlockNumberFor<T> =
				input_2.trim().to_string().parse::<u32>().unwrap().into();

			// remove pending requests update (sequencer read))
			PENDING_REQUESTS::<T>::mutate_exists(
				requests_to_cancel.clone(),
				|maybe_pending_requests_to_cancel| {
					if let Some(ref mut pending_requests_to_cancel) =
						maybe_pending_requests_to_cancel
					{
						// reduce sequencer cancel rights
						SEQUENCER_RIGHTS::<T>::mutate_exists(canceler.clone(), |maybe_sequencer| {
							if let Some(ref mut sequencer) = maybe_sequencer {
								sequencer.readRights -= 1;
							}
						});

						// create hash of pending requests
						let hash_of_pending_request = Self::calculate_hash_of_pending_requests(
							pending_requests_to_cancel.1.as_str(),
						);

						//get last processed request on L1
						//deserialize json
						let pending_requests_json =
							// add SLASH if json is invalid to process
							Self::deserialize_json(pending_requests_to_cancel.1.as_str()).unwrap();
						// get last processed request on L1 and last accepted request on L1
						let last_processed_request_on_l1 =
							pending_requests_json.lastProccessedRequestOnL1;
						let last_accepted_request_on_l1 =
							pending_requests_json.lastAcceptedRequestOnL1;

						// create cancel request
						let cancel_request = Cancel {
							updater: pending_requests_to_cancel.0.clone(),
							canceler: canceler,
							lastProccessedRequestOnL1: last_processed_request_on_l1,
							lastAcceptedRequestOnL1: last_accepted_request_on_l1,
							hash: hash_of_pending_request,
						};
						// increase counter for updates originating on l2
						l2_origin_updates_counter::<T>::put(
							Self::get_l2_origin_updates_counter() + 1,
						);
						// add cancel request to pending updates
						PENDING_UPDATES::<T>::insert(
							Self::get_l2_origin_updates_counter(),
							Update::Cancel(cancel_request),
						);
						// remove whole l1l2 update (read) from pending requests
						PENDING_REQUESTS::<T>::remove(&requests_to_cancel);

						log!(info, "Pending Updates:");
						for (request_id, update) in PENDING_REQUESTS::<T>::iter() {
							log!(info, "request_id: {:?}:  {:?} ", request_id, update);
						}
						log!(info, "Pending requests:");
						for (request_id, update) in PENDING_REQUESTS::<T>::iter() {
							log!(info, "request_id: {:?}:  {:?} ", request_id, update);
						}
					} else {
						log!(
							info,
							"No pending requests co cancel at dispute period end {:?}",
							requests_to_cancel
						);
					}
				},
			);
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	// should run each block, check if dispute period ended, if yes, process pending requests
	fn end_dispute_period() {
		// get pending requests at blocknumber
		if let Some(pending_requests_to_process) =
			PENDING_REQUESTS::<T>::get(<frame_system::Pallet<T>>::block_number())
		{
			log!(info, "dispute end ",);

			let sequencer = &pending_requests_to_process.0;
			let pending_requests_json = &pending_requests_to_process.1.as_str();

			// deserialize json
			match Self::deserialize_json(pending_requests_json) {
				Ok(deserialized) => {
					log!(info, "Deserialized struct: {:?}", deserialized);

					// Add a new variable of type Requests
					let requests: Requests = deserialized;

					// Iterate over requests from lastProccessedRequestOnL1 to lastAcceptedRequestOnL1

					if requests.lastProccessedRequestOnL1 < Self::get_prev_last_processed_request_on_l2()
					{
						log!(info, 
							"lastProccessedRequestOnL1 is less than prev_last_processed_request_on_l2"
						);

						// SLASH sequencer for bringing unnecessary past requests, to be tested
						Self::slash(sequencer);
					} else {
						Self::process_requests(sequencer, &requests);
					}
				}
				Err(e) => {
					log!(info, "Error deserializing JSON: {:?}", e);
					// SLASH sequencer for invalid json
					Self::slash(sequencer);
				}
			}
		}
		PENDING_REQUESTS::<T>::remove(<frame_system::Pallet<T>>::block_number());
	}

	fn deserialize_json(json_str: &str) -> Result<Requests, serde_json::Error> {
		serde_json::from_str(json_str)
	}

	fn process_requests(sequencer: &T::AccountId, requests: &Requests) {
		// check if not missing any request, not processing requests. This is double check, first should be done by sequencers and requests with missing request should be canceled
		for key in (requests.lastProccessedRequestOnL1 + 1_u128)..=requests.lastAcceptedRequestOnL1 {
			if let Some(_request) = requests.requests.get(&key) {
			} else {
				log!(info, "No request found for key: {:?}", key);
				// SLASH sequencer for missing request
				Self::slash(sequencer);
				return;
			}
		}
		// process requests and checks if all requests are present from starting to ending one
		for request_id in
			(requests.lastProccessedRequestOnL1 + 1_u128)..=requests.lastAcceptedRequestOnL1
		{
			if let Some(request) = requests.requests.get(&request_id) {
				// ignore if already processed
				if request_id > Self::get_last_processed_request_on_l2() {
					log!(info, "EXECUTING: ");
					let mut success = true;
					match request {
						Request::Deposit(deposit_request_details) => {
							log!(info, "Deposit: {:?}", deposit_request_details);
							match Self::process_deposit(deposit_request_details) {
								Ok(_) => (),
								Err(_) => success = false,
							};
							PENDING_UPDATES::<T>::insert(request_id, Update::DepositUpdate(success));
						}
						Request::Withdraw(withdraw_request_details) => {
							log!(info, "Withdraw: {:?}", withdraw_request_details);
							match Self::process_withdraw(withdraw_request_details) {
								Ok(_) => (),
								Err(_) => success = false,
							};
							PENDING_UPDATES::<T>::insert(request_id, Update::WithdrawUpdate(success));
						}
						Request::CancelResolution(cancel_resolution_request_details) => {
							log!(info, "CancelResolution: {:?}", cancel_resolution_request_details);
							match Self::process_cancel_resolution(cancel_resolution_request_details) {
								Ok(_) => (),
								Err(_) => success = false,
							};
							PENDING_UPDATES::<T>::insert(request_id, Update::ProcessedOnlyInfoUpdate(success));
						}
	
						Request::L2UpdatesToRemove(updates_to_remove_request_details) => {
							log!(info, "L2UpdatesToRemove: {:?}", updates_to_remove_request_details);
							match Self::process_l2_updates_to_remove(updates_to_remove_request_details) {
								Ok(_) => (),
								Err(_) => success = false,
							};
							PENDING_UPDATES::<T>::insert(request_id, Update::ProcessedOnlyInfoUpdate(success));
						}
					}
					
					// return readRights to sequencer
					SEQUENCER_RIGHTS::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
						match maybe_sequencer{
							&mut Some(ref mut sequencer_rights) if T::SequencerStakingProvider::is_active_sequencer(sequencer.clone()) => {
								sequencer_rights.readRights += 1;
							},
							_ => {},
						}
					});
					last_processed_request_on_l2::<T>::put(request_id);
				}
			} else {
				log!(info, "No request found for request_id: {:?}", request_id);
				// SLASH sequencer for missing request
				Self::slash(sequencer);
				return;
			}
		}
		//TBR
		log!(info, "Pending Updates:");
		for (request_id, update) in PENDING_UPDATES::<T>::iter() {
			log!(info, "request_id: {:?}:  {:?} ", request_id, update);
		}
		//TBR
	}

	fn process_deposit(deposit_request_details: &DepositRequestDetails) -> Result<(), &'static str> {
		let account : T::AccountId = Self::eth_to_dot_address(deposit_request_details.depositRecipient.clone())?;

		// check ferried
		// check if token exists, if not create one
		log!(info, 
			"Deposit processed successfully: {:?}",
			deposit_request_details
		);
		
		// tokens: mint tokens for user
		Ok(())
	}
	
	fn process_withdraw(withdraw_request_details: &WithdrawRequestDetails) -> Result<(), &'static str> {
		
		// fail will occur if user has not enough balance
	
		// if user has enought balance
		if withdraw_request_details.amount > 0 {
			// Successful deposit handling logic goes here
			log!(info, 
				"Withdraw processed successfully: {:?}",
				withdraw_request_details
			);
			Ok(())
		} else {
			// Failed deposit handling logic goes here
			log!(info, "Withdraw handling failed: Not enough balance");
			Err("Not enough balance")
		}
	
		// <T as Config>::Currency::ensure_can_withdraw(
		// 	sold_asset_id.into(),
		// 	sender,
		// 	total_fees,
		// 	WithdrawReasons::all(),
		// 	// Does not fail due to earlier ensure
		// 	Default::default(),
		// )
		// .or(Err(Error::<T>::NotEnoughAssets))?;

		// burn tokes for user
	}
	
	fn process_cancel_resolution(
		cancel_resolution_request_details: &CancelResolutionRequestDetails,
	) -> Result<(), &'static str> {
		let cancel_request_id = cancel_resolution_request_details.l2RequestId;
		let cancel_justified = cancel_resolution_request_details.cancelJustified;
		let cancel_update: Cancel<T::AccountId> = match PENDING_UPDATES::<T>::get(cancel_request_id) {
			Some(update) => {
				if let Update::Cancel(cancel) = update {
					cancel
				} else {
					return Err("Invalid update type");
				}
			}
			None => return Err("Cancel update not found"),
		};
		let updater = cancel_update.updater;
		let canceler = cancel_update.canceler;
		let to_be_slashed = if cancel_justified {
			updater.clone()
		} else {
			canceler.clone()
		};

		
		// return rights to canceler and updater
		// only if active sequencer
		SEQUENCER_RIGHTS::<T>::mutate_exists(updater.clone(), |maybe_sequencer| {
			match maybe_sequencer{
				&mut Some(ref mut sequencer) if T::SequencerStakingProvider::is_active_sequencer(updater) => {
					sequencer.readRights += 1;
				},
				_ => {},
			}
		});
		SEQUENCER_RIGHTS::<T>::mutate_exists(canceler.clone(), |maybe_sequencer| {
			match maybe_sequencer{
				&mut Some(ref mut sequencer) if T::SequencerStakingProvider::is_active_sequencer(canceler) => {
					sequencer.cancelRights += 1;
				},
				_ => {},
			}
		});

		// slash is after adding rights, since slash can reduce stake below required level and remove all rights
		Self::slash(&to_be_slashed);

		log!(info, 
			"Cancel resolution processed successfully: {:?}",
			cancel_resolution_request_details
		);
		
		//additional checks
		Ok(())
	}
	
	fn process_l2_updates_to_remove(
		updates_to_remove_request_details: &UpdatesToRemoveRequestDetails,
	) -> Result<(), &'static str> {
		
		for requestId in updates_to_remove_request_details.updates.iter() {
			PENDING_UPDATES::<T>::remove(requestId);
		}

		log!(info, 
			"Update removal processed successfully, removed: {:?}",
			updates_to_remove_request_details
		);
		//additional checks
	
		Ok(())
	}

	fn slash(sequencer: &T::AccountId) -> Result<(), &'static str> {
		// check if sequencer is active
		let is_active_sequencer_before = T::SequencerStakingProvider::is_active_sequencer(sequencer.clone());
		// slash sequencer
		T::SequencerStakingProvider::slash_sequencer(sequencer.clone())?;
		// check if sequencer is active
		let is_active_sequencer_after = T::SequencerStakingProvider::is_active_sequencer(sequencer.clone());

		// if sequencer was active and is not active anymore, remove rights
		if is_active_sequencer_before && !is_active_sequencer_after {
			SEQUENCER_RIGHTS::<T>::remove(sequencer.clone());
		}
		
		// remove 1 cancel right of all sequencers
		for sequencer in SEQUENCER_RIGHTS::<T>::iter() {
			SEQUENCER_RIGHTS::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
				if let Some(ref mut sequencer) = maybe_sequencer {
					sequencer.cancelRights -= 1;
				}
			});
		}
		
		log!(info, "SLASH for: {:?}", sequencer);
	
		Ok(())
	}

	fn calculate_keccak256_hash_u256(input: &str) -> U256 {
		let mut hasher = Keccak256::new();
		hasher.update(input.as_bytes());
		let result = hasher.finalize();
	
		U256::from(&result[..])
	}

	fn calculate_keccak256_hash(input: &str) -> String {
		let mut hasher = Keccak256::new();
		hasher.update(input.as_bytes());
		let result = hasher.finalize();
	
		hex::encode(result)
	}
	
	fn calculate_hash_of_pending_requests(json_string_to_hash: &str) -> U256 {
	
		let hash_of_pending_request = Self::calculate_keccak256_hash_u256(json_string_to_hash);
		log!(info, "Keccak256 Hash of PENDING_REQUESTS at {:#?}", hash_of_pending_request);
	
		hash_of_pending_request
	}

	fn get_pending_updates_as_u256_array() -> Vec<U256> {
		let mut updates: Vec<U256> = Vec::<U256>::new();

		for (request_id, update) in PENDING_UPDATES::<T>::iter() {
			match update {
				Update::DepositUpdate(success) => {
					updates.push(1.into());
					updates.push(request_id.into());
					if success {
						updates.push(1.into());
					} else {
						updates.push(0.into());
					}
				}
				Update::WithdrawUpdate(success) => {
					updates.push(2.into());
					updates.push(request_id.into());
					if success {
						updates.push(1.into());
					} else {
						updates.push(0.into());
					}
				}
				Update::Cancel(cancel) => {
					updates.push(3.into());
					updates.push(request_id.into());
					updates.push(cancel.lastProccessedRequestOnL1.into());
					updates.push(cancel.lastAcceptedRequestOnL1.into());
					updates.push(cancel.hash);
				}
				Update::ProcessedOnlyInfoUpdate(success) => {
					updates.push(4.into());
					updates.push(request_id.into());
					if success {
						updates.push(1.into());
					} else {
						updates.push(0.into());
					}
				}
				_ => {
				}
			}
		}

		log!(info, "Pending Updates: {:?}", updates);
		updates
	}

	fn calculate_hash_of_u256_array(u256_vec: Vec<U256>) -> U256 {
		let mut hasher = Keccak256::new();
		let mut byte_array: [u8; 32] = Default::default(); 
		for u in u256_vec.iter() {
			u.to_big_endian(&mut byte_array[..]);
			hasher.update(&byte_array[..]);
		}
		let result = hasher.finalize();
		U256::from(&result[..])
	}
	fn eth_to_dot_address(eth_addr: String) -> Result<T::AccountId, &'static str> {
        T::AddressConverter::try_convert(eth_addr).or(Err("Cannot convert address"))
	}
}
