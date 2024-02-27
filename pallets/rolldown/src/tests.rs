use core::future::pending;

use frame_support::assert_err;

use mockall::predicate::eq;
use sp_io::storage::rollback_transaction;
use sp_runtime::traits::ConvertBack;

use crate::{
	mock::{consts::*, *},
	*,
};
use hex_literal::hex;
use messages::{L1Update, L1UpdateRequest};
use serial_test::serial;

pub const ETH_TOKEN_ADDRESS: [u8; 20] = hex!("2CD2188119797153892438E57364D95B32975560");
pub const ETH_TOKEN_ADDRESS_MGX: TokenId = 100_u32;
pub const ETH_RECIPIENT_ACCOUNT: [u8; 20] = hex!("0000000000000000000000000000000000000004");
pub const ETH_RECIPIENT_ACCOUNT_MGX: AccountId = CHARLIE;

pub type TokensOf<Test> = <Test as crate::Config>::Tokens;

struct L1UpdateBuilder(messages::L1Update);
impl L1UpdateBuilder {
	fn new() -> Self {
		Self(Default::default())
	}

	fn with_offset(mut self, offset: u128) -> Self {
		self.0.offset = offset.into();
		self
	}

	fn with_last_accepted(mut self, last_accepted: u128) -> Self {
		self.0.lastAcceptedRequestOnL1 = last_accepted.into();
		self
	}

	fn with_last_processed(mut self, last_processed: u128) -> Self {
		self.0.lastProccessedRequestOnL1 = last_processed.into();
		self
	}

	fn with_requests(mut self, requests: Vec<L1UpdateRequest>) -> Self {
		self.0.lastAcceptedRequestOnL1 += requests.len().into();
		for r in requests {
			match r {
				L1UpdateRequest::Deposit(d) => {
					self.0.order.push(messages::PendingRequestType::DEPOSIT);
					self.0.pendingDeposits.push(d);
				},
				L1UpdateRequest::Withdraw(w) => {
					self.0.order.push(messages::PendingRequestType::WITHDRAWAL);
					self.0.pendingWithdraws.push(w);
				},
				L1UpdateRequest::Cancel(c) => {
					self.0.order.push(messages::PendingRequestType::CANCEL_RESOLUTION);
					self.0.pendingCancelResultions.push(c);
				},
				L1UpdateRequest::Remove(r) => {
					self.0.order.push(messages::PendingRequestType::L2_UPDATES_TO_REMOVE);
					self.0.pendingL2UpdatesToRemove.push(r);
				},
			}
		}
		self
	}

	fn build(self) -> messages::L1Update {
		self.0
	}
}

impl Default for L1UpdateBuilder {
	fn default() -> Self {
		Self(Default::default()).with_offset(1u128)
	}
}

#[test]
#[serial]
fn error_on_empty_update() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(36);
		let update = L1UpdateBuilder::default().build();
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
		let current_block_number =
			<frame_system::Pallet<Test>>::block_number().saturated_into::<u128>();
		let dispute_period: u128 = Rolldown::get_dispute_period();
		let update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
			.build();
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();

		assert_event_emitted!(Event::PendingRequestStored((
			ALICE,
			current_block_number + dispute_period,
			sp_core::U256::from(1u128),
			sp_core::U256::from(0u128),
			H256::from(hex!("8893a1a57e663ca4cb2609b62375b8c110907c847f93f317984eded8d0f35b56"))
		)));
	});
}

#[test]
#[serial]
fn create_pending_update_after_dispute_period() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update1 = L1UpdateBuilder::new()
			.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})])
			.with_offset(1u128)
			.build();

		let update2 = L1UpdateBuilder::new()
			.with_requests(vec![
				L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				}),
				L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				}),
			])
			.with_offset(2u128)
			.build();

		forward_to_block::<Test>(10);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update1).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update2).unwrap();

		assert_eq!(pending_updates::<Test>::iter().next(), None);
		assert!(pending_updates::<Test>::get(sp_core::U256::from(1u128)).is_none());
		assert!(pending_updates::<Test>::get(sp_core::U256::from(2u128)).is_none());

		forward_to_block::<Test>(15);
		assert!(pending_updates::<Test>::get(sp_core::U256::from(1u128)).is_some());

		forward_to_block::<Test>(16);
		assert!(pending_updates::<Test>::get(sp_core::U256::from(2u128)).is_some());
	});
}

