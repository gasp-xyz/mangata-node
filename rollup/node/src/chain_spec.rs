use crate::command::{EvmChain, InitialSequencersSet};
use frame_benchmarking::benchmarking::current_time;
use rand::{thread_rng, Rng};
use rollup_runtime::{
	config::orml_asset_registry::AssetMetadataOf, currency, tokens::RX_TOKEN_ID, AccountId,
	AuraConfig, CustomMetadata, GrandpaConfig, L1Asset, RuntimeGenesisConfig, Signature,
	SudoConfig, SystemConfig, XcmMetadata, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{ecdsa, ByteArray, Encode, Pair, Public, H256};
use sp_keyring::EthereumKeyring;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	BoundedVec,
};
use sp_std::{convert::TryInto, str::FromStr};

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

pub fn rollup_local_config(
	randomize_chain_genesis_salt: bool,
	chain_genesis_salt: Option<String>,
	eth_sequencers: Vec<AccountId>,
	arb_sequencers: Vec<AccountId>,
	base_sequencers: Vec<AccountId>,
	evm_chain: EvmChain,
	decode_url: Option<String>,
) -> ChainSpec {
	let (gasp_token_address, eth_chain_id) = match evm_chain {
		EvmChain::Holesky => (
			array_bytes::hex2array("0x5620cDb94BaAaD10c20483bd8705DA711b2Bc0a3")
				.expect("is correct address"),
			17000u64,
		),
		EvmChain::Anvil => (
			array_bytes::hex2array("0xc351628EB244ec633d5f21fBD6621e1a683B1181")
				.expect("is correct address"),
			31337u64,
		),
		EvmChain::Reth => (
			array_bytes::hex2array("0xc351628EB244ec633d5f21fBD6621e1a683B1181")
				.expect("is correct address"),
			1337u64,
		),
	};

	let mut chain_genesis_salt_arr: [u8; 32] = [0u8; 32];
	if randomize_chain_genesis_salt {
		thread_rng().fill(&mut chain_genesis_salt_arr[..]);
	} else if let Some(salt) = chain_genesis_salt {
		chain_genesis_salt_arr = array_bytes::hex2bytes(salt)
			.expect("chain_genesis_salt should be hex")
			.iter()
			.chain(sp_std::iter::repeat(&0u8))
			.take(32)
			.cloned()
			.collect::<Vec<_>>()
			.try_into()
			.expect("32 bytes");
	}

	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "GASP".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42u32.into());
	properties.insert("isEthereum".into(), true.into());
	// This is quite useless here :/
	properties.insert(
		"chainGenesisSalt".into(),
		array_bytes::bytes2hex("0x", chain_genesis_salt_arr).into(),
	);

	let decode_url = decode_url.unwrap_or(String::from(
		"https://polkadot.js.org/apps/?rpc=ws%253A%252F%252F127.0.0.1%253A9944#/extrinsics/decode/",
	));
	// todo builder
	ChainSpec::from_genesis(
		// Name
		"Rollup Local",
		// ID
		"rollup_local",
		ChainType::Local,
		move || {
			let eth = eth_sequencers.clone();
			let arb = arb_sequencers.clone();
			let base = base_sequencers.clone();

			let tokens_endowment = [
				eth_sequencers.clone(),
				arb_sequencers.clone(),
				base_sequencers.clone(),
				vec![
					get_account_id_from_seed::<ecdsa::Public>("Alith"),
					get_account_id_from_seed::<ecdsa::Public>("Baltathar"),
					get_account_id_from_seed::<ecdsa::Public>("Charleth"),
				],
			]
			.iter()
			.flatten()
			.cloned()
			.map(|account_id| (0u32, 300_000_000__000_000_000_000_000_000u128, account_id))
			.collect::<Vec<_>>();

			rollup_genesis(
				// chain genesis salt
				H256::from(chain_genesis_salt_arr),
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
				tokens_endowment,
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
				vec![
					(
						RX_TOKEN_ID,
						AssetMetadataOf {
							decimals: 18,
							name: BoundedVec::truncate_from(b"Gasp".to_vec()),
							symbol: BoundedVec::truncate_from(b"GASP".to_vec()),
							additional: Default::default(),
							existential_deposit: Default::default(),
						},
						Some(L1Asset::Ethereum(gasp_token_address)),
					),
					(
						1,
						AssetMetadataOf {
							decimals: 18,
							name: BoundedVec::truncate_from(b"Gasp Ethereum".to_vec()),
							symbol: BoundedVec::truncate_from(b"GETH".to_vec()),
							additional: Default::default(),
							existential_deposit: Default::default(),
						},
						Some(L1Asset::Ethereum(
							array_bytes::hex2array("0x0000000000000000000000000000000000000001")
								.unwrap(),
						)),
					),
				],
				eth,
				arb,
				base,
				eth_chain_id,
				decode_url.clone(),
				vec![],
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
		// code
		rollup_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!"),
	)
}

pub fn ethereum_mainnet(decode_url: Option<String>) -> ChainSpec {
	let (_gasp_token_address, eth_chain_id) = ([0u8; 20], 1u64);
	let chain_genesis_salt_arr: [u8; 32] =
		hex_literal::hex!("0011001100110011001100110011001100110011001100110011001100110011");

	let collator01 = hex_literal::hex!("b9fcA08B9cA327a1dE90FDB4d51aa5ae6Ffe512a");
	let collator01_sr25519 =
		hex_literal::hex!("b6bcce45d0431d7cf3b23cf270e60fab48290cc2e129a62bcac04f6eab20e61f");
	let collator01_ed25519 =
		hex_literal::hex!("efc45e2afccbe0f53cab042438aebb6bcfc78585625c2a1b5517f3b258dd1cf8");

	let collator02 = hex_literal::hex!("1f4E3f24d1ad7fE108c6eB3BA6F83ebe8cF0eD20");
	let collator02_sr25519 =
		hex_literal::hex!("860a476d36782b7e2854ab4e93287e67618a835741b84bd7cad0740a83275f3c");
	let collator02_ed25519 =
		hex_literal::hex!("2227445cd1b97943e1ba5c3cf3a94cadabd4494ff4394667be13ff755bae1abe");

	let collator03 = hex_literal::hex!("7F7c7b782fBdAd01Fe33ca8FC647c867ee29deD2");
	let collator03_sr25519 =
		hex_literal::hex!("4899a218a591b9345b92de354b4d251eabd205bc64c787386fdccfe1f2147625");
	let collator03_ed25519 =
		hex_literal::hex!("d59387884193c920e0cef94770d74cc2cef0d534b1ebf5a5d1eb5033fb58746a");

	let collator04 = hex_literal::hex!("4691A9BB90e20a7708182fD881fb723f9845460E");
	let collator04_sr25519 =
		hex_literal::hex!("b0c2a16a1acecd3e05a243d9bf8881f5d64a70b40864701dba01cfd1ee53c85a");
	let collator04_ed25519 =
		hex_literal::hex!("172165647a152929e2bb0af97f55ea0c0deaa087479be8eab7448c2dc8cd0dfe");

	let sudo = hex_literal::hex!("E73e1Bb7B07f6bf447ED71252A5ad08C7ebE5bE5");

	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "GASP".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42u32.into());
	properties.insert("isEthereum".into(), true.into());
	// This is quite useless here :/
	properties.insert(
		"chainGenesisSalt".into(),
		array_bytes::bytes2hex("0x", chain_genesis_salt_arr).into(),
	);

	let decode_url = decode_url.expect("polkadot url is provided");
	// todo builder
	ChainSpec::from_genesis(
		// Name
		"Mainnet",
		// ID
		"mainnet",
		ChainType::Live,
		move || {
			let eth_sequencers: Vec<AccountId> = vec![
				hex_literal::hex!("dFD7f828689FbF00995BAA40d2DE93Eb400Cf60b").into(),
				hex_literal::hex!("88bbb08aF77987D86E9559491fE7cC5910D68f2D").into(),
				hex_literal::hex!("8d3CD208aa5592CF510eB24D8a2376bbF840bb63").into(),
			];
			let arb_sequencers: Vec<AccountId> = vec![
				hex_literal::hex!("b67CB37E9d114731B5624B6E919c007f4ddEa582").into(),
				hex_literal::hex!("71403bFc37f031b60BD7a5B9597115708E391410").into(),
				hex_literal::hex!("25CeF43c3F52db02ae52D951936b390C4B6A998F").into(),
			];
			let base_sequencers: Vec<AccountId> = vec![
				hex_literal::hex!("A395bBE2de17B488a578b972D96EE38933eE3c85").into(),
				hex_literal::hex!("6f52f2D60AdFC152ac561287b754A56A7933F1ae").into(),
				hex_literal::hex!("a7196AF761942A10126165B2c727eFCD46c254e0").into(),
			];

			let council_members = vec![
				hex_literal::hex!("35dbD8Bd2c5617541bd9D9D8e065adf92275b83E").into(),
				hex_literal::hex!("7368bff2fBB4C7B05f854c370eeDD6809186917B").into(),
				hex_literal::hex!("9cA8aFB1326c99EC23B8D4e16C0162Bb206D83b8").into(),
				hex_literal::hex!("8b5368B4BBa80475c9DFb70543F6090A7e986F39").into(),
				hex_literal::hex!("aF3cA574A4903c5ddC7378Ac60d786a2664CbD91").into(),
				hex_literal::hex!("584728a637303e753906a4F05CD8Ced10D80eB5e").into(),
				hex_literal::hex!("8960911c51EaD00db4cCA88FAF395672458da676").into(),
			];

			let sequencers_endownment =
				[eth_sequencers.clone(), arb_sequencers.clone(), base_sequencers.clone()]
					.iter()
					.flatten()
					.cloned()
					.map(|account_id| (RX_TOKEN_ID, 100u128 * currency::DOLLARS, account_id))
					.collect::<Vec<_>>();

			let mut tokens_endowment = sequencers_endownment;
			tokens_endowment.push((RX_TOKEN_ID, 988_100_u128 * currency::DOLLARS, sudo.into()));

			tokens_endowment.append(
				&mut council_members
					.clone()
					.into_iter()
					.map(|addr| (RX_TOKEN_ID, 1000_u128 * currency::DOLLARS, addr))
					.collect::<Vec<_>>(),
			);

			rollup_genesis(
				// chain genesis salt
				H256::from(chain_genesis_salt_arr),
				// initial collators.
				vec![
					(
						collator01.into(),
						(
							AuraId::from_slice(collator01_sr25519.as_slice()).unwrap(),
							GrandpaId::from_slice(collator01_ed25519.as_slice()).unwrap(),
						),
					),
					(
						collator02.into(),
						(
							AuraId::from_slice(collator02_sr25519.as_slice()).unwrap(),
							GrandpaId::from_slice(collator02_ed25519.as_slice()).unwrap(),
						),
					),
					(
						collator03.into(),
						(
							AuraId::from_slice(collator03_sr25519.as_slice()).unwrap(),
							GrandpaId::from_slice(collator03_ed25519.as_slice()).unwrap(),
						),
					),
					(
						collator04.into(),
						(
							AuraId::from_slice(collator04_sr25519.as_slice()).unwrap(),
							GrandpaId::from_slice(collator04_ed25519.as_slice()).unwrap(),
						),
					),
				],
				// Sudo account
				sudo.into(),
				// Tokens endowment
				tokens_endowment,
				// Config for Staking
				// Make sure it works with initial-authorities as staking uses both
				(
					vec![
						(
							// Who gets to stake initially
							collator01.into(),
							// Id of MGA token,
							0u32,
							// How much mangata they stake
							1__001_000u128 * currency::DOLLARS,
						),
						(
							// Who gets to stake initially
							collator02.into(),
							// Id of MGA token,
							0u32,
							// How much mangata they stake
							1__001_000u128 * currency::DOLLARS,
						),
						(
							// Who gets to stake initially
							collator03.into(),
							// Id of MGA token,
							0u32,
							// How much mangata they stake
							1__001_000u128 * currency::DOLLARS,
						),
						(
							// Who gets to stake initially
							collator04.into(),
							// Id of MGA token,
							0u32,
							// How much mangata they stake
							1__001_000u128 * currency::DOLLARS,
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
				vec![
					(
						RX_TOKEN_ID,
						AssetMetadataOf {
							decimals: 18,
							name: BoundedVec::truncate_from(b"Gasp".to_vec()),
							symbol: BoundedVec::truncate_from(b"GASP".to_vec()),
							additional: Default::default(),
							existential_deposit: Default::default(),
						},
						Some(L1Asset::Ethereum(hex_literal::hex!(
							"0000000000000000000000000000000000000000"
						))),
					),
					(
						1,
						AssetMetadataOf {
							decimals: 18,
							name: BoundedVec::truncate_from(b"Gasp Ethereum".to_vec()),
							symbol: BoundedVec::truncate_from(b"GETH".to_vec()),
							additional: Default::default(),
							existential_deposit: Default::default(),
						},
						Some(L1Asset::Ethereum(
							array_bytes::hex2array("0x0000000000000000000000000000000000000001")
								.unwrap(),
						)),
					),
				],
				eth_sequencers,
				arb_sequencers,
				base_sequencers,
				eth_chain_id,
				decode_url.clone(),
				council_members,
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
		// code
		rollup_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!"),
	)
}

/// Configure initial storage state for FRAME modules.
fn rollup_genesis(
	chain_genesis_salt: H256,
	initial_authorities: Vec<(AccountId, (AuraId, GrandpaId))>,
	root_key: AccountId,
	tokens_endowment: Vec<(u32, u128, AccountId)>,
	staking_accounts: (
		Vec<(AccountId, u32, u128)>,
		Vec<(AccountId, u32, u128, u32, u128, u32, u128)>,
	),
	register_assets: Vec<(u32, AssetMetadataOf, Option<L1Asset>)>,
	eth_initial_sequencers: Vec<AccountId>,
	arb_initial_sequencers: Vec<AccountId>,
	base_initial_sequencers: Vec<AccountId>,
	chain_id: u64,
	decode_url: String,
	council_members: Vec<AccountId>,
) -> rollup_runtime::RuntimeGenesisConfig {
	let initial_sequencers_stake = 10_000_000_u128;

	rollup_runtime::RuntimeGenesisConfig {
		system: rollup_runtime::SystemConfig { chain_genesis_salt, ..Default::default() },
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
			fee_lock_amount: Some(50u128 * currency::DOLLARS),
			swap_value_threshold: Some(50u128 * currency::DOLLARS),
			whitelisted_tokens: Default::default(),
		},
		council: rollup_runtime::CouncilConfig {
			phantom: Default::default(),
			members: council_members,
		},
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
			minimal_stake_amount: 100u128 * currency::DOLLARS,
			slash_fine_amount: 1u128 * currency::DOLLARS,
			sequencers_stake: [
				eth_initial_sequencers
					.into_iter()
					.map(|seq| {
						(
							seq,
							pallet_rolldown::messages::Chain::Ethereum,
							100u128 * currency::DOLLARS,
						)
					})
					.collect::<Vec<_>>(),
				arb_initial_sequencers
					.into_iter()
					.map(|seq| {
						(
							seq,
							pallet_rolldown::messages::Chain::Arbitrum,
							100u128 * currency::DOLLARS,
						)
					})
					.collect::<Vec<_>>(),
				base_initial_sequencers
					.into_iter()
					.map(|seq| {
						(seq, pallet_rolldown::messages::Chain::Base, 100u128 * currency::DOLLARS)
					})
					.collect::<Vec<_>>(),
			]
			.iter()
			.flatten()
			.cloned()
			.collect(),
		},
		#[cfg(not(feature = "fast-runtime"))]
		rolldown: rollup_runtime::RolldownConfig {
			_phantom: Default::default(),
			dispute_periods: [
				(pallet_rolldown::messages::Chain::Ethereum, 200u128),
				(pallet_rolldown::messages::Chain::Arbitrum, 200u128),
				(pallet_rolldown::messages::Chain::Base, 200u128),
			]
			.iter()
			.cloned()
			.collect(),
		},
		#[cfg(feature = "fast-runtime")]
		rolldown: rollup_runtime::RolldownConfig {
			_phantom: Default::default(),
			dispute_periods: [
				(pallet_rolldown::messages::Chain::Ethereum, 10u128),
				(pallet_rolldown::messages::Chain::Arbitrum, 15u128),
				(pallet_rolldown::messages::Chain::Base, 15u128),
			]
			.iter()
			.cloned()
			.collect(),
		},
		metamask: rollup_runtime::MetamaskConfig {
			name: "Gasp".to_string(),
			version: "0.0.1".to_string(),
			chain_id,
			decode_url,
			_phantom: Default::default(),
		},
		foundation_members: rollup_runtime::FoundationMembersConfig {
			members: BoundedVec::truncate_from(
				[
					hex_literal::hex!["9cA8aFB1326c99EC23B8D4e16C0162Bb206D83b8"],
					hex_literal::hex!["8960911c51EaD00db4cCA88FAF395672458da676"],
					hex_literal::hex!["35dbD8Bd2c5617541bd9D9D8e065adf92275b83E"],
				]
				.iter()
				.map(|acc| sp_runtime::AccountId20::from(*acc))
				.collect::<Vec<_>>(),
			),
			phantom: Default::default(),
		},
		transfer_members: Default::default(),
	}
}
