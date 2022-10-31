// This file is part of Substrate.

// Copyright (C) 2021-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use cumulus_primitives_core::PersistedValidationData;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_system_rpc_runtime_api::AccountNonceApi;
use mangata_node::service::{parachain_build_import_queue, FullClient};
use mangata_runtime::SystemCall;
use polkadot_runtime_common::BlockHashCount;
use sc_block_builder::{BlockBuilderProvider, BuiltBlock};
use sc_client_api::{execution_extensions::ExecutionStrategies, BlockBackend};
use sc_consensus::{
	block_import::{BlockImportParams, ForkChoiceStrategy},
	BlockImport, StateAction,
};
use sc_service::{
	config::{
		DatabaseSource, KeepBlocks, KeystoreConfig, NetworkConfiguration, OffchainWorkerConfig,
		PruningMode, TransactionStorageMode, WasmExecutionMethod,
	},
	BasePath, Configuration, Role,
};
use sp_api::{Core, ProvideRuntimeApi};
use sp_consensus::BlockOrigin;
use sp_consensus_aura::{digests::CompatibleDigestItem, AuraApi};
use sp_core::{crypto::key_types::AURA, Encode, Pair};
use sp_keyring::Sr25519Keyring;
use sp_keystore::vrf::{VRFTranscriptData, VRFTranscriptValue};
use sp_runtime::{
	generic,
	generic::{BlockId, DigestItem},
	traits::{Block as BlockT, Header as HeaderT},
	transaction_validity::{InvalidTransaction, TransactionValidityError},
	AccountId32, OpaqueExtrinsic, SaturatedConversion,
};
use tokio::runtime::Handle;

use sc_block_builder::BlockBuilderApi;
use sp_consensus_aura::sr25519::AuthoritySignature;
use sp_core::{ByteArray, ShufflingSeed};
use sp_keystore::SyncCryptoStore;
use std::convert::TryInto;

const MINIMUM_PERIOD_FOR_BLOCKS: u64 = 6000;

fn new_config(tokio_handle: Handle) -> Configuration {
	let base_path = BasePath::new_temp_dir()
		.expect("getting the base path of a temporary path doesn't fail; qed");
	let root = base_path.path().to_path_buf();

	let network_config = NetworkConfiguration::new(
		Sr25519Keyring::Alice.to_seed(),
		"network/test/0.1",
		Default::default(),
		None,
	);

	let spec = Box::new(mangata_node::chain_spec::development_config());

	// NOTE: We enforce the use of the WASM runtime to benchmark block production using WASM.
	let execution_strategy = sc_client_api::ExecutionStrategy::AlwaysWasm;

	Configuration {
		impl_name: "BenchmarkImpl".into(),
		impl_version: "1.0".into(),
		// We don't use the authority role since that would start producing blocks
		// in the background which would mess with our benchmark.
		role: Role::Full,
		tokio_handle,
		transaction_pool: Default::default(),
		network: network_config,
		keystore: KeystoreConfig::InMemory,
		keystore_remote: Default::default(),
		database: DatabaseSource::RocksDb { path: root.join("db"), cache_size: 128 },
		state_cache_size: 67108864,
		state_cache_child_ratio: None,
		state_pruning: PruningMode::ArchiveAll,
		keep_blocks: KeepBlocks::All,
		transaction_storage: TransactionStorageMode::BlockBody,
		chain_spec: spec,
		wasm_method: WasmExecutionMethod::Compiled,
		execution_strategies: ExecutionStrategies {
			syncing: execution_strategy,
			importing: execution_strategy,
			block_construction: execution_strategy,
			offchain_worker: execution_strategy,
			other: execution_strategy,
		},
		rpc_http: None,
		rpc_ws: None,
		rpc_ipc: None,
		rpc_ws_max_connections: None,
		rpc_cors: None,
		rpc_methods: Default::default(),
		rpc_max_payload: None,
		ws_max_out_buffer_capacity: None,
		prometheus_config: None,
		telemetry_endpoints: None,
		default_heap_pages: None,
		offchain_worker: OffchainWorkerConfig { enabled: true, indexing_enabled: false },
		force_authoring: false,
		disable_grandpa: false,
		dev_key_seed: Some(Sr25519Keyring::Alice.to_seed()),
		tracing_targets: None,
		tracing_receiver: Default::default(),
		max_runtime_instances: 8,
		runtime_cache_size: 2,
		announce_block: true,
		base_path: Some(base_path),
		informant_output_format: Default::default(),
		wasm_runtime_overrides: None,
	}
}

