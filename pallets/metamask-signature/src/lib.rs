#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, traits::Get, BoundedVec};
pub use pallet::*;
use sp_std::{
	convert::{TryFrom, TryInto},
	prelude::*,
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::alloc::string::{String, ToString};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type StringLimit: Get<u32>;

		#[pallet::constant]
		type UrlStringLimit: Get<u32>;
	}

	#[pallet::storage]
	pub type Name<T: Config> = StorageValue<_, BoundedVec<u8, T::StringLimit>, ValueQuery>;

	#[pallet::storage]
	pub type Version<T: Config> = StorageValue<_, BoundedVec<u8, T::StringLimit>, ValueQuery>;

	#[pallet::storage]
	pub type ChainId<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type DecodeUrl<T: Config> = StorageValue<_, BoundedVec<u8, T::UrlStringLimit>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MetadataUpdated {
			name: Option<BoundedVec<u8, T::StringLimit>>,
			version: Option<BoundedVec<u8, T::StringLimit>>,
			chain_id: Option<u64>,
			decode_url: Option<BoundedVec<u8, T::UrlStringLimit>>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// there should be some updates
		NothingToUpdate,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn update(
			origin: OriginFor<T>,
			name: Option<BoundedVec<u8, T::StringLimit>>,
			version: Option<BoundedVec<u8, T::StringLimit>>,
			chain_id: Option<u64>,
			decode_url: Option<BoundedVec<u8, T::UrlStringLimit>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				name.is_some() || version.is_some() || chain_id.is_some() || decode_url.is_some(),
				Error::<T>::NothingToUpdate
			);

			let mut new_name: Option<BoundedVec<u8, T::StringLimit>> = None;
			let mut new_version: Option<BoundedVec<u8, T::StringLimit>> = None;
			let mut new_chain_id: Option<u64> = None;
			let mut new_decode_url: Option<BoundedVec<u8, T::UrlStringLimit>> = None;

			if let Some(v) = name {
				new_name = Some(v.clone());
				Name::<T>::put(v);
			}

			if let Some(v) = version {
				new_version = Some(v.clone());
				Version::<T>::put(v);
			}

			if let Some(v) = chain_id.clone() {
				new_chain_id = Some(v);
				ChainId::<T>::put(v);
			}

			if let Some(v) = decode_url {
				new_decode_url = Some(v.clone());
				DecodeUrl::<T>::put(v);
			}

			<Pallet<T>>::deposit_event(Event::MetadataUpdated {
				name: new_name,
				version: new_version,
				chain_id: new_chain_id,
				decode_url: new_decode_url,
			});

			Ok(())
		}
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			panic!("you should provide config yourself");
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T> {
		pub name: String,
		pub version: String,
		pub chain_id: u64,
		pub decode_url: String,
		pub _phantom: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Name::<T>::put(
				TryInto::<BoundedVec<_, T::StringLimit>>::try_into(self.name.clone().into_bytes())
					.expect("name is required"),
			);
			Version::<T>::put(
				TryInto::<BoundedVec<_, T::StringLimit>>::try_into(
					self.version.clone().into_bytes(),
				)
				.expect("version is required"),
			);
			ChainId::<T>::put(self.chain_id);
			DecodeUrl::<T>::put(
				TryInto::<BoundedVec<_, T::UrlStringLimit>>::try_into(
					self.decode_url.clone().into_bytes(),
				)
				.expect("decode url is required"),
			);
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_decode_url() -> Option<String> {
			String::from_utf8(DecodeUrl::<T>::get().into_inner()).ok()
		}

		pub fn get_eip_metadata() -> Option<sp_runtime::generic::Eip712Domain> {
			use codec::alloc::string::String;
			let r: sp_runtime::generic::Eip712Domain = sp_runtime::generic::eip712_domain! {
				name: String::from_utf8(Name::<T>::get().into_inner()).ok()?,
				version: String::from_utf8(Version::<T>::get().into_inner()).ok()?,
				chain_id: ChainId::<T>::get(),
			};
			Some(r)
		}

		pub fn eip712_payload(call: String) -> String {
			let input = r#"{
					"types": {
						"EIP712Domain": [
						{
							"name": "name",
							"type": "string"
						},
						{
							"name": "version",
							"type": "string"
						},
						{
							"name": "chainId",
							"type": "uint256"
						}
						],
						"Message": [
						{
							"name": "call",
							"type": "string"
						},
						{
							"name": "tx",
							"type": "string"
						}
						]
					},
					"primaryType": "Message",
					"domain": {
						"name": "",
						"version": "",
						"chainId": ""
					},
					"message": {
						"call": "",
						"tx": ""
					}
			}"#;

			if let Ok(ref mut v) = serde_json::from_str::<serde_json::Value>(input) {
				v["domain"]["name"] = serde_json::Value::String(
					String::from_utf8(Name::<T>::get().into_inner()).unwrap_or_default(),
				);
				v["domain"]["chainId"] = serde_json::Value::Number(ChainId::<T>::get().into());
				v["domain"]["version"] = serde_json::Value::String(
					String::from_utf8(Version::<T>::get().into_inner()).unwrap_or_default(),
				);
				v["message"]["call"] = serde_json::Value::String(call);
				serde_json::to_string_pretty(v).unwrap_or_default()
			} else {
				Default::default()
			}
		}
	}
}
