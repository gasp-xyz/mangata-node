#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Get, StorageVersion},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_std::collections::btree_map::BTreeMap;

use sp_std::{convert::TryInto, prelude::*};

use codec::alloc::string::{String, ToString};
use sp_runtime::serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use scale_info::prelude::format;
use sp_core::U256;


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
		AccountId,
		Balance,
		ValueQuery,
	>;

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
		type MinimumSequencers: Get<u32>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(4, 4).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn provide_sequencer_stake(origin: OriginFor<T>, stake_amount: Balance) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?:

			<SequencerStake<T>>::try_mutate(&sender, |stake| -> DispatchResult {
				*stake = stake.checked_add(stake_amount).map_err(|_| {Error::<T>::MathOverflow})?;
				Ok(())
			})?;

			T::Tokens::reserve(T::NativeTokenId::get().into(), &sender, amount)?;

			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 2).saturating_add(Weight::from_parts(40_000_000, 0)))]
		pub fn set_sequencer_configuration(origin: OriginFor<T>, minimal_stake_amount: Balance, slash_fine_amount: Balance) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?:
			
			<MinimalStakeAmount<T>>::insert(minimal_stake_amount);
			<SlashFineAmount<T>>::insert(slash_fine_amount);

			Ok(().into())

		}
	}
}

impl<T: Config> Pallet<T> {

	fn slash_sequencer(sequencer: AccountId) -> DispatchResult{

		<SequencerStake<T>>::try_mutate(&sequencer, |stake| -> DispatchResult {
			let slash_fine_amount = SlashFineAmount::<T>::get();
			let slash_fine_amount_actual = stake.min(slash_fine_amount);
			*stake = stake.saturating_sub(slash_fine_amount_actual);
			let _ = T::Tokens::slash_reserve(T::NativeTokenId::get().into(), &sequencer, slash_fine_amount_actual);
			Ok(())
		})?;
		
		Ok(().into())
	}

	fn process_potential_authors(authors: Vec<AccountId>) -> Vec<AccountId> {
		let minimal_stake_amount = MinimalStakeAmount::<T>::get();
		let filtered_authors = authors.iter().filter( |&a| <SequencerStake<T>>::get(*a) > minimal_stake_amount );
		if filtered_authors.len() >= T::MinimumSequencers{
			Some(filtered_authors)
		} else {
			None
		}
	}
}
