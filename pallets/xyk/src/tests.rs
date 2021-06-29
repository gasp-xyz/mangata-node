// Copyright (C) 2020 Mangata team
#![allow(non_snake_case)]

use super::*;
use crate::mock::*;
use frame_support::assert_err;

//fn create_pool_W(): create_pool working assert (maps,acocounts values)  //DONE
//fn create_pool_N_already_exists(): create_pool not working if pool already exists  //DONE
//fn create_pool_N_already_exists_other_way(): create_pool not working if pool already exists other way around (create pool X-Y, but pool Y-X exists) //DONE
//fn create_pool_N_not_enough_first_asset(): create_pool not working if account has not enough first asset for initial mint //DONE
//fn create_pool_N_not_enough_second_asset(): create_pool not working if account has not enough second asset for initial mint //DONE
//fn create_pool_N_same_asset(): create_pool not working if creating pool with same asset on both sides //DONE
//fn create_pool_N_zero_first_amount(): create_pool not working if creating with 0 amount of first asset
//fn create_pool_N_zero_second_amount(): create_pool not working if creating with 0 amount of first asset

//fn sell_W(): sell working assert (maps,acocounts values) //DONE
//fn sell_W_other_way(): sell working if sell order in different order as pool (sell pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn sell_N_not_enough_selling_assset(): sell not working if not enough asset to sell //DONE
//fn sell_N_no_such_pool(): sell not working if pool does not exist //DONE
//fn sell_N_insufficient_output_amount(): sell not working if insufficient_output_amount //DONE
//fn sell_N_zero_amount(): sell not working if trying to sell 0 asset

//fn buy_W(): buy working assert (maps,acocounts values) //DONE
//fn buy_W_other_way(): buy working if buy order in different order as pool (buy pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn buy_N_not_enough_selling_assset(): buy not working if not enough asset to sell //DONE
//fn buy_N_not_enough_reserve(): buy not working if not enough liquidity in pool //DONE
//fn buy_N_no_such_pool(): buy not working if pool does not exist //DONE
//fn buy_N_insufficient_input_amount(): sell not working if insufficient_output_amount
//fn buy_N_zero_amount(): buy not working if trying to buy 0 asset

//fn mint_W(): mint working assert (maps,acocounts values) //DONE
//fn mint_W_other_way(): mint working if mint order in different order as pool (mint pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn mint_N_no_such_pool(): mint not working if pool does not exist //DONE
//fn mint_N_not_enough_first_asset(): mint not working, not enough first asset to mint with //DONE
//fn mint_N_not_enough_second_asset(): mint not working, not enough second asset to mint with //DONE
//fn mint_N_second_token_amount_exceeded_expectations:  mint not working, required more second token amount then expected // DONE
//fn mint_W_no_expected_argument:  mint works when providing only 3 arguments // DONE
//fn min_N_zero_amount(): mint not working if trying to mint 0 asset

//fn burn_W(): burn working assert (maps,acocounts values) //DONE
//fn burn_W_other_way(): burn working if burn order in different order as pool (burn pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn burn_N_no_such_pool(): burn not working if pool does not exist //DONE
//fn burn_N_not_enough_liquidity_asset(): burn not enough liquidity asset in liquidity pool to burn //DONE
//fn burn_N_zero_amount(): burn not working if trying to burn 0 asset

//liquidity assets after trade, after burn, after mint

// pub trait Trait: frame_system::Trait {
//     // TODO: Add other types and constants required configure this module.
//     // type Hashing = BlakeTwo256;

//     // The overarching event type.
//     type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
// }

// W - should work
// N - should not work

fn initialize() {
    // creating asset with assetId 0 and minting to accountId 2
    System::set_block_number(1);
    let acc_id: u64 = 2;
    let amount: u128 = 1000000000000000000000;
    XykStorage::create_new_token(&acc_id, amount);
    XykStorage::create_new_token(&acc_id, amount);

    XykStorage::create_pool(
        Origin::signed(2),
        0,
        40000000000000000000,
        1,
        60000000000000000000,
    )
    .unwrap();

    let pool_created_event = TestEvent::xyk(Event::<Test>::PoolCreated(
        acc_id,
        0,
        40000000000000000000,
        1,
        60000000000000000000,
    ));

    assert!(System::events()
        .iter()
        .any(|record| record.event == pool_created_event));
}

