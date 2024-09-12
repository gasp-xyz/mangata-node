// Copyright 2019-2022 PureStake Inc.
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

//! Unit testing

use crate::*;
use frame_support::traits::tokens::currency::MultiTokenCurrency;

use frame_support::{assert_err, assert_noop, assert_ok, traits::MultiTokenVestingSchedule};
use mock::*;
use parity_scale_codec::Encode;
use sp_application_crypto::RuntimePublic;
use sp_core::Pair;
use sp_runtime::{account::EthereumSignature, traits::Dispatchable, DispatchError, ModuleError};

type TokensOf<Test> = <Test as Config>::Tokens;

// Constant that reflects the desired vesting period for the tests
// Most tests complete initialization passing initRelayBlock + VESTING as the endRelayBlock
const VESTING: u64 = 8;

const ALICE: AccountId = 1;
const ALICE_NEW: AccountId = 11;
const BOB: AccountId = 2;
const BOB_NEW: AccountId = 12;

fn transferable_balance(who: &u64, token_id: TokenId) -> u128 {
	TokensOf::<Test>::free_balance(token_id, who) - Vesting::vesting_balance(who, token_id).unwrap()
}

#[test]
fn geneses() {
	empty().execute_with(|| {
		assert!(System::events().is_empty());
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));

		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		assert_eq!(Crowdloan::total_contributors(), 5);

		// accounts_payable
		assert!(Crowdloan::accounts_payable(0, 1).is_some());
		assert!(Crowdloan::accounts_payable(0, 2).is_some());
		assert!(Crowdloan::accounts_payable(0, 3).is_none());
		assert!(Crowdloan::accounts_payable(0, 4).is_none());
		assert!(Crowdloan::accounts_payable(0, 5).is_none());

		// claimed address existence
		assert!(Crowdloan::claimed_relay_chain_ids(0, [1u8; 20]).is_some());
		assert!(Crowdloan::claimed_relay_chain_ids(0, [2u8; 20]).is_some());
		assert!(Crowdloan::claimed_relay_chain_ids(
			0,
			<[u8; 20]>::from(AccountId20::from(pairs[0]))
		)
		.is_none());
		assert!(Crowdloan::claimed_relay_chain_ids(
			0,
			<[u8; 20]>::from(AccountId20::from(pairs[1]))
		)
		.is_none());
		assert!(Crowdloan::claimed_relay_chain_ids(
			0,
			<[u8; 20]>::from(AccountId20::from(pairs[2]))
		)
		.is_none());

		// unassociated_contributions
		assert!(Crowdloan::unassociated_contributions(0, [1u8; 20]).is_none());
		assert!(Crowdloan::unassociated_contributions(0, [2u8; 20]).is_none());
		assert!(Crowdloan::unassociated_contributions(
			0,
			<[u8; 20]>::from(AccountId20::from(pairs[0]))
		)
		.is_some());
		assert!(Crowdloan::unassociated_contributions(
			0,
			<[u8; 20]>::from(AccountId20::from(pairs[1]))
		)
		.is_some());
		assert!(Crowdloan::unassociated_contributions(
			0,
			<[u8; 20]>::from(AccountId20::from(pairs[2]))
		)
		.is_some());
	});
}

