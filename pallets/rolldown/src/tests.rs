use crate::{
	mock::{consts::*, *},
	*,
};
use frame_support::assert_err;
use hex_literal::hex;
use messages::{L1Update, L1UpdateRequest};
use mockall::predicate::eq;
use serial_test::serial;
use sp_io::storage::rollback_transaction;
use sp_runtime::traits::ConvertBack;

pub const ETH_TOKEN_ADDRESS: [u8; 20] = hex!("2CD2188119797153892438E57364D95B32975560");
pub const ETH_TOKEN_ADDRESS_MGX: TokenId = 100_u32;
pub const ETH_RECIPIENT_ACCOUNT: [u8; 20] = hex!("0000000000000000000000000000000000000004");
pub const ETH_RECIPIENT_ACCOUNT_MGX: AccountId = CHARLIE;

pub type TokensOf<Test> = <Test as crate::Config>::Tokens;

fn create_l1_update(requests: Vec<L1UpdateRequest>) -> messages::L1Update {
	create_l1_update_with_offset(requests, sp_core::U256::from(0u128))
}

fn create_l1_update_with_offset(
	requests: Vec<L1UpdateRequest>,
	offset: sp_core::U256,
) -> messages::L1Update {
	let mut update = L1Update::default();
	update.offset = offset;
	for r in requests {
		match r {
			L1UpdateRequest::Deposit(d) => {
				update.order.push(messages::PendingRequestType::DEPOSIT);
				update.pendingDeposits.push(d);
			},
			L1UpdateRequest::Cancel(c) => {
				update.order.push(messages::PendingRequestType::CANCEL_RESOLUTION);
				update.pendingCancelResultions.push(c);
			},
			L1UpdateRequest::Remove(r) => {
				update.order.push(messages::PendingRequestType::L2_UPDATES_TO_REMOVE);
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
			H256::from(hex!("55d453b5212ca1e569708afb43c69a2196a96cecd31d0bf850cc4ce0c77929a8"))
		)));
	});
}

#[test]
#[serial]
fn create_pending_update_after_dispute_period() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update1 = create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
			depositRecipient: ETH_RECIPIENT_ACCOUNT,
			tokenAddress: ETH_TOKEN_ADDRESS,
			amount: sp_core::U256::from(MILLION),
		})]);

		let update2 = create_l1_update_with_offset(
			vec![L1UpdateRequest::Deposit(messages::Deposit {
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

		assert_eq!(pending_updates::<Test>::iter().next(), None);
		assert!(pending_updates::<Test>::get(sp_core::U256::from(0u128)).is_none());
		assert!(pending_updates::<Test>::get(sp_core::U256::from(1u128)).is_none());

		forward_to_block::<Test>(15);
		assert!(pending_updates::<Test>::get(sp_core::U256::from(0u128)).is_some());

		forward_to_block::<Test>(16);
		assert!(pending_updates::<Test>::get(sp_core::U256::from(1u128)).is_some());
	});
}

#[test]
#[serial]
fn l2_counter_updates_when_requests_are_processed() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update1 = create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
			depositRecipient: ETH_RECIPIENT_ACCOUNT,
			tokenAddress: ETH_TOKEN_ADDRESS,
			amount: sp_core::U256::from(MILLION),
		})]);

		let update2 = create_l1_update_with_offset(
			vec![L1UpdateRequest::Deposit(messages::Deposit {
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
			let update = create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
				depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})]);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();
			forward_to_block::<Test>(14);
			assert!(!pending_updates::<Test>::contains_key(sp_core::U256::from(0u128)));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);
			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(0u128)),
				Some(PendingUpdate::RequestResult((true, UpdateType::DEPOSIT)))
			);
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
			let update = create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
				depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})]);
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
fn updates_to_remove_executed_after_dispute_period() {
	ExtBuilder::new()
		.issue(CHARLIE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			let deposit_update =
				create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);

			let l2_updates_to_remove = create_l1_update_with_offset(
				vec![L1UpdateRequest::Remove(messages::L2UpdatesToRemove {
					l2UpdatesToRemove: vec![sp_core::U256::from(0u128)],
				})],
				sp_core::U256::from(1u128),
			);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();

			forward_to_block::<Test>(15);
			assert!(pending_updates::<Test>::contains_key(sp_core::U256::from(0u128)));

			forward_to_block::<Test>(100);
			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(0u128)),
				Some(PendingUpdate::RequestResult((true, UpdateType::DEPOSIT)))
			);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), l2_updates_to_remove)
				.unwrap();

			forward_to_block::<Test>(104);
			assert!(pending_updates::<Test>::contains_key(sp_core::U256::from(0u128)));
			assert!(!pending_updates::<Test>::contains_key(sp_core::U256::from(1u128)));

			forward_to_block::<Test>(105);
			assert!(pending_updates::<Test>::contains_key(sp_core::U256::from(1u128)));
			assert!(!pending_updates::<Test>::contains_key(sp_core::U256::from(0u128)));
		});
}

