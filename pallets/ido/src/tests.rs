use super::*;
use mock::*;

use frame_support::assert_err;
use serial_test::serial;
use sp_runtime::traits::BadOrigin;

const USER_ID: u128 = 0;
const ANOTHER_USER_ID: u128 = 100;
const INITIAL_AMOUNT: u128 = 1_000_000;
const DUMMY_ID: u32 = 2;
const LIQ_TOKEN_ID: TokenId = 10_u32;
const LIQ_TOKEN_AMOUNT: Balance = 1_000_000_u128;
const POOL_CREATE_DUMMY_RETURN_VALUE: Option<(TokenId, Balance)> = Some((LIQ_TOKEN_ID, LIQ_TOKEN_AMOUNT));

fn set_up() {
	let mga_id = Ido::create_new_token(&USER_ID, INITIAL_AMOUNT);
	let ksm_id = Ido::create_new_token(&USER_ID, INITIAL_AMOUNT);
	let dummy_id = Ido::create_new_token(&USER_ID, INITIAL_AMOUNT);
	assert_eq!(mga_id, MGAId::get());
	assert_eq!(ksm_id, KSMId::get());
	assert_eq!(dummy_id, DUMMY_ID);
	assert_eq!(INITIAL_AMOUNT, Ido::balance(KSMId::get(), USER_ID));
	assert_eq!(INITIAL_AMOUNT, Ido::balance(MGAId::get(), USER_ID));
}

fn jump_to_whitelist_phase() {
	let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
	pool_exists_mock.expect().return_const(false);
	Ido::start_ido(Origin::root(), 10_u32.into(), 10, 10).unwrap();
	Ido::on_initialize(15_u32.into());
	assert_eq!(IDOPhase::Whitelist, Phase::<Test>::get());
}

fn jump_to_public_phase() {
	let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
	pool_exists_mock.expect().return_const(false);

	Ido::start_ido(Origin::root(), 10_u32.into(), 10, 10).unwrap();
	Ido::on_initialize(25_u32.into());
	assert_eq!(IDOPhase::Public, Phase::<Test>::get());
}

#[test]
#[serial]
fn do_not_allow_for_donation_in_unsupported_currency() {
	new_test_ext().execute_with(|| {
		set_up();
		assert_err!(
			Ido::donate(Origin::signed(USER_ID), DUMMY_ID, 1000),
			Error::<Test>::UnsupportedTokenId
		)
	});
}

#[test]
#[serial]
fn test_first_donation_with_ksm_fails() {
	new_test_ext().execute_with(|| {
		set_up();
		jump_to_public_phase();

		assert_err!(
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), 1),
			Error::<Test>::FirstDonationWrongToken
		);
	});
}

#[test]
#[serial]
fn test_event_is_published_after_successful_provision() {
	new_test_ext().execute_with(|| {
		set_up();
		jump_to_public_phase();
		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 1).unwrap();

		let event =
			crate::mock::Event::Ido(crate::Event::<Test>::Provisioned(
				MGAId::get(),
				1,
			));

		assert!(System::events().iter().any(|record| record.event == event));
	});
}

#[test]
#[serial]
fn test_dont_allow_for_ksm_donation_before_minimal_valuation_fro_mga_is_provided() {
	new_test_ext().execute_with(|| {
		set_up();

		jump_to_public_phase();

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 1).unwrap();
		assert_eq!(1, Ido::donations(USER_ID, MGAId::get()));
		assert_eq!((1, 0), Ido::valuations());

		assert_err!(
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), 1),
			Error::<Test>::ValuationRatio
		);

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 9999).unwrap();
		assert_eq!(10000, Ido::donations(USER_ID, MGAId::get()));
		assert_eq!((10000, 0), Ido::valuations());

		Ido::donate(Origin::signed(USER_ID), KSMId::get(), 1).unwrap();
		assert_eq!(1, Ido::donations(USER_ID, KSMId::get()));
		assert_eq!((10000, 1), Ido::valuations());

		assert_err!(
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), 1),
			Error::<Test>::ValuationRatio
		);
	});
}

