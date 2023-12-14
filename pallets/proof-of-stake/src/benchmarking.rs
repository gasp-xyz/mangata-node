#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::WithdrawReasons;
use frame_system::RawOrigin;
use mangata_support::traits::{ComputeIssuance, ProofOfStakeRewardsApi};
use orml_tokens::MultiTokenCurrencyExtended;
use sp_runtime::{traits::One, Permill, SaturatedConversion};

use crate::Pallet as PoS;

const MILION: u128 = 1_000__000_000__000_000;

trait ToBalance {
	fn to_balance<T: Config>(self) -> BalanceOf<T>;
}

impl ToBalance for u128 {
	fn to_balance<T: Config>(self) -> BalanceOf<T> {
		self.try_into().ok().expect("u128 should fit into Balance type")
	}
}

fn init<T>()
where
	T: frame_system::Config,
	T: pallet_issuance::Config,
{
	frame_system::Pallet::<T>::set_block_number(1_u32.into());
	pallet_issuance::Pallet::<T>::initialize();
}

type TokensOf<Test> = <Test as Config>::Currency;
type AccountIdOf<Test> = <Test as frame_system::Config>::AccountId;
type XykOf<Test> = <Test as Config>::ValuationApi;

fn forward_to_next_session<T>()
where
	T: frame_system::Config,
	// T: pallet_issuance::Config,
	T: Config,
{
	crate::utils::roll_to_next_session::<T>();
}

