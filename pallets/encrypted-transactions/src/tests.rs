// Copyright (C) 2020 Mangata team
#![allow(non_snake_case)]

use super::*;
use crate::mock::*;
use frame_support::assert_err;
use sp_runtime::traits::BlakeTwo256;
use log::info;

#[test]
fn mat_test() {
    new_test_ext().execute_with(|| {});
}



//fn submit_doubly_encrypted_transaction(origin, doubly_encrypted_call: Vec<u8>, nonce: T::Index, weight: Weight, builder: T::AccountId, executor: T::AccountId) -> DispatchResult{
#[test]
fn W_submit_double() {
    new_test_ext().execute_with(|| {
      
        let builder: u128 = 2;
        let doubly_encrypted_call = vec![1, 2, 3];

        // let mut identifier_vec: Vec<u8> = Vec::<u8>::new();
        // identifier_vec.extend_from_slice(&doubly_encrypted_call[..]);
        // identifier_vec.extend_from_slice(&Encode::encode(&1)[..]);
        // identifier_vec.extend_from_slice(&Encode::encode(&1)[..]);
        // let identifier = BlakeTwo256::hash_of(&identifier_vec);

      
        EncryptedTX::create_new_token(&1, 1000000000000000);
        assert_eq!(EncryptedTX::balance(0, 1), 1000000000000000);

        EncryptedTX::submit_doubly_encrypted_transaction(
            Origin::signed(1),
            doubly_encrypted_call,
            1,
            1,
            2,
            3,
        )
        .unwrap();
        let expected_number_of_tx_in_queue = 1;
        assert_eq!(
            EncryptedTX::doubly_encrypted_queue(&builder).len(),
            expected_number_of_tx_in_queue
        );

        let identifier = System::events();
        for i in System::events().iter(){
            println!("{:?}", i)
        };
        println!("{:?}", identifier)
        //assert_eq!(EncryptedTX::doubly_encrypted_queue(&builder)[0], identifier);
    });
}

#[test]
fn NW_submit_double_BalanceTooLow() {
    new_test_ext().execute_with(|| {
      
     
        let doubly_encrypted_call = vec![1, 2, 3];

        
       
        // assert_err!(
        //     EncryptedTX::submit_doubly_encrypted_transaction(
        //         Origin::signed(1),
        //         doubly_encrypted_call,
        //         1,
        //         1,
        //         2,
        //         3,
        //     ),
        //     Error::<Test>::BalanceTooLow,
        // ); 
    });
}

// ensure!(doubly_encrypted_call.len() <= T::DoublyEncryptedCallMaxLength::get() as usize, Error::<T>::DoublyEncryptedCallMaxLengthExceeded);
//WHERE is the max lenght??

// T::Tokens::ensure_can_withdraw(0u8.into(), &user, fee_charged.into(), WithdrawReasons::all(), Default::default())?;

// TxnRegistry::<T>::insert(identifier, txn_registry_details);

// DONE DoublyEncryptedQueue::<T>::mutate(&builder, |vec_hash| {vec_hash.push(identifier)});
// ? TxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &user, |tree_record| tree_record.insert(identifier, (nonce, fee_charged, false)));
// ? Self::deposit_event(RawEvent::DoublyEncryptedTxnSubmitted(user, nonce, identifier));

