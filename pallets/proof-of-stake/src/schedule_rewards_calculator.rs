use crate::{
	BalanceOf, Config, CurrencyIdOf, Pallet, ScheduleRewardsPerLiquidity, ScheduleRewardsTotal,
	SessionId, TotalActivatedLiquidityForSchedules,
};
use core::marker::PhantomData;
use frame_support::pallet_prelude::*;
use sp_arithmetic::traits::{AtLeast32BitUnsigned, Zero};
use sp_core::U256;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct ActivatedLiquidityPerSchedule<Balance> {
	pending_positive: Balance,
	pending_negative: Balance,
	pending_session_id: SessionId,
	total: Balance,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub enum LiquidityModification {
	Increase,
	Decrease,
}

impl<Balance: AtLeast32BitUnsigned> ActivatedLiquidityPerSchedule<Balance> {
	fn total(&self, now: SessionId) -> Balance {
		if now <= self.pending_session_id {
			self.total.clone()
		} else {
			self.total.clone() + self.pending_positive.clone() - self.pending_negative.clone()
		}
	}

	fn update(&mut self, now: SessionId, amount: Balance, kind: LiquidityModification) {
		if now <= self.pending_session_id {
			if kind == LiquidityModification::Increase {
				self.pending_positive += amount;
			} else {
				self.pending_negative += amount;
			}
		} else {
			self.total =
				self.total.clone() + self.pending_positive.clone() - self.pending_negative.clone();
			if kind == LiquidityModification::Increase {
				self.pending_positive = amount;
				self.pending_negative = Balance::zero();
			} else {
				self.pending_positive = Balance::zero();
				self.pending_negative = amount;
			};
			self.pending_session_id = now;
		}
	}
}

/// Information about single token rewards. Automatically accumulates new rewards into `pending`
/// and once `pending_session_id < current_session` they are moved to `total` and become ready for
/// distribution to end users
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct ScheduleRewards<Balance: AtLeast32BitUnsigned> {
	// Accumulated rewards in current or past session. Once `now > pending_session_id` they
	// should be moved to total
	pending: Balance,

	// id of the session when pending_rewards were recently updated
	pending_session_id: SessionId,

	// total amount of rewards ready for distribution
	total: Balance,
}

impl<Balance: AtLeast32BitUnsigned> ScheduleRewards<Balance> {
	pub fn provide_rewards(&mut self, now: SessionId, amount: Balance) {
		if now <= self.pending_session_id {
			self.pending += amount;
		} else {
			self.total += self.pending.clone();
			self.pending = amount;
			self.pending_session_id = now;
		}
	}

	pub fn total_rewards(&self, now: SessionId) -> Balance {
		if now <= self.pending_session_id {
			self.total.clone()
		} else {
			self.total.clone() + self.pending.clone()
		}
	}

	pub fn transfer_pending(&mut self, now: SessionId) {
		if now > self.pending_session_id {
			self.total += self.pending.clone();
			self.pending = Balance::zero();
			self.pending_session_id = now;
		}
	}

	pub fn clear(&mut self, now: SessionId) {
		self.total = Balance::zero();
		self.pending = Balance::zero();
		self.pending_session_id = now;
	}
}

pub struct ScheduleRewardsCalculator<T> {
	data: PhantomData<T>,
}

/// Class responsible for maintaining and periodically updating cumulative
/// calculations required for 3rdparty rewards
impl<T: Config> ScheduleRewardsCalculator<T> {
	/// updates cumulative number of rewards per 1 liquidity (mulipliedd by u128::MAX) because of
	/// precision.
	pub fn update_cumulative_rewards(
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_reward: CurrencyIdOf<T>,
	) {
		let session_id = Pallet::<T>::session_index();

		let (cumulative, idx) =
			ScheduleRewardsPerLiquidity::<T>::get((liquidity_asset_id, liquidity_assets_reward));
		if idx == (Pallet::<T>::session_index() as u64) {
		} else {
			let total_activated_liquidity =
				Self::total_activated_liquidity(liquidity_asset_id, liquidity_assets_reward);
			let total_schedule_rewards =
				Self::total_schedule_rewards(liquidity_asset_id, liquidity_assets_reward);
			if total_activated_liquidity > BalanceOf::<T>::zero() {
				ScheduleRewardsTotal::<T>::mutate(
					(liquidity_asset_id, liquidity_assets_reward),
					|schedule| {
						schedule.transfer_pending(session_id);
						schedule.clear(session_id);
					},
				);
				let pending = (U256::from(total_schedule_rewards.into()) * U256::from(u128::MAX))
					.checked_div(U256::from(total_activated_liquidity.into()))
					.unwrap_or_default();
				ScheduleRewardsPerLiquidity::<T>::insert(
					(liquidity_asset_id, liquidity_assets_reward),
					(cumulative + pending, (Pallet::<T>::session_index() as u64)),
				);
			}
		}
	}

	/// returns cumulative number of rewards per 1 liquidity (mulipliedd by u128::MAX) because of
	/// precision.
	pub fn total_rewards_for_liquidity(
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_reward: CurrencyIdOf<T>,
	) -> U256 {
		let (cumulative, idx) =
			ScheduleRewardsPerLiquidity::<T>::get((liquidity_asset_id, liquidity_assets_reward));
		if idx == (Pallet::<T>::session_index() as u64) {
			cumulative
		} else {
			let total_activated_liquidity =
				Self::total_activated_liquidity(liquidity_asset_id, liquidity_assets_reward);
			let total_schedule_rewards =
				Self::total_schedule_rewards(liquidity_asset_id, liquidity_assets_reward);
			let pending = (U256::from(total_schedule_rewards.into()) * U256::from(u128::MAX))
				.checked_div(U256::from(total_activated_liquidity.into()))
				.unwrap_or_default();
			cumulative + pending
		}
	}

	/// returns amount of schedule rewards that has been accumulated since last update of `ScheduleRewardsPerLiquidity`
	/// its beeing tracked only for purpose of `ScheduleRewardsPerLiquidity` calculations
	pub fn total_schedule_rewards(
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_reward: CurrencyIdOf<T>,
	) -> BalanceOf<T> {
		ScheduleRewardsTotal::<T>::get((liquidity_asset_id, liquidity_assets_reward))
			.total_rewards(Pallet::<T>::session_index())
	}

	/// returns amount of schedule rewards that has been accumulated since last update of `ScheduleRewardsPerLiquidity`
	/// its beeing tracked only for purpose of `ScheduleRewardsPerLiquidity` calculations
	pub fn update_total_activated_liqudity(
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_reward: CurrencyIdOf<T>,
		diff: BalanceOf<T>,
		change: bool,
	) {
		let session_id = Pallet::<T>::session_index();
		let kind =
			if change { LiquidityModification::Increase } else { LiquidityModification::Decrease };
		TotalActivatedLiquidityForSchedules::<T>::mutate(
			liquidity_asset_id,
			liquidity_assets_reward,
			|s| s.update(session_id, diff, kind),
		);
	}

	/// returns info about total activated liquidity per schedule
	pub fn total_activated_liquidity(
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_reward: CurrencyIdOf<T>,
	) -> BalanceOf<T> {
		let session_id = Pallet::<T>::session_index();
		TotalActivatedLiquidityForSchedules::<T>::get(liquidity_asset_id, liquidity_assets_reward)
			.total(session_id)
	}
}
