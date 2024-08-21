use crate::{
	messages::Chain,
	mock::{consts::*, *},
	*,
};

use frame_support::{assert_err, assert_ok};
use hex_literal::hex;
use messages::L1UpdateRequest;

use serial_test::serial;

use sp_runtime::traits::ConvertBack;
use sp_std::iter::FromIterator;

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
				L1UpdateRequest::FailedWithdrawalResolution(mut w) => {
					w.requestId.id = rid;
					update.pendingFailedWithdrawalResolutions.push(w);
				},
				L1UpdateRequest::Deposit(mut d) => {
					d.requestId.id = rid;
					update.pendingDeposits.push(d);
				},
				L1UpdateRequest::CancelResolution(mut c) => {
					c.requestId.id = rid;
					update.pendingCancelResolutions.push(c);
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
fn test_genesis() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		assert_eq!(
			SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap().read_rights,
			1
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
			messages::Chain::Ethereum,
			ALICE,
			current_block_number + dispute_period,
			(1u128, 1u128).into(),
			hex!("e533f01e84d8d54b6e5b817f59dbbda41efc2b627adc5d59c78b15445ee2d863").into()
		)));
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
		assert_eq!(Rolldown::get_last_processed_request_on_l2(Chain::Ethereum), 0_u128.into());
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update1).unwrap();

		forward_to_block::<Test>(11);
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), update2).unwrap();

		forward_to_block::<Test>(15);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(Chain::Ethereum), 1u128.into());

		forward_to_block::<Test>(16);
		assert_eq!(Rolldown::get_last_processed_request_on_l2(Chain::Ethereum), 2u128.into());
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
			assert!(!L2Requests::<Test>::contains_key(
				Chain::Ethereum,
				RequestId::new(Origin::L1, 0u128)
			));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), MILLION);
		});
}