#[test]
fn proving_assignation_works() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(3);
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut TestSigantureNetworkIdentifier::get().to_vec());
		payload.append(&mut 3u64.encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());
		let signature: EthereumSignature =
			pairs[0].sign(sp_core::testing::ECDSA, &payload).unwrap().into();
		let alread_associated_signature: EthereumSignature =
			pairs[0].sign(sp_core::testing::ECDSA, &1u64.encode()).unwrap().into();
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			],
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		// 4 is not payable first
		assert!(Crowdloan::accounts_payable(0, 3).is_none());
		assert_eq!(
			Crowdloan::accounts_payable(0, 1).unwrap().contributed_relay_addresses,
			vec![[1u8; 20]]
		);

		roll_to(4);
		// Signature is wrong, prove fails
		assert_noop!(
			Crowdloan::associate_native_identity(
				RuntimeOrigin::root(),
				4,
				AccountId20::from(pairs[0]).into(),
				signature.clone()
			),
			Error::<Test>::InvalidClaimSignature
		);

		// Signature is right, but address already claimed
		assert_noop!(
			Crowdloan::associate_native_identity(
				RuntimeOrigin::root(),
				1,
				AccountId20::from(pairs[0]).into(),
				alread_associated_signature
			),
			Error::<Test>::AlreadyAssociated
		);

		// Signature is right, prove passes
		assert_ok!(Crowdloan::associate_native_identity(
			RuntimeOrigin::root(),
			3,
			AccountId20::from(pairs[0]).into(),
			signature.clone()
		));

		// Signature is right, but relay address is no longer on unassociated
		assert_noop!(
			Crowdloan::associate_native_identity(
				RuntimeOrigin::root(),
				3,
				AccountId20::from(pairs[0]).into(),
				signature
			),
			Error::<Test>::NoAssociatedClaim
		);

		// now three is payable
		assert!(Crowdloan::accounts_payable(0, 3).is_some());
		assert_eq!(
			Crowdloan::accounts_payable(0, 3).unwrap().contributed_relay_addresses,
			vec![<[u8; 20]>::from(AccountId20::from(pairs[0]))]
		);

		assert!(Crowdloan::unassociated_contributions(
			0,
			<[u8; 20]>::from(AccountId20::from(pairs[0]))
		)
		.is_none());
		assert!(Crowdloan::claimed_relay_chain_ids(
			0,
			<[u8; 20]>::from(AccountId20::from(pairs[0]))
		)
		.is_some());

		let expected = vec![crate::Event::NativeIdentityAssociated(
			AccountId20::from(pairs[0]).into(),
			3,
			500,
		)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn initializing_multi_relay_to_single_native_address_works() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(1), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		// 1 is payable
		assert!(Crowdloan::accounts_payable(0, 1).is_some());
		assert_eq!(
			Crowdloan::accounts_payable(0, 1).unwrap().contributed_relay_addresses,
			vec![[1u8; 20], [2u8; 20]]
		);

		roll_to(3);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(Crowdloan::accounts_payable(0, 1).unwrap().claimed_reward, 1000);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(3), None),
			Error::<Test>::NoAssociatedClaim
		);

		let expected = vec![crate::Event::RewardsPaid(1, 1000)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn paying_works_step_by_step() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		// 1 is payable
		assert!(Crowdloan::accounts_payable(0, 1).is_some());
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(Crowdloan::accounts_payable(0, 1).unwrap().claimed_reward, 500);
		assert_eq!(transferable_balance(&1, 0), 100);

		roll_to(3);

		assert_eq!(transferable_balance(&1, 0), 200);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(3), None),
			Error::<Test>::NoAssociatedClaim
		);
		roll_to(4);
		assert_eq!(transferable_balance(&1, 0), 250);
		roll_to(5);
		assert_eq!(transferable_balance(&1, 0), 300);
		roll_to(6);
		assert_eq!(transferable_balance(&1, 0), 350);
		roll_to(7);
		assert_eq!(transferable_balance(&1, 0), 400);
		roll_to(8);
		assert_eq!(transferable_balance(&1, 0), 450);
		roll_to(9);
		assert_eq!(transferable_balance(&1, 0), 500);
		roll_to(10);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(1), None),
			Error::<Test>::RewardsAlreadyClaimed
		);

		let expected = vec![crate::Event::RewardsPaid(1, 500)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn paying_works_after_unclaimed_period() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		// 1 is payable
		assert!(Crowdloan::accounts_payable(0, 1).is_some());
		roll_to(3);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(Crowdloan::accounts_payable(0, 1).unwrap().claimed_reward, 500);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(3), None),
			Error::<Test>::NoAssociatedClaim
		);
		roll_to(4);
		// assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(transferable_balance(&1, 0), 250);
		roll_to(5);
		// assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(transferable_balance(&1, 0), 300);
		roll_to(6);
		// assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(transferable_balance(&1, 0), 350);
		roll_to(10);
		// assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(transferable_balance(&1, 0), 500);
		roll_to(329);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(1), None),
			Error::<Test>::RewardsAlreadyClaimed
		);

		let expected = vec![crate::Event::RewardsPaid(1, 500)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn paying_late_joiner_works() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(3);
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut TestSigantureNetworkIdentifier::get().to_vec());
		payload.append(&mut 3u64.encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());
		let signature: EthereumSignature =
			pairs[0].sign(sp_core::testing::ECDSA, &payload).unwrap().into();
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		roll_to(12);
		assert_ok!(Crowdloan::associate_native_identity(
			RuntimeOrigin::root(),
			3,
			AccountId20::from(pairs[0]).into(),
			signature.clone()
		));
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(3), None));
		assert_eq!(Crowdloan::accounts_payable(0, 3).unwrap().claimed_reward, 500);
		let expected = vec![
			crate::Event::NativeIdentityAssociated(AccountId20::from(pairs[0]).into(), 3, 500),
			crate::Event::RewardsPaid(3, 500),
		];
		assert_eq!(events(), expected);
	});
}

