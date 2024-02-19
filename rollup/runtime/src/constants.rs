#![cfg_attr(not(feature = "std"), no_std)]
pub mod fee {
	use crate::{runtime_config::consts::UNIT, weights::VerExtrinsicBaseWeight, Balance};
	use frame_support::weights::{
		constants::WEIGHT_REF_TIME_PER_SECOND, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	};
	use smallvec::smallvec;
	use sp_runtime::Perbill;

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
	///   - `[Balance::min, Balance::max]`
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
			// in mangata, we map to 1/10 of that, or 1/10 MILLIUNIT
			let p = base_tx_in_mgx();
			let q = Balance::from(VerExtrinsicBaseWeight::get().ref_time());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}

	pub fn base_tx_in_mgx() -> Balance {
		UNIT
	}

	pub fn mgx_per_second() -> u128 {
		let base_weight = Balance::from(VerExtrinsicBaseWeight::get().ref_time());
		let base_per_second = (WEIGHT_REF_TIME_PER_SECOND / base_weight as u64) as u128;
		base_per_second * base_tx_in_mgx()
	}
}
