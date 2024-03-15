pub use std::default::Default;

pub use frame_support::{
	assert_err, assert_noop, assert_ok, dispatch::DispatchResultWithPostInfo, error::BadOrigin,
	traits::OnInitialize,
};
pub use orml_traits::currency::{MultiCurrency, MultiCurrencyExtended};
pub use sp_io::TestExternalities;
pub use sp_runtime::{codec::Encode, BoundedVec, BuildStorage, MultiAddress, Permill};

pub use rollup_imports::*;

mod rollup_imports {
	pub use rollup_runtime::{
		consts::UNIT,
		runtime_config::config::{orml_asset_registry::AssetMetadataOf, pallet_proxy::ProxyType},
		AccountId, AssetRegistry, Balance, Bootstrap, CustomMetadata, Identity, ProofOfStake,
		Proxy, Runtime, RuntimeCall, RuntimeOrigin, System, TokenId, Tokens, UncheckedExtrinsic,
		Xyk, XykMetadata,
	};

	pub const NATIVE_ASSET_ID: TokenId = rollup_runtime::runtime_config::tokens::RX_TOKEN_ID;
}

/// Accounts
pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];

pub fn unit(decimals: u32) -> Balance {
	10u128.saturating_pow(decimals)
}

pub fn cent(decimals: u32) -> Balance {
	unit(decimals) / 100
}

pub fn millicent(decimals: u32) -> Balance {
	cent(decimals) / 1000
}

pub fn microcent(decimals: u32) -> Balance {
	millicent(decimals) / 1000
}

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
	pub fn assets(mut self, assets: Vec<(TokenId, AssetMetadataOf)>) -> Self {
		self.assets = assets;
		self
	}

	pub fn balances(mut self, balances: Vec<(AccountId, TokenId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn parachain_id(mut self, parachain_id: u32) -> Self {
		self.parachain_id = parachain_id;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let _ = env_logger::builder().is_test(true).try_init();

		let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			tokens_endowment: self.balances,
			created_tokens_for_staking: vec![],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		parachain_staking::GenesisConfig::<Runtime>::default()
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

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