#[test]
#[serial]
fn test_donation_in_supported_tokens() {
	new_test_ext().execute_with(|| {
		set_up();
		jump_to_public_phase();

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 10000).unwrap();
		assert_eq!(10000, Ido::donations(USER_ID, MGAId::get()));

		Ido::donate(Origin::signed(USER_ID), KSMId::get(), 1).unwrap();
		assert_eq!(1, Ido::donations(USER_ID, KSMId::get()));

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 20000).unwrap();
		assert_eq!(30000, Ido::donations(USER_ID, MGAId::get()));

		Ido::donate(Origin::signed(USER_ID), KSMId::get(), 2).unwrap();
		assert_eq!(3, Ido::donations(USER_ID, KSMId::get()));
	});
}

#[test]
#[serial]
fn test_donation_with_more_tokens_than_available() {
	new_test_ext().execute_with(|| {
		set_up();

		jump_to_public_phase();

		assert_err!(
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), INITIAL_AMOUNT * 2),
			Error::<Test>::NotEnoughAssets
		);

		assert_err!(
			Ido::donate(Origin::signed(USER_ID), MGAId::get(), INITIAL_AMOUNT * 2),
			Error::<Test>::NotEnoughAssets
		);
	});
}

#[test]
#[serial]
fn test_prevent_donations_in_before_start_phase() {
	new_test_ext().execute_with(|| {
		set_up();
		Phase::<Test>::put(IDOPhase::BeforeStart);

		assert_err!(
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), INITIAL_AMOUNT * 2),
			Error::<Test>::UnauthorizedForDonation
		);
	});
}

#[test]
#[serial]
fn test_prevent_donations_in_finished_phase() {
	new_test_ext().execute_with(|| {
		set_up();
		Phase::<Test>::put(IDOPhase::Finished);
		assert_err!(
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), INITIAL_AMOUNT * 2),
			Error::<Test>::UnauthorizedForDonation
		);
	});
}

#[test]
#[serial]
fn test_prevent_non_whitelited_account_to_provision_in_whitelisted_phase() {
	new_test_ext().execute_with(|| {
		set_up();

		jump_to_whitelist_phase();

		assert!(!Ido::is_whitelisted(&USER_ID));
		assert_err!(
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), 1000),
			Error::<Test>::UnauthorizedForDonation
		);

	});
}

#[test]
#[serial]
fn test_allow_non_whitelited_account_to_provision_in_whitelisted_phase_with_mga() {
	new_test_ext().execute_with(|| {
		set_up();

		jump_to_whitelist_phase();

		assert!(!Ido::is_whitelisted(&USER_ID));
		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 1000);

	});
}

#[test]
#[serial]
fn test_incremental_whitliested_donation() {
	new_test_ext().execute_with(|| {
		set_up();

		jump_to_whitelist_phase();

		Ido::whitelist_accounts(Origin::root(), vec![USER_ID]).unwrap();
		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 1000).unwrap();

		Ido::transfer(MGAId::get(), USER_ID.into(), ANOTHER_USER_ID.into(), 10_000).unwrap();
		Ido::whitelist_accounts(Origin::root(), vec![ANOTHER_USER_ID]).unwrap();

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 1000).unwrap();
		Ido::donate(Origin::signed(ANOTHER_USER_ID), MGAId::get(), 1000).unwrap();
		assert_ne!(USER_ID, ANOTHER_USER_ID);
	});
}

#[test]
#[serial]
fn test_non_root_user_can_not_start_ido() {
	new_test_ext().execute_with(|| {
		set_up();
		assert_err!(Ido::start_ido(Origin::signed(USER_ID), 0_u32.into(), 1, 1,), BadOrigin);
	});
}

#[test]
#[serial]
fn test_non_root_user_can_not_whitelist_accounts() {
	new_test_ext().execute_with(|| {
		set_up();
		assert_err!(Ido::whitelist_accounts(Origin::signed(USER_ID), vec![],), BadOrigin);
	});
}

#[test]
#[serial]
fn test_only_root_can_whitelist_accounts() {
	new_test_ext().execute_with(|| {
		set_up();
		Ido::whitelist_accounts(Origin::root(), vec![]).unwrap();
	});
}

#[test]
#[serial]
fn test_ido_start_cannot_happen_in_the_past() {
	new_test_ext().execute_with(|| {
		set_up();
		System::set_block_number(1000);
		assert_err!(
			Ido::start_ido(Origin::root(), 999_u32.into(), 1, 1,),
			Error::<Test>::IDOStartInThePast
		);
	});
}

