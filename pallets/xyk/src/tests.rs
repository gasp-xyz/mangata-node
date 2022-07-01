// Copyright (C) 2020 Mangata team
#![cfg(not(feature = "runtime-benchmarks"))]
#![allow(non_snake_case)]

use super::*;
use crate::mock::*;
use frame_support::assert_err;
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
		Origin::signed(DUMMY_USER_ID),
		1,
		40000000000000000000,
		4,
		60000000000000000000,
	)
	.unwrap();

	let pool_created_event = crate::mock::Event::XykStorage(crate::Event::<Test>::PoolCreated(
		acc_id,
		1,
		40000000000000000000,
		4,
		60000000000000000000,
	));

	assert!(System::events().iter().any(|record| record.event == pool_created_event));
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
	XykStorage::create_pool(Origin::signed(2), 0, 100000000000000, 1, 100000000000000).unwrap();
	XykStorage::create_pool(Origin::signed(2), 1, 100000000000000, 4, 100000000000000).unwrap();
}

fn initialize_liquidity_rewards() {
	System::set_block_number(1);
	let acc_id: u128 = 2;
	let amount: u128 = std::u128::MAX;
	MockPromotedPoolApi::instance().lock().unwrap().clear();
	MockPromotedPoolApi::instance().lock().unwrap().insert(4, 1000000000);
	XykStorage::create_new_token(&acc_id, amount);
	XykStorage::create_new_token(&acc_id, amount);
	XykStorage::create_new_token(&acc_id, amount);
	XykStorage::create_new_token(&acc_id, amount);

	XykStorage::create_pool(Origin::signed(2), 0, 5000, 1, 5000).unwrap();
	XykStorage::activate_liquidity(Origin::signed(2), 4, 5000, None).unwrap();
}

// #[test]
// #[serial]
// fn liquidity_rewards_single_user_work_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		//	let amount: u128 = max;
// 		let amount: u128 = max;
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_pool(Origin::signed(2), 0, max - 1, 1, max - 1).unwrap();
// 		XykStorage::promote_pool(Origin::root(), 4).unwrap();
// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity(Origin::signed(2), 4, liquidity_tokens_owned, None).unwrap();

// 		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 1000000000);

// 		assert_eq!(
// 			XykStorage::liquidity_mining_user((2, 4)),
// 			(
// 				0,
// 				U256::from_dec_str("0").unwrap(),
// 				U256::from_dec_str("340282366920938463463374607431768211454").unwrap()
// 			)
// 		);

// 		let (
// 			user_last_checkpoint,
// 			user_cummulative_work_in_last_checkpoint,
// 			user_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				1,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(0)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				2,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("19261266052128592271511770231986879893").unwrap()
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				3,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("56761346399350496564279671432783054583").unwrap()
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				4,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("111558562201372305723522896226046528132").unwrap()
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				5,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("182100990657336769517439846212925528194").unwrap()
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				10,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("748134763815679538947955385515053496837").unwrap()
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				100,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("28034299255251320439692753381692314106460").unwrap()
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				1000,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("334270711772001883942188322997728384187725").unwrap()
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				10000,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("3396812014060448055112559789586387542980979").unwrap()
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				100000,
// 				user_last_checkpoint,
// 				user_cummulative_work_in_last_checkpoint,
// 				user_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from_dec_str("34022225036944909766816274458445526573840979").unwrap()
// 		);
// 	});
// }



#[test]
#[serial]
fn liquidity_rewards_single_user_work_W() {
	new_test_ext().execute_with(|| {

		MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		XykStorage::create_new_token(&acc_id, amount);
		env_logger::init();
		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
		XykStorage::promote_pool(Origin::root(), 4).unwrap();

		

		let liquidity_tokens_owned = XykStorage::balance(4, 2);
		XykStorage::activate_liquidity(Origin::signed(2), 4, liquidity_tokens_owned, None).unwrap();

		

		let (
			user_last_checkpoint,
			user_cummulative_ratio,
			user_missing_at_last_checkpoint,
		) = XykStorage::liquidity_mining_user((2, 4));

		assert_eq!(
			XykStorage::liquidity_mining_user((2, 4)),
			(
				0,
				0,
				U256::from_dec_str("10000").unwrap()
			)
		);

		System::set_block_number(10);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 0);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),0);
		System::set_block_number(10000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 1);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),556);
		System::set_block_number(20000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 2);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),1666);
		System::set_block_number(30000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 3);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),3269);
		System::set_block_number(40000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 4);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),5348);
		System::set_block_number(50000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 5);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),7876);
		System::set_block_number(60000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 6);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),10826);
		System::set_block_number(70000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 7);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),14176);
		System::set_block_number(80000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 8);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),17902);
		System::set_block_number(90000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 9);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),21983);
		System::set_block_number(100000);
		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 10);
		assert_eq!(XykStorage::calculate_rewards_amount(2,4).unwrap(),26400);

		});
}

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

