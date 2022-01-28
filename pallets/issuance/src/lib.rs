// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;

use frame_support::{
	codec::{Decode, Encode},
	traits::{Get},
};
use mangata_primitives::TokenId;
use scale_info::TypeInfo;
use sp_std::prelude::*;
use sp_runtime::{
	traits::{Zero},
	Percent, RuntimeDebug,
};
use mangata_primitives::Balance;

use sp_runtime::traits::CheckedAdd;
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct IssuanceInfo {
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

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// MGA currency to check total_issuance
		type NativeCurrencyId: Get<TokenId>;
		/// Tokens
		type Tokens: MultiTokenCurrencyExtended<Self::AccountId>;
		/// Number of blocks per session/round
		#[pallet::constant]
		type BlocksPerRound: Get<u32>;
		/// Number of sessions to store issuance history for
		#[pallet::constant]
		type HistoryLimit: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_issuance_config)]
	pub type IssuanceConfigStore<T: Config> = StorageValue<_, IssuanceInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_session_issuance)]
	pub type SessionIssuance<T: Config> = StorageMap<_, Twox64Concat, u32, Option<(Balance, Balance, Balance)>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub issuance_config: IssuanceInfo
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
			assert!(self.issuance_config.linear_issuance_blocks > T::BlocksPerRound::get());
			assert_ne!(T::BlocksPerRound::get(), u32::zero());
			IssuanceConfigStore::<T>::put(self.issuance_config.clone());
			Pallet::<T>::calculate_and_store_round_issuance(0u32).expect("Set issuance is not expected to fail");
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
		let _ = Pallet::<T>::calculate_and_store_round_issuance(n);
		let _ = Pallet::<T>::clear_round_issuance_history(n);
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
		let to_be_issued: Balance = issuance_config.cap - issuance_config.tge;
		let linear_issuance_sessions: u32 = issuance_config.linear_issuance_blocks / T::BlocksPerRound::get();
		let linear_issuance_per_session = to_be_issued / linear_issuance_sessions as Balance;

		let current_round_issuance: Balance;
		// We do not want issuance to overshoot
		// Sessions begin from 0 and linear_issuance_sessions is the total number of linear sessions including 0
		// So we stop before that
		if current_round < linear_issuance_sessions {
			current_round_issuance = linear_issuance_per_session;
		} else {
			let current_mga_total_issuance: Balance = T::Tokens::total_issuance(T::NativeCurrencyId::get().into()).into();
			if issuance_config.cap > current_mga_total_issuance {
				current_round_issuance = linear_issuance_per_session.min(issuance_config.cap - current_mga_total_issuance)
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