benchmarks! {
	claim_native_rewards{
		// 1. create
		// 2. promote
		// 3. mint
		// 4. wait some
		// 5. claim all

		init::<T>();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount: BalanceOf<T> = 1000000000000000000000_u128.try_into().ok().expect("should fit");
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let first_token_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let second_token_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let liquidity_asset_id = <T as Config>::Currency::get_next_currency_id();

		XykOf::<T>::create_pool(
			caller.clone(),
			first_token_id.into(),
			(40000000000000000000_u128/2_u128).to_balance::<T>(),
			second_token_id.into(),
			(60000000000000000000_u128/2_u128).to_balance::<T>()
		).unwrap();

		PoS::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id),
			<T as Config>::Currency::free_balance(liquidity_asset_id, &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id);
		let half_of_minted_liquidity = total_minted_liquidity / 2_u32.into();
		let quater_of_minted_liquidity = total_minted_liquidity / 4_u32.into();

		forward_to_next_session::<T>();

		PoS::<T>::activate_liquidity_for_native_rewards(RawOrigin::Signed(caller.clone()).into(), liquidity_asset_id, quater_of_minted_liquidity, None).unwrap();

		forward_to_next_session::<T>();
		forward_to_next_session::<T>();

		assert!(PoS::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap() > 0_u32.into());

	}: claim_native_rewards(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id)
	verify {

		assert_eq!(
			BalanceOf::<T>::zero(),
			PoS::<T>::calculate_rewards_amount(caller.clone(), liquidity_asset_id).unwrap()
		);

	}


	update_pool_promotion {
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount: BalanceOf<T> = 1000000000000000000000_u128.try_into().ok().expect("should fit");
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();

		let first_token_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let second_token_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let liquidity_asset_id = <T as Config>::Currency::get_next_currency_id();

		XykOf::<T>::create_pool(
			caller.clone(),
			first_token_id.into(),
			(40000000000000000000_u128/2_u128).to_balance::<T>(),
			second_token_id.into(),
			(60000000000000000000_u128/2_u128).to_balance::<T>()
		).unwrap();

	}: update_pool_promotion(RawOrigin::Root, liquidity_asset_id, 1u8)

	verify {
		assert!(
			PoS::<T>::is_enabled(liquidity_asset_id)
		 );
	}

	activate_liquidity_for_native_rewards{
		// activate :
		// 1 crate pool
		// 2 promote pool
		// 3 activate some
		// 4 wait some time
		// 5 mint some

		init::<T>();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount: BalanceOf<T> = 1000000000000000000000_u128.try_into().ok().expect("should fit");
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();
		<T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let first_token_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let second_token_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let liquidity_asset_id = <T as Config>::Currency::get_next_currency_id();

		XykOf::<T>::create_pool(
			caller.clone(),
			first_token_id.into(),
			(40000000000000000000_u128/2_u128).to_balance::<T>(),
			second_token_id.into(),
			(60000000000000000000_u128/2_u128).to_balance::<T>()
		).unwrap();

		PoS::<T>::update_pool_promotion(RawOrigin::Root.into(), liquidity_asset_id, 1u8).unwrap();

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id),
			<T as Config>::Currency::free_balance(liquidity_asset_id, &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id);
		let half_of_minted_liquidity = total_minted_liquidity / 2_u32.into();
		let quater_of_minted_liquidity = total_minted_liquidity / 4_u32.into();

		PoS::<T>::activate_liquidity_for_native_rewards(RawOrigin::Signed(caller.clone()).into(), liquidity_asset_id.into(), quater_of_minted_liquidity, None).unwrap();

		assert_eq!(
			PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			quater_of_minted_liquidity
		);

		forward_to_next_session::<T>();

	}: activate_liquidity_for_native_rewards(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), quater_of_minted_liquidity, None)
	verify {

		assert_eq!(
			PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			half_of_minted_liquidity
		)
	}

	deactivate_liquidity_for_native_rewards{
		// deactivate
		// 1 crate pool
		// 2 promote pool
		// 3 mint some tokens
		// deactivate some tokens (all or some - to be checked)

		init::<T>();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let initial_amount: BalanceOf<T> = 1000000000000000000000_u128.try_into().ok().expect("should fit");
		let expected_native_asset_id  = <T as Config>::NativeCurrencyId::get();
		let native_asset_id = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let non_native_asset_id1 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let non_native_asset_id2 = <T as Config>::Currency::create(&caller, initial_amount).unwrap();
		let amount: BalanceOf<T> = ((40000000000000000000_u128/2_u128) + (60000000000000000000_u128/2_u128)).try_into().ok().expect("should fit");
		let liquidity_asset_id = <T as Config>::Currency::create(&caller, amount).unwrap();
		PoS::<T>::enable(liquidity_asset_id, 1u8);

		assert_eq!(
			<T as Config>::Currency::total_issuance(liquidity_asset_id),
			<T as Config>::Currency::free_balance(liquidity_asset_id, &caller),
		);

		let total_minted_liquidity = <T as Config>::Currency::total_issuance(liquidity_asset_id);
		let half_of_minted_liquidity = total_minted_liquidity / 2_u32.into();
		let quater_of_minted_liquidity = total_minted_liquidity / 4_u32.into();

		PoS::<T>::activate_liquidity_for_native_rewards(RawOrigin::Signed(caller.clone().into()).into(), liquidity_asset_id.into(), half_of_minted_liquidity, None).unwrap();

		assert_eq!(
			 PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			half_of_minted_liquidity
		);

		forward_to_next_session::<T>();

	}: deactivate_liquidity(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id, quater_of_minted_liquidity)
	verify {
		assert_eq!(
			 PoS::<T>::get_rewards_info(caller.clone(), liquidity_asset_id).activated_amount,
			quater_of_minted_liquidity
		);
	}

	reward_pool{
		// 1 crate as many schedules as possible
		// 2 wait for one of the schedules to expire
		// 3 create new schedule that will replace the expired one

		init::<T>();

		let schedules_limit = 10u32;
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let native_asset_id = <T as Config>::NativeCurrencyId::get();

		loop {
			let token_id = TokensOf::<T>::create(&caller, MILION.to_balance::<T>()).unwrap();
			if token_id > native_asset_id {
				break;
			}
		}

		let REWARDS_AMOUNT: u128 = <T as Config>::Min3rdPartyRewardValutationPerSession::get() * 10u128;
		let POOL_VOLUME: u128 = <T as Config>::Min3rdPartyRewardVolume::get() * 1_000_000u128;

		let native_asset_amount: u128 = POOL_VOLUME * Into::<u128>::into(schedules_limit + 1);
		TokensOf::<T>::mint(native_asset_id.into(), &caller, native_asset_amount.to_balance::<T>()).unwrap();

		for _ in 0 .. schedules_limit - 2 {
			let token_id = TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
			XykOf::<T>::create_pool(caller.clone(), native_asset_id.into(), POOL_VOLUME.to_balance::<T>(), token_id, POOL_VOLUME.to_balance::<T>()).unwrap();
			let reward_token = token_id + One::one();
			let balance:u128 = TokensOf::<T>::free_balance(reward_token.into(), &caller).into();

			PoS::<T>::reward_pool(
				RawOrigin::Signed(caller.clone().into()).into(),
				(native_asset_id, token_id),
				reward_token.into(),
				(REWARDS_AMOUNT).to_balance::<T>(),
				10u32.into(),
			).unwrap();
		}

		let token_id = TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		XykOf::<T>::create_pool(caller.clone(), native_asset_id.into(), POOL_VOLUME.to_balance::<T>(), token_id.into(), POOL_VOLUME.to_balance::<T>()).unwrap();
		let reward_token = token_id + One::one();
		PoS::<T>::reward_pool(
			RawOrigin::Signed(caller.clone().into()).into(),
			(native_asset_id, token_id),
			reward_token.into(),
			REWARDS_AMOUNT.to_balance::<T>(),
			2u32.into(),
		).unwrap();

		forward_to_next_session::<T>();
		forward_to_next_session::<T>();
		forward_to_next_session::<T>();

		let token_id = TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		XykOf::<T>::create_pool(caller.clone(), native_asset_id.into(), POOL_VOLUME.to_balance::<T>(), token_id.into(), POOL_VOLUME.to_balance::<T>()).unwrap();
		let reward_token = token_id + One::one();

		assert_eq!(
			PoS::<T>::list_metadata().count,
			(schedules_limit - 1 ) as u64
		);

	}: reward_pool(RawOrigin::Signed(caller.clone().into()), (native_asset_id,token_id), reward_token.into(), REWARDS_AMOUNT.to_balance::<T>(), 10u32.into())
	verify {

		assert_eq!(
			PoS::<T>::list_metadata().count,
			(schedules_limit as u64)
		);

	}

	activate_liquidity_for_3rdparty_rewards{
		// 1 create pool that can be rewarded
		// 2 create token that is used as reward
		// 3 activate rewards

		init::<T>();

		let schedules_limit = 10u32;
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let native_asset_id = <T as Config>::NativeCurrencyId::get();
		let REWARDS_AMOUNT: u128 = <T as Config>::Min3rdPartyRewardValutationPerSession::get() * (schedules_limit as u128);
		let POOL_VOLUME: u128 = <T as Config>::Min3rdPartyRewardVolume::get() * 1_000_000u128;

		loop {
			let token_id = TokensOf::<T>::create(&caller, REWARDS_AMOUNT.to_balance::<T>()).unwrap();
			if token_id > native_asset_id {
				break;
			}
		}

		let native_asset_amount: u128 = POOL_VOLUME * Into::<u128>::into(schedules_limit);
		TokensOf::<T>::mint(native_asset_id.into(), &caller, native_asset_amount.to_balance::<T>()).unwrap();

		let first_token_id= TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		XykOf::<T>::create_pool(caller.clone(), native_asset_id, POOL_VOLUME.to_balance::<T>(), first_token_id, POOL_VOLUME.to_balance::<T>()).unwrap();
		let liquidity_asset_id = first_token_id + One::one();

		let second_token_id = TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		XykOf::<T>::create_pool(caller.clone(), native_asset_id.into(), POOL_VOLUME.to_balance::<T>(), second_token_id.into(), POOL_VOLUME.to_balance::<T>()).unwrap();
		let reward_token_id = second_token_id + One::one();


		PoS::<T>::reward_pool(
			RawOrigin::Signed(caller.clone().into()).into(),
			(native_asset_id, first_token_id),
			reward_token_id.into(),
			REWARDS_AMOUNT.to_balance::<T>(),
			2u32.into(),
		).unwrap();

	}: activate_liquidity_for_3rdparty_rewards(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id.into(), 10_000u128.to_balance::<T>(), reward_token_id, None)
	verify {
		forward_to_next_session::<T>();
		forward_to_next_session::<T>();
		assert_eq!(
		   PoS::<T>::calculate_3rdparty_rewards_amount(caller, liquidity_asset_id.into(), reward_token_id).unwrap(),
					(REWARDS_AMOUNT/2).to_balance::<T>()
		)
	}

	deactivate_liquidity_for_3rdparty_rewards{
		// 1 create pool that can be rewarded
		// 2 create token that is used as reward
		// 3 activate rewards
		// 4 deactivate rewards and unlock them

		init::<T>();

		let schedules_limit = <T as Config>::RewardsSchedulesLimit::get();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let native_asset_id = <T as Config>::NativeCurrencyId::get();
		let REWARDS_AMOUNT: u128 = 2u128 * <T as Config>::Min3rdPartyRewardValutationPerSession::get();
		let POOL_VOLUME: u128 = <T as Config>::Min3rdPartyRewardVolume::get() * 1_000_000u128;

		loop {
			let token_id = TokensOf::<T>::create(&caller, REWARDS_AMOUNT.to_balance::<T>()).unwrap();
			if token_id > native_asset_id {
				break;
			}
		}

		let native_asset_amount: u128 = POOL_VOLUME * Into::<u128>::into(schedules_limit);
		TokensOf::<T>::mint(native_asset_id.into(), &caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		let first_token_id = TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		XykOf::<T>::create_pool(caller.clone(), native_asset_id.into(), POOL_VOLUME.to_balance::<T>(), first_token_id.into(), POOL_VOLUME.to_balance::<T>()).unwrap();
		let liquidity_asset_id = first_token_id + One::one();

		TokensOf::<T>::mint(native_asset_id.into(), &caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		let second_token_id = TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		XykOf::<T>::create_pool(caller.clone(), native_asset_id.into(), POOL_VOLUME.to_balance::<T>(), second_token_id.into(), POOL_VOLUME.to_balance::<T>()).unwrap();
		let reward_token_id = second_token_id + One::one();

		PoS::<T>::reward_pool(
			RawOrigin::Signed(caller.clone().into()).into(),
			(native_asset_id, first_token_id),
			reward_token_id.into(),
			REWARDS_AMOUNT.to_balance::<T>(),
			2u32.into(),
		).unwrap();

		assert!(TokensOf::<T>::ensure_can_withdraw(
			liquidity_asset_id.into(),
			&caller,
			REWARDS_AMOUNT.to_balance::<T>(),
			WithdrawReasons::all(),
			Default::default(),
		).is_ok());

		PoS::<T>::activate_liquidity_for_3rdparty_rewards(
			RawOrigin::Signed(caller.clone().into()).into(),
			liquidity_asset_id,
			10_000u128.to_balance::<T>(),
			reward_token_id,
			None
		).unwrap();

		assert!(
			TokensOf::<T>::ensure_can_withdraw(
			liquidity_asset_id.into(),
			&caller,
			POOL_VOLUME.to_balance::<T>(),
			WithdrawReasons::all(),
			Default::default(),
			).is_err()
		);


	}: deactivate_liquidity_for_3rdparty_rewards(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id, 10_000u128.to_balance::<T>(), reward_token_id)
	verify {

		assert!(TokensOf::<T>::ensure_can_withdraw(
			liquidity_asset_id.into(),
			&caller,
			POOL_VOLUME.to_balance::<T>(),
			WithdrawReasons::all(),
			Default::default(),
		).is_ok());
	}


	claim_3rdparty_rewards{
		// 1 create pool that can be rewarded
		// 2 create token that is used as reward
		// 3 activate rewards
		// 4 wait for rewards to be avialble
		// 5 claim rewards

		init::<T>();

		let schedules_limit = <T as Config>::RewardsSchedulesLimit::get();
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller();
		let native_asset_id = <T as Config>::NativeCurrencyId::get();
		let REWARDS_AMOUNT: u128 = 2u128 * <T as Config>::Min3rdPartyRewardValutationPerSession::get();
		let POOL_VOLUME: u128 = <T as Config>::Min3rdPartyRewardVolume::get() * 1_000_000u128;

		loop {
			let token_id = TokensOf::<T>::create(&caller, REWARDS_AMOUNT.to_balance::<T>()).unwrap();
			if token_id > native_asset_id {
				break;
			}
		}

		let native_asset_amount: u128 = POOL_VOLUME * Into::<u128>::into(schedules_limit);
		TokensOf::<T>::mint(native_asset_id.into(), &caller, native_asset_amount.to_balance::<T>()).unwrap();

		let first_token_id = TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		XykOf::<T>::create_pool(caller.clone(), native_asset_id.into(), POOL_VOLUME.to_balance::<T>(), first_token_id.into(), POOL_VOLUME.to_balance::<T>()).unwrap();
		let liquidity_asset_id = first_token_id + One::one();

		let second_token_id = TokensOf::<T>::create(&caller, POOL_VOLUME.to_balance::<T>()).unwrap();
		XykOf::<T>::create_pool(caller.clone(), native_asset_id.into(), POOL_VOLUME.to_balance::<T>(), second_token_id.into(), POOL_VOLUME.to_balance::<T>()).unwrap();
		let reward_token_id = second_token_id + One::one();

		PoS::<T>::reward_pool(
			RawOrigin::Signed(caller.clone().into()).into(),
			(native_asset_id, first_token_id),
			reward_token_id.into(),
			REWARDS_AMOUNT.to_balance::<T>(),
			2u32.into(),
		).unwrap();

		PoS::<T>::activate_liquidity_for_3rdparty_rewards(
			RawOrigin::Signed(caller.clone().into()).into(),
			liquidity_asset_id,
			10_000u128.to_balance::<T>(),
			reward_token_id,
			None
		).unwrap();

		forward_to_next_session::<T>();
		forward_to_next_session::<T>();
		assert_eq!(
			PoS::<T>::calculate_3rdparty_rewards_amount(caller.clone(), liquidity_asset_id, reward_token_id).unwrap(),
			(REWARDS_AMOUNT / 2).to_balance::<T>()
		);
		forward_to_next_session::<T>();
		assert_eq!(
			PoS::<T>::calculate_3rdparty_rewards_amount(caller.clone(), liquidity_asset_id, reward_token_id).unwrap(),
			REWARDS_AMOUNT.to_balance::<T>()
		);
		forward_to_next_session::<T>();
		assert_eq!(
			PoS::<T>::calculate_3rdparty_rewards_amount(caller.clone(), liquidity_asset_id, reward_token_id).unwrap(),
			REWARDS_AMOUNT.to_balance::<T>()
		);

		let balance_before:u128 = TokensOf::<T>::free_balance(reward_token_id.into(), &caller).into();

	}: claim_3rdparty_rewards(RawOrigin::Signed(caller.clone().into()), liquidity_asset_id, reward_token_id)
	verify {

		let balance_after:u128 = TokensOf::<T>::free_balance(reward_token_id.into(), &caller).into();
		assert_eq!(
			PoS::<T>::calculate_3rdparty_rewards_amount(caller.clone(), liquidity_asset_id, reward_token_id).unwrap(),
			0u128.to_balance::<T>()
		);

		assert_eq!(balance_after - balance_before, REWARDS_AMOUNT);
	}

	impl_benchmark_test_suite!(PoS, crate::mock::new_test_ext(), crate::mock::Test)
}
