use crate::{mock::*, AssetInfo, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn set_info_and_retrieve_works_ok() {
    new_test_ext().execute_with(|| {
        let info = AssetInfo {
            name: b"name".to_vec(),
            symbol: b"SYM".to_vec(),
            description: b"desc".to_vec(),
            decimals: 8,
        };
        // Dispatch a signed extrinsic.
        assert_ok!(AssetsInfoModule::set_info(
            Origin::root(),
            0,
            Some(info.name.clone()),
            Some(info.symbol.clone()),
            Some(info.description.clone()),
            Some(info.decimals)
        ));
        // Read pallet storage and assert an expected result.
        assert_eq!(AssetsInfoModule::get_info(0), info);
    });
}

/*
#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            TemplateModule::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}
*/