fn extrinsic_set_time(now: u64) -> OpaqueExtrinsic {
	mangata_runtime::UncheckedExtrinsic {
		signature: None,
		function: mangata_runtime::Call::Timestamp(pallet_timestamp::Call::set { now }),
	}
	.into()
}

fn extrinsic_set_validation_data() -> OpaqueExtrinsic {
	let sproof_builder = RelayStateSproofBuilder::default();
	let (relay_parent_storage_root, proof) = sproof_builder.into_state_root_and_proof();

	let d = cumulus_primitives_parachain_inherent::ParachainInherentData {
		validation_data: PersistedValidationData {
			parent_head: Default::default(),
			relay_parent_storage_root,
			relay_parent_number: Default::default(),
			max_pov_size: Default::default(),
		},
		downward_messages: Default::default(),
		horizontal_messages: Default::default(),
		relay_chain_state: proof,
	};

	mangata_runtime::UncheckedExtrinsic {
		signature: None,
		function: mangata_runtime::Call::ParachainSystem(
			cumulus_pallet_parachain_system::Call::set_validation_data { data: d },
		),
	}
	.into()
}

fn import_block(
	mut client: &FullClient,
	built: BuiltBlock<
		mangata_runtime::opaque::Block,
		<FullClient as sp_api::CallApiAt<mangata_runtime::opaque::Block>>::StateBackend,
	>,
) {
	let mut params = BlockImportParams::new(BlockOrigin::File, built.block.header);
	params.state_action =
		StateAction::ApplyChanges(sc_consensus::StorageChanges::Changes(built.storage_changes));
	params.fork_choice = Some(ForkChoiceStrategy::LongestChain);
	futures::executor::block_on(client.import_block(params, Default::default()))
		.expect("importing a block doesn't fail");
}

pub fn fetch_nonce(client: &FullClient, account: sp_core::sr25519::Pair) -> u32 {
	let best_hash = client.chain_info().best_hash;
	client
		.runtime_api()
		.account_nonce(&generic::BlockId::Hash(best_hash), account.public().into())
		.expect("Fetching account nonce works; qed")
}

