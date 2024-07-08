use crate::{
	mock::{consts::*, *},
	*,
};
use core::{convert::TryFrom, future::pending};
use frame_support::{assert_err, assert_ok};
use hex_literal::hex;
use mockall::predicate::eq;
use orml_traits::MultiReservableCurrency;
use serial_test::serial;
use sp_io::storage::rollback_transaction;
use sp_runtime::BoundedBTreeSet;

pub type TokensOf<Test> = <Test as crate::Config>::Currency;

#[test]
#[serial]
fn test_genesis_build() {
	let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
	new_sequencer_active_mock.expect().times(2).return_const(());
	ExtBuilder::new().build().execute_with(|| {
		assert_eq!(SequencerStake::<Test>::get(&(ALICE, consts::DEFAULT_CHAIN_ID)), MINIMUM_STAKE);
		assert_eq!(SequencerStake::<Test>::get(&(BOB, consts::DEFAULT_CHAIN_ID)), MINIMUM_STAKE);

		assert!(SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &ALICE));
		assert!(SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &BOB));

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

		let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(1).return_const(());

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), 0);
		assert_eq!(SequencerStake::<Test>::get(&(CHARLIE, consts::DEFAULT_CHAIN_ID)), 0);
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			MINIMUM_STAKE
		));
		assert_eq!(
			SequencerStake::<Test>::get(&(CHARLIE, consts::DEFAULT_CHAIN_ID)),
			MINIMUM_STAKE
		);
		assert!(SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
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

		let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(0).return_const(());

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), 0);
		assert_eq!(SequencerStake::<Test>::get(&(CHARLIE, consts::DEFAULT_CHAIN_ID)), 0);
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			MINIMUM_STAKE - 1
		));
		assert_eq!(
			SequencerStake::<Test>::get(&(CHARLIE, consts::DEFAULT_CHAIN_ID)),
			MINIMUM_STAKE - 1
		);
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), MINIMUM_STAKE - 1);
	});
}

#[test]
#[serial]
fn test_provide_sequencer_stake_works_and_does_not_activate_due_to_max_seq_bound() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(0).return_const(());

		SequencerStaking::set_active_sequencers(
			(20u64..31u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), 0);
		assert_eq!(SequencerStake::<Test>::get(&(CHARLIE, consts::DEFAULT_CHAIN_ID)), 0);
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			MINIMUM_STAKE
		));
		assert_eq!(
			SequencerStake::<Test>::get(&(CHARLIE, consts::DEFAULT_CHAIN_ID)),
			MINIMUM_STAKE
		);
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
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

		let handle_sequencer_deactivations_mock =
			MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());

		assert_err!(
			SequencerStaking::leave_active_sequencers(
				RuntimeOrigin::signed(CHARLIE),
				consts::DEFAULT_CHAIN_ID
			),
			Error::<Test>::SequencerIsNotInActiveSet
		);

		assert!(SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &ALICE));
		assert_ok!(SequencerStaking::leave_active_sequencers(
			RuntimeOrigin::signed(ALICE),
			consts::DEFAULT_CHAIN_ID
		));
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &ALICE));
	});
}

#[test]
#[serial]
fn test_rejoin_active_sequencer_works() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		assert_err!(
			SequencerStaking::rejoin_active_sequencers(
				RuntimeOrigin::signed(ALICE),
				consts::DEFAULT_CHAIN_ID
			),
			Error::<Test>::SequencerAlreadyInActiveSet
		);

		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			MINIMUM_STAKE - 1
		));
		assert_err!(
			SequencerStaking::rejoin_active_sequencers(
				RuntimeOrigin::signed(CHARLIE),
				consts::DEFAULT_CHAIN_ID
			),
			Error::<Test>::NotEnoughSequencerStake
		);

		SequencerStaking::set_active_sequencers(
			(20u64..31u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			1
		));

		SequencerStaking::set_active_sequencers(
			(20u64..30u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(1).return_const(());
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
		assert_ok!(SequencerStaking::rejoin_active_sequencers(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID
		));
		assert!(SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
	});
}

#[test]
#[serial]
fn test_can_not_join_set_if_full() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);
		ActiveSequencers::<Test>::kill();
		let seq_limit = <<Test as Config>::MaxSequencers as Get<u32>>::get() as AccountId;
		let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(seq_limit as usize).return_const(());

		for seq in 0u64..seq_limit {
			Tokens::mint(RuntimeOrigin::root(), NATIVE_TOKEN_ID, seq, MINIMUM_STAKE).unwrap();
			assert_ok!(SequencerStaking::provide_sequencer_stake(
				RuntimeOrigin::signed(seq),
				consts::DEFAULT_CHAIN_ID,
				MINIMUM_STAKE,
			));
			assert!(SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &seq));
		}

		Tokens::mint(RuntimeOrigin::root(), NATIVE_TOKEN_ID, seq_limit, MINIMUM_STAKE).unwrap();
		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(seq_limit),
			consts::DEFAULT_CHAIN_ID,
			MINIMUM_STAKE,
		));
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &seq_limit));
		assert_err!(
			SequencerStaking::rejoin_active_sequencers(
				RuntimeOrigin::signed(seq_limit),
				consts::DEFAULT_CHAIN_ID
			),
			Error::<Test>::MaxSequencersLimitReached
		);
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &seq_limit));
	});
}

