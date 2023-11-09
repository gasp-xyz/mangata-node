use frame_support::dispatch::DispatchResult;

use crate::{schedule_rewards_calculator::ScheduleRewardsCalculator, Config, Error, Pallet};
use frame_support::pallet_prelude::*;
use sp_core::U256;
use sp_std::{
	convert::{TryFrom, TryInto},
	prelude::*,
};

// Quocient ratio in which liquidity minting curve is rising
const Q: f64 = 1.03;
// Precision used in rewards calculation rounding
const REWARDS_PRECISION: u32 = 10000;

fn calculate_q_pow(q: f64, pow: u32) -> u128 {
	libm::floor(libm::pow(q, pow as f64) * REWARDS_PRECISION as f64) as u128
}

/// Stores all the information required for non iterative rewards calculation between
/// last_checkpoint and particular subsequent block ('now' in most cases)
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct RewardInfo<Balance> {
	// amount of activated token
	pub activated_amount: Balance,
	// when doing checkpoint we need to store rewards up to this point
	pub rewards_not_yet_claimed: Balance,
	// there is no checkpoint during for claim_rewards
	pub rewards_already_claimed: Balance,
	// block number of last checkpoint
	pub last_checkpoint: u32,
	// ration betwen rewards : liquidity (as in table)
	pub pool_ratio_at_last_checkpoint: U256,
	// related to the table in the doc
	pub missing_at_last_checkpoint: U256,
}

//impl<Balance> RewardInfo<Balance>
//where
//	Balance: frame_support::traits::tokens::Balance + Into<u128> + TryFrom<u128>,

pub struct RewardsContext {
	pub current_time: u32,
	pub pool_ratio_current: U256,
}

pub struct RewardsCalculator<Curve, Balance> {
	rewards_context: RewardsContext,
	rewards_info: RewardInfo<Balance>,
	_curve: sp_std::marker::PhantomData<Curve>,
}

impl<Balance> RewardsCalculator<AsymptoticCurveRewards<Balance>, Balance> {
	pub fn mining_rewards<T: Config>(
		user: T::AccountId,
		asset_id: crate::CurrencyIdOf<T>,
	) -> sp_std::result::Result<Self, DispatchError> {
		let current_time: u32 = Pallet::<T>::get_current_rewards_time()?;
		let pool_ratio_current = Pallet::<T>::get_pool_rewards(asset_id)?;
		let default_rewards = RewardInfo {
			activated_amount: 0_u128,
			rewards_not_yet_claimed: 0_u128,
			rewards_already_claimed: 0_u128,
			last_checkpoint: current_time,
			pool_ratio_at_last_checkpoint: pool_ratio_current,
			missing_at_last_checkpoint: U256::from(0u128),
		};

		let rewards_info =
			crate::RewardsInfo::<T>::try_get(user.clone(), asset_id).unwrap_or(default_rewards);

		Ok(Self {
			rewards_context: RewardsContext {
				current_time: Pallet::<T>::get_current_rewards_time()?,
				pool_ratio_current: Pallet::<T>::get_pool_rewards(asset_id)?,
			},
			rewards_info,
			_curve: PhantomData::<AsymptoticCurveRewards>,
		})
	}
}

impl<Balance> RewardsCalculator<ConstCurveRewards<Balance>, Balance> {
	pub fn schedule_rewards<T: Config>(
		user: T::AccountId,
		asset_id: crate::CurrencyIdOf<T>,
		reward_asset_id: crate::CurrencyIdOf<T>,
	) -> sp_std::result::Result<Self, DispatchError> {
		let current_time: u32 = Pallet::<T>::get_current_rewards_time()?;
		ensure!(
			crate::RewardTokensPerPool::<T>::try_get(asset_id, reward_asset_id).is_ok(),
			crate::Error::<T>::NotAPromotedPool
		);
		let pool_ratio_current =
			ScheduleRewardsCalculator::<T>::total_rewards_for_liquidity(asset_id, reward_asset_id);

		let default_rewards = RewardInfo {
			activated_amount: 0_u128,
			rewards_not_yet_claimed: 0_u128,
			rewards_already_claimed: 0_u128,
			last_checkpoint: current_time,
			pool_ratio_at_last_checkpoint: pool_ratio_current.into(),
			missing_at_last_checkpoint: U256::from(0u128),
		};

		let rewards_info = crate::RewardsInfoForScheduleRewards::<T>::try_get(
			user.clone(),
			(asset_id, reward_asset_id),
		)
		.unwrap_or(default_rewards);

		Ok(Self {
			rewards_context: RewardsContext {
				current_time: Pallet::<T>::get_current_rewards_time()?,
				pool_ratio_current: pool_ratio_current.into(),
			},
			rewards_info,
			_curve: PhantomData::<ConstCurveRewards>,
		})
	}
}

