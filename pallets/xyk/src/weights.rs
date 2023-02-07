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
	fn update_pool_promotion() -> Weight;
	fn claim_rewards_v2() -> Weight;
	fn claim_rewards_all_v2() -> Weight;
	fn activate_liquidity_v2() -> Weight;
	fn deactivate_liquidity_v2() -> Weight;
	fn rewards_migrate_v1_to_v2() -> Weight;
	fn provide_liquidity_with_conversion() -> Weight;
	fn compound_rewards() -> Weight;
	fn multiswap_sell_asset(x: u32, ) -> Weight;
	fn multiswap_buy_asset(x: u32, ) -> Weight;
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
	// Storage: AssetRegistry Metadata (r:3 w:0)
	// Storage: Xyk Pools (r:6 w:4)
	// Storage: Tokens Accounts (r:12 w:12)
	// Storage: System Account (r:2 w:2)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	fn multiswap_sell_asset(x: u32, ) -> Weight {
		(Weight::from_ref_time(426_792_000))
			// Standard Error: 1_264_113
			.saturating_add((Weight::from_ref_time(182_232_707)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads((8 as u64).saturating_mul(x as u64)))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
			.saturating_add(RocksDbWeight::get().writes((6 as u64).saturating_mul(x as u64)))
	}
	// Storage: AssetRegistry Metadata (r:3 w:0)
	// Storage: Xyk Pools (r:6 w:4)
	// Storage: Tokens Accounts (r:12 w:12)
	// Storage: System Account (r:2 w:2)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	fn multiswap_buy_asset(x: u32, ) -> Weight {
		(Weight::from_ref_time(463_541_000))
			// Standard Error: 873_994
			.saturating_add((Weight::from_ref_time(183_908_548)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads((8 as u64).saturating_mul(x as u64)))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
			.saturating_add(RocksDbWeight::get().writes((6 as u64).saturating_mul(x as u64)))
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
	fn update_pool_promotion() -> Weight {
		Weight::from_ref_time(36_108_000)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}


	//TODO retest
	fn claim_rewards_v2() -> Weight {
		Weight::from_ref_time(156_724_000)
			.saturating_add(RocksDbWeight::get().reads(8 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}

	//TODO retest
	fn claim_rewards_all_v2() -> Weight {
		Weight::from_ref_time(156_724_000)
			.saturating_add(RocksDbWeight::get().reads(8 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	//TODO retest
	fn activate_liquidity_v2() -> Weight {
		Weight::from_ref_time(119_779_000)
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	
	//TODO retest
	fn deactivate_liquidity_v2() -> Weight {
		Weight::from_ref_time(133_607_000)
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(7 as u64))
	}

	//TODO retest
	fn rewards_migrate_v1_to_v2() -> Weight {
		Weight::from_ref_time(133_607_000)
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(7 as u64))
	}

	// todo run on reference machine
	fn provide_liquidity_with_conversion() -> Weight {
		Weight::from_ref_time(275_376_000)
			.saturating_add(RocksDbWeight::get().reads(21 as u64))
			.saturating_add(RocksDbWeight::get().writes(11 as u64))
	}

	// todo run on reference machine
	fn compound_rewards() -> Weight {
		Weight::from_ref_time(220_046_000)
			.saturating_add(RocksDbWeight::get().reads(24 as u64))
			.saturating_add(RocksDbWeight::get().writes(16 as u64))
	}
}
