use super::*;
use crate::mock::*;
use frame_support::{assert_ok, assert_noop};
use frame_system::RawOrigin;
use sp_std::{collections::btree_map::BTreeMap};
use orml_tokens::AccountData;

#[test]
fn update_timeout_metadata_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(TokenTimeout::update_timeout_metadata(Origin::root(),
		Some(500),
		Some(0),
		None,
		), Error::<Test>::InvalidTimeoutMetadata);

		assert_noop!(TokenTimeout::update_timeout_metadata(Origin::root(),
		Some(500),
		None,
		None,
		), Error::<Test>::InvalidTimeoutMetadata);

		assert_noop!(TokenTimeout::update_timeout_metadata(Origin::root(),
		Some(0),
		Some(500),
		None,
		), Error::<Test>::InvalidTimeoutMetadata);
		
		assert_noop!(TokenTimeout::update_timeout_metadata(Origin::root(),
		None,
		Some(500),
		None,
		), Error::<Test>::InvalidTimeoutMetadata);

		assert_ok!(TokenTimeout::update_timeout_metadata(Origin::root(),
		Some(1000),
		Some(500),
		Some(vec![(0, Some(500)), (1, Some(1000))]),
		));
		assert_eq!(TokenTimeout::get_timeout_metadata(), Some(TimeoutMetadataInfo{
			period_length:1000,
			timeout_amount:500,
			swap_value_threshold: {BoundedBTreeMap::<TokenId, Balance, MaxCuratedTokens>::try_from(vec![(0, 500), (1, 1000)].into_iter().map(|(k,v)| (k,v)).collect::<BTreeMap<TokenId, Balance>>()).unwrap()}
		}));

		assert_ok!(TokenTimeout::update_timeout_metadata(Origin::root(),
		Some(2000),
		Some(2500),
		Some(vec![(0, None), (1, Some(2000)), (2, Some(4000))]),
		));
		assert_eq!(TokenTimeout::get_timeout_metadata(), Some(TimeoutMetadataInfo{
			period_length:2000,
			timeout_amount:2500,
			swap_value_threshold: {BoundedBTreeMap::<TokenId, Balance, MaxCuratedTokens>::try_from(vec![(1, 2000), (2, 4000)].into_iter().map(|(k,v)| (k,v)).collect::<BTreeMap<TokenId, Balance>>()).unwrap()}
		}));

		assert_noop!(TokenTimeout::update_timeout_metadata(Origin::root(),
		None,
		Some(0),
		None,
		), Error::<Test>::InvalidTimeoutMetadata);

		assert_noop!(TokenTimeout::update_timeout_metadata(Origin::root(),
		Some(0),
		None,
		None,
		), Error::<Test>::InvalidTimeoutMetadata);

		assert_ok!(TokenTimeout::update_timeout_metadata(Origin::root(),
		None,
		Some(8000),
		None,
		));
		assert_eq!(TokenTimeout::get_timeout_metadata(), Some(TimeoutMetadataInfo{
			period_length:2000,
			timeout_amount:8000,
			swap_value_threshold: {BoundedBTreeMap::<TokenId, Balance, MaxCuratedTokens>::try_from(vec![(1, 2000), (2, 4000)].into_iter().map(|(k,v)| (k,v)).collect::<BTreeMap<TokenId, Balance>>()).unwrap()}
		}));
	})
}

#[test]
fn process_timeout_trigger_works() {
	new_test_ext().execute_with(|| {

		let period_length = 10;
		let timeout_amount= 1000;

		let caller: u128 = 0u128;
		let initial_amount: Balance = 2_000_000__u128;
		let token_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount.into()).unwrap().into();

		// The Native token id
		assert_eq!(token_id, NATIVE_CURRENCY_ID);

		// We initaite the threshold map as empty as it is not required here
		assert_ok!(TokenTimeout::update_timeout_metadata(Origin::root(),
		Some(period_length),
		Some(timeout_amount),
		None,
		));
		assert_eq!(TokenTimeout::get_timeout_metadata(), Some(TimeoutMetadataInfo{
			period_length:period_length,
			timeout_amount:timeout_amount,
			swap_value_threshold: {BoundedBTreeMap::<TokenId, Balance, MaxCuratedTokens>::try_from(vec![].into_iter().map(|(k,v)| (k,v)).collect::<BTreeMap<TokenId, Balance>>()).unwrap()}
		}));

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128,
			reserved: 0__u128,
			frozen: 0__u128,
		});

		let now = System::block_number();

		assert_eq!(now, 0);


		// First timeout on empty user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - timeout_amount,
			reserved: timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: timeout_amount,
			last_timeout_block: now,
		});

		// Second timeout on user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 2*timeout_amount,
			reserved: 2*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 2*timeout_amount,
			last_timeout_block: now,
		});

		// Third timeout on user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 3*timeout_amount,
			reserved: 3*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 3*timeout_amount,
			last_timeout_block: now,
		});

		// Into the next period
		System::set_block_number(12);

		let now = System::block_number();

		// First timeout in current period on Thrice timedout user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 1*timeout_amount,
			reserved: 1*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 1*timeout_amount,
			last_timeout_block: now,
		});

		// Into the next period
		System::set_block_number(22);

		let now = System::block_number();

		// First timeout in current period on Once timedout user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 1*timeout_amount,
			reserved: 1*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 1*timeout_amount,
			last_timeout_block: now,
		});

		// Second timeout
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 2*timeout_amount,
			reserved: 2*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 2*timeout_amount,
			last_timeout_block: now,
		});

		// Into a far off period
		System::set_block_number(22214);

		let now = System::block_number();

		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 1*timeout_amount,
			reserved: 1*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 1*timeout_amount,
			last_timeout_block: now,
		});

	})
}

