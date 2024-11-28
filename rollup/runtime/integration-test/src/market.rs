use crate::setup::*;

use pallet_market::{AtomicSwap, Event, PoolKind};
use sp_runtime::{traits::Zero, DispatchResult};

const ASSET_ID_1: u32 = NATIVE_ASSET_ID + 1;
const ASSET_ID_2: u32 = ASSET_ID_1 + 1;
const ASSET_ID_3: u32 = ASSET_ID_2 + 1;
const POOL_ID_1: u32 = ASSET_ID_3 + 1;
const POOL_ID_2: u32 = POOL_ID_1 + 1;
const POOL_ID_3: u32 = POOL_ID_2 + 1;

fn test_env() -> TestExternalities {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_2, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_3, 100 * UNIT),
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

fn create_pool_unb(kind: PoolKind, assets: (u32, u32)) -> DispatchResult {
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
			amounts_provided: (10000000000000000000, 10000000000000000000),
			lp_token: POOL_ID_1,
			lp_token_minted: 10000000000000000000,
			total_supply: 10000000000000000000,
		}));

		assert_ok!(create_pool(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

		let p = pallet_stable_swap::Pools::<Runtime>::get(POOL_ID_2).unwrap();
		assert_eq!(p.rate_multipliers[0], UNIT);
		assert_eq!(p.rate_multipliers[1], UNIT);
		System::assert_has_event(RuntimeEvent::Market(Event::PoolCreated {
			creator: AccountId::from(ALICE),
			pool_id: POOL_ID_2,
			lp_token: POOL_ID_2,
			assets: (0, 1),
		}));
		System::assert_has_event(RuntimeEvent::Market(Event::LiquidityMinted {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_2,
			amounts_provided: (10000000000000000000, 10000000000000000000),
			lp_token: POOL_ID_2,
			lp_token_minted: 20000000000000000000,
			total_supply: 20000000000000000000,
		}));

		assert_ok!(create_pool_unb(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

		let p = pallet_stable_swap::Pools::<Runtime>::get(POOL_ID_3).unwrap();
		println!("{:?}", p);
		assert_eq!(p.rate_multipliers[0], 5 * UNIT);
		assert_eq!(p.rate_multipliers[1], 10 * UNIT);

		System::assert_has_event(RuntimeEvent::Market(Event::PoolCreated {
			creator: AccountId::from(ALICE),
			pool_id: POOL_ID_3,
			lp_token: POOL_ID_3,
			assets: (0, 1),
		}));
		// lp_token_minted are correct since we applied the rates, and the pool is balanced
		System::assert_has_event(RuntimeEvent::Market(Event::LiquidityMinted {
			who: AccountId::from(ALICE),
			pool_id: POOL_ID_3,
			amounts_provided: (10000000000000000000, 5000000000000000000),
			lp_token: POOL_ID_3,
			lp_token_minted: 100000000000000000000,
			total_supply: 100000000000000000000,
		}));
	})
}

#[test]
fn add_liquidity_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool_unb(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool_unb(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

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
			total_supply: 114992390693470089383,
		}));
	})
}

#[test]
fn add_liquidity_fixed_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool_unb(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool_unb(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

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
			total_supply: 154925100814226884776,
		}));
	})
}

#[test]
fn remove_liquidity_works() {
	test_env().execute_with(|| {
		assert_ok!(create_pool_unb(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool_unb(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));

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
			amounts: (100000000000000000, 50000000000000000),
			burned_amount: 1000000000000000000,
			total_supply: 99000000000000000000,
		}));
	})
}

#[test]
fn multiswap_should_work_xyk() {
	test_env().execute_with(|| {
		assert_ok!(create_pool_unb(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool_unb(PoolKind::Xyk, (ASSET_ID_1, ASSET_ID_2)));
		assert_ok!(create_pool_unb(PoolKind::Xyk, (ASSET_ID_2, ASSET_ID_3)));

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
		// 2:1 rate
		assert_ok!(create_pool_unb(PoolKind::StableSwap, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool_unb(PoolKind::StableSwap, (ASSET_ID_1, ASSET_ID_2)));
		// 1:1 rate
		assert_ok!(create_pool(PoolKind::StableSwap, (ASSET_ID_2, ASSET_ID_3)));
		// for bnb
		assert_ok!(create_pool_unb(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool_unb(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_2)));
		assert_ok!(create_pool_unb(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_3)));

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
		assert_eq!(after, 99999001734203514767);

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
					amount_out: 498447826003559573,
				},
				AtomicSwap {
					pool_id: POOL_ID_2,
					kind: PoolKind::StableSwap,
					asset_in: 1,
					asset_out: 2,
					amount_in: 498447826003559573,
					amount_out: 248463606016707341,
				},
				AtomicSwap {
					pool_id: POOL_ID_3,
					kind: PoolKind::StableSwap,
					asset_in: 2,
					asset_out: 3,
					amount_in: 248463606016707341,
					amount_out: 247712005295675461,
				},
			],
		}));
	})
}

#[test]
fn multiswap_should_work_mixed() {
	test_env().execute_with(|| {
		assert_ok!(create_pool_unb(PoolKind::Xyk, (NATIVE_ASSET_ID, ASSET_ID_1)));
		assert_ok!(create_pool_unb(PoolKind::StableSwap, (ASSET_ID_1, ASSET_ID_2)));
		assert_ok!(create_pool_unb(PoolKind::Xyk, (ASSET_ID_2, ASSET_ID_3)));

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
					amount_out: 225962336828316482,
				},
				AtomicSwap {
					pool_id: POOL_ID_3,
					kind: PoolKind::Xyk,
					asset_in: 2,
					asset_out: 3,
					amount_in: 225962336828316482,
					amount_out: 110160480582936294,
				},
			],
		}));
	})
}
