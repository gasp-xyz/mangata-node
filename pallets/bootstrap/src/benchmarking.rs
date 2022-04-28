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
use frame_system::RawOrigin;
use orml_tokens::MultiTokenCurrencyExtended;

use crate::Pallet as Bootstrap;

const MILION: u128 = 1_000__000_000__000_000;

benchmarks! {

	provision {

		// // NOTE: that duplicates test XYK::liquidity_rewards_claim_W
		let caller: T::AccountId = whitelisted_caller();
		// let initial_amount:mangata_primitives::Balance = 1000000000000;
        //
		// let asset_id_1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		// let asset_id_2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		// let liquidity_asset_id = asset_id_2 + 1;
        //
		// Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), 5000, asset_id_2.into(), 5000).unwrap();
        //
		// frame_system::Pallet::<T>::set_block_number(30001_u32.into());
        //
		// let rewards_to_claim = 30000;
		// let (rewards_total_user, rewards_claimed_user) = Xyk::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id, rewards_to_claim).unwrap();
		// let pool_rewards = Xyk::<T>::calculate_available_rewards_for_pool(liquidity_asset_id, rewards_to_claim).unwrap();
        //
		// assert_eq!(pool_rewards, 30000000);
		// assert_eq!(rewards_total_user, 30000000);
		// assert_eq!(rewards_claimed_user, 0);
		// assert!(LiquidityMiningUserClaimed::<T>::try_get((caller.clone(), liquidity_asset_id)).is_err());
		// assert!(LiquidityMiningPoolClaimed::<T>::try_get(liquidity_asset_id).is_err());

	}: provision(RawOrigin::Signed(caller.clone().into()), 0_u32, 0_u128)

	verify {
		// assert_eq!(
		// 	Xyk::<T>::liquidity_mining_user_claimed((caller.clone(), liquidity_asset_id)),
		// 	(rewards_claimed_user as i128) + ( rewards_to_claim as i128 )
		// );
        //
		// assert_eq!(
		// 	Xyk::<T>::liquidity_mining_pool_claimed(liquidity_asset_id),
		// 	rewards_to_claim as u128
		// );

	}

	impl_benchmark_test_suite!(Bootstrap, crate::mock::new_test_ext(), crate::mock::Test)
}
