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
// fn set_info_should_work() {
//     new_test_ext().execute_with(|| {
//         // creating asset with assetId 0 and minting to accountId 2
//         let acc_id: u128 = 2;
//         let amount: u128 = 1000000000000000000000;
//         XykStorage::create_new_token(&acc_id, amount);
//         XykStorage::create_new_token(&acc_id, amount);

//         XykStorage::create_pool(
//             Origin::signed(2),
//             0,
//             40000000000000000000,
//             1,
//             60000000000000000000,
//         )
//         .unwrap();

//         assert_eq!(
//             <assets_info::Module<Test>>::get_info(2u32),
//             assets_info::AssetInfo {
//                 name: Some(b"LiquidityPoolToken0x00000002".to_vec()),
//                 symbol: Some(b"TKN0x00000000-TKN0x00000001".to_vec()),
//                 description: Some(b"Generated Info for Liquidity Pool Token".to_vec()),
//                 decimals: Some(18u32),
//             }
//         );
//     });
// }

//fn submit_doubly_encrypted_transaction(origin, doubly_encrypted_call: Vec<u8>, nonce: T::Index, weight: Weight, builder: T::AccountId, executor: T::AccountId) -> DispatchResult{
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

          let identifier = BlakeTwo256::hash_of(&identifier_vec[..]);
          let identifier: T::Hash = Hashing::hash(&identifier_vec[..]);
           TxnRegistry::<T>::insert(identifier, txn_registry_details);
           DoublyEncryptedQueue::<T>::mutate(&builder, |vec_hash| {vec_hash.push(identifier)});
           TxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &user, |tree_record| tree_record.insert(identifier, (nonce, fee_charged, false)));
           Self::deposit_event(RawEvent::DoublyEncryptedTxnSubmitted(user, nonce, identifier));


        
        EncryptedTX::submit_doubly_encrypted_transaction(Origin::signed(1),doubly_encrypted_call,1,1,2,3);
        assert_eq!(EncryptedTX::doubly_encrypted_queue(&builder), 833266599933266);
    });
}    