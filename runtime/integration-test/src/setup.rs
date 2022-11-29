use frame_support::traits::GenesisBuild;
use mangata_types::{AccountId, Balance, TokenId};
use sp_runtime::codec::Encode;

#[cfg(feature = "with-kusama-runtime")]
pub use kusama_imports::*;

#[cfg(feature = "with-kusama-runtime")]
mod kusama_imports {
	pub use mangata_kusama_runtime::{
		AssetMetadataOf, Bootstrap, Runtime, RuntimeOrigin, System, UNIT,
	};
	use mangata_types::TokenId;

	pub const NATIVE_ASSET_ID: TokenId = mangata_kusama_runtime::MGX_TOKEN_ID;
}

/// Accounts
pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];

const PARA_ID: u32 = 2110;

pub struct ExtBuilder {
	pub parachain_id: u32,
	pub assets: Vec<(TokenId, AssetMetadataOf)>,
	pub balances: Vec<(AccountId, TokenId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { parachain_id: PARA_ID, assets: vec![], balances: vec![] }
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let _ = env_logger::builder().is_test(true).try_init();

		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			tokens_endowment: self.balances,
			created_tokens_for_staking: vec![],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let encoded: Vec<(TokenId, Vec<u8>)> = self
			.assets
			.iter()
			.map(|(id, meta)| {
				let encoded = AssetMetadataOf::encode(&meta);
				(*id, encoded)
			})
			.collect();

		orml_asset_registry::GenesisConfig::<Runtime> { assets: encoded }
			.assimilate_storage(&mut t)
			.unwrap();

		<parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&parachain_info::GenesisConfig { parachain_id: self.parachain_id.into() },
			&mut t,
		)
		.unwrap();

		<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_xcm::GenesisConfig { safe_xcm_version: Some(2) },
			&mut t,
		)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
