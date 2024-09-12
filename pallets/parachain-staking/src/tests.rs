// Copyright 2019-2021 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! # Staking Pallet Unit Tests
//! The unit tests are organized by the call they test. The order matches the order
//! of the calls in the `lib.rs`.
//! 1. Root
//! 2. Monetary Governance
//! 3. Public (Collator, Nominator)
//! 4. Miscellaneous Property-Based Tests

use crate::mock::{
	payout_collator_for_round, roll_to, set_author, AccountId, Balance, ExtBuilder,
	RuntimeEvent as MetaEvent, RuntimeOrigin as Origin, Stake, StakeCurrency, Test, MGA_TOKEN_ID,
};

use crate::{
	assert_eq_events, assert_event_emitted, assert_last_event, Bond, CandidateBondChange,
	CandidateBondRequest, CollatorStatus, DelegationChange, DelegationRequest, DelegatorAdded,
	Error, Event, MetadataUpdateAction, PairedOrLiquidityToken, PayoutRounds, RoundAggregatorInfo,
	RoundCollatorRewardInfo, TotalSelected,
};
use frame_support::{assert_noop, assert_ok, traits::tokens::currency::MultiTokenCurrency};
use orml_tokens::MultiTokenReservableCurrency;
use sp_runtime::{traits::Zero, DispatchError, ModuleError, Perbill};

// ~~ ROOT ~~

#[test]
fn invalid_root_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_total_selected(Origin::signed(45), 6u32),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			Stake::set_collator_commission(Origin::signed(45), Perbill::from_percent(5)),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

// SET TOTAL SELECTED

#[test]
fn set_total_selected_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_total_selected(Origin::root(), 6u32));
		assert_last_event!(MetaEvent::Stake(Event::TotalSelectedSet(5u32, 6u32)));
	});
}

#[test]
fn set_total_selected_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Stake::total_selected(), 5u32);
		assert_ok!(Stake::set_total_selected(Origin::root(), 6u32));
		assert_eq!(Stake::total_selected(), 6u32);
	});
}

#[test]
fn cannot_set_total_selected_to_current_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_total_selected(Origin::root(), 5u32),
			Error::<Test>::NoWritingSameValue
		);
	});
}

#[test]
fn cannot_set_total_selected_below_module_min() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_total_selected(Origin::root(), 4u32),
			Error::<Test>::CannotSetBelowMin
		);
	});
}

// SET COLLATOR COMMISSION

#[test]
fn set_collator_commission_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_collator_commission(Origin::root(), Perbill::from_percent(5)));
		assert_last_event!(MetaEvent::Stake(Event::CollatorCommissionSet(
			Perbill::from_percent(20),
			Perbill::from_percent(5),
		)));
	});
}

#[test]
fn set_collator_commission_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Stake::collator_commission(), Perbill::from_percent(20));
		assert_ok!(Stake::set_collator_commission(Origin::root(), Perbill::from_percent(5)));
		assert_eq!(Stake::collator_commission(), Perbill::from_percent(5));
	});
}

#[test]
fn cannot_set_collator_commission_to_current_collator_commission() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_collator_commission(Origin::root(), Perbill::from_percent(20)),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// ~~ PUBLIC ~~

// JOIN CANDIDATES

#[test]
fn join_candidates_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(999, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::join_candidates(
				Origin::signed(1),
				10u128,
				1u32,
				None,
				1u32,
				10000u32
			));
			assert_last_event!(MetaEvent::Stake(Event::JoinedCollatorCandidates(
				1, 10u128, 20u128,
			)));
		});
}

#[test]
fn join_candidates_reserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(999, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &1), 0);
			assert_eq!(StakeCurrency::free_balance(1, &1), 10);
			assert_ok!(Stake::join_candidates(
				Origin::signed(1),
				10u128,
				1u32,
				None,
				1u32,
				10000u32
			));
			assert_eq!(StakeCurrency::reserved_balance(1, &1), 10);
			assert_eq!(StakeCurrency::free_balance(1, &1), 0);
		});
}

#[test]
fn join_candidates_increases_total_staked() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(999, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(1u32), 10);
			assert_ok!(Stake::join_candidates(
				Origin::signed(1),
				10u128,
				1u32,
				None,
				1u32,
				10000u32
			));
			assert_eq!(Stake::total(1u32), 20);
		});
}

#[test]
fn join_candidates_creates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(999, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::candidate_state(1).is_none());
			assert_ok!(Stake::join_candidates(
				Origin::signed(1),
				10u128,
				1u32,
				None,
				1u32,
				10000u32
			));
			let candidate_state = Stake::candidate_state(1).expect("just joined => exists");
			assert_eq!(candidate_state.bond, 10u128);
		});
}

#[test]
fn join_candidates_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(999, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::candidate_pool().0.len(), 1usize);
			assert_ok!(Stake::join_candidates(
				Origin::signed(1),
				10u128,
				1u32,
				None,
				1u32,
				10000u32
			));
			let candidate_pool = Stake::candidate_pool();
			assert_eq!(
				candidate_pool.0[0],
				Bond { owner: 1, amount: 10u128, liquidity_token: 1u32 }
			);
		});
}

#[test]
fn cannot_join_candidates_if_candidate() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 1000)])
		.with_default_token_candidates(vec![(1, 500)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(1), 11u128, 1u32, None, 100u32, 10000u32),
				Error::<Test>::CandidateExists
			);
		});
}

#[test]
fn cannot_join_candidates_if_delegator() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50), (2, 20)])
		.with_default_token_candidates(vec![(1, 50)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(2), 10u128, 1u32, None, 1u32, 10000u32),
				Error::<Test>::DelegatorExists
			);
		});
}

#[test]
fn cannot_join_candidates_without_min_bond() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 1000)])
		.with_default_token_candidates(vec![(999, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(1), 9u128, 1u32, None, 100u32, 10000u32),
				Error::<Test>::CandidateBondBelowMin
			);
		});
}

#[test]
fn cannot_join_candidates_with_more_than_available_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 500)])
		.with_default_token_candidates(vec![(999, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(1), 501u128, 1u32, None, 100u32, 10000u32),
				DispatchError::Module(ModuleError {
					index: 1,
					error: [0; 4],
					message: Some("BalanceTooLow")
				})
			);
		});
}

#[test]
fn insufficient_join_candidates_weight_hint_fails() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			for i in 0..5 {
				assert_noop!(
					Stake::join_candidates(Origin::signed(6), 20, 1u32, None, i, 10000u32),
					Error::<Test>::TooLowCandidateCountWeightHintJoinCandidates
				);
			}
		});
}

#[test]
fn sufficient_join_candidates_weight_hint_succeeds() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 20),
			(2, 20),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			let mut count = 5u32;
			for i in 6..10 {
				assert_ok!(Stake::join_candidates(
					Origin::signed(i),
					20,
					1u32,
					None,
					10000u32,
					count
				));
				count += 1u32;
			}
		});
}

// SCHEDULE LEAVE CANDIDATES

#[test]
fn leave_candidates_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_last_event!(MetaEvent::Stake(Event::CandidateScheduledExit(0, 1, 2)));
		});
}

#[test]
fn leave_candidates_removes_candidate_from_candidate_pool() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::candidate_pool().0.len(), 1);
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert!(Stake::candidate_pool().0.is_empty());
		});
}

#[test]
fn cannot_leave_candidates_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_leave_candidates(Origin::signed(1), 1u32),
			Error::<Test>::CandidateDNE
		);
	});
}

#[test]
fn cannot_leave_candidates_if_already_leaving_candidates() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_noop!(
				Stake::schedule_leave_candidates(Origin::signed(1), 1u32),
				Error::<Test>::CandidateAlreadyLeaving
			);
		});
}

#[test]
fn insufficient_leave_candidates_weight_hint_fails() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			for i in 1..6 {
				assert_noop!(
					Stake::schedule_leave_candidates(Origin::signed(i), 4u32),
					Error::<Test>::TooLowCandidateCountToLeaveCandidates
				);
			}
		});
}

#[test]
fn sufficient_leave_candidates_weight_hint_succeeds() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			let mut count = 5u32;
			for i in 1..6 {
				assert_ok!(Stake::schedule_leave_candidates(Origin::signed(i), count));
				count -= 1u32;
			}
		});
}

// EXECUTE LEAVE CANDIDATES

#[test]
fn execute_leave_candidates_emits_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1, 10000u32));
			assert_last_event!(MetaEvent::Stake(Event::CandidateLeft(1, 10, 0)));
		});
}

#[test]
fn execute_leave_candidates_callable_by_any_signed() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(2), 1, 10000u32));
		});
}

#[test]
fn execute_leave_candidates_unreserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &1), 10);
			assert_eq!(StakeCurrency::free_balance(1, &1), 0);
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1, 10000u32));
			assert_eq!(StakeCurrency::reserved_balance(1, &1), 0);
			assert_eq!(StakeCurrency::free_balance(1, &1), 10);
		});
}

#[test]
fn execute_leave_candidates_decreases_total_staked() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(1u32), 10);
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1, 10000u32));
			assert_eq!(Stake::total(1u32), 0);
		});
}

#[test]
fn execute_leave_candidates_removes_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			// candidate state is not immediately removed
			let candidate_state = Stake::candidate_state(1).expect("just left => still exists");
			assert_eq!(candidate_state.bond, 10u128);
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1, 10000u32));
			assert!(Stake::candidate_state(1).is_none());
		});
}

#[test]
fn cannot_execute_leave_candidates_before_delay() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_noop!(
				Stake::execute_leave_candidates(Origin::signed(3), 1, 10000u32),
				Error::<Test>::CandidateCannotLeaveYet
			);
			roll_to(8);
			assert_noop!(
				Stake::execute_leave_candidates(Origin::signed(3), 1, 10000u32),
				Error::<Test>::CandidateCannotLeaveYet
			);
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(3), 1, 10000u32));
		});
}

// CANCEL LEAVE CANDIDATES

#[test]
fn cancel_leave_candidates_emits_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_ok!(Stake::cancel_leave_candidates(Origin::signed(1), 1));
			assert_last_event!(MetaEvent::Stake(Event::CancelledCandidateExit(1)));
		});
}

#[test]
fn cancel_leave_candidates_updates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_ok!(Stake::cancel_leave_candidates(Origin::signed(1), 1));
			let candidate = Stake::candidate_state(&1).expect("just cancelled leave so exists");
			assert!(candidate.is_active());
		});
}

#[test]
fn cancel_leave_candidates_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 10)])
		.with_default_token_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_ok!(Stake::cancel_leave_candidates(Origin::signed(1), 1));
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond { owner: 1, amount: 10, liquidity_token: 1u32 }
			);
		});
}

// GO OFFLINE

#[test]
fn go_offline_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert_last_event!(MetaEvent::Stake(Event::CandidateWentOffline(0, 1)));
		});
}

#[test]
fn go_offline_removes_candidate_from_candidate_pool() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::candidate_pool().0.len(), 1);
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert!(Stake::candidate_pool().0.is_empty());
		});
}

#[test]
fn go_offline_updates_candidate_state_to_idle() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let candidate_state = Stake::candidate_state(1).expect("is active candidate");
			assert_eq!(candidate_state.state, CollatorStatus::Active);
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			let candidate_state = Stake::candidate_state(1).expect("is candidate, just offline");
			assert_eq!(candidate_state.state, CollatorStatus::Idle);
		});
}

#[test]
fn cannot_go_offline_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(Stake::go_offline(Origin::signed(3)), Error::<Test>::CandidateDNE);
	});
}

#[test]
fn cannot_go_offline_if_already_offline() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert_noop!(Stake::go_offline(Origin::signed(1)), Error::<Test>::AlreadyOffline);
		});
}

// GO ONLINE

#[test]
fn go_online_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert_ok!(Stake::go_online(Origin::signed(1)));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBackOnline(0, 1)));
		});
}

#[test]
fn go_online_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert!(Stake::candidate_pool().0.is_empty());
			assert_ok!(Stake::go_online(Origin::signed(1)));
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond { owner: 1, amount: 20, liquidity_token: 1u32 }
			);
		});
}

#[test]
fn go_online_storage_updates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			let candidate_state = Stake::candidate_state(1).expect("offline still exists");
			assert_eq!(candidate_state.state, CollatorStatus::Idle);
			assert_ok!(Stake::go_online(Origin::signed(1)));
			let candidate_state = Stake::candidate_state(1).expect("online so exists");
			assert_eq!(candidate_state.state, CollatorStatus::Active);
		});
}

#[test]
fn cannot_go_online_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(Stake::go_online(Origin::signed(3)), Error::<Test>::CandidateDNE);
	});
}

#[test]
fn cannot_go_online_if_already_online() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(Stake::go_online(Origin::signed(1)), Error::<Test>::AlreadyActive);
		});
}

#[test]
fn cannot_go_online_if_leaving() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_noop!(
				Stake::go_online(Origin::signed(1)),
				Error::<Test>::CannotGoOnlineIfLeaving
			);
		});
}

// SCHEDULE CANDIDATE BOND MORE

#[test]
fn schedule_candidate_bond_more_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBondMoreRequested(1, 30, 2,)));
		});
}

#[test]
fn schedule_candidate_bond_more_updates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None));
			let state = Stake::candidate_state(&1).expect("request bonded more so exists");
			assert_eq!(
				state.request,
				Some(CandidateBondRequest {
					amount: 30,
					change: CandidateBondChange::Increase,
					when_executable: 2,
				})
			);
		});
}

#[test]
fn cannot_schedule_candidate_bond_more_if_request_exists() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 40)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 5, None));
			assert_noop!(
				Stake::schedule_candidate_bond_more(Origin::signed(1), 5, None),
				Error::<Test>::PendingCandidateRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_schedule_candidate_bond_more_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_candidate_bond_more(Origin::signed(6), 50, None),
			Error::<Test>::CandidateDNE
		);
	});
}

#[test]
fn cannot_schedule_candidate_bond_more_if_insufficient_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_candidate_bond_more(Origin::signed(1), 1, None),
				Error::<Test>::InsufficientBalance
			);
		});
}

#[test]
fn can_schedule_candidate_bond_more_if_leaving_candidates() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None));
		});
}

#[test]
fn cannot_schedule_candidate_bond_more_if_exited_candidates() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1, 10000u32));
			assert_noop!(
				Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None),
				Error::<Test>::CandidateDNE
			);
		});
}

// CANDIDATE BOND LESS

#[test]
fn schedule_candidate_bond_less_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBondLessRequested(1, 10, 2,)));
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_request_exists() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 5));
			assert_noop!(
				Stake::schedule_candidate_bond_less(Origin::signed(1), 5),
				Error::<Test>::PendingCandidateRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_candidate_bond_less(Origin::signed(6), 50),
			Error::<Test>::CandidateDNE
		);
	});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_new_total_below_min_candidate_stk() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_candidate_bond_less(Origin::signed(1), 21),
				Error::<Test>::CandidateBondBelowMin
			);
		});
}

#[test]
fn can_schedule_candidate_bond_less_if_leaving_candidates() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_exited_candidates() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1, 10000u32));
			assert_noop!(
				Stake::schedule_candidate_bond_less(Origin::signed(1), 10),
				Error::<Test>::CandidateDNE
			);
		});
}