#[test]
fn update_address_works() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		roll_to(3);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(8), None),
			Error::<Test>::NoAssociatedClaim
		);
		assert_ok!(Crowdloan::update_reward_address(RuntimeOrigin::signed(1), 8, None,));
		assert_eq!(transferable_balance(&1, 0), 200);
		roll_to(5);

		assert_eq!(transferable_balance(&1, 0), 300);
		// The initial payment is not
		let expected =
			vec![crate::Event::RewardsPaid(1, 500), crate::Event::RewardAddressUpdated(1, 8)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn update_address_with_existing_address_fails() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		// The init relay block gets inserted
		roll_to(2);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		roll_to(4);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(2), None));
		assert_noop!(
			Crowdloan::update_reward_address(RuntimeOrigin::signed(1), 2, None),
			Error::<Test>::AlreadyAssociated
		);
	});
}

#[test]
fn update_address_with_existing_with_multi_address_works() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(1), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		roll_to(3);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));

		// We make sure all rewards go to the new address
		assert_ok!(Crowdloan::update_reward_address(RuntimeOrigin::signed(1), 2, None));
		assert_eq!(Crowdloan::accounts_payable(0, 2).unwrap().claimed_reward, 1000);
		assert_eq!(Crowdloan::accounts_payable(0, 2).unwrap().total_reward, 1000);

		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(1), None),
			Error::<Test>::NoAssociatedClaim
		);
	});
}

#[test]
fn initialize_new_addresses() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		roll_to(2);
		// Insert contributors
		let pairs = get_ecdsa_pairs(3);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[2]).into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		assert!(Crowdloan::initialized());

		roll_to(4);
		assert_noop!(
			Crowdloan::initialize_reward_vec(
				RuntimeOrigin::root(),
				vec![([1u8; 20], Some(1), 500u32.into())]
			),
			Error::<Test>::RewardVecAlreadyInitialized,
		);

		assert_noop!(
			Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				init_block,
				init_block + VESTING * 2
			),
			Error::<Test>::RewardVecAlreadyInitialized,
		);
	});
}

#[test]
fn initialize_new_addresses_not_matching_funds() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		roll_to(2);
		// Insert contributors
		let pairs = get_ecdsa_pairs(2);
		let init_block = 1u64;
		// Total supply is 2500.Lets ensure inserting 2495 is not working.
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 20], Some(1), 500u32.into()),
				([2u8; 20], Some(2), 500u32.into()),
				(AccountId20::from(pairs[0]).into(), None, 500u32.into()),
				(AccountId20::from(pairs[1]).into(), None, 995u32.into()),
			]
		));
		assert_noop!(
			Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				init_block,
				init_block + VESTING
			),
			Error::<Test>::RewardsDoNotMatchFund
		);
	});
}

