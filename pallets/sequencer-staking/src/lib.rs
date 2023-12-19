#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Get, StorageVersion, ReservableCurrency, Currency},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_std::collections::btree_map::BTreeMap;

use sp_std::{convert::TryInto, prelude::*};

use codec::alloc::string::{String, ToString};
use sp_runtime::serde::{Deserialize, Serialize};
use scale_info::prelude::format;
use sp_core::U256;
use sp_runtime::traits::CheckedAdd;
use sp_runtime::Saturating;
pub use mangata_support::traits::{
	SequencerStakingProviderTrait, RolldownProviderTrait
};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

pub(crate) const LOG_TARGET: &'static str = "sequencer-staking";

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
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
	}

	#[pallet::storage]
	#[pallet::getter(fn get_sequencer_stake)]
	pub type SequencerStake<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		BalanceOf<T>,
		ValueQuery,
	>;


	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_slash_fine_amount)]
	pub type SlashFineAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_minimal_stake_amount)]
	pub type MinimalStakeAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		OperationFailed,
		MathOverflow
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: ReservableCurrency<Self::AccountId>;
		#[pallet::constant]
		type MinimumSequencers: Get<u32>;
		type RolldownProvider: RolldownProviderTrait<Self::AccountId>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(4, 4).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn provide_sequencer_stake(origin: OriginFor<T>, stake_amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;

			<SequencerStake<T>>::try_mutate(&sender, |stake| -> DispatchResult {
				let previous_stake = *stake;
				*stake = stake.checked_add(&stake_amount).ok_or(Error::<T>::MathOverflow)?;
				if previous_stake < MinimalStakeAmount::<T>::get() && *stake >= MinimalStakeAmount::<T>::get(){
					T::RolldownProvider::new_sequencer_active(sender.clone());
				}
				Ok(())
			})?;

			// add full rights to sequencer (create whole entry in sequencer_rights @ rolldown)
			// add +1 cancel right to all other sequencers (non active are deleted from sequencer_rights @ rolldown)
			T::Currency::reserve(&sender, stake_amount)?;

			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 2).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn set_sequencer_configuration(origin: OriginFor<T>, minimal_stake_amount: BalanceOf<T>, slash_fine_amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			
			let _ = ensure_root(origin)?;
			
			<MinimalStakeAmount<T>>::put(minimal_stake_amount);
			<SlashFineAmount<T>>::put(slash_fine_amount);

			Ok(().into())

		}
	}
}

impl<T: Config> SequencerStakingProviderTrait<AccountIdOf<T>, BalanceOf<T>> for Pallet<T> {

	fn is_active_sequencer(sequencer: AccountIdOf<T>) -> bool{

		<SequencerStake<T>>::get(&sequencer) >= MinimalStakeAmount::<T>::get()

	}

	fn slash_sequencer(sequencer: AccountIdOf<T>) -> DispatchResult{
		// Use slashed amount partially to reward canceler, partially to vault to pay for l1 fees
		<SequencerStake<T>>::try_mutate(&sequencer, |stake| -> DispatchResult {
			let slash_fine_amount = SlashFineAmount::<T>::get();
			let slash_fine_amount_actual = (*stake).min(slash_fine_amount);
			*stake = stake.saturating_sub(slash_fine_amount_actual);
			let _ = T::Currency::slash_reserved(&sequencer, slash_fine_amount_actual);
			Ok(())
		})?;
		
		Ok(().into())
	}

	fn process_potential_authors(authors: Vec<(AccountIdOf<T>, BalanceOf<T>)>) -> Option<Vec<(AccountIdOf<T>, BalanceOf<T>)>> {
		let minimal_stake_amount = MinimalStakeAmount::<T>::get();
		let filtered_authors: Vec<_> = authors.into_iter().filter( |a| <SequencerStake<T>>::get(&a.0) >= minimal_stake_amount).collect::<>();
		if filtered_authors.len() as u32 >= T::MinimumSequencers::get(){
			Some(filtered_authors)
		} else {
			None
		}
	}
}

/// Simple ensure origin struct to filter for the active sequencer accounts.
pub struct EnsureActiveSequencer<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> EnsureOrigin<<T as frame_system::Config>::RuntimeOrigin> for EnsureActiveSequencer<T> {
	type Success = T::AccountId;
	fn try_origin(o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
		o.into().and_then(|o| match o {
			frame_system::RawOrigin::Signed(ref who) if <Pallet<T> as SequencerStakingProviderTrait<AccountIdOf<T>, BalanceOf<T>>>::is_active_sequencer(who.clone()) => Ok(who.clone()),
			r => Err(T::RuntimeOrigin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<T::RuntimeOrigin, ()> {
		let founder = Founder::<T>::get().ok_or(())?;
		Ok(T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(founder)))
	}
}
