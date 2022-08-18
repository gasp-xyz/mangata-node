use super::*;
use mock::{
	new_test_ext, new_test_ext_without_issuance_config, roll_to_while_minting, BlocksPerRound,
	Issuance, Origin, StakeCurrency, System, Test, Tokens, Vesting, MGA_TOKEN_ID,
};
use sp_runtime::SaturatedConversion;

use frame_support::{assert_noop, assert_ok};

#[test]
fn init_issuance_config_works() {
	new_test_ext_without_issuance_config().execute_with(|| {
		let current_issuance = StakeCurrency::total_issuance(MGA_TOKEN_ID);

		assert_eq!(Issuance::is_tge_finalized(), false);
		assert_ok!(Issuance::finalize_tge(Origin::root()));
		assert_eq!(Issuance::is_tge_finalized(), true);

		assert_ok!(Issuance::init_issuance_config(Origin::root()));
		assert_eq!(
			Issuance::get_issuance_config(),
			Some(IssuanceInfo {
				cap: 4_000_000_000u128,
				issuance_at_init: current_issuance,
				linear_issuance_blocks: 22_222u32,
				liquidity_mining_split: Perbill::from_parts(555555556),
				staking_split: Perbill::from_parts(444444444),
				total_crowdloan_allocation: 200_000_000u128,
			})
		);
	});
}

#[test]
fn cannot_finalize_tge_when_already_finalized() {
	new_test_ext_without_issuance_config().execute_with(|| {
		assert_eq!(Issuance::is_tge_finalized(), false);
		assert_ok!(Issuance::finalize_tge(Origin::root()));
		assert_eq!(Issuance::is_tge_finalized(), true);

		assert_noop!(Issuance::finalize_tge(Origin::root()), Error::<Test>::TGEIsAlreadyFinalized);
	});
}

#[test]
fn cannot_init_issuance_config_when_tge_is_not_finalized() {
	new_test_ext_without_issuance_config().execute_with(|| {
		assert_eq!(Issuance::is_tge_finalized(), false);

		assert_noop!(
			Issuance::init_issuance_config(Origin::root()),
			Error::<Test>::TGENotFinalized
		);
	});
}

#[test]
fn cannot_init_issuance_config_when_already_init() {
	new_test_ext_without_issuance_config().execute_with(|| {
		let current_issuance = StakeCurrency::total_issuance(MGA_TOKEN_ID);

		assert_eq!(Issuance::is_tge_finalized(), false);
		assert_ok!(Issuance::finalize_tge(Origin::root()));
		assert_eq!(Issuance::is_tge_finalized(), true);

		assert_ok!(Issuance::init_issuance_config(Origin::root()));
		assert_eq!(
			Issuance::get_issuance_config(),
			Some(IssuanceInfo {
				cap: 4_000_000_000u128,
				issuance_at_init: current_issuance,
				linear_issuance_blocks: 22_222u32,
				liquidity_mining_split: Perbill::from_parts(555555556),
				staking_split: Perbill::from_parts(444444444),
				total_crowdloan_allocation: 200_000_000u128,
			})
		);
		assert_noop!(
			Issuance::init_issuance_config(Origin::root()),
			Error::<Test>::IssuanceConfigAlreadyInitialized
		);
	});
}

