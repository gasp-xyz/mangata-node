// Copyright (C) 2020 Mangata team
#![cfg(not(feature = "runtime-benchmarks"))]
#![allow(non_snake_case)]

use super::{*};
use crate::mock::*;
use frame_support::{assert_err, assert_ok};


use mangata_support::traits::{GetIssuance, ComputeIssuance};


type TokensOf<Test> = <Test as Config>::Currency;

fn mint_and_activate_tokens(who: AccountId, token_id: TokenId, amount: Balance) {
		TokensOf::<Test>::mint(token_id, &who, amount).unwrap();
		ProofOfStake::activate_liquidity( who, token_id, amount, None) .unwrap();
}

fn initialize_liquidity_rewards() {
	System::set_block_number(1);
	let acc_id: u128 = 2;
	let amount: u128 = std::u128::MAX;
	// MockPromotedPoolApi::instance().lock().unwrap().clear();
	// MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));
	CumulativeTotalLiquidityToRewardsRatio::<Test>::get();
	TokensOf::<Test>::create(&acc_id, amount).unwrap();
	TokensOf::<Test>::create(&acc_id, amount).unwrap();
	TokensOf::<Test>::create(&acc_id, amount).unwrap();
	TokensOf::<Test>::create(&acc_id, amount).unwrap();
	TokensOf::<Test>::create(&acc_id, 10000).unwrap();

	ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 2u8).unwrap();
	CumulativeTotalLiquidityToRewardsRatio::<Test>::mutate(|pools|{
		pools.get_mut(&4).unwrap().rewards = U256::from(0);
	});

	ProofOfStake::activate_liquidity_v2(RuntimeOrigin::signed(2), 4, 10000, None).unwrap();
}

fn forward_to_block(n: u32){
	forward_to_block_with_custom_rewards(n, 10000);
}

fn forward_to_block_with_custom_rewards(n: u32, rewards: u128){
	while System::block_number().saturated_into::<u32>() <= n {
		if System::block_number().saturated_into::<u32>() % ProofOfStake::rewards_period() == 0 {
			println!("NEW SESSION");
			ProofOfStake::distribute_rewards(rewards);
		}
		System::set_block_number(System::block_number().saturated_into::<u64>() + 1);
	}
}

#[test]
fn liquidity_rewards_single_user_mint_W() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();


		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);

		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 2u8).unwrap();
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
		assert_eq!(rewards_info.missing_at_last_checkpoint, U256::from(10000));

		// MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);

		System::set_block_number(10);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX * 1));
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 291);
		System::set_block_number(20);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 2);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 873);
		System::set_block_number(30);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 3);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 1716);
		System::set_block_number(40);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 4);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 2847);
		System::set_block_number(50);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 5);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 4215);
		System::set_block_number(60);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 6);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 5844);
		System::set_block_number(70);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 7);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 7712);
		System::set_block_number(80);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 8);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 9817);
		System::set_block_number(90);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 9);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 12142);
		System::set_block_number(100);
		ProofOfStake::distribute_rewards(10000 * 1);
		// MockPromotedPoolApi::instance() .lock() .unwrap() .insert(4, U256::from(u128::MAX) * 10);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);
	});
}

#[test]
fn liquidity_rewards_three_users_burn_W() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();


		TokensOf::<Test>::transfer(0, &2, &3, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &3, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(0, &2, &4, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &4, 1000000, ExistenceRequirement::AllowDeath).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		// System::set_block_number(100);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 100000 / 10000);
		forward_to_block(100);

		// XykStorage::mint_liquidity(RuntimeOrigin::signed(3), 0, 1, 10000, 10010).unwrap();
		mint_and_activate_tokens(3, 4, 10000);
		
		// System::set_block_number(200);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 150000 / 10000);
		forward_to_block(200);

		// XykStorage::mint_liquidity(RuntimeOrigin::signed(4), 0, 1, 10000, 10010).unwrap();
		mint_and_activate_tokens(4, 4, 10000);
        
		// System::set_block_number(240);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 163300 / 10000);
		forward_to_block(240);
		
		// XykStorage::burn_liquidity(RuntimeOrigin::signed(4), 0, 1, 5000).unwrap();
		ProofOfStake::deactivate_liquidity( 4, 4, 5000) .unwrap();
        
		// System::set_block_number(400);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 227300 / 10000);
		forward_to_block(400);
		
		// assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 95951);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 95965);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 44130);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 44142);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 10628);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 10630);
	});
}


