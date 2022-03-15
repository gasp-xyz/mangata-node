// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;

use frame_support::{
	codec::{Decode, Encode},
	traits::{Get, Imbalance},
};
use mangata_primitives::{Balance, TokenId};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, Perbill, RuntimeDebug};
use sp_std::prelude::*;

use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended};
use sp_runtime::traits::CheckedAdd;

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
	pub liquidity_mining_split: Perbill,
	// The split of issuance assgined to staking
	pub staking_split: Perbill,
	// The mga allocated to crowdloan rewards
	pub crowdloan_allocation: Balance,
}

/// Api to interact with pools marked as promoted
///
/// TODO add errors to functions
pub trait PoolPromoteApi {
	/// Returns true if pool was promoted, false if it has been promoted already
	fn promote_pool(liquidity_token_id: TokenId) -> bool;
	/// Returns available reward for pool
	fn get_pool_rewards(liquidity_token_id: TokenId) -> Option<Balance>;
	/// Returns available reward for pool
	fn claim_pool_rewards(liquidity_token_id: TokenId, claimed_amount: Balance) -> bool;
	/// Returns number of promoted pools
	fn len() -> usize;
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

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
		#[pallet::constant]
		/// The account id that holds the liquidity mining issuance
		type LiquidityMiningIssuanceVault: Get<Self::AccountId>;
		#[pallet::constant]
		/// The account id that holds the liquidity mining issuance
		type StakingIssuanceVault: Get<Self::AccountId>;
		#[pallet::constant]
		/// The account id that holds the liquidity mining issuance
		type CrowdloanIssuanceVault: Get<Self::AccountId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_issuance_config)]
	pub type IssuanceConfigStore<T: Config> = StorageValue<_, IssuanceInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_session_issuance)]
	pub type SessionIssuance<T: Config> =
		StorageMap<_, Twox64Concat, u32, Option<(Balance, Balance)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_promoted_pools_rewards)]
	pub type PromotedPoolsRewards<T: Config> =
		StorageMap<_, Twox64Concat, TokenId, Balance, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub issuance_config: IssuanceInfo,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { issuance_config: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			assert_eq!(
				self.issuance_config
					.liquidity_mining_split
					.checked_add(&self.issuance_config.staking_split)
					.unwrap(),
				Perbill::from_percent(100)
			);
			assert!(
				self.issuance_config.cap >=
					self.issuance_config.tge + self.issuance_config.crowdloan_allocation
			);
			assert_ne!(self.issuance_config.linear_issuance_blocks, u32::zero());
			assert!(self.issuance_config.linear_issuance_blocks > T::BlocksPerRound::get());
			assert_ne!(T::BlocksPerRound::get(), u32::zero());
			IssuanceConfigStore::<T>::put(self.issuance_config.clone());
			Pallet::<T>::calculate_and_store_round_issuance(0u32)
				.expect("Set issuance is not expected to fail");
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Issuance for upcoming session issued
		SessionIssuanceIssued(u32, Balance, Balance),
		/// Issuance for upcoming session calculated and recorded
		SessionIssuanceRecorded(u32, Balance, Balance),
	}
}

pub trait ComputeIssuance {
	fn compute_issuance(n: u32);
}

impl<T: Config> ComputeIssuance for Pallet<T> {
	fn compute_issuance(n: u32) {
		let _ = Pallet::<T>::calculate_and_store_round_issuance(n);
		let _ = Pallet::<T>::clear_round_issuance_history(n);
	}
}

impl<T: Config> PoolPromoteApi for Pallet<T> {
	fn promote_pool(liquidity_token_id: TokenId) -> bool {
		if PromotedPoolsRewards::<T>::contains_key(liquidity_token_id) {
			false
		} else {
			PromotedPoolsRewards::<T>::insert(liquidity_token_id, 0);
			true
		}
	}

	fn get_pool_rewards(liquidity_token_id: TokenId) -> Option<Balance> {
		PromotedPoolsRewards::<T>::try_get(liquidity_token_id).ok()
	}

	fn claim_pool_rewards(liquidity_token_id: TokenId, claimed_amount: Balance) -> bool {
		PromotedPoolsRewards::<T>::try_mutate(liquidity_token_id, |rewards| {
			rewards.checked_sub(claimed_amount).ok_or(())
		})
		.is_ok()
	}

