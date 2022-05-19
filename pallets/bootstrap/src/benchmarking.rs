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

use crate::Pallet as BootstrapPallet;

const MILION: u128 = 1_000__000_000__000_000;
const DEFAULT_RATIO: (u128, u128) = (1_u128, 10_000_u128);

benchmarks! {

	start_ido {
		assert!(crate::BootstrapSchedule::<T>::get().is_none());
	}: start_ido(RawOrigin::Root, 123_456_789_u32.into(), 100_000_u32, 100_000_u32, DEFAULT_RATIO)
	verify {
		assert!(crate::BootstrapSchedule::<T>::get().is_some());
	}

	provision {
		let caller: T::AccountId = whitelisted_caller();
		let mut token_id = 0;
		while token_id < <T as Config>::MGATokenId::get() ||
		token_id < <T as Config>::KSMTokenId::get() {
			token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		}
		let ksm_provision_amount = 100_000_u128;
		let mga_provision_amount = ksm_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;
		<T as Config>::Currency::mint(<T as Config>::MGATokenId::get().into(), &caller, MILION.into()).expect("Token creation failed");
		<T as Config>::Currency::mint(<T as Config>::KSMTokenId::get().into(), &caller, MILION.into()).expect("Token creation failed");

		BootstrapPallet::<T>::start_ido(RawOrigin::Root.into(), 10_u32.into(), 10_u32, 10_u32, DEFAULT_RATIO).unwrap();
		// jump to public phase
		BootstrapPallet::<T>::on_initialize(20_u32.into());
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::MGATokenId::get(), mga_provision_amount).unwrap();

	}: provision(RawOrigin::Signed(caller.clone().into()), <T as Config>::KSMTokenId::get(), ksm_provision_amount)
	verify {
		assert_eq!(BootstrapPallet::<T>::provisions(caller, <T as Config>::KSMTokenId::get()), ksm_provision_amount);
	}


	provision_vested {
		let caller: T::AccountId = whitelisted_caller();
		let mut token_id = 0;
		while token_id < <T as Config>::MGATokenId::get() ||
		token_id < <T as Config>::KSMTokenId::get() {
			token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		}
		<T as Config>::Currency::mint(<T as Config>::MGATokenId::get().into(), &caller, MILION.into()).expect("Token creation failed");
		<T as Config>::Currency::mint(<T as Config>::KSMTokenId::get().into(), &caller, MILION.into()).expect("Token creation failed");
		let ksm_provision_amount = 100_000_u128;
		let mga_provision_amount = ksm_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;

		let lock = 100_000_000_u128;

		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		<T as Config>::VestingProvider::lock_tokens(&caller, <T as Config>::KSMTokenId::get().into(), (ksm_provision_amount*2).into(), lock.into()).unwrap();
		frame_system::Pallet::<T>::set_block_number(2_u32.into());

		BootstrapPallet::<T>::start_ido(RawOrigin::Root.into(), 10_u32.into(), 10_u32, 10_u32, DEFAULT_RATIO).unwrap();
		// jump to public phase
		BootstrapPallet::<T>::on_initialize(20_u32.into());
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::MGATokenId::get(), mga_provision_amount).unwrap();

	}: provision_vested(RawOrigin::Signed(caller.clone().into()), <T as Config>::KSMTokenId::get(), ksm_provision_amount)
	verify {
		assert_eq!(BootstrapPallet::<T>::vested_provisions(caller, <T as Config>::KSMTokenId::get()).0, (ksm_provision_amount));
	}

	claim_rewards {
		let caller: T::AccountId = whitelisted_caller();
		let mut token_id = 0;
		while token_id < <T as Config>::MGATokenId::get() ||
		token_id < <T as Config>::KSMTokenId::get() {
			token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		}
		<T as Config>::Currency::mint(<T as Config>::MGATokenId::get().into(), &caller, MILION.into()).expect("Token creation failed");
		<T as Config>::Currency::mint(<T as Config>::KSMTokenId::get().into(), &caller, MILION.into()).expect("Token creation failed");
		let ksm_provision_amount = 100_000_u128;
		let ksm_vested_provision_amount = 300_000_u128;
		let mga_provision_amount = ksm_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;
		let mga_vested_provision_amount = ksm_vested_provision_amount * DEFAULT_RATIO.1 / DEFAULT_RATIO.0;
		let total_ksm_provision = ksm_provision_amount + ksm_vested_provision_amount;
		let total_mga_provision = mga_provision_amount + mga_vested_provision_amount;
		let total_provision = total_ksm_provision + total_mga_provision;
		let lock = 150_u128;

		<T as Config>::VestingProvider::lock_tokens(&caller, <T as Config>::KSMTokenId::get().into(), (ksm_provision_amount + ksm_vested_provision_amount).into(), lock.into()).unwrap();
		<T as Config>::VestingProvider::lock_tokens(&caller, <T as Config>::MGATokenId::get().into(), (mga_provision_amount + mga_vested_provision_amount).into(), lock.into()).unwrap();

		BootstrapPallet::<T>::start_ido(RawOrigin::Root.into(), 10_u32.into(), 10_u32, 10_u32, DEFAULT_RATIO).unwrap();
		BootstrapPallet::<T>::on_initialize(20_u32.into());
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::MGATokenId::get(), mga_provision_amount).unwrap();
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::KSMTokenId::get(), ksm_provision_amount).unwrap();
		BootstrapPallet::<T>::provision_vested(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::MGATokenId::get(), mga_vested_provision_amount).unwrap();
		BootstrapPallet::<T>::provision_vested(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::KSMTokenId::get(), ksm_vested_provision_amount).unwrap();
		BootstrapPallet::<T>::on_initialize(30_u32.into());

		assert_eq!(BootstrapPallet::<T>::phase(), BootstrapPhase::Finished);
		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), <T as Config>::KSMTokenId::get()), 0_u128);
		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), <T as Config>::MGATokenId::get()), 0_u128);
		assert_eq!(BootstrapPallet::<T>::valuations(), (total_mga_provision, total_ksm_provision));
		assert_eq!(BootstrapPallet::<T>::provisions(caller.clone(), <T as Config>::KSMTokenId::get()), (ksm_provision_amount));
		assert_eq!(BootstrapPallet::<T>::provisions(caller.clone(), <T as Config>::MGATokenId::get()), (mga_provision_amount));
		assert_eq!(BootstrapPallet::<T>::vested_provisions(caller.clone(), <T as Config>::KSMTokenId::get()), (ksm_vested_provision_amount, lock + 1));
		assert_eq!(BootstrapPallet::<T>::vested_provisions(caller.clone(), <T as Config>::MGATokenId::get()), (mga_vested_provision_amount, lock + 1));

	}: claim_rewards(RawOrigin::Signed(caller.clone().into()))
	verify {
		let (total_mga_provision, total_ksm_provision) = BootstrapPallet::<T>::valuations();
		let ksm_non_vested_rewards = total_provision / 2 / 2 * ksm_provision_amount / total_ksm_provision;
		let ksm_vested_rewards = total_provision / 2 / 2 * ksm_vested_provision_amount / total_ksm_provision;
		let mga_non_vested_rewards = total_provision / 2 / 2 * mga_provision_amount / total_mga_provision;
		let mga_vested_rewards = total_provision / 2 / 2 * mga_vested_provision_amount / total_mga_provision;

		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), <T as Config>::KSMTokenId::get()), ksm_vested_rewards + ksm_non_vested_rewards);
		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), <T as Config>::MGATokenId::get()), mga_vested_rewards + mga_non_vested_rewards);
	}

	impl_benchmark_test_suite!(BootstrapPallet, crate::mock::new_test_ext(), crate::mock::Test)
}