#[test]
#[ignore]
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
			assert!(!L2Requests::<Test>::contains_key(
				Chain::Ethereum,
				RequestId::new(Origin::L1, 0u128)
			));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);

			forward_to_block::<Test>(15);

			assert_event_emitted!(Event::RequestProcessedOnL2(
				messages::Chain::Ethereum,
				1u128,
				false
			));
			assert_eq!(TokensOf::<Test>::free_balance(ETH_TOKEN_ADDRESS_MGX, &CHARLIE), 0_u128);
			//TODO: check that withdrawal is created in place of failed deposit
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
			assert!(!L2Requests::<Test>::contains_key(
				Chain::Ethereum,
				RequestId::new(Origin::L1, 1u128)
			));
			assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 0u128.into());
			Rolldown::force_update_l2_from_l1(RuntimeOrigin::root(), update).unwrap();
			assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 1u128.into());
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
			assert!(!L2Requests::<Test>::contains_key(
				Chain::Ethereum,
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

			assert!(!PendingSequencerUpdates::<Test>::contains_key(15u128, Chain::Ethereum));

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			assert!(PendingSequencerUpdates::<Test>::contains_key(15u128, Chain::Ethereum));
			Rolldown::cancel_requests_from_l1(
				RuntimeOrigin::signed(BOB),
				consts::CHAIN,
				15u128.into(),
			)
			.unwrap();

			// Assert
			assert!(!PendingSequencerUpdates::<Test>::contains_key(15u128, Chain::Ethereum));
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

			Rolldown::cancel_requests_from_l1(
				RuntimeOrigin::signed(BOB),
				consts::CHAIN,
				15u128.into(),
			)
			.unwrap();

			assert_eq!(
				L2Requests::<Test>::get(Chain::Ethereum, RequestId::new(Origin::L2, 1u128))
					.unwrap()
					.0,
				Cancel {
					requestId: RequestId::new(Origin::L2, 1u128),
					updater: ALICE,
					canceler: BOB,
					range: (1u128, 1u128).into(),
					hash: hex!("e533f01e84d8d54b6e5b817f59dbbda41efc2b627adc5d59c78b15445ee2d863")
						.into()
				}
				.into()
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

			let l2_request_id = Rolldown::get_l2_origin_updates_counter(Chain::Ethereum);
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
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 2u128 }
			);

			forward_to_block::<Test>(11);
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), consts::CHAIN, 15u128)
				.unwrap();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 1u128 }
			);

			forward_to_block::<Test>(12);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 1u128 }
			);

			forward_to_block::<Test>(16);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock
				.expect()
				.withf(|chain, a, b| {
					*chain == consts::CHAIN && *a == ALICE && b.cloned() == Some(BOB)
				})
				.times(1)
				.return_const(Ok(().into()));

			forward_to_block::<Test>(17);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
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

			let l2_request_id = Rolldown::get_l2_origin_updates_counter(Chain::Ethereum);
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

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 2u128 }
			);
			forward_to_block::<Test>(11);
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), consts::CHAIN, 15u128)
				.unwrap();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 1u128 }
			);

			forward_to_block::<Test>(12);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			forward_to_block::<Test>(16);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock
				.expect()
				.withf(|chain, a, b| *chain == consts::CHAIN && *a == BOB && b.cloned() == None)
				.times(1)
				.return_const(Ok(().into()));
			forward_to_block::<Test>(17);

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
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
				Rolldown::cancel_requests_from_l1(
					RuntimeOrigin::signed(BOB),
					consts::CHAIN,
					15u128.into()
				),
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

			let l2_request_id = Rolldown::get_l2_origin_updates_counter(Chain::Ethereum);
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
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 2u128 }
			);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);

			Rolldown::cancel_requests_from_l1(
				RuntimeOrigin::signed(BOB),
				consts::CHAIN,
				15u128.into(),
			)
			.unwrap();

			assert!(!PendingSequencerUpdates::<Test>::contains_key(15u128, Chain::Ethereum));
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 2u128 }
			);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 1u128 }
			);

			forward_to_block::<Test>(11);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 1u128 }
			);

			forward_to_block::<Test>(16);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
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
				L1UpdateRequest::FailedWithdrawalResolution(messages::FailedWithdrawalResolution {
					requestId: RequestId::new(Origin::L1, 9u128),
					l2RequestId: 10u128,
					timeStamp: sp_core::U256::from(3),
				}),
			])
			.build();
		let hash = Rolldown::calculate_hash_of_sequencer_update(update);
		assert_eq!(
			hash,
			hex!("64ba87c85cf50b0c6596157b5505a863c56e638d36a59cc8d84f1d0b21a07ad0").into()
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
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update).unwrap();
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 2u128 }
			);

			Rolldown::force_cancel_requests_from_l1(
				RuntimeOrigin::root(),
				consts::CHAIN,
				15u128.into(),
			)
			.unwrap();
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
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
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 0u128.into());
		assert_eq!(UpdatesExecutionQueueNextId::<Test>::get(), 0u128);

		forward_to_block::<Test>(15);
		assert_eq!(
			LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum),
			Rolldown::get_max_requests_per_block().into()
		);

		forward_to_block::<Test>(16);
		assert_eq!(
			LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum),
			(2u128 * Rolldown::get_max_requests_per_block()).into()
		);

		forward_to_block::<Test>(17);
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), requests_count as u128);

		forward_to_block::<Test>(100);
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), requests_count as u128);
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
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 0u128.into());

		forward_to_block::<Test>(15);
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 5u128.into());

		forward_to_block::<Test>(16);
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 6u128.into());
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
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 0u128.into());

		forward_to_block::<Test>(15);
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 10u128.into());

		forward_to_block::<Test>(16);
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 20u128.into());
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

		forward_to_block::<Test>(14);
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 0);

		forward_to_block::<Test>(15);
		let mut expected_updates = L2Requests::<Test>::iter_prefix(Chain::Ethereum)
			.map(|(k, _)| k.id)
			.collect::<Vec<_>>();
		expected_updates.sort();

		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 10);

		forward_to_block::<Test>(16);
		assert_eq!(LastProcessedRequestOnL2::<Test>::get(Chain::Ethereum), 20);
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
				consts::CHAIN,
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
				L2Requests::<Test>::get(Chain::Ethereum, RequestId::new(Origin::L2, 1u128))
					.unwrap()
					.0,
				L2Request::Withdrawal(withdrawal_update)
			);
			assert_eq!(Rolldown::get_l2_origin_updates_counter(Chain::Ethereum), 2);
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
					consts::CHAIN,
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
				.with_requests(vec![L1UpdateRequest::FailedWithdrawalResolution(
					messages::FailedWithdrawalResolution {
						requestId: RequestId::new(Origin::L1, 4u128),
						l2RequestId: 3u128,
						timeStamp: sp_core::U256::from(1),
					},
				)])
				.with_offset(3u128)
				.build();

			forward_to_block::<Test>(10);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();

			forward_to_block::<Test>(20);
			assert!(!L2Requests::<Test>::contains_key(
				Chain::Ethereum,
				RequestId::new(Origin::L2, 3u128)
			));
			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				10u128,
			)
			.unwrap();
			assert_eq!(Rolldown::get_last_processed_request_on_l2(Chain::Ethereum), 2_u128.into());
			let withdrawal_update =
				L2Requests::<Test>::get(Chain::Ethereum, RequestId::new(Origin::L2, 1u128));
			assert!(matches!(withdrawal_update, Some((L2Request::Withdrawal(_), _))));

			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), second_update).unwrap();

			forward_to_block::<Test>(40);
			assert!(!L2Requests::<Test>::contains_key(
				Chain::Ethereum,
				RequestId::new(Origin::L2, 3u128)
			));
		});
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
						requestId: RequestId::new(Origin::L1, 31u128),
						depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
						tokenAddress: ETH_TOKEN_ADDRESS,
						amount: sp_core::U256::from(MILLION),
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::Deposit(messages::Deposit {
						requestId: RequestId::new(Origin::L1, 32u128),
						depositRecipient: DummyAddressConverter::convert_back(CHARLIE),
						tokenAddress: ETH_TOKEN_ADDRESS,
						amount: sp_core::U256::from(MILLION),
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::FailedWithdrawalResolution(
						messages::FailedWithdrawalResolution {
							requestId: RequestId::new(Origin::L1, 30u128),
							l2RequestId: 31u128,
							timeStamp: sp_core::U256::from(1),
						},
					),
				])
				.build();

			LastProcessedRequestOnL2::<Test>::insert(Chain::Ethereum, 29);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), first_update).unwrap();
		});
}

