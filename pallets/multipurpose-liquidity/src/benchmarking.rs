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
use frame_support::assert_err;
use frame_system::RawOrigin;
use orml_tokens::MultiTokenCurrencyExtended;
use frame_support::{assert_ok};

use crate::Pallet as MultiPurposeLiquidity;

benchmarks! {

	reserve_vesting_liquidity_tokens{
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount: Balance = 2_000_000__u128;
		let asset_id_1: TokenId = <T as Config>::Tokens::create(&caller, initial_amount.into()).unwrap().into();
		let asset_id_2: TokenId = <T as Config>::Tokens::create(&caller, initial_amount.into()).unwrap().into();
		let asset_id: TokenId = asset_id_2 + 1;

		<T as Config>::Xyk::create_pool(caller.clone(), asset_id_1.into(), initial_amount.into(), asset_id_2.into(), initial_amount.into()).unwrap();

		let locked_amount: Balance = 500_000__u128;
		let lock_ending_block_as_balance: Balance = 1_000__u128;

		let reserve_amount: Balance = 200_000__u128;

		// Assuming max locks is 50
		// Let's add 49 dummy ones for worst case

		let n = 49;
		let dummy_lock_amount = 1000u128;
		let dummy_end_block = 10_u128;

		for _ in 0..n{
			<T as Config>::VestingProvider::lock_tokens(&caller, asset_id.into(), dummy_lock_amount.into(), dummy_end_block.into()).unwrap();
		}
		<T as Config>::VestingProvider::lock_tokens(&caller, asset_id.into(), locked_amount.into(), lock_ending_block_as_balance.into()).unwrap();

	}: {assert_ok!(MultiPurposeLiquidity::<T>::reserve_vesting_liquidity_tokens(RawOrigin::Signed(caller.clone().into()).into(), asset_id, reserve_amount));}
	verify{
		assert_eq!(<T as Config>::Tokens::locked_balance(asset_id.into(), &caller).into(), 343600);
		assert_eq!(<T as Config>::Tokens::reserved_balance(asset_id.into(), &caller).into(), 200000);
		assert_eq!(MultiPurposeLiquidity::<T>::get_reserve_status(caller.clone(), asset_id).relock_amount, reserve_amount);
		assert_eq!(MultiPurposeLiquidity::<T>::get_relock_status(caller, asset_id)[0], RelockStatusInfo{amount: reserve_amount, ending_block_as_balance: lock_ending_block_as_balance});
	}

	unreserve_and_relock_instance{
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount: Balance = 2_000_000__u128;
		let asset_id_1: TokenId = <T as Config>::Tokens::create(&caller, initial_amount.into()).unwrap().into();
		let asset_id_2: TokenId = <T as Config>::Tokens::create(&caller, initial_amount.into()).unwrap().into();
		let asset_id: TokenId = asset_id_2 + 1;

		<T as Config>::Xyk::create_pool(caller.clone(), asset_id_1.into(), initial_amount.into(), asset_id_2.into(), initial_amount.into()).unwrap();

		let locked_amount: Balance = 500_000__u128;
		let lock_ending_block_as_balance: Balance = 1_000__u128;

		let reserve_amount: Balance = 200_000__u128;

		// Assuming max locks is 50
		// Let's add 48 dummy ones for worst case

		let n = 48;
		let dummy_lock_amount = 1000u128;
		let dummy_end_block = 10_u128;

		for _ in 0..n{
			<T as Config>::VestingProvider::lock_tokens(&caller, asset_id.into(), dummy_lock_amount.into(), dummy_end_block.into()).unwrap();
		}
		<T as Config>::VestingProvider::lock_tokens(&caller, asset_id.into(), locked_amount.into(), lock_ending_block_as_balance.into()).unwrap();

		MultiPurposeLiquidity::<T>::reserve_vesting_liquidity_tokens(RawOrigin::Signed(caller.clone().into()).into(), asset_id, reserve_amount).unwrap();
		assert_eq!(<T as Config>::Tokens::locked_balance(asset_id.into(), &caller).into(), 348000);
		assert_eq!(<T as Config>::Tokens::reserved_balance(asset_id.into(), &caller).into(), 200000);
		assert_eq!(MultiPurposeLiquidity::<T>::get_reserve_status(caller.clone(), asset_id).relock_amount, reserve_amount);
		assert_eq!(MultiPurposeLiquidity::<T>::get_relock_status(caller.clone(), asset_id)[0], RelockStatusInfo{amount: reserve_amount, ending_block_as_balance: lock_ending_block_as_balance});
	
	}: {assert_ok!(MultiPurposeLiquidity::<T>::unreserve_and_relock_instance(RawOrigin::Signed(caller.clone().into()).into(), asset_id, 0u32));}
	verify{
		assert_eq!(<T as Config>::Tokens::locked_balance(asset_id.into(), &caller).into(), 542900);
		assert_eq!(<T as Config>::Tokens::reserved_balance(asset_id.into(), &caller).into(), 0);
		assert_eq!(MultiPurposeLiquidity::<T>::get_reserve_status(caller.clone(), asset_id).relock_amount, Balance::zero());
		assert_eq!(MultiPurposeLiquidity::<T>::get_relock_status(caller, asset_id), vec![]);
	}

	// impl_benchmark_test_suite!(MultiPurposeLiquidity, crate::mock::new_test_ext(), crate::mock::Test)
}