#[test]
#[serial]
fn test_cancel_removes_pending_requests() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			// Arrange
			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			let deposit_update =
				create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);

			assert!(!pending_requests::<Test>::contains_key(U256::from(15u128)));

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
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
			let deposit_update =
				create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			let req: messages::eth_abi::L1Update =
				pending_requests::<Test>::get(U256::from(15u128)).unwrap().1.into();

			assert_eq!(
				Rolldown::get_l2_update(),
				messages::eth_abi::L2Update {
					withdrawals: vec![],
					cancels: vec![],
					results: vec![]
				}
			);

			let update_id = Rolldown::get_l2_origin_updates_counter();
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();

			assert_eq!(
				Rolldown::get_l2_update(),
				messages::eth_abi::L2Update {
					cancels: vec![messages::eth_abi::Cancel {
						l2RequestId: messages::to_eth_u256(U256::from(update_id)),
						lastProccessedRequestOnL1: messages::to_eth_u256(U256::from(0u128)),
						lastAcceptedRequestOnL1: messages::to_eth_u256(U256::from(0u128)),
						hash: alloy_primitives::FixedBytes::<32>::from_slice(
							Keccak256::digest(&req.abi_encode()[..]).as_ref()
						),
					}],
					withdrawals: vec![],
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

			let deposit_update =
				create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);

			let l2_request_id = Rolldown::get_l2_origin_updates_counter() + 1;
			let cancel_resolution = create_l1_update_with_offset(
				vec![L1UpdateRequest::Cancel(messages::CancelResolution {
					l2RequestId: U256::from(l2_request_id),
					cancelJustified: true,
				})],
				sp_core::U256::from(1u128),
			);

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
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

			let deposit_update =
				create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);

			let l2_request_id = Rolldown::get_l2_origin_updates_counter() + 1;
			let cancel_resolution = create_l1_update_with_offset(
				vec![L1UpdateRequest::Cancel(messages::CancelResolution {
					l2RequestId: U256::from(l2_request_id),
					cancelJustified: false,
				})],
				sp_core::U256::from(1u128),
			);

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
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
			let deposit_update =
				create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);

			let cancel_resolution = create_l1_update_with_offset(
				vec![L1UpdateRequest::Cancel(messages::CancelResolution {
					l2RequestId: U256::from(l2_request_id),
					cancelJustified: true,
				})],
				sp_core::U256::from(1u128),
			);

			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 2u128 }
			);
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 2u128 }
			);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();

			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 2u128 }
			);
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 2u128 }
			);

			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();

			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 2u128 }
			);
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 1u128 }
			);

			forward_to_block::<Test>(11);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 1u128 }
			);

			forward_to_block::<Test>(16);
			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 2u128 }
			);
			assert_eq!(
				sequencer_rights::<Test>::get(BOB).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 2u128 }
			);
		});
}

#[test]
#[serial]
// this test ensures that the hash calculated on rust side matches hash calculated in contract
fn test_l1_update_hash_compare_with_solidty() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update = create_l1_update_with_offset(
			vec![L1UpdateRequest::Deposit(messages::Deposit {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			})],
			sp_core::U256::from(1u128),
		);
		let hash = Rolldown::calculate_hash_of_pending_requests(update.clone());
		assert_eq!(
			hash,
			hex!("033f8b8c13f3dd23df7d50f5a5106621177bb75f71cb39e9a8fd4ab565bec339").into()
		);
	});
}

