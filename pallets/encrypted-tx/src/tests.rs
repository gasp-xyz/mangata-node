#![cfg(not(feature = "runtime-benchmarks"))]
use super::*;
use mock::*;
use serial_test::serial;

use frame_support::{assert_err, assert_ok};
use sp_application_crypto::Pair;
use sp_core::hexdisplay::AsBytesRef;
use sp_core::crypto::CryptoTypePublicPair;
use sp_core::sr25519;
use sp_runtime::traits::BadOrigin;
use frame_support::weights::Weight;
use sp_keystore::vrf::{VRFTranscriptData, VRFTranscriptValue};
use sp_keystore::SyncCryptoStore;


const ALICE: u128 = 1;
const BUILDER: u128 = 100;
const EXECUTOR: u128 = 101;
const MILLION: Balance = 1_000_000;
const ZERO_WEIGHT: Weight = Weight::from_ref_time(0);

fn encrypt_data(input: &[u8]) -> (Vec<u8>, Vec<u8>) {
	let secret_uri = "//Alice";
	let key_type = sp_core::crypto::KeyTypeId(*b"aura");
	let key_pair = sp_core::sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");
	let keystore = sp_keystore::testing::KeyStore::new();
	keystore
		.insert_unknown(key_type, secret_uri, key_pair.public().as_ref())
		.unwrap();

	let transcript = VRFTranscriptData {
		label: b"ved",
		items: vec![("input", VRFTranscriptValue::Bytes(input.to_vec()))],
	};
	let pub_key = CryptoTypePublicPair(sr25519::CRYPTO_ID, key_pair.public().to_vec());

	let singly_encrypted_call = keystore.sign_with( key_type, &pub_key, input).unwrap().unwrap();
	let doubly_encrypted_call = keystore.sign_with( key_type, &pub_key, &singly_encrypted_call).unwrap().unwrap();

	(doubly_encrypted_call, singly_encrypted_call)
}

fn hash(input: &Vec<u8>) -> <Test as frame_system::Config>::Hash {
	<Test as frame_system::Config>::Hashing::hash(input) 
}


#[test]
fn test_submit_double_encrypted_tx() {
	ExtBuilder::new()
	.build()
	.execute_with(|| {

		assert!(DoublyEncryptedQueue::<Test>::try_get(ALICE).is_err());
		let cnt = UniqueId::<Test>::get();

		let input = b"dummy_data".to_vec();
		let (doubly_enc,singly_enc) = encrypt_data(&input);
		let hashes = (hash(&singly_enc), hash(&input));

		let identifier = EncryptedTx::calculate_unique_id(&ALICE, cnt, &doubly_enc);
		EncryptedTx::submit_doubly_encrypted_transaction( RuntimeOrigin::signed(ALICE),
			doubly_enc.clone(),
			hashes,
			0,
			ZERO_WEIGHT,
			BUILDER,
			EXECUTOR
		).unwrap();


		let doubly_encrypted_txs = DoublyEncryptedQueue::<Test>::try_get(BUILDER).expect("dummy_call is stored");
		assert_eq!(doubly_encrypted_txs, vec![(Encryption::Double, identifier)]);
		
		let registry_info = TxnRegistry::<Test>::get(identifier).expect("data in registry");

		assert_eq!(
		TxnRegistryDetails{
			doubly_encrypted_call: doubly_enc,
			singly_encrypted_call_signature: hash(&singly_enc),
			call_signature: hash(&input),
			user: ALICE,
			weight: ZERO_WEIGHT,
			builder: BUILDER,
			executor: EXECUTOR,
			singly_encrypted_call: None,
			decrypted_call: None,
		}, registry_info);

	});
}

