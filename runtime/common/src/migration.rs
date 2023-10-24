use crate::{config::orml_asset_registry::StringLimit, Balance, TokenId};
use frame_support::{
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, StorageVersion},
	weights::Weight,
};
use log::info;
use mangata_types::assets::CustomMetadata;
use sp_std::marker::PhantomData;

pub struct AssetRegistryMigration<Runtime>(PhantomData<Runtime>);

impl<T> OnRuntimeUpgrade for AssetRegistryMigration<T>
where
	T: orml_asset_registry::Config<
			CustomMetadata = CustomMetadata,
			AssetId = TokenId,
			Balance = Balance,
			StringLimit = StringLimit,
		> + orml_tokens::Config<CurrencyId = TokenId>,
{
	fn on_runtime_upgrade() -> Weight {
		info!(
			target: "asset_registry",
			"on_runtime_upgrade: Attempted to apply AssetRegistry migration"
		);

		let version = orml_asset_registry::Pallet::<T>::on_chain_storage_version();
		let mut weight: Weight = <T as frame_system::Config>::DbWeight::get().reads(1);
		if version == 2 {
			info!(target: "asset-registry", "No migration applied, remove")
		} else {
			StorageVersion::new(2).put::<orml_asset_registry::Pallet<T>>();
			weight
				.saturating_accrue(<T as frame_system::Config>::DbWeight::get().reads_writes(0, 1));
		}

		weight
	}
}
