use crate::setup::*;

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

#[test]
fn rolldown_rpc_works_with_maintenance_mode() {
	use rolldown_runtime_api::runtime_decl_for_rolldown_runtime_api::RolldownRuntimeApiV1;
	test_env().execute_with(|| {
		System::set_block_number(1);
		pallet_rolldown::L2Requests::<Runtime>::insert(
			pallet_rolldown::messages::Chain::Ethereum,
			pallet_rolldown::messages::RequestId::default(),
			pallet_rolldown::L2Request::Withdrawal(Default::default()),
		);

		assert!(!Runtime::get_l2_request(pallet_rolldown::messages::Chain::Ethereum).is_empty());
		assert!(!Runtime::get_l2_request_hash(pallet_rolldown::messages::Chain::Ethereum).is_zero());

		assert_ok!(Maintenance::switch_maintenance_mode_on(
			RuntimeOrigin::signed(
				FoundationAccountsProvider::<Runtime>::get()
					.pop()
					.expect("There atleast 1 F acc")
			)
			.into()
		));
		assert!(Runtime::get_l2_request(pallet_rolldown::messages::Chain::Ethereum).is_empty());
		assert!(Runtime::get_l2_request_hash(pallet_rolldown::messages::Chain::Ethereum).is_zero());

		assert_ok!(Maintenance::switch_maintenance_mode_off(
			RuntimeOrigin::signed(
				FoundationAccountsProvider::<Runtime>::get()
					.pop()
					.expect("There atleast 1 F acc")
			)
			.into()
		));
		assert!(!Runtime::get_l2_request(pallet_rolldown::messages::Chain::Ethereum).is_empty());
		assert!(!Runtime::get_l2_request_hash(pallet_rolldown::messages::Chain::Ethereum).is_zero());
	});
}
