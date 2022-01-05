use artemis_core::{App, AppId};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use mangata_runtime::{AccountId, AuraId, Balance, InflationInfo, Range, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public, H160};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<mangata_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_public_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_public_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn mangata_session_keys(keys: AuraId) -> mangata_runtime::SessionKeys {
	mangata_runtime::SessionKeys { aura: keys }
}

pub fn mangata_inflation_config() -> InflationInfo<Balance> {
	InflationInfo {
		expect: Range {
			min: 100_000_000 * 1__000_000_000_000_000_000,
			ideal: 200_000_000 * 1__000_000_000_000_000_000,
			max: 500_000_000 * 1__000_000_000_000_000_000,
		},
		annual: Range {
			min: Perbill::from_percent(4),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(5),
		},
		// 8760 hours in a year AND
		// 4 hours in a round => 2190
		round: Range {
			min: Perbill::from_parts(Perbill::from_percent(4).deconstruct() / 2190),
			ideal: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 2190),
			max: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 2190),
		},
	}
}

pub fn development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				// Initial relay account
				get_account_id_from_seed::<sr25519::Public>("Relay"),
				// Sudo account
				"0xec00ad0ec6eeb271a9689888f644d9262016a26a25314ff4ff5d756404c44112"
					.parse()
					.unwrap(),
				// Ethereum AppId for SnowBridged Assets
				vec![
					(
						App::ETH,
						H160::from_slice(&hex!["dd514baa317bf095ddba2c0a847765feb389c6a0"][..])
							.into(),
					),
					(
						App::ERC20,
						H160::from_slice(&hex!["00e392c04743359e39f00cd268a5390d27ef6b44"][..])
							.into(),
					),
				],
				// SnowBridged Assets
				vec![
					(
						b"Mangata".to_vec(),
						b"MGA".to_vec(),
						b"Mangata Asset".to_vec(),
						18u32,
						0u32,
						H160::from_slice(&hex!["F8F7758FbcEfd546eAEff7dE24AFf666B6228e73"][..]),
						30_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Alice"),
					),
					(
						b"Ether".to_vec(),
						b"ETH".to_vec(),
						b"Ethereum Ether".to_vec(),
						18u32,
						1u32,
						H160::zero(),
						0u128,
						get_account_id_from_seed::<sr25519::Public>("Alice"),
					),
				],
				// Tokens endowment
				vec![
					(
						0u32,
						40_000_000__000_000_000_000_000_000u128,
						"0xec00ad0ec6eeb271a9689888f644d9262016a26a25314ff4ff5d756404c44112"
							.parse()
							.unwrap(),
					),
					(
						0u32,
						10_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Relay"),
					),
					(
						0u32,
						10_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Bob"),
					),
					(
						0u32,
						10_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
					),
				],
				// Config for Staking
				// Make sure it works with initial-authorities as staking uses both
				vec![
					(
						// Who gets to stake initially
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						// Id of MGA token,
						0u32,
						// How much mangata they pool
						10_000__000_000_000_000_000_000u128,
						// Id of the dummy token,
						2u32,
						// How many dummy tokens they pool,
						20_000__000_000_000_000_000_000u128,
						// Id of the liquidity token that is generated
						3u32,
						// How many liquidity tokens they stake,
						10_000__000_000_000_000_000_000u128,
					),
					(
						// Who gets to stake initially
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						// Id of MGA token,
						0u32,
						// How much mangata they pool
						8_000__000_000_000_000_000_000u128,
						// Id of the dummy token,
						2u32,
						// How many dummy tokens they pool,
						20_000__000_000_000_000_000_000u128,
						// Id of the liquidity token that is generated
						3u32,
						// How many liquidity tokens they stake,
						5_000__000_000_000_000_000_000u128,
					),
				],
				2000.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 2000,
		},
	)
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				// Initial relay account
				get_account_id_from_seed::<sr25519::Public>("Relay"),
				// Sudo account
				"0xec00ad0ec6eeb271a9689888f644d9262016a26a25314ff4ff5d756404c44112"
					.parse()
					.unwrap(),
				// Ethereum AppId for SnowBridged Assets
				vec![
					(
						App::ETH,
						H160::from_slice(&hex!["Fc97A6197dc90bef6bbEFD672742Ed75E9768553"][..])
							.into(),
					),
					(
						App::ERC20,
						H160::from_slice(&hex!["EDa338E4dC46038493b885327842fD3E301CaB39"][..])
							.into(),
					),
				],
				// SnowBridged Assets
				vec![
					(
						b"Mangata".to_vec(),
						b"MGA".to_vec(),
						b"Mangata Asset".to_vec(),
						18u32,
						0u32,
						H160::from_slice(&hex!["F8F7758FbcEfd546eAEff7dE24AFf666B6228e73"][..]),
						30_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Alice"),
					),
					(
						b"Ether".to_vec(),
						b"ETH".to_vec(),
						b"Ethereum Ether".to_vec(),
						18u32,
						1u32,
						H160::zero(),
						0u128,
						get_account_id_from_seed::<sr25519::Public>("Alice"),
					),
				],
				// Tokens endowment
				vec![
					(
						0u32,
						40_000_000__000_000_000_000_000_000u128,
						"0xec00ad0ec6eeb271a9689888f644d9262016a26a25314ff4ff5d756404c44112"
							.parse()
							.unwrap(),
					),
					(
						0u32,
						10_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Relay"),
					),
					(
						0u32,
						10_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Bob"),
					),
					(
						0u32,
						10_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
					),
				],
				// Config for Staking
				// Make sure it works with initial-authorities as staking uses both
				vec![
					(
						// Who gets to stake initially
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						// Id of MGA token,
						0u32,
						// How much mangata they pool
						10_000__000_000_000_000_000_000u128,
						// Id of the dummy token,
						2u32,
						// How many dummy tokens they pool,
						20_000__000_000_000_000_000_000u128,
						// Id of the liquidity token that is generated
						3u32,
						// How many liquidity tokens they stake,
						10_000__000_000_000_000_000_000u128,
					),
					(
						// Who gets to stake initially
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						// Id of MGA token,
						0u32,
						// How much mangata they pool
						8_000__000_000_000_000_000_000u128,
						// Id of the dummy token,
						2u32,
						// How many dummy tokens they pool,
						20_000__000_000_000_000_000_000u128,
						// Id of the liquidity token that is generated
						3u32,
						// How many liquidity tokens they stake,
						5_000__000_000_000_000_000_000u128,
					),
				],
				2000.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("mangata-local"),
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 2000,
		},
	)
}