// EXECUTE CANDIDATE BOND REQUEST
// 1. BOND MORE REQUEST

#[test]
fn execute_candidate_bond_more_emits_correct_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBondedMore(1, 30, 50)));
		});
}

#[test]
fn execute_candidate_bond_more_reserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &1), 20);
			assert_eq!(StakeCurrency::free_balance(1, &1), 30);
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			assert_eq!(StakeCurrency::reserved_balance(1, &1), 50);
			assert_eq!(StakeCurrency::free_balance(1, &1), 0);
		});
}

#[test]
fn execute_candidate_bond_more_increases_total() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let mut total = Stake::total(1u32);
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			total += 30;
			assert_eq!(Stake::total(1u32), total);
		});
}

#[test]
fn execute_candidate_bond_more_updates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let candidate_state = Stake::candidate_state(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 20);
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			let candidate_state = Stake::candidate_state(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 50);
		});
}

#[test]
fn execute_candidate_bond_more_updates_candidate_pool() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond { owner: 1, amount: 20, liquidity_token: 1u32 }
			);
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 30, None));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond { owner: 1, amount: 50, liquidity_token: 1u32 }
			);
		});
}

// 2. BOND LESS REQUEST

#[test]
fn execute_candidate_bond_less_emits_correct_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 50)])
		.with_default_token_candidates(vec![(1, 50)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 30));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBondedLess(1, 30, 20)));
		});
}

#[test]
fn execute_candidate_bond_less_unreserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &1), 30);
			assert_eq!(StakeCurrency::free_balance(1, &1), 0);
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			assert_eq!(StakeCurrency::reserved_balance(1, &1), 20);
			assert_eq!(StakeCurrency::free_balance(1, &1), 10);
		});
}

#[test]
fn execute_candidate_bond_less_decreases_total() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			let mut total = Stake::total(1u32);
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			total -= 10;
			assert_eq!(Stake::total(1u32), total);
		});
}

#[test]
fn execute_candidate_bond_less_updates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			let candidate_state = Stake::candidate_state(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 30);
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			let candidate_state = Stake::candidate_state(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 20);
		});
}

#[test]
fn execute_candidate_bond_less_updates_candidate_pool() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond { owner: 1, amount: 30, liquidity_token: 1u32 }
			);
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_request(Origin::signed(1), 1, None));
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond { owner: 1, amount: 20, liquidity_token: 1u32 }
			);
		});
}

// CANCEL CANDIDATE BOND REQUEST
// 1. CANCEL CANDIDATE BOND MORE REQUEST

#[test]
fn cancel_candidate_bond_more_emits_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 40)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 10, None));
			assert_ok!(Stake::cancel_candidate_bond_request(Origin::signed(1)));
			assert_last_event!(MetaEvent::Stake(Event::CancelledCandidateBondChange(
				1,
				CandidateBondRequest {
					amount: 10,
					change: CandidateBondChange::Increase,
					when_executable: 2,
				},
			)));
		});
}

#[test]
fn cancel_candidate_bond_more_updates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 40)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 10, None));
			assert_ok!(Stake::cancel_candidate_bond_request(Origin::signed(1)));
			assert!(Stake::candidate_state(&1).unwrap().request.is_none());
		});
}

#[test]
fn only_candidate_can_cancel_candidate_bond_more_request() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 40)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_more(Origin::signed(1), 10, None));
			assert_noop!(
				Stake::cancel_candidate_bond_request(Origin::signed(2)),
				Error::<Test>::CandidateDNE
			);
		});
}

// 2. CANCEL CANDIDATE BOND LESS REQUEST

#[test]
fn cancel_candidate_bond_less_emits_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			assert_ok!(Stake::cancel_candidate_bond_request(Origin::signed(1)));
			assert_last_event!(MetaEvent::Stake(Event::CancelledCandidateBondChange(
				1,
				CandidateBondRequest {
					amount: 10,
					change: CandidateBondChange::Decrease,
					when_executable: 2,
				},
			)));
		});
}

#[test]
fn cancel_candidate_bond_less_updates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			assert_ok!(Stake::cancel_candidate_bond_request(Origin::signed(1)));
			assert!(Stake::candidate_state(&1).unwrap().request.is_none());
		});
}

#[test]
fn only_candidate_can_cancel_candidate_bond_less_request() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			assert_noop!(
				Stake::cancel_candidate_bond_request(Origin::signed(2)),
				Error::<Test>::CandidateDNE
			);
		});
}

// NOMINATE

#[test]
fn delegate_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, None, 0, 0));
			assert_last_event!(MetaEvent::Stake(Event::Delegation(
				2,
				10,
				1,
				DelegatorAdded::AddedToTop { new_total: 40 },
			)));
		});
}

#[test]
fn delegate_reserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 0);
			assert_eq!(StakeCurrency::free_balance(1, &2), 10);
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, None, 0, 0));
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 10);
			assert_eq!(StakeCurrency::free_balance(1, &2), 0);
		});
}

#[test]
fn delegate_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(2).is_none());
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, None, 0, 0));
			let delegator_state = Stake::delegator_state(2).expect("just delegated => exists");
			assert_eq!(
				delegator_state.delegations.0[0],
				Bond { owner: 1, amount: 10, liquidity_token: 1u32 }
			);
		});
}

#[test]
fn delegate_updates_collator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			let candidate_state = Stake::candidate_state(1).expect("registered in genesis");
			assert_eq!(candidate_state.total_backing, 30);
			assert_eq!(candidate_state.total_counted, 30);
			assert!(candidate_state.top_delegations.is_empty());
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, None, 0, 0));
			let candidate_state = Stake::candidate_state(1).expect("just delegated => exists");
			assert_eq!(candidate_state.total_backing, 40);
			assert_eq!(candidate_state.total_counted, 40);
			assert_eq!(
				candidate_state.top_delegations[0],
				Bond { owner: 2, amount: 10, liquidity_token: 1u32 }
			);
		});
}

#[test]
fn can_delegate_immediately_after_other_join_candidates() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 20)])
		.with_default_token_candidates(vec![(999, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::join_candidates(Origin::signed(1), 20, 1u32, None, 10000u32, 1));
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 20, None, 0, 0));
		});
}

#[test]
fn can_delegate_if_revoking() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 30), (3, 20), (4, 20)])
		.with_default_token_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_ok!(Stake::delegate(Origin::signed(2), 4, 10, None, 0, 2));
		});
}

#[test]
fn cannot_delegate_if_leaving() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 20), (3, 20)])
		.with_default_token_candidates(vec![(1, 20), (3, 20)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_noop!(
				Stake::delegate(Origin::signed(2), 3, 10, None, 0, 1),
				Error::<Test>::CannotDelegateIfLeaving
			);
		});
}

#[test]
fn cannot_delegate_if_candidate() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 20)])
		.with_default_token_candidates(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::delegate(Origin::signed(2), 1, 10, None, 0, 0),
				Error::<Test>::CandidateExists
			);
		});
}

#[test]
fn cannot_delegate_if_already_delegated() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 30)])
		.with_default_token_candidates(vec![(1, 20)])
		.with_delegations(vec![(2, 1, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::delegate(Origin::signed(2), 1, 10, None, 1, 1),
				Error::<Test>::AlreadyDelegatedCandidate
			);
		});
}

#[test]
fn cannot_delegate_more_than_max_delegations() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 50), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_default_token_candidates(vec![(1, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10), (2, 4, 10), (2, 5, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::delegate(Origin::signed(2), 6, 10, None, 0, 4),
				Error::<Test>::ExceedMaxDelegationsPerDelegator,
			);
		});
}

#[test]
fn sufficient_delegate_weight_hint_succeeds() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 20),
			(2, 20),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
			(10, 20),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			let mut count = 4u32;
			for i in 7..11 {
				assert_ok!(Stake::delegate(Origin::signed(i), 1, 10, None, count, 0u32));
				count += 1u32;
			}
			let mut count = 0u32;
			for i in 3..11 {
				assert_ok!(Stake::delegate(Origin::signed(i), 2, 10, None, count, 1u32));
				count += 1u32;
			}
		});
}

#[test]
fn insufficient_delegate_weight_hint_fails() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 20),
			(2, 20),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
			(10, 20),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			let mut count = 3u32;
			for i in 7..11 {
				assert_noop!(
					Stake::delegate(Origin::signed(i), 1, 10, None, count, 0u32),
					Error::<Test>::TooLowCandidateDelegationCountToDelegate
				);
			}
			// to set up for next error test
			count = 4u32;
			for i in 7..11 {
				assert_ok!(Stake::delegate(Origin::signed(i), 1, 10, None, count, 0u32));
				count += 1u32;
			}
			count = 0u32;
			for i in 3..11 {
				assert_noop!(
					Stake::delegate(Origin::signed(i), 2, 10, None, count, 0u32),
					Error::<Test>::TooLowDelegationCountToDelegate
				);
				count += 1u32;
			}
		});
}

// SCHEDULE LEAVE DELEGATORS

#[test]
fn schedule_leave_delegators_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_last_event!(MetaEvent::Stake(Event::DelegatorExitScheduled(0, 2, 2)));
		});
}

#[test]
fn cannot_schedule_leave_delegators_if_already_leaving() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_noop!(
				Stake::schedule_leave_delegators(Origin::signed(2)),
				Error::<Test>::DelegatorAlreadyLeaving
			);
		});
}

#[test]
fn cannot_schedule_leave_delegators_if_not_delegator() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_leave_delegators(Origin::signed(2)),
				Error::<Test>::DelegatorDNE
			);
		});
}

// EXECUTE LEAVE DELEGATORS

#[test]
fn execute_leave_delegators_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
			assert_event_emitted!(Event::DelegatorLeft(2, 10));
		});
}

#[test]
fn execute_leave_delegators_unreserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 10);
			assert_eq!(StakeCurrency::free_balance(1, &2), 0);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 0);
			assert_eq!(StakeCurrency::free_balance(1, &2), 10);
		});
}

#[test]
fn execute_leave_delegators_decreases_total_staked() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(1u32), 40);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
			assert_eq!(Stake::total(1u32), 30);
		});
}

#[test]
fn execute_leave_delegators_removes_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(2).is_some());
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
			assert!(Stake::delegator_state(2).is_none());
		});
}

#[test]
fn execute_leave_delegators_removes_delegations_from_collator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 100), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_default_token_candidates(vec![(2, 20), (3, 20), (4, 20), (5, 20)])
		.with_delegations(vec![(1, 2, 10), (1, 3, 10), (1, 4, 10), (1, 5, 10)])
		.build()
		.execute_with(|| {
			for i in 2..6 {
				let candidate_state =
					Stake::candidate_state(i).expect("initialized in ext builder");
				assert_eq!(
					candidate_state.top_delegations[0],
					Bond { owner: 1, amount: 10, liquidity_token: 1u32 }
				);
				assert_eq!(candidate_state.delegators.0[0], 1);
				assert_eq!(candidate_state.total_backing, 30);
			}
			assert_eq!(Stake::delegator_state(1).unwrap().delegations.0.len(), 4usize);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(1)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(1), 1, 10));
			for i in 2..6 {
				let candidate_state =
					Stake::candidate_state(i).expect("initialized in ext builder");
				assert!(candidate_state.top_delegations.is_empty());
				assert!(candidate_state.delegators.0.is_empty());
				assert_eq!(candidate_state.total_backing, 20);
			}
		});
}

#[test]
fn cannot_execute_leave_delegators_before_delay() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_noop!(
				Stake::execute_leave_delegators(Origin::signed(2), 2, 1),
				Error::<Test>::DelegatorCannotLeaveYet
			);
			// can execute after delay
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
		});
}

#[test]
fn insufficient_execute_leave_delegators_weight_hint_fails() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			for i in 3..7 {
				assert_ok!(Stake::schedule_leave_delegators(Origin::signed(i)));
			}
			roll_to(10);
			for i in 3..7 {
				assert_noop!(
					Stake::execute_leave_delegators(Origin::signed(i), i, 0),
					Error::<Test>::TooLowDelegationCountToLeaveDelegators
				);
			}
		});
}

#[test]
fn sufficient_execute_leave_delegators_weight_hint_succeeds() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_default_token_candidates(vec![(1, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			for i in 3..7 {
				assert_ok!(Stake::schedule_leave_delegators(Origin::signed(i)));
			}
			roll_to(10);
			for i in 3..7 {
				assert_ok!(Stake::execute_leave_delegators(Origin::signed(i), i, 1));
			}
		});
}

// CANCEL LEAVE DELEGATORS

#[test]
fn cancel_leave_delegators_emits_correct_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::cancel_leave_delegators(Origin::signed(2)));
			assert_last_event!(MetaEvent::Stake(Event::DelegatorExitCancelled(2)));
		});
}

#[test]
fn cancel_leave_delegators_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::cancel_leave_delegators(Origin::signed(2)));
			let delegator = Stake::delegator_state(&2).expect("just cancelled exit so exists");
			assert!(delegator.is_active());
		});
}

// SCHEDULE REVOKE DELEGATION

#[test]
fn revoke_delegation_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 20), (3, 30)])
		.with_default_token_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::DelegationRevocationScheduled(0, 2, 1, 2,)));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_event_emitted!(Event::DelegatorLeftCandidate(2, 1, 10, 30));
		});
}

#[test]
fn can_revoke_delegation_if_revoking_another_delegation() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 20), (3, 20)])
		.with_default_token_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			// this is an exit implicitly because last delegation revoked
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 3));
		});
}

#[test]
fn can_revoke_if_leaving() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 20), (3, 20)])
		.with_default_token_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 3));
		});
}

#[test]
fn cannot_revoke_delegation_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_revoke_delegation(Origin::signed(2), 1),
			Error::<Test>::DelegatorDNE
		);
	});
}

#[test]
fn cannot_revoke_delegation_that_dne() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_revoke_delegation(Origin::signed(2), 3),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
// See `cannot_execute_revoke_delegation_below_min_delegator_stake` for where the "must be above
// MinDelegatorStk" rule is now enforced.
fn can_schedule_revoke_delegation_below_min_delegator_stake() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 8), (3, 20)])
		.with_default_token_candidates(vec![(1, 20), (3, 20)])
		.with_delegations(vec![(2, 1, 5), (2, 3, 3)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
		});
}

// SCHEDULE DELEGATOR BOND MORE

#[test]
fn delegator_bond_more_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			assert_last_event!(MetaEvent::Stake(Event::DelegationIncreaseScheduled(2, 1, 5, 2,)));
		});
}

#[test]
fn delegator_bond_more_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			let state = Stake::delegator_state(&2).expect("just request bonded less so exists");
			assert_eq!(
				state.requests().get(&1),
				Some(&DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 2,
					action: DelegationChange::Increase
				})
			);
		});
}

#[test]
fn can_delegator_bond_more_if_leaving() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
		});
}

#[test]
fn cannot_delegator_bond_more_if_revoking() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 25), (3, 20)])
		.with_default_token_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_noop!(
				Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_delegator_bond_more_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None),
			Error::<Test>::DelegatorDNE
		);
	});
}

#[test]
fn cannot_delegator_bond_more_if_candidate_dne() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_more(Origin::signed(2), 3, 5, None),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
fn cannot_delegator_bond_more_if_delegation_dne() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10), (3, 30)])
		.with_default_token_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_more(Origin::signed(2), 3, 5, None),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
