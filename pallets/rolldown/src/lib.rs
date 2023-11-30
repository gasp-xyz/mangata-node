#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Get, StorageVersion},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_std::collections::btree_map::BTreeMap;

use sp_std::{convert::TryInto, prelude::*};

use codec::alloc::string::{String, ToString};
use sp_runtime::serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use scale_info::prelude::format;

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
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			Self::end_dispute_period();
			T::DbWeight::get().reads_writes(10,10)
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
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize
	)]
	pub struct Requests {
		pub requests: BTreeMap<u128, Request>,
		pub lastProccessedRequestOnL1: u128,
		pub lastAcceptedRequestOnL1: u128,
	}
	
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Deserialize
	)]
	pub enum Request {
		pub L2UpdatesToRemove(UpdatesToRemoveRequestDetails),
		pub Deposit(DepositRequestDetails),
		pub RightsToUpdate(UpdateRightsRequestDetails),
		pub Withdraw(WithdrawRequestDetails),
	}
	
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode , TypeInfo, Default, Deserialize
	)]
	pub struct DepositRequestDetails {
		pub depositRecipient: String,
		pub tokenAddress: String,
		pub amount: u128,
	}
	
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize
	)]
	pub struct WithdrawRequestDetails {
		pub withdrawRecipient: String,
		pub tokenAddress: String,
		pub amount: u128,
	}
	
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize
	)]
	pub struct UpdateRightsRequestDetails {
		pub sequencer: String,
		pub readRights: u128,
		pub cancelRights: u128,
	}
	
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize
	)]
	pub struct UpdatesToRemoveRequestDetails {
		pub updates: Vec<u128>,
	}
	//L1 incomming request structs
	//L1 incomming request structs
	
	//L2 outgoing updates structs
	//L2 outgoing updates structs
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize
	)]
	pub enum Update {
		pub Cancel(Cancel),
		pub ProcessedUpdate(bool),
		pub Slash(Slash),
	}
	
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize
	)]
	pub struct Cancel {
		pub updater: String,
		pub canceler: String,
		pub lastProccessedRequestOnL1: u128,
		pub lastAcceptedRequestOnL1: u128,
		pub hash: String,
	}
	
	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize
	)]
	pub struct Slash {
		pub sequencerToBeSlashed: String,
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
	#[pallet::getter(fn get_pending_requests)]
	pub type PENDING_REQUESTS<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		(String, String),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_sequencer_rights)]
	pub type SEQUENCER_RIGHTS<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		String,
		SequencerRights,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_updates)]
	pub type PENDING_UPDATES<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u128,
		Update,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		OperationFailed
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		// sequencer read
		//fn update_l2_from_l1(origin: String, l1_pending_requests_json: &'static str, current_block_number: u32) {
		pub fn update_l2_from_l1(origin: OriginFor<T>, input: String, input_json: String) -> DispatchResultWithPostInfo {
			
			//check json length to prevent big data spam, maybe not necessary as it will be checked later and slashed
			let current_block_number= <frame_system::Pallet<T>>::block_number();
			let dispute_period_end = current_block_number + (DISPUTE_PERIOD_LENGTH as u32).into();
			//let sequencer = ensure_signed(origin)?;
		

			// //TBR IO showcase part
			// let mut input = String::new();
			// println!("Enter ORIGIN: ");
			// io::stdin()
			// 	.read_line(&mut input)
			// 	.expect("Failed to read line");
			// let origin = input.trim().to_string();
			// let mut input_json = String::new();
			// println!("Enter L1 pending requests JSON: ");
			// io::stdin()
			// 	.read_line(&mut input_json)
			// 	.expect("Failed to read line");
			// let l1_pending_requests_json = input_json.trim().to_string();
			// //TBR IO showcase part

			let origin = input.trim().to_string();
			let l1_pending_requests_json = input_json.trim().to_string();
		
			// // ensure sequencer has rights to update
			// let mut sequencer_rights_map = SEQUENCER_RIGHTS.lock().unwrap();
			if let Some(sequencer) = SEQUENCER_RIGHTS::<T>::get(&origin) {
				if sequencer.readRights == 0 {
					log!(info, "{:?} does not have sufficient readRights", origin);
					return Err(Error::<T>::OperationFailed.into());
				}
			} else {
				log!(info, "{:?} not a sequencer, CHEEKY BASTARD!", origin);
				return Err(Error::<T>::OperationFailed.into());
			}
		
			// // Decrease readRights by 1
			// if let Some(sequencer) = sequencer_rights_map.get_mut(&origin) {
			// 	sequencer.readRights -= 1;
			// }
			SEQUENCER_RIGHTS::<T>::mutate_exists(origin.clone(), |maybe_sequencer| {
				if let Some(ref mut sequencer) = maybe_sequencer {
					sequencer.readRights -= 1;
				}
			});
		
			// insert pending_requests
			// let mut pending_requests_map = PENDING_REQUESTS.lock().unwrap();
			// pending_requests_map.insert(dispute_period_end, (origin, l1_pending_requests_json));
			PENDING_REQUESTS::<T>::insert(dispute_period_end, (origin.clone(), l1_pending_requests_json));
		
			//TBR
			log!(info, "Pending Requests:");
			for (dispute_period_end, (origin, json)) in PENDING_REQUESTS::<T>::iter() {
				log!(info,
					"dispute period end: {:?}: Origin: {:?}, JSON: {:?}",
					dispute_period_end, origin, json
				);
			}
			// drop(pending_requests_map);
			// drop(sequencer_rights_map);
			//TBR
			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		//EXTRINSIC2 (who canceled, dispute_period_end(u32-blocknum)))
		pub fn cancel_requests_from_l1(origin: OriginFor<T>, input: String, input_2: String) -> DispatchResultWithPostInfo {
			let origin = input.trim().to_string();
			let requests_to_cancel: BlockNumberFor<T> = input_2.trim().to_string().parse::<u32>().unwrap().into();

			// remove pending requests update (sequencer read))
			PENDING_REQUESTS::<T>::mutate_exists(requests_to_cancel.clone(), |maybe_pending_requests_to_cancel| {
				if let Some(ref mut pending_requests_to_cancel) = maybe_pending_requests_to_cancel {
					// reduce sequencer cancel rights
					SEQUENCER_RIGHTS::<T>::mutate_exists(origin.clone(), |maybe_sequencer| {
						if let Some(ref mut sequencer) = maybe_sequencer {
							sequencer.readRights -= 1;
						}
					});

					// create hash of pending requests
					let hash_of_pending_request =
						Self::calculate_hash_of_pending_requests(pending_requests_to_cancel.1.as_str());
					//get last processed request on L1

					//deserialize json
					let pending_requests_json =
						// add SLASH if json is invalid to process
						Self::deserialize_json(pending_requests_to_cancel.1.as_str()).unwrap();
					// get last processed request on L1 and last accepted request on L1
					let last_processed_request_on_l1 = pending_requests_json.lastProccessedRequestOnL1;
					let last_accepted_request_on_l1 = pending_requests_json.lastAcceptedRequestOnL1;

					// create cancel request
					let cancel_request = Cancel {
						updater: pending_requests_to_cancel.0.clone(),
						canceler: origin,
						lastProccessedRequestOnL1: last_processed_request_on_l1,
						lastAcceptedRequestOnL1: last_accepted_request_on_l1,
						hash: hash_of_pending_request,
					};
					// add cancel request to pending updates
					l2_origin_updates_counter::<T>::put(Self::get_l2_origin_updates_counter() + 1);
					PENDING_UPDATES::<T>::insert(
						Self::get_l2_origin_updates_counter(),
						Update::Cancel(cancel_request),
					);
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
					log!(info, 
						"No pending requests co cancel at dispute period end {:?}",
						requests_to_cancel
					);
				}
			});
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

						// SLASH sequencer for bringing far past updates
						Self::process_node_slash(sequencer);
					} else {
						Self::process_requests(sequencer, &requests);
					}
				}
				Err(e) => {
					log!(info, "Error deserializing JSON: {:?}", e);
					// SLASH sequencer for invalid json
					Self::process_node_slash(sequencer);
				}
			}
		}
		// pending_requests_map.remove(unsafe { &CURRENT_BLOCK_NUMBER });
		PENDING_REQUESTS::<T>::remove(<frame_system::Pallet<T>>::block_number());
	}

	fn deserialize_json(json_str: &str) -> Result<Requests, serde_json::Error> {
		serde_json::from_str(json_str)
	}

	fn process_requests(sequencer: &String, requests: &Requests) {
		// check if not missing any request, not processing requests. This is double check, first should be done by sequencers and requests with missing request should be canceled
		for key in (requests.lastProccessedRequestOnL1 + 1_u128)..=requests.lastAcceptedRequestOnL1 {
			if let Some(_request) = requests.requests.get(&key) {
			} else {
				log!(info, "No request found for key: {:?}", key);
				// SLASH sequencer for missing request
				Self::process_node_slash(sequencer);
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
						}
						Request::Withdraw(withdraw_request_details) => {
							log!(info, "Withdraw: {:?}", withdraw_request_details);
							match Self::process_withdraw(withdraw_request_details) {
								Ok(_) => (),
								Err(_) => success = false,
							};
						}
						Request::RightsToUpdate(update_rights_request_details) => {
							log!(info, "RightsToUpdate: {:?}", update_rights_request_details);
							match Self::process_update_rights(update_rights_request_details) {
								Ok(_) => (),
								Err(_) => success = false,
							};
						}
	
						Request::L2UpdatesToRemove(updates_to_remove_request_details) => {
							log!(info, "L2UpdatesToRemove: {:?}", updates_to_remove_request_details);
							match Self::process_l2_updates_to_remove(updates_to_remove_request_details) {
								Ok(_) => (),
								Err(_) => success = false,
							};
						}
					}
					// return readRights to sequencer
					SEQUENCER_RIGHTS::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
						if let Some(ref mut sequencer) = maybe_sequencer {
							sequencer.readRights += 1;
						}
					});
					PENDING_UPDATES::<T>::insert(request_id, Update::ProcessedUpdate(success));
					// unsafe { last_processed_request_on_l2 = request_id };
					last_processed_request_on_l2::<T>::put(request_id);
				}
			} else {
				log!(info, "No request found for request_id: {:?}", request_id);
				// SLASH sequencer for missing request
				Self::process_node_slash(sequencer);
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
		// for ilustration purposes
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
	
		// burn tokes for user
	}
	
	fn process_update_rights(
		update_rights_request_details: &UpdateRightsRequestDetails,
	) -> Result<(), &'static str> {
		// check if math ok, not to negative values etc	
		// Insert or update sequencer rights
		// if let Some(sequencer) = sequencer_rights_map.get_mut(&update_rights_request_details.sequencer)
		// {
		// 	sequencer.readRights += sequencer.readRights;
		// 	// might be only readrights that are updated, possibility to simplify update rights, as if seq stake is low on l1, it will be removed from list completely, think later
		// 	sequencer.cancelRights += sequencer.cancelRights;
		// }
		SEQUENCER_RIGHTS::<T>::mutate_exists(update_rights_request_details.sequencer.clone(), |maybe_sequencer| {
			if let Some(ref mut sequencer) = maybe_sequencer {
				sequencer.readRights += sequencer.readRights;
				// might be only readrights that are updated, possibility to simplify update rights, as if seq stake is low on l1, it will be removed from list completely, think later
				sequencer.cancelRights += sequencer.cancelRights;
			}
		});
		log!(info, 
			"Rights update processed successfully for sequencer to plus: {:?}",
			update_rights_request_details
		);
		
		//additional checks
		Ok(())
	}
	
	fn process_l2_updates_to_remove(
		updates_to_remove_request_details: &UpdatesToRemoveRequestDetails,
	) -> Result<(), &'static str> {
		// let mut pending_updates_map = PENDING_UPDATES.lock().unwrap();
	
		// for request_id in updates_to_remove_request_details.updates.iter() {
		//     if let Some(_update) = pending_updates_map.get(&request_id) {
		//         pending_updates_map.remove(request_id);
		//     }
		// }
		// drop(pending_updates_map);
	
		//doesn't because of locks, solve later, will not be a problem on node
		log!(info, 
			"Update removal processed successfully, removed: {:?}",
			updates_to_remove_request_details
		);
		//additional checks
	
		Ok(())
	}

	fn process_node_slash(sequencer: &String) -> Result<(), &'static str> {
		// let mut sequencer_rights_map = SEQUENCER_RIGHTS.lock().unwrap();
		// if let Some(sequencer) = sequencer_rights_map.get_mut(sequencer) {
		// 	sequencer.readRights -= 1;
		// }
		SEQUENCER_RIGHTS::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
			if let Some(ref mut sequencer) = maybe_sequencer {
				sequencer.readRights -= 1;
			}
		});
		PENDING_UPDATES::<T>::insert(
			Self::get_l2_origin_updates_counter(),
			Update::Slash(Slash {
				sequencerToBeSlashed: sequencer.clone(),
			}),
		);
		log!(info, "Node SLASH processed successfully: {:?}", sequencer);
	
		Ok(())
	}

	fn calculate_keccak256_hash(input: &str) -> String {
		let mut hasher = Keccak256::new();
		hasher.update(input.as_bytes());
		let result = hasher.finalize();
	
		hex::encode(result)
	}
	
	fn calculate_hash_of_pending_requests(json_string_to_hash: &str) -> String {
		let mut hash = "0".to_string();
	
		let hash_of_pending_request = Self::calculate_keccak256_hash(json_string_to_hash);
		hash = "0x".to_string() + &hash_of_pending_request;
		log!(info, "Keccak256 Hash of PENDING_REQUESTS at {:?}", hash);
	
		hash
	}

	// for presentation purposes => json
	fn get_pending_updates_json() {
		// let pending_updates_map = PENDING_UPDATES.lock().unwrap();
		let mut updates_json = String::new();
		updates_json.push_str("{\n");

		for (request_id, update) in PENDING_UPDATES::<T>::iter() {
			updates_json.push_str(&format!("\t\"{}\": ", request_id));
			match update {
				Update::Cancel(cancel) => {
					updates_json.push_str(&serde_json::to_string(&cancel).unwrap());
				}
				Update::ProcessedUpdate(success) => {
					updates_json.push_str(&format!("{{ \"ProcessedUpdate\": {} }}", success));
				}
				Update::Slash(slash) => {
					updates_json.push_str(&serde_json::to_string(&slash).unwrap());
				}
			}
			updates_json.push_str(",\n");
		}

		updates_json.push_str("}\n");
		log!(info, "Pending Updates:\n{}", updates_json);
	}
}
