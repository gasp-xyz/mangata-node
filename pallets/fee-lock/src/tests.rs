use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_std::convert::TryFrom;
use test_case::test_case;

use orml_tokens::AccountData;
use sp_std::collections::btree_set::BTreeSet;

#[test]
fn update_fee_lock_metadata_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			FeeLock::update_fee_lock_metadata(
				RuntimeOrigin::root(),
				Some(500),
				Some(0),
				Some(1000),
				None,
			),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_noop!(
			FeeLock::update_fee_lock_metadata(
				RuntimeOrigin::root(),
				Some(500),
				None,
				Some(1000),
				None,
			),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_noop!(
			FeeLock::update_fee_lock_metadata(
				RuntimeOrigin::root(),
				Some(0),
				Some(500),
				Some(1000),
				None,
			),
			Error::<Test>::InvalidFeeLockMetadata
		);

		assert_noop!(
			FeeLock::update_fee_lock_metadata(
				RuntimeOrigin::root(),
				None,
				Some(500),
				Some(1000),
				None,
			),
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
						vec![0, 1].into_iter().collect::<BTreeSet<TokenId>>(),
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
						vec![1, 2].into_iter().collect::<BTreeSet<TokenId>>(),
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
						vec![1, 2].into_iter().collect::<BTreeSet<TokenId>>(),
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
						vec![].into_iter().collect::<BTreeSet<TokenId>>(),
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
						vec![].into_iter().collect::<BTreeSet<TokenId>>(),
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
						vec![1, 2].into_iter().collect::<BTreeSet<TokenId>>(),
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

		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(0, 1000),
			Some(1000)
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(0, 0),
			Some(0)
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(0, u128::max_value()),
			Some(u128::max_value())
		);

		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(1, 1000),
			Some(500)
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(1, 0),
			Some(0)
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(1, u128::max_value()),
			Some(u128::max_value() / 2)
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(2, 1000),
			Some(2000)
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(2, 0),
			Some(0)
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(2, u128::max_value()),
			Some(u128::max_value())
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(3, 1000),
			None
		);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(3, 0), None);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(3, u128::max_value()),
			None
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(4, 1000),
			None
		);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(4, 0), None);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(4, u128::max_value()),
			None
		);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(5, 1000),
			None
		);
		assert_eq!(<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(5, 0), None);
		assert_eq!(
			<FeeLock as FeeLockTriggerTrait<_>>::get_swap_valuation_for_token(5, u128::max_value()),
			None
		);
	})
}

const PERIOD_LENGTH: u64 = 10;
const FEE_LOCK_AMOUNT: u128 = 1000;
const SWAP_VALUE_THRESHOLD: u128 = 1000;
const ALICE: u128 = 0u128;
const BOB: u128 = 1u128;
const CHARLIE: u128 = 2u128;
const INITIAL_AMOUNT: Balance = 2_000_000__u128;
const UNLIMITED_WEIGHT: Weight = Weight::from_ref_time(u64::MAX);
const ACCOUNT_WITHOUT_LOCKED_TOKENS: orml_tokens::AccountData<u128> = AccountData {
	free: INITIAL_AMOUNT - 0 * FEE_LOCK_AMOUNT,
	reserved: 0 * FEE_LOCK_AMOUNT,
	frozen: 0u128,
};
const ACCOUNT_WITH_LOCKED_TOKENS: orml_tokens::AccountData<u128> = AccountData {
	free: INITIAL_AMOUNT - 1 * FEE_LOCK_AMOUNT,
	reserved: 1 * FEE_LOCK_AMOUNT,
	frozen: 0u128,
};

fn calculate_estimated_weight(unlock_fee_calls: u64, reads: u64, writes: u64) -> Weight {
	<Test as frame_system::Config>::DbWeight::get().reads(reads) +
		<Test as frame_system::Config>::DbWeight::get().writes(writes) +
		(<Test as Config>::WeightInfo::unlock_fee() * unlock_fee_calls)
}

#[test_case(
	UNLIMITED_WEIGHT,
	calculate_estimated_weight(1, 6, 1),
	ACCOUNT_WITHOUT_LOCKED_TOKENS; "unlocks tokens for an user")]
