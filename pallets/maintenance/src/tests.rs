use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn switching_maintenance_mode_on_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), false);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), true);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: false, is_upgradable_in_maintenance: false }
		);

		assert_noop!(Maintenance::switch_maintenance_mode_on(RuntimeOrigin::root()), BadOrigin);
		assert_noop!(
			Maintenance::switch_maintenance_mode_on(RuntimeOrigin::signed(0u128.into())),
			Error::<Test>::NotFoundationAccount
		);

		assert_ok!(Maintenance::switch_maintenance_mode_on(RuntimeOrigin::signed(999u128.into())),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), true);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), false);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: false }
		);

		assert_noop!(
			Maintenance::switch_maintenance_mode_on(RuntimeOrigin::signed(999u128.into())),
			Error::<Test>::AlreadyInMaintenanceMode
		);
	})
}

#[test]
fn switching_maintenance_mode_off_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Maintenance::switch_maintenance_mode_on(RuntimeOrigin::signed(999u128.into())),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), true);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), false);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: false }
		);

		assert_noop!(Maintenance::switch_maintenance_mode_off(RuntimeOrigin::root()), BadOrigin);
		assert_noop!(
			Maintenance::switch_maintenance_mode_off(RuntimeOrigin::signed(0u128.into())),
			Error::<Test>::NotFoundationAccount
		);

		assert_ok!(Maintenance::switch_maintenance_mode_off(RuntimeOrigin::signed(999u128.into())),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), false);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), true);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: false, is_upgradable_in_maintenance: false }
		);

		assert_noop!(
			Maintenance::switch_maintenance_mode_off(RuntimeOrigin::signed(999u128.into())),
			Error::<Test>::NotInMaintenanceMode
		);
	})
}

#[test]
fn switching_maintenance_mode_off_while_upgradable_works_correctly() {
	new_test_ext().execute_with(|| {
		assert_ok!(Maintenance::switch_maintenance_mode_on(RuntimeOrigin::signed(999u128.into())),);

		assert_ok!(Maintenance::switch_upgradability_in_maintenance_mode_on(
			RuntimeOrigin::signed(999u128.into())
		),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), true);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), true);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: true }
		);

		assert_ok!(Maintenance::switch_maintenance_mode_off(RuntimeOrigin::signed(999u128.into())),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), false);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), true);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: false, is_upgradable_in_maintenance: false }
		);
	})
}

#[test]
fn switch_upgradability_in_maintenance_mode_on_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), false);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), true);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: false, is_upgradable_in_maintenance: false }
		);

		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_on(RuntimeOrigin::signed(
				999u128.into()
			)),
			Error::<Test>::NotInMaintenanceMode
		);

		assert_ok!(Maintenance::switch_maintenance_mode_on(RuntimeOrigin::signed(999u128.into())),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), true);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), false);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: false }
		);

		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_on(RuntimeOrigin::root()),
			BadOrigin
		);
		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_on(RuntimeOrigin::signed(
				0u128.into()
			)),
			Error::<Test>::NotFoundationAccount
		);

		assert_ok!(Maintenance::switch_upgradability_in_maintenance_mode_on(
			RuntimeOrigin::signed(999u128.into())
		),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), true);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), true);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: true }
		);

		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_on(RuntimeOrigin::signed(
				999u128.into()
			)),
			Error::<Test>::AlreadyUpgradableInMaintenanceMode
		);
	})
}

#[test]
fn switch_upgradability_in_maintenance_mode_off_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_off(RuntimeOrigin::signed(
				999u128.into()
			)),
			Error::<Test>::NotInMaintenanceMode
		);

		assert_ok!(Maintenance::switch_maintenance_mode_on(RuntimeOrigin::signed(999u128.into())),);

		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_off(RuntimeOrigin::signed(
				999u128.into()
			)),
			Error::<Test>::AlreadyNotUpgradableInMaintenanceMode
		);

		assert_ok!(Maintenance::switch_upgradability_in_maintenance_mode_on(
			RuntimeOrigin::signed(999u128.into())
		),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), true);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), true);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: true }
		);

		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_off(RuntimeOrigin::root()),
			BadOrigin
		);
		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_off(RuntimeOrigin::signed(
				0u128.into()
			)),
			Error::<Test>::NotFoundationAccount
		);

		assert_ok!(Maintenance::switch_upgradability_in_maintenance_mode_off(
			RuntimeOrigin::signed(999u128.into())
		),);

		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_maintenance(), true);
		assert_eq!(<Maintenance as GetMaintenanceStatusTrait>::is_upgradable(), false);

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: false }
		);

		assert_noop!(
			Maintenance::switch_upgradability_in_maintenance_mode_off(RuntimeOrigin::signed(
				999u128.into()
			)),
			Error::<Test>::AlreadyNotUpgradableInMaintenanceMode
		);
	})
}

#[test]
fn test_triggering_maintanance_mode_through_api_triggers_an_event() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: false, is_upgradable_in_maintenance: false }
		);

		<Maintenance as SetMaintenanceModeOn>::trigger_maintanance_mode();

		assert_eq!(
			MaintenanceStatus::<Test>::get(),
			MaintenanceStatusInfo { is_maintenance: true, is_upgradable_in_maintenance: false }
		);
	})
}
