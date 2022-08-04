// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

use frame_support::{
	codec::{Decode, Encode},
	traits::{Get, Imbalance},
};
use mangata_primitives::{Balance, TokenId};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended};
use pallet_vesting_mangata::MultiTokenVestingSchedule;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{CheckedAdd, CheckedSub, One, Zero},
	Perbill, Percent, RuntimeDebug,
};
use sp_std::{convert::TryInto, prelude::*};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct IssuanceInfo {
	// Max number of MGA to target
	pub cap: Balance,
	// MGA created at token generation event
	// We aasume that there is only one tge
	pub issuance_at_init: Balance,
	// Time between which the total issuance of MGA grows to cap, as number of blocks
	pub linear_issuance_blocks: u32,
	// The split of issuance assgined to liquidity_mining
	pub liquidity_mining_split: Perbill,
	// The split of issuance assgined to staking
	pub staking_split: Perbill,
	// The total mga allocated to crowdloan rewards
	pub total_crowdloan_allocation: Balance,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct TgeInfo<A> {
	// The tge target
	pub who: A,
	// Amount distributed at tge
	pub amount: Balance,
}

pub trait PoolPromoteApi {
	/// Returns true if pool was promoted, false if it has been promoted already
	fn promote_pool(liquidity_token_id: TokenId) -> bool;
	fn unpromote_pool(liquidity_token_id: TokenId) -> bool;

	/// Returns available reward for pool
	fn get_pool_rewards_v2(liquidity_token_id: TokenId) -> Option<Balance>;

	fn len_v2() -> usize;

	//REWARDS V1 to be removed
	fn claim_pool_rewards(liquidity_token_id: TokenId, claimed_amount: Balance) -> bool;
	//REWARDS V1 to be removed
	fn get_pool_rewards(liquidity_token_id: TokenId) -> Option<Balance>;
	//REWARDS V1 to be removed
	fn len() -> usize;
}

/// Weight functions needed for pallet_xyk.
pub trait WeightInfo {
	fn init_issuance_config() -> Weight;
	fn finalize_tge() -> Weight;
	fn execute_tge(x: u32) -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	fn init_issuance_config() -> Weight {
		50_642_000 as Weight
	}
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	fn finalize_tge() -> Weight {
		50_830_000 as Weight
	}
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn execute_tge(l: u32) -> Weight {
		(52_151_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((130_000 as Weight).saturating_mul(l as Weight))
	}
}

pub trait ActivedPoolQueryApi {
	fn get_pool_activate_amount(liquidity_token_id: TokenId) -> Option<Balance>;
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
		/// The account id that holds the staking issuance
		type StakingIssuanceVault: Get<Self::AccountId>;
		#[pallet::constant]
		/// The total mga allocated for crowdloans
		type TotalCrowdloanAllocation: Get<u128>;
		#[pallet::constant]
		/// The maximum amount of Mangata tokens
		type ImmediateTGEReleasePercent: Get<Percent>;
		#[pallet::constant]
		/// The maximum amount of Mangata tokens
		type IssuanceCap: Get<Balance>;
		#[pallet::constant]
		/// The number of blocks the issuance is linear
		type LinearIssuanceBlocks: Get<u32>;
		#[pallet::constant]
		/// The split of issuance for liquidity mining rewards
		type LiquidityMiningSplit: Get<Perbill>;
		#[pallet::constant]
		/// The split of issuance for staking rewards
		type StakingSplit: Get<Perbill>;
		#[pallet::constant]
		/// The number of blocks the tge tokens vest for
		type TGEReleasePeriod: Get<u32>;
		#[pallet::constant]
		/// The block at which the tge tokens begin to vest
		type TGEReleaseBegin: Get<u32>;
		/// The vesting pallet
		type VestingProvider: MultiTokenVestingSchedule<
			Self::AccountId,
			Currency = Self::Tokens,
			Moment = Self::BlockNumber,
		>;
		type WeightInfo: WeightInfo;

		type ActivedPoolQueryApiType: ActivedPoolQueryApi;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_issuance_config)]
	pub type IssuanceConfigStore<T: Config> = StorageValue<_, IssuanceInfo, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_tge_total)]
	pub type TGETotal<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_tge_finalized)]
	pub type IsTGEFinalized<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_session_issuance)]
	pub type SessionIssuance<T: Config> =
		StorageMap<_, Twox64Concat, u32, Option<(Balance, Balance)>, ValueQuery>;

	//to be removed
	#[pallet::storage]
	#[pallet::getter(fn get_promoted_pools_rewards)]
	pub type PromotedPoolsRewards<T: Config> =
		StorageMap<_, Twox64Concat, TokenId, Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_promoted_pools_rewards_v2)]
	pub type PromotedPoolsRewardsV2<T: Config> =
		StorageMap<_, Twox64Concat, TokenId, Balance, ValueQuery>;

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// The issuance config has already been initialized
		IssuanceConfigAlreadyInitialized,
		/// The issuance config has not been initialized
		IssuanceConfigNotInitialized,
		/// TGE must be finalized before issuance config is inti
		TGENotFinalized,
		/// The TGE is already finalized
		TGEIsAlreadyFinalized,
		/// The issuance config is invalid
		IssuanceConfigInvalid,
		/// An underflow or an overflow has occured
		MathError,
		/// unknown pool
		UnknownPool,
	}

	// XYK extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::init_issuance_config())]
		pub fn init_issuance_config(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			Self::do_init_issuance_config()
		}

		#[pallet::weight(T::WeightInfo::finalize_tge())]
		pub fn finalize_tge(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(!IsTGEFinalized::<T>::get(), Error::<T>::TGEIsAlreadyFinalized);

			IsTGEFinalized::<T>::put(true);

			Pallet::<T>::deposit_event(Event::TGEFinalized);

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::execute_tge(tge_infos.len() as u32))]
		pub fn execute_tge(
			origin: OriginFor<T>,
			tge_infos: Vec<TgeInfo<T::AccountId>>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(!IsTGEFinalized::<T>::get(), Error::<T>::TGEIsAlreadyFinalized);

			ensure!(!T::TGEReleasePeriod::get().is_zero(), Error::<T>::MathError);

			let lock_percent: Percent = Percent::from_percent(100)
				.checked_sub(&T::ImmediateTGEReleasePercent::get())
				.ok_or(Error::<T>::MathError)?;

			for tge_info in tge_infos {
				let locked: Balance = (lock_percent * tge_info.amount).max(One::one());
				let per_block: Balance =
					(locked / T::TGEReleasePeriod::get() as Balance).max(One::one());

				if T::VestingProvider::can_add_vesting_schedule(
					&tge_info.who,
					locked.into(),
					per_block.into(),
					T::TGEReleaseBegin::get().into(),
					T::NativeCurrencyId::get().into(),
				)
				.is_ok()
				{
					let imb = T::Tokens::deposit_creating(
						T::NativeCurrencyId::get().into(),
						&tge_info.who.clone(),
						tge_info.amount.into(),
					);

					if !tge_info.amount.is_zero() && imb.peek().is_zero() {
						Pallet::<T>::deposit_event(Event::TGEInstanceFailed(tge_info));
					} else {
						let _ = T::VestingProvider::add_vesting_schedule(
							&tge_info.who,
							locked.into(),
							per_block.into(),
							T::TGEReleaseBegin::get().into(),
							T::NativeCurrencyId::get().into(),
						);
						TGETotal::<T>::mutate(|v| *v = v.saturating_add(tge_info.amount));
						Pallet::<T>::deposit_event(Event::TGEInstanceSucceeded(tge_info));
					}
				} else {
					Pallet::<T>::deposit_event(Event::TGEInstanceFailed(tge_info));
				}
			}

			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Issuance for upcoming session issued
		SessionIssuanceIssued(u32, Balance, Balance),
		/// Issuance for upcoming session calculated and recorded
		SessionIssuanceRecorded(u32, Balance, Balance),
		/// Issuance configuration has been finalized
		IssuanceConfigInitialized(IssuanceInfo),
		/// TGE has been finalized
		TGEFinalized,
		/// A TGE instance has failed
		TGEInstanceFailed(TgeInfo<T::AccountId>),
		/// A TGE instance has succeeded
		TGEInstanceSucceeded(TgeInfo<T::AccountId>),
	}
}