#[test_case(
	Weight::from_parts(0, 0),
	Weight::from_parts(0, 0),
	ACCOUNT_WITH_LOCKED_TOKENS; "does not unlock tokens when weigh is zero")]
#[test_case(
	calculate_estimated_weight(1, 6, 1),
	calculate_estimated_weight(1, 6, 1),
	ACCOUNT_WITHOUT_LOCKED_TOKENS; "unlock tokens using exact amount of weight required")]
#[test_case(
	calculate_estimated_weight(1, 4, 1),
	Weight::from_parts(0, 0),
	ACCOUNT_WITH_LOCKED_TOKENS; "unlock tokens using a too small weight that required")]
#[test_case(
	calculate_estimated_weight(1, 7, 1),
	calculate_estimated_weight(1, 6, 1),
	ACCOUNT_WITHOUT_LOCKED_TOKENS; "unlock tokens using a bit more weight that required")]
fn test_on_idle_unlock_for_single_user(
	availabe_weight: Weight,
	consumed_weight: Weight,
	expected_account_data: AccountData<Balance>,
) {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();
			fast_forward_blocks(PERIOD_LENGTH);

			// assert
			assert_ok!(<FeeLock as FeeLockTriggerTrait<_>>::can_unlock_fee(&ALICE));
			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				AccountData {
					free: INITIAL_AMOUNT - FEE_LOCK_AMOUNT,
					reserved: 1 * FEE_LOCK_AMOUNT,
					frozen: 0__u128,
				}
			);

			assert_eq!(FeeLock::on_idle(System::block_number(), availabe_weight), consumed_weight);

			assert_eq!(Tokens::accounts(ALICE, NativeCurrencyId::get()), expected_account_data);
		});
}

#[test_case(
	Weight::from_ref_time(u64::MAX),
	calculate_estimated_weight(2, 9, 2),
	vec![
	(ALICE, ACCOUNT_WITHOUT_LOCKED_TOKENS),
	(BOB, ACCOUNT_WITHOUT_LOCKED_TOKENS),
	]; "unlocks tokens for both users with unlimited input weight")]
#[test_case(
	calculate_estimated_weight(2, 9, 2),
	calculate_estimated_weight(2, 9, 2),
	vec![
	(ALICE, ACCOUNT_WITHOUT_LOCKED_TOKENS),
	(BOB, ACCOUNT_WITHOUT_LOCKED_TOKENS),
	]; "unlocks tokens for both users using exact required weight ")]
#[test_case(
	calculate_estimated_weight(1, 6, 1),
	calculate_estimated_weight(1, 6, 1),
	vec![
	(ALICE, ACCOUNT_WITHOUT_LOCKED_TOKENS),
	(BOB, ACCOUNT_WITH_LOCKED_TOKENS),
	]; "unlocks tokens for single account only with limited weight")]
fn test_on_idle_unlock_multiple_users(
	availabe_weight: Weight,
	consumed_weight: Weight,
	expected_account_data: Vec<(<Test as frame_system::Config>::AccountId, AccountData<Balance>)>,
) {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(BOB, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			for (account, _) in expected_account_data.iter() {
				<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(account).unwrap();
			}
			fast_forward_blocks(PERIOD_LENGTH);

			let consumed = FeeLock::on_idle(System::block_number(), availabe_weight);

			for data in expected_account_data {
				assert_eq!(Tokens::accounts(data.0, NativeCurrencyId::get()), data.1);
			}

			assert_eq!(consumed_weight, consumed);
		});
}

#[test]
fn test_unlock_happens_not_sooner_but_after_period() {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			// lets move to some block that is not aligned with period start
			fast_forward_blocks(7);
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();

			for _ in 0..PERIOD_LENGTH - 1 {
				fast_forward_blocks(1);
				FeeLock::on_idle(System::block_number(), UNLIMITED_WEIGHT);
				assert_eq!(
					Tokens::accounts(ALICE, NativeCurrencyId::get()),
					ACCOUNT_WITH_LOCKED_TOKENS
				);
			}

			// lock period ends now
			fast_forward_blocks(1);

			FeeLock::on_idle(System::block_number(), UNLIMITED_WEIGHT);
			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);
		});
}

