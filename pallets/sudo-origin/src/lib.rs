// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # SudoOrigin Pallet
//!
//! - [`Config`]
//! - [`Call`]
//! - [`SudoOrigin`]
//!
//! ## Overview
//!
//! The SudoOrigin pallet allows for an origin
//! to execute dispatchable functions that require a `Root` call.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! Only the sudo origin can call the dispatchable functions from the SudoOrigin pallet.
//!
//! * `sudo` - Make a `Root` call to a dispatchable function.
//!
//! ## Usage
//!
//! ### Executing Privileged Functions
//!
//! The SudoOrigin pallet is intended to be used with the Council. The council can use this pallet to make `Root` calls
//! You can build "privileged functions" (i.e. functions that require `Root` origin) in
//! other pallets. You can execute these privileged functions by calling `sudo` with the sudo origin.
//! Privileged functions cannot be directly executed via an extrinsic.
//!
//! Learn more about privileged functions and `Root` origin in the [`Origin`] type documentation.
//!
//! ### Simple Code Snippet
//!
//! This is an example of a pallet that exposes a privileged function:
//!
//! ```
//!
//! #[frame_support::pallet]
//! pub mod logger {
//! 	use frame_support::pallet_prelude::*;
//! 	use frame_system::pallet_prelude::*;
//! 	use super::*;
//!
//! 	#[pallet::config]
//! 	pub trait Config: frame_system::Config {}
//!
//! 	#[pallet::pallet]
//! 	pub struct Pallet<T>(PhantomData<T>);
//!
//! 	#[pallet::call]
//! 	impl<T: Config> Pallet<T> {
//! 		#[pallet::weight(0)]
//!         pub fn privileged_function(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
//!             ensure_root(origin)?;
//!
//!             // do something...
//!
//!             Ok(().into())
//!         }
//! 	}
//! }
//! # fn main() {}
//! ```
//!
//! ## Genesis Config
//!
//! The SudoOrigin pallet depends on the runtiem config.
//!
//! ## Related Pallets
//!
//! * Collective
//!
//! [`Origin`]: https://docs.substrate.io/v3/runtime/origins

#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{traits::StaticLookup, DispatchResult};
use sp_std::prelude::*;
use sp_std::convert::TryInto;
use frame_support::{traits::UnfilteredDispatchable, weights::GetDispatchInfo};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::{DispatchResult, *};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// A sudo-able call.
		type Call: Parameter + UnfilteredDispatchable<Origin = Self::Origin> + GetDispatchInfo;

		/// The Origin allowed to use sudo
		type SudoOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Authenticates the SudoOrigin and dispatches a function call with `Root` origin.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB write (event).
		/// - Weight of derivative `call` execution + 10,000.
		/// # </weight>
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight.saturating_add(10_000), dispatch_info.class)
		})]
		pub fn sudo(
			origin: OriginFor<T>,
			call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			// This is a public call, so we ensure that the origin is SudoOrigin.
			T::SudoOrigin::ensure_origin(origin)?;

			let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
			Self::deposit_event(Event::SuOriginDid(res.map(|_| ()).map_err(|e| e.error)));
			// Sudo user does not pay a fee.
			Ok(Pays::No.into())
		}

		/// Authenticates the SudoOrigin and dispatches a function call with `Root` origin.
		/// This function does not check the weight of the call, and instead allows the
		/// SudoOrigin to specify the weight of the call.
		///
		/// # <weight>
		/// - O(1).
		/// - The weight of this call is defined by the caller.
		/// # </weight>
		#[pallet::weight((*_weight, call.get_dispatch_info().class))]
		pub fn sudo_unchecked_weight(
			origin: OriginFor<T>,
			call: Box<<T as Config>::Call>,
			_weight: Weight,
		) -> DispatchResultWithPostInfo {
			// This is a public call, so we ensure that the origin is SudoOrigin.
			T::SudoOrigin::ensure_origin(origin)?;

			let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
			Self::deposit_event(Event::SuOriginDid(res.map(|_| ()).map_err(|e| e.error)));
			// Sudo user does not pay a fee.
			Ok(Pays::No.into())
		}

		/// Authenticates the SudoOrigin and dispatches a function call with `Signed` origin from
		/// a given account.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB write (event).
		/// - Weight of derivative `call` execution + 10,000.
		/// # </weight>
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(
				dispatch_info.weight
					.saturating_add(10_000)
					// AccountData for inner call origin accountdata.
					.saturating_add(T::DbWeight::get().reads_writes(1, 1)),
				dispatch_info.class,
			)
		})]
		pub fn sudo_as(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
			call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			// This is a public call, so we ensure that the origin is SudoOrigin.
			T::SudoOrigin::ensure_origin(origin)?;

			let who = T::Lookup::lookup(who)?;

			let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Signed(who).into());

			Self::deposit_event(Event::SuOriginDoAsDone(res.map(|_| ()).map_err(|e| e.error)));
			// Sudo user does not pay a fee.
			Ok(Pays::No.into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A sudo just took place. \[result\]
		SuOriginDid(DispatchResult),
		/// A sudo just took place. \[result\]
		SuOriginDoAsDone(DispatchResult),
	}

	#[pallet::error]
	/// Error for the Sudo pallet
	pub enum Error<T> {}
}