fn cannot_delegator_bond_more_if_insufficient_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None),
				Error::<Test>::InsufficientBalance
			);
		});
}

// DELEGATOR BOND LESS

#[test]
fn delegator_bond_less_event_emits_correctly() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			assert_last_event!(MetaEvent::Stake(Event::DelegationDecreaseScheduled(2, 1, 5, 2,)));
		});
}

#[test]
fn delegator_bond_less_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			let state = Stake::delegator_state(&2).expect("just request bonded less so exists");
			assert_eq!(
				state.requests().get(&1),
				Some(&DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 2,
					action: DelegationChange::Decrease
				})
			);
		});
}

#[test]
fn can_delegator_bond_less_if_leaving() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 1));
		});
}

#[test]
fn cannot_delegator_bond_less_if_revoking() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 25), (3, 20)])
		.with_default_token_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 1),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_delegator_bond_less_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5),
			Error::<Test>::DelegatorDNE
		);
	});
}

#[test]
fn cannot_delegator_bond_less_if_candidate_dne() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 3, 5),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
fn cannot_delegator_bond_less_if_delegation_dne() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10), (3, 30)])
		.with_default_token_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 3, 5),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
fn cannot_delegator_bond_less_more_than_total_delegation() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 11),
				Error::<Test>::DelegationBelowMin
			);
		});
}

#[test]
fn cannot_delegator_bond_less_below_min_delegation() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 20), (3, 30)])
		.with_default_token_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 8),
				Error::<Test>::DelegationBelowMin
			);
		});
}

// EXECUTE PENDING DELEGATION REQUEST

// 1. REVOKE DELEGATION

#[test]
fn execute_revoke_delegation_emits_exit_event_if_exit_happens() {
	// last delegation is revocation
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_event_emitted!(Event::DelegatorLeftCandidate(2, 1, 10, 30));
			assert_event_emitted!(Event::DelegatorLeft(2, 10));
		});
}

#[test]
fn revoke_delegation_executes_exit_if_last_delegation() {
	// last delegation is revocation
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_event_emitted!(Event::DelegatorLeftCandidate(2, 1, 10, 30));
			assert_event_emitted!(Event::DelegatorLeft(2, 10));
		});
}

#[test]
fn execute_revoke_delegation_emits_correct_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 20), (3, 30)])
		.with_default_token_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_event_emitted!(Event::DelegatorLeftCandidate(2, 1, 10, 30));
		});
}

#[test]
fn execute_revoke_delegation_unreserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 10);
			assert_eq!(StakeCurrency::free_balance(1, &2), 0);
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 0);
			assert_eq!(StakeCurrency::free_balance(1, &2), 10);
		});
}

#[test]
fn execute_revoke_delegation_adds_revocation_to_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 20), (3, 20)])
		.with_default_token_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(2).expect("exists").requests.requests.get(&1).is_none());
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert!(Stake::delegator_state(2).expect("exists").requests.requests.get(&1).is_some());
		});
}

#[test]
fn execute_revoke_delegation_removes_revocation_from_delegator_state_upon_execution() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 20), (3, 20)])
		.with_default_token_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert!(Stake::delegator_state(2).expect("exists").requests.requests.get(&1).is_none());
		});
}

#[test]
fn execute_revoke_delegation_decreases_total_staked() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(1u32), 40);
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(Stake::total(1u32), 30);
		});
}

#[test]
fn execute_revoke_delegation_for_last_delegation_removes_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(2).is_some());
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			// this will be confusing for people
			// if status is leaving, then execute_delegation_request works if last delegation
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert!(Stake::delegator_state(2).is_none());
		});
}

#[test]
fn execute_revoke_delegation_removes_delegation_from_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::candidate_state(1).expect("exists").delegators.0.len(), 1usize);
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert!(Stake::candidate_state(1).expect("exists").delegators.0.is_empty());
		});
}

#[test]
fn can_execute_revoke_delegation_for_leaving_candidate() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			// can execute delegation request for leaving candidate
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
		});
}

#[test]
fn can_execute_leave_candidates_if_revoking_candidate() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			// revocation executes during execute leave candidates (callable by anyone)
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1, 10000u32));
			assert!(!Stake::is_delegator(&2));
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 0);
			assert_eq!(StakeCurrency::free_balance(1, &2), 10);
		});
}

#[test]
fn delegator_bond_more_after_revoke_delegation_does_not_effect_exit() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 30), (3, 30)])
		.with_default_token_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_noop!(
				Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 10, None),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 3, 10, None));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 3, None));
			assert!(Stake::is_delegator(&2));
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 20);
			assert_eq!(StakeCurrency::free_balance(1, &2), 10);
		});
}

#[test]
fn delegator_bond_less_after_revoke_delegation_does_not_effect_exit() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 30), (3, 30)])
		.with_default_token_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::DelegationRevocationScheduled(0, 2, 1, 2,)));
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 2),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 3, 2));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 3, None));
			assert_last_event!(MetaEvent::Stake(Event::DelegationDecreased(2, 3, 2, true)));
			assert!(Stake::is_delegator(&2));
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 8);
			assert_eq!(StakeCurrency::free_balance(1, &2), 22);
		});
}

// 2. EXECUTE BOND MORE

#[test]
fn execute_delegator_bond_more_reserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 10);
			assert_eq!(StakeCurrency::free_balance(1, &2), 5);
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 15);
			assert_eq!(StakeCurrency::free_balance(1, &2), 0);
		});
}

#[test]
fn execute_delegator_bond_more_increases_total_staked() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(1u32), 40);
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(Stake::total(1u32), 45);
		});
}

#[test]
fn execute_delegator_bond_more_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
		});
}

#[test]
fn execute_delegator_bond_more_updates_candidate_state_top_delegations() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_state(1).expect("exists").top_delegations[0],
				Bond { owner: 2, amount: 10, liquidity_token: 1u32 }
			);
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(
				Stake::candidate_state(1).expect("exists").top_delegations[0],
				Bond { owner: 2, amount: 15, liquidity_token: 1u32 }
			);
		});
}

#[test]
fn execute_delegator_bond_more_updates_candidate_state_bottom_delegations() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10), (3, 1, 20), (4, 1, 20), (5, 1, 20), (6, 1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_state(1).expect("exists").bottom_delegations[0],
				Bond { owner: 2, amount: 10, liquidity_token: 1u32 }
			);
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_last_event!(MetaEvent::Stake(Event::DelegationIncreased(2, 1, 5, false)));
			assert_eq!(
				Stake::candidate_state(1).expect("exists").bottom_delegations[0],
				Bond { owner: 2, amount: 15, liquidity_token: 1u32 }
			);
		});
}

#[test]
fn execute_delegator_bond_more_increases_total() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(1u32), 40);
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(Stake::total(1u32), 45);
		});
}

#[test]
fn can_execute_delegator_bond_more_for_leaving_candidate() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			roll_to(10);
			// can execute bond more delegation request for leaving candidate
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
		});
}

// 3, EXECUTE BOND LESS

#[test]
fn execute_delegator_bond_less_unreserves_balance() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 10);
			assert_eq!(StakeCurrency::free_balance(1, &2), 0);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(StakeCurrency::reserved_balance(1, &2), 5);
			assert_eq!(StakeCurrency::free_balance(1, &2), 5);
		});
}

#[test]
fn execute_delegator_bond_less_decreases_total_staked() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(1u32), 40);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(Stake::total(1u32), 35);
		});
}

#[test]
fn execute_delegator_bond_less_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
		});
}

#[test]
fn execute_delegator_bond_less_updates_candidate_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_state(1).expect("exists").top_delegations[0],
				Bond { owner: 2, amount: 10, liquidity_token: 1u32 }
			);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(
				Stake::candidate_state(1).expect("exists").top_delegations[0],
				Bond { owner: 2, amount: 5, liquidity_token: 1u32 }
			);
		});
}

#[test]
fn execute_delegator_bond_less_decreases_total() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(1u32), 40);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_eq!(Stake::total(1u32), 35);
		});
}

#[test]
fn execute_delegator_bond_less_updates_just_bottom_delegations() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 10), (3, 11), (4, 12), (5, 14), (6, 15)])
		.with_default_token_candidates(vec![(1, 20)])
		.with_delegations(vec![(2, 1, 10), (3, 1, 11), (4, 1, 12), (5, 1, 14), (6, 1, 15)])
		.build()
		.execute_with(|| {
			let pre_call_collator_state =
				Stake::candidate_state(&1).expect("delegated by all so exists");
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 2));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			let post_call_collator_state =
				Stake::candidate_state(&1).expect("delegated by all so exists");
			let mut not_equal = false;
			for Bond { owner, amount, .. } in pre_call_collator_state.bottom_delegations {
				for Bond { owner: post_owner, amount: post_amount, .. } in
					&post_call_collator_state.bottom_delegations
				{
					if &owner == post_owner {
						if &amount != post_amount {
							not_equal = true;
							break;
						}
					}
				}
			}
			assert!(not_equal);
			let mut equal = true;
			for Bond { owner, amount, .. } in pre_call_collator_state.top_delegations {
				for Bond { owner: post_owner, amount: post_amount, .. } in
					&post_call_collator_state.top_delegations
				{
					if &owner == post_owner {
						if &amount != post_amount {
							equal = false;
							break;
						}
					}
				}
			}
			assert!(equal);
			assert_eq!(
				pre_call_collator_state.total_backing - 2,
				post_call_collator_state.total_backing
			);
			assert_eq!(
				pre_call_collator_state.total_counted,
				post_call_collator_state.total_counted
			);
		});
}

#[test]
fn execute_delegator_bond_less_does_not_delete_bottom_delegations() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 10), (3, 11), (4, 12), (5, 14), (6, 15)])
		.with_default_token_candidates(vec![(1, 20)])
		.with_delegations(vec![(2, 1, 10), (3, 1, 11), (4, 1, 12), (5, 1, 14), (6, 1, 15)])
		.build()
		.execute_with(|| {
			let pre_call_collator_state =
				Stake::candidate_state(&1).expect("delegated by all so exists");
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(6), 1, 4));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(6), 6, 1, None));
			let post_call_collator_state =
				Stake::candidate_state(&1).expect("delegated by all so exists");
			let mut equal = true;
			for Bond { owner, amount, .. } in pre_call_collator_state.bottom_delegations {
				for Bond { owner: post_owner, amount: post_amount, .. } in
					&post_call_collator_state.bottom_delegations
				{
					if &owner == post_owner {
						if &amount != post_amount {
							equal = false;
							break;
						}
					}
				}
			}
			assert!(equal);
			let mut not_equal = false;
			for Bond { owner, amount, .. } in pre_call_collator_state.top_delegations {
				for Bond { owner: post_owner, amount: post_amount, .. } in
					&post_call_collator_state.top_delegations
				{
					if &owner == post_owner {
						if &amount != post_amount {
							not_equal = true;
							break;
						}
					}
				}
			}
			assert!(not_equal);
			assert_eq!(
				pre_call_collator_state.total_backing - 4,
				post_call_collator_state.total_backing
			);
			assert_eq!(
				pre_call_collator_state.total_counted - 4,
				post_call_collator_state.total_counted
			);
		});
}

#[test]
fn can_execute_delegator_bond_less_for_leaving_candidate() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			// can execute bond more delegation request for leaving candidate
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
		});
}

// CANCEL PENDING DELEGATION REQUEST
// 1. CANCEL REVOKE DELEGATION

#[test]
fn cancel_revoke_delegation_emits_correct_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::CancelledDelegationRequest(
				2,
				DelegationRequest {
					collator: 1,
					amount: 10,
					when_executable: 2,
					action: DelegationChange::Revoke,
				},
			)));
		});
}

#[test]
fn cancel_revoke_delegation_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 10)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			let state = Stake::delegator_state(&2).unwrap();
			assert_eq!(
				state.requests().get(&1),
				Some(&DelegationRequest {
					collator: 1,
					amount: 10,
					when_executable: 2,
					action: DelegationChange::Revoke,
				})
			);
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			let state = Stake::delegator_state(&2).unwrap();
			assert!(state.requests().get(&1).is_none());
		});
}

// 2. CANCEL DELEGATOR BOND MORE

#[test]
fn cancel_delegator_bond_more_emits_correct_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::CancelledDelegationRequest(
				2,
				DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 2,
					action: DelegationChange::Increase,
				},
			)));
		});
}

#[test]
fn cancel_delegator_bond_more_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(2), 1, 5, None));
			let state = Stake::delegator_state(&2).unwrap();
			assert_eq!(
				state.requests().get(&1),
				Some(&DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 2,
					action: DelegationChange::Increase,
				})
			);
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			let state = Stake::delegator_state(&2).unwrap();
			assert!(state.requests().get(&1).is_none());
		});
}

// 3. CANCEL DELEGATOR BOND LESS

#[test]
fn cancel_delegator_bond_less_correct_event() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::CancelledDelegationRequest(
				2,
				DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 2,
					action: DelegationChange::Decrease,
				},
			)));
		});
}

#[test]
fn cancel_delegator_bond_less_updates_delegator_state() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 30), (2, 15)])
		.with_default_token_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			let state = Stake::delegator_state(&2).unwrap();
			assert_eq!(
				state.requests().get(&1),
				Some(&DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 2,
					action: DelegationChange::Decrease,
				})
			);
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			let state = Stake::delegator_state(&2).unwrap();
			assert!(state.requests().get(&1).is_none());
		});
}

// ~~ PROPERTY-BASED TESTS ~~

#[test]
fn delegator_schedule_revocation() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 40), (3, 20), (4, 20), (5, 20)])
		.with_default_token_candidates(vec![(1, 20), (3, 20), (4, 20), (5, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10), (2, 4, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1, None));
			assert_ok!(Stake::delegate(Origin::signed(2), 5, 10, None, 0, 2));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 3));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 4));
			roll_to(20);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 3, None));
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 4, None));
		});
}