#[test]
#[serial]
fn test_new_sequencer_active() {
	ExtBuilder::new_without_default_sequencers().build().execute_with(|| {
		for i in 0..100 {
			Rolldown::new_sequencer_active(consts::CHAIN, &i);
			let read_rights: u128 = 1;
			let cancel_rights: u128 = i.into();
			assert_eq!(
				SequencersRights::<Test>::get(consts::CHAIN).into_values().count() as u128,
				<u64 as Into<u128>>::into(i) + 1
			);
			assert!(SequencersRights::<Test>::get(consts::CHAIN)
				.iter()
				.all(|(_, x)| x.read_rights == read_rights && x.cancel_rights == cancel_rights));

			assert_eq!(
				SequencersRights::<Test>::get(consts::CHAIN).into_values().count(),
				(i + 1) as usize
			);
		}
	});
}

#[test]
#[serial]
fn test_sequencer_unstaking() {
	ExtBuilder::new_without_default_sequencers().build().execute_with(|| {
		forward_to_block::<Test>(1);
		let dispute_period_length = Rolldown::get_dispute_period();
		let now = frame_system::Pallet::<Test>::block_number().saturated_into::<u128>();

		LastUpdateBySequencer::<Test>::insert((consts::CHAIN, ALICE), now);
		forward_to_block::<Test>((now + dispute_period_length).saturated_into::<u64>());
		assert_err!(
			Rolldown::sequencer_unstaking(consts::CHAIN, &ALICE),
			Error::<Test>::SequencerLastUpdateStillInDisputePeriod
		);
		forward_to_block::<Test>((now + dispute_period_length + 1).saturated_into::<u64>());
		assert_ok!(Rolldown::sequencer_unstaking(consts::CHAIN, &ALICE));
		assert_eq!(LastUpdateBySequencer::<Test>::get((consts::CHAIN, ALICE)), 0);

		AwaitingCancelResolution::<Test>::insert(
			(consts::CHAIN, ALICE),
			BTreeSet::from([(0, DisputeRole::Canceler)]),
		);
		assert_err!(
			Rolldown::sequencer_unstaking(consts::CHAIN, &ALICE),
			Error::<Test>::SequencerAwaitingCancelResolution
		);

		AwaitingCancelResolution::<Test>::remove((consts::CHAIN, ALICE));
		assert_ok!(Rolldown::sequencer_unstaking(consts::CHAIN, &ALICE));
		assert_eq!(AwaitingCancelResolution::<Test>::get((consts::CHAIN, ALICE)), BTreeSet::new());
	});
}

#[test]
#[serial]
fn test_last_update_by_sequencer_is_updated() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		let block = 36;
		forward_to_block::<Test>(block);

		assert_eq!(LastUpdateBySequencer::<Test>::get((consts::CHAIN, ALICE)), 0);

		let update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
			.build();
		Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();

		assert_eq!(LastUpdateBySequencer::<Test>::get((consts::CHAIN, ALICE)), block.into());
	});
}

#[test]
#[serial]
fn test_cancel_updates_awaiting_cancel_resolution() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![
					L1UpdateRequest::Deposit(Default::default()),
					L1UpdateRequest::Deposit(Default::default()),
				])
				.build();

			assert!(!PendingSequencerUpdates::<Test>::contains_key(15u128, Chain::Ethereum));

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), deposit_update.clone())
				.unwrap();
			assert!(PendingSequencerUpdates::<Test>::contains_key(15u128, Chain::Ethereum));

			let l2_request_id = Rolldown::get_l2_origin_updates_counter(Chain::Ethereum);
			Rolldown::cancel_requests_from_l1(
				RuntimeOrigin::signed(BOB),
				consts::CHAIN,
				15u128.into(),
			)
			.unwrap();

			assert_event_emitted!(Event::L1ReadCanceled {
				canceled_sequencer_update: 15u128,
				chain: consts::CHAIN,
				assigned_id: RequestId::new(Origin::L2, l2_request_id)
			});

			// Assert
			assert_eq!(
				AwaitingCancelResolution::<Test>::get((consts::CHAIN, ALICE)),
				BTreeSet::from([(1, DisputeRole::Submitter)])
			);
			assert_eq!(
				AwaitingCancelResolution::<Test>::get((consts::CHAIN, BOB)),
				BTreeSet::from([(1, DisputeRole::Canceler)])
			);
		});
}