#[test]
#[serial]
fn test_cannot_start_ido_with_whitelist_phase_length_equal_zero() {
	new_test_ext().execute_with(|| {
		set_up();
		assert_err!(
			Ido::start_ido(Origin::root(), 100_u32.into(), 0, 1,),
			Error::<Test>::PhaseLengthCannotBeZero
		);
	});
}

#[test]
#[serial]
fn test_cannot_start_ido_with_public_phase_length_equal_zero() {
	new_test_ext().execute_with(|| {
		set_up();
		assert_err!(
			Ido::start_ido(Origin::root(), 100_u32.into(), 1, 0,),
			Error::<Test>::PhaseLengthCannotBeZero
		);
	});
}

#[test]
#[serial]
fn test_bootstrap_can_be_modified_only_before_its_started() {
	new_test_ext().execute_with(|| {
		set_up();

		let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
		pool_exists_mock.expect().return_const(false);

		Ido::start_ido(Origin::root(), 100_u32.into(), 10, 20).unwrap();

		Ido::on_initialize(100_u32.into());

		assert_err!(
			Ido::start_ido(Origin::root(), 100_u32.into(), 10, 20,),
			Error::<Test>::AlreadyStarted
		);
	});
}

#[test]
#[serial]
fn test_bootstrap_state_transitions() {
	new_test_ext().execute_with(|| {
		set_up();

		const BOOTSTRAP_WHITELIST_START: u64 = 100;
		const BOOTSTRAP_PUBLIC_START: u64 = 110;
		const BOOTSTRAP_FINISH: u64 = 130;

		let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
		pool_exists_mock.expect().return_const(false);

		let pool_create_mock = MockPoolCreateApiMock::pool_create_context();
		pool_create_mock.expect().times(1).return_const(POOL_CREATE_DUMMY_RETURN_VALUE);

		Ido::start_ido(
			Origin::root(),
			BOOTSTRAP_WHITELIST_START.into(),
			(BOOTSTRAP_PUBLIC_START - BOOTSTRAP_WHITELIST_START).try_into().unwrap(),
			(BOOTSTRAP_FINISH - BOOTSTRAP_PUBLIC_START).try_into().unwrap(),
		)
		.unwrap();

		for i in 1..BOOTSTRAP_WHITELIST_START {
			Ido::on_initialize(i);
			assert_eq!(Ido::phase(), IDOPhase::BeforeStart);
		}

		Ido::on_initialize(BOOTSTRAP_WHITELIST_START);
		assert_eq!(Ido::phase(), IDOPhase::Whitelist);

		for i in BOOTSTRAP_WHITELIST_START..BOOTSTRAP_PUBLIC_START {
			Ido::on_initialize(i);
			assert_eq!(Ido::phase(), IDOPhase::Whitelist);
		}

		Ido::on_initialize(BOOTSTRAP_PUBLIC_START);
		assert_eq!(Ido::phase(), IDOPhase::Public);

		for i in BOOTSTRAP_PUBLIC_START..BOOTSTRAP_FINISH {
			Ido::on_initialize(i);
			assert_eq!(Ido::phase(), IDOPhase::Public);
		}

		Ido::on_initialize(BOOTSTRAP_FINISH);
		assert_eq!(Ido::phase(), IDOPhase::Finished);
	});
}

#[test]
#[serial]
fn test_bootstrap_state_transitions_when_on_initialized_is_not_called() {
	new_test_ext().execute_with(|| {
		set_up();

		let pool_create_mock = MockPoolCreateApiMock::pool_create_context();
		pool_create_mock.expect().times(1).return_const(POOL_CREATE_DUMMY_RETURN_VALUE);

		const BOOTSTRAP_WHITELIST_START: u64 = 100;
		const BOOTSTRAP_PUBLIC_START: u64 = 110;
		const BOOTSTRAP_FINISH: u64 = 130;
		let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
		pool_exists_mock.expect().return_const(false);

		Ido::start_ido(
			Origin::root(),
			BOOTSTRAP_WHITELIST_START.into(),
			(BOOTSTRAP_PUBLIC_START - BOOTSTRAP_WHITELIST_START).try_into().unwrap(),
			(BOOTSTRAP_FINISH - BOOTSTRAP_PUBLIC_START).try_into().unwrap(),
		)
		.unwrap();

		assert_eq!(Ido::phase(), IDOPhase::BeforeStart);
		Ido::on_initialize(200);
		assert_eq!(Ido::phase(), IDOPhase::Finished);
	});
}

