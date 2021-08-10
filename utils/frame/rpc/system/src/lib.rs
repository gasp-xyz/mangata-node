// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
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

use codec::{self, Codec, Decode, Encode};
use futures::future::{ready, TryFutureExt};
use jsonrpc_core::{
    futures::future::{self as rpc_future, result, Future},
    Error as RpcError, ErrorCode,
};
use jsonrpc_derive::rpc;
use sc_client_api::light::{future_header, Fetcher, RemoteBlockchain, RemoteCallRequest};
use sc_client_api::{
	client::{BlockBackend},
};
use sc_rpc_api::DenyUnsafe;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as ClientError, HeaderBackend};
use sp_core::{hexdisplay::HexDisplay, Bytes};
use sp_runtime::{generic::BlockId, traits};
use sp_transaction_pool::{InPoolTransaction, TransactionPool};
use sp_api::{ApiExt, ApiRef, ProvideRuntimeApi, TransactionOutcome};

pub use self::gen_client::Client as SystemClient;
pub use frame_system_rpc_runtime_api::AccountNonceApi;
use extrinsic_info_runtime_api::runtime_api::ExtrinsicInfoRuntimeApi;

/// Future that resolves to account nonce.
pub type FutureResult<T> = Box<dyn Future<Item = T, Error = RpcError> + Send>;

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
        FullSystem {
            client,
            pool,
            deny_unsafe,
            _marker: Default::default(),
        }
    }
}

impl<'a, P, C, Block, AccountId, Index> SystemApi<<Block as traits::Block>::Hash, AccountId, Index>
    for FullSystem<P, C, Block>
where
    C: sp_api::ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C: BlockBackend<Block>,
    C: Send + Sync + 'static,
    C::Api: AccountNonceApi<Block, AccountId, Index>,
    C::Api: BlockBuilder<Block>,
    C::Api: ExtrinsicInfoRuntimeApi<Block>,
    P: TransactionPool + 'static,
    Block: traits::Block,
    AccountId: Clone + std::fmt::Display + Codec + Decode + std::cmp::PartialEq,
    Index: Clone + std::fmt::Display + Codec + Send + traits::AtLeast32Bit + 'static,
{
    fn nonce(&self, account: AccountId) -> FutureResult<Index> {
        let get_nonce = || {
            let api = self.client.runtime_api();
            let best_hash = self.client.info().best_hash;
            let at = BlockId::hash(best_hash);

            let nonce = api
                .account_nonce(&at, account.clone())
                .map_err(|e| RpcError {
                    code: ErrorCode::ServerError(Error::RuntimeError.into()),
                    message: "Unable to query nonce.".into(),
                    data: Some(format!("{:?}", e).into()),
                })?;

            let best_block_extrinsics: Vec<Block::Extrinsic> =
                self.client.block_body(&at)
                    .map_err(|e| RpcError {
                    code: ErrorCode::ServerError(Error::RuntimeError.into()),
                    message: "Failed to get parent blocks extrinsics.".into(),
                    data: Some(format!("{:?}", e).into()),
                })?
                .unwrap_or_default();

            Ok(adjust_nonce::<P, C, AccountId, Index, Block>(&*self.pool, account, nonce, best_block_extrinsics, at, &api))
        };

        Box::new(result(get_nonce()))
    }

    fn dry_run(
        &self,
        extrinsic: Bytes,
        at: Option<<Block as traits::Block>::Hash>,
    ) -> FutureResult<Bytes> {
        if let Err(err) = self.deny_unsafe.check_if_safe() {
            return Box::new(rpc_future::err(err.into()));
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

            let _result = "test";
            let result = api.apply_extrinsic(&at, uxt).map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Unable to dry run extrinsic.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;

            Ok(Encode::encode(&result).into())
        };

        Box::new(result(dry_run()))
    }
}

/// An implementation of System-specific RPC methods on light client.
pub struct LightSystem<P: TransactionPool, C, F, Block> {
    client: Arc<C>,
    remote_blockchain: Arc<dyn RemoteBlockchain<Block>>,
    fetcher: Arc<F>,
    pool: Arc<P>,
}

impl<P: TransactionPool, C, F, Block> LightSystem<P, C, F, Block> {
    /// Create new `LightSystem`.
    pub fn new(
        client: Arc<C>,
        remote_blockchain: Arc<dyn RemoteBlockchain<Block>>,
        fetcher: Arc<F>,
        pool: Arc<P>,
    ) -> Self {
        LightSystem {
            client,
            remote_blockchain,
            fetcher,
            pool,
        }
    }
}

