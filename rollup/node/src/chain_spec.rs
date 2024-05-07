use rollup_runtime::{
	config::orml_asset_registry::AssetMetadataOf, tokens::RX_TOKEN_ID, AccountId, AuraConfig,
	AuraId, CustomMetadata, GrandpaConfig, L1Asset, RuntimeGenesisConfig, Signature, SudoConfig,
	SystemConfig, XcmMetadata, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{ecdsa, ByteArray, Encode, Pair, Public};
use sp_keyring::EthereumKeyring;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	BoundedVec,
};
use sp_std::str::FromStr;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	let pair = TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed");
	// log::info!("Dev Account Seed Info - {:?}, {:x?}", seed, array_bytes::bytes2hex("0x", pair.to_raw_vec()));
	pair.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	let account = EthereumKeyring::from_str(seed)
		.expect("The keypair should be defined")
		.to_account_id();
	// log::info!("Dev Account PublicKey Info - {:?}, {:?}", seed, account);
	account
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn rollup_session_keys(aura: AuraId, grandpa: GrandpaId) -> rollup_runtime::SessionKeys {
	rollup_runtime::SessionKeys { aura, grandpa }
}

pub fn rollup_local_config(initial_collators_as_sequencers: bool, eth_chain_id: u64) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "GASP".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42u32.into());
	properties.insert("isEthereum".into(), true.into());

	ChainSpec::from_genesis(
		// Name
		"Rollup Local",
		// ID
		"rollup_local",
		ChainType::Local,
		move || {
			rollup_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<ecdsa::Public>("Alith"),
						authority_keys_from_seed("Alith"),
					),
					(
						get_account_id_from_seed::<ecdsa::Public>("Baltathar"),
						authority_keys_from_seed("Baltathar"),
					),
				],
				// Sudo account
				get_account_id_from_seed::<ecdsa::Public>("Alith"),
				// Tokens endowment
				vec![
					// MGA
					(
						0u32,
						300_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<ecdsa::Public>("Alith"),
					),
					(
						0u32,
						100_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<ecdsa::Public>("Baltathar"),
					),
					(
						0u32,
						100_000_000__000_000_000_000_000_000u128,
						get_account_id_from_seed::<ecdsa::Public>("Charleth"),
					),
				],
				// Config for Staking
				// Make sure it works with initial-authorities as staking uses both
				(
					vec![
						(
							// Who gets to stake initially
							get_account_id_from_seed::<ecdsa::Public>("Alith"),
							// Id of MGA token,
							0u32,
							// How much mangata they stake
							100_000_000__000_000_000_000_000_000_u128,
						),
						(
							// Who gets to stake initially
							get_account_id_from_seed::<ecdsa::Public>("Baltathar"),
							// Id of MGA token,
							0u32,
							// How much mangata they stake
							80_000_000__000_000_000_000_000_000_u128,
						),
					],
					vec![
						// Who gets to stake initially
						// Id of MGA token,
						// How much mangata they pool
						// Id of the dummy token,
						// How many dummy tokens they pool,
						// Id of the liquidity token that is generated
						// How many liquidity tokens they stake,
					],
				),
				vec![(
					RX_TOKEN_ID,
					AssetMetadataOf {
						decimals: 18,
						name: BoundedVec::truncate_from(b"Gasp".to_vec()),
						symbol: BoundedVec::truncate_from(b"GASP".to_vec()),
						additional: Default::default(),
						existential_deposit: Default::default(),
						location: None,
					},
					Some(L1Asset::Ethereum(
						array_bytes::hex2array("0x1317106Dd45FF0EB911e9F0aF78D63FBF9076f69")
							.unwrap(),
					)),
				)],
				initial_collators_as_sequencers,
				eth_chain_id,
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		None,
		// ForkId
		None,
		// Properties
		Some(properties),
		// Extensions
		None,
	)
}

