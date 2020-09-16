use super::*;
use crate::mock::*;
use crate::mock::Test;

use frame_support::{assert_ok, assert_noop, assert_err};

//fn set_vault_W(): set_vault working assert id set  //DONE
//fn set_vault_N_already_initiated(): set_vault not working, already initialized  //DONE

//fn create_pool_W(): create_pool working assert (maps,acocounts values)  //DONE
//fn create_pool_N_already_exists(): create_pool not working if pool already exists  //DONE
//fn create_pool_N_already_exists_other_way(): create_pool not working if pool already exists other way around (create pool X-Y, but pool Y-X exists) //DONE
//fn create_pool_N_not_enough_first_asset(): create_pool not working if account has not enough first asset for initial mint //DONE
//fn create_pool_N_not_enough_second_asset(): create_pool not working if account has not enough second asset for initial mint //DONE
//fn create_pool not working if no such assets  //TODO?
// 0 amount

//fn sell_W(): sell working assert (maps,acocounts values) //DONE
//fn sell_W_other_way(): sell working if sell order in different order as pool (sell pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn sell_N_not_enough_selling_assset(): sell not working if not enough asset to sell //DONE
//fn sell_N_no_such_pool(): sell not working if pool does not exist //DONE
// 0 amount

//fn buy_W(): buy working assert (maps,acocounts values) //DONE
//fn buy_W_other_way(): buy working if buy order in different order as pool (buy pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn buy_N_not_enough_selling_assset(): buy not working if not enough asset to sell //DONE
//fn buy_N_not_enough_reserve(): buy not working if not enough liquidity in pool //DONE
//fn buy_N_no_such_pool(): buy not working if pool does not exist //DONE
// 0 amount

//fn mint_W(): mint working assert (maps,acocounts values) //DONE
//fn mint_W_other_way(): mint working if mint order in different order as pool (mint pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn mint_N_no_such_pool(): mint not working if pool does not exist //DONE
//fn mint_N_not_enough_first_asset(): mint not working, not enough first assets to mint with //DONE
//fn mint_N_not_enough_second_asset(): mint not working, not enough second assets to mint with //DONE
// 0 amount

//fn burn_W(): burn working assert (maps,acocounts values) //DONE
//fn burn_W_other_way(): burn working if burn order in different order as pool (burn pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn burn_N_no_such_pool(): burn not working if pool does not exist //DONE
//fn burn_N_not_enough_liquidity_asset(): burn not enough liquidity assets to burn //DONE
// 0 amount


pub trait Trait: generic_asset::Trait {
    // TODO: Add other types and constants required configure this module.
    // type Hashing = BlakeTwo256;

    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// W - should work
// N - should not work

fn initialize() {
	XykStorage::set_vault_id(Origin::signed(1));
	// creating asset with assetId 0 and minting to accountId 2
	XykStorage::create_asset(
		 Origin::signed(2),
		1000000,
	);
	// creating asset with assetId 1 and minting to accountId 2
	XykStorage::create_asset(
		 Origin::signed(2),
		1000000,
	);
	// creating pool by assetId 2
	XykStorage::create_pool(
		Origin::signed(2),
		0,
		500000,
		1,
		500000,
	);
}

//set_vault working, id set
#[test]
fn set_vault_W() {
	new_test_ext().execute_with(|| {
        assert_ok!(XykStorage::set_vault_id(Origin::signed(1)));
        assert_eq!(XykStorage::vault_id(), 1);
	});
}

//set_vault not working, already initialized
#[test]
fn set_vault_N_already_initiated() {
	new_test_ext().execute_with(|| {
		XykStorage::set_vault_id(Origin::signed(1));
		assert_err!(
			XykStorage::set_vault_id(Origin::signed(1)),
			Error::<Test>::VaultAlreadySet,
		);	   
	});
}

//create_pool working assert (right values in maps and accounts)
#[test]
fn create_pool_W() {
	new_test_ext().execute_with(|| {
		
		// setting vault to accountId 1
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset(
		 	Origin::signed(2),
			1000000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset(
		 	Origin::signed(2),
			1000000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			500000,
			1,
			500000,
		);

		assert_eq!(XykStorage::asset_pool((0, 1)), 500000); // amount in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 500000); // amount in pool map
		assert_eq!(XykStorage::liquidity_asset((0, 1)), 2); // liquidity assetId corresponding to newly created pool
		assert_eq!(XykStorage::liquidity_pool(2),(0, 1)); // liquidity assetId corresponding to newly created pool
		assert_eq!(XykStorage::get_total_issuance(2), 1000000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2, 2), 1000000); // amount of liquidity assets owned by user by creating pool / initial minting (500+500)
		assert_eq!(XykStorage::get_free_balance(0, 2), 500000); // amount in user acc after creating pool / initial minting 
		assert_eq!(XykStorage::get_free_balance(1, 2), 500000); // amount in user acc after creating pool / initial minting 
		assert_eq!(XykStorage::get_free_balance(0, 1), 500000); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1, 1), 500000); // amount in vault acc
	});
}

