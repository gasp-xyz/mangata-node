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
const ZERO_WEIGHT: Weight = Weight::from_ref_time(0);

#[test]
#[serial]
fn test_submit_double_encrypted_tx() {
	ExtBuilder::new()
	.build()
	.execute_with(|| {

		assert!(DoublyEncryptedQueue::<Test>::try_get(ALICE).is_err());
		let cnt = UniqueId::<Test>::get();

		let dummy_call = b"dummy data".to_vec();
		EncryptedTx::submit_doubly_encrypted_transaction( RuntimeOrigin::signed(ALICE),
			dummy_call.clone(),
			0,
			ZERO_WEIGHT,
			BUILDER,
			EXECUTOR
		).unwrap();


		let identifier = EncryptedTx::calculate_unique_id(&ALICE, cnt, &dummy_call);
		let doubly_encrypted_txs = DoublyEncryptedQueue::<Test>::try_get(BUILDER).expect("dummy_call is stored");
		assert_eq!(doubly_encrypted_txs, vec![(Encryption::Double, identifier)]);

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
				b"dummy data".to_vec(),
				0,
				ZERO_WEIGHT,
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
				b"dummy data".to_vec(),
				fee,
				ZERO_WEIGHT,
				BUILDER,
				EXECUTOR)
			,
			Error::<Test>::NotEnoughtBalance
		);
	});
}
//
// #[test]
// #[serial]
// fn test_cannot_submit_tx_with_not_enought_tokens_to_pay_declared_fee() {
// 	ExtBuilder::new()
// 	.create_token(NativeCurrencyId::get())
// 	.build()
// 	.execute_with(|| {
// 		let fee = 100_u128;
//
// 		assert!(fee > OrmlTokens::accounts(ALICE, NativeCurrencyId::get()).free);
//
// 		assert_err!(
// 			EncryptedTx::submit_doubly_encrypted_transaction(
// 				RuntimeOrigin::signed(ALICE),
// 				b"dummy data".to_vec(),
// 				fee,
// 				ZERO_WEIGHT,
// 				BUILDER,
// 				EXECUTOR)
// 			,
// 			Error::<Test>::NotEnoughtBalance
// 		);
// 	});
// }