pub trait ComputeIssuance {
	/// should be only used for testing purposes
	fn initialize() {}
	fn compute_issuance(n: u32);
}

impl<T: Config> ComputeIssuance for Pallet<T> {
	/// should be only used for testing purposes
	fn initialize() {
		IsTGEFinalized::<T>::put(true);
		Self::do_init_issuance_config().unwrap();
	}

	fn compute_issuance(n: u32) {
		let _ = Pallet::<T>::calculate_and_store_round_issuance(n);
		let _ = Pallet::<T>::clear_round_issuance_history(n);
	}
}

impl<T: Config> PoolPromoteApi for Pallet<T> {
	fn promote_pool(liquidity_token_id: TokenId) -> bool {
		if PromotedPoolsRewardsV2::<T>::contains_key(liquidity_token_id) {
			false
		} else {
			PromotedPoolsRewardsV2::<T>::insert(liquidity_token_id, 0);
			true
		}
	}

	fn get_pool_rewards_v2(liquidity_token_id: TokenId) -> Option<Balance> {
		PromotedPoolsRewardsV2::<T>::try_get(liquidity_token_id).ok()
	}

	fn unpromote_pool(liquidity_token_id: TokenId) -> bool {
		if PromotedPoolsRewardsV2::<T>::contains_key(liquidity_token_id) {
			true
		} else {
			PromotedPoolsRewardsV2::<T>::remove(liquidity_token_id);
			false
		}
	}

