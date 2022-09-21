use super::*;
use sp_runtime::{traits::ConstU32, WeakBoundedVec};

mod deprecated {
	use frame_support::sp_runtime::RuntimeDebug;

	use super::*;

	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
	pub struct AssetInfo {
		pub name: Option<Vec<u8>>,
		pub symbol: Option<Vec<u8>>,
		pub description: Option<Vec<u8>>,
		pub decimals: Option<u32>,
	}
}

pub mod asset_registry {
	#[cfg(feature = "try-runtime")]
	use frame_support::{migration::storage_key_iter, Twox64Concat};
	use frame_support::{storage::unhashed::kill_prefix, traits::OnRuntimeUpgrade};

	use sp_io::{hashing::twox_128, KillStorageResult};

	use super::*;

	pub struct AssetRegistryMigration;
	impl OnRuntimeUpgrade for AssetRegistryMigration {
		fn on_runtime_upgrade() -> Weight {
			log::info!(
				target: "asset_registry",
				"on_runtime_upgrade: Attempted to apply asset_registry migration"
			);

			// the harder way
			// fetch asset info
			// fetch locations
			// construct metadata & merge
			// add fee_per_second ?
			// register

			// the easy way, insert new data
			let metadata = vec![
				(
					0,
					AssetMetadataOf {
						decimals: 18,
						name: b"Mangata".to_vec(),
						symbol: b"MGA".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
				(
					1,
					AssetMetadataOf {
						decimals: 18,
						name: b"Ether".to_vec(),
						symbol: b"ETH".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
				(
					ROC_TOKEN_ID,
					AssetMetadataOf {
						decimals: 12,
						name: b"Rococo Native".to_vec(),
						symbol: b"ROC".to_vec(),
						additional: CustomMetadata {
							xcm: Some(XcmMetadata { fee_per_second: roc_per_second() }),
						},
						existential_deposit: Default::default(),
						location: None,
					},
				),
				// LP ROC-MGR
				(
					5,
					AssetMetadataOf {
						decimals: 18,
						name: b"LiquidityPoolToken0x00000005".to_vec(),
						symbol: b"TKN0x00000004-TKN0x00000000".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
				(
					KAR_TOKEN_ID,
					AssetMetadataOf {
						decimals: 12,
						name: b"Karura".to_vec(),
						symbol: b"KAR".to_vec(),
						additional: CustomMetadata {
							// 100:1 KAR:ROC
							xcm: Some(XcmMetadata { fee_per_second: roc_per_second() * 100 }),
						},
						existential_deposit: Default::default(),
						location: Some(
							MultiLocation::new(
								1,
								X2(
									Parachain(karura::ID),
									GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
										karura::KAR_KEY.to_vec(),
										None,
									)),
								),
							)
							.into(),
						),
					},
				),
				(
					TUR_TOKEN_ID,
					AssetMetadataOf {
						decimals: 10,
						name: b"Turing native token".to_vec(),
						symbol: b"TUR".to_vec(),
						additional: CustomMetadata {
							// 100:1 TUR:ROC, 10/12 decimals
							xcm: Some(XcmMetadata { fee_per_second: roc_per_second() }),
						},
						existential_deposit: Default::default(),
						location: Some(MultiLocation::new(1, X1(Parachain(turing::ID))).into()),
					},
				),
				(
					8,
					AssetMetadataOf {
						decimals: 18,
						name: b"LiquidityPoolToken0x00000008".to_vec(),
						symbol: b"TKN0x00000000-TKN0x00000007".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
				(
					9,
					AssetMetadataOf {
						decimals: 18,
						name: b"LiquidityPoolToken0x00000009".to_vec(),
						symbol: b"TKN0x00000000-TKN0x00000007".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
				(
					10,
					AssetMetadataOf {
						decimals: 18,
						name: b"LiquidityPoolToken0x0000000A".to_vec(),
						symbol: b"TKN0x00000000-TKN0x00000007".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
				(
					11,
					AssetMetadataOf {
						decimals: 18,
						name: b"LiquidityPoolToken0x0000000B".to_vec(),
						symbol: b"TKN0x00000004-TKN0x00000007".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
				(
					12,
					AssetMetadataOf {
						decimals: 18,
						name: b"LiquidityPoolToken0x0000000C".to_vec(),
						symbol: b"TKN0x00000006-TKN0x00000007".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
				(
					13,
					AssetMetadataOf {
						decimals: 18,
						name: b"LiquidityPoolToken0x0000000D".to_vec(),
						symbol: b"TKN0x00000006-TKN0x00000000".to_vec(),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
				),
			];

			// kill storage first
			let names = ["AssetRegistry", "AssetsInfo"];
			let mut total_rw: u32 = 0;
			for name in names {
				match kill_prefix(&twox_128(name.as_bytes()), None) {
					KillStorageResult::AllRemoved(n) => {
						total_rw += n;
					},
					KillStorageResult::SomeRemaining(n) => {
						total_rw += n;
					},
				}
			}
			log::info!(
				target: "asset_registry",
				"on_runtime_upgrade: Deprecated storage killed, entries: {}",
				total_rw
			);

			// insert new data
			for (id, metadata) in metadata.iter() {
				orml_asset_registry::Pallet::<Runtime>::do_register_asset_without_asset_processor(
					metadata.clone(),
					*id,
				)
				.expect("should not fail");
			}
			total_rw += metadata.len() as u32 + 2; // each asset + 2 locations
			log::info!(
				target: "asset_registry",
				"on_runtime_upgrade: New data inserted"
			);

			<Runtime as frame_system::Config>::DbWeight::get()
				.reads_writes(total_rw as Weight + 1, total_rw as Weight + 1)
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			log::info!(
				target: "asset_registry",
				"pre_upgrade check: asset_registry"
			);

			let asset_info_storage =
				storage_key_iter::<TokenId, deprecated::AssetInfo, Twox64Concat>(
					b"AssetsInfo",
					b"AssetsInfo",
				)
				.collect::<Vec<_>>();

			assert!(asset_info_storage.len() > 0);

			log::info!(
				target: "asset_registry",
				"pre_upgrade: asset_info_storage entries: {}",
				asset_info_storage.len()
			);

			Ok(())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			log::info!(
				target: "asset_registry",
				"post_upgrade check: asset_registry"
			);

			// old data should be cleared
			let asset_info_storage =
				storage_key_iter::<TokenId, deprecated::AssetInfo, Twox64Concat>(
					b"AssetsInfo",
					b"AssetsInfo",
				)
				.collect::<Vec<_>>();

			let asset_location_storage = storage_key_iter::<TokenId, MultiLocation, Twox64Concat>(
				b"AssetRegistry",
				b"AssetLocations",
			)
			.collect::<Vec<_>>();

			let location_to_asset_storage =
				storage_key_iter::<TokenId, MultiLocation, Twox64Concat>(
					b"AssetRegistry",
					b"LocationToCurrencyIds",
				)
				.collect::<Vec<_>>();

			assert_eq!(asset_info_storage.len(), 0);
			assert_eq!(asset_location_storage.len(), 0);
			assert_eq!(location_to_asset_storage.len(), 0);

			// new data check
			let metadata_storage = storage_key_iter::<TokenId, AssetMetadataOf, Twox64Concat>(
				b"AssetRegistry",
				b"Metadata",
			)
			.collect::<Vec<_>>();

			let locations_storage = storage_key_iter::<TokenId, MultiLocation, Twox64Concat>(
				b"AssetRegistry",
				b"LocationToAssetId",
			)
			.collect::<Vec<_>>();

			assert_eq!(metadata_storage.len(), 12);
			assert_eq!(locations_storage.len(), 2);

			Ok(())
		}
	}
}
