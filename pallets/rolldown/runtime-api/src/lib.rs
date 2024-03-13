// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::Decode;
use sp_std::vec::Vec;
sp_api::decl_runtime_apis! {
	pub trait RolldownRuntimeApi<L1Update> where
	L1Update: Decode {
		fn get_pending_updates_hash() -> sp_core::H256;
		fn get_pending_updates() -> Vec<u8>;
		fn update_eth_raw(payload: Vec<u8>) -> L1Update;
	}
}
