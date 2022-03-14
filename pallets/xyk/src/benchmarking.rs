// This file is part of Substrate.

// Copyright (C) 2020-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Balances pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
// use crate::mock::*;

use frame_benchmarking::{account, benchmarks, benchmarks_instance_pallet, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

use crate::Pallet as Xyk;

const SEED: u32 = 0;
// existential deposit multiplier
const ED_MULTIPLIER: u32 = 10;
const MILION: u32 = 1_000_000_000;

benchmarks! {
	// Benchmark `transfer` extrinsic with the worst possible conditions:
	// * Transfer will kill the sender account.
	// * Transfer will create the recipient account.
	create_pool {
		// let existential_deposit = T::ExistentialDeposit::get();
		let caller: T::AccountId = whitelisted_caller();
		let first_asset_amount = MILION;
		let second_asset_amount = MILION;
		let first_asset_id = <T as Config>::Currency::create(&caller, first_asset_amount.into()).unwrap();
		let second_asset_id = <T as Config>::Currency::create(&caller, second_asset_amount.into()).unwrap();
		let liquidity_asset_id = second_asset_id.into() + 1;

	}: create_pool(RawOrigin::Signed(caller.clone().into()), first_asset_id.into(), first_asset_amount.into(), second_asset_id.into(), second_asset_amount.into())
	verify {

		assert_eq!(
			Xyk::<T>::asset_pool((first_asset_id.into(), second_asset_id.into())),
			(first_asset_amount as u128, second_asset_amount as u128)
		);

		assert!(
			Xyk::<T>::liquidity_asset((first_asset_id.into(), second_asset_id.into())).is_some()
		);

		assert_eq!(
			Xyk::<T>::liquidity_pool(liquidity_asset_id),
			Some((first_asset_id.into(), second_asset_id.into()))
		);

		assert!(LiquidityMiningUser::<T>::try_get((caller.clone(), liquidity_asset_id)).is_ok());
		assert!(LiquidityMiningPool::<T>::try_get(liquidity_asset_id).is_ok());

	}

	impl_benchmark_test_suite!(Xyk, crate::mock::new_test_ext(), crate::mock::Test)
}