#[test]
fn initialize_new_addresses_with_batch() {
	empty().execute_with(|| {
		// This time should succeed trully
		roll_to(10);
		let init_block = 1u64;
		assert_ok!(mock::RuntimeCall::Utility(UtilityCall::batch_all {
			calls: vec![
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([4u8; 20], Some(3), 1250)],
				}),
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([5u8; 20], Some(1), 1250)],
				})
			]
		})
		.dispatch(RuntimeOrigin::root()));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		assert_eq!(Crowdloan::total_contributors(), 2);
		// Verify that the second ending block provider had no effect
		assert_eq!(Crowdloan::crowdloan_period(0), Some((init_block, init_block + VESTING)));

		// Batch calls always succeed. We just need to check the inner event
		assert_ok!(mock::RuntimeCall::Utility(UtilityCall::batch {
			calls: vec![mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
				rewards: vec![([4u8; 20], Some(3), 500)]
			})]
		})
		.dispatch(RuntimeOrigin::root()));

		let expected = vec![
			pallet_utility::Event::ItemCompleted,
			pallet_utility::Event::ItemCompleted,
			pallet_utility::Event::BatchCompleted,
			pallet_utility::Event::BatchInterrupted {
				index: 0,
				error: DispatchError::Module(ModuleError {
					index: 2,
					error: [8, 0, 0, 0],
					message: None,
				}),
			},
		];
		assert_eq!(batch_events(), expected);
	});
}

#[test]
fn floating_point_arithmetic_works() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(mock::RuntimeCall::Utility(UtilityCall::batch_all {
			calls: vec![
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([4u8; 20], Some(1), 1190)]
				}),
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([5u8; 20], Some(2), 1185)]
				}),
				// We will work with this. This has 100/8=12.5 payable per block
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([3u8; 20], Some(3), 125)]
				})
			]
		})
		.dispatch(RuntimeOrigin::root()));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		assert_eq!(Crowdloan::total_contributors(), 3);

		assert_eq!(Crowdloan::accounts_payable(0, 3).unwrap().claimed_reward, 0);

		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(3), None));

		let expected = vec![crate::Event::RewardsPaid(3, 125)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn test_initialization_errors() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		roll_to(2);
		let init_block = 1u64;

		let pot = Crowdloan::get_crowdloan_allocation(0);

		// Too many contributors
		assert_noop!(
			Crowdloan::initialize_reward_vec(
				RuntimeOrigin::root(),
				vec![
					([1u8; 20], Some(1), 1),
					([2u8; 20], Some(2), 1),
					([3u8; 20], Some(3), 1),
					([4u8; 20], Some(4), 1),
					([5u8; 20], Some(5), 1),
					([6u8; 20], Some(6), 1),
					([7u8; 20], Some(7), 1),
					([8u8; 20], Some(8), 1),
					([9u8; 20], Some(9), 1)
				]
			),
			Error::<Test>::TooManyContributors
		);

		// Go beyond fund pot
		assert_noop!(
			Crowdloan::initialize_reward_vec(
				RuntimeOrigin::root(),
				vec![([1u8; 20], Some(1), pot + 1)]
			),
			Error::<Test>::BatchBeyondFundPot
		);

		// Dont fill rewards
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![([1u8; 20], Some(1), pot - 1)]
		));

		// Fill rewards
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![([2u8; 20], Some(2), 1)]
		));

		// Insert a non-valid vesting period
		assert_noop!(
			Crowdloan::complete_initialization(RuntimeOrigin::root(), init_block, init_block),
			Error::<Test>::VestingPeriodNonValid
		);

		// Cannot claim if we dont complete initialization
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(1), None),
			Error::<Test>::RewardVecNotFullyInitializedYet
		);
		// Complete
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		// Cannot initialize again
		assert_noop!(
			Crowdloan::complete_initialization(RuntimeOrigin::root(), init_block, init_block),
			Error::<Test>::RewardVecAlreadyInitialized
		);
	});
}

