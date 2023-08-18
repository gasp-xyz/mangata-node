// Copyright (C) 2020 Mangata team
#![cfg(not(feature = "runtime-benchmarks"))]
#![allow(non_snake_case)]

use super::{Event, *};
use crate::mock::*;
use frame_support::{assert_err, assert_err_ignore_postinfo, dispatch::GetDispatchInfo};
use mangata_support::traits::{ComputeIssuance, GetIssuance, LiquidityMiningApi};
use mangata_types::assets::CustomMetadata;
use orml_traits::asset_registry::AssetMetadata;
use serial_test::serial;
use sp_runtime::{traits::Dispatchable, Permill};
use test_case::test_case;

type TokensOf<T> = <T as Config>::Currency;

const DUMMY_USER_ID: u128 = 2;
const TRADER_ID: u128 = 3;

fn initialize() {
	// creating asset with assetId 0 and minting to accountId 2
	System::set_block_number(1);
	let acc_id: u128 = 2;
	let amount: u128 = 1000000000000000000000;
	// creates token with ID = 0;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 1;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 2;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 3;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 4;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);

	XykStorage::create_pool(
		RuntimeOrigin::signed(DUMMY_USER_ID),
		1,
		40000000000000000000,
		4,
		60000000000000000000,
	)
	.unwrap();

	let pool_created_event = crate::mock::RuntimeEvent::XykStorage(
		crate::Event::<Test>::PoolCreated(acc_id, 1, 40000000000000000000, 4, 60000000000000000000),
	);

	assert!(System::events().iter().any(|record| record.event == pool_created_event));
}

fn multi_initialize() {
	// creating asset with assetId 0 and minting to accountId 2
	System::set_block_number(1);
	let _acc_id: u128 = 2;
	let amount: u128 = 1000000000000000000000;
	// creates token with ID = 0;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 1;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 2;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 3;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 4;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
	// creates token with ID = 5;
	XykStorage::create_new_token(&DUMMY_USER_ID, amount);

	// creates token with ID = 0;
	XykStorage::mint_token(0, &TRADER_ID, amount);
	// creates token with ID = 1;
	XykStorage::mint_token(1, &TRADER_ID, amount);
	// creates token with ID = 2;
	XykStorage::mint_token(2, &TRADER_ID, amount);
	// creates token with ID = 3;
	XykStorage::mint_token(3, &TRADER_ID, amount);
	// creates token with ID = 4;
	XykStorage::mint_token(4, &TRADER_ID, amount);
	// creates token with ID = 5;
	XykStorage::mint_token(5, &TRADER_ID, amount);

	XykStorage::create_pool(
		RuntimeOrigin::signed(DUMMY_USER_ID),
		1,
		40000000000000000000,
		2,
		60000000000000000000,
	)
	.unwrap();

	XykStorage::create_pool(
		RuntimeOrigin::signed(DUMMY_USER_ID),
		2,
		40000000000000000000,
		3,
		60000000000000000000,
	)
	.unwrap();

	XykStorage::create_pool(
		RuntimeOrigin::signed(DUMMY_USER_ID),
		3,
		40000000000000000000,
		4,
		60000000000000000000,
	)
	.unwrap();

	XykStorage::create_pool(
		RuntimeOrigin::signed(DUMMY_USER_ID),
		4,
		40000000000000000000,
		5,
		60000000000000000000,
	)
	.unwrap();
}

fn initialize_buy_and_burn() {
	// creating asset with assetId 0 and minting to accountId 2
	let acc_id: u128 = 2;
	let amount: u128 = 1000000000000000;

	XykStorage::create_new_token(&acc_id, amount);
	XykStorage::create_new_token(&acc_id, amount);
	XykStorage::create_new_token(&acc_id, amount); // token id 2 is dummy
	XykStorage::create_new_token(&acc_id, amount); // token id 3 is LT for mga-dummy
	XykStorage::create_new_token(&acc_id, amount);
	XykStorage::create_new_token(&acc_id, amount);
	XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 100000000000000, 1, 100000000000000)
		.unwrap();
	XykStorage::create_pool(RuntimeOrigin::signed(2), 1, 100000000000000, 4, 100000000000000)
		.unwrap();
}

#[test]
#[serial]
fn set_info_should_work() {
	new_test_ext().execute_with(|| {
		// creating asset with assetId 0 and minting to accountId 2
		let acc_id: u128 = 2;
		let amount: u128 = 1000000000000000000000;
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);

		XykStorage::create_pool(
			RuntimeOrigin::signed(2),
			0,
			40000000000000000000,
			1,
			60000000000000000000,
		)
		.unwrap();

		assert_eq!(
			*MockAssetRegister::instance().lock().unwrap().get(&2u32).unwrap(),
			AssetMetadata {
				name: b"LiquidityPoolToken0x00000002".to_vec(),
				symbol: b"TKN0x00000000-TKN0x00000001".to_vec(),
				decimals: 18u32,
				location: None,
				additional: CustomMetadata::default(),
				existential_deposit: 0u128,
			}
		);
	});
}

#[test]
#[serial]
fn set_info_should_work_with_small_numbers() {
	new_test_ext().execute_with(|| {
		// creating asset with assetId 0 and minting to accountId 2
		let acc_id: u128 = 2;
		let amount: u128 = 1000000000000000000000;
		const N: u32 = 12345u32;

		for _ in 0..N {
			XykStorage::create_new_token(&acc_id, amount);
		}

		XykStorage::create_pool(
			RuntimeOrigin::signed(2),
			15,
			40000000000000000000,
			12233,
			60000000000000000000,
		)
		.unwrap();

		assert_eq!(
			*MockAssetRegister::instance().lock().unwrap().get(&N).unwrap(),
			AssetMetadata {
				name: b"LiquidityPoolToken0x00003039".to_vec(),
				symbol: b"TKN0x0000000F-TKN0x00002FC9".to_vec(),
				decimals: 18u32,
				location: None,
				additional: CustomMetadata::default(),
				existential_deposit: 0u128,
			}
		);
	});
}

#[test]
#[serial]
#[ignore]
fn set_info_should_work_with_large_numbers() {
	new_test_ext().execute_with(|| {
		// creating asset with assetId 0 and minting to accountId 2
		let acc_id: u128 = 2;
		let amount: u128 = 1000000000000000000000;
		const N: u32 = 1524501234u32;

		for _ in 0..N {
			XykStorage::create_new_token(&acc_id, amount);
		}

		XykStorage::create_pool(
			RuntimeOrigin::signed(2),
			15000000,
			40000000000000000000,
			12233000,
			60000000000000000000,
		)
		.unwrap();

		assert_eq!(
			*MockAssetRegister::instance().lock().unwrap().get(&1524501234u32).unwrap(),
			AssetMetadata {
				name: b"LiquidityPoolToken0x5ADE0AF2".to_vec(),
				symbol: b"TKN0x00E4E1C0-TKN0x00BAA928".to_vec(),
				decimals: 18u32,
				location: None,
				additional: CustomMetadata::default(),
				existential_deposit: 0u128,
			}
		);
	});
}

#[test]
#[serial]
fn buy_and_burn_sell_mangata() {
	new_test_ext().execute_with(|| {
		initialize_buy_and_burn();

		XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![0, 1], 50000000000000, 0)
			.unwrap();

		assert_eq!(XykStorage::asset_pool((0, 1)), (149949999999998, 66733400066734));
		assert_eq!(XykStorage::balance(0, 2), 850000000000000);
		assert_eq!(XykStorage::balance(1, 2), 833266599933266);
		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 149949999999998);
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 166733400066734);
		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 25000000001);
		assert_eq!(XykStorage::balance(1, XykStorage::treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(1, XykStorage::bnb_treasury_account_id()), 0);
	});
}

