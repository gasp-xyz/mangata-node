// Copyright (C) 2020 Mangata team
#![cfg(not(feature = "runtime-benchmarks"))]
#![allow(non_snake_case)]

use super::*;
use crate::mock::*;
use frame_support::assert_err;
use mangata_types::assets::CustomMetadata;
use orml_traits::asset_registry::AssetMetadata;
use serial_test::serial;
//fn create_pool_W(): create_pool working assert (maps,acocounts values)  //DONE
//fn create_pool_N_already_exists(): create_pool not working if pool already exists  //DONE
//fn create_pool_N_already_exists_other_way(): create_pool not working if pool already exists other way around (create pool X-Y, but pool Y-X exists) //DONE
//fn create_pool_N_not_enough_first_asset(): create_pool not working if account has not enough first asset for initial mint //DONE
//fn create_pool_N_not_enough_second_asset(): create_pool not working if account has not enough second asset for initial mint //DONE
//fn create_pool_N_same_asset(): create_pool not working if creating pool with same asset on both sides //DONE
//fn create_pool_N_zero_first_amount(): create_pool not working if creating with 0 amount of first asset
//fn create_pool_N_zero_second_amount(): create_pool not working if creating with 0 amount of first asset

//fn sell_W(): sell working assert (maps,acocounts values) //DONE
//fn sell_W_other_way(): sell working if sell order in different order as pool (sell pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn sell_N_not_enough_selling_assset(): sell not working if not enough asset to sell //DONE
//fn sell_N_no_such_pool(): sell not working if pool does not exist //DONE
//fn sell_N_insufficient_output_amount(): sell not working if insufficient_output_amount //DONE
//fn sell_N_zero_amount(): sell not working if trying to sell 0 asset

//fn buy_W(): buy working assert (maps,acocounts values) //DONE
//fn buy_W_other_way(): buy working if buy order in different order as pool (buy pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn buy_N_not_enough_selling_assset(): buy not working if not enough asset to sell //DONE
//fn buy_N_not_enough_reserve(): buy not working if not enough liquidity in pool //DONE
//fn buy_N_no_such_pool(): buy not working if pool does not exist //DONE
//fn buy_N_insufficient_input_amount(): sell not working if insufficient_output_amount
//fn buy_N_zero_amount(): buy not working if trying to buy 0 asset

//fn mint_W(): mint working assert (maps,acocounts values) //DONE
//fn mint_W_other_way(): mint working if mint order in different order as pool (mint pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn mint_N_no_such_pool(): mint not working if pool does not exist //DONE
//fn mint_N_not_enough_first_asset(): mint not working, not enough first asset to mint with //DONE
//fn mint_N_not_enough_second_asset(): mint not working, not enough second asset to mint with //DONE
//fn mint_N_second_token_amount_exceeded_expectations:  mint not working, required more second token amount then expected // DONE
//fn mint_W_no_expected_argument:  mint works when providing only 3 arguments // DONE
//fn min_N_zero_amount(): mint not working if trying to mint 0 asset

//fn burn_W(): burn working assert (maps,acocounts values) //DONE
//fn burn_W_other_way(): burn working if burn order in different order as pool (burn pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn burn_N_no_such_pool(): burn not working if pool does not exist //DONE
//fn burn_N_not_enough_liquidity_asset(): burn not enough liquidity asset in liquidity pool to burn //DONE
//fn burn_N_zero_amount(): burn not working if trying to burn 0 asset

//liquidity assets after trade, after burn, after mint

// pub trait Trait: frame_system::Trait {
//     // TODO: Add other types and constants required configure this module.
//     // type Hashing = BlakeTwo256;

//     // The overarching event type.
//     type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
// }

// W - should work
// N - should not work
const DUMMY_USER_ID: u128 = 2;

// fn initialize() {
// 	// creating asset with assetId 0 and minting to accountId 2
// 	System::set_block_number(1);
// 	let acc_id: u128 = 2;
// 	let amount: u128 = 1000000000000000000000;
// 	// creates token with ID = 0;
// 	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
// 	// creates token with ID = 1;
// 	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
// 	// creates token with ID = 2;
// 	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
// 	// creates token with ID = 3;
// 	XykStorage::create_new_token(&DUMMY_USER_ID, amount);
// 	// creates token with ID = 4;
// 	XykStorage::create_new_token(&DUMMY_USER_ID, amount);

// 	XykStorage::create_pool(
// 		Origin::signed(DUMMY_USER_ID),
// 		1,
// 		40000000000000000000,
// 		4,
// 		60000000000000000000,
// 	)
// 	.unwrap();

// 	let pool_created_event = crate::mock::Event::XykStorage(crate::Event::<Test>::PoolCreated(
// 		acc_id,
// 		1,
// 		40000000000000000000,
// 		4,
// 		60000000000000000000,
// 	));

// 	assert!(System::events().iter().any(|record| record.event == pool_created_event));
// }

// fn initialize_buy_and_burn() {
// 	// creating asset with assetId 0 and minting to accountId 2
// 	let acc_id: u128 = 2;
// 	let amount: u128 = 1000000000000000;

// 	XykStorage::create_new_token(&acc_id, amount);
// 	XykStorage::create_new_token(&acc_id, amount);
// 	XykStorage::create_new_token(&acc_id, amount); // token id 2 is dummy
// 	XykStorage::create_new_token(&acc_id, amount); // token id 3 is LT for mga-dummy
// 	XykStorage::create_new_token(&acc_id, amount);
// 	XykStorage::create_new_token(&acc_id, amount);
// 	XykStorage::create_pool(Origin::signed(2), 0, 100000000000000, 1, 100000000000000).unwrap();
// 	XykStorage::create_pool(Origin::signed(2), 1, 100000000000000, 4, 100000000000000).unwrap();
// }

// fn initialize_liquidity_rewards() {
// 	System::set_block_number(1);
// 	let acc_id: u128 = 2;
// 	let amount: u128 = std::u128::MAX;
// 	MockPromotedPoolApi::instance().lock().unwrap().clear();
// 	MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));
// 	XykStorage::create_new_token(&acc_id, amount);
// 	XykStorage::create_new_token(&acc_id, amount);
// 	XykStorage::create_new_token(&acc_id, amount);
// 	XykStorage::create_new_token(&acc_id, amount);

// 	XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 	XykStorage::activate_liquidity_v2(Origin::signed(2), 4, 10000, None).unwrap();
// }

// #[test]
// #[serial]
// fn liquidity_rewards_single_user_mint_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();

// 		// let (user_last_checkpoint, user_cummulative_ratio, user_missing_at_last_checkpoint) =
// 		// 	XykStorage::liquidity_mining_user_v2((2, 4));

// 		// assert_eq!(
// 		// 	XykStorage::liquidity_mining_user_v2((2, 4)),
// 		// 	(0, 0, U256::from_dec_str("10000").unwrap())
// 		// );

// 		let rewards_info = XykStorage::get_rewards_info(2, 4);

// 		assert_eq!(rewards_info.activated_amount, 10000);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		assert_eq!(rewards_info.last_checkpoint, 0);
// 		assert_eq!(rewards_info.pool_ratio_at_last_checkpoint, U256::from(0));
// 		assert_eq!(rewards_info.missing_at_last_checkpoint, U256::from_dec_str("10000").unwrap());

// 		System::set_block_number(10);
// 		MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 0);
// 		System::set_block_number(10);

// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX * 1));
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 291);
// 		System::set_block_number(20);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 2);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 873);
// 		System::set_block_number(30);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 3);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 1716);
// 		System::set_block_number(40);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 4);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 2847);
// 		System::set_block_number(50);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 5);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 4215);
// 		System::set_block_number(60);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 6);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 5844);
// 		System::set_block_number(70);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 7);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 7712);
// 		System::set_block_number(80);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 8);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 9817);
// 		System::set_block_number(90);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 9);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 12142);
// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 10);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 14704);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_three_users_mint_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(0, 2, 4, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 4, 1000000).unwrap();
// 		//
// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();

// 		let rewards_info = XykStorage::get_rewards_info(2, 4);
// 		assert_eq!(rewards_info.activated_amount, 10000);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		assert_eq!(rewards_info.last_checkpoint, 0);
// 		assert_eq!(rewards_info.pool_ratio_at_last_checkpoint, U256::from(0));
// 		assert_eq!(rewards_info.missing_at_last_checkpoint, U256::from_dec_str("10000").unwrap());

// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 100000 / 10000);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 14704);

// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 10000, 10010).unwrap();

// 		System::set_block_number(200);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 150000 / 10000);
// 		XykStorage::mint_liquidity(Origin::signed(4), 0, 1, 10000, 10010).unwrap();

