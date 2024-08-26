use crate::setup::*;
use sp_runtime::testing::H256;

fn test_env() -> TestExternalities {
	ExtBuilder { ..ExtBuilder::default() }.build()
}

#[test]
fn system_set_code_works_with_maintenance_mode() {
	let mut ext = test_env();
	ext.execute_with(|| {
		System::set_block_number(1);
		assert_ok!(System::set_code_without_checks(
			frame_system::RawOrigin::Root.into(),
			vec![1, 2, 3, 4],
		));

		assert_ok!(Maintenance::switch_maintenance_mode_on(
			RuntimeOrigin::signed(
				FoundationAccountsProvider::<Runtime>::get()
					.pop()
					.expect("There atleast 1 F acc")
			)
			.into()
		));
		assert_err!(
			System::set_code_without_checks(frame_system::RawOrigin::Root.into(), vec![1, 2, 3, 4],),
			pallet_maintenance::Error::<Runtime>::UpgradeBlockedByMaintenance
		);

		assert_ok!(Maintenance::switch_upgradability_in_maintenance_mode_on(
			RuntimeOrigin::signed(
				FoundationAccountsProvider::<Runtime>::get()
					.pop()
					.expect("There atleast 1 F acc")
			)
			.into()
		));
		assert_ok!(System::set_code_without_checks(
			frame_system::RawOrigin::Root.into(),
			vec![1, 2, 3, 4],
		));

		assert_ok!(Maintenance::switch_upgradability_in_maintenance_mode_off(
			RuntimeOrigin::signed(
				FoundationAccountsProvider::<Runtime>::get()
					.pop()
					.expect("There atleast 1 F acc")
			)
			.into()
		));
		assert_err!(
			System::set_code_without_checks(frame_system::RawOrigin::Root.into(), vec![1, 2, 3, 4],),
			pallet_maintenance::Error::<Runtime>::UpgradeBlockedByMaintenance
		);

		assert_ok!(Maintenance::switch_maintenance_mode_off(
			RuntimeOrigin::signed(
				FoundationAccountsProvider::<Runtime>::get()
					.pop()
					.expect("There atleast 1 F acc")
			)
			.into()
		));
		assert_ok!(System::set_code_without_checks(
			frame_system::RawOrigin::Root.into(),
			vec![1, 2, 3, 4],
		));
	});
}