#[test]
#[serial]
fn buy_and_burn_sell_has_mangata_pair() {
	new_test_ext().execute_with(|| {
		initialize_buy_and_burn();

		XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![1, 4], 50000000000000, 0)
			.unwrap();

		assert_eq!(XykStorage::asset_pool((0, 1)), (99950024987505, 100050000000002));
		assert_eq!(XykStorage::asset_pool((1, 4)), (149949999999998, 66733400066734));
		assert_eq!(XykStorage::balance(1, 2), 750000000000000); // user acc: regular trade result
		assert_eq!(XykStorage::balance(4, 2), 933266599933266); // user acc: regular trade result
		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 99950024987505);
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 250000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 66733400066734); // vault: regular trade result
		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 24987506247); // 24987506247 mangata in treasury
		assert_eq!(XykStorage::balance(1, XykStorage::treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(1, XykStorage::bnb_treasury_account_id()), 0);
	});
}

#[test]
#[serial]
fn buy_and_burn_sell_none_have_mangata_pair() {
	new_test_ext().execute_with(|| {
		initialize_buy_and_burn();

		XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![4, 1], 50000000000000, 0)
			.unwrap();

		assert_eq!(XykStorage::asset_pool((0, 1)), (100000000000000, 100000000000000));
		assert_eq!(XykStorage::asset_pool((1, 4)), (66733400066734, 149949999999998));
		assert_eq!(XykStorage::balance(1, 2), 833266599933266); // user acc: regular trade result
		assert_eq!(XykStorage::balance(4, 2), 850000000000000); // user acc: regular trade result
		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 100000000000000);
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 166733400066734);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 149949999999998); // vault: regular trade result
		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 0); // 24987506247 mangata in treasury
		assert_eq!(XykStorage::balance(4, XykStorage::treasury_account_id()), 25000000001);
		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(4, XykStorage::bnb_treasury_account_id()), 25000000001);
	});
}

#[test]
#[serial]
fn buy_and_burn_buy_where_sold_is_mangata() {
	new_test_ext().execute_with(|| {
		initialize_buy_and_burn();

		XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(2),
			vec![0, 1],
			33266599933266,
			50000000000001,
		)
		.unwrap();

		assert_eq!(XykStorage::asset_pool((0, 1)), (149949999999999, 66733400066734));
		assert_eq!(XykStorage::balance(0, 2), 850000000000001);
		assert_eq!(XykStorage::balance(1, 2), 833266599933266);

		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 149949999999999);
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 166733400066734);
		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 25000000000);
		assert_eq!(XykStorage::balance(1, XykStorage::treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(1, XykStorage::bnb_treasury_account_id()), 0);
	});
}

#[test]
#[serial]
fn buy_and_burn_buy_where_sold_has_mangata_pair() {
	new_test_ext().execute_with(|| {
		initialize_buy_and_burn();

		XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(2),
			vec![1, 4],
			33266599933266,
			50000000000001,
		)
		.unwrap();

		assert_eq!(XykStorage::asset_pool((0, 1)), (99950024987507, 100050000000000));
		assert_eq!(XykStorage::asset_pool((1, 4)), (149949999999999, 66733400066734));
		assert_eq!(XykStorage::balance(1, 2), 750000000000001); // user acc: regular trade result
		assert_eq!(XykStorage::balance(4, 2), 933266599933266); // user acc: regular trade result
		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 99950024987507);
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 249999999999999);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 66733400066734); // vault: regular trade result
		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 24987506246); // 24987506247 mangata in treasury
		assert_eq!(XykStorage::balance(1, XykStorage::treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(1, XykStorage::bnb_treasury_account_id()), 0);
	});
}

#[test]
#[serial]
fn buy_and_burn_buy_none_have_mangata_pair() {
	new_test_ext().execute_with(|| {
		initialize_buy_and_burn();

		XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(2),
			vec![4, 1],
			33266599933266,
			50000000000001,
		)
		.unwrap();

		assert_eq!(XykStorage::asset_pool((0, 1)), (100000000000000, 100000000000000));
		assert_eq!(XykStorage::asset_pool((1, 4)), (66733400066734, 149949999999999));
		assert_eq!(XykStorage::balance(1, 2), 833266599933266); // user acc: regular trade result
		assert_eq!(XykStorage::balance(4, 2), 850000000000001); // user acc: regular trade result
		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 100000000000000);
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 166733400066734);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 149949999999999); // vault: regular trade result
		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 0); // 24987506247 mangata in treasury
		assert_eq!(XykStorage::balance(4, XykStorage::treasury_account_id()), 25000000000);
		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
		assert_eq!(XykStorage::balance(4, XykStorage::bnb_treasury_account_id()), 25000000000);
	});
}

#[test]
#[serial]
fn multi() {
	new_test_ext().execute_with(|| {
		let acc_id: u128 = 2;
		let amount: u128 = 2000000000000000000000000;

		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_pool(
			RuntimeOrigin::signed(2),
			1,
			1000000000000000000000000,
			4,
			500000000000000000000000,
		)
		.unwrap();
		assert_eq!(
			XykStorage::asset_pool((1, 4)),
			(1000000000000000000000000, 500000000000000000000000)
		);
		assert_eq!(XykStorage::liquidity_asset((1, 4)), Some(5)); // liquidity assetId corresponding to newly created pool
		assert_eq!(XykStorage::liquidity_pool(5), Some((1, 4))); // liquidity assetId corresponding to newly created pool
		assert_eq!(XykStorage::total_supply(5), 750000000000000000000000); // total liquidity assets
		assert_eq!(XykStorage::balance(5, 2), 750000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
		assert_eq!(XykStorage::balance(1, 2), 1000000000000000000000000); // amount of asset 1 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(4, 2), 1500000000000000000000000); // amount of asset 2 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 1000000000000000000000000); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 500000000000000000000000); // amount of asset 1 in vault acc after creating pool

		XykStorage::mint_liquidity(
			RuntimeOrigin::signed(2),
			1,
			4,
			500000000000000000000000,
			5000000000000000000000000,
		)
		.unwrap();

		assert_eq!(
			XykStorage::asset_pool((1, 4)),
			(1500000000000000000000000, 750000000000000000000001)
		);
		assert_eq!(XykStorage::total_supply(5), 1125000000000000000000000); // total liquidity assets
		assert_eq!(XykStorage::balance(5, 2), 1125000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
		assert_eq!(XykStorage::balance(1, 2), 500000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(4, 2), 1249999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 1500000000000000000000000); // amount of asset 1 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 750000000000000000000001); // amount of asset 2 in vault acc after creating pool

		XykStorage::burn_liquidity(RuntimeOrigin::signed(2), 1, 4, 225000000000000000000000)
			.unwrap();

		assert_eq!(
			XykStorage::asset_pool((1, 4)),
			(1200000000000000000000000, 600000000000000000000001)
		);
		assert_eq!(XykStorage::total_supply(5), 900000000000000000000000); // total liquidity assets
		assert_eq!(XykStorage::balance(5, 2), 900000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
		assert_eq!(XykStorage::balance(1, 2), 800000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(4, 2), 1399999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 1200000000000000000000000); // amount of asset 1 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 600000000000000000000001); // amount of asset 2 in vault acc after creating pool

		XykStorage::burn_liquidity(RuntimeOrigin::signed(2), 1, 4, 225000000000000000000000)
			.unwrap();

		assert_eq!(
			XykStorage::asset_pool((1, 4)),
			(900000000000000000000000, 450000000000000000000001)
		);
		assert_eq!(XykStorage::total_supply(5), 675000000000000000000000); // total liquidity assets
		assert_eq!(XykStorage::balance(5, 2), 675000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
		assert_eq!(XykStorage::balance(1, 2), 1100000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(4, 2), 1549999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 900000000000000000000000); // amount of asset 1 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 450000000000000000000001); // amount of asset 2 in vault acc after creating pool

		XykStorage::mint_liquidity(
			RuntimeOrigin::signed(2),
			1,
			4,
			1000000000000000000000000,
			10000000000000000000000000,
		)
		.unwrap();

		assert_eq!(
			XykStorage::asset_pool((1, 4)),
			(1900000000000000000000000, 950000000000000000000003)
		);
		assert_eq!(XykStorage::total_supply(5), 1425000000000000000000000); // total liquidity assets
		assert_eq!(XykStorage::balance(5, 2), 1425000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
		assert_eq!(XykStorage::balance(1, 2), 100000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(4, 2), 1049999999999999999999997); // amount of asset 1 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 1900000000000000000000000); // amount of asset 1 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 950000000000000000000003);
		// amount of asset 0 in vault acc after creating pool
	});
}

#[test]
#[serial]
fn create_pool_W() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_eq!(XykStorage::asset_pool((1, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::liquidity_asset((1, 4)), Some(5)); // liquidity assetId corresponding to newly created pool
		assert_eq!(XykStorage::liquidity_pool(5), Some((1, 4))); // liquidity assetId corresponding to newly created pool
		assert_eq!(XykStorage::total_supply(5), 50000000000000000000); // total liquidity assets
		assert_eq!(XykStorage::balance(5, 2), 50000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
		assert_eq!(XykStorage::balance(1, 2), 960000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(4, 2), 940000000000000000000); // amount of asset 1 in user acc after creating pool / initial minting
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 60000000000000000000);
		// amount of asset 1 in vault acc after creating pool
	});
}

#[test]
#[serial]
fn create_pool_N_already_exists() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err!(
			XykStorage::create_pool(RuntimeOrigin::signed(2), 1, 500000, 4, 500000,),
			Error::<Test>::PoolAlreadyExists,
		);
	});
}

#[test]
#[serial]
fn create_pool_N_already_exists_other_way() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err!(
			XykStorage::create_pool(RuntimeOrigin::signed(2), 4, 500000, 1, 500000,),
			Error::<Test>::PoolAlreadyExists,
		);
	});
}

