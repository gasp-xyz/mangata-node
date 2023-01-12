//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-02 (Y/M/D)
//! HOSTNAME: `6b0c83a231cd`, CPU: `AMD EPYC 7B13`
//!
//! SHORT-NAME: `extrinsic`, LONG-NAME: `ExtrinsicBase`, RUNTIME: `Mangata Development`
//! WARMUPS: `10`, REPEAT: `100`
//! WEIGHT-PATH: ``
//! WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1.0`, WEIGHT-ADD: `0`

// Executed Command:
//   target/release/mangata-node
//   benchmark
//   overhead
//   --execution
//   native
//   --chain
//   dev
//   -lblock_builder=debug
//   --base-path
//   .

use frame_support::{
	parameter_types,
	weights::{constants::WEIGHT_REF_TIME_PER_NANOS, Weight},
};

parameter_types! {
	/// Time to execute a NO-OP extrinsic, for example `System::remark`.
	/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
	///
	/// Stats nanoseconds:
	///   Min, Max: 110_553, 113_483
	///   Average:  111_477
	///   Median:   111_264
	///   Std-Dev:  681.74
	///
	/// Percentiles nanoseconds:
	///   99th: 113_085
	///   95th: 112_899
	///   75th: 111_871
	pub const ExtrinsicBaseWeight: Weight = Weight::from_ref_time(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(111_477));
}

#[cfg(test)]
mod test_weights {
	use frame_support::weights::constants;

	/// Checks that the weight exists and is sane.
	// NOTE: If this test fails but you are sure that the generated values are fine,
	// you can delete it.
	#[test]
	fn sane() {
		let w = super::ExtrinsicBaseWeight::get();

		// At least 10 µs.
		assert!(
			w.ref_time() >= 10u64 * constants::WEIGHT_REF_TIME_PER_MICROS,
			"Weight should be at least 10 µs."
		);
		// At most 1 ms.
		assert!(
			w.ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
			"Weight should be at most 1 ms."
		);
	}
}
