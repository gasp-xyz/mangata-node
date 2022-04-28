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

benchmarks! {

	start_ido {
		assert!(crate::BootstrapSchedule::<T>::get().is_none());
	}: start_ido(RawOrigin::Root, 123_456_789_u32.into(), 100_000_u32, 100_000_u32)
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
		let mga_provision_amount = ksm_provision_amount * T::KsmToMgaRatioDenominator::get() / T::KsmToMgaRatioNumerator::get();

		BootstrapPallet::<T>::start_ido(RawOrigin::Root.into(), 10_u32.into(), 10_u32, 10_u32).unwrap();
		// jump to public phase
		BootstrapPallet::<T>::on_initialize(20_u32.into());
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::MGATokenId::get(), mga_provision_amount).unwrap();

	}: provision(RawOrigin::Signed(caller.clone().into()), <T as Config>::KSMTokenId::get(), ksm_provision_amount)
	verify {
		assert_eq!(BootstrapPallet::<T>::provisions(caller, <T as Config>::KSMTokenId::get()), ksm_provision_amount);
	}

	claim_rewards {
		let caller: T::AccountId = whitelisted_caller();
		let mut token_id = 0;
		while token_id < <T as Config>::MGATokenId::get() ||
		token_id < <T as Config>::KSMTokenId::get() {
			token_id = <T as Config>::Currency::create(&caller, MILION.into()).expect("Token creation failed").into();
		}
		let ksm_provision_amount = 100_000_u128;
		let mga_provision_amount = ksm_provision_amount * T::KsmToMgaRatioDenominator::get() / T::KsmToMgaRatioNumerator::get();

		BootstrapPallet::<T>::start_ido(RawOrigin::Root.into(), 10_u32.into(), 10_u32, 10_u32).unwrap();
		BootstrapPallet::<T>::on_initialize(20_u32.into());
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::MGATokenId::get(), mga_provision_amount).unwrap();
		BootstrapPallet::<T>::provision(RawOrigin::Signed(caller.clone().into()).into(), <T as Config>::KSMTokenId::get(), ksm_provision_amount).unwrap();
		BootstrapPallet::<T>::on_initialize(30_u32.into());

		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), <T as Config>::KSMTokenId::get()), 0_u128);
		assert_eq!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), <T as Config>::MGATokenId::get()), 0_u128);

	}: claim_rewards(RawOrigin::Signed(caller.clone().into()))
	verify {
		assert_ne!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), <T as Config>::KSMTokenId::get()), 0_u128);
		assert_ne!(BootstrapPallet::<T>::claimed_rewards(caller.clone(), <T as Config>::MGATokenId::get()), 0_u128);
	}

	impl_benchmark_test_suite!(BootstrapPallet, crate::mock::new_test_ext(), crate::mock::Test)
}
