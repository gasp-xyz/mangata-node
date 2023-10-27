#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use mangata_support::traits::{ComputeIssuance, ProofOfStakeRewardsApi};
use orml_tokens::MultiTokenCurrencyExtended;
use sp_runtime::{SaturatedConversion};

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
	claim_rewards_all{
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
		let non_native_asset_id3 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();

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

		PoS::<T>::activate_liquidity(RawOrigin::Signed(caller.clone()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		forward_to_next_session!();
		forward_to_next_session!();

		assert!(PoS::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap() > 0);

	}: claim_rewards_all(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id)
	verify {

		assert_eq!(
			0,
			PoS::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap()
		);

	}


	update_pool_promotion {
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount:mangata_types::Balance = 1000000000000;
		let non_native_asset_id0 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id1 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id2 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let non_native_asset_id3 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_token_id : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();

	}: update_pool_promotion(RawOrigin::Root, liquidity_token_id, 1u8)

	verify {
		assert!(
			PoS::<T>::is_enabled(liquidity_token_id)
		 );
	}

	activate_liquidity{
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
		let non_native_asset_id3 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();

		let liquidity_asset_id : TokenId= <T as Config>::Currency::create(&caller, ((40000000000000000000_u128/2_u128) + (60000000000000000000_u128/2_u128)).into()).unwrap().into();
	   PoS::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity: u128 = <T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();
		let half_of_minted_liquidity = total_minted_liquidity / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity / 4_u128;

		PoS::<T>::activate_liquidity(RawOrigin::Signed(caller.clone()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		assert_eq!(
			PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			quater_of_minted_liquidity
		);

		forward_to_next_session!();

	}: activate_liquidity(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity, None)
	verify {

		assert_eq!(
		 PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			half_of_minted_liquidity
		)
	}

	deactivate_liquidity{
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
		let non_native_asset_id3 : TokenId= <T as Config>::Currency::create(&caller, initial_amount.into()).unwrap().into();
		let liquidity_asset_id : TokenId= <T as Config>::Currency::create(&caller, ((40000000000000000000_u128/2_u128) + (60000000000000000000_u128/2_u128)).into()).unwrap().into();

		PoS::<T>::enable(liquidity_asset_id, 1u8);

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()),
			<T as Config>::Currency::free_balance(liquidity_asset_id.into(), &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id.into());
		let half_of_minted_liquidity = total_minted_liquidity.into() / 2_u128;
		let quater_of_minted_liquidity = total_minted_liquidity.into() / 4_u128;

		PoS::<T>::activate_liquidity(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), half_of_minted_liquidity, None).unwrap();

		assert_eq!(
		 PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			half_of_minted_liquidity
		);

		forward_to_next_session!();

	}: deactivate_liquidity(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity.into())
	verify {
		assert_eq!(
		 PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			quater_of_minted_liquidity
		);
	}

	impl_benchmark_test_suite!(PoS, crate::mock::new_test_ext(), crate::mock::Test)
}