#[test]
#[serial]
fn l2_counter_updates_when_requests_are_processed() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update1 = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})])
			.build();

		let update2 = L1UpdateBuilder::default()
			.with_requests(vec![
				L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				}),
				L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				}),
			])
			.build();

		forward_to_block::<Test>(10);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(), 0_u128);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update1).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update2).unwrap();

		forward_to_block::<Test>(15);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(), 1u128);

		forward_to_block::<Test>(16);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(), 2u128);
	});
}

#[test]
#[serial]
fn deposit_executed_after_dispute_period() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, 0u128)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			let update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();
			forward_to_block::<Test>(14);
			assert!(!pending_updates::<Test>::contains_key(sp_core::U256::from(0u128)));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);
			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(1u128)),
				Some(PendingUpdate::RequestResult((true, UpdateType::DEPOSIT)))
			);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);
		});
}

#[test]
#[serial]
fn l1_upate_executed_immaidately_if_force_submitted() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, 0u128)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			let update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);
			assert!(!pending_updates::<Test>::contains_key(sp_core::U256::from(1u128)));
			Rolldown::force_update_l2_from_l1(RuntimeOrigin::root(), update).unwrap();
			assert!(pending_updates::<Test>::contains_key(sp_core::U256::from(1u128)));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);
		});
}

#[test]
#[serial]
fn each_request_executed_only_once() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, 0u128)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			let update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update.clone()).unwrap();

			forward_to_block::<Test>(11);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update).unwrap();

			forward_to_block::<Test>(14);
			assert!(!pending_updates::<Test>::contains_key(sp_core::U256::from(0u128)));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);

			forward_to_block::<Test>(20);
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
			let update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();
			forward_to_block::<Test>(14);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);

			forward_to_block::<Test>(15);
			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(1u128)),
				Some(PendingUpdate::RequestResult((true, UpdateType::WITHDRAWAL)))
			);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);
		});
}

#[test]
#[serial]
fn withdraw_executed_after_dispute_period_when_not_enough_tokens() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);
		let update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})])
			.build();

		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();

		forward_to_block::<Test>(15);
		assert_eq!(
			pending_updates::<Test>::get(sp_core::U256::from(1u128)),
			Some(PendingUpdate::RequestResult((false, UpdateType::WITHDRAWAL)))
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

			let withdraw_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			let l2_updates_to_remove = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Remove(messages::L2UpdatesToRemove {
					l2UpdatesToRemove: vec![sp_core::U256::from(1u128)],
				})])
				.with_offset(2u128)
				.with_last_processed(1u128)
				.with_last_accepted(2u128)
				.build();

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();

			forward_to_block::<Test>(15);
			assert!(pending_updates::<Test>::contains_key(sp_core::U256::from(1u128)));

			forward_to_block::<Test>(100);
			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(1u128)),
				Some(PendingUpdate::RequestResult((true, UpdateType::WITHDRAWAL)))
			);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), l2_updates_to_remove)
				.unwrap();

			forward_to_block::<Test>(104);
			assert!(pending_updates::<Test>::contains_key(sp_core::U256::from(1u128)));
			assert!(!pending_updates::<Test>::contains_key(sp_core::U256::from(2u128)));

			forward_to_block::<Test>(105);
			assert!(pending_updates::<Test>::contains_key(sp_core::U256::from(2u128)));
			assert!(!pending_updates::<Test>::contains_key(sp_core::U256::from(1u128)));
		});
}

#[test]
#[serial]
fn test_cancel_removes_pending_update() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			// Arrange
			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			let withdraw_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			let cancel_resolution = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Cancel(messages::CancelResolution {
					l2RequestId: U256::from(1u128),
					cancelJustified: true,
				})])
				.with_offset(0u128)
				.build();

			assert!(!pending_requests::<Test>::contains_key(U256::from(15u128)));

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();
			assert!(pending_requests::<Test>::contains_key(U256::from(15u128)));
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();

			// Assert
			assert!(!pending_requests::<Test>::contains_key(U256::from(15u128)));
		});
}