// 		XykStorage::promote_pool(Origin::root(), 4).unwrap();
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
// 		XykStorage::promote_pool(Origin::root(), 4).unwrap();

// 		assert_err!(
// 			XykStorage::promote_pool(Origin::root(), 4),
// 			Error::<Test>::PoolAlreadyPromoted
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_two_users_minting_W() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();

// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();

// 		System::set_block_number(100001);

// 		XykStorage::mint_liquidity(Origin::signed(2), 0, 1, 5000, 5001).unwrap();
// 		System::set_block_number(200001);
// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 5000, 5001).unwrap();
// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			user3_last_checkpoint,
// 			user3_cummulative_work_in_last_checkpoint,
// 			user3_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((3, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		let user2_work = XykStorage::calculate_work_user(
// 			2,
// 			4,
// 			30,
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		)
// 		.unwrap();
// 		let user3_work = XykStorage::calculate_work_user(
// 			3,
// 			4,
// 			30,
// 			user3_last_checkpoint,
// 			user3_cummulative_work_in_last_checkpoint,
// 			user3_missing_at_last_checkpoint,
// 		)
// 		.unwrap();
// 		let pool_work = XykStorage::calculate_work_pool(
// 			4,
// 			30,
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		)
// 		.unwrap();

// 		assert_eq!(user2_work, U256::from(116256));
// 		assert_eq!(user3_work, U256::from(10992));
// 		assert_eq!(pool_work, U256::from(127251));
// 		System::set_block_number(300001);
// 		assert!(pool_work >= user2_work + user3_work);
// 		let user2_rewards = XykStorage::calculate_rewards_amount(2, 4).unwrap();
// 		let user3_rewards = XykStorage::calculate_rewards_amount(3, 4).unwrap();
// 		let pool_rewards = *MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap();
// 		assert_eq!(user2_rewards, (913595963));
// 		assert_eq!(user3_rewards, (86380460));
// 		assert_eq!(pool_rewards, 1000000000);

// 		assert!(pool_rewards >= user2_rewards + user3_rewards);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_claim_W() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();
// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();
// 		System::set_block_number(300001);

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (1000000000));
// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap(), 1000000000);
// 		assert_eq!(XykStorage::balance(0, 2), 340282366920938463463374607421768206455);

// 		XykStorage::claim_rewards(Origin::signed(2), 4, 500000000).unwrap();

// 		assert_eq!(XykStorage::balance(0, 2), 340282366920938463463374607422268206455);

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (500000000));
// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap(), 1000000000);

// 		System::set_block_number(400001);
// 		XykStorage::claim_rewards(Origin::signed(2), 4, 500000000).unwrap();

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (0));
// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap(), 1000000000);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_claim_more_NW() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();
// 		System::set_block_number(300001);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (1000000000));
// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap(), 1000000000);

// 		assert_err!(
// 			XykStorage::claim_rewards(Origin::signed(2), 4, 5000000000),
// 			Error::<Test>::NotEnoughtRewardsEarned,
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_burn_W() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();
// 		System::set_block_number(300001);
// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);

// 		let liquidity_tokens_activated = XykStorage::liquidity_mining_active_user((2, 4));
// 		assert_eq!(liquidity_tokens_activated, 5000);

// 		XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 2500).unwrap();

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::liquidity_mining_user((2, 4)),
// 			(30, U256::from(38523), U256::from(435))
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(38523)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(38523)
// 		);
// 		assert_eq!(XykStorage::liquidity_mining_user_to_be_claimed((2, 4)), 500000000);
// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap(), 500000000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (1000000000));

// 		XykStorage::claim_rewards(Origin::signed(2), 4, 750000000).unwrap();

// 		assert_eq!(
// 			XykStorage::liquidity_mining_user((2, 4)),
// 			(30, U256::from(38523), U256::from(435))
// 		);

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint,
// 			)
// 			.unwrap(),
// 			U256::from(38523)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint,
// 			)
// 			.unwrap(),
// 			U256::from(38523)
// 		);
// 		assert_eq!(XykStorage::liquidity_mining_user_to_be_claimed((2, 4)), 0);
// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap(), 500000000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (250000000));