	fn len() -> usize {
		PromotedPoolsRewards::<T>::iter_keys().count()
	}
}

pub trait GetIssuance {
	fn get_all_issuance(n: u32) -> Option<(Balance, Balance)>;
	fn get_liquidity_mining_issuance(n: u32) -> Option<Balance>;
	fn get_staking_issuance(n: u32) -> Option<Balance>;
}

impl<T: Config> GetIssuance for Pallet<T> {
	fn get_all_issuance(n: u32) -> Option<(Balance, Balance)> {
		SessionIssuance::<T>::get(n)
	}
	fn get_liquidity_mining_issuance(n: u32) -> Option<Balance> {
		match SessionIssuance::<T>::get(n) {
			Some((x, _)) => Some(x),
			None => None,
		}
	}
	fn get_staking_issuance(n: u32) -> Option<Balance> {
		match SessionIssuance::<T>::get(n) {
			Some((_, x)) => Some(x),
			None => None,
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn calculate_and_store_round_issuance(current_round: u32) -> DispatchResult {
		let issuance_config = IssuanceConfigStore::<T>::get();
		let to_be_issued: Balance =
			issuance_config.cap - issuance_config.tge - issuance_config.crowdloan_allocation;
		let linear_issuance_sessions: u32 =
			issuance_config.linear_issuance_blocks / T::BlocksPerRound::get();
		let linear_issuance_per_session = to_be_issued / linear_issuance_sessions as Balance;

		let current_round_issuance: Balance;
		// We do not want issuance to overshoot
		// Sessions begin from 0 and linear_issuance_sessions is the total number of linear sessions including 0
		// So we stop before that
		if current_round < linear_issuance_sessions {
			current_round_issuance = linear_issuance_per_session;
		} else {
			let current_mga_total_issuance: Balance =
				T::Tokens::total_issuance(T::NativeCurrencyId::get().into()).into();
			if issuance_config.cap > current_mga_total_issuance {
				// TODO
				// Here we assume that the crowdloan ends before linear issuance period ends
				// We could get the amount that the crowdloan rewards still need to mint and account for that
				// But that largely depends on on how the next crowdloan will be implemented
				// Not very useful for the first crowdloan, as we know that it will end before linear issuance period ends and we check for this
				current_round_issuance = linear_issuance_per_session
					.min(issuance_config.cap - current_mga_total_issuance)
			} else {
				current_round_issuance = Zero::zero();
			}
		}

		let liquidity_mining_issuance =
			issuance_config.liquidity_mining_split * current_round_issuance;
		let staking_issuance = issuance_config.staking_split * current_round_issuance;

		let promoted_pools_count = <Self as PoolPromoteApi>::len();

		// TODO: what about roundings? transfer mod to next session?

		let liquidity_mining_issuance_per_pool = if promoted_pools_count == 0 {
			liquidity_mining_issuance
		} else {
			liquidity_mining_issuance / promoted_pools_count as u128
		};

		PromotedPoolsRewards::<T>::translate(|_, v: Balance| {
			Some(v + liquidity_mining_issuance_per_pool)
		});

		{
			let liquidity_mining_issuance_issued = T::Tokens::deposit_creating(
				T::NativeCurrencyId::get().into(),
				&T::LiquidityMiningIssuanceVault::get(),
				liquidity_mining_issuance.into(),
			);
			let staking_issuance_issued = T::Tokens::deposit_creating(
				T::NativeCurrencyId::get().into(),
				&T::StakingIssuanceVault::get(),
				staking_issuance.into(),
			);
			Self::deposit_event(Event::SessionIssuanceIssued(
				current_round,
				liquidity_mining_issuance_issued.peek().into(),
				staking_issuance_issued.peek().into(),
			));
		}

		SessionIssuance::<T>::insert(
			current_round,
			Some((liquidity_mining_issuance, staking_issuance)),
		);

		Pallet::<T>::deposit_event(Event::SessionIssuanceRecorded(
			current_round,
			liquidity_mining_issuance,
			staking_issuance,
		));

		Ok(())
	}

	pub fn clear_round_issuance_history(current_round: u32) -> DispatchResult {
		if current_round >= T::HistoryLimit::get() {
			SessionIssuance::<T>::remove(current_round - T::HistoryLimit::get());
		}
		Ok(())
	}
}
