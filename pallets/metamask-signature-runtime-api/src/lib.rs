// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]

use codec::alloc::string::{String, ToString};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait MetamaskSignatureRuntimeApi{
		fn get_eip712_sign_data( call: Vec<u8>) -> String;
	}
}

pub fn eip712_payload(call: String) -> String {
	let input = r#"{
					"types": {
						"EIP712Domain": [
						{
							"name": "name",
							"type": "string"
						},
						{
							"name": "version",
							"type": "string"
						},
						{
							"name": "chainId",
							"type": "uint256"
						},
						{
							"name": "verifyingContract",
							"type": "address"
						}
						],
						"Message": [
						{
							"name": "method",
							"type": "string"
						},
						{
							"name": "params",
							"type": "string"
						},
						{
							"name": "tx",
							"type": "string"
						}
						]
					},
					"primaryType": "Message",
					"domain": {
						"name": "Mangata",
						"version": "1",
						"chainId": "5",
						"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
					},
					"message": {
						"call": "",
						"tx": ""
					}
				}"#;
	if let Ok(ref mut v) = serde_json::from_str::<serde_json::Value>(input) {
		v["message"]["call"] = serde_json::Value::String(call);
		v.to_string()
	} else {
		Default::default()
	}
}
