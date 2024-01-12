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

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use mangata_support::traits::{ComputeIssuance, ProofOfStakeRewardsApi};
use orml_tokens::MultiTokenCurrencyExtended;
use sp_runtime::{Permill, SaturatedConversion};

use crate::Pallet as Xyk;

const MILION: u128 = 1_000__000_000__000_000;

trait ToBalance {
	fn to_balance<T: Config>(self) -> BalanceOf<T>;
}

impl ToBalance for u128 {
	fn to_balance<T: Config>(self) -> BalanceOf<T> {
		self.try_into().ok().expect("u128 should fit into Balance type")
	}
}

#[macro_export]
macro_rules! init {
	() => {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		pallet_issuance::Pallet::<T>::initialize();
	};
}

#[macro_export]
macro_rules! forward_to_next_session {
	() => {
		let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();

		let blocks_per_session: u32 = pallet_proof_of_stake::Pallet::<T>::rewards_period();
		let target_block_nr: u32;
		let target_session_nr: u32;

		if (current_block == 0_u32 || current_block == 1_u32) {
			target_session_nr = 1_u32;
			target_block_nr = blocks_per_session;
		} else {
			// to fail on user trying to manage block nr on its own
			assert!(current_block % blocks_per_session == 0);
			target_session_nr = (current_block / blocks_per_session) + 1_u32;
			target_block_nr = (target_session_nr * blocks_per_session);
		}

		frame_system::Pallet::<T>::set_block_number(target_block_nr.into());
		pallet_issuance::Pallet::<T>::compute_issuance(target_session_nr);
	};
}