#[test]
#[serial]
fn test_provide_stake_fails_on_sequencers_limit_reached() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		assert_err!(
			SequencerStaking::rejoin_active_sequencers(
				RuntimeOrigin::signed(ALICE),
				consts::DEFAULT_CHAIN_ID
			),
			Error::<Test>::SequencerAlreadyInActiveSet
		);

		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			MINIMUM_STAKE - 1
		));
		assert_err!(
			SequencerStaking::rejoin_active_sequencers(
				RuntimeOrigin::signed(CHARLIE),
				consts::DEFAULT_CHAIN_ID
			),
			Error::<Test>::NotEnoughSequencerStake
		);

		SequencerStaking::set_active_sequencers(
			(20u64..31u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			1
		));

		SequencerStaking::set_active_sequencers(
			(20u64..30u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(1).return_const(());
		assert!(!SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
		assert_ok!(SequencerStaking::rejoin_active_sequencers(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID
		));
		assert!(SequencerStaking::is_active_sequencer(consts::DEFAULT_CHAIN_ID, &CHARLIE));
	});
}

#[test]
#[serial]
fn test_sequencer_unstaking() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		assert_err!(
			SequencerStaking::unstake(RuntimeOrigin::signed(ALICE), consts::DEFAULT_CHAIN_ID),
			Error::<Test>::CantUnstakeWhileInActiveSet
		);

		let sequencer_unstaking_mock = MockRolldownProviderApi::sequencer_unstaking_context();
		sequencer_unstaking_mock.expect().times(1).return_const(Ok(()));
		let handle_sequencer_deactivations_mock =
			MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());
		assert_ok!(SequencerStaking::leave_active_sequencers(
			RuntimeOrigin::signed(ALICE),
			consts::DEFAULT_CHAIN_ID
		));

		assert_eq!(TokensOf::<Test>::total_balance(&ALICE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&ALICE), MINIMUM_STAKE);
		assert_eq!(SequencerStake::<Test>::get(&(ALICE, consts::DEFAULT_CHAIN_ID)), MINIMUM_STAKE);
		assert_ok!(SequencerStaking::unstake(
			RuntimeOrigin::signed(ALICE),
			consts::DEFAULT_CHAIN_ID
		));
		assert_eq!(SequencerStake::<Test>::get(&(ALICE, consts::DEFAULT_CHAIN_ID)), 0);
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

		let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(1).return_const(());

		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			MINIMUM_STAKE + 1
		));

		let handle_sequencer_deactivations_mock =
			MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());

		assert_ok!(SequencerStaking::set_sequencer_configuration(
			RuntimeOrigin::root(),
			consts::DEFAULT_CHAIN_ID,
			MINIMUM_STAKE + 1,
			SLASH_AMOUNT - 1
		));
		assert_eq!(
			ActiveSequencers::<Test>::get().get(&consts::DEFAULT_CHAIN_ID).unwrap().len(),
			1
		);
		assert_eq!(
			ActiveSequencers::<Test>::get()
				.get(&consts::DEFAULT_CHAIN_ID)
				.unwrap()
				.iter()
				.next(),
			Some(&CHARLIE)
		);
		assert_eq!(MinimalStakeAmount::<Test>::get(), MINIMUM_STAKE + 1);
		assert_eq!(SlashFineAmount::<Test>::get(), SLASH_AMOUNT - 1);
	});
}

