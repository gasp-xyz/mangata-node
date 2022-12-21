// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-12-21 (Y/M/D)
//! HOSTNAME: `bc0895bf85e9`, CPU: `AMD EPYC 7B13`
//!
//! SHORT-NAME: `block`, LONG-NAME: `BlockExecution`, RUNTIME: `Mangata Development`
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
	weights::{constants::WEIGHT_PER_NANOS, Weight},
};

parameter_types! {
	/// Time to execute an empty block.
	/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
	///
	/// Stats nanoseconds:
	///   Min, Max: 123_995_923, 126_175_513
	///   Average:  124_348_501
	///   Median:   124_261_224
	///   Std-Dev:  292902.24
	///
	/// Percentiles nanoseconds:
	///   99th: 125_812_824
	///   95th: 124_734_144
	///   75th: 124_418_653
	pub const BlockExecutionWeight: Weight = Weight::from_ref_time(124_348_501).saturating_mul(WEIGHT_PER_NANOS.ref_time());
}
