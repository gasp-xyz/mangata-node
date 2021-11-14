// Copyright (C) 2020 Mangata team
#![allow(non_snake_case)]

use super::*;
use crate::mock::*;
use frame_support::assert_err;
use sp_runtime::traits::BlakeTwo256;

#[test]
fn mat_test() {
    new_test_ext().execute_with(|| {});
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
fn W_submit_double() {
    new_test_ext().execute_with(|| {
        let mut user = ensure_signed(Origin::signed(1));
        let builder: u128 = 2;
        let doubly_encrypted_call = vec![1, 2, 3];
        let mut identifier_vec: Vec<u8> = Vec::<u8>::new();

        identifier_vec.extend_from_slice(&doubly_encrypted_call[..]);
        identifier_vec.extend_from_slice(&Encode::encode(&1)[..]);
        identifier_vec.extend_from_slice(&Encode::encode(&1)[..]);

        let identifier = BlakeTwo256::hash_of(&identifier_vec);

        let user: u128 = 1;
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
        //assert_eq!(EncryptedTX::doubly_encrypted_queue(&builder)[0], identifier);
    });
}
