use crate::{
	mock::{consts::*, *},
	*,
};
use core::future::pending;
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

struct L1UpdateBuilder(Option<u128>, Vec<L1UpdateRequest>);
impl L1UpdateBuilder {
	fn new() -> Self {
		Self(None, Default::default())
	}

	fn with_offset(mut self, offset: u128) -> Self {
		self.0 = Some(offset);
		self
	}

	fn with_requests(mut self, requests: Vec<L1UpdateRequest>) -> Self {
		self.1 = requests;
		self
	}

	fn build(self) -> messages::L1Update {
		let mut update = messages::L1Update::default();

		for (id, r) in self.1.into_iter().enumerate() {
			let rid = if let Some(offset) = self.0 { (id as u128) + offset } else { r.id() };
			match r {
				L1UpdateRequest::WithdrawalResolution(mut w) => {
					w.requestId.id = rid;
					update.pendingWithdrawalResolutions.push(w);
				},
				L1UpdateRequest::Deposit(mut d) => {
					d.requestId.id = rid;
					update.pendingDeposits.push(d);
				},
				L1UpdateRequest::CancelResolution(mut c) => {
					c.requestId.id = rid;
					update.pendingCancelResultions.push(c);
				},
				L1UpdateRequest::Remove(mut r) => {
					r.requestId.id = rid;
					update.pendingL2UpdatesToRemove.push(r);
				},
			}
		}
		update
	}
}

impl Default for L1UpdateBuilder {
	fn default() -> Self {
		Self(Some(1u128), Default::default())
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

		assert_event_emitted!(Event::L1ReadStored((
			ALICE,
			current_block_number + dispute_period,
			(1u128, 1u128).into(),
			H256::from(hex!("4c3d07c246f723174706be0b6e191b5d91d72180377232d9df34c0b6e5a0d397"))
		)));
	});
}

#[test]
#[serial]
fn create_pending_update_after_dispute_period() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update1 = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit::default())])
			.build();

		let update2 = L1UpdateBuilder::new()
			.with_requests(vec![
				L1UpdateRequest::Deposit(messages::Deposit::default()),
				L1UpdateRequest::Deposit(messages::Deposit::default()),
			])
			.with_offset(1u128)
			.build();

		forward_to_block::<Test>(10);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update1).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update2).unwrap();

		assert_eq!(pending_updates::<Test>::iter().next(), None);
		assert!(
			pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L1, 1u128)).is_none()
		);
		assert!(
			pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L1, 2u128)).is_none()
		);

		forward_to_block::<Test>(15);
		assert!(
			pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L1, 1u128)).is_some()
		);

		forward_to_block::<Test>(16);
		assert!(
			pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L1, 2u128)).is_some()
		);
	});
}

#[test]
#[serial]
fn l2_counter_updates_when_requests_are_processed() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let update1 = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
			.build();

		let update2 = L1UpdateBuilder::default()
			.with_requests(vec![
				L1UpdateRequest::Deposit(Default::default()),
				L1UpdateRequest::Deposit(Default::default()),
			])
			.build();

		forward_to_block::<Test>(10);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(L1::Ethereum), 0_u128.into());
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update1).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update2).unwrap();

		forward_to_block::<Test>(15);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(L1::Ethereum), 1u128.into());

		forward_to_block::<Test>(16);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(L1::Ethereum), 2u128.into());
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
					requestId: Default::default(),
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
					timeStamp: sp_core::U256::from(1),
				})])
				.build();

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();
			forward_to_block::<Test>(14);
			assert!(!pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 0u128)
			));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);
			assert_eq!(
				pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L1, 1u128)),
				Some(PendingUpdate::RequestResult(RequestResult {
					requestId: RequestId::new(Origin::L2, 1u128),
					originRequestId: 1u128,
					status: true,
					updateType: UpdateType::DEPOSIT
				}))
			);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);
		});
}

