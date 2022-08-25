#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use mangata_primitives::{Balance, TokenId};
use sp_runtime::traits::MaybeDisplay;
use sp_std::fmt::Debug;

pub trait PoolCreateApi {
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;

	fn pool_exists(first: TokenId, second: TokenId) -> bool;

	fn pool_create(
		account: Self::AccountId,
		first: TokenId,
		first_amount: Balance,
		second: TokenId,
		second_amount: Balance,
	) -> Option<(TokenId, Balance)>;
}

pub trait RewardsApi {
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;

	fn is_pool_promoted(first: TokenId, second: TokenId) -> bool;

	fn activate_liquidity_tokens(
		user: Self::AccountId,
		liquidity_asset_id: TokenId,
		amount: Balance,
	) -> bool;
}
