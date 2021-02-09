// Copyright (C) 2020 Mangata team

use super::*;
use crate::mock::Test;
use crate::mock::*;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system as system;
use pallet_assets;

//fn create_pool_W(): create_pool working assert (maps,acocounts values)  //DONE
//fn create_pool_N_already_exists(): create_pool not working if pool already exists  //DONE
//fn create_pool_N_already_exists_other_way(): create_pool not working if pool already exists other way around (create pool X-Y, but pool Y-X exists) //DONE
//fn create_pool_N_not_enough_first_asset(): create_pool not working if account has not enough first asset for initial mint //DONE
//fn create_pool_N_not_enough_second_asset(): create_pool not working if account has not enough second asset for initial mint //DONE
// 0 amount

//fn sell_W(): sell working assert (maps,acocounts values) //DONE
//fn sell_W_other_way(): sell working if sell order in different order as pool (sell pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn sell_N_not_enough_selling_assset(): sell not working if not enough asset to sell //DONE
//fn sell_N_no_such_pool(): sell not working if pool does not exist //DONE
//fn sell_N_insufficient_output_amount(): sell not working if insufficient_output_amount //DONE
// 0 amount

//fn buy_W(): buy working assert (maps,acocounts values) //DONE
//fn buy_W_other_way(): buy working if buy order in different order as pool (buy pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn buy_N_not_enough_selling_assset(): buy not working if not enough asset to sell //DONE
//fn buy_N_not_enough_reserve(): buy not working if not enough liquidity in pool //DONE
//fn buy_N_no_such_pool(): buy not working if pool does not exist //DONE
//fn buy_N_insufficient_input_amount(): sell not working if insufficient_output_amount
// 0 amount

//fn mint_W(): mint working assert (maps,acocounts values) //DONE
//fn mint_W_other_way(): mint working if mint order in different order as pool (mint pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn mint_N_no_such_pool(): mint not working if pool does not exist //DONE
//fn mint_N_not_enough_first_asset(): mint not working, not enough first asset to mint with //DONE
//fn mint_N_not_enough_second_asset(): mint not working, not enough second asset to mint with //DONE
// 0 amount

//fn burn_W(): burn working assert (maps,acocounts values) //DONE
//fn burn_W_other_way(): burn working if burn order in different order as pool (burn pool X-Y, but pool Y-X exists), assert (maps,acocounts values) //DONE
//fn burn_N_no_such_pool(): burn not working if pool does not exist //DONE
//fn burn_N_not_enough_first_asset(): burn not enough first asset in liquidity pool to burn //DONE
//fn burn_N_not_enough_second_asset(): burn not enough second asset in liquidity pool to burn //DONE
// 0 amount

//liquidity assets after trade, after burn, after mint

pub trait Trait: assets::Trait {
	// TODO: Add other types and constants required configure this module.
	// type Hashing = BlakeTwo256;

	// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// W - should work
// N - should not work

fn initialize() {
	// creating asset with assetId 0 and minting to accountId 2
	let accId: u64 = 2;
	let amount: u128 = 1000000000000000000000;

	<pallet_assets::Module<Test>>::assets_issue(&accId, &amount);
	<pallet_assets::Module<Test>>::assets_issue(&accId, &amount);
	XykStorage::create_pool(
		Origin::signed(2),
		0,
		40000000000000000000,
		1,
		60000000000000000000,
	);
}

#[test]
fn create_pool_W() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_eq!(XykStorage::asset_pool((0, 1)), 40000000000000000000); // amount of asset 0 in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 60000000000000000000); // amount of asset 1 in pool map
		assert_eq!(XykStorage::liquidity_asset((0, 1)), 2); // liquidity assetId corresponding to newly created pool
		assert_eq!(XykStorage::liquidity_pool(2), (0, 1)); // liquidity assetId corresponding to newly created pool
		assert_eq!(
			<pallet_assets::Module<Test>>::total_supply(2),
			100000000000000000000
		); // total liquidity assets
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(2, 2),
			100000000000000000000
		); // amount of liquidity assets owned by user by creating pool / initial minting
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			960000000000000000000
		); // amount of asset 0 in user acc after creating pool / initial minting
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			940000000000000000000
		); // amount of asset 1 in user acc after creating pool / initial minting
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
		let accId: u64 = 2;
		let amount: u128 = 1000000;
		<pallet_assets::Module<Test>>::assets_issue(&accId, &amount);
		<pallet_assets::Module<Test>>::assets_issue(&accId, &amount);

		assert_err!(
			XykStorage::create_pool(Origin::signed(2), 0, 1500000, 10, 500000,),
			Error::<Test>::NotEnoughAssets,
		); //asset 0 issued to user 1000000, trying to create pool using 1500000
	});
}

