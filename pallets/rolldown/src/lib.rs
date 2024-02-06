#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use eth_api::L1UpdateRequest;
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, Get, StorageVersion},
	StorageHasher,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::traits::SaturatedConversion;
use sp_std::collections::btree_map::BTreeMap;

use codec::{
	alloc::string::{String, ToString},
	EncodeAsRef, WrapperTypeDecode,
};
use frame_support::traits::WithdrawReasons;
use mangata_support::traits::{
	AssetRegistryProviderTrait, RolldownProviderTrait, SequencerStakingProviderTrait,
};
use mangata_types::assets::L1Asset;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use scale_info::prelude::format;
use sha3::{Digest, Keccak256};
use sp_core::{H256, U256};
use sp_runtime::{
	serde::{Deserialize, Serialize},
	traits::Convert,
};
use sp_std::{convert::TryInto, prelude::*};

use alloy_primitives::address;
use alloy_sol_types::{sol, SolStruct, SolValue};

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

mod eth_api;

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
			// TODO: fix
			let pending_updates_u256_array = Self::get_pending_updates_as_u256_array();

			// PendingUpdatesU256Array::<T>::insert(n, pending_updates_u256_array.clone());
			// let hash_of_pending_updates_u256_array =
			// 	Self::calculate_hash_of_u256_array(pending_updates_u256_array);
			// HashPendingUpdatesU256Array::<T>::insert(n, hash_of_pending_updates_u256_array);
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
	pub struct PendingRequestStatus {
		pub status: bool,
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

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default)]
	pub struct L1ToL2Update {
		pub requests: Vec<(u128, MatRequest)>,
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

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo)]
	pub enum MatRequest {
		Deposit(DepositRequestDetailsMat),
		Withdraw(WithdrawRequestDetailsMat),
		L2UpdatesToRemove(UpdatesToRemoveRequestDetailsMat),
		CancelResolution(CancelResolutionRequestDetailsMat),
	}

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Deserialize,
	)]
	struct EthAddress(pub [u8; 20]);

	struct U256Wrapper(sp_core::U256);

	impl<'a> EncodeAsRef<'a, alloy_primitives::Address> for EthAddress {
		type RefType = &'a [u8; 20];
	}

	// impl<'a> EncodeAsRef<'a, alloy_primitives::Address> for U256Wrapper {
	// 	type RefType = &'a [u8; 32];
	// }

	// impl WrapperTypeDecode for U256Wrapper {
	// 	type Wrapped = alloy_primitives::U256;
	// }

	// impl From<alloy_primitives::U256> for U256Wrapper {
	// 	fn from(input: alloy_primitives::U256) -> Self {
	// 		Default::default()
	// 	}
	// }

	impl Into<alloy_primitives::U256> for U256Wrapper {
		fn into(self) -> alloy_primitives::U256 {
			alloy_primitives::U256::default()
		}
	}

	impl Into<alloy_primitives::Address> for EthAddress {
		fn into(self) -> alloy_primitives::Address {
			alloy_primitives::Address::from([0u8; 20])
		}
	}

	impl TypeInfo for Foo {
		type Identity = Self;

		fn type_info() -> scale_info::Type {
			scale_info::Type::builder()
				.path(scale_info::Path::new("Foo", module_path!()))
				.composite(
					scale_info::build::Fields::named().field(|f| {
						f.ty::<[u8; 20]>().name("depositRecipient").type_name("[u8;20]")
					}),
				)
		}
	}

	impl TypeInfo for Deposit {
		type Identity = Self;

		fn type_info() -> scale_info::Type {
			scale_info::Type::builder()
				.path(scale_info::Path::new("Deposit", module_path!()))
				.composite(
					scale_info::build::Fields::named()
						.field(|f| f.ty::<[u8; 20]>().name("depositRecipient").type_name("[u8;20]"))
						.field(|f| f.ty::<[u8; 20]>().name("tokenAddress").type_name("[u8;20]"))
						.field(|f| f.ty::<u128>().name("tokenAddress").type_name("u128")),
				)
		}
	}

	sol! {
		#[derive( Eq, Debug, PartialEq, Encode, Default)]
		struct Foo {
			#[codec(encoded_as = "EthAddress")]
			address depositRecipient;
			// #[codec(encoded_as = "U256Wrapper")]
			// uint256 bar;
		}

		#[derive(Debug)]
		struct Foo2 {
			address depositRecipient;
		}
	}

	// impl TypeInfo for Foo {
	//
	// }
	//
	sol! {

		// type U256 is uint256;
		// #[derive( Eq, Debug, PartialEq, Default, Encode)]
		// struct Foo {
		// 	#[codec(encoded_as = "[u8; 20]")]
		// 	address depositRecipient;
		// }

		// #[derive( Eq, Debug, PartialEq, Default, Encode)]
		// struct Foo {
		// 	uint256 depositRecipient;
		// }


		#[derive( Eq, Debug, PartialEq, Encode, Decode, Default)]
		struct Deposit {
			#[codec(encoded_as = "EthAddress")]
			address depositRecipient;
			#[codec(encoded_as = "EthAddress")]
			address tokenAddress;
			uint128 amount;
		}

		#[derive( Eq, Debug, PartialEq, Encode, Decode, Default)]
		struct Withdraw {
			#[codec(encoded_as = "EthAddress")]
			address withdrawRecipient;
			#[codec(encoded_as = "EthAddress")]
			address tokenAddress;
			uint128 amount;
		}

		#[derive( Eq, Debug, PartialEq, Encode, Decode, TypeInfo, Default)]
		struct L2UpdatesToRemove {
			uint128[] l2UpdatesToRemove;
		}

		#[derive( Eq, Debug, PartialEq, Encode, Decode, TypeInfo, Default)]
		struct CancelResolution {
			uint128 l2RequestId;
			bool cancelJustified;
		}

		#[derive( Eq, Debug, PartialEq, Encode, Decode, TypeInfo)]
		enum PendingRequestType{ DEPOSIT, WITHDRAWAL, CANCEL_RESOLUTION, L2_UPDATES_TO_REMOVE}

		#[derive( Eq, Debug, PartialEq, Encode, Decode)]
		struct L1Update {
		  PendingRequestType[] order;
		  Withdraw[] pendingWithdraws;
		  Deposit[] pendingDeposits;
		  CancelResolution[] pendingCancelResultions;
		  L2UpdatesToRemove[] pendingL2UpdatesToRemove;
		}


		// /// PENING REQUESTS TYPES (L1)
		// struct Withdraw {
		//     address withdrawRecipient;
		//     address tokenAddress;
		//     uint256 amount;
		// }
		//
		// struct L2UpdatesToRemove {
		//     uint256[] l2UpdatesToRemove;
		// }
		//
		// struct CancelResolution {
		//     uint256 l2RequestId;
		//     bool cancelJustified;
		// }
		//
		// enum PendingRequestType{ DEPOSIT, WITHDRAWAL, CANCEL_RESOLUTION, L2_UPDATES_TO_REMOVE}
		//
		// struct L1Update {
		//   PendingRequestType[] order;
		//   Withdraw[] pendingWithdraws;
		//   Deposit[] pendingDeposits;
		//   CancelResolution[] pendingCancelResultions;
		//   L2UpdatesToRemove[] pendingL2UpdatesToRemove;
		// }


		#[derive(
			Eq, Debug, PartialEq, Encode, Decode, TypeInfo, Default
		)]
		struct DepositRequestDetailsMat {
			bytes depositRecipient;
			bytes tokenAddress;
			uint128 amount;
		}

		#[derive(
			Eq, Debug, PartialEq, Encode, Decode, TypeInfo, Default
		)]
		struct WithdrawRequestDetailsMat {
			bytes withdrawRecipient;
			bytes tokenAddress;
			uint128 amount;
		}


		#[derive(
			Eq, Debug, PartialEq, Encode, Decode, TypeInfo, Default
		)]
		struct CancelResolutionRequestDetailsMat {
			uint128 l2RequestId;
			bool cancelJustified;
		}

		#[derive(
			Eq, Debug, PartialEq, Encode, Decode, TypeInfo, Default
		)]
		struct UpdatesToRemoveRequestDetailsMat {
			uint128[] updates;
		}

		enum UpdateType{ DEPOSIT, WITHDRAWAL, INDEX_UPDATE}

		struct StatusUpdate {
			UpdateType updateType;
			uint128 requestId;
			bool status;
		}

		struct L2ToL1Update {
			StatusUpdate[] updates;
			CancelEth[] cancels;
		}

		struct CancelEth {
			bytes32 updater;
			bytes32 canceler;
			uint128 lastProccessedRequestOnL1;
			uint128 lastAcceptedRequestOnL1;
			bytes32 hash;
		}
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
		pub lastProccessedRequestOnL1: U256,
		pub lastAcceptedRequestOnL1: U256,
		pub hash: H256,
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo)]
	pub enum PendingUpdate<AccountId> {
		Status(bool),
		Cancel(Cancel<AccountId>),
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
	#[pallet::getter(fn get_pending_updates_json)]
	pub type PendingUpdatesJson<T: Config> = StorageValue<_, String, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_hash_pending_updates_json)]
	pub type HashPendingUpdatesJson<T: Config> = StorageValue<_, String, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_updates_u256_array)]
	pub type PendingUpdatesU256Array<T: Config> =
		StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, Vec<U256>, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_hash_pending_updates_u256_array)]
	pub type HashPendingUpdatesU256Array<T: Config> =
		StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, U256, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_requests)]
	pub type PENDING_REQUESTS<T: Config> =
		StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, (T::AccountId, String), OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_requests_mat)]
	pub type PENDING_REQUESTS_MAT<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, (T::AccountId, eth_api::L1Update), OptionQuery>;

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

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_pending_updates_mat)]
	pub type PENDING_UPDATES_MAT<T: Config> =
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
		EmptyUpdate,
		AddressDeserializationFailure,
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
				SEQUENCER_RIGHTS::<T>::insert(
					s.clone(),
					SequencerRights {
						readRights: 1,
						cancelRights: (self.sequencers.len() - 1) as u128,
					},
				);
			}
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
			requests: eth_api::L1Update,
		) -> DispatchResultWithPostInfo {
			let sequencer = ensure_signed(origin)?;

			ensure!(!requests.order.is_empty(), Error::<T>::EmptyUpdate);

			// check json length to prevent big data spam, maybe not necessary as it will be checked later and slashed
			let current_block_number =
				<frame_system::Pallet<T>>::block_number().saturated_into::<u128>();
			let dispute_period_end: u128 = current_block_number + (DISPUTE_PERIOD_LENGTH as u128);

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
			PENDING_REQUESTS_MAT::<T>::insert(
				U256::from(dispute_period_end),
				(sequencer.clone(), requests.clone()),
			);

			let update: eth_api::eth::L1Update = requests.clone().into();
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
			// let requests_to_cancel: BlockNumberFor<T> =
			// 	input_2.trim().to_string().parse::<u32>().unwrap().into();

			// remove pending requests update (sequencer read))
			PENDING_REQUESTS_MAT::<T>::mutate_exists(
				requests_to_cancel.clone(),
				|maybe_pending_requests_to_cancel| {
					if let Some(ref mut pending_requests_to_cancel) =
						maybe_pending_requests_to_cancel
					{
						// reduce sequencer cancel rights
						SEQUENCER_RIGHTS::<T>::mutate_exists(canceler.clone(), |maybe_sequencer| {
							if let Some(ref mut sequencer) = maybe_sequencer {
								sequencer.cancelRights -= 1;
							}
						});

						// r.encode()

						// create hash of pending requests
						let hash_of_pending_request = Self::calculate_hash_of_pending_requests(
							pending_requests_to_cancel.1.clone(),
						);

						//get last processed request on L1
						//deserialize json

						// get last processed request on L1 and last accepted request on L1
						let last_processed_request_on_l1 =
							pending_requests_to_cancel.1.lastProccessedRequestOnL1;
						let last_accepted_request_on_l1 =
							pending_requests_to_cancel.1.lastAcceptedRequestOnL1;


						// create cancel request
						let cancel_request = Cancel {
							updater: pending_requests_to_cancel.0.clone(),
							canceler,
							lastProccessedRequestOnL1: last_processed_request_on_l1,
							lastAcceptedRequestOnL1: last_accepted_request_on_l1,
							hash: hash_of_pending_request,
						};
						// increase counter for updates originating on l2
						l2_origin_updates_counter::<T>::put(
							Self::get_l2_origin_updates_counter() + 1,
						);
						// add cancel request to pending updates
						PENDING_UPDATES_MAT::<T>::insert(
							U256::from(Self::get_l2_origin_updates_counter()),
							PendingUpdate::Cancel(cancel_request),
						);
						// remove whole l1l2 update (read) from pending requests
						PENDING_REQUESTS_MAT::<T>::remove(&requests_to_cancel);

						log!(info, "Pending Updates:");
						for (request_id, update) in PENDING_REQUESTS::<T>::iter() {
							log!(info, "request_id: {:?}:  {:?} ", request_id, update);
						}
						log!(info, "Pending requests:");
						for (request_id, update) in PENDING_UPDATES::<T>::iter() {
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

		// #[pallet::call_index(2)]
		// #[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		// //EXTRINSIC2 (who canceled, dispute_period_end(u32-blocknum)))
		// pub fn mat(origin: OriginFor<T>, f: Foo) -> DispatchResultWithPostInfo {
		// 	let canceler = ensure_signed(origin)?;
		// 	Ok(().into())
		// }

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
		// 			log!(info, "Deserialized struct: {:?}", deserialized);
		//
		// 			// Add a new variable of type Requests
		// 			let requests: Requests = deserialized;
		//
		// 			if requests.lastProccessedRequestOnL1 <
		// 				Self::get_prev_last_processed_request_on_l2()
		// 			{
		// 				log!(info, "lastProccessedRequestOnL1 is less than prev_last_processed_request_on_l2");
		// 			} else {
		// 				for request_id in (requests.lastProccessedRequestOnL1 + 1_u128)..=
		// 					requests.lastAcceptedRequestOnL1
		// 				{
		// 					if let Some(request) = requests.requests.get(&request_id) {
		// 						// ignore if already processed
		// 						if request_id > Self::get_last_processed_request_on_l2() {
		// 							log!(info, "EXECUTING: ");
		// 							let mut success = true;
		// 							match request {
		// 								Request::Deposit(deposit_request_details) => {
		// 									log!(info, "Deposit: {:?}", deposit_request_details);
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
		// 									log!(info, "Withdraw: {:?}", withdraw_request_details);
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
		// 						log!(info, "No request found for request_id: {:?}", request_id);
		// 					}
		// 				}
		//
		// 				log!(info, "Pending Updates:");
		// 				for (request_id, update) in PENDING_UPDATES::<T>::iter() {
		// 					log!(info, "request_id: {:?}:  {:?} ", request_id, update);
		// 				}
		// 			}
		// 		},
		// 		Err(e) => {
		// 			log!(info, "Error deserializing JSON: {:?}", e);
		// 		},
		// 	}
		// 	Ok(().into())
		// }
	}
}

impl<T: Config> Pallet<T> {
	// should run each block, check if dispute period ended, if yes, process pending requests
	fn end_dispute_period() {
		if let Some(pending_requests_to_process) = PENDING_REQUESTS_MAT::<T>::get(U256::from(
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
		)) {
			log!(info, "dispute end ",);

			let sequencer = &pending_requests_to_process.0;
			let requests = pending_requests_to_process.1.clone();

			if requests.lastProccessedRequestOnL1 <
				sp_core::U256::from(Self::get_prev_last_processed_request_on_l2())
			{
				log!(
					info,
					"lastProccessedRequestOnL1 is less than prev_last_processed_request_on_l2"
				);

				// TODO: SLASH sequencer for bringing unnecessary past requests, to be tested
				// Self::slash(sequencer);
			}

			// return readRights to sequencer
			SEQUENCER_RIGHTS::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
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
		PENDING_REQUESTS_MAT::<T>::remove(U256::from(
			<frame_system::Pallet<T>>::block_number().saturated_into::<u128>(),
		));
		// TODO: prev_last_processed_request_on_l2 update goes here?
	}

	fn process_requests(sequencer: &T::AccountId, update: eth_api::L1Update) {
		// TODO: check if not missing any request, not processing requests. This is double check, first should be done by sequencers and requests with missing request should be canceled
		// for key in (requests.lastProccessedRequestOnL1 + 1_u128)..=requests.lastAcceptedRequestOnL1
		// {
		// 	if let Some(_request) = requests.requests.get(&key) {
		// 	} else {
		// 		log!(info, "No request found for key: {:?}", key);
		// 		// SLASH sequencer for missing request
		// 		Self::slash(sequencer);
		// 		return
		// 	}
		// }

		for (request_id, request_details) in update.into_requests() {
			if PENDING_UPDATES_MAT::<T>::contains_key(request_id) {
				continue
			}

			match request_details {
				eth_api::L1UpdateRequest::Deposit(deposit) => PENDING_UPDATES_MAT::<T>::insert(
					request_id,
					PendingUpdate::Status(Self::process_deposit(&deposit).is_ok()),
				),
				eth_api::L1UpdateRequest::Withdraw(withdraw) => PENDING_UPDATES_MAT::<T>::insert(
					request_id,
					PendingUpdate::Status(Self::process_withdraw(&withdraw).is_ok()),
				),
				// eth_api::L1UpdateRequest::Cancel(cancel) => PENDING_UPDATES_MAT::insert(
				// 	request_id,
				// 	PendingUpdate::StatusUpdate(Self::process_cancel_resolution(&cancel).is_ok()),
				// ),
				eth_api::L1UpdateRequest::Remove(remove) => PENDING_UPDATES_MAT::<T>::insert(
					request_id,
					PendingUpdate::Status(Self::process_l2_updates_to_remove(&remove).is_ok()),
				),
				_ => {},
			};
			// if success, increase last_processed_request_on_l2
			let request_id: u128 = request_id.try_into().unwrap();
			last_processed_request_on_l2::<T>::put(request_id);
		}
	}

	fn process_deposit(deposit_request_details: &eth_api::Deposit) -> Result<(), &'static str> {
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
				.or(Err("Failed to create L1 Asset"))?,
		};
		log!(info, "Deposit processed successfully: {:?}", deposit_request_details);

		// ADD tokens: mint tokens for user
		T::Tokens::mint(
			asset_id,
			&account,
			amount.try_into().map_err(|_| "u128 to Balance failed")?,
		)?;
		Ok(())
	}

	fn process_withdraw(withdraw_request_details: &eth_api::Withdraw) -> Result<(), &'static str> {
		// fail will occur if user has not enough balance
		let amount: u128 = withdraw_request_details
			.amount
			.try_into()
			.map_err(|_| "u128 to Balance failed")?;
		let account: T::AccountId =
			T::AddressConverter::convert(withdraw_request_details.depositRecipient);

		let eth_asset = L1Asset::Ethereum(withdraw_request_details.tokenAddress);
		let asset_id = T::AssetRegistryProvider::get_l1_asset_id(eth_asset.clone())
			.ok_or("L1AssetNotFound")?;

		T::Tokens::ensure_can_withdraw(
			asset_id.into(),
			&account,
			amount.try_into().map_err(|_| "u128 to Balance failed")?,
			WithdrawReasons::all(),
			Default::default(),
		)
		.or(Err("NotEnoughAssets"))?;

		// burn tokes for user
		T::Tokens::burn_and_settle(
			asset_id,
			&account,
			amount.try_into().map_err(|_| "u128 to Balance failed")?,
		)?;

		Ok(())
	}

	fn process_cancel_resolution(
		cancel_resolution_request_details: &CancelResolutionRequestDetailsMat,
	) -> Result<(), &'static str> {
		let cancel_request_id = cancel_resolution_request_details.l2RequestId;
		let cancel_justified = cancel_resolution_request_details.cancelJustified;
		let cancel_update: Cancel<T::AccountId> = match PENDING_UPDATES::<T>::get(cancel_request_id)
		{
			Some(update) =>
				if let Update::Cancel(cancel) = update {
					cancel
				} else {
					return Err("Invalid update type")
				},
			None => return Err("Cancel update not found"),
		};
		let updater = cancel_update.updater;
		let canceler = cancel_update.canceler;
		let to_be_slashed = if cancel_justified { updater.clone() } else { canceler.clone() };

		// return rights to canceler and updater
		// only if active sequencer
		SEQUENCER_RIGHTS::<T>::mutate_exists(updater.clone(), |maybe_sequencer| {
			match maybe_sequencer {
				&mut Some(ref mut sequencer)
					if T::SequencerStakingProvider::is_active_sequencer(updater) =>
				{
					sequencer.readRights += 1;
				},
				_ => {},
			}
		});
		SEQUENCER_RIGHTS::<T>::mutate_exists(canceler.clone(), |maybe_sequencer| {
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

		log!(
			info,
			"Cancel resolution processed successfully: {:?}",
			cancel_resolution_request_details
		);

		//additional checks
		Ok(())
	}

	fn process_l2_updates_to_remove(
		updates_to_remove_request_details: &eth_api::L2UpdatesToRemove,
	) -> Result<(), &'static str> {
		for requestId in updates_to_remove_request_details.l2UpdatesToRemove.iter() {
			PENDING_UPDATES_MAT::<T>::remove(requestId);
		}

		log!(
			info,
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

	fn calculate_hash_of_pending_requests(input: eth_api::L1Update) -> H256 {
		let mut digest = Keccak256::new();

		//TODO: check hash
		// for (_, r) in input.requests.iter() {
		// 	match r {
		// 		MatRequest::Deposit(r) => digest.update(r.abi_encode()),
		// 		MatRequest::Withdraw(r) => digest.update(r.abi_encode()),
		// 		MatRequest::CancelResolution(r) => digest.update(r.abi_encode()),
		// 		MatRequest::L2UpdatesToRemove(r) => digest.update(r.abi_encode()),
		// 	};
		// }

		let array: [u8; 32] = digest.finalize().into();
		H256::from(array)
	}

	fn get_pending_updates_as_u256_array() -> eth_api::eth::L2Update {
		let mut update = eth_api::eth::L2Update { results: Vec::new(), cancles: Vec::new() };

		for (request_id, req) in PENDING_UPDATES_MAT::<T>::iter() {
			match req {
				PendingUpdate::Status(status) => update.results.push(eth_api::eth::RequestResult {
					// TODO: proper U256 conversion
					requestId: alloy_primitives::U256::from(0),
					status,
				}),
				PendingUpdate::Cancel(status) => unimplemented!(),
			};
		}
		update
	}

	fn handle_sequencer_deactivation(deactivated_sequencer: T::AccountId) {
		// lower sequencer count
		sequencer_count::<T>::put(Self::get_sequencer_count() - 1);
		// remove all rights of deactivated sequencer
		SEQUENCER_RIGHTS::<T>::remove(deactivated_sequencer.clone());
		// remove 1 cancel right of all sequencers
		for (sequencer, sequencer_rights) in SEQUENCER_RIGHTS::<T>::iter() {
			if sequencer_rights.cancelRights > 0 {
				SEQUENCER_RIGHTS::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
					if let Some(ref mut sequencer) = maybe_sequencer {
						sequencer.cancelRights -= RIGHTS_MULTIPLIER;
					}
				});
			}
		}
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

	pub fn pending_updates_proof() -> sp_core::H256 {
		unimplemented!()
	}

	pub fn pending_updates_eth_encoded() -> Vec<u8> {
		let update = Pallet::<T>::get_pending_updates_as_u256_array();
		update.abi_encode()
	}
}

impl<T: Config> RolldownProviderTrait<AccountIdOf<T>> for Pallet<T> {
	fn new_sequencer_active(sequencer: AccountIdOf<T>) {
		// raise sequencer count
		sequencer_count::<T>::put(Self::get_sequencer_count() + 1);
		// add rights to new sequencer
		SEQUENCER_RIGHTS::<T>::insert(
			sequencer.clone(),
			SequencerRights {
				readRights: RIGHTS_MULTIPLIER,
				cancelRights: RIGHTS_MULTIPLIER * (sequencer_count::<T>::get() - 1),
			},
		);

		// add 1 cancel right of all sequencers
		for (sequencer, sequencer_rights) in SEQUENCER_RIGHTS::<T>::iter() {
			if sequencer_rights.cancelRights > 0 {
				SEQUENCER_RIGHTS::<T>::mutate_exists(sequencer.clone(), |maybe_sequencer| {
					if let Some(ref mut sequencer) = maybe_sequencer {
						sequencer.cancelRights += RIGHTS_MULTIPLIER;
					}
				});
			}
		}
	}
}