#[test]
#[serial]
fn test_cancel_produce_update_with_correct_hash() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			// Arrange
			let withdraw_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();
			let req: messages::eth_abi::L1Update =
				pending_requests::<Test>::get(U256::from(15u128)).unwrap().1.into();

			assert_eq!(
				Rolldown::get_l2_update(),
				messages::eth_abi::L2Update { cancles: vec![], results: vec![] }
			);

			let update_id = Rolldown::get_l2_origin_updates_counter();
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();

			assert_eq!(
				Rolldown::get_l2_update(),
				messages::eth_abi::L2Update {
					cancles: vec![messages::eth_abi::Cancel {
						l2RequestId: messages::to_eth_u256(U256::from(update_id)),
						lastProccessedRequestOnL1: messages::to_eth_u256(U256::from(0u128)),
						lastAcceptedRequestOnL1: messages::to_eth_u256(U256::from(1u128)),
						hash: alloy_primitives::FixedBytes::<32>::from_slice(
							Keccak256::digest(&req.abi_encode()[..]).as_ref()
						),
					}],
					results: vec![]
				}
			);
		});
}

#[test]
#[serial]
fn test_malicious_sequencer_is_slashed_when_honest_sequencer_cancels_malicious_read() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			// Arrange

			let withdraw_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			let l2_request_id = Rolldown::get_l2_origin_updates_counter() + 1;
			let cancel_resolution = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Cancel(messages::CancelResolution {
					l2RequestId: U256::from(l2_request_id),
					cancelJustified: true,
				})])
				.with_offset(1u128)
				.build();

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();
			forward_to_block::<Test>(11);
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();
			forward_to_block::<Test>(12);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			forward_to_block::<Test>(16);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().with(eq(ALICE)).return_const(Ok(().into()));
			forward_to_block::<Test>(17);
		})
}

#[test]
#[serial]
fn test_malicious_canceler_is_slashed_when_honest_read_is_canceled() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			// Arrange

			let withdraw_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			let l2_request_id = Rolldown::get_l2_origin_updates_counter() + 1;
			let cancel_resolution = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Cancel(messages::CancelResolution {
					l2RequestId: U256::from(l2_request_id),
					cancelJustified: false,
				})])
				.with_offset(1u128)
				.build();

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();
			forward_to_block::<Test>(11);
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();
			forward_to_block::<Test>(12);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			forward_to_block::<Test>(16);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().with(eq(BOB)).return_const(Ok(().into()));
			forward_to_block::<Test>(17);
		})
}

#[test]
#[serial]
fn test_cancel_unexisting_request_fails() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			assert_err!(
				Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()),
				Error::<Test>::RequestDoesNotExist
			);
		});
}

#[test]
#[serial]
fn test_cancel_removes_cancel_right() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			let l2_request_id = Rolldown::get_l2_origin_updates_counter();
			let withdraw_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			let cancel_resolution = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Cancel(messages::CancelResolution {
					l2RequestId: U256::from(l2_request_id),
					cancelJustified: true,
				})])
				.with_offset(1u128)
				.build();

			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();

			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 1u128 }
			);
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);

			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();

			assert!(!pending_requests::<Test>::contains_key(U256::from(15u128)));
			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 1u128 }
			);
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 0u128 }
			);

			forward_to_block::<Test>(11);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 0u128 }
			);

			forward_to_block::<Test>(16);
			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);
		});
}

#[test]
#[serial]
// this test ensures that the hash calculated on rust side matches hash calculated in contract
fn test_l1_update_hash_compare_with_solidty() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update = L1UpdateBuilder::new()
			.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})])
			.with_offset(1u128)
			.with_last_accepted(0u128)
			.build();
		let hash = Rolldown::calculate_hash_of_pending_requests(update.clone());
		assert_eq!(
			hash,
			hex!("acf3b87e37038f4bc2dd017cb4818eef8c9da4cb36a23b8abcd6d3c17d69d65f").into()
		);
	});
}

#[test]
#[serial]
fn cancel_request_as_council_executed_immadiately() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			let withdraw_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})])
				.build();

			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update).unwrap();
			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 1u128 }
			);

			Rolldown::force_cancel_requests_from_l1(RuntimeOrigin::root(), 15u128.into()).unwrap();
			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);
		});
}

#[test]
#[serial]
fn reject_update_with_too_many_requests() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		let requests = vec![
			L1UpdateRequest::Withdraw(messages::Withdraw {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			});
			11
		];

		let withdraw_update = L1UpdateBuilder::default().with_requests(requests).build();

		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update),
			Error::<Test>::TooManyRequests
		);
	});
}

