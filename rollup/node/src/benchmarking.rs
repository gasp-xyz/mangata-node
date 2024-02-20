//! Setup code for [`super::command`] which would otherwise bloat that module.
//!
//! Should only be used for benchmarking as it may break in other contexts.

use crate::service::Block;
use rollup_runtime as runtime;
use rollup_runtime::config::frame_system::BlockHashCount;
use runtime::{AccountId, Balance, RuntimeApi, SystemCall, TokenId, TokensCall};
use sc_cli::Result;
use sc_client_api::BlockBackend;
use sc_executor::WasmExecutor;
use sp_api::ProvideRuntimeApi;
use sp_core::{crypto::key_types::AURA, Encode, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_keystore::Keystore;
use sp_runtime::{traits::Zero, OpaqueExtrinsic, SaturatedConversion};
use std::{
	sync::{Arc, Mutex},
	time::Duration,
};
use substrate_frame_rpc_system::AccountNonceApi;

#[cfg(not(feature = "runtime-benchmarks"))]
type HostFunctions = sp_io::SubstrateHostFunctions;

#[cfg(feature = "runtime-benchmarks")]
type HostFunctions =
	(sp_io::SubstrateHostFunctions, frame_benchmarking::benchmarking::HostFunctions);

type WasmFullClient = sc_service::TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>;

pub fn fetch_nonce(client: &WasmFullClient, account: sp_core::sr25519::Pair) -> u32 {
	let best_hash = client.chain_info().best_hash;
	client
		.runtime_api()
		.account_nonce(best_hash, account.public().into())
		.expect("Fetching account nonce works; qed")
}

/// Create a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
pub fn create_benchmark_extrinsic(
	client: &WasmFullClient,
	sender: sp_core::sr25519::Pair,
	call: runtime::RuntimeCall,
	nonce: Option<u32>,
) -> runtime::UncheckedExtrinsic {
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	let best_block = client.chain_info().best_number;
	let nonce = nonce.unwrap_or_else(|| fetch_nonce(client, sender.clone()));

	let period =
		BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
	let extra: runtime::SignedExtra = (
		frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
		frame_system::CheckTxVersion::<runtime::Runtime>::new(),
		frame_system::CheckGenesis::<runtime::Runtime>::new(),
		frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
			period,
			best_block.saturated_into(),
		)),
		frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<runtime::Runtime>::new(),
		pallet_transaction_payment_mangata::ChargeTransactionPayment::<runtime::Runtime>::from(0),
		frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
	);

	let raw_payload = runtime::SignedPayload::from_raw(
		call.clone(),
		extra.clone(),
		(
			runtime::VERSION.spec_version,
			runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| sender.sign(e));

	runtime::UncheckedExtrinsic::new_signed(
		call,
		sp_runtime::AccountId32::from(sender.public()).into(),
		runtime::Signature::Sr25519(signature),
		extra,
	)
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub async fn inherent_benchmark_data(
	prev_seed: [u8; 32],
	duration: Duration,
) -> Result<InherentData> {
	let keystore = sp_keystore::testing::MemoryKeystore::new();
	let secret_uri = "//Alice";
	let key_pair =
		sp_core::sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");
	keystore
		.insert(AURA, secret_uri, key_pair.public().as_ref())
		.expect("Inserts unknown key");

	let seed =
		sp_ver::calculate_next_seed_from_bytes(&keystore, &key_pair.public(), prev_seed.to_vec())
			.unwrap();

	let mut inherent_data = InherentData::new();

	sp_timestamp::InherentDataProvider::new(duration.into())
		.provide_inherent_data(&mut inherent_data)
		.await
		.map_err(|e| format!("creating inherent data: {:?}", e))?;

	sp_ver::RandomSeedInherentDataProvider(seed)
		.provide_inherent_data(&mut inherent_data)
		.await
		.map_err(|e| format!("creating inherent data: {:?}", e))?;

	Ok(inherent_data)
}

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder {
	client: Arc<Mutex<WasmFullClient>>,
}

impl RemarkBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<Mutex<WasmFullClient>>) -> Self {
		Self { client }
	}
}

impl frame_benchmarking_cli::ExtrinsicBuilder for RemarkBuilder {
	fn pallet(&self) -> &str {
		"system"
	}

	fn extrinsic(&self) -> &str {
		"remark"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			&*self.client.lock().unwrap(),
			acc,
			SystemCall::remark { remark: vec![] }.into(),
			Some(nonce),
		)
		.into();

		Ok(extrinsic)
	}
}

/// Generates `Balances::TransferKeepAlive` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct TransferKeepAliveBuilder {
	client: Arc<Mutex<WasmFullClient>>,
	dest: AccountId,
	value: Balance,
}

impl TransferKeepAliveBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<Mutex<WasmFullClient>>, dest: AccountId, value: Balance) -> Self {
		Self { client, dest, value }
	}
}

impl frame_benchmarking_cli::ExtrinsicBuilder for TransferKeepAliveBuilder {
	fn pallet(&self) -> &str {
		"balances"
	}

	fn extrinsic(&self) -> &str {
		"transfer_keep_alive"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			&*self.client.lock().unwrap(),
			acc,
			TokensCall::transfer_keep_alive {
				dest: self.dest.clone().into(),
				currency_id: TokenId::zero(),
				amount: self.value,
			}
			.into(),
			Some(nonce),
		)
		.into();

		Ok(extrinsic)
	}
}
