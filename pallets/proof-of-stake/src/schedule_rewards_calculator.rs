
use core::marker::PhantomData;
use crate::Config;
use crate::Pallet;
use crate::ScheduleRewardsPerLiquidity;
use crate::TotalActivatedLiquidityForSchedules;
use crate::ScheduleRewardsTotal;
use mangata_types::{Balance, TokenId};
use sp_core::U256;

	pub struct ScheduleRewardsCalculator<T> {
		data: PhantomData<T>
	}

	impl<T: Config> ScheduleRewardsCalculator<T> {

		pub fn update_cumulative_rewards(liquidity_asset_id: TokenId, liquidity_assets_reward: TokenId) {
			let (cumulative, idx) =
				ScheduleRewardsPerLiquidity::<T>::get((liquidity_asset_id, liquidity_assets_reward));
			if idx == (Pallet::<T>::session_index() as u64) {
			} else {
				let total_activated_liquidity =
					Self::total_activated_liquidity(liquidity_asset_id, liquidity_assets_reward);
				let total_schedule_rewards =
					Self::total_schedule_rewards(liquidity_asset_id, liquidity_assets_reward);
				if total_activated_liquidity > 0 {
					ScheduleRewardsTotal::<T>::mutate(
						(liquidity_asset_id, liquidity_assets_reward),
						|(cumulative, _, _)| {
							*cumulative = 0;
						},
					);
					let pending = (U256::from(total_schedule_rewards) * U256::from(u128::MAX))
						.checked_div(U256::from(total_activated_liquidity))
						.unwrap_or_default();
					ScheduleRewardsPerLiquidity::<T>::insert(

						(liquidity_asset_id, liquidity_assets_reward),
						(cumulative + pending, (Pallet::<T>::session_index() as u64)),
					);
				}
			}
		}

		pub fn total_rewards_for_liquidity(
			liquidity_asset_id: TokenId,
			liquidity_assets_reward: TokenId,
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
				let pending = (U256::from(total_schedule_rewards) * U256::from(u128::MAX))
					.checked_div(U256::from(total_activated_liquidity))
					.unwrap_or_default();
				cumulative + pending
			}
		}

		pub fn total_activated_liquidity(
			liquidity_asset_id: TokenId,
			liquidity_assets_reward: TokenId,
		) -> Balance {
			let (pending_negative, pending_positive, idx, cumulative) =
			TotalActivatedLiquidityForSchedules::<T>::get(
				liquidity_asset_id,
				liquidity_assets_reward,
			);
			if idx == (Pallet::<T>::session_index() as u64) {
				cumulative
			} else {
				cumulative + pending_positive - pending_negative
			}
		}

		pub fn total_schedule_rewards(
			liquidity_asset_id: TokenId,
			liquidity_assets_reward: TokenId,
		) -> Balance {
			let (pending, idx, cumulative) =
			ScheduleRewardsTotal::<T>::get((liquidity_asset_id, liquidity_assets_reward));
			if idx == (Pallet::<T>::session_index() as u64) {
				cumulative
			} else {
				cumulative + pending
			}
		}

		pub fn update_total_activated_liqudity(
			liquidity_asset_id: TokenId,
			liquidity_assets_reward: TokenId,
			diff: Balance,
			change: bool,
		) {
			// TODO: make configurable
			let session_id = Pallet::<T>::session_index() as u64;

			TotalActivatedLiquidityForSchedules::<T>::mutate(
				liquidity_asset_id,
				liquidity_assets_reward,
				|(pending_negative, pending_positive, idx, cumulative)| {
					if *idx == session_id {
						if change {
							*pending_positive += diff;
						} else {
							*pending_negative += diff;
						};
					} else {
						// NOTE: handle burn so negative diff
						*cumulative = *cumulative + *pending_positive - *pending_negative;
						if change {
							*pending_positive = diff;
							*pending_negative = 0u128;
						} else {
							*pending_positive = 0u128;
							*pending_negative = diff;
						};
						*idx = session_id;
					}
				},
			);
		}
	}
