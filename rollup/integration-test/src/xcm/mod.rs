// #[cfg(feature = "with-kusama-runtime")]
// pub mod kusama_test_net;

// #[cfg(feature = "with-kusama-runtime")]
// pub mod kusama_xcm_transfer;

pub use fee_test::{asset_unit_cost, native_per_second_as_fee, relay_per_second_as_fee};
use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};
use sp_runtime::{FixedPointNumber, FixedU128};

// N * unit_weight * (weight/10^12) * per_second
fn asset_weight(instruction_count: u32, unit_weight: Weight, per_second: u128) -> u128 {
	let weight = unit_weight.saturating_mul(instruction_count as u64);
	let weight_ratio =
		FixedU128::saturating_from_rational(weight.ref_time(), WEIGHT_REF_TIME_PER_SECOND);
	weight_ratio.saturating_mul_int(per_second)
}

mod fee_test {
	use super::asset_weight;
	use crate::setup::*;

	pub fn asset_unit_cost(instruction_count: u32, per_second: u128) -> u128 {
		#[cfg(feature = "with-kusama-runtime")]
		let unit_weight: Weight = common_runtime::xcm_config::UnitWeightCost::get();
		#[cfg(feature = "with-kusama-runtime")]
		assert_eq!(unit_weight, Weight::from_parts(150_000_000, 0));

		asset_weight(instruction_count, unit_weight, per_second)
	}

	pub fn relay_per_second_as_fee(instruction_count: u32) -> u128 {
		#[cfg(feature = "with-kusama-runtime")]
		let relay_per_second = common_runtime::ksm_per_second();
		#[cfg(feature = "with-kusama-runtime")]
		assert_eq!(8_714 * 10 * millicent(12), relay_per_second);

		asset_unit_cost(instruction_count, relay_per_second)
	}

	pub fn native_per_second_as_fee(instruction_count: u32) -> u128 {
		#[cfg(feature = "with-kusama-runtime")]
		let native_per_second = common_runtime::mgx_per_second();
		#[cfg(feature = "with-kusama-runtime")]
		assert_eq!(8_714 * unit(18), native_per_second);

		asset_unit_cost(instruction_count, native_per_second)
	}

	#[cfg(feature = "with-kusama-runtime")]
	#[test]
	fn mangata_kusama_per_second_works() {
		assert_eq!(52_284 * microcent(12), relay_per_second_as_fee(4));
		assert_eq!(52_284 * 10 * millicent(18), native_per_second_as_fee(4));
	}
}

#[test]
fn weight_to_fee_works() {
	#[cfg(any(feature = "with-kusama-runtime"))]
	use frame_support::weights::{Weight, WeightToFee as WeightToFeeT};

	// Kusama
	#[cfg(feature = "with-kusama-runtime")]
	{
		use kusama_runtime_constants::fee::WeightToFee;

		let base_weight: Weight = kusama_runtime::xcm_config::BaseXcmWeight::get();
		assert_eq!(base_weight, Weight::from_parts(1_000_000_000, 65536));

		let weight: Weight = base_weight.saturating_mul(4);
		let fee = WeightToFee::weight_to_fee(&weight);
		assert_eq!(1_069_181_380, fee);

		// transfer_to_relay_chain weight in KusamaRelay
		let weight: Weight = Weight::from_parts(299_506_000, 0);
		let fee = WeightToFee::weight_to_fee(&weight);
		assert_eq!(80_056_560, fee);
	}

	// Mangata
	#[cfg(feature = "with-kusama-runtime")]
	{
		use common_runtime::constants::fee::WeightToFee;

		let base_weight: Weight = common_runtime::xcm_config::BaseXcmWeight::get();
		assert_eq!(base_weight, Weight::from_parts(100_000_000, 0));

		let unit_weight: Weight = common_runtime::xcm_config::UnitWeightCost::get();
		assert_eq!(unit_weight, Weight::from_parts(150_000_000, 0));

		let weight: Weight = base_weight.saturating_mul(4);
		let fee = WeightToFee::weight_to_fee(&weight);
		assert_eq!(3_485_656_523_406_183_554, fee);

		let weight: Weight = unit_weight.saturating_mul(4);
		let fee = WeightToFee::weight_to_fee(&weight);
		assert_eq!(5_228_484_785_109_275_332, fee);
	}
}
