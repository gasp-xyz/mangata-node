use super::*;

mod deprecated {
	use codec::MaxEncodedLen;
	use frame_support::sp_runtime::RuntimeDebug;

	use super::*;

	/// An active voter.
	#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, TypeInfo)]
	pub struct Voter<AccountId, Balance> {
		/// The members being backed.
		pub votes: Vec<AccountId>,
		/// The amount of stake placed on this vote.
		pub stake: Balance,
		/// The amount of deposit reserved for this vote.
		///
		/// To be unreserved upon removal.
		pub deposit: Balance,
	}

	impl<AccountId, Balance: Default> Default for Voter<AccountId, Balance> {
		fn default() -> Self {
			Self { votes: vec![], stake: Default::default(), deposit: Default::default() }
		}
	}

	/// A holder of a seat as either a member or a runner-up.
	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, TypeInfo)]
	pub struct SeatHolder<AccountId, Balance> {
		/// The holder.
		pub who: AccountId,
		/// The total backing stake.
		pub stake: Balance,
		/// The amount of deposit held on-chain.
		///
		/// To be unreserved upon renouncing, or slashed upon being a loser.
		pub deposit: Balance,
	}

	#[derive(
		Clone,
		Copy,
		Default,
		PartialOrd,
		Ord,
		PartialEq,
		Eq,
		Debug,
		Encode,
		Decode,
		TypeInfo,
		MaxEncodedLen,
	)]
	pub struct CustomMetadata {
		pub xcm: Option<XcmMetadata>,
	}
}

pub mod phragmen_elections {
	#[cfg(feature = "try-runtime")]
	use frame_support::{
		migration::{have_storage_value, storage_iter, storage_key_iter},
		Twox64Concat,
	};
	use frame_support::{storage::unhashed::clear_prefix, traits::OnRuntimeUpgrade};
	use sp_io::{hashing::twox_128, KillStorageResult};

	use super::*;

	pub struct PhragmenElectionsMigration;
	impl OnRuntimeUpgrade for PhragmenElectionsMigration {
		fn on_runtime_upgrade() -> Weight {
			log::info!(
				target: "phragmen_elections",
				"on_runtime_upgrade: Attempted to apply phragmen_elections migration"
			);

			let module_name = "Elections";

			let clear_result = clear_prefix(&twox_128(module_name.as_bytes()), None, None);
			log::info!(
				target: "phragmen_elections",
				"clear_result: {:?}, {:?}",
				clear_result.maybe_cursor, clear_result.loops
			);

			<Runtime as frame_system::Config>::DbWeight::get().reads_writes(
				(clear_result.loops as u64) + 1_u64,
				(clear_result.loops as u64) + 1_u64,
			)
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			log::info!(
				target: "phragmen_elections",
				"pre_upgrade check: phragmen_elections"
			);

			assert!(have_storage_value(b"Elections", b"Members", b"",));

			assert!(!have_storage_value(b"Elections", b"RunnersUp", b"",));

			assert!(!have_storage_value(b"Elections", b"Candidates", b"",));

			assert!(!have_storage_value(b"Elections", b"ElectionRounds", b"",));

			assert!(storage_key_iter::<
				<Runtime as frame_system::Config>::AccountId,
				deprecated::Voter<<Runtime as frame_system::Config>::AccountId, Balance>,
				Twox64Concat,
			>(b"Elections", b"Voting",)
			.next()
			.is_some());

			assert!(have_storage_value(b"Elections", b":__STORAGE_VERSION__:", b"",));

			Ok(())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			log::info!(
				target: "phragmen_elections",
				"post_upgrade check: phragmen_elections"
			);

			assert!(!have_storage_value(b"Elections", b"Members", b"",));

			assert!(!have_storage_value(b"Elections", b"RunnersUp", b"",));

			assert!(!have_storage_value(b"Elections", b"Candidates", b"",));

			assert!(!have_storage_value(b"Elections", b"ElectionRounds", b"",));

			assert!(storage_key_iter::<
				<Runtime as frame_system::Config>::AccountId,
				deprecated::Voter<<Runtime as frame_system::Config>::AccountId, Balance>,
				Twox64Concat,
			>(b"Elections", b"Voting",)
			.next()
			.is_none());

			assert!(!have_storage_value(b"Elections", b":__STORAGE_VERSION__:", b"",));

			Ok(())
		}
	}
}