#[test]
fn test_submit_double_encrypted_tx_multiple_times() {
	ExtBuilder::new()
	.build()
	.execute_with(|| {
		for _ in 1..10 {
			let input = b"dummy_data".to_vec();
			let (doubly_enc,singly_enc) = encrypt_data(&input);
			let hashes = (hash(&singly_enc), hash(&input));

			EncryptedTx::submit_doubly_encrypted_transaction( RuntimeOrigin::signed(ALICE),
				doubly_enc,
				hashes,
				0,
				ZERO_WEIGHT,
				BUILDER,
				EXECUTOR
			).unwrap();
		}
	});
}

#[test]
fn test_cannot_submit_tx_with_not_enought_tokens_to_pay_declared_fee() {
	ExtBuilder::new()
	.create_token(NativeCurrencyId::get())
	.build()
	.execute_with(|| {
		let fee = 100_u128;

		assert!(fee > OrmlTokens::accounts(ALICE, NativeCurrencyId::get()).free);

		let input = b"dummy_data".to_vec();
		let (doubly_enc,singly_enc) = encrypt_data(&input);
		let hashes = (hash(&singly_enc), hash(&input));

		assert_err!(
			EncryptedTx::submit_doubly_encrypted_transaction(
				RuntimeOrigin::signed(ALICE),
				doubly_enc,
				hashes,
				fee,
				ZERO_WEIGHT,
				BUILDER,
				EXECUTOR)
			,
			Error::<Test>::NotEnoughtBalance
		);
	});
}

#[test]
fn test_submit_encrypted_call_error_because_of_unknown_tx() {
	ExtBuilder::new()
	.create_token(NativeCurrencyId::get())
	.build()
	.execute_with(|| {
		let dummy_call = b"dummy_data".to_vec();
		let identifier = EncryptedTx::calculate_unique_id(&ALICE, UniqueId::<Test>::get(), &dummy_call);
		assert_err!(
		EncryptedTx::submit_singly_encrypted_transaction(
			RuntimeOrigin::signed(ALICE),
			identifier,
			b"dummy data".to_vec(),
		),
		Error::<Test>::TxnDoesNotExistsInRegistry
		);

	});
}

#[test]
fn test_submit_encrypted_call_error_because_of_bad_account() {
	ExtBuilder::new()
	.create_token(NativeCurrencyId::get())
	.build()
	.execute_with(|| {
		let input = b"dummy_data".to_vec();
		let (doubly_enc,singly_enc) = encrypt_data(&input);
		let hashes = (hash(&singly_enc), hash(&input));
		let identifier = EncryptedTx::calculate_unique_id(&ALICE, UniqueId::<Test>::get(), &doubly_enc);

		EncryptedTx::submit_doubly_encrypted_transaction(
			RuntimeOrigin::signed(ALICE),
			doubly_enc,
			hashes,
			0,
			ZERO_WEIGHT,
			BUILDER,
			EXECUTOR).unwrap();


		assert_err!(
		EncryptedTx::submit_singly_encrypted_transaction(
			RuntimeOrigin::signed(ALICE),
			identifier,
			b"dummy data".to_vec(),
		),
		Error::<Test>::WrongAccount
		);
	});
}

#[test]
fn test_submit_encrypted_call_error_because_of_bad_proof() {
	ExtBuilder::new()
	.create_token(NativeCurrencyId::get())
	.build()
	.execute_with(|| {
		let input = b"dummy_data".to_vec();
		let (doubly_enc,singly_enc) = encrypt_data(&input);
		let hashes = (hash(&singly_enc), hash(&input));
		let identifier = EncryptedTx::calculate_unique_id(&ALICE, UniqueId::<Test>::get(), &doubly_enc);

		EncryptedTx::submit_doubly_encrypted_transaction(
			RuntimeOrigin::signed(ALICE),
			doubly_enc,
			hashes,
			0,
			ZERO_WEIGHT,
			BUILDER,
			EXECUTOR).unwrap();

		assert_err!(
		EncryptedTx::submit_singly_encrypted_transaction(
			RuntimeOrigin::signed(BUILDER),
			identifier,
			b"incorrectly decoded message".to_vec(),
		),
		Error::<Test>::ProofError
		);
	});
}

