use crate::chain_spec::Extensions;
use artemis_core::{App, AppId};
use codec::Encode;
use cumulus_primitives_core::ParaId;
use hex::FromHex;
use hex_literal::hex;
use mangata_rococo_runtime::{
	constants::parachains, roc_per_second, AccountId, AssetMetadataOf, AuraId, CustomMetadata,
	GeneralKey, MultiLocation, Parachain, Signature, XcmMetadata, KAR_TOKEN_ID, ROC_TOKEN_ID,
	TUR_TOKEN_ID, X1, X2,
};
use sc_service::ChainType;
use sp_core::{sr25519, ByteArray, Pair, Public, H160};
use sp_runtime::{
	traits::{ConstU32, IdentifyAccount, Verify},
	WeakBoundedVec,
};

pub mod public_testnet_keys {
	pub const ALICE_SR25519: &str =
		"0x76e810a1f116b779fea0962f4102e9acb3d22ba603999ecb08ad163582de192c";
	pub const BOB_SR25519: &str =
		"0x2eec12f5af27c95fd5661323e544dc857155ea50689eaccdfd580af7c1be921c";
	pub const CHARLIE_SR25519: &str =
		"0xbc443b4f7023de2d868f74f9e51159961482dc46f76aa90a1c6ce58efff4be6a";
	pub const SUDO_SR25519: &str =
		"0x249af97c2ab99f229cdf18cc966833b894ae7c4d94c13fa86341209c64c8ec18";
	pub const RELAY_SR25519: &str =
		"0x7481b06f37b3500bb6ec8d569d2cede4ffcb151daee75f7de20c5bda2e22bb13";
}

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
	sc_service::GenericChainSpec<mangata_rococo_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
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
pub fn mangata_session_keys(keys: AuraId) -> mangata_rococo_runtime::SessionKeys {
	mangata_rococo_runtime::SessionKeys { aura: keys }
}

pub fn public_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "MGAT".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Mangata Public Testnet",
		// ID
		"mangata_public_testnet",
		ChainType::Live,
		move || {
			mangata_genesis(
				// initial collators.
				vec![
					(
						public_testnet_keys::ALICE_SR25519.parse::<AccountId>().unwrap().into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								public_testnet_keys::ALICE_SR25519.strip_prefix("0x").unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						public_testnet_keys::BOB_SR25519.parse::<AccountId>().unwrap().into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								public_testnet_keys::BOB_SR25519.strip_prefix("0x").unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
				],
				// Initial relay account
				public_testnet_keys::RELAY_SR25519.parse::<AccountId>().unwrap().into(),
				// Sudo account
				public_testnet_keys::SUDO_SR25519.parse::<AccountId>().unwrap().into(),
				// Tokens endowment
				vec![
					// MGA
					(
						0u32,
						300_000_000__000_000_000_000_000_000u128,
						public_testnet_keys::ALICE_SR25519.parse::<AccountId>().unwrap().into(),
					),
					// ETH
					(
						1u32,
						0u128,
						public_testnet_keys::ALICE_SR25519.parse::<AccountId>().unwrap().into(),
					),
					(
						0u32,
						400_000_000__000_000_000_000_000_000u128,
						public_testnet_keys::SUDO_SR25519.parse::<AccountId>().unwrap().into(),
					),
					(
						0u32,
						100_000_000__000_000_000_000_000_000u128,
						public_testnet_keys::RELAY_SR25519.parse::<AccountId>().unwrap().into(),
					),
					(
						0u32,
						100_000_000__000_000_000_000_000_000u128,
						public_testnet_keys::BOB_SR25519.parse::<AccountId>().unwrap().into(),
					),
					(
						0u32,
						100_000_000__000_000_000_000_000_000u128,
						public_testnet_keys::CHARLIE_SR25519.parse::<AccountId>().unwrap().into(),
					),
				],
				// Config for Staking
				// Make sure it works with initial-authorities as staking uses both
				vec![
					(
						// Who gets to stake initially
						public_testnet_keys::ALICE_SR25519.parse::<AccountId>().unwrap().into(),
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
						public_testnet_keys::BOB_SR25519.parse::<AccountId>().unwrap().into(),
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
				vec![
					(
						0,
						AssetMetadataOf {
							decimals: 18,
							name: b"Mangata".to_vec(),
							symbol: b"MGR".to_vec(),
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
								// 10_000:1 MGR:ROC
								xcm: Some(XcmMetadata { fee_per_second: roc_per_second() }),
							},
							existential_deposit: Default::default(),
							location: None,
						},
					),
				],
				parachains::mangata::ID.into(),
			)
		},
		Vec::new(),
		None,
		// Protocol ID
		Some("mangata-public-testnet"),
		// ForkId
		None,
		// Properties
		Some(properties),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: parachains::mangata::ID,
		},
	)
}

