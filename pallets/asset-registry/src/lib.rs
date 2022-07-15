// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! # Asset Registry Module
//!
//! foreign assets management. The foreign assets can be updated without runtime upgrade.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	dispatch::DispatchResult, ensure, pallet_prelude::*, traits::EnsureOrigin, transactional,
};
use frame_system::pallet_prelude::*;
use mangata_primitives::TokenId;
use sp_std::boxed::Box;

// NOTE:v1::MultiLocation is used in storages, we would need to do migration if upgrade the
// MultiLocation in the future.
pub use xcm::{v1::MultiLocation, VersionedMultiLocation};

use orml_tokens::MultiTokenCurrencyExtended;

pub mod benchmarking;
mod mock;
mod tests;
pub mod weights;

pub use pallet::*;

/// Weight functions needed for module_asset_registry.
pub trait WeightInfo {
	fn register_asset() -> Weight;
	fn update_asset() -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

		/// Currency type for withdraw and balance storage.
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>;

		/// Required origin for registering asset.
		type RegisterOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type TreasuryAddress: Get<Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The given location could not be used (e.g. because it cannot be expressed in the
		/// desired version of XCM).
		BadLocation,
		/// MultiLocation existed
		MultiLocationExisted,
		/// AssetId not exists
		AssetIdNotExists,
		/// AssetId exists
		AssetIdExisted,
		/// Creating a token for the foreign asset failed
		TokenCreationFailed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event {
		/// The asset registered.
		AssetRegistered { asset_id: TokenId, asset_address: MultiLocation },
		/// The asset updated.
		AssetUpdated { asset_id: TokenId, asset_address: MultiLocation },
	}

	/// The storages for MultiLocations.
	///
	/// AssetLocations: map TokenId => Option<MultiLocation>
	#[pallet::storage]
	#[pallet::getter(fn asset_locations)]
	pub type AssetLocations<T: Config> =
		StorageMap<_, Twox64Concat, TokenId, MultiLocation, OptionQuery>;

	/// The storages for CurrencyIds.
	///
	/// LocationToCurrencyIds: map MultiLocation => Option<TokenId>
	#[pallet::storage]
	#[pallet::getter(fn location_to_currency_ids)]
	pub type LocationToCurrencyIds<T: Config> =
		StorageMap<_, Twox64Concat, MultiLocation, TokenId, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub init_xcm_tokens: Vec<(TokenId, Option<Vec<u8>>)>,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig { init_xcm_tokens: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			self.init_xcm_tokens.iter().for_each(
				|(token_id, maybe_versioned_asset_multilocation_encoded)| {
					if let Some(versioned_asset_multilocation_encoded) =
						maybe_versioned_asset_multilocation_encoded
					{
						let versioned_asset_multilocation =
							MultiLocation::decode(&mut &versioned_asset_multilocation_encoded[..])
								.expect("Error decoding multilocation");
						let asset_multilocation: MultiLocation = versioned_asset_multilocation
							.try_into()
							.expect("Error unable to unversion multilocation");
						let created_token_id = Pallet::<T>::do_register_asset(&asset_multilocation)
							.expect("Error registering xcm asset");
						assert!(
							created_token_id == *token_id,
							"Assets not initialized in the expected sequence"
						);
					} else {
						let created_token_id: TokenId =
							T::Currency::create(&T::TreasuryAddress::get(), Default::default())
								.expect("Error creating token for xcm asset")
								.into();
						assert!(
							created_token_id == *token_id,
							"Assets not initialized in the expected sequence"
						);
					}
				},
			)
		}
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::register_asset())]
		#[transactional]
		pub fn register_asset(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			T::RegisterOrigin::ensure_origin(origin)?;

			let location: MultiLocation =
				(*location).try_into().map_err(|()| Error::<T>::BadLocation)?;
			let asset_id = Self::do_register_asset(&location)?;

			Self::deposit_event(Event::AssetRegistered { asset_id, asset_address: location });
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::update_asset())]
		#[transactional]
		pub fn update_asset(
			origin: OriginFor<T>,
			asset_id: TokenId,
			location: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			T::RegisterOrigin::ensure_origin(origin)?;

			let location: MultiLocation =
				(*location).try_into().map_err(|()| Error::<T>::BadLocation)?;
			Self::do_update_asset(asset_id, &location)?;

			Self::deposit_event(Event::AssetUpdated { asset_id, asset_address: location });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_register_asset(location: &MultiLocation) -> Result<TokenId, DispatchError> {
		let asset_id: TokenId = T::Currency::create(&T::TreasuryAddress::get(), Default::default())
			.map_err(|_| DispatchError::from(Error::<T>::TokenCreationFailed))?
			.into();
		LocationToCurrencyIds::<T>::try_mutate(location, |maybe_currency_ids| -> DispatchResult {
			ensure!(maybe_currency_ids.is_none(), Error::<T>::MultiLocationExisted);
			*maybe_currency_ids = Some(asset_id);

			AssetLocations::<T>::try_mutate(asset_id, |maybe_location| -> DispatchResult {
				ensure!(maybe_location.is_none(), Error::<T>::AssetIdExisted);
				*maybe_location = Some(location.clone());
				Ok(())
			})
		})?;

		Ok(asset_id)
	}

	fn do_update_asset(asset_id: TokenId, location: &MultiLocation) -> DispatchResult {
		AssetLocations::<T>::try_mutate(asset_id, |maybe_multi_locations| -> DispatchResult {
			let old_multi_locations =
				maybe_multi_locations.as_mut().ok_or(Error::<T>::AssetIdNotExists)?;
			// modify location
			if location != old_multi_locations {
				LocationToCurrencyIds::<T>::remove(old_multi_locations.clone());
				LocationToCurrencyIds::<T>::try_mutate(
					location,
					|maybe_currency_ids| -> DispatchResult {
						ensure!(maybe_currency_ids.is_none(), Error::<T>::MultiLocationExisted);
						*maybe_currency_ids = Some(asset_id);
						Ok(())
					},
				)?;
			}
			*old_multi_locations = location.clone();
			Ok(())
		})
	}
}

/// A mapping between AssetId and Locations.
pub trait AssetIdMapping<MultiLocation> {
	/// Returns the MultiLocation associated with a given ForeignAssetId.
	fn get_multi_location(token_id: TokenId) -> Option<MultiLocation>;
	/// Returns the CurrencyId associated with a given MultiLocation.
	fn get_currency_id(multi_location: MultiLocation) -> Option<TokenId>;
}

pub struct AssetIdMaps<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AssetIdMapping<MultiLocation> for AssetIdMaps<T> {
	fn get_multi_location(token_id: TokenId) -> Option<MultiLocation> {
		Pallet::<T>::asset_locations(token_id)
	}

	fn get_currency_id(multi_location: MultiLocation) -> Option<TokenId> {
		Pallet::<T>::location_to_currency_ids(multi_location)
	}
}
