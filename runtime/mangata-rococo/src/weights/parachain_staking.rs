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

//! Autogenerated weights for parachain_staking
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

/// Weight functions needed for parachain_staking.
pub trait WeightInfo {
	fn set_total_selected() -> Weight;
	fn set_collator_commission() -> Weight;
	fn join_candidates(x: u32, y: u32, ) -> Weight;
	fn schedule_leave_candidates(x: u32, ) -> Weight;
	fn execute_leave_candidates(x: u32, ) -> Weight;
	fn cancel_leave_candidates(x: u32, ) -> Weight;
	fn go_offline() -> Weight;
	fn go_online() -> Weight;
	fn schedule_candidate_bond_more() -> Weight;
	fn schedule_candidate_bond_less() -> Weight;
	fn execute_candidate_bond_more() -> Weight;
	fn execute_candidate_bond_less() -> Weight;
	fn cancel_candidate_bond_more() -> Weight;
	fn cancel_candidate_bond_less() -> Weight;
	fn delegate(x: u32, y: u32, ) -> Weight;
	fn schedule_leave_delegators() -> Weight;
	fn execute_leave_delegators(x: u32, ) -> Weight;
	fn cancel_leave_delegators() -> Weight;
	fn schedule_revoke_delegation() -> Weight;
	fn schedule_delegator_bond_more() -> Weight;
	fn schedule_delegator_bond_less() -> Weight;
	fn execute_revoke_delegation() -> Weight;
	fn execute_delegator_bond_more() -> Weight;
	fn execute_delegator_bond_less() -> Weight;
	fn cancel_revoke_delegation() -> Weight;
	fn cancel_delegator_bond_more() -> Weight;
	fn cancel_delegator_bond_less() -> Weight;
	fn add_staking_liquidity_token(x: u32, ) -> Weight;
	fn remove_staking_liquidity_token(x: u32, ) -> Weight;
	fn passive_session_change() -> Weight;
	fn active_session_change(x: u32, y: u32, z: u32, w: u32, ) -> Weight;
	fn payout_collator_rewards() -> Weight;
	fn payout_delegator_reward() -> Weight;
}

