// use crate::service::{create_extrinsic, FullClient};

// use node_runtime::SystemCall;
use codec::Encode;
use sc_cli::Result;
use sc_client_api::BlockBackend;
use sp_api::ProvideRuntimeApi;
use sp_core::{crypto::key_types::AURA, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_keystore::SyncCryptoStore;
use sp_runtime::{generic, OpaqueExtrinsic, SaturatedConversion};
use substrate_frame_rpc_system::AccountNonceApi;

use std::{cell::RefCell, rc::Rc, time::Duration};

#[cfg(feature = "mangata-kusama")]
pub type KusamaFullClient = crate::service::FullClient<
	mangata_kusama_runtime::RuntimeApi,
	crate::service::MangataKusamaRuntimeExecutor,
>;

#[cfg(feature = "mangata-kusama")]
pub fn fetch_nonce(client: &KusamaFullClient, account: sp_core::sr25519::Pair) -> u32 {
	let best_hash = client.chain_info().best_hash;
	client
		.runtime_api()
		.account_nonce(best_hash, account.public().into())
		.expect("Fetching account nonce works; qed")
}

#[cfg(feature = "mangata-kusama")]
pub fn create_extrinsic(
	client: &KusamaFullClient,
	sender: sp_core::sr25519::Pair,
	function: impl Into<mangata_kusama_runtime::RuntimeCall>,
	nonce: Option<u32>,
) -> mangata_kusama_runtime::UncheckedExtrinsic {
	let function = function.into();
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	let best_block = client.chain_info().best_number;
	let nonce = nonce.unwrap_or_else(|| fetch_nonce(client, sender.clone()));

	let period = mangata_kusama_runtime::BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;
	let tip = 0;
	let extra: mangata_kusama_runtime::SignedExtra = (
		frame_system::CheckSpecVersion::<mangata_kusama_runtime::Runtime>::new(),
		frame_system::CheckTxVersion::<mangata_kusama_runtime::Runtime>::new(),
		frame_system::CheckGenesis::<mangata_kusama_runtime::Runtime>::new(),
		frame_system::CheckEra::<mangata_kusama_runtime::Runtime>::from(generic::Era::mortal(
			period,
			best_block.saturated_into(),
		)),
		frame_system::CheckNonce::<mangata_kusama_runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<mangata_kusama_runtime::Runtime>::new(),
		pallet_transaction_payment_mangata::ChargeTransactionPayment::<
			mangata_kusama_runtime::Runtime,
		>::from(tip),
	);

	let raw_payload = mangata_kusama_runtime::SignedPayload::from_raw(
		function.clone(),
		extra.clone(),
		(
			mangata_kusama_runtime::VERSION.spec_version,
			mangata_kusama_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| sender.sign(e));

	mangata_kusama_runtime::UncheckedExtrinsic::new_signed(
		function,
		sp_runtime::AccountId32::from(sender.public()).into(),
		mangata_kusama_runtime::Signature::Sr25519(signature),
		extra,
	)
}

/// Generates extrinsics for the `benchmark overhead` command.
#[cfg(feature = "mangata-kusama")]
pub struct BenchmarkExtrinsicBuilder {
	client: Rc<RefCell<KusamaFullClient>>,
}

#[cfg(feature = "mangata-kusama")]
impl BenchmarkExtrinsicBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Rc<RefCell<KusamaFullClient>>) -> Self {
		Self { client }
	}
}

#[cfg(feature = "mangata-kusama")]
impl frame_benchmarking_cli::ExtrinsicBuilder for BenchmarkExtrinsicBuilder {
	fn pallet(&self) -> &str {
		"system"
	}

	fn extrinsic(&self) -> &str {
		"remark"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_extrinsic(
			&*self.client.borrow(),
			acc,
			frame_system::Call::remark { remark: vec![] },
			Some(nonce),
		)
		.into();

		Ok(extrinsic)
	}
}

/// Generates inherent data for the `benchmark overhead` command.
pub async fn inherent_benchmark_data(
	prev_seed: [u8; 32],
	duration: Duration,
) -> Result<InherentData> {
	let keystore = sp_keystore::testing::KeyStore::new();
	let secret_uri = "//Alice";
	let key_pair =
		sp_core::sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");
	keystore
		.insert_unknown(AURA, secret_uri, key_pair.public().as_ref())
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

	cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider {
		current_para_block: 0,
		relay_offset: 0,
		relay_blocks_per_para_block: 2,
		para_blocks_per_relay_epoch: 0,
		relay_randomness_config: (),
		xcm_config: Default::default(),
		raw_downward_messages: Default::default(),
		raw_horizontal_messages: Default::default(),
	}
	.provide_inherent_data(&mut inherent_data)
	.await
	.map_err(|e| format!("creating inherent data: {:?}", e))?;

	Ok(inherent_data)
}
