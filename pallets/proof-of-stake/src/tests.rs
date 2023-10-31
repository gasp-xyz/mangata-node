// Copyright (C) 2020 Mangata team
#![cfg(not(feature = "runtime-benchmarks"))]
#![allow(non_snake_case)]

use super::*;
use crate::mock::*;
use frame_support::{assert_err, assert_err_ignore_postinfo, assert_ok};
use mockall::predicate::eq;
use serial_test::serial;

use mangata_support::traits::{ComputeIssuance, GetIssuance};

type TokensOf<Test> = <Test as Config>::Currency;

fn mint_and_activate_tokens(who: AccountId, token_id: TokenId, amount: Balance) {
	TokensOf::<Test>::mint(token_id, &who, amount).unwrap();
	ProofOfStake::activate_liquidity_for_native_rewards(
		RuntimeOrigin::signed(who),
		token_id,
		amount,
		None,
	)
	.unwrap();
}

fn initialize_liquidity_rewards() {
	System::set_block_number(1);
	let acc_id: u128 = 2;
	let amount: u128 = std::u128::MAX;
	PromotedPoolRewards::<Test>::get();
	TokensOf::<Test>::create(&acc_id, amount).unwrap();
	TokensOf::<Test>::create(&acc_id, amount).unwrap();
	TokensOf::<Test>::create(&acc_id, amount).unwrap();
	TokensOf::<Test>::create(&acc_id, amount).unwrap();
	TokensOf::<Test>::create(&acc_id, 10000).unwrap();

	ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 2u8).unwrap();
	PromotedPoolRewards::<Test>::mutate(|pools| {
		pools.get_mut(&4).unwrap().rewards = U256::from(0);
	});

	ProofOfStake::activate_liquidity_for_native_rewards(RuntimeOrigin::signed(2), 4, 10000, None)
		.unwrap();
}

fn process_all_schedules_in_current_session() {
	let session = ProofOfStake::session_index();
	loop {
		if ProofOfStake::session_index() > session {
			panic!("couldnt process all schedules within the session");
		}

		if !ProofOfStake::is_new_session() && ProofOfStake::pos() == ProofOfStake::tail() {
			break
		}
		roll_to_next_block();
	}
}

fn roll_to_next_block() {
	forward_to_block((System::block_number() + 1).saturated_into::<u32>());
}

fn roll_to_next_session() {
	let current_session = ProofOfStake::session_index();
	roll_to_session(current_session + 1);
}

pub fn roll_to_session(n: u32) {
	while ProofOfStake::session_index() < n {
		roll_to_next_block();
	}
}

fn forward_to_block(n: u32) {
	forward_to_block_with_custom_rewards(n, 10000);
}

fn forward_to_block_with_custom_rewards(n: u32, rewards: u128) {
	while System::block_number().saturated_into::<u32>() < n {
		let new_block_number = System::block_number().saturated_into::<u64>() + 1;
		System::set_block_number(new_block_number);

		System::on_initialize(new_block_number);
		ProofOfStake::on_initialize(new_block_number);

		if ProofOfStake::is_new_session() {
			ProofOfStake::distribute_rewards(rewards);
		}

		ProofOfStake::on_finalize(new_block_number);
		System::on_finalize(new_block_number);
	}
}

#[test]
#[serial]
fn liquidity_rewards_single_user_mint_W() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::create(&acc_id, 10000).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);

		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 2u8).unwrap();
		ProofOfStake::activate_liquidity_for_native_rewards(
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

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);

		System::set_block_number(10);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 291);
		System::set_block_number(20);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 873);
		System::set_block_number(30);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 1716);
		System::set_block_number(40);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 2847);
		System::set_block_number(50);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 4215);
		System::set_block_number(60);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 5844);
		System::set_block_number(70);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 7712);
		System::set_block_number(80);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 9817);
		System::set_block_number(90);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 12142);
		System::set_block_number(100);
		ProofOfStake::distribute_rewards(10000 * 1);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);
	});
}

#[test]
#[serial]
fn liquidity_rewards_three_users_burn_W() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();

		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		TokensOf::<Test>::transfer(0, &2, &3, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &3, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(0, &2, &4, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &4, 1000000, ExistenceRequirement::AllowDeath).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		forward_to_block(100);

		mint_and_activate_tokens(3, 4, 10000);

		forward_to_block(200);

		mint_and_activate_tokens(4, 4, 10000);
		forward_to_block(240);

		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(4), 4, 5000)
			.unwrap();
		forward_to_block(400);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 95965);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 44142);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 10630);
	});
}

#[test]
#[serial]
fn liquidity_rewards_claim_W() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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

		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();
		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		forward_to_block(10);
		forward_to_block(90);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 12142);
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(2), 4).unwrap();

		forward_to_block(100);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 2562);
	});
}

#[test]
#[serial]
fn liquidity_rewards_promote_pool_W() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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
	});
}

#[test]
#[serial]
fn liquidity_rewards_promote_pool_already_promoted_NW() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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
#[serial]
fn liquidity_rewards_work_after_burn_W() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		TokensOf::<Test>::transfer(0, &2, &3, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &3, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(0, &2, &4, 1000000, ExistenceRequirement::AllowDeath).unwrap();
		TokensOf::<Test>::transfer(1, &2, &4, 1000000, ExistenceRequirement::AllowDeath).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		forward_to_block(100);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);

		mint_and_activate_tokens(3, 4, 10000);
		forward_to_block(200);

		mint_and_activate_tokens(4, 4, 10000);
		forward_to_block(240);

		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(4), 4, 10000)
			.unwrap();
		forward_to_block(400);

		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 948);

		mint_and_activate_tokens(4, 4, 20000);
		forward_to_block(500);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 8299);
	});
}

#[test]
#[serial]
fn liquidity_rewards_deactivate_transfer_controled_W() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);

		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();
		assert_err!(
			TokensOf::<Test>::transfer(4, &2, &3, 10, ExistenceRequirement::AllowDeath),
			orml_tokens::Error::<Test>::BalanceTooLow,
		);

		forward_to_block(100);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);

		ProofOfStake::deactivate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
		)
		.unwrap();
		TokensOf::<Test>::transfer(4, &2, &3, 10, ExistenceRequirement::AllowDeath).unwrap();
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 14704);
	});
}

#[test]
#[serial]
fn liquidity_rewards_deactivate_more_NW() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();
		assert_err!(
			ProofOfStake::deactivate_liquidity_for_native_rewards(
				RuntimeOrigin::signed(2),
				4,
				liquidity_tokens_owned + 1
			),
			Error::<Test>::NotEnoughAssets
		);
	});
}

#[test]
#[serial]
fn liquidity_rewards_activate_more_NW() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);
		assert_err!(
			ProofOfStake::activate_liquidity_for_native_rewards(
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
#[serial]
fn liquidity_rewards_calculate_rewards_pool_not_promoted() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		assert_err!(ProofOfStake::calculate_rewards_amount(2, 4), Error::<Test>::NotAPromotedPool);
	});
}

#[test]
#[serial]
fn liquidity_rewards_claim_pool_not_promoted() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let max = std::u128::MAX;
		System::set_block_number(1);
		let acc_id: u128 = 2;
		let amount: u128 = max;

		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();
		TokensOf::<Test>::create(&acc_id, amount).unwrap();

		assert_err!(
			ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(2), 7),
			Error::<Test>::NotAPromotedPool,
		);
	});
}

#[test]
#[serial]
fn liquidity_rewards_transfer_not_working() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		initialize_liquidity_rewards();

		assert_err!(
			TokensOf::<Test>::transfer(4, &2, &3, 10, ExistenceRequirement::AllowDeath),
			orml_tokens::Error::<Test>::BalanceTooLow,
		);
	});
}

