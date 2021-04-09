// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    sp_runtime::RuntimeDebug,
    traits::{Get, Vec},
};
use frame_system::ensure_root;

use pallet_assets as assets;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: assets::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

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
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct AssetInfo {
    name: Option<Vec<u8>>,
    symbol: Option<Vec<u8>>,
    description: Option<Vec<u8>>,
    decimals: Option<u32>,
}

decl_storage! {
    trait Store for Module<T: Trait> as AssetsInfoModule {
        /// TWOX-NOTE: `AssetId` is trusted, so this is safe.
        AssetsInfo get(fn get_info): map hasher(twox_64_concat) T::AssetId => AssetInfo;
    }
    add_extra_genesis {
        #[allow(clippy::type_complexity)]
        config(bridged_assets_info): Vec<(Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, Option<u32>, T::AssetId)>;
        build(|config: &GenesisConfig<T>|{
            for (name, token, description, decimals, asset_id) in config.bridged_assets_info.iter(){
                <AssetsInfo<T>>::insert(
                    asset_id,
                    AssetInfo {
                        name: name.clone(),
                        symbol: token.clone(),
                        description: description.clone(),
                        decimals: decimals.to_owned(),
                    }
                );
            }
        });
    }

}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AssetId = <T as assets::Trait>::AssetId,
    {
        /// Asset info stored. [assetId, info]
        InfoStored(AssetId, AssetInfo),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
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
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn set_info(origin, asset: T::AssetId, name: Option<Vec<u8>>, symbol: Option<Vec<u8>>, description: Option<Vec<u8>>, decimals: Option<u32>) -> dispatch::DispatchResult {
            ensure_root(origin)?;

            let info = Self::set_asset_info(asset, name, symbol, description, decimals)?;

            Self::deposit_event(RawEvent::InfoStored(asset, info));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn set_asset_info(
        asset: T::AssetId,
        name: Option<Vec<u8>>,
        symbol: Option<Vec<u8>>,
        description: Option<Vec<u8>>,
        decimals: Option<u32>,
    ) -> Result<AssetInfo, Error<T>> {
        // is this the correct approach, could be a separate fn at least ?
        #[cfg(not(test))]
        {
            let id = <assets::Module<T>>::next_asset_id();
            ensure!(asset < id, Error::<T>::AssetNotExist);
        }

        let current: AssetInfo = Self::get_info(asset);

        let info = AssetInfo {
            name: name.or(current.name),
            symbol: symbol.or(current.symbol),
            description: description.or(current.description),
            decimals: decimals.or(current.decimals),
        };
        let to_check = info.clone();

        if to_check.name.is_some() {
            let name = to_check.name.unwrap();
            ensure!(
                name.len() >= T::MinLengthName::get(),
                Error::<T>::TooShortName
            );
            ensure!(
                name.len() <= T::MaxLengthName::get(),
                Error::<T>::TooLongName
            );
        }
        if to_check.symbol.is_some() {
            let sym = to_check.symbol.unwrap();
            ensure!(
                sym.len() >= T::MinLengthSymbol::get(),
                Error::<T>::TooShortSymbol
            );
            ensure!(
                sym.len() <= T::MaxLengthSymbol::get(),
                Error::<T>::TooLongSymbol
            );
        }
        if to_check.description.is_some() {
            let desc = to_check.description.unwrap();
            ensure!(
                desc.len() >= T::MinLengthDescription::get(),
                Error::<T>::TooShortDescription
            );
            ensure!(
                desc.len() <= T::MaxLengthDescription::get(),
                Error::<T>::TooLongDescription
            );
        }
        if to_check.decimals.is_some() {
            let decimals = to_check.decimals.unwrap();
            ensure!(
                decimals <= T::MaxDecimals::get() as u32,
                Error::<T>::DecimalsOutOfRange
            );
        }

        <AssetsInfo<T>>::insert(asset, info.clone());

        Ok(info)
    }
}
