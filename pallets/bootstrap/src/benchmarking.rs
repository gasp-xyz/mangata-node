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
use frame_support::assert_ok;

use crate::Pallet as BootstrapPallet;

const MILION: u128 = 1_000__000_000__000_000;
const DEFAULT_RATIO: (u128, u128) = (1_u128, 10_000_u128);

benchmarks! {

	schedule_bootstrap {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		assert!(crate::BootstrapSchedule::<T>::get().is_none());
		let caller: T::AccountId = whitelisted_caller();
		let first_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		let second_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
	}: schedule_bootstrap(RawOrigin::Root, first_token_id, second_token_id, 123_456_789_u32.into(), Some(100_000_u32), 100_000_u32, Some(DEFAULT_RATIO), true)
	verify {
		assert!(crate::BootstrapSchedule::<T>::get().is_some());
	}

	provision {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		let caller: T::AccountId = whitelisted_caller();
		let first_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		let second_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		let ksm_provision_amount = 100_000_u128;
		let mga_provision_amount = ksm_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;

		BootstrapPallet::<T>::schedule_bootstrap(RawOrigin::Root.into(), first_token_id, second_token_id, 10_u32.into(), Some(10_u32), 10_u32, Some(DEFAULT_RATIO), false).unwrap();
		// jump to public phase
		BootstrapPallet::<T>::on_initialize(20_u32.into());
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), second_token_id, mga_provision_amount).unwrap();

	}: provision(RawOrigin::Signed(caller.clone().into()), first_token_id, ksm_provision_amount)
	verify {
		assert_eq!(BootstrapPallet::<T>::provisions(caller, first_token_id), ksm_provision_amount);
	}

	// provision_vested {
	// 	frame_system::Pallet::<T>::set_block_number(1_u32.into());
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	let first_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
	// 	let second_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();

	// 	let ksm_provision_amount = 100_000_u128;
	// 	let mga_provision_amount = ksm_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;

	// 	let lock = 100_000_000_u128;

	// 	frame_system::Pallet::<T>::set_block_number(1_u32.into());
	// 	<T as Config>::VestingProvider::lock_tokens(&caller, first_token_id.into(), (ksm_provision_amount*2).into(), None, lock.into()).unwrap();
	// 	frame_system::Pallet::<T>::set_block_number(2_u32.into());

	// 	BootstrapPallet::<T>::schedule_bootstrap(RawOrigin::Root.into(), first_token_id, second_token_id, Some(10_u32.into()), 10_u32, 10_u32, Some(DEFAULT_RATIO), false).unwrap();
	// 	// jump to public phase
	// 	BootstrapPallet::<T>::on_initialize(20_u32.into());
	// 	BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), second_token_id, mga_provision_amount).unwrap();

	// 	assert_eq!(BootstrapPallet::<T>::vested_provisions(caller.clone(), first_token_id), (0, 0, 0));
	// }: provision_vested(RawOrigin::Signed(caller.clone().into()), first_token_id, ksm_provision_amount)
	// verify {
	// 	assert_eq!(BootstrapPallet::<T>::vested_provisions(caller, first_token_id).0, ksm_provision_amount);
	// }

	claim_and_activate_liquidity_tokens {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		let caller: T::AccountId = whitelisted_caller();
		let first_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		let second_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		let liquidity_asset_id = second_token_id + 1;

		let ksm_provision_amount = 100_000_u128;
		// let ksm_vested_provision_amount = 300_000_u128;
		let ksm_vested_provision_amount = 0_u128;
		let mga_provision_amount = ksm_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;
		let mga_vested_provision_amount = ksm_vested_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;
		let total_ksm_provision = ksm_provision_amount + ksm_vested_provision_amount;
		let total_mga_provision = mga_provision_amount + mga_vested_provision_amount;
		let total_provision = total_ksm_provision + total_mga_provision;
		let lock = 150_u128;

		<T as Config>::VestingProvider::lock_tokens(&caller, first_token_id.into(), (ksm_provision_amount + ksm_vested_provision_amount).into(), None, lock.into()).unwrap();
		<T as Config>::VestingProvider::lock_tokens(&caller, second_token_id.into(), (mga_provision_amount + mga_vested_provision_amount).into(), None, lock.into()).unwrap();

		BootstrapPallet::<T>::schedule_bootstrap(RawOrigin::Root.into(), first_token_id, second_token_id, 10_u32.into(), Some(10_u32), 10_u32, Some(DEFAULT_RATIO), false).unwrap();
		BootstrapPallet::<T>::on_initialize(20_u32.into());
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), second_token_id, mga_provision_amount).unwrap();
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), first_token_id, ksm_provision_amount).unwrap();
		// BootstrapPallet::<T>::provision_vested(RawOrigin::Signed(caller.clone().into()).into(), second_token_id, mga_vested_provision_amount).unwrap();
		// BootstrapPallet::<T>::provision_vested(RawOrigin::Signed(caller.clone().into()).into(), first_token_id, ksm_vested_provision_amount).unwrap();
		BootstrapPallet::<T>::on_initialize(30_u32.into());

		assert_eq!(BootstrapPallet::<T>::phase(), BootstrapPhase::Finished);
		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), first_token_id), 0_u128);
		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), second_token_id), 0_u128);
		assert_eq!(BootstrapPallet::<T>::valuations(), (total_mga_provision, total_ksm_provision));
		assert_eq!(BootstrapPallet::<T>::provisions(caller.clone(), first_token_id), (ksm_provision_amount));
		assert_eq!(BootstrapPallet::<T>::provisions(caller.clone(), second_token_id), (mga_provision_amount));
		// assert_eq!(BootstrapPallet::<T>::vested_provisions(caller.clone(), first_token_id), (ksm_vested_provision_amount, 1, lock + 1));
		// assert_eq!(BootstrapPallet::<T>::vested_provisions(caller.clone(), second_token_id), (mga_vested_provision_amount, 1, lock + 1));

		// promote pool
		pallet_issuance::PromotedPoolsRewards::<T>::insert(liquidity_asset_id, 0_u128);

	}: claim_and_activate_liquidity_tokens(RawOrigin::Signed(caller.clone().into()))
	verify {
		let (total_mga_provision, total_ksm_provision) = BootstrapPallet::<T>::valuations();
		let ksm_non_vested_rewards = total_provision / 2 / 2 * ksm_provision_amount / total_ksm_provision;
		let ksm_vested_rewards = total_provision / 2 / 2 * ksm_vested_provision_amount / total_ksm_provision;
		let mga_non_vested_rewards = total_provision / 2 / 2 * mga_provision_amount / total_mga_provision;
		let mga_vested_rewards = total_provision / 2 / 2 * mga_vested_provision_amount / total_mga_provision;

		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), first_token_id), ksm_vested_rewards + ksm_non_vested_rewards);
		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), second_token_id), mga_vested_rewards + mga_non_vested_rewards);
	}

	finalize {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		let caller: T::AccountId = whitelisted_caller();
		let first_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		let second_token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();

		let ksm_provision_amount = 100_000_u128;
		// let ksm_vested_provision_amount = 300_000_u128;
		let ksm_vested_provision_amount = 0_u128;
		let mga_provision_amount = ksm_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;
		let mga_vested_provision_amount = ksm_vested_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;
		let total_ksm_provision = ksm_provision_amount + ksm_vested_provision_amount;
		let total_mga_provision = mga_provision_amount + mga_vested_provision_amount;
		let total_provision = total_ksm_provision + total_mga_provision;
		let lock = 150_u128;

		<T as Config>::VestingProvider::lock_tokens(&caller, first_token_id.into(), (ksm_provision_amount + ksm_vested_provision_amount).into(), None, lock.into()).unwrap();
		<T as Config>::VestingProvider::lock_tokens(&caller, second_token_id.into(), (mga_provision_amount + mga_vested_provision_amount).into(), None, lock.into()).unwrap();

		BootstrapPallet::<T>::schedule_bootstrap(RawOrigin::Root.into(), first_token_id, second_token_id, 10_u32.into(), Some(10_u32), 10_u32, Some(DEFAULT_RATIO), false).unwrap();
		BootstrapPallet::<T>::on_initialize(20_u32.into());
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), second_token_id, mga_provision_amount).unwrap();
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), first_token_id, ksm_provision_amount).unwrap();
		// BootstrapPallet::<T>::provision_vested(RawOrigin::Signed(caller.clone().into()).into(), second_token_id, mga_vested_provision_amount).unwrap();
		// BootstrapPallet::<T>::provision_vested(RawOrigin::Signed(caller.clone().into()).into(), first_token_id, ksm_vested_provision_amount).unwrap();
		BootstrapPallet::<T>::on_initialize(30_u32.into());

		BootstrapPallet::<T>::claim_liquidity_tokens(RawOrigin::Signed(caller.clone().into()).into()).unwrap();
		assert_eq!(BootstrapPallet::<T>::phase(), BootstrapPhase::Finished);

		assert_ok!(BootstrapPallet::<T>::pre_finalize(RawOrigin::Signed(caller.clone().into()).into()));
	}: finalize(RawOrigin::Signed(caller.clone().into()))
	verify {
		assert_eq!(BootstrapPallet::<T>::phase(), BootstrapPhase::BeforeStart);
	}

	impl_benchmark_test_suite!(BootstrapPallet, crate::mock::new_test_ext(), crate::mock::Test)
}