#[test]
fn execute_tge_works() {
	new_test_ext_without_issuance_config().execute_with(|| {
		assert_eq!(Issuance::is_tge_finalized(), false);

		assert_ok!(Issuance::execute_tge(
			Origin::root(),
			vec![
				TgeInfo { who: 1, amount: 1000u128 },
				TgeInfo { who: 2, amount: 2000u128 },
				TgeInfo { who: 3, amount: 3000u128 },
				TgeInfo { who: 4, amount: 4000u128 }
			]
		));
		assert_eq!(Issuance::get_tge_total(), 10_000u128);

		assert_eq!(StakeCurrency::free_balance(MGA_TOKEN_ID, &1), 1000u128);
		assert_eq!(StakeCurrency::free_balance(MGA_TOKEN_ID, &2), 2000u128);
		assert_eq!(StakeCurrency::free_balance(MGA_TOKEN_ID, &3), 3000u128);
		assert_eq!(StakeCurrency::free_balance(MGA_TOKEN_ID, &4), 4000u128);

		assert_eq!(Tokens::locks(&1, MGA_TOKEN_ID)[0].amount, 800u128);
		assert_eq!(Tokens::locks(&2, MGA_TOKEN_ID)[0].amount, 1600u128);
		assert_eq!(Tokens::locks(&3, MGA_TOKEN_ID)[0].amount, 2400u128);
		assert_eq!(Tokens::locks(&4, MGA_TOKEN_ID)[0].amount, 3200u128);

		assert_eq!(Vesting::vesting(&1, MGA_TOKEN_ID).unwrap()[0].locked(), 800u128);
		assert_eq!(Vesting::vesting(&2, MGA_TOKEN_ID).unwrap()[0].locked(), 1600u128);
		assert_eq!(Vesting::vesting(&3, MGA_TOKEN_ID).unwrap()[0].locked(), 2400u128);
		assert_eq!(Vesting::vesting(&4, MGA_TOKEN_ID).unwrap()[0].locked(), 3200u128);

		assert_eq!(Vesting::vesting(&1, MGA_TOKEN_ID).unwrap()[0].per_block(), 8u128);
		assert_eq!(Vesting::vesting(&2, MGA_TOKEN_ID).unwrap()[0].per_block(), 16u128);
		assert_eq!(Vesting::vesting(&3, MGA_TOKEN_ID).unwrap()[0].per_block(), 24u128);
		assert_eq!(Vesting::vesting(&4, MGA_TOKEN_ID).unwrap()[0].per_block(), 32u128);

		assert_ok!(Issuance::finalize_tge(Origin::root()));
		assert_eq!(Issuance::is_tge_finalized(), true);
	});
}

#[test]
fn cannot_execute_tge_if_already_finalized() {
	new_test_ext_without_issuance_config().execute_with(|| {
		assert_eq!(Issuance::is_tge_finalized(), false);
		assert_ok!(Issuance::finalize_tge(Origin::root()));
		assert_eq!(Issuance::is_tge_finalized(), true);

		assert_noop!(
			Issuance::execute_tge(Origin::root(), vec![]),
			Error::<Test>::TGEIsAlreadyFinalized
		);
	});
}

