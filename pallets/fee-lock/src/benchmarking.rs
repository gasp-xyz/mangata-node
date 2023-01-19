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

//! FeeLock pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_err, assert_ok, traits::tokens::currency::MultiTokenCurrency};
use frame_system::RawOrigin;
use orml_tokens::MultiTokenCurrencyExtended;
use sp_std::collections::btree_set::BTreeSet;

use crate::Pallet as FeeLock;

const MGA_TOKEN_ID: TokenId = 0;

benchmarks! {

	update_fee_lock_metadata{
		let period_length: T::BlockNumber = 1000u32.into();
		let fee_lock_amount: Balance = 1000;
		let swap_value_threshold: Balance = 1000;
		let mut whitelisted_tokens: Vec<(TokenId, Option<Balance>)> = Vec::new();
		for i in 0..<T as Config>::MaxCuratedTokens::get() {
			whitelisted_tokens.push((i, true));
		}
	}: {assert_ok!(FeeLock::<T>::update_fee_lock_metadata(RawOrigin::Root.into(), Some(period_length), Some(fee_lock_amount), Some(swap_value_threshold), Some(whitelisted_tokens)));}
	verify{
		assert_eq!(FeeLock::<T>::get_fee_lock_metadata().unwrap().period_length, period_length);
		assert_eq!(FeeLock::<T>::get_fee_lock_metadata().unwrap().fee_lock_amount, fee_lock_amount);
		assert_eq!(FeeLock::<T>::get_fee_lock_metadata().unwrap().swap_value_threshold, swap_value_threshold);
		assert_eq!(FeeLock::<T>::get_fee_lock_metadata().unwrap().whitelisted_tokens.len(), <T as Config>::MaxCuratedTokens::get() as usize);
	}

	unlock_fee{

		let caller: T::AccountId = whitelisted_caller();
		let period_length: T::BlockNumber = 10u32.into();
		let fee_lock_amount: Balance = 1000;
		let swap_value_threshold: Balance = 1000;

		let now= <frame_system::Pallet<T>>::block_number();
		let token_id = MGA_TOKEN_ID;

		if <T as Config>::Tokens::get_next_currency_id().into() > TokenId::from(MGA_TOKEN_ID){
			assert_ok!(<T as Config>::Tokens::mint(token_id.into(), &caller.clone(), 1_000_000u128.into()));
		} else {
			assert_eq!(<T as Config>::Tokens::create(&caller.clone(), 1_000_000u128.into()).unwrap().into(), token_id);
		}

		let initial_user_free_balance:Balance = <T as Config>::Tokens::free_balance(token_id.into(), &caller.clone()).into();
		let initial_user_reserved_balance:Balance = <T as Config>::Tokens::reserved_balance(token_id.into(), &caller.clone()).into();
		let initial_user_locked_balance:Balance = <T as Config>::Tokens::locked_balance(token_id.into(), &caller.clone()).into();

		assert_ok!(FeeLock::<T>::update_fee_lock_metadata(RawOrigin::Root.into(), Some(period_length), Some(fee_lock_amount), Some(swap_value_threshold), None));

		assert_eq!(FeeLock::<T>::get_fee_lock_metadata().unwrap().period_length, period_length);
		assert_eq!(FeeLock::<T>::get_fee_lock_metadata().unwrap().fee_lock_amount, fee_lock_amount);
		assert_eq!(FeeLock::<T>::get_fee_lock_metadata().unwrap().swap_value_threshold, swap_value_threshold);
		assert_eq!(FeeLock::<T>::get_fee_lock_metadata().unwrap().whitelisted_tokens.len(), 0u32 as usize);

		assert_ok!(
			<FeeLock<T> as FeeLockTriggerTrait<_>>::process_fee_lock(&caller)
		);

		assert_eq!(<T as Config>::Tokens::free_balance(token_id.into(), &caller.clone()).into(),
			initial_user_free_balance - fee_lock_amount);
		assert_eq!(<T as Config>::Tokens::reserved_balance(token_id.into(), &caller.clone()).into(),
			initial_user_reserved_balance + fee_lock_amount);
		assert_eq!(<T as Config>::Tokens::locked_balance(token_id.into(), &caller.clone()).into(),
			initial_user_locked_balance);

		assert_eq!(FeeLock::<T>::get_account_fee_lock_data(caller.clone()), AccountFeeLockDataInfo{
			total_fee_lock_amount: fee_lock_amount,
			last_fee_lock_block: now,
		});

		frame_system::Pallet::<T>::set_block_number(now + period_length);

	}: {assert_ok!(FeeLock::<T>::unlock_fee(RawOrigin::Signed(caller.clone().into()).into()));}
	verify{
		assert_eq!(<T as Config>::Tokens::free_balance(token_id.into(), &caller.clone()).into(),
			initial_user_free_balance);
		assert_eq!(<T as Config>::Tokens::reserved_balance(token_id.into(), &caller.clone()).into(),
			initial_user_reserved_balance);
		assert_eq!(<T as Config>::Tokens::locked_balance(token_id.into(), &caller.clone()).into(),
			initial_user_locked_balance);

		assert_eq!(FeeLock::<T>::get_account_fee_lock_data(caller.clone()), AccountFeeLockDataInfo{
			total_fee_lock_amount: 0,
			last_fee_lock_block: 0u32.into(),
		});
	}


	impl_benchmark_test_suite!(FeeLock, crate::mock::new_test_ext(), crate::mock::Test)
}
