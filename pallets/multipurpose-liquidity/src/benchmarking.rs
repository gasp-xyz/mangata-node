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

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use orml_tokens::MultiTokenCurrencyExtended;

use crate::Pallet as MultiPurposeLiquidity;

benchmarks! {

	reserve_vesting_liquidity_tokens{
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount: BalanceOf<T> = 2_000_000__u32.into();
		let asset_id_1 = <T as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let asset_id_2 = <T as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let asset_id = asset_id_2 + 1_u32.into();

		<T as Config>::Xyk::create_pool(caller.clone(), asset_id_1, initial_amount, asset_id_2, initial_amount).unwrap();

		let locked_amount: BalanceOf<T> = 500_000__u32.into();
		let lock_ending_block_as_balance: BalanceOf<T> = 1_000__u32.into();

		let reserve_amount: BalanceOf<T> = 200_000__u32.into();

		// Assuming max locks is 50
		// Let's add 49 dummy ones for worst case

		let n = 49;
		let dummy_lock_amount: BalanceOf<T> = 1000u32.into();
		let dummy_end_block: BalanceOf<T> = 10_u32.into();

		for _ in 0..n{
			<T as Config>::VestingProvider::lock_tokens(&caller, asset_id, dummy_lock_amount, None, dummy_end_block).unwrap();
		}
		<T as Config>::VestingProvider::lock_tokens(&caller, asset_id, locked_amount, None, lock_ending_block_as_balance).unwrap();
		let now: BlockNumberFor<T> = <frame_system::Pallet<T>>::block_number();

	}: {assert_ok!(MultiPurposeLiquidity::<T>::reserve_vesting_liquidity_tokens(RawOrigin::Signed(caller.clone().into()).into(), asset_id, reserve_amount));}
	verify{
		assert_eq!(<T as Config>::Tokens::locked_balance(asset_id, &caller), 343600_u32.into());
		assert_eq!(<T as Config>::Tokens::reserved_balance(asset_id, &caller), 200000_u32.into());
		assert_eq!(MultiPurposeLiquidity::<T>::get_reserve_status(caller.clone(), asset_id).relock_amount, reserve_amount);
		assert_eq!(MultiPurposeLiquidity::<T>::get_relock_status(caller, asset_id)[0], RelockStatusInfo{amount: reserve_amount, starting_block: now + 1_u32.into(), ending_block_as_balance: lock_ending_block_as_balance});
	}

	unreserve_and_relock_instance{
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount: BalanceOf<T> = 2_000_000__u32.into();
		let asset_id_1 = <T as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let asset_id_2 = <T as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let asset_id = asset_id_2 + 1_u32.into();

		<T as Config>::Xyk::create_pool(caller.clone(), asset_id_1, initial_amount, asset_id_2, initial_amount).unwrap();

		let locked_amount: BalanceOf<T> = 500_000__u32.into();
		let lock_ending_block_as_balance: BalanceOf<T> = 1_000__u32.into();

		let reserve_amount: BalanceOf<T> = 200_000__u32.into();

		// Assuming max locks is 50
		// Let's add 48 dummy ones for worst case

		let n = 48;
		let dummy_lock_amount: BalanceOf<T> = 1000u32.into();
		let dummy_end_block: BalanceOf<T> = 10_u32.into();

		for _ in 0..n{
			<T as Config>::VestingProvider::lock_tokens(&caller, asset_id, dummy_lock_amount, None, dummy_end_block).unwrap();
		}
		<T as Config>::VestingProvider::lock_tokens(&caller, asset_id, locked_amount, None, lock_ending_block_as_balance).unwrap();

		let now: BlockNumberFor<T> = <frame_system::Pallet<T>>::block_number();

		MultiPurposeLiquidity::<T>::reserve_vesting_liquidity_tokens(RawOrigin::Signed(caller.clone().into()).into(), asset_id, reserve_amount).unwrap();
		assert_eq!(<T as Config>::Tokens::locked_balance(asset_id, &caller), 348000_u32.into());
		assert_eq!(<T as Config>::Tokens::reserved_balance(asset_id, &caller), 200000_u32.into());
		assert_eq!(MultiPurposeLiquidity::<T>::get_reserve_status(caller.clone(), asset_id).relock_amount, reserve_amount);
		assert_eq!(MultiPurposeLiquidity::<T>::get_relock_status(caller.clone(), asset_id)[0], RelockStatusInfo{amount: reserve_amount, starting_block: now, ending_block_as_balance: lock_ending_block_as_balance});

	}: {assert_ok!(MultiPurposeLiquidity::<T>::unreserve_and_relock_instance(RawOrigin::Signed(caller.clone().into()).into(), asset_id, 0u32));}
	verify{
		assert_eq!(<T as Config>::Tokens::locked_balance(asset_id, &caller), 542700_u32.into());
		assert_eq!(<T as Config>::Tokens::reserved_balance(asset_id, &caller), 0_u32.into());
		assert_eq!(MultiPurposeLiquidity::<T>::get_reserve_status(caller.clone(), asset_id).relock_amount,  0_u32.into());
		assert_eq!(MultiPurposeLiquidity::<T>::get_relock_status(caller, asset_id), vec![]);
	}

	// impl_benchmark_test_suite!(MultiPurposeLiquidity, crate::mock::new_test_ext(), crate::mock::Test)
}