// 		System::set_block_number(240);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 163300 / 10000);
// 		XykStorage::mint_liquidity(Origin::signed(4), 0, 1, 10000, 10010).unwrap();

// 		System::set_block_number(400);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 203300 / 10000);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 85820);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 35810);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(4, 4).unwrap(), 21647);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_three_users_burn_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(0, 2, 4, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 4, 1000000).unwrap();

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();

// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 100000 / 10000);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 14704);

// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 10000, 10010).unwrap();

// 		System::set_block_number(200);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 150000 / 10000);
// 		XykStorage::mint_liquidity(Origin::signed(4), 0, 1, 10000, 10010).unwrap();

// 		System::set_block_number(240);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 163300 / 10000);
// 		XykStorage::burn_liquidity(Origin::signed(4), 0, 1, 5000).unwrap();

// 		System::set_block_number(400);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 227300 / 10000);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 95951);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 44130);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(4, 4).unwrap(), 10628);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_claim_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();

// 		// assert_eq!(
// 		// 	XykStorage::liquidity_mining_user_v2((2, 4)),
// 		// 	(0, 0, U256::from_dec_str("10000").unwrap())
// 		// );

// 		System::set_block_number(10);
// 		MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));

// 		System::set_block_number(90);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 90000 / 10000);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 12142);
// 		XykStorage::claim_rewards_v2(Origin::signed(2), 4, 12141).unwrap();

// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 100000 / 10000);
// 		System::set_block_number(100);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 2563);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_promote_pool_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_pool(Origin::signed(2), 0, 5000, 1, 5000).unwrap();

// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_promote_pool_already_promoted_NW() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_pool(Origin::signed(2), 0, 5000, 1, 5000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();

// 		assert!(<Test as Config>::PoolPromoteApi::get_pool_rewards_v2(4).is_some());
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_claim_more_NW() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();
// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 100000 / 10000);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), (14704));

// 		assert_err!(
// 			XykStorage::claim_rewards_v2(Origin::signed(2), 4, 15000),
// 			Error::<Test>::NotEnoughtRewardsEarned,
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_work_after_burn_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(0, 2, 4, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 4, 1000000).unwrap();

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();

// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 100000 / 10000);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 14704);

// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 10000, 10010).unwrap();

// 		System::set_block_number(200);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 150000 / 10000);
// 		XykStorage::mint_liquidity(Origin::signed(4), 0, 1, 10000, 10010).unwrap();

// 		System::set_block_number(240);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 163300 / 10000);

// 		XykStorage::burn_liquidity(Origin::signed(4), 0, 1, 10000).unwrap();

// 		System::set_block_number(400);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 243300 / 10000);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(4, 4).unwrap(), 946);

// 		XykStorage::mint_liquidity(Origin::signed(4), 0, 1, 20000, 20010).unwrap();

// 		System::set_block_number(500);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 268300 / 10000);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(4, 4).unwrap(), 8297);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_deactivate_transfer_controled_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);

// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();
// 		assert_err!(XykStorage::transfer(4, 2, 3, 10), orml_tokens::Error::<Test>::BalanceTooLow,);

// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 100000 / 10000);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 14704);

// 		XykStorage::deactivate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned).unwrap();
// 		XykStorage::transfer(4, 2, 3, 10).unwrap();
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 14704);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_deactivate_more_NW() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();
// 		assert_err!(
// 			XykStorage::deactivate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned + 1),
// 			Error::<Test>::NotEnoughAssets
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_activate_more_NW() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		assert_err!(
// 			XykStorage::activate_liquidity_v2(
// 				Origin::signed(2),
// 				4,
// 				liquidity_tokens_owned + 1,
// 				None
// 			),
// 			Error::<Test>::NotEnoughAssets
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_calculate_rewards_pool_not_promoted() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 0);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_claim_pool_not_promoted() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		assert_err!(
// 			XykStorage::claim_rewards_v2(Origin::signed(2), 7, 5000000000),
// 			Error::<Test>::NotEnoughtRewardsEarned,
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_transfer_not_working() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();

// 		assert_err!(XykStorage::transfer(4, 2, 3, 10), orml_tokens::Error::<Test>::BalanceTooLow,);
// 	});
// }

// #[test]
// fn set_info_should_work() {
// 	new_test_ext().execute_with(|| {
// 		// creating asset with assetId 0 and minting to accountId 2
// 		let acc_id: u128 = 2;
// 		let amount: u128 = 1000000000000000000000;
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(
// 			Origin::signed(2),
// 			0,
// 			40000000000000000000,
// 			1,
// 			60000000000000000000,
// 		)
// 		.unwrap();

// 		assert_eq!(
// 			*MockAssetRegister::instance().lock().unwrap().get(&2u32).unwrap(),
// 			AssetMetadata {
// 				name: b"LiquidityPoolToken0x00000002".to_vec(),
// 				symbol: b"TKN0x00000000-TKN0x00000001".to_vec(),
// 				decimals: 18u32,
// 				location: None,
// 				additional: CustomMetadata::default(),
// 				existential_deposit: 0u128,
// 			}
// 		);
// 	});
// }

// #[test]
// fn set_info_should_work_with_small_numbers() {
// 	new_test_ext().execute_with(|| {
// 		// creating asset with assetId 0 and minting to accountId 2
// 		let acc_id: u128 = 2;
// 		let amount: u128 = 1000000000000000000000;
// 		const N: u32 = 12345u32;

// 		for _ in 0..N {
// 			XykStorage::create_new_token(&acc_id, amount);
// 		}

// 		XykStorage::create_pool(
// 			Origin::signed(2),
// 			15,
// 			40000000000000000000,
// 			12233,
// 			60000000000000000000,
// 		)
// 		.unwrap();

// 		assert_eq!(
// 			*MockAssetRegister::instance().lock().unwrap().get(&N).unwrap(),
// 			AssetMetadata {
// 				name: b"LiquidityPoolToken0x00003039".to_vec(),
// 				symbol: b"TKN0x0000000F-TKN0x00002FC9".to_vec(),
// 				decimals: 18u32,
// 				location: None,
// 				additional: CustomMetadata::default(),
// 				existential_deposit: 0u128,
// 			}
// 		);
// 	});
// }

// #[test]
// #[ignore]
// fn set_info_should_work_with_large_numbers() {
// 	new_test_ext().execute_with(|| {
// 		// creating asset with assetId 0 and minting to accountId 2
// 		let acc_id: u128 = 2;
// 		let amount: u128 = 1000000000000000000000;
// 		const N: u32 = 1524501234u32;

// 		for _ in 0..N {
// 			XykStorage::create_new_token(&acc_id, amount);
// 		}

// 		XykStorage::create_pool(
// 			Origin::signed(2),
// 			15000000,
// 			40000000000000000000,
// 			12233000,
// 			60000000000000000000,
// 		)
// 		.unwrap();

// 		assert_eq!(
// 			*MockAssetRegister::instance().lock().unwrap().get(&1524501234u32).unwrap(),
// 			AssetMetadata {
// 				name: b"LiquidityPoolToken0x5ADE0AF2".to_vec(),
// 				symbol: b"TKN0x00E4E1C0-TKN0x00BAA928".to_vec(),
// 				decimals: 18u32,
// 				location: None,
// 				additional: CustomMetadata::default(),
// 				existential_deposit: 0u128,
// 			}
// 		);
// 	});
// }

// #[test]
// fn buy_and_burn_sell_mangata() {
// 	new_test_ext().execute_with(|| {
// 		initialize_buy_and_burn();
// 		XykStorage::sell_asset(Origin::signed(2), 0, 1, 50000000000000, 0).unwrap();

// 		assert_eq!(XykStorage::asset_pool((0, 1)), (149949999999998, 66733400066734));
// 		assert_eq!(XykStorage::balance(0, 2), 850000000000000);
// 		assert_eq!(XykStorage::balance(1, 2), 833266599933266);
// 		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 149949999999998);
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 166733400066734);
// 		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 25000000001);
// 		assert_eq!(XykStorage::balance(1, XykStorage::treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(1, XykStorage::bnb_treasury_account_id()), 0);
// 	});
// }

// #[test]
// fn buy_and_burn_sell_has_mangata_pair() {
// 	new_test_ext().execute_with(|| {
// 		initialize_buy_and_burn();
// 		XykStorage::sell_asset(Origin::signed(2), 1, 4, 50000000000000, 0).unwrap();

// 		assert_eq!(XykStorage::asset_pool((0, 1)), (99950024987505, 100050000000002));
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (149949999999998, 66733400066734));
// 		assert_eq!(XykStorage::balance(1, 2), 750000000000000); // user acc: regular trade result
// 		assert_eq!(XykStorage::balance(4, 2), 933266599933266); // user acc: regular trade result
// 		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 99950024987505);
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 250000000000000);
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 66733400066734); // vault: regular trade result
// 		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 24987506247); // 24987506247 mangata in treasury
// 		assert_eq!(XykStorage::balance(1, XykStorage::treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(1, XykStorage::bnb_treasury_account_id()), 0);
// 	});
// }

