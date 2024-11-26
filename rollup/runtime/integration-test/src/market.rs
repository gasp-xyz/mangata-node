use crate::setup::*;

use pallet_market::{AtomicSwap, Event, PoolKind};
use sp_runtime::{traits::Zero, DispatchResult};

const ASSET_ID_1: u32 = NATIVE_ASSET_ID + 1;
const ASSET_ID_2: u32 = ASSET_ID_1 + 1;
const ASSET_ID_3: u32 = ASSET_ID_2 + 1;
const ASSET_ID_4_DISABLED: u32 = ASSET_ID_3 + 1;
const ASSET_ID_5: u32 = ASSET_ID_4_DISABLED + 1;
const ASSET_ID_6: u32 = ASSET_ID_5 + 1;
const POOL_ID_1: u32 = ASSET_ID_6 + 1;
const POOL_ID_2: u32 = POOL_ID_1 + 1;
const POOL_ID_3: u32 = POOL_ID_2 + 1;

fn test_env() -> TestExternalities {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_2, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_3, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_4_DISABLED, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_5, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_6, 100 * UNIT),
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
			(
				ASSET_ID_5,
				AssetMetadataOf {
					decimals: 10,
					name: BoundedVec::truncate_from(b"Asset".to_vec()),
					symbol: BoundedVec::truncate_from(b"Asset".to_vec()),
					existential_deposit: Default::default(),
					additional: Default::default(),
				},
			),
			(
				ASSET_ID_6,
				AssetMetadataOf {
					decimals: 12,
					name: BoundedVec::truncate_from(b"Asset".to_vec()),
					symbol: BoundedVec::truncate_from(b"Asset".to_vec()),
					existential_deposit: Default::default(),
					additional: Default::default(),
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
	Market::create_pool(origin(), kind, assets.0, 10 * UNIT, assets.1, 5 * UNIT)
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
		System::assert_has_event(RuntimeEvent::Market(Event::PoolCreated {
			creator: AccountId::from(ALICE),
			pool_id: POOL_ID_1,
			lp_token: POOL_ID_1,
			assets: (0, 1),
		}));
		System::assert_has_event(RuntimeEvent::Market(Event::LiquidityMinted {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_1,
			amounts_provided: (10000000000000000000, 5000000000000000000),
			lp_token: POOL_ID_1,
			lp_token_minted: 7500000000000000000,
			total_supply: 7500000000000000000,
		}));

		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));
		System::assert_has_event(RuntimeEvent::Market(Event::PoolCreated {
			creator: AccountId::from(ALICE),
			pool_id: POOL_ID_2,
			lp_token: POOL_ID_2,
			assets: (0, 1),
		}));
		System::assert_has_event(RuntimeEvent::Market(Event::LiquidityMinted {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_2,
			amounts_provided: (10000000000000000000, 5000000000000000000),
			lp_token: POOL_ID_2,
			lp_token_minted: 14999063611862273044,
			total_supply: 14999063611862273044,
		}));
	})
}

#[test]
fn add_liquidity_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

		let expected =
			Market::calculate_expected_amount_for_minting(POOL_ID_1, NATIVE_ASSET_ID, UNIT)
				.unwrap();
		let lp_expected =
			Market::calculate_expected_lp_minted(POOL_ID_1, (UNIT, expected)).unwrap();
		assert_ok!(Market::mint_liquidity(origin(), POOL_ID_1, NATIVE_ASSET_ID, UNIT, 10 * UNIT));
		System::assert_last_event(RuntimeEvent::Market(Event::LiquidityMinted {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_1,
			amounts_provided: (1000000000000000000, expected),
			lp_token: POOL_ID_1,
			lp_token_minted: lp_expected,
			total_supply: 8250000000000000000,
		}));

		let expected =
			Market::calculate_expected_amount_for_minting(POOL_ID_2, NATIVE_ASSET_ID, UNIT)
				.unwrap();
		let lp_expected =
			Market::calculate_expected_lp_minted(POOL_ID_2, (UNIT, expected)).unwrap();
		assert_ok!(Market::mint_liquidity(origin(), POOL_ID_2, NATIVE_ASSET_ID, UNIT, 10 * UNIT));
		System::assert_last_event(RuntimeEvent::Market(Event::LiquidityMinted {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_2,
			amounts_provided: (1000000000000000000, expected),
			lp_token: POOL_ID_2,
			lp_token_minted: lp_expected,
			total_supply: 16998182477145509576,
		}));
	})
}

#[test]
fn add_liquidity_fixed_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

		assert_ok!(Market::mint_liquidity_fixed_amounts(origin(), POOL_ID_1, (UNIT, 0), 0));
		System::assert_last_event(RuntimeEvent::Market(Event::LiquidityMinted {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_1,
			amounts_provided: (1000000000000000000, 0),
			lp_token: POOL_ID_1,
			lp_token_minted: 365524961509654622,
			total_supply: 7865524961509654622,
		}));

		let expected = Market::calculate_expected_lp_minted(POOL_ID_2, (UNIT, 5 * UNIT)).unwrap();
		assert_ok!(Market::mint_liquidity_fixed_amounts(origin(), POOL_ID_2, (UNIT, 5 * UNIT), 0));
		System::assert_last_event(RuntimeEvent::Market(Event::LiquidityMinted {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_2,
			amounts_provided: (1000000000000000000, 5000000000000000000),
			lp_token: POOL_ID_2,
			lp_token_minted: expected,
			total_supply: 20990943480975169792,
		}));
	})
}

