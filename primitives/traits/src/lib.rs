#![cfg_attr(not(feature = "std"), no_std)]
use codec::FullCodec;
use frame_support::pallet_prelude::*;
use mangata_primitives::{Balance, TokenId};
use mp_multipurpose_liquidity::{ActivateKind, BondKind};
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeDisplay};
use sp_std::fmt::Debug;

pub trait StakingReservesProviderTrait {
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;

	fn can_bond(
		token_id: TokenId,
		account_id: &Self::AccountId,
		amount: Balance,
		use_balance_from: Option<BondKind>,
	) -> bool;

	fn bond(
		token_id: TokenId,
		account_id: &Self::AccountId,
		amount: Balance,
		use_balance_from: Option<BondKind>,
	) -> DispatchResult;

	fn unbond(token_id: TokenId, account_id: &Self::AccountId, amount: Balance) -> Balance;
}

pub trait ActivationReservesProviderTrait {
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;

	fn get_max_instant_unreserve_amount(token_id: TokenId, account_id: &Self::AccountId)
		-> Balance;

	fn can_activate(
		token_id: TokenId,
		account_id: &Self::AccountId,
		amount: Balance,
		use_balance_from: Option<ActivateKind>,
	) -> bool;

	fn activate(
		token_id: TokenId,
		account_id: &Self::AccountId,
		amount: Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult;

	fn deactivate(token_id: TokenId, account_id: &Self::AccountId, amount: Balance) -> Balance;
}

pub trait XykFunctionsTrait<AccountId> {
	type Balance: AtLeast32BitUnsigned
		+ FullCodec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default
		+ From<Balance>
		+ Into<Balance>;

	type CurrencyId: Parameter
		+ Member
		+ Copy
		+ MaybeSerializeDeserialize
		+ Ord
		+ Default
		+ AtLeast32BitUnsigned
		+ FullCodec
		+ From<TokenId>
		+ Into<TokenId>;

	fn create_pool(
		sender: AccountId,
		first_asset_id: Self::CurrencyId,
		first_asset_amount: Self::Balance,
		second_asset_id: Self::CurrencyId,
		second_asset_amount: Self::Balance,
	) -> DispatchResult;

	fn sell_asset(
		sender: AccountId,
		sold_asset_id: Self::CurrencyId,
		bought_asset_id: Self::CurrencyId,
		sold_asset_amount: Self::Balance,
		min_amount_out: Self::Balance,
	) -> DispatchResult;

	fn buy_asset(
		sender: AccountId,
		sold_asset_id: Self::CurrencyId,
		bought_asset_id: Self::CurrencyId,
		bought_asset_amount: Self::Balance,
		max_amount_in: Self::Balance,
	) -> DispatchResult;

	fn mint_liquidity(
		sender: AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		first_asset_amount: Self::Balance,
		expected_second_asset_amount: Self::Balance,
	) -> Result<(Self::CurrencyId, Self::Balance), DispatchError>;

	fn burn_liquidity(
		sender: AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		liquidity_asset_amount: Self::Balance,
	) -> DispatchResult;

	fn get_tokens_required_for_minting(
		liquidity_asset_id: Self::CurrencyId,
		liquidity_token_amount: Self::Balance,
	) -> Result<(Self::CurrencyId, Self::Balance, Self::CurrencyId, Self::Balance), DispatchError>;

	fn claim_rewards(
		sender: AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn promote_pool(liquidity_token_id: TokenId) -> DispatchResult;

	fn activate_liquidity(
		sender: AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult;

	fn deactivate_liquidity(
		sender: AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn is_liquidity_token(liquidity_asset_id: TokenId) -> bool;
}