#[test]
#[serial]
fn test_cancel_resolution_updates_awaiting_cancel_resolution() {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			// Arrange

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
				.build();

			let l2_request_id = Rolldown::get_l2_origin_updates_counter(Chain::Ethereum);

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
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(BOB), consts::CHAIN, 15u128)
				.unwrap();
			forward_to_block::<Test>(12);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), cancel_resolution).unwrap();
			assert_eq!(
				AwaitingCancelResolution::<Test>::get((consts::CHAIN, ALICE)),
				BTreeSet::from([(1, DisputeRole::Submitter)])
			);
			assert_eq!(
				AwaitingCancelResolution::<Test>::get((consts::CHAIN, BOB)),
				BTreeSet::from([(1, DisputeRole::Canceler)])
			);
			forward_to_block::<Test>(16);

			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock
				.expect()
				.withf(|chain, a, b| {
					*chain == consts::CHAIN && *a == ALICE && b.cloned() == Some(BOB)
				})
				.times(1)
				.return_const(Ok(().into()));
			forward_to_block::<Test>(17);

			assert_eq!(
				AwaitingCancelResolution::<Test>::get((consts::CHAIN, ALICE)),
				BTreeSet::new()
			);
			assert_eq!(
				AwaitingCancelResolution::<Test>::get((consts::CHAIN, BOB)),
				BTreeSet::new()
			);
		})
}

#[test]
#[serial]
fn test_handle_sequencer_deactivations() {
	ExtBuilder::new_without_default_sequencers().build().execute_with(|| {
		let total_sequencers = 100;
		for i in 0..total_sequencers {
			Rolldown::new_sequencer_active(consts::CHAIN, &i);
		}

		let n_max = 14;
		let mut acc = 0;
		for n in 1..n_max {
			Rolldown::handle_sequencer_deactivations(
				consts::CHAIN,
				Vec::<AccountId>::from_iter(acc..(n + acc)),
			);
			acc += n;
			let read_rights: u128 = 1;
			let cancel_rights: u128 = (total_sequencers - acc - 1).into();
			assert_eq!(
				SequencersRights::<Test>::get(consts::CHAIN).into_values().count() as u128,
				<u64 as Into<u128>>::into(total_sequencers - acc)
			);
			assert!(SequencersRights::<Test>::get(consts::CHAIN)
				.iter()
				.all(|(_, x)| x.read_rights == read_rights && x.cancel_rights == cancel_rights));
			assert_eq!(
				SequencersRights::<Test>::get(consts::CHAIN).keys().count(),
				(total_sequencers - acc) as usize
			);
		}
	});
}

#[test]
#[serial]
fn test_maintenance_mode_blocks_extrinsics() {
	ExtBuilder::new().build().execute_with(|| {
		let is_maintenance_mock = MockMaintenanceStatusProviderApi::is_maintenance_context();
		is_maintenance_mock.expect().return_const(true);

		assert_err!(
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), Default::default()),
			Error::<Test>::BlockedByMaintenanceMode
		);
		assert_err!(
			Rolldown::force_update_l2_from_l1(RuntimeOrigin::root(), Default::default()),
			Error::<Test>::BlockedByMaintenanceMode
		);
		assert_err!(
			Rolldown::cancel_requests_from_l1(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				Default::default()
			),
			Error::<Test>::BlockedByMaintenanceMode
		);
		assert_err!(
			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				Default::default(),
				Default::default(),
				Default::default()
			),
			Error::<Test>::BlockedByMaintenanceMode
		);
		assert_err!(
			Rolldown::force_cancel_requests_from_l1(
				RuntimeOrigin::root(),
				consts::CHAIN,
				Default::default()
			),
			Error::<Test>::BlockedByMaintenanceMode
		);
	});
}

#[test]
#[serial]
fn test_single_sequencer_cannot_cancel_request_without_cancel_rights_in_same_block() {
	ExtBuilder::single_sequencer(BOB)
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			Rolldown::new_sequencer_active(consts::CHAIN, &BOB);

			// Arrange
			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![
					L1UpdateRequest::Deposit(Default::default()),
					L1UpdateRequest::Deposit(Default::default()),
				])
				.build();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 0u128 }
			);

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), deposit_update).unwrap();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 0u128 }
			);

			assert_err!(
				Rolldown::cancel_requests_from_l1(
					RuntimeOrigin::signed(BOB),
					consts::CHAIN,
					15u128.into()
				),
				Error::<Test>::CancelRightsExhausted
			);
		});
}