#[test]
#[serial]
fn deposit_fail_creates_update_with_status_false() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, 0u128)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			let update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit {
					requestId: Default::default(),
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from("3402823669209384634633746074317682114560"),
					timeStamp: sp_core::U256::from(1),
				})])
				.build();

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();
			forward_to_block::<Test>(14);
			assert!(!pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 0u128)
			));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);
			assert_eq!(
				pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L1, 1u128)),
				Some(PendingUpdate::RequestResult(RequestResult {
					requestId: RequestId::new(Origin::L2, 1u128),
					originRequestId: 1u128,
					status: false,
					updateType: UpdateType::DEPOSIT
				}))
			);
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
					requestId: Default::default(),
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
					timeStamp: sp_core::U256::from(1),
				})])
				.build();

			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);
			assert!(!pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 1u128)
			));
			Rolldown::force_update_l2_from_l1(RuntimeOrigin::root(), update).unwrap();
			assert!(pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 1u128)
			));
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
					requestId: Default::default(),
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
					timeStamp: sp_core::U256::from(1),
				})])
				.build();
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update.clone()).unwrap();

			forward_to_block::<Test>(11);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update).unwrap();

			forward_to_block::<Test>(14);
			assert!(!pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 0u128)
			));
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

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit {
					requestId: Default::default(),
					depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
					tokenAddress: ETH_TOKEN_ADDRESS,
					amount: sp_core::U256::from(MILLION),
					timeStamp: sp_core::U256::from(1),
				})])
				.build();

			let l2_updates_to_remove = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Remove(messages::L2UpdatesToRemove {
					requestId: Default::default(),
					l2UpdatesToRemove: vec![1u128],
					timeStamp: sp_core::U256::from(1),
				})])
				.with_offset(2u128)
				.build();

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();

			forward_to_block::<Test>(15);
			assert!(pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 1u128)
			));

			forward_to_block::<Test>(100);
			assert_eq!(
				pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L1, 1u128)),
				Some(PendingUpdate::RequestResult(RequestResult {
					requestId: RequestId::new(Origin::L2, 1u128),
					originRequestId: 1u128,
					status: true,
					updateType: UpdateType::DEPOSIT
				}))
			);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), l2_updates_to_remove)
				.unwrap();

			forward_to_block::<Test>(104);
			assert!(pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 1u128)
			));
			assert!(!pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 2u128)
			));

			forward_to_block::<Test>(105);
			assert!(pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 2u128)
			));
			assert!(!pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L1, 1u128)
			));
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

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![
					L1UpdateRequest::Deposit(Default::default()),
					L1UpdateRequest::Deposit(Default::default()),
				])
				.build();

			let cancel_resolution = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::CancelResolution(
					messages::CancelResolution {
						requestId: Default::default(),
						l2RequestId: 1u128,
						cancelJustified: true,
						timeStamp: sp_core::U256::from(1),
					},
				)])
				.build();

			assert!(!pending_requests::<Test>::contains_key(15u128, L1::Ethereum));

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			assert!(pending_requests::<Test>::contains_key(15u128, L1::Ethereum));
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();

			// Assert
			assert!(!pending_requests::<Test>::contains_key(15u128, L1::Ethereum));
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
			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
				.build();

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			let req: messages::eth_abi::L1Update =
				pending_requests::<Test>::get(15u128, L1::Ethereum).unwrap().1.into();

			assert_eq!(
				Rolldown::get_l2_update(L1::Ethereum),
				messages::eth_abi::L2Update {
					withdrawals: vec![],
					cancels: vec![],
					results: vec![]
				}
			);

			let update_id = Rolldown::get_l2_origin_updates_counter(L1::Ethereum);
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();

			assert_eq!(
				Rolldown::get_l2_update(L1::Ethereum),
				messages::eth_abi::L2Update {
					cancels: vec![messages::eth_abi::Cancel {
						requestId: messages::eth_abi::RequestId {
							origin: messages::eth_abi::Origin::L2,
							id: messages::to_eth_u256(U256::from(update_id))
						},
						range: messages::eth_abi::Range {
							start: messages::to_eth_u256(U256::from(1)),
							end: messages::to_eth_u256(U256::from(1))
						},
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

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
				.build();

			let l2_request_id = Rolldown::get_l2_origin_updates_counter(L1::Ethereum) + 1;
			let cancel_resolution = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::CancelResolution(
					messages::CancelResolution {
						requestId: Default::default(),
						l2RequestId: l2_request_id,
						cancelJustified: true,
						timeStamp: sp_core::U256::from(1),
					},
				)])
				.with_offset(1u128)
				.build();

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			forward_to_block::<Test>(11);
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128).unwrap();
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

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
				.build();

			let l2_request_id = Rolldown::get_l2_origin_updates_counter(L1::Ethereum) + 1;
			let cancel_resolution = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::CancelResolution(
					messages::CancelResolution {
						requestId: Default::default(),
						l2RequestId: l2_request_id,
						cancelJustified: false,
						timeStamp: sp_core::U256::from(1),
					},
				)])
				.with_offset(1u128)
				.build();

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			forward_to_block::<Test>(11);
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128).unwrap();
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

			let l2_request_id = Rolldown::get_l2_origin_updates_counter(L1::Ethereum);
			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit::default())])
				.build();

			let cancel_resolution = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::CancelResolution(
					messages::CancelResolution {
						requestId: Default::default(),
						l2RequestId: l2_request_id,
						cancelJustified: true,
						timeStamp: sp_core::U256::from(1),
					},
				)])
				.with_offset(1u128)
				.build();

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

			assert!(!pending_requests::<Test>::contains_key(15u128, L1::Ethereum));
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
		let update = L1UpdateBuilder::new()
			.with_requests(vec![
				L1UpdateRequest::Deposit(messages::Deposit {
					requestId: RequestId::new(Origin::L1, 1u128),
					depositRecipient: hex!("0000000000000000000000000000000000000002"),
					tokenAddress: hex!("0000000000000000000000000000000000000003"),
					amount: 4u128.into(),
					timeStamp: sp_core::U256::from(1),
				}),
				L1UpdateRequest::CancelResolution(messages::CancelResolution {
					requestId: RequestId::new(Origin::L1, 6u128),
					l2RequestId: 7u128,
					cancelJustified: true,
					timeStamp: sp_core::U256::from(2),
				}),
				L1UpdateRequest::WithdrawalResolution(messages::WithdrawalResolution {
					requestId: RequestId::new(Origin::L1, 9u128),
					l2RequestId: 10u128,
					status: true,
					timeStamp: sp_core::U256::from(3),
				}),
				L1UpdateRequest::Remove(messages::L2UpdatesToRemove {
					requestId: RequestId::new(Origin::L1, 12u128),
					l2UpdatesToRemove: vec![13u128],
					timeStamp: sp_core::U256::from(4),
				}),
			])
			.build();
		let hash = Rolldown::calculate_hash_of_pending_requests(update);
		assert_eq!(
			hash,
			hex!("6ebab65d2a7e2e2ac74b0415ccb2943ed7818bec57609986ab154b6880311c89").into()
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

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(messages::Deposit::default())])
				.build();

			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 2u128 }
			);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 0u128, cancelRights: 2u128 }
			);

			Rolldown::force_cancel_requests_from_l1(RuntimeOrigin::root(), 15u128.into()).unwrap();
			assert_eq!(
				sequencer_rights::<Test>::get(ALICE).unwrap(),
				SequencerRights { readRights: 1u128, cancelRights: 2u128 }
			);
		});
}