#[test]
#[serial]
fn test_bootstrap_schedule_overflow() {
	new_test_ext().execute_with(|| {
		set_up();

		assert_err!(
			Ido::start_ido(Origin::root(), u64::MAX.into(), u32::MAX, 1_u32,),
			Error::<Test>::MathOverflow
		);

		assert_err!(
			Ido::start_ido(Origin::root(), u64::MAX.into(), 1_u32, u32::MAX,),
			Error::<Test>::MathOverflow
		);

		assert_err!(
			Ido::start_ido(Origin::root(), u64::MAX.into(), u32::MAX, u32::MAX,),
			Error::<Test>::MathOverflow
		);
	});
}
#[serial]
fn test_do_not_allow_for_creating_starting_bootstrap_for_existing_pool() {
	new_test_ext().execute_with(|| {
		set_up();

		let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
		pool_exists_mock.expect().return_const(true);

		assert_err!(
			Ido::start_ido(Origin::root(), 100_u32.into(), 10, 10,),
			Error::<Test>::PoolAlreadyExists
		);
	});
}

#[test]
#[serial]
fn test_crate_pool_is_called_with_proper_arguments_after_bootstrap_finish() {
	new_test_ext().execute_with(|| {
		set_up();

		use mockall::predicate::eq;
		const KSM_PROVISON: Balance = 30;
		const MGA_PROVISON: Balance = 500_000;

		let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
		pool_exists_mock.expect().return_const(false);

		let pool_create_mock = MockPoolCreateApiMock::pool_create_context();
		pool_create_mock.expect()
			.with(
				eq(KSMId::get()),
				eq(KSM_PROVISON),
				eq(MGAId::get()),
				eq(MGA_PROVISON),
			)
			.times(1).return_const(POOL_CREATE_DUMMY_RETURN_VALUE);

		Ido::start_ido(Origin::root(), 100_u32.into(), 10, 10).unwrap();

		Ido::on_initialize(110_u32.into());
		Ido::donate(Origin::signed(USER_ID), MGAId::get(), MGA_PROVISON).unwrap();
		Ido::donate(Origin::signed(USER_ID), KSMId::get(), KSM_PROVISON).unwrap();
		Ido::on_initialize(120_u32.into());
	});
}

#[test]
#[serial]
fn test_cannot_claim_rewards_when_bootstrap_is_not_finished() {
	new_test_ext().execute_with(|| {
		set_up();
		use mockall::predicate::eq;

		let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
		pool_exists_mock.expect().return_const(false);

		Ido::start_ido(Origin::root(), 100_u32.into(), 10, 10).unwrap();

		Ido::on_initialize(100_u32.into());
		assert_eq!(IDOPhase::Whitelist, Phase::<Test>::get());

		Ido::on_initialize(110_u32.into());
		assert_eq!(IDOPhase::Public, Phase::<Test>::get());

		assert_err!(
			Ido::claim_rewards(Origin::signed(USER_ID)),
			Error::<Test>::NotFinishedYet
		);
	});
}

