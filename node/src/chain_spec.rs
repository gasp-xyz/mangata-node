use sp_core::{Pair, Public, sr25519};
use mangata_runtime::{
	AccountId, AuraConfig, GenesisConfig, GrandpaConfig, SessionKeys,
	SudoConfig, SystemConfig, WASM_BINARY, Signature, TokensConfig,
	AssetsInfoConfig, TreasuryConfig, XykConfig, CouncilConfig, ElectionsConfig,
	BridgeConfig, BridgedAssetConfig, VerifierConfig, SessionConfig, StakingConfig, StakerStatus
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};
use sc_service::ChainType;
use hex_literal::hex;
use artemis_core::{App, AppId};
use sp_core::H160;

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
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
	(
        get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

fn session_keys(
    aura: AuraId,
    grandpa: GrandpaId
) -> SessionKeys {
    SessionKeys {
        aura,
        grandpa
    }
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

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
                vec![(
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
                )],
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

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

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
                vec![(
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
                )],
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

type BridgedAssetsType = Vec<(Vec<u8>, Vec<u8>, Vec<u8>, u32, u32, H160, u128, AccountId)>;

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	relay_key: AccountId,
    root_key: AccountId,
    bridged_app_ids: Vec<(App, AppId)>,
    bridged_assets: BridgedAssetsType,
    tokens_endowment: Vec<(u32, u128, AccountId)>,
	staking_accounts: Vec<(AccountId, u32, u128, u32, u128, u32, u128)>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		orml_tokens: Some(TokensConfig {
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
        }),
		pallet_aura: Some(AuraConfig {
			authorities: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key,
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
		pallet_treasury: Some(TreasuryConfig::default()),
		pallet_xyk: Some(XykConfig {
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
        }),
        pallet_collective_Instance1: Some(CouncilConfig::default()),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: tokens_endowment
                .iter()
                .cloned()
                .map(|(_, _, member)| (member, 100 * 100_000_000_000_000))
                .collect(),
        }),
		pallet_verifier: Some(VerifierConfig { key: relay_key }),
		artemis_asset: Some(BridgedAssetConfig {
            bridged_assets_links: bridged_assets
                .iter()
                .cloned()
                .map(|x| {
                    let (.., asset_id, bridged_asset_id, initial_supply, initial_owner) = x;
                    (asset_id, bridged_asset_id, initial_supply, initial_owner)
                })
                .collect(),
        }),
		pallet_bridge: Some(BridgeConfig {
            bridged_app_id_registry: bridged_app_ids,
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: staking_accounts
                .iter()
                .map(|x| {
                    let (account_id, _, _, _, _, _liquidity_token_id, _liquidity_token_amount) = x;
                    (
                        account_id.clone(),
                        account_id.clone(),
                        1_000__000_000_000_000_000_000u128,
                        StakerStatus::Validator,
                    )
                })
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.1.clone(), x.2.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
	}
}
