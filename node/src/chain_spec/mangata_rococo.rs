use crate::chain_spec::{
	get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION,
};

use common_runtime::{
	config::orml_asset_registry::AssetMetadataOf,
	constants::parachains,
	ksm_per_second,
	tokens::{KAR_TOKEN_ID, MGX_TOKEN_ID, RELAY_TOKEN_ID, TUR_TOKEN_ID},
	xcm_config::general_key,
	AccountId, AuraId, CustomMetadata, XcmMetadata,
};
use cumulus_primitives_core::ParaId;
use sc_service::ChainType;
use sp_core::{sr25519, ByteArray, Encode};
use sp_runtime::BoundedVec;
use xcm::prelude::{MultiLocation, Parachain, X1, X2};

use hex::FromHex;

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
	sc_service::GenericChainSpec<mangata_rococo_runtime::RuntimeGenesisConfig, Extensions>;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn mangata_session_keys(keys: AuraId) -> mangata_rococo_runtime::SessionKeys {
	mangata_rococo_runtime::SessionKeys { aura: keys }
}

pub fn mangata_rococo_prod_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "MGR".into());
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
						100_000_000__000_000_000_000_000_000_u128,
						// Id of the dummy token,
						2u32,
						// How many dummy tokens they pool,
						200_000_000__000_000_000_000_000_000_u128,
						// Id of the liquidity token that is generated
						3u32,
						// How many liquidity tokens they stake,
						100_000_000__000_000_000_000_000_000_u128,
					),
					(
						// Who gets to stake initially
						public_testnet_keys::BOB_SR25519.parse::<AccountId>().unwrap().into(),
						// Id of MGA token,
						0u32,
						// How much mangata they pool
						80_000_000__000_000_000_000_000_000_u128,
						// Id of the dummy token,
						2u32,
						// How many dummy tokens they pool,
						200_000_000__000_000_000_000_000_000_u128,
						// Id of the liquidity token that is generated
						3u32,
						// How many liquidity tokens they stake,
						50_000_000__000_000_000_000_000_000_u128,
					),
				],
				vec![
					(
						MGX_TOKEN_ID,
						AssetMetadataOf {
							decimals: 18,
							name: BoundedVec::truncate_from(b"Mangata".to_vec()),
							symbol: BoundedVec::truncate_from(b"MGR".to_vec()),
							additional: Default::default(),
							existential_deposit: Default::default(),
							location: None,
						},
					),
					(
						1,
						AssetMetadataOf {
							decimals: 18,
							name: BoundedVec::truncate_from(b"Ether".to_vec()),
							symbol: BoundedVec::truncate_from(b"ETH".to_vec()),
							additional: Default::default(),
							existential_deposit: Default::default(),
							location: None,
						},
					),
					(
						RELAY_TOKEN_ID,
						AssetMetadataOf {
							decimals: 12,
							name: BoundedVec::truncate_from(b"Rococo Native".to_vec()),
							symbol: BoundedVec::truncate_from(b"ROC".to_vec()),
							additional: CustomMetadata {
								// 10_000:1 MGR:ROC
								xcm: Some(XcmMetadata { fee_per_second: ksm_per_second() }),
								xyk: None,
							},
							existential_deposit: Default::default(),
							location: None,
						},
					),
				],
				parachains::mangata::ID.into(),
				false,
			)
		},
		Vec::new(),
		None,
		// Protocol ID
		Some("mangata-rococo-testnet"),
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

