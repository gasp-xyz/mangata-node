// Copyright (C) 2020 Mangata team

use hex_literal::hex;
use mangata_runtime::{
    AccountId, AssetsInfoConfig, BabeConfig, BalancesConfig, BridgeConfig, BridgedAssetConfig,
    GenesisConfig, GrandpaConfig, SessionConfig, SessionKeys, Signature, StakerStatus,
    StakingConfig, SudoConfig, SystemConfig, VerifierConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public, H160};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};

use artemis_core::{App, AppId};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (BabeId, GrandpaId, AccountId) {
    (
        // get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
        // get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<GrandpaId>(s),
        get_account_id_from_seed::<sr25519::Public>(s),
    )
}

fn session_keys(grandpa: GrandpaId, babe: BabeId) -> SessionKeys {
    SessionKeys { grandpa, babe }
}

#[allow(clippy::inconsistent_digit_grouping)]
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
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
                        b"MNG".to_vec(),
                        b"Mangata Asset".to_vec(),
                        18u32,
                        0u32,
                        H160::from_slice(&hex!["F8F7758FbcEfd546eAEff7dE24AFf666B6228e73"][..]),
                        100_000_000__000_000_000_000_000_000u128,
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
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Relay"),
                    "0xec00ad0ec6eeb271a9689888f644d9262016a26a25314ff4ff5d756404c44112"
                        .parse()
                        .unwrap(),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

#[allow(clippy::inconsistent_digit_grouping)]
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Initial relay account
                get_account_id_from_seed::<sr25519::Public>("Relay"),
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
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
                        b"MNG".to_vec(),
                        b"Mangata Asset".to_vec(),
                        18u32,
                        0u32,
                        H160::from_slice(&hex!["F8F7758FbcEfd546eAEff7dE24AFf666B6228e73"][..]),
                        100_000_000__000_000_000_000_000_000u128,
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
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}
type BriedgedAssetsType = Vec<(Vec<u8>, Vec<u8>, Vec<u8>, u32, u32, H160, u128, AccountId)>;

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(BabeId, GrandpaId, AccountId)>,
    relay_key: AccountId,
    root_key: AccountId,
    bridged_app_ids: Vec<(App, AppId)>,
    bridged_assets: BriedgedAssetsType,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.2.clone(),
                        x.2.clone(),
                        session_keys(x.1.clone(), x.0.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.2.clone(), x.2.clone(), 0_u128, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.2.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key,
        }),
        verifier: Some(VerifierConfig { key: relay_key }),

        bridge: Some(BridgeConfig {
            bridged_app_id_registry: bridged_app_ids,
        }),

        pallet_assets_info: Some(AssetsInfoConfig {
            bridged_assets_info: bridged_assets
                .iter()
                .cloned()
                .map(|x| {
                    let (name, token, description, decimals, asset_id, ..) = x;
                    (
                        Some(name),
                        Some(token),
                        Some(description),
                        Some(decimals),
                        asset_id,
                    )
                })
                .collect(),
        }),

        bridged_asset: Some(BridgedAssetConfig {
            bridged_assets_links: bridged_assets
                .iter()
                .cloned()
                .map(|x| {
                    let (.., asset_id, bridged_asset_id, initial_supply, initial_owner) = x;
                    (asset_id, bridged_asset_id, initial_supply, initial_owner)
                })
                .collect(),
        }),
    }
}