impl<P, C, F, Block, AccountId, Index> SystemApi<<Block as traits::Block>::Hash, AccountId, Index>
    for LightSystem<P, C, F, Block>
where
    P: TransactionPool + 'static,
    C: HeaderBackend<Block>,
    C: Send + Sync + 'static,
    F: Fetcher<Block> + 'static,
    Block: traits::Block,
    AccountId: Clone + std::fmt::Display + Codec + Send + 'static + Decode + std::cmp::PartialEq,
    Index: Clone + std::fmt::Display + Codec + Send + traits::AtLeast32Bit + 'static,
{
    fn nonce(&self, account: AccountId) -> FutureResult<Index> {
        let best_hash = self.client.info().best_hash;
        let best_id = BlockId::hash(best_hash);
        let future_best_header = future_header(&*self.remote_blockchain, &*self.fetcher, best_id);
        let fetcher = self.fetcher.clone();
        let call_data = account.encode();
        let future_best_header = future_best_header.and_then(move |maybe_best_header| {
            ready(match maybe_best_header {
                Some(best_header) => Ok(best_header),
                None => Err(ClientError::UnknownBlock(format!("{}", best_hash))),
            })
        });
        let future_nonce = future_best_header
            .and_then(move |best_header| {
                fetcher.remote_call(RemoteCallRequest {
                    block: best_hash,
                    header: best_header,
                    method: "AccountNonceApi_account_nonce".into(),
                    call_data,
                    retry_count: None,
                })
            })
            .compat();
        let future_nonce = future_nonce.and_then(|nonce| {
            Decode::decode(&mut &nonce[..])
                .map_err(|e| ClientError::CallResultDecode("Cannot decode account nonce", e))
        });
        let future_nonce = future_nonce.map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to query nonce.".into(),
            data: Some(format!("{:?}", e).into()),
        });

        #[allow(unused_variables)]
        let pool = self.pool.clone();

        // // FIXME Fetch block extrinsics rather than use default. 
        // let best_block_extrinsics: Vec<Block::Extrinsic> =
        //         Default::default();
        // // FIXME Pass in the api?
        // let future_nonce = future_nonce.map(move |nonce| adjust_nonce::<P, C, AccountId, Index, Block>(&*pool, account, nonce, best_block_extrinsics, best_id, ));

        Box::new(future_nonce)
    }

    fn dry_run(
        &self,
        _extrinsic: Bytes,
        _at: Option<<Block as traits::Block>::Hash>,
    ) -> FutureResult<Bytes> {
        Box::new(result(Err(RpcError {
            code: ErrorCode::MethodNotFound,
            message: "Unable to dry run extrinsic.".into(),
            data: None,
        })))
    }
}

