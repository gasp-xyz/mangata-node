// Copyright (C) 2021 Mangata team

use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};

use array_bytes::hex2bytes;
use codec::{Decode, Encode};
use rolldown_runtime_api::RolldownRuntimeApi;
pub use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[rpc(client, server)]
pub trait RolldownApi<BlockHash, L1Update, L1> {
	/// Calculates amount of available native rewards
	///
	/// * `account` - user account address
	/// * `liquidity_token` - liquidity token id
	/// * `at` - optional block hash
	#[method(name = "rolldown_pending_updates_hash")]
	fn pending_updates_hash(&self, at: Option<BlockHash>) -> RpcResult<sp_core::H256>;

	#[method(name = "rolldown_pending_updates")]
	fn pending_updates(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;

	#[method(name = "rolldown_get_native_l1_update")]
	fn get_native_l1_update(
		&self,
		hex_payload: String,
		at: Option<BlockHash>,
	) -> RpcResult<Option<L1Update>>;
	#[method(name = "rolldown_verify_pending_requests")]
	fn verify_pending_requests(
		&self,
		hash: H256,
		request_id: u128,
		at: Option<BlockHash>,
	) -> RpcResult<bool>;
}

pub struct Rolldown<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, P> Rolldown<C, P> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block, L1Update, L1> RolldownApiServer<<Block as BlockT>::Hash, L1Update, L1>
	for Rolldown<C, Block>
where
	Block: BlockT,
	L1Update: Decode,
	L1: Encode,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: RolldownRuntimeApi<Block, L1Update, L1>,
{
	fn pending_updates_hash(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<sp_core::H256> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(self.client.info().best_hash);
		api.get_pending_updates_hash(at).map_err(|e| {
			JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
				1,
				"Unable to serve the request",
				Some(format!("{:?}", e)),
			)))
		})
	}

	fn pending_updates(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(self.client.info().best_hash);
		api.get_pending_updates(at).map_err(|e| {
			JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
				1,
				"Unable to serve the request",
				Some(format!("{:?}", e)),
			)))
		})
	}
	fn get_native_l1_update(
		&self,
		hex_payload: String,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<L1Update>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(self.client.info().best_hash);

		let payload = hex2bytes(hex_payload).map_err(|e| {
			JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
				0,
				"Unable to serve the request",
				Some(format!("{:?}", e)),
			)))
		})?;

		api.get_native_l1_update(at, payload).map_err(|e| {
			JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
				1,
				"Unable to serve the request",
				Some(format!("{:?}", e)),
			)))
		})
	}

	fn verify_pending_requests(
		&self,
		hash: H256,
		request_id: u128,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<bool> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(self.client.info().best_hash);
		api.verify_pending_requests(at, hash, request_id)
			.map_err(|e| {
				JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
					1,
					"Unable to serve the request",
					Some(format!("{:?}", e)),
				)))
			})
			.and_then(|e| match e {
				Some(result) => Ok(result),
				None => Err(JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
					1,
					"Unable to serve the request",
					Some("Request does not exist".to_string()),
				)))),
			})
	}
}