#[test]
#[serial]
fn test_rewards_are_distributed_properly_with_single_user() {
	new_test_ext().execute_with(|| {
		use std::rc::Rc;
		use std::sync::{Arc, Mutex};
		set_up();

		const KSM_PROVISON: Balance = 10;
		const MGA_PROVISON: Balance = 100_000;
		let liq_token_id: Arc<Mutex<TokenId>> = Arc::new(Mutex::new(0_u32.into()));
		let ref_liq_token_id = liq_token_id.clone();

		let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
		pool_exists_mock.expect().return_const(false);

		let pool_create_mock = MockPoolCreateApiMock::pool_create_context();
		pool_create_mock.expect()
			.times(1).returning(move |_, ksm_amount, _, mga_amount| {
			let issuance = (ksm_amount + mga_amount) / 2;
			println!("hello world");
			let id = Ido::create_new_token(&Ido::vault_address(), issuance);
			*(ref_liq_token_id.lock().unwrap()) = id;
			Some((id, issuance))
		});

		Ido::start_ido(Origin::root(), 100_u32.into(), 10, 10).unwrap();

		Ido::on_initialize(100_u32.into());
		assert_eq!(IDOPhase::Whitelist, Phase::<Test>::get());

		Ido::on_initialize(110_u32.into());
		assert_eq!(IDOPhase::Public, Phase::<Test>::get());

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), MGA_PROVISON).unwrap();
		Ido::donate(Origin::signed(USER_ID), KSMId::get(), KSM_PROVISON).unwrap();

		Ido::on_initialize(120_u32.into());
		assert_eq!(IDOPhase::Finished, Phase::<Test>::get());


		let (mga_valuation, ksm_valuation) = Ido::valuations();
		let liquidity_token_id = *(liq_token_id.lock().unwrap());
		let liquidity_token_amount = (mga_valuation + ksm_valuation)/ 2;

		assert_eq!(
			Ido::balance( liquidity_token_id, Ido::vault_address()),
			liquidity_token_amount
		);
		assert_eq!(
			Ido::minted_liquidity(),
			(liquidity_token_id, liquidity_token_amount)
		);

		Ido::claim_rewards(Origin::signed(USER_ID)).unwrap();

		assert_eq!(
			Ido::claimed_rewards(USER_ID, MGAId::get()),
			liquidity_token_amount / 2
		);

		assert_eq!(
			Ido::claimed_rewards(USER_ID, KSMId::get()),
			liquidity_token_amount / 2
		);

		assert_eq!(
			Ido::balance(liquidity_token_id, USER_ID),
			// KSM rewards                  MGA rewards
			(liquidity_token_amount / 2) + (liquidity_token_amount / 2)
		);
	});
}

