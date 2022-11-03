use frame_support::{assert_noop, assert_ok};
use std::default::Default;

use mangata_types::{
	assets::{CustomMetadata, XykMetadata},
	AccountId,
};

use crate::setup::*;

const ASSET_ID_1: u32 = 1;

#[test]
fn create_pool_works() {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
		],
		assets: vec![(
			ASSET_ID_1,
			AssetMetadataOf {
				decimals: 18,
				name: b"Asset".to_vec(),
				symbol: b"Asset".to_vec(),
				location: None,
				existential_deposit: Default::default(),
				additional: CustomMetadata {
					xyk: Some(XykMetadata { pool_creation_disabled: false }),
					..CustomMetadata::default()
				},
			},
		)],
		..ExtBuilder::default()
	}
	.build()
	.execute_with(|| {
		assert_ok!(pallet_xyk::Pallet::<Runtime>::create_pool(
			Origin::signed(AccountId::from(ALICE)),
			NATIVE_ASSET_ID,
			10_ * UNIT,
			ASSET_ID_1,
			10 * UNIT,
		));
	});
}

#[test]
fn create_pool_is_disabled() {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
		],
		assets: vec![(
			ASSET_ID_1,
			AssetMetadataOf {
				decimals: 18,
				name: b"Asset".to_vec(),
				symbol: b"Asset".to_vec(),
				location: None,
				existential_deposit: Default::default(),
				additional: CustomMetadata {
					xyk: Some(XykMetadata { pool_creation_disabled: true }),
					..CustomMetadata::default()
				},
			},
		)],
		..ExtBuilder::default()
	}
	.build()
	.execute_with(|| {
		assert_noop!(
			pallet_xyk::Pallet::<Runtime>::create_pool(
				Origin::signed(AccountId::from(ALICE)),
				NATIVE_ASSET_ID,
				10_ * UNIT,
				ASSET_ID_1,
				10 * UNIT,
			),
			pallet_xyk::Error::<Runtime>::FunctionNotAvailableForThisToken
		);
	});
}