// #[test]
// fn buy_and_burn_sell_none_have_mangata_pair() {
// 	new_test_ext().execute_with(|| {
// 		initialize_buy_and_burn();
// 		XykStorage::sell_asset(Origin::signed(2), 4, 1, 50000000000000, 0).unwrap();

// 		assert_eq!(XykStorage::asset_pool((0, 1)), (100000000000000, 100000000000000));
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (66733400066734, 149949999999998));
// 		assert_eq!(XykStorage::balance(1, 2), 833266599933266); // user acc: regular trade result
// 		assert_eq!(XykStorage::balance(4, 2), 850000000000000); // user acc: regular trade result
// 		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 100000000000000);
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 166733400066734);
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 149949999999998); // vault: regular trade result
// 		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 0); // 24987506247 mangata in treasury
// 		assert_eq!(XykStorage::balance(4, XykStorage::treasury_account_id()), 25000000001);
// 		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(4, XykStorage::bnb_treasury_account_id()), 25000000001);
// 	});
// }

// #[test]
// fn buy_and_burn_buy_where_sold_is_mangata() {
// 	new_test_ext().execute_with(|| {
// 		initialize_buy_and_burn();
// 		XykStorage::buy_asset(Origin::signed(2), 0, 1, 33266599933266, 50000000000001).unwrap();

// 		assert_eq!(XykStorage::asset_pool((0, 1)), (149949999999999, 66733400066734));
// 		assert_eq!(XykStorage::balance(0, 2), 850000000000001);
// 		assert_eq!(XykStorage::balance(1, 2), 833266599933266);

// 		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 149949999999999);
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 166733400066734);
// 		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 25000000000);
// 		assert_eq!(XykStorage::balance(1, XykStorage::treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(1, XykStorage::bnb_treasury_account_id()), 0);
// 	});
// }

// #[test]
// fn buy_and_burn_buy_where_sold_has_mangata_pair() {
// 	new_test_ext().execute_with(|| {
// 		initialize_buy_and_burn();
// 		XykStorage::buy_asset(Origin::signed(2), 1, 4, 33266599933266, 50000000000001).unwrap();

// 		assert_eq!(XykStorage::asset_pool((0, 1)), (99950024987507, 100050000000000));
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (149949999999999, 66733400066734));
// 		assert_eq!(XykStorage::balance(1, 2), 750000000000001); // user acc: regular trade result
// 		assert_eq!(XykStorage::balance(4, 2), 933266599933266); // user acc: regular trade result
// 		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 99950024987507);
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 249999999999999);
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 66733400066734); // vault: regular trade result
// 		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 24987506246); // 24987506247 mangata in treasury
// 		assert_eq!(XykStorage::balance(1, XykStorage::treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(1, XykStorage::bnb_treasury_account_id()), 0);
// 	});
// }

// #[test]
// fn buy_and_burn_buy_none_have_mangata_pair() {
// 	new_test_ext().execute_with(|| {
// 		initialize_buy_and_burn();
// 		XykStorage::buy_asset(Origin::signed(2), 4, 1, 33266599933266, 50000000000001).unwrap();

// 		assert_eq!(XykStorage::asset_pool((0, 1)), (100000000000000, 100000000000000));
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (66733400066734, 149949999999999));
// 		assert_eq!(XykStorage::balance(1, 2), 833266599933266); // user acc: regular trade result
// 		assert_eq!(XykStorage::balance(4, 2), 850000000000001); // user acc: regular trade result
// 		assert_eq!(XykStorage::balance(0, XykStorage::account_id()), 100000000000000);
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 166733400066734);
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 149949999999999); // vault: regular trade result
// 		assert_eq!(XykStorage::balance(0, XykStorage::treasury_account_id()), 0); // 24987506247 mangata in treasury
// 		assert_eq!(XykStorage::balance(4, XykStorage::treasury_account_id()), 25000000000);
// 		assert_eq!(XykStorage::balance(0, XykStorage::bnb_treasury_account_id()), 0);
// 		assert_eq!(XykStorage::balance(4, XykStorage::bnb_treasury_account_id()), 25000000000);
// 	});
// }

// #[test]
// fn multi() {
// 	new_test_ext().execute_with(|| {
// 		let acc_id: u128 = 2;
// 		let amount: u128 = 2000000000000000000000000;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_pool(
// 			Origin::signed(2),
// 			1,
// 			1000000000000000000000000,
// 			4,
// 			500000000000000000000000,
// 		)
// 		.unwrap();
// 		assert_eq!(
// 			XykStorage::asset_pool((1, 4)),
// 			(1000000000000000000000000, 500000000000000000000000)
// 		);
// 		assert_eq!(XykStorage::liquidity_asset((1, 4)), Some(5)); // liquidity assetId corresponding to newly created pool
// 		assert_eq!(XykStorage::liquidity_pool(5), Some((1, 4))); // liquidity assetId corresponding to newly created pool
// 		assert_eq!(XykStorage::total_supply(5), 750000000000000000000000); // total liquidity assets
// 		assert_eq!(XykStorage::balance(5, 2), 750000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, 2), 1000000000000000000000000); // amount of asset 1 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(4, 2), 1500000000000000000000000); // amount of asset 2 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 1000000000000000000000000); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 500000000000000000000000); // amount of asset 1 in vault acc after creating pool

// 		XykStorage::mint_liquidity(
// 			Origin::signed(2),
// 			1,
// 			4,
// 			500000000000000000000000,
// 			5000000000000000000000000,
// 		)
// 		.unwrap();

// 		assert_eq!(
// 			XykStorage::asset_pool((1, 4)),
// 			(1500000000000000000000000, 750000000000000000000001)
// 		);
// 		assert_eq!(XykStorage::total_supply(5), 1125000000000000000000000); // total liquidity assets
// 		assert_eq!(XykStorage::balance(5, 2), 1125000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, 2), 500000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(4, 2), 1249999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 1500000000000000000000000); // amount of asset 1 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 750000000000000000000001); // amount of asset 2 in vault acc after creating pool

// 		XykStorage::burn_liquidity(Origin::signed(2), 1, 4, 225000000000000000000000).unwrap();

// 		assert_eq!(
// 			XykStorage::asset_pool((1, 4)),
// 			(1200000000000000000000000, 600000000000000000000001)
// 		);
// 		assert_eq!(XykStorage::total_supply(5), 900000000000000000000000); // total liquidity assets
// 		assert_eq!(XykStorage::balance(5, 2), 900000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, 2), 800000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(4, 2), 1399999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 1200000000000000000000000); // amount of asset 1 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 600000000000000000000001); // amount of asset 2 in vault acc after creating pool

// 		XykStorage::burn_liquidity(Origin::signed(2), 1, 4, 225000000000000000000000).unwrap();

// 		assert_eq!(
// 			XykStorage::asset_pool((1, 4)),
// 			(900000000000000000000000, 450000000000000000000001)
// 		);
// 		assert_eq!(XykStorage::total_supply(5), 675000000000000000000000); // total liquidity assets
// 		assert_eq!(XykStorage::balance(5, 2), 675000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, 2), 1100000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(4, 2), 1549999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 900000000000000000000000); // amount of asset 1 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 450000000000000000000001); // amount of asset 2 in vault acc after creating pool

// 		XykStorage::mint_liquidity(
// 			Origin::signed(2),
// 			1,
// 			4,
// 			1000000000000000000000000,
// 			10000000000000000000000000,
// 		)
// 		.unwrap();

// 		assert_eq!(
// 			XykStorage::asset_pool((1, 4)),
// 			(1900000000000000000000000, 950000000000000000000003)
// 		);
// 		assert_eq!(XykStorage::total_supply(5), 1425000000000000000000000); // total liquidity assets
// 		assert_eq!(XykStorage::balance(5, 2), 1425000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, 2), 100000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(4, 2), 1049999999999999999999997); // amount of asset 1 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 1900000000000000000000000); // amount of asset 1 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 950000000000000000000003);
// 		// amount of asset 0 in vault acc after creating pool
// 	});
// }