#[test]
#[serial]
fn test_rewards_are_distributed_properly_with_multiple_user() {
	new_test_ext().execute_with(|| {
		use std::rc::Rc;
		use std::sync::{Arc, Mutex};
		set_up();

		let provisioned_ev = |id, amount| {
			crate::mock::Event::Ido(crate::Event::<Test>::Provisioned(
				id,
				amount
			))
		};

		let rewards_claimed_ev = |id, amount| {
			crate::mock::Event::Ido(crate::Event::<Test>::RewardsClaimed(
				id,
				amount
			))
		};

		const USER_KSM_PROVISON: Balance = 15;
		const USER_MGA_PROVISON: Balance = 400_000;
		const ANOTHER_USER_KSM_PROVISON: Balance = 20;
		const ANOTHER_USER_MGA_PROVISON: Balance = 100_000;
		let liq_token_id: Arc<Mutex<TokenId>> = Arc::new(Mutex::new(0_u32.into()));
		let ref_liq_token_id = liq_token_id.clone();

		let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
		pool_exists_mock.expect().return_const(false);

		let pool_create_mock = MockPoolCreateApiMock::pool_create_context();
		pool_create_mock.expect()
			.times(1).returning(move |_, ksm_amount, _, mga_amount| {
			let issuance = (ksm_amount + mga_amount) / 2;
			println!("hello world");
			let id = Ido::create_new_token(&Ido::vault_address(), issuance);
			*(ref_liq_token_id.lock().unwrap()) = id;
			Some((id, issuance))
		});

		Ido::start_ido(Origin::root(), 100_u32.into(), 10, 10).unwrap();

		Ido::on_initialize(100_u32.into());
		assert_eq!(IDOPhase::Whitelist, Phase::<Test>::get());

		Ido::on_initialize(110_u32.into());
		assert_eq!(IDOPhase::Public, Phase::<Test>::get());

		Ido::transfer(MGAId::get(), USER_ID.into(), ANOTHER_USER_ID.into(), 500_000).unwrap();
		Ido::transfer(KSMId::get(), USER_ID.into(), ANOTHER_USER_ID.into(), 500_000).unwrap();

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), USER_MGA_PROVISON).unwrap();
		Ido::donate(Origin::signed(ANOTHER_USER_ID), MGAId::get(), ANOTHER_USER_MGA_PROVISON).unwrap();

		Ido::donate(Origin::signed(USER_ID), KSMId::get(), USER_KSM_PROVISON).unwrap();
		Ido::donate(Origin::signed(ANOTHER_USER_ID), KSMId::get(), ANOTHER_USER_KSM_PROVISON).unwrap();

		assert!(System::events().iter().any(|record| record.event == provisioned_ev(MGAId::get(), USER_MGA_PROVISON)));
		assert!(System::events().iter().any(|record| record.event == provisioned_ev(KSMId::get(), USER_KSM_PROVISON)));
		assert!(System::events().iter().any(|record| record.event == provisioned_ev(MGAId::get(), ANOTHER_USER_MGA_PROVISON)));
		assert!(System::events().iter().any(|record| record.event == provisioned_ev(KSMId::get(), ANOTHER_USER_KSM_PROVISON)));

		Ido::on_initialize(120_u32.into());
		assert_eq!(IDOPhase::Finished, Phase::<Test>::get());


		let (mga_valuation, ksm_valuation) = Ido::valuations();
		assert_eq!(mga_valuation, 500_000);
		assert_eq!(ksm_valuation, 35);
		let liquidity_token_id = *(liq_token_id.lock().unwrap());
		let liquidity_token_amount = (mga_valuation + ksm_valuation)/ 2;

		assert_eq!(
			Ido::balance( liquidity_token_id, Ido::vault_address()),
			liquidity_token_amount
		);
		assert_eq!(
			Ido::minted_liquidity(),
			(liquidity_token_id, liquidity_token_amount)
		);

		assert_eq!( Ido::claimed_rewards(ANOTHER_USER_ID, MGAId::get()), 0);
		assert_eq!( Ido::claimed_rewards(ANOTHER_USER_ID, KSMId::get()), 0);
		assert_eq!( Ido::claimed_rewards(USER_ID, MGAId::get()), 0);
		assert_eq!( Ido::claimed_rewards(USER_ID, KSMId::get()), 0);

		let user_expected_ksm_rewards = liquidity_token_amount / 2 * USER_KSM_PROVISON / ksm_valuation;
		let user_expected_mga_rewards = liquidity_token_amount / 2 * USER_MGA_PROVISON / mga_valuation;
		let user_expected_liq_amount = user_expected_ksm_rewards + user_expected_mga_rewards;

		let user2_expected_ksm_rewards = liquidity_token_amount / 2 * ANOTHER_USER_KSM_PROVISON / ksm_valuation;
		let user2_expected_mga_rewards = liquidity_token_amount / 2 * ANOTHER_USER_MGA_PROVISON / mga_valuation;
		let user2_expected_liq_amount = user2_expected_ksm_rewards + user2_expected_mga_rewards;

		Ido::claim_rewards(Origin::signed(USER_ID)).unwrap();
		Ido::claim_rewards(Origin::signed(ANOTHER_USER_ID)).unwrap();

		assert_eq!( Ido::claimed_rewards(USER_ID, MGAId::get()), user_expected_mga_rewards);
		assert_eq!( Ido::claimed_rewards(USER_ID, KSMId::get()), user_expected_ksm_rewards	);
		assert_eq!( Ido::claimed_rewards(ANOTHER_USER_ID, MGAId::get()), user2_expected_mga_rewards);
		assert_eq!( Ido::claimed_rewards(ANOTHER_USER_ID, KSMId::get()), user2_expected_ksm_rewards);

		assert_err!( Ido::claim_rewards(Origin::signed(USER_ID)), Error::<Test>::NothingToClaim);
		assert_err!( Ido::claim_rewards(Origin::signed(ANOTHER_USER_ID)), Error::<Test>::NothingToClaim);

		assert_eq!( Ido::balance(liquidity_token_id, USER_ID), user_expected_liq_amount);
		assert_eq!( Ido::balance(liquidity_token_id, ANOTHER_USER_ID), user2_expected_liq_amount);


		assert!(System::events().iter().any(|record| record.event == rewards_claimed_ev(liquidity_token_id, user_expected_liq_amount)));
		assert!(System::events().iter().any(|record| record.event == rewards_claimed_ev(liquidity_token_id, user2_expected_liq_amount)));

	});

}



// TODO: refactor -> claim
// TODO: rename
