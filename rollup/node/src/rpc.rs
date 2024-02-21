//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sc_client_api::BlockBackend;

use rollup_runtime::runtime_config::{
	opaque::Block,
	types::{AccountId, Balance, Nonce, TokenId},
};

use metamask_signature_rpc::MetamaskSignatureApiServer;
use ver_api::VerNonceApi;

pub use sc_rpc_api::DenyUnsafe;

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + BlockBackend<Block> + 'static,
	C: Send + Sync + 'static,
	C::Api: pallet_transaction_payment_mangata_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: xyk_rpc::XykRuntimeApi<Block, Balance, TokenId, AccountId>,
	C::Api: proof_of_stake_rpc::ProofOfStakeRuntimeApi<Block, Balance, TokenId, AccountId>,
	C::Api: metamask_signature_rpc::MetamaskSignatureRuntimeApi<Block>,
	C::Api: BlockBuilder<Block>,
	C::Api: VerNonceApi<Block, AccountId>,
	P: TransactionPool + 'static,
{
	use metamask_signature_rpc::MetamaskSignature;
	use pallet_transaction_payment_mangata_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use proof_of_stake_rpc::{ProofOfStake, ProofOfStakeApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};
	use xyk_rpc::{Xyk, XykApiServer};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(Xyk::new(client.clone()).into_rpc())?;
	module.merge(ProofOfStake::new(client.clone()).into_rpc())?;
	module.merge(MetamaskSignature::new(client).into_rpc())?;

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

	Ok(module)
}
