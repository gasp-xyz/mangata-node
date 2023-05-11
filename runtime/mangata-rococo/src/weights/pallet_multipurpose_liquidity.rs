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

//! Autogenerated weights for pallet_multipurpose_liquidity
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-13, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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

/// Weight functions needed for pallet_multipurpose_liquidity.
pub trait WeightInfo {
	fn reserve_vesting_liquidity_tokens() -> Weight;
	fn unreserve_and_relock_instance() -> Weight;
}

/// Weights for pallet_multipurpose_liquidity using the Mangata node and recommended hardware.
pub struct ModuleWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multipurpose_liquidity::WeightInfo for ModuleWeight<T> {
	// Storage: Xyk LiquidityPools (r:1 w:0)
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Tokens Locks (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: MultiPurposeLiquidity RelockStatus (r:1 w:1)
	fn reserve_vesting_liquidity_tokens() -> Weight {
		(Weight::from_parts(116_880_000, 0))
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: MultiPurposeLiquidity RelockStatus (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Tokens Locks (r:1 w:1)
	fn unreserve_and_relock_instance() -> Weight {
		(Weight::from_parts(107_990_000, 0))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Xyk LiquidityPools (r:1 w:0)
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Tokens Locks (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: MultiPurposeLiquidity RelockStatus (r:1 w:1)
	fn reserve_vesting_liquidity_tokens() -> Weight {
		(Weight::from_parts(116_880_000, 0))
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: MultiPurposeLiquidity RelockStatus (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Tokens Locks (r:1 w:1)
	fn unreserve_and_relock_instance() -> Weight {
		(Weight::from_parts(107_990_000, 0))
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
}