#[test]
#[serial]
fn liquidity_rewards_not_yet_claimed_already_claimed_W() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		forward_to_block(10);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 291);
		ProofOfStake::deactivate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
		)
		.unwrap();

		let rewards_info = ProofOfStake::get_rewards_info(2, 4);
		assert_eq!(rewards_info.rewards_not_yet_claimed, 291);

		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		forward_to_block(100);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 12433);
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(2), 4).unwrap();

		let rewards_info = ProofOfStake::get_rewards_info(2, 4);
		assert_eq!(rewards_info.rewards_already_claimed, 12142);
	});
}

#[test]
#[serial]
fn extreme_case_pool_ratio() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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

		ProofOfStake::activate_liquidity_for_native_rewards(RuntimeOrigin::signed(2), 4, 1, None)
			.unwrap();

		PromotedPoolRewards::<Test>::mutate(|pools| {
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
#[serial]
fn rewards_rounding_during_often_mint() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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

		let calculate_liq_tokens_amount = |first_amount: u128, second_amount: u128| -> u128 {
			((first_amount / 2) + (second_amount / 2)).try_into().unwrap()
		};
		TokensOf::<Test>::create(
			&acc_id,
			calculate_liq_tokens_amount(250000000000000000000000000, 10000000000000000),
		)
		.unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 1u8).unwrap();
		TokensOf::<Test>::transfer(
			0,
			&2,
			&3,
			10000000000000000000000000,
			ExistenceRequirement::AllowDeath,
		)
		.unwrap();
		TokensOf::<Test>::transfer(
			1,
			&2,
			&3,
			10000000000000000000000000,
			ExistenceRequirement::AllowDeath,
		)
		.unwrap();
		mint_and_activate_tokens(
			2,
			4,
			calculate_liq_tokens_amount(25000000000000000000000, 2000000000000),
		);
		mint_and_activate_tokens(
			3,
			4,
			calculate_liq_tokens_amount(25000000000000000000000, 2000000000000),
		);

		let mut non_minter_higher_rewards_counter = 0;
		let mut higher_rewards_cumulative = 0;
		for n in 1..10000 {
			System::set_block_number(n);
			if ProofOfStake::is_new_session() {
				ProofOfStake::distribute_rewards(10000);

				mint_and_activate_tokens(
					3,
					4,
					calculate_liq_tokens_amount(34000000000000000000, 68000000000000000000),
				);
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

#[test]
#[serial]
fn rewards_storage_right_amounts_start1() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			10000,
			None,
		)
		.unwrap();
		mint_and_activate_tokens(3, 4, 10000);
		mint_and_activate_tokens(4, 4, 10000);
		mint_and_activate_tokens(5, 4, 10000);
		mint_and_activate_tokens(6, 4, 10000);

		forward_to_block_with_custom_rewards(100, 50000); // No clue why we considr 50k rewards per
		assert_eq!(
			U256::from(u128::MAX) * U256::from(10),
			PromotedPoolRewards::<Test>::get().get(&4).unwrap().rewards
		);

		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(2), 4).unwrap();
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(3), 4).unwrap();
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(4), 4).unwrap();
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(5), 4).unwrap();
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(6), 4).unwrap();

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

		forward_to_block_with_custom_rewards(200, 50000);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(5, 4).unwrap(), 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(6, 4).unwrap(), 36530);

		// starting point for blue cases

		// usecase 3 claim (all)
		let mut user_balance_before = TokensOf::<Test>::free_balance(0, &2);
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(2), 4).unwrap();
		let mut user_balance_after = TokensOf::<Test>::free_balance(0, &2);
		rewards_info = ProofOfStake::get_rewards_info(2, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 51234);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);
		assert_eq!(user_balance_after - user_balance_before, 36530);

		// usecase 6 burn some
		user_balance_before = TokensOf::<Test>::free_balance(0, &3);
		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(3), 4, 5000)
			.unwrap();

		user_balance_after = TokensOf::<Test>::free_balance(0, &3);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530); // total rewards 51234, while 14704 were already claimed. Burning puts all rewards to not_yet_claimed, but zeroes the already_claimed. 51234 - 14704 = 36530
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 36530);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 7 mint some
		user_balance_before = TokensOf::<Test>::free_balance(0, &4);
		mint_and_activate_tokens(4, 4, 5000);
		user_balance_after = TokensOf::<Test>::free_balance(0, &4);
		rewards_info = ProofOfStake::get_rewards_info(4, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 36530);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 8 deactivate some
		user_balance_before = TokensOf::<Test>::free_balance(0, &5);
		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(5), 4, 5000)
			.unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &5);
		rewards_info = ProofOfStake::get_rewards_info(5, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 36530);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(5, 4).unwrap(), 36530);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 16 claim some
		user_balance_before = TokensOf::<Test>::free_balance(0, &6);
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(6), 4).unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &6);
		rewards_info = ProofOfStake::get_rewards_info(6, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 14704 + 36530);
		assert_eq!(ProofOfStake::calculate_rewards_amount(6, 4).unwrap(), 0);
		assert_eq!(user_balance_after - user_balance_before, 36530);
	});
}

// starting point, user burned some rewards, then new rewards were generated (yellow)
#[test]
#[serial]
fn rewards_storage_right_amounts_start2() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			10000,
			None,
		)
		.unwrap();
		mint_and_activate_tokens(3, 4, 10000);
		mint_and_activate_tokens(4, 4, 10000);
		mint_and_activate_tokens(5, 4, 10000);

		forward_to_block_with_custom_rewards(100, 40000);
		assert_eq!(
			U256::from(u128::MAX) * U256::from(10),
			PromotedPoolRewards::<Test>::get().get(&4).unwrap().rewards
		);

		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(2), 4, 5000)
			.unwrap();
		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(3), 4, 5000)
			.unwrap();
		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(4), 4, 5000)
			.unwrap();
		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(5), 4, 5000)
			.unwrap();

		forward_to_block_with_custom_rewards(200, 20000); //its really weird that rewards are
												  //decreased from 40k to 20k in single
		assert_eq!(
			U256::from(u128::MAX) * U256::from(20),
			PromotedPoolRewards::<Test>::get().get(&4).unwrap().rewards
		);

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
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(2), 4).unwrap();
		let mut user_balance_after = TokensOf::<Test>::free_balance(0, &2);
		rewards_info = ProofOfStake::get_rewards_info(2, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 18269);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);
		assert_eq!(user_balance_after - user_balance_before, 32973);

		// usecase 9 burn some
		user_balance_before = TokensOf::<Test>::free_balance(0, &3);
		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(3), 4, 5000)
			.unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &3);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 32973);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 32973);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 10 mint some
		user_balance_before = TokensOf::<Test>::free_balance(0, &4);
		mint_and_activate_tokens(4, 4, 5000);
		user_balance_after = TokensOf::<Test>::free_balance(0, &4);
		rewards_info = ProofOfStake::get_rewards_info(4, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 32973);
		assert_eq!(rewards_info.rewards_already_claimed, 0);
		assert_eq!(ProofOfStake::calculate_rewards_amount(4, 4).unwrap(), 32973);
		assert_eq!(user_balance_after - user_balance_before, 0);

		// usecase 11 deactivate some
		user_balance_before = TokensOf::<Test>::free_balance(0, &5);
		ProofOfStake::deactivate_liquidity_for_native_rewards(RuntimeOrigin::signed(5), 4, 5000)
			.unwrap();
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
#[serial]
fn rewards_storage_right_amounts_start3() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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

		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			10000,
			None,
		)
		.unwrap();
		mint_and_activate_tokens(3, 4, 10000);

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
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(2), 4).unwrap();
		let mut user_balance_after = TokensOf::<Test>::free_balance(0, &2);
		rewards_info = ProofOfStake::get_rewards_info(2, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 14704);
		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);
		assert_eq!(user_balance_after - user_balance_before, 14704);

		// usecase 17 claim some
		user_balance_before = TokensOf::<Test>::free_balance(0, &3);
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(3), 4).unwrap();
		user_balance_after = TokensOf::<Test>::free_balance(0, &3);
		rewards_info = ProofOfStake::get_rewards_info(3, 4);

		assert_eq!(rewards_info.rewards_not_yet_claimed, 0);
		assert_eq!(rewards_info.rewards_already_claimed, 10000 + 4704);
		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 0);
		assert_eq!(user_balance_after - user_balance_before, 10000 + 4704);
	});
}

