// Copyright (C) 2020 Mangata team
#![cfg(not(feature = "runtime-benchmarks"))]
#![allow(non_snake_case)]

use super::{Event, *};
use crate::mock::*;
use frame_support::assert_err;
use mangata_types::assets::CustomMetadata;
use orml_traits::asset_registry::AssetMetadata;
use serial_test::serial;
use sp_runtime::Permill;

type TokensOf<Test> = <Test as Config>::Currency;



#[test]
#[serial]
fn liquidity_rewards_single_user_mint_W() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount);
		TokensOf::<Test>::create(&acc_id, amount);
		TokensOf::<Test>::create(&acc_id, amount);
		TokensOf::<Test>::create(&acc_id, amount);
		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();

		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);

		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();


		let rewards_info = ProofOfStake::get_rewards_info(2, 4);

		assert_eq!(rewards_info.activated_amount, 10000);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(rewards_info.last_checkpoint, 0);
		assert_eq!(rewards_info.pool_ratio_at_last_checkpoint, U256::from(0));
		assert_eq!(rewards_info.missing_at_last_checkpoint, U256::from_dec_str("10000").unwrap());
        //
		// System::set_block_number(10);
		// // MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);
		// System::set_block_number(10);
        //
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX * 1));
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 291);
		// System::set_block_number(20);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 2);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 873);
		// System::set_block_number(30);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 3);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 1716);
		// System::set_block_number(40);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 4);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 2847);
		// System::set_block_number(50);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 5);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 4215);
		// System::set_block_number(60);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 6);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 5844);
		// System::set_block_number(70);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 7);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 7712);
		// System::set_block_number(80);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 8);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 9817);
		// System::set_block_number(90);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 9);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 12142);
		// System::set_block_number(100);
		// // MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 10);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);
	});
}
