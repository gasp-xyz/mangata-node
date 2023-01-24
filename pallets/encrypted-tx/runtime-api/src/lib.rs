// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

sp_api::decl_runtime_apis! {
	pub trait VedRuntimeApi {
		fn decrypt_txs(
			public: [u8; 32],
			private: [u8; 64],
		) -> Option<sp_std::vec::Vec<u8>>;
	}
}
