// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]

use codec::alloc::string::String;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait MetamaskSignatureRuntimeApi{
		fn get_eip712_sign_data( call: Vec<u8>) -> String;
	}
}
