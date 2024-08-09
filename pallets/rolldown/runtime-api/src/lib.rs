// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{Decode, Encode, H256};
use sp_std::vec::Vec;
sp_api::decl_runtime_apis! {
	pub trait RolldownRuntimeApi<L1Update, Chain> where
		L1Update: Decode,
		Chain: Encode
	{
		fn get_l2_request_hash(chain: Chain) -> H256;
		fn get_l2_request(chain: Chain) -> Vec<u8>;
		fn get_abi_encoded_l2_request(chain: Chain, requestId: u128) -> Vec<u8>;
		fn get_native_sequencer_update(hex_payload: Vec<u8>) -> Option<L1Update>;
		fn verify_sequencer_update(chain: Chain, hash: H256, request_id: u128) -> Option<bool>;
		fn get_last_processed_request_on_l2(chain: Chain) -> Option<u128>;
		fn get_number_of_pending_requests(chain: Chain) -> Option<u128>;
		fn get_total_number_of_deposits() -> u32;
		fn get_total_number_of_withdrawals() -> u32;
		fn get_merkle_root(chain: Chain, range : (u128, u128)) -> H256;
		fn get_merkle_proof_for_tx(chain: Chain, range : (u128, u128), tx_id: u128) -> Vec<H256>;
		fn verify_merkle_proof_for_tx(chain: Chain, range: (u128, u128), tx_id: u128, root: H256, proof: Vec<H256>) -> bool;
	}
}