#[test]
#[serial]
fn liquidity_rewards_transfered_liq_tokens_produce_rewards_W() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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

		TokensOf::<Test>::create(&acc_id, 10000).unwrap();
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 2u8).unwrap();

		let liquidity_tokens_owned = TokensOf::<Test>::free_balance(4, &2);

		TokensOf::<Test>::transfer(
			4,
			&2,
			&3,
			liquidity_tokens_owned,
			ExistenceRequirement::AllowDeath,
		)
		.unwrap();

		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(3),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		forward_to_block(100);

		assert_eq!(ProofOfStake::calculate_rewards_amount(3, 4).unwrap(), 14704);
		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(3), 4).unwrap();
	});
}

#[test]
#[serial]
fn test_migrated_from_pallet_issuance() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		env_logger::try_init();
		System::set_block_number(1);
		const LIQUIDITY_ISSUANCE: Balance = 450045;

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
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(99999),
			1,
			1,
			None,
		)
		.unwrap();

		forward_to_block_with_custom_rewards(9, LIQUIDITY_ISSUANCE);
		assert_eq!(
			U256::from_dec_str("153142377820933750789374425201630124724265475").unwrap(),
			ProofOfStake::get_pool_rewards(1).unwrap()
		);
		forward_to_block_with_custom_rewards(19, LIQUIDITY_ISSUANCE);
		assert_eq!(
			U256::from_dec_str("306284755641867501578748850403260249448530950").unwrap(),
			ProofOfStake::get_pool_rewards(1).unwrap()
		);

		assert_eq!(2, TokensOf::<Test>::create(&99999, 1_000_000u128).unwrap());
		ProofOfStake::enable(2, 1u8);
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(99999),
			2,
			1,
			None,
		)
		.unwrap();
		forward_to_block_with_custom_rewards(29, LIQUIDITY_ISSUANCE);
		assert_eq!(
			U256::from_dec_str("382855774411150916504204331316771595926557960").unwrap(),
			ProofOfStake::get_pool_rewards(1).unwrap()
		);
		assert_eq!(
			U256::from_dec_str("76571018769283414925455480913511346478027010").unwrap(),
			ProofOfStake::get_pool_rewards(2).unwrap()
		);

		forward_to_block_with_custom_rewards(39, LIQUIDITY_ISSUANCE);
		assert_eq!(
			U256::from_dec_str("459426793180434331429659812230282942404584970").unwrap(),
			ProofOfStake::get_pool_rewards(1).unwrap()
		);
		assert_eq!(
			U256::from_dec_str("153142037538566829850910961827022692956054020").unwrap(),
			ProofOfStake::get_pool_rewards(2).unwrap()
		);
	});
}

#[test]
#[serial]
fn claim_rewards_from_pool_that_has_been_disabled() {
	ExtBuilder::new().execute_with_default_mocks(|| {
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
		ProofOfStake::activate_liquidity_for_native_rewards(
			RuntimeOrigin::signed(2),
			4,
			liquidity_tokens_owned,
			None,
		)
		.unwrap();

		forward_to_block(10);

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 291);

		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), 4, 0u8).unwrap();

		ProofOfStake::claim_native_rewards(RuntimeOrigin::signed(2), 4).unwrap();

		assert_eq!(ProofOfStake::calculate_rewards_amount(2, 4).unwrap(), 0);
	});
}

const MILLION: u128 = 1_000_000;
const REWARDED_PAIR: (TokenId, TokenId) = (0u32, 4u32);
const FIRST_REWARDED_PAIR: (TokenId, TokenId) = (0u32, 4u32);
const SECOND_REWARDED_PAIR: (TokenId, TokenId) = (0u32, 5u32);
const REWARD_AMOUNT: u128 = 10_000u128;
const REWARD_TOKEN: u32 = 5u32;
const FIRST_REWARD_TOKEN: u32 = REWARD_TOKEN;
const SECOND_REWARD_TOKEN: u32 = 6u32;
const LIQUIDITY_TOKEN: u32 = 10;
const FIRST_LIQUIDITY_TOKEN: u32 = 10;
const SECOND_LIQUIDITY_TOKEN: u32 = 11;
const TOKEN_PAIRED_WITH_MGX: u32 = 15;
const ALICE: u128 = 2;
const BOB: u128 = 3;
const CHARLIE: u128 = 4;
const EVE: u128 = 5;


fn min_req_valutation() -> u128 {
 <<Test as Config>::Min3rdPartyRewardValutationPerSession as sp_core::Get<u128>>::get()
}


#[test]
#[serial]
fn user_can_provide_3rdparty_rewards() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.execute_with_default_mocks(|| {
			System::set_block_number(1);
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT / 2,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(5);
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT / 2,
				6u32.into(),
			)
			.unwrap();
		});
}

#[test]
#[serial]
fn cant_schedule_rewards_in_past() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			roll_to_session(5);
			assert_err!(
				ProofOfStake::reward_pool(
					RuntimeOrigin::signed(ALICE),
					REWARDED_PAIR,
					REWARD_TOKEN,
					REWARD_AMOUNT,
					1u32.into()
				),
				Error::<Test>::CannotScheduleRewardsInPast
			);
			assert_err!(
				ProofOfStake::reward_pool(
					RuntimeOrigin::signed(ALICE),
					REWARDED_PAIR,
					REWARD_TOKEN,
					REWARD_AMOUNT,
					4u32.into()
				),
				Error::<Test>::CannotScheduleRewardsInPast
			);
			assert_err!(
				ProofOfStake::reward_pool(
					RuntimeOrigin::signed(ALICE),
					REWARDED_PAIR,
					REWARD_TOKEN,
					REWARD_AMOUNT,
					5u32.into()
				),
				Error::<Test>::CannotScheduleRewardsInPast
			);
		});
}

#[test]
#[serial]
fn cannot_reward_unexisting_pool() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.build()
		.execute_with(|| {
			let get_liquidity_asset_mock = MockValuationApi::get_liquidity_asset_context();
			get_liquidity_asset_mock
				.expect()
				.return_const(Err(Error::<Test>::PoolDoesNotExist.into()));
			let valuate_liquidity_token_mock = MockValuationApi::valuate_liquidity_token_context();
			valuate_liquidity_token_mock.expect().return_const(11u128);

			assert_err!(
				ProofOfStake::reward_pool(
					RuntimeOrigin::signed(ALICE),
					REWARDED_PAIR,
					REWARD_TOKEN,
					REWARD_AMOUNT,
					5u32.into()
				),
				Error::<Test>::PoolDoesNotExist
			);
		});
}