	// TODO
	// Specifically account for this in benchmarking
	fn len() -> usize {
		PromotedPoolsRewards::<T>::iter_keys().count()
	}

	fn len_v2() -> usize {
		PromotedPoolsRewardsV2::<T>::iter_keys().count()
	}

	//REWARDS V1 to be removed
	fn get_pool_rewards(liquidity_token_id: TokenId) -> Option<Balance> {
		PromotedPoolsRewards::<T>::try_get(liquidity_token_id).ok()
	}

	//REWARDS V1 to be removed
	fn claim_pool_rewards(liquidity_token_id: TokenId, claimed_amount: Balance) -> bool {
		PromotedPoolsRewards::<T>::try_mutate(liquidity_token_id, |rewards| {
			if let Some(val) = rewards.checked_sub(claimed_amount) {
				*rewards = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.is_ok()
	}
}

pub trait ProvideTotalCrowdloanRewardAllocation {
	fn get_total_crowdloan_allocation() -> Option<Balance>;
}

impl<T: Config> ProvideTotalCrowdloanRewardAllocation for Pallet<T> {
	fn get_total_crowdloan_allocation() -> Option<Balance> {
		match IssuanceConfigStore::<T>::get() {
			Some(issuance_config) => Some(issuance_config.total_crowdloan_allocation),
			None => None,
		}
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
	pub fn do_init_issuance_config() -> DispatchResultWithPostInfo {
		ensure!(
			IssuanceConfigStore::<T>::get().is_none(),
			Error::<T>::IssuanceConfigAlreadyInitialized
		);
		ensure!(IsTGEFinalized::<T>::get(), Error::<T>::TGENotFinalized);

		let issuance_config: IssuanceInfo = IssuanceInfo {
			cap: T::IssuanceCap::get(),
			issuance_at_init: T::Tokens::total_issuance(T::NativeCurrencyId::get().into()).into(),
			linear_issuance_blocks: T::LinearIssuanceBlocks::get(),
			liquidity_mining_split: T::LiquidityMiningSplit::get(),
			staking_split: T::StakingSplit::get(),
			total_crowdloan_allocation: T::TotalCrowdloanAllocation::get(),
		};

		Pallet::<T>::build_issuance_config(issuance_config.clone())?;

		Pallet::<T>::deposit_event(Event::IssuanceConfigInitialized(issuance_config));

		Ok(().into())
	}

	pub fn build_issuance_config(issuance_config: IssuanceInfo) -> DispatchResult {
		ensure!(
			issuance_config
				.liquidity_mining_split
				.checked_add(&issuance_config.staking_split)
				.ok_or(Error::<T>::IssuanceConfigInvalid)? ==
				Perbill::from_percent(100),
			Error::<T>::IssuanceConfigInvalid
		);
		ensure!(
			issuance_config.cap >=
				issuance_config
					.issuance_at_init
					.checked_add(issuance_config.total_crowdloan_allocation)
					.ok_or(Error::<T>::IssuanceConfigInvalid)?,
			Error::<T>::IssuanceConfigInvalid
		);
		ensure!(
			issuance_config.linear_issuance_blocks != u32::zero(),
			Error::<T>::IssuanceConfigInvalid
		);
		ensure!(
			issuance_config.linear_issuance_blocks > T::BlocksPerRound::get(),
			Error::<T>::IssuanceConfigInvalid
		);
		ensure!(T::BlocksPerRound::get() != u32::zero(), Error::<T>::IssuanceConfigInvalid);
		IssuanceConfigStore::<T>::put(issuance_config.clone());
		Ok(())
	}

	pub fn calculate_and_store_round_issuance(current_round: u32) -> DispatchResult {
		let issuance_config =
			IssuanceConfigStore::<T>::get().ok_or(Error::<T>::IssuanceConfigNotInitialized)?;
		let to_be_issued: Balance = issuance_config
			.cap
			.checked_sub(issuance_config.issuance_at_init)
			.ok_or(Error::<T>::MathError)?
			.checked_sub(issuance_config.total_crowdloan_allocation)
			.ok_or(Error::<T>::MathError)?;
		let linear_issuance_sessions: u32 = issuance_config
			.linear_issuance_blocks
			.checked_div(T::BlocksPerRound::get())
			.ok_or(Error::<T>::MathError)?;
		let linear_issuance_per_session = to_be_issued
			.checked_div(linear_issuance_sessions as Balance)
			.ok_or(Error::<T>::MathError)?;

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
				current_round_issuance = linear_issuance_per_session.min(
					issuance_config
						.cap
						.checked_sub(current_mga_total_issuance)
						.ok_or(Error::<T>::MathError)?,
				)
			} else {
				current_round_issuance = Zero::zero();
			}
		}

		let liquidity_mining_issuance =
			issuance_config.liquidity_mining_split * current_round_issuance;

		let staking_issuance = issuance_config.staking_split * current_round_issuance;

		let promoted_pools_count = <Self as PoolPromoteApi>::len_v2();

		let liquidity_mining_issuance_per_pool = if promoted_pools_count == 0 {
			liquidity_mining_issuance
		} else {
			liquidity_mining_issuance / promoted_pools_count as u128
		};

		// TODO remove later

		// benchmark with max of X prom pools
		for (liquidity_token_id, v) in PromotedPoolsRewardsV2::<T>::iter() {
			let activated_amount =
				T::ActivedPoolQueryApiType::get_pool_activate_amount(liquidity_token_id)
					.ok_or_else(|| DispatchError::from(Error::<T>::UnknownPool))?;

			let rewards_per_liquidity = liquidity_mining_issuance_per_pool
				.checked_mul(10000)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathError))?
				.checked_div(activated_amount)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathError))?
				.checked_add(v)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathError))?;

			PromotedPoolsRewardsV2::<T>::insert(liquidity_token_id, rewards_per_liquidity);
		}

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