#[test]
#[serial]
fn test_single_sequencer_cannot_cancel_request_without_cancel_rights_in_next_block() {
	ExtBuilder::single_sequencer(BOB)
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			Rolldown::new_sequencer_active(consts::CHAIN, &BOB);

			// Arrange
			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			let deposit_update = L1UpdateBuilder::default()
				.with_requests(vec![
					L1UpdateRequest::Deposit(Default::default()),
					L1UpdateRequest::Deposit(Default::default()),
				])
				.build();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 0u128 }
			);

			// Act
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), deposit_update).unwrap();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&BOB).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 0u128 }
			);

			forward_to_block::<Test>(11);
			assert_err!(
				Rolldown::cancel_requests_from_l1(
					RuntimeOrigin::signed(BOB),
					consts::CHAIN,
					15u128.into()
				),
				Error::<Test>::CancelRightsExhausted
			);
		});
}

#[test]
#[serial]
fn consider_awaiting_cancel_resolutions_and_cancel_disputes_when_assigning_initial_cancel_rights_to_sequencer(
) {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			// Arrange
			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock
				.expect()
				.withf(|chain, a, b| *chain == consts::CHAIN && *a == ALICE && b.cloned() == None)
				.times(2)
				.return_const(Ok(().into()));

			// honest update
			let honest_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
				.build();

			forward_to_block::<Test>(10);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), honest_update.clone()).unwrap();
			Rolldown::cancel_requests_from_l1(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				15u128.into(),
			)
			.unwrap();

			forward_to_block::<Test>(11);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(CHARLIE), honest_update.clone())
				.unwrap();
			Rolldown::cancel_requests_from_l1(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				16u128.into(),
			)
			.unwrap();

			// lets pretned that alice misbehaved and got slashed, as a result her stake dropped below
			// active sequencer threshold and she got immadietely removed from sequencers set
			Rolldown::handle_sequencer_deactivations(consts::CHAIN, vec![ALICE]);

			// then lets pretned that alice provided more stake and got approved as active sequencer
			Rolldown::new_sequencer_active(consts::CHAIN, &ALICE);

			// resolve previous cancel disputes
			Rolldown::force_update_l2_from_l1(
				RuntimeOrigin::root(),
				L1UpdateBuilder::default()
					.with_requests(vec![
						L1UpdateRequest::CancelResolution(messages::CancelResolution {
							requestId: Default::default(),
							l2RequestId: 1u128,
							cancelJustified: false,
							timeStamp: sp_core::U256::from(1),
						}),
						L1UpdateRequest::CancelResolution(messages::CancelResolution {
							requestId: Default::default(),
							l2RequestId: 2u128,
							cancelJustified: false,
							timeStamp: sp_core::U256::from(1),
						}),
					])
					.build(),
			)
			.unwrap();

			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
		});
}

#[test]
#[serial]
fn consider_awaiting_l1_READ_update_in_dispute_period_when_assigning_initial_read_rights_to_sequencer(
) {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			// Arrange
			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock
				.expect()
				.withf(|chain, a, b| *chain == consts::CHAIN && *a == ALICE && b.cloned() == None)
				.times(1)
				.return_const(Ok(().into()));

			let honest_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
				.build();
			forward_to_block::<Test>(10);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), honest_update.clone()).unwrap();

			// accidently canceling honest update
			forward_to_block::<Test>(11);
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(ALICE), consts::CHAIN, 15u128)
				.unwrap();

			forward_to_block::<Test>(12);
			let honest_update = L1UpdateBuilder::default()
				.with_requests(vec![
					L1UpdateRequest::Deposit(Default::default()),
					L1UpdateRequest::CancelResolution(messages::CancelResolution {
						requestId: Default::default(),
						l2RequestId: 1u128,
						cancelJustified: false,
						timeStamp: sp_core::U256::from(1),
					}),
				])
				.build();
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(CHARLIE), honest_update).unwrap();

			forward_to_block::<Test>(15);
			let honest_update = L1UpdateBuilder::default()
				.with_requests(vec![
					L1UpdateRequest::Deposit(Default::default()),
					L1UpdateRequest::CancelResolution(messages::CancelResolution {
						requestId: Default::default(),
						l2RequestId: 1u128,
						cancelJustified: false,
						timeStamp: sp_core::U256::from(1),
					}),
					L1UpdateRequest::Deposit(Default::default()),
				])
				.build();
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), honest_update).unwrap();

			forward_to_block::<Test>(17);
			// at this point alice will be slashed by cancel resolution provided by CHALIE in block 12
			Rolldown::handle_sequencer_deactivations(consts::CHAIN, vec![ALICE]);
			// then lets pretned that alice provided more stake and got approved as active sequencer
			Rolldown::new_sequencer_active(consts::CHAIN, &ALICE);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 0u128, cancel_rights: 2u128 }
			);

			// at this point ALICE is sequencer again and her update provided at block 13 gets executed
			forward_to_block::<Test>(20);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
		});
}

