// Copyright (C) 2020 Mangata team

use crate::{mock::*, AssetInfo, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn set_info_and_retrieve_works_ok() {
    new_test_ext().execute_with(|| {
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
            info.decimals.clone(),
        ));
        // Read pallet storage and assert an expected result.
        assert_eq!(AssetsInfoModule::get_info(ASSET_ID), info);
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
fn correct_error_for_invalid_symbol_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            AssetsInfoModule::set_info(Origin::root(), 0, None, Some(vec![]), None, None,),
            Error::<Test>::TooShortSymbol
        );
    });
}
