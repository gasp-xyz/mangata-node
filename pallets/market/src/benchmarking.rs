use super::*;

use crate::Pallet as Market;
use frame_benchmarking::{v2::*, whitelisted_caller};
use frame_system::RawOrigin as SystemOrigin;

const UNIT: u128 = 1_000_000_000_000_000_000;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn create_asset<T: Config>(who: &T::AccountId) -> T::CurrencyId
where
	T::Balance: From<u128>,
{
	T::Currency::create(who, (1_000 * UNIT).into()).expect("is ok")
}

#[benchmarks(where <T as Config>::Balance: From<u128>)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_pool(x: Linear<0,1>) {
        let kind: PoolKind = match x {
            0 => PoolKind::Xyk,
            _ => PoolKind::StableSwap,
        };
		let caller: T::AccountId = whitelisted_caller();
		let asset1 = create_asset::<T>(&caller);
		let asset2 = create_asset::<T>(&caller);
		let lp_token = T::Currency::get_next_currency_id();

		#[extrinsic_call]
		_(
			SystemOrigin::Signed(caller.clone()),
			kind,
			asset1,
			UNIT.into(),
			asset2,
			UNIT.into(),
		);
		let lp_supply = T::Currency::total_issuance(lp_token);

		assert_last_event::<T>(
			Event::LiquidityMinted {
				who: caller,
				pool_id: lp_token,
				amounts_provided: (UNIT.into(), UNIT.into()),
				lp_token,
				lp_token_minted: lp_supply,
				total_supply: lp_supply,
			}
			.into(),
		);
	}

	impl_benchmark_test_suite!(Market, crate::mock::new_test_ext(), crate::mock::Test);
}
