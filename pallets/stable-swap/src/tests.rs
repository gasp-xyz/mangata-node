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

		let amounts = vec![5 * mint, mint, 40 * mint];
		let expected = StableSwap::calc_lp_token_amount(&3, amounts.clone(), true).unwrap();

		// imbalanced add, should have fees
		assert_ok!(StableSwap::add_liquidity(
			RuntimeOrigin::signed(account),
			3,
			amounts.clone(),
			1,
		));

		assert_event_emitted!(Event::LiquidityMinted {
			who: 2,
			pool_id: 3,
			amounts_provided: BoundedVec::truncate_from(amounts),
			lp_token: 3,
			lp_token_minted: expected,
			total_supply: 3_000 * UNIT + 45322880144288283584179,
			fees: BoundedVec::truncate_from(vec![
				12498711677479364060,
				21094110011003080268,
				30692298654602638774
			]),
		});

		// half of fees goes to treasury
		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 6249355838739682030);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 10547055005501540134);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 15346149327301319387);
		assert_eq!(StableSwap::get_virtual_price(&3).unwrap(), 1000721478556158249);
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
			amount: 9_984_833_787_927_106_037,
			burned_amount: 10 * UNIT,
			total_supply: total_supply - 10 * UNIT,
		});

		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 7499724481278582);
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
			burned_amount: 35019044399900264325,
			total_supply,
			fees: BoundedVec::truncate_from(vec![
				1875110298930542,
				9374909700834153,
				7500120299077607,
			]),
		});

		// got what requested
		assert_eq!(StableSwap::balance(0, 2), balance - mint + amounts[0]);
		assert_eq!(StableSwap::balance(1, 2), balance - mint + amounts[1]);
		assert_eq!(StableSwap::balance(2, 2), balance - mint + amounts[2]);
		assert_eq!(StableSwap::balance(3, 2), 3 * mint - 35019044399900264325);

		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 937555149465271);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 4687454850417076);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 3750060149538803);
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
fn swap_should_work_dy() {
	new_test_ext().execute_with(|| {
		let (account, _, _) = prep_pool();

		let dy = StableSwap::get_dy(&3, 0, 2, 100 * UNIT).unwrap();

		assert_ok!(StableSwap::swap(RuntimeOrigin::signed(account), 3, 0, 2, 100 * UNIT, 0));

		assert_event_emitted!(Event::AssetsSwapped {
			who: 2,
			pool_id: 3,
			asset_in: 0,
			amount_in: 100 * UNIT,
			asset_out: 2,
			amount_out: dy
		});

		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 150112205918755700);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 0);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 0);
	});
}

#[test]
fn swap_should_work_dx() {
	new_test_ext().execute_with(|| {
		let (account, _, _) = prep_pool();

		let dx = StableSwap::get_dx(&3, 0, 2, 100 * UNIT).unwrap();

		assert_ok!(StableSwap::swap(RuntimeOrigin::signed(account), 3, 0, 2, dx, 0));

		assert_event_emitted!(Event::AssetsSwapped {
			who: 2,
			pool_id: 3,
			asset_in: 0,
			amount_in: dx,
			asset_out: 2,
			amount_out: 100 * UNIT,
		});

		assert_eq!(StableSwap::balance(0, TreasuryAccount::get()), 150641448635003547);
		assert_eq!(StableSwap::balance(1, TreasuryAccount::get()), 0);
		assert_eq!(StableSwap::balance(2, TreasuryAccount::get()), 0);
	});
}