#[test]
#[serial]
fn create_pool_N_not_enough_first_asset() {
	new_test_ext().execute_with(|| {
		let acc_id: u128 = 2;
		let amount: u128 = 1000000;
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);

		assert_err!(
			XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 1500000, 1, 500000,),
			Error::<Test>::NotEnoughAssets,
		); //asset 0 issued to user 1000000, trying to create pool using 1500000
	});
}

#[test]
#[serial]
fn create_pool_N_not_enough_second_asset() {
	new_test_ext().execute_with(|| {
		let acc_id: u128 = 2;
		let amount: u128 = 1000000;
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);

		assert_err!(
			XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 500000, 1, 1500000,),
			Error::<Test>::NotEnoughAssets,
		); //asset 1 issued to user 1000000, trying to create pool using 1500000
	});
}

#[test]
#[serial]
fn create_pool_N_same_asset() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err!(
			XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 500000, 0, 500000,),
			Error::<Test>::SameAsset,
		);
	});
}

#[test]
#[serial]
fn create_pool_N_zero_first_amount() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err!(
			XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 0, 1, 500000,),
			Error::<Test>::ZeroAmount,
		);
	});
}

#[test]
#[serial]
fn create_pool_N_zero_second_amount() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err!(
			XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 500000, 1, 0,),
			Error::<Test>::ZeroAmount,
		);
	});
}

#[test]
#[serial]
fn sell_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		initialize();

		XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(2),
			vec![1, 4],
			20000000000000000000,
			0,
		)
		.unwrap(); // selling 20000000000000000000 assetId 0 of pool 0 1

		assert_eq!(XykStorage::balance(1, 2), 940000000000000000000); // amount in user acc after selling
		assert_eq!(XykStorage::balance(4, 2), 959959959959959959959); // amount in user acc after buying
		assert_eq!(XykStorage::asset_pool((1, 4)), (59979999999999999998, 40040040040040040041)); // amount of asset 0 in pool map
																						  //   assert_eq!(XykStorage::asset_pool2((1, 0)), 40040040040040040041); // amount of asset 1 in pool map
		assert_eq!(XykStorage::balance(1, 2), 940000000000000000000); // amount of asset 0 on account 2
		assert_eq!(XykStorage::balance(4, 2), 959959959959959959959); // amount of asset 1 on account 2
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 59979999999999999998); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 40040040040040040041); // amount of asset 1 in vault acc after creating pool

		let assets_swapped_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::AssetsSwapped(
				2,
				vec![1_u32, 4_u32],
				20000000000000000000,
				19959959959959959959,
			));

		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
#[serial]
fn sell_W_other_way() {
	new_test_ext().execute_with(|| {
		initialize();

		XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(2),
			vec![4, 1],
			30000000000000000000,
			0,
		)
		.unwrap(); // selling 30000000000000000000 assetId 1 of pool 0 1

		assert_eq!(XykStorage::balance(1, 2), 973306639973306639973); // amount of asset 0 in user acc after selling
		assert_eq!(XykStorage::balance(4, 2), 910000000000000000000); // amount of asset 1 in user acc after buying
															  // assert_eq!(XykStorage::asset_pool((1, 2)), 26684462240017795575); // amount of asset 0 in pool map
															  // assert_eq!(XykStorage::asset_pool((1, 0)), 90000000000000000000); // amount of asset 1 in pool map
		assert_eq!(XykStorage::asset_pool((1, 4)), (26693360026693360027, 89969999999999999998));
		assert_eq!(XykStorage::balance(1, 2), 973306639973306639973); // amount of asset 0 on account 2
		assert_eq!(XykStorage::balance(4, 2), 910000000000000000000); // amount of asset 1 on account 2
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 26693360026693360027); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 89969999999999999998);
		// amount of asset 1 in vault acc after creating pool
	});
}

#[test]
#[serial]
fn sell_N_no_such_pool() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![0, 10], 250000, 0),
			Error::<Test>::NoSuchPool,
		); // selling 250000 assetId 0 of pool 0 10 (only pool 0 1 exists)
	});
}
#[test]
#[serial]
fn sell_N_not_enough_selling_assset() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err_ignore_postinfo!(
			// XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![1, 4], 1000000000000000000000, 0),
			XykStorage::sell_asset(RuntimeOrigin::signed(2), 1, 4, 1000000000000000000000, 0),
			orml_tokens::Error::<Test>::BalanceTooLow,
		); // selling 1000000000000000000000 assetId 0 of pool 0 1 (user has only 960000000000000000000)
	});
}

#[test]
#[serial]
fn sell_W_insufficient_output_amount() {
	new_test_ext().execute_with(|| {
		initialize();

		let input_balance_before = XykStorage::balance(1, 2);
		let output_balance_before = XykStorage::balance(4, 2);

		let mut expected_events =
			vec![Event::PoolCreated(2, 1, 40000000000000000000, 4, 60000000000000000000)];
		assert_eq_events!(expected_events.clone());

		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(2),
			vec![1, 4],
			250000,
			500000
		)); // selling 250000 assetId 0 of pool 0 1, by the formula user should get 166333 asset 1, but is requesting 500000

		let mut new_events_0 =
			vec![Event::SellAssetFailedDueToSlippage(2, 1, 250000, 4, 373874, 500000)];

		expected_events.append(&mut new_events_0);
		assert_eq_events!(expected_events.clone());

		let input_balance_after = XykStorage::balance(1, 2);
		let output_balance_after = XykStorage::balance(4, 2);

		assert_ne!(input_balance_before, input_balance_after);
		assert_eq!(output_balance_before, output_balance_after);
	});
}

#[test]
#[serial]
fn sell_N_insufficient_output_amount_inner_function_error_upon_bad_slippage() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err!(
			<XykStorage as XykFunctionsTrait<AccountId>>::sell_asset(2, 1, 4, 250000, 500000, true),
			Error::<Test>::InsufficientOutputAmount,
		); // selling 250000 assetId 0 of pool 0 1, by the formula user should get 166333 asset 1, but is requesting 500000
	});
}

#[test]
#[serial]
fn sell_W_insufficient_output_amount_inner_function_NO_error_upon_bad_slippage() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_ok!(<XykStorage as XykFunctionsTrait<AccountId>>::sell_asset(
			2, 1, 4, 250000, 500000, false
		),); // selling 250000 assetId 0 of pool 0 1, by the formula user should get 166333 asset 1, but is requesting 500000
	});
}

#[test]
#[serial]
fn sell_N_zero_amount() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![1, 4], 0, 500000),
			Error::<Test>::ZeroAmount,
		); // selling 0 assetId 0 of pool 0 1
	});
}