#[test]
fn test_unlock_stops_after_single_iteration_without_consuming_unnecessary_weight() {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(BOB, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(CHARLIE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			// lets move to some block that is not aligned with period start
			fast_forward_blocks(3);
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&BOB).unwrap();
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&CHARLIE).unwrap();

			fast_forward_blocks(3);
			let consumed_weight = FeeLock::on_idle(System::block_number(), UNLIMITED_WEIGHT);
			assert_eq!(consumed_weight, calculate_estimated_weight(0, 6, 1));

			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITH_LOCKED_TOKENS
			);
		});
}

#[test]
fn test_autounlock_on_empty_unlock_queue() {
	ExtBuilder::new()
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			FeeLock::on_idle(System::block_number(), UNLIMITED_WEIGHT);
		});
}

#[test]
fn test_maintain_queue_with_subsequent_fee_locks_on_single_account() {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(BOB, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(CHARLIE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			fast_forward_blocks(3);
			assert_eq!(UnlockQueue::<Test>::get(0), None);
			assert_eq!(UnlockQueue::<Test>::get(1), None);
			assert_eq!(UnlockQueue::<Test>::get(2), None);
			assert_eq!(FeeLockMetadataQeueuePosition::<Test>::get(ALICE), None);
			assert_eq!(FeeLockMetadataQeueuePosition::<Test>::get(BOB), None);
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 0);
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();

			assert_eq!(UnlockQueue::<Test>::get(0), Some(ALICE));
			assert_eq!(UnlockQueue::<Test>::get(1), None);
			assert_eq!(UnlockQueue::<Test>::get(2), None);
			assert_eq!(FeeLockMetadataQeueuePosition::<Test>::get(ALICE), Some(0));
			assert_eq!(FeeLockMetadataQeueuePosition::<Test>::get(BOB), None);
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 1);

			fast_forward_blocks(1);
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&BOB).unwrap();
			assert_eq!(UnlockQueue::<Test>::get(0), Some(ALICE));
			assert_eq!(UnlockQueue::<Test>::get(1), Some(BOB));
			assert_eq!(UnlockQueue::<Test>::get(2), None);
			assert_eq!(FeeLockMetadataQeueuePosition::<Test>::get(ALICE), Some(0));
			assert_eq!(FeeLockMetadataQeueuePosition::<Test>::get(BOB), Some(1));
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 2);

			fast_forward_blocks(1);
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();
			assert_eq!(UnlockQueue::<Test>::get(0), None);
			assert_eq!(UnlockQueue::<Test>::get(1), Some(BOB));
			assert_eq!(UnlockQueue::<Test>::get(2), Some(ALICE));
			assert_eq!(FeeLockMetadataQeueuePosition::<Test>::get(ALICE), Some(2));
			assert_eq!(FeeLockMetadataQeueuePosition::<Test>::get(BOB), Some(1));
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 3);
		});
}

#[test]
fn test_process_queue_and_ignore_outdated_items_in_unlock_queue_because_of_subsequent_process_fee_lock_calls(
) {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(BOB, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(CHARLIE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			fast_forward_blocks(3);
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 0);

			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 1);
			fast_forward_blocks(1);

			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&BOB).unwrap();
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 2);
			fast_forward_blocks(1);

			FeeLock::on_idle(System::block_number(), UNLIMITED_WEIGHT);
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 2);

			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 3);

			// outdated queue item was consumed
			FeeLock::on_idle(System::block_number(), UNLIMITED_WEIGHT);
			assert_eq!(UnlockQueueBegin::<Test>::get(), 1);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 3);
		});
}