#[test]
#[serial]
fn consider_awaiting_cancel_resolutions_and_cancel_disputes_when_assigning_initial_read_rights_to_sequencer(
) {
	ExtBuilder::new()
		.issue(ETH_RECIPIENT_ACCOUNT_MGX, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			// Arrange
			let slash_sequencer_mock = MockSequencerStakingProviderApi::slash_sequencer_context();
			slash_sequencer_mock.expect().return_const(Ok(().into()));

			// honest update
			let honest_update = L1UpdateBuilder::default()
				.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
				.build();

			forward_to_block::<Test>(10);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(BOB), honest_update.clone()).unwrap();
			Rolldown::cancel_requests_from_l1(RuntimeOrigin::signed(ALICE), consts::CHAIN, 15u128)
				.unwrap();

			forward_to_block::<Test>(15);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), honest_update.clone())
				.unwrap();
			// lets assume single person controls multiple sequencers (alice&charlie) and charlie intentionally cancels honest update
			Rolldown::cancel_requests_from_l1(
				RuntimeOrigin::signed(CHARLIE),
				consts::CHAIN,
				20u128,
			)
			.unwrap();

			// and then CHARLIE provbides honest update - as a result ALICE will be slashed
			Rolldown::update_l2_from_l1(
				RuntimeOrigin::signed(CHARLIE),
				L1UpdateBuilder::default()
					.with_requests(vec![
						L1UpdateRequest::Deposit(Default::default()),
						L1UpdateRequest::CancelResolution(messages::CancelResolution {
							requestId: Default::default(),
							l2RequestId: 1u128,
							cancelJustified: false,
							timeStamp: sp_core::U256::from(1),
						}),
					])
					.build(),
			)
			.unwrap();

			forward_to_block::<Test>(20);
			// alice is slashed for her first malicious cancel but then she got slashed with honest update but that has not been yet processed
			Rolldown::handle_sequencer_deactivations(consts::CHAIN, vec![ALICE]);

			Rolldown::update_l2_from_l1(
				RuntimeOrigin::signed(CHARLIE),
				L1UpdateBuilder::default()
					.with_requests(vec![L1UpdateRequest::CancelResolution(
						messages::CancelResolution {
							requestId: Default::default(),
							l2RequestId: 2u128,
							cancelJustified: false,
							timeStamp: sp_core::U256::from(1),
						},
					)])
					.with_offset(3u128)
					.build(),
			)
			.unwrap();

			forward_to_block::<Test>(24);
			// lets consider alice provided more stake and just got into the active set of sequencers
			Rolldown::new_sequencer_active(consts::CHAIN, &ALICE);

			forward_to_block::<Test>(25);
			assert_eq!(
				*SequencersRights::<Test>::get(consts::CHAIN).get(&ALICE).unwrap(),
				SequencerRights { read_rights: 1u128, cancel_rights: 2u128 }
			);
		});
}

#[test]
#[serial]
fn test_merkle_proof_works() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			for i in 0..500 {
				Rolldown::withdraw(
					RuntimeOrigin::signed(ALICE),
					consts::CHAIN,
					ETH_RECIPIENT_ACCOUNT,
					ETH_TOKEN_ADDRESS,
					i as u128,
				)
				.unwrap();
			}

			let range = (1u128, 300u128);
			let root_hash = Pallet::<Test>::get_merkle_root(consts::CHAIN, range);
			let proof_hashes = Pallet::<Test>::get_merkle_proof_for_tx(consts::CHAIN, range, 257);
			Pallet::<Test>::verify_merkle_proof_for_tx(
				consts::CHAIN,
				range,
				root_hash,
				257,
				proof_hashes,
			);
		});
}

#[test]
#[serial]
fn test_batch_is_created_automatically_when_l2requests_count_exceeds_MerkleRootAutomaticBatchSize()
{
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.build()
		.execute_with(|| {
			let selected_sequencer_mock =
				MockSequencerStakingProviderApi::selected_sequencer_context();
			selected_sequencer_mock.expect().return_const(Some(consts::ALICE));
			let get_l1_asset_id_mock = MockAssetRegistryProviderApi::get_l1_asset_id_context();
			get_l1_asset_id_mock.expect().return_const(crate::tests::ETH_TOKEN_ADDRESS_MGX);
			let is_maintenance_mock = MockMaintenanceStatusProviderApi::is_maintenance_context();
			is_maintenance_mock.expect().return_const(false);
			let _selected_sequencer_mock =
				MockSequencerStakingProviderApi::selected_sequencer_context();

			forward_to_block::<Test>(10);
			assert_eq!(L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN), None);

			for _ in 0..Rolldown::automatic_batch_size() - 1 {
				Rolldown::withdraw(
					RuntimeOrigin::signed(ALICE),
					consts::CHAIN,
					ETH_RECIPIENT_ACCOUNT,
					ETH_TOKEN_ADDRESS,
					1000u128,
				)
				.unwrap();
			}
			forward_to_block::<Test>(11);
			assert_eq!(L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN), None);

			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1000u128,
			)
			.unwrap();
			assert_eq!(L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN), None);

			forward_to_block::<Test>(12);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(12u64.into(), 1u128, (1, 10)))
			);

			for _ in 0..Rolldown::automatic_batch_size() - 1 {
				Rolldown::withdraw(
					RuntimeOrigin::signed(ALICE),
					consts::CHAIN,
					ETH_RECIPIENT_ACCOUNT,
					ETH_TOKEN_ADDRESS,
					1000u128,
				)
				.unwrap();
			}

			forward_to_block::<Test>(13);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(12u64.into(), 1u128, (1, 10)))
			);

			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1000u128,
			)
			.unwrap();

			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(12u64.into(), 1u128, (1, 10)))
			);
			forward_to_block::<Test>(14);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(14u64.into(), 2u128, (11, 20)))
			);
		});
}