#[test]
#[serial]
fn rewards_are_stored_in_pallet_account() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			assert_eq!(
				TokensOf::<Test>::free_balance(REWARD_TOKEN, &Pallet::<Test>::pallet_account()),
				0
			);
			assert_eq!(TokensOf::<Test>::free_balance(REWARD_TOKEN, &ALICE), REWARD_AMOUNT);

			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				5u32.into()
			),);

			assert_eq!(TokensOf::<Test>::free_balance(REWARD_TOKEN, &ALICE), 0);
			assert_eq!(
				TokensOf::<Test>::free_balance(REWARD_TOKEN, &Pallet::<Test>::pallet_account()),
				REWARD_AMOUNT
			);
		});
}

#[test]
#[serial]
fn rewards_schedule_is_stored() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				5u32.into()
			),);

			let rewards_per_session = REWARD_AMOUNT / 5;
			assert_eq!(
				RewardsSchedulesList::<Test>::get(0).unwrap(),
				(
					Schedule {
						scheduled_at: 0u64,
						last_session: 5u64,
						liq_token: LIQUIDITY_TOKEN,
						reward_token: REWARD_TOKEN,
						amount_per_session: rewards_per_session,
					},
					None
				)
			);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
		});
}

#[test]
#[serial]
fn rewards_linked_list_insert_multiple_schedules() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 2 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				1u32.into()
			),);

			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				2u32.into()
			),);

			assert_eq!(
				RewardsSchedulesList::<Test>::get(0).unwrap(),
				(
					Schedule {
						scheduled_at: 0u64,
						last_session: 1u64,
						liq_token: LIQUIDITY_TOKEN,
						reward_token: REWARD_TOKEN,
						amount_per_session: REWARD_AMOUNT / 1,
					},
					Some(1)
				)
			);

			assert_eq!(
				RewardsSchedulesList::<Test>::get(1).unwrap(),
				(
					Schedule {
						scheduled_at: 0u64,
						last_session: 2u64,
						liq_token: LIQUIDITY_TOKEN,
						reward_token: REWARD_TOKEN,
						amount_per_session: REWARD_AMOUNT / 2,
					},
					None
				)
			);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));

			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));
		});
}

#[test]
#[serial]
fn rewards_linked_list_removes_outdated_schedule_automatically() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 2 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				1u32.into()
			),);

			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				2u32.into()
			),);

			assert_ok!(RewardsSchedulesList::<Test>::get(0).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(1).ok_or(()));

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));

			forward_to_block(2);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListPos::<Test>::get(), Some(1u64));

			forward_to_block(5);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListPos::<Test>::get(), Some(1u64));

			forward_to_block(11);
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListPos::<Test>::get(), Some(1u64));

			forward_to_block(21);
			assert_eq!(ScheduleListHead::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListPos::<Test>::get(), Some(1u64));

			forward_to_block(25);
			assert_eq!(ScheduleListHead::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListPos::<Test>::get(), Some(1u64));

			forward_to_block(29);
			assert_eq!(ScheduleListHead::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));

			forward_to_block(30);
			assert_eq!(ScheduleListHead::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), None);
			assert_eq!(ScheduleListPos::<Test>::get(), None);
		});
}

// rewards_first_schedule_from_linked_list
// rewards_last_schedule_from_linked_list
// rewards_middle_schedule_from_linked_list
// rewards_multipleall_schedule_from_linked_list

fn insert_schedule_ending_at_session(n: u32) {
	assert_ok!(ProofOfStake::reward_pool(
		RuntimeOrigin::signed(ALICE),
		REWARDED_PAIR,
		REWARD_TOKEN,
		REWARD_AMOUNT,
		n.into(),
	),);
}

#[test]
#[serial]
fn rewards_first_schedule_from_linked_list_of_four() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 4 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(2);

			assert_ok!(RewardsSchedulesList::<Test>::get(0).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(1).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(2).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(3).ok_or(()));
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));

			forward_to_block(21);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(1u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(1u64).unwrap().1, Some(2u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(2u64).unwrap().1, Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(3u64).unwrap().1, None);
		});
}

#[test]
#[serial]
fn remove_last_schedule_from_linked_list() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 4 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(1);

			assert_ok!(RewardsSchedulesList::<Test>::get(0).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(1).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(2).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(3).ok_or(()));
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));

			forward_to_block(21);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(2u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(0u64).unwrap().1, Some(1u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(1u64).unwrap().1, Some(2u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(2u64).unwrap().1, None);
		});
}

#[test]
#[serial]
fn remove_middle_schedule_from_linked_list() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 4 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(2);

			assert_ok!(RewardsSchedulesList::<Test>::get(0).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(1).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(2).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(3).ok_or(()));
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));

			forward_to_block(21);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(0u64).unwrap().1, Some(2u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(2u64).unwrap().1, Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(3u64).unwrap().1, None);
		});
}

#[test]
#[serial]
fn remove_first_few_elems_at_once_from_linked_list() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 4 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(2);

			assert_ok!(RewardsSchedulesList::<Test>::get(0).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(1).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(2).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(3).ok_or(()));
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));

			forward_to_block(20);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(2u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(2u64).unwrap().1, Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(3u64).unwrap().1, None);
		});
}

#[test]
#[serial]
fn remove_few_last_elems_at_once_from_linked_list() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 4 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(1);

			assert_ok!(RewardsSchedulesList::<Test>::get(0).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(1).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(2).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(3).ok_or(()));
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));

			forward_to_block(21);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(0u64).unwrap().1, Some(1u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(1u64).unwrap().1, None);
		});
}

#[test]
#[serial]
fn remove_few_middle_elements_from_linkedd_list() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 4 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(2);

			assert_ok!(RewardsSchedulesList::<Test>::get(0).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(1).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(2).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(3).ok_or(()));
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));

			forward_to_block(21);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(0u64).unwrap().1, Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(3u64).unwrap().1, None);
		});
}

#[test]
#[serial]
fn remove_random_elements_from_linked_list() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 5 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(2);
			insert_schedule_ending_at_session(1);
			insert_schedule_ending_at_session(2);

			assert_ok!(RewardsSchedulesList::<Test>::get(0).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(1).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(2).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(3).ok_or(()));
			assert_ok!(RewardsSchedulesList::<Test>::get(4).ok_or(()));
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(4u64));

			forward_to_block(21);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(4u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(0u64).unwrap().1, Some(2u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(2u64).unwrap().1, Some(4u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(4u64).unwrap().1, None);
		});
}

#[test]
#[serial]
fn remove_random_elements_from_linked_list_over_time() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 7 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(3); // 0
			insert_schedule_ending_at_session(2); // 1
			insert_schedule_ending_at_session(1); // 2
			insert_schedule_ending_at_session(2); // 3
			insert_schedule_ending_at_session(2); // 4
			insert_schedule_ending_at_session(1); // 5
			insert_schedule_ending_at_session(3); // 6

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(6u64));

			roll_to_session(2);
			process_all_schedules_in_current_session();
			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(6u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(0u64).unwrap().1, Some(1u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(1u64).unwrap().1, Some(3u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(3u64).unwrap().1, Some(4u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(4u64).unwrap().1, Some(6u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(6u64).unwrap().1, None);

			roll_to_session(3);
			process_all_schedules_in_current_session();

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(6u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(0u64).unwrap().1, Some(6u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(6u64).unwrap().1, None);
		});
}

#[test]
#[serial]
fn remove_lot_of_schedules_from_linked_list_in_single_iteration() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 10 * REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			insert_schedule_ending_at_session(3); // 0
			insert_schedule_ending_at_session(1); // 1
			insert_schedule_ending_at_session(1); // 1
			insert_schedule_ending_at_session(1); // 1
			insert_schedule_ending_at_session(1); // 2
			insert_schedule_ending_at_session(1); // 3
			insert_schedule_ending_at_session(1); // 4
			insert_schedule_ending_at_session(1); // 5
			insert_schedule_ending_at_session(3); // 6

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListPos::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), Some(8u64));

			forward_to_block(24);

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(8u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(0u64).unwrap().1, Some(8u64));
			assert_eq!(RewardsSchedulesList::<Test>::get(8u64).unwrap().1, None);

			forward_to_block(100);
			assert_eq!(ScheduleListHead::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), None);
		});
}

