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

//! Autogenerated weights for pallet_issuance
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-09, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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

/// Weight functions needed for pallet_issuance.
pub trait WeightInfo {
	fn init_issuance_config() -> Weight;
	fn finalize_tge() -> Weight;
	fn execute_tge(x: u32, ) -> Weight;
}

/// Weights for pallet_issuance using the Mangata node and recommended hardware.
pub struct ModuleWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_issuance::WeightInfo for ModuleWeight<T> {
	// Storage: Issuance IssuanceConfigStore (r:1 w:1)
	// Storage: Issuance IsTGEFinalized (r:1 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:0)
	fn init_issuance_config() -> Weight {
		(Weight::from_ref_time(33_091_000))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Issuance IsTGEFinalized (r:1 w:1)
	fn finalize_tge() -> Weight {
		(Weight::from_ref_time(21_970_000))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Issuance IsTGEFinalized (r:1 w:0)
	// Storage: Vesting Vesting (r:3 w:3)
	// Storage: Tokens Accounts (r:3 w:3)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System Account (r:3 w:3)
	// Storage: Tokens Locks (r:3 w:3)
	// Storage: Issuance TGETotal (r:1 w:1)
	fn execute_tge(x: u32, ) -> Weight {
		(Weight::from_ref_time(39_957_272))
			// Standard Error: 26_196
			.saturating_add((Weight::from_ref_time(60_128_112)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().reads((4 as u64).saturating_mul(x as u64)))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
			.saturating_add(T::DbWeight::get().writes((4 as u64).saturating_mul(x as u64)))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Issuance IssuanceConfigStore (r:1 w:1)
	// Storage: Issuance IsTGEFinalized (r:1 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:0)
	fn init_issuance_config() -> Weight {
		(Weight::from_ref_time(33_091_000))
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Issuance IsTGEFinalized (r:1 w:1)
	fn finalize_tge() -> Weight {
		(Weight::from_ref_time(21_970_000))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Issuance IsTGEFinalized (r:1 w:0)
	// Storage: Vesting Vesting (r:3 w:3)
	// Storage: Tokens Accounts (r:3 w:3)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System Account (r:3 w:3)
	// Storage: Tokens Locks (r:3 w:3)
	// Storage: Issuance TGETotal (r:1 w:1)
	fn execute_tge(x: u32, ) -> Weight {
		(Weight::from_ref_time(39_957_272))
			// Standard Error: 26_196
			.saturating_add((Weight::from_ref_time(60_128_112)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().reads((4 as u64).saturating_mul(x as u64)))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
			.saturating_add(RocksDbWeight::get().writes((4 as u64).saturating_mul(x as u64)))
	}
}