#[test]
#[serial]
fn reject_update_with_too_many_requests() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		let requests = vec![
			L1UpdateRequest::Deposit(messages::Deposit {
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
			});
			11
		];

		let deposit_update = create_l1_update(requests);

		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update),
			Error::<Test>::TooManyRequests
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
			Error::<Test>::InvalidUpdate
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

#[test]
#[serial]
fn test_withdraw() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			assert_eq!(TokensOf::<Test>::total_issuance(ETH_TOKEN_ADDRESS_MGX), MILLION);
			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1_000_000u128,
			)
			.unwrap();

			let withdrawal_update = Withdrawal {
				l2RequestId: sp_core::U256::from(u128::MAX / 2),
				withdrawalRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: U256::from(1_000_000u128),
			};
			// check iftokens were burned
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &ALICE), 0_u128);
			assert_eq!(TokensOf::<Test>::total_issuance(ETH_TOKEN_ADDRESS_MGX), 0_u128);
			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(u128::MAX / 2)),
				Some(PendingUpdate::Withdrawal(withdrawal_update))
			);
			assert_eq!(Rolldown::get_l2_origin_updates_counter(), u128::MAX / 2 + 1);
		});
}

#[test]
#[serial]
fn error_on_withdrawal_more() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			assert_err!(
				Rolldown::withdraw(
					RuntimeOrigin::signed(ALICE),
					ETH_RECIPIENT_ACCOUNT,
					ETH_TOKEN_ADDRESS,
					10_000_000u128
				),
				Error::<Test>::NotEnoughAssets
			);
		});
}

#[test]
#[serial]
fn test_remove_pending_updates() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			let deposit_request =
				create_l1_update(vec![L1UpdateRequest::Deposit(messages::Deposit {
					depositRecipient: ETH_RECIPIENT_ACCOUNT,
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
				})]);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_request.clone())
				.unwrap();
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();
			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1_000_000u128,
			)
			.unwrap();
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), deposit_request).unwrap();
			forward_to_block::<Test>(20);

			let withdrawal_update = Withdrawal {
				l2RequestId: sp_core::U256::from(u128::MAX / 2 + 1),
				withdrawalRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: U256::from(1_000_000u128),
			};
			let cancel_update = Cancel {
				l2RequestId: sp_core::U256::from(u128::MAX / 2),
				updater: 2,
				canceler: 3,
				lastProccessedRequestOnL1: sp_core::U256::from(0u128),
				lastAcceptedRequestOnL1: sp_core::U256::from(0u128),
				hash: H256::from(hex!(
					"196433741a7c64431d86b02f498f8fc42d567db23167fc0fccaa0e7b9a35092f"
				)),
			};

			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(0u128)),
				Some(PendingUpdate::RequestResult((true, UpdateType::DEPOSIT)))
			);
			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(u128::MAX / 2)),
				Some(PendingUpdate::Cancel(cancel_update))
			);
			assert_eq!(
				pending_updates::<Test>::get(sp_core::U256::from(u128::MAX / 2 + 1)),
				Some(PendingUpdate::Withdrawal(withdrawal_update))
			);

			let cancel_resolution_request = messages::CancelResolution {
				l2RequestId: sp_core::U256::from(u128::MAX / 2),
				cancelJustified: false,
			};

			let remove_pending_updates_request = messages::L2UpdatesToRemove {
				l2UpdatesToRemove: vec![
					sp_core::U256::from(0u128),
					sp_core::U256::from(u128::MAX / 2 + 1),
				],
			};
			let update = create_l1_update_with_offset(
				vec![
					L1UpdateRequest::Remove(remove_pending_updates_request),
					L1UpdateRequest::Cancel(cancel_resolution_request),
				],
				1_u128.into(),
			);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(CHARLIE), update).unwrap();
			forward_to_block::<Test>(30);
			assert_eq!(pending_updates::<Test>::get(sp_core::U256::from(0u128)), None);
			assert_eq!(pending_updates::<Test>::get(sp_core::U256::from(u128::MAX / 2)), None);
			assert_eq!(pending_updates::<Test>::get(sp_core::U256::from(u128::MAX / 2 + 1)), None);
		});
}