fn initialize_buy_and_burn() {
    // creating asset with assetId 0 and minting to accountId 2
    let acc_id: u64 = 2;
    let amount: u128 = 1000000000000000;

    XykStorage::create_new_token(&acc_id, amount);
    XykStorage::create_new_token(&acc_id, amount);
    XykStorage::create_new_token(&acc_id, amount);
    XykStorage::create_new_token(&acc_id, amount);
    XykStorage::create_new_token(&acc_id, amount);

    XykStorage::create_pool(Origin::signed(2), 0, 100000000000000, 1, 100000000000000).unwrap();

    XykStorage::create_pool(Origin::signed(2), 0, 100000000000000, 2, 100000000000000).unwrap();

    XykStorage::create_pool(Origin::signed(2), 1, 100000000000000, 3, 100000000000000).unwrap();

    XykStorage::create_pool(Origin::signed(2), 3, 100000000000000, 4, 100000000000000).unwrap();

    XykStorage::create_pool(Origin::signed(2), 1, 100000000000000, 2, 100000000000000).unwrap();
}

#[test]
fn set_info_should_work() {
    new_test_ext().execute_with(|| {
        // creating asset with assetId 0 and minting to accountId 2
        let acc_id: u64 = 2;
        let amount: u128 = 1000000000000000000000;
        XykStorage::create_new_token(&acc_id, amount);
        XykStorage::create_new_token(&acc_id, amount);

        XykStorage::create_pool(
            Origin::signed(2),
            0,
            40000000000000000000,
            1,
            60000000000000000000,
        )
        .unwrap();

        assert_eq!(
            <assets_info::Module<Test>>::get_info(2u32),
            assets_info::AssetInfo {
                name: Some(b"LiquidityPoolToken0x00000002".to_vec()),
                symbol: Some(b"TKN0x00000000-TKN0x00000001".to_vec()),
                description: Some(b"Generated Info for Liquidity Pool Token".to_vec()),
                decimals: Some(18u32),
            }
        );
    });
}

#[test]
fn buy_and_burn_sell_mangata() {
    new_test_ext().execute_with(|| {
        initialize_buy_and_burn();
        XykStorage::sell_asset(Origin::signed(2), 0, 1, 50000000000000, 0).unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 149950000000000); // pool: regular trade result - 25000000000 treasury - 25000000000 burn
        assert_eq!(XykStorage::asset_pool((1, 0)), 66733400066734); // pool: regular trade result
        assert_eq!(XykStorage::balance(0, 2), 750000000000000); // user acc: regular trade result
        assert_eq!(XykStorage::balance(1, 2), 733266599933266); // user acc: regular trade result
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            249975000000000
        ); // vault: pool (0-1) + pool (0-2) + treasury (5)
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            266733400066734
        ); // vault: pool (0-1) + pool (1-2) + pool (1-3) - regular trade result
        assert_eq!(XykStorage::treasury(0), 25000000000); // 25000000000 mangata in treasury
        assert_eq!(XykStorage::treasury(1), 0);
        assert_eq!(XykStorage::treasury_burn(0), 0);
        assert_eq!(XykStorage::treasury_burn(1), 0);
    });
}

#[test]
fn buy_and_burn_sell_other_for_mangata() {
    new_test_ext().execute_with(|| {
        initialize_buy_and_burn();
        XykStorage::sell_asset(Origin::signed(2), 1, 0, 50000000000000, 0).unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 66711155600046); // pool: regular trade result - 11122233344 treasury 11122233344 burn
        assert_eq!(XykStorage::asset_pool((1, 0)), 150000000000000); // pool: regular trade result
        assert_eq!(XykStorage::balance(0, 2), 833266599933266); // user acc: regular trade result
        assert_eq!(XykStorage::balance(1, 2), 650000000000000); // user acc: regular trade result
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            166722277833390
        ); // vault: pool (0-1) + pool (0-2) + 11122233344 treasury
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            350000000000000
        ); // vault: regular trade result
        assert_eq!(XykStorage::treasury(0), 11122233344); // 11122233344 mangata in treasury
        assert_eq!(XykStorage::treasury(1), 0);
        assert_eq!(XykStorage::treasury_burn(0), 0);
        assert_eq!(XykStorage::treasury_burn(1), 0);
    });
}

#[test]
fn buy_and_burn_sell_only_sold_has_mangata_pair() {
    new_test_ext().execute_with(|| {
        initialize_buy_and_burn();
        XykStorage::sell_asset(Origin::signed(2), 1, 3, 50000000000000, 0).unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 99950012496876); // pool: regular trade result - 24993751562 treasury - 24993751562 burn
        assert_eq!(XykStorage::asset_pool((1, 0)), 100050000000000); // pool: regular trade result + 25000000000 treasury + 25000000000 burn / swapped for 24993751562 in pool (0-1)
        assert_eq!(XykStorage::asset_pool((1, 3)), 149950000000000); // pool: regular trade result - 25000000000 treasury - 25000000000 burn
        assert_eq!(XykStorage::asset_pool((3, 1)), 66733400066734); // pool: regular trade result
        assert_eq!(XykStorage::balance(1, 2), 650000000000000); // user acc: regular trade result
        assert_eq!(XykStorage::balance(3, 2), 833266599933266); // user acc: regular trade result
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            199975006248438
        ); // vault:  pool (0-1) + pool (0-2) + 24993751562 treasury
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            350000000000000
        ); // vault: - regular trade result
        assert_eq!(
            XykStorage::balance(3, XykStorage::account_id()),
            166733400066734
        ); // vault: - regular trade result
        assert_eq!(XykStorage::treasury(0), 24993751562); // 24993751562 mangata in treasury
        assert_eq!(XykStorage::treasury(1), 0);
        assert_eq!(XykStorage::treasury_burn(0), 0);
        assert_eq!(XykStorage::treasury_burn(1), 0);
    });
}