#[test]
#[serial]
fn test_slash_sequencer() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let handle_sequencer_deactivations_mock =
			MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());

		assert_eq!(TokensOf::<Test>::total_balance(&ALICE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&ALICE), MINIMUM_STAKE);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4);

		assert_ok!(SequencerStaking::slash_sequencer(consts::DEFAULT_CHAIN_ID, &ALICE, Some(&EVE)));

		assert_eq!(TokensOf::<Test>::total_balance(&ALICE), TOKENS_ENDOWED - SLASH_AMOUNT);
		assert_eq!(TokensOf::<Test>::reserved_balance(&ALICE), MINIMUM_STAKE - SLASH_AMOUNT);
		assert_eq!(
			TokensOf::<Test>::total_balance(&EVE),
			0 + CancellerRewardPercentage::get() * SLASH_AMOUNT
		);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(
			TokensOf::<Test>::total_issuance(),
			TOKENS_ENDOWED * 4 - (SLASH_AMOUNT - CancellerRewardPercentage::get() * SLASH_AMOUNT)
		);

		let handle_sequencer_deactivations_mock =
			MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().times(1).return_const(());

		let total_issuance_0 = TokensOf::<Test>::total_issuance();
		assert_eq!(TokensOf::<Test>::total_balance(&BOB), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&BOB), MINIMUM_STAKE);

		assert_ok!(SequencerStaking::slash_sequencer(consts::DEFAULT_CHAIN_ID, &BOB, None));

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
		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			amount
		));

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), amount);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4);

		assert_ok!(SequencerStaking::slash_sequencer(
			consts::DEFAULT_CHAIN_ID,
			&CHARLIE,
			Some(&EVE)
		));

		let repatriated_amount = 10;
		let amount_slashed = 10;
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED - amount_slashed);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), amount - amount_slashed);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0 + repatriated_amount);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(
			TokensOf::<Test>::total_issuance(),
			TOKENS_ENDOWED * 4 - (amount_slashed - repatriated_amount)
		);

		let amount = 10;
		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(DAVE),
			consts::DEFAULT_CHAIN_ID,
			amount
		));

		let total_issuance_0 = TokensOf::<Test>::total_issuance();
		assert_eq!(TokensOf::<Test>::total_balance(&DAVE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&DAVE), amount);

		assert_ok!(SequencerStaking::slash_sequencer(consts::DEFAULT_CHAIN_ID, &DAVE, None));

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
		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(CHARLIE),
			consts::DEFAULT_CHAIN_ID,
			amount
		));

		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), amount);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(TokensOf::<Test>::total_issuance(), TOKENS_ENDOWED * 4);

		assert_ok!(SequencerStaking::slash_sequencer(
			consts::DEFAULT_CHAIN_ID,
			&CHARLIE,
			Some(&EVE)
		));

		let repatriated_amount = 20;
		let amount_slashed = 50;
		assert_eq!(TokensOf::<Test>::total_balance(&CHARLIE), TOKENS_ENDOWED - amount_slashed);
		assert_eq!(TokensOf::<Test>::reserved_balance(&CHARLIE), amount - amount_slashed);
		assert_eq!(TokensOf::<Test>::total_balance(&EVE), 0 + repatriated_amount);
		assert_eq!(TokensOf::<Test>::reserved_balance(&EVE), 0);
		assert_eq!(
			TokensOf::<Test>::total_issuance(),
			TOKENS_ENDOWED * 4 - (amount_slashed - repatriated_amount)
		);

		let amount = 50;
		assert_ok!(SequencerStaking::provide_sequencer_stake(
			RuntimeOrigin::signed(DAVE),
			consts::DEFAULT_CHAIN_ID,
			amount
		));

		let total_issuance_0 = TokensOf::<Test>::total_issuance();
		assert_eq!(TokensOf::<Test>::total_balance(&DAVE), TOKENS_ENDOWED);
		assert_eq!(TokensOf::<Test>::reserved_balance(&DAVE), amount);

		assert_ok!(SequencerStaking::slash_sequencer(consts::DEFAULT_CHAIN_ID, &DAVE, None));

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

		SequencerStaking::set_active_sequencers(
			[
				(consts::DEFAULT_CHAIN_ID, ALICE),
				(consts::DEFAULT_CHAIN_ID, BOB),
				(consts::DEFAULT_CHAIN_ID, CHARLIE),
			]
			.iter()
			.cloned(),
		)
		.unwrap();

		let handle_sequencer_deactivations_mock =
			MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock
			.expect()
			.with(eq(consts::DEFAULT_CHAIN_ID), eq(vec![BOB, CHARLIE]))
			.times(1)
			.return_const(());

		SequencerStaking::maybe_remove_sequencers_from_active_set(
			[BOB, CHARLIE, EVE].iter().cloned().collect(),
		);
	});
}