#[test]
#[ignore]
#[serial]
fn number_of_active_schedules_is_limited() {
	ExtBuilder::new().issue(ALICE, REWARD_TOKEN, MILLION).execute_with_default_mocks(|| {
		System::set_block_number(1);





		let max_schedules: u32 =
			<<Test as Config>::RewardsSchedulesLimit as sp_core::Get<_>>::get();
		for i in 0..(max_schedules) {
			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				(5u32 + i).into()
			));
		}

		assert_err!(
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				100u32.into()
			),
			Error::<Test>::TooManySchedules
		);

		roll_to_session(10);

		assert_ok!(ProofOfStake::reward_pool(
			RuntimeOrigin::signed(ALICE),
			REWARDED_PAIR,
			REWARD_TOKEN,
			REWARD_AMOUNT,
			100u32.into()
		));
	});
}

#[test]
#[serial]
fn duplicated_schedules_works() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			assert_eq!(ScheduleListHead::<Test>::get(), None);
			assert_eq!(ScheduleListTail::<Test>::get(), None);

			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT / 2,
				5u32.into()
			));

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(0u64));

			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT / 2,
				5u32.into()
			));

			assert_eq!(ScheduleListHead::<Test>::get(), Some(0u64));
			assert_eq!(ScheduleListTail::<Test>::get(), Some(1u64));
		});
}

#[test]
#[serial]
fn reject_schedule_with_too_little_rewards_per_session() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.build()
		.execute_with(|| {
			System::set_block_number(1);
			let get_liquidity_asset_mock = MockValuationApi::get_liquidity_asset_context();
			get_liquidity_asset_mock.expect().return_const(Ok(10u32));
			let valuate_liquidity_token_mock = MockValuationApi::valuate_liquidity_token_context();
			valuate_liquidity_token_mock.expect().return_const(1u128);

			let valuate_non_liquidity_token_mock =
				MockValuationApi::valuate_non_liquidity_token_context();
			valuate_non_liquidity_token_mock.expect().return_const(0u128);

			roll_to_session(4);

			assert_err!(
				ProofOfStake::reward_pool(
					RuntimeOrigin::signed(ALICE),
					REWARDED_PAIR,
					REWARD_TOKEN,
					1,
					5u32.into()
				),
				Error::<Test>::TooLittleRewards
			);
		});
}

#[test]
#[serial]
fn accept_schedule_valuated_in_native_token() {
	ExtBuilder::new()
		.issue(ALICE, ProofOfStake::native_token_id(), REWARD_AMOUNT)
		.build()
		.execute_with(|| {
			System::set_block_number(1);
			let get_liquidity_asset_mock = MockValuationApi::get_liquidity_asset_context();
			get_liquidity_asset_mock.expect().return_const(Ok(10u32));
			let valuate_liquidity_token_mock = MockValuationApi::valuate_liquidity_token_context();
			valuate_liquidity_token_mock.expect().return_const(0u128);
			let get_pool_state_mock = MockValuationApi::get_pool_state_context();
			get_pool_state_mock.expect().return_const(Some((min_req_valutation(),min_req_valutation())));

			roll_to_session(4);

			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				ProofOfStake::native_token_id(),
				10,
				5u32.into()
			),);
		});
}

#[test]
#[serial]
fn accept_schedule_valuated_in_token_paired_with_native_token() {
	ExtBuilder::new()
		.issue(ALICE, ProofOfStake::native_token_id(), REWARD_AMOUNT)
		.issue(ALICE, TOKEN_PAIRED_WITH_MGX, REWARD_AMOUNT)
		.build()
		.execute_with(|| {
			System::set_block_number(1);
			let get_liquidity_asset_mock = MockValuationApi::get_liquidity_asset_context();
			get_liquidity_asset_mock.expect().return_const(Ok(10u32));

			let valuate_liquidity_token_mock = MockValuationApi::valuate_liquidity_token_context();
			valuate_liquidity_token_mock.expect().return_const(0u128);

			let valuate_non_liquidity_token_mock =
				MockValuationApi::valuate_non_liquidity_token_context();
			valuate_non_liquidity_token_mock.expect().return_const(10u128);
			let get_pool_state_mock = MockValuationApi::get_pool_state_context();
			get_pool_state_mock.expect().return_const(Some((min_req_valutation(),min_req_valutation())));

			roll_to_session(4);

			assert_ok!(ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				TOKEN_PAIRED_WITH_MGX,
				REWARD_AMOUNT,
				5u32.into()
			),);
		});
}

#[test]
#[serial]
fn user_can_claim_3rdparty_rewards() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)
		.issue(CHARLIE, LIQUIDITY_TOKEN, 100)
		.issue(EVE, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &BOB, 100).unwrap();
			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &CHARLIE, 100).unwrap();
			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &EVE, 100).unwrap();

			let amount = 10_000u128;

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(1);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(0)
			);

			roll_to_session(2);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(CHARLIE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				),
				Ok(0)
			);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(1000)
			);

			roll_to_session(3);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(1500)
			);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				),
				Ok(500)
			);
		});
}

#[test]
#[serial]
fn overlapping_3rdparty_rewards_works() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.build()
		.execute_with(|| {
			System::set_block_number(1);
			const LIQUIDITY_TOKEN: u32 = 5;
			let get_liquidity_asset_mock = MockValuationApi::get_liquidity_asset_context();
			get_liquidity_asset_mock.expect().return_const(Ok(LIQUIDITY_TOKEN));
			let valuate_liquidity_token_mock = MockValuationApi::valuate_liquidity_token_context();
			valuate_liquidity_token_mock.expect().return_const(11u128);
			let get_pool_state_mock = MockValuationApi::get_pool_state_context();
			get_pool_state_mock.expect().return_const(Some((min_req_valutation(),min_req_valutation())));

			let first_reward_token = TokensOf::<Test>::create(&ALICE, MILLION).unwrap();
			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &BOB, 200).unwrap();

			let pair: (TokenId, TokenId) = (0u32.into(), 4u32.into());
			let amount = 10_000u128;

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				pair,
				first_reward_token,
				amount,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(1);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				first_reward_token,
				None,
			)
			.unwrap();

			roll_to_session(5);
			let second_reward_token_id = TokensOf::<Test>::create(&ALICE, MILLION).unwrap();
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				pair,
				second_reward_token_id,
				100_000,
				15u32.into(),
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				second_reward_token_id,
				None,
			)
			.unwrap();

			roll_to_session(7);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					second_reward_token_id
				),
				Ok(10000)
			);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					first_reward_token
				),
				Ok(6000)
			);
		});
}

#[test]
#[serial]
fn reuse_activated_liquiddity_tokens_for_multiple_3rdparty_schedules() {
	ExtBuilder::new()
		.issue(ALICE, FIRST_REWARD_TOKEN, REWARD_AMOUNT)
		.issue(ALICE, SECOND_REWARD_TOKEN, 100_000u128)
		.issue(BOB, LIQUIDITY_TOKEN, 200)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				FIRST_REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(1);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(5);
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				SECOND_REWARD_TOKEN,
				100_000,
				15u32.into(),
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				SECOND_REWARD_TOKEN,
				Some(ThirdPartyActivationKind::ActivatedLiquidity(FIRST_REWARD_TOKEN)),
			)
			.unwrap();

			roll_to_session(7);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					SECOND_REWARD_TOKEN
				),
				Ok(10000)
			);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					FIRST_REWARD_TOKEN
				),
				Ok(6000)
			);
		});
}

