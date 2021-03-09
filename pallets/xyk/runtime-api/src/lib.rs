// Copyright (C) 2021 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::{Encode, Codec, Decode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};

// Workaround for substrate/serde issue
#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct RpcResult<Balance> {
    #[cfg_attr(feature = "std", serde(bound(serialize = "Balance: std::fmt::Display")))]
    #[cfg_attr(feature = "std", serde(serialize_with = "serialize_as_string"))]
    #[cfg_attr(feature = "std", serde(bound(deserialize = "Balance: std::str::FromStr")))]
    #[cfg_attr(feature = "std", serde(deserialize_with = "deserialize_from_string"))]
    pub price: Balance,
}

#[cfg(feature = "std")]
fn serialize_as_string<S: Serializer, T: std::fmt::Display>(t: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&t.to_string())
}

#[cfg(feature = "std")]
fn deserialize_from_string<'de, D: Deserializer<'de>, T: std::str::FromStr>(deserializer: D) -> Result<T, D::Error> {
    let s = String::deserialize(deserializer)?;
    s.parse::<T>().map_err(|_| serde::de::Error::custom("Parse from string failed"))
}

sp_api::decl_runtime_apis! {
	pub trait XykApi<Balance> where
		Balance: Codec + MaybeDisplay + MaybeFromStr {
		fn calculate_sell_price(
		    input_reserve: Balance,
        	output_reserve: Balance,
        	sell_amount: Balance
        ) -> RpcResult<Balance>;

        fn calculate_buy_price(
		    input_reserve: Balance,
        	output_reserve: Balance,
        	buy_amount: Balance
        ) -> RpcResult<Balance>;
	}
}