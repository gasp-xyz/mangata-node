use crate::{
	mock::{consts::*, *},
	*,
};
use core::future::pending;
use frame_support::{assert_err, assert_ok};
use hex_literal::hex;
use mockall::predicate::eq;
use serial_test::serial;
use sp_io::storage::rollback_transaction;
use orml_traits::MultiReservableCurrency;

pub type TokensOf<Test> = <Test as crate::Config>::Currency;

#[test]
#[serial]
fn test_genesis_build() {
	let new_sequencer_active_mock =
		MockRolldownProviderApi::new_sequencer_active_context();
	new_sequencer_active_mock.expect().times(2).return_const(());
	ExtBuilder::new().build().execute_with(|| {

		assert_eq!(SequencerStake::<Test>::get(&ALICE), MINIMUM_STAKE);
		assert_eq!(SequencerStake::<Test>::get(&BOB), MINIMUM_STAKE);
		assert_eq!(ActiveSequencers::<Test>::get(), vec![ALICE, BOB]);
		assert_eq!(TokensOf::<Test>::total_balance(&ALICE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&ALICE), MINIMUM_STAKE);
		assert_eq!(TokensOf::<Test>::total_balance(&BOB), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&BOB), MINIMUM_STAKE);

	});
}

#[test]
#[serial]
fn test_provide_sequencer_stake_works_and_activates() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let new_sequencer_active_mock =
			MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(1).return_const(());

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), 0);
		assert_eq!(SequencerStake::<Test>::get(&CHARLIE), 0);
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), false);
		EligibleToBeSequencers::<Test>::put(BTreeMap::from(
			[(consts::ALICE, 1u32), (consts::BOB, 1u32), (consts::CHARLIE, 1u32)]
		));
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), MINIMUM_STAKE));
		assert_eq!(SequencerStake::<Test>::get(&CHARLIE), MINIMUM_STAKE);
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), true);
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), MINIMUM_STAKE);

	});
}


#[test]
#[serial]
fn test_provide_sequencer_stake_works_and_does_not_activate_due_to_eligibility() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let new_sequencer_active_mock =
			MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(0).return_const(());

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), 0);
		assert_eq!(SequencerStake::<Test>::get(&CHARLIE), 0);
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), false);
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), MINIMUM_STAKE));
		assert_eq!(SequencerStake::<Test>::get(&CHARLIE), MINIMUM_STAKE);
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), false);
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), MINIMUM_STAKE);

	});
}


#[test]
#[serial]
fn test_provide_sequencer_stake_works_and_does_not_activate_due_to_insufficient_stake() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let new_sequencer_active_mock =
			MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(0).return_const(());

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), 0);
		assert_eq!(SequencerStake::<Test>::get(&CHARLIE), 0);
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), false);
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), MINIMUM_STAKE - 1));
		assert_eq!(SequencerStake::<Test>::get(&CHARLIE), MINIMUM_STAKE - 1);
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), false);
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), MINIMUM_STAKE-1);

	});
}

#[test]
#[serial]
fn test_provide_sequencer_stake_works_and_does_not_activate_due_to_max_seq_bound() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let new_sequencer_active_mock =
			MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(0).return_const(());

		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(DAVE), MINIMUM_STAKE));

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), 0);
		assert_eq!(SequencerStake::<Test>::get(&CHARLIE), 0);
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), false);
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), MINIMUM_STAKE));
		assert_eq!(SequencerStake::<Test>::get(&CHARLIE), MINIMUM_STAKE);
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), false);
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), MINIMUM_STAKE);

	});
}


#[test]
#[serial]
fn test_leave_active_sequencer_set() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let handle_sequencer_deactivations_mock = MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());

		assert_err!(SequencerStaking::leave_active_sequencers(RuntimeOrigin::signed(CHARLIE)), Error::<Test>::SequencerIsNotInActiveSet);

		assert_eq!(ActiveSequencers::<Test>::get().contains(&ALICE), true);
		assert_ok!(SequencerStaking::leave_active_sequencers(RuntimeOrigin::signed(ALICE)));
		assert_eq!(ActiveSequencers::<Test>::get().contains(&ALICE), false);

	});
}


#[test]
#[serial]
fn test_rejoin_active_sequencer_works() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		assert_err!(SequencerStaking::rejoin_active_sequencers(RuntimeOrigin::signed(ALICE)), Error::<Test>::SequencerAlreadyInActiveSet);

		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), MINIMUM_STAKE - 1));
		assert_err!(SequencerStaking::rejoin_active_sequencers(RuntimeOrigin::signed(CHARLIE)), Error::<Test>::NotEnoughSequencerStake);

		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), 1));
		assert_err!(SequencerStaking::rejoin_active_sequencers(RuntimeOrigin::signed(CHARLIE)), Error::<Test>::NotEligibleToBeSequencer);


		EligibleToBeSequencers::<Test>::put(BTreeMap::from(
			[(consts::ALICE, 1u32), (consts::BOB, 1u32), (consts::CHARLIE, 1u32), (consts::DAVE, 1u32)]
		));
		let new_sequencer_active_mock =
			MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(1).return_const(());
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), false);
		assert_ok!(SequencerStaking::rejoin_active_sequencers(RuntimeOrigin::signed(CHARLIE)));
		assert_eq!(ActiveSequencers::<Test>::get().contains(&CHARLIE), true);

		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(DAVE), MINIMUM_STAKE));
		assert_err!(SequencerStaking::rejoin_active_sequencers(RuntimeOrigin::signed(DAVE)), Error::<Test>::MaxSequencersLimitReached);
		
	});
}