#[test]
fn buy_and_burn_sell_only_bought_has_mangata_pair() {
    new_test_ext().execute_with(|| {
        initialize_buy_and_burn();
        XykStorage::sell_asset(Origin::signed(2), 3, 1, 50000000000000, 0).unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 99977758007120); // pool: regular trade result - 11120996440 treasury - 11120996440 burn
        assert_eq!(XykStorage::asset_pool((1, 0)), 100022244466688); // pool: regular trade result + 11122233344 treasury + 11122233344 burn / swapped for 11120996440 in pool (0-1)
        assert_eq!(XykStorage::asset_pool((1, 3)), 66711155600046); // pool: regular trade result - 11122233344 treasury - 11122233344 burn
        assert_eq!(XykStorage::asset_pool((3, 1)), 150000000000000); // pool regular trade result
        assert_eq!(XykStorage::balance(1, 2), 733266599933266); // user acc: regular trade result
        assert_eq!(XykStorage::balance(3, 2), 750000000000000); // user acc: regular trade result
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            199988879003560
        ); // vault:  pool (0-1) + pool (0-2) + 11120996440 treasury
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            266733400066734
        ); // vault - pool (0-1) + pool (1-3)
        assert_eq!(
            XykStorage::balance(3, XykStorage::account_id()),
            250000000000000
        ); // vault - regular trade result
        assert_eq!(XykStorage::treasury(0), 11120996440); // 11120996440 mangata in treasury
        assert_eq!(XykStorage::treasury(1), 0);
        assert_eq!(XykStorage::treasury_burn(0), 0);
        assert_eq!(XykStorage::treasury_burn(1), 0);
    });
}

#[test]
fn buy_and_burn_sell_both_have_mangata_pair() {
    new_test_ext().execute_with(|| {
        initialize_buy_and_burn();
        XykStorage::sell_asset(Origin::signed(2), 2, 1, 50000000000000, 0).unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 99977758007120); // pool: regular trade result - 11120996440 treasury - 11120996440 burn
        assert_eq!(XykStorage::asset_pool((1, 0)), 100022244466688); // pool: regular trade result + 11122233344 treasury + 11122233344 burn / swapped for 11120996440 in pool (0-1)
        assert_eq!(XykStorage::asset_pool((1, 2)), 66711155600046); // pool: regular trade result - 11122233344 treasury - 11122233344 burn
        assert_eq!(XykStorage::asset_pool((2, 1)), 150000000000000); // pool regular trade result
        assert_eq!(XykStorage::balance(1, 2), 733266599933266); // user acc: regular trade result
        assert_eq!(XykStorage::balance(2, 2), 750000000000000); // user acc: regular trade result
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            199988879003560
        ); // vault:  pool (0-1) + pool (0-2) + 11120996440 treasury
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            266733400066734
        ); // vault - pool (0-1) + pool (1-3)
        assert_eq!(
            XykStorage::balance(2, XykStorage::account_id()),
            250000000000000
        ); // vault - regular trade result
        assert_eq!(XykStorage::treasury(0), 11120996440); // 11120996440 mangata in treasury
        assert_eq!(XykStorage::treasury(1), 0);
        assert_eq!(XykStorage::treasury_burn(0), 0);
        assert_eq!(XykStorage::treasury_burn(1), 0);
    });
}