#[test]
fn liquidity_rewards_claim_W() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::transfer(
			0,
			&2,
			&<Test as Config>::LiquidityMiningIssuanceVault::get(),
			10000000000,
			ExistenceRequirement::AllowDeath,
		)
		.unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();
		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		// assert_eq!(
		// 	xykstorage::liquidity_mining_user_v2((2, 4)),
		// 	(0, 0, u256::from_dec_str("10000").unwrap())
		// );

		// System::set_block_number(10);
		// MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));
		forward_to_block(10);

		// System::set_block_number(90);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 90000 / 10000);
		forward_to_block(90);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 12142);
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(2), 4).unwrap();

		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 100000 / 10000);
		// System::set_block_number(100);
		
		forward_to_block(100);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 2562);
	});
}

#[test]
fn liquidity_rewards_promote_pool_W() {
	new_test_ext().execute_with(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 5000, 1, 5000).unwrap();

		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();
	});
}

#[test]
fn liquidity_rewards_promote_pool_already_promoted_NW() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		// assert!(Test::is_enabled(4));
		assert!(ProofOfStake::is_enabled(4));
	});
}

#[test]
fn liquidity_rewards_claim_more_NW() {
	new_test_ext().execute_with(|| {
		// TODO: remove as claim_rwards_all should be used instead
		// initialize_liquidity_rewards();
		// System::set_block_number(100);
		// // MockPromotedPoolApi::instance()
		// // 	.lock()
		// // 	.unwrap()
		// // 	.insert(4, U256::from(u128::MAX) * 100000 / 10000);
        //
		// assert_eq!(ProofOfStake::calculate_rewards_amount_v2(2, 4).unwrap(), (14704));
        //
		// assert_err!(
		// 	ProofOfStake::claim_rewards_v2(RuntimeOrigin::signed(2), 4, 15000),
		// 	pallet_proof_of_stake::Error::<Test>::NotEnoughRewardsEarned,
		// );
	});
}



#[test]
fn liquidity_rewards_work_after_burn_W() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		TokensOf::<Test>::transfer(0, &2, &3, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &3, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(0, &2, &4, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &4, 1000000, ExistenceRequirement::AllowDeath).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		// System::set_block_number(100);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 100000 / 10000);
		forward_to_block(100);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);

		// XykStorage::mint_liquidity(RuntimeOrigin::signed(3), 0, 1, 10000, 10010).unwrap();
		mint_and_activate_tokens(3, 4, 10000);

		// System::set_block_number(200);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 150000 / 10000);
		forward_to_block(200);

		// XykStorage::mint_liquidity(RuntimeOrigin::signed(4), 0, 1, 10000, 10010).unwrap();
		mint_and_activate_tokens(4, 4, 10000);

		// System::set_block_number(240);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 163300 / 10000);
		forward_to_block(240);

		// XykStorage::burn_liquidity(RuntimeOrigin::signed(4), 0, 1, 10000).unwrap();
		ProofOfStake::deactivate_liquidity( 4, 4, 10000) .unwrap();

		// System::set_block_number(400);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 243300 / 10000);
		forward_to_block(400);

		// assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 946);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 948);

		// XykStorage::mint_liquidity(RuntimeOrigin::signed(4), 0, 1, 20000, 20010).unwrap();
		mint_and_activate_tokens(4, 4, 20000);

		// System::set_block_number(500);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 268300 / 10000);
		forward_to_block(500);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 8299);
	});
}

#[test]
fn liquidity_rewards_deactivate_transfer_controled_W() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);

		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();
		assert_err!(TokensOf::<Test>::transfer(4, &2,&3, 10, ExistenceRequirement::AllowDeath), orml_tokens::Error::<Test>::BalanceTooLow,);

		// System::set_block_number(100);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 100000 / 10000);
		forward_to_block(100);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);

		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(2), 4, liquidity_tokens_owned)
			.unwrap();
		TokensOf::<Test>::transfer(4, &2, &3, 10, ExistenceRequirement::AllowDeath).unwrap();
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);
	});
}