#[test]
#[serial]
fn test_sequencer_unstaking() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		assert_err!(SequencerStaking::unstake(RuntimeOrigin::signed(ALICE)), Error::<Test>::CantUnstakeWhileInActiveSet);

		let sequencer_unstaking_mock =
			MockRolldownProviderApi::sequencer_unstaking_context();
		sequencer_unstaking_mock.expect().times(1).return_const(Ok(()));
		let handle_sequencer_deactivations_mock = MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());
		assert_ok!(SequencerStaking::leave_active_sequencers(RuntimeOrigin::signed(ALICE)));

		assert_eq!(TokensOf::<Test>::total_balance(&ALICE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&ALICE), MINIMUM_STAKE);
		assert_eq!(SequencerStake::<Test>::get(&ALICE), MINIMUM_STAKE);
		assert_ok!(SequencerStaking::unstake(RuntimeOrigin::signed(ALICE)));
		assert_eq!(SequencerStake::<Test>::get(&ALICE), 0);
		assert_eq!(TokensOf::<Test>::total_balance(&ALICE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&ALICE), 0);
		
	});
}

#[test]
#[serial]
fn test_set_sequencer_configuration() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		
		let new_sequencer_active_mock =
			MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(1).return_const(());

		EligibleToBeSequencers::<Test>::put(BTreeMap::from(
			[(consts::ALICE, 1u32), (consts::BOB, 1u32), (consts::CHARLIE, 1u32)]
		));
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), MINIMUM_STAKE+1));

		let handle_sequencer_deactivations_mock = MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());
		assert_ok!(SequencerStaking::set_sequencer_configuration(RuntimeOrigin::root(), MINIMUM_STAKE+1, SLASH_AMOUNT-1));
		assert_eq!(ActiveSequencers::<Test>::get(), vec![CHARLIE]);
		assert_eq!(MinimalStakeAmount::<Test>::get(), MINIMUM_STAKE+1);
		assert_eq!(SlashFineAmount::<Test>::get(), SLASH_AMOUNT-1);
		
	});
}

#[test]
#[serial]
fn test_slash_sequencer() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);


		let handle_sequencer_deactivations_mock = MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());


		assert_eq!(TokensOf::<Test>::total_balance(&ALICE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&ALICE), MINIMUM_STAKE);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4);

		assert_ok!(SequencerStaking::slash_sequencer(&ALICE, Some(&EVE)));

		assert_eq!(TokensOf::<Test>::total_balance(&ALICE), TOKENS_ENDOWED - SLASH_AMOUNT);
		assert_eq!(TokensOf::<Test>::reserved_balance(&ALICE), MINIMUM_STAKE - SLASH_AMOUNT);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0 + CancellerRewardPercentage::get() * SLASH_AMOUNT);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4 - (SLASH_AMOUNT - CancellerRewardPercentage::get() * SLASH_AMOUNT));


		let handle_sequencer_deactivations_mock = MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());

		let total_issuance_0 = TokensOf::<Test>::total_issuance();
		assert_eq!(TokensOf::<Test>::total_balance(&BOB), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&BOB), MINIMUM_STAKE);

		assert_ok!(SequencerStaking::slash_sequencer(&BOB, None));

		assert_eq!(TokensOf::<Test>::total_balance(&BOB), TOKENS_ENDOWED - SLASH_AMOUNT);
		assert_eq!(TokensOf::<Test>::reserved_balance(&BOB), MINIMUM_STAKE - SLASH_AMOUNT);
		assert_eq!(total_issuance_0 - TokensOf::<Test>::total_issuance(), SLASH_AMOUNT);

	});
}

#[test]
#[serial]
fn test_slash_sequencer_when_stake_less_than_repatriated_amount() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let amount = 10;
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), amount));

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), amount);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4);

		assert_ok!(SequencerStaking::slash_sequencer(&CHARLIE, Some(&EVE)));

		let repatriated_amount = 10;
		let amount_slashed = 10;
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED - amount_slashed);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), amount - amount_slashed);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0 + repatriated_amount);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4 - (amount_slashed - repatriated_amount));


		let amount = 10;
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(DAVE), amount));

		let total_issuance_0 = TokensOf::<Test>::total_issuance();
		assert_eq!(TokensOf::<Test>::total_balance(&DAVE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&DAVE), amount);

		assert_ok!(SequencerStaking::slash_sequencer(&DAVE, None));

		let repatriated_amount = 0;
		let amount_slashed = 10;
		assert_eq!(TokensOf::<Test>::total_balance(&DAVE), TOKENS_ENDOWED - amount_slashed);
		assert_eq!(TokensOf::<Test>::reserved_balance(&DAVE), amount - amount_slashed);
		assert_eq!(total_issuance_0 - TokensOf::<Test>::total_issuance(), amount_slashed);
		
	});
}


