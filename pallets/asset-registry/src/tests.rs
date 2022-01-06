// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Unit tests for asset registry module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{
	AssetRegistry, new_test_ext, Origin, Runtime,
};

#[test]
fn versioned_multi_location_convert_work() {
	new_test_ext().execute_with(|| {
		// v0
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));
		let location: MultiLocation = v0_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(1000))
			}
		);

		// v1
		let v1_location = VersionedMultiLocation::V1(MultiLocation {
			parents: 0,
			interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(1000)),
		});
		let location: MultiLocation = v1_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(1000))
			}
		);

		// handle all of VersionedMultiLocation
		assert!(match location.into() {
			VersionedMultiLocation::V0 { .. } | VersionedMultiLocation::V1 { .. } => true,
		});
	});
}

#[test]
fn register_asset_work() {
	new_test_ext().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));

		assert_ok!(AssetRegistry::register_asset(
			Origin::root(),
			Box::new(v0_location.clone()),
		));

		let location: MultiLocation = v0_location.try_into().unwrap();
		// System::assert_last_event(Event::AssetRegistry(crate::Event::AssetRegistered {
		// 	asset_id: TokenId::from(0u32),
		// 	asset_address: location.clone(),
		// }));

		assert_eq!(AssetLocations::<Runtime>::get(0), Some(location.clone()));
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(location),
			Some(0)
		);
	});
}

#[test]
fn register_asset_should_not_work() {
	new_test_ext().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));
		assert_ok!(AssetRegistry::register_asset(
			Origin::root(),
			Box::new(v0_location.clone()),
		));

		assert_noop!(
			AssetRegistry::register_asset(
				Origin::root(),
				Box::new(v0_location),
			),
			Error::<Runtime>::MultiLocationExisted
		);
	});
}

#[test]
fn update_asset_work() {
	new_test_ext().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));

		assert_ok!(AssetRegistry::register_asset(
			Origin::root(),
			Box::new(v0_location.clone()),
		));

		assert_ok!(AssetRegistry::update_asset(
			Origin::root(),
			0,
			Box::new(v0_location.clone()),
		));

		let location: MultiLocation = v0_location.try_into().unwrap();
		// System::assert_last_event(Event::AssetRegistry(crate::Event::ForeignAssetUpdated {
		// 	asset_id: 0,
		// 	asset_address: location.clone(),
		// }));

		assert_eq!(AssetLocations::<Runtime>::get(0), Some(location.clone()));
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(location.clone()),
			Some(0)
		);

		// modify location
		let new_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(2000)));
		assert_ok!(AssetRegistry::update_asset(
			Origin::root(),
			0,
			Box::new(new_location.clone()),
		));
		let new_location: MultiLocation = new_location.try_into().unwrap();
		assert_eq!(AssetLocations::<Runtime>::get(0), Some(new_location.clone()));
		assert_eq!(LocationToCurrencyIds::<Runtime>::get(location), None);
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(new_location),
			Some(0)
		);
	});
}

#[test]
fn update_asset_should_not_work() {
	new_test_ext().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));

		assert_noop!(
			AssetRegistry::update_asset(
				Origin::root(),
				0,
				Box::new(v0_location.clone()),
			),
			Error::<Runtime>::AssetIdNotExists
		);

		assert_ok!(AssetRegistry::register_asset(
			Origin::root(),
			Box::new(v0_location.clone()),
		));

		assert_ok!(AssetRegistry::update_asset(
			Origin::root(),
			0,
			Box::new(v0_location),
		));

		// existed location
		let new_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(2000)));
		assert_ok!(AssetRegistry::register_asset(
			Origin::root(),
			Box::new(new_location.clone()),
		));
		assert_noop!(
			AssetRegistry::update_asset(
				Origin::root(),
				0,
				Box::new(new_location),
			),
			Error::<Runtime>::MultiLocationExisted
		);
	});
}