// 		XykStorage::claim_rewards(Origin::signed(2), 4, 250000000).unwrap();
// 		assert_eq!(
// 			XykStorage::liquidity_mining_user((2, 4)),
// 			(30, U256::from(38523), U256::from(435))
// 		);

// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 5000, 5002).unwrap();

// 		let liquidity_tokens_activated = XykStorage::liquidity_mining_active_user((2, 4));
// 		assert_eq!(liquidity_tokens_activated, 2500);

// 		XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 2500).unwrap();

// 		assert_eq!(XykStorage::liquidity_mining_user((2, 4)), (0, U256::from(0), U256::from(0)));

// 		XykStorage::mint_liquidity(Origin::signed(2), 0, 1, 5000, 5003).unwrap();

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				40,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(10992)
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn calculate_liqudity_work_after_all_deactivate() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = std::u128::MAX;

// 		XykStorage::create_new_token(&acc_id, amount); //0
// 		XykStorage::create_new_token(&acc_id, amount); //1
// 		XykStorage::create_new_token(&acc_id, amount); //2
// 		XykStorage::create_new_token(&acc_id, amount); //3
// 		XykStorage::create_new_token(&acc_id, amount); //4
// 		XykStorage::create_new_token(&acc_id, amount); //5

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 4, 10000000, 5, 10000000).unwrap();
// 		//XykStorage::activate_liquidity(Origin::signed(2), 4, 5000).unwrap();
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		MockPromotedPoolApi::instance().lock().unwrap().insert(6, 1000000000);

// 		XykStorage::transfer(4, 2, 3, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(5, 2, 3, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(4, 2, 4, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(5, 2, 4, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(4, 2, 5, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(5, 2, 5, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(4, 2, 6, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(5, 2, 6, 500_000_000_000_000_000_000_000_000_000).unwrap();

// 		XykStorage::mint_liquidity(
// 			Origin::signed(3),
// 			4,
// 			5,
// 			50_000_000_000_000_000_000_000_000_000,
// 			500_000_000_000_000_000_000_000_000_001,
// 		)
// 		.unwrap();

// 		XykStorage::mint_liquidity(
// 			Origin::signed(4),
// 			4,
// 			5,
// 			50_000_000_000_000_000_000_000_000_000,
// 			500_000_000_000_000_000_000_000_000_001,
// 		)
// 		.unwrap();
// 		XykStorage::mint_liquidity(
// 			Origin::signed(5),
// 			4,
// 			5,
// 			50_000_000_000_000_000_000_000_000_000,
// 			500_000_000_000_000_000_000_000_000_001,
// 		)
// 		.unwrap();
// 		XykStorage::mint_liquidity(
// 			Origin::signed(6),
// 			4,
// 			5,
// 			50_000_000_000_000_000_000_000_000_000,
// 			500_000_000_000_000_000_000_000_000_001,
// 		)
// 		.unwrap();

// 		assert_eq!(XykStorage::balance(6, 3), 0);
// 		assert_eq!(XykStorage::reserved(6, 3), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(
// 			XykStorage::liquidity_mining_active_user((3, 6)),
// 			50_000_000_000_000_000_000_000_000_000
// 		);

// 		assert_eq!(XykStorage::balance(6, 4), 0);
// 		assert_eq!(XykStorage::reserved(6, 4), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(
// 			XykStorage::liquidity_mining_active_user((4, 6)),
// 			50_000_000_000_000_000_000_000_000_000
// 		);

// 		assert_eq!(XykStorage::balance(6, 5), 0);
// 		assert_eq!(XykStorage::reserved(6, 5), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(
// 			XykStorage::liquidity_mining_active_user((5, 6)),
// 			50_000_000_000_000_000_000_000_000_000
// 		);

// 		assert_eq!(XykStorage::balance(6, 6), 0);
// 		assert_eq!(XykStorage::reserved(6, 6), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(
// 			XykStorage::liquidity_mining_active_user((6, 6)),
// 			50_000_000_000_000_000_000_000_000_000
// 		);

// 		System::set_block_number(100000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 6).unwrap(), (0));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(5, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(6, 6).unwrap(), (250000000));