#[test]
fn buy_and_burn_sell_none_have_mangata_pair() {
    new_test_ext().execute_with(|| {
        initialize_buy_and_burn();
        XykStorage::sell_asset(Origin::signed(2), 3, 4, 50000000000000, 0).unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 100000000000000); // pool - 3 treasury - 3 burn
        assert_eq!(XykStorage::asset_pool((1, 0)), 100000000000000); // pool + 4 treasury + 4 burn / swapped for - 4 - 4 in pool (0-1)
        assert_eq!(XykStorage::asset_pool((3, 4)), 150000000000000); // pool - 4 treasury - 4 burn
        assert_eq!(XykStorage::asset_pool((4, 3)), 66711155600046); // pool regular trade result
        assert_eq!(XykStorage::balance(3, 2), 750000000000000); // user acc - regular trade result
        assert_eq!(XykStorage::balance(4, 2), 933266599933266); // user acc - regular trade result
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            200000000000000
        ); // vault - pool (0-1), pool (0-2), treasury (4)
        assert_eq!(
            XykStorage::balance(3, XykStorage::account_id()),
            250000000000000
        ); // vault - regular trade result
        assert_eq!(
            XykStorage::balance(4, XykStorage::account_id()),
            66733400066734
        ); // vault - regular trade result
        assert_eq!(XykStorage::treasury(0), 0);
        assert_eq!(XykStorage::treasury(4), 11122233344); // 11122233344 token 4 in treasury
        assert_eq!(XykStorage::treasury_burn(0), 0);
        assert_eq!(XykStorage::treasury_burn(4), 11122233344); // 11122233344 token 4 in burn
    });
}

#[test]
fn multi() {
    new_test_ext().execute_with(|| {
        let acc_id: u64 = 2;
        let amount: u128 = 2000000000000000000000000;

        XykStorage::create_new_token(&acc_id, amount);
        XykStorage::create_new_token(&acc_id, amount);
        XykStorage::create_pool(
            Origin::signed(2),
            0,
            1000000000000000000000000,
            1,
            500000000000000000000000,
        )
        .unwrap();
        assert_eq!(XykStorage::asset_pool((0, 1)), 1000000000000000000000000); // amount of asset 0 in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 500000000000000000000000); // amount of asset 1 in pool map
        assert_eq!(XykStorage::liquidity_asset((0, 1)), Some(2)); // liquidity assetId corresponding to newly created pool
        assert_eq!(XykStorage::liquidity_pool(2), Some((0, 1))); // liquidity assetId corresponding to newly created pool
        assert_eq!(XykStorage::total_supply(2), 1500000000000000000000000); // total liquidity assets
        assert_eq!(XykStorage::balance(2, 2), 1500000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
        assert_eq!(XykStorage::balance(0, 2), 1000000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
        assert_eq!(XykStorage::balance(1, 2), 1500000000000000000000000); // amount of asset 1 in user acc after creating pool / initial minting
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            1000000000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            500000000000000000000000
        ); // amount of asset 1 in vault acc after creating pool

        XykStorage::mint_liquidity(
            Origin::signed(2),
            0,
            1,
            500000000000000000000000,
            5000000000000000000000000,
        )
        .unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 1500000000000000000000000); // amount of asset 0 in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 750000000000000000000001); // amount of asset 1 in pool map
        assert_eq!(XykStorage::total_supply(2), 2250000000000000000000000); // total liquidity assets
        assert_eq!(XykStorage::balance(2, 2), 2250000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
        assert_eq!(XykStorage::balance(0, 2), 500000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
        assert_eq!(XykStorage::balance(1, 2), 1249999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            1500000000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            750000000000000000000001
        ); // amount of asset 1 in vault acc after creating pool

        XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 450000000000000000000000).unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 1200000000000000000000000); // amount of asset 0 in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 600000000000000000000001); // amount of asset 1 in pool map
        assert_eq!(XykStorage::total_supply(2), 1800000000000000000000000); // total liquidity assets
        assert_eq!(XykStorage::balance(2, 2), 1800000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
        assert_eq!(XykStorage::balance(0, 2), 800000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
        assert_eq!(XykStorage::balance(1, 2), 1399999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            1200000000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            600000000000000000000001
        ); // amount of asset 1 in vault acc after creating pool

        XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 450000000000000000000000).unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 900000000000000000000000); // amount of asset 0 in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 450000000000000000000001); // amount of asset 1 in pool map
        assert_eq!(XykStorage::total_supply(2), 1350000000000000000000000); // total liquidity assets
        assert_eq!(XykStorage::balance(2, 2), 1350000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
        assert_eq!(XykStorage::balance(0, 2), 1100000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
        assert_eq!(XykStorage::balance(1, 2), 1549999999999999999999999); // amount of asset 1 in user acc after creating pool / initial minting
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            900000000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            450000000000000000000001
        ); // amount of asset 1 in vault acc after creating pool

        XykStorage::mint_liquidity(
            Origin::signed(2),
            0,
            1,
            1000000000000000000000000,
            10000000000000000000000000,
        )
        .unwrap();

        assert_eq!(XykStorage::asset_pool((0, 1)), 1900000000000000000000000); // amount of asset 0 in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 950000000000000000000003); // amount of asset 1 in pool map
        assert_eq!(XykStorage::total_supply(2), 2850000000000000000000000); // total liquidity assets
        assert_eq!(XykStorage::balance(2, 2), 2850000000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
        assert_eq!(XykStorage::balance(0, 2), 100000000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
        assert_eq!(XykStorage::balance(1, 2), 1049999999999999999999997); // amount of asset 1 in user acc after creating pool / initial minting
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            1900000000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            950000000000000000000003
        ); // amount of asset 1 in vault acc after creating pool
    });
}

#[test]
fn create_pool_W() {
    new_test_ext().execute_with(|| {
        initialize();
        assert_eq!(XykStorage::asset_pool((0, 1)), 40000000000000000000); // amount of asset 0 in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 60000000000000000000); // amount of asset 1 in pool map
        assert_eq!(XykStorage::liquidity_asset((0, 1)), Some(2)); // liquidity assetId corresponding to newly created pool
        assert_eq!(XykStorage::liquidity_pool(2), Some((0, 1))); // liquidity assetId corresponding to newly created pool
        assert_eq!(XykStorage::total_supply(2), 100000000000000000000); // total liquidity assets
        assert_eq!(XykStorage::balance(2, 2), 100000000000000000000); // amount of liquidity assets owned by user by creating pool / initial minting
        assert_eq!(XykStorage::balance(0, 2), 960000000000000000000); // amount of asset 0 in user acc after creating pool / initial minting
        assert_eq!(XykStorage::balance(1, 2), 940000000000000000000); // amount of asset 1 in user acc after creating pool / initial minting
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            40000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            60000000000000000000
        ); // amount of asset 1 in vault acc after creating pool
    });
}

#[test]
fn create_pool_N_already_exists() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::create_pool(Origin::signed(2), 0, 500000, 1, 500000,),
            Error::<Test>::PoolAlreadyExists,
        );
    });
}

