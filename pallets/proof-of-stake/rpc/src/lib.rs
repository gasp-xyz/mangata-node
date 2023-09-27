// Copyright (C) 2021 Mangata team

use codec::Codec;
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
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
pub use proof_of_stake_runtime_api::XykApi as XykRuntimeApi;
use proof_of_stake_runtime_api::{RpcAmountsResult, RpcAssetMetadata, XYKRpcResult};

#[rpc(client, server)]
pub trait ProofOfStakeApi<
	BlockHash,
	Balance,
	TokenId,
	AccountId
>
{
	#[method(name = "foo")]
	fn foo(
		&self,
		account: AccountId,
		liquidity_token: TokenId,
		reward_token: TokenId,
		at: Option<BlockHash>,
	) -> RpcResult<bool>;
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
	ProofOfStakeApiServer<
		<Block as BlockT>::Hash,
		NumberOrHex,
		TokenId,
		AccountId
	> for ProofOfStake<C, Block>
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
	fn foo(
		&self,
		account: AccountId,
		liquidity_token: TokenId,
		reward_token: TokenId,
		at: Option<BlockHash>,
	) -> RpcResult<XYKRpcResult<Balance>> {
	}

}