// #[test]
// fn create_pool_W() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_eq!(XykStorage::asset_pool((1, 4)), (40000000000000000000, 60000000000000000000));
// 		assert_eq!(XykStorage::liquidity_asset((1, 4)), Some(5)); // liquidity assetId corresponding to newly created pool
// 		assert_eq!(XykStorage::liquidity_pool(5), Some((1, 4))); // liquidity assetId corresponding to newly created pool
// 		assert_eq!(XykStorage::total_supply(5), 50000000000000000000); // total liquidity assets
// 		assert_eq!(XykStorage::balance(5, 2), 50000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, 2), 960000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(4, 2), 940000000000000000000); // amount of asset 1 in user acc after creating pool / initial minting
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 40000000000000000000); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 60000000000000000000);
// 		// amount of asset 1 in vault acc after creating pool
// 	});
// }

// #[test]
// fn create_pool_N_already_exists() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::create_pool(Origin::signed(2), 1, 500000, 4, 500000,),
// 			Error::<Test>::PoolAlreadyExists,
// 		);
// 	});
// }

// #[test]
// fn create_pool_N_already_exists_other_way() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::create_pool(Origin::signed(2), 4, 500000, 1, 500000,),
// 			Error::<Test>::PoolAlreadyExists,
// 		);
// 	});
// }

// #[test]
// fn create_pool_N_not_enough_first_asset() {
// 	new_test_ext().execute_with(|| {
// 		let acc_id: u128 = 2;
// 		let amount: u128 = 1000000;
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		assert_err!(
// 			XykStorage::create_pool(Origin::signed(2), 0, 1500000, 1, 500000,),
// 			Error::<Test>::NotEnoughAssets,
// 		); //asset 0 issued to user 1000000, trying to create pool using 1500000
// 	});
// }

// #[test]
// fn create_pool_N_not_enough_second_asset() {
// 	new_test_ext().execute_with(|| {
// 		let acc_id: u128 = 2;
// 		let amount: u128 = 1000000;
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		assert_err!(
// 			XykStorage::create_pool(Origin::signed(2), 0, 500000, 1, 1500000,),
// 			Error::<Test>::NotEnoughAssets,
// 		); //asset 1 issued to user 1000000, trying to create pool using 1500000
// 	});
// }

// #[test]
// fn create_pool_N_same_asset() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::create_pool(Origin::signed(2), 0, 500000, 0, 500000,),
// 			Error::<Test>::SameAsset,
// 		);
// 	});
// }

// #[test]
// fn create_pool_N_zero_first_amount() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::create_pool(Origin::signed(2), 0, 0, 1, 500000,),
// 			Error::<Test>::ZeroAmount,
// 		);
// 	});
// }

// #[test]
// fn create_pool_N_zero_second_amount() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::create_pool(Origin::signed(2), 0, 500000, 1, 0,),
// 			Error::<Test>::ZeroAmount,
// 		);
// 	});
// }

// #[test]
// fn sell_W() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
// 		initialize();
// 		XykStorage::sell_asset(Origin::signed(2), 1, 4, 20000000000000000000, 0).unwrap(); // selling 20000000000000000000 assetId 0 of pool 0 1

// 		assert_eq!(XykStorage::balance(1, 2), 940000000000000000000); // amount in user acc after selling
// 		assert_eq!(XykStorage::balance(4, 2), 959959959959959959959); // amount in user acc after buying
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (59979999999999999998, 40040040040040040041)); // amount of asset 0 in pool map
// 																						  //   assert_eq!(XykStorage::asset_pool2((1, 0)), 40040040040040040041); // amount of asset 1 in pool map
// 		assert_eq!(XykStorage::balance(1, 2), 940000000000000000000); // amount of asset 0 on account 2
// 		assert_eq!(XykStorage::balance(4, 2), 959959959959959959959); // amount of asset 1 on account 2
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 59979999999999999998); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 40040040040040040041); // amount of asset 1 in vault acc after creating pool

// 		let assets_swapped_event =
// 			crate::mock::Event::XykStorage(crate::Event::<Test>::AssetsSwapped(
// 				2,
// 				1,
// 				20000000000000000000,
// 				4,
// 				19959959959959959959,
// 			));

// 		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
// 	});
// }

// #[test]
// fn sell_W_other_way() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		XykStorage::sell_asset(Origin::signed(2), 4, 1, 30000000000000000000, 0).unwrap(); // selling 30000000000000000000 assetId 1 of pool 0 1

// 		assert_eq!(XykStorage::balance(1, 2), 973306639973306639973); // amount of asset 0 in user acc after selling
// 		assert_eq!(XykStorage::balance(4, 2), 910000000000000000000); // amount of asset 1 in user acc after buying
// 															  // assert_eq!(XykStorage::asset_pool((1, 2)), 26684462240017795575); // amount of asset 0 in pool map
// 															  // assert_eq!(XykStorage::asset_pool((1, 0)), 90000000000000000000); // amount of asset 1 in pool map
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (26693360026693360027, 89969999999999999998));
// 		assert_eq!(XykStorage::balance(1, 2), 973306639973306639973); // amount of asset 0 on account 2
// 		assert_eq!(XykStorage::balance(4, 2), 910000000000000000000); // amount of asset 1 on account 2
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 26693360026693360027); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 89969999999999999998);
// 		// amount of asset 1 in vault acc after creating pool
// 	});
// }

// #[test]
// fn sell_N_no_such_pool() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::sell_asset(Origin::signed(2), 0, 10, 250000, 0),
// 			Error::<Test>::NoSuchPool,
// 		); // selling 250000 assetId 0 of pool 0 10 (only pool 0 1 exists)
// 	});
// }
// #[test]
// fn sell_N_not_enough_selling_assset() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::sell_asset(Origin::signed(2), 1, 4, 1000000000000000000000, 0),
// 			Error::<Test>::NotEnoughAssets,
// 		); // selling 1000000000000000000000 assetId 0 of pool 0 1 (user has only 960000000000000000000)
// 	});
// }

// #[test]
// fn sell_N_insufficient_output_amount() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::sell_asset(Origin::signed(2), 1, 4, 250000, 500000),
// 			Error::<Test>::InsufficientOutputAmount,
// 		); // selling 250000 assetId 0 of pool 0 1, by the formula user should get 166333 asset 1, but is requesting 500000
// 	});
// }

// #[test]
// fn sell_N_zero_amount() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::sell_asset(Origin::signed(2), 1, 4, 0, 500000),
// 			Error::<Test>::ZeroAmount,
// 		); // selling 0 assetId 0 of pool 0 1
// 	});
// }

// #[test]
// fn buy_W() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
// 		initialize();
// 		// buying 30000000000000000000 assetId 1 of pool 0 1
// 		XykStorage::buy_asset(
// 			Origin::signed(2),
// 			1,
// 			4,
// 			30000000000000000000,
// 			3000000000000000000000,
// 		)
// 		.unwrap();
// 		assert_eq!(XykStorage::balance(1, 2), 919879638916750250752); // amount in user acc after selling
// 		assert_eq!(XykStorage::balance(4, 2), 970000000000000000000); // amount in user acc after buying
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (80080240722166499498, 30000000000000000000));
// 		assert_eq!(XykStorage::balance(1, 2), 919879638916750250752); // amount of asset 0 on account 2
// 		assert_eq!(XykStorage::balance(4, 2), 970000000000000000000); // amount of asset 1 on account 2
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 80080240722166499498); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 30000000000000000000); // amount of asset 1 in vault acc after creating pool

// 		let assets_swapped_event =
// 			crate::mock::Event::XykStorage(crate::Event::<Test>::AssetsSwapped(
// 				2,
// 				1,
// 				40120361083249749248,
// 				4,
// 				30000000000000000000,
// 			));

// 		assert!(System::events().iter().any(|record| record.event == assets_swapped_event));
// 	});
// }

// #[test]
// fn buy_W_other_way() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		// buying 30000000000000000000 assetId 0 of pool 0 1
// 		XykStorage::buy_asset(
// 			Origin::signed(2),
// 			4,
// 			1,
// 			30000000000000000000,
// 			3000000000000000000000,
// 		)
// 		.unwrap();
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 10000000000000000000); // amount of asset 1 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 240361083249749247743); // amount of asset 2 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(1, 2), 990000000000000000000); // amount in user acc after selling
// 		assert_eq!(XykStorage::balance(4, 2), 759458375125376128385); // amount in user acc after buying
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (10000000000000000000, 240361083249749247743));
// 		assert_eq!(XykStorage::balance(1, 2), 990000000000000000000); // amount of asset 0 on account 2
// 		assert_eq!(XykStorage::balance(4, 2), 759458375125376128385); // amount of asset 1 on account 2
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 10000000000000000000); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 240361083249749247743);
// 		// amount of asset 1 in vault acc after creating pool
// 	});
// }