pub fn mangata_rococo_local_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "MGRL".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42u32.into());

	ChainSpec::from_genesis(
		// Name
		"Mangata Rococo Local",
		// ID
		"mangata_rococo_local",
		ChainType::Local,
		move || {
			mangata_genesis(
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
				// Tokens endowment
				vec![
					// MGA
					(
						0u32,
						300_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Alice"),
					),
					// ETH
					(1u32, 0u128, get_account_id_from_seed::<sr25519::Public>("Alice")),
					(
						0u32,
						400_000_000__000_000_000_000_000_000u128,
						"0xec00ad0ec6eeb271a9689888f644d9262016a26a25314ff4ff5d756404c44112"
							.parse()
							.unwrap(),
					),
					(
						0u32,
						100_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Relay"),
					),
					(
						0u32,
						100_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Bob"),
					),
					(
						0u32,
						100_000_000__000_000_000_000_000_000u128,
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
				vec![
					(
						0,
						AssetMetadataOf {
							decimals: 18,
							name: b"Mangata".to_vec(),
							symbol: b"MGR".to_vec(),
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
								// 10_000:1 MGR:ROC
								xcm: Some(XcmMetadata { fee_per_second: roc_per_second() }),
							},
							existential_deposit: Default::default(),
							location: None,
						},
					),
					// empty placeholder to issueToken & increment nextAssetId
					(
						5,
						AssetMetadataOf {
							decimals: 0,
							name: vec![],
							symbol: vec![],
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
								// 100:1 MGR:KAR
								xcm: Some(XcmMetadata { fee_per_second: roc_per_second() * 100 }),
							},
							existential_deposit: Default::default(),
							location: Some(
								MultiLocation::new(
									1,
									X2(
										Parachain(parachains::karura::ID),
										GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
											parachains::karura::KAR_KEY.to_vec(),
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
							location: Some(
								MultiLocation::new(1, X1(Parachain(parachains::turing::ID))).into(),
							),
						},
					),
				],
				parachains::mangata::ID.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("mangata-rococo-local"),
		// ForkId
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: parachains::mangata::ID,
		},
	)
}

fn mangata_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	relay_key: AccountId,
	root_key: AccountId,
	tokens_endowment: Vec<(u32, u128, AccountId)>,
	staking_accounts: Vec<(AccountId, u32, u128, u32, u128, u32, u128)>,
	register_assets: Vec<(u32, AssetMetadataOf)>,
	id: ParaId,
) -> mangata_rococo_runtime::GenesisConfig {
	mangata_rococo_runtime::GenesisConfig {
		system: mangata_rococo_runtime::SystemConfig {
			code: mangata_rococo_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		tokens: mangata_rococo_runtime::TokensConfig {
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
		parachain_info: mangata_rococo_runtime::ParachainInfoConfig { parachain_id: id },
		parachain_staking: mangata_rococo_runtime::ParachainStakingConfig {
			candidates: staking_accounts
				.iter()
				.map(|x| {
					let (account_id, _, _, _, _, liquidity_token_id, liquidity_token_amount) = x;
					(account_id.clone(), *liquidity_token_amount, *liquidity_token_id)
				})
				.collect(),
			delegations: vec![],
		},
		session: mangata_rococo_runtime::SessionConfig {
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
		xyk: mangata_rococo_runtime::XykConfig {
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
		council: Default::default(),
		elections: mangata_rococo_runtime::ElectionsConfig {
			members: vec![],
		},
		sudo: mangata_rococo_runtime::SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		polkadot_xcm: mangata_rococo_runtime::PolkadotXcmConfig { safe_xcm_version: Some(2) },
		asset_registry: mangata_rococo_runtime::AssetRegistryConfig {
			assets: register_assets
				.iter()
				.cloned()
				.map(|(id, meta)| {
					let encoded = AssetMetadataOf::encode(&meta);
					(id, encoded)
				})
				.collect(),
		},
	}
}