#[test]
fn remove_liquidity_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

		assert_ok!(Market::burn_liquidity(origin(), POOL_ID_1, UNIT, 0, 0));
		System::assert_last_event(RuntimeEvent::Market(Event::LiquidityBurned {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_1,
			amounts: (1333333333333333333, 666666666666666666),
			burned_amount: 1000000000000000000,
			total_supply: 6500000000000000000,
		}));

		assert_ok!(Market::burn_liquidity(origin(), POOL_ID_2, UNIT, 0, 0));
		System::assert_last_event(RuntimeEvent::Market(Event::LiquidityBurned {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_2,
			amounts: (666708286515387818, 333354143257693909),
			burned_amount: 1000000000000000000,
			total_supply: 13999063611862273044,
		}));
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

		println!("{:#?}", events());

		System::assert_last_event(RuntimeEvent::Market(Event::AssetsSwapped {
			who: AccountId::from(ALICE),
			swaps: vec![
				AtomicSwap {
					pool_id: POOL_ID_1,
					kind: PoolKind::Xyk,
					asset_in: 0,
					asset_out: 1,
					amount_in: 1000000000000000000,
					amount_out: 453305446940074565,
				},
				AtomicSwap {
					pool_id: POOL_ID_2,
					kind: PoolKind::Xyk,
					asset_in: 1,
					asset_out: 2,
					amount_in: 453305446940074565,
					amount_out: 216201629292906575,
				},
				AtomicSwap {
					pool_id: POOL_ID_3,
					kind: PoolKind::Xyk,
					asset_in: 2,
					asset_out: 3,
					amount_in: 216201629292906575,
					amount_out: 105502376567411557,
				},
			],
		}));
	})
}

#[test]
fn multiswap_should_work_stable_swap_with_bnb() {
	test_env().execute_with(|| {
		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::StableSwap, (ASSET_ID_1, ASSET_ID_2)));
		assert_ok!(create_pool(PoolKind::StableSwap, (ASSET_ID_2, ASSET_ID_3)));
		// for bnb
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_2)));
		assert_ok!(create_pool(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_3)));

		let before = Tokens::total_issuance(NATIVE_ASSET_ID);

		assert_ok!(Market::multiswap_asset(
			origin(),
			vec![POOL_ID_1, POOL_ID_2, POOL_ID_3],
			NATIVE_ASSET_ID,
			UNIT,
			ASSET_ID_3,
			Zero::zero(),
		));

		let after = Tokens::total_issuance(NATIVE_ASSET_ID);
		// issuance decreased because of bnb
		assert!(before > after);
		assert_eq!(before, 100000000000000000000);
		assert_eq!(after, 99996758378067624442);

		println!("{:#?}", events());

		System::assert_last_event(RuntimeEvent::Market(Event::AssetsSwapped {
			who: AccountId::from(ALICE),
			swaps: vec![
				AtomicSwap {
					pool_id: POOL_ID_1,
					kind: PoolKind::StableSwap,
					asset_in: 0,
					asset_out: 1,
					amount_in: 1000000000000000000,
					amount_out: 995595345298031754,
				},
				AtomicSwap {
					pool_id: POOL_ID_2,
					kind: PoolKind::StableSwap,
					asset_in: 1,
					asset_out: 2,
					amount_in: 995595345298031754,
					amount_out: 991212132384121611,
				},
				AtomicSwap {
					pool_id: POOL_ID_3,
					kind: PoolKind::StableSwap,
					asset_in: 2,
					asset_out: 3,
					amount_in: 991212132384121611,
					amount_out: 986850235267668399,
				},
			],
		}));
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

		println!("{:#?}", events());

		System::assert_last_event(RuntimeEvent::Market(Event::AssetsSwapped {
			who: AccountId::from(ALICE),
			swaps: vec![
				AtomicSwap {
					pool_id: POOL_ID_1,
					kind: PoolKind::Xyk,
					asset_in: 0,
					asset_out: 1,
					amount_in: 1000000000000000000,
					amount_out: 453305446940074565,
				},
				AtomicSwap {
					pool_id: POOL_ID_2,
					kind: PoolKind::StableSwap,
					asset_in: 1,
					asset_out: 2,
					amount_in: 453305446940074565,
					amount_out: 451412806019623895,
				},
				AtomicSwap {
					pool_id: POOL_ID_3,
					kind: PoolKind::Xyk,
					asset_in: 2,
					asset_out: 3,
					amount_in: 451412806019623895,
					amount_out: 215337820687860400,
				},
			],
		}));
	})
}

#[test]
fn test_diff_decimals_work() {
	test_env().execute_with(|| {
		let unit10 = 10_000_000_000_u128;
		let unit12 = 1_000_000_000_000_u128;
		assert_ok!(Market::create_pool(
			origin(),
			PoolKind::StableSwap,
			ASSET_ID_5,
			100 * unit10,
			ASSET_ID_6,
			100 * unit12
		));

		let pool = Market::get_pools(Some(POOL_ID_1));
		let price = Market::calculate_sell_price(POOL_ID_1, ASSET_ID_5, 1).unwrap();

		println!("{:#?}", pool);
		println!("{:#?}", price);

		assert_ok!(Market::multiswap_asset(
			origin(),
			vec![POOL_ID_1],
			ASSET_ID_5,
			1,
			ASSET_ID_6,
			1,
		));

		println!("{:#?}", events());

		System::assert_last_event(RuntimeEvent::Market(Event::AssetsSwapped {
			who: AccountId::from(ALICE),
			swaps: vec![AtomicSwap {
				pool_id: POOL_ID_1,
				kind: PoolKind::StableSwap,
				asset_in: ASSET_ID_5,
				asset_out: ASSET_ID_6,
				amount_in: 1,
				amount_out: 99,
			}],
		}));
	})
}