#[test]
fn multiswap_sell_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 4, 5],
			20000000000000000000,
			0
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 980000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1019903585399816558050);

		assert_eq!(XykStorage::asset_pool((1, 2)), (59979999999999999998, 40040040040040040041));
		assert_eq!(XykStorage::asset_pool((2, 3)), (59939999999999999999, 40066724398222064173));
		assert_eq!(XykStorage::asset_pool((3, 4)), (59913342326176157891, 40084527730110691659));
		assert_eq!(XykStorage::asset_pool((4, 5)), (59895556797619419031, 40096414600183441950));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 59979999999999999998);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 99980040040040040040);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 99980066724398222064);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 99980084527730110690);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 40096414600183441950);

		let assets_swapped_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::AssetsSwapped(
				TRADER_ID,
				vec![1, 2, 3, 4, 5],
				20000000000000000000,
				19903585399816558050,
			));

		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
fn multiswap_sell_bad_slippage_charges_fee_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 4, 5],
			20000000000000000000,
			20000000000000000000
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 999939999999999999997);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40040000000000000001, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40040000000000000001);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_event_emitted!(crate::Event::<Test>::MultiSwapAssetFailedOnAtomicSwap(
			TRADER_ID,
			vec![1, 2, 3, 4, 5],
			20000000000000000000,
			module_err(Error::<Test>::InsufficientOutputAmount)
		));
	});
}

#[test]
fn multiswap_sell_bad_atomic_swap_charges_fee_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 6, 5],
			20000000000000000000,
			20000000000000000000
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 999939999999999999997);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40040000000000000001, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40040000000000000001);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		// let assets_swapped_event = crate::mock::RuntimeEvent::XykStorage(
		// 	crate::Event::<Test>::MultiSellAssetFailedOnAtomicSwap(
		// 		TRADER_ID,
		// 		vec![1, 2, 3, 6, 5],
		// 		20000000000000000000,
		// 		Error::<Test>::PoolAlreadyExists.into()
		// 	),
		// );
		//
		// assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
fn multiswap_sell_not_enough_assets_pay_fees_fails_early_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_sell_asset(
				RuntimeOrigin::signed(TRADER_ID),
				vec![1, 2, 3, 6, 5],
				2000000000000000000000000,
				0
			),
			Error::<Test>::NotEnoughAssets
		);

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);
	});
}

#[test]
fn multiswap_sell_just_enough_assets_pay_fee_but_not_to_swap_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 4, 5],
			2000000000000000000000,
			0
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 993999999999999999997);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (44000000000000000001, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 44000000000000000001);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_event_emitted!(crate::Event::<Test>::MultiSwapAssetFailedOnAtomicSwap(
			TRADER_ID,
			vec![1, 2, 3, 4, 5],
			2000000000000000000000,
			module_err(Error::<Test>::NotEnoughAssets)
		));
	});
}

#[test]
fn multiswap_sell_with_two_hops_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3],
			20000000000000000000,
			0
		));

		let assets_swapped_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::AssetsSwapped(
				TRADER_ID,
				vec![1, 2, 3],
				20000000000000000000,
				19933275601777935827,
			));

		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
fn multiswap_sell_with_single_hops_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();
		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2],
			20000000000000000000,
			0
		),);
	});
}

#[test]
fn multiswap_sell_same_pool_works_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);

		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 1],
			20000000000000000000,
			0
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 999913320173720252676);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40066679826279747322, 59980040040040040040));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40066679826279747322);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 99980040040040040040);

		let assets_swapped_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::AssetsSwapped(
				TRADER_ID,
				vec![1, 2, 1],
				20000000000000000000,
				19913320173720252676,
			));

		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
fn multiswap_sell_loop_works_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);

		assert_ok!(XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 2, 1, 2],
			20000000000000000000,
			0
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 980000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1019787116737807948784);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (59960144443715038028, 40106459299365557069));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40066590593459994181, 59980066724398222064));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 59960144443715038028);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 80173049892825551250);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 99980066724398222064);

		let assets_swapped_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::AssetsSwapped(
				TRADER_ID,
				vec![1, 2, 3, 2, 1, 2],
				20000000000000000000,
				19787116737807948784,
			));

		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
#[serial]
fn multiswap_sell_zero_amount_does_not_work_N() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_sell_asset(
				RuntimeOrigin::signed(TRADER_ID),
				vec![1, 2, 3],
				0,
				20000000000000000000
			),
			Error::<Test>::ZeroAmount
		);
	});
}

#[test]
#[serial]
fn buy_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		initialize();

		// buying 30000000000000000000 assetId 1 of pool 0 1
		XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(2),
			vec![1, 4],
			30000000000000000000,
			3000000000000000000000,
		)
		.unwrap();
		assert_eq!(XykStorage::balance(1, 2), 919879638916750250752); // amount in user acc after selling
		assert_eq!(XykStorage::balance(4, 2), 970000000000000000000); // amount in user acc after buying
		assert_eq!(XykStorage::asset_pool((1, 4)), (80080240722166499498, 30000000000000000000));
		assert_eq!(XykStorage::balance(1, 2), 919879638916750250752); // amount of asset 0 on account 2
		assert_eq!(XykStorage::balance(4, 2), 970000000000000000000); // amount of asset 1 on account 2
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 80080240722166499498); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 30000000000000000000); // amount of asset 1 in vault acc after creating pool

		let assets_swapped_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::AssetsSwapped(
				2,
				vec![1_u32, 4_u32],
				40120361083249749248,
				30000000000000000000,
			));

		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
#[serial]
fn buy_W_other_way() {
	new_test_ext().execute_with(|| {
		initialize();

		// buying 30000000000000000000 assetId 0 of pool 0 1
		XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(2),
			vec![4, 1],
			30000000000000000000,
			3000000000000000000000,
		)
		.unwrap();
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 10000000000000000000); // amount of asset 1 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 240361083249749247743); // amount of asset 2 in vault acc after creating pool
		assert_eq!(XykStorage::balance(1, 2), 990000000000000000000); // amount in user acc after selling
		assert_eq!(XykStorage::balance(4, 2), 759458375125376128385); // amount in user acc after buying
		assert_eq!(XykStorage::asset_pool((1, 4)), (10000000000000000000, 240361083249749247743));
		assert_eq!(XykStorage::balance(1, 2), 990000000000000000000); // amount of asset 0 on account 2
		assert_eq!(XykStorage::balance(4, 2), 759458375125376128385); // amount of asset 1 on account 2
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 10000000000000000000); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 240361083249749247743);
		// amount of asset 1 in vault acc after creating pool
	});
}

#[test]
#[serial]
fn buy_N_no_such_pool() {
	new_test_ext().execute_with(|| {
		initialize();

		// buying 150000 assetId 1 of pool 0 10 (only pool 0 1 exists)
		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(RuntimeOrigin::signed(2), vec![0, 10], 150000, 5000000),
			Error::<Test>::NoSuchPool,
		);
	});
}

fn module_err(e: Error<Test>) -> ModuleError {
	if let DispatchError::Module(module_err) = DispatchError::from(e).stripped() {
		module_err
	} else {
		panic!("cannot convert error");
	}
}

#[test]
#[serial]
fn buy_N_not_enough_reserve() {
	new_test_ext().execute_with(|| {
		initialize();

		// buying 70000000000000000000 assetId 0 of pool 0 1 , only 60000000000000000000 in reserve
		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(
				RuntimeOrigin::signed(2),
				vec![1, 4],
				70000000000000000000,
				5000000000000000000000
			),
			Error::<Test>::NotEnoughReserve,
		);
	});
}

#[test]
#[serial]
fn buy_N_not_enough_selling_assset() {
	new_test_ext().execute_with(|| {
		initialize();

		// buying 59000000000000000000 assetId 1 of pool 0 1 should sell 2.36E+21 assetId 0, only 9.6E+20 in acc
		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(
				RuntimeOrigin::signed(2),
				vec![1, 4],
				59000000000000000000,
				59000000000000000000000
			),
			Error::<Test>::NotEnoughAssets,
		);
	});
}