pub fn mangata_rococo_local_config(initial_collators_as_sequencers: bool) -> ChainSpec {
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
						get_account_id_from_seed::<sr25519::Public>("Eve"),
						get_collator_keys_from_seed("Eve"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				// Initial relay account
				get_account_id_from_seed::<sr25519::Public>("Relay"),
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Tokens endowment
				vec![
					// MGA
					(
						0u32,
						300_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Alice"),
					),
					(
						0u32,
						300_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<sr25519::Public>("Eve"),
					),
					// ETH
					(1u32, 0u128, get_account_id_from_seed::<sr25519::Public>("Eve")),
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
						get_account_id_from_seed::<sr25519::Public>("Eve"),
						// Id of MGA token,
						0u32,
						// How much mangata they pool
						100_000_000__000_000_000_000_000_000_u128,
						// Id of the dummy token,
						2u32,
						// How many dummy tokens they pool,
						200_000_000__000_000_000_000_000_000_u128,
						// Id of the liquidity token that is generated
						3u32,
						// How many liquidity tokens they stake,
						100_000_000__000_000_000_000_000_000_u128,
					),
					(
						// Who gets to stake initially
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						// Id of MGA token,
						0u32,
						// How much mangata they pool
						80_000_000__000_000_000_000_000_000_u128,
						// Id of the dummy token,
						2u32,
						// How many dummy tokens they pool,
						200_000_000__000_000_000_000_000_000_u128,
						// Id of the liquidity token that is generated
						3u32,
						// How many liquidity tokens they stake,
						50_000_000__000_000_000_000_000_000_u128,
					),
				],
				vec![
					(
						MGX_TOKEN_ID,
						AssetMetadataOf {
							decimals: 18,
							name: BoundedVec::truncate_from(b"Mangata".to_vec()),
							symbol: BoundedVec::truncate_from(b"MGRL".to_vec()),
							additional: Default::default(),
							existential_deposit: Default::default(),
							location: None,
						},
					),
					(
						1,
						AssetMetadataOf {
							decimals: 18,
							name: BoundedVec::truncate_from(b"Ether".to_vec()),
							symbol: BoundedVec::truncate_from(b"ETH".to_vec()),
							additional: Default::default(),
							existential_deposit: Default::default(),
							location: None,
						},
					),
					(
						RELAY_TOKEN_ID,
						AssetMetadataOf {
							decimals: 12,
							name: BoundedVec::truncate_from(b"Rococo Native".to_vec()),
							symbol: BoundedVec::truncate_from(b"ROC".to_vec()),
							additional: CustomMetadata {
								// 10_000:1 MGR:ROC
								xcm: Some(XcmMetadata { fee_per_second: ksm_per_second() }),
								xyk: None,
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
							name: BoundedVec::new(),
							symbol: BoundedVec::new(),
							additional: Default::default(),
							existential_deposit: Default::default(),
							location: None,
						},
					),
					(
						KAR_TOKEN_ID,
						AssetMetadataOf {
							decimals: 12,
							name: BoundedVec::truncate_from(b"Karura".to_vec()),
							symbol: BoundedVec::truncate_from(b"KAR".to_vec()),
							additional: CustomMetadata {
								// 100:1 MGR:KAR
								xcm: Some(XcmMetadata { fee_per_second: ksm_per_second() * 100 }),
								xyk: None,
							},
							existential_deposit: Default::default(),
							location: Some(
								MultiLocation::new(
									1,
									X2(
										Parachain(parachains::karura::ID),
										general_key(parachains::karura::KAR_KEY),
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
							name: BoundedVec::truncate_from(b"Turing native token".to_vec()),
							symbol: BoundedVec::truncate_from(b"TUR".to_vec()),
							additional: CustomMetadata {
								// 100:1 TUR:ROC, 10/12 decimals
								xcm: Some(XcmMetadata { fee_per_second: ksm_per_second() }),
								xyk: None,
							},
							existential_deposit: Default::default(),
							location: Some(
								MultiLocation::new(1, X1(Parachain(parachains::turing::ID))).into(),
							),
						},
					),
				],
				parachains::mangata::ID.into(),
				initial_collators_as_sequencers,
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

pub(crate) fn mangata_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	_relay_key: AccountId,
	root_key: AccountId,
	tokens_endowment: Vec<(u32, u128, AccountId)>,
	staking_accounts: Vec<(AccountId, u32, u128, u32, u128, u32, u128)>,
	register_assets: Vec<(u32, AssetMetadataOf)>,
	id: ParaId,
	initial_collators_as_sequencers: bool,
) -> mangata_rococo_runtime::RuntimeGenesisConfig {
	mangata_rococo_runtime::RuntimeGenesisConfig {
		system: mangata_rococo_runtime::SystemConfig {
			code: mangata_rococo_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
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
		parachain_info: mangata_rococo_runtime::ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
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
				.clone()
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
		polkadot_xcm: mangata_rococo_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
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
		fee_lock: mangata_rococo_runtime::FeeLockConfig {
			period_length: Some(10),
			fee_lock_amount: Some(50__000_000_000_000_000_000u128),
			swap_value_threshold: Some(1000__000_000_000_000_000_000u128),
			whitelisted_tokens: Default::default(),
		},
		council: Default::default(),
		transaction_payment: Default::default(),
		sudo: mangata_rococo_runtime::SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
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
		vesting: Default::default(),
		rolldown: mangata_rococo_runtime::RolldownConfig {
			sequencers: if initial_collators_as_sequencers {
				initial_authorities.iter().map(|(acc, _)| acc.clone()).collect()
			} else {
				Default::default()
			},
		},
	}
}
