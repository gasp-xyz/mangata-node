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
use frame_support::assert_err;
use frame_system::RawOrigin;
use orml_tokens::MultiTokenCurrencyExtended;
use pallet_issuance::ComputeIssuance;

use crate::Pallet as Xyk;

const MaxNumberOfPromotedPools: u32 = 100;

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

		let blocks_per_session: u32 = T::RewardsDistributionPeriod::get().into();
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


	   Xyk::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, Some(1u8)).unwrap();

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
	   Xyk::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, Some(1u8)).unwrap();


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
	   Xyk::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, Some(1u8)).unwrap();


	   assert!(Xyk::<T>::liquidity_pool(liquidity_asset_id).is_some());

	   Xyk::<T>::mint_liquidity(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), non_native_asset_id2.into(), pool_mint_first_token_amount, pool_mint_second_token_amount).unwrap();

	   forward_to_next_session!();
	   let total_liquidity_after_minting = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();


   }: burn_liquidity(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1.into(), non_native_asset_id2.into(), total_liquidity_after_minting)
   verify {
	   assert!(Xyk::<T>::liquidity_pool(liquidity_asset_id).is_none());
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
	   Xyk::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, Some(1u8)).unwrap();

	   assert_eq!(			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
		   <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
	   );

	   let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());

	   let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

	   forward_to_next_session!();

	   Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

	   forward_to_next_session!();
	   forward_to_next_session!();

	   let available_rewards = Xyk::<T>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap();
	   //println!("{}",available_rewards);
	   assert!(available_rewards > 0);

   }: claim_rewards_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id, 1)

   verify {

	   assert_eq!(
		   available_rewards - 1,
		   Xyk::<T>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap()
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
	   Xyk::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, Some(1u8)).unwrap();

	   assert_eq!(
		   <T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
		   <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
	   );

	   let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
	   let half_of_minted_liquidity = total_minted_liquidity.into() / 2_u128;
	   let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

	   forward_to_next_session!();

	   Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

	   forward_to_next_session!();
	   forward_to_next_session!();

	   assert!(Xyk::<T>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap() > 0);

   }: claim_rewards_all_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id)


   verify {

	   assert_eq!(
		   0,
		   Xyk::<T>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap()
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


   }: update_pool_promotion(RawOrigin::Root, liquidity_asset_id, Some(1u8))

   verify {
	   assert!(
			<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id).is_some()
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
	   Xyk::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, Some(1u8)).unwrap();

	   assert_eq!(
		   <T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
		   <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
	   );

	   let total_minted_liquidity: u128 = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();
	   let half_of_minted_liquidity = total_minted_liquidity / 2_u128;
	   let quater_of_minted_liquidity = total_minted_liquidity / 4_u128;

	   Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

	   assert_eq!(
		   Xyk::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
		   quater_of_minted_liquidity
	   );

	   forward_to_next_session!();

   }: activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity, None)
   verify {

	   assert_eq!(
		   Xyk::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
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
	   Xyk::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, Some(1u8)).unwrap();

	   assert_eq!(
		   <T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
		   <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
	   );

	   let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
	   let half_of_minted_liquidity = total_minted_liquidity.into() / 2_u128;
	   let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

	   Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), half_of_minted_liquidity, None).unwrap();
	   assert_eq!(
		   Xyk::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
		   half_of_minted_liquidity
	   );

	   forward_to_next_session!();

   }: deactivate_liquidity_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity.into())
   verify {
	   assert_eq!(
		   Xyk::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
		   quater_of_minted_liquidity
	   );
   }

   rewards_migrate_v1_to_v2 {
	   // 0 fill storage with rew1 values
	   // 1 crate pool
	   // 2 promote pool v2
	   // 3 migrate (last migration to be checked)

	//    XykStorage::transfer(
	// 	0,
	// 	2,
	// 	<Test as Config>::LiquidityMiningIssuanceVault::get(),
	// 	10000000000000,
	// )
	//.unwrap();

	   init!();
	   let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
	   let user1: T::AccountId = account("user1", 0, 0);
	   let user2: T::AccountId = account("user2", 1, 0);

	   pallet_issuance::PromotedPoolsRewards::<T>::insert(4, 200000);
	   Xyk::<T>::update_pool_promotion(RawOrigin::Root.into(), 4, Some(1u8));

	   LiquidityMiningActiveUser::<T>::insert((user1.clone(), 4), 10000);
	   LiquidityMiningActiveUser::<T>::insert((user2.clone(), 4), 10000);
	   LiquidityMiningActivePool::<T>::insert(4, 20000);
	   LiquidityMiningUser::<T>::insert((user1.clone(), 4), (0, U256::from(0_u64), U256::from(10000_u64)));
	   LiquidityMiningUser::<T>::insert((user2.clone(), 4), (10, U256::from(0_u64), U256::from(10000_u64)));
	   LiquidityMiningPool::<T>::insert(4, (10, U256::from(21985_u64), U256::from(15584_u64)));
	   LiquidityMiningUserToBeClaimed::<T>::insert((user1.clone(), 4), 0);
	   LiquidityMiningUserClaimed::<T>::insert((user1.clone(), 4), 0);
	   LiquidityMiningUserToBeClaimed::<T>::insert((user2.clone(), 4), 0);
	   LiquidityMiningUserClaimed::<T>::insert((user2.clone(), 4), 0);

	   frame_system::Pallet::<T>::set_block_number(24000_u32.into());
	   T::PoolPromoteApi::compute_issuance(24000_u32.into());

		let account32: sp_runtime::AccountId32 =
			hex_literal::hex!["0e33df23356eb2e9e3baf0e8a5faae15bc70a6a5cce88f651a9faf6e8e937324"]
				.into();
		let mut init_account32 = sp_runtime::AccountId32::as_ref(&account32);
		let acc = T::AccountId::decode(&mut init_account32).unwrap();

		Xyk::<T>::rewards_migrate_v1_to_v2(RawOrigin::Signed(acc.clone()).into(),user1.clone(), 4);


   }: rewards_migrate_v1_to_v2(RawOrigin::Signed(acc.clone()),user2.clone(), 4)
   verify {

	   assert_eq!(Xyk::<T>::get_rewards_info(user1.clone(), 4).activated_amount, 10000);
	   assert_eq!(Xyk::<T>::get_rewards_info(user1.clone(), 4).rewards_not_yet_claimed, 0);
	   assert_eq!(Xyk::<T>::get_rewards_info(user1.clone(), 4).rewards_already_claimed, 0);
	   assert_eq!(Xyk::<T>::get_rewards_info(user1.clone(), 4).last_checkpoint, 20);
	   assert_eq!(Xyk::<T>::get_rewards_info(user1.clone(), 4).pool_ratio_at_last_checkpoint, U256::from(0_u64)); //these values will be from rew2, but reading pool_ratio_at_last_checkpoint works
	   assert_eq!(Xyk::<T>::get_rewards_info(user1.clone(), 4).missing_at_last_checkpoint, U256::from(3118_u64));

	   assert_eq!(Xyk::<T>::get_rewards_info(user2.clone(), 4).activated_amount, 10000);
	   assert_eq!(Xyk::<T>::get_rewards_info(user2.clone(), 4).rewards_not_yet_claimed, 0);
	   assert_eq!(Xyk::<T>::get_rewards_info(user2.clone(), 4).rewards_already_claimed, 0);
	   assert_eq!(Xyk::<T>::get_rewards_info(user2.clone(), 4).last_checkpoint, 20);
	   assert_eq!(Xyk::<T>::get_rewards_info(user2.clone(), 4).pool_ratio_at_last_checkpoint, U256::from(0_u64)); //these values will be from rew2, but reading pool_ratio_at_last_checkpoint works
	   assert_eq!(Xyk::<T>::get_rewards_info(user2.clone(), 4).missing_at_last_checkpoint, U256::from(5584_u64));

   }

	provide_liquidity_with_conversion {
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1_000_000_000;
		let asset_id_1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let asset_id_2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = asset_id_2 + 1;
		<<T as Config>::PoolPromoteApi as ComputeIssuance>::initialize();

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), 500_000_000, asset_id_2.into(), 500_000_000).unwrap();

	}: provide_liquidity_with_conversion(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), asset_id_1, 100_000_u128)
	verify {

		let post_asset_amount_1 = <T as Config>::Currency::free_balance(asset_id_1.into(), &caller).into();
		let post_asset_amount_2 = <T as Config>::Currency::free_balance(asset_id_2.into(), &caller).into();
		assert_eq!(post_asset_amount_1, 499_900_000,);
		assert_eq!(post_asset_amount_2, 500_000_001);

		let post_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		assert_eq!(post_pool_balance.0, 500_099_948);
		assert_eq!(post_pool_balance.1, 499_999_999);
	}

	compound_rewards {
		let other: T::AccountId = account("caller1", 0, 0);
		let caller: T::AccountId = whitelisted_caller();
		let reward_ratio = 1_000_000;
		let initial_amount:mangata_primitives::Balance = 100_000_000_000_000;
		let pool_amount:mangata_primitives::Balance = initial_amount / 2;

		let asset_id_1 : TokenId= <T as Config>::NativeCurrencyId::get().into();
		let asset_id_2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		<T as Config>::Currency::mint(asset_id_1.into(), &caller, initial_amount.into()).unwrap();
		<T as Config>::Currency::mint(asset_id_1.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
		<T as Config>::Currency::mint(asset_id_2.into(), &other, (initial_amount * reward_ratio).into()).unwrap();

		let liquidity_asset_id = asset_id_2 + 1;
		<<T as Config>::PoolPromoteApi as ComputeIssuance>::initialize();

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), pool_amount, asset_id_2.into(), pool_amount).unwrap();
		Xyk::<T>::promote_pool(RawOrigin::Root.into(), liquidity_asset_id).unwrap();
		Xyk::<T>::activate_liquidity(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id, pool_amount, None).unwrap();
		// mint for other to split the rewards rewards_ratio:1
		Xyk::<T>::mint_liquidity(
			RawOrigin::Signed(other.clone().into()).into(),
			asset_id_1,
			asset_id_2,
			pool_amount * reward_ratio,
			pool_amount * reward_ratio + 1,
		).unwrap();

		frame_system::Pallet::<T>::set_block_number(50_000u32.into());
		<<T as Config>::PoolPromoteApi as ComputeIssuance>::compute_issuance(1);

		let mut pre_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		let rewards_to_claim = Xyk::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap();
		let swap_amount = Xyk::<T>::calculate_balanced_sell_amount(rewards_to_claim, pre_pool_balance.0).unwrap();
		let pre_claim_native_tokens_amount = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
		pre_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));

	}: compound_rewards(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), 1000_u128)
	verify {

		assert_eq!(rewards_to_claim, 135_463_177_684_253_389);
		assert_eq!(swap_amount, 67_787_502_354_636_248);

		assert_eq!(
			Xyk::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap(),
			(0_u128)
		);

		let post_claim_native_tokens_amount = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
		assert_eq!( pre_claim_native_tokens_amount, post_claim_native_tokens_amount);

		let post_pool_balance = Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
		assert!( pre_pool_balance.0 < post_pool_balance.0);
		assert!( pre_pool_balance.1 >= post_pool_balance.1);
	}

	impl_benchmark_test_suite!(Xyk, crate::mock::new_test_ext(), crate::mock::Test)
}