#[test]
fn create_pool_N_not_enough_second_asset() {
	new_test_ext().execute_with(|| {
		let accId: u64 = 2;
		let amount: u128 = 1000000;
		<pallet_assets::Module<Test>>::assets_issue(&accId, &amount);
		<pallet_assets::Module<Test>>::assets_issue(&accId, &amount);

		assert_err!(
			XykStorage::create_pool(Origin::signed(2), 0, 500000, 1, 1500000,),
			Error::<Test>::NotEnoughAssets,
		); //asset 1 issued to user 1000000, trying to create pool using 1500000
	});
}

#[test]
fn sell_W() {
	new_test_ext().execute_with(|| {
		initialize();
		XykStorage::sell_asset(Origin::signed(2), 0, 1, 20000000000000000000, 0); // selling 20000000000000000000 assetId 0 of pool 0 1

		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			940000000000000000000
		); // amount in user acc after selling
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			959959959959959959959
		); // amount in user acc after buying
		assert_eq!(XykStorage::asset_pool((0, 1)), 60000000000000000000); // amount of asset 0 in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 40040040040040040041); // amount of asset 1 in pool map
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			940000000000000000000
		); // amount of asset 0 on account 2
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			959959959959959959959
		); // amount of asset 1 on account 2
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(0, 1), 750000); // amount in vault acc
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(1, 1), 333668); // amount in vault acc
	});
}

#[test]
fn sell_W_other_way() {
	new_test_ext().execute_with(|| {
		initialize();
		XykStorage::sell_asset(Origin::signed(2), 1, 0, 30000000000000000000, 0); // selling 30000000000000000000 assetId 1 of pool 0 1

		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			973306639973306639973
		); // amount of asset 0 in user acc after selling
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			910000000000000000000
		); // amount of asset 1 in user acc after buying
		assert_eq!(XykStorage::asset_pool((0, 1)), 26693360026693360027); // amount of asset 0 in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 90000000000000000000); // amount of asset 1 in pool map
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			973306639973306639973
		); // amount of asset 0 on account 2
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			910000000000000000000
		); // amount of asset 1 on account 2
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(0, 1), 750000); // amount in vault acc
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(1, 1), 333668); // amount in vault acc
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
fn buy_W() {
	new_test_ext().execute_with(|| {
		initialize();
		// buying 30000000000000000000 assetId 1 of pool 0 1
		XykStorage::buy_asset(
			Origin::signed(2),
			0,
			1,
			30000000000000000000,
			3000000000000000000000,
		);
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			919879638916750250752
		); // amount in user acc after selling
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			970000000000000000000
		); // amount in user acc after buying
		assert_eq!(XykStorage::asset_pool((0, 1)), 80120361083249749248); // amount in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 30000000000000000000); // amount in pool map
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			919879638916750250752
		); // amount of asset 0 on account 2
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			970000000000000000000
		); // amount of asset 1 on account 2
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(0, 1), 750000); // amount in vault acc
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(1, 1), 333668); // amount in vault acc
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
		);
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			990000000000000000000
		); // amount in user acc after selling
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			759458375125376128385
		); // amount in user acc after buying
		assert_eq!(XykStorage::asset_pool((0, 1)), 10000000000000000000); // amount in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 240541624874623871615); // amount in pool map
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			990000000000000000000
		); // amount of asset 0 on account 2
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			759458375125376128385
		); // amount of asset 1 on account 2
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(0, 1), 750000); // amount in vault acc
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(1, 1), 333668); // amount in vault acc
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
fn mint_W() {
	new_test_ext().execute_with(|| {
		initialize();
		// minting pool 0 1 with 20000000000000000000 assetId 0
		XykStorage::mint_liquidity(Origin::signed(2), 0, 1, 20000000000000000000);

		assert_eq!(
			<pallet_assets::Module<Test>>::total_supply(2),
			150000000000000000000
		); // total liquidity assets
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(2, 2),
			150000000000000000000
		); // amount of liquidity assets owned by user by creating pool and minting
		assert_eq!(XykStorage::asset_pool((0, 1)), 60000000000000000000); // amount in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 90000000000000000001); // amount in pool map
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			940000000000000000000
		); // amount of asset 0 in user acc after minting
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			909999999999999999999
		); // amount of asset 1 in user acc after minting
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(0, 1), 750000); // amount in vault acc
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(1, 1), 750001); // amount in vault acc
	});
}

