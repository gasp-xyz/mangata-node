// Copyright (C) 2020 Mangata team

//! # SudoOrigin Module
//!
//! ## Overview
//!
//! The SudoOrigin module allows for a origin
//! to execute dispatchable functions that require a `Root` call.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! Only the configured origin can call the dispatchable functions from the Sudo module.
//!
//! * `sudo` - Make a `Root` call to a dispatchable function.
//!
//! ## Usage
//!
//! ### Executing Privileged Functions
//!
//! The SudoOrigin module itself is not intended to be used within other modules.
//! Instead, you can build "privileged functions" (i.e. functions that require `Root` origin) in other modules.
//! You can execute these privileged functions by calling `sudo` from the configured origin.
//! Privileged functions cannot be directly executed via an extrinsic.
//!
//! Learn more about privileged functions and `Root` origin in the [`Origin`] type documentation.
//!
//! ### Simple Code Snippet
//!
//! This is an example of a module that exposes a privileged function:
//!
//! ```
//! use frame_support::{decl_module, dispatch};
//! use frame_system::ensure_root;
//!
//! pub trait Trait: frame_system::Trait {}
//!
//! decl_module! {
//!     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
//! 		#[weight = 0]
//!         pub fn privileged_function(origin) -> dispatch::DispatchResult {
//!             ensure_root(origin)?;
//!
//!             // do something...
//!
//!             Ok(())
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ## Runtime Config
//!
//! The SudoOrigin module depends on the Runtime for its accepted Origin configuration.
//!
//! ## Related Modules
//!
//! * [Democracy](../pallet_democracy/index.html)
//!
//! [`Call`]: ./enum.Call.html
//! [`Trait`]: ./trait.Trait.html
//! [`Origin`]: https://docs.substrate.dev/docs/substrate-types

#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{traits::StaticLookup, DispatchResult};
use sp_std::prelude::*;

use frame_support::traits::EnsureOrigin;
use frame_support::{decl_error, decl_event, decl_module, Parameter};
use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    traits::UnfilteredDispatchable,
    weights::{GetDispatchInfo, Pays, Weight},
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// A sudo-able call.
    type Call: Parameter + UnfilteredDispatchable<Origin = Self::Origin> + GetDispatchInfo;

    /// The Origin allowed to use sudo
    type SudoOrigin: EnsureOrigin<Self::Origin>;
}

decl_module! {
    /// Sudo module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Authenticates the sudo key and dispatches a function call with `Root` origin.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// - O(1).
        /// - Limited storage reads.
        /// - One DB write (event).
        /// - Weight of derivative `call` execution + 10,000.
        /// # </weight>
        #[weight = (call.get_dispatch_info().weight + 10_000, call.get_dispatch_info().class)]
        fn sudo(origin, call: Box<<T as Trait>::Call>) -> DispatchResultWithPostInfo {
            // This is a public call, so we ensure that the origin is authorized.
            T::SudoOrigin::ensure_origin(origin)?;

            let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
            Self::deposit_event(Event::SuOriginDid(res.map(|_| ()).map_err(|e| e.error)));
            // Sudo user does not pay a fee.
            Ok(Pays::No.into())
        }

        /// Authenticates the sudo key and dispatches a function call with `Root` origin.
        /// This function does not check the weight of the call, and instead allows the
        /// Sudo user to specify the weight of the call.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// - O(1).
        /// - The weight of this call is defined by the caller.
        /// # </weight>
        #[weight = (*_weight, call.get_dispatch_info().class)]
        fn sudo_unchecked_weight(origin, call: Box<<T as Trait>::Call>, _weight: Weight) -> DispatchResultWithPostInfo {
            // This is a public call, so we ensure that the origin is authorized.
            T::SudoOrigin::ensure_origin(origin)?;

            let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
            Self::deposit_event(Event::SuOriginDid(res.map(|_| ()).map_err(|e| e.error)));
            // Sudo user does not pay a fee.
            Ok(Pays::No.into())
        }


        /// Authenticates the sudo key and dispatches a function call with `Signed` origin from
        /// a given account.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// - O(1).
        /// - Limited storage reads.
        /// - One DB write (event).
        /// - Weight of derivative `call` execution + 10,000.
        /// # </weight>
        #[weight = (call.get_dispatch_info().weight + 10_000, call.get_dispatch_info().class)]
        fn sudo_as(origin,
            who: <T::Lookup as StaticLookup>::Source,
            call: Box<<T as Trait>::Call>
        ) -> DispatchResultWithPostInfo {
            // This is a public call, so we ensure that the origin is authorized.
            T::SudoOrigin::ensure_origin(origin)?;

            let who = T::Lookup::lookup(who)?;

            let res = match call.dispatch_bypass_filter(frame_system::RawOrigin::Signed(who).into()) {
                Ok(_) => true,
                Err(e) => {
                    sp_runtime::print(e);
                    false
                }
            };

            Self::deposit_event(Event::SuOriginDoAsDone(res));
            // Sudo user does not pay a fee.
            Ok(Pays::No.into())
        }
    }
}

decl_event!(
    pub enum Event {
        /// A sudo just took place. \[result\]
        SuOriginDid(DispatchResult),
        /// A sudo just took place. \[result\]
        SuOriginDoAsDone(bool),
    }
);

decl_error! {
    /// Error for the Sudo module
    pub enum Error for Module<T: Trait> {

    }
}
