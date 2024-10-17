//! Rolldown pallet benchmarking.

use super::*;
use crate::Pallet as Rolldown;
use frame_benchmarking::{v2::*, whitelisted_caller};
use frame_support::{
	assert_ok,
};
use frame_system::RawOrigin as SystemOrigin;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*};

#[benchmarks(where BalanceOf<T>: From<u128>)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_manual_batch_extra_fee() {

		assert_eq!(ManualBatchExtraFee::<T>::get(), BalanceOf::<T>::zero());

		#[extrinsic_call]
		_(SystemOrigin::Root, 700u128.into());
	
		assert_eq!(ManualBatchExtraFee::<T>::get(), 700u128.into());
	}


	impl_benchmark_test_suite!(Rolldown, crate::mock::new_test_ext(), crate::mock::Test);
}
