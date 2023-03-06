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
use mp_traits::ProofOfStakeRewardsApi;
use orml_tokens::MultiTokenCurrencyExtended;
use pallet_issuance::ComputeIssuance;
use sp_runtime::Permill;

use crate::Pallet as Xyk;

const MILION: u128 = 1_000__000_000__000_000;

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

		let blocks_per_session: u32 =
			<T as pallet_proof_of_stake::Config>::RewardsDistributionPeriod::get().into();
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
		T::PoolPromoteApi::compute_issuance(target_session_nr);
	};
}

benchmarks! {

   create_pool {
	   init!();
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

   }

   sell_asset {
	   // NOTE: duplicates test case XYK::buy_and_burn_sell_none_have_mangata_pair

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount:mangata_types::Balance = 1000000000000000;
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

   multiswap_sell_asset {

		// NOTE: All atomic swaps in the chain involve sold tokens that are pooled with the native token for bnb, which is the worst case

		// liquidity tokens
		let x in 3..100;

		init!();
		let caller: T::AccountId = whitelisted_caller();

		let mint_amount: mangata_types::Balance = 1000000000000000000;
		let pool_creation_amount: mangata_types::Balance = 10000000000000000;
		let trade_amount: mangata_types::Balance = 1000000000000;

		let mut initial_asset_id: mangata_types::TokenId = <T as Config>::Currency::get_next_currency_id().into();
		let native_asset_id : mangata_types::TokenId = <T as Config>::NativeCurrencyId::get().into();

		if initial_asset_id == native_asset_id {
			assert_eq!(<T as Config>::Currency::create(&caller, (mint_amount * x as u128).into()).unwrap().into(), native_asset_id);
			initial_asset_id = initial_asset_id + 1;
		} else {
			assert_ok!(<T as Config>::Currency::mint(native_asset_id.into(), &caller, (mint_amount * x as u128).into()));
		}

		// Create all the non-native tokens we will need
		for i in 0..x{
			assert_eq!(<T as Config>::Currency::create(&caller, (mint_amount).into()).unwrap().into(), initial_asset_id + i);
		}

		// Create all pool with the subsequent non-native tokens
		for i in 0.. (x - 1){
			assert_ok!(Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), {initial_asset_id + i}.into(), pool_creation_amount, {initial_asset_id + i+1}.into(), pool_creation_amount));
		}

		// Create all pool with the subsequent non-native tokens
		for i in 0..x{
			assert_ok!(Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id.into(), pool_creation_amount, {initial_asset_id + i}.into(), pool_creation_amount));
		}

		let mut swap_token_list: Vec<TokenId> = vec![];

		for i in 0..x{
			swap_token_list.push({initial_asset_id + i}.into());
		}

		assert_eq!(<T as Config>::Currency::free_balance({initial_asset_id + x - 1}.into(), &caller).into(), mint_amount - 2 * pool_creation_amount);

	}: multiswap_sell_asset(RawOrigin::Signed(caller.clone().into()), swap_token_list.into(), trade_amount, 0)
	verify {
		// verify only trading result as rest of the assertion is in unit test
		assert!(<T as Config>::Currency::free_balance({initial_asset_id + x - 1}.into(), &caller).into() > mint_amount - 2 * pool_creation_amount);

	}

   buy_asset {
	   // NOTE: duplicates test case XYK::buy_and_burn_buy_where_sold_has_mangata_pair

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount:mangata_types::Balance = 1000000000000000;
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

   multiswap_buy_asset {

		// NOTE: All atomic swaps in the chain involve sold tokens that are pooled with the native token for bnb, which is the worst case

		// liquidity tokens
		let x in 3..100;

		init!();
		let caller: T::AccountId = whitelisted_caller();

		let mint_amount: mangata_types::Balance = 1000000000000000000;
		let pool_creation_amount: mangata_types::Balance = 10000000000000000;
		let trade_amount: mangata_types::Balance = 1000000000000;

		let mut initial_asset_id: mangata_types::TokenId = <T as Config>::Currency::get_next_currency_id().into();
		let native_asset_id : mangata_types::TokenId = <T as Config>::NativeCurrencyId::get().into();

		if initial_asset_id == native_asset_id {
			assert_eq!(<T as Config>::Currency::create(&caller, (mint_amount * x as u128).into()).unwrap().into(), native_asset_id);
			initial_asset_id = initial_asset_id + 1;
		} else {
			assert_ok!(<T as Config>::Currency::mint(native_asset_id.into(), &caller, (mint_amount * x as u128).into()));
		}

		// Create all the non-native tokens we will need
		for i in 0..x{
			assert_eq!(<T as Config>::Currency::create(&caller, (mint_amount).into()).unwrap().into(), initial_asset_id + i);
		}

		// Create all pool with the subsequent non-native tokens
		for i in 0.. (x - 1){
			assert_ok!(Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), {initial_asset_id + i}.into(), pool_creation_amount, {initial_asset_id + i+1}.into(), pool_creation_amount));
		}

		// Create all pool with the subsequent non-native tokens
		for i in 0..x{
			assert_ok!(Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id.into(), pool_creation_amount, {initial_asset_id + i}.into(), pool_creation_amount));
		}

		let mut swap_token_list: Vec<TokenId> = vec![];

		for i in 0..x{
			swap_token_list.push({initial_asset_id + i}.into());
		}

		assert_eq!(<T as Config>::Currency::free_balance({initial_asset_id + x - 1}.into(), &caller).into(), mint_amount - 2 * pool_creation_amount);

	}: multiswap_buy_asset(RawOrigin::Signed(caller.clone().into()), swap_token_list.into(), trade_amount, trade_amount*10)
	verify {
		// verify only trading result as rest of the assertion is in unit test
		assert!(<T as Config>::Currency::free_balance({initial_asset_id + x - 1}.into(), &caller).into() > mint_amount - 2 * pool_creation_amount);
	}

   mint_liquidity {
	   // 1. create,
	   // 2. promote,
	   // 3. mint/activate_v2,
	   // 4. wait some,
	   // 5. mint – second mint is prob harder then 1st, as there are some data in

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount:mangata_types::Balance = 1000000000000000000000;
	   let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
	   let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	   let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	   let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	   let liquidity_asset_id = non_native_asset_id2 + 1;
	   let pool_create_first_token_amount = 40000000000000000000_u128;
	   let pool_create_second_token_amount = 60000000000000000000_u128;
	   let pool_mint_first_token_amount = 20000000000000000000_u128;
	   let pool_mint_second_token_amount = 30000000000000000001_u128;


	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(),
		   non_native_asset_id1.into(),
		   pool_create_first_token_amount,
		   non_native_asset_id2.into(),
		   pool_create_second_token_amount
	   ).unwrap();
	   let initial_liquidity_amount = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());


	   <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::update_pool_promotion( liquidity_asset_id, Some(1u8)).unwrap();

	   Xyk::<T>::mint_liquidity(
		   RawOrigin::Signed(caller.clone().into()).into(),
		   non_native_asset_id1.into(),
		   non_native_asset_id2.into(),
		   pool_mint_first_token_amount,
		   pool_mint_second_token_amount,
	   ).unwrap();

	   let liquidity_amount_after_first_mint = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());

	   assert!( liquidity_amount_after_first_mint > initial_liquidity_amount);

	   forward_to_next_session!();

   }: mint_liquidity(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1.into(), non_native_asset_id2.into(), 20000000000000000000, 30000000000000000001)
   verify {
	   let liquidity_amount_after_second_mint = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());

	   assert!(
		   liquidity_amount_after_second_mint > liquidity_amount_after_first_mint
	   )
   }

   mint_liquidity_using_vesting_native_tokens {
	   // NOTE: duplicates test case XYK::mint_W

	   init!();
	   let caller: T::AccountId = whitelisted_caller();
	   let initial_amount:mangata_types::Balance = 1000000000000000000000;
	   let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
	   let native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get();
	   while <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into() < native_asset_id {
	   }

	   <T as Config>::Currency::mint(native_asset_id.into(), &caller, MILION.into()).expect("Token creation failed");
	   let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	   let liquidity_asset_id = non_native_asset_id2 + 1;
	   let pool_creation_asset_1_amount = 40000000000000000000_u128;
	   let pool_creation_asset_2_amount = 60000000000000000000_u128;
	   let initial_liquidity_amount = pool_creation_asset_1_amount / 2_u128 + pool_creation_asset_2_amount / 2_u128;
	   let lock = 1_000_000_u128;

	   <T as Config>::Currency::mint(
		   <T as Config>::NativeCurrencyId::get().into(),
		   &caller,
		   initial_amount.into()
	   ).expect("Token creation failed");

	   Xyk::<T>::create_pool(
		   RawOrigin::Signed(caller.clone().into()).into(),
		   native_asset_id.into(),
		   pool_creation_asset_1_amount,
		   non_native_asset_id2.into(),
		   pool_creation_asset_2_amount
	   ).unwrap();
	   <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::update_pool_promotion( liquidity_asset_id, Some(1u8)).unwrap();


	   assert_eq!(
		   <T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
		   initial_liquidity_amount.into()
	   );

	   forward_to_next_session!();

	   <T as Config>::VestingProvider::lock_tokens(&caller, native_asset_id.into(), (initial_amount - pool_creation_asset_1_amount).into(), None, lock.into()).unwrap();

	   forward_to_next_session!();

	   Xyk::<T>::mint_liquidity_using_vesting_native_tokens(RawOrigin::Signed(caller.clone().into()).into(), 10000000000000000000, non_native_asset_id2.into(), 20000000000000000000).unwrap();

	   forward_to_next_session!();

	   let pre_minting_liq_token_amount = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());

   }: mint_liquidity_using_vesting_native_tokens(RawOrigin::Signed(caller.clone().into()), 10000000000000000000, non_native_asset_id2.into(), 20000000000000000000)
   verify {
	   assert!(
		   <T as Config>::Currency::total_issuance(liquidity_asset_id.into()) > pre_minting_liq_token_amount
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
	   let initial_amount:mangata_types::Balance = 1000000000000000000000;
	   let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
	   let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	   let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	   let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	   let liquidity_asset_id = non_native_asset_id2 + 1;
	   let pool_create_first_token_amount = 40000000000000000000_u128;
	   let pool_create_second_token_amount = 60000000000000000000_u128;
	   let pool_mint_first_token_amount = 20000000000000000000_u128;
	   let pool_mint_second_token_amount = 30000000000000000001_u128;

	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), pool_create_first_token_amount, non_native_asset_id2.into(), pool_create_second_token_amount).unwrap();
	   <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::update_pool_promotion( liquidity_asset_id, Some(1u8)).unwrap();


	   assert!(Xyk::<T>::liquidity_pool(liquidity_asset_id).is_some());

	   Xyk::<T>::mint_liquidity(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), non_native_asset_id2.into(), pool_mint_first_token_amount, pool_mint_second_token_amount).unwrap();

	   forward_to_next_session!();
	   let total_liquidity_after_minting = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();


   }: burn_liquidity(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1.into(), non_native_asset_id2.into(), total_liquidity_after_minting)
   verify {
	   assert!(Xyk::<T>::liquidity_pool(liquidity_asset_id).is_none());
   }

	provide_liquidity_with_conversion {
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1_000_000_000;
		let asset_id_1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let asset_id_2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = asset_id_2 + 1;

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), 500_000_000, asset_id_2.into(), 500_000_000).unwrap();

	}: provide_liquidity_with_conversion(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), asset_id_1, 100_000_u128)
	verify {

		let post_asset_amount_1 = <T as Config>::Currency::free_balance(asset_id_1.into(), &caller).into();
		let post_asset_amount_2 = <T as Config>::Currency::free_balance(asset_id_2.into(), &caller).into();
		assert_eq!(post_asset_amount_1, 499_900_002,);
		assert_eq!(post_asset_amount_2, 500_000_000);

		let post_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		assert_eq!(post_pool_balance.0, 500_099_946);
		assert_eq!(post_pool_balance.1, 500_000_000);
	}

	compound_rewards {
		let other: T::AccountId = account("caller1", 0, 0);
		let caller: T::AccountId = whitelisted_caller();
		let reward_ratio = 1_000_000;
		let initial_amount:mangata_types::Balance = 1_000_000_000;
		let pool_amount:mangata_types::Balance = initial_amount / 2;

		let next_asset_id: TokenId = <T as Config>::Currency::get_next_currency_id().into();
		let asset_id_1: TokenId;
		let asset_id_2: TokenId;
		if next_asset_id == 0 {
			// in test there is no other currencies created
			asset_id_1 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
			<T as Config>::Currency::mint(asset_id_1.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
			asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
			<T as Config>::Currency::mint(asset_id_2.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
		} else {
			// in bench the genesis sets up the assets
			asset_id_1 = <T as Config>::NativeCurrencyId::get().into();
			<T as Config>::Currency::mint(asset_id_1.into(), &caller, initial_amount.into()).unwrap();
			<T as Config>::Currency::mint(asset_id_1.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
			asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
			<T as Config>::Currency::mint(asset_id_2.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
		}

		let liquidity_asset_id = asset_id_2 + 1;
		<pallet_issuance::Pallet<T> as ComputeIssuance>::initialize();

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), pool_amount, asset_id_2.into(), pool_amount).unwrap();
		<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::update_pool_promotion( liquidity_asset_id, Some(1u8)).unwrap();
		<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::activate_liquidity_v2(caller.clone().into(), liquidity_asset_id, pool_amount, None).unwrap();
		// mint for other to split the rewards rewards_ratio:1
		Xyk::<T>::mint_liquidity(
			RawOrigin::Signed(other.clone().into()).into(),
			asset_id_1,
			asset_id_2,
			pool_amount * reward_ratio,
			pool_amount * reward_ratio + 1,
		).unwrap();

		frame_system::Pallet::<T>::set_block_number(50_000u32.into());
		<pallet_issuance::Pallet<T> as ComputeIssuance>::compute_issuance(1);

		let mut pre_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		let rewards_to_claim = <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap();
		let swap_amount = Xyk::<T>::calculate_balanced_sell_amount(rewards_to_claim, pre_pool_balance.0).unwrap();
		let balance_native_before = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
		let balance_asset_before = <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller).into();
		pre_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));

	}: compound_rewards(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), Permill::one())
	verify {

		assert_eq!(
			<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap(),
			(0_u128)
		);

		let balance_native_after = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
		let balance_asset_after = <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller).into();
		// surplus asset amount
		assert!(balance_native_before < balance_native_after);
		assert_eq!(balance_asset_before, balance_asset_after);

		let post_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		assert!( pre_pool_balance.0 < post_pool_balance.0);
		assert!( pre_pool_balance.1 >= post_pool_balance.1);
	}

	claim_rewards_v2 {
		// 1. create
		// 2. promote
		// 3. activate
		// 4. wait some
		// 5. claim some

		init!();
		// NOTE: need to use actual issuance pallet and call its hooks properly
		// NOTE: that duplicates test XYK::liquidity_rewards_claim_W
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000000000000000;
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;

	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
	   <pallet_proof_of_stake::Pallet<T>>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());

		let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

		forward_to_next_session!();

		<pallet_proof_of_stake::Pallet<T>>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		forward_to_next_session!();
		forward_to_next_session!();

		let available_rewards =<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap();
		//println!("{}",available_rewards);
		assert!(available_rewards > 0);

	}: {<pallet_proof_of_stake::Pallet<T>>::claim_rewards_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id, 1)}

	verify {

		assert_eq!(
			available_rewards - 1,
			<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap()
		);

	}

	claim_rewards_all_v2 {
		// 1. create
		// 2. promote
		// 3. mint
		// 4. wait some
		// 5. claim all

		init!();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;

	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
	   <pallet_proof_of_stake::Pallet<T>>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
		let half_of_minted_liquidity = total_minted_liquidity.into() / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

		forward_to_next_session!();

		<pallet_proof_of_stake::Pallet<T>>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		forward_to_next_session!();
		forward_to_next_session!();

		assert!(<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap() > 0);

	}: {<pallet_proof_of_stake::Pallet<T>>::claim_rewards_all_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id)}


	verify {

		assert_eq!(
			0,
			<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap()
		);

	}


	update_pool_promotion {
		// NOTE: that duplicates test XYK::liquidity_rewards_claim_W
		//
		init!();
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000;

		let asset_id_1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let asset_id_2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = asset_id_2 + 1;

	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), 5000, asset_id_2.into(), 5000).unwrap();


	}: {<pallet_proof_of_stake::Pallet<T>>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8)}

	verify {
		assert!(
			<T::XykRewards as CumulativeWorkRewardsApi>::get_pool_rewards_v2(liquidity_asset_id).is_some()
		 );
	}

	activate_liquidity_v2 {
		// activate :
		// 1 crate pool
		// 2 promote pool
		// 3 activate some
		// 4 wait some time
		// 5 mint some

		init!();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;

	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
	   <pallet_proof_of_stake::Pallet<T>>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity: u128 = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();
		let half_of_minted_liquidity = total_minted_liquidity / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity / 4_u128;

		<pallet_proof_of_stake::Pallet<T>>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		assert_eq!(
		 <pallet_proof_of_stake::Pallet<T>>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			quater_of_minted_liquidity
		);

		forward_to_next_session!();

	}: {<pallet_proof_of_stake::Pallet<T>>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None)}
	verify {

		assert_eq!(
		 <pallet_proof_of_stake::Pallet<T>>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			half_of_minted_liquidity
		)
	}

	deactivate_liquidity_v2 {
		// deactivate
		// 1 crate pool
		// 2 promote pool
		// 3 mint some tokens
		// deactivate some tokens (all or some - to be checked)

		init!();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;

	   Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
	   <pallet_proof_of_stake::Pallet<T>>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
		let half_of_minted_liquidity = total_minted_liquidity.into() / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

		<pallet_proof_of_stake::Pallet<T>>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), half_of_minted_liquidity, None).unwrap();
		assert_eq!(
		 <pallet_proof_of_stake::Pallet<T>>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			half_of_minted_liquidity
		);

		forward_to_next_session!();

	}: {<pallet_proof_of_stake::Pallet<T>>::deactivate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity.into())}
	verify {
		assert_eq!(
		 <pallet_proof_of_stake::Pallet<T>>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			quater_of_minted_liquidity
		);
	}

	proof_of_stake_compound_rewards {
		 let other: T::AccountId = account("caller1", 0, 0);
		 let caller: T::AccountId = whitelisted_caller();
		 let reward_ratio = 1_000_000;
		 let initial_amount:mangata_types::Balance = 1_000_000_000;
		 let pool_amount:mangata_types::Balance = initial_amount / 2;

		 let next_asset_id: TokenId = <T as Config>::Currency::get_next_currency_id().into();
		 let asset_id_1: TokenId;
		 let asset_id_2: TokenId;
		 if next_asset_id == 0 {
			 // in test there is no other currencies created
			 asset_id_1 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
			 <T as Config>::Currency::mint(asset_id_1.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
			 asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
			 <T as Config>::Currency::mint(asset_id_2.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
		 } else {
			 // in bench the genesis sets up the assets
			 asset_id_1 = <T as Config>::NativeCurrencyId::get().into();
			 <T as Config>::Currency::mint(asset_id_1.into(), &caller, initial_amount.into()).unwrap();
			 <T as Config>::Currency::mint(asset_id_1.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
			 asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
			 <T as Config>::Currency::mint(asset_id_2.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
		 }

		 let liquidity_asset_id = asset_id_2 + 1;
		 <pallet_issuance::Pallet<T> as ComputeIssuance>::initialize();

		 Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), pool_amount, asset_id_2.into(), pool_amount).unwrap();
		 <pallet_proof_of_stake::Pallet<T>>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();
		 <pallet_proof_of_stake::Pallet<T>>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id, pool_amount, None).unwrap();
		 // mint for other to split the rewards rewards_ratio:1
		 Xyk::<T>::mint_liquidity(
			 RawOrigin::Signed(other.clone().into()).into(),
			 asset_id_1,
			 asset_id_2,
			 pool_amount * reward_ratio,
			 pool_amount * reward_ratio + 1,
		 ).unwrap();

		 frame_system::Pallet::<T>::set_block_number(50_000u32.into());
		 <pallet_issuance::Pallet<T> as ComputeIssuance>::compute_issuance(1);

		 let mut pre_pool_balance =Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		 let rewards_to_claim =<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap();
		 let swap_amount =Xyk::<T>::calculate_balanced_sell_amount(rewards_to_claim, pre_pool_balance.0).unwrap();
		 let balance_native_before = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
		 let balance_asset_before = <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller).into();
		 pre_pool_balance =Xyk::<T>::asset_pool((asset_id_1, asset_id_2));

	 }: {<pallet_proof_of_stake::Pallet<T>>::compound_rewards(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), Permill::one())}
	 verify {

		 assert_eq!(
			 <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap(),
			 (0_u128)
		 );

		 let balance_native_after = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
		 let balance_asset_after = <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller).into();
		 // surplus asset amount
		 assert!(balance_native_before < balance_native_after);
		 assert_eq!(balance_asset_before, balance_asset_after);

		 let post_pool_balance =Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		 assert!( pre_pool_balance.0 < post_pool_balance.0);
		 assert!( pre_pool_balance.1 >= post_pool_balance.1);
	 }

	impl_benchmark_test_suite!(Xyk, crate::mock::new_test_ext(), crate::mock::Test)
}