#[test]
fn liquidity_rewards_deactivate_more_NW() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();
		assert_err!(
			ProofOfStake::deactivate_liquidity_v2(
				RuntimeOrigin::signed(2),
				4,
				liquidity_tokens_owned + 1
			),
			Error::<Test>::NotEnoughAssets
		);
	});
}

#[test]
fn liquidity_rewards_activate_more_NW() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		assert_err!(
			ProofOfStake::activate_liquidity_v2(
				RuntimeOrigin::signed(2),
				4,
				liquidity_tokens_owned + 1,
				None
			),
			Error::<Test>::NotEnoughAssets
		);
	});
}

#[test]
fn liquidity_rewards_calculate_rewards_pool_not_promoted() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		//TODO: ask stano when we want to return 0, rpc api ?
		assert_err!(ProofOfStake::calculate_rewards_amount(2, 4), Error::<Test>::NotAPromotedPool);
	});
}

#[test]
fn liquidity_rewards_claim_pool_not_promoted() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		assert_err!(
			ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(2), 7),
			Error::<Test>::NotAPromotedPool,
		);
	});
}

#[test]
fn liquidity_rewards_transfer_not_working() {
	new_test_ext().execute_with(|| {
		initialize_liquidity_rewards();

		assert_err!(TokensOf::<Test>::transfer(4, &2, &3, 10, ExistenceRequirement::AllowDeath), orml_tokens::Error::<Test>::BalanceTooLow,);
	});
}


// A user that moved the Liq.tokens to the derived account , can request asymtotic_curve_rewards
// ??

// A user that bonded Liq.tokens can request asymtotic_curve_rewards
// user that activated, contained in all tests

// Asymtotic_curve_rewards are divided between the people that provided liquidity in the same pool ???
// Contained in liquidity_rewards_three_users_mint_W

// [Accuracy] A user obtain the right tokens from asymtotic_curve_rewards: Burning and minting during time
// Contained in liquidity_rewards_work_after_burn_W

// Asymtotic_curve_rewards are given for destroyed pools
// No unpromote pool fn

// Snaphshot is done when a pool is minted or burned
// Contained in all tests, snapshots are no longer done for pool

// Claim individuals is the same as claim in group
// Contained in liquidity_rewards_claim_W, user is after claiming is getting the claimed amount less, not touching reward amount of others

// Claim all the tokens that the user should have
// Contained in liquidity_rewards_claim_W

// Claim more tokens that the user should have
// Contained in liquidity_rewards_claim_more_NW

// Increasing pool size during <period> triggers checkpoint and the division of rewards is right
// Contained in all tests, snapshots are no longer done for pool

// Decreasing pool size during <period> triggers checkpoint and the division of rewards is right
// Contained in all tests, snapshots are no longer done for pool

// Token status:  When activate the tokens are in reserved.
// Contained in liquidity_rewards_transfer_not_working

// Token status:  When deActivate the tokens are in Free.
// Contained in liquidity_rewards_transfered_liq_tokens_produce_rewards_W

// Fees are in toBeClaimed when deactivate and  notYetClaimed when liquidity is activated

#[test]
fn liquidity_rewards_not_yet_claimed_already_claimed_W() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::transfer(
			0,
			&2,
			&<Test as Config>::LiquidityMiningIssuanceVault::get(),
			10000000000,
			ExistenceRequirement::AllowDeath,
		)
		.unwrap();

		TokensOf::<Test>::create(&2, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		// System::set_block_number(10);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 10000 / 10000);
		forward_to_block(10);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 291);
		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(2), 4, liquidity_tokens_owned)
			.unwrap();

		let rewards_info = ProofOfStake::get_rewards_info(2, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 291);

		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		// System::set_block_number(100);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 100000 / 10000);
		forward_to_block(100);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 12433);
		// ProofOfStake::claim_rewards_v2(RuntimeOrigin::signed(2), 4, 12432).unwrap();
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(2), 4).unwrap();

		let rewards_info = ProofOfStake::get_rewards_info(2, 4);
		// assert_eq!(rewards_info.rewards_already_claimed, 12141);
		assert_eq!(rewards_info.rewards_already_claimed, 12142);
	});
}