// #[test]
// fn buy_N_no_such_pool() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		// buying 150000 assetId 1 of pool 0 10 (only pool 0 1 exists)
// 		assert_err!(
// 			XykStorage::buy_asset(Origin::signed(2), 0, 10, 150000, 5000000),
// 			Error::<Test>::NoSuchPool,
// 		);
// 	});
// }

// #[test]
// fn buy_N_not_enough_reserve() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		// buying 70000000000000000000 assetId 0 of pool 0 1 , only 60000000000000000000 in reserve
// 		assert_err!(
// 			XykStorage::buy_asset(
// 				Origin::signed(2),
// 				1,
// 				4,
// 				70000000000000000000,
// 				5000000000000000000000
// 			),
// 			Error::<Test>::NotEnoughReserve,
// 		);
// 	});
// }

// #[test]
// fn buy_N_not_enough_selling_assset() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		// buying 59000000000000000000 assetId 1 of pool 0 1 should sell 2.36E+21 assetId 0, only 9.6E+20 in acc
// 		assert_err!(
// 			XykStorage::buy_asset(
// 				Origin::signed(2),
// 				1,
// 				4,
// 				59000000000000000000,
// 				59000000000000000000000
// 			),
// 			Error::<Test>::NotEnoughAssets,
// 		);
// 	});
// }

// #[test]
// fn buy_N_insufficient_input_amount() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		// buying 150000 liquidity assetId 1 of pool 0 1
// 		assert_err!(
// 			XykStorage::buy_asset(Origin::signed(2), 1, 4, 150000, 10),
// 			Error::<Test>::InsufficientInputAmount,
// 		);
// 	});
// }

// #[test]
// fn buy_N_zero_amount() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		assert_err!(
// 			XykStorage::buy_asset(Origin::signed(2), 1, 4, 0, 0),
// 			Error::<Test>::ZeroAmount,
// 		); // buying 0 assetId 0 of pool 0 1
// 	});
// }

// #[test]
// fn mint_W() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		// minting pool 0 1 with 20000000000000000000 assetId 0
// 		XykStorage::mint_liquidity(
// 			Origin::signed(2),
// 			1,
// 			4,
// 			20000000000000000000,
// 			30000000000000000001,
// 		)
// 		.unwrap();

// 		assert_eq!(XykStorage::total_supply(5), 75000000000000000000); // total liquidity assets
// 		assert_eq!(XykStorage::balance(5, 2), 75000000000000000000); // amount of liquidity assets owned by user by creating pool and minting
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (60000000000000000000, 90000000000000000001));
// 		assert_eq!(XykStorage::balance(1, 2), 940000000000000000000); // amount of asset 1 in user acc after minting
// 		assert_eq!(XykStorage::balance(4, 2), 909999999999999999999); // amount of asset 2 in user acc after minting
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 60000000000000000000); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 90000000000000000001); // amount of asset 1 in vault acc after creating pool
// 		let liquidity_minted_event =
// 			crate::mock::Event::XykStorage(crate::Event::<Test>::LiquidityMinted(
// 				2,
// 				1,
// 				20000000000000000000,
// 				4,
// 				30000000000000000001,
// 				5,
// 				25000000000000000000,
// 			));

// 		assert!(System::events().iter().any(|record| record.event == liquidity_minted_event));
// 	});
// }

// #[test]
// fn mint_W_other_way() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		// minting pool 0 1 with 30000000000000000000 assetId 1
// 		XykStorage::mint_liquidity(
// 			Origin::signed(2),
// 			4,
// 			1,
// 			30000000000000000000,
// 			300000000000000000000,
// 		)
// 		.unwrap();

// 		assert_eq!(XykStorage::total_supply(5), 75000000000000000000); // total liquidity assets
// 		assert_eq!(XykStorage::balance(5, 2), 75000000000000000000); // amount of liquidity assets owned by user by creating pool and minting
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (60000000000000000001, 90000000000000000000));
// 		assert_eq!(XykStorage::balance(1, 2), 939999999999999999999); // amount of asset 0 in user acc after minting
// 		assert_eq!(XykStorage::balance(4, 2), 910000000000000000000); // amount of asset 1 in user acc after minting
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 60000000000000000001); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 90000000000000000000);
// 		// amount of asset 1 in vault acc after creating pool
// 	});
// }

// #[test]
// fn mint_N_no_such_pool() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		assert_err!(
// 			XykStorage::mint_liquidity(Origin::signed(2), 0, 10, 250000, 250000),
// 			Error::<Test>::NoSuchPool,
// 		); // minting pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
// 	});
// }

// #[test]
// fn mint_N_not_enough_first_asset() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		assert_err!(
// 			XykStorage::mint_liquidity(
// 				Origin::signed(2),
// 				1,
// 				4,
// 				1000000000000000000000,
// 				10000000000000000000000
// 			),
// 			Error::<Test>::NotEnoughAssets,
// 		); // minting pool 0 1 with 1000000000000000000000 assetId 0 (user has only 960000000000000000000)
// 	});
// }

// #[test]
// fn mint_N_not_enough_second_asset() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		assert_err!(
// 			XykStorage::mint_liquidity(
// 				Origin::signed(2),
// 				4,
// 				1,
// 				1000000000000000000000,
// 				10000000000000000000000,
// 			),
// 			Error::<Test>::NotEnoughAssets,
// 		); // minting pool 0 1 with 1000000000000000000000 assetId 1 (user has only 940000000000000000000)
// 	});
// }

// #[test]
// fn min_N_zero_amount() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		assert_err!(
// 			XykStorage::mint_liquidity(Origin::signed(2), 1, 4, 0, 10),
// 			Error::<Test>::ZeroAmount,
// 		); // minting pool 0 1 with 0 assetId 1
// 	});
// }

// #[test]
// fn mint_N_second_asset_amount_exceeded_expectations() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		assert_err!(
// 			XykStorage::mint_liquidity(Origin::signed(2), 1, 4, 250000, 10),
// 			Error::<Test>::SecondAssetAmountExceededExpectations,
// 		); // minting pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
// 	});
// }

// #[test]
// fn burn_W() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		XykStorage::burn_liquidity(Origin::signed(2), 1, 4, 25000000000000000000).unwrap(); // burning 20000000000000000000 asset 0 of pool 0 1

// 		assert_eq!(XykStorage::balance(5, 2), 25000000000000000000); // amount of liquidity assets owned by user by creating pool and burning
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (20000000000000000000, 30000000000000000000));
// 		assert_eq!(XykStorage::balance(1, 2), 980000000000000000000); // amount of asset 0 in user acc after burning
// 		assert_eq!(XykStorage::balance(4, 2), 970000000000000000000); // amount of asset 1 in user acc after burning
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 20000000000000000000); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 30000000000000000000); // amount of asset 1 in vault acc after creating pool

// 		let liquidity_burned =
// 			crate::mock::Event::XykStorage(crate::Event::<Test>::LiquidityBurned(
// 				2,
// 				1,
// 				20000000000000000000,
// 				4,
// 				30000000000000000000,
// 				5,
// 				25000000000000000000,
// 			));

// 		assert!(System::events().iter().any(|record| record.event == liquidity_burned));
// 	});
// }

// #[test]
// fn burn_W_other_way() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		XykStorage::burn_liquidity(Origin::signed(2), 4, 1, 25000000000000000000).unwrap(); // burning 30000000000000000000 asset 1 of pool 0 1

// 		assert_eq!(XykStorage::balance(5, 2), 25000000000000000000); // amount of liquidity assets owned by user by creating pool and burning
// 		assert_eq!(XykStorage::asset_pool((1, 4)), (20000000000000000000, 30000000000000000000));
// 		assert_eq!(XykStorage::balance(1, 2), 980000000000000000000); // amount of asset 0 in user acc after burning
// 		assert_eq!(XykStorage::balance(4, 2), 970000000000000000000); // amount of asset 1 in user acc after burning
// 		assert_eq!(XykStorage::balance(1, XykStorage::account_id()), 20000000000000000000); // amount of asset 0 in vault acc after creating pool
// 		assert_eq!(XykStorage::balance(4, XykStorage::account_id()), 30000000000000000000);
// 		// amount of asset 1 in vault acc after creating pool
// 	});
// }

// #[test]
// fn burn_N_not_enough_liquidity_asset() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		// burning pool 0 1 with 500000000000000000000 liquidity asset amount (user has only 100000000000000000000 liquidity asset amount)
// 		assert_err!(
// 			XykStorage::burn_liquidity(Origin::signed(2), 1, 4, 500000000000000000000,),
// 			Error::<Test>::NotEnoughAssets,
// 		);
// 	});
// }

// #[test]
// fn burn_N_no_such_pool() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		// burning pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
// 		assert_err!(
// 			XykStorage::burn_liquidity(Origin::signed(2), 0, 10, 250000,),
// 			Error::<Test>::NoSuchPool,
// 		);
// 	});
// }

