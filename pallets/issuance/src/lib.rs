// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;

use frame_support::{
	codec::{Decode, Encode},
	ensure,
	traits::{Get, Currency},
};
use frame_system::{ensure_root, pallet_prelude::*};
use mangata_primitives::TokenId;
use scale_info::TypeInfo;
use sp_std::prelude::*;
use sp_runtime::{
	traits::{Saturating, Zero, One},
	Perbill, Permill, Percent, RuntimeDebug,
	helpers_128bit::multiply_by_rational,
};
use mangata_primitives::Balance;

use sp_runtime::traits::CheckedAdd;
use orml_tokens::{MultiTokenCurrency};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

// TODO
// Use primitives for round and linear_time types

// TODO
// Use assert to ensure blockperround is > 2

// use orml_tokens::MultiTokenCurrencyExtended;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct IssuanceConfigType {
	// Max number of MGA to target
	pub cap: Balance,
	// MGA created at token generation event
	pub tge: Balance,
	// Time between which the total issuance of MGA grows to cap, as number of blocks
	pub linear_issuance_blocks: u32,
	// The split of issuance assgined to liquidity_mining
	pub liquidity_mining_split: Percent,
	// The split of issuance assgined to staking
	pub staking_split: Percent,
	// The split of issuance assgined to crowdloan
	pub crowdloan_split: Percent,
}

// TODO
// assert percents add up to 100

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		// TODO
		// Add hooks to calculate the issuance in an session.
		// The issuance of a session needs to be decided before it starts.
		// Perhaps do it as a part of the session pallet?

		// TODO
		// Discard states after history period expires

		// TODO
		// on_init must be before session and staking 

		// fn on_initialize(n: T::BlockNumber) -> Weight {
		// 	if T::ShouldEndSessionProvider::should_end_session(n) {
		// 		let current_round = T::RoundInfoProvider::get_current_round();
		// 		Pallet::<T>::calculate_and_store_round_issuance(current_round);
		// 		Pallet::<T>::clear_round_issuance_history(current_round);
		// 		// TODO
		// 		// return weight
		// 	}
		// 	0.into()
		// }
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// MGA currency to check total_issuance
		type NativeCurrencyId: Get<TokenId>;
		/// Tokens
		type Tokens: MultiTokenCurrency<Self::AccountId>;
		/// Number of blocks per session/round
		#[pallet::constant]
		type BlocksPerRound: Get<u32>;
		/// Number of sessions to store issuance history for
		#[pallet::constant]
		type HistoryLimit: Get<u32>;
	}
	
	// TODO
	// Add genesis for session 0 issuance

	#[pallet::storage]
	#[pallet::getter(fn get_issuance_config)]
	pub type IssuanceConfigStore<T: Config> = StorageValue<_, IssuanceConfigType, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_session_issuance)]
	pub type SessionIssuance<T: Config> = StorageMap<_, Twox64Concat, u32, Option<(Balance, Balance, Balance)>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub issuance_config: IssuanceConfigType
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { 	issuance_config: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			assert_eq!(self.issuance_config.liquidity_mining_split.checked_add(&self.issuance_config.staking_split).unwrap()
				.checked_add(&self.issuance_config.crowdloan_split).unwrap(), Percent::from_percent(100));
			assert!(self.issuance_config.cap >= self.issuance_config.tge);
			assert_ne!(self.issuance_config.linear_issuance_blocks, u32::zero());
			IssuanceConfigStore::<T>::put(self.issuance_config.clone());
			Pallet::<T>::calculate_and_store_round_issuance(0u32);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Issuance for upcoming session calculated and recorded
		SessionIssuanceRecorded(u32, Balance, Balance, Balance),
	}
}

pub trait ComputeIssuance {
	fn compute_issuance(n: u32);
}

impl<T: Config> ComputeIssuance for Pallet <T>{
	fn compute_issuance(n: u32){
		Pallet::<T>::calculate_and_store_round_issuance(n);
		Pallet::<T>::clear_round_issuance_history(n);
	}
}

pub trait GetIssuance {
	fn get_all_issuance(n: u32) -> Option<(Balance, Balance, Balance)>;
	fn get_liquidity_mining_issuance(n: u32) -> Option<Balance>;
	fn get_staking_issuance(n: u32) -> Option<Balance>;
	fn get_crowdloan_issuance(n: u32) -> Option<Balance>;
}

impl<T: Config> GetIssuance for Pallet <T>{
	fn get_all_issuance(n: u32) -> Option<(Balance, Balance, Balance)>{
		SessionIssuance::<T>::get(n)
	}
	fn get_liquidity_mining_issuance(n: u32) -> Option<Balance>{
		match SessionIssuance::<T>::get(n) {
			Some((x, _, _)) => Some(x),
			None => None,
		}
	}
	fn get_staking_issuance(n: u32) -> Option<Balance>{
		match SessionIssuance::<T>::get(n) {
			Some((_, x, _)) => Some(x),
			None => None,
		}
	}
	fn get_crowdloan_issuance(n: u32) -> Option<Balance>{
		match SessionIssuance::<T>::get(n) {
			Some((_, _, x)) => Some(x),
			None => None,
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn calculate_and_store_round_issuance(
		current_round: u32,
	) -> DispatchResult {
		let issuance_config = IssuanceConfigStore::<T>::get();
		let yet_to_be_issued: Balance = issuance_config.cap - issuance_config.tge;
		let linear_issuance_sessions: u32 = issuance_config.linear_issuance_blocks / T::BlocksPerRound::get();
		let legacy_issuance_per_session = yet_to_be_issued / linear_issuance_sessions as Balance;

		let mut current_round_issuance: Balance = Zero::zero(); 
		if current_round <= linear_issuance_sessions {
			current_round_issuance = legacy_issuance_per_session;
		} else {
			let current_mga_total_issuance: Balance = T::Tokens::total_issuance(T::NativeCurrencyId::get().into()).into();
			if issuance_config.cap > current_mga_total_issuance {
				current_round_issuance = legacy_issuance_per_session.min(issuance_config.cap - current_mga_total_issuance)
			} else {
				current_round_issuance = Zero::zero();
			}
		}

		let liquidity_mining_issuance = issuance_config.liquidity_mining_split * current_round_issuance;
		let staking_issuance = issuance_config.staking_split * current_round_issuance;
		let crowdloan_issuance = issuance_config.crowdloan_split * current_round_issuance;
		
		SessionIssuance::<T>::insert(current_round, Some((liquidity_mining_issuance,
			staking_issuance,
			crowdloan_issuance)));

		Pallet::<T>::deposit_event(Event::SessionIssuanceRecorded(
			current_round,
			liquidity_mining_issuance,
			staking_issuance,
			crowdloan_issuance
		));

		Ok(())
	}

	pub fn clear_round_issuance_history(
		current_round: u32,
	) -> DispatchResult {
		if current_round >= T::HistoryLimit::get() {
			SessionIssuance::<T>::remove(current_round - T::HistoryLimit::get());
		}
		Ok(())
	}
}