pub trait CurveRewards {
	type Balance;
	fn calculate_curve_position(
		ctx: &RewardsContext,
		user_info: &RewardInfo<Self::Balance>,
	) -> Option<U256>;
	fn calculate_curve_rewards(
		ctx: &RewardsContext,
		user_info: &RewardInfo<Self::Balance>,
	) -> Option<Self::Balance>;
}

pub struct ConstCurveRewards<Balance>(RewardsContext, RewardInfo<Balance>);
pub struct AsymptoticCurveRewards<Balance>(RewardsContext, RewardInfo<Balance>);

impl<Balance> CurveRewards for AsymptoticCurveRewards<Balance> {
	type Balance = Balance;
	fn calculate_curve_position(
		ctx: &RewardsContext,
		user_info: &RewardInfo<Balance>,
	) -> Option<U256> {
		let time_passed = ctx.current_time.checked_sub(user_info.last_checkpoint).unwrap();
		let q_pow = calculate_q_pow(Q, time_passed);
		Some(user_info.missing_at_last_checkpoint * U256::from(REWARDS_PRECISION) / q_pow)
	}

	fn calculate_curve_rewards(
		ctx: &RewardsContext,
		user_info: &RewardInfo<Balance>,
	) -> Option<Balance> {
		let pool_rewards_ratio_new =
			ctx.pool_ratio_current.checked_sub(user_info.pool_ratio_at_last_checkpoint)?;

		let rewards_base: U256 = U256::from(user_info.activated_amount)
			.checked_mul(pool_rewards_ratio_new)?
			.checked_div(U256::from(u128::MAX))?; // always fit into u128

		let time_passed = ctx.current_time.checked_sub(user_info.last_checkpoint).unwrap();
		// .ok_or(Error::<T>::PastTimeCalculation)?;
		let mut cummulative_work = U256::from(0);
		let mut cummulative_work_max_possible_for_ratio = U256::from(1);

		if time_passed != 0 && user_info.activated_amount != 0 {
			let liquidity_assets_amount_u256: U256 = user_info.activated_amount.into();

			// whole formula: 	missing_at_last_checkpoint*106/6 - missing_at_last_checkpoint*106*precision/6/q_pow
			// q_pow is multiplied by precision, thus there needs to be *precision in numenator as well

			cummulative_work_max_possible_for_ratio =
				liquidity_assets_amount_u256.checked_mul(U256::from(time_passed))?;

			// whole formula: 	missing_at_last_checkpoint*Q*100/(Q*100-100) - missing_at_last_checkpoint*Q*100/(Q*100-100)*REWARDS_PRECISION/q_pow
			// q_pow is multiplied by precision, thus there needs to be *precision in numenator as well
			let base = user_info
				.missing_at_last_checkpoint
				.checked_mul(U256::from(libm::floor(Q * 100_f64) as u128))?
				.checked_div(U256::from(libm::floor(Q * 100_f64 - 100_f64) as u128))?;

			let q_pow = calculate_q_pow(Q, time_passed.checked_add(1)?);

			let cummulative_missing_new = base -
				base * U256::from(REWARDS_PRECISION) / q_pow -
				user_info.missing_at_last_checkpoint;

			cummulative_work =
				cummulative_work_max_possible_for_ratio.checked_sub(cummulative_missing_new)?
		}

		rewards_base
			.checked_mul(cummulative_work)?
			.checked_div(cummulative_work_max_possible_for_ratio)?
			.try_into()
			.ok()
	}
}

impl<Balance> CurveRewards for ConstCurveRewards<Balance> {
	type Balance = Balance;
	fn calculate_curve_position(
		ctx: &RewardsContext,
		user_info: &RewardInfo<Balance>,
	) -> Option<U256> {
		Some(U256::from(0))
	}

	fn calculate_curve_rewards(
		ctx: &RewardsContext,
		user_info: &RewardInfo<Balance>,
	) -> Option<Balance> {
		let pool_rewards_ratio_new =
			ctx.pool_ratio_current.checked_sub(user_info.pool_ratio_at_last_checkpoint)?;

		let rewards_base: U256 = U256::from(user_info.activated_amount)
			.checked_mul(pool_rewards_ratio_new)?
			.checked_div(U256::from(u128::MAX))?; // always fit into u128

		rewards_base.try_into().ok()
	}
}