#[test]
fn test_relay_signatures_can_change_reward_addresses() {
	empty().execute_with(|| {
		// 5 relay keys
		let pairs = get_ecdsa_pairs(5);

		// The init relay block gets inserted
		roll_to(2);
		let init_block = 1u64;

		// We will have all pointint to the same reward account
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				(AccountId20::from(pairs[0]).into(), Some(1), 500u32.into()),
				(AccountId20::from(pairs[1]).into(), Some(1), 500u32.into()),
				(AccountId20::from(pairs[2]).into(), Some(1), 500u32.into()),
				(AccountId20::from(pairs[3]).into(), Some(1), 500u32.into()),
				(AccountId20::from(pairs[4]).into(), Some(1), 500u32.into())
			],
		));

		// Complete
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		let reward_info = Crowdloan::accounts_payable(0, 1).unwrap();

		// We should have all of them as contributors
		for pair in pairs.clone() {
			assert!(reward_info
				.contributed_relay_addresses
				.contains(&AccountId20::from(pair).into()))
		}

		// Threshold is set to 50%, so we need at least 3 votes to pass
		// Let's make sure that we dont pass with 2
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut TestSigantureNetworkIdentifier::get().to_vec());
		payload.append(&mut 2u64.encode());
		payload.append(&mut 1u64.encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

		let mut insufficient_proofs: Vec<([u8; 20], EthereumSignature)> = vec![];
		for i in 0..2 {
			insufficient_proofs.push((
				AccountId20::from(pairs[i]).into(),
				pairs[i].sign(sp_core::testing::ECDSA, &payload).unwrap().into(),
			));
		}

		// Not sufficient proofs presented
		assert_noop!(
			Crowdloan::change_association_with_relay_keys(
				RuntimeOrigin::root(),
				2,
				1,
				insufficient_proofs.clone()
			),
			Error::<Test>::InsufficientNumberOfValidProofs
		);

		// With three votes we should passs
		let mut sufficient_proofs = insufficient_proofs.clone();

		// We push one more
		sufficient_proofs.push((
			AccountId20::from(pairs[2]).into(),
			pairs[2].sign(sp_core::testing::ECDSA, &payload).unwrap().into(),
		));

		// This time should pass
		assert_ok!(Crowdloan::change_association_with_relay_keys(
			RuntimeOrigin::root(),
			2,
			1,
			sufficient_proofs.clone()
		));

		// 1 should no longer be payable
		assert!(Crowdloan::accounts_payable(0, 1).is_none());

		// 2 should be now payable
		let reward_info_2 = Crowdloan::accounts_payable(0, 2).unwrap();

		// The reward info should be identical
		assert_eq!(reward_info, reward_info_2);
	});
}

#[test]
fn test_restart_crowdloan() {
	empty().execute_with(|| {
		// 5 relay keys
		let pairs = get_ecdsa_pairs(2);

		// The init relay block gets inserted
		roll_to(2);
		let init_block = 1u64;

		// We will have all pointint to the same reward account
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				(AccountId20::from(pairs[0]).into(), Some(1), 2000u32.into()),
				(AccountId20::from(pairs[1]).into(), Some(2), 500u32.into()),
			],
		));

		// Complete
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		assert!(Crowdloan::accounts_payable(0, 1).is_some());
		assert!(Crowdloan::accounts_payable(0, 2).is_some());

		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));

		assert_ok!(Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128));
		assert_eq!(Crowdloan::get_crowdloan_id(), 1);

		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(3), 2500u32.into()),],
		));

		assert_err!(
			Crowdloan::claim(RuntimeOrigin::signed(3), Some(1)),
			Error::<Test>::RewardVecNotFullyInitializedYet
		);

		assert_ok!(Crowdloan::complete_initialization(RuntimeOrigin::root(), 100, 200));

		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(3), Some(1)));
	});
}

