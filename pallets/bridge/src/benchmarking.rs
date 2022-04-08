#![allow(unused_must_use)]

use super::*;
use crate::Module as BridgeModule;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use hex_literal::hex;
use sp_core::H160;

benchmarks! {
	_ { }

	update_registry_without_current_app_id{

		BridgeModule::<T>::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			None,
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
		);

		BridgeModule::<T>::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			None,
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
		);

		assert_eq!(
			BridgeModule::<T>::app_registry(
				hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"]
			),
			Some(App::ETH)
		);

		assert_eq!(
			BridgeModule::<T>::app_registry(
				hex!["EDa338E4dC46038493b885327842fD3E301CaB39"]
			),
			Some(App::ERC20)
		);

	}: update_registry(
		RawOrigin::Root,
		App::ETH,
		None,
		H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"][..]).into()
		)
	verify{
	}

	update_registry_with_current_app_id{

		BridgeModule::<T>::update_registry(
			RawOrigin::Root.into(),
			App::ETH,
			None,
			H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
		);

		BridgeModule::<T>::update_registry(
			RawOrigin::Root.into(),
			App::ERC20,
			None,
			H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
		);

		assert_eq!(
			BridgeModule::<T>::app_registry(
				hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"]
			),
			Some(App::ETH)
		);

		assert_eq!(
			BridgeModule::<T>::app_registry(
				hex!["EDa338E4dC46038493b885327842fD3E301CaB39"]
			),
			Some(App::ERC20)
		);

	}: update_registry(
		RawOrigin::Root,
		App::ETH,
		Some(H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()),
		H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768500"][..]).into()
		)
	verify{
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn benchmark_update_registry_without_current_app_id() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_update_registry_without_current_app_id::<Test>());
		});
	}

	#[test]
	fn benchmark_update_registry_with_current_app_id() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_update_registry_with_current_app_id::<Test>());
		});
	}
}