// // Curve is the same if user claim rewards or not.
// // Contained in liquidity_rewards_claim_W, user is after claiming is getting the claimed amount less, not touching reward amount of others
//
//test for extreme values inside calculate rewards, mainly pool ratio
#[test]
fn extreme_case_pool_ratio() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::create(&acc_id, max).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		ProofOfStake::activate_liquidity_v2(RuntimeOrigin::signed(2), 4, 1, None).unwrap();

		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(u128::MAX));
		CumulativeTotalLiquidityToRewardsRatio::<Test>::mutate(|pools|{
			pools.get_mut(&4).unwrap().rewards = U256::from(u128::MAX) * U256::from(u128::MAX);
		});

		System::set_block_number(10000);
		assert_eq!(
			ProofOfStake::calculate_rewards_amount(2, 4).unwrap(),
			329053048812547494169083245386519860476
		);
	});
}

#[test]
fn rewards_rounding_during_often_mint() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::transfer(
			0,
			&2,
			&<Test as Config>::LiquidityMiningIssuanceVault::get(),
			10000000000,
			ExistenceRequirement::AllowDeath
		)
		.unwrap();

		// XykStorage::create_pool(
		// 	RuntimeOrigin::signed(2),
		// 	0,
		// 	250000000000000000000000000,
		// 	1,
		// 	10000000000000000,
		// )
		// .unwrap();
		let calculate_liq_tokens_amount = |first_amount: u128, second_amount: u128| -> u128{
			((first_amount / 2) + (second_amount / 2)).try_into().unwrap()
		};
		TokensOf::<Test>::create(&acc_id,calculate_liq_tokens_amount(250000000000000000000000000, 10000000000000000)).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(0));
		TokensOf::<Test>::transfer(0, &2, &3, 10000000000000000000000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &3, 10000000000000000000000000, ExistenceRequirement::AllowDeath).unwrap();
		// TokensOf::<Test>::mint_liquidity(
		// 	RuntimeOrigin::signed(2),
		// 	0,
		// 	1,
		// 	25000000000000000000000,
		// 	2000000000000,
		// )
		// .unwrap();
		mint_and_activate_tokens(2, 4, calculate_liq_tokens_amount(25000000000000000000000, 2000000000000));
		// XykStorage::mint_liquidity(
		// 	RuntimeOrigin::signed(3),
		// 	0,
		// 	1,
		// 	25000000000000000000000,
		// 	2000000000000,
		// )
		// .unwrap();
		mint_and_activate_tokens(3, 4, calculate_liq_tokens_amount(25000000000000000000000, 2000000000000));

		let mut non_minter_higher_rewards_counter = 0;
		let mut higher_rewards_cumulative = 0;
		for n in 1..10000 {
			System::set_block_number(n);
			if (n + 1) % (ProofOfStake::rewards_period() as u64) == 0 {
				// MockPromotedPoolApi::instance()
				// 	.lock()
				// 	.unwrap()
				// 	.insert(4, U256::from(u128::MAX) * U256::from(n + 1));
				ProofOfStake::distribute_rewards(10000);

				// XykStorage::mint_liquidity(
				// 	RuntimeOrigin::signed(3),
				// 	0,
				// 	1,
				// 	34000000000000000000,
				// 	68000000000000000000,
				// )
				// .unwrap();
				mint_and_activate_tokens(3, 4, calculate_liq_tokens_amount(34000000000000000000, 68000000000000000000));
				log::info!("----------------------------");
				let rew_non_minter = ProofOfStake::calculate_rewards_amount(2, 4).unwrap();
				let rew_minter = ProofOfStake::calculate_rewards_amount(3, 4).unwrap();
				log::info!("rew        {} {}", n, rew_non_minter);
				log::info!("rew minter {} {}", n, rew_minter);

				if rew_non_minter > rew_minter {
					non_minter_higher_rewards_counter += 1;
					higher_rewards_cumulative += rew_minter * 10000 / rew_non_minter;
				}
			}
		}
		log::info!(
			"times minting user had lower rewards {}   avg minter/nonminter * 10000  {}",
			non_minter_higher_rewards_counter,
			higher_rewards_cumulative / non_minter_higher_rewards_counter
		);
	});
}

