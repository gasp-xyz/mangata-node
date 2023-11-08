use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use pallet_vesting_mangata::VestingInfo;

#[test]
fn reserve_vesting_liquidity_tokens_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 2_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let locked_amount: Balance = 500_000__u128;
		let lock_ending_block_as_balance: Balance = 1_000__u128;

		let reserve_amount: Balance = 200_000__u128;

		// Assuming max locks is 50
		// Let's add 49 dummy ones for worst case

		let n = 49;
		let dummy_lock_amount = 1000u128;
		let dummy_end_block = 10_u128;

		for _ in 0..n {
			<Test as Config>::VestingProvider::lock_tokens(
				&caller,
				asset_id,
				dummy_lock_amount,
				None,
				dummy_end_block,
			)
			.unwrap();
		}
		<Test as Config>::VestingProvider::lock_tokens(
			&caller,
			asset_id,
			locked_amount,
			None,
			lock_ending_block_as_balance,
		)
		.unwrap();

		let now = <frame_system::Pallet<Test>>::block_number();
		assert_ok!(MultiPurposeLiquidity::reserve_vesting_liquidity_tokens(
			RawOrigin::Signed(caller.clone()).into(),
			asset_id,
			reserve_amount
		));

		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			1800000u128
		);
		assert_eq!(
			<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance,
			349000u128
		);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			200000u128
		);
		assert_eq!(
			Vesting::vesting(caller.clone(), asset_id as TokenId)
				.unwrap()
				.into_inner()
				.pop()
				.unwrap()
				.locked(),
			300000
		);
		assert_eq!(
			MultiPurposeLiquidity::get_reserve_status(caller.clone(), asset_id).relock_amount,
			reserve_amount
		);
		assert_eq!(
			MultiPurposeLiquidity::get_relock_status(caller, asset_id)[0],
			RelockStatusInfo::<Balance, BlockNumberFor<Test>> {
				amount: reserve_amount,
				starting_block: now,
				ending_block_as_balance: lock_ending_block_as_balance
			}
		);
	})
}

#[test]
fn unreserve_and_relock_instance_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 2_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let locked_amount: Balance = 500_000__u128;
		let lock_ending_block_as_balance: Balance = 1_000__u128;

		let reserve_amount: Balance = 200_000__u128;

		// Assuming max locks is 50
		// Let's add 49 dummy ones for worst case

		let n = 48;
		let dummy_lock_amount = 1000u128;
		let dummy_end_block = 10_u128;

		for _ in 0..n {
			<Test as Config>::VestingProvider::lock_tokens(
				&caller,
				asset_id,
				dummy_lock_amount,
				None,
				dummy_end_block,
			)
			.unwrap();
		}
		<Test as Config>::VestingProvider::lock_tokens(
			&caller,
			asset_id,
			locked_amount,
			None,
			lock_ending_block_as_balance,
		)
		.unwrap();

		let now = <frame_system::Pallet<Test>>::block_number();
		assert_ok!(MultiPurposeLiquidity::reserve_vesting_liquidity_tokens(
			RawOrigin::Signed(caller.clone()).into(),
			asset_id,
			reserve_amount
		));

		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			1_800_000__u128
		);
		assert_eq!(
			<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance,
			348000u128
		);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			200000u128
		);
		assert_eq!(
			Vesting::vesting(caller.clone(), asset_id as TokenId)
				.unwrap()
				.into_inner()
				.pop()
				.unwrap()
				.locked(),
			300000
		);
		assert_eq!(
			MultiPurposeLiquidity::get_reserve_status(caller.clone(), asset_id).relock_amount,
			reserve_amount
		);
		assert_eq!(
			MultiPurposeLiquidity::get_relock_status(caller, asset_id)[0],
			RelockStatusInfo::<Balance, BlockNumberFor<Test>> {
				amount: reserve_amount,
				starting_block: now,
				ending_block_as_balance: lock_ending_block_as_balance
			}
		);

		assert_ok!(MultiPurposeLiquidity::unreserve_and_relock_instance(
			RawOrigin::Signed(caller.clone()).into(),
			asset_id,
			0u32
		));

		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			2_000_000__u128
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 548000);
		assert_eq!(<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			Vesting::vesting(caller.clone(), asset_id as TokenId)
				.unwrap()
				.into_inner()
				.pop()
				.unwrap()
				.locked(),
			200000
		);
		assert_eq!(
			MultiPurposeLiquidity::get_reserve_status(caller.clone(), asset_id).relock_amount,
			Balance::zero()
		);
		assert_eq!(MultiPurposeLiquidity::get_relock_status(caller, asset_id), vec![]);
	})
}