#[test]
#[serial]
fn test_slash_sequencer_when_stake_less_than_stake_but_greater_than_repatriated_amount() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let amount = 50;
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(CHARLIE), amount));

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), amount);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4);

		assert_ok!(SequencerStaking::slash_sequencer(&CHARLIE, Some(&EVE)));

		let repatriated_amount = 20;
		let amount_slashed = 50;
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED - amount_slashed);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), amount - amount_slashed);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0 + repatriated_amount);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4 - (amount_slashed - repatriated_amount));


		let amount = 50;
		assert_ok!(SequencerStaking::provide_sequencer_stake(RuntimeOrigin::signed(DAVE), amount));

		let total_issuance_0 = TokensOf::<Test>::total_issuance();
		assert_eq!(TokensOf::<Test>::total_balance(&DAVE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&DAVE), amount);

		assert_ok!(SequencerStaking::slash_sequencer(&DAVE, None));

		let repatriated_amount = 0;
		let amount_slashed = 50;
		assert_eq!(TokensOf::<Test>::total_balance(&DAVE), TOKENS_ENDOWED - amount_slashed);
		assert_eq!(TokensOf::<Test>::reserved_balance(&DAVE), amount - amount_slashed);
		assert_eq!(total_issuance_0 - TokensOf::<Test>::total_issuance(), amount_slashed);
		
	});
}

#[test]
#[serial]
fn test_maybe_remove_sequencers_from_active_set_works() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		ActiveSequencers::<Test>::put(vec![ALICE, BOB, CHARLIE, DAVE]);

		let handle_sequencer_deactivations_mock = MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().with(eq(vec![BOB, DAVE])).times(1).return_const(());

		SequencerStaking::maybe_remove_sequencers_from_active_set(vec![BOB, DAVE, EVE]);

	});
}


#[test]
#[serial]
fn test_remove_sequencers_works_correctly() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let handle_sequencer_deactivations_mock = MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().return_const(());

		SelectedSequencer::<Test>::put(4);
		NextSequencerIndex::<Test>::put(6);
		ActiveSequencers::<Test>::put(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

		SequencerStaking::remove_sequencers_from_active_set(vec![1, 4, 5, 6, 8, 11]);

		assert_eq!(SelectedSequencer::<Test>::get(), None);
		assert_eq!(NextSequencerIndex::<Test>::get(), 3);
		assert_eq!(ActiveSequencers::<Test>::get(), vec![0, 2, 3, 7, 9, 10]);



		SelectedSequencer::<Test>::put(4);
		NextSequencerIndex::<Test>::put(4);
		ActiveSequencers::<Test>::put(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

		SequencerStaking::remove_sequencers_from_active_set(vec![4]);

		assert_eq!(SelectedSequencer::<Test>::get(), None);
		assert_eq!(NextSequencerIndex::<Test>::get(), 4);
		assert_eq!(ActiveSequencers::<Test>::get(), vec![0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11]);



		SelectedSequencer::<Test>::put(4);
		NextSequencerIndex::<Test>::put(6);
		ActiveSequencers::<Test>::put(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

		SequencerStaking::remove_sequencers_from_active_set(vec![2, 3, 5, 8, 11]);

		assert_eq!(SelectedSequencer::<Test>::get(), Some(4));
		assert_eq!(NextSequencerIndex::<Test>::get(), 3);
		assert_eq!(ActiveSequencers::<Test>::get(), vec![0, 1, 4, 6, 7, 9, 10]);
	});
}


#[test]
#[serial]
fn test_on_finalize_works_correctly() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		SelectedSequencer::<Test>::put(5);
		NextSequencerIndex::<Test>::put(6);
		ActiveSequencers::<Test>::put(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

		SequencerStaking::on_finalize(10);

		assert_eq!(SelectedSequencer::<Test>::get(), Some(6));
		assert_eq!(NextSequencerIndex::<Test>::get(), 7);


		SelectedSequencer::<Test>::put(5);
		NextSequencerIndex::<Test>::put(12);
		ActiveSequencers::<Test>::put(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

		SequencerStaking::on_finalize(10);

		assert_eq!(SelectedSequencer::<Test>::get(), Some(0));
		assert_eq!(NextSequencerIndex::<Test>::get(), 1);


		SelectedSequencer::<Test>::put(5);
		NextSequencerIndex::<Test>::put(13);
		ActiveSequencers::<Test>::put(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

		SequencerStaking::on_finalize(10);

		assert_eq!(SelectedSequencer::<Test>::get(), Some(0));
		assert_eq!(NextSequencerIndex::<Test>::get(), 1);


		SelectedSequencer::<Test>::put(5);
		NextSequencerIndex::<Test>::put(6);
		ActiveSequencers::<Test>::put(Vec::<AccountId>::new());

		SequencerStaking::on_finalize(10);

		assert_eq!(SelectedSequencer::<Test>::get(), None);
		assert_eq!(NextSequencerIndex::<Test>::get(), 0);
	});
}