// Copyright (C) 2021 Mangata team

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};
use std::sync::Arc;
pub use xyk_runtime_api::XykApi as XykRuntimeApi;
use xyk_runtime_api::{RpcAmountsResult, RpcResult};

#[rpc]
pub trait XykApi<BlockHash, Balance, TokenId, ResponseTypePrice, ResponseTypeAmounts> {
    #[rpc(name = "xyk_calculate_sell_price")]
    fn calculate_sell_price(
        &self,
        input_reserve: Balance,
        output_reserve: Balance,
        sell_amount: Balance,
        at: Option<BlockHash>,
    ) -> Result<ResponseTypePrice>;

    #[rpc(name = "xyk_calculate_buy_price")]
    fn calculate_buy_price(
        &self,
        input_reserve: Balance,
        output_reserve: Balance,
        buy_amount: Balance,
        at: Option<BlockHash>,
    ) -> Result<ResponseTypePrice>;

    #[rpc(name = "xyk_calculate_sell_price_id")]
    fn calculate_sell_price_id(
        &self,
        sold_token_id: TokenId,
        bought_token_id: TokenId,
        sell_amount: Balance,
        at: Option<BlockHash>,
    ) -> Result<ResponseTypePrice>;

    #[rpc(name = "xyk_calculate_buy_price_id")]
    fn calculate_buy_price_id(
        &self,
        sold_token_id: TokenId,
        bought_token_id: TokenId,
        buy_amount: Balance,
        at: Option<BlockHash>,
    ) -> Result<ResponseTypePrice>;

    #[rpc(name = "xyk_get_burn_amount")]
    fn get_burn_amount(
        &self,
        first_asset_id: TokenId,
        second_asset_id: TokenId,
        liquidity_asset_amount: Balance,
        at: Option<BlockHash>,
    ) -> Result<ResponseTypeAmounts>;
}

pub struct Xyk<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, P> Xyk<C, P> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, Balance, TokenId>
    XykApi<<Block as BlockT>::Hash, Balance, TokenId, RpcResult<Balance>, RpcAmountsResult<Balance>>
    for Xyk<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: XykRuntimeApi<Block, Balance, TokenId>,
    Balance: Codec + MaybeDisplay + MaybeFromStr,
    TokenId: Codec + MaybeDisplay + MaybeFromStr,
{
    fn calculate_sell_price(
        &self,
        input_reserve: Balance,
        output_reserve: Balance,
        sell_amount: Balance,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RpcResult<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result =
            api.calculate_sell_price(&at, input_reserve, output_reserve, sell_amount);
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
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RpcResult<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result =
            api.calculate_buy_price(&at, input_reserve, output_reserve, buy_amount);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Unable to serve the request".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn calculate_sell_price_id(
        &self,
        sold_token_id: TokenId,
        bought_token_id: TokenId,
        sell_amount: Balance,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RpcResult<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result =
            api.calculate_sell_price_id(&at, sold_token_id, bought_token_id, sell_amount);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Unable to serve the request".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn calculate_buy_price_id(
        &self,
        sold_token_id: TokenId,
        bought_token_id: TokenId,
        buy_amount: Balance,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RpcResult<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result =
            api.calculate_buy_price_id(&at, sold_token_id, bought_token_id, buy_amount);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Unable to serve the request".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_burn_amount(
        &self,
        first_asset_id: TokenId,
        second_asset_id: TokenId,
        liquidity_asset_amount: Balance,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RpcAmountsResult<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result =
            api.get_burn_amount(&at, first_asset_id, second_asset_id, liquidity_asset_amount);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Unable to serve the request".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
