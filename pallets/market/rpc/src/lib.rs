use codec::Codec;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::ErrorObject,
};
pub use pallet_market::MarketRuntimeApi;
use pallet_market::{RpcAssetMetadata, RpcPoolInfo};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::U256;
use sp_rpc::number::NumberOrHex;
use sp_runtime::traits::{Block as BlockT, MaybeDisplay, MaybeFromStr};
use sp_std::convert::{TryFrom, TryInto};
use std::sync::Arc;

#[rpc(client, server)]
pub trait MarketApi<BlockHash, Balance, TokenId> {
	#[method(name = "market_calculate_sell_price")]
	fn calculate_sell_price(
		&self,
		pool_id: TokenId,
		sell_asset_id: TokenId,
		sell_amount: NumberOrHex,
		at: Option<BlockHash>,
	) -> RpcResult<NumberOrHex>;

	#[method(name = "market_calculate_sell_price_with_impact")]
	fn calculate_sell_price_with_impact(
		&self,
		pool_id: TokenId,
		sell_asset_id: TokenId,
		sell_amount: NumberOrHex,
		at: Option<BlockHash>,
	) -> RpcResult<(NumberOrHex, NumberOrHex)>;

	#[method(name = "market_calculate_buy_price")]
	fn calculate_buy_price(
		&self,
		pool_id: TokenId,
		buy_asset_id: TokenId,
		buy_amount: NumberOrHex,
		at: Option<BlockHash>,
	) -> RpcResult<NumberOrHex>;

	#[method(name = "market_calculate_buy_price_with_impact")]
	fn calculate_buy_price_with_impact(
		&self,
		pool_id: TokenId,
		buy_asset_id: TokenId,
		buy_amount: NumberOrHex,
		at: Option<BlockHash>,
	) -> RpcResult<(NumberOrHex, NumberOrHex)>;

	#[method(name = "market_calculate_expected_amount_for_minting")]
	fn calculate_expected_amount_for_minting(
		&self,
		pool_id: TokenId,
		asset_id: TokenId,
		amount: NumberOrHex,
		at: Option<BlockHash>,
	) -> RpcResult<NumberOrHex>;

	#[method(name = "market_calculate_expected_lp_minted")]
	fn calculate_expected_lp_minted(
		&self,
		pool_id: TokenId,
		amounts: (NumberOrHex, NumberOrHex),
		at: Option<BlockHash>,
	) -> RpcResult<NumberOrHex>;

	#[method(name = "market_get_burn_amount")]
	fn get_burn_amount(
		&self,
		pool_id: TokenId,
		liquidity_asset_amount: NumberOrHex,
		at: Option<BlockHash>,
	) -> RpcResult<(NumberOrHex, NumberOrHex)>;

	#[method(name = "market_get_pools_for_trading")]
	fn get_pools_for_trading(&self, at: Option<BlockHash>) -> RpcResult<Vec<TokenId>>;

	#[method(name = "market_get_tradeable_tokens")]
	fn get_tradeable_tokens(
		&self,
		at: Option<BlockHash>,
	) -> RpcResult<sp_std::vec::Vec<RpcAssetMetadata<TokenId>>>;

	#[method(name = "market_get_pools")]
	fn get_pools(
		&self,
		pool_id: Option<TokenId>,
		at: Option<BlockHash>,
	) -> RpcResult<sp_std::vec::Vec<RpcPoolInfo<TokenId, NumberOrHex>>>;
}

pub struct Market<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, P> Market<C, P> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

trait TryIntoBalance<Balance> {
	fn try_into_balance(self) -> RpcResult<Balance>;
}

impl<T: TryFrom<U256>> TryIntoBalance<T> for NumberOrHex {
	fn try_into_balance(self) -> RpcResult<T> {
		self.into_u256().try_into().or(Err(ErrorObject::owned(
			1,
			"Unable to serve the request",
			Some(String::from("input parameter doesnt fit into u128")),
		)))
	}
}

