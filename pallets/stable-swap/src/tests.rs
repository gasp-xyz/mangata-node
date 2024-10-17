use std::vec;

use frame_support::{assert_err, assert_ok};
use sp_runtime::BoundedVec;

use crate::{assert_event_emitted, mock::*, Config, Error, Event};

const UNIT: u128 = 10_u128.pow(18);

fn prep_pool() -> (AccountId, Balance, Balance) {
	let account: AccountId = 2;
	let amount: Balance = 1_000_000 * UNIT;
	let mint: Balance = 1_000 * UNIT;

	StableSwap::create_new_token(&account, amount);
	StableSwap::create_new_token(&account, amount);
	StableSwap::create_new_token(&account, amount);
	StableSwap::create_pool(
		RuntimeOrigin::signed(account),
		vec![0, 1, 2],
		vec![UNIT, UNIT, UNIT],
		200,
	)
	.unwrap();

	assert_ok!(StableSwap::add_liquidity(
		RuntimeOrigin::signed(account),
		3,
		vec![mint, mint, mint],
		0,
	));

	return (account, amount, mint)
}

// create pool tests

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		let account: AccountId = 2;
		let amount: Balance = 1_000_000_000;
		let rate: u128 = 10_u128.pow(18);

		StableSwap::create_new_token(&account, amount);
		StableSwap::create_new_token(&account, amount);
		StableSwap::create_new_token(&account, amount);

		StableSwap::create_pool(
			RuntimeOrigin::signed(account),
			vec![0, 1, 2],
			vec![rate, rate, rate],
			100,
		)
		.unwrap();

		assert!(<Test as Config>::Currency::exists(3));
	});
}

#[test]
fn create_pool_should_fail_on_nonexistent_asset() {
	new_test_ext().execute_with(|| {
		let account: AccountId = 2;
		let amount: Balance = 1_000_000_000;
		let rate: u128 = 10_u128.pow(18);

		StableSwap::create_new_token(&account, amount);
		StableSwap::create_new_token(&account, amount);

		assert_err!(
			StableSwap::create_pool(
				RuntimeOrigin::signed(account),
				vec![0, 1, 2],
				vec![rate, rate, rate],
				100
			),
			Error::<Test>::AssetDoesNotExist,
		);
	});
}

#[test]
fn create_pool_should_fail_on_same_asset() {
	new_test_ext().execute_with(|| {
		let account: AccountId = 2;
		let amount: Balance = 1_000_000_000;
		let rate: u128 = 10_u128.pow(18);

		StableSwap::create_new_token(&account, amount);
		StableSwap::create_new_token(&account, amount);

		assert_err!(
			StableSwap::create_pool(
				RuntimeOrigin::signed(account),
				vec![0, 1, 1],
				vec![rate, rate, rate],
				100
			),
			Error::<Test>::SameAsset,
		);
	});
}

#[test]
fn create_pool_should_fail_on_too_many_assets() {
	new_test_ext().execute_with(|| {
		let account: AccountId = 2;
		let amount: Balance = 1_000_000_000;
		let rate: u128 = 10_u128.pow(18);

		StableSwap::create_new_token(&account, amount);
		StableSwap::create_new_token(&account, amount);

		assert_err!(
			StableSwap::create_pool(
				RuntimeOrigin::signed(account),
				vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
				vec![rate, rate, rate],
				100
			),
			Error::<Test>::TooManyAssets,
		);
	});
}

#[test]
fn create_pool_should_fail_on_coeff_out_out_range() {
	new_test_ext().execute_with(|| {
		let account: AccountId = 2;
		let amount: Balance = 1_000_000_000;
		let rate: u128 = 10_u128.pow(18);

		StableSwap::create_new_token(&account, amount);
		StableSwap::create_new_token(&account, amount);

		assert_err!(
			StableSwap::create_pool(
				RuntimeOrigin::signed(account),
				vec![0, 1],
				vec![rate, rate, rate],
				0
			),
			Error::<Test>::AmpCoeffOutOfRange,
		);
		assert_err!(
			StableSwap::create_pool(
				RuntimeOrigin::signed(account),
				vec![0, 1],
				vec![rate, rate, rate],
				u128::MAX
			),
			Error::<Test>::AmpCoeffOutOfRange,
		);
	});
}

