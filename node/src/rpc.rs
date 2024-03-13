// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Parachain-specific RPCs implementation.

#![warn(missing_docs)]

use std::sync::Arc;

use common_runtime::{
	opaque::Block,
	types::{AccountId, Balance, Nonce, TokenId},
};

use metamask_signature_rpc::MetamaskSignatureApiServer;
use sc_client_api::AuxStore;
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
		+ sc_client_api::BlockBackend<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: pallet_transaction_payment_mangata_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: xyk_rpc::XykRuntimeApi<Block, Balance, TokenId, AccountId>,
	C::Api: proof_of_stake_rpc::ProofOfStakeRuntimeApi<Block, Balance, TokenId, AccountId>,
	C::Api: metamask_signature_rpc::MetamaskSignatureRuntimeApi<Block>,
	C::Api: rolldown_runtime_api::RolldownRuntimeApi<Block, pallet_rolldown::messages::L1Update>,
	C::Api: BlockBuilder<Block>,
	C::Api: VerNonceApi<Block, AccountId>,
	P: TransactionPool + Sync + Send + 'static,
{
	use metamask_signature_rpc::MetamaskSignature;
	use pallet_transaction_payment_mangata_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use proof_of_stake_rpc::{ProofOfStake, ProofOfStakeApiServer};
	use rolldown_rpc::{Rolldown, RolldownApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};
	use xyk_rpc::{Xyk, XykApiServer};

	let mut module = RpcExtension::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(Xyk::new(client.clone()).into_rpc())?;
	module.merge(Rolldown::new(client.clone()).into_rpc())?;
	module.merge(ProofOfStake::new(client.clone()).into_rpc())?;
	module.merge(MetamaskSignature::new(client).into_rpc())?;

	Ok(module)
}
