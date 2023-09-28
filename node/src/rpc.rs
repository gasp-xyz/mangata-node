//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use mangata_types::{AccountId, Balance, Block, BlockNumber, Index as Nonce, TokenId};

use sc_client_api::{AuxStore, BlockBackend};
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use ver_api::VerNonceApi;

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Full client dependencies
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ BlockBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: pallet_transaction_payment_mangata_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: xyk_rpc::XykRuntimeApi<Block, Balance, TokenId, AccountId>,
	C::Api: BlockBuilder<Block>,
	C::Api: VerNonceApi<Block, AccountId>,
	P: TransactionPool + Sync + Send + 'static,
{
	use pallet_transaction_payment_mangata_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use proof_of_stake_rpc::{ProofOfStake, ProofOfStakeApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};
	use xyk_rpc::{Xyk, XykApiServer};

	let mut module = RpcExtension::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(Xyk::new(client.clone()).into_rpc())?;
	module.merge(ProofOfStake::new(client.clone()).into_rpc())?;

	Ok(module)
}