// #[ignore]
#[test]
fn reworked_deprecated_test() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 600, 0),
			(1, 100, 1),
			(2, 100, 1),
			(3, 100, 1),
			(4, 100, 1),
			(5, 100, 1),
			(6, 100, 1),
			(7, 100, 1),
			(8, 100, 1),
			(9, 100, 1),
			(10, 100, 1),
			(11, 1, 1),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegations(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(StakeCurrency::free_balance(1, &11), 1);
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 50),
				Event::CollatorChosen(2, 2, 40),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 1, 5, 140),
			];
			assert_eq_events!(expected.clone());
			assert_eq!(StakeCurrency::free_balance(1, &11), 1);
			roll_to(10);
			let mut new0 = vec![
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 40),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 4, 20),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 2, 5, 140),
			];
			expected.append(&mut new0);
			assert_eq_events!(expected.clone());
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(20);
			payout_collator_for_round(2);
			// distribute total issuance to collator 1 and its delegators 6, 7, 19
			let mut new = vec![
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 40),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 4, 20),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 3, 5, 140),
				Event::CollatorChosen(5, 1, 50),
				Event::CollatorChosen(5, 2, 40),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 4, 20),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 4, 5, 140),
				Event::Rewarded(2, 1, 157),
				Event::DelegatorDueReward(2, 1, 6, 48),
				Event::DelegatorDueReward(2, 1, 7, 48),
				Event::DelegatorDueReward(2, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());

			assert_eq!(StakeCurrency::free_balance(0, &11), 0);
			// ~ set block author as 1 for all blocks this round
			set_author(3, 1, 100);
			set_author(4, 1, 100);
			set_author(5, 1, 100);
			// 1. ensure delegators are paid for 2 rounds after they leave
			assert_noop!(
				Stake::schedule_leave_delegators(Origin::signed(66)),
				Error::<Test>::DelegatorDNE
			);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(6)));
			// fast forward to block in which delegator 6 exit executes
			roll_to(30);
			payout_collator_for_round(3);
			payout_collator_for_round(4);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(6), 6, 10));
			roll_to(35);
			payout_collator_for_round(5);
			let mut new2 = vec![
				Event::DelegatorExitScheduled(4, 6, 6),
				Event::CollatorChosen(6, 1, 50),
				Event::CollatorChosen(6, 2, 40),
				Event::CollatorChosen(6, 3, 20),
				Event::CollatorChosen(6, 4, 20),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 5, 5, 140),
				Event::CollatorChosen(7, 1, 50),
				Event::CollatorChosen(7, 2, 40),
				Event::CollatorChosen(7, 3, 20),
				Event::CollatorChosen(7, 4, 20),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 6, 5, 140),
				Event::Rewarded(3, 1, 157),
				Event::DelegatorDueReward(3, 1, 6, 48),
				Event::DelegatorDueReward(3, 1, 7, 48),
				Event::DelegatorDueReward(3, 1, 10, 48),
				Event::Rewarded(4, 1, 157),
				Event::DelegatorDueReward(4, 1, 6, 48),
				Event::DelegatorDueReward(4, 1, 7, 48),
				Event::DelegatorDueReward(4, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
				Event::DelegatorLeftCandidate(6, 1, 10, 40),
				Event::DelegatorLeft(6, 10),
				Event::CollatorChosen(8, 1, 40),
				Event::CollatorChosen(8, 2, 40),
				Event::CollatorChosen(8, 3, 20),
				Event::CollatorChosen(8, 4, 20),
				Event::CollatorChosen(8, 5, 10),
				Event::NewRound(35, 7, 5, 130),
				Event::Rewarded(5, 1, 157),
				Event::DelegatorDueReward(5, 1, 6, 48),
				Event::DelegatorDueReward(5, 1, 7, 48),
				Event::DelegatorDueReward(5, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			assert_eq!(StakeCurrency::free_balance(0, &11), 0);
			// 6 won't be paid for this round because they left already
			set_author(6, 1, 100);
			roll_to(40);
			payout_collator_for_round(6);
			// keep paying 6
			let mut new3 = vec![
				Event::CollatorChosen(9, 1, 40),
				Event::CollatorChosen(9, 2, 40),
				Event::CollatorChosen(9, 3, 20),
				Event::CollatorChosen(9, 4, 20),
				Event::CollatorChosen(9, 5, 10),
				Event::NewRound(40, 8, 5, 130),
				Event::Rewarded(6, 1, 157),
				Event::DelegatorDueReward(6, 1, 6, 48),
				Event::DelegatorDueReward(6, 1, 7, 48),
				Event::DelegatorDueReward(6, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new3);
			assert_eq_events!(expected.clone());
			assert_eq!(StakeCurrency::free_balance(0, &11), 0);
			set_author(7, 1, 100);
			roll_to(45);
			payout_collator_for_round(7);
			// no more paying 6
			let mut new4 = vec![
				Event::CollatorChosen(10, 1, 40),
				Event::CollatorChosen(10, 2, 40),
				Event::CollatorChosen(10, 3, 20),
				Event::CollatorChosen(10, 4, 20),
				Event::CollatorChosen(10, 5, 10),
				Event::NewRound(45, 9, 5, 130),
				Event::Rewarded(7, 1, 157),
				Event::DelegatorDueReward(7, 1, 6, 48),
				Event::DelegatorDueReward(7, 1, 7, 48),
				Event::DelegatorDueReward(7, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new4);
			assert_eq_events!(expected.clone());
			assert_eq!(StakeCurrency::free_balance(0, &11), 0);
			set_author(8, 1, 100);
			assert_ok!(Stake::delegate(Origin::signed(8), 1, 10, None, 10, 10));
			roll_to(50);
			payout_collator_for_round(8);
			// new delegation is not rewarded yet
			let mut new5 = vec![
				Event::Delegation(8, 10, 1, DelegatorAdded::AddedToTop { new_total: 50 }),
				Event::CollatorChosen(11, 1, 50),
				Event::CollatorChosen(11, 2, 40),
				Event::CollatorChosen(11, 3, 20),
				Event::CollatorChosen(11, 4, 20),
				Event::CollatorChosen(11, 5, 10),
				Event::NewRound(50, 10, 5, 140),
				Event::Rewarded(8, 1, 182),
				Event::DelegatorDueReward(8, 1, 7, 61),
				Event::DelegatorDueReward(8, 1, 10, 61),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new5);
			assert_eq_events!(expected.clone());
			assert_eq!(StakeCurrency::free_balance(0, &11), 0);
			set_author(9, 1, 100);
			set_author(10, 1, 100);
			roll_to(55);
			payout_collator_for_round(9);
			// new delegation is still not rewarded yet
			let mut new6 = vec![
				Event::CollatorChosen(12, 1, 50),
				Event::CollatorChosen(12, 2, 40),
				Event::CollatorChosen(12, 3, 20),
				Event::CollatorChosen(12, 4, 20),
				Event::CollatorChosen(12, 5, 10),
				Event::NewRound(55, 11, 5, 140),
				Event::Rewarded(9, 1, 182),
				Event::DelegatorDueReward(9, 1, 7, 61),
				Event::DelegatorDueReward(9, 1, 10, 61),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new6);
			assert_eq_events!(expected.clone());
			assert_eq!(StakeCurrency::free_balance(0, &11), 0);
			roll_to(60);
			payout_collator_for_round(10);
			// new delegation is rewarded, 2 rounds after joining (`RewardPaymentDelay` is 2)
			let mut new7 = vec![
				Event::CollatorChosen(13, 1, 50),
				Event::CollatorChosen(13, 2, 40),
				Event::CollatorChosen(13, 3, 20),
				Event::CollatorChosen(13, 4, 20),
				Event::CollatorChosen(13, 5, 10),
				Event::NewRound(60, 12, 5, 140),
				Event::Rewarded(10, 1, 182),
				Event::DelegatorDueReward(10, 1, 7, 61),
				Event::DelegatorDueReward(10, 1, 10, 61),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new7);
			assert_eq_events!(expected);
			assert_eq!(StakeCurrency::free_balance(0, &11), 0);
		});
}

#[test]
fn paid_collator_commission_matches_config() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 600, 0),
			(1, 100, 1),
			(2, 100, 1),
			(3, 100, 1),
			(4, 100, 1),
			(5, 100, 1),
			(6, 100, 1),
		])
		.with_default_token_candidates(vec![(1, 20)])
		.with_delegations(vec![(2, 1, 10), (3, 1, 10)])
		.build()
		.execute_with(|| {
			roll_to(7);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![Event::CollatorChosen(2, 1, 40), Event::NewRound(5, 1, 1, 40)];
			assert_eq_events!(expected.clone());
			assert_ok!(Stake::join_candidates(
				Origin::signed(4),
				20u128,
				1u32,
				None,
				100u32,
				10000u32
			));
			assert_last_event!(MetaEvent::Stake(Event::JoinedCollatorCandidates(
				4, 20u128, 60u128,
			)));
			roll_to(8);
			assert_ok!(Stake::delegate(Origin::signed(5), 4, 10, None, 10, 10));
			assert_ok!(Stake::delegate(Origin::signed(6), 4, 10, None, 10, 10));
			roll_to(10);
			let mut new = vec![
				Event::JoinedCollatorCandidates(4, 20, 60),
				Event::Delegation(5, 10, 4, DelegatorAdded::AddedToTop { new_total: 30 }),
				Event::Delegation(6, 10, 4, DelegatorAdded::AddedToTop { new_total: 40 }),
				Event::CollatorChosen(3, 1, 40),
				Event::CollatorChosen(3, 4, 40),
				Event::NewRound(10, 2, 2, 80),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			roll_to(15);
			let mut new1 = vec![
				Event::CollatorChosen(4, 1, 40),
				Event::CollatorChosen(4, 4, 40),
				Event::NewRound(15, 3, 2, 80),
			];
			expected.append(&mut new1);
			assert_eq_events!(expected.clone());
			// only reward author with id 4
			set_author(3, 4, 100);
			roll_to(25);
			payout_collator_for_round(3);
			// 20% of 10 is commission + due_portion (4) = 2 + 4 = 6
			// all delegator payouts are 10-2 = 8 * stake_pct
			let mut new2 = vec![
				Event::CollatorChosen(5, 1, 40),
				Event::CollatorChosen(5, 4, 40),
				Event::NewRound(20, 4, 2, 80),
				Event::CollatorChosen(6, 1, 40),
				Event::CollatorChosen(6, 4, 40),
				Event::NewRound(25, 5, 2, 80),
				Event::Rewarded(3, 4, 182),
				Event::DelegatorDueReward(3, 4, 5, 61),
				Event::DelegatorDueReward(3, 4, 6, 61),
				Event::CollatorRewardsDistributed(4, PayoutRounds::All),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected);
		});
}

#[test]
fn collator_exit_executes_after_delay() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 1000),
			(2, 300),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
		])
		.with_default_token_candidates(vec![(1, 500), (2, 200)])
		.with_delegations(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			roll_to(11);
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(2), 2));
			let info = Stake::candidate_state(&2).unwrap();
			assert_eq!(info.state, CollatorStatus::Leaving(4));
			roll_to(21);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(2), 2, 10000u32));
			// we must exclude leaving collators from rewards while
			// holding them retroactively accountable for previous faults
			// (within the last T::SlashingWindow blocks)
			let expected = vec![
				Event::CollatorChosen(2, 1, 700),
				Event::CollatorChosen(2, 2, 400),
				Event::NewRound(5, 1, 2, 1100),
				Event::CollatorChosen(3, 1, 700),
				Event::CollatorChosen(3, 2, 400),
				Event::NewRound(10, 2, 2, 1100),
				Event::CandidateScheduledExit(2, 2, 4),
				Event::CollatorChosen(4, 1, 700),
				Event::NewRound(15, 3, 1, 700),
				Event::CollatorChosen(5, 1, 700),
				Event::NewRound(20, 4, 1, 700),
				Event::CandidateLeft(2, 400, 700),
			];
			assert_eq_events!(expected);
		});
}

#[test]
fn collator_selection_chooses_top_candidates() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 33),
			(8, 33),
			(9, 33),
		])
		.with_default_token_candidates(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
		.build()
		.execute_with(|| {
			roll_to(8);
			// should choose top TotalSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 1, 5, 400),
			];
			assert_eq_events!(expected.clone());
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(6), 6));
			assert_last_event!(MetaEvent::Stake(Event::CandidateScheduledExit(1, 6, 3)));
			roll_to(21);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(6), 6, 10000u32));
			assert_ok!(Stake::join_candidates(
				Origin::signed(6),
				69u128,
				1u32,
				None,
				100u32,
				10000u32
			));
			assert_last_event!(MetaEvent::Stake(Event::JoinedCollatorCandidates(
				6, 69u128, 469u128,
			)));
			roll_to(27);
			// should choose top TotalSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 1, 5, 400),
				Event::CandidateScheduledExit(1, 6, 3),
				Event::CollatorChosen(3, 1, 100),
				Event::CollatorChosen(3, 2, 90),
				Event::CollatorChosen(3, 3, 80),
				Event::CollatorChosen(3, 4, 70),
				Event::CollatorChosen(3, 5, 60),
				Event::NewRound(10, 2, 5, 400),
				Event::CollatorChosen(4, 1, 100),
				Event::CollatorChosen(4, 2, 90),
				Event::CollatorChosen(4, 3, 80),
				Event::CollatorChosen(4, 4, 70),
				Event::CollatorChosen(4, 5, 60),
				Event::NewRound(15, 3, 5, 400),
				Event::CollatorChosen(5, 1, 100),
				Event::CollatorChosen(5, 2, 90),
				Event::CollatorChosen(5, 3, 80),
				Event::CollatorChosen(5, 4, 70),
				Event::CollatorChosen(5, 5, 60),
				Event::NewRound(20, 4, 5, 400),
				Event::CandidateLeft(6, 50, 400),
				Event::JoinedCollatorCandidates(6, 69, 469),
				Event::CollatorChosen(6, 1, 100),
				Event::CollatorChosen(6, 2, 90),
				Event::CollatorChosen(6, 3, 80),
				Event::CollatorChosen(6, 4, 70),
				Event::CollatorChosen(6, 6, 69),
				Event::NewRound(25, 5, 5, 409),
			];
			assert_eq_events!(expected);
		});
}

