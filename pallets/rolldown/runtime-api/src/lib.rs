// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]
sp_api::decl_runtime_apis! {
	pub trait RolldownRuntimeApi{
		fn get_pending_requests_hash() -> sp_core::H256;
	}
}