// starting point, user claimed some rewards, then new rewards were generated (blue)
#[test]
fn rewards_storage_right_amounts_start1() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::transfer(
			0,
			&2,
			&<Test as Config>::LiquidityMiningIssuanceVault::get(),
			10000000000,
			ExistenceRequirement::AllowDeath
		)
		.unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 1, 10000, 2, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(0));

		TokensOf::<Test>::transfer(1, &2, &3, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &3, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &4, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &4, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &5, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &5, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &6, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &6, 20010, ExistenceRequirement::AllowDeath).unwrap();
		ProofOfStake::activate_liquidity_v2(RuntimeOrigin::signed(2), 4, 10000, None).unwrap();
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(3), 1, 2, 10000, 10010).unwrap();
		mint_and_activate_tokens(3, 4, 10000);
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(4), 1, 2, 10000, 10010).unwrap();
		mint_and_activate_tokens(4, 4, 10000);
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(5), 1, 2, 10000, 10010).unwrap();
		mint_and_activate_tokens(5, 4, 10000);
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(6), 1, 2, 10000, 10010).unwrap();
		mint_and_activate_tokens(6, 4, 10000);
 
		// System::set_block_number(100);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(10));
		forward_to_block_with_custom_rewards(100, 50000); // No clue why we considr 50k rewards per
														  // block here
		assert_eq!(U256::from(u128::MAX) * U256::from(10), CumulativeTotalLiquidityToRewardsRatio::<Test>::get().get(&4).unwrap().rewards);

		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(2), 4).unwrap();
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(3), 4).unwrap();
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(4), 4).unwrap();
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(5), 4).unwrap();
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(6), 4).unwrap();

		let mut rewards_info = ProofOfStake::get_rewards_info(2, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 14704);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 14704);
		rewards_info = ProofOfStake::get_rewards_info(4, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 14704);
		rewards_info = ProofOfStake::get_rewards_info(5, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 14704);
		rewards_info = ProofOfStake::get_rewards_info(6, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 14704);

		// System::set_block_number(200);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(20));
		forward_to_block_with_custom_rewards(200, 50000);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(5, 4).unwrap(), 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(6, 4).unwrap(), 36530);

		// starting point for blue cases

		// usecase 3 claim (all)
		let mut user_balance_before = TokensOf::<Test>::free_balance(0, &2);
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(2), 4).unwrap();
		let mut user_balance_after = TokensOf::<Test>::free_balance(0, &2);
		rewards_info = ProofOfStake::get_rewards_info(2, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 51234);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);
		assert_eq!(user_balance_after - user_balance_before, 36530);

		// usecase 6 burn some
		user_balance_before = TokensOf::<Test>::free_balance(0, &3);
		// XykStorage::burn_liquidity(RuntimeOrigin::signed(3), 1, 2, 5000).unwrap();
		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(3), 4, 5000).unwrap();

		user_balance_after = TokensOf::<Test>::free_balance(0, &3);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530); // total rewards 51234, while 14704 were already claimed. Burning puts all rewards to not_yet_claimed, but zeroes the already_claimed. 51234 - 14704 = 36530
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 36530);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 7 mint some
		user_balance_before = TokensOf::<Test>::free_balance(0, &4);
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(4), 1, 2, 5000, 5010).unwrap();
		mint_and_activate_tokens(4, 4, 5000);
		user_balance_after = TokensOf::<Test>::free_balance(0, &4);
		rewards_info = ProofOfStake::get_rewards_info(4, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 36530);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 8 deactivate some
		user_balance_before = TokensOf::<Test>::free_balance(0, &5);
		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(5), 4, 5000).unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &5);
		rewards_info = ProofOfStake::get_rewards_info(5, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(5, 4).unwrap(), 36530);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 16 claim some
		user_balance_before = TokensOf::<Test>::free_balance(0, &6);
		// ProofOfStake::claim_rewards_v2(RuntimeOrigin::signed(6), 4, 20000).unwrap();
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(6), 4).unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &6);
		rewards_info = ProofOfStake::get_rewards_info(6, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		// assert_eq!(rewards_info.rewards_already_claimed, 34704);
		assert_eq!(rewards_info.rewards_already_claimed, 14704 + 36530);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(6, 4).unwrap(), 16530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(6, 4).unwrap(), 0);
		// assert_eq!(user_balance_after - user_balance_before, 20000);
		assert_eq!(user_balance_after - user_balance_before, 36530);
	});
}

