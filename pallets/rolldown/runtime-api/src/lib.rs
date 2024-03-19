// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{Decode, H256};
use sp_std::vec::Vec;
sp_api::decl_runtime_apis! {
	pub trait RolldownRuntimeApi<L1Update> where
	L1Update: Decode {
		fn get_pending_updates_hash() -> H256;
		fn get_pending_updates() -> Vec<u8>;
		fn get_native_l1_update(payload: Vec<u8>) -> Option<L1Update>;
		fn verify_pending_requests(hash: H256, request_id: u128) -> Option<bool>;
	}
}
