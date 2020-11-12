#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage, dispatch,
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
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct AssetInfo {
    name: Vec<u8>,
    symbol: Vec<u8>,
    description: Vec<u8>,
    decimals: u32,
}

decl_storage! {
    trait Store for Module<T: Trait> as AssetsInfoModule {
        /// TWOX-NOTE: `AssetId` is trusted, so this is safe.
        AssetsInfo get(fn get_info): map hasher(twox_64_concat) T::AssetId => AssetInfo;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AssetId = <T as assets::Trait>::AssetId,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        InfoStored(AssetId, Vec<u8>),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
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

            Self::deposit_event(RawEvent::InfoStored(asset, info.name));

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
        // should check value constraints eg MIN/MAX len

        let current: AssetInfo = Self::get_info(asset);

        let info = AssetInfo {
            name: name.unwrap_or(current.name),
            symbol: symbol.unwrap_or(current.symbol),
            description: description.unwrap_or(current.description),
            decimals: decimals.unwrap_or(current.decimals),
        };

        <AssetsInfo<T>>::insert(asset, info.clone());

        Ok(info)
    }
}