#[test]
#[serial]
fn test_remove_sequencers_works_correctly() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		let handle_sequencer_deactivations_mock =
			MockRolldownProviderApi::handle_sequencer_deactivations_context();
		handle_sequencer_deactivations_mock.expect().return_const(());

		// 1.
		SelectedSequencer::<Test>::mutate(|set| set.insert(consts::DEFAULT_CHAIN_ID, 4));
		NextSequencerIndex::<Test>::mutate(|ids| ids.insert(consts::DEFAULT_CHAIN_ID, 6));

		SequencerStaking::set_active_sequencers(
			(0u64..11u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		SequencerStaking::remove_sequencers_from_active_set(
			consts::DEFAULT_CHAIN_ID,
			[1, 4, 5, 6, 8, 11].iter().cloned().collect(),
		);

		assert_eq!(SelectedSequencer::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), None);
		assert_eq!(NextSequencerIndex::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&3u32));

		assert_eq!(
			ActiveSequencers::<Test>::get()
				.get(&consts::DEFAULT_CHAIN_ID)
				.unwrap()
				.clone()
				.into_inner(),
			[0, 2, 3, 7, 9, 10].iter().cloned().collect::<Vec<_>>()
		);

		// 2.
		SelectedSequencer::<Test>::mutate(|set| set.insert(consts::DEFAULT_CHAIN_ID, 4));
		NextSequencerIndex::<Test>::mutate(|ids| ids.insert(consts::DEFAULT_CHAIN_ID, 4));
		SequencerStaking::set_active_sequencers(
			(0u64..11u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		SequencerStaking::remove_sequencers_from_active_set(
			consts::DEFAULT_CHAIN_ID,
			std::iter::once(4).collect(),
		);

		assert_eq!(SelectedSequencer::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), None);
		assert_eq!(NextSequencerIndex::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&4u32));
		assert_eq!(
			ActiveSequencers::<Test>::get()
				.get(&consts::DEFAULT_CHAIN_ID)
				.unwrap()
				.clone()
				.into_inner(),
			[0, 1, 2, 3, 5, 6, 7, 8, 9, 10].iter().cloned().collect::<Vec<_>>()
		);

		// 3.
		SelectedSequencer::<Test>::mutate(|set| set.insert(consts::DEFAULT_CHAIN_ID, 4));
		NextSequencerIndex::<Test>::mutate(|ids| ids.insert(consts::DEFAULT_CHAIN_ID, 6));
		SequencerStaking::set_active_sequencers(
			(0u64..11u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		SequencerStaking::remove_sequencers_from_active_set(
			consts::DEFAULT_CHAIN_ID,
			[2, 3, 5, 8, 11].iter().cloned().collect(),
		);

		assert_eq!(SelectedSequencer::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&4u64));
		assert_eq!(NextSequencerIndex::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&3u32));
		assert_eq!(
			ActiveSequencers::<Test>::get()
				.get(&consts::DEFAULT_CHAIN_ID)
				.unwrap()
				.clone()
				.into_inner(),
			[0, 1, 4, 6, 7, 9, 10].iter().cloned().collect::<Vec<_>>()
		);
	});
}

#[test]
#[serial]
fn test_on_finalize_works_correctly() {
	set_default_mocks!();
	ExtBuilder::new().build().execute_with(|| {
		forward_to_block::<Test>(10);

		// 1.
		SelectedSequencer::<Test>::mutate(|set| set.insert(consts::DEFAULT_CHAIN_ID, 5));
		NextSequencerIndex::<Test>::mutate(|ids| ids.insert(consts::DEFAULT_CHAIN_ID, 6));
		SequencerStaking::set_active_sequencers(
			(0u64..11u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		SequencerStaking::on_finalize(10);

		assert_eq!(SelectedSequencer::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&6u64));
		assert_eq!(NextSequencerIndex::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&7u32));

		// 2.
		SelectedSequencer::<Test>::mutate(|set| set.insert(consts::DEFAULT_CHAIN_ID, 5));
		NextSequencerIndex::<Test>::mutate(|ids| ids.insert(consts::DEFAULT_CHAIN_ID, 12));
		SequencerStaking::set_active_sequencers(
			(0u64..11u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		SequencerStaking::on_finalize(10);

		assert_eq!(SelectedSequencer::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&0u64));
		assert_eq!(NextSequencerIndex::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&1u32));

		// // 3.
		SelectedSequencer::<Test>::mutate(|set| set.insert(consts::DEFAULT_CHAIN_ID, 5));
		NextSequencerIndex::<Test>::mutate(|ids| ids.insert(consts::DEFAULT_CHAIN_ID, 13));
		SequencerStaking::set_active_sequencers(
			(0u64..11u64).map(|i| (consts::DEFAULT_CHAIN_ID, i)),
		)
		.unwrap();

		SequencerStaking::on_finalize(10);

		assert_eq!(SelectedSequencer::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&0u64));
		assert_eq!(NextSequencerIndex::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), Some(&1u32));

		// 4.
		SelectedSequencer::<Test>::mutate(|set| set.insert(consts::DEFAULT_CHAIN_ID, 5));
		NextSequencerIndex::<Test>::mutate(|ids| ids.insert(consts::DEFAULT_CHAIN_ID, 13));
		SequencerStaking::set_active_sequencers(Vec::new()).unwrap();

		SequencerStaking::on_finalize(10);

		assert_eq!(SelectedSequencer::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), None);
		assert_eq!(NextSequencerIndex::<Test>::get().get(&consts::DEFAULT_CHAIN_ID), None);
	});
}
