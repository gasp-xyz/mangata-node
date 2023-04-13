#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use mangata_support::traits::{ComputeIssuance, ProofOfStakeRewardsApi};
use orml_tokens::MultiTokenCurrencyExtended;
use sp_runtime::{Permill, SaturatedConversion};

use crate::Pallet as PoS;

const MILION: u128 = 1_000__000_000__000_000;

#[macro_export]
macro_rules! init {
	() => {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		pallet_issuance::Pallet::<T>::initialize();
	};
}

#[macro_export]
macro_rules! forward_to_next_session {
	() => {
		let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();

		let blocks_per_session: u32 = PoS::<T>::rewards_period();
		let target_block_nr: u32;
		let target_session_nr: u32;

		if (current_block == 0_u32 || current_block == 1_u32) {
			target_session_nr = 1_u32;
			target_block_nr = blocks_per_session;
		} else {
			// to fail on user trying to manage block nr on its own
			assert!(current_block % blocks_per_session == 0);
			target_session_nr = (current_block / blocks_per_session) + 1_u32;
			target_block_nr = (target_session_nr * blocks_per_session);
		}

		frame_system::Pallet::<T>::set_block_number(target_block_nr.into());
		pallet_issuance::Pallet::<T>::compute_issuance(target_session_nr);
	};
}