#[test]
#[serial]
fn deactivate_3rdparty_rewards() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)
		.issue(CHARLIE, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(1);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(CHARLIE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(2);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(500)
			);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				),
				Ok(500)
			);
			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(CHARLIE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
			)
			.unwrap();

			roll_to_session(3);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(1500)
			);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				),
				Ok(500)
			);
		});
}

#[test]
#[serial]
fn calculate_and_claim_rewards_from_multiple_schedules_using_single_liquidity() {
	ExtBuilder::new()
		.issue(ALICE, FIRST_REWARD_TOKEN, 2 * REWARD_AMOUNT)
		.issue(ALICE, SECOND_REWARD_TOKEN, 4 * REWARD_AMOUNT)
		.issue(BOB, FIRST_LIQUIDITY_TOKEN, 100)
		.issue(BOB, SECOND_LIQUIDITY_TOKEN, 100)
		.build()
		.execute_with(|| {
			let get_liquidity_asset_mock = MockValuationApi::get_liquidity_asset_context();
			get_liquidity_asset_mock
				.expect()
				.with(eq(FIRST_REWARDED_PAIR.0), eq(FIRST_REWARDED_PAIR.1))
				.return_const(Ok(FIRST_LIQUIDITY_TOKEN));
			get_liquidity_asset_mock
				.expect()
				.with(eq(SECOND_REWARDED_PAIR.0), eq(SECOND_REWARDED_PAIR.1))
				.return_const(Ok(SECOND_LIQUIDITY_TOKEN));

			let valuate_liquidity_token_mock = MockValuationApi::valuate_liquidity_token_context();
			valuate_liquidity_token_mock.expect().return_const(11u128);

			let get_pool_state_mock = MockValuationApi::get_pool_state_context();
			get_pool_state_mock.expect().return_const(Some((min_req_valutation(),min_req_valutation())));

			System::set_block_number(1);
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				FIRST_REWARDED_PAIR,
				FIRST_REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				FIRST_LIQUIDITY_TOKEN,
				100u128,
				FIRST_REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(2);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 1 * 1000u128),]
			);
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				FIRST_REWARDED_PAIR,
				SECOND_REWARD_TOKEN,
				2 * REWARD_AMOUNT,
				12u32.into(),
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				FIRST_LIQUIDITY_TOKEN,
				100u128,
				SECOND_REWARD_TOKEN,
				Some(ThirdPartyActivationKind::ActivatedLiquidity(FIRST_REWARD_TOKEN)),
			)
			.unwrap();

			roll_to_session(3);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 2 * 1000u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 0 * 2000u128),
				]
			);

			roll_to_session(4);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 3 * 1000u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 1 * 2000u128),
				]
			);
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				SECOND_REWARDED_PAIR,
				FIRST_REWARD_TOKEN,
				REWARD_AMOUNT,
				14u32.into(),
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				SECOND_LIQUIDITY_TOKEN,
				100u128,
				FIRST_REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(5);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 4 * 1000u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 2 * 2000u128),
					(SECOND_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0 * 1000u128),
				]
			);
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				SECOND_REWARDED_PAIR,
				SECOND_REWARD_TOKEN,
				2 * REWARD_AMOUNT,
				15u32.into(),
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				SECOND_LIQUIDITY_TOKEN,
				100u128,
				SECOND_REWARD_TOKEN,
				Some(ThirdPartyActivationKind::ActivatedLiquidity(FIRST_REWARD_TOKEN)),
			)
			.unwrap();

			roll_to_session(7);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 6 * 1000u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 4 * 2000u128),
					(SECOND_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 2 * 1000u128),
					(SECOND_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 1 * 2000u128),
				]
			);

			ProofOfStake::claim_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				FIRST_LIQUIDITY_TOKEN,
				FIRST_REWARD_TOKEN,
			)
			.unwrap();

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 4 * 2000u128),
					(SECOND_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 2 * 1000u128),
					(SECOND_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 1 * 2000u128),
				]
			);

			ProofOfStake::claim_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				FIRST_LIQUIDITY_TOKEN,
				SECOND_REWARD_TOKEN,
			)
			.unwrap();
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 0u128),
					(SECOND_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 2 * 1000u128),
					(SECOND_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 1 * 2000u128),
				]
			);

			ProofOfStake::claim_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				SECOND_LIQUIDITY_TOKEN,
				FIRST_REWARD_TOKEN,
			)
			.unwrap();
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 0u128),
					(SECOND_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0u128),
					(SECOND_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 1 * 2000u128),
				]
			);

			ProofOfStake::claim_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				SECOND_LIQUIDITY_TOKEN,
				SECOND_REWARD_TOKEN,
			)
			.unwrap();

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 0u128),
					(SECOND_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0u128),
					(SECOND_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 0u128),
				]
			);

			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				FIRST_LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
			)
			.unwrap();
			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				FIRST_LIQUIDITY_TOKEN,
				100,
				SECOND_REWARD_TOKEN,
			)
			.unwrap();

			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				SECOND_LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
			)
			.unwrap();
			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				SECOND_LIQUIDITY_TOKEN,
				100,
				SECOND_REWARD_TOKEN,
			)
			.unwrap();
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_all(BOB).unwrap(),
				vec![
					(FIRST_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0u128),
					(FIRST_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 0u128),
					(SECOND_LIQUIDITY_TOKEN, FIRST_REWARD_TOKEN, 0u128),
					(SECOND_LIQUIDITY_TOKEN, SECOND_REWARD_TOKEN, 0u128),
				]
			);
		});
}

#[test]
#[serial]
fn liquidity_minting_liquidity_can_be_resused() {
	ExtBuilder::new()
		.issue(ALICE, FIRST_REWARD_TOKEN, REWARD_AMOUNT)
		.issue(ALICE, SECOND_REWARD_TOKEN, 100_000u128)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), LIQUIDITY_TOKEN, 1u8)
				.unwrap();
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				FIRST_REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_native_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				None,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
				Some(ThirdPartyActivationKind::LiquidityMining),
			)
			.unwrap();

			roll_to_session(2);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					FIRST_REWARD_TOKEN
				),
				Ok(1000)
			);
		});
}

