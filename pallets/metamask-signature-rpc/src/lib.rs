// Copyright (C) 2021 Mangata team

use codec::Codec;
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use metamask_signature_runtime_api::MetamaskSignatureRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[rpc(client, server)]
pub trait MetamaskSignatureApi<BlockHash> {
	/// Returns eip712 compatible SignedData V4 struct
	///
	#[method(name = "metamask_get_eip712_sign_data")]
	fn get_eip712_sign_data(
		&self,
		encoded_call: Vec<u8>,
		at: Option<BlockHash>,
	) -> RpcResult<String>;
}

pub struct MetamaskSignature<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, P> MetamaskSignature<C, P> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block> MetamaskSignatureApiServer<<Block as BlockT>::Hash> for MetamaskSignature<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: MetamaskSignatureRuntimeApi<Block>,
{
	fn get_eip712_sign_data(
		&self,
		call: Vec<u8>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or(self.client.info().best_hash);

		api.get_eip712_sign_data(at, call).map_err(|e| {
			JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
				0,
				"Unable to serve the request",
				Some(format!("{:?}", e)),
			)))
		})
	}
}