fn create_pool_N_already_exists() {
	new_test_ext().execute_with(|| {
		
		initialize();

		assert_err!(
			XykStorage::create_pool(
				Origin::signed(2),
				0,
				500000,
				1,
				500000,
			),
			Error::<Test>::VaultAlreadySet,
		);
	});
}

fn create_pool_N_already_exists_other_way() {
	new_test_ext().execute_with(|| {
		
		initialize();

		assert_err!(
			XykStorage::create_pool(
				Origin::signed(2),
				1,
				500000,
				0,
				500000,
			),
			Error::<Test>::VaultAlreadySet,
		);
	});
}

fn create_pool_N_not_enough_first_asset() {
	new_test_ext().execute_with(|| {
		
		initialize();

		assert_err!(
			XykStorage::create_pool(
				Origin::signed(2),
				0,
				1500000,
				1,
				500000,
			),
			Error::<Test>::NotEnoughAssets,
		);
	});
}

fn create_pool_N_not_enough_second_asset() {
	new_test_ext().execute_with(|| {
		
		initialize();

		assert_err!(
			XykStorage::create_pool(
				Origin::signed(2),
				0,
				500000,
				1,
				1500000,
			),
			Error::<Test>::NotEnoughAssets,
		);
	});
}

#[test]
fn mint_W() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// minting pool 0 1 with 250000 assetId 0
		XykStorage::mint_liquidity(
			Origin::signed(2),
			0,
			1,
			250000,
		);

		assert_eq!(XykStorage::get_total_issuance(2), 1500000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1500000); // amount of liquidity assets owned by user by creating pool and minting
		assert_eq!(XykStorage::get_free_balance(0,2), 250000); // amount in user acc after minting 
		assert_eq!(XykStorage::get_free_balance(1,2), 249999); // amount in user acc after minting 
		assert_eq!(XykStorage::get_free_balance(0,1), 750000); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 750001); // amount in vault acc
	});
}

#[test]
fn mint_W_other_way() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// minting pool 0 1 with 250000 assetId 1
		XykStorage::mint_liquidity(
			Origin::signed(2),
			1,
			0,
			250000,
		);

		assert_eq!(XykStorage::get_total_issuance(2), 1500000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1500000); // amount of liquidity assets owned by user by creating pool and minting
		assert_eq!(XykStorage::get_free_balance(0,2), 249999); // amount in user acc after minting 
		assert_eq!(XykStorage::get_free_balance(1,2), 250000); // amount in user acc after minting 
		assert_eq!(XykStorage::get_free_balance(0,1), 750001); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 750000); // amount in vault acc
	});
}

#[test]
fn mint_N_no_such_pool() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// minting pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
		assert_err!(
			XykStorage::mint_liquidity(
				Origin::signed(2),
				0,
				10,
				250000,
			),
			Error::<Test>::NoSuchPool,
		);
	});
}

