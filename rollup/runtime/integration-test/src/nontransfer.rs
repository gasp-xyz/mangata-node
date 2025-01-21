use rollup_runtime::tokens::NontransferableTokens;

use crate::setup::*;

const ASSET_ID_1: TokenId = NATIVE_ASSET_ID + 1;
const POOL_ID: TokenId = ASSET_ID_1 + 1;
const ARB: [u8; 20] = hex_literal::hex!["fc741134c82b81b7ab7efbf334b0c90ff8dbf22c"];

fn test_env() -> TestExternalities {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ARB), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
		],
		..ExtBuilder::default()
	}
	.build()
}

fn root() -> RuntimeOrigin {
	RuntimeOrigin::root().into()
}

fn origin() -> RuntimeOrigin {
	RuntimeOrigin::signed(AccountId::from(ALICE))
}

#[test]
fn test_tokens_calls_should_block() {
	let mut ext = test_env();
	ext.execute_with(|| {
		let who = AccountId::from(ALICE);
		let bob = AccountId::from(BOB);
		let amount = UNIT;
		assert_err!(
			orml_tokens::Pallet::<Runtime>::mint(root(), NATIVE_ASSET_ID, who, amount),
			orml_tokens::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			orml_tokens::Pallet::<Runtime>::set_balance(
				root(),
				who,
				NATIVE_ASSET_ID,
				amount,
				amount
			),
			orml_tokens::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			orml_tokens::Pallet::<Runtime>::transfer(origin(), bob, NATIVE_ASSET_ID, amount),
			orml_tokens::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			orml_tokens::Pallet::<Runtime>::force_transfer(
				root(),
				who,
				bob,
				NATIVE_ASSET_ID,
				amount
			),
			orml_tokens::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			orml_tokens::Pallet::<Runtime>::transfer_all(origin(), bob, NATIVE_ASSET_ID, false),
			orml_tokens::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			orml_tokens::Pallet::<Runtime>::transfer_keep_alive(
				origin(),
				bob,
				NATIVE_ASSET_ID,
				amount
			),
			orml_tokens::Error::<Runtime>::NontransferableToken
		);
	});
}

#[test]
fn test_tokens_calls_should_work_for_allow_listed() {
	let mut ext = test_env();
	ext.execute_with(|| {
		let who = AccountId::from(ALICE);
		let bob = AccountId::from(BOB);
		let amount = UNIT;

		assert_ok!(TransferMembers::add_member(root(), who));

		assert_err!(
			orml_tokens::Pallet::<Runtime>::mint(root(), NATIVE_ASSET_ID, who, amount),
			orml_tokens::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			orml_tokens::Pallet::<Runtime>::set_balance(
				root(),
				who,
				NATIVE_ASSET_ID,
				amount,
				amount
			),
			orml_tokens::Error::<Runtime>::NontransferableToken
		);
		assert_ok!(orml_tokens::Pallet::<Runtime>::transfer(
			origin(),
			bob,
			NATIVE_ASSET_ID,
			amount
		));
		assert_ok!(orml_tokens::Pallet::<Runtime>::force_transfer(
			root(),
			who,
			bob,
			NATIVE_ASSET_ID,
			amount
		));
		assert_ok!(orml_tokens::Pallet::<Runtime>::transfer_keep_alive(
			origin(),
			bob,
			NATIVE_ASSET_ID,
			amount
		));
		assert_ok!(orml_tokens::Pallet::<Runtime>::transfer_all(
			origin(),
			bob,
			NATIVE_ASSET_ID,
			false
		));
	});
}

#[test]
fn test_market_should_block() {
	let mut ext = test_env();
	ext.execute_with(|| {
		let foundation = AccountId::from(ALICE);
		let bob = AccountId::from(BOB);
		let amount = 100 * UNIT;

		// user cannot create pool, foundation can
		assert_err!(
			pallet_market::Pallet::<Runtime>::create_pool(
				RuntimeOrigin::signed(bob),
				pallet_market::PoolKind::Xyk,
				NATIVE_ASSET_ID,
				amount,
				ASSET_ID_1,
				amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_ok!(pallet_market::Pallet::<Runtime>::create_pool(
			RuntimeOrigin::signed(foundation),
			pallet_market::PoolKind::Xyk,
			NATIVE_ASSET_ID,
			amount,
			ASSET_ID_1,
			amount,
		));

		// none can mint
		assert_err!(
			pallet_market::Pallet::<Runtime>::mint_liquidity(
				RuntimeOrigin::signed(foundation),
				POOL_ID,
				NATIVE_ASSET_ID,
				amount,
				amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			pallet_market::Pallet::<Runtime>::mint_liquidity_fixed_amounts(
				RuntimeOrigin::signed(foundation),
				POOL_ID,
				(amount, amount),
				amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			pallet_market::Pallet::<Runtime>::mint_liquidity_using_vesting_native_tokens(
				RuntimeOrigin::signed(foundation),
				POOL_ID,
				amount,
				amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			pallet_market::Pallet::<Runtime>::mint_liquidity_using_vesting_native_tokens_by_vesting_index(
				RuntimeOrigin::signed(foundation),
				POOL_ID,
				0,
                None,
                amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);

		// user cannot burn, foundation can
		assert_err!(
			pallet_market::Pallet::<Runtime>::burn_liquidity(
				RuntimeOrigin::signed(bob),
				POOL_ID,
				amount,
				amount,
				amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_ok!(pallet_market::Pallet::<Runtime>::burn_liquidity(
			RuntimeOrigin::signed(foundation),
			POOL_ID,
			UNIT,
			0,
			0,
		));

		// user & foundation cannot sell, ARB bot can
		assert_err!(
			pallet_market::Pallet::<Runtime>::multiswap_asset(
				RuntimeOrigin::signed(bob),
				vec![POOL_ID],
				NATIVE_ASSET_ID,
				UNIT,
				ASSET_ID_1,
				0,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			pallet_market::Pallet::<Runtime>::multiswap_asset(
				RuntimeOrigin::signed(foundation),
				vec![POOL_ID],
				NATIVE_ASSET_ID,
				UNIT,
				ASSET_ID_1,
				0,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			pallet_market::Pallet::<Runtime>::multiswap_asset(
				RuntimeOrigin::signed(AccountId::from(ARB)),
				vec![POOL_ID],
				NATIVE_ASSET_ID,
				UNIT,
				ASSET_ID_1,
				0,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			pallet_market::Pallet::<Runtime>::multiswap_asset_buy(
				RuntimeOrigin::signed(bob),
				vec![POOL_ID],
				ASSET_ID_1,
				UNIT,
				NATIVE_ASSET_ID,
				amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			pallet_market::Pallet::<Runtime>::multiswap_asset_buy(
				RuntimeOrigin::signed(foundation),
				vec![POOL_ID],
				ASSET_ID_1,
				UNIT,
				NATIVE_ASSET_ID,
				amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
		assert_err!(
			pallet_market::Pallet::<Runtime>::multiswap_asset_buy(
				RuntimeOrigin::signed(AccountId::from(ARB)),
				vec![POOL_ID],
				ASSET_ID_1,
				UNIT,
				NATIVE_ASSET_ID,
				amount,
			),
			pallet_market::Error::<Runtime>::NontransferableToken
		);
	});
}
