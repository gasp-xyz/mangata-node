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

use crate::Pallet as Xyk;

const SEED: u32 = 0;
// existential deposit multiplier
const ED_MULTIPLIER: u32 = 10;

benchmarks! {
	// Benchmark `transfer` extrinsic with the worst possible conditions:
	// * Transfer will kill the sender account.
	// * Transfer will create the recipient account.
	create_pool {
		// let existential_deposit = T::ExistentialDeposit::get();
		let caller: T::AccountId = whitelisted_caller();
        //
		// // Give some multiple of the existential deposit
		// let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
		// let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&caller, balance);
        //
		// // Transfer `e - 1` existential deposits + 1 unit, which guarantees to create one account,
		// // and reap this user.
		// let recipient: T::AccountId = account("recipient", 0, SEED);
		// let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		// let transfer_amount = existential_deposit.saturating_mul((ED_MULTIPLIER - 1).into()) + 1u32.into();
	}: create_pool(RawOrigin::Signed(caller.into()), 0, 0, 0, 0)
	verify {
		// assert_eq!(Xyk::<T, I>::free_balance(&caller), Zero::zero());
		// assert_eq!(Xyk::<T, I>::free_balance(&recipient), transfer_amount);
	}


    // impl_benchmark_test_suite!(Xyk, crate::mock::ExtBuilder::default().build(), crate::mock::Runtime)    
}
