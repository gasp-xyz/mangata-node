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
use pallet_issuance::ComputeIssuance;

use crate::Pallet as Xyk;

const MILION: u128 = 1_000__000_000__000_000;

benchmarks! {

	create_pool {
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
		// 1. create,
		// 2. promote,
		// 3. mint/activate_v2,
		// 4. wait some,
		// 5. mint – second mint is prob harder then 1st, as there are some data in

		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;
		let initial_liquidity_amount = 40000000000000000000_u128 / 2_u128 + 60000000000000000000_u128 / 2_u128;

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
		Xyk::<T>::promote_pool(RawOrigin::Root.into(), liquidity_asset_id).unwrap();


		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			initial_liquidity_amount.into()
		);

		Xyk::<T>::mint_liquidity(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), non_native_asset_id2.into(), 20000000000000000000, 30000000000000000001).unwrap();

		T::PoolPromoteApi::compute_issuance(1_u32).unwrap();
		frame_system::Pallet::<T>::set_block_number(101_000u32.into());


	}: mint_liquidity(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1.into(), non_native_asset_id2.into(), 20000000000000000000, 30000000000000000001)
	verify {
		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			100000000000000000000_u128.into()
		);
	}

	// mint_liquidity_using_vesting_native_tokens {
	// 	// NOTE: duplicates test case XYK::mint_W
	//
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	let initial_amount:mangata_primitives::Balance = 1000000000000000000000;
	// 	let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
	// 	let native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get();
	// 	while <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into() < native_asset_id {
	// 	}
	//
	// 	<T as Config>::Currency::mint(native_asset_id.into(), &caller, MILION.into()).expect("Token creation failed");
	// 	let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	// 	let liquidity_asset_id = non_native_asset_id2 + 1;
	// 	let pool_creation_asset_1_amount = 40000000000000000000_u128;
	// 	let pool_creation_asset_2_amount = 60000000000000000000_u128;
	// 	let initial_liquidity_amount = pool_creation_asset_1_amount / 2_u128 + pool_creation_asset_2_amount / 2_u128;
	// 	let lock = 1_000_000_u128;
	// 	<T as Config>::Currency::mint(<T as Config>::NativeCurrencyId::get().into(), &caller, initial_amount.into()).expect("Token creation failed");
	//
	// 	Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), native_asset_id.into(), pool_creation_asset_1_amount, non_native_asset_id2.into(), pool_creation_asset_2_amount).unwrap();
	// 	Xyk::<T>::promote_pool(RawOrigin::Root.into(), liquidity_asset_id).unwrap();
	//
	//
	// 	assert_eq!(
	// 		<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
	// 		initial_liquidity_amount.into()
	// 	);
	//
	// 	frame_system::Pallet::<T>::set_block_number(1_u32.into());
	// 	<T as Config>::VestingProvider::lock_tokens(&caller, native_asset_id.into(), (initial_amount - pool_creation_asset_1_amount).into(), lock.into()).unwrap();
	// 	frame_system::Pallet::<T>::set_block_number(2_u32.into());
	//
	// 	Xyk::<T>::mint_liquidity_using_vesting_native_tokens(RawOrigin::Signed(caller.clone().into()).into(), 10000000000000000000, non_native_asset_id2.into(), 20000000000000000000).unwrap();
	//
	// 	frame_system::Pallet::<T>::set_block_number(100_000_u32.into());
	// 	let pre_minting_liq_token_amount = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
	//
	// }: mint_liquidity_using_vesting_native_tokens(RawOrigin::Signed(caller.clone().into()), 10000000000000000000, non_native_asset_id2.into(), 20000000000000000000)
	// verify {
	// 	assert!(
	// 		<T as Config>::Currency::total_issuance(liquidity_asset_id.into()) > pre_minting_liq_token_amount
	// 	);
	// }

	burn_liquidity {
		// 1. create,
		// 2. promote,
		// 3. mint( activates tokens automatically)
		// 4. wait some,
		// 5. burn all ( automatically unreserves )

		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;
		let initial_liquidity_amount = 40000000000000000000_u128 / 2_u128 + 60000000000000000000_u128 / 2_u128;

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
		Xyk::<T>::promote_pool(RawOrigin::Root.into(), liquidity_asset_id).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			initial_liquidity_amount.into()
		);
		assert!(Xyk::<T>::liquidity_pool(liquidity_asset_id).is_some());

		let total_issuance :u128 = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();

		Xyk::<T>::mint_liquidity(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), non_native_asset_id2.into(), 20000000000000000000, 30000000000000000001).unwrap();

		let total_issuance :u128 = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();
		assert_ne!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			initial_liquidity_amount.into()
		);

		let total_liquidity_after_minting: u128 = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();

		T::PoolPromoteApi::compute_issuance(1_u32).unwrap();
		frame_system::Pallet::<T>::set_block_number(100_000u32.into());

	}: burn_liquidity(RawOrigin::Signed(caller.clone().into()), non_native_asset_id1.into(), non_native_asset_id2.into(), total_liquidity_after_minting)
	verify {
		assert!(Xyk::<T>::liquidity_pool(liquidity_asset_id).is_none());
	}

	// claim_rewards_v2 {
	// 	// 1. create
	// 	// 2. promote
	// 	// 3. mint
	// 	// 4. wait some
	// 	// 5. claim some
	//
	//
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	let initial_amount:mangata_primitives::Balance = 1000000000000000;
	//
	// 	let asset_id_1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	// 	let asset_id_2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	// 	let liquidity_asset_id = asset_id_2 + 1;
	//
	// 	Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), 5000, asset_id_2.into(), 5000).unwrap();
	// 	Xyk::<T>::promote_pool(RawOrigin::Root.into(), liquidity_asset_id).unwrap();
	//
	// 	Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id, initial_amount, Some(ActivateKind::AvailableBalance)).unwrap();
	//
	// 	<<T as Config>::PoolPromoteApi as ComputeIssuance>::compute_issuance(1);
	// 	frame_system::Pallet::<T>::set_block_number(100_000u32.into());
	//
	// 	<<T as Config>::PoolPromoteApi as ComputeIssuance>::compute_issuance(2);
	// 	frame_system::Pallet::<T>::set_block_number(100_000u32.into());
	//
	// 	// // activate liquidity by minting
	// 	// Xyk::<T>::mint_liquidity(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), asset_id_2.into(), 200000000000000, 300000000000000).unwrap();
	// 	// frame_system::Pallet::<T>::set_block_number(200_000_u32.into());
	// 	//
	// 	// Xyk::<T>::burn_liquidity(RawOrigin::Signed(caller.clone().into()).into(), asset_id_2.into(), asset_id_2.into(), 150000000000000).unwrap();
	// 	//
	// 	// frame_system::Pallet::<T>::set_block_number(300_000_u32.into());
	// 	// <<T as Config>::PoolPromoteApi as ComputeIssuance>::compute_issuance(2);
	// 	//
	// 	let rewards_to_claim = Xyk::<T>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap();
	//
	// 	assert_ne!(
	// 		rewards_to_claim,
	// 		0,
	// 	);
	//
	// 	let pre_claim_native_tokens_amount = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
	// 	//
	// 	// frame_system::Pallet::<T>::set_block_number(100_000_000u32.into());
	//
	// }: claim_rewards_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id, rewards_to_claim as u128 )
	//
	// verify {
	//
	// 	assert_eq!(
	// 		Xyk::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap(),
	// 		(0_u128)
	// 	);
	// 	let post_claim_native_tokens_amount = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
	//
	// 	assert!( pre_claim_native_tokens_amount < post_claim_native_tokens_amount);
	//
	// }


	promote_pool {
		// NOTE: that duplicates test XYK::liquidity_rewards_claim_W
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1000000000000;

		let asset_id_1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let asset_id_2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = asset_id_2 + 1;

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), 5000, asset_id_2.into(), 5000).unwrap();


	}: promote_pool(RawOrigin::Root, liquidity_asset_id)

	verify {
		assert_err!(
			Xyk::<T>::promote_pool(RawOrigin::Root.into(), liquidity_asset_id),
			Error::<T>::PoolAlreadyPromoted
		);
	}

	activate_liquidity_v2 {
		// activate :
		// 1 crate pool
		// 2 promote pool
		// 3 mint some tokens
		// 4 wait some time
		// 5 mint some

		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
		Xyk::<T>::promote_pool(RawOrigin::Root.into(), liquidity_asset_id).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
		let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

		Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		T::PoolPromoteApi::compute_issuance(1_u32).unwrap();
		frame_system::Pallet::<T>::set_block_number(101_000u32.into());

		assert!(
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller).into() > 0u128
		);

		assert!(
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller) < total_minted_liquidity
		);

	}: activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity, None)
	verify {

		Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), 2_u128*quater_of_minted_liquidity, None).unwrap();
		assert_err!(
			Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id, 1_u32.into(), None),
			Error::<T>::NotEnoughAssets
		)
	}

	deactivate_liquidity_v2 {
		// deactivate
		// 1 crate pool
		// 2 promote pool
		// 3 mint some tokens
		// deactivate some tokens (all or some - to be checked)

		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount:mangata_primitives::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id = non_native_asset_id2 + 1;

		Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
		Xyk::<T>::promote_pool(RawOrigin::Root.into(), liquidity_asset_id).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
		let half_of_minted_liquidity = total_minted_liquidity.into() / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

		Xyk::<T>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), half_of_minted_liquidity, None).unwrap();

		T::PoolPromoteApi::compute_issuance(1_u32).unwrap();
		frame_system::Pallet::<T>::set_block_number(101_000u32.into());

		assert!(
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller).into() > 0u128
		);

		assert!(
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller) < total_minted_liquidity
		);


	}: deactivate_liquidity_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity.into())
	verify {
	}




	impl_benchmark_test_suite!(Xyk, crate::mock::new_benchmark_ext(), crate::mock::Test)
}