#[test]
#[serial]
fn check_request_ids_starts_from_one() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let requests = vec![L1UpdateRequest::Withdraw(messages::Withdraw {
			depositRecipient: ETH_RECIPIENT_ACCOUNT,
			tokenAddress: ETH_TOKEN_ADDRESS,
			amount: sp_core::U256::from(MILLION),
		})];

		assert_err!(
			Rolldown::update_l2_from_l1(
				RuntimeOrigin::signed(ALICE),
				L1UpdateBuilder::new().with_requests(requests.clone()).build()
			),
			Error::<Test>::WrongRequestId
		);

		assert_err!(
			Rolldown::update_l2_from_l1(
				RuntimeOrigin::signed(ALICE),
				L1UpdateBuilder::new().with_requests(requests).with_offset(2u128).build()
			),
			Error::<Test>::WrongRequestId
		);
	});
}

#[test]
#[serial]
fn reject_consecutive_update_with_invalid_counters() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		let requests = vec![L1UpdateRequest::Withdraw(messages::Withdraw {
			depositRecipient: ETH_RECIPIENT_ACCOUNT,
			tokenAddress: ETH_TOKEN_ADDRESS,
			amount: sp_core::U256::from(MILLION),
		})];

		let withdraw_update =
			L1UpdateBuilder::default().with_requests(requests).with_offset(100u128).build();

		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update),
			Error::<Test>::WrongRequestId
		);
	});
}

#[test]
#[serial]
fn reject_update_with_invalid_too_high_request_id() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		let withdraw_update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})])
			.with_offset(2u128)
			.build();

		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update),
			Error::<Test>::WrongRequestId
		);
	});
}

#[test]
#[serial]
fn reject_second_update_in_the_same_block() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);
		let withdraw_update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Withdraw(messages::Withdraw {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})])
			.build();

		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), withdraw_update.clone()).unwrap();

		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), withdraw_update).unwrap();
	});
}

#[test]
#[serial]
fn accept_consecutive_update_split_into_two() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		// imagine that there are 20 request on L1 waiting to be processed
		// they need to be split into 2 update_l2_from_l1 calls

		let dummy_update = L1UpdateRequest::Withdraw(messages::Withdraw {
			depositRecipient: ETH_RECIPIENT_ACCOUNT,
			tokenAddress: ETH_TOKEN_ADDRESS,
			amount: sp_core::U256::from(MILLION),
		});

		let first_update = L1UpdateBuilder::default()
			.with_requests(vec![
				dummy_update.clone();
				Rolldown::get_max_requests_per_block() as usize
			])
			.with_last_accepted(20)
			.with_last_processed(0)
			.with_offset(1u128)
			.build();

		let second_update = L1UpdateBuilder::default()
			.with_requests(vec![dummy_update; Rolldown::get_max_requests_per_block() as usize])
			.with_last_accepted(20)
			.with_last_processed(0)
			.with_offset(11u128)
			.build();
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();

		forward_to_block::<Test>(12);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), second_update).unwrap();

		forward_to_block::<Test>(17);

		let mut expected_updates = pending_updates::<Test>::iter_keys().collect::<Vec<_>>();
		expected_updates.sort();

		assert_eq!(
			(1u128..21u128)
				.collect::<Vec<_>>()
				.into_iter()
				.map(|id: u128| sp_core::U256::from(id))
				.collect::<Vec<sp_core::U256>>(),
			expected_updates
		);
	});
}

#[test]
#[serial]
fn reject_update_with_missing_requests() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		let update =
			L1Update { order: vec![messages::PendingRequestType::DEPOSIT], ..Default::default() };

		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update),
			Error::<Test>::EmptyUpdate
		);
	});
}

#[test]
fn test_conversion_u256() {
	let val = sp_core::U256::from(1u8);
	let eth_val = alloy_primitives::U256::from(1u8);
	assert_eq!(messages::to_eth_u256(val), eth_val);
}

#[test]
fn test_conversion_address() {
	let byte_address: [u8; 20] = DummyAddressConverter::convert_back(consts::CHARLIE);
	assert_eq!(DummyAddressConverter::convert(byte_address), consts::CHARLIE);
}