#[test]
fn mint_W_other_way() {
	new_test_ext().execute_with(|| {
		initialize();
		// minting pool 0 1 with 30000000000000000000 assetId 1
		XykStorage::mint_liquidity(Origin::signed(2), 1, 0, 30000000000000000000);

		assert_eq!(
			<pallet_assets::Module<Test>>::total_supply(2),
			150000000000000000000
		); // total liquidity assets
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(2, 2),
			150000000000000000000
		); // amount of liquidity assets owned by user by creating pool and minting
		assert_eq!(XykStorage::asset_pool((0, 1)), 60000000000000000001); // amount in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 90000000000000000000); // amount in pool map
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			939999999999999999999
		); // amount of asset 0 in user acc after minting
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			910000000000000000000
		); // amount of asset 1 in user acc after minting
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(0, 1), 750000); // amount in vault acc
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(1, 1), 750001); // amount in vault acc
	});
}

#[test]
fn mint_N_no_such_pool() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::mint_liquidity(Origin::signed(2), 0, 10, 250000,),
			Error::<Test>::NoSuchPool,
		); // minting pool 0 10 with 250000 assetId 0 (only pool 0 1 exists)
	});
}

#[test]
fn mint_N_not_enough_first_asset() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::mint_liquidity(Origin::signed(2), 0, 1, 1000000000000000000000,),
			Error::<Test>::NotEnoughAssets,
		); // minting pool 0 1 with 1000000000000000000000 assetId 0 (user has only 960000000000000000000)
	});
}

#[test]
fn mint_N_not_enough_second_asset() {
	new_test_ext().execute_with(|| {
		initialize();
		assert_err!(
			XykStorage::mint_liquidity(Origin::signed(2), 1, 0, 1000000000000000000000,),
			Error::<Test>::NotEnoughAssets,
		); // minting pool 0 1 with 1000000000000000000000 assetId 1 (user has only 940000000000000000000)
	});
}

#[test]
fn burn_W() {
	new_test_ext().execute_with(|| {
		initialize();

		XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 20000000000000000000); // burning 20000000000000000000 asset 0 of pool 0 1

		assert_eq!(
			<pallet_assets::Module<Test>>::balance(2, 2),
			50000000000000000000
		); // amount of liquidity assets owned by user by creating pool and burning
		assert_eq!(XykStorage::asset_pool((0, 1)), 20000000000000000000); // amount in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 30000000000000000000); // amount in pool map
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			980000000000000000000
		); // amount of asset 0 in user acc after burning
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			970000000000000000000
		); // amount of asset 1 in user acc after burning
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(0, 1), 250000); // amount in vault acc
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(1, 1), 250000); // amount in vault acc
	});
}

#[test]
fn burn_W_other_way() {
	new_test_ext().execute_with(|| {
		initialize();
		XykStorage::burn_liquidity(Origin::signed(2), 1, 0, 30000000000000000000); // burning 30000000000000000000 asset 1 of pool 0 1

		assert_eq!(
			<pallet_assets::Module<Test>>::balance(2, 2),
			50000000000000000000
		); // amount of liquidity assets owned by user by creating pool and burning
		assert_eq!(XykStorage::asset_pool((0, 1)), 20000000000000000000); // amount in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 30000000000000000000); // amount in pool map
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(0, 2),
			980000000000000000000
		); // amount of asset 0 in user acc after burning
		assert_eq!(
			<pallet_assets::Module<Test>>::balance(1, 2),
			970000000000000000000
		); // amount of asset 1 in user acc after burning
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(0, 1), 250000); // amount in vault acc
		 // assert_eq!(<pallet_assets::Module<Test>>::balance(1, 1), 250000); // amount in vault acc
	});
}

#[test]
fn burn_N_not_enough_first_asset() {
	new_test_ext().execute_with(|| {
		initialize();
		// burning pool 0 1 with 50000000000000000000 assetId 0 (user has only 40000000000000000000 assetId 0 in liquidity pool)
		assert_err!(
			XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 50000000000000000000,),
			Error::<Test>::NotEnoughAssets,
		);
	});
}

#[test]
fn burn_N_not_enough_second_asset() {
	new_test_ext().execute_with(|| {
		initialize();
		// burning pool 0 1 with 70000000000000000000 assetId 0 (user has only 60000000000000000000 assetId 0 in liquidity pool)
		assert_err!(
			XykStorage::burn_liquidity(Origin::signed(2), 0, 1, 70000000000000000000,),
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