#[test]
fn bond_from_available_balance_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 1_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let bond_amount: Balance = 100_000__u128;

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);

		assert_ok!(
			<Pallet<Test> as StakingReservesProviderTrait<AccountId, Balance, TokenId>>::bond(
				asset_id,
				&caller,
				bond_amount,
				Some(BondKind::AvailableBalance)
			)
		);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, bond_amount);
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - bond_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			bond_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);
	})
}

#[test]
fn bond_from_activated_unstaked_liquidity_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 1_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let activated_amount: Balance = 200_000__u128;
		let bond_amount: Balance = 100_000__u128;

		assert_ok!(<Test as Config>::Tokens::reserve(asset_id, &caller, activated_amount));
		let mut updated_reserve_status =
			Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		updated_reserve_status.activated_unstaked_reserves = activated_amount;
		ReserveStatus::<Test>::insert(caller, asset_id, updated_reserve_status);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, activated_amount);
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - activated_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			activated_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);

		assert_ok!(
			<Pallet<Test> as StakingReservesProviderTrait<AccountId, Balance, TokenId>>::bond(
				asset_id,
				&caller,
				bond_amount,
				Some(BondKind::ActivatedUnstakedReserves)
			)
		);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, activated_amount - bond_amount);
		assert_eq!(reserve_status.staked_and_activated_reserves, bond_amount);
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - activated_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			activated_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);
	})
}

#[test]
fn bond_from_unspent_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 1_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let unspent_amount: Balance = 200_000__u128;
		let bond_amount: Balance = 100_000__u128;

		assert_ok!(<Test as Config>::Tokens::reserve(asset_id, &caller, unspent_amount));
		let mut updated_reserve_status =
			Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		updated_reserve_status.unspent_reserves = unspent_amount;
		ReserveStatus::<Test>::insert(caller, asset_id, updated_reserve_status);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, unspent_amount);
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - unspent_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			unspent_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);

		assert_ok!(
			<Pallet<Test> as StakingReservesProviderTrait<AccountId, Balance, TokenId>>::bond(
				asset_id,
				&caller,
				bond_amount,
				Some(BondKind::UnspentReserves)
			)
		);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, bond_amount);
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, unspent_amount - bond_amount);
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - unspent_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			unspent_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);
	})
}

#[test]
fn activate_from_available_balance_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 1_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let activate_amount: Balance = 100_000__u128;

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);

		assert_ok!(<Pallet<Test> as ActivationReservesProviderTrait<
			AccountId,
			Balance,
			TokenId,
		>>::activate(
			asset_id, &caller, activate_amount, Some(ActivateKind::AvailableBalance)
		));

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, activate_amount);
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - activate_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			activate_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);
	})
}

#[test]
fn activate_from_staked_unactivated_liquidity_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 1_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let bonded_amount: Balance = 200_000__u128;
		let activate_amount: Balance = 100_000__u128;

		assert_ok!(<Test as Config>::Tokens::reserve(asset_id, &caller, bonded_amount));
		let mut updated_reserve_status =
			Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		updated_reserve_status.staked_unactivated_reserves = bonded_amount;
		ReserveStatus::<Test>::insert(caller, asset_id, updated_reserve_status);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, bonded_amount);
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - bonded_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			bonded_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);

		assert_ok!(<Pallet<Test> as ActivationReservesProviderTrait<
			AccountId,
			Balance,
			TokenId,
		>>::activate(
			asset_id,
			&caller,
			activate_amount,
			Some(ActivateKind::StakedUnactivatedReserves)
		));

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, bonded_amount - activate_amount);
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, activate_amount);
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - bonded_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			bonded_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);
	})
}