#[test]
fn mint_N_not_enough_first_asset() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// minting pool 0 1 with 750000 assetId 0 (user has only 500000)
		assert_err!(
			XykStorage::mint_liquidity(
				Origin::signed(2),
				0,
				1,
				750000,
			),
			Error::<Test>::NotEnoughAssets,
		);
	});
}

#[test]
fn mint_N_not_enough_second_asset() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// minting pool 0 1 with 750000 assetId 1 (user has only 500000)
		assert_err!(
			XykStorage::mint_liquidity(
				Origin::signed(2),
				1,
				0,
				750000,
			),
			Error::<Test>::NotEnoughAssets,
		);
	});
}

#[test]
fn burn_W() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// burning 250000 liquidity assetId2 of pool 0 1
		XykStorage::burn_liquidity(
			Origin::signed(2),
			0,
			1,
			500000,
		);

		assert_eq!(XykStorage::get_total_issuance(2), 500000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 500000); // amount of liquidity assets owned by user by creating pool and burning
		assert_eq!(XykStorage::get_free_balance(0,2), 750000); // amount in user acc after burning 
		assert_eq!(XykStorage::get_free_balance(1,2), 750000); // amount in user acc after burning 
		assert_eq!(XykStorage::get_free_balance(0,1), 250000); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 250000); // amount in vault acc
	});
}

#[test]
fn burn_W_other_way() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// burning 250000 liquidity assetId2 of pool 0 1
		XykStorage::burn_liquidity(
			Origin::signed(2),
			1,
			0,
			500000,
		);

		assert_eq!(XykStorage::get_total_issuance(2), 500000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 500000); // amount of liquidity assets owned by user by creating pool and burning
		assert_eq!(XykStorage::get_free_balance(0,2), 750000); // amount in user acc after burning 
		assert_eq!(XykStorage::get_free_balance(1,2), 750000); // amount in user acc after burning 
		assert_eq!(XykStorage::get_free_balance(0,1), 250000); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 250000); // amount in vault acc
	});
}


#[test]
fn burn_N_not_enough_liquidity_asset() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// burning pool 0 1 with 1500000 assetId 0 (user has only 1000000)
		assert_err!(
			XykStorage::burn_liquidity(
				Origin::signed(2),
				0,
				1,
				1500000,
			),
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
			XykStorage::burn_liquidity(
				Origin::signed(2),
				0,
				10,
				250000,
			),
			Error::<Test>::NoSuchPool,
		);
	});
}

#[test]
fn sell_W() {
	new_test_ext().execute_with(|| {
		
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset(
			Origin::signed(2),
			1000000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset(
			Origin::signed(2),
			1000000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			500000,
			1,
			500000,
		);
		// selling 250000 assetId 0 of pool 0 1
		XykStorage::sell_asset(
			Origin::signed(2),
			0,
			1,
			250000,
		);

		assert_eq!(XykStorage::get_total_issuance(2), 1000000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1000000); // amount of liquidity assets owned by user by creating pool and initial minting
		assert_eq!(XykStorage::get_free_balance(0,2), 250000); // amount in user acc after selling 
		assert_eq!(XykStorage::get_free_balance(1,2), 666332); // amount in user acc after buying (check rounding should be 666333?)
		assert_eq!(XykStorage::get_free_balance(0,1), 750000); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 333668); // amount in vault acc (check rounding should be 666337?)
	});
}

#[test]
fn sell_W_other_way() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// selling 250000 assetId 1 of pool 0 1
		XykStorage::sell_asset(
			Origin::signed(2),
			1,
			0,
			250000,
		);

		assert_eq!(XykStorage::get_total_issuance(2), 1000000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1000000); // amount of liquidity assets owned by user by creating pool and initial minting
		assert_eq!(XykStorage::get_free_balance(0,2), 666332); // amount in user acc after selling 
		assert_eq!(XykStorage::get_free_balance(1,2), 250000); // amount in user acc after buying (check rounding should be 666333?)
		assert_eq!(XykStorage::get_free_balance(0,1), 333668); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 750000); // amount in vault acc (check rounding should be 666337?)
	});
}