#[test]
fn payout_distribution_to_solo_collators() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 1000, 0),
			(1, 1000, 1),
			(2, 1000, 1),
			(3, 1000, 1),
			(4, 1000, 1),
			(5, 1000, 1),
			(6, 1000, 1),
			(7, 33, 1),
			(8, 33, 1),
			(9, 33, 1),
		])
		.with_default_token_candidates(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
		.build()
		.execute_with(|| {
			roll_to(8);
			// should choose top TotalCandidatesSelected (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 1, 5, 400),
			];
			assert_eq_events!(expected.clone());
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// pay total issuance to 1
			let mut new = vec![
				Event::CollatorChosen(3, 1, 100),
				Event::CollatorChosen(3, 2, 90),
				Event::CollatorChosen(3, 3, 80),
				Event::CollatorChosen(3, 4, 70),
				Event::CollatorChosen(3, 5, 60),
				Event::NewRound(10, 2, 5, 400),
				Event::CollatorChosen(4, 1, 100),
				Event::CollatorChosen(4, 2, 90),
				Event::CollatorChosen(4, 3, 80),
				Event::CollatorChosen(4, 4, 70),
				Event::CollatorChosen(4, 5, 60),
				Event::NewRound(15, 3, 5, 400),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			// ~ set block author as 1 for 3 blocks this round
			set_author(4, 1, 60);
			// ~ set block author as 2 for 2 blocks this round
			set_author(4, 2, 40);
			roll_to(26);
			payout_collator_for_round(2);
			// pay 60% total issuance to 1 and 40% total issuance to 2
			let mut new1 = vec![
				Event::CollatorChosen(5, 1, 100),
				Event::CollatorChosen(5, 2, 90),
				Event::CollatorChosen(5, 3, 80),
				Event::CollatorChosen(5, 4, 70),
				Event::CollatorChosen(5, 5, 60),
				Event::NewRound(20, 4, 5, 400),
				Event::CollatorChosen(6, 1, 100),
				Event::CollatorChosen(6, 2, 90),
				Event::CollatorChosen(6, 3, 80),
				Event::CollatorChosen(6, 4, 70),
				Event::CollatorChosen(6, 5, 60),
				Event::NewRound(25, 5, 5, 400),
				Event::Rewarded(2, 1, 304),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new1);
			assert_eq_events!(expected.clone());
			// ~ each collator produces 1 block this round
			set_author(6, 1, 20);
			set_author(6, 2, 20);
			set_author(6, 3, 20);
			set_author(6, 4, 20);
			set_author(6, 5, 20);
			roll_to(36);
			payout_collator_for_round(4);
			payout_collator_for_round(5);
			// pay 20% issuance for all collators
			let mut new2 = vec![
				Event::CollatorChosen(7, 1, 100),
				Event::CollatorChosen(7, 2, 90),
				Event::CollatorChosen(7, 3, 80),
				Event::CollatorChosen(7, 4, 70),
				Event::CollatorChosen(7, 5, 60),
				Event::NewRound(30, 6, 5, 400),
				Event::CollatorChosen(8, 1, 100),
				Event::CollatorChosen(8, 2, 90),
				Event::CollatorChosen(8, 3, 80),
				Event::CollatorChosen(8, 4, 70),
				Event::CollatorChosen(8, 5, 60),
				Event::NewRound(35, 7, 5, 400),
				Event::Rewarded(4, 1, 182),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
				Event::Rewarded(4, 2, 121),
				Event::CollatorRewardsDistributed(2, PayoutRounds::All),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected);

			roll_to(46);
			payout_collator_for_round(6);
			payout_collator_for_round(7);
			// pay 20% issuance for all collators
			let mut new3 = vec![
				Event::CollatorChosen(9, 1, 100),
				Event::CollatorChosen(9, 2, 90),
				Event::CollatorChosen(9, 3, 80),
				Event::CollatorChosen(9, 4, 70),
				Event::CollatorChosen(9, 5, 60),
				Event::NewRound(40, 8, 5, 400),
				Event::CollatorChosen(10, 1, 100),
				Event::CollatorChosen(10, 2, 90),
				Event::CollatorChosen(10, 3, 80),
				Event::CollatorChosen(10, 4, 70),
				Event::CollatorChosen(10, 5, 60),
				Event::NewRound(45, 9, 5, 400),
				Event::Rewarded(6, 5, 60),
				Event::CollatorRewardsDistributed(5, PayoutRounds::All),
				Event::Rewarded(6, 3, 60),
				Event::CollatorRewardsDistributed(3, PayoutRounds::All),
				Event::Rewarded(6, 1, 60),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
				Event::Rewarded(6, 4, 60),
				Event::CollatorRewardsDistributed(4, PayoutRounds::All),
				Event::Rewarded(6, 2, 60),
				Event::CollatorRewardsDistributed(2, PayoutRounds::All),
			];
			expected.append(&mut new3);
			assert_eq_events!(expected);
			// check that distributing rewards clears awarded pts
			assert!(Stake::awarded_pts(1, 1).is_zero());
			assert!(Stake::awarded_pts(4, 1).is_zero());
			assert!(Stake::awarded_pts(4, 2).is_zero());
			assert!(Stake::awarded_pts(6, 1).is_zero());
			assert!(Stake::awarded_pts(6, 2).is_zero());
			assert!(Stake::awarded_pts(6, 3).is_zero());
			assert!(Stake::awarded_pts(6, 4).is_zero());
			assert!(Stake::awarded_pts(6, 5).is_zero());
		});
}

#[test]
fn multiple_delegations() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegations(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.build()
		.execute_with(|| {
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 50),
				Event::CollatorChosen(2, 2, 40),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 1, 5, 140),
			];
			assert_eq_events!(expected.clone());
			assert_ok!(Stake::delegate(Origin::signed(6), 2, 10, None, 10, 10));
			assert_ok!(Stake::delegate(Origin::signed(6), 3, 10, None, 10, 10));
			assert_ok!(Stake::delegate(Origin::signed(6), 4, 10, None, 10, 10));
			roll_to(16);
			let mut new = vec![
				Event::Delegation(6, 10, 2, DelegatorAdded::AddedToTop { new_total: 50 }),
				Event::Delegation(6, 10, 3, DelegatorAdded::AddedToTop { new_total: 30 }),
				Event::Delegation(6, 10, 4, DelegatorAdded::AddedToTop { new_total: 30 }),
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 50),
				Event::CollatorChosen(3, 3, 30),
				Event::CollatorChosen(3, 4, 30),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 2, 5, 170),
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 50),
				Event::CollatorChosen(4, 3, 30),
				Event::CollatorChosen(4, 4, 30),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 3, 5, 170),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			roll_to(21);
			assert_ok!(Stake::delegate(Origin::signed(7), 2, 80, None, 10, 10));
			assert_ok!(Stake::delegate(Origin::signed(10), 2, 10, None, 10, 10),);
			roll_to(26);
			let mut new2 = vec![
				Event::CollatorChosen(5, 1, 50),
				Event::CollatorChosen(5, 2, 50),
				Event::CollatorChosen(5, 3, 30),
				Event::CollatorChosen(5, 4, 30),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 4, 5, 170),
				Event::Delegation(7, 80, 2, DelegatorAdded::AddedToTop { new_total: 130 }),
				Event::Delegation(10, 10, 2, DelegatorAdded::AddedToBottom),
				Event::CollatorChosen(6, 1, 50),
				Event::CollatorChosen(6, 2, 130),
				Event::CollatorChosen(6, 3, 30),
				Event::CollatorChosen(6, 4, 30),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 5, 5, 250),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(2), 5));
			assert_last_event!(MetaEvent::Stake(Event::CandidateScheduledExit(5, 2, 7)));
			roll_to(31);
			let mut new3 = vec![
				Event::CandidateScheduledExit(5, 2, 7),
				Event::CollatorChosen(7, 1, 50),
				Event::CollatorChosen(7, 3, 30),
				Event::CollatorChosen(7, 4, 30),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 6, 4, 120),
			];
			expected.append(&mut new3);
			assert_eq_events!(expected);
			// verify that delegations are removed after collator leaves, not before
			assert_eq!(Stake::delegator_state(7).unwrap().delegations.0.len(), 2usize);
			assert_eq!(Stake::delegator_state(6).unwrap().delegations.0.len(), 4usize);
			assert_eq!(StakeCurrency::reserved_balance(1, &6), 40);
			assert_eq!(StakeCurrency::reserved_balance(1, &7), 90);
			assert_eq!(StakeCurrency::free_balance(1, &6), 60);
			assert_eq!(StakeCurrency::free_balance(1, &7), 10);
			roll_to(40);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(2), 2, 10000u32));
			assert_eq!(Stake::delegator_state(7).unwrap().delegations.0.len(), 1usize);
			assert_eq!(Stake::delegator_state(6).unwrap().delegations.0.len(), 3usize);
			assert_eq!(StakeCurrency::reserved_balance(1, &6), 30);
			assert_eq!(StakeCurrency::reserved_balance(1, &7), 10);
			assert_eq!(StakeCurrency::free_balance(1, &6), 70);
			assert_eq!(StakeCurrency::free_balance(1, &7), 90);
		});
}

#[test]
fn payouts_follow_delegation_changes() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 1000, 0),
			(1, 100, 1),
			(2, 100, 1),
			(3, 100, 1),
			(4, 100, 1),
			(5, 100, 1),
			(6, 100, 1),
			(7, 100, 1),
			(8, 100, 1),
			(9, 100, 1),
			(10, 100, 1),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegations(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.build()
		.execute_with(|| {
			roll_to(10);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 50),
				Event::CollatorChosen(2, 2, 40),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 1, 5, 140),
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 40),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 4, 20),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 2, 5, 140),
			];
			assert_eq_events!(expected.clone());
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// distribute total issuance to collator 1 and its delegators 6, 7, 19
			let mut new = vec![
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 40),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 4, 20),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 3, 5, 140),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());

			// ~ set block author as 1 for all blocks this round
			set_author(3, 1, 100);
			set_author(4, 1, 100);
			set_author(5, 1, 100);
			set_author(6, 1, 100);
			// 1. ensure delegators are paid for 2 rounds after they leave
			assert_noop!(
				Stake::schedule_leave_delegators(Origin::signed(66)),
				Error::<Test>::DelegatorDNE
			);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(6)));
			// fast forward to block in which delegator 6 exit executes
			roll_to(25);

			payout_collator_for_round(2);
			payout_collator_for_round(3);

			assert_ok!(Stake::execute_leave_delegators(Origin::signed(6), 6, 10));
			// keep paying 6 (note: inflation is in terms of total issuance so that's why 1 is 21)
			let mut new2 = vec![
				Event::DelegatorExitScheduled(3, 6, 5),
				Event::CollatorChosen(5, 1, 50),
				Event::CollatorChosen(5, 2, 40),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 4, 20),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 4, 5, 140),
				Event::CollatorChosen(6, 1, 50),
				Event::CollatorChosen(6, 2, 40),
				Event::CollatorChosen(6, 3, 20),
				Event::CollatorChosen(6, 4, 20),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 5, 5, 140),
				Event::Rewarded(2, 1, 157),
				Event::DelegatorDueReward(2, 1, 6, 48),
				Event::DelegatorDueReward(2, 1, 7, 48),
				Event::DelegatorDueReward(2, 1, 10, 48),
				Event::Rewarded(3, 1, 157),
				Event::DelegatorDueReward(3, 1, 6, 48),
				Event::DelegatorDueReward(3, 1, 7, 48),
				Event::DelegatorDueReward(3, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
				Event::DelegatorLeftCandidate(6, 1, 10, 40),
				Event::DelegatorLeft(6, 10),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			// 6 won't be paid for this round because they left already
			set_author(7, 1, 100);
			roll_to(35);

			payout_collator_for_round(4);
			payout_collator_for_round(5);

			// keep paying 6
			let mut new3 = vec![
				Event::CollatorChosen(7, 1, 40),
				Event::CollatorChosen(7, 2, 40),
				Event::CollatorChosen(7, 3, 20),
				Event::CollatorChosen(7, 4, 20),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 6, 5, 130),
				Event::CollatorChosen(8, 1, 40),
				Event::CollatorChosen(8, 2, 40),
				Event::CollatorChosen(8, 3, 20),
				Event::CollatorChosen(8, 4, 20),
				Event::CollatorChosen(8, 5, 10),
				Event::NewRound(35, 7, 5, 130),
				Event::Rewarded(5, 1, 157),
				Event::DelegatorDueReward(5, 1, 6, 48),
				Event::DelegatorDueReward(5, 1, 7, 48),
				Event::DelegatorDueReward(5, 1, 10, 48),
				Event::Rewarded(4, 1, 157),
				Event::DelegatorDueReward(4, 1, 6, 48),
				Event::DelegatorDueReward(4, 1, 7, 48),
				Event::DelegatorDueReward(4, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new3);
			assert_eq_events!(expected.clone());
			set_author(8, 1, 100);
			roll_to(40);

			payout_collator_for_round(6);

			// no more paying 6
			let mut new4 = vec![
				Event::CollatorChosen(9, 1, 40),
				Event::CollatorChosen(9, 2, 40),
				Event::CollatorChosen(9, 3, 20),
				Event::CollatorChosen(9, 4, 20),
				Event::CollatorChosen(9, 5, 10),
				Event::NewRound(40, 8, 5, 130),
				Event::Rewarded(6, 1, 157),
				Event::DelegatorDueReward(6, 1, 6, 48),
				Event::DelegatorDueReward(6, 1, 7, 48),
				Event::DelegatorDueReward(6, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new4);
			assert_eq_events!(expected.clone());
			set_author(9, 1, 100);
			assert_ok!(Stake::delegate(Origin::signed(8), 1, 10, None, 10, 10));
			roll_to(45);

			payout_collator_for_round(7);

			// new delegation is not rewarded yet
			let mut new5 = vec![
				Event::Delegation(8, 10, 1, DelegatorAdded::AddedToTop { new_total: 50 }),
				Event::CollatorChosen(10, 1, 50),
				Event::CollatorChosen(10, 2, 40),
				Event::CollatorChosen(10, 3, 20),
				Event::CollatorChosen(10, 4, 20),
				Event::CollatorChosen(10, 5, 10),
				Event::NewRound(45, 9, 5, 140),
				Event::Rewarded(7, 1, 182),
				Event::DelegatorDueReward(7, 1, 7, 61),
				Event::DelegatorDueReward(7, 1, 10, 61),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new5);
			assert_eq_events!(expected.clone());
			set_author(10, 1, 100);
			roll_to(50);
			payout_collator_for_round(8);
			// new delegation not rewarded yet
			let mut new6 = vec![
				Event::CollatorChosen(11, 1, 50),
				Event::CollatorChosen(11, 2, 40),
				Event::CollatorChosen(11, 3, 20),
				Event::CollatorChosen(11, 4, 20),
				Event::CollatorChosen(11, 5, 10),
				Event::NewRound(50, 10, 5, 140),
				Event::Rewarded(8, 1, 182),
				Event::DelegatorDueReward(8, 1, 7, 61),
				Event::DelegatorDueReward(8, 1, 10, 61),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new6);
			assert_eq_events!(expected.clone());
			roll_to(55);
			payout_collator_for_round(9);
			// new delegation is rewarded for first time
			// 2 rounds after joining (`RewardPaymentDelay` = 2)
			let mut new7 = vec![
				Event::CollatorChosen(12, 1, 50),
				Event::CollatorChosen(12, 2, 40),
				Event::CollatorChosen(12, 3, 20),
				Event::CollatorChosen(12, 4, 20),
				Event::CollatorChosen(12, 5, 10),
				Event::NewRound(55, 11, 5, 140),
				Event::Rewarded(9, 1, 182),
				Event::DelegatorDueReward(9, 1, 7, 61),
				Event::DelegatorDueReward(9, 1, 10, 61),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new7);
			assert_eq_events!(expected);
			roll_to(60);
			payout_collator_for_round(10);
			// new delegation is rewarded for first time
			// 2 rounds after joining (`RewardPaymentDelay` = 2)
			let mut new8 = vec![
				Event::CollatorChosen(13, 1, 50),
				Event::CollatorChosen(13, 2, 40),
				Event::CollatorChosen(13, 3, 20),
				Event::CollatorChosen(13, 4, 20),
				Event::CollatorChosen(13, 5, 10),
				Event::NewRound(60, 12, 5, 140),
				Event::Rewarded(10, 1, 157),
				Event::DelegatorDueReward(10, 1, 7, 48),
				Event::DelegatorDueReward(10, 1, 8, 48),
				Event::DelegatorDueReward(10, 1, 10, 48),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			expected.append(&mut new8);
			assert_eq_events!(expected);
		});
}