#[test]
fn test_claim_rewards_from_consecutive_crowdloans_with_different_schedules() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(2);
		let first_crowdloan_period = (1u64, 9u64);
		let second_crowdloan_period = (101u64, 109u64);

		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128).unwrap();
		// We will have all pointint to the same reward account
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 2500u32.into()),],
		));

		// Complete
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));

		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(ALICE), None));

		// 20% of all rewards
		assert_eq!(transferable_balance(&ALICE, 0), 500);
		assert_eq!(Crowdloan::get_crowdloan_id(), 0);

		assert_ok!(Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128));
		assert_eq!(Crowdloan::get_crowdloan_id(), 1);

		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 2500u32.into()),],
		));

		assert_err!(
			Crowdloan::claim(RuntimeOrigin::signed(ALICE), Some(1)),
			Error::<Test>::RewardVecNotFullyInitializedYet
		);

		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			second_crowdloan_period.0,
			second_crowdloan_period.1
		));
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(ALICE), Some(1)));

		assert_eq!(transferable_balance(&ALICE, 0), 1000);

		roll_to(first_crowdloan_period.0 + 1);
		assert_eq!(transferable_balance(&ALICE, 0), 1000 + (2000 / 8));
		roll_to(first_crowdloan_period.0 + 2);
		assert_eq!(transferable_balance(&ALICE, 0), 1000 + (2000 / 8) * 2);
		roll_to(first_crowdloan_period.1 + 1);
		assert_eq!(transferable_balance(&ALICE, 0), 3000);

		roll_to(second_crowdloan_period.0 + 1);
		assert_eq!(transferable_balance(&ALICE, 0), 3000 + (2000 / 8));
		roll_to(second_crowdloan_period.0 + 2);
		assert_eq!(transferable_balance(&ALICE, 0), 3000 + (2000 / 8) * 2);
		roll_to(second_crowdloan_period.1 + 1);
		assert_eq!(transferable_balance(&ALICE, 0), 5000);
	});
}

#[test]
fn test_claim_rewards_from_consecutive_crowdloans_with_overlapping_schedules() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(2);
		let first_crowdloan_period = (1u64, 9u64);
		let second_crowdloan_period = (5u64, 13u64);

		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128).unwrap();
		// We will have all pointint to the same reward account
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 2500u32.into()),],
		));

		// Complete
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));

		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(ALICE), None));

		// 20% of all rewards
		assert_eq!(transferable_balance(&ALICE, 0), 500);
		assert_eq!(Crowdloan::get_crowdloan_id(), 0);

		assert_ok!(Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128));
		assert_eq!(Crowdloan::get_crowdloan_id(), 1);

		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 2500u32.into()),],
		));

		assert_err!(
			Crowdloan::claim(RuntimeOrigin::signed(ALICE), Some(1)),
			Error::<Test>::RewardVecNotFullyInitializedYet
		);
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			second_crowdloan_period.0,
			second_crowdloan_period.1
		));
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(ALICE), Some(1)));

		assert_eq!(transferable_balance(&ALICE, 0), 1000);

		roll_to(2);
		assert_eq!(transferable_balance(&ALICE, 0), 1000 + (2000 / 8));
		roll_to(3);
		assert_eq!(transferable_balance(&ALICE, 0), 1000 + (2000 / 8) * 2);
		roll_to(4);
		assert_eq!(transferable_balance(&ALICE, 0), 1000 + (2000 / 8) * 3);

		roll_to(6);
		assert_eq!(transferable_balance(&ALICE, 0), 1000 + (2000 / 8) * 5 + (2000 / 8));
		roll_to(7);
		assert_eq!(transferable_balance(&ALICE, 0), 1000 + (2000 / 8) * 6 + (2000 / 8) * 2);
		roll_to(second_crowdloan_period.1);
		assert_eq!(transferable_balance(&ALICE, 0), 5000);
	});
}