type BridgedAssetsType = Vec<(Vec<u8>, Vec<u8>, Vec<u8>, u32, u32, H160, u128, AccountId)>;

fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	relay_key: AccountId,
	root_key: AccountId,
	bridged_app_ids: Vec<(App, AppId)>,
	bridged_assets: BridgedAssetsType,
	tokens_endowment: Vec<(u32, u128, AccountId)>,
	staking_accounts: Vec<(AccountId, u32, u128, u32, u128, u32, u128)>,
	id: ParaId,
) -> mangata_runtime::GenesisConfig {
	mangata_runtime::GenesisConfig {
		system: mangata_runtime::SystemConfig {
			code: mangata_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		tokens: mangata_runtime::TokensConfig {
			tokens_endowment: tokens_endowment
				.iter()
				.cloned()
				.map(|(token_id, amount, account)| (account, token_id, amount))
				.collect(),
			created_tokens_for_staking: {
				let mut created_tokens_for_staking_token_1: Vec<(AccountId, u32, u128)> =
					staking_accounts
						.iter()
						.cloned()
						.map(|x| {
							let (who, _, _, token_id, initial_amount, _, _) = x;
							(who.clone(), token_id, initial_amount)
						})
						.collect();
				let mut created_tokens_for_staking_token_2: Vec<(AccountId, u32, u128)> =
					staking_accounts
						.iter()
						.cloned()
						.map(|x| {
							let (who, token_id, initial_amount, _, _, _, _) = x;
							(who.clone(), token_id, initial_amount)
						})
						.collect();
				created_tokens_for_staking_token_1.append(&mut created_tokens_for_staking_token_2);
				created_tokens_for_staking_token_1
			},
		},
		treasury: Default::default(),
		parachain_info: mangata_runtime::ParachainInfoConfig { parachain_id: id },
		parachain_staking: mangata_runtime::ParachainStakingConfig {
			candidates: staking_accounts
				.iter()
				.map(|x| {
					let (account_id, _, _, _, _, liquidity_token_id, liquidity_token_amount) = x;
					(account_id.clone(), *liquidity_token_amount, *liquidity_token_id)
				})
				.collect(),
			delegations: vec![],
			inflation_config: mangata_inflation_config(),
		},
		session: mangata_runtime::SessionConfig {
			keys: initial_authorities
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                // account id
						acc,                        // validator id
						mangata_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		assets_info: mangata_runtime::AssetsInfoConfig {
			bridged_assets_info: bridged_assets
				.iter()
				.cloned()
				.map(|x| {
					let (name, token, description, decimals, asset_id, ..) = x;
					(Some(name), Some(token), Some(description), Some(decimals), asset_id)
				})
				.collect(),
		},
		xyk: mangata_runtime::XykConfig {
			created_pools_for_staking: staking_accounts
				.iter()
				.map(|x| {
					let (
						account_id,
						native_token_id,
						native_token_amount,
						pooled_token_id,
						pooled_token_amount,
						liquidity_token_id,
						_,
					) = x;
					(
						account_id.clone(),
						*native_token_id,
						*native_token_amount,
						*pooled_token_id,
						*pooled_token_amount,
						*liquidity_token_id,
					)
				})
				.collect(),
		},
		bridge: mangata_runtime::BridgeConfig { bridged_app_id_registry: bridged_app_ids },
		bridged_asset: mangata_runtime::BridgedAssetConfig {
			bridged_assets_links: bridged_assets
				.iter()
				.cloned()
				.map(|x| {
					let (.., asset_id, bridged_asset_id, initial_supply, initial_owner) = x;
					(asset_id, bridged_asset_id, initial_supply, initial_owner)
				})
				.collect(),
		},
		verifier: mangata_runtime::VerifierConfig { key: relay_key },
		council: Default::default(),
		elections: mangata_runtime::ElectionsConfig {
			members: tokens_endowment
				.iter()
				.cloned()
				.map(|(_, _, member)| (member, 100 * 100_000_000_000_000))
				.collect(),
		},
		sudo: mangata_runtime::SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		polkadot_xcm: mangata_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(2),
		},
	}
}