// #[test]
// fn burn_N_zero_amount() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		assert_err!(
// 			XykStorage::burn_liquidity(Origin::signed(2), 1, 4, 0,),
// 			Error::<Test>::ZeroAmount,
// 		); // burning pool 0 1 with 0 assetId 1
// 	});
// }

// // TODO https://trello.com/c/rEygIR7t/428-fix-panic-in-xyksellasset
// #[test]
// #[ignore]
// fn buy_assets_with_small_expected_amount_does_not_cause_panic() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		let first_token_balance = XykStorage::balance(1, DUMMY_USER_ID);
// 		XykStorage::buy_asset(Origin::signed(2), 1, 4, 1, first_token_balance).unwrap();
// 	});
// }

// #[test]
// #[ignore]
// fn successful_buy_assets_does_not_charge_fee() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		let first_token_balance = XykStorage::balance(1, DUMMY_USER_ID);
// 		let post_info =
// 			XykStorage::buy_asset(Origin::signed(2), 1, 4, 1000, first_token_balance).unwrap();
// 		assert_eq!(post_info.pays_fee, Pays::No);
// 	});
// }

// #[test]
// #[ignore]
// fn unsuccessful_buy_assets_charges_fee() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
// 		//try to sell non owned, non existing tokens
// 		let post_info =
// 			XykStorage::buy_asset(Origin::signed(2), 100, 200, 0, 0).unwrap_err().post_info;
// 		assert_eq!(post_info.pays_fee, Pays::Yes);
// 	});
// }

// #[test]
// #[ignore]
// fn successful_sell_assets_does_not_charge_fee() {
// 	new_test_ext().execute_with(|| {
// 		initialize();
// 		let first_token_balance = XykStorage::balance(1, DUMMY_USER_ID);
// 		let post_info =
// 			XykStorage::sell_asset(Origin::signed(2), 1, 4, first_token_balance, 0).unwrap();
// 		assert_eq!(post_info.pays_fee, Pays::No);
// 	});
// }

// #[test]
// fn unsuccessful_sell_assets_charges_fee() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
// 		//try to sell non owned, non existing tokens
// 		let post_info =
// 			XykStorage::sell_asset(Origin::signed(2), 100, 200, 0, 0).unwrap_err().post_info;
// 		assert_eq!(post_info.pays_fee, Pays::Yes);
// 	});
// }

// #[test]
// fn PoolCreateApi_test_pool_exists_return_false_for_non_existing_pool() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
// 		assert!(!<XykStorage as PoolCreateApi>::pool_exists(1_u32.into(), 4_u32.into()));
// 	});
// }

// #[test]
// fn PoolCreateApi_pool_exists_return_true_for_existing_pool() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		XykStorage::create_pool(Origin::signed(2), 0, 500000, 1, 10000).unwrap();
// 		assert!(<XykStorage as PoolCreateApi>::pool_exists(0_u32.into(), 1_u32.into()));
// 	});
// }

// #[test]
// fn PoolCreateApi_pool_create_creates_a_pool() {
// 	new_test_ext().execute_with(|| {
// 		initialize();

// 		let first_asset_id = 0_u32;
// 		let first_asset_amount = 10_000_u128;
// 		let second_asset_id = 1_u32;
// 		let second_asset_amount = 5_000_u128;
// 		assert!(!<XykStorage as PoolCreateApi>::pool_exists(
// 			first_asset_id.into(),
// 			second_asset_id.into()
// 		));

// 		let liq_token_id = Tokens::next_asset_id();
// 		let liq_token_amount = (first_asset_amount + second_asset_amount) / 2;

// 		assert_eq!(
// 			<XykStorage as PoolCreateApi>::pool_create(
// 				DUMMY_USER_ID.into(),
// 				first_asset_id.into(),
// 				first_asset_amount,
// 				second_asset_id.into(),
// 				second_asset_amount
// 			),
// 			Some((liq_token_id, liq_token_amount))
// 		);

// 		assert_ne!(liq_token_id, Tokens::next_asset_id());
// 		assert_eq!(liq_token_amount, XykStorage::balance(liq_token_id, DUMMY_USER_ID.into()));

// 		assert!(<XykStorage as PoolCreateApi>::pool_exists(0_u32.into(), 1_u32.into()));
// 	});
// }

// #[test]
// fn test_create_blacklisted_pool() {
// 	new_test_ext().execute_with(|| {
// 		let blaclisted_first_asset_id = 1;
// 		let blaclisted_second_asset_id = 9;

// 		assert_err!(
// 			XykStorage::create_pool(
// 				Origin::signed(2),
// 				blaclisted_first_asset_id,
// 				100000000000000,
// 				blaclisted_second_asset_id,
// 				100000000000000
// 			),
// 			Error::<Test>::DisallowedPool
// 		);

// 		assert_err!(
// 			XykStorage::create_pool(
// 				Origin::signed(2),
// 				blaclisted_second_asset_id,
// 				100000000000000,
// 				blaclisted_first_asset_id,
// 				100000000000000
// 			),
// 			Error::<Test>::DisallowedPool
// 		);
// 	});
// }

// //E2E test cases
// //------------------------------------------------------------------
// // [Happy path] A user obtain extra tokens from asymtotic_curve_rewards
// //	Contained in liquidity_rewards_single_user_mint_W

// // [Accuracy] A user obtain the right tokens from asymtotic_curve_rewards
// //	Contained in liquidity_rewards_single_user_mint_W

// // [Accuracy] A user obtain the right tokens from asymtotic_curve_rewards when burn half of those
// // Contained in liquidity_rewards_work_after_burn_W

// // A user that mints two times, at different period gets the sum of those two kinds of asymtotic_curve_rewards
// // Contained in liquidity_rewards_three_users_mint_W

// // A user that got transfered Liq.tokens, can request asymtotic_curve_rewards
// #[test]
// #[serial]
// fn liquidity_rewards_transfered_liq_tokens_produce_rewards_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();

// 		MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);

// 		XykStorage::transfer(4, 2, 3, liquidity_tokens_owned).unwrap();

// 		XykStorage::activate_liquidity_v2(Origin::signed(3), 4, liquidity_tokens_owned, None)
// 			.unwrap();

// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 100000 / 10000);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 14704);
// 		XykStorage::claim_rewards_v2(Origin::signed(3), 4, 14704).unwrap();
// 	});
// }

// // A user that moved the Liq.tokens to the derived account , can request asymtotic_curve_rewards
// // ??

// // A user that bonded Liq.tokens can request asymtotic_curve_rewards
// // user that activated, contained in all tests

// // Asymtotic_curve_rewards are divided between the people that provided liquidity in the same pool ???
// // Contained in liquidity_rewards_three_users_mint_W

// // [Accuracy] A user obtain the right tokens from asymtotic_curve_rewards: Burning and minting during time
// // Contained in liquidity_rewards_work_after_burn_W

// // Asymtotic_curve_rewards are given for destroyed pools
// // No unpromote pool fn

// // Snaphshot is done when a pool is minted or burned
// // Contained in all tests, snapshots are no longer done for pool

// // Claim individuals is the same as claim in group
// // Contained in liquidity_rewards_claim_W, user is after claiming is getting the claimed amount less, not touching reward amount of others

// // Claim all the tokens that the user should have
// // Contained in liquidity_rewards_claim_W

// // Claim more tokens that the user should have
// // Contained in liquidity_rewards_claim_more_NW

// // Increasing pool size during <period> triggers checkpoint and the division of rewards is right
// // Contained in all tests, snapshots are no longer done for pool

// // Decreasing pool size during <period> triggers checkpoint and the division of rewards is right
// // Contained in all tests, snapshots are no longer done for pool

// // Token status:  When activate the tokens are in reserved.
// // Contained in liquidity_rewards_transfer_not_working

// // Token status:  When deActivate the tokens are in Free.
// // Contained in liquidity_rewards_transfered_liq_tokens_produce_rewards_W

// // Fees are in toBeClaimed when deactivate and  notYetClaimed when liquidity is activated

// #[test]
// #[serial]
// fn liquidity_rewards_not_yet_claimed_already_claimed_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();

// 		System::set_block_number(10);

// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 10000 / 10000);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 291);
// 		XykStorage::deactivate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned).unwrap();

// 		let rewards_info = XykStorage::get_rewards_info(2, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 291);

// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, liquidity_tokens_owned, None)
// 			.unwrap();
// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * 100000 / 10000);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 12433);
// 		XykStorage::claim_rewards_v2(Origin::signed(2), 4, 12432).unwrap();