#[test]
fn change_crowdloan_allocation_before_finalization() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(2);
		let first_crowdloan_period = (1u64, 9u64);
		let _second_crowdloan_period = (5u64, 13u64);

		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128).unwrap();
		// We will have all pointint to the same reward account
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 500u32.into()),],
		));

		assert_err!(
			Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				first_crowdloan_period.0,
				first_crowdloan_period.1
			),
			Error::<Test>::RewardsDoNotMatchFund
		);

		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128).unwrap();

		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));
		assert!(Initialized::<Test>::get());
		assert_eq!(Crowdloan::get_crowdloan_id(), 0);

		// schedule following crowdloan
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 1000u128).unwrap();
		assert_eq!(Crowdloan::get_crowdloan_id(), 1);
		assert!(!Initialized::<Test>::get());
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 500u32.into()),],
		));
		assert!(!Initialized::<Test>::get());

		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128).unwrap();
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));

		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 1000u128).unwrap();
		assert_eq!(Crowdloan::get_crowdloan_id(), 2);
	});
}

#[test]
fn change_crowdloan_allocation_before_finalization_to_lower_value_than_initialized_so_far() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(2);
		let first_crowdloan_period = (1u64, 9u64);
		let _second_crowdloan_period = (5u64, 13u64);

		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128).unwrap();
		// We will have all pointint to the same reward account
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 500u32.into()),],
		));

		assert_err!(
			Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 499u128),
			Error::<Test>::AllocationDoesNotMatch
		);

		assert_ok!(Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128));

		assert_err!(
			Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				first_crowdloan_period.0,
				first_crowdloan_period.1
			),
			Error::<Test>::RewardsDoNotMatchFund
		);

		assert_ok!(Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128));

		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));
	});
}

#[test]
fn track_total_crowdloan_contributors_for_each_crowdloan_separately() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(2);
		let first_crowdloan_period = (1u64, 9u64);
		let _second_crowdloan_period = (5u64, 13u64);

		// FIRST CROWDLOAN
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128).unwrap();
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 500u32.into()),],
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));

		// SECOND CROWDLOAN
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128).unwrap();
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 500u32.into()),],
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));

		assert_eq!(Crowdloan::total_contributors_by_id(0), 1);
		assert_eq!(Crowdloan::total_contributors_by_id(1), 1);
	});
}

#[test]
fn update_rewards_address_for_past_crwdloans() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(2);
		let first_crowdloan_period = (1u64, 9u64);
		let _second_crowdloan_period = (5u64, 13u64);

		// FIRST CROWDLOAN
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128).unwrap();
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 500u32.into()),],
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));

		// SECOND CROWDLOAN
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128).unwrap();
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(BOB), 500u32.into()),],
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));

		assert!(Crowdloan::accounts_payable(0, ALICE).is_some(),);
		assert!(Crowdloan::accounts_payable(1, BOB).is_some(),);

		assert_err!(
			Crowdloan::update_reward_address(RuntimeOrigin::signed(ALICE), ALICE_NEW, Some(1),),
			Error::<Test>::NoAssociatedClaim
		);

		assert_err!(
			Crowdloan::update_reward_address(RuntimeOrigin::signed(BOB), BOB_NEW, Some(0),),
			Error::<Test>::NoAssociatedClaim
		);

		assert_ok!(Crowdloan::update_reward_address(
			RuntimeOrigin::signed(ALICE),
			ALICE_NEW,
			Some(0),
		));

		assert_ok!(Crowdloan::update_reward_address(RuntimeOrigin::signed(BOB), BOB_NEW, Some(1),));

		assert!(Crowdloan::accounts_payable(0, ALICE_NEW).is_some(),);
		assert!(Crowdloan::accounts_payable(1, BOB_NEW).is_some(),);
	});
}

