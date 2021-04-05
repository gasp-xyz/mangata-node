use super::*;
use crate::Module as BridgeModule;
use frame_benchmarking::{benchmarks};
use frame_support::{assert_ok};
use sp_core::H160;
use frame_system::{RawOrigin};
use hex_literal::hex;

benchmarks! {
	_ { }

	update_registry_without_current_app_id{

		assert_ok!(
			BridgeModule::<T>::update_registry(
				RawOrigin::Root.into(),
				App::ETH,
				None,
				H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
				)
		);
		assert_ok!(
			BridgeModule::<T>::update_registry(
				RawOrigin::Root.into(),
				App::ERC20,
				None,
				H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
				)
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

		assert_ok!(
			BridgeModule::<T>::update_registry(
				RawOrigin::Root.into(),
				App::ETH,
				None,
				H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..]).into()
				)
		);
		assert_ok!(
			BridgeModule::<T>::update_registry(
				RawOrigin::Root.into(),
				App::ERC20,
				None,
				H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..]).into()
				)
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
	use crate::mock::{Test, new_test_ext};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_update_registry_without_current_app_id::<Test>());
			assert_ok!(test_benchmark_update_registry_with_current_app_id::<Test>());
		});
	}
}
