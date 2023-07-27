#![cfg_attr(not(feature = "std"), no_std)]
use super::*;
use frame_support::traits::OnRuntimeUpgrade;
use log::info;
use sp_runtime::traits::Zero;

pub struct AssetRegistryMigration;
impl OnRuntimeUpgrade for AssetRegistryMigration {
	fn on_runtime_upgrade() -> Weight {
		info!(
			target: "asset_registry",
			"on_runtime_upgrade: Attempted to apply AssetRegistry migration"
		);

		let mut weight: Weight = Weight::zero();

		orml_asset_registry::Metadata::<Runtime>::translate(
			|token_id, meta: AssetMetadataOf| {
				weight.saturating_accrue(
					<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 1),
				);

				let issuance = orml_tokens::Pallet::<Runtime>::total_issuance(token_id);
				let name = sp_std::str::from_utf8(&meta.name);
				if issuance.is_zero() && name.map_or(false, |n| n.starts_with("Liquidity")) {
					// By returning None from f for an element, we’ll remove it from the map.
					// Based on the docs of translate method
					None
				} else {
					Some(meta)
				}
			},
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		info!(
			target: "asset_registry",
			"pre_upgrade: checks"
		);
		let mut has_zero_issuance: Vec<u32> = vec![];
		orml_asset_registry::Metadata::<Runtime>::iter().for_each(|(token_id, meta)| {
			let issuance = orml_tokens::Pallet::<Runtime>::total_issuance(token_id);
			let name = sp_std::str::from_utf8(&meta.name);
			if issuance.is_zero() && name.map_or(false, |n| n.starts_with("Liquidity")) {
				has_zero_issuance.push(token_id);
			}
		});

		assert!(!has_zero_issuance.is_empty(), "No migration is required as we have identified only those liquidity assets with non-zero issuance.");

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
