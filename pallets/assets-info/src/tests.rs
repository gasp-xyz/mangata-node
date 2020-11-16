use crate::{mock::*, AssetInfo, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn set_info_and_retrieve_works_ok() {
    new_test_ext().execute_with(|| {
        const ASSET_ID: u32 = 0;
        let info = AssetInfo {
            name: b"name".to_vec(),
            symbol: b"SYM".to_vec(),
            description: b"desc".to_vec(),
            decimals: 8,
        };
        // Dispatch a signed extrinsic.
        assert_ok!(AssetsInfoModule::set_info(
            Origin::root(),
            ASSET_ID,
            Some(info.name.clone()),
            Some(info.symbol.clone()),
            Some(info.description.clone()),
            Some(info.decimals)
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
            name: vec![],
            symbol: b"SYM".to_vec(),
            description: vec![],
            decimals: 8,
        };
        // Dispatch a signed extrinsic.
        assert_ok!(AssetsInfoModule::set_info(
            Origin::root(),
            ASSET_ID,
            None,
            // None,
            Some(info.symbol.clone()),
            None,
            Some(info.decimals)
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
            AssetsInfoModule::set_info(
                Origin::root(),
                0,
                None,
                None,
                None,
                None,
            ),
            Error::<Test>::TooShortSymbol
        );
    });
}
