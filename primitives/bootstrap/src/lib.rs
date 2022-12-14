#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use mangata_types::{Balance, TokenId};
use sp_runtime::{traits::MaybeDisplay};
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

	/// checks whether given pool is promoted and tokens
	/// can be activated
	fn can_activate(liquidity_asset_id: TokenId) -> bool;

	/// Activates liquidity tokens for rewards minting
	fn activate_liquidity_tokens(
		user: &Self::AccountId,
		liquidity_asset_id: TokenId,
		amount: Balance,
	) -> DispatchResult;

	fn update_pool_promotion(
		liquidity_token_id: TokenId,
		liquidity_mining_issuance_weight: Option<u8>,
	);
}

pub trait AssetRegistryApi {
	fn enable_pool_creation(assets: (TokenId, TokenId)) -> bool;
}
