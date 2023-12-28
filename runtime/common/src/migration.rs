use crate::{config::orml_asset_registry::StringLimit, Balance, TokenId};
use frame_support::{
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, StorageVersion},
	weights::Weight,
};
use log::info;
use mangata_types::assets::CustomMetadata;
use sp_std::marker::PhantomData;

pub struct PalletsVersionsAlignment<Runtime>(PhantomData<Runtime>);

impl<T> OnRuntimeUpgrade for PalletsVersionsAlignment<T>
where
	T: orml_asset_registry::Config,
	T: orml_tokens::Config,
	T: pallet_maintenance::Config,
	T: orml_unknown_tokens::Config,
	T: pallet_xcm::Config,
	T: pallet_bootstrap::Config,
	T: pallet_crowdloan_rewards::Config,
	T: pallet_fee_lock::Config,
	T: cumulus_pallet_dmp_queue::Config,
	T: cumulus_pallet_xcmp_queue::Config,
{
	fn on_runtime_upgrade() -> Weight {
		info!(
			target: "migration::versions-alignment",
			"on_runtime_upgrade: Attempted to apply AssetRegistry migration"
		);

		let mut reads = 0;
		let mut writes = 0;

		// Maintanance -> 0
		// currently set to null that defaults to 0
		StorageVersion::new(0).put::<pallet_maintenance::Pallet<T>>();
		writes += 1;

		// AssetRegistry -> 2
		if orml_asset_registry::Pallet::<T>::on_chain_storage_version() == 2 {
			info!(target: "migration::asset-registry", "No migration applied, remove");
			reads += 1;
		} else {
			info!(target: "migration::asset-registry", "Migration applied");
			StorageVersion::new(2).put::<orml_asset_registry::Pallet<T>>();
			reads += 1;
			writes += 1;
		};

		//UnknwonTokens -> 2
		if orml_unknown_tokens::Pallet::<T>::on_chain_storage_version() == 2 {
			info!(target: "migration::unknown-tokens", "No migration applied, remove");
			reads += 1;
		} else {
			info!(target: "migration::unknown-tokens", "Migration applied");
			StorageVersion::new(2).put::<orml_unknown_tokens::Pallet<T>>();
			reads += 1;
			writes += 1;
		};

		// PolkadotXcm -> 1
		if pallet_xcm::Pallet::<T>::on_chain_storage_version() == 1 {
			info!(target: "migration::pallet_xcm", "No migration applied, remove");
			reads += 1;
		} else {
			info!(target: "migration::pallet_xcm", "Migration applied");
			StorageVersion::new(1).put::<pallet_xcm::Pallet<T>>();
			reads += 1;
			writes += 1;
		};

		// Bootstrap -> 2
		if pallet_bootstrap::Pallet::<T>::on_chain_storage_version() == 2 {
			info!(target: "migration::pallet_bootstrap", "No migration applied, remove");
			reads += 1;
		} else {
			info!(target: "migration::pallet_bootstrap", "Migration applied");
			StorageVersion::new(2).put::<pallet_bootstrap::Pallet<T>>();
			reads += 1;
			writes += 1;
		};

		// Crowdloan -> 1
		if pallet_crowdloan_rewards::Pallet::<T>::on_chain_storage_version() == 1 {
			info!(target: "migration::pallet_crowdloan_rewards", "No migration applied, remove");
			reads += 1;
		} else {
			info!(target: "migration::pallet_crowdloan_rewards", "Migration applied");
			StorageVersion::new(1).put::<pallet_crowdloan_rewards::Pallet<T>>();
			reads += 1;
			writes += 1;
		};

		// FeeLock -> 0
		// currently set to null that defaults to 0
		StorageVersion::new(0).put::<pallet_fee_lock::Pallet<T>>();
		writes += 1;

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		assert_eq!(orml_asset_registry::Pallet::<T>::on_chain_storage_version(), 2);
		assert_eq!(pallet_maintenance::Pallet::<T>::on_chain_storage_version(), 0);
		assert_eq!(orml_unknown_tokens::Pallet::<T>::on_chain_storage_version(), 2);
		assert_eq!(pallet_bootstrap::Pallet::<T>::on_chain_storage_version(), 2);
		assert_eq!(pallet_crowdloan_rewards::Pallet::<T>::on_chain_storage_version(), 1);
		assert_eq!(pallet_fee_lock::Pallet::<T>::on_chain_storage_version(), 0);
		assert_eq!(cumulus_pallet_dmp_queue::Pallet::<T>::on_chain_storage_version(), 2);
		assert_eq!(cumulus_pallet_xcmp_queue::Pallet::<T>::on_chain_storage_version(), 3);
		assert_eq!(pallet_xcm::Pallet::<T>::on_chain_storage_version(), 1);
		Ok(())
	}
}