#[test]
fn delegations_merged_before_reward_payout() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 280, 0),
			(1, 20, 1),
			(2, 20, 1),
			(3, 20, 1),
			(4, 20, 1),
			(5, 120, 1),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(5, 1, 30), (5, 2, 30), (5, 3, 30), (5, 4, 30)])
		.build()
		.execute_with(|| {
			roll_to(8);
			set_author(1, 1, 1);
			set_author(1, 2, 1);
			set_author(1, 3, 1);
			set_author(1, 4, 1);

			roll_to(16);

			payout_collator_for_round(1);

			assert_eq!(StakeCurrency::free_balance(0u32, &1), 39);
			let expected_events = vec![
				Event::CollatorChosen(2, 1, 50),
				Event::CollatorChosen(2, 2, 50),
				Event::CollatorChosen(2, 3, 50),
				Event::CollatorChosen(2, 4, 50),
				Event::NewRound(5, 1, 4, 200),
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 50),
				Event::CollatorChosen(3, 3, 50),
				Event::CollatorChosen(3, 4, 50),
				Event::NewRound(10, 2, 4, 200),
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 50),
				Event::CollatorChosen(4, 3, 50),
				Event::CollatorChosen(4, 4, 50),
				Event::NewRound(15, 3, 4, 200),
				Event::Rewarded(1, 3, 39),
				Event::DelegatorDueReward(1, 3, 5, 36),
				Event::CollatorRewardsDistributed(3, PayoutRounds::All),
				Event::Rewarded(1, 1, 39),
				Event::DelegatorDueReward(1, 1, 5, 36),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
				Event::Rewarded(1, 4, 39),
				Event::DelegatorDueReward(1, 4, 5, 36),
				Event::CollatorRewardsDistributed(4, PayoutRounds::All),
				Event::Rewarded(1, 2, 39),
				Event::DelegatorDueReward(1, 2, 5, 36),
				Event::CollatorRewardsDistributed(2, PayoutRounds::All),
			];
			assert_eq_events!(expected_events);
		});
}

#[test]
// MaxDelegatorsPerCandidate = 4
fn bottom_delegations_are_empty_when_top_delegations_not_full() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 20), (2, 10), (3, 10), (4, 10), (5, 10)])
		.with_default_token_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// no top delegators => no bottom delegators
			let collator_state = Stake::candidate_state(1).unwrap();
			assert!(collator_state.top_delegations.is_empty());
			assert!(collator_state.bottom_delegations.is_empty());
			// 1 delegator => 1 top delegator, 0 bottom delegators
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, None, 10, 10));
			let collator_state = Stake::candidate_state(1).unwrap();
			assert_eq!(collator_state.top_delegations.len(), 1usize);
			assert!(collator_state.bottom_delegations.is_empty());
			// 2 delegators => 2 top delegators, 0 bottom delegators
			assert_ok!(Stake::delegate(Origin::signed(3), 1, 10, None, 10, 10));
			let collator_state = Stake::candidate_state(1).unwrap();
			assert_eq!(collator_state.top_delegations.len(), 2usize);
			assert!(collator_state.bottom_delegations.is_empty());
			// 3 delegators => 3 top delegators, 0 bottom delegators
			assert_ok!(Stake::delegate(Origin::signed(4), 1, 10, None, 10, 10));
			let collator_state = Stake::candidate_state(1).unwrap();
			assert_eq!(collator_state.top_delegations.len(), 3usize);
			assert!(collator_state.bottom_delegations.is_empty());
			// 4 delegators => 4 top delegators, 0 bottom delegators
			assert_ok!(Stake::delegate(Origin::signed(5), 1, 10, None, 10, 10));
			let collator_state = Stake::candidate_state(1).unwrap();
			assert_eq!(collator_state.top_delegations.len(), 4usize);
			assert!(collator_state.bottom_delegations.is_empty());
		});
}

#[test]
// MaxDelegatorsPerCandidate = 4
fn candidate_pool_updates_when_total_counted_changes() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 20),
			(3, 19),
			(4, 20),
			(5, 21),
			(6, 22),
			(7, 15),
			(8, 16),
			(9, 17),
			(10, 18),
		])
		.with_default_token_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(3, 1, 11),
			(4, 1, 12),
			(5, 1, 13),
			(6, 1, 14),
			(7, 1, 15),
			(8, 1, 16),
			(9, 1, 17),
			(10, 1, 18),
		])
		.build()
		.execute_with(|| {
			fn is_candidate_pool_bond(account: u64, bond: u128) {
				let pool = Stake::candidate_pool();
				for candidate in pool.0 {
					if candidate.owner == account {
						println!(
							"Stake::candidate_state(candidate.owner): {:?}",
							Stake::candidate_state(candidate.owner)
						);
						assert_eq!(candidate.amount, bond);
					}
				}
			}
			// 15 + 16 + 17 + 18 + 20 = 86 (top 4 + self bond)
			is_candidate_pool_bond(1, 86);
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(3), 1, 8, None));
			roll_to(10);
			// 3: 11 -> 19 => 3 is in top, bumps out 7
			assert_ok!(Stake::execute_delegation_request(Origin::signed(3), 3, 1, None));
			// 16 + 17 + 18 + 19 + 20 = 90 (top 4 + self bond)
			is_candidate_pool_bond(1, 90);
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(4), 1, 8, None));
			roll_to(20);
			// 4: 12 -> 20 => 4 is in top, bumps out 8
			assert_ok!(Stake::execute_delegation_request(Origin::signed(4), 4, 1, None));
			// 17 + 18 + 19 + 20 + 20 = 94 (top 4 + self bond)
			is_candidate_pool_bond(1, 94);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(10), 1, 3));
			roll_to(30);
			// 10: 18 -> 15 => 10 bumped to bottom, 8 bumped to top (- 18 + 16 = -2 for count)
			assert_ok!(Stake::execute_delegation_request(Origin::signed(10), 10, 1, None));
			// 16 + 17 + 19 + 20 + 20 = 92 (top 4 + self bond)
			is_candidate_pool_bond(1, 92);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(9), 1, 4));
			roll_to(40);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(9), 9, 1, None));
			// 15 + 16 + 19 + 20 + 20 = 90 (top 4 + self bond)
			is_candidate_pool_bond(1, 90);
		});
}

#[test]
// MaxDelegatorsPerCandidate = 4
fn only_top_collators_are_counted() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 20),
			(3, 19),
			(4, 20),
			(5, 21),
			(6, 22),
			(7, 15),
			(8, 16),
			(9, 17),
			(10, 18),
		])
		.with_default_token_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(3, 1, 11),
			(4, 1, 12),
			(5, 1, 13),
			(6, 1, 14),
			(7, 1, 15),
			(8, 1, 16),
			(9, 1, 17),
			(10, 1, 18),
		])
		.build()
		.execute_with(|| {
			// sanity check that 3-10 are delegators immediately
			for i in 3..11 {
				assert!(Stake::is_delegator(&i));
			}
			let collator_state = Stake::candidate_state(1).unwrap();
			// 15 + 16 + 17 + 18 + 20 = 86 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 86);
			// 11 + 12 + 13 + 14 = 50
			assert_eq!(collator_state.total_counted + 50, collator_state.total_backing);
			// bump bottom to the top
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(3), 1, 8, None));
			assert_event_emitted!(Event::DelegationIncreaseScheduled(3, 1, 8, 2));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(3), 3, 1, None));
			assert_event_emitted!(Event::DelegationIncreased(3, 1, 8, true));
			let collator_state = Stake::candidate_state(1).unwrap();
			// 16 + 17 + 18 + 19 + 20 = 90 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 90);
			// 12 + 13 + 14 + 15 = 54
			assert_eq!(collator_state.total_counted + 54, collator_state.total_backing);
			// bump bottom to the top
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(4), 1, 8, None));
			assert_event_emitted!(Event::DelegationIncreaseScheduled(4, 1, 8, 4));
			roll_to(20);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(4), 4, 1, None));
			assert_event_emitted!(Event::DelegationIncreased(4, 1, 8, true));
			let collator_state = Stake::candidate_state(1).unwrap();
			// 17 + 18 + 19 + 20 + 20 = 94 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 94);
			// 13 + 14 + 15 + 16 = 58
			assert_eq!(collator_state.total_counted + 58, collator_state.total_backing);
			// bump bottom to the top
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(5), 1, 8, None));
			assert_event_emitted!(Event::DelegationIncreaseScheduled(5, 1, 8, 6));
			roll_to(30);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(5), 5, 1, None));
			assert_event_emitted!(Event::DelegationIncreased(5, 1, 8, true));
			let collator_state = Stake::candidate_state(1).unwrap();
			// 18 + 19 + 20 + 21 + 20 = 98 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 98);
			// 14 + 15 + 16 + 17 = 62
			assert_eq!(collator_state.total_counted + 62, collator_state.total_backing);
			// bump bottom to the top
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(6), 1, 8, None));
			assert_event_emitted!(Event::DelegationIncreaseScheduled(6, 1, 8, 8));
			roll_to(40);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(6), 6, 1, None));
			assert_event_emitted!(Event::DelegationIncreased(6, 1, 8, true));
			let collator_state = Stake::candidate_state(1).unwrap();
			// 19 + 20 + 21 + 22 + 20 = 102 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 102);
			// 15 + 16 + 17 + 18 = 66
			assert_eq!(collator_state.total_counted + 66, collator_state.total_backing);
		});
}

#[test]
fn delegation_events_convey_correct_position() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20)])
		.with_delegations(vec![(3, 1, 11), (4, 1, 12), (5, 1, 13), (6, 1, 14)])
		.build()
		.execute_with(|| {
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 11 + 12 + 13 + 14 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 70);
			assert_eq!(collator1_state.total_counted, collator1_state.total_backing);
			// Top delegations are full, new highest delegation is made
			assert_ok!(Stake::delegate(Origin::signed(7), 1, 15, None, 10, 10));
			assert_event_emitted!(Event::Delegation(
				7,
				15,
				1,
				DelegatorAdded::AddedToTop { new_total: 74 },
			));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 12 + 13 + 14 + 15 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 74);
			// 11 = 11
			assert_eq!(collator1_state.total_counted + 11, collator1_state.total_backing);
			// New delegation is added to the bottom
			assert_ok!(Stake::delegate(Origin::signed(8), 1, 10, None, 10, 10));
			assert_event_emitted!(Event::Delegation(8, 10, 1, DelegatorAdded::AddedToBottom));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 12 + 13 + 14 + 15 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 74);
			// 10 + 11 = 21
			assert_eq!(collator1_state.total_counted + 21, collator1_state.total_backing);
			// 8 increases delegation to the top
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(8), 1, 3, None));
			assert_event_emitted!(Event::DelegationIncreaseScheduled(8, 1, 3, 2));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(8), 8, 1, None));
			assert_event_emitted!(Event::DelegationIncreased(8, 1, 3, true));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 13 + 13 + 14 + 15 + 20 = 75 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 75);
			// 11 + 12 = 23
			assert_eq!(collator1_state.total_counted + 23, collator1_state.total_backing);
			// 3 increases delegation but stays in bottom
			assert_ok!(Stake::schedule_delegator_bond_more(Origin::signed(3), 1, 1, None));
			assert_event_emitted!(Event::DelegationIncreaseScheduled(3, 1, 1, 4));
			roll_to(20);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(3), 3, 1, None));
			assert_event_emitted!(Event::DelegationIncreased(3, 1, 1, false));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 13 + 13 + 14 + 15 + 20 = 75 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 75);
			// 12 + 12 = 24
			assert_eq!(collator1_state.total_counted + 24, collator1_state.total_backing);
			// 6 decreases delegation but stays in top
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(6), 1, 2));
			assert_event_emitted!(Event::DelegationDecreaseScheduled(6, 1, 2, 6));
			roll_to(30);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(6), 6, 1, None));
			assert_event_emitted!(Event::DelegationDecreased(6, 1, 2, true));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 12 + 13 + 13 + 15 + 20 = 73 (top 4 + self bond)ƒ
			assert_eq!(collator1_state.total_counted, 73);
			// 12 + 12 = 24
			assert_eq!(collator1_state.total_counted + 24, collator1_state.total_backing);
			// 6 decreases delegation and is bumped to bottom
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(6), 1, 1));
			assert_event_emitted!(Event::DelegationDecreaseScheduled(6, 1, 1, 8));
			roll_to(40);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(6), 6, 1, None));
			assert_event_emitted!(Event::DelegationDecreased(6, 1, 1, false));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 12 + 13 + 13 + 15 + 20 = 73 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 73);
			// 11 + 12 = 23
			assert_eq!(collator1_state.total_counted + 23, collator1_state.total_backing);
		});
}

#[test]
fn start_and_new_session_works() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)])
		.with_default_token_candidates(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			let mut expected = vec![];
			assert_eq_events!(expected.clone());

			assert_eq!(Stake::at_stake(0, 1).bond, 20);
			assert_eq!(Stake::at_stake(0, 2).bond, 20);

			assert_eq!(Stake::at_stake(1, 1).bond, 20);
			assert_eq!(Stake::at_stake(1, 2).bond, 20);

			roll_to(5);

			assert_eq!(Stake::at_stake(2, 1).bond, 20);
			assert_eq!(Stake::at_stake(2, 2).bond, 20);

			assert_eq!(
				<Stake as pallet_session::SessionManager<_>>::new_session(Default::default()),
				Some(vec![1, 2])
			);

			let mut new = vec![
				Event::CollatorChosen(2, 1, 20),
				Event::CollatorChosen(2, 2, 20),
				Event::NewRound(5, 1, 2, 40),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());

			assert_ok!(Stake::join_candidates(
				Origin::signed(3),
				10u128,
				1u32,
				None,
				2u32,
				10000u32
			));

			roll_to(10);

			assert_eq!(Stake::at_stake(3, 1).bond, 20);
			assert_eq!(Stake::at_stake(3, 2).bond, 20);
			assert_eq!(Stake::at_stake(3, 3).bond, 10);

			assert_eq!(
				<Stake as pallet_session::SessionManager<_>>::new_session(Default::default()),
				Some(vec![1, 2, 3])
			);

			let mut new1 = vec![
				Event::JoinedCollatorCandidates(3, 10, 50),
				Event::CollatorChosen(3, 1, 20),
				Event::CollatorChosen(3, 2, 20),
				Event::CollatorChosen(3, 3, 10),
				Event::NewRound(10, 2, 3, 50),
			];
			expected.append(&mut new1);
			assert_eq_events!(expected.clone());

			assert_ok!(Stake::join_candidates(
				Origin::signed(4),
				10u128,
				1u32,
				None,
				3u32,
				10000u32
			));
			assert_ok!(Stake::join_candidates(
				Origin::signed(5),
				10u128,
				1u32,
				None,
				4u32,
				10000u32
			));

			roll_to(15);

			assert_eq!(Stake::at_stake(4, 1).bond, 20);
			assert_eq!(Stake::at_stake(4, 2).bond, 20);
			assert_eq!(Stake::at_stake(4, 3).bond, 10);
			assert_eq!(Stake::at_stake(4, 4).bond, 10);
			assert_eq!(Stake::at_stake(4, 5).bond, 10);

			assert_eq!(
				<Stake as pallet_session::SessionManager<_>>::new_session(Default::default()),
				Some(vec![1, 2, 3, 4, 5])
			);

			let mut new2 = vec![
				Event::JoinedCollatorCandidates(4, 10, 60),
				Event::JoinedCollatorCandidates(5, 10, 70),
				Event::CollatorChosen(4, 1, 20),
				Event::CollatorChosen(4, 2, 20),
				Event::CollatorChosen(4, 3, 10),
				Event::CollatorChosen(4, 4, 10),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 3, 5, 70),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
		});
}

