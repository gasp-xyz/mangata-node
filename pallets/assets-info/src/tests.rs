// Copyright (C) 2020 Mangata team

use crate::{
	mock::{AssetsInfoModule, *},
	AssetInfo, Error, Event,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn set_info_and_retrieve_works_ok() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		const ASSET_ID: u32 = 0;
		let info = AssetInfo {
			name: Some(b"name".to_vec()),
			symbol: Some(b"SYM".to_vec()),
			description: Some(b"desc".to_vec()),
			decimals: Some(8),
		};
		// Dispatch a signed extrinsic.
		assert_ok!(AssetsInfoModule::set_info(
			Origin::root(),
			ASSET_ID,
			info.name.clone(),
			info.symbol.clone(),
			info.description.clone(),
			info.decimals,
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(AssetsInfoModule::get_info(ASSET_ID), info);

		let info_stored_event =
			crate::mock::Event::AssetsInfoModule(Event::InfoStored(ASSET_ID, info));

		assert!(System::events().iter().any(|record| record.event == info_stored_event));
	});
}

#[test]
fn set_info_optional_and_retrieve_works_ok() {
	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 0;
		let info = AssetInfo {
			name: None,
			symbol: Some(b"SYM".to_vec()),
			description: None,
			decimals: Some(8),
		};
		// Dispatch a signed extrinsic.
		assert_ok!(AssetsInfoModule::set_info(
			Origin::root(),
			ASSET_ID,
			None,
			// None,
			info.symbol.clone(),
			None,
			info.decimals,
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(AssetsInfoModule::get_info(ASSET_ID), info);
	});
}

#[test]
fn min_length_name_check() {
	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 0;
		let info =
			AssetInfo { name: Some(vec![]), symbol: None, description: None, decimals: None };
		// Dispatch a signed extrinsic.

		assert_noop!(
			AssetsInfoModule::set_info(
				Origin::root(),
				ASSET_ID,
				info.name.clone(),
				info.symbol.clone(),
				info.description.clone(),
				info.decimals,
			),
			Error::<Test>::TooShortName
		);
		// Read pallet storage and assert an expected result.
	});
}

#[test]
fn min_length_symbol_check() {
	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 0;
		let info =
			AssetInfo { name: None, symbol: Some(vec![]), description: None, decimals: Some(8) };
		// Dispatch a signed extrinsic.
		assert_noop!(
			AssetsInfoModule::set_info(
				Origin::root(),
				ASSET_ID,
				info.name.clone(),
				info.symbol.clone(),
				info.description.clone(),
				info.decimals,
			),
			Error::<Test>::TooShortSymbol
		);
		// Read pallet storage and assert an expected result.
	});
}

#[test]
fn min_length_description_check() {
	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 0;
		let info =
			AssetInfo { name: None, symbol: None, description: Some(vec![]), decimals: Some(8) };
		// Dispatch a signed extrinsic.
		assert_noop!(
			AssetsInfoModule::set_info(
				Origin::root(),
				ASSET_ID,
				info.name.clone(),
				info.symbol.clone(),
				info.description.clone(),
				info.decimals,
			),
			Error::<Test>::TooShortDescription
		);
		// Read pallet storage and assert an expected result.
	});
}

#[test]
fn max_length_name_check() {
	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 0;
		let info = AssetInfo {
			name: Some(b"veryLongName".to_vec()),
			symbol: Some(b"SYM".to_vec()),
			description: Some(b"desc".to_vec()),
			decimals: Some(8),
		};
		// Dispatch a signed extrinsic.
		assert_noop!(
			AssetsInfoModule::set_info(
				Origin::root(),
				ASSET_ID,
				info.name.clone(),
				info.symbol.clone(),
				info.description.clone(),
				info.decimals,
			),
			Error::<Test>::TooLongName
		);
		// Read pallet storage and assert an expected result.
	});
}

#[test]
fn max_length_symbol_check() {
	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 0;
		let info = AssetInfo {
			name: Some(b"name".to_vec()),
			symbol: Some(b"veryLongSymbol".to_vec()),
			description: Some(b"desc".to_vec()),
			decimals: Some(8),
		};
		// Dispatch a signed extrinsic.
		assert_noop!(
			AssetsInfoModule::set_info(
				Origin::root(),
				ASSET_ID,
				info.name.clone(),
				info.symbol.clone(),
				info.description.clone(),
				info.decimals,
			),
			Error::<Test>::TooLongSymbol
		);
		// Read pallet storage and assert an expected result.
	});
}

#[test]
fn max_length_description_check() {
	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 0;
		let info = AssetInfo {
			name: Some(b"name".to_vec()),
			symbol: Some(b"SYM".to_vec()),
			description: Some(b"veryLongDescription".to_vec()),
			decimals: Some(8),
		};
		// Dispatch a signed extrinsic.
		assert_noop!(
			AssetsInfoModule::set_info(
				Origin::root(),
				ASSET_ID,
				info.name.clone(),
				info.symbol.clone(),
				info.description.clone(),
				info.decimals,
			),
			Error::<Test>::TooLongDescription
		);
		// Read pallet storage and assert an expected result.
	});
}

#[test]
fn max_decimals_check() {
	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 0;
		let info = AssetInfo {
			name: Some(b"name".to_vec()),
			symbol: Some(b"SYM".to_vec()),
			description: Some(b"desc".to_vec()),
			decimals: Some(11),
		};
		// Dispatch a signed extrinsic.
		assert_noop!(
			AssetsInfoModule::set_info(
				Origin::root(),
				ASSET_ID,
				info.name.clone(),
				info.symbol.clone(),
				info.description.clone(),
				info.decimals,
			),
			Error::<Test>::DecimalsOutOfRange
		);
		// Read pallet storage and assert an expected result.
	});
}
