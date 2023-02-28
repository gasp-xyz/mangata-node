#![cfg_attr(not(feature = "std"), no_std)]
use codec::FullCodec;
use frame_support::pallet_prelude::*;
use mangata_types::{Balance, TokenId};
use mp_multipurpose_liquidity::{ActivateKind, BondKind};
use sp_runtime::{traits::{AtLeast32BitUnsigned, MaybeDisplay}, Permill};
use sp_std::{fmt::Debug, vec::Vec};
use sp_core::U256;

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
		err_upon_bad_slippage: bool,
	) -> Result<Self::Balance, DispatchError>;

	fn multiswap_sell_asset(
		sender: AccountId,
		swap_token_list: Vec<Self::CurrencyId>,
		sold_asset_amount: Self::Balance,
		min_amount_out: Self::Balance,
		err_upon_bad_slippage: bool,
		err_upon_non_slippage_fail: bool,
	) -> Result<Self::Balance, DispatchError>;

	fn do_multiswap_sell_asset(
		sender: AccountId,
		swap_token_list: Vec<Self::CurrencyId>,
		sold_asset_amount: Self::Balance,
		min_amount_out: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;
	fn do_multiswap_buy_asset(
		sender: AccountId,
		swap_token_list: Vec<Self::CurrencyId>,
		bought_asset_amount: Self::Balance,
		max_amount_in: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;

	fn buy_asset(
		sender: AccountId,
		sold_asset_id: Self::CurrencyId,
		bought_asset_id: Self::CurrencyId,
		bought_asset_amount: Self::Balance,
		max_amount_in: Self::Balance,
		err_upon_bad_slippage: bool,
	) -> Result<Self::Balance, DispatchError>;

	fn multiswap_buy_asset(
		sender: AccountId,
		swap_token_list: Vec<Self::CurrencyId>,
		bought_asset_amount: Self::Balance,
		max_amount_in: Self::Balance,
		err_upon_bad_slippage: bool,
		err_upon_non_slippage_fail: bool,
	) -> Result<Self::Balance, DispatchError>;

	fn mint_liquidity(
		sender: AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		first_asset_amount: Self::Balance,
		expected_second_asset_amount: Self::Balance,
		activate_minted_liquidity: bool,
	) -> Result<(Self::CurrencyId, Self::Balance), DispatchError>;

	fn provide_liquidity_with_conversion(
		sender: AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		provided_asset_id: Self::CurrencyId,
		provided_asset_amount: Self::Balance,
		activate_minted_liquidity: bool,
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

	fn do_compound_rewards(
		sender: AccountId,
		liquidity_asset_id: Self::CurrencyId,
		amount_permille: Permill,
	) -> DispatchResult;


	fn is_liquidity_token(liquidity_asset_id: TokenId) -> bool;
}

pub trait ProofOfStakeRewardsApi<AccountId> {
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

	fn claim_rewards_v2(
		sender: AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn claim_rewards_all_v2(
		sender: AccountId,
		liquidity_token_id: Self::CurrencyId,
	) -> Result<Self::Balance, DispatchError>;

	fn update_pool_promotion(
		liquidity_token_id: TokenId,
		liquidity_mining_issuance_weight: Option<u8>,
	) -> DispatchResult;

	fn activate_liquidity_v2(
		sender: AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult;

	fn deactivate_liquidity_v2(
		sender: AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn current_rewards_time() -> Option<u32>;

	fn calculate_rewards_amount_v2(
		user: AccountId,
		liquidity_asset_id: Self::CurrencyId,
	) -> Result<Balance, DispatchError>;

	fn set_liquidity_minting_checkpoint_v2(
		user: AccountId,
		liquidity_asset_id: Self::CurrencyId,
		liquidity_assets_added: Self::Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult;

	fn set_liquidity_burning_checkpoint_v2(
		user: AccountId,
		liquidity_asset_id: Self::CurrencyId,
		liquidity_assets_burned: Self::Balance,
	) -> DispatchResult;
}

pub trait XykRewardsApi {
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;

	fn get_pool_rewards_v2(liquidity_asset_id: TokenId) -> Option<U256>;

	fn claim_rewards_all_v2(
		user: Self::AccountId,
		liquidity_asset_id: TokenId,
	) -> Result<Balance, DispatchError>;
	
	fn set_liquidity_minting_checkpoint_v2(
		user: Self::AccountId,
		liquidity_asset_id: TokenId,
		liquidity_assets_added: Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult;

	fn set_liquidity_burning_checkpoint_v2(
		user: Self::AccountId,
		liquidity_asset_id: TokenId,
		liquidity_assets_burned: Balance,
	) -> DispatchResult;
}

pub trait PreValidateSwaps {
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;

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

	fn pre_validate_sell_asset(
		sender: &Self::AccountId,
		sold_asset_id: Self::CurrencyId,
		bought_asset_id: Self::CurrencyId,
		sold_asset_amount: Self::Balance,
		min_amount_out: Self::Balance,
	) -> Result<
		(Self::Balance, Self::Balance, Self::Balance, Self::Balance, Self::Balance, Self::Balance),
		DispatchError,
	>;

	fn pre_validate_multiswap_sell_asset(
		sender: &Self::AccountId,
		swap_token_list: Vec<Self::CurrencyId>,
		sold_asset_amount: Self::Balance,
		min_amount_out: Self::Balance,
	) -> Result<
		(
			Self::Balance,
			Self::Balance,
			Self::Balance,
			Self::Balance,
			Self::Balance,
			Self::CurrencyId,
			Self::CurrencyId,
		),
		DispatchError,
	>;

	fn pre_validate_buy_asset(
		sender: &Self::AccountId,
		sold_asset_id: Self::CurrencyId,
		bought_asset_id: Self::CurrencyId,
		bought_asset_amount: Self::Balance,
		max_amount_in: Self::Balance,
	) -> Result<
		(Self::Balance, Self::Balance, Self::Balance, Self::Balance, Self::Balance, Self::Balance),
		DispatchError,
	>;

	fn pre_validate_multiswap_buy_asset(
		sender: &Self::AccountId,
		swap_token_list: Vec<Self::CurrencyId>,
		final_bought_asset_amount: Self::Balance,
		max_amount_in: Self::Balance,
	) -> Result<
		(
			Self::Balance,
			Self::Balance,
			Self::Balance,
			Self::Balance,
			Self::Balance,
			Self::CurrencyId,
			Self::CurrencyId,
		),
		DispatchError,
	>;
}

pub trait FeeLockTriggerTrait<AccountId> {
	fn process_fee_lock(who: &AccountId) -> DispatchResult;

	fn can_unlock_fee(who: &AccountId) -> DispatchResult;

	fn is_whitelisted(token_id: TokenId) -> bool;

	fn get_swap_valuation_for_token(
		valuating_token_id: TokenId,
		valuating_token_amount: Balance,
	) -> Option<Balance>;

	fn unlock_fee(who: &AccountId) -> DispatchResult;
}
