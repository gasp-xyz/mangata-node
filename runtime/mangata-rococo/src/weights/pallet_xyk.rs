// This file is part of Mangata.

// Copyright (C) 2020-2022 Mangata Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for pallet_xyk
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-02, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/release/mangata-node
// benchmark
// pallet
// -l=info,xyk=error,collective-mangata=warn,bootstrap=warn
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// *
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --template
// ./templates/module-weight-template.hbs
// --output
// ./benchmarks/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_xyk.
pub trait WeightInfo {
	fn create_pool() -> Weight;
	fn sell_asset() -> Weight;
	fn multiswap_sell_asset(x: u32, ) -> Weight;
	fn buy_asset() -> Weight;
	fn multiswap_buy_asset(x: u32, ) -> Weight;
	fn mint_liquidity() -> Weight;
	fn mint_liquidity_using_vesting_native_tokens() -> Weight;
	fn burn_liquidity() -> Weight;
	fn claim_rewards_v2() -> Weight;
	fn claim_rewards_all_v2() -> Weight;
	fn update_pool_promotion() -> Weight;
	fn activate_liquidity_v2() -> Weight;
	fn deactivate_liquidity_v2() -> Weight;
	fn provide_liquidity_with_conversion() -> Weight;
	fn compound_rewards() -> Weight;
}

