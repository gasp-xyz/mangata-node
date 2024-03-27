use crate::setup::*;
use frame_support::traits::{OnFinalize, OnInitialize};
use orml_tokens::MultiTokenCurrencyExtended;

type TokensOf<Test> = <Test as pallet_proof_of_stake::Config>::Currency;
type XykOf<Test> = <Test as pallet_proof_of_stake::Config>::ValuationApi;

fn forward_to_block(n: u32) {
	while frame_system::Pallet::<Runtime>::block_number() < n {
		let i = frame_system::Pallet::<Runtime>::block_number() + 1;
		frame_system::Pallet::<Runtime>::set_block_number(i);

		frame_system::Pallet::<Runtime>::on_initialize(i);
		parachain_staking::Pallet::<Runtime>::on_initialize(i);
		pallet_session::Pallet::<Runtime>::on_initialize(i);

		pallet_session::Pallet::<Runtime>::on_finalize(i);
		parachain_staking::Pallet::<Runtime>::on_finalize(i);
		frame_system::Pallet::<Runtime>::on_finalize(i);
	}
}

#[test]
fn rewards_are_aligned_with_sessions() {
	ExtBuilder::default().build().execute_with(|| {
		let alice: sp_runtime::AccountId32 = [0u8; 32].into();
		let bob: sp_runtime::AccountId32 = [1u8; 32].into();
		let charlie: sp_runtime::AccountId32 = [2u8; 32].into();
		let amount: u128 = 100_000u128;
		let blocks_per_round = <Runtime as parachain_staking::Config>::BlocksPerRound::get();

		TokensOf::<Runtime>::create(&alice, amount).unwrap();
		TokensOf::<Runtime>::create(&alice, amount).unwrap();
		TokensOf::<Runtime>::create(&alice, amount).unwrap();
		TokensOf::<Runtime>::create(&alice, amount).unwrap();
		let first_token_id = TokensOf::<Runtime>::create(&alice, amount).unwrap();
		let second_token_id = TokensOf::<Runtime>::create(&alice, amount).unwrap();

		let liqudity_token_id = TokensOf::<Runtime>::get_next_currency_id();

		TokensOf::<Runtime>::mint(first_token_id, &bob, amount).unwrap();
		TokensOf::<Runtime>::mint(second_token_id, &bob, amount).unwrap();
		TokensOf::<Runtime>::mint(first_token_id, &charlie, amount).unwrap();
		TokensOf::<Runtime>::mint(second_token_id, &charlie, amount).unwrap();

		XykOf::<Runtime>::create_pool(
			RuntimeOrigin::signed(alice.clone()),
			first_token_id,
			100000_u128,
			second_token_id,
			100000_u128,
		)
		.unwrap();

		assert_eq!(0, pallet_session::Pallet::<Runtime>::current_index());
		ProofOfStake::update_pool_promotion(RuntimeOrigin::root(), liqudity_token_id, 1u8).unwrap();
		ProofOfStake::activate_liquidity(
			RuntimeOrigin::signed(alice.clone()),
			liqudity_token_id,
			amount,
			None,
		)
		.unwrap();

		forward_to_block(blocks_per_round - 10);
		assert_eq!(0, pallet_session::Pallet::<Runtime>::current_index());
		XykOf::<Runtime>::mint_liquidity(
			RuntimeOrigin::signed(charlie.clone()),
			first_token_id,
			second_token_id,
			1000,
			10000,
		)
		.unwrap();

		forward_to_block(blocks_per_round - 2);
		assert_eq!(0, pallet_session::Pallet::<Runtime>::current_index());
		XykOf::<Runtime>::mint_liquidity(
			RuntimeOrigin::signed(bob.clone()),
			first_token_id,
			second_token_id,
			1000,
			10000,
		)
		.unwrap();

		forward_to_block(blocks_per_round - 1);
		assert_eq!(1, pallet_session::Pallet::<Runtime>::current_index());

		assert_eq!(
			ProofOfStake::get_rewards_info(charlie.clone(), liqudity_token_id),
			ProofOfStake::get_rewards_info(bob.clone(), liqudity_token_id)
		);
	});
}