// 		System::set_block_number(100000);
// 		XykStorage::deactivate_liquidity(
// 			Origin::signed(3),
// 			6,
// 			50_000_000_000_000_000_000_000_000_000,
// 		)
// 		.unwrap();
// 		XykStorage::deactivate_liquidity(
// 			Origin::signed(4),
// 			6,
// 			50_000_000_000_000_000_000_000_000_000,
// 		)
// 		.unwrap();
// 		XykStorage::deactivate_liquidity(
// 			Origin::signed(5),
// 			6,
// 			50_000_000_000_000_000_000_000_000_000,
// 		)
// 		.unwrap();
// 		XykStorage::deactivate_liquidity(
// 			Origin::signed(6),
// 			6,
// 			50_000_000_000_000_000_000_000_000_000,
// 		)
// 		.unwrap();

// 		assert_eq!(XykStorage::balance(6, 3), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(XykStorage::reserved(6, 3), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 6)), 0);

// 		assert_eq!(XykStorage::balance(6, 4), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(XykStorage::reserved(6, 4), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((4, 6)), 0);

// 		assert_eq!(XykStorage::balance(6, 5), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(XykStorage::reserved(6, 5), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((5, 6)), 0);

// 		assert_eq!(XykStorage::balance(6, 6), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(XykStorage::reserved(6, 6), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((6, 6)), 0);

// 		System::set_block_number(100000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 6).unwrap(), (0));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(5, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(6, 6).unwrap(), (250000000));

// 		let mut n: u64 = 100000;
// 		while n < 150000 {
// 			System::set_block_number(n.into());

// 			assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (250000000));
// 			n = n + 1000;
// 		}
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_work_after_burn_all_and_mint() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = std::u128::MAX;
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		MockPromotedPoolApi::instance().lock().unwrap().insert(6, 1000000000);
// 		XykStorage::create_new_token(&acc_id, amount); //0
// 		XykStorage::create_new_token(&acc_id, amount); //1
// 		XykStorage::create_new_token(&acc_id, amount); //2
// 		XykStorage::create_new_token(&acc_id, amount); //3
// 		XykStorage::create_new_token(&acc_id, amount); //4
// 		XykStorage::create_new_token(&acc_id, amount); //5

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 4, 5000, 5, 5000).unwrap();

// 		XykStorage::transfer(4, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(5, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(4, 2, 4, 1000000).unwrap();
// 		XykStorage::transfer(5, 2, 4, 1000000).unwrap();

// 		XykStorage::mint_liquidity(Origin::signed(3), 4, 5, 5000, 5002).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(4), 4, 5, 5000, 5002).unwrap();

// 		assert_eq!(XykStorage::balance(6, 3), 0);
// 		assert_eq!(XykStorage::reserved(6, 3), 5000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 6)), 5000);

// 		assert_eq!(XykStorage::balance(6, 4), 0);
// 		assert_eq!(XykStorage::reserved(6, 4), 5000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((4, 6)), 5000);

// 		System::set_block_number(100000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 6).unwrap(), (0));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (499977257));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 6).unwrap(), (499977257));

// 		MockPromotedPoolApi::instance().lock().unwrap().insert(6, 1000000000);
// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&6).unwrap(), 1000000000);

// 		XykStorage::burn_liquidity(Origin::signed(3), 4, 5, 5000).unwrap();
// 		XykStorage::burn_liquidity(Origin::signed(4), 4, 5, 5000).unwrap();

// 		assert_eq!(XykStorage::balance(6, 3), 0);
// 		assert_eq!(XykStorage::reserved(6, 3), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 6)), 0);

// 		assert_eq!(XykStorage::balance(6, 4), 0);
// 		assert_eq!(XykStorage::reserved(6, 4), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((4, 6)), 0);

// 		System::set_block_number(200000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 6).unwrap(), (0));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (499977257));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 6).unwrap(), (499977257));

// 		MockPromotedPoolApi::instance().lock().unwrap().insert(6, 1000000000);

// 		XykStorage::mint_liquidity(Origin::signed(3), 4, 5, 5000, 5002).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(4), 4, 5, 5000, 5002).unwrap();

// 		assert_eq!(XykStorage::balance(6, 3), 0);
// 		assert_eq!(XykStorage::reserved(6, 3), 5000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 6)), 5000);

// 		assert_eq!(XykStorage::balance(6, 4), 0);
// 		assert_eq!(XykStorage::reserved(6, 4), 5000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((4, 6)), 5000);

