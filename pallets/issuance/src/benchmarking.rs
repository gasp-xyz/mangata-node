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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use orml_tokens::MultiTokenCurrencyExtended;

use crate::{Pallet as Issuance, TgeInfo};

const SEED: u32 = u32::max_value();
const TGE_AMOUNT: u32 = 1_000_000;

fn create_tge_infos<T: Config>(x: u32) -> Vec<TgeInfo<T::AccountId, BalanceOf<T>>> {
	let mut tge_infos: Vec<TgeInfo<T::AccountId, BalanceOf<T>>> = Vec::new();
	for i in 0..x {
		tge_infos.push(TgeInfo { who: account("who", 0, SEED - i), amount: TGE_AMOUNT.into() });
	}
	tge_infos
}

benchmarks! {
	init_issuance_config {
		assert!(!IsTGEFinalized::<T>::get());
		assert_ok!(Issuance::<T>::finalize_tge(RawOrigin::Root.into()));
		assert!(
			IssuanceConfigStore::<T>::get().is_none()
		);

	}: _(RawOrigin::Root)
	verify {

		assert!(
			IssuanceConfigStore::<T>::get().is_some()
		);

	}

	finalize_tge {
		assert!(!IsTGEFinalized::<T>::get());
		assert!(
			IssuanceConfigStore::<T>::get().is_none()
		);
	}: _(RawOrigin::Root)
	verify {
		assert!(IsTGEFinalized::<T>::get());
		assert!(
			IssuanceConfigStore::<T>::get().is_none()
		);
	}

	execute_tge {

		let x in 3..100;

		assert!(!IsTGEFinalized::<T>::get());
		assert!(
			IssuanceConfigStore::<T>::get().is_none()
		);

		let tge_infos = create_tge_infos::<T>(x);

	}: _(RawOrigin::Root, tge_infos.clone())
	verify {
		assert!(!IsTGEFinalized::<T>::get());
		assert!(
			IssuanceConfigStore::<T>::get().is_none()
		);

		assert_eq!(TGETotal::<T>::get(), tge_infos.iter().fold(BalanceOf::<T>::zero(), |acc: BalanceOf<T>, tge_info| acc.saturating_add(tge_info.amount)));

		let lock_percent = Percent::from_percent(100)
				.checked_sub(&T::ImmediateTGEReleasePercent::get()).unwrap();

		for tge_info in tge_infos{
			assert_eq!(T::VestingProvider::vesting_balance(&tge_info.who, T::NativeCurrencyId::get()).unwrap(), lock_percent * tge_info.amount);
			assert_eq!(T::Tokens::free_balance(T::NativeCurrencyId::get(), &tge_info.who), tge_info.amount);
			assert_eq!(<T::Tokens as MultiTokenCurrencyExtended<T::AccountId>>::locked_balance(T::NativeCurrencyId::get(), &tge_info.who), lock_percent * tge_info.amount);
		}

	}

	impl_benchmark_test_suite!(Issuance, crate::mock::new_test_ext_without_issuance_config(), crate::mock::Test)
}