// starting point, user burned some rewards, then new rewards were generated (yellow)
#[test]
fn rewards_storage_right_amounts_start2() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::transfer(
			0,
			&2,
			&<Test as Config>::LiquidityMiningIssuanceVault::get(),
			10000000000,
			ExistenceRequirement::AllowDeath,
		)
		.unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 1, 10000, 2, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		TokensOf::<Test>::transfer(1, &2, &3, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &3, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &4, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &4, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &5, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &5, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &6, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &6, 20010, ExistenceRequirement::AllowDeath).unwrap();
		ProofOfStake::activate_liquidity_v2(RuntimeOrigin::signed(2), 4, 10000, None).unwrap();
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(3), 1, 2, 10000, 10010).unwrap();
		mint_and_activate_tokens(3, 4, 10000);
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(4), 1, 2, 10000, 10010).unwrap();
		mint_and_activate_tokens(4, 4, 10000);
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(5), 1, 2, 10000, 10010).unwrap();
		mint_and_activate_tokens(5, 4, 10000);

		// System::set_block_number(100);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(10));
		forward_to_block_with_custom_rewards(100, 40000);
		assert_eq!(U256::from(u128::MAX) * U256::from(10), CumulativeTotalLiquidityToRewardsRatio::<Test>::get().get(&4).unwrap().rewards);

		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(2), 4, 5000).unwrap();
		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(3), 4, 5000).unwrap();
		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(4), 4, 5000).unwrap();
		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(5), 4, 5000).unwrap();

		// System::set_block_number(200);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(20));
		forward_to_block_with_custom_rewards(200, 20000); //TODO: its really weird that rewards are
														  //decreased from 40k to 20k in single
														  //test?
		assert_eq!(U256::from(u128::MAX) * U256::from(20), CumulativeTotalLiquidityToRewardsRatio::<Test>::get().get(&4).unwrap().rewards);

		let mut rewards_info = ProofOfStake::get_rewards_info(2, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 14704);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 14704);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		rewards_info = ProofOfStake::get_rewards_info(4, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 14704);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		rewards_info = ProofOfStake::get_rewards_info(5, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 14704);
		assert_eq!(rewards_info.rewards_already_claimed, 0);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 32973);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 32973);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 32973);
		assert_eq!(ProofOfStake::calculate_rewards_amount(5, 4).unwrap(), 32973);

		// starting point for blue cases

		// usecase 2 claim_all
		let mut user_balance_before = TokensOf::<Test>::free_balance(0, &2);
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(2), 4).unwrap();
		let mut user_balance_after = TokensOf::<Test>::free_balance(0, &2);
		rewards_info = ProofOfStake::get_rewards_info(2, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 18269);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);
		assert_eq!(user_balance_after - user_balance_before, 32973);

		// usecase 9 burn some
		user_balance_before = TokensOf::<Test>::free_balance(0, &3);
		// XykStorage::burn_liquidity(RuntimeOrigin::signed(3), 4, 5000).unwrap();
		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(3), 4, 5000).unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &3);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 32973);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 32973);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 10 mint some
		user_balance_before = TokensOf::<Test>::free_balance(0, &4);
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(4), 1, 2, 5000, 5010).unwrap();
		mint_and_activate_tokens(4, 4, 5000);
		user_balance_after = TokensOf::<Test>::free_balance(0, &4);
		rewards_info = ProofOfStake::get_rewards_info(4, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 32973);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 32973);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 11 deactivate some
		user_balance_before = TokensOf::<Test>::free_balance(0, &5);
		ProofOfStake::deactivate_liquidity_v2(RuntimeOrigin::signed(5), 4, 5000).unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &5);
		rewards_info = ProofOfStake::get_rewards_info(5, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 32973);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(5, 4).unwrap(), 32973);
		assert_eq!(user_balance_after - user_balance_before, 0);
	});
}

