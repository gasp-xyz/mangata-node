#![cfg_attr(not(feature = "std"), no_std)]
use super::*;
use frame_support::{
	storage::{
		migration::{move_prefix, move_storage_from_pallet, storage_key_iter},
		storage_prefix, unhashed,
	},
	traits::OnRuntimeUpgrade,
	StoragePrefixedMap, Twox64Concat,
};
use xcm::IntoVersion;

pub fn move_storage_from_pallet_with_rename(
	old_storage_name: &[u8],
	new_storage_name: &[u8],
	old_pallet_name: &[u8],
	new_pallet_name: &[u8],
) {
	let new_prefix = storage_prefix(new_pallet_name, new_storage_name);
	let old_prefix = storage_prefix(old_pallet_name, old_storage_name);

	move_prefix(&old_prefix, &new_prefix);

	if let Some(value) = unhashed::get_raw(&old_prefix) {
		unhashed::put_raw(&new_prefix, &value);
		unhashed::kill(&old_prefix);
	}
}

pub struct XykRefactorMigration;
impl OnRuntimeUpgrade for XykRefactorMigration {
	fn on_runtime_upgrade() -> Weight {
		log::info!(
			target: "proof_of_stake",
			"on_runtime_upgrade: Attempted to apply xyk refactor migration"
		);

		move_storage_from_pallet_with_rename(
			b"PromotedPoolsRewardsV2",
			b"PromotedPoolRewards",
			b"Issuance",
			b"ProofOfStake",
		);
		move_storage_from_pallet_with_rename(
			b"LiquidityMiningActivePoolV2",
			b"TotalActivatedLiquidity",
			b"ProofOfStake",
			b"ProofOfStake",
		);

		<Runtime as frame_system::Config>::BlockWeights::get().max_block
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		sp_runtime::runtime_logger::RuntimeLogger::init();
		log::info!(
			target: "proof_of_stake",
			"pre_upgrade check: proof_of_stake"
		);

		let pos__liquidity_mining_avitvate_pool_v2_count =
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				|_| Ok(()),
			)
			.count();

		let pos__total_activated_liquidity = frame_support::storage::KeyPrefixIterator::new(
			storage_prefix(b"ProofOfStake", b"TotalActivatedLiquidity").to_vec(),
			storage_prefix(b"ProofOfStake", b"TotalActivatedLiquidity").to_vec(),
			|_| Ok(()),
		)
		.count();

		let issuance__promoted_pool_reards_v2_exists =
			unhashed::get_raw(&storage_prefix(b"Issuance", b"PromotedPoolsRewardsV2")).is_some();
		let pos__promoted_pool_rewards_exists =
			unhashed::get_raw(&storage_prefix(b"ProofOfStake", b"PromotedPoolRewards")).is_some();

		log::info!(target: "migration", "PRE ProofOfStake::LiquidityMiningActivePoolV2 count  :{}", pos__liquidity_mining_avitvate_pool_v2_count);
		log::info!(target: "migration", "PRE Issuance::PromotedPoolsRewardsV2         exists  :{}", issuance__promoted_pool_reards_v2_exists);
		log::info!(target: "migration", "PRE ProofOfStake::RewardsInfo                count  :{}", pos__total_activated_liquidity);
		log::info!(target: "migration", "PRE Issuance::PromotedPoolRewards           exists  :{}", pos__promoted_pool_rewards_exists);

		assert!(pos__liquidity_mining_avitvate_pool_v2_count > 0);
		assert!(issuance__promoted_pool_reards_v2_exists);
		assert!(pos__total_activated_liquidity == 0);
		assert!(!pos__promoted_pool_rewards_exists);

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
		sp_runtime::runtime_logger::RuntimeLogger::init();
		log::info!(
			target: "proof_of_stake",
			"post_upgrade check: proof_of_stake"
		);

		let pos__liquidity_mining_avitvate_pool_v2_count =
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				|_| Ok(()),
			)
			.count();

		let pos__total_activated_liquidity = frame_support::storage::KeyPrefixIterator::new(
			storage_prefix(b"ProofOfStake", b"TotalActivatedLiquidity").to_vec(),
			storage_prefix(b"ProofOfStake", b"TotalActivatedLiquidity").to_vec(),
			|_| Ok(()),
		)
		.count();

		let issuance__promoted_pool_reards_v2_exists =
			unhashed::get_raw(&storage_prefix(b"Issuance", b"PromotedPoolsRewardsV2")).is_some();
		let pos__promoted_pool_rewards_exists =
			unhashed::get_raw(&storage_prefix(b"ProofOfStake", b"PromotedPoolRewards")).is_some();

		log::info!(target: "migration", "POST ProofOfStake::LiquidityMiningActivePoolV2 count  :{}", pos__liquidity_mining_avitvate_pool_v2_count);
		log::info!(target: "migration", "POST Issuance::PromotedPoolsRewardsV2         exists  :{}", issuance__promoted_pool_reards_v2_exists);
		log::info!(target: "migration", "POST ProofOfStake::RewardsInfo                count  :{}", pos__total_activated_liquidity);
		log::info!(target: "migration", "POST Issuance::PromotedPoolRewards           exists  :{}", pos__promoted_pool_rewards_exists);

		assert!(pos__liquidity_mining_avitvate_pool_v2_count == 0);
		assert!(!issuance__promoted_pool_reards_v2_exists);
		assert!(pos__total_activated_liquidity > 0);
		assert!(pos__promoted_pool_rewards_exists);

		Ok(())
	}
}