#[test]
#[serial]
fn execute_a_lot_of_requests_in_following_blocks() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		let requests_count = 25;
		let requests = vec![L1UpdateRequest::Deposit(messages::Deposit::default()); requests_count];

		let deposit_update = L1UpdateBuilder::default().with_requests(requests).build();
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();

		forward_to_block::<Test>(14);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), 0u128.into());
		assert_eq!(request_to_execute_cnt::<Test>::get(), 0u128);

		forward_to_block::<Test>(15);
		assert_eq!(
			last_processed_request_on_l2::<Test>::get(L1::Ethereum),
			Rolldown::get_max_requests_per_block().into()
		);

		forward_to_block::<Test>(16);
		assert_eq!(
			last_processed_request_on_l2::<Test>::get(L1::Ethereum),
			(2u128 * Rolldown::get_max_requests_per_block()).into()
		);

		forward_to_block::<Test>(17);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), requests_count as u128);

		forward_to_block::<Test>(100);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), requests_count as u128);
	});
}

#[test]
#[serial]
fn ignore_duplicated_requests_when_already_executed() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let dummy_request = L1UpdateRequest::Deposit(Default::default());
		let first_update =
			L1UpdateBuilder::default().with_requests(vec![dummy_request.clone(); 5]).build();
		let second_update =
			L1UpdateBuilder::default().with_requests(vec![dummy_request; 6]).build();

		forward_to_block::<Test>(10);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), second_update).unwrap();

		forward_to_block::<Test>(14);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), 0u128.into());

		forward_to_block::<Test>(15);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), 5u128.into());

		forward_to_block::<Test>(16);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), 6u128.into());
	});
}