#[test]
fn adding_removing_staking_token_works() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 100, 0),
			(1, 100, 1),
			(2, 100, 2),
			(3, 100, 3),
			(4, 100, 4),
			(5, 100, 5),
			(6, 100, 6),
			(7, 100, 7),
			(8, 100, 1),
			(9, 100, 2),
			(10, 100, 3),
		])
		.with_candidates(vec![(1, 20, 1), (2, 20, 2)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::staking_liquidity_tokens().get(&1), Some(&Some((1u128, 1u128))));
			assert_eq!(Stake::staking_liquidity_tokens().get(&2), Some(&Some((2u128, 1u128))));

			assert_eq!(Stake::staking_liquidity_tokens().get(&3), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&4), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&5), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&6), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&7), None);

			assert_ok!(Stake::join_candidates(
				Origin::signed(8),
				10u128,
				1u32,
				None,
				100u32,
				10000u32
			));
			assert_ok!(Stake::join_candidates(
				Origin::signed(9),
				10u128,
				2u32,
				None,
				100u32,
				10000u32
			));

			assert_noop!(
				Stake::join_candidates(Origin::signed(3), 10u128, 3u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(4), 10u128, 4u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(5), 10u128, 5u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(6), 10u128, 6u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(7), 10u128, 7u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);

			// Add 3 as a staking token
			assert_ok!(Stake::add_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(3u32),
				100u32
			));
			assert_ok!(Stake::join_candidates(
				Origin::signed(3),
				10u128,
				3u32,
				None,
				100u32,
				10000u32
			));
			assert_eq!(Stake::staking_liquidity_tokens().get(&3), Some(&None));
			// Check that the rest remain the same
			assert_eq!(Stake::staking_liquidity_tokens().get(&4), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&5), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&6), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&7), None);
			assert_noop!(
				Stake::join_candidates(Origin::signed(4), 10u128, 4u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(5), 10u128, 5u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(6), 10u128, 6u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(7), 10u128, 7u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);

			roll_to(5);

			// Check that 3 gets valuated and others don't
			assert_eq!(Stake::staking_liquidity_tokens().get(&3), Some(&Some((5u128, 1u128))));
			// Check that the rest remain the same
			assert_eq!(Stake::staking_liquidity_tokens().get(&4), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&5), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&6), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&7), None);
			assert_noop!(
				Stake::join_candidates(Origin::signed(4), 10u128, 4u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(5), 10u128, 5u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(6), 10u128, 6u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(7), 10u128, 7u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);

			// Adding same liquidity token doesn't work
			assert_noop!(
				Stake::add_staking_liquidity_token(
					Origin::root(),
					PairedOrLiquidityToken::Liquidity(3u32),
					100u32
				),
				Error::<Test>::StakingLiquidityTokenAlreadyListed
			);
			// Remove a liquidity not yet added - noop
			assert_noop!(
				Stake::remove_staking_liquidity_token(
					Origin::root(),
					PairedOrLiquidityToken::Liquidity(4u32),
					100u32
				),
				Error::<Test>::StakingLiquidityTokenNotListed
			);

			// Remove a liquidity token
			assert_ok!(Stake::remove_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(3u32),
				100u32
			));
			// Candidate cannot join using it.
			assert_noop!(
				Stake::join_candidates(Origin::signed(10), 10u128, 3u32, None, 100u32, 10000u32),
				Error::<Test>::StakingLiquidityTokenNotListed
			);

			roll_to(10);

			// Removed token is no longer valuated
			assert_eq!(Stake::staking_liquidity_tokens().get(&3), None);

			// Add more staking tokens
			assert_ok!(Stake::add_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(4u32),
				100u32
			));
			assert_ok!(Stake::add_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(5u32),
				100u32
			));
			assert_ok!(Stake::add_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(6u32),
				100u32
			));
			assert_ok!(Stake::add_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(7u32),
				100u32
			));

			// Candidates can join using the newly added tokens
			assert_ok!(Stake::join_candidates(
				Origin::signed(4),
				10u128,
				4u32,
				None,
				100u32,
				10000u32
			));
			assert_ok!(Stake::join_candidates(
				Origin::signed(6),
				10u128,
				6u32,
				None,
				100u32,
				10000u32
			));
			assert_ok!(Stake::join_candidates(
				Origin::signed(7),
				10u128,
				7u32,
				None,
				100u32,
				10000u32
			));

			roll_to(15);

			assert_eq!(Stake::staking_liquidity_tokens().get(&1), Some(&Some((1u128, 1u128))));
			assert_eq!(Stake::staking_liquidity_tokens().get(&2), Some(&Some((2u128, 1u128))));
			// No entry
			assert_eq!(Stake::staking_liquidity_tokens().get(&3), None);
			assert_eq!(Stake::staking_liquidity_tokens().get(&4), Some(&Some((1u128, 1u128))));
			// Valuated even though no candidates or delegates use it
			assert_eq!(Stake::staking_liquidity_tokens().get(&5), Some(&Some((1u128, 2u128))));
			assert_eq!(Stake::staking_liquidity_tokens().get(&6), Some(&Some((1u128, 5u128))));
			// Valuated as zero
			assert_eq!(Stake::staking_liquidity_tokens().get(&7), Some(&None));
		});
}

#[test]
fn delegation_tokens_work() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 100, 0),
			(1, 100, 1),
			(2, 100, 2),
			(3, 100, 3),
			(4, 100, 4),
			(5, 100, 5),
			(6, 100, 6),
			(7, 100, 7),
			(8, 100, 1),
			(8, 100, 2),
			(9, 100, 1),
			(9, 100, 2),
			(10, 100, 3),
			(11, 100, 7),
		])
		.with_candidates(vec![
			(1, 10, 1),
			(2, 10, 2),
			(3, 20, 3),
			(4, 20, 4),
			(5, 20, 5),
			(6, 20, 6),
			(7, 20, 7),
		])
		.with_delegations(vec![(8, 1, 5), (8, 2, 10), (9, 1, 5), (10, 3, 10), (11, 7, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::delegate(Origin::signed(9), 3, 10, None, 100u32, 100u32),
				DispatchError::Module(ModuleError {
					index: 1,
					error: [0; 4],
					message: Some("BalanceTooLow")
				})
			);
		});
}

#[test]
fn token_valuations_works() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 100, 0),
			(1, 100, 1),
			(2, 100, 2),
			(3, 100, 3),
			(4, 100, 4),
			(5, 100, 5),
			(6, 300, 6),
			(7, 100, 7),
			(8, 100, 1),
			(8, 100, 2),
			(9, 100, 1),
			(9, 100, 2),
			(10, 100, 3),
			(11, 100, 7),
		])
		.with_candidates(vec![
			(1, 10, 1),
			(2, 10, 2),
			(3, 10, 3),
			(4, 20, 4),
			(5, 20, 5),
			(6, 200, 6),
			(7, 10, 7),
		])
		.with_delegations(vec![(8, 1, 5), (8, 2, 10), (9, 1, 5), (10, 3, 10), (11, 7, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::set_total_selected(Origin::root(), 10));

			assert_eq!(
				Stake::candidate_pool().0,
				vec![
					Bond { owner: 1, amount: 20, liquidity_token: 1 },
					Bond { owner: 2, amount: 20, liquidity_token: 2 },
					Bond { owner: 3, amount: 20, liquidity_token: 3 },
					Bond { owner: 4, amount: 20, liquidity_token: 4 },
					Bond { owner: 5, amount: 20, liquidity_token: 5 },
					Bond { owner: 6, amount: 200, liquidity_token: 6 },
					Bond { owner: 7, amount: 20, liquidity_token: 7 }
				]
			);

			assert_eq!(Stake::at_stake(0, 1).bond, 10);
			assert_eq!(Stake::at_stake(0, 1).total, 20);
			assert_eq!(Stake::at_stake(0, 2).bond, 10);
			assert_eq!(Stake::at_stake(0, 2).total, 20);
			assert_eq!(Stake::at_stake(0, 3).bond, 10);
			assert_eq!(Stake::at_stake(0, 3).total, 20);
			assert_eq!(Stake::at_stake(0, 4).bond, 20);
			assert_eq!(Stake::at_stake(0, 4).total, 20);
			assert_eq!(Stake::at_stake(0, 5).bond, 0);
			assert_eq!(Stake::at_stake(0, 5).total, 0);
			assert_eq!(Stake::at_stake(0, 6).bond, 200);
			assert_eq!(Stake::at_stake(0, 6).total, 200);

			assert_eq!(Stake::at_stake(1, 1).bond, 10);
			assert_eq!(Stake::at_stake(1, 1).total, 20);
			assert_eq!(Stake::at_stake(1, 2).bond, 10);
			assert_eq!(Stake::at_stake(1, 2).total, 20);
			assert_eq!(Stake::at_stake(1, 3).bond, 10);
			assert_eq!(Stake::at_stake(1, 3).total, 20);
			assert_eq!(Stake::at_stake(1, 4).bond, 20);
			assert_eq!(Stake::at_stake(1, 4).total, 20);
			assert_eq!(Stake::at_stake(1, 5).bond, 0);
			assert_eq!(Stake::at_stake(1, 5).total, 0);
			assert_eq!(Stake::at_stake(1, 6).bond, 200);
			assert_eq!(Stake::at_stake(1, 6).total, 200);
			assert_eq!(Stake::at_stake(1, 7).bond, 0);
			assert_eq!(Stake::at_stake(1, 7).total, 0);

			roll_to(5);

			assert_eq!(Stake::at_stake(2, 1).bond, 10);
			assert_eq!(Stake::at_stake(2, 1).total, 20);
			assert_eq!(Stake::at_stake(2, 2).bond, 10);
			assert_eq!(Stake::at_stake(2, 2).total, 20);
			assert_eq!(Stake::at_stake(2, 3).bond, 10);
			assert_eq!(Stake::at_stake(2, 3).total, 20);
			assert_eq!(Stake::at_stake(2, 4).bond, 20);
			assert_eq!(Stake::at_stake(2, 4).total, 20);
			assert_eq!(Stake::at_stake(2, 5).bond, 20);
			assert_eq!(Stake::at_stake(2, 5).total, 20);
			assert_eq!(Stake::at_stake(2, 6).bond, 200);
			assert_eq!(Stake::at_stake(2, 6).total, 200);
			assert_eq!(Stake::at_stake(2, 7).bond, 0);
			assert_eq!(Stake::at_stake(2, 7).total, 0);

			let mut expected = vec![
				Event::TotalSelectedSet(5, 10),
				Event::CollatorChosen(2, 1, 20),
				Event::CollatorChosen(2, 2, 40),
				Event::CollatorChosen(2, 3, 100),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::CollatorChosen(2, 6, 40),
				Event::NewRound(5, 1, 6, 230),
			];
			assert_eq_events!(expected.clone());

			assert_ok!(Stake::remove_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(3u32),
				100u32
			));

			roll_to(10);

			assert_eq!(Stake::at_stake(3, 1).bond, 10);
			assert_eq!(Stake::at_stake(3, 1).total, 20);
			assert_eq!(Stake::at_stake(3, 2).bond, 10);
			assert_eq!(Stake::at_stake(3, 2).total, 20);
			assert_eq!(Stake::at_stake(3, 3).bond, 0);
			assert_eq!(Stake::at_stake(3, 3).total, 0);
			assert_eq!(Stake::at_stake(3, 4).bond, 20);
			assert_eq!(Stake::at_stake(3, 4).total, 20);
			assert_eq!(Stake::at_stake(3, 5).bond, 20);
			assert_eq!(Stake::at_stake(3, 5).total, 20);
			assert_eq!(Stake::at_stake(3, 6).bond, 200);
			assert_eq!(Stake::at_stake(3, 6).total, 200);
			assert_eq!(Stake::at_stake(3, 7).bond, 0);
			assert_eq!(Stake::at_stake(3, 7).total, 0);

			let mut new = vec![
				Event::CollatorChosen(3, 1, 20),
				Event::CollatorChosen(3, 2, 40),
				Event::CollatorChosen(3, 4, 20),
				Event::CollatorChosen(3, 5, 10),
				Event::CollatorChosen(3, 6, 40),
				Event::NewRound(10, 2, 5, 130),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
		});
}

#[test]
fn paired_or_liquidity_token_works() {
	ExtBuilder::default()
		.with_default_staking_token(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)])
		.with_default_token_candidates(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::add_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Paired(7000u32),
				100u32
			));
			assert_eq!(Stake::staking_liquidity_tokens().get(&70), Some(&None));

			assert_ok!(Stake::add_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(700u32),
				100u32
			));
			assert_eq!(Stake::staking_liquidity_tokens().get(&700), Some(&None));

			assert_ok!(Stake::remove_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(70u32),
				100u32
			),);
			assert_eq!(Stake::staking_liquidity_tokens().get(&70), None);

			assert_ok!(Stake::remove_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Paired(70000u32),
				100u32
			));
			assert_eq!(Stake::staking_liquidity_tokens().get(&700), None);
		});
}

// Agrregator must be selected instead of collators under that aggregator
// The aggregator must have total weight of all the collators under him
#[test]
fn token_valuations_works_with_aggregators() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 100, 0),
			(1, 100, 1),
			(2, 100, 2),
			(3, 100, 3),
			(4, 100, 4),
			(5, 100, 5),
			(6, 300, 6),
			(7, 100, 7),
			(8, 100, 1),
			(8, 100, 2),
			(9, 100, 1),
			(9, 100, 2),
			(10, 100, 3),
			(11, 100, 7),
		])
		.with_candidates(vec![(1, 20, 1), (2, 30, 2), (3, 10, 3)])
		.with_delegations(vec![(8, 1, 5), (8, 2, 10), (9, 1, 5), (10, 3, 10)])
		.build()
		.execute_with(|| {
			<TotalSelected<Test>>::put(1u32);
			assert_eq!(<TotalSelected<Test>>::get(), 1u32);

			assert_eq!(
				Stake::candidate_pool().0,
				vec![
					Bond { owner: 1, amount: 30, liquidity_token: 1 },
					Bond { owner: 2, amount: 40, liquidity_token: 2 },
					Bond { owner: 3, amount: 20, liquidity_token: 3 },
				]
			);

			assert_ok!(Stake::aggregator_update_metadata(
				Origin::signed(4),
				vec![1, 2],
				MetadataUpdateAction::ExtendApprovedCollators
			));
			assert_ok!(Stake::update_candidate_aggregator(Origin::signed(1), Some(4)));
			assert_ok!(Stake::update_candidate_aggregator(Origin::signed(2), Some(4)));

			roll_to(5);

			assert_eq!(Stake::at_stake(2, 1).bond, 20);
			assert_eq!(Stake::at_stake(2, 1).total, 30);
			assert_eq!(Stake::at_stake(2, 2).bond, 30);
			assert_eq!(Stake::at_stake(2, 2).total, 40);

			roll_to(10);

			assert_eq!(Stake::at_stake(3, 1).bond, 20);
			assert_eq!(Stake::at_stake(3, 1).total, 30);
			assert_eq!(Stake::at_stake(3, 2).bond, 30);
			assert_eq!(Stake::at_stake(3, 2).total, 40);

			let mut expected = vec![
				Event::AggregatorMetadataUpdated(4),
				Event::CandidateAggregatorUpdated(1, Some(4)),
				Event::CandidateAggregatorUpdated(2, Some(4)),
				Event::CollatorChosen(2, 1, 30),
				Event::CollatorChosen(2, 2, 80),
				Event::NewRound(5, 1, 2, 110),
				Event::CollatorChosen(3, 1, 30),
				Event::CollatorChosen(3, 2, 80),
				Event::NewRound(10, 2, 2, 110),
			];
			assert_eq_events!(expected.clone());

			assert_ok!(Stake::remove_staking_liquidity_token(
				Origin::root(),
				PairedOrLiquidityToken::Liquidity(2u32),
				100u32
			));

			roll_to(15);

			assert_eq!(Stake::at_stake(4, 3).bond, 10);
			assert_eq!(Stake::at_stake(4, 3).total, 20);

			let mut new = vec![Event::CollatorChosen(4, 3, 100), Event::NewRound(15, 3, 1, 100)];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
		});
}

