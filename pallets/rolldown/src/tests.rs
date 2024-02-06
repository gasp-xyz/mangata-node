use frame_support::{assert_err, assert_ok};
use sp_core::{crypto::Ss58Codec, hexdisplay::HexDisplay};

use crate::{
	mock::{consts::*, *},
	*,
};
use eth_api::{L1Update, L1UpdateRequest};
use hex_literal::hex;
use serial_test::serial;

pub const ETH_TOKEN_ADDRESS: [u8; 20] = hex!("2CD2188119797153892438E57364D95B32975560");
pub const ETH_TOKEN_ADDRESS_MGX: TokenId = 100_u32;
pub const ETH_RECIPIENT_ACCOUNT: [u8; 20] = hex!("0000000000000000000000000000000000000004");
pub const ETH_RECIPIENT_ACCOUNT_MGX: AccountId = CHARLIE;

pub type TokensOf<Test> = <Test as crate::Config>::Tokens;

fn create_l1_update(requests: Vec<L1UpdateRequest>) -> eth_api::L1Update {
	create_l1_update_with_offset(requests, sp_core::U256::from(0u128))
}

fn create_l1_update_with_offset(
	requests: Vec<L1UpdateRequest>,
	offset: sp_core::U256,
) -> eth_api::L1Update {
	let mut update = L1Update::default();
	update.offset = offset;
	for r in requests {
		match r {
			L1UpdateRequest::Deposit(d) => {
				update.order.push(eth_api::PendingRequestType::DEPOSIT);
				update.pendingDeposits.push(d);
			},
			L1UpdateRequest::Withdraw(w) => {
				update.order.push(eth_api::PendingRequestType::WITHDRAWAL);
				update.pendingWithdraws.push(w);
			},
			L1UpdateRequest::Cancel(c) => {
				update.order.push(eth_api::PendingRequestType::CANCEL_RESOLUTION);
				update.pendingCancelResultions.push(c);
			},
			L1UpdateRequest::Remove(r) => {
				update.order.push(eth_api::PendingRequestType::L2_UPDATES_TO_REMOVE);
				update.pendingL2UpdatesToRemove.push(r);
			},
		}
	}
	update
}

#[test]
#[serial]
fn error_on_empty_update() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(36);
		let update = create_l1_update(vec![]);
		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update),
			Error::<Test>::EmptyUpdate
		);
	});
}

#[test]
#[serial]
fn process_single_deposit() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(36);
		let update = create_l1_update(vec![L1UpdateRequest::Deposit(Default::default())]);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();

		assert_event_emitted!(Event::PendingRequestStored((
			ALICE,
			H256::from(hex!("2a48bbdcd86e4e5571feef2579e2c4098c95b5aecc82a603c873429bf72651c3"))
		)));
	});
}

#[test]
#[serial]
fn create_pending_update_after_dispute_period() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update1 = create_l1_update(vec![L1UpdateRequest::Deposit(eth_api::Deposit {
			depositRecipient: ETH_RECIPIENT_ACCOUNT,
			tokenAddress: ETH_TOKEN_ADDRESS,
			amount: sp_core::U256::from(MILLION),
		})]);

		let update2 = create_l1_update_with_offset(
			vec![L1UpdateRequest::Withdraw(eth_api::Withdraw {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})],
			sp_core::U256::from(1u128),
		);

		forward_to_block::<Test>(10);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update1).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update2).unwrap();

		assert_eq!(PENDING_UPDATES_MAT::<Test>::iter().next(), None);
		assert!(PENDING_UPDATES_MAT::<Test>::get(sp_core::U256::from(0u128)).is_none());
		assert!(PENDING_UPDATES_MAT::<Test>::get(sp_core::U256::from(1u128)).is_none());

		forward_to_block::<Test>(15);
		assert!(PENDING_UPDATES_MAT::<Test>::get(sp_core::U256::from(0u128)).is_some());

		forward_to_block::<Test>(16);
		assert!(PENDING_UPDATES_MAT::<Test>::get(sp_core::U256::from(1u128)).is_some());
	});
}

#[test]
#[serial]
fn l2_counter_updates_when_requests_are_processed() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update1 = create_l1_update(vec![L1UpdateRequest::Deposit(eth_api::Deposit {
			depositRecipient: ETH_RECIPIENT_ACCOUNT,
			tokenAddress: ETH_TOKEN_ADDRESS,
			amount: sp_core::U256::from(MILLION),
		})]);

		let update2 = create_l1_update_with_offset(
			vec![L1UpdateRequest::Withdraw(eth_api::Withdraw {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})],
			sp_core::U256::from(1u128),
		);

		forward_to_block::<Test>(10);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(), 0_u128);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update1).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update2).unwrap();

		forward_to_block::<Test>(15);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(), 0_u128);

		forward_to_block::<Test>(16);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(), 1_u128);
	});
}