pub fn create_extrinsic(
	client: &FullClient,
	sender: sp_core::sr25519::Pair,
	function: impl Into<mangata_runtime::Call>,
	nonce: Option<u32>,
) -> mangata_runtime::UncheckedExtrinsic {
	let function = function.into();
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	let best_block = client.chain_info().best_number;
	let nonce = nonce.unwrap_or_else(|| fetch_nonce(client, sender.clone()));

	let period =
		BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
	let tip = 0;
	let extra: mangata_runtime::SignedExtra = (
		frame_system::CheckSpecVersion::<mangata_runtime::Runtime>::new(),
		frame_system::CheckTxVersion::<mangata_runtime::Runtime>::new(),
		frame_system::CheckGenesis::<mangata_runtime::Runtime>::new(),
		frame_system::CheckEra::<mangata_runtime::Runtime>::from(generic::Era::mortal(
			period,
			best_block.saturated_into(),
		)),
		frame_system::CheckNonce::<mangata_runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<mangata_runtime::Runtime>::new(),
		pallet_transaction_payment::ChargeTransactionPayment::<mangata_runtime::Runtime>::from(tip),
	);

	let raw_payload = mangata_runtime::SignedPayload::from_raw(
		function.clone(),
		extra.clone(),
		(
			mangata_runtime::VERSION.spec_version,
			mangata_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| sender.sign(e));

	mangata_runtime::UncheckedExtrinsic::new_signed(
		function.clone(),
		AccountId32::from(sender.public()).into(),
		mangata_runtime::Signature::Sr25519(signature.clone()),
		extra.clone(),
	)
}

fn prepare_benchmark(
	client: &FullClient,
	digest: sp_runtime::generic::Digest,
) -> Vec<OpaqueExtrinsic> {
	// let mut extrinsics = Vec::new();
	let at = BlockId::Number(1);

	let mut block_builder = client.new_block_at(&at, digest, false).unwrap();
	block_builder.push(extrinsic_set_time(1 + MINIMUM_PERIOD_FOR_BLOCKS)).unwrap();
	block_builder.push(extrinsic_set_validation_data()).unwrap();

	// Creating those is surprisingly costly, so let's only do it once and later just `clone` them.
	let src = Sr25519Keyring::Alice.pair();

	let mut extrinsics = Vec::new();
	// Add as many tranfer extrinsics as possible into a single block.
	block_builder.record_valid_extrinsics_and_revert_changes(|api| {
		for nonce in 0.. {
			let extrinsic: OpaqueExtrinsic = create_extrinsic(
				client,
				src.clone(),
				SystemCall::remark { remark: vec![] },
				Some(nonce),
			)
			.into();
			match api.apply_extrinsic_with_context(
				&at,
				sp_core::ExecutionContext::BlockConstruction,
				extrinsic.clone(),
			) {
				Ok(Err(TransactionValidityError::Invalid(
					InvalidTransaction::ExhaustsResources,
				))) => break,
				Ok(Err(error)) => panic!("{}", error),
				Ok(_) => extrinsics.push(extrinsic),
				Err(error) => panic!("{}", error),
			};
		}
		extrinsics.clone()
	});
	extrinsics
}

fn create_keystore() -> sp_keystore::testing::KeyStore {
	let keystore = sp_keystore::testing::KeyStore::new();
	let secret_uri = "//Alice";
	let key_pair =
		sp_core::sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");
	SyncCryptoStore::insert_unknown(&keystore, AURA, secret_uri, key_pair.public().as_ref())
		.expect("Inserts unknown key");
	keystore
}

fn calculate_shuffling_seed(
	keystore: &sp_keystore::testing::KeyStore,
	prev_seed: &sp_core::H256,
	pub_key: &sp_consensus_aura::sr25519::AuthorityId,
) -> ShufflingSeed {
	let transcript = VRFTranscriptData {
		label: b"shuffling_seed",
		items: vec![("prev_seed", VRFTranscriptValue::Bytes(prev_seed.as_bytes().to_vec()))],
	};

	let signature = SyncCryptoStore::sr25519_vrf_sign(
		keystore,
		AURA,
		&pub_key.as_slice().try_into().unwrap(),
		transcript.clone(),
	)
	.unwrap()
	.unwrap();

	ShufflingSeed {
		seed: signature.output.to_bytes().into(),
		proof: signature.proof.to_bytes().into(),
	}
}

fn create_digest(slot: u64) -> sp_runtime::generic::Digest {
	let digest_item = <DigestItem as CompatibleDigestItem<AuthoritySignature>>::aura_pre_digest(
		sp_consensus_slots::Slot::from(slot),
	);
	sp_runtime::generic::Digest { logs: vec![digest_item] }
}

fn block_production(criterion: &mut Criterion) {
	sp_tracing::try_init_simple();

	let runtime = tokio::runtime::Runtime::new().expect("creating tokio runtime doesn't fail; qed");
	let tokio_handle = runtime.handle().clone();
	let config = new_config(tokio_handle);
	const SLOT_NUMBER: u64 = 1;

	let sc_service::PartialComponents {
		client,
		backend: _,
		task_manager: _,
		import_queue: _,
		keystore_container: _,
		select_chain: _,
		transaction_pool: _,
		other: _,
	} = mangata_node::service::new_partial(&config, parachain_build_import_queue).unwrap();

	let c = &*client;
	let mut block_builder = client.new_block(Default::default()).unwrap();
	block_builder.push(extrinsic_set_time(1)).unwrap();
	let genesis_block = block_builder.build_with_seed(Default::default()).unwrap();
	let first_block = genesis_block.block.clone();
	let prev_seed = genesis_block.block.header().seed().seed;
	import_block(c, genesis_block);

	let digests = create_digest(SLOT_NUMBER);

	// we already have access to keystore_container but it uses LocalKeystore that in opposite
	// to sp_keystore::testing::KeyStore::new() does not allow for custom keys registration
	let keystore = create_keystore();
	let authorities = &c
		.runtime_api()
		.authorities(&BlockId::Number(c.chain_info().best_number))
		.unwrap();
	let seed = calculate_shuffling_seed(&keystore, &prev_seed, &authorities[SLOT_NUMBER as usize]);
	let txs = prepare_benchmark(&client, digests.clone());
	let mut group = criterion.benchmark_group("Block production");

	{
		let mut block_builder =
			client.new_block_at(&BlockId::Number(1), digests.clone(), false).unwrap();
		block_builder.push(extrinsic_set_time(1 + MINIMUM_PERIOD_FOR_BLOCKS)).unwrap();
		block_builder.push(extrinsic_set_validation_data().clone()).unwrap();
		let (block, _, _) = block_builder.build_with_seed(seed.clone()).unwrap().into_inner();
		group.bench_function("empty block", |b| {
			b.iter_batched(
				|| block.clone(),
				|block| {
					c.runtime_api().execute_block(&BlockId::Number(1), block.clone()).unwrap();
				},
				BatchSize::SmallInput,
			)
		});
	}

	cfg_if::cfg_if! {
		if #[cfg(feature = "disable-execution")] {
			let mut block_builder = client.new_block_at(&BlockId::Number(1), digests, false).unwrap();
			block_builder.push(extrinsic_set_time(1 + MINIMUM_PERIOD_FOR_BLOCKS)).unwrap();
			block_builder.push(extrinsic_set_validation_data().clone()).unwrap();
			block_builder.record_valid_extrinsics_and_revert_changes(|_| txs.clone());
			let (mut block, _, _) = block_builder.build_with_seed(seed.clone()).unwrap().into_inner();
			// make all txs executed right away instead of delaying
			block.header.count = 2;
			group.bench_function("full block shuffling without executing extrinsics", |b| {
				b.iter_batched(
					|| block.clone(),
					|block| {
						c.runtime_api().execute_block(&BlockId::Number(1), block.clone()).unwrap();
					},
					BatchSize::SmallInput,
				)
			});
		}
	};
	{
		let txs = (0..10000)
			.map(|nonce| {
				create_extrinsic(
					&client,
					Sr25519Keyring::Alice.pair(),
					SystemCall::remark { remark: vec![] },
					Some(nonce),
				)
				.into()
			})
			.collect::<Vec<OpaqueExtrinsic>>();

		let header = mangata_runtime::Header::new(
			2,
			Default::default(),
			Default::default(),
			first_block.header.hash(),
			Default::default(),
		);

		assert_eq!(first_block.header.hash(), c.chain_info().best_hash);
		let api = c.runtime_api();
		let block_id = BlockId::Number(1);
		api.initialize_block_with_context(
			&block_id,
			sp_core::ExecutionContext::BlockConstruction,
			&header,
		)
		.unwrap();

		let mut cnt = 0;
		let now = std::time::Instant::now();
		for tx in txs {
			if let Ok(Ok(Ok(_))) = api.apply_extrinsic_with_context(
				&BlockId::Number(1),
				sp_core::ExecutionContext::BlockConstruction,
				tx.clone(),
			) {
				cnt += 1;
			} else {
				break;
			}
		}
		let elapsed_micros = now.elapsed().as_micros();
		println!(
			"avarege execution time of {} noop extrinsic : {} microseconds => {}",
			cnt,
			elapsed_micros,
			elapsed_micros / cnt
		);
	}
}

criterion_group!(benches, block_production);
criterion_main!(benches);
