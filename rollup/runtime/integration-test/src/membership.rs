use crate::setup::*;
use frame_support::traits::Contains;
use sp_runtime::testing::H256;

fn test_env() -> TestExternalities {
	ExtBuilder { ..ExtBuilder::default() }.build()
}

#[test]
fn change_key_works() {
	let mut ext = test_env();
	ext.execute_with(|| {
		System::set_block_number(1);

		let alice = AccountId::from(ALICE);
		let bob = AccountId::from(BOB);

		assert_err!(
			FoundationMembers::change_key(RuntimeOrigin::signed(bob.clone()), alice),
			pallet_membership::Error::<Runtime, pallet_membership::Instance1>::NotMember,
		);

		assert!(<FoundationMembers as Contains<_>>::contains(&alice));

		assert_ok!(FoundationMembers::change_key(
			RuntimeOrigin::signed(
				FoundationAccountsProvider::get().pop().expect("There atleast 1 F acc")
			)
			.into(),
			bob,
		));

		assert!(!<FoundationMembers as Contains<_>>::contains(&alice));
		assert!(<FoundationMembers as Contains<_>>::contains(&bob));
	});
}

#[test]
fn other_fn_doesnt_work_for_root() {
	let mut ext = test_env();
	ext.execute_with(|| {
		System::set_block_number(1);

		let alice = AccountId::from(ALICE);
		let bob = AccountId::from(BOB);
		let origin: RuntimeOrigin = frame_system::RawOrigin::Root.into();

		assert_err!(FoundationMembers::add_member(origin.clone(), bob), BadOrigin,);
		assert_err!(FoundationMembers::remove_member(origin.clone(), bob), BadOrigin,);
		assert_err!(FoundationMembers::swap_member(origin.clone(), alice, bob), BadOrigin,);
		assert_err!(FoundationMembers::reset_members(origin.clone(), vec![]), BadOrigin,);
		assert_err!(FoundationMembers::set_prime(origin.clone(), bob), BadOrigin,);
		assert_err!(FoundationMembers::clear_prime(origin.clone()), BadOrigin,);
	});
}

#[test]
fn other_fn_doesnt_work_for_foundation() {
	let mut ext = test_env();
	ext.execute_with(|| {
		System::set_block_number(1);

		let alice = AccountId::from(ALICE);
		let bob = AccountId::from(BOB);
		let origin: RuntimeOrigin = RuntimeOrigin::signed(
			FoundationAccountsProvider::get().pop().expect("There atleast 1 F acc"),
		);

		assert_err!(FoundationMembers::add_member(origin.clone(), bob), BadOrigin,);
		assert_err!(FoundationMembers::remove_member(origin.clone(), bob), BadOrigin,);
		assert_err!(FoundationMembers::swap_member(origin.clone(), alice, bob), BadOrigin,);
		assert_err!(FoundationMembers::reset_members(origin.clone(), vec![]), BadOrigin,);
		assert_err!(FoundationMembers::set_prime(origin.clone(), bob), BadOrigin,);
		assert_err!(FoundationMembers::clear_prime(origin.clone()), BadOrigin,);
	});
}

#[test]
fn transfer_memebership_fns_work_for_root() {
	let mut ext = test_env();
	ext.execute_with(|| {
		System::set_block_number(1);

		let alice = AccountId::from(ALICE);
		let bob = AccountId::from(BOB);
		let origin: RuntimeOrigin = frame_system::RawOrigin::Root.into();

		assert_ok!(TransferMembers::add_member(origin.clone(), bob));
		assert_ok!(TransferMembers::swap_member(origin.clone(), bob, alice));
		assert_ok!(TransferMembers::remove_member(origin.clone(), alice));
		assert_ok!(TransferMembers::reset_members(origin.clone(), vec![]));
	});
}

#[test]
fn transfer_memebership_fns_blocked_for_root() {
	let mut ext = test_env();
	ext.execute_with(|| {
		System::set_block_number(1);

		let alice = AccountId::from(ALICE);
		let bob = AccountId::from(BOB);
		let origin: RuntimeOrigin = frame_system::RawOrigin::Root.into();

		assert_err!(TransferMembers::set_prime(origin.clone(), bob), BadOrigin,);
		assert_err!(TransferMembers::clear_prime(origin.clone()), BadOrigin,);
	});
}
