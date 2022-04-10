use super::*;
use mock::*;

use serial_test::serial;
use frame_support::assert_err;
use sp_runtime::traits::BadOrigin;

const USER_ID: u128 = 0;
const INITIAL_AMOUNT: u128 = 1_000_000;
const DUMMY_ID: u32 = 2;


fn set_up() {
	let mga_id = Ido::create_new_token(&USER_ID, INITIAL_AMOUNT);
	let ksm_id = Ido::create_new_token(&USER_ID, INITIAL_AMOUNT);
	let dummy_id = Ido::create_new_token(&USER_ID, INITIAL_AMOUNT);
	assert_eq!(mga_id, MGAId::get());
	assert_eq!(ksm_id, KSMId::get());
	assert_eq!(dummy_id, DUMMY_ID);
	assert_eq!(INITIAL_AMOUNT, Ido::balance( KSMId::get(), USER_ID));
	assert_eq!(INITIAL_AMOUNT, Ido::balance( MGAId::get(), USER_ID));

}

fn jump_to_whitelist_phase() {
	let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
	pool_exists_mock.expect().return_const(false);
	Ido::start_ido(
		Origin::root(),
		10_u32.into(),
		10,
		10,
		1_u128,
		10000_u128,
	).unwrap();
	Ido::on_initialize(15_u32.into());
	assert_eq!(IDOPhase::Whitelist, Phase::<Test>::get());
}

fn jump_to_public_phase() {
	let pool_exists_mock = MockPoolCreateApiMock::pool_exists_context();
	pool_exists_mock.expect().return_const(false);

	Ido::start_ido(
		Origin::root(),
		10_u32.into(),
		10,
		10,
		1_u128,
		10000_u128,
	).unwrap();
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
fn test_dont_allow_for_ksm_donation_before_minimal_valuation_fro_mga_is_provided() {
	new_test_ext().execute_with(|| {
		set_up();

		jump_to_public_phase();

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 1).unwrap();
		assert_eq!(1, Ido::donations(USER_ID, MGAId::get()));
		assert_eq!((1,0), Ido::valuations());

		assert_err!(
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), 1),
			Error::<Test>::ValuationRatio
		);

		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 9999).unwrap();
		assert_eq!(10000, Ido::donations(USER_ID, MGAId::get()));
		assert_eq!((10000,0), Ido::valuations());

		Ido::donate(Origin::signed(USER_ID), KSMId::get(), 1).unwrap();
		assert_eq!(1, Ido::donations(USER_ID, KSMId::get()));
		assert_eq!((10000,1), Ido::valuations());

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
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), INITIAL_AMOUNT*2),
			Error::<Test>::NotEnoughAssets
		);

		assert_err!(
			Ido::donate(Origin::signed(USER_ID), MGAId::get(), INITIAL_AMOUNT*2),
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
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), INITIAL_AMOUNT*2),
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
			Ido::donate(Origin::signed(USER_ID), KSMId::get(), INITIAL_AMOUNT*2),
			Error::<Test>::UnauthorizedForDonation
		);
	});
}

#[test]
#[serial]
fn test_whitliested_donation() {
	new_test_ext().execute_with(|| {
		set_up();

		jump_to_whitelist_phase();	

		Ido::whitelist_accounts(Origin::root(), vec![USER_ID]).unwrap();
		Ido::donate(Origin::signed(USER_ID), MGAId::get(), 1000).unwrap();
	});
}

#[test]
#[serial]
fn test_non_root_user_can_not_start_ido() {
	new_test_ext().execute_with(|| {
		set_up();
		assert_err!(
			Ido::start_ido(
			Origin::signed(USER_ID),
			0_u32.into(),
			1,
			1,
			1_u128,
			10000_u128,
			),
			BadOrigin
		);
	});
}

#[test]
#[serial]
fn test_non_root_user_can_not_whitelist_accounts() {
	new_test_ext().execute_with(|| {
		set_up();
		assert_err!(
			Ido::whitelist_accounts(
			Origin::signed(USER_ID),
			vec![],
			),
			BadOrigin
		);
	});
}

#[test]
#[serial]
fn test_only_root_can_whitelist_accounts() {
	new_test_ext().execute_with(|| {
		set_up();
		Ido::whitelist_accounts(
			Origin::root(),
			vec![]
		).unwrap();
	});
}

#[test]
#[serial]
fn test_ido_start_cannot_happen_in_the_past() {
	new_test_ext().execute_with(|| {
		set_up();
		System::set_block_number(1000);
		assert_err!(
			Ido::start_ido(
				Origin::root(),
				999_u32.into(),
				1,
				1,
				1_u128,
				10000_u128,
			),
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
			Ido::start_ido(
				Origin::root(),
				100_u32.into(),
				0,
				1,
				1_u128,
				10000_u128,
			),
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
			Ido::start_ido(
				Origin::root(),
				100_u32.into(),
				1,
				0,
				1_u128,
				10000_u128,
			),
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

		Ido::start_ido(
			Origin::root(),
			100_u32.into(),
			10,
			20,
			1_u128,
			10000_u128,
		).unwrap();

		Ido::on_initialize(100_u32.into());

		assert_err!(
			Ido::start_ido(
				Origin::root(),
				100_u32.into(),
				10,
				20,
				1_u128,
				10000_u128,
			),
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

		Ido::start_ido(
			Origin::root(),
			BOOTSTRAP_WHITELIST_START.into(),
			(BOOTSTRAP_PUBLIC_START - BOOTSTRAP_WHITELIST_START).try_into().unwrap(),
			(BOOTSTRAP_FINISH - BOOTSTRAP_PUBLIC_START).try_into().unwrap(),
			1_u128,
			10000_u128,
		).unwrap();

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
			1_u128,
			10000_u128,
		).unwrap();

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
			Ido::start_ido(
				Origin::root(),
				u64::MAX.into(),
				u32::MAX,
				1_u32,
				1_u128,
				10000_u128,
			),
			Error::<Test>::MathOverflow
		);

		assert_err!(
			Ido::start_ido(
				Origin::root(),
				u64::MAX.into(),
				1_u32,
				u32::MAX,
				1_u128,
				10000_u128,
			),
			Error::<Test>::MathOverflow
		);

		assert_err!(
			Ido::start_ido(
				Origin::root(),
				u64::MAX.into(),
				u32::MAX,
				u32::MAX,
				1_u128,
				10000_u128,
			),
			Error::<Test>::MathOverflow
		);

	});
}
