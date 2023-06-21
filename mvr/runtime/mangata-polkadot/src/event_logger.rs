use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Assets received from parachain (parachain_id, amount)
		AssetsReceived(u64, u128)
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn record_transfer(
			origin: OriginFor<T>,
			parachain: u64,
			amount: u128,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			Pallet::<T>::deposit_event(Event::AssetsReceived(parachain, amount));
			Ok(().into())
		}
	}
}