// 		let rewards_info = XykStorage::get_rewards_info(2, 4);
// 		assert_eq!(rewards_info.rewards_already_claimed, 12141);
// 	});
// }

// // Curve is the same if user claim rewards or not.
// // Contained in liquidity_rewards_claim_W, user is after claiming is getting the claimed amount less, not touching reward amount of others

// //test for extreme values inside calculate rewards, mainly pool ratio
// #[test]
// #[serial]
// fn extreme_case_pool_ratio() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::create_pool(Origin::signed(2), 0, max, 1, max).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();

// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, 1, None).unwrap();

// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(u128::MAX));

// 		System::set_block_number(10000);
// 		assert_eq!(
// 			XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(),
// 			329053048812547494169083245386519860476
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn rewards_rounding_during_often_mint() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();
// 		XykStorage::create_pool(
// 			Origin::signed(2),
// 			0,
// 			250000000000000000000000000,
// 			1,
// 			10000000000000000,
// 		)
// 		.unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(0));
// 		XykStorage::transfer(0, 2, 3, 10000000000000000000000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 10000000000000000000000000).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(2), 0, 1, 25000000000000000000000, 2000000000000)
// 			.unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 25000000000000000000000, 2000000000000)
// 			.unwrap();

// 		let mut non_minter_higher_rewards_counter = 0;
// 		let mut higher_rewards_cumulative = 0;
// 		for n in 1..10000 {
// 			System::set_block_number(n);
// 			if (n + 1) % 10 == 0 {
// 				MockPromotedPoolApi::instance()
// 					.lock()
// 					.unwrap()
// 					.insert(4, U256::from(u128::MAX) * U256::from(n + 1));
// 				XykStorage::mint_liquidity(
// 					Origin::signed(3),
// 					0,
// 					1,
// 					34000000000000000000,
// 					68000000000000000000,
// 				)
// 				.unwrap();
// 				log::info!("----------------------------");
// 				let rew_non_minter = XykStorage::calculate_rewards_amount_v2(2, 4).unwrap();
// 				let rew_minter = XykStorage::calculate_rewards_amount_v2(3, 4).unwrap();
// 				log::info!("rew        {} {}", n, rew_non_minter);
// 				log::info!("rew minter {} {}", n, rew_minter);

// 				if rew_non_minter > rew_minter {
// 					non_minter_higher_rewards_counter = non_minter_higher_rewards_counter + 1;
// 					higher_rewards_cumulative =
// 						rew_minter * 10000 / rew_non_minter + higher_rewards_cumulative;
// 				}
// 			}
// 		}
// 		log::info!(
// 			"times minting user had lower rewards {}   avg minter/nonminter * 10000  {}",
// 			non_minter_higher_rewards_counter,
// 			higher_rewards_cumulative / non_minter_higher_rewards_counter
// 		);
// 	});
// }

// // starting point, user claimed some rewards, then new rewards were generated (blue)
// #[test]
// #[serial]
// fn rewards_storage_right_amounts_start1() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 1, 10000, 2, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(0));

// 		XykStorage::transfer(1, 2, 3, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 3, 20010).unwrap();
// 		XykStorage::transfer(1, 2, 4, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 4, 20010).unwrap();
// 		XykStorage::transfer(1, 2, 5, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 5, 20010).unwrap();
// 		XykStorage::transfer(1, 2, 6, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 6, 20010).unwrap();
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, 10000, None).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(3), 1, 2, 10000, 10010).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(4), 1, 2, 10000, 10010).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(5), 1, 2, 10000, 10010).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(6), 1, 2, 10000, 10010).unwrap();

// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(10));

// 		XykStorage::claim_rewards_all_v2(Origin::signed(2), 4).unwrap();
// 		XykStorage::claim_rewards_all_v2(Origin::signed(3), 4).unwrap();
// 		XykStorage::claim_rewards_all_v2(Origin::signed(4), 4).unwrap();
// 		XykStorage::claim_rewards_all_v2(Origin::signed(5), 4).unwrap();
// 		XykStorage::claim_rewards_all_v2(Origin::signed(6), 4).unwrap();

// 		let mut rewards_info = XykStorage::get_rewards_info(2, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 14704);
// 		rewards_info = XykStorage::get_rewards_info(3, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 14704);
// 		rewards_info = XykStorage::get_rewards_info(4, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 14704);
// 		rewards_info = XykStorage::get_rewards_info(5, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 14704);
// 		rewards_info = XykStorage::get_rewards_info(6, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 14704);

// 		System::set_block_number(200);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(20));

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 36530);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 36530);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(4, 4).unwrap(), 36530);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(5, 4).unwrap(), 36530);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(6, 4).unwrap(), 36530);

// 		// starting point for blue cases

// 		// usecase 3 claim (all)
// 		let mut user_balance_before = XykStorage::balance(0, 2);
// 		XykStorage::claim_rewards_v2(Origin::signed(2), 4, 36530).unwrap();
// 		let mut user_balance_after = XykStorage::balance(0, 2);
// 		rewards_info = XykStorage::get_rewards_info(2, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 51234);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 0);
// 		assert_eq!(user_balance_after - user_balance_before, 36530);

// 		// usecase 6 burn some
// 		user_balance_before = XykStorage::balance(0, 3);
// 		XykStorage::burn_liquidity(Origin::signed(3), 1, 2, 5000).unwrap();
// 		user_balance_after = XykStorage::balance(0, 3);
// 		rewards_info = XykStorage::get_rewards_info(3, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530); // total rewards 51234, while 14704 were already claimed. Burning puts all rewards to not_yet_claimed, but zeroes the already_claimed. 51234 - 14704 = 36530
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 36530);
// 		assert_eq!(user_balance_after - user_balance_before, 0);

// 		// usecase 7 mint some
// 		user_balance_before = XykStorage::balance(0, 4);
// 		XykStorage::mint_liquidity(Origin::signed(4), 1, 2, 5000, 5010).unwrap();
// 		user_balance_after = XykStorage::balance(0, 4);
// 		rewards_info = XykStorage::get_rewards_info(4, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(4, 4).unwrap(), 36530);
// 		assert_eq!(user_balance_after - user_balance_before, 0);

// 		// usecase 8 deactivate some
// 		user_balance_before = XykStorage::balance(0, 5);
// 		XykStorage::deactivate_liquidity_v2(Origin::signed(5), 4, 5000).unwrap();
// 		user_balance_after = XykStorage::balance(0, 5);
// 		rewards_info = XykStorage::get_rewards_info(5, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(5, 4).unwrap(), 36530);
// 		assert_eq!(user_balance_after - user_balance_before, 0);

// 		// usecase 16 claim some
// 		user_balance_before = XykStorage::balance(0, 6);
// 		XykStorage::claim_rewards_v2(Origin::signed(6), 4, 20000).unwrap();
// 		user_balance_after = XykStorage::balance(0, 6);
// 		rewards_info = XykStorage::get_rewards_info(6, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 34704);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(6, 4).unwrap(), 16530);
// 		assert_eq!(user_balance_after - user_balance_before, 20000);
// 	});
// }

// // starting point, user burned some rewards, then new rewards were generated (yellow)
// #[test]
// #[serial]
// fn rewards_storage_right_amounts_start2() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 1, 10000, 2, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(0));

// 		XykStorage::transfer(1, 2, 3, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 3, 20010).unwrap();
// 		XykStorage::transfer(1, 2, 4, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 4, 20010).unwrap();
// 		XykStorage::transfer(1, 2, 5, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 5, 20010).unwrap();
// 		XykStorage::transfer(1, 2, 6, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 6, 20010).unwrap();
// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, 10000, None).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(3), 1, 2, 10000, 10010).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(4), 1, 2, 10000, 10010).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(5), 1, 2, 10000, 10010).unwrap();

// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(10));

// 		XykStorage::burn_liquidity(Origin::signed(2), 1, 2, 5000).unwrap();
// 		XykStorage::burn_liquidity(Origin::signed(3), 1, 2, 5000).unwrap();
// 		XykStorage::burn_liquidity(Origin::signed(4), 1, 2, 5000).unwrap();
// 		XykStorage::burn_liquidity(Origin::signed(5), 1, 2, 5000).unwrap();

// 		System::set_block_number(200);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(20));

// 		let mut rewards_info = XykStorage::get_rewards_info(2, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 14704);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		rewards_info = XykStorage::get_rewards_info(3, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 14704);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		rewards_info = XykStorage::get_rewards_info(4, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 14704);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		rewards_info = XykStorage::get_rewards_info(5, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 14704);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 32973);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 32973);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(4, 4).unwrap(), 32973);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(5, 4).unwrap(), 32973);

// 		// starting point for blue cases