#[test]
fn sell_N_no_such_pool() {
	new_test_ext().execute_with(|| {
		
		initialize();

		// selling 250000 assetId 0 of pool 0 10 (only pool 0 1 exists)
		assert_err!(
			XykStorage::sell_asset(
				Origin::signed(2),
				0,
				10,
				250000,
			),
			Error::<Test>::NoSuchPool,
		);
	});
}

#[test]
fn sell_N_not_enough_selling_assset() {
	new_test_ext().execute_with(|| {
		
		initialize();

		// selling 750000 assetId 0 of pool 0 1 (user has only 500000)
		assert_err!(
			XykStorage::sell_asset(
				Origin::signed(2),
				0,
				1,
				750000,
			),
			Error::<Test>::NotEnoughAssets,
		);
	});
}

#[test]
fn buy_W() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// buying 150000 liquidity assetId 1 of pool 0 1
		XykStorage::buy_asset(
			Origin::signed(2),
			0,
			1,
			150000,
		);

		assert_eq!(XykStorage::get_total_issuance(2), 1000000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1000000); // amount of liquidity assets owned by user by creating pool and initial minting
		assert_eq!(XykStorage::get_free_balance(0,2), 285069); // amount in user acc after selling (check rounding)
		assert_eq!(XykStorage::get_free_balance(1,2), 650000); // amount in user acc after buying (check rounding )
		assert_eq!(XykStorage::get_free_balance(0,1), 714931); // amount in vault acc (check rounding)
		assert_eq!(XykStorage::get_free_balance(1,1), 350000); // amount in vault acc (check rounding)
	});
}

#[test]
fn buy_W_other_way() {
	new_test_ext().execute_with(|| {
		
		initialize();
		// buying 150000 liquidity assetId 0 of pool 0 1
		XykStorage::buy_asset(
			Origin::signed(2),
			1,
			0,
			150000,
		);

		assert_eq!(XykStorage::get_total_issuance(2), 1000000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1000000); // amount of liquidity assets owned by user by creating pool and initial minting
		assert_eq!(XykStorage::get_free_balance(0,2), 650000); // amount in user acc after selling (check rounding)
		assert_eq!(XykStorage::get_free_balance(1,2), 285069); // amount in user acc after buying (check rounding )
		assert_eq!(XykStorage::get_free_balance(0,1), 350000); // amount in vault acc (check rounding)
		assert_eq!(XykStorage::get_free_balance(1,1), 714931); // amount in vault acc (check rounding)
	});
}

#[test]
fn buy_N_no_such_pool() {
	new_test_ext().execute_with(|| {
		
		initialize();

		// buying 150000 assetId 1 of pool 0 10 (only pool 0 1 exists)
		assert_err!(
			XykStorage::buy_asset(
				Origin::signed(2),
				0,
				10,
				150000,
			),
			Error::<Test>::NoSuchPool,
		);
	});
}

#[test]
fn buy_N_not_enough_reserve() {
	new_test_ext().execute_with(|| {
		
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset(
			Origin::signed(2),
			1000000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset(
			Origin::signed(2),
			1000000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			100000,
			1,
			100000,
		);

		// buying 150000 assetId 1 of pool 0 1 , only 100000 in reserve
		assert_err!(
			XykStorage::buy_asset(
				Origin::signed(2),
				0,
				1,
				150000,
			),
			Error::<Test>::NotEnoughReserve,
		);
	});
}

#[test]
fn buy_N_not_enough_selling_assset() {
	new_test_ext().execute_with(|| {
		
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset(
			Origin::signed(2),
			600000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset(
			Origin::signed(2),
			600000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			500000,
			1,
			500000,
		);

		// buying 150000 assetId 1 of pool 0 1 should sell 214931 assetId 0, only 10000 in acc
		assert_err!(
			XykStorage::buy_asset(
				Origin::signed(2),
				0,
				1,
				150000,
			),
			Error::<Test>::NotEnoughAssets,
		);
	});
}