#[test]
#[serial]
fn deposit_executed_after_dispute_period() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, 0u128)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			let update = create_l1_update(vec![L1UpdateRequest::Deposit(eth_api::Deposit {
				depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})]);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();
			forward_to_block::<Test>(14);
			assert!(!PENDING_UPDATES_MAT::<Test>::contains_key(sp_core::U256::from(0u128)));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);
			assert_eq!(
				PENDING_UPDATES_MAT::<Test>::get(sp_core::U256::from(0u128)),
				Some(PendingUpdate::Status(true))
			);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);
		});
}

#[test]
#[serial]
fn withdraw_executed_after_dispute_period() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			let update = create_l1_update(vec![L1UpdateRequest::Withdraw(eth_api::Withdraw {
				depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})]);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();
			forward_to_block::<Test>(14);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);

			forward_to_block::<Test>(15);
			assert_eq!(
				PENDING_UPDATES_MAT::<Test>::get(sp_core::U256::from(0u128)),
				Some(PendingUpdate::Status(true))
			);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);
		});
}

#[test]
#[serial]
fn withdraw_executed_after_dispute_period_when_not_enough_tokens() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);
		let update = create_l1_update(vec![L1UpdateRequest::Withdraw(eth_api::Withdraw {
			depositRecipient: ETH_RECIPIENT_ACCOUNT,
			tokenAddress: ETH_TOKEN_ADDRESS,
			amount: sp_core::U256::from(MILLION),
		})]);

		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();

		forward_to_block::<Test>(15);
		assert_eq!(
			PENDING_UPDATES_MAT::<Test>::get(sp_core::U256::from(0u128)),
			Some(PendingUpdate::Status(false))
		);
	});
}

#[test]
#[serial]
fn updates_to_remove_executed_after_dispute_period() {
	ExtBuilder::new()
		.issue(CHARLIE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			let withdraw_update =
				create_l1_update(vec![L1UpdateRequest::Withdraw(eth_api::Withdraw {
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);
			let l2_updates_to_remove = create_l1_update_with_offset(
				vec![L1UpdateRequest::Remove(eth_api::L2UpdatesToRemove {
					l2UpdatesToRemove: vec![sp_core::U256::from(0u128)],
				})],
				sp_core::U256::from(1u128),
			);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();

			forward_to_block::<Test>(15);
			assert!(PENDING_UPDATES_MAT::<Test>::contains_key(sp_core::U256::from(0u128)));

			forward_to_block::<Test>(100);
			assert_eq!(
				PENDING_UPDATES_MAT::<Test>::get(sp_core::U256::from(0u128)),
				Some(PendingUpdate::Status(true))
			);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), l2_updates_to_remove)
				.unwrap();

			forward_to_block::<Test>(104);
			assert!(PENDING_UPDATES_MAT::<Test>::contains_key(sp_core::U256::from(0u128)));
			assert!(!PENDING_UPDATES_MAT::<Test>::contains_key(sp_core::U256::from(1u128)));

			forward_to_block::<Test>(105);
			assert!(PENDING_UPDATES_MAT::<Test>::contains_key(sp_core::U256::from(1u128)));
			assert!(!PENDING_UPDATES_MAT::<Test>::contains_key(sp_core::U256::from(0u128)));
		});
}

#[test]
#[serial]
fn cancel_request() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			let withdraw_update =
				create_l1_update(vec![L1UpdateRequest::Withdraw(eth_api::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);

			let cancel_resolution = create_l1_update_with_offset(
				vec![L1UpdateRequest::Cancel(eth_api::CancelResolution {
					l2RequestId: U256::from(1u128),
					cancelJustified: true,
				})],
				sp_core::U256::from(0u128),
			);

			assert_eq!(
				SEQUENCER_RIGHTS::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);
			assert_eq!(
				SEQUENCER_RIGHTS::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();
			assert!(PENDING_REQUESTS_MAT::<Test>::contains_key(U256::from(15u128)));

			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();

			assert!(!PENDING_REQUESTS_MAT::<Test>::contains_key(U256::from(15u128)));
			assert_eq!(
				SEQUENCER_RIGHTS::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 1u128 }
			);
			assert_eq!(
				SEQUENCER_RIGHTS::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 0u128 }
			);

			forward_to_block::<Test>(11);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			assert_eq!(
				SEQUENCER_RIGHTS::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 0u128 }
			);

			forward_to_block::<Test>(16);
			assert_eq!(
				SEQUENCER_RIGHTS::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);
			assert_eq!(
				SEQUENCER_RIGHTS::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);
		});
}

#[test]
fn test_conversion_u256() {
	let val = sp_core::U256::from(1u8);
	let eth_val = alloy_primitives::U256::from(1u8);

	assert_eq!(Rolldown::to_eth_u256(val), eth_val);
}

#[test]
fn test_conversion_address() {
	let byte_address: [u8; 20] = DummyAddressConverter::convert_back(consts::CHARLIE);

	assert_eq!(DummyAddressConverter::convert(byte_address), consts::CHARLIE);
}
