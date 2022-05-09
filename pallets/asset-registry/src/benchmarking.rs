// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
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

use crate::*;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

benchmarks! {

	register_asset {
		let location = VersionedMultiLocation::V1(MultiLocation {
			parents: 0,
			interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(1000)),
		});

		let next_currency_id: TokenId = <T as pallet::Config>::Currency::get_next_currency_id().into();

	}: _(RawOrigin::Root, Box::new(location.clone()))
	verify {
		assert_eq!(Pallet::<T>::asset_locations(next_currency_id), Some(location.clone().try_into().unwrap()));
		assert_eq!(Pallet::<T>::location_to_currency_ids(MultiLocation::try_from(location).unwrap()), Some(next_currency_id));
	}

	update_asset {
		let location = VersionedMultiLocation::V1(MultiLocation {
			parents: 0,
			interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(1000)),
		});

		let next_currency_id: TokenId =  <T as pallet::Config>::Currency::get_next_currency_id().into();

		Pallet::<T>::register_asset(RawOrigin::Root.into(), Box::new(location.clone()))?;

		assert_eq!(Pallet::<T>::asset_locations(next_currency_id), Some(location.clone().try_into().unwrap()));
		assert_eq!(Pallet::<T>::location_to_currency_ids(MultiLocation::try_from(location).unwrap()), Some(next_currency_id));

		let location = VersionedMultiLocation::V1(MultiLocation {
			parents: 0,
			interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(3000)),
		});

	}: _(RawOrigin::Root, next_currency_id, Box::new(location.clone()))
	verify{

		assert_eq!(Pallet::<T>::asset_locations(next_currency_id), Some(location.clone().try_into().unwrap()));
		assert_eq!(Pallet::<T>::location_to_currency_ids(MultiLocation::try_from(location).unwrap()), Some(next_currency_id));
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime)
}