/// Adjust account nonce from state, so that tx with the nonce will be
/// placed after all ready txpool transactions.
fn adjust_nonce<'a, P, C, AccountId, Index, Block>(pool: &P, account: AccountId, nonce: Index, block_extrinsics: Vec<Block::Extrinsic>, block_id: BlockId<Block>, api: &ApiRef<'a, C::Api>) -> Index
where
    P: TransactionPool,
    AccountId: Clone + std::fmt::Display + Encode + Decode + std::cmp::PartialEq,
    Index: Clone + std::fmt::Display + Encode + traits::AtLeast32Bit + 'static,
    Block: traits::Block,
    C: ProvideRuntimeApi<Block> + 'a,
    C::Api: ExtrinsicInfoRuntimeApi<Block>,
{
    log::debug!(target: "rpc", "State nonce for {}: {}", account, nonce);

    let mut current_nonce = nonce.clone();

    for tx in block_extrinsics.into_iter(){
        let (tx_who, tx_nonce): (sp_runtime::AccountId32, u32) = api.execute_in_transaction(|api| {
            // store deserialized data and revert state modification caused by 'get_info' call
            match api.get_info(&block_id, tx.clone()){
                Ok(result) => TransactionOutcome::Rollback(result),
                Err(_) => TransactionOutcome::Rollback(None)
            }
        })
        // This should always unwrap and never or_else, as it is expected to.
        // Otherwise nonce for AccountId32=0 will be miscalculated as 0
        .map_or_else( || Default::default(), |info| (info.who, info.nonce));

        if (<AccountId>::decode(&mut &<[u8; 32]>::from(tx_who)[..]).unwrap() == account) && (Index::from(tx_nonce) == current_nonce) {
            current_nonce += traits::One::one();
        }

    };

    // Now we need to query the transaction pool
    // and find transactions originating from the same sender.
    //
    // Since extrinsics are opaque to us, we look for them using
    // `provides` tag. And increment the nonce if we find a transaction
    // that matches the current one.
    
    let mut current_tag = (account.clone(), current_nonce.clone()).encode();
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

// TODO Trello 91 - Enable commented out tests
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     use futures::executor::block_on;
//     use sc_transaction_pool::BasicPool;
//     use sp_runtime::{
//         transaction_validity::{InvalidTransaction, TransactionValidityError},
//         ApplyExtrinsicResult,
//     };
//     use substrate_test_runtime_client::{runtime::Transfer, AccountKeyring};
//
//     #[test]
//     fn should_return_next_nonce_for_some_account() {
//         sp_tracing::try_init_simple();
//
//         // given
//         let client = Arc::new(substrate_test_runtime_client::new());
//         let spawner = sp_core::testing::TaskExecutor::new();
//         let pool = BasicPool::new_full(Default::default(), None, spawner, client.clone());
//
//         let source = sp_runtime::transaction_validity::TransactionSource::External;
//         let new_transaction = |nonce: u64| {
//             let t = Transfer {
//                 from: AccountKeyring::Alice.into(),
//                 to: AccountKeyring::Bob.into(),
//                 amount: 5,
//                 nonce,
//             };
//             t.into_signed_tx()
//         };
//         // Populate the pool
//         let ext0 = new_transaction(0);
//         block_on(pool.submit_one(&BlockId::number(0), source, ext0)).unwrap();
//         let ext1 = new_transaction(1);
//         block_on(pool.submit_one(&BlockId::number(0), source, ext1)).unwrap();
//
//         let accounts = FullSystem::new(client, pool, DenyUnsafe::Yes);
//
//         // when
//         let nonce = accounts.nonce(AccountKeyring::Alice.into());
//
//         // then
//         assert_eq!(nonce.wait().unwrap(), 2);
//     }
//
//     #[test]
//     fn dry_run_should_deny_unsafe() {
//         sp_tracing::try_init_simple();
//
//         // given
//         let client = Arc::new(substrate_test_runtime_client::new());
//         let spawner = sp_core::testing::TaskExecutor::new();
//         let pool = BasicPool::new_full(Default::default(), None, spawner, client.clone());
//
//         let accounts = FullSystem::new(client, pool, DenyUnsafe::Yes);
//
//         // when
//         let res = accounts.dry_run(vec![].into(), None);
//
//         // then
//         assert_eq!(res.wait(), Err(RpcError::method_not_found()));
//     }
//
//     #[test]
//     fn dry_run_should_work() {
//         sp_tracing::try_init_simple();
//
//         // given
//         let client = Arc::new(substrate_test_runtime_client::new());
//         let spawner = sp_core::testing::TaskExecutor::new();
//         let pool = BasicPool::new_full(Default::default(), None, spawner, client.clone());
//
//         let accounts = FullSystem::new(client, pool, DenyUnsafe::No);
//
//         let tx = Transfer {
//             from: AccountKeyring::Alice.into(),
//             to: AccountKeyring::Bob.into(),
//             amount: 5,
//             nonce: 0,
//         }
//         .into_signed_tx();
//
//         // when
//         let res = accounts.dry_run(tx.encode().into(), None);
//
//         // then
//         let bytes = res.wait().unwrap().0;
//         let apply_res: ApplyExtrinsicResult = Decode::decode(&mut bytes.as_slice()).unwrap();
//         assert_eq!(apply_res, Ok(Ok(())));
//     }
//
//     #[test]
//     fn dry_run_should_indicate_error() {
//         sp_tracing::try_init_simple();
//
//         // given
//         let client = Arc::new(substrate_test_runtime_client::new());
//         let spawner = sp_core::testing::TaskExecutor::new();
//         let pool = BasicPool::new_full(Default::default(), None, spawner, client.clone());
//
//         let accounts = FullSystem::new(client, pool, DenyUnsafe::No);
//
//         let tx = Transfer {
//             from: AccountKeyring::Alice.into(),
//             to: AccountKeyring::Bob.into(),
//             amount: 5,
//             nonce: 100,
//         }
//         .into_signed_tx();
//
//         // when
//         let res = accounts.dry_run(tx.encode().into(), None);
//
//         // then
//         let bytes = res.wait().unwrap().0;
//         let apply_res: ApplyExtrinsicResult = Decode::decode(&mut bytes.as_slice()).unwrap();
//         assert_eq!(
//             apply_res,
//             Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
//         );
//     }
// }