// Agrregator must be selected instead of collators under that aggregator
// The aggregator must have total weight of all the collators under him
#[test]
fn round_aggregator_info_is_updated() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 100, 0),
			(1, 100, 1),
			(2, 100, 2),
			(3, 100, 3),
			(4, 100, 4),
			(5, 100, 5),
			(6, 300, 6),
			(7, 100, 7),
			(8, 100, 1),
			(8, 100, 2),
			(9, 100, 1),
			(9, 100, 2),
			(10, 100, 3),
			(11, 100, 7),
		])
		.with_candidates(vec![(1, 20, 1), (2, 30, 2), (3, 10, 3)])
		.with_delegations(vec![(8, 1, 5), (8, 2, 10), (9, 1, 5), (10, 3, 10)])
		.build()
		.execute_with(|| {
			<TotalSelected<Test>>::put(1u32);
			assert_eq!(<TotalSelected<Test>>::get(), 1u32);

			assert_eq!(
				Stake::candidate_pool().0,
				vec![
					Bond { owner: 1, amount: 30, liquidity_token: 1 },
					Bond { owner: 2, amount: 40, liquidity_token: 2 },
					Bond { owner: 3, amount: 20, liquidity_token: 3 },
				]
			);

			assert_ok!(Stake::aggregator_update_metadata(
				Origin::signed(4),
				vec![1, 2],
				MetadataUpdateAction::ExtendApprovedCollators
			));
			assert_ok!(Stake::update_candidate_aggregator(Origin::signed(1), Some(4)));
			assert_ok!(Stake::update_candidate_aggregator(Origin::signed(2), Some(4)));

			roll_to(5);

			assert_eq!(Stake::at_stake(2, 1).bond, 20);
			assert_eq!(Stake::at_stake(2, 1).total, 30);
			assert_eq!(Stake::at_stake(2, 2).bond, 30);
			assert_eq!(Stake::at_stake(2, 2).total, 40);

			let agggregator_collator_info: Vec<(
				<Test as frame_system::Config>::AccountId,
				Balance,
			)> = RoundAggregatorInfo::<Test>::get(2)
				.unwrap()
				.get(&4)
				.unwrap()
				.into_iter()
				.map(|(a, b)| (a.clone(), b.clone()))
				.collect::<_>();
			assert_eq!(agggregator_collator_info, [(1, 30), (2, 80)]);
		});
}

// Agrregator must be selected instead of collators under that aggregator
// The aggregator must have total weight of all the collators under him
#[test]
fn payouts_with_aggregators_work() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 100, 0),
			(1, 100, 1),
			(2, 100, 2),
			(3, 100, 3),
			(4, 100, 4),
			(5, 100, 5),
			(6, 300, 6),
			(7, 100, 7),
			(8, 100, 1),
			(8, 100, 2),
			(9, 100, 1),
			(9, 100, 2),
			(10, 100, 3),
			(11, 100, 7),
		])
		.with_candidates(vec![(1, 20, 1), (2, 30, 2), (3, 10, 3)])
		.with_delegations(vec![(8, 1, 5), (8, 2, 10), (9, 1, 5), (10, 3, 10)])
		.build()
		.execute_with(|| {
			<TotalSelected<Test>>::put(2u32);
			assert_eq!(<TotalSelected<Test>>::get(), 2u32);

			assert_eq!(
				Stake::candidate_pool().0,
				vec![
					Bond { owner: 1, amount: 30, liquidity_token: 1 },
					Bond { owner: 2, amount: 40, liquidity_token: 2 },
					Bond { owner: 3, amount: 20, liquidity_token: 3 },
				]
			);

			assert_ok!(Stake::aggregator_update_metadata(
				Origin::signed(4),
				vec![1, 2],
				MetadataUpdateAction::ExtendApprovedCollators
			));
			assert_ok!(Stake::update_candidate_aggregator(Origin::signed(1), Some(4)));
			assert_ok!(Stake::update_candidate_aggregator(Origin::signed(2), Some(4)));

			roll_to(5);

			assert_eq!(Stake::at_stake(2, 1).bond, 20);
			assert_eq!(Stake::at_stake(2, 1).total, 30);
			assert_eq!(Stake::at_stake(2, 2).bond, 30);
			assert_eq!(Stake::at_stake(2, 2).total, 40);

			let agggregator_collator_info: Vec<(
				<Test as frame_system::Config>::AccountId,
				Balance,
			)> = RoundAggregatorInfo::<Test>::get(2)
				.unwrap()
				.get(&4)
				.unwrap()
				.into_iter()
				.map(|(a, b)| (a.clone(), b.clone()))
				.collect::<_>();
			assert_eq!(agggregator_collator_info, [(1, 30), (2, 80)]);

			roll_to(10);

			set_author(2, 4, 100);
			set_author(2, 3, 100);

			roll_to(20);

			let collator_info_reward_info = RoundCollatorRewardInfo::<Test>::get(1, 2).unwrap();
			let collator_info_reward_info_delegators: Vec<(AccountId, Balance)> =
				collator_info_reward_info
					.delegator_rewards
					.into_iter()
					.map(|(a, b)| (a.clone(), b.clone()))
					.collect::<_>();
			assert_eq!(collator_info_reward_info.collator_reward, 29);
			assert_eq!(collator_info_reward_info_delegators, [(8, 5), (9, 5)]);

			let collator_info_reward_info = RoundCollatorRewardInfo::<Test>::get(2, 2).unwrap();
			let collator_info_reward_info_delegators: Vec<(AccountId, Balance)> =
				collator_info_reward_info
					.delegator_rewards
					.into_iter()
					.map(|(a, b)| (a.clone(), b.clone()))
					.collect::<_>();
			assert_eq!(collator_info_reward_info.collator_reward, 88);
			assert_eq!(collator_info_reward_info_delegators, [(8, 22)]);

			let collator_info_reward_info = RoundCollatorRewardInfo::<Test>::get(3, 2).unwrap();
			let collator_info_reward_info_delegators: Vec<(AccountId, Balance)> =
				collator_info_reward_info
					.delegator_rewards
					.into_iter()
					.map(|(a, b)| (a.clone(), b.clone()))
					.collect::<_>();
			assert_eq!(collator_info_reward_info.collator_reward, 91);
			assert_eq!(collator_info_reward_info_delegators, [(10, 61)]);
		});
}

#[test]
fn can_join_candidates_and_be_selected_with_native_token() {
	ExtBuilder::default()
		.with_default_staking_token(vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 33),
			(8, 33),
			(9, 33),
		])
		.with_default_token_candidates(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
		.build()
		.execute_with(|| {
			roll_to(8);

			let expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 1, 5, 400),
			];
			assert_eq_events!(expected);

			assert_ok!(Stake::join_candidates(
				Origin::signed(99999),
				1_000_000u128,
				0u32,
				None,
				6u32,
				10000u32
			));
			assert_last_event!(MetaEvent::Stake(Event::JoinedCollatorCandidates(
				99999,
				1_000_000u128,
				1_000_000u128,
			)));

			roll_to(9);

			let expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 1, 5, 400),
				Event::JoinedCollatorCandidates(99999, 1000000, 1000000),
				Event::CollatorChosen(3, 1, 100),
				Event::CollatorChosen(3, 2, 90),
				Event::CollatorChosen(3, 3, 80),
				Event::CollatorChosen(3, 4, 70),
				Event::CollatorChosen(3, 99999, 500000),
				Event::NewRound(10, 2, 5, 500340),
			];
			assert_eq_events!(expected);

			assert_eq!(StakeCurrency::reserved_balance(MGA_TOKEN_ID, &99999), 1_000_000u128);
		});
}

#[test]
fn test_claiming_rewards_for_more_periods_than_asked_due_to_optimization_based_on_delegators_count()
{
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 280, 0),
			(1, 20, 1),
			(2, 20, 1),
			(3, 20, 1),
			(4, 20, 1),
			(5, 30, 1),
			(6, 30, 1),
			(7, 30, 1),
			(8, 30, 1),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(5, 1, 30), (6, 1, 30), (7, 1, 30), (8, 1, 30)])
		.build()
		.execute_with(|| {
			set_author(1, 1, 1);
			set_author(1, 2, 1);
			set_author(1, 3, 1);
			set_author(1, 4, 1);
			roll_to(6);
			set_author(2, 1, 1);
			set_author(2, 2, 1);
			set_author(2, 3, 1);
			set_author(2, 4, 1);
			roll_to(21);

			assert_eq!(2, RoundCollatorRewardInfo::<Test>::iter_prefix(2).count());

			Stake::payout_collator_rewards(crate::mock::RuntimeOrigin::signed(999), 2, Some(1))
				.unwrap();

			let expected_events = vec![
				Event::CollatorChosen(2, 1, 140),
				Event::CollatorChosen(2, 2, 20),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::NewRound(5, 1, 4, 200),
				Event::CollatorChosen(3, 1, 140),
				Event::CollatorChosen(3, 2, 20),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 4, 20),
				Event::NewRound(10, 2, 4, 200),
				Event::CollatorChosen(4, 1, 140),
				Event::CollatorChosen(4, 2, 20),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 4, 20),
				Event::NewRound(15, 3, 4, 200),
				Event::CollatorChosen(5, 1, 140),
				Event::CollatorChosen(5, 2, 20),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 4, 20),
				Event::NewRound(20, 4, 4, 200),
				Event::Rewarded(1, 2, 76),
				Event::Rewarded(2, 2, 76),
				Event::CollatorRewardsDistributed(2, PayoutRounds::All),
			];
			assert_eq_events!(expected_events);
		});
}

#[test]
fn test_claiming_rewards_for_exactly_one_period_when_delegators_count_is_equal_to_max_available() {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 280, 0),
			(1, 20, 1),
			(2, 20, 1),
			(3, 20, 1),
			(4, 20, 1),
			(5, 30, 1),
			(6, 30, 1),
			(7, 30, 1),
			(8, 30, 1),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(5, 1, 30), (6, 1, 30), (7, 1, 30), (8, 1, 30)])
		.build()
		.execute_with(|| {
			set_author(1, 1, 1);
			set_author(1, 2, 1);
			set_author(1, 3, 1);
			set_author(1, 4, 1);
			roll_to(6);
			set_author(2, 1, 1);
			set_author(2, 2, 1);
			set_author(2, 3, 1);
			set_author(2, 4, 1);
			roll_to(21);

			assert_eq!(2, RoundCollatorRewardInfo::<Test>::iter_prefix(1).count());

			Stake::payout_collator_rewards(crate::mock::RuntimeOrigin::signed(999), 1, Some(1))
				.unwrap();

			let expected_events = vec![
				Event::CollatorChosen(2, 1, 140),
				Event::CollatorChosen(2, 2, 20),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::NewRound(5, 1, 4, 200),
				Event::CollatorChosen(3, 1, 140),
				Event::CollatorChosen(3, 2, 20),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 4, 20),
				Event::NewRound(10, 2, 4, 200),
				Event::CollatorChosen(4, 1, 140),
				Event::CollatorChosen(4, 2, 20),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 4, 20),
				Event::NewRound(15, 3, 4, 200),
				Event::CollatorChosen(5, 1, 140),
				Event::CollatorChosen(5, 2, 20),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 4, 20),
				Event::NewRound(20, 4, 4, 200),
				Event::Rewarded(1, 1, 23),
				Event::DelegatorDueReward(1, 1, 5, 13),
				Event::DelegatorDueReward(1, 1, 6, 13),
				Event::DelegatorDueReward(1, 1, 7, 13),
				Event::DelegatorDueReward(1, 1, 8, 13),
				Event::CollatorRewardsDistributed(1, PayoutRounds::Partial(vec![1])),
			];
			assert_eq_events!(expected_events);
		});
}

#[test]
fn test_claiming_rewards_for_all_periods_in_pesimistic_scenario_with_max_delegators_for_exactly_n_blocks(
) {
	ExtBuilder::default()
		.with_staking_tokens(vec![
			(999, 280, 0),
			(1, 20, 1),
			(2, 20, 1),
			(3, 20, 1),
			(4, 20, 1),
			(5, 30, 1),
			(6, 30, 1),
			(7, 30, 1),
			(8, 30, 1),
		])
		.with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(5, 1, 30), (6, 1, 30), (7, 1, 30), (8, 1, 30)])
		.build()
		.execute_with(|| {
			set_author(1, 1, 1);
			set_author(1, 2, 1);
			set_author(1, 3, 1);
			set_author(1, 4, 1);
			roll_to(6);
			set_author(2, 1, 1);
			set_author(2, 2, 1);
			set_author(2, 3, 1);
			set_author(2, 4, 1);
			roll_to(21);

			assert_eq!(2, RoundCollatorRewardInfo::<Test>::iter_prefix(1).count());

			Stake::payout_collator_rewards(crate::mock::RuntimeOrigin::signed(999), 1, Some(2))
				.unwrap();

			let expected_events = vec![
				Event::CollatorChosen(2, 1, 140),
				Event::CollatorChosen(2, 2, 20),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::NewRound(5, 1, 4, 200),
				Event::CollatorChosen(3, 1, 140),
				Event::CollatorChosen(3, 2, 20),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 4, 20),
				Event::NewRound(10, 2, 4, 200),
				Event::CollatorChosen(4, 1, 140),
				Event::CollatorChosen(4, 2, 20),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 4, 20),
				Event::NewRound(15, 3, 4, 200),
				Event::CollatorChosen(5, 1, 140),
				Event::CollatorChosen(5, 2, 20),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 4, 20),
				Event::NewRound(20, 4, 4, 200),
				Event::Rewarded(1, 1, 23),
				Event::DelegatorDueReward(1, 1, 5, 13),
				Event::DelegatorDueReward(1, 1, 6, 13),
				Event::DelegatorDueReward(1, 1, 7, 13),
				Event::DelegatorDueReward(1, 1, 8, 13),
				Event::Rewarded(2, 1, 23),
				Event::DelegatorDueReward(2, 1, 5, 13),
				Event::DelegatorDueReward(2, 1, 6, 13),
				Event::DelegatorDueReward(2, 1, 7, 13),
				Event::DelegatorDueReward(2, 1, 8, 13),
				Event::CollatorRewardsDistributed(1, PayoutRounds::All),
			];
			assert_eq_events!(expected_events);
		});
}

#[test]
fn test_triggre_error_when_there_are_no_rewards_to_payout() {
	ExtBuilder::default()
		.with_staking_tokens(vec![(999, 280, 0), (1, 20, 1)])
		// .with_default_token_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		// .with_delegations(vec![(5, 1, 30), (6,1,30), (7,1,30), (8,1,30),   ])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::payout_collator_rewards(crate::mock::RuntimeOrigin::signed(999), 33, None),
				Error::<Test>::CollatorRoundRewardsDNE
			);
		});
}