// 		System::set_block_number(300000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 6).unwrap(), (0));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (999954514));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 6).unwrap(), (999954514));

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 6));
// 		let (
// 			user3_last_checkpoint,
// 			user3_cummulative_work_in_last_checkpoint,
// 			user3_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((3, 6));
// 		let (
// 			user4_last_checkpoint,
// 			user4_cummulative_work_in_last_checkpoint,
// 			user4_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((4, 6));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(6);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				6,
// 				50,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(0)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				3,
// 				6,
// 				50,
// 				user3_last_checkpoint,
// 				user3_cummulative_work_in_last_checkpoint,
// 				user3_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				4,
// 				6,
// 				50,
// 				user4_last_checkpoint,
// 				user4_cummulative_work_in_last_checkpoint,
// 				user4_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				6,
// 				50,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(154093)
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_calculate_rewards_work_after_burning_all() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();
// 		System::set_block_number(1);
// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();

// 		XykStorage::transfer(0, 2, 4, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 4, 1000000).unwrap();

// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 5000, 5002).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(4), 0, 1, 5000, 5002).unwrap();

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			user3_last_checkpoint,
// 			user3_cummulative_work_in_last_checkpoint,
// 			user3_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((3, 4));
// 		let (
// 			user4_last_checkpoint,
// 			user4_cummulative_work_in_last_checkpoint,
// 			user4_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((4, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				3,
// 				4,
// 				30,
// 				user3_last_checkpoint,
// 				user3_cummulative_work_in_last_checkpoint,
// 				user3_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				4,
// 				4,
// 				30,
// 				user4_last_checkpoint,
// 				user4_cummulative_work_in_last_checkpoint,
// 				user4_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(231139)
// 		);

// 		let liquidity_tokens_activated = XykStorage::liquidity_mining_active_user((2, 4));
// 		assert_eq!(liquidity_tokens_activated, 5000);
// 		let liquidity_tokens_activated = XykStorage::liquidity_mining_active_user((3, 4));
// 		assert_eq!(liquidity_tokens_activated, 5000);
// 		let liquidity_tokens_activated = XykStorage::liquidity_mining_active_user((4, 4));
// 		assert_eq!(liquidity_tokens_activated, 5000);

// 		System::set_block_number(300000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (333331891));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (333331891));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 4).unwrap(), (333331891));
// 		assert_eq!(XykStorage::balance(2, 4), 0);
// 		assert_eq!(XykStorage::balance(3, 4), 0);
// 		assert_eq!(XykStorage::balance(4, 4), 0);
// 		System::set_block_number(400000);

// 		XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 5000).unwrap();

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (333332409));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (333332409));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 4).unwrap(), (333332409));

// 		System::set_block_number(500000);

// 		XykStorage::mint_liquidity(Origin::signed(2), 0, 1, 5000, 5002).unwrap();
// 		System::set_block_number(600000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (349998719));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (324993059));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 4).unwrap(), (324993059));

// 		XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 5000).unwrap();
// 		XykStorage::burn_liquidity(Origin::signed(3), 0, 1, 5000).unwrap();
// 		XykStorage::burn_liquidity(Origin::signed(4), 0, 1, 5000).unwrap();

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (349998719));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (324993059));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 4).unwrap(), (324993059));

// 		System::set_block_number(700000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (349998719));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (324993059));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 4).unwrap(), (324993059));
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_deactivate_W() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();
// 		System::set_block_number(300001);
// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);

// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 5000);
// 		XykStorage::deactivate_liquidity(Origin::signed(2), 4, 2500).unwrap();
// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 2500);

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::liquidity_mining_user((2, 4)),
// 			(30, U256::from(38523), U256::from(435))
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(38523)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(38523)
// 		);
// 		assert_eq!(XykStorage::liquidity_mining_user_to_be_claimed((2, 4)), 500000000);
// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap(), 500000000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (1000000000));

// 		System::set_block_number(3000001);
// 		assert_eq!(XykStorage::balance(4, 2), 2500);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 2500);

// 		XykStorage::deactivate_liquidity(Origin::signed(2), 4, 2500).unwrap();

// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 0);
// 		assert_eq!(XykStorage::liquidity_mining_user((2, 4)), (300, U256::from(0), U256::from(0)));
// 		assert_eq!(XykStorage::balance(4, 2), 5000);

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			_pool_last_checkpoint,
// 			_pool_cummulative_work_in_last_checkpoint,
// 			_pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				3000005,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(0)
// 		);

