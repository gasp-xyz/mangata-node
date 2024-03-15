#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Get, StorageVersion},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use mangata_support::traits::GetMaintenanceStatusTrait;

use sp_std::{convert::TryInto, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub(crate) const LOG_TARGET: &'static str = "maintenance";

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

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo, Default,
	)]
	pub struct MaintenanceStatusInfo {
		pub is_maintenance: bool,
		pub is_upgradable_in_maintenance: bool,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_maintenance_status)]
	pub type MaintenanceStatus<T: Config> = StorageValue<_, MaintenanceStatusInfo, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Maintenance mode has been switched on
		MaintenanceModeSwitchedOn(T::AccountId),
		/// Maintenance mode has been switched off
		MaintenanceModeSwitchedOff(T::AccountId),
		/// Upgradablilty in maintenance mode has been switched on
		UpgradabilityInMaintenanceModeSwitchedOn(T::AccountId),
		/// Upgradablilty in maintenance mode has been switched off
		UpgradabilityInMaintenanceModeSwitchedOff(T::AccountId),
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// Timeouts were incorrectly initialized
		NotFoundationAccount,
		/// Not in maintenance mode
		NotInMaintenanceMode,
		/// Already in maintenance mode
		AlreadyInMaintenanceMode,
		/// Already upgradable in maintenance mode
		AlreadyUpgradableInMaintenanceMode,
		/// Already not upgradable in maintenance mode
		AlreadyNotUpgradableInMaintenanceMode,
		/// Upgrade blocked by Maintenance
		UpgradeBlockedByMaintenance,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type FoundationAccountsProvider: Get<Vec<Self::AccountId>>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn switch_maintenance_mode_on(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			ensure!(
				T::FoundationAccountsProvider::get().contains(&caller),
				Error::<T>::NotFoundationAccount
			);

			let current_maintenance_status = MaintenanceStatus::<T>::get();

			ensure!(
				!current_maintenance_status.is_maintenance,
				Error::<T>::AlreadyInMaintenanceMode
			);

			let maintenance_status =
				MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: false };

			MaintenanceStatus::<T>::put(maintenance_status);

			Pallet::<T>::deposit_event(Event::MaintenanceModeSwitchedOn(caller));

			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn switch_maintenance_mode_off(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			ensure!(
				T::FoundationAccountsProvider::get().contains(&caller),
				Error::<T>::NotFoundationAccount
			);

			let current_maintenance_status = MaintenanceStatus::<T>::get();

			ensure!(current_maintenance_status.is_maintenance, Error::<T>::NotInMaintenanceMode);

			let maintenance_status = MaintenanceStatusInfo {
				is_maintenance: false,
				is_upgradable_in_maintenance: false,
			};

			MaintenanceStatus::<T>::put(maintenance_status);

			Pallet::<T>::deposit_event(Event::MaintenanceModeSwitchedOff(caller));

			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn switch_upgradability_in_maintenance_mode_on(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			ensure!(
				T::FoundationAccountsProvider::get().contains(&caller),
				Error::<T>::NotFoundationAccount
			);

			let current_maintenance_status = MaintenanceStatus::<T>::get();

			ensure!(current_maintenance_status.is_maintenance, Error::<T>::NotInMaintenanceMode);

			ensure!(
				!current_maintenance_status.is_upgradable_in_maintenance,
				Error::<T>::AlreadyUpgradableInMaintenanceMode
			);

			let maintenance_status =
				MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: true };

			MaintenanceStatus::<T>::put(maintenance_status);

			Pallet::<T>::deposit_event(Event::UpgradabilityInMaintenanceModeSwitchedOn(caller));

			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn switch_upgradability_in_maintenance_mode_off(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			ensure!(
				T::FoundationAccountsProvider::get().contains(&caller),
				Error::<T>::NotFoundationAccount
			);

			let current_maintenance_status = MaintenanceStatus::<T>::get();

			ensure!(current_maintenance_status.is_maintenance, Error::<T>::NotInMaintenanceMode);

			ensure!(
				current_maintenance_status.is_upgradable_in_maintenance,
				Error::<T>::AlreadyNotUpgradableInMaintenanceMode
			);

			let maintenance_status =
				MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: false };

			MaintenanceStatus::<T>::put(maintenance_status);

			Pallet::<T>::deposit_event(Event::UpgradabilityInMaintenanceModeSwitchedOff(caller));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn is_maintenance() -> bool {
		let current_maintenance_status = MaintenanceStatus::<T>::get();
		current_maintenance_status.is_maintenance
	}

	fn is_upgradable() -> bool {
		let current_maintenance_status = MaintenanceStatus::<T>::get();
		(!current_maintenance_status.is_maintenance) ||
			(current_maintenance_status.is_maintenance &&
				current_maintenance_status.is_upgradable_in_maintenance)
	}
}

impl<T: Config> GetMaintenanceStatusTrait for Pallet<T> {
	fn is_maintenance() -> bool {
		<Pallet<T>>::is_maintenance()
	}

	fn is_upgradable() -> bool {
		<Pallet<T>>::is_upgradable()
	}
}