#[async_trait]
impl<C, Block, Balance, TokenId> MarketApiServer<<Block as BlockT>::Hash, Balance, TokenId>
	for Market<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: MarketRuntimeApi<Block, Balance, TokenId>,
	Balance: Codec + MaybeDisplay + MaybeFromStr + TryFrom<U256> + Into<NumberOrHex> + Default,
	TokenId: Codec + MaybeDisplay + MaybeFromStr,
{
	fn calculate_sell_price(
		&self,
		pool_id: TokenId,
		sell_asset_id: TokenId,
		sell_amount: NumberOrHex,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.calculate_sell_price(at, pool_id, sell_asset_id, sell_amount.try_into_balance()?)
			.map(|val| val.unwrap_or_default().into())
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}

	fn calculate_sell_price_with_impact(
		&self,
		pool_id: TokenId,
		sell_asset_id: TokenId,
		sell_amount: NumberOrHex,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(NumberOrHex, NumberOrHex)> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.calculate_sell_price_with_impact(
			at,
			pool_id,
			sell_asset_id,
			sell_amount.try_into_balance()?,
		)
		.map(|val| val.unwrap_or_default())
		.map(|val| (val.0.into(), val.1.into()))
		.map_err(|e| ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e))))
	}

	fn calculate_buy_price(
		&self,
		pool_id: TokenId,
		buy_asset_id: TokenId,
		buy_amount: NumberOrHex,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.calculate_buy_price(at, pool_id, buy_asset_id, buy_amount.try_into_balance()?)
			.map(|val| val.unwrap_or_default().into())
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}

	fn calculate_buy_price_with_impact(
		&self,
		pool_id: TokenId,
		buy_asset_id: TokenId,
		buy_amount: NumberOrHex,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(NumberOrHex, NumberOrHex)> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.calculate_buy_price_with_impact(
			at,
			pool_id,
			buy_asset_id,
			buy_amount.try_into_balance()?,
		)
		.map(|val| val.unwrap_or_default())
		.map(|val| (val.0.into(), val.1.into()))
		.map_err(|e| ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e))))
	}

	fn get_burn_amount(
		&self,
		pool_id: TokenId,
		liquidity_asset_amount: NumberOrHex,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(NumberOrHex, NumberOrHex)> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.get_burn_amount(at, pool_id, liquidity_asset_amount.try_into_balance()?)
			.map(|val| val.unwrap_or_default())
			.map(|(val1, val2)| (val1.into(), val2.into()))
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}

	fn get_pools_for_trading(
		&self,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<TokenId>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.get_pools_for_trading(at).map_err(|e| {
			ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
		})
	}

	fn get_tradeable_tokens(
		&self,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<RpcAssetMetadata<TokenId>>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.get_tradeable_tokens(at).map_err(|e| {
			ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
		})
	}

	fn calculate_expected_amount_for_minting(
		&self,
		pool_id: TokenId,
		asset_id: TokenId,
		amount: NumberOrHex,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.calculate_expected_amount_for_minting(at, pool_id, asset_id, amount.try_into_balance()?)
			.map(|val| val.unwrap_or_default().into())
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}

	fn calculate_expected_lp_minted(
		&self,
		pool_id: TokenId,
		amounts: (NumberOrHex, NumberOrHex),
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		let amount_0 = amounts.0.try_into_balance()?;
		let amount_1 = amounts.1.try_into_balance()?;

		api.calculate_expected_lp_minted(at, pool_id, (amount_0, amount_1))
			.map(|val| val.unwrap_or_default().into())
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}

	fn get_pools(
		&self,
		pool_id: Option<TokenId>,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<RpcPoolInfo<TokenId, NumberOrHex>>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.get_pools(at, pool_id)
			.map(|infos| {
				{
					infos.into_iter().map(|info| RpcPoolInfo {
						pool_id: info.pool_id,
						kind: info.kind,
						lp_token_id: info.lp_token_id,
						assets: info.assets,
						reserves: info.reserves.into_iter().map(|r| r.into()).collect(),
					})
				}
				.collect()
			})
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}
}