// 		System::set_block_number(4000001);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				400,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint
// 			)
// 			.unwrap(),
// 			U256::from(0)
// 		);
// 		assert_eq!(XykStorage::liquidity_mining_user_to_be_claimed((2, 4)), 1000000000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (1000000000));
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_calculate_work_after_deactivate() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		let amount: u128 = std::u128::MAX;

// 		XykStorage::create_new_token(&acc_id, amount); //0
// 		XykStorage::create_new_token(&acc_id, amount); //1
// 		XykStorage::create_new_token(&acc_id, amount); //2
// 		XykStorage::create_new_token(&acc_id, amount); //3
// 		XykStorage::create_new_token(&acc_id, amount); //4
// 		XykStorage::create_new_token(&acc_id, amount); //5

// 		XykStorage::transfer(
// 			0,
// 			2,
// 			<Test as Config>::LiquidityMiningIssuanceVault::get(),
// 			10000000000,
// 		)
// 		.unwrap();

// 		XykStorage::create_pool(Origin::signed(2), 4, 10000000, 5, 10000000).unwrap();
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		MockPromotedPoolApi::instance().lock().unwrap().insert(6, 1000000000);

// 		XykStorage::transfer(4, 2, 3, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(5, 2, 3, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(4, 2, 4, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(5, 2, 4, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(4, 2, 5, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(5, 2, 5, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(4, 2, 6, 500_000_000_000_000_000_000_000_000_000).unwrap();
// 		XykStorage::transfer(5, 2, 6, 500_000_000_000_000_000_000_000_000_000).unwrap();

// 		XykStorage::mint_liquidity(
// 			Origin::signed(3),
// 			4,
// 			5,
// 			50_000_000_000_000_000_000_000_000_000,
// 			500_000_000_000_000_000_000_000_000_001,
// 		)
// 		.unwrap();

// 		XykStorage::mint_liquidity(
// 			Origin::signed(4),
// 			4,
// 			5,
// 			50_000_000_000_000_000_000_000_000_000,
// 			500_000_000_000_000_000_000_000_000_001,
// 		)
// 		.unwrap();
// 		XykStorage::mint_liquidity(
// 			Origin::signed(5),
// 			4,
// 			5,
// 			50_000_000_000_000_000_000_000_000_000,
// 			500_000_000_000_000_000_000_000_000_001,
// 		)
// 		.unwrap();
// 		XykStorage::mint_liquidity(
// 			Origin::signed(6),
// 			4,
// 			5,
// 			50_000_000_000_000_000_000_000_000_000,
// 			500_000_000_000_000_000_000_000_000_001,
// 		)
// 		.unwrap();

// 		assert_eq!(XykStorage::balance(6, 3), 0);
// 		assert_eq!(XykStorage::reserved(6, 3), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(
// 			XykStorage::liquidity_mining_active_user((3, 6)),
// 			50_000_000_000_000_000_000_000_000_000
// 		);

// 		assert_eq!(XykStorage::balance(6, 4), 0);
// 		assert_eq!(XykStorage::reserved(6, 4), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(
// 			XykStorage::liquidity_mining_active_user((4, 6)),
// 			50_000_000_000_000_000_000_000_000_000
// 		);

// 		assert_eq!(XykStorage::balance(6, 5), 0);
// 		assert_eq!(XykStorage::reserved(6, 5), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(
// 			XykStorage::liquidity_mining_active_user((5, 6)),
// 			50_000_000_000_000_000_000_000_000_000
// 		);

// 		assert_eq!(XykStorage::balance(6, 6), 0);
// 		assert_eq!(XykStorage::reserved(6, 6), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(
// 			XykStorage::liquidity_mining_active_user((6, 6)),
// 			50_000_000_000_000_000_000_000_000_000
// 		);

// 		System::set_block_number(100000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 6).unwrap(), (0));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(5, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(6, 6).unwrap(), (250000000));

// 		System::set_block_number(100000);
// 		XykStorage::deactivate_liquidity(
// 			Origin::signed(3),
// 			6,
// 			50_000_000_000_000_000_000_000_000_000,
// 		)
// 		.unwrap();
// 		XykStorage::deactivate_liquidity(
// 			Origin::signed(4),
// 			6,
// 			50_000_000_000_000_000_000_000_000_000,
// 		)
// 		.unwrap();
// 		XykStorage::deactivate_liquidity(
// 			Origin::signed(5),
// 			6,
// 			50_000_000_000_000_000_000_000_000_000,
// 		)
// 		.unwrap();
// 		XykStorage::deactivate_liquidity(
// 			Origin::signed(6),
// 			6,
// 			50_000_000_000_000_000_000_000_000_000,
// 		)
// 		.unwrap();

