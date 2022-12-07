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

//! Autogenerated weights for pallet_bootstrap
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-12-01, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// /home/ubuntu/mangata-node/scripts/..//target/release/mangata-node
// benchmark
// pallet
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_bootstrap
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output
// ./benchmarks/pallet_bootstrap_weights.rs
// --template
// ./templates/module-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_bootstrap.
pub trait WeightInfo {
	fn schedule_bootstrap() -> Weight;
	fn provision() -> Weight;
	fn claim_and_activate_liquidity_tokens() -> Weight;
	fn finalize() -> Weight;
}

/// Weights for pallet_bootstrap using the Mangata node and recommended hardware.
pub struct ModuleWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bootstrap::WeightInfo for ModuleWeight<T> {
	// Storage: Bootstrap Phase (r:1 w:0)
	// Storage: Bootstrap BootstrapSchedule (r:1 w:1)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	// Storage: Xyk Pools (r:2 w:0)
	// Storage: Bootstrap PromoteBootstrapPool (r:0 w:1)
	// Storage: Bootstrap ActivePair (r:0 w:1)
	fn schedule_bootstrap() -> Weight {
		(Weight::from_ref_time(37_838_000))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Bootstrap ActivePair (r:1 w:0)
	// Storage: Bootstrap Phase (r:1 w:0)
	// Storage: Bootstrap WhitelistedAccount (r:1 w:0)
	// Storage: Bootstrap BootstrapSchedule (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Bootstrap Provisions (r:1 w:1)
	// Storage: Bootstrap Valuations (r:1 w:1)
	// Storage: Bootstrap ProvisionAccounts (r:0 w:1)
	fn provision() -> Weight {
		(Weight::from_ref_time(93_582_000))
			.saturating_add(T::DbWeight::get().reads(9 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	// Storage: Bootstrap Phase (r:1 w:0)
	// Storage: Bootstrap MintedLiquidity (r:1 w:0)
	// Storage: Bootstrap ArchivedBootstrap (r:1 w:0)
	// Storage: Bootstrap ActivePair (r:1 w:0)
	// Storage: Bootstrap ClaimedRewards (r:2 w:2)
	// Storage: Bootstrap Valuations (r:1 w:0)
	// Storage: Bootstrap Provisions (r:2 w:0)
	// Storage: Bootstrap VestedProvisions (r:2 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Bootstrap ProvisionAccounts (r:0 w:1)
	fn claim_and_activate_liquidity_tokens() -> Weight {
		(Weight::from_ref_time(142_776_000))
			.saturating_add(T::DbWeight::get().reads(14 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: Bootstrap Phase (r:1 w:1)
	// Storage: Bootstrap ProvisionAccounts (r:1 w:0)
	// Storage: Bootstrap MintedLiquidity (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:0)
	// Storage: Bootstrap BootstrapSchedule (r:1 w:1)
	// Storage: Bootstrap ArchivedBootstrap (r:1 w:1)
	// Storage: Bootstrap Provisions (r:0 w:2)
	// Storage: Bootstrap Valuations (r:0 w:1)
	// Storage: Bootstrap ClaimedRewards (r:0 w:2)
	// Storage: Bootstrap PromoteBootstrapPool (r:0 w:1)
	// Storage: Bootstrap ActivePair (r:0 w:1)
	fn finalize() -> Weight {
		(Weight::from_ref_time(88_769_000))
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(11 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Bootstrap Phase (r:1 w:0)
	// Storage: Bootstrap BootstrapSchedule (r:1 w:1)
	// Storage: Tokens NextCurrencyId (r:1 w:0)
	// Storage: Xyk Pools (r:2 w:0)
	// Storage: Bootstrap PromoteBootstrapPool (r:0 w:1)
	// Storage: Bootstrap ActivePair (r:0 w:1)
	fn schedule_bootstrap() -> Weight {
		(Weight::from_ref_time(37_838_000))
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Bootstrap ActivePair (r:1 w:0)
	// Storage: Bootstrap Phase (r:1 w:0)
	// Storage: Bootstrap WhitelistedAccount (r:1 w:0)
	// Storage: Bootstrap BootstrapSchedule (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Bootstrap Provisions (r:1 w:1)
	// Storage: Bootstrap Valuations (r:1 w:1)
	// Storage: Bootstrap ProvisionAccounts (r:0 w:1)
	fn provision() -> Weight {
		(Weight::from_ref_time(93_582_000))
			.saturating_add(RocksDbWeight::get().reads(9 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	// Storage: Bootstrap Phase (r:1 w:0)
	// Storage: Bootstrap MintedLiquidity (r:1 w:0)
	// Storage: Bootstrap ArchivedBootstrap (r:1 w:0)
	// Storage: Bootstrap ActivePair (r:1 w:0)
	// Storage: Bootstrap ClaimedRewards (r:2 w:2)
	// Storage: Bootstrap Valuations (r:1 w:0)
	// Storage: Bootstrap Provisions (r:2 w:0)
	// Storage: Bootstrap VestedProvisions (r:2 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:0)
	// Storage: Bootstrap ProvisionAccounts (r:0 w:1)
	fn claim_and_activate_liquidity_tokens() -> Weight {
		(Weight::from_ref_time(142_776_000))
			.saturating_add(RocksDbWeight::get().reads(14 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: Bootstrap Phase (r:1 w:1)
	// Storage: Bootstrap ProvisionAccounts (r:1 w:0)
	// Storage: Bootstrap MintedLiquidity (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:0)
	// Storage: Bootstrap BootstrapSchedule (r:1 w:1)
	// Storage: Bootstrap ArchivedBootstrap (r:1 w:1)
	// Storage: Bootstrap Provisions (r:0 w:2)
	// Storage: Bootstrap Valuations (r:0 w:1)
	// Storage: Bootstrap ClaimedRewards (r:0 w:2)
	// Storage: Bootstrap PromoteBootstrapPool (r:0 w:1)
	// Storage: Bootstrap ActivePair (r:0 w:1)
	fn finalize() -> Weight {
		(Weight::from_ref_time(88_769_000))
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(11 as u64))
	}
}
