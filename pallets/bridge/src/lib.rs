//! # Bridge
//!
//! The Bridge module is the primary interface for submitting external messages to the parachain.
//!
//! ## Implementation
//!
//! Before a [Message] is dispatched to a target [`Application`], it is submitted to a [`Verifier`] for verification. The target application is determined using the [`AppId`] submitted along with the message.
//!
//! ## Interface
//!
//! ### Dispatchable Calls
//!
//! - `submit`: Submit a message for verification and dispatch.
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_root, ensure_signed};

use artemis_core::{App, AppId, Application, Message, Verifier};
use sp_std::convert::TryInto;
use sp_std::prelude::*;

mod weights;
use weights::{WeightInfo, WeightInfoTrait};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// The verifier module responsible for verifying submitted messages.
	type Verifier: Verifier<<Self as system::Config>::AccountId>;

	/// ETH Application
	type AppETH: Application;

	/// ERC20 Application
	type AppERC20: Application;
}

decl_storage! {
	trait Store for Module<T: Config> as BridgeModule {
		AppRegistry get(fn app_registry): map hasher(blake2_128_concat) AppId => Option<App>;
	}
	add_extra_genesis {
		config(bridged_app_id_registry): Vec<(App, AppId)>;
		build(|config: &GenesisConfig|{
			for (entry_app, entry_app_id) in config.bridged_app_id_registry.iter(){
				AppRegistry::insert(
					entry_app_id,
					entry_app
				);
			}
		});
	}
}

decl_event! {
	/// Events for the Bridge module.
	pub enum Event
		{
		/// An AppRegistry entry has been updated
		AppUpdated(App, AppId),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Target application not found.
		AppNotFound,

		/// Updated AppId is the same as current
		DifferentAppIdRequired
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Updates an app registry entry. Can use provided current_app_id_option to reduce DB reads.
		#[weight = WeightInfo::update_registry(*current_app_id_option)]
		pub fn update_registry(origin, app: App, current_app_id_option: Option<AppId>, updated_app_id: AppId) -> DispatchResult {

			ensure_root(origin)?;

			let mut current_app_id: AppId = AppId::default();
			let mut current_app_id_found: bool = false;

			match current_app_id_option{
				Some(provided_app_id) => {
					ensure!( AppRegistry::contains_key(&provided_app_id), Error::<T>::AppNotFound);
					current_app_id = provided_app_id;
					current_app_id_found = true;
				},

				None => {
					for (entry_app_id, entry_app) in AppRegistry::iter(){
						if entry_app == app{
							current_app_id = entry_app_id;
							current_app_id_found = true;
							#[cfg(not(feature = "runtime-benchmarks"))]
							break;
						}
					}
				}
			}

			if current_app_id_found {
				ensure!(!(updated_app_id == current_app_id), Error::<T>::DifferentAppIdRequired);
				AppRegistry::remove(&current_app_id);
			};

			AppRegistry::insert(updated_app_id, app);

			Self::deposit_event(Event::AppUpdated(app, updated_app_id));
			Ok(())

		}


		/// Submit `message` for dispatch to a target application identified by `app_id`.
		#[weight = 1_000_000_000]
		pub fn submit(origin, app_id: AppId, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let app = AppRegistry::get(&app_id).ok_or(Error::<T>::AppNotFound)?;
			Self::verify(who, app_id, &message)?;
			Self::dispatch(app, message)
		}

	}
}

impl<T: Config> Module<T> {
	fn verify(sender: T::AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		T::Verifier::verify(sender, app_id, &message)
	}

	fn dispatch(app: App, message: Message) -> DispatchResult {
		match app {
			App::ETH => T::AppETH::handle(message.payload),
			App::ERC20 => T::AppERC20::handle(message.payload),
		}
	}
}
