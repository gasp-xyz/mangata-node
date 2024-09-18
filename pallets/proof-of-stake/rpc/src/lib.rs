// Copyright (C) 2021 Mangata team

use codec::Codec;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::ErrorObject,
};
pub use proof_of_stake_runtime_api::ProofOfStakeApi as ProofOfStakeRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::traits::{Block as BlockT, MaybeDisplay, MaybeFromStr};
use std::sync::Arc;

#[rpc(client, server)]
pub trait ProofOfStakeApi<BlockHash, Balance, TokenId, AccountId> {
	/// Calculates amount of available native rewards
	///
	/// * `account` - user account address
	/// * `liquidity_token` - liquidity token id
	/// * `at` - optional block hash
	#[method(name = "pos_calculate_native_rewards_amount")]
	fn calculate_native_rewards_amount(
		&self,
		account: AccountId,
		liquidity_token: TokenId,
		at: Option<BlockHash>,
	) -> RpcResult<NumberOrHex>;

	/// Calculates amount of available 3rdparty rewards
	///
	/// * `account` - user account address
	/// * `liquidity_token` - liquidity token id
	/// * `reward_token` - particular token that given pool is rewarded with
	/// * `at` - optional block hash
	#[method(name = "pos_calculate_3rdparty_rewards_amount")]
	fn calculate_3rdparty_rewards_amount(
		&self,
		account: AccountId,
		liquidity_token: TokenId,
		rewards_token: TokenId,
		at: Option<BlockHash>,
	) -> RpcResult<NumberOrHex>;

	/// # Calculates amount of all available 3rdparty rewards for given account
	///
	/// * `account` - user account address
	/// * `at` - optional block hash
	#[method(name = "pos_calculate_3rdparty_rewards_all")]
	fn calculate_3rdparty_rewards_all(
		&self,
		account: AccountId,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<(TokenId, TokenId, NumberOrHex)>>;
}

pub struct ProofOfStake<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, P> ProofOfStake<C, P> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block, Balance, TokenId, AccountId>
	ProofOfStakeApiServer<<Block as BlockT>::Hash, Balance, TokenId, AccountId>
	for ProofOfStake<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: ProofOfStakeRuntimeApi<Block, Balance, TokenId, AccountId>,
	Balance: Codec + MaybeDisplay + MaybeFromStr + Into<NumberOrHex>,
	TokenId: Codec + MaybeDisplay + MaybeFromStr,
	AccountId: Codec + MaybeDisplay + MaybeFromStr,
{
	fn calculate_native_rewards_amount(
		&self,
		account: AccountId,
		liquidity_token: TokenId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(self.client.info().best_hash);

		api.calculate_native_rewards_amount(at, account, liquidity_token)
			.map(Into::<NumberOrHex>::into)
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}

	fn calculate_3rdparty_rewards_amount(
		&self,
		account: AccountId,
		liquidity_token: TokenId,
		reward_token: TokenId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(self.client.info().best_hash);

		api.calculate_3rdparty_rewards_amount(at, account, liquidity_token, reward_token)
			.map(Into::<NumberOrHex>::into)
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}

	fn calculate_3rdparty_rewards_all(
		&self,
		account: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<(TokenId, TokenId, NumberOrHex)>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(self.client.info().best_hash);

		api.calculate_3rdparty_rewards_all(at, account)
			.map(|vec| {
				vec.into_iter()
					.map(|(token1, token2, balance)| {
						(token1, token2, Into::<NumberOrHex>::into(balance))
					})
					.collect()
			})
			.map_err(|e| {
				ErrorObject::owned(1, "Unable to serve the request", Some(format!("{:?}", e)))
			})
	}
}