#[test]
#[serial]
fn process_l1_reads_in_order() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let dummy_request = L1UpdateRequest::Deposit(Default::default());
		let first_update = L1UpdateBuilder::default()
			.with_requests(vec![dummy_request.clone(); 11])
			.build();
		let second_update =
			L1UpdateBuilder::default().with_requests(vec![dummy_request; 20]).build();

		forward_to_block::<Test>(10);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), second_update).unwrap();

		forward_to_block::<Test>(14);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), 0u128.into());

		forward_to_block::<Test>(15);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), 10u128.into());

		forward_to_block::<Test>(16);
		assert_eq!(last_processed_request_on_l2::<Test>::get(L1::Ethereum), 20u128.into());
	});
}

#[test]
#[serial]
fn check_request_ids_starts_from_one() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let requests = vec![L1UpdateRequest::Deposit(Default::default())];

		assert_err!(
			Rolldown::update_l2_from_l1(
				RuntimeOrigin::signed(ALICE),
				L1UpdateBuilder::new()
					.with_requests(requests.clone())
					.with_offset(0u128)
					.build()
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

		let deposit_update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
			.with_offset(100u128)
			.build();

		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update),
			Error::<Test>::WrongRequestId
		);
	});
}

#[test]
#[serial]
fn reject_update_with_invalid_too_high_request_id() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		let deposit_update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
			.with_offset(2u128)
			.build();

		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update),
			Error::<Test>::WrongRequestId
		);
	});
}

#[test]
#[serial]
fn reject_second_update_in_the_same_block() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);
		let deposit_update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
			.build();

		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update.clone()).unwrap();
		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), deposit_update),
			Error::<Test>::MultipleUpdatesInSingleBlock
		)
	});
}

#[test]
#[serial]
fn accept_consecutive_update_split_into_two() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);

		// imagine that there are 20 request on L1 waiting to be processed
		// they need to be split into 2 update_l2_from_l1 calls

		let dummy_update = L1UpdateRequest::Deposit(Default::default());

		let first_update = L1UpdateBuilder::default()
			.with_requests(vec![
				dummy_update.clone();
				(2 * Rolldown::get_max_requests_per_block()) as usize
			])
			.with_offset(1u128)
			.build();

		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();

		forward_to_block::<Test>(15);
		let mut expected_updates = pending_updates::<Test>::iter_prefix(L1::Ethereum)
			.map(|(k, _)| k.id)
			.collect::<Vec<_>>();
		expected_updates.sort();

		assert_eq!(
			(1u128..11u128).collect::<Vec<_>>().into_iter().collect::<Vec<_>>(),
			expected_updates
		);

		forward_to_block::<Test>(16);
		let mut expected_updates = pending_updates::<Test>::iter_prefix(L1::Ethereum)
			.map(|(k, _)| k.id)
			.collect::<Vec<_>>();
		expected_updates.sort();
		assert_eq!(
			(1u128..21u128).collect::<Vec<_>>().into_iter().collect::<Vec<_>>(),
			expected_updates
		);
	});
}

#[test]
#[serial]
fn execute_two_consecutive_incremental_reqeusts() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, 0u128)
		.execute_with_default_mocks(|| {
			let dummy_update = L1UpdateRequest::Deposit(messages::Deposit {
				requestId: Default::default(),
				depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
				timeStamp: sp_core::U256::from(1),
			});

			let first_update = L1UpdateBuilder::default()
				.with_requests(vec![dummy_update.clone()])
				.with_offset(1u128)
				.build();

			let second_update = L1UpdateBuilder::default()
				.with_requests(vec![dummy_update.clone(), dummy_update.clone()])
				.with_offset(1u128)
				.build();

			forward_to_block::<Test>(10);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();

			forward_to_block::<Test>(11);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), second_update).unwrap();

			forward_to_block::<Test>(14);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);

			forward_to_block::<Test>(16);
			assert_eq!(
				TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE),
				2 * MILLION
			);

			forward_to_block::<Test>(100);
			assert_eq!(
				TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE),
				2 * MILLION
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
				requestId: (Origin::L2, 1u128).into(),
				withdrawalRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: U256::from(1_000_000u128),
			};
			// check iftokens were burned
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &ALICE), 0_u128);
			assert_eq!(TokensOf::<Test>::total_issuance(ETH_TOKEN_ADDRESS_MGX), 0_u128);
			assert_eq!(
				pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L2, 1u128)),
				Some(PendingUpdate::Withdrawal(withdrawal_update))
			);
			assert_eq!(Rolldown::get_l2_origin_updates_counter(L1::Ethereum), 2);
		});
}