#[test]
fn create_pool_N_already_exists_other_way() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::create_pool(Origin::signed(2), 1, 500000, 0, 500000,),
            Error::<Test>::PoolAlreadyExists,
        );
    });
}

#[test]
fn create_pool_N_not_enough_first_asset() {
    new_test_ext().execute_with(|| {
        let acc_id: u64 = 2;
        let amount: u128 = 1000000;
        XykStorage::create_new_token(&acc_id, amount);
        XykStorage::create_new_token(&acc_id, amount);

        assert_err!(
            XykStorage::create_pool(Origin::signed(2), 0, 1500000, 1, 500000,),
            Error::<Test>::NotEnoughAssets,
        ); //asset 0 issued to user 1000000, trying to create pool using 1500000
    });
}

#[test]
fn create_pool_N_not_enough_second_asset() {
    new_test_ext().execute_with(|| {
        let acc_id: u64 = 2;
        let amount: u128 = 1000000;
        XykStorage::create_new_token(&acc_id, amount);
        XykStorage::create_new_token(&acc_id, amount);

        assert_err!(
            XykStorage::create_pool(Origin::signed(2), 0, 500000, 1, 1500000,),
            Error::<Test>::NotEnoughAssets,
        ); //asset 1 issued to user 1000000, trying to create pool using 1500000
    });
}

#[test]
fn create_pool_N_same_asset() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::create_pool(Origin::signed(2), 0, 500000, 0, 500000,),
            Error::<Test>::SameAsset,
        );
    });
}

#[test]
fn create_pool_N_zero_first_amount() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::create_pool(Origin::signed(2), 0, 0, 0, 500000,),
            Error::<Test>::ZeroAmount,
        );
    });
}

#[test]
fn create_pool_N_zero_second_amount() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::create_pool(Origin::signed(2), 0, 500000, 0, 0,),
            Error::<Test>::ZeroAmount,
        );
    });
}

#[test]
fn sell_W() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        initialize();
        XykStorage::sell_asset(Origin::signed(2), 0, 1, 20000000000000000000, 0).unwrap(); // selling 20000000000000000000 assetId 0 of pool 0 1

        assert_eq!(XykStorage::balance(0, 2), 940000000000000000000); // amount in user acc after selling
        assert_eq!(XykStorage::balance(1, 2), 959959959959959959959); // amount in user acc after buying
        assert_eq!(XykStorage::asset_pool((0, 1)), 59980000000000000000); // amount of asset 0 in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 40040040040040040041); // amount of asset 1 in pool map
        assert_eq!(XykStorage::balance(0, 2), 940000000000000000000); // amount of asset 0 on account 2
        assert_eq!(XykStorage::balance(1, 2), 959959959959959959959); // amount of asset 1 on account 2
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            59990000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            40040040040040040041
        ); // amount of asset 1 in vault acc after creating pool

        let assets_swapped_event = TestEvent::xyk(Event::<Test>::AssetsSwapped(
            2,
            0,
            20000000000000000000,
            1,
            19959959959959959959,
        ));

        assert!(System::events()
            .iter()
            .any(|record| record.event == assets_swapped_event));
    });
}