// starting point, just new rewards were generated (green)
#[test]
fn rewards_storage_right_amounts_start3() {
	new_test_ext().execute_with(|| {
		// MockPromotedPoolApi::instance().lock().unwrap().clear();
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::transfer(
			0,
			&2,
			&<Test as Config>::LiquidityMiningIssuanceVault::get(),
			10000000000,
			ExistenceRequirement::AllowDeath
		)
		.unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 1, 10000, 2, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(0));

		TokensOf::<Test>::transfer(1, &2, &3, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &3, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &4, 20010, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(2, &2, &4, 20010, ExistenceRequirement::AllowDeath).unwrap();

		ProofOfStake::activate_liquidity_v2(RuntimeOrigin::signed(2), 4, 10000, None).unwrap();
		// XykStorage::mint_liquidity(RuntimeOrigin::signed(3), 1, 2, 10000, 10010).unwrap();
		mint_and_activate_tokens(3, 4, 10000);
		
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * U256::from(10));
		forward_to_block_with_custom_rewards(100, 20000);

		let mut rewards_info = ProofOfStake::get_rewards_info(2, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 0);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 14704);

		// starting point for blue cases

		// usecase 1 claim (all)
		let mut user_balance_before = TokensOf::<Test>::free_balance(0, &2);
		// ProofOfStake::claim_rewards_v2(RuntimeOrigin::signed(2), 4, 14704).unwrap();
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(2), 4).unwrap();
		let mut user_balance_after = TokensOf::<Test>::free_balance(0, &2);
		rewards_info = ProofOfStake::get_rewards_info(2, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 14704);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);
		assert_eq!(user_balance_after - user_balance_before, 14704);

		// usecase 17 claim some
		user_balance_before = TokensOf::<Test>::free_balance(0, &3);
		// ProofOfStake::claim_rewards_v2(RuntimeOrigin::signed(3), 4, 10000).unwrap();
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(3), 4).unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &3);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		// assert_eq!(rewards_info.rewards_already_claimed, 10000);
		assert_eq!(rewards_info.rewards_already_claimed, 10000 + 4704);
		// assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 4704);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 0);
		// assert_eq!(user_balance_after - user_balance_before, 10000);
		assert_eq!(user_balance_after - user_balance_before, 10000 + 4704);
	});
}


//E2E test cases
//------------------------------------------------------------------
// [Happy path] A user obtain extra tokens from asymtotic_curve_rewards
//	Contained in liquidity_rewards_single_user_mint_W

// [Accuracy] A user obtain the right tokens from asymtotic_curve_rewards
//	Contained in liquidity_rewards_single_user_mint_W

// [Accuracy] A user obtain the right tokens from asymtotic_curve_rewards when burn half of those
// Contained in liquidity_rewards_work_after_burn_W

// A user that mints two times, at different period gets the sum of those two kinds of asymtotic_curve_rewards
// Contained in liquidity_rewards_three_users_mint_W

// A user that got transfered Liq.tokens, can request asymtotic_curve_rewards
#[test]
fn liquidity_rewards_transfered_liq_tokens_produce_rewards_W() {
	new_test_ext().execute_with(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::transfer(
			0,
			&2,
			&<Test as Config>::LiquidityMiningIssuanceVault::get(),
			10000000000,
			ExistenceRequirement::AllowDeath
		)
		.unwrap();

		// XykStorage::create_pool(RuntimeOrigin::signed(2), 0, 10000, 1, 10000).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 2u8).unwrap();

		// MockPromotedPoolApi::instance().lock().unwrap().insert(4, U256::from(0));

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);

		TokensOf::<Test>::transfer(4, &2, &3, liquidity_tokens_owned, ExistenceRequirement::AllowDeath).unwrap();

		ProofOfStake::activate_liquidity_v2(
			RuntimeOrigin::signed(3),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		// System::set_block_number(100);
		// MockPromotedPoolApi::instance()
		// 	.lock()
		// 	.unwrap()
		// 	.insert(4, U256::from(u128::MAX) * 100000 / 10000);
		forward_to_block(100);

		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 14704);
		ProofOfStake::claim_rewards_all_v2(RuntimeOrigin::signed(3), 4).unwrap();
	});
}