// add liquidity tests
#[test]
fn add_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let (account, _, mint) = prep_pool();

		// check first balanced add
		assert_event_emitted!(Event::LiquidityMinted {
			who: 2,
			pool_id: 3,
			amounts_provided: BoundedVec::truncate_from(vec![mint, mint, mint]),
			lp_token: 3,
			lp_token_minted: 3_000 * UNIT,
			total_supply: 3_000 * UNIT,
			fees: BoundedVec::truncate_from(vec![]),
		});

		assert_eq!(StableSwap::get_virtual_price(&3).unwrap(), 1 * UNIT);

		// imbalanced add, should have fees
		assert_ok!(StableSwap::add_liquidity(
			RuntimeOrigin::signed(account),
			3,
			vec![5 * mint, mint, 40 * mint],
			1,
		));

		assert_event_emitted!(Event::LiquidityMinted {
			who: 2,
			pool_id: 3,
			amounts_provided: BoundedVec::truncate_from(vec![5 * mint, mint, 40 * mint]),
			lp_token: 3,
			lp_token_minted: 32_257_188_974_406_171_723_750,
			total_supply: 3_000 * UNIT + 32_257_188_974_406_171_723_750,
			fees: BoundedVec::truncate_from(vec![
				6_789_799_786_180_899_753,
				13_614_384_181_261_910_763,
				38_328_883_124_617_667_627
			]),
		});

		// half of fees goes to treasury
		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 3_394_899_893_090_449_876);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 6_807_192_090_630_955_381);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 19_164_441_562_308_833_813);
		assert_eq!(StableSwap::get_virtual_price(&3).unwrap(), 1_001_083_007_429_485_699);
	});
}

// remove liquidity
#[test]
fn remove_liquidity_one_asset_should_work() {
	new_test_ext().execute_with(|| {
		let (account, _, _) = prep_pool();
		let total_supply = StableSwap::total_supply(3);

		assert_ok!(StableSwap::remove_liquidity_one_asset(
			RuntimeOrigin::signed(account),
			3,
			0,
			10 * UNIT,
			0
		));

		assert_event_emitted!(Event::LiquidityBurnedOne {
			who: 2,
			pool_id: 3,
			asset_id: 0,
			// a bit less then 10, minus fees
			amount: 9_973_889_203_590_967_782,
			burned_amount: 10 * UNIT,
			total_supply: total_supply - 10 * UNIT,
		});

		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 7_481_199_984_725_130);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 0);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 0);
	});
}

#[test]
fn remove_liquidity_imbalanced_should_work() {
	new_test_ext().execute_with(|| {
		let (account, balance, mint) = prep_pool();
		let amounts = vec![mint / 100, mint / 50, mint / 200];

		assert_eq!(StableSwap::balance(0, 2), balance - mint);
		assert_eq!(StableSwap::balance(1, 2), balance - mint);
		assert_eq!(StableSwap::balance(2, 2), balance - mint);

		assert_ok!(StableSwap::remove_liquidity_imbalanced(
			RuntimeOrigin::signed(account),
			3,
			amounts.clone(),
			u128::MAX,
		));

		let total_supply = StableSwap::total_supply(3);
		assert_event_emitted!(Event::LiquidityBurned {
			who: 2,
			pool_id: 3,
			amounts: BoundedVec::truncate_from(amounts.clone()),
			burned_amount: 35_038_472_342_948_464_202,
			total_supply,
			fees: BoundedVec::truncate_from(vec![
				1_882_389_867_551_015,
				9_367_630_116_683_933,
				7_507_399_877_404_172,
			]),
		});

		// got what requested
		assert_eq!(StableSwap::balance(0, 2), balance - mint + amounts[0]);
		assert_eq!(StableSwap::balance(1, 2), balance - mint + amounts[1]);
		assert_eq!(StableSwap::balance(2, 2), balance - mint + amounts[2]);
		assert_eq!(StableSwap::balance(3, 2), 3 * mint - 35_038_472_342_948_464_202);

		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 941_194_933_775_507);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 4_683_815_058_341_966);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 3_753_699_938_702_086);
	});
}

#[test]
fn remove_liquidity_to_zero_should_work() {
	new_test_ext().execute_with(|| {
		let (account, balance, mint) = prep_pool();
		let total_supply = StableSwap::total_supply(3);

		assert_ok!(StableSwap::remove_liquidity(
			RuntimeOrigin::signed(account),
			3,
			total_supply,
			vec![mint, mint, mint],
		));

		assert_event_emitted!(Event::LiquidityBurned {
			who: 2,
			pool_id: 3,
			amounts: BoundedVec::truncate_from(vec![mint, mint, mint]),
			burned_amount: 3 * mint,
			total_supply: 0,
			fees: BoundedVec::truncate_from(vec![]),
		});

		// all back
		assert_eq!(StableSwap::balance(0, 2), balance);
		assert_eq!(StableSwap::balance(1, 2), balance);
		assert_eq!(StableSwap::balance(2, 2), balance);
		assert_eq!(StableSwap::balance(3, 2), 0);

		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 0);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 0);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 0);
	});
}

// swaps
#[test]
fn swap_should_work() {
	new_test_ext().execute_with(|| {
		let (account, _, _) = prep_pool();

		assert_ok!(StableSwap::swap(RuntimeOrigin::signed(account), 3, 0, 2, 100 * UNIT, 0));

		assert_event_emitted!(Event::AssetsSwapped {
			who: 2,
			pool_id: 3,
			asset_in: 0,
			amount_in: 100 * UNIT,
			asset_out: 2,
			amount_out: 96_760_741_606_773_957_079
		});

		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 145_316_636_395_435_623);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 0);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 0);
	});
}