#[test]
fn sell_W_other_way() {
    new_test_ext().execute_with(|| {
        initialize();
        XykStorage::sell_asset(Origin::signed(2), 1, 0, 30000000000000000000, 0).unwrap(); // selling 30000000000000000000 assetId 1 of pool 0 1

        assert_eq!(XykStorage::balance(0, 2), 973306639973306639973); // amount of asset 0 in user acc after selling
        assert_eq!(XykStorage::balance(1, 2), 910000000000000000000); // amount of asset 1 in user acc after buying
        assert_eq!(XykStorage::asset_pool((0, 1)), 26684462240017795575); // amount of asset 0 in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 90000000000000000000); // amount of asset 1 in pool map
        assert_eq!(XykStorage::balance(0, 2), 973306639973306639973); // amount of asset 0 on account 2
        assert_eq!(XykStorage::balance(1, 2), 910000000000000000000); // amount of asset 1 on account 2
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            26688911133355577801
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            90000000000000000000
        ); // amount of asset 1 in vault acc after creating pool
    });
}

#[test]
fn sell_N_no_such_pool() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::sell_asset(Origin::signed(2), 0, 10, 250000, 0),
            Error::<Test>::NoSuchPool,
        ); // selling 250000 assetId 0 of pool 0 10 (only pool 0 1 exists)
    });
}
#[test]
fn sell_N_not_enough_selling_assset() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::sell_asset(Origin::signed(2), 0, 1, 1000000000000000000000, 0),
            Error::<Test>::NotEnoughAssets,
        ); // selling 1000000000000000000000 assetId 0 of pool 0 1 (user has only 960000000000000000000)
    });
}

#[test]
fn sell_N_insufficient_output_amount() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::sell_asset(Origin::signed(2), 0, 1, 250000, 500000),
            Error::<Test>::InsufficientOutputAmount,
        ); // selling 250000 assetId 0 of pool 0 1, by the formula user should get 166333 asset 1, but is requesting 500000
    });
}

#[test]
fn sell_N_zero_amount() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::sell_asset(Origin::signed(2), 0, 1, 0, 500000),
            Error::<Test>::ZeroAmount,
        ); // selling 0 assetId 0 of pool 0 1
    });
}

#[test]
fn buy_W() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        initialize();
        // buying 30000000000000000000 assetId 1 of pool 0 1
        XykStorage::buy_asset(
            Origin::signed(2),
            0,
            1,
            30000000000000000000,
            3000000000000000000000,
        )
        .unwrap();
        assert_eq!(XykStorage::balance(0, 2), 919879638916750250752); // amount in user acc after selling
        assert_eq!(XykStorage::balance(1, 2), 970000000000000000000); // amount in user acc after buying
        assert_eq!(XykStorage::asset_pool((0, 1)), 80080240722166499500); // amount in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 30000000000000000000); // amount in pool map
        assert_eq!(XykStorage::balance(0, 2), 919879638916750250752); // amount of asset 0 on account 2
        assert_eq!(XykStorage::balance(1, 2), 970000000000000000000); // amount of asset 1 on account 2
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            80100300902708124374
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            30000000000000000000
        ); // amount of asset 1 in vault acc after creating pool

        let assets_swapped_event = TestEvent::xyk(Event::<Test>::AssetsSwapped(
            2,
            0,
            40120361083249749248,
            1,
            30000000000000000000,
        ));

        assert!(System::events()
            .iter()
            .any(|record| record.event == assets_swapped_event));
    });
}

#[test]
fn buy_W_other_way() {
    new_test_ext().execute_with(|| {
        initialize();
        // buying 30000000000000000000 assetId 0 of pool 0 1
        XykStorage::buy_asset(
            Origin::signed(2),
            1,
            0,
            30000000000000000000,
            3000000000000000000000,
        )
        .unwrap();
        assert_eq!(XykStorage::balance(0, 2), 990000000000000000000); // amount in user acc after selling
        assert_eq!(XykStorage::balance(1, 2), 759458375125376128385); // amount in user acc after buying
        assert_eq!(XykStorage::asset_pool((0, 1)), 9992494370778083564); // amount in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 240541624874623871615); // amount in pool map
        assert_eq!(XykStorage::balance(0, 2), 990000000000000000000); // amount of asset 0 on account 2
        assert_eq!(XykStorage::balance(1, 2), 759458375125376128385); // amount of asset 1 on account 2
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            9996247185389041782
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            240541624874623871615
        ); // amount of asset 1 in vault acc after creating pool
    });
}

#[test]
fn buy_N_no_such_pool() {
    new_test_ext().execute_with(|| {
        initialize();

        // buying 150000 assetId 1 of pool 0 10 (only pool 0 1 exists)
        assert_err!(
            XykStorage::buy_asset(Origin::signed(2), 0, 10, 150000, 5000000),
            Error::<Test>::NoSuchPool,
        );
    });
}

