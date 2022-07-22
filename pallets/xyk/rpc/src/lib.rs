// Copyright (C) 2021 Mangata team

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::U256;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};
use sp_std::convert::{TryFrom, TryInto};
use std::sync::Arc;
pub use xyk_runtime_api::XykApi as XykRuntimeApi;
use xyk_runtime_api::{RpcAmountsResult, RpcResult};

#[rpc]
pub trait XykApi<BlockHash, Balance, TokenId, AccountId, ResponseTypePrice, ResponseTypeAmounts> {
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

	#[rpc(name = "xyk_calculate_rewards_amount")]
	fn calculate_rewards_amount(
		&self,
		user: AccountId,
		liquidity_asset_id: TokenId,
		at: Option<BlockHash>,
	) -> Result<ResponseTypePrice>;

	#[rpc(name = "xyk_calculate_rewards_amount_v2")]
	fn calculate_rewards_amount_v2(
		&self,
		user: AccountId,
		liquidity_asset_id: TokenId,
		at: Option<BlockHash>,
	) -> Result<ResponseTypePrice>;
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

trait TryIntoBalance<Balance> {
	fn try_into_balance(self) -> Result<Balance>;
}

impl<T: TryFrom<U256>> TryIntoBalance<T> for NumberOrHex {
	fn try_into_balance(self) -> Result<T> {
		self.into_u256().try_into().or(Err(RpcError {
			code: ErrorCode::ServerError(1),
			message: "Unable to serve the request".into(),
			data: Some(String::from("input parameter doesnt fit into u128").into()),
		}))
	}
}

impl<C, Block, Balance, TokenId, AccountId>
	XykApi<
		<Block as BlockT>::Hash,
		NumberOrHex,
		TokenId,
		AccountId,
		RpcResult<Balance>,
		RpcAmountsResult<Balance>,
	> for Xyk<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: XykRuntimeApi<Block, Balance, TokenId, AccountId>,
	Balance: Codec + MaybeDisplay + MaybeFromStr + TryFrom<U256>,
	TokenId: Codec + MaybeDisplay + MaybeFromStr,
	AccountId: Codec + MaybeDisplay + MaybeFromStr,
{
	fn calculate_sell_price(
		&self,
		input_reserve: NumberOrHex,
		output_reserve: NumberOrHex,
		sell_amount: NumberOrHex,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<RpcResult<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let runtime_api_result = api.calculate_sell_price(
			&at,
			input_reserve.try_into_balance()?,
			output_reserve.try_into_balance()?,
			sell_amount.try_into_balance()?,
		);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(1),
			message: "Unable to serve the request".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calculate_buy_price(
		&self,
		input_reserve: NumberOrHex,
		output_reserve: NumberOrHex,
		buy_amount: NumberOrHex,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<RpcResult<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let runtime_api_result = api.calculate_buy_price(
			&at,
			input_reserve.try_into_balance()?,
			output_reserve.try_into_balance()?,
			buy_amount.try_into_balance()?,
		);
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
		sell_amount: NumberOrHex,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<RpcResult<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let runtime_api_result = api.calculate_sell_price_id(
			&at,
			sold_token_id,
			bought_token_id,
			sell_amount.try_into_balance()?,
		);
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
		buy_amount: NumberOrHex,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<RpcResult<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let runtime_api_result = api.calculate_buy_price_id(
			&at,
			sold_token_id,
			bought_token_id,
			buy_amount.try_into_balance()?,
		);
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
		liquidity_asset_amount: NumberOrHex,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<RpcAmountsResult<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let runtime_api_result = api.get_burn_amount(
			&at,
			first_asset_id,
			second_asset_id,
			liquidity_asset_amount.try_into_balance()?,
		);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(1),
			message: "Unable to serve the request".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calculate_rewards_amount(
		&self,
		user: AccountId,
		liquidity_asset_id: TokenId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<RpcResult<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let runtime_api_result = api.calculate_rewards_amount(&at, user, liquidity_asset_id);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(1),
			message: "Unable to serve the request".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calculate_rewards_amount_v2(
		&self,
		user: AccountId,
		liquidity_asset_id: TokenId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<RpcResult<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::<Block>::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let runtime_api_result = api.calculate_rewards_amount_v2(&at, user, liquidity_asset_id);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(1),
			message: "Unable to serve the request".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