#[test]
#[serial]
fn buy_W_insufficient_input_amount() {
	new_test_ext().execute_with(|| {
		initialize();

		let mut expected_events =
			vec![Event::PoolCreated(2, 1, 40000000000000000000, 4, 60000000000000000000)];
		assert_eq_events!(expected_events.clone());

		let input_balance_before = XykStorage::balance(1, 2);
		let output_balance_before = XykStorage::balance(4, 2);

		// buying 150000 liquidity assetId 1 of pool 0 1
		assert_ok!(XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(2),
			vec![1, 4],
			150000,
			10
		));
		let mut new_events_0 =
			vec![Event::BuyAssetFailedDueToSlippage(2, 1, 100301, 4, 150000, 10)];

		expected_events.append(&mut new_events_0);
		assert_eq_events!(expected_events.clone());

		let input_balance_after = XykStorage::balance(1, 2);
		let output_balance_after = XykStorage::balance(4, 2);

		assert_ne!(input_balance_before, input_balance_after);
		assert_eq!(output_balance_before, output_balance_after);
	});
}

#[test]
#[serial]
fn buy_N_insufficient_input_amount_inner_function_error_upon_bad_slippage() {
	new_test_ext().execute_with(|| {
		initialize();

		// buying 150000 liquidity assetId 1 of pool 0 1
		assert_err!(
			<XykStorage as XykFunctionsTrait<AccountId>>::buy_asset(2, 1, 4, 150000, 10, true),
			Error::<Test>::InsufficientInputAmount,
		);
	});
}

#[test]
#[serial]
fn buy_W_insufficient_input_amount_inner_function_NO_error_upon_bad_slippage() {
	new_test_ext().execute_with(|| {
		initialize();

		// buying 150000 liquidity assetId 1 of pool 0 1
		assert_ok!(<XykStorage as XykFunctionsTrait<AccountId>>::buy_asset(
			2, 1, 4, 150000, 10, false
		));
	});
}

#[test]
#[serial]
fn buy_N_zero_amount() {
	new_test_ext().execute_with(|| {
		initialize();

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(RuntimeOrigin::signed(2), vec![1, 4], 0, 0),
			Error::<Test>::ZeroAmount,
		); // buying 0 assetId 0 of pool 0 1
	});
}

#[test]
fn multiswap_buy_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_ok!(XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 4, 5],
			20000000000000000000,
			200000000000000000000
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 979503362184986098460);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1020000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (60476141177198887638, 39711990180100104523));
		assert_eq!(XykStorage::asset_pool((2, 3)), (60267721810079995581, 39849140591034781894));
		assert_eq!(XykStorage::asset_pool((3, 4)), (60130708549556252886, 39939819458375125376));
		assert_eq!(XykStorage::asset_pool((4, 5)), (60040120361083249748, 40000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 60476141177198887638);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 99979711990180100104);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 99979849140591034780);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 99979939819458375124);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 40000000000000000000);

		let assets_swapped_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::AssetsSwapped(
				TRADER_ID,
				vec![1, 2, 3, 4, 5],
				20496637815013901540,
				20000000000000000000,
			));

		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
fn multiswap_buy_bad_slippage_charges_fee_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_ok!(XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 4, 5],
			20000000000000000000,
			200000000000000000
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 999999399999999999997);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000400000000000001, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000400000000000001);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_event_emitted!(crate::Event::<Test>::MultiSwapAssetFailedOnAtomicSwap(
			TRADER_ID,
			vec![1, 2, 3, 4, 5],
			20000000000000000000,
			module_err(Error::<Test>::InsufficientInputAmount)
		));
	});
}

#[test]
fn multiswap_buy_bad_atomic_swap_charges_fee_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_ok!(XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 6, 5],
			20000000000000000000,
			200000000000000000
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 999999399999999999997);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000400000000000001, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000400000000000001);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		// let assets_swapped_event = crate::mock::RuntimeEvent::XykStorage(
		// 	crate::Event::<Test>::MultiBuyAssetFailedOnAtomicSwap(
		// 		TRADER_ID,
		// 		vec![1, 2, 3, 6, 5],
		// 		20000000000000000000,
		// 	),
		// );
		//
		// assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
fn multiswap_buy_not_enough_assets_pay_fees_fails_early_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(
				RuntimeOrigin::signed(TRADER_ID),
				vec![1, 2, 3, 6, 5],
				2000000000000000000000000,
				2000000000000000000000000
			),
			Error::<Test>::NotEnoughAssets
		);

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);
	});
}

#[test]
fn multiswap_buy_just_enough_assets_pay_fee_but_not_to_swap_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_ok!(<Test as Config>::Currency::transfer(
			1u32.into(),
			&TRADER_ID,
			&DUMMY_USER_ID,
			(1000000000000000000000u128 - 1000000u128).into(),
			ExistenceRequirement::KeepAlive
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 1000000);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_ok!(XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3, 4, 5],
			100000000,
			20000000
		));

		assert_eq!(XykStorage::balance(1, TRADER_ID), 939997);
		assert_eq!(XykStorage::balance(2, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(3, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(4, TRADER_ID), 1000000000000000000000);
		assert_eq!(XykStorage::balance(5, TRADER_ID), 1000000000000000000000);

		assert_eq!(XykStorage::asset_pool((1, 2)), (40000000000000040001, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((2, 3)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((3, 4)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::asset_pool((4, 5)), (40000000000000000000, 60000000000000000000));
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000040001);
		assert_eq!(XykStorage::balance(2, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(3, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 100000000000000000000);
		assert_eq!(XykStorage::balance(5, XykStorage::account_id()), 60000000000000000000);

		assert_event_emitted!(Event::<Test>::MultiSwapAssetFailedOnAtomicSwap(
			TRADER_ID,
			vec![1, 2, 3, 4, 5],
			100000000,
			module_err(Error::<Test>::NotEnoughAssets)
		));
	});
}

#[test]
fn multiswap_buy_with_two_hops_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_ok!(XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![1, 2, 3],
			20000000000000000000,
			2000000000000000000000
		));

		let assets_swapped_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::AssetsSwapped(
				TRADER_ID,
				vec![1, 2, 3],
				20150859408965218106,
				20000000000000000000,
			));

		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
	});
}

#[test]
fn multiswap_buy_with_single_hops_W() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();
		assert_ok!(XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(TRADER_ID),
			vec![2, 1],
			20000000000000000000,
			40000000000000000000,
		),);
	});
}

#[test]
fn multiswap_buy_same_pool_does_not_work_N() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(
				RuntimeOrigin::signed(TRADER_ID),
				vec![1, 2, 1],
				20000000000000000000,
				20000000000000000000
			),
			Error::<Test>::MultiBuyAssetCantHaveSamePoolAtomicSwaps
		);
	});
}

#[test]
fn multiswap_buy_loop_does_not_work_N() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(
				RuntimeOrigin::signed(TRADER_ID),
				vec![1, 2, 3, 2, 1, 2],
				20000000000000000000,
				20000000000000000000
			),
			Error::<Test>::MultiBuyAssetCantHaveSamePoolAtomicSwaps
		);
	});
}

#[test]
#[serial]
fn multiswap_buy_zero_amount_does_not_work_N() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		multi_initialize();

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(
				RuntimeOrigin::signed(TRADER_ID),
				vec![1, 2, 3],
				0,
				20000000000000000000
			),
			Error::<Test>::ZeroAmount
		);
		assert_err_ignore_postinfo!(
			XykStorage::multiswap_buy_asset(
				RuntimeOrigin::signed(TRADER_ID),
				vec![1, 2, 3],
				20000000000000000000,
				0
			),
			Error::<Test>::ZeroAmount
		);
	});
}

