// Copyright (C) 2021 Mangata team
#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Codec, Decode, Encode};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait ProofOfStakeApi<Balance, TokenId, AccountId> where
		Balance: Codec + MaybeDisplay + MaybeFromStr,
		TokenId: Codec + MaybeDisplay + MaybeFromStr,
		AccountId: Codec + MaybeDisplay + MaybeFromStr,{

		fn calculate_native_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance;

		fn calculate_3rdparty_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
			reward_asset_id: TokenId,
		) -> Balance;

		fn calculate_3rdparty_rewards_amount_all(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Vec<(TokenId, Balance)>;
	}
}