#[test]
#[serial]
fn error_on_withdraw_too_much() {
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

			let deposit_request = L1UpdateRequest::Deposit(messages::Deposit {
				requestId: Default::default(),
				depositRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: sp_core::U256::from(MILLION),
				timeStamp: sp_core::U256::from(1),
			});

			let update_with_deposit = L1UpdateBuilder::default()
				.with_requests(vec![deposit_request.clone()])
				.with_offset(1u128)
				.build();

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update_with_deposit.clone())
				.unwrap();
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), 15u128.into()).unwrap();
			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1_000_000u128,
			)
			.unwrap();

			let withdrawal_update = Withdrawal {
				requestId: (Origin::L2, 2u128).into(),
				withdrawalRecipient: ETH_RECIPIENT_ACCOUNT,
				tokenAddress: ETH_TOKEN_ADDRESS,
				amount: U256::from(1_000_000u128),
			};
			let cancel_update = Cancel {
				requestId: (Origin::L2, 1u128).into(),
				updater: 2,
				canceler: 3,
				range: (1u128, 1u128).into(),
				hash: H256::from(hex!(
					"604b7bda301c1bc3cfb8c2c41476a6587a6f5cbf583f38c2100dd7cd27f146d1"
				)),
			};
			assert_eq!(
				pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L2, 1u128)),
				Some(PendingUpdate::Cancel(cancel_update))
			);
			assert_eq!(
				pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L2, 2u128)),
				Some(PendingUpdate::Withdrawal(withdrawal_update))
			);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update_with_deposit).unwrap();
			forward_to_block::<Test>(20);

			assert_eq!(
				pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L1, 1u128)),
				Some(PendingUpdate::RequestResult(RequestResult {
					requestId: RequestId::new(Origin::L2, 3u128),
					originRequestId: 1u128,
					status: true,
					updateType: UpdateType::DEPOSIT
				}))
			);

			let cancel_resolution_request = messages::CancelResolution {
				requestId: RequestId { origin: Origin::L1, id: 2u128 },
				l2RequestId: 1u128,
				cancelJustified: false,
				timeStamp: sp_core::U256::from(1),
			};

			let remove_pending_updates_request = messages::L2UpdatesToRemove {
				requestId: RequestId { origin: Origin::L1, id: 3u128 },
				l2UpdatesToRemove: vec![1u128],
				timeStamp: sp_core::U256::from(1),
			};

			let update_with_remove_and_resolution = L1UpdateBuilder::new()
				.with_requests(vec![
					L1UpdateRequest::Remove(remove_pending_updates_request),
					L1UpdateRequest::CancelResolution(cancel_resolution_request),
				])
				.build();

			Rolldown::update_l2_from_l1(
				RuntimeOrigin::signed(CHARLIE),
				update_with_remove_and_resolution,
			)
			.unwrap();

			forward_to_block::<Test>(30);
			assert_eq!(
				pending_updates::<Test>::get(
					L1::Ethereum,
					RequestId { origin: Origin::L1, id: 1u128 }
				),
				None
			);
			assert_eq!(
				pending_updates::<Test>::get(
					L1::Ethereum,
					RequestId { origin: Origin::L2, id: 1u128 }
				),
				None
			);
		});
}