benchmarks! {

   create_pool {
	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let first_asset_amount = MILION.to_balance::<T>();
	   let second_asset_amount = MILION.to_balance::<T>();
	   let first_asset_id = <T as Config>::Currency::create(&caller, first_asset_amount).unwrap();
	   let second_asset_id = <T as Config>::Currency::create(&caller, second_asset_amount).unwrap();
	   let liquidity_asset_id = second_asset_id + 1_u32.into();

   }: create_pool(RawOrigin::Signed(caller.clone().into()), first_asset_id, first_asset_amount, second_asset_id, second_asset_amount)
   verify {

	   assert_eq!(
		   Xyk::<T>::asset_pool((first_asset_id, second_asset_id)),
		   (first_asset_amount, second_asset_amount)
	   );

	   assert!(
		   Xyk::<T>::liquidity_asset((first_asset_id, second_asset_id)).is_some()
	   );

   }

   sell_asset {
	   // NOTE: duplicates test case XYK::buy_and_burn_sell_none_have_mangata_pair

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount: BalanceOf<T> = 1000000000000000.to_balance::<T>();
	   let expected_amount = BalanceOf::<T>::zero();
	   let expected_native_asset_id  = <T as Config>::NativeCurrencyId::get();
	   let native_asset_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let non_native_asset_id1 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let non_native_asset_id2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();

	   let pool_amount: BalanceOf<T> = 100_000_000_000_000.to_balance::<T>();
	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id, pool_amount, non_native_asset_id1, pool_amount).unwrap();
	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1, pool_amount, non_native_asset_id2, pool_amount).unwrap();

   }: sell_asset(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1, non_native_asset_id2, pool_amount / 2_u32.into(), 0_u32.into())
   verify {
	   // verify only trading result as rest of the assertion is in unit test
	   assert_eq!(<T as Config>::Currency::free_balance(non_native_asset_id1, &caller).into(), 750000000000000_u128);
	   assert_eq!(<T as Config>::Currency::free_balance(non_native_asset_id2, &caller).into(), 933266599933266_u128);

   }

   multiswap_sell_asset {

		// NOTE: All atomic swaps in the chain involve sold tokens that are pooled with the native token for bnb, which is the worst case

		// liquidity tokens
		let x in 3_u32..100;

		init!();
		let caller: T::AccountId = whitelisted_caller();

		let mint_amount: BalanceOf<T> = 1000000000000000000.to_balance::<T>();
		let pool_creation_amount: BalanceOf<T> = 10000000000000000.to_balance::<T>();
		let trade_amount: BalanceOf<T> = 1000000000000.to_balance::<T>();

		let mut initial_asset_id = <T as Config>::Currency::get_next_currency_id();
		let native_asset_id = <T as Config>::NativeCurrencyId::get();

		if initial_asset_id == native_asset_id {
			assert_eq!(<T as Config>::Currency::create(&caller, mint_amount * x.into()).unwrap(), native_asset_id);
			initial_asset_id = initial_asset_id + 1_u32.into();
		} else {
			assert_ok!(<T as Config>::Currency::mint(native_asset_id, &caller, mint_amount * x.into()));
		}

		// Create all the non-native tokens we will need
		for i in 0_u32..x{
			assert_eq!(<T as Config>::Currency::create(&caller, mint_amount).unwrap(), initial_asset_id + i.into());
		}

		// Create all pool with the subsequent non-native tokens
		for i in 0_u32.. (x - 1){
			assert_ok!(Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), initial_asset_id + i.into(), pool_creation_amount, initial_asset_id + (i+1).into(), pool_creation_amount));
		}

		// Create all pool with the subsequent non-native tokens
		for i in 0_u32..x{
			assert_ok!(Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id, pool_creation_amount, initial_asset_id + i.into(), pool_creation_amount));
		}

		let mut swap_token_list: Vec<CurrencyIdOf<T>> = vec![];

		for i in 0_u32..x{
			swap_token_list.push(initial_asset_id + i.into());
		}

		let expected: BalanceOf<T> = mint_amount - pool_creation_amount - pool_creation_amount;
		assert_eq!(<T as Config>::Currency::free_balance(initial_asset_id + (x - 1).into(), &caller), expected);

	}: multiswap_sell_asset(RawOrigin::Signed(caller.clone().into()), swap_token_list.into(), trade_amount, 0_u32.into())
	verify {
		// verify only trading result as rest of the assertion is in unit test
		assert!(<T as Config>::Currency::free_balance(initial_asset_id + (x - 1).into(), &caller) > expected);

	}

   buy_asset {
	   // NOTE: duplicates test case XYK::buy_and_burn_buy_where_sold_has_mangata_pair

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount: BalanceOf<T> = 1000000000000000.to_balance::<T>();
	   let expected_amount = BalanceOf::<T>::zero();
	   let expected_native_asset_id  = <T as Config>::NativeCurrencyId::get();
	   let native_asset_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let non_native_asset_id1 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let non_native_asset_id2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();

	   let amount = 100000000000000.to_balance::<T>();
	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id, amount, non_native_asset_id1, amount).unwrap();
	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1, amount, non_native_asset_id2, amount).unwrap();

   }: buy_asset(RawOrigin::Signed(caller.clone().into()), non_native_asset_id2.into(), non_native_asset_id1.into(), 33266599933266.to_balance::<T>(), 50000000000001.to_balance::<T>())
   verify {
	   // verify only trading result as rest of the assertion is in unit test
	   assert_eq!(<T as Config>::Currency::free_balance(non_native_asset_id1, &caller).into(), 833266599933266);
	   assert_eq!(<T as Config>::Currency::free_balance(non_native_asset_id2, &caller).into(), 850000000000001);
   }

   multiswap_buy_asset {

		// NOTE: All atomic swaps in the chain involve sold tokens that are pooled with the native token for bnb, which is the worst case

		// liquidity tokens
		let x in 3_u32..100;

		init!();
		let caller: T::AccountId = whitelisted_caller();

		let mint_amount: BalanceOf<T> = 1000000000000000000.to_balance::<T>();
		let pool_creation_amount: BalanceOf<T> = 10000000000000000.to_balance::<T>();
		let trade_amount: BalanceOf<T> = 1000000000000.to_balance::<T>();

		let mut initial_asset_id = <T as Config>::Currency::get_next_currency_id();
		let native_asset_id = <T as Config>::NativeCurrencyId::get();

		if initial_asset_id == native_asset_id {
			assert_eq!(<T as Config>::Currency::create(&caller, mint_amount * x.into()).unwrap(), native_asset_id);
			initial_asset_id = initial_asset_id + 1_u32.into();
		} else {
			assert_ok!(<T as Config>::Currency::mint(native_asset_id, &caller, mint_amount * x.into()));
		}

		// Create all the non-native tokens we will need
		for i in 0..x{
			assert_eq!(<T as Config>::Currency::create(&caller, mint_amount).unwrap(), initial_asset_id + i.into());
		}

		// Create all pool with the subsequent non-native tokens
		for i in 0.. (x - 1){
			assert_ok!(Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), initial_asset_id + i.into(), pool_creation_amount, initial_asset_id + (i+1).into(), pool_creation_amount));
		}

		// Create all pool with the subsequent non-native tokens
		for i in 0..x{
			assert_ok!(Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id, pool_creation_amount, initial_asset_id + i.into(), pool_creation_amount));
		}

		let mut swap_token_list: Vec<CurrencyIdOf<T>> = vec![];

		for i in 0..x{
			swap_token_list.push(initial_asset_id + i.into());
		}

		let expected: BalanceOf<T> = mint_amount - pool_creation_amount - pool_creation_amount;
		assert_eq!(<T as Config>::Currency::free_balance(initial_asset_id + (x - 1).into(), &caller), expected);

	}: multiswap_buy_asset(RawOrigin::Signed(caller.clone().into()), swap_token_list, trade_amount, trade_amount*10_u32.into())
	verify {
		// verify only trading result as rest of the assertion is in unit test
		assert!(<T as Config>::Currency::free_balance(initial_asset_id + (x - 1).into(), &caller) > expected);
	}

   mint_liquidity {
	   // 1. create,
	   // 2. promote,
	   // 3. mint/activate,
	   // 4. wait some,
	   // 5. mint – second mint is prob harder then 1st, as there are some data in

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount:BalanceOf<T> = 1000000000000000000000.to_balance::<T>();
	   let expected_native_asset_id  = <T as Config>::NativeCurrencyId::get();
	   let native_asset_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let non_native_asset_id1 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let non_native_asset_id2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let liquidity_asset_id = non_native_asset_id2 + 1_u32.into();
	   let pool_create_first_token_amount = 40000000000000000000_u128.to_balance::<T>();
	   let pool_create_second_token_amount = 60000000000000000000_u128.to_balance::<T>();
	   let pool_mint_first_token_amount = 20000000000000000000_u128.to_balance::<T>();
	   let pool_mint_second_token_amount = 30000000000000000001_u128.to_balance::<T>();


	   Xyk::<T>::create_pool(
		   RawOrigin::Signed(caller.clone().into()).into(),
		   non_native_asset_id1,
		   pool_create_first_token_amount,
		   non_native_asset_id2,
		   pool_create_second_token_amount
	   ).unwrap();
	   let initial_liquidity_amount = <T as Config>::Currency::total_issuance(liquidity_asset_id);


	   // <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::update_pool_promotion( liquidity_asset_id, Some(1u8)).unwrap();
		T::LiquidityMiningRewards::enable(liquidity_asset_id, 1u8);

	   Xyk::<T>::mint_liquidity(
		   RawOrigin::Signed(caller.clone().into()).into(),
		   non_native_asset_id1,
		   non_native_asset_id2,
		   pool_mint_first_token_amount,
		   pool_mint_second_token_amount,
	   ).unwrap();

	   let liquidity_amount_after_first_mint = <T as Config>::Currency::total_issuance(liquidity_asset_id);

	   assert!( liquidity_amount_after_first_mint > initial_liquidity_amount);

	   forward_to_next_session!();

   }: mint_liquidity(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1, non_native_asset_id2, 20000000000000000000.to_balance::<T>(), 30000000000000000001.to_balance::<T>())
   verify {
	   let liquidity_amount_after_second_mint = <T as Config>::Currency::total_issuance(liquidity_asset_id);

	   assert!(
		   liquidity_amount_after_second_mint > liquidity_amount_after_first_mint
	   )
   }

   mint_liquidity_using_vesting_native_tokens {
	   // NOTE: duplicates test case XYK::mint_W

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount:BalanceOf<T> = 1000000000000000000000.to_balance::<T>();
	   let expected_native_asset_id  = <T as Config>::NativeCurrencyId::get();
	   let native_asset_id  = <T as Config>::NativeCurrencyId::get();
	   while <T as Config>::Currency::create(&caller, initial_amount).unwrap() < native_asset_id {
	   }

	   <T as Config>::Currency::mint(native_asset_id, &caller, MILION.to_balance::<T>()).expect("Token creation failed");
	   let non_native_asset_id2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let liquidity_asset_id = non_native_asset_id2 + 1_u32.into();
	   let pool_creation_asset_1_amount = 40000000000000000000_u128.to_balance::<T>();
	   let pool_creation_asset_2_amount = 60000000000000000000_u128.to_balance::<T>();
	   let initial_liquidity_amount = pool_creation_asset_1_amount / 2_u32.into() + pool_creation_asset_2_amount / 2_u32.into();
	   let lock = 1_000_000_u32;

	   <T as Config>::Currency::mint(
		   <T as Config>::NativeCurrencyId::get(),
		   &caller,
		   initial_amount
	   ).expect("Token creation failed");

	   Xyk::<T>::create_pool(
		   RawOrigin::Signed(caller.clone().into()).into(),
		   native_asset_id,
		   pool_creation_asset_1_amount,
		   non_native_asset_id2,
		   pool_creation_asset_2_amount
	   ).unwrap();
	   // <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::update_pool_promotion( liquidity_asset_id, Some(1u8)).unwrap();
		T::LiquidityMiningRewards::enable(liquidity_asset_id, 1u8);


	   assert_eq!(
		   <T as Config>::Currency::total_issuance(liquidity_asset_id),
		   initial_liquidity_amount
	   );

	   forward_to_next_session!();

	   <T as Config>::VestingProvider::lock_tokens(&caller, native_asset_id, initial_amount - pool_creation_asset_1_amount, None, lock.into()).unwrap();

	   forward_to_next_session!();

	   Xyk::<T>::mint_liquidity_using_vesting_native_tokens(
		  RawOrigin::Signed(caller.clone().into()).into(), 10000000000000000000.to_balance::<T>(), non_native_asset_id2, 20000000000000000000.to_balance::<T>()
	   ).unwrap();

	   forward_to_next_session!();

	   let pre_minting_liq_token_amount = <T as Config>::Currency::total_issuance(liquidity_asset_id);

   }: mint_liquidity_using_vesting_native_tokens(RawOrigin::Signed(caller.clone().into()), 10000000000000000000.to_balance::<T>(), non_native_asset_id2, 20000000000000000000.to_balance::<T>())
   verify {
	   assert!(
		   <T as Config>::Currency::total_issuance(liquidity_asset_id) > pre_minting_liq_token_amount
	   );
   }

   burn_liquidity {
	   // 1. create,
	   // 2. promote,
	   // 3. mint( activates tokens automatically)
	   // 4. wait some,
	   // 5. burn all ( automatically unreserves )

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount:BalanceOf<T> = 1000000000000000000000.to_balance::<T>();
	   let expected_native_asset_id  = <T as Config>::NativeCurrencyId::get();
	   let native_asset_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let non_native_asset_id1 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let non_native_asset_id2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
	   let liquidity_asset_id = non_native_asset_id2 + 1_u32.into();
	   let pool_create_first_token_amount = 40000000000000000000_u128.to_balance::<T>();
	   let pool_create_second_token_amount = 60000000000000000000_u128.to_balance::<T>();
	   let pool_mint_first_token_amount = 20000000000000000000_u128.to_balance::<T>();
	   let pool_mint_second_token_amount = 30000000000000000001_u128.to_balance::<T>();

	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1, pool_create_first_token_amount, non_native_asset_id2, pool_create_second_token_amount).unwrap();
	   // <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::update_pool_promotion( liquidity_asset_id, Some(1u8)).unwrap();
		T::LiquidityMiningRewards::enable(liquidity_asset_id, 1u8);


	   assert!(Xyk::<T>::liquidity_pool(liquidity_asset_id).is_some());

	   Xyk::<T>::mint_liquidity(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1, non_native_asset_id2, pool_mint_first_token_amount, pool_mint_second_token_amount).unwrap();

	   forward_to_next_session!();
	   let total_liquidity_after_minting = <T as Config>::Currency::total_issuance(liquidity_asset_id);


   }: burn_liquidity(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1, non_native_asset_id2, total_liquidity_after_minting)
   verify {
		assert_eq!(Xyk::<T>::liquidity_pool(liquidity_asset_id), Some((non_native_asset_id1, non_native_asset_id2)));
		assert_eq!(<T as Config>::Currency::total_issuance(liquidity_asset_id), BalanceOf::<T>::zero());
   }

	provide_liquidity_with_conversion {
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:BalanceOf<T> = 1_000_000_000.to_balance::<T>();
		let asset_id_1 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let liquidity_asset_id = asset_id_2 + 1_u32.into();

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1, 500_000_000.to_balance::<T>(), asset_id_2, 500_000_000.to_balance::<T>()).unwrap();

	}: provide_liquidity_with_conversion(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id, asset_id_1, 100_000_u128.to_balance::<T>())
	verify {

		let post_asset_amount_1 = <T as Config>::Currency::free_balance(asset_id_1, &caller);
		let post_asset_amount_2 = <T as Config>::Currency::free_balance(asset_id_2, &caller);
		assert_eq!(post_asset_amount_1, 499_900_002.to_balance::<T>());
		assert_eq!(post_asset_amount_2, 500_000_000.to_balance::<T>());

		let post_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		assert_eq!(post_pool_balance.0, 500_099_946.to_balance::<T>());
		assert_eq!(post_pool_balance.1, 500_000_000.to_balance::<T>());
	}

	compound_rewards {
		let other: T::AccountId = account("caller1", 0, 0);
		let caller: T::AccountId = whitelisted_caller();
		let reward_ratio = 1_000_000.to_balance::<T>();
		let initial_amount:BalanceOf<T> = 1_000_000_000.to_balance::<T>();
		let pool_amount:BalanceOf<T> = initial_amount / 2_u32.into();

		let next_asset_id = <T as Config>::Currency::get_next_currency_id();
		let asset_id_1;
		let asset_id_2;
		if next_asset_id == 0_u32.into() {
			// in test there is no other currencies created
			asset_id_1 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
			<T as Config>::Currency::mint(asset_id_1, &other, initial_amount * reward_ratio).unwrap();
			asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
			<T as Config>::Currency::mint(asset_id_2, &other, initial_amount * reward_ratio).unwrap();
		} else {
			// in bench the genesis sets up the assets
			asset_id_1 = <T as Config>::NativeCurrencyId::get();
			<T as Config>::Currency::mint(asset_id_1, &caller, initial_amount).unwrap();
			<T as Config>::Currency::mint(asset_id_1, &other, initial_amount * reward_ratio).unwrap();
			asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
			<T as Config>::Currency::mint(asset_id_2, &other, initial_amount * reward_ratio).unwrap();
		}

		let liquidity_asset_id = asset_id_2 + 1_u32.into();
		<pallet_issuance::Pallet<T> as ComputeIssuance>::initialize();

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1, pool_amount, asset_id_2, pool_amount).unwrap();
		T::LiquidityMiningRewards::enable(liquidity_asset_id, 1u8);
		T::LiquidityMiningRewards::activate_liquidity(caller.clone(), liquidity_asset_id, pool_amount, None).unwrap();

		// mint for other to split the rewards rewards_ratio:1
		Xyk::<T>::mint_liquidity(
			RawOrigin::Signed(other.clone().into()).into(),
			asset_id_1,
			asset_id_2,
			pool_amount * reward_ratio,
			pool_amount * reward_ratio + 1_u32.into(),
		).unwrap();

		frame_system::Pallet::<T>::set_block_number(50_000u32.into());
		<pallet_issuance::Pallet<T> as ComputeIssuance>::compute_issuance(1);

		let mut pre_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		let rewards_to_claim = T::LiquidityMiningRewards::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap();
		let swap_amount = Xyk::<T>::calculate_balanced_sell_amount(rewards_to_claim, pre_pool_balance.0).unwrap();
		let balance_native_before = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get(), &caller);
		let balance_asset_before = <T as Config>::Currency::free_balance(liquidity_asset_id, &caller);
		pre_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));

	}: compound_rewards(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id, Permill::one())
	verify {

		assert_eq!(
			T::LiquidityMiningRewards::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap(),
			0_u32.into(),
		);

		let balance_native_after = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get(), &caller);
		let balance_asset_after = <T as Config>::Currency::free_balance(liquidity_asset_id, &caller);
		// surplus asset amount
		assert!(balance_native_before < balance_native_after);
		assert_eq!(balance_asset_before, balance_asset_after);

		let post_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		assert!( pre_pool_balance.0 < post_pool_balance.0);
		assert!( pre_pool_balance.1 >= post_pool_balance.1);
	}


	impl_benchmark_test_suite!(Xyk, crate::mock::new_test_ext(), crate::mock::Test)
}
