#![cfg(not(feature = "runtime-benchmarks"))]
use super::*;
use mock::*;
use serial_test::serial;

use frame_support::{assert_err, assert_ok};
use sp_runtime::traits::BadOrigin;
use frame_support::weights::Weight;

const ALICE: u128 = 1;
const BUILDER: u128 = 100;
const EXECUTOR: u128 = 101;
const MILLION: Balance = 1_000_000;

#[test]
#[serial]
fn test_submit_double_encrypted_tx() {
	ExtBuilder::new()
	.build()
	.execute_with(|| {
		EncryptedTx::submit_doubly_encrypted_transaction( RuntimeOrigin::signed(ALICE),
			RuntimeCall::EncryptedTx(crate::Call::dummy_tx { data: Default::default() }).encode(),
			0,
			Weight::from_ref_time(0u64),
			BUILDER,
			EXECUTOR
		).unwrap();
	});
}

#[test]
#[serial]
fn test_submit_double_encrypted_tx_multiple_times() {
	ExtBuilder::new()
	.build()
	.execute_with(|| {
		for _ in 1..10 {
			EncryptedTx::submit_doubly_encrypted_transaction( RuntimeOrigin::signed(ALICE),
				RuntimeCall::EncryptedTx(crate::Call::dummy_tx { data: Default::default() }).encode(),
				0,
				Weight::from_ref_time(0u64),
				BUILDER,
				EXECUTOR
			).unwrap();
		}
	});
}

#[test]
#[serial]
fn test_cannot_submit_tx_with_not_enought_tokens_to_pay_declared_fee() {
	ExtBuilder::new()
	.create_token(NativeCurrencyId::get())
	.build()
	.execute_with(|| {
		let fee = 100_u128;

		assert!(fee > OrmlTokens::accounts(ALICE, NativeCurrencyId::get()).free);

		assert_err!(
			EncryptedTx::submit_doubly_encrypted_transaction(
				RuntimeOrigin::signed(ALICE),
				RuntimeCall::EncryptedTx(crate::Call::dummy_tx { data: Default::default() }).encode(),
				100,
				Weight::from_ref_time(0u64),
				BUILDER,
				EXECUTOR)
			,
			Error::<Test>::NotEnoughtBalance
		);
	});
}
