use core::convert::TryInto;

use crate as pos;
use frame_support::traits::Hooks;
use frame_system::{pallet_prelude::*, Config, Pallet};
use mangata_support::traits::LiquidityMiningApi;
use pos::BalanceOf;
use sp_core::Get;
use sp_runtime::Saturating;

pub type TokensOf<Test> = <Test as crate::Config>::Currency;
use orml_tokens::MultiTokenCurrencyExtended;

pub fn roll_to_next_block<T>()
where
	T: pos::Config,
	T: frame_system::Config,
{
	let new_block_number = frame_system::Pallet::<T>::block_number().saturating_add(1u32.into());
	forward_to_block::<T>(new_block_number);
}

pub fn roll_to_next_session<T>()
where
	T: pos::Config,
	T: frame_system::Config,
{
	let current_session = pos::Pallet::<T>::session_index();
	roll_to_session::<T>(current_session + 1);
}

pub fn roll_to_session<T>(n: u32)
where
	T: pos::Config,
	T: frame_system::Config,
{
	while pos::Pallet::<T>::session_index() < n {
		roll_to_next_block::<T>();
	}
}

pub fn forward_to_block<T>(n: BlockNumberFor<T>)
where
	T: pos::Config,
	T: frame_system::Config,
{
	forward_to_block_with_custom_rewards::<T>(n, 10000u128.try_into().unwrap_or_default());
}

pub fn forward_to_block_with_custom_rewards<T>(n: BlockNumberFor<T>, rewards: BalanceOf<T>)
where
	T: pos::Config,
	T: frame_system::Config,
{
	while frame_system::Pallet::<T>::block_number() < n {
		let new_block_number =
			frame_system::Pallet::<T>::block_number().saturating_add(1u32.into());
		frame_system::Pallet::<T>::set_block_number(new_block_number);

		frame_system::Pallet::<T>::on_initialize(new_block_number);
		pos::Pallet::<T>::on_initialize(new_block_number);

		if pos::Pallet::<T>::is_new_session() {
			TokensOf::<T>::mint(
				pos::Pallet::<T>::native_token_id().into(),
				&<T as crate::Config>::LiquidityMiningIssuanceVault::get().into(),
				rewards,
			)
			.unwrap();

			pos::Pallet::<T>::distribute_rewards(rewards);
		}

		pos::Pallet::<T>::on_finalize(new_block_number);
		frame_system::Pallet::<T>::on_finalize(new_block_number);
	}
}