#[test]
fn activate_from_unspent_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 1_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let unspent_amount: Balance = 200_000__u128;
		let activate_amount: Balance = 100_000__u128;

		assert_ok!(<Test as Config>::Tokens::reserve(asset_id, &caller, unspent_amount));
		let mut updated_reserve_status =
			Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		updated_reserve_status.unspent_reserves = unspent_amount;
		ReserveStatus::<Test>::insert(caller, asset_id, updated_reserve_status);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, unspent_amount);
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - unspent_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			unspent_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);

		assert_ok!(<Pallet<Test> as ActivationReservesProviderTrait<
			AccountId,
			Balance,
			TokenId,
		>>::activate(
			asset_id, &caller, activate_amount, Some(ActivateKind::UnspentReserves)
		));

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, activate_amount);
		assert_eq!(reserve_status.staked_and_activated_reserves, Balance::zero());
		assert_eq!(reserve_status.unspent_reserves, unspent_amount - activate_amount);
		assert_eq!(reserve_status.relock_amount, Balance::zero());
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - unspent_amount
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			unspent_amount
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);
	})
}

#[test]
fn unbond_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 1_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let staked_unactivated_amount: Balance = 50_000__u128;
		let staked_and_activated_amount: Balance = 85_000__u128;
		let relock_amount: Balance = 100_000__u128;
		let unbond_amount: Balance = 90_000_u128;

		assert_ok!(<Test as Config>::Tokens::reserve(
			asset_id,
			&caller,
			staked_unactivated_amount + staked_and_activated_amount
		));
		let mut updated_reserve_status =
			Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		updated_reserve_status.staked_unactivated_reserves = staked_unactivated_amount;
		updated_reserve_status.staked_and_activated_reserves = staked_and_activated_amount;
		updated_reserve_status.relock_amount = relock_amount;
		ReserveStatus::<Test>::insert(caller, asset_id, updated_reserve_status);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, staked_unactivated_amount);
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, staked_and_activated_amount);
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, relock_amount);
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - (staked_unactivated_amount + staked_and_activated_amount)
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			(staked_unactivated_amount + staked_and_activated_amount)
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);

		assert_eq!(
			<Pallet<Test> as StakingReservesProviderTrait<AccountId, Balance, TokenId>>::unbond(
				asset_id,
				&caller,
				unbond_amount
			),
			Balance::zero()
		);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, 40_000__u128);
		assert_eq!(reserve_status.staked_and_activated_reserves, 45_000__u128);
		assert_eq!(reserve_status.unspent_reserves, 15_000__u128);
		assert_eq!(reserve_status.relock_amount, relock_amount);
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - 135_000__u128 + 35_000__u128
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			135_000__u128 - 35_000__u128
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);
	})
}

#[test]
fn deactivate_works() {
	new_test_ext().execute_with(|| {
		let caller = 0u64;
		let initial_amount: Balance = 1_000_000__u128;
		let asset_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount).unwrap();
		let activated_unstaked_amount: Balance = 50_000__u128;
		let staked_and_activated_amount: Balance = 85_000__u128;
		let relock_amount: Balance = 100_000__u128;
		let deactivate_amount: Balance = 90_000_u128;

		assert_ok!(<Test as Config>::Tokens::reserve(
			asset_id,
			&caller,
			activated_unstaked_amount + staked_and_activated_amount
		));
		let mut updated_reserve_status =
			Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		updated_reserve_status.activated_unstaked_reserves = activated_unstaked_amount;
		updated_reserve_status.staked_and_activated_reserves = staked_and_activated_amount;
		updated_reserve_status.relock_amount = relock_amount;
		ReserveStatus::<Test>::insert(caller, asset_id, updated_reserve_status);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, Balance::zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, activated_unstaked_amount);
		assert_eq!(reserve_status.staked_and_activated_reserves, staked_and_activated_amount);
		assert_eq!(reserve_status.unspent_reserves, Balance::zero());
		assert_eq!(reserve_status.relock_amount, relock_amount);
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - (activated_unstaked_amount + staked_and_activated_amount)
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			(activated_unstaked_amount + staked_and_activated_amount)
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);

		assert_eq!(
			<Pallet<Test> as ActivationReservesProviderTrait<AccountId, Balance, TokenId>>::deactivate(
				asset_id,
				&caller,
				deactivate_amount
			),
			Balance::zero()
		);

		let reserve_status = Pallet::<Test>::get_reserve_status(caller.clone(), asset_id);
		let relock_status = Pallet::<Test>::get_relock_status(caller.clone(), asset_id);
		assert_eq!(reserve_status.staked_unactivated_reserves, 40_000__u128);
		assert_eq!(reserve_status.activated_unstaked_reserves, Balance::zero());
		assert_eq!(reserve_status.staked_and_activated_reserves, 45_000__u128);
		assert_eq!(reserve_status.unspent_reserves, 15_000__u128);
		assert_eq!(reserve_status.relock_amount, relock_amount);
		assert_eq!(relock_status, Vec::<RelockStatusInfo<Balance, BlockNumberFor<Test>>>::new());
		assert_eq!(
			<Test as Config>::Tokens::free_balance(asset_id, &caller) as Balance,
			initial_amount - 135_000__u128 + 35_000__u128
		);
		assert_eq!(<Test as Config>::Tokens::locked_balance(asset_id, &caller) as Balance, 0);
		assert_eq!(
			<Test as Config>::Tokens::reserved_balance(asset_id, &caller) as Balance,
			135_000__u128 - 35_000__u128
		);
		assert_eq!(Vesting::vesting(caller.clone(), asset_id as TokenId), None);
	})
}