pub mod asset_register {
	use frame_support::{log, traits::OnRuntimeUpgrade};
	#[cfg(feature = "try-runtime")]
	use frame_support::{migration::storage_key_iter, Twox64Concat};

	use super::*;

	pub struct MigrateToXykMetadata;
	impl OnRuntimeUpgrade for MigrateToXykMetadata {
		fn on_runtime_upgrade() -> Weight {
			log::info!(
				target: "asset-registry",
				"Running asset metadata xyk migration",
			);

			orml_asset_registry::Metadata::<Runtime>::translate::<
				AssetMetadata<Balance, deprecated::CustomMetadata>,
				_,
			>(|_: TokenId, v0: AssetMetadata<Balance, deprecated::CustomMetadata>| {
				Some(AssetMetadataOf {
					decimals: v0.decimals,
					name: v0.name,
					symbol: v0.symbol,
					existential_deposit: v0.existential_deposit,
					location: v0.location,
					additional: CustomMetadata { xcm: v0.additional.xcm, xyk: None },
				})
			});

			let count: u64 =
				orml_asset_registry::Metadata::<Runtime>::iter().count().try_into().unwrap();

			<Runtime as frame_system::Config>::DbWeight::get().reads_writes(count + 1, count + 1)
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			log::info!(target: "asset-registry", "AssetRegistry::pre_upgrade, check deprecated items count");
			let count = storage_key_iter::<
				TokenId,
				AssetMetadata<Balance, deprecated::CustomMetadata>,
				Twox64Concat,
			>(b"AssetRegistry", b"Metadata")
			.count();
			assert!(count > 0);
			Ok(())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			log::info!(target: "asset-registry", "AssetRegistry::post_upgrade, check upgraded count");
			let count = storage_key_iter::<TokenId, AssetMetadataOf, Twox64Concat>(
				b"AssetRegistry",
				b"Metadata",
			)
			.count();
			assert!(count > 0);
			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use frame_support::{storage, traits::OnRuntimeUpgrade};

	use crate::migration::asset_register::MigrateToXykMetadata;

	use super::*;

	#[test]
	fn test_migration_to_v1() {
		let t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);

			let v0: AssetMetadata<Balance, deprecated::CustomMetadata> = AssetMetadata {
				decimals: 12,
				name: "para A native token".as_bytes().to_vec(),
				symbol: "paraA".as_bytes().to_vec(),
				existential_deposit: 0,
				location: Some(
					MultiLocation::new(
						1,
						X2(Parachain(1), GeneralKey(vec![0].try_into().unwrap())),
					)
					.into(),
				),
				additional: deprecated::CustomMetadata {
					xcm: Some(XcmMetadata { fee_per_second: 1_000_000_000_000 }),
				},
			};

			storage::unhashed::put_raw(
				&orml_asset_registry::Metadata::<Runtime>::hashed_key_for(0_u32),
				&v0.encode(),
			);

			assert!(orml_asset_registry::Metadata::<Runtime>::get(0).is_none());

			MigrateToXykMetadata::on_runtime_upgrade();

			let v1: AssetMetadataOf = orml_asset_registry::Metadata::<Runtime>::get(0).unwrap();

			assert_eq!(v0.decimals, v1.decimals);
			assert_eq!(v0.name, v1.name);
			assert_eq!(v0.symbol, v1.symbol);
			assert_eq!(v0.existential_deposit, v1.existential_deposit);
			assert_eq!(v0.location, v1.location);
			assert_eq!(v0.additional.xcm, v1.additional.xcm);
			assert_eq!(None, v1.additional.xyk);
		});
	}
}