benchmarks! {
	claim_rewards_all_v2 {
		// 1. create
		// 2. promote
		// 3. mint
		// 4. wait some
		// 5. claim all

		init!();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();

	   // Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
		let liquidity_asset_id : TokenId= <T as Config>::Currency::create(&caller, ((40000000000000000000_u128/2_u128) + (60000000000000000000_u128/2_u128)).into()).unwrap().into();
	   PoS::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
		let half_of_minted_liquidity = total_minted_liquidity.into() / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

		forward_to_next_session!();

		PoS::<T>::activate_liquidity(caller.clone(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		forward_to_next_session!();
		forward_to_next_session!();

		assert!(PoS::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap() > 0);

	}: claim_rewards_all_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id)
	verify {

		assert_eq!(
			0,
			PoS::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap()
		);

	}


	update_pool_promotion {
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000;
		let token_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();

	}: update_pool_promotion(RawOrigin::Root, token_id, 1u8)

	verify {
		assert!(
			PoS::<T>::is_enabled(token_id)
		 );
	}

	activate_liquidity_v2 {
		// activate :
		// 1 crate pool
		// 2 promote pool
		// 3 activate some
		// 4 wait some time
		// 5 mint some

		init!();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();

	   // Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
		let liquidity_asset_id : TokenId= <T as Config>::Currency::create(&caller, ((40000000000000000000_u128/2_u128) + (60000000000000000000_u128/2_u128)).into()).unwrap().into();
	   PoS::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity: u128 = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();
		let half_of_minted_liquidity = total_minted_liquidity / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity / 4_u128;

		PoS::<T>::activate_liquidity(caller.clone(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		assert_eq!(
			PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			quater_of_minted_liquidity
		);

		forward_to_next_session!();

	}: activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity, None)
	verify {

		assert_eq!(
		 PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			half_of_minted_liquidity
		)
	}

	deactivate_liquidity_v2 {
		// deactivate
		// 1 crate pool
		// 2 promote pool
		// 3 mint some tokens
		// deactivate some tokens (all or some - to be checked)

		init!();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000000000000;
		let expected_native_asset_id : TokenId = <T as Config>::NativeCurrencyId::get().into();
		let native_asset_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	   // Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), non_native_asset_id1.into(), 40000000000000000000, non_native_asset_id2.into(), 60000000000000000000).unwrap();
		let liquidity_asset_id : TokenId= <T as Config>::Currency::create(&caller, ((40000000000000000000_u128/2_u128) + (60000000000000000000_u128/2_u128)).into()).unwrap().into();
		PoS::<T>::enable(liquidity_asset_id, 1u8);

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
		let half_of_minted_liquidity = total_minted_liquidity.into() / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

		PoS::<T>::activate_liquidity(caller.clone(), liquidity_asset_id.into(), half_of_minted_liquidity, None).unwrap();
		assert_eq!(
		 PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			half_of_minted_liquidity
		);

		forward_to_next_session!();

	}: deactivate_liquidity_v2(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity.into())
	verify {
		assert_eq!(
		 PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			quater_of_minted_liquidity
		);
	}

	// proof_of_stake_compound_rewards {
	// 	 let other: T::AccountId = account("caller1", 0, 0);
	// 	 let caller: T::AccountId = whitelisted_caller();
	// 	 let reward_ratio = 1_000_000;
	// 	 let initial_amount:mangata_types::Balance = 1_000_000_000;
	// 	 let pool_amount:mangata_types::Balance = initial_amount / 2;
	//
	// 	 let next_asset_id: TokenId = <T as Config>::Currency::get_next_currency_id().into();
	// 	 let asset_id_1: TokenId;
	// 	 let asset_id_2: TokenId;
	// 	 if next_asset_id == 0 {
	// 		 // in test there is no other currencies created
	// 		 asset_id_1 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	// 		 <T as Config>::Currency::mint(asset_id_1.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
	// 		 asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	// 		 <T as Config>::Currency::mint(asset_id_2.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
	// 	 } else {
	// 		 // in bench the genesis sets up the assets
	// 		 asset_id_1 = <T as Config>::NativeCurrencyId::get().into();
	// 		 <T as Config>::Currency::mint(asset_id_1.into(), &caller, initial_amount.into()).unwrap();
	// 		 <T as Config>::Currency::mint(asset_id_1.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
	// 		 asset_id_2 = <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
	// 		 <T as Config>::Currency::mint(asset_id_2.into(), &other, (initial_amount * reward_ratio).into()).unwrap();
	// 	 }
	//
	// 	 let liquidity_asset_id = asset_id_2 + 1;
	// 	 <pallet_issuance::Pallet<T> as ComputeIssuance>::initialize();
	//
	// 	 Xyk::<T>::create_pool(RawOrigin::Signed(caller.clone().into()).into(), asset_id_1.into(), pool_amount, asset_id_2.into(), pool_amount).unwrap();
	// 	 <pallet_proof_of_stake::Pallet<T>>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();
	// 	 <pallet_proof_of_stake::Pallet<T>>::activate_liquidity_v2(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id, pool_amount, None).unwrap();
	// 	 // mint for other to split the rewards rewards_ratio:1
	// 	 Xyk::<T>::mint_liquidity(
	// 		 RawOrigin::Signed(other.clone().into()).into(),
	// 		 asset_id_1,
	// 		 asset_id_2,
	// 		 pool_amount * reward_ratio,
	// 		 pool_amount * reward_ratio + 1,
	// 	 ).unwrap();
	//
	// 	 frame_system::Pallet::<T>::set_block_number(50_000u32.into());
	// 	 <pallet_issuance::Pallet<T> as ComputeIssuance>::compute_issuance(1);
	//
	// 	 let mut pre_pool_balance =Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
	// 	 let rewards_to_claim =<pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap();
	// 	 let swap_amount =Xyk::<T>::calculate_balanced_sell_amount(rewards_to_claim, pre_pool_balance.0).unwrap();
	// 	 let balance_native_before = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
	// 	 let balance_asset_before = <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller).into();
	// 	 pre_pool_balance =Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
	//
	//  }: {<pallet_proof_of_stake::Pallet<T>>::compound_rewards(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), Permill::one())}
	//  verify {
	//
	// 	 assert_eq!(
	// 		 <pallet_proof_of_stake::Pallet<T> as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(caller.clone(), liquidity_asset_id).unwrap(),
	// 		 (0_u128)
	// 	 );
	//
	// 	 let balance_native_after = <T as Config>::Currency::free_balance(<T as Config>::NativeCurrencyId::get().into(), &caller).into();
	// 	 let balance_asset_after = <T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller).into();
	// 	 // surplus asset amount
	// 	 assert!(balance_native_before < balance_native_after);
	// 	 assert_eq!(balance_asset_before, balance_asset_after);
	//
	// 	 let post_pool_balance =Xyk::<T>::asset_pool((asset_id_1, asset_id_2));
	// 	 assert!( pre_pool_balance.0 < post_pool_balance.0);
	// 	 assert!( pre_pool_balance.1 >= post_pool_balance.1);
	//  }
	impl_benchmark_test_suite!(PoS, crate::mock::new_test_ext(), crate::mock::Test)
}
