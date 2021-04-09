// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
use codec::{Encode, Codec, Decode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
// Workaround for substrate/serde issue
#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "std", serde(bound(
    serialize = "Balance: Serialize",
    deserialize = "Balance: Deserialize<'de>",
)))]
pub struct RpcResult<Balance> {
    pub price: Balance,
}
sp_api::decl_runtime_apis! {
    pub trait XykApi<Balance, AssetId> where
        Balance: Codec + MaybeDisplay + MaybeFromStr,
        AssetId: Codec + MaybeDisplay + MaybeFromStr,{
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
        fn get_burn_amount(
            first_asset_id: AssetId,
            second_asset_id: AssetId,
            liquidity_asset_amount: Balance,
        ) -> RpcResult<(Balance,Balance)>;
    }
}