#[test]
fn test_claim_previous_crowdloan_rewards_during_initialization_of_another_one() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(2);
		let first_crowdloan_period = (1u64, 9u64);
		let _second_crowdloan_period = (5u64, 13u64);

		// FIRST CROWDLOAN
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128).unwrap();
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(ALICE), 500u32.into()),],
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));

		// SECOND CROWDLOAN
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 500u128).unwrap();
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![(AccountId20::from(pairs[0]).into(), Some(BOB), 500u32.into()),],
		));

		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(ALICE), Some(0)));

		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			first_crowdloan_period.0,
			first_crowdloan_period.1
		));
	});
}

#[test]
fn reproduce_mgx654_bug_report() {
	empty().execute_with(|| {
		let pairs = get_ecdsa_pairs(3);
		const ALICE: u64 = 1u64;
		const BOB: u64 = 2u64;
		const CHARLIE: u64 = 3u64;
		const SINGLE_USER_REWARDS: u128 = 1_000_000u128;

		// FIRST CROWDLOAN
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 3 * SINGLE_USER_REWARDS)
			.unwrap();
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				(AccountId20::from(pairs[0]).into(), Some(ALICE), SINGLE_USER_REWARDS),
				(AccountId20::from(pairs[1]).into(), Some(BOB), SINGLE_USER_REWARDS),
				(AccountId20::from(pairs[2]).into(), Some(CHARLIE), SINGLE_USER_REWARDS),
			],
		));

		assert_ok!(Crowdloan::complete_initialization(RuntimeOrigin::root(), 10, 60,));

		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(ALICE, 0),
			orml_tokens::AccountData { free: 0u128, reserved: 0u128, frozen: 0u128 }
		);

		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(BOB, 0),
			orml_tokens::AccountData { free: 0u128, reserved: 0u128, frozen: 0u128 }
		);

		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(CHARLIE, 0),
			orml_tokens::AccountData { free: 0u128, reserved: 0u128, frozen: 0u128 }
		);

		roll_to(8);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(ALICE), Some(0)));
		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(ALICE, 0),
			orml_tokens::AccountData {
				free: SINGLE_USER_REWARDS,
				reserved: 0u128,
				frozen: SINGLE_USER_REWARDS
					- (<Test as Config>::InitializationPayment::get() * SINGLE_USER_REWARDS)
			}
		);

		roll_to(50);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(BOB), Some(0)));
		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(BOB, 0),
			orml_tokens::AccountData {
				free: SINGLE_USER_REWARDS,
				reserved: 0u128,
				frozen: (SINGLE_USER_REWARDS
					- (<Test as Config>::InitializationPayment::get() * SINGLE_USER_REWARDS))
					* 10 / 50
			}
		);

		roll_to(70);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(CHARLIE), Some(0)));
		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(CHARLIE, 0),
			orml_tokens::AccountData { free: SINGLE_USER_REWARDS, reserved: 0u128, frozen: 0u128 }
		);

		assert_ok!(pallet_vesting_mangata::Pallet::<Test>::vest(RuntimeOrigin::signed(ALICE), 0));
		assert_ok!(pallet_vesting_mangata::Pallet::<Test>::vest(RuntimeOrigin::signed(BOB), 0));
		assert_err!(
			pallet_vesting_mangata::Pallet::<Test>::vest(RuntimeOrigin::signed(CHARLIE), 0),
			pallet_vesting_mangata::Error::<Test>::NotVesting
		);

		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(ALICE, 0),
			orml_tokens::AccountData { free: SINGLE_USER_REWARDS, reserved: 0u128, frozen: 0u128 }
		);

		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(BOB, 0),
			orml_tokens::AccountData { free: SINGLE_USER_REWARDS, reserved: 0u128, frozen: 0u128 }
		);

		assert_eq!(
			orml_tokens::Pallet::<Test>::accounts(CHARLIE, 0),
			orml_tokens::AccountData { free: SINGLE_USER_REWARDS, reserved: 0u128, frozen: 0u128 }
		);
	});
}
