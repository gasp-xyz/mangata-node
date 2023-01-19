use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_std::convert::TryFrom;

use orml_tokens::AccountData;
use sp_std::collections::btree_set::BTreeSet;

#[test]
fn update_fee_lock_metadata_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			FeeLock::update_fee_lock_metadata(RuntimeOrigin::root(), Some(500), Some(0), Some(1000), None,),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_noop!(
			FeeLock::update_fee_lock_metadata(RuntimeOrigin::root(), Some(500), None, Some(1000), None,),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_noop!(
			FeeLock::update_fee_lock_metadata(RuntimeOrigin::root(), Some(0), Some(500), Some(1000), None,),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_noop!(
			FeeLock::update_fee_lock_metadata(RuntimeOrigin::root(), None, Some(500), Some(1000), None,),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_ok!(FeeLock::update_fee_lock_metadata(
			RuntimeOrigin::root(),
			Some(1000),
			Some(500),
			Some(1000),
			Some(vec![(0, true), (1, true)]),
		));
		assert_eq!(
			FeeLock::get_fee_lock_metadata(),
			Some(FeeLockMetadataInfo {
				period_length: 1000,
				fee_lock_amount: 500,
				swap_value_threshold: 1000,
				whitelisted_tokens: {
					BoundedBTreeSet::<TokenId, MaxCuratedTokens>::try_from(
						vec![0, 1]
							.into_iter()
							.collect::<BTreeSet<TokenId>>(),
					)
					.unwrap()
				}
			})
		);

		assert_ok!(FeeLock::update_fee_lock_metadata(
			RuntimeOrigin::root(),
			Some(2000),
			Some(2500),
			Some(3000),
			Some(vec![(0, false), (1, true), (2, true)]),
		));
		assert_eq!(
			FeeLock::get_fee_lock_metadata(),
			Some(FeeLockMetadataInfo {
				period_length: 2000,
				fee_lock_amount: 2500,
				swap_value_threshold: 3000,
				whitelisted_tokens: {
					BoundedBTreeSet::<TokenId, MaxCuratedTokens>::try_from(
						vec![1, 2]
							.into_iter()
							.collect::<BTreeSet<TokenId>>(),
					)
					.unwrap()
				}
			})
		);

		assert_noop!(
			FeeLock::update_fee_lock_metadata(RuntimeOrigin::root(), None, Some(0), None, None,),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_noop!(
			FeeLock::update_fee_lock_metadata(RuntimeOrigin::root(), Some(0), None, None, None,),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_ok!(FeeLock::update_fee_lock_metadata(
			RuntimeOrigin::root(),
			None,
			Some(8000),
			None,
			None
		));
		assert_eq!(
			FeeLock::get_fee_lock_metadata(),
			Some(FeeLockMetadataInfo {
				period_length: 2000,
				fee_lock_amount: 8000,
				swap_value_threshold: 3000,
				whitelisted_tokens: {
					BoundedBTreeSet::<TokenId, MaxCuratedTokens>::try_from(
						vec![1, 2]
							.into_iter()
							.collect::<BTreeSet<TokenId>>(),
					)
					.unwrap()
				}
			})
		);
	})
}

#[test]
fn process_fee_lock_trigger_works() {
	new_test_ext().execute_with(|| {
		let period_length = 10;
		let fee_lock_amount = 1000;
		let swap_value_threshold = 1000;

		let caller: u128 = 0u128;
		let initial_amount: Balance = 2_000_000__u128;
		let token_id: TokenId =
			<Test as Config>::Tokens::create(&caller, initial_amount.into()).unwrap().into();

		// The Native token id
		assert_eq!(token_id, NATIVE_CURRENCY_ID);

		// We initaite the threshold map as empty as it is not required here
		assert_ok!(FeeLock::update_fee_lock_metadata(
			RuntimeOrigin::root(),
			Some(period_length),
			Some(fee_lock_amount),
			Some(swap_value_threshold),
			None,
		));
		assert_eq!(
			FeeLock::get_fee_lock_metadata(),
			Some(FeeLockMetadataInfo {
				period_length,
				fee_lock_amount,
				swap_value_threshold,
				whitelisted_tokens: {
					BoundedBTreeSet::<TokenId, MaxCuratedTokens>::try_from(
						vec![]
							.into_iter()
							.collect::<BTreeSet<TokenId>>(),
					)
					.unwrap()
				}
			})
		);

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData { free: 2_000_000__u128, reserved: 0__u128, frozen: 0__u128 }
		);

		let now = System::block_number();

		assert_eq!(now, 0);

		// First timeout on empty user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - fee_lock_amount,
				reserved: fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		// Second timeout on user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 2 * fee_lock_amount,
				reserved: 2 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 2 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		// Third timeout on user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 3 * fee_lock_amount,
				reserved: 3 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 3 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		// Into the next period
		System::set_block_number(12);

		let now = System::block_number();

		// First timeout in current period on Thrice timedout user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 1 * fee_lock_amount,
				reserved: 1 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 1 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		// Into the next period
		System::set_block_number(22);

		let now = System::block_number();

		// First timeout in current period on Once timedout user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 1 * fee_lock_amount,
				reserved: 1 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 1 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		// Second timeout
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 2 * fee_lock_amount,
				reserved: 2 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 2 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		// Into a far off period
		System::set_block_number(22214);

		let now = System::block_number();

		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 1 * fee_lock_amount,
				reserved: 1 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 1 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);
	})
}