#[test]
fn buy_N_not_enough_reserve() {
    new_test_ext().execute_with(|| {
        initialize();

        // buying 70000000000000000000 assetId 0 of pool 0 1 , only 60000000000000000000 in reserve
        assert_err!(
            XykStorage::buy_asset(
                Origin::signed(2),
                0,
                1,
                70000000000000000000,
                5000000000000000000000
            ),
            Error::<Test>::NotEnoughReserve,
        );
    });
}

#[test]
fn buy_N_not_enough_selling_assset() {
    new_test_ext().execute_with(|| {
        initialize();

        // buying 59000000000000000000 assetId 1 of pool 0 1 should sell 2.36E+21 assetId 0, only 9.6E+20 in acc
        assert_err!(
            XykStorage::buy_asset(
                Origin::signed(2),
                0,
                1,
                59000000000000000000,
                59000000000000000000000
            ),
            Error::<Test>::NotEnoughAssets,
        );
    });
}

#[test]
fn buy_N_insufficient_input_amount() {
    new_test_ext().execute_with(|| {
        initialize();
        // buying 150000 liquidity assetId 1 of pool 0 1
        assert_err!(
            XykStorage::buy_asset(Origin::signed(2), 0, 1, 150000, 10),
            Error::<Test>::InsufficientInputAmount,
        );
    });
}

#[test]
fn buy_N_zero_amount() {
    new_test_ext().execute_with(|| {
        initialize();

        assert_err!(
            XykStorage::buy_asset(Origin::signed(2), 0, 1, 0, 0),
            Error::<Test>::ZeroAmount,
        ); // buying 0 assetId 0 of pool 0 1
    });
}

#[test]
fn mint_W() {
    new_test_ext().execute_with(|| {
        initialize();
        // minting pool 0 1 with 20000000000000000000 assetId 0
        XykStorage::mint_liquidity(
            Origin::signed(2),
            0,
            1,
            20000000000000000000,
            200000000000000000000,
        )
        .unwrap();

        assert_eq!(XykStorage::total_supply(2), 150000000000000000000); // total liquidity assets
        assert_eq!(XykStorage::balance(2, 2), 150000000000000000000); // amount of liquidity assets owned by user by creating pool and minting
        assert_eq!(XykStorage::asset_pool((0, 1)), 60000000000000000000); // amount in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 90000000000000000001); // amount in pool map
        assert_eq!(XykStorage::balance(0, 2), 940000000000000000000); // amount of asset 0 in user acc after minting
        assert_eq!(XykStorage::balance(1, 2), 909999999999999999999); // amount of asset 1 in user acc after minting
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            60000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            90000000000000000001
        ); // amount of asset 1 in vault acc after creating pool
        let liquidity_minted_event = TestEvent::xyk(Event::<Test>::LiquidityMinted(
            2,
            0,
            20000000000000000000,
            1,
            30000000000000000001,
            2,
            50000000000000000000,
        ));

        assert!(System::events()
            .iter()
            .any(|record| record.event == liquidity_minted_event));
    });
}

#[test]
fn mint_W_other_way() {
    new_test_ext().execute_with(|| {
        initialize();
        // minting pool 0 1 with 30000000000000000000 assetId 1
        XykStorage::mint_liquidity(
            Origin::signed(2),
            1,
            0,
            30000000000000000000,
            300000000000000000000,
        )
        .unwrap();

        assert_eq!(XykStorage::total_supply(2), 150000000000000000000); // total liquidity assets
        assert_eq!(XykStorage::balance(2, 2), 150000000000000000000); // amount of liquidity assets owned by user by creating pool and minting
        assert_eq!(XykStorage::asset_pool((0, 1)), 60000000000000000001); // amount in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 90000000000000000000); // amount in pool map
        assert_eq!(XykStorage::balance(0, 2), 939999999999999999999); // amount of asset 0 in user acc after minting
        assert_eq!(XykStorage::balance(1, 2), 910000000000000000000); // amount of asset 1 in user acc after minting
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            60000000000000000001
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            90000000000000000000
        ); // amount of asset 1 in vault acc after creating pool
    });
}

#[test]
fn mint_N_no_such_pool() {
    new_test_ext().execute_with(|| {
        initialize();
        assert_err!(
            XykStorage::mint_liquidity(Origin::signed(2), 0, 10, 250000, 250000),
            Error::<Test>::NoSuchPool,
        ); // minting pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
    });
}