type TokensOf<Test> = <Test as Config>::Tokens;

#[test]
fn vested_mpl_vested_transition_works_for_native_tokens() {
	new_test_ext().execute_with(|| {
		const MGX: u32 = <Test as Config>::NativeCurrencyId::get();
		const ALICE: u64 = 0u64;
		const MILLION: u128 = 1_000_000__000_000_000_000_000_000u128;
		const PER_BLOCK: u128 = 1;
		const STARTING_BLOCK: u64 = 10;

		let token_id = <Test as Config>::Tokens::create(&ALICE, MILLION).unwrap();
		assert_eq!(token_id, MGX);
		assert_eq!(TokensOf::<Test>::free_balance(MGX, &ALICE), MILLION);
		assert_eq!(orml_tokens::Accounts::<Test>::get(ALICE, MGX).frozen, 0u128);

		// vest tokens
		pallet_vesting_mangata::Pallet::<Test>::force_vested_transfer(
			RawOrigin::Root.into(),
			MGX,
			ALICE,
			ALICE,
			VestingInfo::new(MILLION, PER_BLOCK, STARTING_BLOCK),
		)
		.unwrap();
		let infos = pallet_vesting_mangata::Pallet::<Test>::vesting(ALICE, MGX).unwrap();
		assert_eq!(infos.get(0).unwrap().locked(), MILLION);
		assert_eq!(infos.get(0).unwrap().per_block(), PER_BLOCK);
		assert_eq!(infos.get(0).unwrap().starting_block(), STARTING_BLOCK);
		assert_eq!(orml_tokens::Accounts::<Test>::get(ALICE, MGX).frozen, MILLION);

		// move vested tokens to MPL
		Pallet::<Test>::reserve_vesting_native_tokens_by_vesting_index(
			RawOrigin::Signed(ALICE).into(),
			0,
			Some(MILLION),
		)
		.unwrap();
		assert_eq!(orml_tokens::Accounts::<Test>::get(ALICE, MGX).frozen, 0u128);
		assert!(pallet_vesting_mangata::Pallet::<Test>::vesting(ALICE, MGX).is_none());

		// move tokens from MPL to vested
		Pallet::<Test>::unreserve_and_relock_instance(RawOrigin::Signed(ALICE).into(), MGX, 0)
			.unwrap();
		let infos = pallet_vesting_mangata::Pallet::<Test>::vesting(ALICE, MGX).unwrap();
		assert_eq!(infos.get(0).unwrap().locked(), MILLION);
		assert_eq!(infos.get(0).unwrap().per_block(), PER_BLOCK);
		assert_eq!(infos.get(0).unwrap().starting_block(), STARTING_BLOCK);
		assert_eq!(orml_tokens::Accounts::<Test>::get(ALICE, MGX).frozen, MILLION);
	})
}