#[derive(Debug)]
pub enum RewardsCalcError {
	CheckpointMathError,
}

impl<T: Config> Into<Error<T>> for RewardsCalcError {
	fn into(self) -> Error<T> {
		Error::<T>::LiquidityCheckpointMathError
	}
}

impl<T: CurveRewards, Balance> RewardsCalculator<T, Balance> {
	pub fn activate_more(
		self,
		liquidity_assets_added: Balance,
	) -> sp_std::result::Result<RewardInfo<Balance>, RewardsCalcError> {
		let activated_amount = self
			.rewards_info
			.activated_amount
			.checked_add(liquidity_assets_added)
			.ok_or(RewardsCalcError::CheckpointMathError)?;

		let missing_at_last_checkpoint =
			T::calculate_curve_position(&self.rewards_context, &self.rewards_info)
				.and_then(|v| v.checked_add(U256::from(liquidity_assets_added)))
				.ok_or(RewardsCalcError::CheckpointMathError)?;

		let user_current_rewards = self.calculate_rewards_impl()?;

		let rewards_not_yet_claimed = user_current_rewards
			.checked_add(self.rewards_info.rewards_not_yet_claimed)
			.and_then(|v| v.checked_sub(self.rewards_info.rewards_already_claimed))
			.ok_or(RewardsCalcError::CheckpointMathError)?;

		Ok(RewardInfo {
			activated_amount,
			pool_ratio_at_last_checkpoint: self.rewards_context.pool_ratio_current,
			rewards_already_claimed: 0_u128,
			missing_at_last_checkpoint,
			rewards_not_yet_claimed,
			last_checkpoint: self.rewards_context.current_time,
		})
	}

	pub fn activate_less(
		self,
		liquidity_assets_removed: Balance,
	) -> sp_std::result::Result<RewardInfo<Balance>, RewardsCalcError> {
		let activated_amount = self
			.rewards_info
			.activated_amount
			.checked_sub(liquidity_assets_removed)
			.ok_or(RewardsCalcError::CheckpointMathError)?;

		let missing_at_checkpoint_new =
			T::calculate_curve_position(&self.rewards_context, &self.rewards_info)
				.ok_or(RewardsCalcError::CheckpointMathError)?;

		let activated_amount_new = self
			.rewards_info
			.activated_amount
			.checked_sub(liquidity_assets_removed)
			.ok_or(RewardsCalcError::CheckpointMathError)?;
		let missing_at_checkpoint_after_burn = U256::from(activated_amount_new)
			.checked_mul(missing_at_checkpoint_new)
			.and_then(|v| v.checked_div(self.rewards_info.activated_amount.into()))
			.ok_or(RewardsCalcError::CheckpointMathError)?;

		let user_current_rewards = self.calculate_rewards_impl()?;

		let total_available_rewards = user_current_rewards
			.checked_add(self.rewards_info.rewards_not_yet_claimed)
			.and_then(|v| v.checked_sub(self.rewards_info.rewards_already_claimed))
			.ok_or(RewardsCalcError::CheckpointMathError)?;

		Ok(RewardInfo {
			activated_amount,
			pool_ratio_at_last_checkpoint: self.rewards_context.pool_ratio_current,
			rewards_already_claimed: 0_u128,
			missing_at_last_checkpoint: missing_at_checkpoint_after_burn,
			rewards_not_yet_claimed: total_available_rewards,
			last_checkpoint: self.rewards_context.current_time,
		})
	}

	pub fn claim_rewards(
		self,
	) -> sp_std::result::Result<(RewardInfo<Balance>, Balance), RewardsCalcError> {
		let current_rewards = self.calculate_rewards_impl()?;

		let total_available_rewards = current_rewards
			.checked_add(self.rewards_info.rewards_not_yet_claimed)
			.and_then(|v| v.checked_sub(self.rewards_info.rewards_already_claimed))
			.ok_or(RewardsCalcError::CheckpointMathError)?;

		let mut info = self.rewards_info.clone();

		info.rewards_not_yet_claimed = 0_u128;
		info.rewards_already_claimed = current_rewards;
		Ok((info, total_available_rewards))
	}

	pub fn calculate_rewards(self) -> sp_std::result::Result<Balance, RewardsCalcError> {
		self.calculate_rewards_impl()
	}

	fn calculate_rewards_impl(&self) -> sp_std::result::Result<Balance, RewardsCalcError> {
		T::calculate_curve_rewards(&self.rewards_context, &self.rewards_info)
			.ok_or(RewardsCalcError::CheckpointMathError)
	}
}