#[test]
fn unlock_fee_works() {
	new_test_ext().execute_with(|| {
		let period_length = 10;
		let fee_lock_amount = 1000;
		let swap_value_threshold = 1000;

		let caller: u128 = 0u128;
		let initial_amount: Balance = 2_000_000__u128;
		let token_id: TokenId =
			<Test as Config>::Tokens::create(&caller, initial_amount.into()).unwrap().into();

		// The Native token id
		assert_eq!(token_id, NATIVE_CURRENCY_ID);

		// We initaite the threshold map as empty as it is not required here
		assert_ok!(FeeLock::update_fee_lock_metadata(
			RuntimeOrigin::root(),
			Some(period_length),
			Some(fee_lock_amount),
			Some(swap_value_threshold),
			None,
		));
		assert_eq!(
			FeeLock::get_fee_lock_metadata(),
			Some(FeeLockMetadataInfo {
				period_length,
				fee_lock_amount,
				swap_value_threshold,
				whitelisted_tokens: {
					BoundedBTreeSet::<TokenId, MaxCuratedTokens>::try_from(
						vec![]
							.into_iter()
							.collect::<BTreeSet<TokenId>>(),
					)
					.unwrap()
				}
			})
		);

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData { free: 2_000_000__u128, reserved: 0__u128, frozen: 0__u128 }
		);

		let now = System::block_number();

		assert_eq!(now, 0);

		assert_noop!(
			<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128),
			Error::<Test>::NotFeeLocked
		);
		assert_noop!(
			FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()),
			Error::<Test>::NotFeeLocked
		);

		// First timeout on empty user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - fee_lock_amount,
				reserved: fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		// Second timeout on user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 2 * fee_lock_amount,
				reserved: 2 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 2 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		// Third timeout on user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 3 * fee_lock_amount,
				reserved: 3 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 3 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		assert_noop!(
			<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128),
			Error::<Test>::CantUnlockFeeYet
		);
		assert_noop!(
			FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()),
			Error::<Test>::CantUnlockFeeYet
		);

		// Into the next period
		System::set_block_number(12);

		let now = System::block_number();

		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128));
		assert_ok!(FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 0 * fee_lock_amount,
				reserved: 0 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 0 * fee_lock_amount,
				last_fee_lock_block: 0 * now,
			}
		);

		assert_noop!(
			<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128),
			Error::<Test>::NotFeeLocked
		);
		assert_noop!(
			FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()),
			Error::<Test>::NotFeeLocked
		);

		// First timeout in current period on Thrice timedout user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 1 * fee_lock_amount,
				reserved: 1 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 1 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		assert_noop!(
			<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128),
			Error::<Test>::CantUnlockFeeYet
		);
		assert_noop!(
			FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()),
			Error::<Test>::CantUnlockFeeYet
		);

		// Into the next period
		System::set_block_number(22);

		let now = System::block_number();

		// First timeout in current period on Once timedout user state
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 1 * fee_lock_amount,
				reserved: 1 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 1 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		assert_noop!(
			<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128),
			Error::<Test>::CantUnlockFeeYet
		);
		assert_noop!(
			FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()),
			Error::<Test>::CantUnlockFeeYet
		);

		// Second timeout
		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&0u128));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 2 * fee_lock_amount,
				reserved: 2 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 2 * fee_lock_amount,
				last_fee_lock_block: now,
			}
		);

		assert_noop!(
			<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128),
			Error::<Test>::CantUnlockFeeYet
		);
		assert_noop!(
			FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()),
			Error::<Test>::CantUnlockFeeYet
		);

		// Into a far off period
		System::set_block_number(22214);

		let now = System::block_number();

		assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128));
		assert_ok!(FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()));

		assert_eq!(
			Tokens::accounts(0u128, token_id),
			AccountData {
				free: 2_000_000__u128 - 0 * fee_lock_amount,
				reserved: 0 * fee_lock_amount,
				frozen: 0__u128,
			}
		);

		assert_eq!(
			FeeLock::get_account_fee_lock_data(0u128),
			AccountFeeLockDataInfo {
				total_fee_lock_amount: 0 * fee_lock_amount,
				last_fee_lock_block: 0 * now,
			}
		);

		assert_noop!(
			<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&0u128),
			Error::<Test>::NotFeeLocked
		);
		assert_noop!(
			FeeLock::unlock_fee(RuntimeOrigin::signed(0u128).into()),
			Error::<Test>::NotFeeLocked
		);
	})
}

