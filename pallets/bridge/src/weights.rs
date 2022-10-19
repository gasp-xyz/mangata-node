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
		Weight::from_ref_time(52_195_000)
			.saturating_add(DbWeight::get().reads(3 as u64))
			.saturating_add(DbWeight::get().writes(2 as u64))
	}
	fn update_registry_with_current_app_id() -> Weight {
		Weight::from_ref_time(37_172_000)
			.saturating_add(DbWeight::get().reads(1 as u64))
			.saturating_add(DbWeight::get().writes(2 as u64))
	}
}
