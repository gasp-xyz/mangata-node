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
//! DATE: 2022-12-30 (Y/M/D)
//! HOSTNAME: `ecd803424b9c`, CPU: `AMD EPYC 7B13`
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
	///   Min, Max: 2_366_040, 2_458_889
	///   Average:  2_389_834
	///   Median:   2_388_079
	///   Std-Dev:  16515.8
	///
	/// Percentiles nanoseconds:
	///   99th: 2_456_300
	///   95th: 2_410_270
	///   75th: 2_394_860
	pub const BlockExecutionWeight: Weight = WEIGHT_PER_NANOS.saturating_mul(2_389_834);
}