#[test]
fn mint_N_not_enough_first_asset() {
    new_test_ext().execute_with(|| {
        initialize();
        assert_err!(
            XykStorage::mint_liquidity(
                Origin::signed(2),
                0,
                1,
                1000000000000000000000,
                10000000000000000000000
            ),
            Error::<Test>::NotEnoughAssets,
        ); // minting pool 0 1 with 1000000000000000000000 assetId 0 (user has only 960000000000000000000)
    });
}

#[test]
fn mint_N_not_enough_second_asset() {
    new_test_ext().execute_with(|| {
        initialize();
        assert_err!(
            XykStorage::mint_liquidity(
                Origin::signed(2),
                1,
                0,
                1000000000000000000000,
                10000000000000000000000,
            ),
            Error::<Test>::NotEnoughAssets,
        ); // minting pool 0 1 with 1000000000000000000000 assetId 1 (user has only 940000000000000000000)
    });
}

#[test]
fn min_N_zero_amount() {
    new_test_ext().execute_with(|| {
        initialize();
        assert_err!(
            XykStorage::mint_liquidity(Origin::signed(2), 1, 0, 0, 10),
            Error::<Test>::ZeroAmount,
        ); // minting pool 0 1 with 0 assetId 1
    });
}

#[test]
fn mint_N_second_asset_amount_exceeded_expectations() {
    new_test_ext().execute_with(|| {
        initialize();
        assert_err!(
            XykStorage::mint_liquidity(Origin::signed(2), 0, 1, 250000, 10),
            Error::<Test>::SecondAssetAmountExceededExpectations,
        ); // minting pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
    });
}

#[test]
fn burn_W() {
    new_test_ext().execute_with(|| {
        initialize();

        XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 50000000000000000000).unwrap(); // burning 20000000000000000000 asset 0 of pool 0 1

        assert_eq!(XykStorage::balance(2, 2), 50000000000000000000); // amount of liquidity assets owned by user by creating pool and burning
        assert_eq!(XykStorage::asset_pool((0, 1)), 20000000000000000000); // amount in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 30000000000000000000); // amount in pool map
        assert_eq!(XykStorage::balance(0, 2), 980000000000000000000); // amount of asset 0 in user acc after burning
        assert_eq!(XykStorage::balance(1, 2), 970000000000000000000); // amount of asset 1 in user acc after burning
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            20000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            30000000000000000000
        ); // amount of asset 1 in vault acc after creating pool

        let liquidity_burned = TestEvent::xyk(Event::<Test>::LiquidityBurned(
            2,
            0,
            20000000000000000000,
            1,
            30000000000000000000,
            2,
            50000000000000000000,
        ));

        assert!(System::events()
            .iter()
            .any(|record| record.event == liquidity_burned));
    });
}

#[test]
fn burn_W_other_way() {
    new_test_ext().execute_with(|| {
        initialize();
        XykStorage::burn_liquidity(Origin::signed(2), 1, 0, 50000000000000000000).unwrap(); // burning 30000000000000000000 asset 1 of pool 0 1

        assert_eq!(XykStorage::balance(2, 2), 50000000000000000000); // amount of liquidity assets owned by user by creating pool and burning
        assert_eq!(XykStorage::asset_pool((0, 1)), 20000000000000000000); // amount in pool map
        assert_eq!(XykStorage::asset_pool((1, 0)), 30000000000000000000); // amount in pool map
        assert_eq!(XykStorage::balance(0, 2), 980000000000000000000); // amount of asset 0 in user acc after burning
        assert_eq!(XykStorage::balance(1, 2), 970000000000000000000); // amount of asset 1 in user acc after burning
        assert_eq!(
            XykStorage::balance(0, XykStorage::account_id()),
            20000000000000000000
        ); // amount of asset 0 in vault acc after creating pool
        assert_eq!(
            XykStorage::balance(1, XykStorage::account_id()),
            30000000000000000000
        ); // amount of asset 1 in vault acc after creating pool
    });
}

#[test]
fn burn_N_not_enough_liquidity_asset() {
    new_test_ext().execute_with(|| {
        initialize();
        // burning pool 0 1 with 500000000000000000000 liquidity asset amount (user has only 100000000000000000000 liquidity asset amount)
        assert_err!(
            XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 500000000000000000000,),
            Error::<Test>::NotEnoughAssets,
        );
    });
}

#[test]
fn burn_N_no_such_pool() {
    new_test_ext().execute_with(|| {
        initialize();
        // burning pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
        assert_err!(
            XykStorage::burn_liquidity(Origin::signed(2), 0, 10, 250000,),
            Error::<Test>::NoSuchPool,
        );
    });
}

#[test]
fn burn_N_zero_amount() {
    new_test_ext().execute_with(|| {
        initialize();
        assert_err!(
            XykStorage::burn_liquidity(Origin::signed(2), 1, 0, 0,),
            Error::<Test>::ZeroAmount,
        ); // burning pool 0 1 with 0 assetId 1
    });
}
