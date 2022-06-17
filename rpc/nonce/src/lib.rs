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
use futures::FutureExt;
use jsonrpc_core::{Error as RpcError, ErrorCode};
use jsonrpc_derive::rpc;
use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::{InPoolTransaction, TransactionPool};
use sp_api::ApiExt;
use sp_block_builder::BlockBuilder;
use sp_blockchain::HeaderBackend;
use sp_core::{hexdisplay::HexDisplay, Bytes};
use sp_runtime::{
	generic::BlockId,
	traits,
	traits::{Block as BlockT, Header as HeaderT},
	SaturatedConversion,
};
use std::convert::TryInto;

pub use self::gen_client::Client as SystemClient;
pub use frame_system_rpc_runtime_api::AccountNonceApi;
use sc_client_api::BlockBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::TransactionOutcome;
use ver_api::VerApi;

/// Future that resolves to account nonce.
type FutureResult<T> = jsonrpc_core::BoxFuture<Result<T, RpcError>>;

/// System RPC methods.
#[rpc]
pub trait SystemApi<BlockHash, AccountId, Index> {
	/// Returns the next valid index (aka nonce) for given account.
	///
	/// This method takes into consideration all pending transactions
	/// currently in the pool and if no transactions are found in the pool
	/// it fallbacks to query the index from the runtime (aka. state nonce).
	#[rpc(name = "system_accountNextIndex", alias("account_nextIndex"))]
	fn nonce(&self, account: AccountId) -> FutureResult<Index>;

	/// Dry run an extrinsic at a given block. Return SCALE encoded ApplyExtrinsicResult.
	#[rpc(name = "system_dryRun", alias("system_dryRunAt"))]
	fn dry_run(&self, extrinsic: Bytes, at: Option<BlockHash>) -> FutureResult<Bytes>;
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

/// An implementation of System-specific RPC methods on full client.
pub struct FullSystem<P: TransactionPool, C, B> {
	client: Arc<C>,
	pool: Arc<P>,
	deny_unsafe: DenyUnsafe,
	_marker: std::marker::PhantomData<B>,
}

impl<P: TransactionPool, C, B> FullSystem<P, C, B> {
	/// Create new `FullSystem` given client and transaction pool.
	pub fn new(client: Arc<C>, pool: Arc<P>, deny_unsafe: DenyUnsafe) -> Self {
		FullSystem { client, pool, deny_unsafe, _marker: Default::default() }
	}
}

impl<P, C, Block, AccountId, Index> SystemApi<<Block as traits::Block>::Hash, AccountId, Index>
	for FullSystem<P, C, Block>
where
	C: sp_api::ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C: BlockBackend<Block>,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C::Api: AccountNonceApi<Block, AccountId, Index>,
	C::Api: BlockBuilder<Block>,
	C::Api: VerApi<Block>,
	P: TransactionPool + 'static,
	Block: traits::Block,
	AccountId: Clone + std::fmt::Display + Codec + std::cmp::PartialEq,
	Index: Clone + std::fmt::Display + Codec + Send + traits::AtLeast32Bit + 'static,
{
	fn nonce(&self, account: AccountId) -> FutureResult<Index> {
		let get_nonce = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let mut nonce = api.account_nonce(&at, account.clone()).map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to query nonce.".into(),
				data: Some(format!("{:?}", e).into()),
			})?;

			for _ in 0..number_of_delayed_txs(self.client.clone(), account.clone()) {
				nonce += traits::One::one();
			}

			Ok(adjust_nonce(&*self.pool, account, nonce))
		};

		let res = get_nonce();
		async move { res }.boxed()
	}

	fn dry_run(
		&self,
		extrinsic: Bytes,
		at: Option<<Block as traits::Block>::Hash>,
	) -> FutureResult<Bytes> {
		if let Err(err) = self.deny_unsafe.check_if_safe() {
			return async move { Err(err.into()) }.boxed();
		}

		let dry_run = || {
			let api = self.client.runtime_api();
			let at = BlockId::<Block>::hash(at.unwrap_or_else(||
				// If the block hash is not supplied assume the best block.
				self.client.info().best_hash));

			let uxt: <Block as traits::Block>::Extrinsic = Decode::decode(&mut &*extrinsic)
				.map_err(|e| RpcError {
					code: ErrorCode::ServerError(Error::DecodeError.into()),
					message: "Unable to dry run extrinsic.".into(),
					data: Some(format!("{:?}", e).into()),
				})?;

			let result = api.apply_extrinsic(&at, uxt).map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to dry run extrinsic.".into(),
				data: Some(format!("{:?}", e).into()),
			})?;

			Ok(Encode::encode(&result).into())
		};

		let res = dry_run();

		async move { res }.boxed()
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

fn number_of_delayed_txs<C, Block, AccountId>(client: Arc<C>, signer_id: AccountId) -> u32
where
	C: HeaderBackend<Block>,
	C: BlockBackend<Block>,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C::Api: BlockBuilder<Block>,
	C::Api: VerApi<Block>,
	Block: BlockT,
	AccountId: Clone + std::fmt::Display + Codec + std::cmp::PartialEq,
{
	let api = client.runtime_api();
	let best = client.info().best_hash;
	let at = BlockId::<Block>::hash(best);

	let previous_block_header = client.header(at).unwrap().unwrap();

	let best_block_extrinsics: Vec<Block::Extrinsic> = client
		.block_body(&at)
		.map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Failed to get parent blocks extrinsics.".into(),
			data: Some(format!("{:?}", e).into()),
		})
		.unwrap()
		.unwrap_or_default();

	let result = best_block_extrinsics
		.into_iter()
		.take((*previous_block_header.count()).saturated_into::<usize>())
		.map(|tx|
         //TODO limit to unexecuted txs
             api.execute_in_transaction(|api| {
                    // store deserialized data and revert state modification caused by 'get_info' call
                    match api.get_signer(&at, tx.clone()) {
                        Ok(result) => TransactionOutcome::Rollback(result),
                        Err(_) => TransactionOutcome::Rollback(None),
                    }
            }))
		.filter(|result| {
			if let Some((who, _nonce)) = result {
				<AccountId>::decode(&mut &<[u8; 32]>::from(who.clone())[..]).unwrap() == signer_id
			} else {
				false
			}
		})
		.count()
		.try_into()
		.unwrap();
	log::debug!(target: "rpc_nonce", "advance nonce for {} : {}", signer_id, result);
	result
}