#[test]
#[serial]
fn burn_all_liq_and_mint_it_again() {
	new_test_ext().execute_with(|| {
		initialize();

		let (asset_value_1, asset_value_4) = XykStorage::asset_pool((1, 4));
		let liq_token_id = XykStorage::liquidity_asset((1, 4));
		let total_issuance_of_liq_amount: u128 =
			<Test as Config>::Currency::total_issuance(liq_token_id.unwrap()).into();

		//burn half of the liquidity
		XykStorage::burn_liquidity(
			RuntimeOrigin::signed(2),
			1,
			4,
			total_issuance_of_liq_amount / 2,
		)
		.unwrap();

		let (divided_asset_value_1, divided_asset_value_4) = XykStorage::asset_pool((1, 4));
		let divided_total_issuance_of_liq_amount: u128 =
			<Test as Config>::Currency::total_issuance(liq_token_id.unwrap()).into();

		assert_eq!(divided_asset_value_1, asset_value_1 / 2);
		assert_eq!(divided_asset_value_4, asset_value_4 / 2);
		assert_eq!(divided_total_issuance_of_liq_amount, total_issuance_of_liq_amount / 2);

		//burn second half of liquidity
		XykStorage::burn_liquidity(
			RuntimeOrigin::signed(2),
			1,
			4,
			total_issuance_of_liq_amount / 2,
		)
		.unwrap();

		let (empty_asset_value_1, empty_asset_value_4) = XykStorage::asset_pool((1, 4));
		let empty_total_issuance_of_liq_amount: u128 =
			<Test as Config>::Currency::total_issuance(liq_token_id.unwrap()).into();

		assert_eq!(empty_asset_value_1, 0);
		assert_eq!(empty_asset_value_4, 0);
		assert_eq!(empty_total_issuance_of_liq_amount, 0);

		// XykStorage::mint_liquidity(
		// 	RuntimeOrigin::signed(2),
		// 	4,
		// 	1,
		// 	asset_value_1,
		// 	asset_value_4,
		// ).unwrap();
		//
		// let liq_token_idaa = XykStorage::liquidity_asset((1, 4));
		//
		// let total_issuance_of_liq_amount: u128 =
		// 	<Test as Config>::Currency::total_issuance(liq_token_idaa.unwrap()).into();
	});
}

#[test]
#[serial]
fn mint_W() {
	new_test_ext().execute_with(|| {
		initialize();
		// minting pool 0 1 with 20000000000000000000 assetId 0
		XykStorage::mint_liquidity(
			RuntimeOrigin::signed(2),
			1,
			4,
			20000000000000000000,
			30000000000000000001,
		)
		.unwrap();

		assert_eq!(XykStorage::total_supply(5), 75000000000000000000); // total liquidity assets
		assert_eq!(XykStorage::balance(5, 2), 75000000000000000000); // amount of liquidity assets owned by user by creating pool and minting
		assert_eq!(XykStorage::asset_pool((1, 4)), (60000000000000000000, 90000000000000000001));
		assert_eq!(XykStorage::balance(1, 2), 940000000000000000000); // amount of asset 1 in user acc after minting
		assert_eq!(XykStorage::balance(4, 2), 909999999999999999999); // amount of asset 2 in user acc after minting
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 60000000000000000000); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 90000000000000000001); // amount of asset 1 in vault acc after creating pool
		let liquidity_minted_event =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::LiquidityMinted(
				2,
				1,
				20000000000000000000,
				4,
				30000000000000000001,
				5,
				25000000000000000000,
			));

		assert!(System::events().iter().any(|record| record.event == liquidity_minted_event));
	});
}

#[test]
#[serial]
fn mint_W_other_way() {
	new_test_ext().execute_with(|| {
		initialize();
		// minting pool 0 1 with 30000000000000000000 assetId 1
		XykStorage::mint_liquidity(
			RuntimeOrigin::signed(2),
			4,
			1,
			30000000000000000000,
			300000000000000000000,
		)
		.unwrap();

		assert_eq!(XykStorage::total_supply(5), 75000000000000000000); // total liquidity assets
		assert_eq!(XykStorage::balance(5, 2), 75000000000000000000); // amount of liquidity assets owned by user by creating pool and minting
		assert_eq!(XykStorage::asset_pool((1, 4)), (60000000000000000001, 90000000000000000000));
		assert_eq!(XykStorage::balance(1, 2), 939999999999999999999); // amount of asset 0 in user acc after minting
		assert_eq!(XykStorage::balance(4, 2), 910000000000000000000); // amount of asset 1 in user acc after minting
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 60000000000000000001); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 90000000000000000000);
		// amount of asset 1 in vault acc after creating pool
	});
}

#[test]
#[serial]
fn mint_N_no_such_pool() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::mint_liquidity(RuntimeOrigin::signed(2), 0, 10, 250000, 250000),
			Error::<Test>::NoSuchPool,
		); // minting pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
	});
}

#[test]
#[serial]
fn mint_N_not_enough_first_asset() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::mint_liquidity(
				RuntimeOrigin::signed(2),
				1,
				4,
				1000000000000000000000,
				10000000000000000000000
			),
			Error::<Test>::NotEnoughAssets,
		); // minting pool 0 1 with 1000000000000000000000 assetId 0 (user has only 960000000000000000000)
	});
}

#[test]
#[serial]
fn mint_N_not_enough_second_asset() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::mint_liquidity(
				RuntimeOrigin::signed(2),
				4,
				1,
				1000000000000000000000,
				10000000000000000000000,
			),
			Error::<Test>::NotEnoughAssets,
		); // minting pool 0 1 with 1000000000000000000000 assetId 1 (user has only 940000000000000000000)
	});
}

#[test]
#[serial]
fn min_N_zero_amount() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::mint_liquidity(RuntimeOrigin::signed(2), 1, 4, 0, 10),
			Error::<Test>::ZeroAmount,
		); // minting pool 0 1 with 0 assetId 1
	});
}

#[test]
#[serial]
fn mint_N_second_asset_amount_exceeded_expectations() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::mint_liquidity(RuntimeOrigin::signed(2), 1, 4, 250000, 10),
			Error::<Test>::SecondAssetAmountExceededExpectations,
		); // minting pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
	});
}

#[test]
#[serial]
fn burn_W() {
	new_test_ext().execute_with(|| {
		initialize();
		XykStorage::burn_liquidity(RuntimeOrigin::signed(2), 1, 4, 25000000000000000000).unwrap(); // burning 20000000000000000000 asset 0 of pool 0 1

		assert_eq!(XykStorage::balance(5, 2), 25000000000000000000); // amount of liquidity assets owned by user by creating pool and burning
		assert_eq!(XykStorage::asset_pool((1, 4)), (20000000000000000000, 30000000000000000000));
		assert_eq!(XykStorage::balance(1, 2), 980000000000000000000); // amount of asset 0 in user acc after burning
		assert_eq!(XykStorage::balance(4, 2), 970000000000000000000); // amount of asset 1 in user acc after burning
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 20000000000000000000); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 30000000000000000000); // amount of asset 1 in vault acc after creating pool

		let liquidity_burned =
			crate::mock::RuntimeEvent::XykStorage(crate::Event::<Test>::LiquidityBurned(
				2,
				1,
				20000000000000000000,
				4,
				30000000000000000000,
				5,
				25000000000000000000,
			));

		assert!(System::events().iter().any(|record| record.event == liquidity_burned));
	});
}

#[test]
#[serial]
fn burn_W_other_way() {
	new_test_ext().execute_with(|| {
		initialize();
		XykStorage::burn_liquidity(RuntimeOrigin::signed(2), 4, 1, 25000000000000000000).unwrap(); // burning 30000000000000000000 asset 1 of pool 0 1

		assert_eq!(XykStorage::balance(5, 2), 25000000000000000000); // amount of liquidity assets owned by user by creating pool and burning
		assert_eq!(XykStorage::asset_pool((1, 4)), (20000000000000000000, 30000000000000000000));
		assert_eq!(XykStorage::balance(1, 2), 980000000000000000000); // amount of asset 0 in user acc after burning
		assert_eq!(XykStorage::balance(4, 2), 970000000000000000000); // amount of asset 1 in user acc after burning
		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 20000000000000000000); // amount of asset 0 in vault acc after creating pool
		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 30000000000000000000);
		// amount of asset 1 in vault acc after creating pool
	});
}

