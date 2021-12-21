//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use super::*;
use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfoTrait {
	fn update_registry(current_app_id_option: Option<AppId>) -> Weight;
}

/// Weight functions for bridge.
pub struct WeightInfo;
impl WeightInfoTrait for WeightInfo {
	fn update_registry(current_app_id_option: Option<AppId>) -> Weight {
		match current_app_id_option {
			Some(_) => Self::update_registry_with_current_app_id(),
			None => Self::update_registry_without_current_app_id(),
		}
	}
}

impl WeightInfo {
	fn update_registry_without_current_app_id() -> Weight {
		(52_195_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn update_registry_with_current_app_id() -> Weight {
		(37_172_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
}
