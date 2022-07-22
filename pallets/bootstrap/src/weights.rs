#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_bootstrap.
pub trait WeightInfo {
	fn schedule_bootstrap() -> Weight;
	fn provision() -> Weight;
	fn provision_vested() -> Weight;
	fn claim_rewards() -> Weight;
	fn finalize() -> Weight;
}


// For backwards compatibility and tests
impl WeightInfo for () {
	fn schedule_bootstrap() -> Weight {
		(20_281_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn provision() -> Weight {
		(72_511_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(9 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
	fn provision_vested() -> Weight {
		(101_340_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(11 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	fn claim_rewards() -> Weight {
		(163_272_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(15 as Weight))
			.saturating_add(RocksDbWeight::get().writes(7 as Weight))
	}
	fn finalize() -> Weight {
		(70_962_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(12 as Weight))
	}
}