#[test]
fn test_process_queue_and_ignore_outdated_items_in_unlock_queue_because_of_manual_unlock() {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(BOB, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(CHARLIE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			fast_forward_blocks(3);
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 0);

			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 1);
			fast_forward_blocks(PERIOD_LENGTH / 2);

			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&BOB).unwrap();
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 2);

			FeeLock::on_idle(System::block_number(), UNLIMITED_WEIGHT);
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 2);

			fast_forward_blocks(PERIOD_LENGTH / 2);
			FeeLock::unlock_fee(RuntimeOrigin::signed(ALICE).into()).unwrap();
			assert_eq!(UnlockQueueBegin::<Test>::get(), 0);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 2);
			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);

			// nothing to unlock, but increment UnlockQueueBegin counter
			FeeLock::on_idle(System::block_number(), UNLIMITED_WEIGHT);
			assert_eq!(UnlockQueueBegin::<Test>::get(), 1);
			assert_eq!(UnlockQueueEnd::<Test>::get(), 2);

			assert_eq!(Tokens::accounts(BOB, NativeCurrencyId::get()), ACCOUNT_WITH_LOCKED_TOKENS);
		});
}

#[test]
fn test_unlock_happens_in_order() {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(BOB, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(CHARLIE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			let weight_for_single_unlock: Weight = calculate_estimated_weight(1, 6, 1);

			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&BOB).unwrap();
			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&CHARLIE).unwrap();
			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITH_LOCKED_TOKENS
			);
			assert_eq!(Tokens::accounts(BOB, NativeCurrencyId::get()), ACCOUNT_WITH_LOCKED_TOKENS);
			assert_eq!(
				Tokens::accounts(CHARLIE, NativeCurrencyId::get()),
				ACCOUNT_WITH_LOCKED_TOKENS
			);

			fast_forward_blocks(PERIOD_LENGTH);
			FeeLock::on_idle(System::block_number(), weight_for_single_unlock);

			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);
			assert_eq!(Tokens::accounts(BOB, NativeCurrencyId::get()), ACCOUNT_WITH_LOCKED_TOKENS);
			assert_eq!(
				Tokens::accounts(CHARLIE, NativeCurrencyId::get()),
				ACCOUNT_WITH_LOCKED_TOKENS
			);

			FeeLock::on_idle(System::block_number(), weight_for_single_unlock);
			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);
			assert_eq!(
				Tokens::accounts(BOB, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);
			assert_eq!(
				Tokens::accounts(CHARLIE, NativeCurrencyId::get()),
				ACCOUNT_WITH_LOCKED_TOKENS
			);

			FeeLock::on_idle(System::block_number(), weight_for_single_unlock);
			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);
			assert_eq!(
				Tokens::accounts(BOB, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);
			assert_eq!(
				Tokens::accounts(CHARLIE, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);
		});
}

#[test]
fn test_queue_storage_is_cleaned_up() {
	ExtBuilder::new()
		.create_token(NativeCurrencyId::get())
		.mint(ALICE, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.mint(BOB, NativeCurrencyId::get(), INITIAL_AMOUNT)
		.initialize_fee_locks(PERIOD_LENGTH, FEE_LOCK_AMOUNT, SWAP_VALUE_THRESHOLD)
		.build()
		.execute_with(|| {
			assert_eq!(UnlockQueue::<Test>::get(0), None);
			assert_eq!(UnlockQueue::<Test>::get(1), None);

			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&ALICE).unwrap();
			assert_eq!(UnlockQueue::<Test>::get(0), Some(ALICE));
			assert_eq!(UnlockQueue::<Test>::get(1), None);
			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITH_LOCKED_TOKENS
			);

			<FeeLock as FeeLockTriggerTrait<_>>::process_fee_lock(&BOB).unwrap();
			assert_eq!(UnlockQueue::<Test>::get(0), Some(ALICE));
			assert_eq!(UnlockQueue::<Test>::get(1), Some(BOB));
			assert_eq!(Tokens::accounts(BOB, NativeCurrencyId::get()), ACCOUNT_WITH_LOCKED_TOKENS);

			fast_forward_blocks(PERIOD_LENGTH);
			FeeLock::on_idle(System::block_number(), Weight::from_ref_time(u64::MAX));

			assert_eq!(
				Tokens::accounts(ALICE, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);
			assert_eq!(
				Tokens::accounts(BOB, NativeCurrencyId::get()),
				ACCOUNT_WITHOUT_LOCKED_TOKENS
			);

			assert_eq!(UnlockQueue::<Test>::get(0), None);
			assert_eq!(UnlockQueue::<Test>::get(1), None);
		});
}
