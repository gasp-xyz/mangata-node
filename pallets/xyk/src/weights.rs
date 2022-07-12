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
	fn claim_rewards() -> Weight;
	fn promote_pool() -> Weight;
	fn activate_liquidity() -> Weight;
	fn deactivate_liquidity() -> Weight;
	fn claim_rewards_v2() -> Weight;
	fn activate_liquidity_v2() -> Weight;
	fn deactivate_liquidity_v2() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_pool() -> Weight {
		(238_160_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(11 as Weight))
			.saturating_add(RocksDbWeight::get().writes(12 as Weight))
	}
	fn sell_asset() -> Weight {
		(262_953_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(11 as Weight))
			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
	}
	fn buy_asset() -> Weight {
		(274_407_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(12 as Weight))
			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
	}
	fn mint_liquidity() -> Weight {
		(270_706_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(14 as Weight))
			.saturating_add(RocksDbWeight::get().writes(11 as Weight))
	}
	fn mint_liquidity_using_vesting_native_tokens() -> Weight {
		(378_541_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(18 as Weight))
			.saturating_add(RocksDbWeight::get().writes(15 as Weight))
	}
	fn burn_liquidity() -> Weight {
		(260_718_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(14 as Weight))
			.saturating_add(RocksDbWeight::get().writes(17 as Weight))
	}
	fn claim_rewards() -> Weight {
		(156_724_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
	fn promote_pool() -> Weight {
		(36_108_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn activate_liquidity() -> Weight {
		(119_779_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn deactivate_liquidity() -> Weight {
		(133_607_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(7 as Weight))
	}
	//TODO retest
	fn claim_rewards_v2() -> Weight {
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
}
