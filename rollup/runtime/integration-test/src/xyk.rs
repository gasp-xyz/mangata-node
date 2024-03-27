use crate::setup::*;

const ASSET_ID_1: u32 = 1;

fn test_env(xyk_metadata: Option<XykMetadata>) -> TestExternalities {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
		],
		assets: vec![(
			ASSET_ID_1,
			AssetMetadataOf {
				decimals: 18,
				name: BoundedVec::truncate_from(b"Asset".to_vec()),
				symbol: BoundedVec::truncate_from(b"Asset".to_vec()),
				location: None,
				existential_deposit: Default::default(),
				additional: CustomMetadata { xyk: xyk_metadata, ..CustomMetadata::default() },
			},
		)],
		..ExtBuilder::default()
	}
	.build()
}

fn create_pool() -> DispatchResultWithPostInfo {
	pallet_xyk::Pallet::<Runtime>::create_pool(
		RuntimeOrigin::signed(AccountId::from(ALICE)),
		NATIVE_ASSET_ID,
		10_ * UNIT,
		ASSET_ID_1,
		10 * UNIT,
	)
}

#[test]
fn create_pool_works_meta_allowed() {
	test_env(Some(XykMetadata { operations_disabled: false })).execute_with(|| {
		assert_ok!(create_pool());
	});
}

#[test]
fn create_pool_works_no_meta() {
	test_env(None).execute_with(|| {
		assert_ok!(create_pool());
	});
}

#[test]
fn create_pool_disabled_meta_disabled() {
	test_env(Some(XykMetadata { operations_disabled: true })).execute_with(|| {
		assert_err!(create_pool(), pallet_xyk::Error::<Runtime>::FunctionNotAvailableForThisToken);
	});
}

use codec::Decode;
use hex_literal::hex;

#[test]
fn test_decode_extrinsic() {
	test_env(None).execute_with(|| {
		UncheckedExtrinsic::decode(&mut hex!("b0040a00008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4800000000821a0600").as_ref()).unwrap();
	});
}
