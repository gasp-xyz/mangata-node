// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! System FRAME specific RPC methods.

use std::sync::Arc;

use codec::{Codec, Decode, Encode};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::{InPoolTransaction, TransactionPool};

use sp_block_builder::BlockBuilder;
use sp_blockchain::HeaderBackend;
use sp_core::{hexdisplay::HexDisplay, Bytes};
use sp_runtime::{traits};

pub use frame_system_rpc_runtime_api::AccountNonceApi;
use sc_client_api::BlockBackend;
use sp_api::ProvideRuntimeApi;

use ver_api::VerNonceApi;

/// System RPC methods.
#[rpc(client, server)]
pub trait SystemApi<BlockHash, AccountId, Index> {
	/// Returns the next valid index (aka nonce) for given account.
	///
	/// This method takes into consideration all pending transactions
	/// currently in the pool and if no transactions are found in the pool
	/// it fallbacks to query the index from the runtime (aka. state nonce).
	#[method(name = "system_accountNextIndex", aliases = ["account_nextIndex"])]
	async fn nonce(&self, account: AccountId) -> RpcResult<Index>;

	/// Dry run an extrinsic at a given block. Return SCALE encoded ApplyExtrinsicResult.
	#[method(name = "system_dryRun", aliases = ["system_dryRunAt"])]
	async fn dry_run(&self, extrinsic: Bytes, at: Option<BlockHash>) -> RpcResult<Bytes>;
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

/// An implementation of System-specific RPC methods on full client.
pub struct System<P: TransactionPool, C, B> {
	client: Arc<C>,
	pool: Arc<P>,
	deny_unsafe: DenyUnsafe,
	_marker: std::marker::PhantomData<B>,
}

impl<P: TransactionPool, C, B> System<P, C, B> {
	/// Create new `FullSystem` given client and transaction pool.
	pub fn new(client: Arc<C>, pool: Arc<P>, deny_unsafe: DenyUnsafe) -> Self {
		Self { client, pool, deny_unsafe, _marker: Default::default() }
	}
}

#[async_trait]
impl<P, C, Block, AccountId, Index>
	SystemApiServer<<Block as traits::Block>::Hash, AccountId, Index> for System<P, C, Block>
where
	C: sp_api::ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C: BlockBackend<Block>,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C::Api: AccountNonceApi<Block, AccountId, Index>,
	C::Api: BlockBuilder<Block>,
	C::Api: VerNonceApi<Block, AccountId>,
	P: TransactionPool + 'static,
	Block: traits::Block,
	AccountId: Clone + std::fmt::Display + Codec + Send + 'static + std::cmp::PartialEq,
	Index: Clone + std::fmt::Display + Codec + Send + traits::AtLeast32Bit + 'static,
{
	async fn nonce(&self, account: AccountId) -> RpcResult<Index> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		let mut nonce = api.account_nonce(at, account.clone()).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query nonce.",
				Some(e.to_string()),
			))
		})?;

		let txs_in_queue = api.enqueued_txs_count(at, account.clone()).unwrap();
		for _ in 0..txs_in_queue {
			nonce += traits::One::one();
		}
		log::debug!(target: "rpc::nonce", "nonce for {} at block {} => {} ({} in queue)", account, at, nonce, txs_in_queue);

		Ok(adjust_nonce(&*self.pool, account, nonce))
	}

	async fn dry_run(
		&self,
		extrinsic: Bytes,
		at: Option<<Block as traits::Block>::Hash>,
	) -> RpcResult<Bytes> {
		self.deny_unsafe.check_if_safe()?;
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		let uxt: <Block as traits::Block>::Extrinsic =
			Decode::decode(&mut &*extrinsic).map_err(|e| {
				CallError::Custom(ErrorObject::owned(
					Error::DecodeError.into(),
					"Unable to dry run extrinsic",
					Some(e.to_string()),
				))
			})?;

		let result = api.apply_extrinsic(at, uxt).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to dry run extrinsic.",
				Some(e.to_string()),
			))
		})?;

		Ok(Encode::encode(&result).into())
	}
}

/// Adjust account nonce from state, so that tx with the nonce will be
/// placed after all ready txpool transactions.
fn adjust_nonce<P, AccountId, Index>(pool: &P, account: AccountId, nonce: Index) -> Index
where
	P: TransactionPool,
	AccountId: Clone + std::fmt::Display + Encode,
	Index: Clone + std::fmt::Display + Encode + traits::AtLeast32Bit + 'static,
{
	log::debug!(target: "rpc", "State nonce for {}: {}", account, nonce);
	// Now we need to query the transaction pool
	// and find transactions originating from the same sender.
	//
	// Since extrinsics are opaque to us, we look for them using
	// `provides` tag. And increment the nonce if we find a transaction
	// that matches the current one.
	let mut current_nonce = nonce.clone();
	let mut current_tag = (account.clone(), nonce).encode();
	for tx in pool.ready() {
		log::debug!(
			target: "rpc",
			"Current nonce to {}, checking {} vs {:?}",
			current_nonce,
			HexDisplay::from(&current_tag),
			tx.provides().iter().map(|x| format!("{}", HexDisplay::from(x))).collect::<Vec<_>>(),
		);
		// since transactions in `ready()` need to be ordered by nonce
		// it's fine to continue with current iterator.
		if tx.provides().get(0) == Some(&current_tag) {
			current_nonce += traits::One::one();
			current_tag = (account.clone(), current_nonce.clone()).encode();
		}
	}

	current_nonce
}