#[test]
#[serial]
fn burn_N_not_enough_liquidity_asset() {
	new_test_ext().execute_with(|| {
		initialize();
		// burning pool 0 1 with 500000000000000000000 liquidity asset amount (user has only 100000000000000000000 liquidity asset amount)
		assert_err!(
			XykStorage::burn_liquidity(RuntimeOrigin::signed(2), 1, 4, 500000000000000000000,),
			Error::<Test>::NotEnoughAssets,
		);
	});
}

#[test]
#[serial]
fn burn_N_no_such_pool() {
	new_test_ext().execute_with(|| {
		initialize();
		// burning pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
		assert_err!(
			XykStorage::burn_liquidity(RuntimeOrigin::signed(2), 0, 10, 250000,),
			Error::<Test>::NoSuchPool,
		);
	});
}

#[test]
#[serial]
fn burn_N_zero_amount() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::burn_liquidity(RuntimeOrigin::signed(2), 1, 4, 0,),
			Error::<Test>::ZeroAmount,
		); // burning pool 0 1 with 0 assetId 1
	});
}

#[test]
#[serial]
fn buy_assets_with_small_expected_amount_does_not_cause_panic() {
	new_test_ext().execute_with(|| {
		initialize();
		let first_token_balance = XykStorage::balance(1, DUMMY_USER_ID);
		let _ = XykStorage::multiswap_buy_asset(
			RuntimeOrigin::signed(2),
			vec![1, 4],
			1,
			first_token_balance,
		);
	});
}

#[test]
#[serial]
fn swaps_are_annotated_as_PaysNo() {
	new_test_ext().execute_with(|| {
		let calls = [
			mock::RuntimeCall::XykStorage(Call::sell_asset {
				sold_asset_id: 0u32,
				bought_asset_id: 1u32,
				sold_asset_amount: 10u128,
				min_amount_out: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::buy_asset {
				sold_asset_id: 0u32,
				bought_asset_id: 1u32,
				bought_asset_amount: 10u128,
				max_amount_in: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::multiswap_buy_asset {
				swap_token_list: vec![0, 2],
				bought_asset_amount: 10u128,
				max_amount_in: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::multiswap_sell_asset {
				swap_token_list: vec![0, 2],
				sold_asset_amount: 10,
				min_amount_out: 0,
			}),
		];

		assert!(calls
			.iter()
			.map(|call| call.get_dispatch_info())
			.all(|dispatch| dispatch.pays_fee == Pays::No));
	});
}

#[test]
#[serial]
fn successful_swaps_are_free() {
	new_test_ext().execute_with(|| {
		initialize();
		XykStorage::create_pool(RuntimeOrigin::signed(2), 1, 1000, 0, 1000).unwrap();

		let calls = [
			mock::RuntimeCall::XykStorage(Call::sell_asset {
				sold_asset_id: 0u32,
				bought_asset_id: 1u32,
				sold_asset_amount: 10u128,
				min_amount_out: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::buy_asset {
				sold_asset_id: 0u32,
				bought_asset_id: 1u32,
				bought_asset_amount: 10u128,
				max_amount_in: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::multiswap_buy_asset {
				swap_token_list: vec![0, 1],
				bought_asset_amount: 10u128,
				max_amount_in: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::multiswap_sell_asset {
				swap_token_list: vec![0, 1],
				sold_asset_amount: 10,
				min_amount_out: 0,
			}),
		];

		assert!(calls.iter().all(|call| matches!(
			call.clone().dispatch(RuntimeOrigin::signed(2)),
			Ok(post_info) if post_info.pays_fee == Pays::No
		)));
	});
}

#[test]
#[serial]
fn unsuccessful_swaps_are_not_free() {
	new_test_ext().execute_with(|| {
		initialize();
		// XykStorage::create_pool(RuntimeOrigin::signed(2), 1, 1000, 0, 1000).unwrap();

		let calls = [
			mock::RuntimeCall::XykStorage(Call::sell_asset {
				sold_asset_id: 0u32,
				bought_asset_id: 1u32,
				sold_asset_amount: 10u128,
				min_amount_out: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::buy_asset {
				sold_asset_id: 0u32,
				bought_asset_id: 1u32,
				bought_asset_amount: 10u128,
				max_amount_in: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::multiswap_buy_asset {
				swap_token_list: vec![0, 1],
				bought_asset_amount: 10u128,
				max_amount_in: 1u128,
			}),
			mock::RuntimeCall::XykStorage(Call::multiswap_sell_asset {
				swap_token_list: vec![0, 1],
				sold_asset_amount: 10,
				min_amount_out: 0,
			}),
		];

		assert!(calls.iter().all(|call| {
			matches!(
				call.clone().dispatch(RuntimeOrigin::signed(2)),
				Err(err) if err.post_info.pays_fee == Pays::Yes
				&& err.post_info.actual_weight.unwrap().ref_time() > 0
			)
		}));
	});
}

#[test]
#[serial]
fn unsuccessful_buy_assets_charges_fee() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let post_info =
			XykStorage::multiswap_buy_asset(RuntimeOrigin::signed(2), vec![100, 200], 0, 0)
				.unwrap_err()
				.post_info;
		assert_eq!(post_info.pays_fee, Pays::Yes);

		let post_info =
			XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![100, 200], 0, 0)
				.unwrap_err()
				.post_info;
		assert_eq!(post_info.pays_fee, Pays::Yes);
	});
}

#[test]
#[serial]
#[ignore]
fn successful_sell_assets_does_not_charge_fee() {
	new_test_ext().execute_with(|| {
		initialize();
		let first_token_balance = XykStorage::balance(1, DUMMY_USER_ID);
		let post_info = XykStorage::multiswap_sell_asset(
			RuntimeOrigin::signed(2),
			vec![1, 4],
			first_token_balance,
			0,
		)
		.unwrap();
		assert_eq!(post_info.pays_fee, Pays::No);
	});
}

#[test]
#[serial]
fn unsuccessful_sell_assets_charges_fee() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		//try to sell non owned, non existing tokens
		let post_info =
			XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![100, 200], 0, 0)
				.unwrap_err()
				.post_info;
		assert_eq!(post_info.pays_fee, Pays::Yes);
	});
}

#[test]
#[serial]
fn PoolCreateApi_test_pool_exists_return_false_for_non_existing_pool() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert!(!<XykStorage as PoolCreateApi>::pool_exists(1_u32, 4_u32));
	});
}

#[test]
#[serial]
fn PoolCreateApi_pool_exists_return_true_for_existing_pool() {
	new_test_ext().execute_with(|| {
		initialize();

		XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 500000, 1, 10000).unwrap();
		assert!(<XykStorage as PoolCreateApi>::pool_exists(0_u32, 1_u32));
	});
}

#[test]
#[serial]
fn PoolCreateApi_pool_create_creates_a_pool() {
	new_test_ext().execute_with(|| {
		initialize();

		let first_asset_id = 0_u32;
		let first_asset_amount = 10_000_u128;
		let second_asset_id = 1_u32;
		let second_asset_amount = 5_000_u128;
		assert!(!<XykStorage as PoolCreateApi>::pool_exists(first_asset_id, second_asset_id));

		let liq_token_id = Tokens::next_asset_id();
		let liq_token_amount = (first_asset_amount + second_asset_amount) / 2;

		assert_eq!(
			<XykStorage as PoolCreateApi>::pool_create(
				DUMMY_USER_ID,
				first_asset_id,
				first_asset_amount,
				second_asset_id,
				second_asset_amount
			),
			Some((liq_token_id, liq_token_amount))
		);

		assert_ne!(liq_token_id, Tokens::next_asset_id());
		assert_eq!(liq_token_amount, XykStorage::balance(liq_token_id, DUMMY_USER_ID));

		assert!(<XykStorage as PoolCreateApi>::pool_exists(0_u32, 1_u32));
	});
}

#[test]
#[serial]
fn test_create_blacklisted_pool() {
	new_test_ext().execute_with(|| {
		let blaclisted_first_asset_id = 1;
		let blaclisted_second_asset_id = 9;

		assert_err!(
			XykStorage::create_pool(
				RuntimeOrigin::signed(2),
				blaclisted_first_asset_id,
				100000000000000,
				blaclisted_second_asset_id,
				100000000000000
			),
			Error::<Test>::DisallowedPool
		);

		assert_err!(
			XykStorage::create_pool(
				RuntimeOrigin::signed(2),
				blaclisted_second_asset_id,
				100000000000000,
				blaclisted_first_asset_id,
				100000000000000
			),
			Error::<Test>::DisallowedPool
		);
	});
}