// 		// usecase 2 claim_all
// 		let mut user_balance_before = XykStorage::balance(0, 2);
// 		XykStorage::claim_rewards_all_v2(Origin::signed(2), 4).unwrap();
// 		let mut user_balance_after = XykStorage::balance(0, 2);
// 		rewards_info = XykStorage::get_rewards_info(2, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 18269);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 0);
// 		assert_eq!(user_balance_after - user_balance_before, 32973);

// 		// usecase 9 burn some
// 		user_balance_before = XykStorage::balance(0, 3);
// 		XykStorage::burn_liquidity(Origin::signed(3), 1, 2, 5000).unwrap();
// 		user_balance_after = XykStorage::balance(0, 3);
// 		rewards_info = XykStorage::get_rewards_info(3, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 32973);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 32973);
// 		assert_eq!(user_balance_after - user_balance_before, 0);

// 		// usecase 10 mint some
// 		user_balance_before = XykStorage::balance(0, 4);
// 		XykStorage::mint_liquidity(Origin::signed(4), 1, 2, 5000, 5010).unwrap();
// 		user_balance_after = XykStorage::balance(0, 4);
// 		rewards_info = XykStorage::get_rewards_info(4, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 32973);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(4, 4).unwrap(), 32973);
// 		assert_eq!(user_balance_after - user_balance_before, 0);

// 		// usecase 11 deactivate some
// 		user_balance_before = XykStorage::balance(0, 5);
// 		XykStorage::deactivate_liquidity_v2(Origin::signed(5), 4, 5000).unwrap();
// 		user_balance_after = XykStorage::balance(0, 5);
// 		rewards_info = XykStorage::get_rewards_info(5, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 32973);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(5, 4).unwrap(), 32973);
// 		assert_eq!(user_balance_after - user_balance_before, 0);
// 	});
// }

// // starting point, just new rewards were generated (green)
// #[test]
// #[serial]
// fn rewards_storage_right_amounts_start3() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = max;

// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 1, 10000, 2, 10000).unwrap();
// 		XykStorage::update_pool_promotion(Origin::root(), 4, Some(1u8)).unwrap();
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(0));

// 		XykStorage::transfer(1, 2, 3, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 3, 20010).unwrap();
// 		XykStorage::transfer(1, 2, 4, 20010).unwrap();
// 		XykStorage::transfer(2, 2, 4, 20010).unwrap();

// 		XykStorage::activate_liquidity_v2(Origin::signed(2), 4, 10000, None).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(3), 1, 2, 10000, 10010).unwrap();

// 		System::set_block_number(100);
// 		MockPromotedPoolApi::instance()
// 			.lock()
// 			.unwrap()
// 			.insert(4, U256::from(u128::MAX) * U256::from(10));

// 		let mut rewards_info = XykStorage::get_rewards_info(2, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);
// 		rewards_info = XykStorage::get_rewards_info(3, 4);
// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 0);

// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 14704);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 14704);

// 		// starting point for blue cases

// 		// usecase 1 claim (all)
// 		let mut user_balance_before = XykStorage::balance(0, 2);
// 		XykStorage::claim_rewards_v2(Origin::signed(2), 4, 14704).unwrap();
// 		let mut user_balance_after = XykStorage::balance(0, 2);
// 		rewards_info = XykStorage::get_rewards_info(2, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 14704);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(2, 4).unwrap(), 0);
// 		assert_eq!(user_balance_after - user_balance_before, 14704);

// 		// usecase 17 claim some
// 		user_balance_before = XykStorage::balance(0, 3);
// 		XykStorage::claim_rewards_v2(Origin::signed(3), 4, 10000).unwrap();
// 		user_balance_after = XykStorage::balance(0, 3);
// 		rewards_info = XykStorage::get_rewards_info(3, 4);

// 		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
// 		assert_eq!(rewards_info.rewards_already_claimed, 10000);
// 		assert_eq!(XykStorage::calculate_rewards_amount_v2(3, 4).unwrap(), 4704);
// 		assert_eq!(user_balance_after - user_balance_before, 10000);
// 	});
// }


#[test]
#[serial]
fn migration_work() {
	new_test_ext().execute_with(|| {
		env_logger::init();
		MockPromotedPoolApi::instance().lock().unwrap().clear();
		System::set_block_number(1);

		let acc_id: u128 = 2;
		let amount: u128 = 10000000000;

		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::transfer(
			0,
			2,
			<Test as Config>::LiquidityMiningIssuanceVault::get(),
			10000000000,
		)
		.unwrap();

		System::set_block_number(1350);
		MockPromotedPoolApi::instance().lock().unwrap().insert(5, U256::from(200000));

		LiquidityMiningActiveUser::<Test>::insert((2, 5), 10000);
		LiquidityMiningActiveUser::<Test>::insert((3, 5), 10000);
		LiquidityMiningActivePool::<Test>::insert(5, 20000);
		LiquidityMiningUser::<Test>::insert((2, 5), (0, U256::from(0), U256::from(10000)));
		LiquidityMiningUser::<Test>::insert((3, 5), (10, U256::from(0), U256::from(10000)));
		LiquidityMiningPool::<Test>::insert(5, (10, U256::from(21985), U256::from(15584)));
		LiquidityMiningUserToBeClaimed::<Test>::insert((2, 5), 0);
		LiquidityMiningUserClaimed::<Test>::insert((2, 5), 0);
		LiquidityMiningUserToBeClaimed::<Test>::insert((3, 5), 0);
		LiquidityMiningUserClaimed::<Test>::insert((3, 5), 0);

		log::info!("{:?}", XykStorage::liquidity_mining_user((2, 5)));
		log::info!("{:?}", XykStorage::liquidity_mining_user((3, 5)));
		log::info!("{:?}", XykStorage::liquidity_mining_pool(5));

		assert_eq!(XykStorage::balance(0, 2), 0);
		assert_eq!(XykStorage::balance(0, 3), 0);

		//at this point users have 156202 and 43792 rewards
		//simulating v1 claim of user1, by adding whole claimable rewards to rewars already claimed
		//create negative rewards at user1 by user2 minting big sum, simulating it by adding work, as mint v1 fn are no longer available
		LiquidityMiningUserClaimed::<Test>::insert((2, 5), 156202);
		LiquidityMiningUser::<Test>::insert((3, 5), (10, U256::from(200000), U256::from(10000)));
		LiquidityMiningPool::<Test>::insert(5, (10, U256::from(221985), U256::from(15584)));

		<XykStorage as XykFunctionsTrait<AccountId>>::rewards_migrate_v1_to_v2(2, 5).unwrap();
		log::info!("{:?}", MockPromotedPoolApi::get_pool_rewards(5));
		<XykStorage as XykFunctionsTrait<AccountId>>::rewards_migrate_v1_to_v2(3, 5).unwrap();

		let mut rewards_info = XykStorage::get_rewards_info(2, 5);

		log::info!("{:?}", rewards_info.activated_amount);
		log::info!("{:?}", rewards_info.rewards_not_yet_claimed);
		log::info!("{:?}", rewards_info.rewards_already_claimed);
		log::info!("{:?}", rewards_info.last_checkpoint);
		log::info!("{:?}", rewards_info.pool_ratio_at_last_checkpoint);
		log::info!("{:?}", rewards_info.missing_at_last_checkpoint);

		assert_eq!(rewards_info.activated_amount, 10000);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 103994);
		assert_eq!(rewards_info.last_checkpoint, 20);
		assert_eq!(rewards_info.pool_ratio_at_last_checkpoint, U256::from(200000)); //these values will be from rew2, but reading pool_ratio_at_last_checkpoint works
		assert_eq!(rewards_info.missing_at_last_checkpoint, U256::from_dec_str("3118").unwrap());

		rewards_info = XykStorage::get_rewards_info(3, 5);
		assert_eq!(rewards_info.activated_amount, 10000);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 147790);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(rewards_info.last_checkpoint, 20);
		assert_eq!(rewards_info.pool_ratio_at_last_checkpoint, U256::from(147792)); //these values will be from rew2, but reading pool_ratio_at_last_checkpoint works
		assert_eq!(rewards_info.missing_at_last_checkpoint, U256::from_dec_str("5584").unwrap());

		assert_eq!(MockPromotedPoolApi::get_pool_rewards(4).unwrap(), 2) // leftover becouse of rounding

		// rewards in vault were 200 000. after migration, the owned amount is: user1 -103 994, user2 +147 790 = 43 796, + 156 202 user already took out = 200 000
		// currently balance in vault is 200 000 - 156 202 = 43 798, which is the same balance as in rewards storage user1 -103 994, user2 +147 790 = 43 796 (vault balance is +2 because of rounding in rew1)
	});
}
