// Copyright (C) 2021 Mangata team

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::{Block as BlockT, MaybeDisplay, MaybeFromStr}};
use std::sync::Arc;
use codec::Codec;
pub use xyk_runtime_api::XykApi as XykRuntimeApi;
use xyk_runtime_api::RpcResult;

#[rpc]
pub trait XykApi<BlockHash, Balance, ResponseType> {
    #[rpc(name = "xyk_calculate_sell_price")]
    fn calculate_sell_price(
        &self,
        input_reserve: Balance,
        output_reserve: Balance,
        sell_amount: Balance,
        at: Option<BlockHash>
    ) -> Result<ResponseType>;


    #[rpc(name = "xyk_calculate_buy_price")]
    fn calculate_buy_price(
        &self,
        input_reserve: Balance,
        output_reserve: Balance,
        buy_amount: Balance,
        at: Option<BlockHash>
    ) -> Result<ResponseType>;
}

pub struct Xyk<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, P> Xyk<C, P> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block, Balance> XykApi<<Block as BlockT>::Hash, Balance, RpcResult<Balance>>
for Xyk<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: XykRuntimeApi<Block, Balance>,
        Balance: Codec + MaybeDisplay + MaybeFromStr,
{
    fn calculate_sell_price(
        &self,
        input_reserve: Balance,
        output_reserve: Balance,
        sell_amount: Balance,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<RpcResult<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        ));

        let runtime_api_result = api.calculate_sell_price(&at, input_reserve, output_reserve, sell_amount);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Unable to serve the request".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn calculate_buy_price(
        &self,
        input_reserve: Balance,
        output_reserve: Balance,
        buy_amount: Balance,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<RpcResult<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        ));

        let runtime_api_result = api.calculate_buy_price(&at, input_reserve, output_reserve, buy_amount);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Unable to serve the request".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}