/// AssetRegistry migrate v2 to v3
pub struct AssetRegistryMigration;
impl OnRuntimeUpgrade for AssetRegistryMigration {
	fn on_runtime_upgrade() -> Weight {
		log::info!(
			target: "asset_registry",
			"on_runtime_upgrade: Attempted to apply AssetRegistry v2 to v3 migration"
		);

		let mut weight: Weight = Weight::zero();

		// migrate the value type of ForeignAssetLocations
		orml_asset_registry::Metadata::<Runtime>::translate(
			|_key, mut old_meta: AssetMetadataOf| {
				weight.saturating_accrue(
					<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 1),
				);
				// let mut new_meta = old_meta.clone();
				let new_location = old_meta.location.and_then(|l| l.into_version(3).ok());
				old_meta.location = new_location;
				Some(old_meta)
			},
		);

		// migrate the key type of LocationToCurrencyIds
		let module_prefix = orml_asset_registry::LocationToAssetId::<Runtime>::module_prefix();
		let storage_prefix = orml_asset_registry::LocationToAssetId::<Runtime>::storage_prefix();
		let old_data = storage_key_iter::<xcm::v2::MultiLocation, TokenId, Twox64Concat>(
			module_prefix,
			storage_prefix,
		)
		.drain()
		.collect::<sp_std::vec::Vec<_>>();
		for (old_key, value) in old_data {
			weight.saturating_accrue(
				<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 1),
			);
			let new_key: MultiLocation = old_key.try_into().expect("Stored xcm::v2::MultiLocation");
			orml_asset_registry::LocationToAssetId::<Runtime>::insert(new_key, value);
		}

		weight
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use frame_support::{
		migration::{get_storage_value, put_storage_value},
		StorageHasher,
	};
	use mangata_types::TokenId;
	use orml_asset_registry::*;

	use crate::AssetMetadataOf;

	#[test]
	fn test_v2_to_v3_migration() {
		let t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);

			let metadata_module_prefix = Metadata::<Runtime>::module_prefix();
			let metadata_storage_prefix = Metadata::<Runtime>::storage_prefix();

			let location_to_asset_id_module_prefix = LocationToAssetId::<Runtime>::module_prefix();
			let location_to_asset_id_storage_prefix =
				LocationToAssetId::<Runtime>::storage_prefix();

			let meta: AssetMetadataOf = AssetMetadata {
				decimals: 0,
				name: b"Asset".to_vec(),
				symbol: b"SYM".to_vec(),
				existential_deposit: 0,
				location: None,
				additional: Default::default(),
			};

			let old_multilocation_0 = xcm::v2::MultiLocation::new(
				0,
				xcm::v2::Junctions::X1(xcm::v2::Junction::GeneralKey(
					vec![0, 0, 0, 0].try_into().unwrap(),
				)),
			);
			let old_multilocation_1 = xcm::v2::MultiLocation::new(
				1,
				xcm::v2::Junctions::X2(
					xcm::v2::Junction::Parachain(2121),
					xcm::v2::Junction::GeneralKey(vec![0, 96].try_into().unwrap()),
				),
			);
			let old_multilocation_2 = xcm::v2::MultiLocation::new(
				1,
				xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(2114)),
			);

			let new_multilocation_0 = MultiLocation::try_from(old_multilocation_0.clone()).unwrap();
			let new_multilocation_1 = MultiLocation::try_from(old_multilocation_1.clone()).unwrap();
			let new_multilocation_2 = MultiLocation::try_from(old_multilocation_2.clone()).unwrap();

			let mut meta_0 = meta.clone();
			meta_0.location = Some(old_multilocation_0.clone().versioned());
			let mut meta_1 = meta.clone();
			meta_1.location = Some(old_multilocation_1.clone().versioned());
			let mut meta_2 = meta.clone();
			meta_2.location = Some(old_multilocation_2.clone().versioned());

			let asset_id_0: TokenId = 0;
			let asset_id_1: TokenId = 11;
			let asset_id_2: TokenId = 7;

			put_storage_value(
				metadata_module_prefix,
				metadata_storage_prefix,
				&Twox64Concat::hash(&asset_id_0.encode()),
				&meta_0,
			);
			put_storage_value(
				metadata_module_prefix,
				metadata_storage_prefix,
				&Twox64Concat::hash(&asset_id_1.encode()),
				&meta_1,
			);
			put_storage_value(
				metadata_module_prefix,
				metadata_storage_prefix,
				&Twox64Concat::hash(&asset_id_2.encode()),
				&meta_2,
			);

			put_storage_value(
				location_to_asset_id_module_prefix,
				location_to_asset_id_storage_prefix,
				&Twox64Concat::hash(&old_multilocation_0.encode()),
				asset_id_0,
			);
			put_storage_value(
				location_to_asset_id_module_prefix,
				location_to_asset_id_storage_prefix,
				&Twox64Concat::hash(&old_multilocation_1.encode()),
				asset_id_1,
			);
			put_storage_value(
				location_to_asset_id_module_prefix,
				location_to_asset_id_storage_prefix,
				&Twox64Concat::hash(&old_multilocation_2.encode()),
				asset_id_2,
			);

			assert_eq!(
				get_storage_value::<TokenId>(
					location_to_asset_id_module_prefix,
					location_to_asset_id_storage_prefix,
					&Twox64Concat::hash(&old_multilocation_0.encode()),
				),
				Some(asset_id_0)
			);
			assert_eq!(
				get_storage_value::<TokenId>(
					location_to_asset_id_module_prefix,
					location_to_asset_id_storage_prefix,
					&Twox64Concat::hash(&old_multilocation_0.encode()),
				),
				Some(asset_id_1)
			);
			assert_eq!(
				get_storage_value::<TokenId>(
					location_to_asset_id_module_prefix,
					location_to_asset_id_storage_prefix,
					&Twox64Concat::hash(&old_multilocation_0.encode()),
				),
				Some(asset_id_2)
			);

			// Assert the v3 multilocation value does not exist
			// assert!(AssetRegistry::metadata(asset_id_0).unwrap().location.unwrap());
			// assert_eq!(AssetRegistry::metadata(asset_id_1).location, None);

			// Assert v3 multilocation key does not exist in LocationToCurrencyIds
			assert_eq!(AssetRegistry::location_to_asset_id(new_multilocation_0), None);
			assert_eq!(AssetRegistry::location_to_asset_id(new_multilocation_1), None);

			// Run migration
			assert_eq!(
				crate::migration::AssetRegistryMigration::on_runtime_upgrade(),
				<<Runtime as frame_system::Config>::DbWeight as Get<
					frame_support::weights::RuntimeDbWeight,
				>>::get()
				.reads_writes(6, 6)
			);

			// Assert the value type of ForeignAssetLocations has been migrated to v3 MultiLocation
			assert_eq!(
				AssetRegistry::metadata(asset_id_0).unwrap().location.unwrap(),
				new_multilocation_0.into_versioned()
			);
			assert_eq!(
				AssetRegistry::metadata(asset_id_1).unwrap().location.unwrap(),
				new_multilocation_1.into_versioned()
			);
			assert_eq!(
				AssetRegistry::metadata(asset_id_2).unwrap().location.unwrap(),
				new_multilocation_2.into_versioned()
			);

			// Assert the key type of LocationToCurrencyIds has been migrated to v3 MultiLocation
			assert_eq!(AssetRegistry::location_to_asset_id(new_multilocation_0), Some(asset_id_0));
			assert_eq!(AssetRegistry::location_to_asset_id(new_multilocation_1), Some(asset_id_1));
			assert_eq!(AssetRegistry::location_to_asset_id(new_multilocation_2), Some(asset_id_2));

			// Assert the old key does not exist anymore
			assert_eq!(
				get_storage_value::<TokenId>(
					location_to_asset_id_module_prefix,
					location_to_asset_id_storage_prefix,
					&Twox64Concat::hash(&old_multilocation_0.encode()),
				),
				None
			);
			assert_eq!(
				get_storage_value::<TokenId>(
					location_to_asset_id_module_prefix,
					location_to_asset_id_storage_prefix,
					&Twox64Concat::hash(&old_multilocation_1.encode()),
				),
				None
			);
			assert_eq!(
				get_storage_value::<TokenId>(
					location_to_asset_id_module_prefix,
					location_to_asset_id_storage_prefix,
					&Twox64Concat::hash(&old_multilocation_2.encode()),
				),
				None
			);
		});
	}
}