#[test]
#[serial]
fn fail_to_transfer_tokens_that_has_been_partially_deactivated() {
	// 1. activate tokens for native rewards
	// 2. re-activate tokens for 3rdparty rewards
	// 4. deactivate tokens for 3rdparty rewards
	// 5. fail to transfer assets as they are still locked
	// 6. deactivate tokens for native rewards
	// 7. successfully transfer unlocked tokens
	ExtBuilder::new()
		.issue(ALICE, FIRST_REWARD_TOKEN, REWARD_AMOUNT)
		.issue(ALICE, SECOND_REWARD_TOKEN, 100_000u128)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), LIQUIDITY_TOKEN, 1u8)
				.unwrap();
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				FIRST_REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_native_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				None,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
				Some(ThirdPartyActivationKind::LiquidityMining),
			)
			.unwrap();

			assert_err!(
				ProofOfStake::deactivate_liquidity_for_native_rewards(
					RuntimeOrigin::signed(BOB),
					LIQUIDITY_TOKEN,
					100,
				),
				Error::<Test>::LiquidityLockedIn3rdpartyRewards
			);

			assert_err!(
				TokensOf::<Test>::transfer(
					LIQUIDITY_TOKEN,
					&BOB,
					&CHARLIE,
					100,
					ExistenceRequirement::AllowDeath
				),
				orml_tokens::Error::<Test>::BalanceTooLow
			);

			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
			)
			.unwrap();
			ProofOfStake::deactivate_liquidity_for_native_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
			)
			.unwrap();

			assert_ok!(TokensOf::<Test>::transfer(
				LIQUIDITY_TOKEN,
				&BOB,
				&CHARLIE,
				100,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
#[serial]
fn when_liquidity_mining_is_reused_it_is_unlocked_properly() {
	ExtBuilder::new()
		.issue(ALICE, FIRST_REWARD_TOKEN, REWARD_AMOUNT)
		.issue(ALICE, SECOND_REWARD_TOKEN, 100_000u128)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), LIQUIDITY_TOKEN, 1u8)
				.unwrap();
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				FIRST_REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				SECOND_REWARD_TOKEN,
				2 * REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_native_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				None,
			)
			.unwrap();
			assert_err!(
				TokensOf::<Test>::transfer(
					LIQUIDITY_TOKEN,
					&BOB,
					&CHARLIE,
					100,
					ExistenceRequirement::AllowDeath
				),
				orml_tokens::Error::<Test>::BalanceTooLow
			);

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
				Some(ThirdPartyActivationKind::LiquidityMining),
			)
			.unwrap();

			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &BOB, 100).unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
				None,
			)
			.unwrap();

			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				200,
				FIRST_REWARD_TOKEN,
			)
			.unwrap();
			assert_err!(
				TokensOf::<Test>::transfer(
					LIQUIDITY_TOKEN,
					&BOB,
					&CHARLIE,
					101,
					ExistenceRequirement::AllowDeath
				),
				orml_tokens::Error::<Test>::BalanceTooLow
			);

			assert_ok!(TokensOf::<Test>::transfer(
				LIQUIDITY_TOKEN,
				&BOB,
				&CHARLIE,
				100,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
#[serial]
fn liquidity_can_be_deactivated_when_all_reward_participation_were_deactivated() {
	ExtBuilder::new()
		.issue(ALICE, FIRST_REWARD_TOKEN, REWARD_AMOUNT)
		.issue(ALICE, SECOND_REWARD_TOKEN, 100_000u128)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), LIQUIDITY_TOKEN, 1u8)
				.unwrap();
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				FIRST_REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				SECOND_REWARD_TOKEN,
				2 * REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
				None,
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				SECOND_REWARD_TOKEN,
				Some(ThirdPartyActivationKind::ActivatedLiquidity(FIRST_REWARD_TOKEN)),
			)
			.unwrap();

			assert_err!(
				TokensOf::<Test>::transfer(
					0,
					&BOB,
					&CHARLIE,
					100,
					ExistenceRequirement::AllowDeath
				),
				orml_tokens::Error::<Test>::BalanceTooLow
			);
			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
			)
			.unwrap();
			assert_err!(
				TokensOf::<Test>::transfer(
					LIQUIDITY_TOKEN,
					&BOB,
					&CHARLIE,
					100,
					ExistenceRequirement::AllowDeath
				),
				orml_tokens::Error::<Test>::BalanceTooLow
			);
			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				SECOND_REWARD_TOKEN,
			)
			.unwrap();

			assert_ok!(TokensOf::<Test>::transfer(
				LIQUIDITY_TOKEN,
				&BOB,
				&CHARLIE,
				100,
				ExistenceRequirement::AllowDeath
			),);
		});
}

#[test]
#[serial]
fn can_claim_schedule_rewards() {
	ExtBuilder::new()
		.issue(ALICE, FIRST_REWARD_TOKEN, REWARD_AMOUNT)
		.issue(ALICE, SECOND_REWARD_TOKEN, 100_000u128)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {




			System::set_block_number(1);

			ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), LIQUIDITY_TOKEN, 1u8)
				.unwrap();
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				FIRST_REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				SECOND_REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				FIRST_REWARD_TOKEN,
				None,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				SECOND_REWARD_TOKEN,
				Some(ThirdPartyActivationKind::ActivatedLiquidity(FIRST_REWARD_TOKEN)),
			)
			.unwrap();

			forward_to_block(20);

			assert_eq!(TokensOf::<Test>::free_balance(FIRST_REWARD_TOKEN, &BOB), 0,);
			assert_eq!(TokensOf::<Test>::free_balance(SECOND_REWARD_TOKEN, &BOB), 0,);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					FIRST_REWARD_TOKEN,
				)
				.unwrap(),
				1000
			);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					SECOND_REWARD_TOKEN,
				)
				.unwrap(),
				1000
			);

			ProofOfStake::claim_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				FIRST_REWARD_TOKEN,
			)
			.unwrap();

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					FIRST_REWARD_TOKEN,
				)
				.unwrap(),
				0
			);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					SECOND_REWARD_TOKEN,
				)
				.unwrap(),
				1000
			);

			ProofOfStake::claim_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				SECOND_REWARD_TOKEN,
			)
			.unwrap();

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					FIRST_REWARD_TOKEN,
				)
				.unwrap(),
				0
			);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					SECOND_REWARD_TOKEN,
				)
				.unwrap(),
				0
			);

			assert_eq!(TokensOf::<Test>::free_balance(FIRST_REWARD_TOKEN, &BOB), 1000);
			assert_eq!(TokensOf::<Test>::free_balance(SECOND_REWARD_TOKEN, &BOB), 1000);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					FIRST_REWARD_TOKEN
				),
				Ok(0)
			);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					BOB,
					LIQUIDITY_TOKEN,
					SECOND_REWARD_TOKEN
				),
				Ok(0)
			);
		});
}

#[test]
#[serial]
fn can_not_provide_liquidity_for_schedule_rewards_when_its_only_activated_for_liq_minting() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {




			System::set_block_number(1);

			ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), LIQUIDITY_TOKEN, 1u8)
				.unwrap();

			assert_err_ignore_postinfo!(
				ProofOfStake::activate_liquidity_for_3rdparty_rewards(
					RuntimeOrigin::signed(BOB),
					LIQUIDITY_TOKEN,
					100,
					REWARD_TOKEN,
					None,
				),
				Error::<Test>::NotAPromotedPool
			);
		});
}

#[test]
#[serial]
fn can_not_provide_liquidity_for_mining_rewards_when_its_only_activated_for_schedule_rewards() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			assert_err!(
				ProofOfStake::activate_liquidity_for_native_rewards(
					RuntimeOrigin::signed(BOB),
					LIQUIDITY_TOKEN,
					100,
					None,
				),
				Error::<Test>::NotAPromotedPool
			);
		});
}

use frame_support::dispatch::{GetDispatchInfo, Pays};
use sp_runtime::{traits::Dispatchable, Permill};

#[test]
#[ignore]
#[serial]
fn activate_deactivate_calls_are_free_of_charge() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		System::set_block_number(1);

		let activate_call =
			mock::RuntimeCall::ProofOfStake(Call::activate_liquidity_for_3rdparty_rewards {
				liquidity_token_id: LIQUIDITY_TOKEN,
				amount: 100,
				reward_token: REWARD_TOKEN,
				use_balance_from: None,
			});

		let deactivate_call =
			mock::RuntimeCall::ProofOfStake(Call::deactivate_liquidity_for_3rdparty_rewards {
				liquidity_token_id: LIQUIDITY_TOKEN,
				amount: 100,
				reward_token: REWARD_TOKEN,
			});

		assert_eq!(activate_call.get_dispatch_info().pays_fee, Pays::No);

		assert_eq!(deactivate_call.get_dispatch_info().pays_fee, Pays::No);
	});
}