#[test]
fn linear_issuance_works() {
	new_test_ext().execute_with(|| {
		let session_number = System::block_number().saturated_into::<u32>() / BlocksPerRound::get();
		let session_issuance = <Issuance as GetIssuance>::get_all_issuance(session_number)
			.expect("session issuance is always populated in advance");
		let block_issuance = (session_issuance.0 + session_issuance.1) /
			(BlocksPerRound::get().saturated_into::<u128>());

		// Mint in block 1
		// We are not minting in block 0, but that's okay
		assert_eq!(405040, (session_issuance.0 + session_issuance.1));
		assert_eq!(81008, block_issuance);

		roll_to_while_minting(10000, Some(81008));

		// Mint for crowdloan
		let _ = orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(
			0u32.into(),
			&1u128,
			200_000_000u128.into(),
		);

		roll_to_while_minting(22218, Some(81008));

		assert_eq!(3999997760, Tokens::total_issuance(0u32) as Balance);

		// This the point the next session's issuance will be calculated and minted
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(81008));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn linear_issuance_doesnt_change_upon_burn() {
	new_test_ext().execute_with(|| {
		roll_to_while_minting(15000, Some(81008));

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000_000,
		)
		.unwrap();

		assert_eq!(3115525040, Tokens::total_issuance(0u32) as Balance);

		// Mint for crowdloan
		let _ = orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(
			0u32.into(),
			&1u128,
			200_000_000u128.into(),
		);

		roll_to_while_minting(22218, Some(81008));

		assert_eq!(3899997760, Tokens::total_issuance(0u32) as Balance);

		// This the point the next session's issuance will be calculated and minted
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(81008));

		assert_eq!(3900402800, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn issuance_stops_upon_reaching_cap() {
	new_test_ext().execute_with(|| {
		// This the point the next session's issuance will be calculated and minted
		// on the basis of total_issuance

		// Mint for crowdloan
		let _ = orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(
			0u32.into(),
			&1u128,
			200_000_000u128.into(),
		);

		// At this point the entirety of the missing issuance will be allocated to the next session
		roll_to_while_minting(22219, Some(81008));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(22224, Some(448));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		// Now there is not enough missing issuance to issue so no more mga will be issued

		roll_to_while_minting(23000, Some(0));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn issuance_does_not_stop_upon_burn() {
	new_test_ext().execute_with(|| {
		// Mint for crowdloan
		let _ = orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(
			0u32.into(),
			&1u128,
			200_000_000u128.into(),
		);

		// This the point the next session's issuance will be calculated and minted
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(81008));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(22221, Some(448));

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000,
		)
		.unwrap();

		// At this point the entirety of the missing issuance will be allocated to the next session

		roll_to_while_minting(22224, Some(448));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(22229, Some(20000));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(24001, Some(0));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn issuance_restarts_upon_burn() {
	new_test_ext().execute_with(|| {
		// Mint for crowdloan
		let _ = orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(
			0u32.into(),
			&1u128,
			200_000_000u128.into(),
		);

		// This the point the next session's issuance will be calculated and minted
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(81008));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		// At this point the entirety of the missing issuance will be allocated to the next session

		roll_to_while_minting(22224, Some(448));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		// Now there is not enough missing issuance to issue so no more mga will be issued

		roll_to_while_minting(23002, Some(0));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000,
		)
		.unwrap();

		assert_eq!(3999900000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(23004, Some(0));

		roll_to_while_minting(23009, Some(20000));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(24001, Some(0));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn issuance_after_linear_period_never_execeeds_linear() {
	new_test_ext().execute_with(|| {
		// Mint for crowdloan
		let _ = orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(
			0u32.into(),
			&1u128,
			200_000_000u128.into(),
		);

		// This the point the next session's issuance will be calculated and minted
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(81008));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		// At this point the entirety of the missing issuance will be allocated to the next session

		roll_to_while_minting(22224, Some(448));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		// Now there is not enough missing issuance to issue so no more mga will be issued

		roll_to_while_minting(23002, Some(0));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000,
		)
		.unwrap();

		assert_eq!(3999900000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(23004, Some(0));

		roll_to_while_minting(23009, Some(20000));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(23023, Some(0));

		assert_eq!(4000000000, Tokens::total_issuance(0u32) as Balance);

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000_000,
		)
		.unwrap();

		assert_eq!(3900000000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(23024, Some(0));

		roll_to_while_minting(23051, Some(81008));

		assert_eq!(3902430240, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn promote_pool_api_works() {
	new_test_ext().execute_with(|| {
		Issuance::promote_pool(1);

		roll_to_while_minting(4, None);
		assert_eq!(
			U256::from_dec_str("76571018769283414925455480913511346478027010").unwrap(),
			Issuance::get_pool_rewards_v2(1).unwrap()
		);
		roll_to_while_minting(9, None);
		assert_eq!(
			U256::from_dec_str("153142037538566829850910961827022692956054020").unwrap(),
			Issuance::get_pool_rewards_v2(1).unwrap()
		);

		Issuance::promote_pool(2);
		//	assert_eq!(2, Issuance::len());
		roll_to_while_minting(14, None);
		assert_eq!(
			U256::from_dec_str("191427546923208537313638702283778366195067525").unwrap(),
			Issuance::get_pool_rewards_v2(1).unwrap()
		);
		assert_eq!(
			U256::from_dec_str("38285509384641707462727740456755673239013505").unwrap(),
			Issuance::get_pool_rewards_v2(2).unwrap()
		);

		roll_to_while_minting(19, None);
		assert_eq!(
			U256::from_dec_str("229713056307850244776366442740534039434081030").unwrap(),
			Issuance::get_pool_rewards_v2(1).unwrap()
		);
		assert_eq!(
			U256::from_dec_str("76571018769283414925455480913511346478027010").unwrap(),
			Issuance::get_pool_rewards_v2(2).unwrap()
		);
	});
}

#[test]
fn mock_use_demo() {
	new_test_ext().execute_with(|| {
		// sets to some value
		mock::MockActivedPoolQueryApi::instance().lock().unwrap().replace(1_u128);

		// sets to none
		mock::MockActivedPoolQueryApi::instance().lock().unwrap().take();
	});
}

//PoolPromoteApi

// promote pool or 2
// go to block 100000 one by one with issuing, check rewards, claim rewards
