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

//! Autogenerated weights for pallet_collective_mangata
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-12-30, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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

/// Weight functions needed for pallet_collective_mangata.
pub trait WeightInfo {
	fn set_members(m: u32, n: u32, p: u32, ) -> Weight;
	fn execute(b: u32, m: u32, ) -> Weight;
	fn propose_execute(b: u32, m: u32, ) -> Weight;
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight;
	fn vote(m: u32, ) -> Weight;
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight;
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight;
	fn close_disapproved(m: u32, p: u32, ) -> Weight;
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight;
	fn disapprove_proposal(p: u32, ) -> Weight;
}

/// Weights for pallet_collective_mangata using the Mangata node and recommended hardware.
pub struct ModuleWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective_mangata::WeightInfo for ModuleWeight<T> {
	// Storage: Council Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Voting (r:100 w:100)
	// Storage: Council Prime (r:0 w:1)
	fn set_members(m: u32, n: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(0))
			// Standard Error: 11_000
			.saturating_add((Weight::from_ref_time(8_846_000)).saturating_mul(m as u64))
			// Standard Error: 11_000
			.saturating_add((Weight::from_ref_time(63_000)).saturating_mul(n as u64))
			// Standard Error: 11_000
			.saturating_add((Weight::from_ref_time(13_168_000)).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().reads((1 as u64).saturating_mul(p as u64)))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
	}
	// Storage: Council Members (r:1 w:0)
	fn execute(b: u32, m: u32, ) -> Weight {
		(Weight::from_ref_time(34_989_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(2_000)).saturating_mul(b as u64))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(21_000)).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:0)
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		(Weight::from_ref_time(36_926_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(2_000)).saturating_mul(b as u64))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(38_000)).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalCount (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:0 w:1)
	// Storage: Council Voting (r:0 w:1)
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(46_534_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(7_000)).saturating_mul(b as u64))
			// Standard Error: 1_000
			.saturating_add((Weight::from_ref_time(21_000)).saturating_mul(m as u64))
			// Standard Error: 1_000
			.saturating_add((Weight::from_ref_time(406_000)).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Voting (r:1 w:1)
	fn vote(m: u32, ) -> Weight {
		(Weight::from_ref_time(65_318_000))
			// Standard Error: 1_000
			.saturating_add((Weight::from_ref_time(78_000)).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(66_184_000))
			// Standard Error: 3_000
			.saturating_add((Weight::from_ref_time(30_000)).saturating_mul(m as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(348_000)).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(82_398_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(5_000)).saturating_mul(b as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(58_000)).saturating_mul(m as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(400_000)).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(67_129_000))
			// Standard Error: 3_000
			.saturating_add((Weight::from_ref_time(61_000)).saturating_mul(m as u64))
			// Standard Error: 3_000
			.saturating_add((Weight::from_ref_time(353_000)).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(85_977_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(5_000)).saturating_mul(b as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(63_000)).saturating_mul(m as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(394_000)).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:0 w:1)
	// Storage: Council Voting (r:0 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	fn disapprove_proposal(p: u32, ) -> Weight {
		(Weight::from_ref_time(40_224_000))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(358_000)).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Council Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Voting (r:100 w:100)
	// Storage: Council Prime (r:0 w:1)
	fn set_members(m: u32, n: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(0))
			// Standard Error: 11_000
			.saturating_add((Weight::from_ref_time(8_846_000)).saturating_mul(m as u64))
			// Standard Error: 11_000
			.saturating_add((Weight::from_ref_time(63_000)).saturating_mul(n as u64))
			// Standard Error: 11_000
			.saturating_add((Weight::from_ref_time(13_168_000)).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().reads((1 as u64).saturating_mul(p as u64)))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
			.saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
	}
	// Storage: Council Members (r:1 w:0)
	fn execute(b: u32, m: u32, ) -> Weight {
		(Weight::from_ref_time(34_989_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(2_000)).saturating_mul(b as u64))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(21_000)).saturating_mul(m as u64))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:0)
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		(Weight::from_ref_time(36_926_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(2_000)).saturating_mul(b as u64))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(38_000)).saturating_mul(m as u64))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalCount (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:0 w:1)
	// Storage: Council Voting (r:0 w:1)
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(46_534_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(7_000)).saturating_mul(b as u64))
			// Standard Error: 1_000
			.saturating_add((Weight::from_ref_time(21_000)).saturating_mul(m as u64))
			// Standard Error: 1_000
			.saturating_add((Weight::from_ref_time(406_000)).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Voting (r:1 w:1)
	fn vote(m: u32, ) -> Weight {
		(Weight::from_ref_time(65_318_000))
			// Standard Error: 1_000
			.saturating_add((Weight::from_ref_time(78_000)).saturating_mul(m as u64))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(66_184_000))
			// Standard Error: 3_000
			.saturating_add((Weight::from_ref_time(30_000)).saturating_mul(m as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(348_000)).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(82_398_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(5_000)).saturating_mul(b as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(58_000)).saturating_mul(m as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(400_000)).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(67_129_000))
			// Standard Error: 3_000
			.saturating_add((Weight::from_ref_time(61_000)).saturating_mul(m as u64))
			// Standard Error: 3_000
			.saturating_add((Weight::from_ref_time(353_000)).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		(Weight::from_ref_time(85_977_000))
			// Standard Error: 0
			.saturating_add((Weight::from_ref_time(5_000)).saturating_mul(b as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(63_000)).saturating_mul(m as u64))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(394_000)).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalProposedTime (r:0 w:1)
	// Storage: Council Voting (r:0 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	fn disapprove_proposal(p: u32, ) -> Weight {
		(Weight::from_ref_time(40_224_000))
			// Standard Error: 2_000
			.saturating_add((Weight::from_ref_time(358_000)).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
}
