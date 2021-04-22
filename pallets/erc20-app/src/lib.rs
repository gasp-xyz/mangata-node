// Based on Snowfork bridge implementation
//! # ERC20
//!
//! An application that implements bridged ERC20 token assets.
//!
//! ## Overview
//!
//! ETH balances are stored in the tightly-coupled [`asset`] runtime module. When an account holder burns
//! some of their balance, a `Transfer` event is emitted. An external relayer will listen for this event
//! and relay it to the other chain.
//!
//! ## Interface
//!
//! This application implements the [`Application`] trait and conforms to its interface.
//!
//! ### Dispatchable Calls
//!
//! - `burn`: Burn an ERC20 token balance.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use sp_core::{H160, U256};
use sp_std::prelude::*;

use artemis_asset as asset;
use artemis_core::{Application, BridgedAssetId};
use sp_runtime::traits::SaturatedConversion;

use orml_tokens::{MultiTokenCreatableCurrency, MultiTokenCurrency};

mod payload;
use payload::Payload;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait + asset::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

type BalanceOf<T> =
    <<T as artemis_asset::Trait>::Currency as MultiTokenCurrency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_storage! {
    trait Store for Module<T: Trait> as Erc20Module {}
}

decl_event!(
    /// Events for the ERC20 module.
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        /// Signal a cross-chain transfer.
        Transfer(BridgedAssetId, AccountId, H160, U256),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Asset ID is invalid.
        InvalidAssetId,
        /// The submitted payload could not be decoded.
        InvalidPayload,
        /// Asset could not be burned
        BurnFailure,
        /// The recipient address is null/default value
        NullRecipient
    }
}

decl_module! {

    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;

        /// Burn an ERC20 token balance
        #[weight = 0]
        pub fn burn(origin, asset_id: BridgedAssetId, recipient: H160, amount: U256) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // The asset_id 0 is reserved for the ETH app
            if asset_id == H160::zero() {
                return Err(Error::<T>::InvalidAssetId.into())
            }


            let native_asset_id = <asset::Module<T>>::get_native_asset_id(asset_id);

            ensure!(T::Currency::can_slash(
                native_asset_id,
                &who,
                amount.low_u128().saturated_into::<BalanceOf<T>>(),
                ),
                Error::<T>::BurnFailure);

            T::Currency::slash(
                native_asset_id,
                &who,
                amount.low_u128().saturated_into::<BalanceOf<T>>(),
                );


            Self::deposit_event(RawEvent::Transfer(asset_id, who, recipient, amount));
            Ok(())
        }

    }
}

impl<T: Trait> Module<T> {
    fn handle_event(payload: Payload<T::AccountId>) -> DispatchResult {
        if payload.token_addr.is_zero() {
            return Err(Error::<T>::InvalidAssetId.into());
        }

        if T::AccountId::default() == payload.recipient_addr {
            return Err(Error::<T>::NullRecipient.into());
        }

        //FIXME overflow unsafe!
        if !<asset::Module<T>>::exists(payload.token_addr) {
            let id = T::Currency::create(
                &payload.recipient_addr,
                payload.amount.low_u128().saturated_into::<BalanceOf<T>>(),
            );
            <asset::Module<T>>::link_assets(id, payload.token_addr);
        } else {
            let id = <asset::Module<T>>::get_native_asset_id(payload.token_addr);
            T::Currency::mint(
                id,
                &payload.recipient_addr,
                payload.amount.low_u128().saturated_into::<BalanceOf<T>>(),
            )?;
        }

        Ok(())
    }
}

impl<T: Trait> Application for Module<T> {
    fn handle(payload: Vec<u8>) -> DispatchResult {
        let payload_decoded = Payload::decode(payload).map_err(|_| Error::<T>::InvalidPayload)?;

        Self::handle_event(payload_decoded)
    }
}
