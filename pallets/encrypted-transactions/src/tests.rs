// Copyright (C) 2020 Mangata team
#![allow(non_snake_case)]

use super::*;
use crate::mock::*;
use frame_support::assert_err;
use sp_runtime::traits::BlakeTwo256;

#[test]
fn mat_test() {
    new_test_ext().execute_with(|| {
    });
}

// #[test]
    #[test]
fn W_submit_double(){
    new_test_ext().execute_with(|| {
        let user = ensure_signed(Origin::signed(1));
        let builder: u128 = 2;
        let doubly_encrypted_call = vec![1, 2, 3];
        let mut identifier_vec: Vec<u8> = Vec::<u8>::new();
        let nonce = 1;
            identifier_vec.extend_from_slice(&doubly_encrypted_call[..]);
            identifier_vec.extend_from_slice(&Encode::encode(&user)[..]);
            identifier_vec.extend_from_slice(&Encode::encode(&nonce)[..]);

          let identifier = BlakeTwo256::hash_of(&identifier_vec);
          // let identifier = Hashing::hash(&identifier_vec[..]);
           // TxnRegistry::<T>::insert(identifier, txn_registry_details);
           // DoublyEncryptedQueue::<T>::mutate(&builder, |vec_hash| {vec_hash.push(identifier)});
           // TxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &user, |tree_record| tree_record.insert(identifier, (nonce, fee_charged, false)));
           // Self::deposit_event(RawEvent::DoublyEncryptedTxnSubmitted(user, nonce, identifier));


        
        EncryptedTX::submit_doubly_encrypted_transaction(Origin::signed(1),doubly_encrypted_call,1,1,2,3).unwrap();
        let expected_number_of_tx_in_queue = 1;
        assert_eq!(EncryptedTX::doubly_encrypted_queue(&builder).len(), expected_number_of_tx_in_queue);
        assert_eq!(EncryptedTX::doubly_encrypted_queue(&builder)[0], identifier);
    });
}    