#[test]
fn whitelist_and_valuation_works() {
	new_test_ext().execute_with(|| {

		assert_ok!(FeeLock::update_fee_lock_metadata(
			RuntimeOrigin::root(),
			Some(1000),
			Some(500),
			Some(1000),
			Some(vec![(1, true), (2, true)]),
		));
		assert_eq!(
			FeeLock::get_fee_lock_metadata(),
			Some(FeeLockMetadataInfo {
				period_length: 1000,
				fee_lock_amount: 500,
				swap_value_threshold: 1000,
				whitelisted_tokens: {
					BoundedBTreeSet::<TokenId, MaxCuratedTokens>::try_from(
						vec![1, 2]
							.into_iter()
							.collect::<BTreeSet<TokenId>>(),
					)
					.unwrap()
				}
			})
		);

		// Native is always whitelisted
		assert!(<FeeLock as FeeLockTriggerTrait<_>>::is_whitelisted(0));

		assert!(<FeeLock as FeeLockTriggerTrait<_>>::is_whitelisted(1));
		assert!(<FeeLock as FeeLockTriggerTrait<_>>::is_whitelisted(2));

		assert!(!<FeeLock as FeeLockTriggerTrait<_>>::is_whitelisted(3));


		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(0, 1000), Some(1000));
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(0, 0), Some(0));
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(0, u128::max_value()), Some(u128::max_value()));

		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(1, 1000), Some(500));
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(1, 0), Some(0));
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(1, u128::max_value()), Some(u128::max_value()/2));
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(2, 1000), Some(2000));
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(2, 0), Some(0));
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(2, u128::max_value()), Some(u128::max_value()));
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(3, 1000), None);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(3, 0), None);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(3, u128::max_value()), None);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(4, 1000), None);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(4, 0), None);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(4, u128::max_value()), None);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(5, 1000), None);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(5, 0), None);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(5, u128::max_value()), None);
	})
}