#[test]
fn release_timeout_works() {
	new_test_ext().execute_with(|| {

		let period_length = 10;
		let timeout_amount= 1000;

		let caller: u128 = 0u128;
		let initial_amount: Balance = 2_000_000__u128;
		let token_id: TokenId = <Test as Config>::Tokens::create(&caller, initial_amount.into()).unwrap().into();

		// The Native token id
		assert_eq!(token_id, NATIVE_CURRENCY_ID);

		// We initaite the threshold map as empty as it is not required here
		assert_ok!(TokenTimeout::update_timeout_metadata(Origin::root(),
		Some(period_length),
		Some(timeout_amount),
		None,
		));
		assert_eq!(TokenTimeout::get_timeout_metadata(), Some(TimeoutMetadataInfo{
			period_length:period_length,
			timeout_amount:timeout_amount,
			swap_value_threshold: {BoundedBTreeMap::<TokenId, Balance, MaxCuratedTokens>::try_from(vec![].into_iter().map(|(k,v)| (k,v)).collect::<BTreeMap<TokenId, Balance>>()).unwrap()}
		}));

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128,
			reserved: 0__u128,
			frozen: 0__u128,
		});

		let now = System::block_number();

		assert_eq!(now, 0);

		
		assert_noop!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128), Error::<Test>::NotTimedout);
		assert_noop!(TokenTimeout::release_timeout(Origin::signed(0u128).into()), Error::<Test>::NotTimedout);

		// First timeout on empty user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - timeout_amount,
			reserved: timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: timeout_amount,
			last_timeout_block: now,
		});

		// Second timeout on user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 2*timeout_amount,
			reserved: 2*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 2*timeout_amount,
			last_timeout_block: now,
		});

		// Third timeout on user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 3*timeout_amount,
			reserved: 3*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 3*timeout_amount,
			last_timeout_block: now,
		});

		assert_noop!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128), Error::<Test>::CantReleaseYet);
		assert_noop!(TokenTimeout::release_timeout(Origin::signed(0u128).into()), Error::<Test>::CantReleaseYet);

		// Into the next period
		System::set_block_number(12);

		let now = System::block_number();
		
		assert_ok!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128));
		assert_ok!(TokenTimeout::release_timeout(Origin::signed(0u128).into()));

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 0*timeout_amount,
			reserved: 0*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 0*timeout_amount,
			last_timeout_block: 0*now,
		});
		
		assert_noop!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128), Error::<Test>::NotTimedout);
		assert_noop!(TokenTimeout::release_timeout(Origin::signed(0u128).into()), Error::<Test>::NotTimedout);

		// First timeout in current period on Thrice timedout user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 1*timeout_amount,
			reserved: 1*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 1*timeout_amount,
			last_timeout_block: now,
		});

		
		assert_noop!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128), Error::<Test>::CantReleaseYet);
		assert_noop!(TokenTimeout::release_timeout(Origin::signed(0u128).into()), Error::<Test>::CantReleaseYet);

		// Into the next period
		System::set_block_number(22);

		let now = System::block_number();

		// First timeout in current period on Once timedout user state
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 1*timeout_amount,
			reserved: 1*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 1*timeout_amount,
			last_timeout_block: now,
		});

		
		assert_noop!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128), Error::<Test>::CantReleaseYet);
		assert_noop!(TokenTimeout::release_timeout(Origin::signed(0u128).into()), Error::<Test>::CantReleaseYet);

		// Second timeout
		assert_ok!(
			<TokenTimeout as TimeoutTriggerTrait<_>>::process_timeout(&0u128)
		);

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 2*timeout_amount,
			reserved: 2*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 2*timeout_amount,
			last_timeout_block: now,
		});

		assert_noop!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128), Error::<Test>::CantReleaseYet);
		assert_noop!(TokenTimeout::release_timeout(Origin::signed(0u128).into()), Error::<Test>::CantReleaseYet);

		// Into a far off period
		System::set_block_number(22214);

		let now = System::block_number();

		assert_ok!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128));
		assert_ok!(TokenTimeout::release_timeout(Origin::signed(0u128).into()));

		assert_eq!(Tokens::accounts(0u128, token_id), AccountData{
			free: 2_000_000__u128 - 0*timeout_amount,
			reserved: 0*timeout_amount,
			frozen: 0__u128,
		});

		assert_eq!(TokenTimeout::get_account_timeout_data(0u128), AccountTimeoutDataInfo{
			total_timeout_amount: 0*timeout_amount,
			last_timeout_block: 0*now,
		});
		
		assert_noop!(<TokenTimeout as TimeoutTriggerTrait<_>>::can_release_timeout(&0u128), Error::<Test>::NotTimedout);
		assert_noop!(TokenTimeout::release_timeout(Origin::signed(0u128).into()), Error::<Test>::NotTimedout);

	})
}

