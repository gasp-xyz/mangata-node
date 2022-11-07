use frame_support::{assert_err, assert_ok, traits::OnInitialize};
use std::default::Default;

use mangata_types::{
	assets::{CustomMetadata, XykMetadata},
	AccountId,
};
use pallet_bootstrap::{BootstrapPhase, Phase};

use crate::setup::*;

const ASSET_ID_1: u32 = 1;

#[test]
fn bootstrap_updates_metadata_and_creates_pool_correctly() {
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
		assert_err!(
			pallet_xyk::Pallet::<Runtime>::create_pool(
				Origin::signed(AccountId::from(ALICE)),
				NATIVE_ASSET_ID,
				10_ * UNIT,
				ASSET_ID_1,
				10 * UNIT,
			),
			pallet_xyk::Error::<Runtime>::FunctionNotAvailableForThisToken
		);

		assert_ok!(pallet_bootstrap::Pallet::<Runtime>::schedule_bootstrap(
			Origin::root(),
			NATIVE_ASSET_ID,
			ASSET_ID_1,
			10_u32.into(),
			Some(10),
			10,
			None,
			false,
		));

		pallet_bootstrap::Pallet::<Runtime>::on_initialize(25_u32);
		assert_eq!(BootstrapPhase::Public, Phase::<Runtime>::get());

		assert_ok!(pallet_bootstrap::Pallet::<Runtime>::provision(
			Origin::signed(AccountId::from(ALICE)),
			ASSET_ID_1,
			10 * UNIT,
		));
		assert_ok!(pallet_bootstrap::Pallet::<Runtime>::provision(
			Origin::signed(AccountId::from(ALICE)),
			NATIVE_ASSET_ID,
			10 * UNIT,
		));

		assert_eq!(
			pallet_xyk::LiquidityAssets::<Runtime>::get((NATIVE_ASSET_ID, ASSET_ID_1)),
			None
		);

		pallet_bootstrap::Pallet::<Runtime>::on_initialize(40_u32);
		assert_eq!(BootstrapPhase::Finished, Phase::<Runtime>::get());

		assert_eq!(
			pallet_xyk::LiquidityAssets::<Runtime>::get((NATIVE_ASSET_ID, ASSET_ID_1)),
			Some(ASSET_ID_1 + 1)
		);

		let meta: Option<AssetMetadataOf> =
			orml_asset_registry::Metadata::<Runtime>::get(ASSET_ID_1);
		assert_eq!(meta.unwrap().additional.xyk.unwrap().pool_creation_disabled, false);
	});
}