pub(crate) fn roll_to_while_minting(n: u64, expected_amount_minted: Option<Balance>) {
	let mut session_number: u32;
	let mut session_issuance: (Balance, Balance);
	let mut block_issuance: Balance;
	while System::block_number() < n {
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		session_number = System::block_number().saturated_into::<u32>() / BlocksPerRound::get();
		session_issuance = <Issuance as GetIssuance>::get_all_issuance(session_number)
			.expect("session issuance is always populated in advance");
		block_issuance = (session_issuance.0 + session_issuance.1) /
			(BlocksPerRound::get().saturated_into::<u128>());

		if let Some(x) = expected_amount_minted {
			assert_eq!(x, block_issuance);
		}

		// Compute issuance for the next session only after all issuance has been issued is current session
		// To avoid overestimating the missing issuance and overshooting the cap
		if ((System::block_number().saturated_into::<u32>() + 1u32) % BlocksPerRound::get()) == 0 {
			<Issuance as ComputeIssuance>::compute_issuance(session_number + 1u32);
		}
	}
}

#[test]
fn test_migrated_from_pallet_issuance() {
	new_test_ext().execute_with(|| {
		env_logger::init();
		System::set_block_number(1);

		let token_id = TokensOf::<Test>::create(&99999, 2000_000_000).unwrap();
		assert_eq!(token_id, 0);

		let current_issuance = TokensOf::<Test>::total_issuance(token_id);
		let target_tge = 2_000_000_000u128;
		assert!(current_issuance <= target_tge);

		assert_ok!(TokensOf::<Test>::mint(token_id, &99999, target_tge - current_issuance));

		assert_ok!(Issuance::finalize_tge(RuntimeOrigin::root()));
		assert_ok!(Issuance::init_issuance_config(RuntimeOrigin::root()));
		assert_ok!(Issuance::calculate_and_store_round_issuance(0u32));

		assert_eq!(1, TokensOf::<Test>::create(&99999, 1_000_000u128).unwrap());
		ProofOfStake::enable(1, 1u8);
		ProofOfStake::activate_liquidity(99999, 1, 1, None).unwrap();

		roll_to_while_minting(4, None);
		assert_eq!(
			U256::from_dec_str("76571018769283414925455480913511346478027010").unwrap(),
			ProofOfStake::get_pool_rewards(1).unwrap()
		);
		roll_to_while_minting(9, None);
		assert_eq!(
			U256::from_dec_str("153142037538566829850910961827022692956054020").unwrap(),
			ProofOfStake::get_pool_rewards(1).unwrap()
		);

		assert_eq!(2, TokensOf::<Test>::create(&99999, 1_000_000u128).unwrap());
		ProofOfStake::enable(2, 1u8);
		ProofOfStake::activate_liquidity(99999, 2, 1, None).unwrap();
		//	assert_eq!(2, Issuance::len());
		roll_to_while_minting(14, None);
		assert_eq!(
			U256::from_dec_str("191427546923208537313638702283778366195067525").unwrap(),
			ProofOfStake::get_pool_rewards(1).unwrap()
		);
		assert_eq!(
			U256::from_dec_str("38285509384641707462727740456755673239013505").unwrap(),
			ProofOfStake::get_pool_rewards(2).unwrap()
		);

		roll_to_while_minting(19, None);
		assert_eq!(
			U256::from_dec_str("229713056307850244776366442740534039434081030").unwrap(),
			ProofOfStake::get_pool_rewards(1).unwrap()
		);
		assert_eq!(
			U256::from_dec_str("76571018769283414925455480913511346478027010").unwrap(),
			ProofOfStake::get_pool_rewards(2).unwrap()
		);
	});
}