#[test_case(200_000_000_000_000_000_000_u128, 1_000_000_000_000_u128, 1_u128 ; "swap plus 1 leftover")]
#[test_case(2_000_u128, 100_u128, 2_u128 ; "swap plus 2 leftover")]
#[test_case(1_000_000_000_000_000_000_000_000_000, 135_463_177_684_253_389, 2_u128 ; "benchmark case")]
#[serial]
fn test_compound_calculate_balanced_swap_for_liquidity(amount: u128, reward: u128, surplus: u128) {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let acc_id: u128 = 2;
		let pool = amount / 2;
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_pool(RuntimeOrigin::signed(2), 0, pool, 1, pool).unwrap();
		let balance_before_0 = XykStorage::balance(0, 2);
		let balance_before_1 = XykStorage::balance(1, 2);

		let swap_amount = XykStorage::calculate_balanced_sell_amount(reward, pool).unwrap();
		let swapped_amount = XykStorage::calculate_sell_price(pool, pool, swap_amount).unwrap();

		XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![0, 1], swap_amount, 0)
			.unwrap();

		XykStorage::mint_liquidity(RuntimeOrigin::signed(2), 1, 0, swapped_amount, u128::MAX)
			.unwrap();

		assert_eq!(XykStorage::balance(0, 2), balance_before_0 - reward + surplus);
		assert_eq!(XykStorage::balance(1, 2), balance_before_1);
	});
}

#[test_case(100_000, 1_000, 2, 1 ; "surplus of 1")]
#[test_case(100_000_000_000, 1_000, 2, 1 ; "large reserve, surplus of 1")]
#[test_case(100_000_000_000, 1_000_000_000, 1_000_000, 52815 ; "small pool, large surplus")]
#[test_case(1_000_000_000, 100_000, 2, 2 ; "benchmark precision test")]
#[serial]
fn test_compound_provide_liquidity(amount: u128, reward: u128, pool_r: u128, surplus: u128) {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let acc_id: u128 = 2;
		let pool = amount / pool_r;
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_pool(RuntimeOrigin::signed(2), 0, pool, 1, pool).unwrap();
		let balance_before_0 = XykStorage::balance(0, 2);
		let balance_before_1 = XykStorage::balance(1, 2);

		XykStorage::provide_liquidity_with_conversion(RuntimeOrigin::signed(2), 2, 0, reward)
			.unwrap();

		assert_eq!(XykStorage::balance(0, 2), balance_before_0 - reward + surplus);
		assert_eq!(XykStorage::balance(1, 2), balance_before_1);
	});
}

#[test_case(2_000_000, 1_000_000, 0 ; "compound all rewards")]
#[test_case(2_000_000, 500_000, 0 ; "compound half rewards")]
#[test_case(100_000_000_000_000_000_000, 1_000_000, 1 ; "benchmark precision test")]
#[serial]
fn test_compound_rewards(amount: u128, part_permille: u32, surplus: u128) {
	new_test_ext().execute_with(|| {
		let amount_permille = Permill::from_parts(part_permille);
		System::set_block_number(1);

		// MockPromotedPoolApi::instance().lock().unwrap().clear();

		XykStorage::create_new_token(&2, amount);
		XykStorage::create_new_token(&2, amount);
		XykStorage::create_pool(RuntimeOrigin::signed(2), 0, amount / 2, 1, amount / 2).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 2, 1u8).unwrap();
		ProofOfStake::activate_liquidity(RuntimeOrigin::signed(2), 2, amount / 2, None).unwrap();

		// MockPromotedPoolApi::instance().lock().unwrap().insert(2, U256::from(u128::MAX));
		ProofOfStake::distribute_rewards(amount / 2);
		System::set_block_number(10);

		let amount = ProofOfStake::calculate_rewards_amount(2, 2).unwrap();
		XykStorage::transfer(
			0,
			2,
			<Test as pallet_proof_of_stake::Config>::LiquidityMiningIssuanceVault::get(),
			amount,
		)
		.unwrap();

		let balance_before_0 = XykStorage::balance(0, 2);
		let balance_before_1 = XykStorage::balance(1, 2);
		let balance_not_compounded: u128 = (Permill::one() - amount_permille) * amount;
		XykStorage::compound_rewards(RuntimeOrigin::signed(2), 2, amount_permille).unwrap();

		assert_eq!(XykStorage::balance(0, 2), balance_before_0 + surplus + balance_not_compounded);
		assert_eq!(XykStorage::balance(1, 2), balance_before_1);
	});
}

#[test]
#[serial]
fn test_compound_rewards_pool_assets_order_swapped() {
	new_test_ext().execute_with(|| {
		let amount = 2_000_000_u128;
		let amount_permille = Permill::from_parts(1_000_000);
		System::set_block_number(1);
		// MockPromotedPoolApi::instance().lock().unwrap().clear();

		XykStorage::create_new_token(&2, amount);
		XykStorage::create_new_token(&2, amount);
		XykStorage::create_pool(RuntimeOrigin::signed(2), 1, amount / 2, 0, amount / 2).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 2, 1u8).unwrap();
		ProofOfStake::activate_liquidity(RuntimeOrigin::signed(2), 2, amount / 2, None).unwrap();

		// MockPromotedPoolApi::instance().lock().unwrap().insert(2, U256::from(u128::MAX));
		ProofOfStake::distribute_rewards(amount / 2);

		System::set_block_number(10);

		let amount = ProofOfStake::calculate_rewards_amount(2, 2).unwrap();
		XykStorage::transfer(
			0,
			2,
			<Test as pallet_proof_of_stake::Config>::LiquidityMiningIssuanceVault::get(),
			amount,
		)
		.unwrap();

		let balance_before_0 = XykStorage::balance(0, 2);
		let balance_before_1 = XykStorage::balance(1, 2);
		let balance_not_compounded: u128 = (Permill::one() - amount_permille) * amount;
		XykStorage::compound_rewards(RuntimeOrigin::signed(2), 2, amount_permille).unwrap();

		assert_eq!(XykStorage::balance(0, 2), balance_before_0 + balance_not_compounded);
		assert_eq!(XykStorage::balance(1, 2), balance_before_1);
	});
}

#[test]
#[serial]
fn sell_N_maintenance_mode() {
	new_test_ext().execute_with(|| {
		initialize();

		MockMaintenanceStatusProvider::set_maintenance(true);

		assert_err_ignore_postinfo!(
			XykStorage::multiswap_sell_asset(RuntimeOrigin::signed(2), vec![1, 4], 20000000, 0),
			Error::<Test>::TradingBlockedByMaintenanceMode,
		);
	});
}

#[test]
#[serial]
fn test_compound_rewards_error_on_non_native_pool() {
	new_test_ext().execute_with(|| {
		XykStorage::create_new_token(&2, 2_000_000_u128);
		XykStorage::create_new_token(&2, 2_000_000_u128);
		XykStorage::create_new_token(&2, 2_000_000_u128);
		XykStorage::create_pool(RuntimeOrigin::signed(2), 1, 1000, 2, 1000).unwrap();

		assert_err!(
			XykStorage::compound_rewards(
				RuntimeOrigin::signed(2),
				3,
				Permill::from_parts(1_000_000)
			),
			Error::<Test>::FunctionNotAvailableForThisToken
		);
	});
}

#[test]
#[serial]
fn buy_W_maintenance_mode() {
	new_test_ext().execute_with(|| {
		initialize();

		MockMaintenanceStatusProvider::set_maintenance(true);

		assert_err_ignore_postinfo!(
			// buying 30000000000000000000 assetId 1 of pool 0 1
			XykStorage::multiswap_buy_asset(
				RuntimeOrigin::signed(2),
				vec![1, 4],
				30000000000000000000,
				3000000000000000000000,
			),
			Error::<Test>::TradingBlockedByMaintenanceMode,
		);
	});
}
