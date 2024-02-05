use frame_support::{assert_err, assert_ok};
use sp_core::{crypto::Ss58Codec, hexdisplay::HexDisplay};

use crate::*;
use crate::mock::*;
use crate::mock::consts::*;
use eth_api::{L1Update, L1UpdateRequest};

fn create_l1_update(requests: Vec<L1UpdateRequest>) -> eth_api::L1Update{
	let mut update = L1Update::default();
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
fn parse_ethereum_address_from_20bytes_hex_string() {
	assert_ok!(EthereumAddressConverter::try_convert(
		"0x0000000000000000000000000000000000000000".to_string()
	));
	assert_ok!(EthereumAddressConverter::try_convert(
		"0xbd0f320b5343c5d52f18b85dd1a1d0f6844fbb1a".to_string()
	));
}

#[test]
fn parse_ethereum_address_from_20bytes_hex_string_without_prefix() {
	assert_ok!(EthereumAddressConverter::try_convert(
		"0000000000000000000000000000000000000000".to_string()
	));
	assert_ok!(EthereumAddressConverter::try_convert(
		"bd0f320b5343c5d52f18b85dd1a1d0f6844fbb1a".to_string()
	));
}

#[test]
fn parse_ethereum_address_from_20bytes_hex_string_without_and_without_capital_leters() {
	assert_ok!(EthereumAddressConverter::try_convert(
		"0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string()
	));
	assert_ok!(EthereumAddressConverter::try_convert(
		"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()
	));
}

#[test]
fn parse_ethereum_address_from_too_short_string_fails() {
	assert!(EthereumAddressConverter::try_convert(
		"0x000000000000000000000000000000000000000".to_string()
	)
	.is_err());
}


#[test]
fn parse_ethereum_address_from_too_long_string_fails() {
	assert!(EthereumAddressConverter::try_convert(
		"0x000000000000000000000000000000000000000000".to_string()
	)
	.is_err());
}



#[test]
fn error_on_empty_update() {
	ExtBuilder::new()
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(36);
			let update = create_l1_update(vec![]);
			assert_err!(
				Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update),
				Error::<Test>::EmptyUpdate
			);
		});
}

#[test]
#[ignore]
fn process_single_deposit() {
	ExtBuilder::new()
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(36);
			let update = create_l1_update(vec![L1UpdateRequest::Deposit(Default::default())]);
				Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();

			todo!("implement check for hash of deposited update");
		});
}

#[test]
fn create_pending_update_after_dispute_period() {
	ExtBuilder::new()
		.execute_with_default_mocks(|| {
			forward_to_block::<Test>(10);
			let update = create_l1_update(vec![L1UpdateRequest::Deposit(Default::default())]);
			Rolldown::update_l2_from_l1(RuntimeOrigin::signed(ALICE), update).unwrap();
			assert_eq!(PENDING_UPDATES_MAT::<Test>::iter().next(), None);
			forward_to_block::<Test>(21);
			PENDING_UPDATES_MAT::<Test>::iter().next().unwrap();
		});
}
