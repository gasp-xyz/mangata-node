#![cfg_attr(not(feature = "std"), no_std)]
use super::*;
use log::info;
use sp_runtime::traits::Zero;
use frame_support::{
	storage::{
		migration::{move_prefix, storage_key_iter},
		storage_prefix, unhashed,
	},
	traits::OnRuntimeUpgrade,
	StoragePrefixedMap, Twox64Concat,
};
use xcm::IntoVersion;

pub struct AssetRegistryMigration;
impl OnRuntimeUpgrade for AssetRegistryMigration {
	fn on_runtime_upgrade() -> Weight {
		info!(
			target: "asset_registry",
			"on_runtime_upgrade: Attempted to apply AssetRegistry migration"
		);

		let mut weight: Weight = Weight::zero();


		orml_asset_registry::Metadata::<Runtime>::translate(
			|_key, mut old_meta: AssetMetadataOf| {
				weight.saturating_accrue(
					<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 1),
				);

				let issuance = orml_tokens::Pallet::<Runtime>::total_issuance(_key);
				let name = sp_std::str::from_utf8(&old_meta.name);
				if issuance.is_zero() && name.map_or(false, |n| n.starts_with("Liquidity")) {
					// By returning None from f for an element, we’ll remove it from the map.
					// Based on the docs of translate method
					None
				} else {
					Some(old_meta)
				}
			},
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
		info!(
			target: "asset_registry",
			"post_upgrade: checks"
		);
		orml_asset_registry::Metadata::<Runtime>::iter().for_each(|(token_id, meta)| {
			let issuance = orml_tokens::Pallet::<Runtime>::total_issuance(token_id);
			let name = sp_std::str::from_utf8(&meta.name);
			if name.map_or(false, |n| n.starts_with("Liquidity")) {
				assert!(issuance > 0);
			}
		});

		Ok(())
	}
}
