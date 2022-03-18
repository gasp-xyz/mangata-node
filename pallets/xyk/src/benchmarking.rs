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
use orml_tokens::MultiTokenCurrencyExtended;

use crate::Pallet as Xyk;

const SEED: u32 = 0;
// existential deposit multiplier
const ED_MULTIPLIER: u32 = 10;
const MILION: u128 = 1_000__000_000__000_000;

benchmarks! {
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
			(first_asset_amount, second_asset_amount)
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

	sell_asset {
		// NOTE: duplicates test case XYK::buy_and_burn_sell_none_have_mangata_pair
		
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1000000000000000;
		let expected_amount = 0;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id.into(), 100000000000000, non_native_asset_id1.into(), 100000000000000).unwrap();
		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 100000000000000, non_native_asset_id2.into(), 100000000000000).unwrap();

	}: sell_asset(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1.into(), non_native_asset_id2.into(), 50000000000000, 0)
	verify {
		// verify only trading result as rest of the assertion is in unit test
		assert_eq!(<T as Config>::Currency::free_balance(non_native_asset_id1.into(), &caller).into(), 750000000000000);
		assert_eq!(<T as Config>::Currency::free_balance(non_native_asset_id2.into(), &caller).into(), 933266599933266);

	}

	buy_asset {
		// NOTE: duplicates test case XYK::buy_and_burn_buy_where_sold_has_mangata_pair
		
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1000000000000000;
		let expected_amount = 0;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id.into(), 100000000000000, non_native_asset_id1.into(), 100000000000000).unwrap();
		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 100000000000000, non_native_asset_id2.into(), 100000000000000).unwrap();

	}: buy_asset(RawOrigin::Signed(caller.clone().into()), non_native_asset_id2.into(), non_native_asset_id1.into(), 33266599933266, 50000000000001)
	verify {
		// verify only trading result as rest of the assertion is in unit test
		assert_eq!(<T as Config>::Currency::free_balance(non_native_asset_id1.into(), &caller).into(), 833266599933266);
		assert_eq!(<T as Config>::Currency::free_balance(non_native_asset_id2.into(), &caller).into(), 850000000000001);
	}

	mint_liquidity {
		// NOTE: duplicates test case XYK::mint_W
		
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;
		let initial_liquidity_amount = 40000000000000000000_u128 / 2_u128 + 60000000000000000000_u128 / 2_u128;

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			initial_liquidity_amount.into()
		);

	}: mint_liquidity(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1.into(), non_native_asset_id2.into(), 20000000000000000000, 30000000000000000001)
	verify {
		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			75000000000000000000_u128.into()
		);
	}



	impl_benchmark_test_suite!(Xyk, crate::mock::new_test_ext(), crate::mock::Test)
}