#[test]
#[serial]
fn test_batch_is_created_automatically_when_MerkleRootAutomaticBatchPeriod_passes() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.build()
		.execute_with(|| {
			let get_l1_asset_id_mock = MockAssetRegistryProviderApi::get_l1_asset_id_context();
			get_l1_asset_id_mock.expect().return_const(crate::tests::ETH_TOKEN_ADDRESS_MGX);
			let is_maintenance_mock = MockMaintenanceStatusProviderApi::is_maintenance_context();
			is_maintenance_mock.expect().return_const(false);
			let selected_sequencer_mock =
				MockSequencerStakingProviderApi::selected_sequencer_context();
			selected_sequencer_mock.expect().return_const(Some(consts::ALICE));

			forward_to_block::<Test>(1);
			assert_eq!(L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN), None);

			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1000u128,
			)
			.unwrap();

			forward_to_block::<Test>((Rolldown::automatic_batch_period() as u64) - 1u64);
			assert_eq!(L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN), None);
			forward_to_block::<Test>(Rolldown::automatic_batch_period() as u64);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(25u64, 1u128, (1, 1)))
			);

			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1000u128,
			)
			.unwrap();

			forward_to_block::<Test>((2 * Rolldown::automatic_batch_period() as u64) - 1u64);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(25u64, 1u128, (1, 1)))
			);
			forward_to_block::<Test>(2 * Rolldown::automatic_batch_period() as u64);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(50u64, 2u128, (2, 2)))
			);

			forward_to_block::<Test>(10 * Rolldown::automatic_batch_period() as u64);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(50u64, 2u128, (2, 2)))
			);
		});
}

#[test]
#[serial]
fn test_period_based_batch_respects_sized_batches() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.build()
		.execute_with(|| {
			let selected_sequencer_mock =
				MockSequencerStakingProviderApi::selected_sequencer_context();
			selected_sequencer_mock.expect().return_const(Some(consts::ALICE));
			let get_l1_asset_id_mock = MockAssetRegistryProviderApi::get_l1_asset_id_context();
			get_l1_asset_id_mock.expect().return_const(crate::tests::ETH_TOKEN_ADDRESS_MGX);
			let is_maintenance_mock = MockMaintenanceStatusProviderApi::is_maintenance_context();
			is_maintenance_mock.expect().return_const(false);
			let _selected_sequencer_mock =
				MockSequencerStakingProviderApi::selected_sequencer_context();

			forward_to_block::<Test>(10);
			assert_eq!(L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN), None);

			for _ in 0..Rolldown::automatic_batch_size() {
				Rolldown::withdraw(
					RuntimeOrigin::signed(ALICE),
					consts::CHAIN,
					ETH_RECIPIENT_ACCOUNT,
					ETH_TOKEN_ADDRESS,
					1000u128,
				)
				.unwrap();
			}
			forward_to_block::<Test>(11);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(11u64.into(), 1u128, (1, 10)))
			);
			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1000u128,
			)
			.unwrap();

			forward_to_block::<Test>(Rolldown::automatic_batch_period() as u64);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(11u64.into(), 1u128, (1, 10)))
			);

			forward_to_block::<Test>(11 + Rolldown::automatic_batch_period() as u64);
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(36u64.into(), 2u128, (11, 11)))
			);
		});
}

#[test]
#[serial]
fn test_create_manual_batch_fails_for_wrong_range() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			assert_err!(
				Rolldown::create_batch(RuntimeOrigin::signed(ALICE), consts::CHAIN, (5, 1), None),
				Error::<Test>::InvalidRange
			);
		})
}

#[test]
#[serial]
fn test_create_manual_batch_fails_for_range_that_does_not_exists() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			assert_err!(
				Rolldown::create_batch(RuntimeOrigin::signed(ALICE), consts::CHAIN, (1, 1), None),
				Error::<Test>::NonExistingRequestId
			);
		})
}

