use super::*;
use crate::mock::{new_test_ext, BridgeModule};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use hex_literal::hex;
use sp_core::H160;

#[test]
fn adding_app_id_into_empty_registry_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			None,
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
		));
		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"]),
			Some(App::ETH)
		);
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			None,
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
		));
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB39"]),
			Some(App::ERC20)
		);
	});
}

#[test]
fn changing_app_id_without_current_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			None,
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
		));
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			None,
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
		));
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			None,
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"][..]).into()
		));
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			None,
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB00"][..]).into()
		));

		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"]),
			Some(App::ETH)
		);
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB00"]),
			Some(App::ERC20)
		);

		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"]),
			None
		);
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB39"]),
			None
		);
	});
}

#[test]
fn changing_app_id_with_current_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			None,
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
		));
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			None,
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
		));
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			Some(H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()),
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"][..]).into()
		));
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			Some(H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()),
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB00"][..]).into()
		));

		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"]),
			Some(App::ETH)
		);
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB00"]),
			Some(App::ERC20)
		);

		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"]),
			None
		);
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB39"]),
			None
		);
	});
}

#[test]
fn changing_app_id_with_wrong_current_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			None,
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
		));
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			None,
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
		));
		assert_noop!(
			BridgeModule::update_registry(
				RawOrigin::Root.into(),
				App::ETH,
				Some(
					H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"][..]).into()
				),
				H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"][..]).into()
			),
			Error::<mock::Test>::AppNotFound
		);
		assert_noop!(
			BridgeModule::update_registry(
				RawOrigin::Root.into(),
				App::ERC20,
				Some(
					H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB00"][..]).into()
				),
				H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB00"][..]).into()
			),
			Error::<mock::Test>::AppNotFound
		);

		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"]),
			Some(App::ETH)
		);
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB39"]),
			Some(App::ERC20)
		);

		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"]),
			None
		);
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB00"]),
			None
		);
	});
}

#[test]
fn changing_app_id_to_current_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			None,
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
		));
		assert_ok!(BridgeModule::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			None,
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
		));
		assert_noop!(
			BridgeModule::update_registry(
				RawOrigin::Root.into(),
				App::ETH,
				Some(
					H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
				),
				H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
			),
			Error::<mock::Test>::DifferentAppIdRequired
		);
		assert_noop!(
			BridgeModule::update_registry(
				RawOrigin::Root.into(),
				App::ERC20,
				Some(
					H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
				),
				H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
			),
			Error::<mock::Test>::DifferentAppIdRequired
		);

		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"]),
			Some(App::ETH)
		);
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB39"]),
			Some(App::ERC20)
		);

		assert_eq!(
			BridgeModule::app_registry(hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"]),
			None
		);
		assert_eq!(
			BridgeModule::app_registry(hex!["EDa338E4dC46038493b885327842fD3E301CaB00"]),
			None
		);
	});
}