/// Configure initial storage state for FRAME modules.
fn rollup_genesis(
	initial_authorities: Vec<(AccountId, (AuraId, GrandpaId))>,
	root_key: AccountId,
	tokens_endowment: Vec<(u32, u128, AccountId)>,
	staking_accounts: (
		Vec<(AccountId, u32, u128)>,
		Vec<(AccountId, u32, u128, u32, u128, u32, u128)>,
	),
	register_assets: Vec<(u32, AssetMetadataOf, Option<L1Asset>)>,
	initial_collators_as_sequencers: bool,
	chain_id: u64,
) -> rollup_runtime::RuntimeGenesisConfig {
	rollup_runtime::RuntimeGenesisConfig {
		system: rollup_runtime::SystemConfig {
			code: rollup_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
		},
		tokens: rollup_runtime::TokensConfig {
			tokens_endowment: tokens_endowment
				.iter()
				.cloned()
				.map(|(token_id, amount, account)| (account, token_id, amount))
				.collect(),
			created_tokens_for_staking: {
				let mut created_tokens_for_staking_token_1: Vec<(AccountId, u32, u128)> =
					staking_accounts
						.1
						.iter()
						.cloned()
						.map(|x| {
							let (who, _, _, token_id, initial_amount, _, _) = x;
							(who.clone(), token_id, initial_amount)
						})
						.collect();
				let mut created_tokens_for_staking_token_2: Vec<(AccountId, u32, u128)> =
					staking_accounts
						.1
						.iter()
						.cloned()
						.map(|x| {
							let (who, token_id, initial_amount, _, _, _, _) = x;
							(who.clone(), token_id, initial_amount)
						})
						.collect();
				let mut created_tokens_for_staking_token_3: Vec<(AccountId, u32, u128)> =
					staking_accounts.0.clone();
				created_tokens_for_staking_token_1.append(&mut created_tokens_for_staking_token_2);
				created_tokens_for_staking_token_1.append(&mut created_tokens_for_staking_token_3);
				created_tokens_for_staking_token_1
			},
		},
		treasury: Default::default(),
		parachain_staking: rollup_runtime::ParachainStakingConfig {
			candidates: {
				let mut parachain_staking_accounts_1: Vec<_> = staking_accounts
					.0
					.iter()
					.map(|x| {
						let (account_id, liquidity_token_id, liquidity_token_amount) = x;
						(account_id.clone(), *liquidity_token_amount, *liquidity_token_id)
					})
					.collect();
				let mut parachain_staking_accounts_2: Vec<_> = staking_accounts
					.1
					.iter()
					.map(|x| {
						let (account_id, _, _, _, _, liquidity_token_id, liquidity_token_amount) =
							x;
						(account_id.clone(), *liquidity_token_amount, *liquidity_token_id)
					})
					.collect();
				parachain_staking_accounts_1.append(&mut parachain_staking_accounts_2);
				parachain_staking_accounts_1
			},
			delegations: vec![],
		},
		session: rollup_runtime::SessionConfig {
			keys: initial_authorities
				.clone()
				.into_iter()
				.map(|(acc, (aura, grandpa))| {
					(
						acc.clone(),                        // account id
						acc,                                // validator id
						rollup_session_keys(aura, grandpa), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		grandpa: Default::default(),
		xyk: rollup_runtime::XykConfig {
			created_pools_for_staking: staking_accounts
				.1
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
		fee_lock: rollup_runtime::FeeLockConfig {
			period_length: Some(10),
			fee_lock_amount: Some(50__000_000_000_000_000_000u128),
			swap_value_threshold: Some(1000__000_000_000_000_000_000u128),
			whitelisted_tokens: Default::default(),
		},
		council: Default::default(),
		transaction_payment: Default::default(),
		sudo: rollup_runtime::SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		asset_registry: rollup_runtime::AssetRegistryConfig {
			assets: register_assets
				.iter()
				.cloned()
				.map(|(id, meta, maybe_l1_asset)| {
					let encoded = AssetMetadataOf::encode(&meta);
					(id, encoded, maybe_l1_asset)
				})
				.collect(),
		},
		vesting: Default::default(),
		sequencer_staking: rollup_runtime::SequencerStakingConfig {
			minimal_stake_amount: 1_000_000_u128,
			slash_fine_amount: 100_000_u128,
			sequencers_stake: if initial_collators_as_sequencers {
				initial_authorities
					.iter()
					.rev()
					.take(1)
					.map(|(acc, _)| (acc.clone(), 10_000_000_u128))
					.collect()
			} else {
				Default::default()
			},
		},
		rolldown: rollup_runtime::RolldownConfig { _phantom: Default::default() },
		metamask: rollup_runtime::MetamaskConfig {
			name: "Gasp".to_string(),
			version: "0.0.1".to_string(),
			chain_id,
			_phantom: Default::default(),
		},
	}
}