#[test]
#[serial]
fn test_create_manual_batch_works() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);

			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1_000u128,
			)
			.unwrap();
			assert_ok!(Rolldown::create_batch(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				(1, 1),
				None
			));
			assert_event_emitted!(Event::TxBatchCreated {
				chain: consts::CHAIN,
				source: BatchSource::Manual,
				assignee: ALICE,
				batch_id: 1,
				range: (1, 1),
			});

			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1_000u128,
			)
			.unwrap();

			assert_ok!(Rolldown::create_batch(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				(1, 1),
				None
			));
			assert_event_emitted!(Event::TxBatchCreated {
				chain: consts::CHAIN,
				source: BatchSource::Manual,
				assignee: ALICE,
				batch_id: 2,
				range: (1, 1),
			});

			assert_ok!(Rolldown::create_batch(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				(1, 2),
				None
			));

			assert_event_emitted!(Event::TxBatchCreated {
				chain: consts::CHAIN,
				source: BatchSource::Manual,
				assignee: ALICE,
				batch_id: 3,
				range: (1, 2),
			});
		})
}

#[test]
#[serial]
fn test_create_manual_batch_fails_for_invalid_alias_account() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.issue(BOB, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			let selected_sequencer_mock =
				MockSequencerStakingProviderApi::is_active_sequencer_alias_context();
			selected_sequencer_mock.expect().return_const(false);

			forward_to_block::<Test>(10);

			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1_000u128,
			)
			.unwrap();

			assert_err!(
				Rolldown::create_batch(
					RuntimeOrigin::signed(BOB),
					consts::CHAIN,
					(1, 1),
					Some(ALICE)
				),
				Error::<Test>::UnknownAliasAccount
			);
		})
}

#[test]
#[serial]
fn test_create_manual_batch_work_for_alias_account() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.issue(BOB, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			let selected_sequencer_mock =
				MockSequencerStakingProviderApi::is_active_sequencer_alias_context();
			selected_sequencer_mock.expect().return_const(true);

			forward_to_block::<Test>(10);

			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1_000u128,
			)
			.unwrap();

			Rolldown::create_batch(RuntimeOrigin::signed(BOB), consts::CHAIN, (1, 1), Some(ALICE))
				.unwrap();
			assert_event_emitted!(Event::TxBatchCreated {
				chain: consts::CHAIN,
				source: BatchSource::Manual,
				assignee: ALICE,
				batch_id: 1,
				range: (1, 1),
			});
			assert_eq!(
				L2RequestsBatchLast::<Test>::get().get(&consts::CHAIN),
				Some(&(10u64.into(), 1u128, (1, 1)))
			);
		})
}

#[test]
#[serial]
fn test_merkle_proof_for_single_element_tree_is_empty() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			Rolldown::withdraw(
				RuntimeOrigin::signed(ALICE),
				consts::CHAIN,
				ETH_RECIPIENT_ACCOUNT,
				ETH_TOKEN_ADDRESS,
				1,
			)
			.unwrap();

			let range = (1u128, 1u128);
			let root_hash = Pallet::<Test>::get_merkle_root(consts::CHAIN, range);
			let proof_hashes = Pallet::<Test>::get_merkle_proof_for_tx(consts::CHAIN, range, 1);
			Pallet::<Test>::verify_merkle_proof_for_tx(
				consts::CHAIN,
				range,
				root_hash,
				1,
				proof_hashes,
			);
		});
}

#[test]
#[serial]
fn test_manual_batch_fee_update() {
	ExtBuilder::new().execute_with_default_mocks(|| {
		forward_to_block::<Test>(10);
		let fee = 12345;
		assert_eq!(ManualBatchExtraFee::<Test>::get(), 0);
		Rolldown::set_manual_batch_extra_fee(RuntimeOrigin::root(), fee).unwrap();
		assert_eq!(ManualBatchExtraFee::<Test>::get(), fee);
		assert_event_emitted!(Event::ManualBatchExtraFeeSet(fee));
	});
}

#[test]
#[serial]
fn do_not_allow_for_batches_with_gaps() {
	ExtBuilder::new()
		.issue(ALICE, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.issue(BOB, ETH_TOKEN_ADDRESS_MGX, MILLION)
		.execute_with_default_mocks(|| {
			let selected_sequencer_mock =
				MockSequencerStakingProviderApi::is_active_sequencer_alias_context();
			selected_sequencer_mock.expect().return_const(true);

			forward_to_block::<Test>(10);

			for _ in 0..10 {
				Rolldown::withdraw(
					RuntimeOrigin::signed(ALICE),
					consts::CHAIN,
					ETH_RECIPIENT_ACCOUNT,
					ETH_TOKEN_ADDRESS,
					1_000u128,
				)
				.unwrap();
			}

			Rolldown::create_batch(RuntimeOrigin::signed(BOB), consts::CHAIN, (1, 5), Some(ALICE))
				.unwrap();
			assert_err!(
				Rolldown::create_batch(
					RuntimeOrigin::signed(BOB),
					consts::CHAIN,
					(7, 10),
					Some(ALICE)
				),
				Error::<Test>::InvalidRange
			);
		})
}