#[test]
#[serial]
fn unsuccessul_activate_deactivate_calls_charges_fees() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			let activate_call =
				mock::RuntimeCall::ProofOfStake(Call::activate_liquidity_for_3rdparty_rewards {
					liquidity_token_id: LIQUIDITY_TOKEN,
					amount: 100,
					reward_token: REWARD_TOKEN,
					use_balance_from: None,
				});

			let deactivate_call =
				mock::RuntimeCall::ProofOfStake(Call::deactivate_liquidity_for_3rdparty_rewards {
					liquidity_token_id: LIQUIDITY_TOKEN,
					amount: 100,
					reward_token: REWARD_TOKEN,
				});

			assert_eq!(
				activate_call
					.dispatch(RuntimeOrigin::signed(BOB))
					.unwrap_err()
					.post_info
					.pays_fee,
				Pays::Yes
			);

			assert_eq!(
				deactivate_call
					.dispatch(RuntimeOrigin::signed(BOB))
					.unwrap_err()
					.post_info
					.pays_fee,
				Pays::Yes
			);
		});
}

#[test]
#[serial]
fn claim_rewards_from_multiple_sessions_at_once() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)
		.issue(CHARLIE, LIQUIDITY_TOKEN, 100)
		.issue(EVE, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &BOB, 100).unwrap();
			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &CHARLIE, 100).unwrap();
			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &EVE, 100).unwrap();

			let amount = 10_000u128;

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(1);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(0)
			);

			roll_to_session(2);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(1000)
			);

			roll_to_session(5);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(4000)
			);
		});
}

#[test]
#[serial]
fn multi_user_rewards_distributeion_scenario() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)
		.issue(CHARLIE, LIQUIDITY_TOKEN, 100)
		.issue(EVE, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &BOB, 100).unwrap();
			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &CHARLIE, 100).unwrap();
			TokensOf::<Test>::mint(LIQUIDITY_TOKEN, &EVE, 100).unwrap();

			let amount = 10_000u128;

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(1);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(0)
			);

			roll_to_session(2);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(CHARLIE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				),
				Ok(0)
			);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(1000)
			);

			roll_to_session(3);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(1500)
			);
			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				),
				Ok(500)
			);
		});
}

#[test]
#[serial]
fn test_all_scheduled_rewards_are_distributed_when_activated_instantly() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			let amount = 10_000u128;

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(12);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(REWARD_AMOUNT)
			);
		});
}

#[test]
#[serial]
fn test_all_scheduled_rewards_are_distributed_when_activated_after_few_sessions() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			let amount = 10_000u128;

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(7);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(15);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(REWARD_AMOUNT)
			);
		});
}

#[test]
#[serial]
fn test_all_scheduled_rewards_are_distributed_when_activated_schedule_is_finished() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			let amount = 10_000u128;

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(15);

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(16);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(REWARD_AMOUNT)
			);
		});
}

#[test]
#[serial]
fn test_multiple_activations_in_same_block() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)
		.issue(CHARLIE, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);






			let amount = 10_000u128;

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			roll_to_session(1);

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(CHARLIE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(2);

			assert_eq!(
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN),
				Ok(REWARD_AMOUNT / 10 / 2)
			);
		});
}

#[test]
#[serial]
fn rewards_are_available_in_next_session_after_rewards_are_provided() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 10 * 3 * REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)
		.issue(CHARLIE, LIQUIDITY_TOKEN, 100)
		.issue(EVE, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				10 * 3 * REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);

			roll_to_session(1);

			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(CHARLIE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);

			roll_to_session(2);

			assert_eq!(
				15_000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				15_000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(EVE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			assert_eq!(
				15_000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				15_000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);

			roll_to_session(3);

			assert_eq!(
				15_000u128 + 10_000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				15_000u128 + 10_000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				10_000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
		});
}

#[test]
#[serial]
fn multiple_activations_and_deactivations_from_multiple_users_on_the_same_schedule() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 10 * 3 * REWARD_AMOUNT)
		.issue(ALICE, LIQUIDITY_TOKEN, 100)
		.issue(BOB, LIQUIDITY_TOKEN, 100)
		.issue(CHARLIE, LIQUIDITY_TOKEN, 100)
		.issue(EVE, LIQUIDITY_TOKEN, 100)

		.execute_with_default_mocks(|| {
			System::set_block_number(1);





			/// 1000 rewards per session
			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT,
				10u32.into(),
			)
			.unwrap();

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(ALICE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			roll_to_session(2);

			assert_eq!(
				500u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					ALICE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				500u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);

			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(CHARLIE),
				LIQUIDITY_TOKEN,
				50,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(EVE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(ALICE),
				LIQUIDITY_TOKEN,
				50,
				REWARD_TOKEN,
			)
			.unwrap();
			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
			)
			.unwrap();
			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(EVE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
			)
			.unwrap();

			assert_eq!(
				500u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					ALICE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				500u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);

			roll_to_session(3);

			assert_eq!(
				1000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					ALICE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				500u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				500u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);

			ProofOfStake::deactivate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(ALICE),
				LIQUIDITY_TOKEN,
				50,
				REWARD_TOKEN,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(ALICE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(CHARLIE),
				LIQUIDITY_TOKEN,
				50,
				REWARD_TOKEN,
				None,
			)
			.unwrap();
			ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(EVE),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			)
			.unwrap();

			assert_eq!(
				1000u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					ALICE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				500u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				500u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				0u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);

			roll_to_session(4);

			assert_eq!(
				1249u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					ALICE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				749u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(BOB, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
			assert_eq!(
				749u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(
					CHARLIE,
					LIQUIDITY_TOKEN,
					REWARD_TOKEN
				)
				.unwrap()
			);
			assert_eq!(
				249u128,
				ProofOfStake::calculate_3rdparty_rewards_amount(EVE, LIQUIDITY_TOKEN, REWARD_TOKEN)
					.unwrap()
			);
		});
}

#[test]
#[serial]
fn activity_for_schedule_rewards_can_be_activated_only_after_pool_is_rewarded_for_the_first_time() {
	ExtBuilder::new()
		.issue(ALICE, REWARD_TOKEN, 10 * 3 * REWARD_AMOUNT)
		.issue(BOB, LIQUIDITY_TOKEN, 100)
		.execute_with_default_mocks(|| {
			System::set_block_number(1);

			assert_err_ignore_postinfo!(
				ProofOfStake::activate_liquidity_for_3rdparty_rewards(
					RuntimeOrigin::signed(BOB),
					LIQUIDITY_TOKEN,
					100,
					REWARD_TOKEN,
					None,
				),
				Error::<Test>::NotAPromotedPool
			);

			ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), LIQUIDITY_TOKEN, 2u8)
				.unwrap();

			assert_err_ignore_postinfo!(
				ProofOfStake::activate_liquidity_for_3rdparty_rewards(
					RuntimeOrigin::signed(BOB),
					LIQUIDITY_TOKEN,
					100,
					REWARD_TOKEN,
					None,
				),
				Error::<Test>::NotAPromotedPool
			);

			ProofOfStake::reward_pool(
				RuntimeOrigin::signed(ALICE),
				REWARDED_PAIR,
				REWARD_TOKEN,
				REWARD_AMOUNT / 2,
				10u32.into(),
			)
			.unwrap();

			assert_ok!(ProofOfStake::activate_liquidity_for_3rdparty_rewards(
				RuntimeOrigin::signed(BOB),
				LIQUIDITY_TOKEN,
				100,
				REWARD_TOKEN,
				None,
			));
		});
}
