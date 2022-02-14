// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	ensure, parameter_types,
	sp_runtime::RuntimeDebug,
	traits::Get,
	BoundedVec,
};
use frame_system::{ensure_root, pallet_prelude::*};
use mangata_primitives::TokenId;
use scale_info::TypeInfo;
use sp_std::prelude::*;

use orml_tokens::MultiTokenCurrencyExtended;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

parameter_types! {
	pub const NameMaxSize: u32 = 128;
	pub const SymbolMaxSize: u32 = 32;
	pub const DescMaxSize: u32 = 256;
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AssetInfo {
	pub name: Option<BoundedVec<u8, NameMaxSize>>,
	pub symbol: Option<BoundedVec<u8, SymbolMaxSize>>,
	pub description: Option<BoundedVec<u8, DescMaxSize>>,
	pub decimals: Option<u32>,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The minimum length a name may be.
		type MinLengthName: Get<usize>;

		/// The maximum length a name may be.
		type MaxLengthName: Get<usize>;

		/// The minimum length a symbol may be.
		type MinLengthSymbol: Get<usize>;

		/// The maximum length a symbol may be.
		type MaxLengthSymbol: Get<usize>;

		/// The minimum length a description may be.
		type MinLengthDescription: Get<usize>;

		/// The maximum length a description may be.
		type MaxLengthDescription: Get<usize>;

		/// The maximum decimal points an asset may be.
		type MaxDecimals: Get<u32>;

		type Currency: MultiTokenCurrencyExtended<Self::AccountId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_info)]
	pub type AssetsInfo<T: Config> = StorageMap<_, Twox64Concat, TokenId, AssetInfo, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub bridged_assets_info:
			Vec<(Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, Option<u32>, TokenId)>,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { bridged_assets_info: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			for (name, token, description, decimals, asset_id) in self.bridged_assets_info.iter() {
				AssetsInfo::<T>::insert(
					asset_id,
					AssetInfo {
						name: name.as_ref().map(|n| BoundedVec::try_from(n.clone()).unwrap()),
						symbol: token.as_ref().map(|t| BoundedVec::try_from(t.clone()).unwrap()),
						description: description
							.as_ref()
							.map(|d| BoundedVec::try_from(d.clone()).unwrap()),
						decimals: decimals.to_owned().into(),
					},
				);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	pub enum Event<T: Config> {
		/// Asset info stored. [assetId, info]
		InfoStored(TokenId, AssetInfo),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// A name is too short.
		TooShortName,
		/// A name is too long.
		TooLongName,
		/// A symbol is too short.
		TooShortSymbol,
		/// A symbol is too long.
		TooLongSymbol,
		/// A description is too short.
		TooShortDescription,
		/// A description is too long.
		TooLongDescription,
		/// A decimals point value is out of range
		DecimalsOutOfRange,
		/// Asset does not exist
		AssetNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
		pub fn set_info(
			origin: OriginFor<T>,
			asset: TokenId,
			name: Option<Vec<u8>>,
			symbol: Option<Vec<u8>>,
			description: Option<Vec<u8>>,
			decimals: Option<u32>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let info = Self::set_asset_info(asset, name, symbol, description, decimals)?;

			Self::deposit_event(Event::InfoStored(asset, info));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn set_asset_info(
		asset: TokenId,
		name: Option<Vec<u8>>,
		symbol: Option<Vec<u8>>,
		description: Option<Vec<u8>>,
		decimals: Option<u32>,
	) -> Result<AssetInfo, Error<T>> {
		// is this the correct approach, could be a separate fn at least ?
		#[cfg(not(test))]
		{
			ensure!(T::Currency::exists(asset.into()), Error::<T>::AssetNotExist);
		}
		//
		let current: AssetInfo = Self::get_info(asset);

		let info = AssetInfo {
			name: name.as_ref().map(|n| BoundedVec::try_from(n.clone()).unwrap()).or(current.name),
			symbol: symbol
				.as_ref()
				.map(|t| BoundedVec::try_from(t.clone()).unwrap())
				.or(current.symbol),
			description: description
				.as_ref()
				.map(|d| BoundedVec::try_from(d.clone()).unwrap())
				.or(current.description),
			decimals: decimals.or(current.decimals),
		};
		let to_check = info.clone();

		if to_check.name.is_some() {
			let name = to_check.name.unwrap();
			ensure!(name.len() >= T::MinLengthName::get(), Error::<T>::TooShortName);
			ensure!(name.len() <= T::MaxLengthName::get(), Error::<T>::TooLongName);
		}
		if to_check.symbol.is_some() {
			let sym = to_check.symbol.unwrap();
			ensure!(sym.len() >= T::MinLengthSymbol::get(), Error::<T>::TooShortSymbol);
			ensure!(sym.len() <= T::MaxLengthSymbol::get(), Error::<T>::TooLongSymbol);
		}
		if to_check.description.is_some() {
			let desc = to_check.description.unwrap();
			ensure!(desc.len() >= T::MinLengthDescription::get(), Error::<T>::TooShortDescription);
			ensure!(desc.len() <= T::MaxLengthDescription::get(), Error::<T>::TooLongDescription);
		}
		if to_check.decimals.is_some() {
			let decimals = to_check.decimals.unwrap();
			ensure!(decimals <= T::MaxDecimals::get() as u32, Error::<T>::DecimalsOutOfRange);
		}

		AssetsInfo::<T>::insert(asset, info.clone());

		Ok(info)
	}
}