#[test]
#[serial]
fn test_reproduce_bug_with_incremental_updates() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, 10_000u128)
		.execute_with_default_mocks(|| {
			let first_update = L1UpdateBuilder::new()
				.with_requests(vec![
					L1UpdateRequest::Deposit(messages::Deposit {
						requestId: RequestId::new(Origin::L1, 1u128),
						depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
						tokenAddress: ETH_TOKEN_ADDRESS,
						amount: sp_core::U256::from(MILLION),
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::Deposit(messages::Deposit {
						requestId: RequestId::new(Origin::L1, 2u128),
						depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
						tokenAddress: ETH_TOKEN_ADDRESS,
						amount: sp_core::U256::from(MILLION),
						timeStamp: sp_core::U256::from(1),
					}),
				])
				.with_offset(1u128)
				.build();

			let second_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Remove(messages::L2UpdatesToRemove {
					requestId: RequestId::new(Origin::L1, 3u128),
					l2UpdatesToRemove: vec![1u128, 2u128],
					timeStamp: sp_core::U256::from(1),
				})])
				.with_offset(3u128)
				.build();

			let third_update = L1UpdateBuilder::default()
				.with_requests(vec![
					L1UpdateRequest::Remove(messages::L2UpdatesToRemove {
						requestId: RequestId::new(Origin::L1, 3u128),
						l2UpdatesToRemove: vec![1u128, 2u128],
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::WithdrawalResolution(messages::WithdrawalResolution {
						requestId: RequestId::new(Origin::L1, 4u128),
						l2RequestId: 3u128,
						status: false,
						timeStamp: sp_core::U256::from(1),
					}),
				])
				.with_offset(3u128)
				.build();

			forward_to_block::<Test>(10);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();

			forward_to_block::<Test>(20);
			assert!(!pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L2, 3u128)
			));
			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				10u128,
			)
			.unwrap();
			assert!(pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L2, 3u128)
			));
			let withdrawal_update =
				pending_updates::<Test>::get(L1::Ethereum, RequestId::new(Origin::L2, 3u128));
			assert!(matches!(withdrawal_update, Some(PendingUpdate::Withdrawal(_))));

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), second_update).unwrap();

			forward_to_block::<Test>(30);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), third_update).unwrap();

			forward_to_block::<Test>(40);
			assert!(!pending_updates::<Test>::contains_key(
				L1::Ethereum,
				RequestId::new(Origin::L2, 3u128)
			));
		});
}

#[test]
#[serial]
fn test_get_native_l1_update() {
	let raw_payload_fetched_from_contract = hex!(
	"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000b7278a61aa25c888815afc32ad3cc52ff24fe57500000000000000000000000000000000000000000000000000000000000007d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000b7278a61aa25c888815afc32ad3cc52ff24fe57500000000000000000000000000000000000000000000000000000000000007d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
	);
	let mut input = Vec::new();
	input.extend_from_slice(&raw_payload_fetched_from_contract);
	Rolldown::convert_eth_l1update_to_substrate_l1update(input).expect("conversion works");
}

#[test]
#[serial]
fn test_withdrawal_resolution_works_passes_validation() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, 10_000u128)
		.execute_with_default_mocks(|| {
			let first_update = L1UpdateBuilder::new()
				.with_requests(vec![
					L1UpdateRequest::Deposit(messages::Deposit {
						requestId: RequestId::new(Origin::L1, 33u128),
						depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
						tokenAddress: ETH_TOKEN_ADDRESS,
						amount: sp_core::U256::from(MILLION),
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::Deposit(messages::Deposit {
						requestId: RequestId::new(Origin::L1, 34u128),
						depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
						tokenAddress: ETH_TOKEN_ADDRESS,
						amount: sp_core::U256::from(MILLION),
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::WithdrawalResolution(messages::WithdrawalResolution {
						requestId: RequestId::new(Origin::L1, 30u128),
						l2RequestId: 31u128,
						status: true,
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::Remove(messages::L2UpdatesToRemove {
						requestId: RequestId::new(Origin::L1, 31u128),
						l2UpdatesToRemove: vec![27u128, 28u128],
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::Remove(messages::L2UpdatesToRemove {
						requestId: RequestId::new(Origin::L1, 32u128),
						l2UpdatesToRemove: vec![29u128],
						timeStamp: sp_core::U256::from(1),
					}),
				])
				.build();

			last_processed_request_on_l2::<Test>::insert(L1::Ethereum, 29);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();
		});
}
