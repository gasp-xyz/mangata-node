// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
use codec::{Codec, Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use sp_std::vec::Vec;
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct RpcAssetMetadata<TokenId> {
	pub token_id: TokenId,
	pub decimals: u32,
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,

sp_api::decl_runtime_apis! {
	pub trait XykApi<Balance, TokenId, AccountId> where
		Balance: Codec + MaybeDisplay + MaybeFromStr,
		TokenId: Codec + MaybeDisplay + MaybeFromStr,
		AccountId: Codec + MaybeDisplay + MaybeFromStr,{
		fn calculate_sell_price(
			input_reserve: Balance,
			output_reserve: Balance,
			sell_amount: Balance
		) -> Balance;
		fn calculate_buy_price(
			input_reserve: Balance,
			output_reserve: Balance,
			buy_amount: Balance
		) -> Balance;
		fn calculate_sell_price_id(
			sold_token_id: TokenId,
			bought_token_id: TokenId,
			sell_amount: Balance
		) -> Balance;
		fn calculate_buy_price_id(
			sold_token_id: TokenId,
			bought_token_id: TokenId,
			buy_amount: Balance
		) -> Balance;
		fn get_burn_amount(
			first_asset_id: TokenId,
			second_asset_id: TokenId,
			liquidity_asset_amount: Balance,
		) -> (Balance,Balance);
		fn get_max_instant_burn_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance;
		fn get_max_instant_unreserve_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance;
		fn calculate_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance;
		fn calculate_balanced_sell_amount(
			total_amount: Balance,
			reserve_amount: Balance,
		) -> Balance;

		fn is_buy_asset_lock_free(
			path: sp_std::vec::Vec<TokenId>,
			input_amount: Balance,
		) -> Option<bool>;

		fn is_sell_asset_lock_free(
			path: sp_std::vec::Vec<TokenId>,
			input_amount: Balance,
		) -> Option<bool>;

		fn get_tradeable_tokens() -> Vec<RpcAssetMetadata<TokenId>>;
	}
}