/// Weights for parachain_staking using the Mangata node and recommended hardware.
pub struct ModuleWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> parachain_staking::WeightInfo for ModuleWeight<T> {
	// Storage: ParachainStaking TotalSelected (r:1 w:1)
	fn set_total_selected() -> Weight {
		(Weight::from_parts(24_450_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CollatorCommission (r:1 w:1)
	fn set_collator_commission() -> Weight {
		(Weight::from_parts(24_550_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking DelegatorState (r:1 w:0)
	// Storage: ParachainStaking StakingLiquidityTokens (r:1 w:0)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn join_candidates(x: u32, y: u32, ) -> Weight {
		(Weight::from_parts(84_432_975, 0))
			// Standard Error: 6_931
			.saturating_add((Weight::from_parts(340_211, 0)).saturating_mul(x as u64))
			// Standard Error: 2_235
			.saturating_add((Weight::from_parts(84_897, 0)).saturating_mul(y as u64))
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	fn schedule_leave_candidates(x: u32, ) -> Weight {
		(Weight::from_parts(44_154_958, 0))
			// Standard Error: 4_382
			.saturating_add((Weight::from_parts(275_090, 0)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:2 w:2)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_leave_candidates(x: u32, ) -> Weight {
		(Weight::from_parts(65_541_821, 0))
			// Standard Error: 53_421
			.saturating_add((Weight::from_parts(25_563_077, 0)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().reads((3 as u64).saturating_mul(x as u64)))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
			.saturating_add(T::DbWeight::get().writes((3 as u64).saturating_mul(x as u64)))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	fn cancel_leave_candidates(x: u32, ) -> Weight {
		(Weight::from_parts(43_033_481, 0))
			// Standard Error: 4_552
			.saturating_add((Weight::from_parts(230_825, 0)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn go_offline() -> Weight {
		(Weight::from_parts(43_510_000, 0))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn go_online() -> Weight {
		(Weight::from_parts(43_790_000, 0))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:0)
	// Storage: Tokens Accounts (r:1 w:0)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_candidate_bond_more() -> Weight {
		(Weight::from_parts(57_871_000, 0))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_candidate_bond_less() -> Weight {
		(Weight::from_parts(38_240_000, 0))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	fn execute_candidate_bond_more() -> Weight {
		(Weight::from_parts(98_150_000, 0))
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	fn execute_candidate_bond_less() -> Weight {
		(Weight::from_parts(93_240_000, 0))
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	fn cancel_candidate_bond_more() -> Weight {
		(Weight::from_parts(35_820_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	fn cancel_candidate_bond_less() -> Weight {
		(Weight::from_parts(36_180_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn delegate(x: u32, y: u32, ) -> Weight {
		(Weight::from_parts(95_304_714, 0))
			// Standard Error: 6_331
			.saturating_add((Weight::from_parts(360_554, 0)).saturating_mul(x as u64))
			// Standard Error: 16_565
			.saturating_add((Weight::from_parts(330_594, 0)).saturating_mul(y as u64))
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_leave_delegators() -> Weight {
		(Weight::from_parts(37_960_000, 0))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_leave_delegators(x: u32, ) -> Weight {
		(Weight::from_parts(42_857_318, 0))
			// Standard Error: 37_023
			.saturating_add((Weight::from_parts(30_275_693, 0)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().reads((1 as u64).saturating_mul(x as u64)))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(x as u64)))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	fn cancel_leave_delegators() -> Weight {
		(Weight::from_parts(35_760_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_revoke_delegation() -> Weight {
		(Weight::from_parts(38_830_000, 0))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:0)
	// Storage: Tokens Accounts (r:1 w:0)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_delegator_bond_more() -> Weight {
		(Weight::from_parts(60_030_000, 0))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_delegator_bond_less() -> Weight {
		(Weight::from_parts(39_060_000, 0))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_revoke_delegation() -> Weight {
		(Weight::from_parts(110_710_000, 0))
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_delegator_bond_more() -> Weight {
		(Weight::from_parts(105_980_000, 0))
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_delegator_bond_less() -> Weight {
		(Weight::from_parts(105_860_000, 0))
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	fn cancel_revoke_delegation() -> Weight {
		(Weight::from_parts(37_460_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	fn cancel_delegator_bond_more() -> Weight {
		(Weight::from_parts(50_590_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	fn cancel_delegator_bond_less() -> Weight {
		(Weight::from_parts(46_480_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking StakingLiquidityTokens (r:1 w:1)
	fn add_staking_liquidity_token(x: u32, ) -> Weight {
		(Weight::from_parts(16_668_803, 0))
			// Standard Error: 760
			.saturating_add((Weight::from_parts(89_706, 0)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking StakingLiquidityTokens (r:1 w:1)
	fn remove_staking_liquidity_token(x: u32, ) -> Weight {
		(Weight::from_parts(17_006_059, 0))
			// Standard Error: 872
			.saturating_add((Weight::from_parts(92_237, 0)).saturating_mul(x as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking Round (r:1 w:0)
	fn passive_session_change() -> Weight {
		(Weight::from_parts(10_280_000, 0))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
	// Storage: ParachainStaking Round (r:1 w:1)
	// Storage: Session CurrentIndex (r:1 w:1)
	// Storage: Session QueuedChanged (r:1 w:1)
	// Storage: Session QueuedKeys (r:1 w:1)
	// Storage: Session DisabledValidators (r:1 w:0)
	// Storage: ParachainStaking Points (r:1 w:1)
	// Storage: Issuance SessionIssuance (r:1 w:1)
	// Storage: ParachainStaking CollatorCommission (r:1 w:0)
	// Storage: ParachainStaking AwardedPts (r:34 w:33)
	// Storage: ParachainStaking AtStake (r:33 w:66)
	// Storage: Tokens Accounts (r:431 w:431)
	// Storage: System Account (r:430 w:429)
	// Storage: ParachainStaking StakingLiquidityTokens (r:1 w:1)
	// Storage: Xyk LiquidityPools (r:3 w:0)
	// Storage: Xyk Pools (r:3 w:0)
	// Storage: Tokens TotalIssuance (r:4 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:0)
	// Storage: ParachainStaking TotalSelected (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:33 w:0)
	// Storage: Issuance IssuanceConfigStore (r:1 w:0)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:1)
	// Storage: Session NextKeys (r:33 w:0)
	// Storage: Aura Authorities (r:1 w:0)
	// Storage: ParachainStaking SelectedCandidates (r:0 w:1)
	// Storage: Session Validators (r:0 w:1)
	fn active_session_change(x: u32, y: u32, z: u32, w: u32, ) -> Weight {
		(Weight::from_parts(7_352_138_000, 0))
			// Standard Error: 7_580_085
			.saturating_add((Weight::from_parts(7_824_608, 0)).saturating_mul(y as u64))
			// Standard Error: 8_477_288
			.saturating_add((Weight::from_parts(1_003_102_339, 0)).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(715 as u64))
			.saturating_add(T::DbWeight::get().reads((1 as u64).saturating_mul(x as u64)))
			.saturating_add(T::DbWeight::get().reads((46 as u64).saturating_mul(z as u64)))
			.saturating_add(T::DbWeight::get().writes(376 as u64))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(y as u64)))
			.saturating_add(T::DbWeight::get().writes((49 as u64).saturating_mul(z as u64)))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(w as u64)))
	}
	fn payout_collator_rewards() -> Weight{
		Weight::from_parts(7_352_138_000, 0)
	}

	fn payout_delegator_reward() -> Weight{
		Weight::from_parts(7_352_138_000, 0)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: ParachainStaking TotalSelected (r:1 w:1)
	fn set_total_selected() -> Weight {
		(Weight::from_parts(24_450_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CollatorCommission (r:1 w:1)
	fn set_collator_commission() -> Weight {
		(Weight::from_parts(24_550_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking DelegatorState (r:1 w:0)
	// Storage: ParachainStaking StakingLiquidityTokens (r:1 w:0)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn join_candidates(x: u32, y: u32, ) -> Weight {
		(Weight::from_parts(84_432_975, 0))
			// Standard Error: 6_931
			.saturating_add((Weight::from_parts(340_211, 0)).saturating_mul(x as u64))
			// Standard Error: 2_235
			.saturating_add((Weight::from_parts(84_897, 0)).saturating_mul(y as u64))
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	fn schedule_leave_candidates(x: u32, ) -> Weight {
		(Weight::from_parts(44_154_958, 0))
			// Standard Error: 4_382
			.saturating_add((Weight::from_parts(275_090, 0)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:2 w:2)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_leave_candidates(x: u32, ) -> Weight {
		(Weight::from_parts(65_541_821, 0))
			// Standard Error: 53_421
			.saturating_add((Weight::from_parts(25_563_077, 0)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().reads((3 as u64).saturating_mul(x as u64)))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
			.saturating_add(RocksDbWeight::get().writes((3 as u64).saturating_mul(x as u64)))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	fn cancel_leave_candidates(x: u32, ) -> Weight {
		(Weight::from_parts(43_033_481, 0))
			// Standard Error: 4_552
			.saturating_add((Weight::from_parts(230_825, 0)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn go_offline() -> Weight {
		(Weight::from_parts(43_510_000, 0))
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn go_online() -> Weight {
		(Weight::from_parts(43_790_000, 0))
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:0)
	// Storage: Tokens Accounts (r:1 w:0)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_candidate_bond_more() -> Weight {
		(Weight::from_parts(57_871_000, 0))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_candidate_bond_less() -> Weight {
		(Weight::from_parts(38_240_000, 0))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	fn execute_candidate_bond_more() -> Weight {
		(Weight::from_parts(98_150_000, 0))
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	fn execute_candidate_bond_less() -> Weight {
		(Weight::from_parts(93_240_000, 0))
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	fn cancel_candidate_bond_more() -> Weight {
		(Weight::from_parts(35_820_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	fn cancel_candidate_bond_less() -> Weight {
		(Weight::from_parts(36_180_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn delegate(x: u32, y: u32, ) -> Weight {
		(Weight::from_parts(95_304_714, 0))
			// Standard Error: 6_331
			.saturating_add((Weight::from_parts(360_554, 0)).saturating_mul(x as u64))
			// Standard Error: 16_565
			.saturating_add((Weight::from_parts(330_594, 0)).saturating_mul(y as u64))
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_leave_delegators() -> Weight {
		(Weight::from_parts(37_960_000, 0))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_leave_delegators(x: u32, ) -> Weight {
		(Weight::from_parts(42_857_318, 0))
			// Standard Error: 37_023
			.saturating_add((Weight::from_parts(30_275_693, 0)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().reads((1 as u64).saturating_mul(x as u64)))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
			.saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(x as u64)))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	fn cancel_leave_delegators() -> Weight {
		(Weight::from_parts(35_760_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_revoke_delegation() -> Weight {
		(Weight::from_parts(38_830_000, 0))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:0)
	// Storage: Tokens Accounts (r:1 w:0)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_delegator_bond_more() -> Weight {
		(Weight::from_parts(60_030_000, 0))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	fn schedule_delegator_bond_less() -> Weight {
		(Weight::from_parts(39_060_000, 0))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_revoke_delegation() -> Weight {
		(Weight::from_parts(110_710_000, 0))
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_delegator_bond_more() -> Weight {
		(Weight::from_parts(105_980_000, 0))
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:1 w:1)
	// Storage: MultiPurposeLiquidity ReserveStatus (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	fn execute_delegator_bond_less() -> Weight {
		(Weight::from_parts(105_860_000, 0))
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	fn cancel_revoke_delegation() -> Weight {
		(Weight::from_parts(37_460_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	fn cancel_delegator_bond_more() -> Weight {
		(Weight::from_parts(50_590_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	fn cancel_delegator_bond_less() -> Weight {
		(Weight::from_parts(46_480_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking StakingLiquidityTokens (r:1 w:1)
	fn add_staking_liquidity_token(x: u32, ) -> Weight {
		(Weight::from_parts(16_668_803, 0))
			// Standard Error: 760
			.saturating_add((Weight::from_parts(89_706, 0)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking StakingLiquidityTokens (r:1 w:1)
	fn remove_staking_liquidity_token(x: u32, ) -> Weight {
		(Weight::from_parts(17_006_059, 0))
			// Standard Error: 872
			.saturating_add((Weight::from_parts(92_237, 0)).saturating_mul(x as u64))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainStaking Round (r:1 w:0)
	fn passive_session_change() -> Weight {
		(Weight::from_parts(10_280_000, 0))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
	}
	// Storage: ParachainStaking Round (r:1 w:1)
	// Storage: Session CurrentIndex (r:1 w:1)
	// Storage: Session QueuedChanged (r:1 w:1)
	// Storage: Session QueuedKeys (r:1 w:1)
	// Storage: Session DisabledValidators (r:1 w:0)
	// Storage: ParachainStaking Points (r:1 w:1)
	// Storage: Issuance SessionIssuance (r:1 w:1)
	// Storage: ParachainStaking CollatorCommission (r:1 w:0)
	// Storage: ParachainStaking AwardedPts (r:34 w:33)
	// Storage: ParachainStaking AtStake (r:33 w:66)
	// Storage: Tokens Accounts (r:431 w:431)
	// Storage: System Account (r:430 w:429)
	// Storage: ParachainStaking StakingLiquidityTokens (r:1 w:1)
	// Storage: Xyk LiquidityPools (r:3 w:0)
	// Storage: Xyk Pools (r:3 w:0)
	// Storage: Tokens TotalIssuance (r:4 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:0)
	// Storage: ParachainStaking TotalSelected (r:1 w:0)
	// Storage: ParachainStaking CandidateState (r:33 w:0)
	// Storage: Issuance IssuanceConfigStore (r:1 w:0)
	// Storage: Issuance PromotedPoolsRewardsV2 (r:1 w:1)
	// Storage: Session NextKeys (r:33 w:0)
	// Storage: Aura Authorities (r:1 w:0)
	// Storage: ParachainStaking SelectedCandidates (r:0 w:1)
	// Storage: Session Validators (r:0 w:1)
	fn active_session_change(x: u32, y: u32, z: u32, w: u32, ) -> Weight {
		(Weight::from_parts(7_352_138_000, 0))
			// Standard Error: 7_580_085
			.saturating_add((Weight::from_parts(7_824_608, 0)).saturating_mul(y as u64))
			// Standard Error: 8_477_288
			.saturating_add((Weight::from_parts(1_003_102_339, 0)).saturating_mul(z as u64))
			.saturating_add(RocksDbWeight::get().reads(715 as u64))
			.saturating_add(RocksDbWeight::get().reads((1 as u64).saturating_mul(x as u64)))
			.saturating_add(RocksDbWeight::get().reads((46 as u64).saturating_mul(z as u64)))
			.saturating_add(RocksDbWeight::get().writes(376 as u64))
			.saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(y as u64)))
			.saturating_add(RocksDbWeight::get().writes((49 as u64).saturating_mul(z as u64)))
			.saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(w as u64)))
	}

	fn payout_collator_rewards() -> Weight{
		Weight::from_parts(7_352_138_000, 0)
	}

	fn payout_delegator_reward() -> Weight{
		Weight::from_parts(7_352_138_000, 0)
	}
}
