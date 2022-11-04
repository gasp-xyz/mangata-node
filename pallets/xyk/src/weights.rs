#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
/// Weight functions needed for pallet_xyk.
pub trait WeightInfo {
	fn create_pool() -> Weight;
	fn sell_asset() -> Weight;
	fn buy_asset() -> Weight;
	fn mint_liquidity() -> Weight;
	fn mint_liquidity_using_vesting_native_tokens() -> Weight;
	fn burn_liquidity() -> Weight;
	fn promote_pool() -> Weight;
	fn claim_rewards_v2() -> Weight;
	fn claim_rewards_all_v2() -> Weight;
	fn activate_liquidity_v2() -> Weight;
	fn deactivate_liquidity_v2() -> Weight;
	fn rewards_migrate_v1_to_v2() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_pool() -> Weight {
		Weight::from_ref_time(238_160_000)
			.saturating_add(RocksDbWeight::get().reads(11 as u64))
			.saturating_add(RocksDbWeight::get().writes(12 as u64))
	}
	fn sell_asset() -> Weight {
		Weight::from_ref_time(262_953_000)
			.saturating_add(RocksDbWeight::get().reads(11 as u64))
			.saturating_add(RocksDbWeight::get().writes(9 as u64))
	}
	fn buy_asset() -> Weight {
		Weight::from_ref_time(274_407_000)
			.saturating_add(RocksDbWeight::get().reads(12 as u64))
			.saturating_add(RocksDbWeight::get().writes(9 as u64))
	}
	fn mint_liquidity() -> Weight {
		Weight::from_ref_time(270_706_000)
			.saturating_add(RocksDbWeight::get().reads(14 as u64))
			.saturating_add(RocksDbWeight::get().writes(11 as u64))
	}
	fn mint_liquidity_using_vesting_native_tokens() -> Weight {
		Weight::from_ref_time(378_541_000)
			.saturating_add(RocksDbWeight::get().reads(18 as u64))
			.saturating_add(RocksDbWeight::get().writes(15 as u64))
	}
	fn burn_liquidity() -> Weight {
		Weight::from_ref_time(260_718_000)
			.saturating_add(RocksDbWeight::get().reads(14 as u64))
			.saturating_add(RocksDbWeight::get().writes(17 as u64))
	}
	fn promote_pool() -> Weight {
		Weight::from_ref_time(36_108_000)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	//TODO retest
	fn claim_rewards_v2() -> Weight {
		(156_724_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}

	//TODO retest
	fn claim_rewards_all_v2() -> Weight {
		(156_724_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
	//TODO retest
	fn activate_liquidity_v2() -> Weight {
		(119_779_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	
	//TODO retest
	fn deactivate_liquidity_v2() -> Weight {
		(133_607_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(7 as Weight))
	}

	//TODO retest
	fn rewards_migrate_v1_to_v2() -> Weight {
		(133_607_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(7 as Weight))
	}
}