/// Weights for pallet_xyk using the Mangata node and recommended hardware.
pub struct ModuleWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_xyk::WeightInfo for ModuleWeight<T> {
	// Storage: AssetRegistry Metadata (r:3 w:1)
	// Storage: Bootstrap ActivePair (r:1 w:0)
	// Storage: Xyk Pools (r:2 w:1)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: System Account (r:1 w:1)
	// Storage: Tokens NextCurrencyId (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Xyk LiquidityAssets (r:0 w:1)
	// Storage: Xyk LiquidityPools (r:0 w:1)
	fn create_pool() -> Weight {
		(Weight::from_ref_time(181_420_000))
			.saturating_add(T::DbWeight::get().reads(14 as u64))
			.saturating_add(T::DbWeight::get().writes(12 as u64))
	}
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk Pools (r:3 w:1)
	// Storage: Tokens Accounts (r:6 w:6)
	// Storage: System Account (r:2 w:2)
	fn sell_asset() -> Weight {
		(Weight::from_ref_time(190_780_000))
			.saturating_add(T::DbWeight::get().reads(13 as u64))
			.saturating_add(T::DbWeight::get().writes(9 as u64))
	}
	// Storage: AssetRegistry Metadata (r:3 w:0)
	// Storage: Xyk Pools (r:6 w:4)
	// Storage: Tokens Accounts (r:12 w:12)
	// Storage: System Account (r:2 w:2)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	fn multiswap_sell_asset(x: u32, ) -> Weight {
		(Weight::from_ref_time(501_840_000))
			// Standard Error: 238_647
			.saturating_add((Weight::from_ref_time(191_025_097)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(24 as u64))
			.saturating_add(T::DbWeight::get().reads((8 as u64).saturating_mul(x as u64)))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
			.saturating_add(T::DbWeight::get().writes((6 as u64).saturating_mul(x as u64)))
	}
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk Pools (r:4 w:1)
	// Storage: Tokens Accounts (r:6 w:6)
	// Storage: System Account (r:2 w:2)
	fn buy_asset() -> Weight {
		(Weight::from_ref_time(198_420_000))
			.saturating_add(T::DbWeight::get().reads(14 as u64))
			.saturating_add(T::DbWeight::get().writes(9 as u64))
	}
	// Storage: AssetRegistry Metadata (r:3 w:0)
	// Storage: Xyk Pools (r:6 w:4)
	// Storage: Tokens Accounts (r:12 w:12)
	// Storage: System Account (r:2 w:2)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	fn multiswap_buy_asset(x: u32, ) -> Weight {
		(Weight::from_ref_time(514_820_000))
			// Standard Error: 237_432
			.saturating_add((Weight::from_ref_time(196_435_381)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(24 as u64))
			.saturating_add(T::DbWeight::get().reads((8 as u64).saturating_mul(x as u64)))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
			.saturating_add(T::DbWeight::get().writes((6 as u64).saturating_mul(x as u64)))
	}
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk LiquidityAssets (r:1 w:0)
	// Storage: Xyk Pools (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	fn mint_liquidity() -> Weight {
		(Weight::from_ref_time(195_169_000))
			.saturating_add(T::DbWeight::get().reads(15 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	// Storage: Xyk LiquidityAssets (r:1 w:0)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Vesting Vesting (r:2 w:2)
	// Storage: Tokens Locks (r:2 w:2)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: Xyk Pools (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	fn mint_liquidity_using_vesting_native_tokens() -> Weight {
		(Weight::from_ref_time(226_990_000))
			.saturating_add(T::DbWeight::get().reads(14 as u64))
			.saturating_add(T::DbWeight::get().writes(11 as u64))
	}
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk LiquidityAssets (r:1 w:2)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Xyk Pools (r:1 w:2)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Xyk LiquidityPools (r:0 w:1)
	fn burn_liquidity() -> Weight {
		(Weight::from_ref_time(188_970_000))
			.saturating_add(T::DbWeight::get().reads(14 as u64))
			.saturating_add(T::DbWeight::get().writes(14 as u64))
	}
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	fn claim_rewards_v2() -> Weight {
		(Weight::from_ref_time(89_570_000))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	fn claim_rewards_all_v2() -> Weight {
		(Weight::from_ref_time(88_160_000))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:1)
	fn update_pool_promotion() -> Weight {
		(Weight::from_ref_time(29_650_000))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	fn activate_liquidity_v2() -> Weight {
		(Weight::from_ref_time(97_160_000))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	fn deactivate_liquidity_v2() -> Weight {
		(Weight::from_ref_time(91_000_000))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Xyk LiquidityPools (r:1 w:0)
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk Pools (r:4 w:1)
	// Storage: Tokens Accounts (r:7 w:7)
	// Storage: System Account (r:2 w:2)
	// Storage: Xyk LiquidityAssets (r:2 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	fn provide_liquidity_with_conversion() -> Weight {
		(Weight::from_ref_time(305_150_000))
			.saturating_add(T::DbWeight::get().reads(21 as u64))
			.saturating_add(T::DbWeight::get().writes(11 as u64))
	}
	// Storage: Xyk LiquidityPools (r:1 w:0)
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Tokens Accounts (r:8 w:8)
	// Storage: Xyk Pools (r:2 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Tokens TotalIssuance (r:2 w:2)
	// Storage: Xyk LiquidityAssets (r:2 w:0)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	fn compound_rewards() -> Weight {
		(Weight::from_ref_time(419_380_000))
			.saturating_add(T::DbWeight::get().reads(24 as u64))
			.saturating_add(T::DbWeight::get().writes(16 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: AssetRegistry Metadata (r:3 w:1)
	// Storage: Bootstrap ActivePair (r:1 w:0)
	// Storage: Xyk Pools (r:2 w:1)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: System Account (r:1 w:1)
	// Storage: Tokens NextCurrencyId (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Xyk LiquidityAssets (r:0 w:1)
	// Storage: Xyk LiquidityPools (r:0 w:1)
	fn create_pool() -> Weight {
		(Weight::from_ref_time(181_420_000))
			.saturating_add(RocksDbWeight::get().reads(14 as u64))
			.saturating_add(RocksDbWeight::get().writes(12 as u64))
	}
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk Pools (r:3 w:1)
	// Storage: Tokens Accounts (r:6 w:6)
	// Storage: System Account (r:2 w:2)
	fn sell_asset() -> Weight {
		(Weight::from_ref_time(190_780_000))
			.saturating_add(RocksDbWeight::get().reads(13 as u64))
			.saturating_add(RocksDbWeight::get().writes(9 as u64))
	}
	// Storage: AssetRegistry Metadata (r:3 w:0)
	// Storage: Xyk Pools (r:6 w:4)
	// Storage: Tokens Accounts (r:12 w:12)
	// Storage: System Account (r:2 w:2)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	fn multiswap_sell_asset(x: u32, ) -> Weight {
		(Weight::from_ref_time(501_840_000))
			// Standard Error: 238_647
			.saturating_add((Weight::from_ref_time(191_025_097)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(24 as u64))
			.saturating_add(RocksDbWeight::get().reads((8 as u64).saturating_mul(x as u64)))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
			.saturating_add(RocksDbWeight::get().writes((6 as u64).saturating_mul(x as u64)))
	}
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk Pools (r:4 w:1)
	// Storage: Tokens Accounts (r:6 w:6)
	// Storage: System Account (r:2 w:2)
	fn buy_asset() -> Weight {
		(Weight::from_ref_time(198_420_000))
			.saturating_add(RocksDbWeight::get().reads(14 as u64))
			.saturating_add(RocksDbWeight::get().writes(9 as u64))
	}
	// Storage: AssetRegistry Metadata (r:3 w:0)
	// Storage: Xyk Pools (r:6 w:4)
	// Storage: Tokens Accounts (r:12 w:12)
	// Storage: System Account (r:2 w:2)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	fn multiswap_buy_asset(x: u32, ) -> Weight {
		(Weight::from_ref_time(514_820_000))
			// Standard Error: 237_432
			.saturating_add((Weight::from_ref_time(196_435_381)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(24 as u64))
			.saturating_add(RocksDbWeight::get().reads((8 as u64).saturating_mul(x as u64)))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
			.saturating_add(RocksDbWeight::get().writes((6 as u64).saturating_mul(x as u64)))
	}
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk LiquidityAssets (r:1 w:0)
	// Storage: Xyk Pools (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	fn mint_liquidity() -> Weight {
		(Weight::from_ref_time(195_169_000))
			.saturating_add(RocksDbWeight::get().reads(15 as u64))
			.saturating_add(RocksDbWeight::get().writes(10 as u64))
	}
	// Storage: Xyk LiquidityAssets (r:1 w:0)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Vesting Vesting (r:2 w:2)
	// Storage: Tokens Locks (r:2 w:2)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: Xyk Pools (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	fn mint_liquidity_using_vesting_native_tokens() -> Weight {
		(Weight::from_ref_time(226_990_000))
			.saturating_add(RocksDbWeight::get().reads(14 as u64))
			.saturating_add(RocksDbWeight::get().writes(11 as u64))
	}
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk LiquidityAssets (r:1 w:2)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Xyk Pools (r:1 w:2)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Xyk LiquidityPools (r:0 w:1)
	fn burn_liquidity() -> Weight {
		(Weight::from_ref_time(188_970_000))
			.saturating_add(RocksDbWeight::get().reads(14 as u64))
			.saturating_add(RocksDbWeight::get().writes(14 as u64))
	}
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	fn claim_rewards_v2() -> Weight {
		(Weight::from_ref_time(89_570_000))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	fn claim_rewards_all_v2() -> Weight {
		(Weight::from_ref_time(88_160_000))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:1)
	fn update_pool_promotion() -> Weight {
		(Weight::from_ref_time(29_650_000))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	fn activate_liquidity_v2() -> Weight {
		(Weight::from_ref_time(97_160_000))
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	fn deactivate_liquidity_v2() -> Weight {
		(Weight::from_ref_time(91_000_000))
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Xyk LiquidityPools (r:1 w:0)
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk Pools (r:4 w:1)
	// Storage: Tokens Accounts (r:7 w:7)
	// Storage: System Account (r:2 w:2)
	// Storage: Xyk LiquidityAssets (r:2 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	fn provide_liquidity_with_conversion() -> Weight {
		(Weight::from_ref_time(305_150_000))
			.saturating_add(RocksDbWeight::get().reads(21 as u64))
			.saturating_add(RocksDbWeight::get().writes(11 as u64))
	}
	// Storage: Xyk LiquidityPools (r:1 w:0)
	// Storage: AssetRegistry Metadata (r:2 w:0)
	// Storage: Xyk RewardsInfo (r:1 w:1)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Tokens Accounts (r:8 w:8)
	// Storage: Xyk Pools (r:2 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Tokens TotalIssuance (r:2 w:2)
	// Storage: Xyk LiquidityAssets (r:2 w:0)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	// Storage: Xyk LiquidityMiningActivePoolV2 (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	fn compound_rewards() -> Weight {
		(Weight::from_ref_time(419_380_000))
			.saturating_add(RocksDbWeight::get().reads(24 as u64))
			.saturating_add(RocksDbWeight::get().writes(16 as u64))
	}
}