// 		assert_eq!(XykStorage::balance(6, 3), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(XykStorage::reserved(6, 3), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 6)), 0);

// 		assert_eq!(XykStorage::balance(6, 4), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(XykStorage::reserved(6, 4), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((4, 6)), 0);

// 		assert_eq!(XykStorage::balance(6, 5), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(XykStorage::reserved(6, 5), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((5, 6)), 0);

// 		assert_eq!(XykStorage::balance(6, 6), 50_000_000_000_000_000_000_000_000_000);
// 		assert_eq!(XykStorage::reserved(6, 6), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((6, 6)), 0);

// 		let leftRewardsInPool = *MockPromotedPoolApi::instance().lock().unwrap().get(&6).unwrap();
// 		assert_eq!(leftRewardsInPool, 0);
// 		System::set_block_number(100000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 6).unwrap(), (0));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(4, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(5, 6).unwrap(), (250000000));
// 		assert_eq!(XykStorage::calculate_rewards_amount(6, 6).unwrap(), (250000000));

// 		let mut n: u64 = 100000;
// 		while n < 150000 {
// 			System::set_block_number(n.into());

// 			assert_eq!(XykStorage::calculate_rewards_amount(3, 6).unwrap(), (250000000));
// 			n = n + 1000;
// 		}
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_tokens_reserved_after_mint_W() {
// 	new_test_ext().execute_with(|| {
// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		let max = std::u128::MAX;
// 		System::set_block_number(1);
// 		let acc_id: u128 = 2;
// 		//	let amount: u128 = max;
// 		let amount: u128 = max;
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_new_token(&acc_id, amount);
// 		XykStorage::create_pool(Origin::signed(2), 0, 10000, 1, 10000).unwrap();
// 		XykStorage::promote_pool(Origin::root(), 4).unwrap();

// 		let liquidity_tokens_owned = XykStorage::balance(4, 2);
// 		XykStorage::activate_liquidity(Origin::signed(2), 4, liquidity_tokens_owned, None).unwrap();
// 		MockPromotedPoolApi::instance().lock().unwrap().insert(4, 1000000000);

// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();

// 		assert_eq!(XykStorage::reserved(4, 2), 10000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 10000);

// 		XykStorage::mint_liquidity(Origin::signed(2), 0, 1, 10000, 10001).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 10000, 10001).unwrap();

// 		assert_eq!(XykStorage::reserved(4, 2), 20000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 20000);

// 		assert_eq!(XykStorage::reserved(4, 3), 10000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 4)), 10000);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_calculate_rewards_pool_not_promoted() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();

// 		assert_err!(XykStorage::calculate_rewards_amount(2, 8), Error::<Test>::NotAPromotedPool,);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_claim_pool_not_promoted() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();

// 		assert_err!(
// 			XykStorage::claim_rewards(Origin::signed(2), 7, 5000000000),
// 			Error::<Test>::NotAPromotedPool,
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
// #[serial]
// fn liquidity_rewards_transfer_working_after_deactivate() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();

// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 5000);
// 		assert_eq!(XykStorage::balance(4, 2), 0);

// 		XykStorage::deactivate_liquidity(Origin::signed(2), 4, 2500).unwrap();
// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 2500);
// 		assert_eq!(XykStorage::balance(4, 2), 2500);

// 		XykStorage::transfer(4, 2, 3, 2500).unwrap();
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 4)), 0);
// 		assert_eq!(XykStorage::balance(4, 3), 2500);

// 		XykStorage::activate_liquidity(Origin::signed(3), 4, 2500, None).unwrap();

// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 2500);
// 		assert_eq!(XykStorage::balance(4, 2), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 4)), 2500);
// 		assert_eq!(XykStorage::balance(4, 3), 0);

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));
// 		let (
// 			user3_last_checkpoint,
// 			user3_cummulative_work_in_last_checkpoint,
// 			user3_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((3, 4));
// 		let (
// 			pool_last_checkpoint,
// 			pool_cummulative_work_in_last_checkpoint,
// 			pool_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_pool(4);

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint,
// 			)
// 			.unwrap(),
// 			U256::from(38523)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				3,
// 				4,
// 				30,
// 				user3_last_checkpoint,
// 				user3_cummulative_work_in_last_checkpoint,
// 				user3_missing_at_last_checkpoint,
// 			)
// 			.unwrap(),
// 			U256::from(38523)
// 		);
// 		assert_eq!(
// 			XykStorage::calculate_work_pool(
// 				4,
// 				30,
// 				pool_last_checkpoint,
// 				pool_cummulative_work_in_last_checkpoint,
// 				pool_missing_at_last_checkpoint,
// 			)
// 			.unwrap(),
// 			U256::from(77046)
// 		);
// 	});
// }

// #[test]
// #[serial]
// fn liquidity_rewards_rewards_not_gaining_after_burn() {
// 	new_test_ext().execute_with(|| {
// 		initialize_liquidity_rewards();

// 		XykStorage::transfer(0, 2, 3, 1000000).unwrap();
// 		XykStorage::transfer(1, 2, 3, 1000000).unwrap();
// 		XykStorage::mint_liquidity(Origin::signed(3), 0, 1, 5000, 5001).unwrap();

// 		assert_eq!(XykStorage::reserved(4, 2), 5000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 5000);
// 		assert_eq!(XykStorage::balance(4, 2), 0);

// 		assert_eq!(XykStorage::reserved(4, 3), 5000);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((3, 4)), 5000);
// 		assert_eq!(XykStorage::balance(4, 3), 0);

// 		System::set_block_number(100000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (499977257));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (499977257));

// 		XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 5000).unwrap();
// 		assert_eq!(XykStorage::reserved(4, 2), 0);
// 		assert_eq!(XykStorage::liquidity_mining_active_user((2, 4)), 0);
// 		assert_eq!(XykStorage::balance(4, 2), 0);

// 		let (
// 			user2_last_checkpoint,
// 			user2_cummulative_work_in_last_checkpoint,
// 			user2_missing_at_last_checkpoint,
// 		) = XykStorage::liquidity_mining_user((2, 4));

// 		System::set_block_number(100000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (499977257));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (499977257));

// 		System::set_block_number(200000);

// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (499977257));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (499997238));

// 		let pool_rewards = *MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap();
// 		let new_pool_rewards = pool_rewards + 1000000000;

// 		MockPromotedPoolApi::instance().lock().unwrap().clear();
// 		MockPromotedPoolApi::instance().lock().unwrap().insert(4, new_pool_rewards);

// 		assert_eq!(*MockPromotedPoolApi::instance().lock().unwrap().get(&4).unwrap(), 1500022743);

// 		System::set_block_number(300000);
// 		assert_eq!(XykStorage::calculate_rewards_amount(2, 4).unwrap(), (499977257));
// 		assert_eq!(XykStorage::calculate_rewards_amount(3, 4).unwrap(), (1500003274));

// 		assert_eq!(
// 			XykStorage::calculate_work_user(
// 				2,
// 				4,
// 				30,
// 				user2_last_checkpoint,
// 				user2_cummulative_work_in_last_checkpoint,
// 				user2_missing_at_last_checkpoint,
// 			)
// 			.unwrap(),
// 			U256::from(0)
// 		);
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
// 			<assets_info::Pallet<Test>>::get_info(2u32),
// 			assets_info::AssetInfo {
// 				name: Some(b"LiquidityPoolToken0x00000002".to_vec()),
// 				symbol: Some(b"TKN0x00000000-TKN0x00000001".to_vec()),
// 				description: Some(b"Generated Info for Liquidity Pool Token".to_vec()),
// 				decimals: Some(18u32),
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
// 			<assets_info::Pallet<Test>>::get_info(N),
// 			assets_info::AssetInfo {
// 				name: Some(b"LiquidityPoolToken0x00003039".to_vec()),
// 				symbol: Some(b"TKN0x0000000F-TKN0x00002FC9".to_vec()),
// 				description: Some(b"Generated Info for Liquidity Pool Token".to_vec()),
// 				decimals: Some(18u32),
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
// 			<assets_info::Pallet<Test>>::get_info(1524501234u32),
// 			assets_info::AssetInfo {
// 				name: Some(b"LiquidityPoolToken0x5ADE0AF2".to_vec()),
// 				symbol: Some(b"TKN0x00E4E1C0-TKN0x00BAA928".to_vec()),
// 				description: Some(b"Generated Info for Liquidity Pool Token".to_vec()),
// 				decimals: Some(18u32),
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
