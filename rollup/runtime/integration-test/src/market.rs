use crate::setup::*;

use pallet_market::PoolKind;
use sp_runtime::{traits::Zero, DispatchResult};

const ASSET_ID_1: u32 = 1;
const ASSET_ID_2: u32 = 2;
const ASSET_ID_3: u32 = 3;
const ASSET_ID_4_DISABLED: u32 = 4;
const ASSET_ID_5_UNREGISTERED: u32 = 5;
const POOL_ID_1: u32 = 6;
const POOL_ID_2: u32 = 7;
const POOL_ID_3: u32 = 8;

fn test_env() -> TestExternalities {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_2, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_3, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_4_DISABLED, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_5_UNREGISTERED, 100 * UNIT),
		],
		assets: vec![
			(
				NATIVE_ASSET_ID,
				AssetMetadataOf {
					decimals: 18,
					name: BoundedVec::truncate_from(b"Asset".to_vec()),
					symbol: BoundedVec::truncate_from(b"Asset".to_vec()),
					existential_deposit: Default::default(),
					additional: Default::default(),
				},
			),
			(
				ASSET_ID_1,
				AssetMetadataOf {
					decimals: 18,
					name: BoundedVec::truncate_from(b"Asset".to_vec()),
					symbol: BoundedVec::truncate_from(b"Asset".to_vec()),
					existential_deposit: Default::default(),
					additional: Default::default(),
				},
			),
			(
				ASSET_ID_2,
				AssetMetadataOf {
					decimals: 18,
					name: BoundedVec::truncate_from(b"Asset".to_vec()),
					symbol: BoundedVec::truncate_from(b"Asset".to_vec()),
					existential_deposit: Default::default(),
					additional: Default::default(),
				},
			),
			(
				ASSET_ID_3,
				AssetMetadataOf {
					decimals: 18,
					name: BoundedVec::truncate_from(b"Asset".to_vec()),
					symbol: BoundedVec::truncate_from(b"Asset".to_vec()),
					existential_deposit: Default::default(),
					additional: Default::default(),
				},
			),
			(
				ASSET_ID_4_DISABLED,
				AssetMetadataOf {
					decimals: 18,
					name: BoundedVec::truncate_from(b"Asset".to_vec()),
					symbol: BoundedVec::truncate_from(b"Asset".to_vec()),
					existential_deposit: Default::default(),
					additional: CustomMetadata {
						xyk: Some(XykMetadata { operations_disabled: true }),
						..CustomMetadata::default()
					},
				},
			),
		],
		..ExtBuilder::default()
	}
	.build()
}

type Market = pallet_market::Pallet<Runtime>;

fn origin() -> RuntimeOrigin {
	RuntimeOrigin::signed(AccountId::from(ALICE))
}

fn create_pool(kind: PoolKind, assets: (u32, u32)) -> DispatchResult {
	Market::create_pool(origin(), kind, assets.0, 10 * UNIT, assets.1, 10 * UNIT)
}

pub(crate) fn events() -> Vec<RuntimeEvent> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| match e {
			RuntimeEvent::Xyk(_) | RuntimeEvent::Market(_) | RuntimeEvent::StableSwap(_) => Some(e),
			_ => None,
		})
		.collect::<Vec<_>>()
}

#[test]
fn create_pool_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));
	})
}

#[test]
fn add_liquidity_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

		assert_ok!(Market::mint_liquidity(origin(), POOL_ID_1, NATIVE_ASSET_ID, UNIT, 10 * UNIT));
		assert_ok!(Market::mint_liquidity(origin(), POOL_ID_2, NATIVE_ASSET_ID, UNIT, 10 * UNIT));
	})
}

#[test]
fn remove_liquidity_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

		assert_ok!(Market::burn_liquidity(origin(), POOL_ID_1, UNIT, 0, 0));
		assert_ok!(Market::burn_liquidity(origin(), POOL_ID_2, UNIT, 0, 0));
	})
}

#[test]
fn multiswap_should_work_xyk() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::Xyk, (ASSET_ID_1, ASSET_ID_2)));
		assert_ok!(create_pool(PoolKind::Xyk, (ASSET_ID_2, ASSET_ID_3)));

		assert_ok!(Market::multiswap_asset(
			origin(),
			vec![POOL_ID_1, POOL_ID_2, POOL_ID_3],
			NATIVE_ASSET_ID,
			UNIT,
			ASSET_ID_3,
			Zero::zero(),
		));
	})
}

#[test]
fn multiswap_should_work_stable_swap() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (ASSET_ID_1, ASSET_ID_2)));
		assert_ok!(create_pool(PoolKind::StableSwap, (ASSET_ID_2, ASSET_ID_3)));

		assert_ok!(Market::multiswap_asset(
			origin(),
			vec![POOL_ID_1, POOL_ID_2, POOL_ID_3],
			NATIVE_ASSET_ID,
			UNIT,
			ASSET_ID_3,
			Zero::zero(),
		));
	})
}

#[test]
fn multiswap_should_work_mixed() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (ASSET_ID_1, ASSET_ID_2)));
		assert_ok!(create_pool(PoolKind::Xyk, (ASSET_ID_2, ASSET_ID_3)));

		assert_ok!(Market::multiswap_asset(
			origin(),
			vec![POOL_ID_1, POOL_ID_2, POOL_ID_3],
			NATIVE_ASSET_ID,
			UNIT,
			ASSET_ID_3,
			Zero::zero(),
		));

		System::assert_last_event(RuntimeEvent::Market(pallet_market::Event::AssetsSwapped {
			who: AccountId::from(ALICE),
			swap_pool_list: vec![6, 7, 8],
			swap_assets_list: vec![(0, 1), (1, 2), (2, 3)],
			amount_in: UNIT,
			amount_out: 826609041367995045,
		}));
	})
}

#[test]
fn multiswap_buy_should_work_mixed() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (ASSET_ID_1, ASSET_ID_2)));
		assert_ok!(create_pool(PoolKind::Xyk, (ASSET_ID_2, ASSET_ID_3)));

		assert_ok!(Market::multiswap_asset_buy(
			origin(),
			vec![POOL_ID_1, POOL_ID_2, POOL_ID_3],
			ASSET_ID_3,
			UNIT,
			NATIVE_ASSET_ID,
			2 * UNIT,
		));

		System::assert_last_event(RuntimeEvent::Market(pallet_market::Event::AssetsSwapped {
			who: AccountId::from(ALICE),
			swap_pool_list: vec![6, 7, 8],
			swap_assets_list: vec![(0, 1), (1, 2), (2, 3)],
			amount_in: 1262436229778965569,
			amount_out: UNIT,
		}));

		println!("{:?}", events());
	})
}
