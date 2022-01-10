//! # XYK pallet

//! Provides functions for token operations, swapping tokens, creating token pools, minting and burning liquidity and supporting public functions
//!
//! ### Token operation functions:
//! - create_pool
//! - mint_liquidity
//! - burn_liquidity
//! - sell_asset
//! - buy_asset
//!
//! ### Supporting public functions:
//! - calculate_sell_price
//! - calculate_buy_price
//! - calculate_sell_price_id
//! - calculate_buy_price_id
//! - get_liquidity_token
//! - get_burn_amount
//! - account_id
//! - settle_treasury_buy_and_burn
//!
//! # fn create_pool
//! -Sets the initial ratio/price of both assets to each other depending on amounts of each assets when creating pool.
//!
//! -Transfers assets from user to vault and makes appropriate entry to pools map, where are assets amounts kept.
//!
//! -Issues new liquidity asset in amount corresponding to amounts creating the pool, marks them as liquidity assets corresponding to this pool and transfers them to user.
//! first_token_amount
//! ### arguments
//! `origin` - sender of a fn, user creating the pool
//!
//! `first_token_id` - id of first token which will be directly inter-tradeable in a pair of first_token_id-second_token_id
//!
//! `first_token_amount` - amount of first token in which the pool will be initiated, which will set their initial ratio/price
//!
//! `second_token_id` - id of second token which will be directly inter-tradeable in a pair of first_token_id-second_token_id
//!
//! `second_token_amount` - amount of second token in which the pool will be initiated, which will set their initial ratio/price
//!
//! ### Example
//! ```ignore
//! create_pool(
//!    Origin::signed(1),
//!    0,
//!    1000,
//!    1,
//!    2000,
//! )
//! ```
//! Account_id 1 created pool with tokens 0 and 1, with amounts 1000, 2000. Initial ratio is 1:2. Liquidity token with new id created in an amount of 1500 and transfered to user 1.
//!
//! ### Errors
//! `ZeroAmount` - creating pool with 0 amount of first or second token
//!
//! `PoolAlreadyExists` - creating pool which already exists
//!
//! `NotEnoughTokens` - creating pool with amounts higher then user owns
//!
//! `SameToken` - creating pool with same token
//!
//! # fn sell_token
//! -Sells/exchanges set amount of sold token for corresponding amount by xyk formula of bought token
//! ### arguments
//! `origin` - sender of a fn, user creating the pool
//!
//! `sold_token_id` - token which will be sold
//!
//! `bought_token_id` - token which will be bought
//!
//! `sold_token_amount` - amount of token to be sold
//!
//! `min_amount_out` - minimal acceptable amount of bought token received after swap
//!
//! ### Example
//! ```ignore
//! sell_token (
//!    Origin::signed(1),
//!    0,
//!    1,
//!    1000,
//!    800,
//!)
//! ```
//! Account_id 1 sells/exchanges 1000 token 0 for corresponding amount of token 1, while requiring at least 800 token 1
//!
//! ### Errors
//! `ZeroAmount` - buying 0 tokens
//!
//! `NoSuchPool` - pool sold_token_id - bought_token_id does not exist
//!
//! `NotEnoughTokens` - selling more tokens then user owns
//!
//! `InsufficientOutputAmount` - bought tokens to receive amount is lower then required min_amount_out
//!
//! # fn buy_token
//! -Buys/exchanges set amount of bought token for corresponding amount by xyk formula of sold token
//! ### arguments
//! `origin` - sender of a fn, user creating the pool
//!
//! `sold_token_id` - token which will be sold
//!
//! `bought_token_id` - token which will be bought
//!
//! `bought_token_amount` - amount of token to be bought
//!
//! `max_amount_in` - maximal acceptable amount of sold token to pay for requested bought amount
//!
//! ### Example
//! ```ignore
//! buy_token (
//!    Origin::signed(1),
//!    0,
//!    1,
//!    1000,
//!    800,
//!)
//! ```
//! Account_id 1 buys/exchanges 1000 tokens 1 by paying corresponding amount by xyk formula of tokens 0
//!
//! ### Errors
//! `ZeroAmount` - selling 0 tokens
//!
//! `NoSuchPool` - pool sold_token_id - bought_token_id does not exist
//!
//! `NotEnoughTokens` - selling more tokens then user owns
//!
//! `InsufficientInputAmount` - sold tokens to pay is higher then maximum acceptable value of max_amount_in
//!
//! # fn mint_liquidity
//! -Adds liquidity to pool, providing both tokens in actual ratio
//! -First token amount is provided by user, second token amount is calculated by function, depending on actual ratio
//! -Mints and transfers corresponding amount of liquidity token to mintin user
//!
//! ### arguments
//! `origin` - sender of a fn, user creating the pool
//!
//! first_token_id - first token in pair
//!
//! second_token_id - second token in pair
//!
//! first_token_amount - amount of first_token_id, second token amount will be calculated
//!
//! ### Example
//! ```ignore
//! mint_liquidity (
//!    Origin::signed(1),
//!    0,
//!    1,
//!    1000,
//!)
//! ```
//! If pool token 0 - token 1 has tokens in amounts 9000:18000 (total liquidity tokens 27000)
//!
//! Account_id 1 added liquidity to pool token 0 - token 1, by providing 1000 token 0 and corresponding amount of token 1. In this case 2000, as the ratio in pool is 1:2.
//! Account_id 1 also receives corresponding liquidity tokens in corresponding amount. In this case he gets 10% of all corresponding liquidity tokens, as he is providing 10% of all provided liquidity in pool.
//! 3000 out of total 30000 liquidity tokens is now owned by Account_id 1
//!
//! ### Errors
//! `ZeroAmount` - minting with 0 tokens
//!
//! `NoSuchPool` - pool first_token_id - second_token_id does not exist
//!
//! `NotEnoughTokens` -  minting with more tokens then user owns, either first_token_id or second_token_id
//!     
//! # fn burn_liquidity
//! -Removes tokens from liquidity pool and transfers them to user, by burning user owned liquidity tokens
//! -Amount of tokens is determined by their ratio in pool and amount of liq tokens burned
//!
//! ### arguments
//! `origin` - sender of a fn, user creating the pool
//!
//! first_token_id - first token in pair
//!
//! second_token_id - second token in pair
//!
//! liquidity_token_amount - amount of liquidity token amount to burn
//!
//! ### Example
//! ```ignore
//! burn_liquidity (
//!    Origin::signed(1),
//!    0,
//!    1,
//!    3000,
//!)
//! ```
//! If pool token 0 - token 1 has tokens in amounts 10000:20000 (total liquidity tokens 30000)
//!
//! Account_id 1 is burning 3000 liquidity tokens of pool token 0 - token 1
//! As Account_id 1 is burning 10% of total liquidity tokens for this pool, user receives in this case 1000 token 0 and 2000 token 1
//!
//! ### Errors
//! `ZeroAmount` - burning 0 liquidity tokens
//!
//! `NoSuchPool` - pool first_token_id - second_token_id does not exist
//!
//! `NotEnoughTokens` -  burning more liquidity tokens than user owns
//!
//! # calculate_sell_price
//! - Supporting public function accessible through rpc call which calculates and returns bought_token_amount while providing sold_token_amount and respective reserves
//! # calculate_buy_price
//! - Supporting public function accessible through rpc call which calculates and returns sold_token_amount while providing bought_token_amount and respective reserves
//! # calculate_sell_price_id
//! - Same as calculate_sell_price, but providing token_id instead of reserves. Reserves are fetched by function.
//! # calculate_buy_price_id
//! - Same as calculate_buy_price, but providing token_id instead of reserves. Reserves are fetched by function.
//! # get_liquidity_token
//! - Supporting public function accessible through rpc call which returns liquidity_token_id while providing pair token ids
//! # get_burn_amount
//! - Supporting public function accessible through rpc call which returns amounts of tokens received by burning provided liquidity_token_amount in pool of provided token ids
//! # account_id
//! - Returns palled account_id
//! # settle_treasury_buy_and_burn
//! - Supporting function which takes tokens to alocate to treasury and tokens to be used to burn mangata
//! - First step is deciding whether we are using sold or bought token id, depending which is closer to mangata token
//! - In second step, if tokens are mangata, they are placed to treasury and removed from corresponding pool. If tokens are not mangata, but are available in mangata pool,
//!   they are swapped to mangata and placed to treasury and removed from corresponding pool. If token is not connected to mangata, token is temporarily placed to treasury and burn treasury.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    sp_runtime::ModuleId,
    weights::Pays,
    StorageMap,
};
use frame_system::ensure_signed;
use sp_core::U256;
// TODO documentation!
use codec::FullCodec;
use frame_support::sp_runtime::traits::AccountIdConversion;
use frame_support::traits::{ExistenceRequirement, Get, Vec, WithdrawReasons};
use frame_support::Parameter;
use mangata_primitives::{Balance, TokenId};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended};
use pallet_assets_info as assets_info;
use sp_arithmetic::helpers_128bit::multiply_by_rational;
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member};
use sp_runtime::traits::{CheckedDiv, Zero};
use sp_runtime::SaturatedConversion;
use sp_std::convert::TryFrom;
use sp_std::fmt::Debug;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub(crate) const LOG_TARGET: &'static str = "xyk";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		frame_support::debug::$level!(
			target: crate::LOG_TARGET,
			$patter $(, $values)*
		)
	};
}

type AccountIdOf<T> = <T as frame_system::Trait>::AccountId;
pub trait Trait: frame_system::Trait + pallet_assets_info::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: MultiTokenCurrencyExtended<Self::AccountId>;
    type NativeCurrencyId: Get<TokenId>;
    type TreasuryModuleId: Get<ModuleId>;
    type BnbTreasurySubAccDerive: Get<[u8; 4]>;
}

const PALLET_ID: ModuleId = ModuleId(*b"79b14c96");
// 1/100 %
const TREASURY_PERCENTAGE: u128 = 5;
const BUYANDBURN_PERCENTAGE: u128 = 5;
const FEE_PERCENTAGE: u128 = 30;
const POOL_FEE_PERCENTAGE: u128 = FEE_PERCENTAGE - TREASURY_PERCENTAGE - BUYANDBURN_PERCENTAGE;

// Keywords for asset_info
const LIQUIDITY_TOKEN_IDENTIFIER: &[u8] = b"LiquidityPoolToken";
const HEX_INDICATOR: &[u8] = b"0x";
const TOKEN_SYMBOL: &[u8] = b"TKN";
const TOKEN_SYMBOL_SEPARATOR: &[u8] = b"-";
const LIQUIDITY_TOKEN_DESCRIPTION: &[u8] = b"Generated Info for Liquidity Pool Token";
const DEFAULT_DECIMALS: u32 = 18u32;

decl_error! {
    /// Errors
    pub enum Error for Module<T: Trait> {
        VaultAlreadySet,
        PoolAlreadyExists,
        NotEnoughAssets,
        NoSuchPool,
        NoSuchLiquidityAsset,
        NotEnoughReserve,
        ZeroAmount,
        InsufficientInputAmount,
        InsufficientOutputAmount,
        SameAsset,
        AssetAlreadyExists,
        AssetDoesNotExists,
        DivisionByZero,
        UnexpectedFailure,
        NotMangataLiquidityAsset,
        SecondAssetAmountExceededExpectations,
        MathOverflow,
        NotEligibleToClaimAmount,
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        //TODO add trading events
        PoolCreated(AccountId, TokenId, Balance, TokenId, Balance),
        AssetsSwapped(AccountId, TokenId, Balance, TokenId, Balance),
        LiquidityMinted(
            AccountId,
            TokenId,
            Balance,
            TokenId,
            Balance,
            TokenId,
            Balance,
        ),
        LiquidityBurned(
            AccountId,
            TokenId,
            Balance,
            TokenId,
            Balance,
            TokenId,
            Balance,
        ),
    }
);

// XYK exchange pallet storage.
decl_storage! {
    trait Store for Module<T: Trait> as XykStorage {

        // Pools get(fn asset_pool): map hasher(opaque_blake2_256) (TokenId, TokenId) => Balance;
        Pools get(fn asset_pool): map hasher(opaque_blake2_256) (TokenId, TokenId) => (Balance, Balance);

        LiquidityAssets get(fn liquidity_asset): map hasher(opaque_blake2_256) (TokenId, TokenId) => Option<TokenId>;
        LiquidityPools get(fn liquidity_pool): map hasher(opaque_blake2_256) TokenId => Option<(TokenId, TokenId)>;

        //(last checkpoint, total work until last checkpoint, missing actual at last checkpoint)
        LiquidityMiningUser get(fn liquidity_mining_user): map hasher(opaque_blake2_256) (AccountIdOf<T>, TokenId) => Option<(u32, U256, U256, )>;
        LiquidityMiningPool get(fn liquidity_mining_pool): map hasher(opaque_blake2_256) TokenId => Option<(u32, U256, U256)>;
        LiquidityMiningUserClaimed get(fn liquidity_mining_user_claimed): map hasher(opaque_blake2_256) (AccountIdOf<T>, TokenId) => Option<i128>;
    }
    add_extra_genesis {
        config(created_pools_for_staking): Vec<(T::AccountId, TokenId, Balance, TokenId, Balance, TokenId)>;

        build(|config: &GenesisConfig<T>| {
            config.created_pools_for_staking.iter().for_each(|(account_id, native_token_id, native_token_amount, pooled_token_id, pooled_token_amount, liquidity_token_id)| {
                if <T as Trait>::Currency::exists({*liquidity_token_id}.into()){
                    assert!(<Module<T>>::mint_liquidity( T::Origin::from(Some(account_id.clone()).into()), *native_token_id, *pooled_token_id, *native_token_amount, *pooled_token_amount).is_ok(), "Pool mint failed");
                }
                else{
                    let created_liquidity_token_id: TokenId = <T as Trait>::Currency::get_next_currency_id().into();
                    assert!(created_liquidity_token_id == *liquidity_token_id, "Assets not initialized in the expected sequence");
                    assert!(<Module<T>>::create_pool( T::Origin::from(Some(account_id.clone()).into()), *native_token_id, *native_token_amount, *pooled_token_id, *pooled_token_amount).is_ok(), "Pool creation failed");
                }
            })
        })
    }
}

// XYK extrinsics.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn create_pool(
            origin,
            first_asset_id: TokenId,
            first_asset_amount: Balance,
            second_asset_id: TokenId,
            second_asset_amount: Balance
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            <Self as XykFunctionsTrait<T::AccountId>>::create_pool(sender, first_asset_id, first_asset_amount, second_asset_id, second_asset_amount)

        }

        // you will sell your sold_asset_amount of sold_asset_id to get some amount of bought_asset_id
        #[weight = (10_000, Pays::No)]
        pub fn sell_asset (
            origin,
            sold_asset_id: TokenId,
            bought_asset_id: TokenId,
            sold_asset_amount: Balance,
            min_amount_out: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            <Self as XykFunctionsTrait<T::AccountId>>::sell_asset(sender, sold_asset_id, bought_asset_id, sold_asset_amount, min_amount_out)

        }

        #[weight = (10_000, Pays::No)]
        pub fn buy_asset (
            origin,
            sold_asset_id: TokenId,
            bought_asset_id: TokenId,
            bought_asset_amount: Balance,
            max_amount_in: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            <Self as XykFunctionsTrait<T::AccountId>>::buy_asset(sender, sold_asset_id, bought_asset_id, bought_asset_amount, max_amount_in)

        }

        #[weight = 10_000]
        pub fn mint_liquidity (
            origin,
            first_asset_id: TokenId,
            second_asset_id: TokenId,
            first_asset_amount: Balance,
            expected_second_asset_amount: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            <Self as XykFunctionsTrait<T::AccountId>>::mint_liquidity(sender, first_asset_id, second_asset_id, first_asset_amount, expected_second_asset_amount)

        }

        #[weight = 10_000]
        pub fn burn_liquidity (
            origin,
            first_asset_id: TokenId,
            second_asset_id: TokenId,
            liquidity_asset_amount: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            <Self as XykFunctionsTrait<T::AccountId>>::burn_liquidity(sender, first_asset_id, second_asset_id, liquidity_asset_amount)

        }

        #[weight = (10_000, Pays::No)]
        pub fn claim_rewards (
            origin,
            liquidity_token_id: TokenId,
            amount: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            <Self as XykFunctionsTrait<T::AccountId>>::claim_rewards(sender, liquidity_token_id, amount)

        }
    }
}

impl<T: Trait> Module<T> {
    pub fn calculate_rewards(
        work_user: U256,
        work_pool: U256,
        block_number: u32,
    ) -> Result<Balance, DispatchError> {
        //TODO, proper storage and calculation for total_mangata_rewards per pool, should be substracted by every claim and liq_burn (this reward should be calculated at this point, and removed from total pool, so it is no longer in consideration for any next calculation of user_work/total rewards pool)
        let total_mangata_rewards: U256 = U256::from(block_number)
            .checked_mul(U256::from(1000))
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
        let user_rewards_amount = Balance::try_from(
            total_mangata_rewards
                .checked_mul(work_user)
                .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
                .checked_div(work_pool)
                .ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?,
        )
        .map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

        Ok(user_rewards_amount)
    }

    pub fn calculate_rewards_amount(
        user: AccountIdOf<T>,
        liquidity_asset_id: TokenId,
        block_number: u32,
    ) -> Result<(Balance, i128), DispatchError> {
        let current_time = block_number / 1000;
        let work_user = Self::calculate_work_user(user.clone(), liquidity_asset_id, current_time)?;
        let work_pool = Self::calculate_work_pool(liquidity_asset_id, current_time)?;

        let already_claimed =
            Self::liquidity_mining_user_claimed((user, &liquidity_asset_id)).unwrap();
        let user_rewards_amount = Self::calculate_rewards(work_user, work_pool, block_number)?;
        Ok((user_rewards_amount, already_claimed))
    }

    pub fn calculate_work(
        asymptote: Balance,
        time: u32,
        last_checkpoint: u32,
        cummulative_work_in_last_checkpoint: U256,
        missing_at_last_checkpoint: U256,
    ) -> Result<U256, DispatchError> {
        // let (last_checkpoint,cummulative_work_in_last_checkpoint,missing_at_last_checkpoint,  )= UsersMinting::<T>::get((&user, &liquidity_asset_id)).unwrap();

        let time_passed = time - last_checkpoint;

        let asymptote_u256: U256 = asymptote.into();
        let cummulative_work_new_max_possible: U256 = asymptote_u256
            .checked_mul(U256::from(time_passed))
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
        let base = missing_at_last_checkpoint
            .checked_mul(U256::from(11))
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

        log!(
            info,
            "XXXXXXX XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
        );
        log!(
            info,
            "XXXXXXX cummulative_work_new_max_possible: {}",
            cummulative_work_new_max_possible
        );

        let precision: u32 = 10000;
        let q_pow: f64 = libm::floor(libm::pow(1.1, time_passed as f64) * precision as f64);

        log!(info, "XXXXXXX base: {}", base);
        log!(info, "XXXXXXX time_passed: {}", time_passed);
        log!(info, "XXXXXXX q_pow: {}", q_pow);
        // let q_pow2: f64 = 1.1_f64.powf(time_passed.into());
        // log!(
        //     info,
        //     "q_pow2: {:?}", q_pow2
        // );
        let cummulative_missing_new = base - base * U256::from(precision) / q_pow as u128;
        log!(
            info,
            "XXXXXXX cummulative_missing_new: {}",
            cummulative_missing_new
        );
        let cummulative_work_new = cummulative_work_new_max_possible - cummulative_missing_new;
        log!(
            info,
            "XXXXXXX cummulative_work_new: {}",
            cummulative_work_new
        );
        let work_total = cummulative_work_in_last_checkpoint + cummulative_work_new;

        log!(
            info,
            "calculate_work: {}, {}, {}, {}, {}, {} -> {}",
            asymptote,
            time,
            last_checkpoint,
            time_passed,
            cummulative_work_in_last_checkpoint,
            missing_at_last_checkpoint,
            work_total
        );
        log!(
            info,
            "XXXXXXX XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
        );

        Ok(work_total)
    }

    //TODO MODIFY FOR POOL
    pub fn calculate_work_pool(
        liquidity_asset_id: TokenId,
        current_time: u32,
    ) -> Result<U256, DispatchError> {
        let liquidity_assets_amount: Balance =
            <T as Trait>::Currency::total_issuance(liquidity_asset_id.into()).into();
        let (last_checkpoint, cummulative_work_in_last_checkpoint, missing_at_last_checkpoint) =
            Self::liquidity_mining_pool(&liquidity_asset_id).unwrap();

        let pool_work = Self::calculate_work(
            liquidity_assets_amount,
            current_time,
            last_checkpoint,
            cummulative_work_in_last_checkpoint,
            missing_at_last_checkpoint,
        )?;
        log!(
            info,
            "calculate_work_pool: {}, {}, {}, {} -> {}",
            liquidity_asset_id,
            current_time,
            last_checkpoint,
            liquidity_assets_amount,
            pool_work,
        );
        Ok(pool_work)
    }

    pub fn calculate_work_user(
        user: AccountIdOf<T>,
        liquidity_asset_id: TokenId,
        current_time: u32,
    ) -> Result<U256, DispatchError> {
        let liquidity_assets_amount: Balance =
            <T as Trait>::Currency::total_balance(liquidity_asset_id.into(), &user).into();
        let (last_checkpoint, cummulative_work_in_last_checkpoint, missing_at_last_checkpoint) =
            Self::liquidity_mining_user((&user, &liquidity_asset_id)).unwrap();
        log!(info, "");

        let user_work = Self::calculate_work(
            liquidity_assets_amount,
            current_time,
            last_checkpoint,
            cummulative_work_in_last_checkpoint,
            missing_at_last_checkpoint,
        )?;
        log!(
            info,
            "calculate_work_user: {:?}, {}, {}, {}, {} -> {}",
            user,
            liquidity_asset_id,
            current_time,
            last_checkpoint,
            liquidity_assets_amount,
            user_work,
        );
        log!(info, "");
        Ok(user_work)
    }

    pub fn calculate_missing_at_checkpoint(
        time_passed: u32,
        liquidity_assets_added: Balance,
        missing_at_last_checkpoint: U256,
    ) -> Result<U256, DispatchError> {
        let precision: u32 = 10000;
        let q_pow: f64 = libm::floor(libm::pow(1.1, time_passed as f64) * precision as f64);
        let liquidity_assets_added_u256: U256 = liquidity_assets_added.into();

        let missing_at_checkpoint: U256 = liquidity_assets_added_u256
            + missing_at_last_checkpoint * U256::from(precision) / q_pow as u128;
        log!(info, "/////////////calculate_missing_at_checkpoint",);

        log!(
            info,
            "stuff: {}, {:?}, {:?}",
            q_pow,
            time_passed,
            missing_at_last_checkpoint * U256::from(precision) / q_pow as u128,
        );
        log!(
            info,
            "liquidity_assets_added_u256, missing_at_checkpoint: {}, {:?}",
            liquidity_assets_added,
            missing_at_last_checkpoint * U256::from(precision) / q_pow as u128,
        );
        log!(
            info,
            "calculate_missing_at_checkpoint: {}, {}, {} -> {}",
            time_passed,
            liquidity_assets_added,
            missing_at_last_checkpoint,
            missing_at_checkpoint,
        );
        log!(info, "/////////////calculate_missing_at_checkpoint",);
        Ok(missing_at_checkpoint)
    }

    pub fn calculate_liquidity_checkpoint(
        user: AccountIdOf<T>,
        liquidity_asset_id: TokenId,
        liquidity_assets_added: Balance,
    ) -> Result<(u32, U256, U256, U256, U256), DispatchError> {
        let current_time: u32 =
            <frame_system::Module<T>>::block_number().saturated_into::<u32>() / 1000;
        log!(
            info,
            "***********for user********************************************************"
        );
        let mut user_work_total: U256 = U256::from(0_u32);
        let mut user_missing_at_checkpoint: U256 = liquidity_assets_added.into();
        let mut pool_work_total: U256 = U256::from(0_u32);
        let mut pool_missing_at_checkpoint: U256 = liquidity_assets_added.into();

        if LiquidityMiningUser::<T>::contains_key((&user, &liquidity_asset_id)) {
            log!(
                info,
                "already minted something?: {:?}, {}, {:?} ",
                user,
                liquidity_asset_id,
                LiquidityMiningUser::<T>::contains_key((&user, &liquidity_asset_id)),
            );
            let (
                user_last_checkpoint,
                user_cummulative_work_in_last_checkpoint,
                user_missing_at_last_checkpoint,
            ) = Self::liquidity_mining_user((&user, &liquidity_asset_id)).unwrap();
            let user_time_passed = current_time - user_last_checkpoint;
            user_missing_at_checkpoint = Self::calculate_missing_at_checkpoint(
                user_time_passed,
                liquidity_assets_added,
                user_missing_at_last_checkpoint,
            )?;
            user_work_total =
                Self::calculate_work_user(user.clone(), liquidity_asset_id, current_time)?;

            LiquidityMiningUser::<T>::insert(
                (&user, &liquidity_asset_id),
                (current_time, user_work_total, user_missing_at_checkpoint),
            );
            log!(
                info,
                "liquidity_mining_checkpoint_user: {:?}, {},{}, {} ",
                user,
                current_time,
                user_work_total,
                user_missing_at_checkpoint,
            );
        }

        log!(
            info,
            "***********for user********************************************************"
        );
        log!(info, "");
        log!(
            info,
            "***********for pool********************************************************"
        );
        if LiquidityMiningPool::contains_key(&liquidity_asset_id) {
            let (
                pool_last_checkpoint,
                pool_cummulative_work_in_last_checkpoint,
                pool_missing_at_last_checkpoint,
            ) = Self::liquidity_mining_pool(&liquidity_asset_id).unwrap();
            let pool_time_passed = current_time - pool_last_checkpoint;
            pool_missing_at_checkpoint = Self::calculate_missing_at_checkpoint(
                pool_time_passed,
                liquidity_assets_added,
                pool_missing_at_last_checkpoint,
            )?;
            pool_work_total = Self::calculate_work_pool(liquidity_asset_id, current_time)?;

            LiquidityMiningPool::insert(
                &liquidity_asset_id,
                (current_time, pool_work_total, pool_missing_at_checkpoint),
            );
            log!(
                info,
                "liquidity_mining_checkpoint_pool: {}, {}, {} ",
                current_time,
                pool_work_total,
                pool_missing_at_checkpoint,
            );
        }

        log!(
            info,
            "***********for pool********************************************************"
        );
        Ok((
            current_time,
            user_work_total,
            user_missing_at_checkpoint,
            pool_work_total,
            pool_missing_at_checkpoint,
        ))
    }

    pub fn set_liquidity_minting_checkpoint(
        user: AccountIdOf<T>,
        liquidity_asset_id: TokenId,
        liquidity_assets_added: Balance,
    ) -> DispatchResult {
        let (
            current_time,
            user_work_total,
            user_missing_at_checkpoint,
            pool_work_total,
            pool_missing_at_checkpoint,
        ) = Self::calculate_liquidity_checkpoint(
            user.clone(),
            liquidity_asset_id,
            liquidity_assets_added,
        )?;

        LiquidityMiningUser::<T>::insert(
            (&user, &liquidity_asset_id),
            (current_time, user_work_total, user_missing_at_checkpoint),
        );
        LiquidityMiningPool::insert(
            &liquidity_asset_id,
            (current_time, pool_work_total, pool_missing_at_checkpoint),
        );
        Ok(())
    }

    pub fn set_liquidity_burning_checkpoint(
        user: AccountIdOf<T>,
        liquidity_asset_id: TokenId,
        liquidity_assets_burned: Balance,
    ) -> DispatchResult {
        let (
            current_time,
            mut user_work_total,
            mut user_missing_at_checkpoint,
            mut pool_work_total,
            mut pool_missing_at_checkpoint,
        ) = Self::calculate_liquidity_checkpoint(user.clone(), liquidity_asset_id, 0 as u128)?;

        let liquidity_assets_amount: Balance =
            <T as Trait>::Currency::total_balance(liquidity_asset_id.into(), &user).into();
        // let input_reserve_saturated: U256 = input_reserve.into();
        let liquidity_assets_burned_U265: U256 = liquidity_assets_burned.into();
        let user_work_burned: U256 = liquidity_assets_burned_U265
            .checked_mul(user_work_total)
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
            .checked_div(liquidity_assets_amount.into())
            .ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;
        let user_missing_burned: U256 = liquidity_assets_burned_U265
            .checked_mul(user_missing_at_checkpoint)
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
            .checked_div(liquidity_assets_amount.into())
            .ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;

        log!(info, "LIQ BURNING -----------------------");
        log!(
            info,
            "PREV {},{},{},{},",
            user_work_total,
            user_missing_at_checkpoint,
            pool_work_total,
            pool_missing_at_checkpoint
        );
        log!(info, "MINUS {},{}", user_work_burned, user_missing_burned);
        log!(
            info,
            "NEW {},{},{},{},",
            user_work_total - user_work_burned,
            user_missing_at_checkpoint - user_missing_burned,
            pool_work_total - user_work_burned,
            pool_missing_at_checkpoint - user_missing_burned
        );
        log!(info, "LIQ BURNING -----------------------");

        LiquidityMiningUser::<T>::insert(
            (user.clone(), &liquidity_asset_id),
            (
                current_time,
                user_work_total - user_work_burned,
                user_missing_at_checkpoint - user_missing_burned,
            ),
        );
        LiquidityMiningPool::insert(
            &liquidity_asset_id,
            (
                current_time,
                pool_work_total - user_work_burned,
                pool_missing_at_checkpoint - user_missing_burned,
            ),
        );

        let mut rewards_claimed: i128 = 0;

        if LiquidityMiningUserClaimed::<T>::contains_key((user.clone(), &liquidity_asset_id)) {
            rewards_claimed =
                Self::liquidity_mining_user_claimed((user.clone(), &liquidity_asset_id)).unwrap();
        }

        let claimable_reward =
            Self::calculate_rewards(user_work_burned, pool_work_total, current_time * 1000)?;
        let rewards_claimed_new = rewards_claimed - claimable_reward as i128;
        LiquidityMiningUserClaimed::<T>::insert((&user, liquidity_asset_id), rewards_claimed_new);
        Ok(())
    }

    //  pub fn liquidity_burning_checkpoint
    //  pub fn claim_rewards

    // Sets the liquidity token's info
    // May fail if liquidity_asset_id does not exsist
    // Should not fail otherwise as the parameters for the max and min length in pallet_assets_info should be set appropriately
    pub fn set_liquidity_asset_info(
        liquidity_asset_id: TokenId,
        first_asset_id: TokenId,
        second_asset_id: TokenId,
    ) -> DispatchResult {
        let mut name: Vec<u8> = Vec::<u8>::new();
        name.extend_from_slice(LIQUIDITY_TOKEN_IDENTIFIER);
        name.extend_from_slice(HEX_INDICATOR);
        for bytes in liquidity_asset_id.to_be_bytes().iter() {
            match (bytes >> 4) as u8 {
                x @ 0u8..=9u8 => name.push(x.saturating_add(48u8)),
                x => name.push(x.saturating_add(55u8)),
            }
            match (bytes & 0b0000_1111) as u8 {
                x @ 0u8..=9u8 => name.push(x.saturating_add(48u8)),
                x => name.push(x.saturating_add(55u8)),
            }
        }

        let mut symbol: Vec<u8> = Vec::<u8>::new();
        symbol.extend_from_slice(TOKEN_SYMBOL);
        symbol.extend_from_slice(HEX_INDICATOR);
        for bytes in first_asset_id.to_be_bytes().iter() {
            match (bytes >> 4) as u8 {
                x @ 0u8..=9u8 => symbol.push(x.saturating_add(48u8)),
                x => symbol.push(x.saturating_add(55u8)),
            }
            match (bytes & 0b0000_1111) as u8 {
                x @ 0u8..=9u8 => symbol.push(x.saturating_add(48u8)),
                x => symbol.push(x.saturating_add(55u8)),
            }
        }
        symbol.extend_from_slice(TOKEN_SYMBOL_SEPARATOR);
        symbol.extend_from_slice(TOKEN_SYMBOL);
        symbol.extend_from_slice(HEX_INDICATOR);
        for bytes in second_asset_id.to_be_bytes().iter() {
            match (bytes >> 4) as u8 {
                x @ 0u8..=9u8 => symbol.push(x.saturating_add(48u8)),
                x => symbol.push(x.saturating_add(55u8)),
            }
            match (bytes & 0b0000_1111) as u8 {
                x @ 0u8..=9u8 => symbol.push(x.saturating_add(48u8)),
                x => symbol.push(x.saturating_add(55u8)),
            }
        }

        let mut description: Vec<u8> = Vec::<u8>::new();
        description.extend_from_slice(LIQUIDITY_TOKEN_DESCRIPTION);

        <assets_info::Module<T>>::set_asset_info(
            liquidity_asset_id,
            Some(name),
            Some(symbol.to_vec()),
            Some(description),
            Some(DEFAULT_DECIMALS),
        )?;
        Ok(())
    }

    // Calculate amount of tokens to be bought by sellling sell_amount
    pub fn calculate_sell_price(
        input_reserve: Balance,
        output_reserve: Balance,
        sell_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let after_fee_percentage: u128 = 10000 - FEE_PERCENTAGE;
        let input_reserve_saturated: U256 = input_reserve.into();
        let output_reserve_saturated: U256 = output_reserve.into();
        let sell_amount_saturated: U256 = sell_amount.into();

        let input_amount_with_fee: U256 =
            sell_amount_saturated.saturating_mul(after_fee_percentage.into());

        let numerator: U256 = input_amount_with_fee
            .checked_mul(output_reserve_saturated)
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

        let denominator: U256 = input_reserve_saturated
            .saturating_mul(10000.into())
            .checked_add(input_amount_with_fee)
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

        let result_u256 = numerator
            .checked_div(denominator)
            .ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;

        let result = Balance::try_from(result_u256)
            .map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
        log!(
            info,
            "calculate_sell_price: ({}, {}, {}) -> {}",
            input_reserve,
            output_reserve,
            sell_amount,
            result
        );
        Ok(result)
    }

    pub fn calculate_sell_price_no_fee(
        // Callculate amount of tokens to be received by sellling sell_amount, without fee
        input_reserve: Balance,
        output_reserve: Balance,
        sell_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let input_reserve_saturated: U256 = input_reserve.into();
        let output_reserve_saturated: U256 = output_reserve.into();
        let sell_amount_saturated: U256 = sell_amount.into();

        let numerator: U256 = sell_amount_saturated.saturating_mul(output_reserve_saturated);
        let denominator: U256 = input_reserve_saturated.saturating_add(sell_amount_saturated);
        let result_u256 = numerator
            .checked_div(denominator)
            .ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;
        let result = Balance::try_from(result_u256)
            .map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
        log!(
            info,
            "calculate_sell_price_no_fee: ({}, {}, {}) -> {}",
            input_reserve,
            output_reserve,
            sell_amount,
            result
        );
        Ok(result)
    }

    // Calculate amount of tokens to be paid, when buying buy_amount
    pub fn calculate_buy_price(
        input_reserve: Balance,
        output_reserve: Balance,
        buy_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let after_fee_percentage: u128 = 10000 - FEE_PERCENTAGE;
        let input_reserve_saturated: U256 = input_reserve.into();
        let output_reserve_saturated: U256 = output_reserve.into();
        let buy_amount_saturated: U256 = buy_amount.into();

        let numerator: U256 = input_reserve_saturated
            .saturating_mul(buy_amount_saturated)
            .checked_mul(10000.into())
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

        let denominator: U256 = output_reserve_saturated
            .checked_sub(buy_amount_saturated)
            .ok_or_else(|| DispatchError::from(Error::<T>::NotEnoughReserve))?
            .checked_mul(after_fee_percentage.into())
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

        let result_u256 = numerator
            .checked_div(denominator)
            .ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?
            .checked_add(1.into())
            .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

        let result = Balance::try_from(result_u256)
            .map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
        log!(
            info,
            "calculate_buy_price: ({}, {}, {}) -> {}",
            input_reserve,
            output_reserve,
            buy_amount,
            result
        );
        Ok(result)
    }

    pub fn get_liquidity_asset(
        first_asset_id: TokenId,
        second_asset_id: TokenId,
    ) -> Result<TokenId, DispatchError> {
        if LiquidityAssets::contains_key((first_asset_id, second_asset_id)) {
            LiquidityAssets::get((first_asset_id, second_asset_id))
                .ok_or_else(|| Error::<T>::UnexpectedFailure.into())
        } else {
            LiquidityAssets::get((second_asset_id, first_asset_id))
                .ok_or_else(|| Error::<T>::NoSuchPool.into())
        }
    }

    pub fn calculate_sell_price_id(
        sold_token_id: TokenId,
        bought_token_id: TokenId,
        sell_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let (input_reserve, output_reserve) =
            Module::<T>::get_reserves(sold_token_id, bought_token_id)?;

        Self::calculate_sell_price(input_reserve, output_reserve, sell_amount)
    }

    pub fn calculate_buy_price_id(
        sold_token_id: TokenId,
        bought_token_id: TokenId,
        buy_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let (input_reserve, output_reserve) =
            Module::<T>::get_reserves(sold_token_id, bought_token_id)?;

        Self::calculate_buy_price(input_reserve, output_reserve, buy_amount)
    }

    pub fn get_reserves(
        first_asset_id: TokenId,
        second_asset_id: TokenId,
    ) -> Result<(Balance, Balance), DispatchError> {
        let mut reserves = Pools::get((first_asset_id, second_asset_id));

        if Pools::contains_key((first_asset_id, second_asset_id)) {
            return Ok((reserves.0, reserves.1));
        } else if Pools::contains_key((second_asset_id, first_asset_id)) {
            reserves = Pools::get((second_asset_id, first_asset_id));
            return Ok((reserves.1, reserves.0));
        } else {
            return Err(DispatchError::from(Error::<T>::NoSuchPool));
        }
    }

    pub fn set_reserves(
        first_asset_id: TokenId,
        first_asset_amount: Balance,
        second_asset_id: TokenId,
        second_asset_amount: Balance,
    ) -> DispatchResult {
        if Pools::contains_key((first_asset_id, second_asset_id)) {
            Pools::insert(
                (first_asset_id, second_asset_id),
                (first_asset_amount, second_asset_amount),
            );
        } else if Pools::contains_key((second_asset_id, first_asset_id)) {
            Pools::insert(
                (second_asset_id, first_asset_id),
                (second_asset_amount, first_asset_amount),
            );
        } else {
            return Err(DispatchError::from(Error::<T>::NoSuchPool));
        }

        Ok(())
    }

    // Calculate first and second token amounts depending on liquidity amount to burn
    pub fn get_burn_amount(
        first_asset_id: TokenId,
        second_asset_id: TokenId,
        liquidity_asset_amount: Balance,
    ) -> Result<(Balance, Balance), DispatchError> {
        // Get token reserves and liquidity asset id
        let liquidity_asset_id = Self::get_liquidity_asset(first_asset_id, second_asset_id)?;
        let (first_asset_reserve, second_asset_reserve) =
            Module::<T>::get_reserves(first_asset_id, second_asset_id)?;

        let (first_asset_amount, second_asset_amount) = Self::get_burn_amount_reserves(
            first_asset_reserve,
            second_asset_reserve,
            liquidity_asset_id,
            liquidity_asset_amount,
        )?;

        log!(
            info,
            "get_burn_amount: ({}, {}, {}) -> ({}, {})",
            first_asset_id,
            second_asset_id,
            liquidity_asset_amount,
            first_asset_amount,
            second_asset_amount
        );

        Ok((first_asset_amount, second_asset_amount))
    }

    pub fn get_burn_amount_reserves(
        first_asset_reserve: Balance,
        second_asset_reserve: Balance,
        liquidity_asset_id: TokenId,
        liquidity_asset_amount: Balance,
    ) -> Result<(Balance, Balance), DispatchError> {
        // Get token reserves and liquidity asset id

        let total_liquidity_assets: Balance =
            <T as Trait>::Currency::total_issuance(liquidity_asset_id.into()).into();

        // Calculate first and second token amount to be withdrawn
        ensure!(
            !total_liquidity_assets.is_zero(),
            Error::<T>::DivisionByZero
        );
        let first_asset_amount = multiply_by_rational(
            first_asset_reserve,
            liquidity_asset_amount,
            total_liquidity_assets,
        )
        .map_err(|_| Error::<T>::UnexpectedFailure)?;
        let second_asset_amount = multiply_by_rational(
            second_asset_reserve,
            liquidity_asset_amount,
            total_liquidity_assets,
        )
        .map_err(|_| Error::<T>::UnexpectedFailure)?;

        Ok((first_asset_amount, second_asset_amount))
    }

    //TODO if pool contains key !
    fn settle_treasury_and_burn(
        sold_asset_id: TokenId,
        burn_amount: Balance,
        treasury_amount: Balance,
    ) -> DispatchResult {
        let vault = Self::account_id();
        let mangata_id: TokenId = T::NativeCurrencyId::get();
        let treasury_account: T::AccountId = Self::treasury_account_id();
        let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

        // If settling token is mangata, treasury amount is added to treasury and burn amount is burned from corresponding pool
        if sold_asset_id == mangata_id {
            // treasury_amount of MGA is already in treasury at this point

            // MGA burned from bnb_treasury_account
            <T as Trait>::Currency::burn_and_settle(
                sold_asset_id.into(),
                &bnb_treasury_account,
                burn_amount.into(),
            )?;
        }
        //If settling token is connected to mangata, token is swapped in corresponding pool to mangata without fee
        else if Pools::contains_key((sold_asset_id, mangata_id))
            || Pools::contains_key((mangata_id, sold_asset_id))
        {
            // Getting token reserves
            let (input_reserve, output_reserve) =
                Module::<T>::get_reserves(sold_asset_id, mangata_id)?;

            // Calculating swapped mangata amount
            let settle_amount_in_mangata = Self::calculate_sell_price_no_fee(
                input_reserve,
                output_reserve,
                treasury_amount + burn_amount,
            )?;
            let treasury_amount_in_mangata = settle_amount_in_mangata * TREASURY_PERCENTAGE
                / (TREASURY_PERCENTAGE + BUYANDBURN_PERCENTAGE);

            let burn_amount_in_mangata = settle_amount_in_mangata - treasury_amount_in_mangata;

            // Apply changes in token pools, adding treasury and burn amounts of settling token, removing  treasury and burn amounts of mangata

            Module::<T>::set_reserves(
                sold_asset_id,
                input_reserve
                    .saturating_add(treasury_amount)
                    .saturating_add(burn_amount),
                mangata_id,
                output_reserve
                    .saturating_sub(treasury_amount_in_mangata)
                    .saturating_sub(burn_amount_in_mangata),
            )?;

            <T as Trait>::Currency::transfer(
                sold_asset_id.into(),
                &treasury_account,
                &vault,
                treasury_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;

            <T as Trait>::Currency::transfer(
                mangata_id.into(),
                &vault,
                &treasury_account,
                treasury_amount_in_mangata.into(),
                ExistenceRequirement::KeepAlive,
            )?;

            <T as Trait>::Currency::transfer(
                sold_asset_id.into(),
                &bnb_treasury_account,
                &vault,
                burn_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;

            // Mangata burned from pool
            <T as Trait>::Currency::burn_and_settle(
                mangata_id.into(),
                &vault,
                burn_amount_in_mangata.into(),
            )?;
        }
        // Settling token has no mangata connection, settling token is added to treasuries
        else {
            // Both treasury_amount and buy_and_burn_amount of sold_asset are in their respective treasuries
        }
        Ok(())
    }

    fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

    fn treasury_account_id() -> T::AccountId {
        T::TreasuryModuleId::get().into_account()
    }

    fn bnb_treasury_account_id() -> T::AccountId {
        T::TreasuryModuleId::get().into_sub_account(T::BnbTreasurySubAccDerive::get())
    }
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
    ) -> DispatchResult;

    fn burn_liquidity(
        sender: AccountId,
        first_asset_id: Self::CurrencyId,
        second_asset_id: Self::CurrencyId,
        liquidity_asset_amount: Self::Balance,
    ) -> DispatchResult;

    fn claim_rewards(
        sender: AccountId,
        liquidity_asset_id: Self::CurrencyId,
        liquidity_asset_amount: Self::Balance,
    ) -> DispatchResult;

    fn get_tokens_required_for_minting(
        liquidity_asset_id: Self::CurrencyId,
        liquidity_token_amount: Self::Balance,
    ) -> Result<
        (
            Self::CurrencyId,
            Self::Balance,
            Self::CurrencyId,
            Self::Balance,
        ),
        DispatchError,
    >;
}

impl<T: Trait> XykFunctionsTrait<T::AccountId> for Module<T> {
    type Balance = Balance;

    type CurrencyId = TokenId;

    fn create_pool(
        sender: T::AccountId,
        first_asset_id: Self::CurrencyId,
        first_asset_amount: Self::Balance,
        second_asset_id: Self::CurrencyId,
        second_asset_amount: Self::Balance,
    ) -> DispatchResult {
        let vault: T::AccountId = Module::<T>::account_id();

        // Ensure pool is not created with zero amount
        ensure!(
            !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
            Error::<T>::ZeroAmount,
        );

        // Ensure pool does not exists yet
        ensure!(
            !Pools::contains_key((first_asset_id, second_asset_id)),
            Error::<T>::PoolAlreadyExists,
        );

        // Ensure pool does not exists yet
        ensure!(
            !Pools::contains_key((second_asset_id, first_asset_id)),
            Error::<T>::PoolAlreadyExists,
        );

        // Getting users token balances
        let first_asset_free_balance: Self::Balance =
            <T as Trait>::Currency::free_balance(first_asset_id.into(), &sender).into();
        let second_asset_free_balance: Self::Balance =
            <T as Trait>::Currency::free_balance(second_asset_id.into(), &sender).into();

        // Ensure user has enough withdrawable tokens to create pool in amounts required

        <T as Trait>::Currency::ensure_can_withdraw(
            first_asset_id.into(),
            &sender,
            first_asset_amount.into(),
            WithdrawReasons::all(),
            // Does not fail due to earlier ensure
            { first_asset_free_balance.saturating_sub(first_asset_amount) }.into(),
        )
        .or(Err(Error::<T>::NotEnoughAssets))?;

        <T as Trait>::Currency::ensure_can_withdraw(
            second_asset_id.into(),
            &sender,
            second_asset_amount.into(),
            WithdrawReasons::all(),
            // Does not fail due to earlier ensure
            { second_asset_free_balance.saturating_sub(second_asset_amount) }.into(),
        )
        .or(Err(Error::<T>::NotEnoughAssets))?;

        // Ensure pool is not created with same token in pair
        ensure!(first_asset_id != second_asset_id, Error::<T>::SameAsset,);

        // Liquidity token amount calculation
        let mut initial_liquidity = first_asset_amount / 2 + second_asset_amount / 2;
        if initial_liquidity == 0 {
            initial_liquidity = 1
        }

        Pools::insert(
            (first_asset_id, second_asset_id),
            (first_asset_amount, second_asset_amount),
        );

        // Pools::insert((second_asset_id, first_asset_id), second_asset_amount);

        // Moving tokens from user to vault
        <T as Trait>::Currency::transfer(
            first_asset_id.into(),
            &sender,
            &vault,
            first_asset_amount.into(),
            ExistenceRequirement::AllowDeath,
        )?;

        <T as Trait>::Currency::transfer(
            second_asset_id.into(),
            &sender,
            &vault,
            second_asset_amount.into(),
            ExistenceRequirement::AllowDeath,
        )?;

        // Creating new liquidity token and transfering it to user
        let liquidity_asset_id: Self::CurrencyId =
            <T as Trait>::Currency::create(&sender, initial_liquidity.into()).into();

        // Adding info about liquidity asset
        LiquidityAssets::insert((first_asset_id, second_asset_id), liquidity_asset_id);
        LiquidityPools::insert(liquidity_asset_id, (first_asset_id, second_asset_id));

        log!(
            info,
            "create_pool: ({:?}, {}, {}, {}, {}) -> ({}, {})",
            sender,
            first_asset_id,
            first_asset_amount,
            second_asset_id,
            second_asset_amount,
            liquidity_asset_id,
            initial_liquidity
        );

        log!(
            info,
            "pool-state: [({}, {}) -> {}, ({}, {}) -> {}]",
            first_asset_id,
            second_asset_id,
            first_asset_amount,
            second_asset_id,
            first_asset_id,
            second_asset_amount
        );
        // This, will and should, never fail
        Module::<T>::set_liquidity_asset_info(liquidity_asset_id, first_asset_id, second_asset_id)?;
        Module::<T>::set_liquidity_minting_checkpoint(
            sender.clone(),
            liquidity_asset_id,
            initial_liquidity,
        );

        Module::<T>::deposit_event(RawEvent::PoolCreated(
            sender,
            first_asset_id,
            first_asset_amount,
            second_asset_id,
            second_asset_amount,
        ));

        Ok(())
    }

    fn sell_asset(
        sender: T::AccountId,
        sold_asset_id: Self::CurrencyId,
        bought_asset_id: Self::CurrencyId,
        sold_asset_amount: Self::Balance,
        min_amount_out: Self::Balance,
    ) -> DispatchResult {
        // Ensure not selling zero amount
        ensure!(!sold_asset_amount.is_zero(), Error::<T>::ZeroAmount,);

        let buy_and_burn_amount =
            multiply_by_rational(sold_asset_amount, BUYANDBURN_PERCENTAGE, 10000)
                .map_err(|_| Error::<T>::UnexpectedFailure)?
                + 1;

        let treasury_amount = multiply_by_rational(sold_asset_amount, TREASURY_PERCENTAGE, 10000)
            .map_err(|_| Error::<T>::UnexpectedFailure)?
            + 1;

        let pool_fee_amount = multiply_by_rational(sold_asset_amount, POOL_FEE_PERCENTAGE, 10000)
            .map_err(|_| Error::<T>::UnexpectedFailure)?
            + 1;

        // for future implementation of min fee if necessary
        // let min_fee: u128 = 0;
        // if buy_and_burn_amount + treasury_amount + pool_fee_amount < min_fee {
        //     buy_and_burn_amount = min_fee * FEE_PERCENTAGE / BUYANDBURN_PERCENTAGE;
        //     treasury_amount = min_fee * FEE_PERCENTAGE / TREASURY_PERCENTAGE;
        //     pool_fee_amount = min_fee - buy_and_burn_amount - treasury_amount;
        // }

        // Get token reserves

        let (input_reserve, output_reserve) =
            Module::<T>::get_reserves(sold_asset_id, bought_asset_id)?;

        ensure!(
            input_reserve.checked_add(sold_asset_amount).is_some(),
            Error::<T>::MathOverflow
        );

        // Calculate bought asset amount to be received by paying sold asset amount
        let bought_asset_amount =
            Module::<T>::calculate_sell_price(input_reserve, output_reserve, sold_asset_amount)?;

        // Getting users token balances
        let sold_asset_free_balance: Self::Balance =
            <T as Trait>::Currency::free_balance(sold_asset_id.into(), &sender).into();

        // Ensure user has enough tokens to sell
        <T as Trait>::Currency::ensure_can_withdraw(
            sold_asset_id.into(),
            &sender,
            sold_asset_amount.into(),
            WithdrawReasons::all(),
            // Does not fail due to earlier ensure
            { sold_asset_free_balance.saturating_sub(sold_asset_amount) }.into(),
        )
        .or(Err(Error::<T>::NotEnoughAssets))?;

        let vault = Module::<T>::account_id();
        let treasury_account: T::AccountId = Self::treasury_account_id();
        let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

        // Transfer of fees, before tx can fail on min amount out
        <T as Trait>::Currency::transfer(
            sold_asset_id.into(),
            &sender,
            &vault,
            pool_fee_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;

        <T as Trait>::Currency::transfer(
            sold_asset_id.into(),
            &sender,
            &treasury_account,
            treasury_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;

        <T as Trait>::Currency::transfer(
            sold_asset_id.into(),
            &sender,
            &bnb_treasury_account,
            buy_and_burn_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;

        // Add pool fee to pool
        Module::<T>::set_reserves(
            sold_asset_id,
            input_reserve.saturating_add(pool_fee_amount),
            bought_asset_id,
            output_reserve,
        )?;

        // Ensure bought token amount is higher then requested minimal amount
        if bought_asset_amount >= min_amount_out {
            // Transfer the rest of sold token amount from user to vault and bought token amount from vault to user
            <T as Trait>::Currency::transfer(
                sold_asset_id.into(),
                &sender,
                &vault,
                (sold_asset_amount - buy_and_burn_amount - treasury_amount - pool_fee_amount)
                    .into(),
                ExistenceRequirement::KeepAlive,
            )?;
            <T as Trait>::Currency::transfer(
                bought_asset_id.into(),
                &vault,
                &sender,
                bought_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;

            // Apply changes in token pools, adding sold amount and removing bought amount
            // Neither should fall to zero let alone underflow, due to how pool destruction works
            // Won't overflow due to earlier ensure
            let input_reserve_updated = input_reserve
                .saturating_add(sold_asset_amount - treasury_amount - buy_and_burn_amount);
            let output_reserve_updated = output_reserve.saturating_sub(bought_asset_amount);

            Module::<T>::set_reserves(
                sold_asset_id,
                input_reserve_updated,
                bought_asset_id,
                output_reserve_updated,
            )?;

            log!(
                info,
                "sell_asset: ({:?}, {}, {}, {}, {}) -> {}",
                sender,
                sold_asset_id,
                bought_asset_id,
                sold_asset_amount,
                min_amount_out,
                bought_asset_amount
            );

            log!(
                info,
                "pool-state: [({}, {}) -> {}, ({}, {}) -> {}]",
                sold_asset_id,
                bought_asset_id,
                input_reserve_updated,
                bought_asset_id,
                sold_asset_id,
                output_reserve_updated
            );

            Module::<T>::deposit_event(RawEvent::AssetsSwapped(
                sender,
                sold_asset_id,
                sold_asset_amount,
                bought_asset_id,
                bought_asset_amount,
            ));
        }

        // Settle tokens which goes to treasury and for buy and burn purpose
        Module::<T>::settle_treasury_and_burn(sold_asset_id, buy_and_burn_amount, treasury_amount)?;

        if bought_asset_amount < min_amount_out {
            return Err(DispatchError::from(Error::<T>::InsufficientOutputAmount));
        }

        Ok(())
    }

    fn buy_asset(
        sender: T::AccountId,
        sold_asset_id: Self::CurrencyId,
        bought_asset_id: Self::CurrencyId,
        bought_asset_amount: Self::Balance,
        max_amount_in: Self::Balance,
    ) -> DispatchResult {
        // Get token reserves
        let (input_reserve, output_reserve) =
            Module::<T>::get_reserves(sold_asset_id, bought_asset_id)?;

        // Ensure there are enough tokens in reserves
        ensure!(
            output_reserve > bought_asset_amount,
            Error::<T>::NotEnoughReserve,
        );

        // Ensure not buying zero amount
        ensure!(!bought_asset_amount.is_zero(), Error::<T>::ZeroAmount,);

        // Calculate amount to be paid from bought amount
        let sold_asset_amount =
            Module::<T>::calculate_buy_price(input_reserve, output_reserve, bought_asset_amount)?;

        let buy_and_burn_amount =
            multiply_by_rational(sold_asset_amount, BUYANDBURN_PERCENTAGE, 10000)
                .map_err(|_| Error::<T>::UnexpectedFailure)?
                + 1;

        let treasury_amount = multiply_by_rational(sold_asset_amount, TREASURY_PERCENTAGE, 10000)
            .map_err(|_| Error::<T>::UnexpectedFailure)?
            + 1;

        let pool_fee_amount = multiply_by_rational(sold_asset_amount, POOL_FEE_PERCENTAGE, 10000)
            .map_err(|_| Error::<T>::UnexpectedFailure)?
            + 1;

        // for future implementation of min fee if necessary
        // let min_fee: u128 = 0;
        // if buy_and_burn_amount + treasury_amount + pool_fee_amount < min_fee {
        //     buy_and_burn_amount = min_fee * FEE_PERCENTAGE / BUYANDBURN_PERCENTAGE;
        //     treasury_amount = min_fee * FEE_PERCENTAGE / TREASURY_PERCENTAGE;
        //     pool_fee_amount = min_fee - buy_and_burn_amount - treasury_amount;
        // }

        ensure!(
            input_reserve.checked_add(sold_asset_amount).is_some(),
            Error::<T>::MathOverflow
        );

        // Getting users token balances
        let sold_asset_free_balance: Self::Balance =
            <T as Trait>::Currency::free_balance(sold_asset_id.into(), &sender).into();

        // Ensure user has enough tokens to sell
        <T as Trait>::Currency::ensure_can_withdraw(
            sold_asset_id.into(),
            &sender,
            sold_asset_amount.into(),
            WithdrawReasons::all(),
            // Does not fail due to earlier ensure
            { sold_asset_free_balance.saturating_sub(sold_asset_amount) }.into(),
        )
        .or(Err(Error::<T>::NotEnoughAssets))?;

        let vault = Module::<T>::account_id();
        let treasury_account: T::AccountId = Self::treasury_account_id();
        let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

        // Transfer of fees, before tx can fail on min amount out
        <T as Trait>::Currency::transfer(
            sold_asset_id.into(),
            &sender,
            &vault,
            pool_fee_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;

        <T as Trait>::Currency::transfer(
            sold_asset_id.into(),
            &sender,
            &treasury_account,
            treasury_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;

        <T as Trait>::Currency::transfer(
            sold_asset_id.into(),
            &sender,
            &bnb_treasury_account,
            buy_and_burn_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;

        // Add pool fee to pool
        Module::<T>::set_reserves(
            sold_asset_id,
            input_reserve.saturating_add(pool_fee_amount),
            bought_asset_id,
            output_reserve,
        )?;

        // Ensure paid amount is less then maximum allowed price
        if sold_asset_amount <= max_amount_in {
            // Transfer sold token amount from user to vault and bought token amount from vault to user
            <T as Trait>::Currency::transfer(
                sold_asset_id.into(),
                &sender,
                &vault,
                (sold_asset_amount - buy_and_burn_amount - treasury_amount - pool_fee_amount)
                    .into(),
                ExistenceRequirement::KeepAlive,
            )?;
            <T as Trait>::Currency::transfer(
                bought_asset_id.into(),
                &vault,
                &sender,
                bought_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;

            // Apply changes in token pools, adding sold amount and removing bought amount
            // Neither should fall to zero let alone underflow, due to how pool destruction works
            // Won't overflow due to earlier ensure
            let input_reserve_updated = input_reserve
                .saturating_add(sold_asset_amount - treasury_amount - buy_and_burn_amount);
            let output_reserve_updated = output_reserve.saturating_sub(bought_asset_amount);
            Module::<T>::set_reserves(
                sold_asset_id,
                input_reserve_updated,
                bought_asset_id,
                output_reserve_updated,
            )?;

            log!(
                info,
                "buy_asset: ({:?}, {}, {}, {}, {}) -> {}",
                sender,
                sold_asset_id,
                bought_asset_id,
                bought_asset_amount,
                max_amount_in,
                sold_asset_amount
            );

            log!(
                info,
                "pool-state: [({}, {}) -> {}, ({}, {}) -> {}]",
                sold_asset_id,
                bought_asset_id,
                input_reserve_updated,
                bought_asset_id,
                sold_asset_id,
                output_reserve_updated
            );

            Module::<T>::deposit_event(RawEvent::AssetsSwapped(
                sender,
                sold_asset_id,
                sold_asset_amount,
                bought_asset_id,
                bought_asset_amount,
            ));
        }
        // Settle tokens which goes to treasury and for buy and burn purpose
        Module::<T>::settle_treasury_and_burn(sold_asset_id, buy_and_burn_amount, treasury_amount)?;

        if sold_asset_amount > max_amount_in {
            return Err(DispatchError::from(Error::<T>::InsufficientInputAmount));
        }

        Ok(())
    }

    fn mint_liquidity(
        sender: T::AccountId,
        first_asset_id: Self::CurrencyId,
        second_asset_id: Self::CurrencyId,
        first_asset_amount: Self::Balance,
        expected_second_asset_amount: Self::Balance,
    ) -> DispatchResult {
        let vault = Module::<T>::account_id();

        // Ensure pool exists
        ensure!(
            (LiquidityAssets::contains_key((first_asset_id, second_asset_id))
                || LiquidityAssets::contains_key((second_asset_id, first_asset_id))),
            Error::<T>::NoSuchPool,
        );

        // TODO move ensure in get_liq_asset ?
        // Get liquidity token id
        let liquidity_asset_id = Module::<T>::get_liquidity_asset(first_asset_id, second_asset_id)?;

        // Get token reserves
        let (first_asset_reserve, second_asset_reserve) =
            Module::<T>::get_reserves(first_asset_id, second_asset_id)?;
        let total_liquidity_assets: Self::Balance =
            <T as Trait>::Currency::total_issuance(liquidity_asset_id.into()).into();

        // Calculation of required second asset amount and received liquidity token amount
        ensure!(!first_asset_reserve.is_zero(), Error::<T>::DivisionByZero);
        let second_asset_amount = multiply_by_rational(
            first_asset_amount,
            second_asset_reserve,
            first_asset_reserve,
        )
        .map_err(|_| Error::<T>::UnexpectedFailure)?
        .checked_add(1)
        .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
        let liquidity_assets_minted = multiply_by_rational(
            first_asset_amount,
            total_liquidity_assets,
            first_asset_reserve,
        )
        .map_err(|_| Error::<T>::UnexpectedFailure)?;

        ensure!(
            second_asset_amount <= expected_second_asset_amount,
            Error::<T>::SecondAssetAmountExceededExpectations,
        );

        // Ensure minting amounts are not zero
        ensure!(
            !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
            Error::<T>::ZeroAmount,
        );

        // Getting users token balances
        let first_asset_free_balance: Self::Balance =
            <T as Trait>::Currency::free_balance(first_asset_id.into(), &sender).into();
        let second_asset_free_balance: Self::Balance =
            <T as Trait>::Currency::free_balance(second_asset_id.into(), &sender).into();

        // Ensure user has enough withdrawable tokens to create pool in amounts required

        <T as Trait>::Currency::ensure_can_withdraw(
            first_asset_id.into(),
            &sender,
            first_asset_amount.into(),
            WithdrawReasons::all(),
            // Does not fail due to earlier ensure
            { first_asset_free_balance.saturating_sub(first_asset_amount) }.into(),
        )
        .or(Err(Error::<T>::NotEnoughAssets))?;

        <T as Trait>::Currency::ensure_can_withdraw(
            second_asset_id.into(),
            &sender,
            second_asset_amount.into(),
            WithdrawReasons::all(),
            // Does not fail due to earlier ensure
            { second_asset_free_balance.saturating_sub(second_asset_amount) }.into(),
        )
        .or(Err(Error::<T>::NotEnoughAssets))?;

        // Transfer of token amounts from user to vault
        <T as Trait>::Currency::transfer(
            first_asset_id.into(),
            &sender,
            &vault,
            first_asset_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;
        <T as Trait>::Currency::transfer(
            second_asset_id.into(),
            &sender,
            &vault,
            second_asset_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;
        Module::<T>::set_liquidity_minting_checkpoint(
            sender.clone(),
            liquidity_asset_id,
            liquidity_assets_minted,
        );

        // Creating new liquidity tokens to user
        <T as Trait>::Currency::mint(
            liquidity_asset_id.into(),
            &sender,
            liquidity_assets_minted.into(),
        )?;

        // Apply changes in token pools, adding minted amounts
        // Won't overflow due earlier ensure
        let first_asset_reserve_updated = first_asset_reserve.saturating_add(first_asset_amount);
        let second_asset_reserve_updated = second_asset_reserve.saturating_add(second_asset_amount);
        Module::<T>::set_reserves(
            first_asset_id,
            first_asset_reserve_updated,
            second_asset_id,
            second_asset_reserve_updated,
        )?;
        log!(
            info,
            "mint_liquidity: ({:?}, {}, {}, {}) -> ({}, {}, {})",
            sender,
            first_asset_id,
            second_asset_id,
            first_asset_amount,
            second_asset_amount,
            liquidity_asset_id,
            liquidity_assets_minted
        );

        log!(
            info,
            "pool-state: [({}, {}) -> {}, ({}, {}) -> {}]",
            first_asset_id,
            second_asset_id,
            first_asset_reserve_updated,
            second_asset_id,
            first_asset_id,
            second_asset_reserve_updated
        );

        Module::<T>::deposit_event(RawEvent::LiquidityMinted(
            sender,
            first_asset_id,
            first_asset_amount,
            second_asset_id,
            second_asset_amount,
            liquidity_asset_id,
            liquidity_assets_minted,
        ));

        Ok(())
    }

    fn burn_liquidity(
        sender: T::AccountId,
        first_asset_id: Self::CurrencyId,
        second_asset_id: Self::CurrencyId,
        liquidity_asset_amount: Self::Balance,
    ) -> DispatchResult {
        let vault = Module::<T>::account_id();

        // Get token reserves and liquidity asset id
        let (first_asset_reserve, second_asset_reserve) =
            Module::<T>::get_reserves(first_asset_id, second_asset_id)?;
        let liquidity_asset_id = Module::<T>::get_liquidity_asset(first_asset_id, second_asset_id)?;

        // Ensure user has enought liquidity tokens to burn
        ensure!(
            <T as Trait>::Currency::can_slash(
                liquidity_asset_id.into(),
                &sender,
                liquidity_asset_amount.into()
            ),
            Error::<T>::NotEnoughAssets,
        );
        let new_balance: Self::Balance =
            <T as Trait>::Currency::free_balance(liquidity_asset_id.into(), &sender)
                .into()
                .checked_sub(liquidity_asset_amount)
                .ok_or_else(|| DispatchError::from(Error::<T>::NotEnoughAssets))?;

        <T as Trait>::Currency::ensure_can_withdraw(
            liquidity_asset_id.into(),
            &sender,
            liquidity_asset_amount.into(),
            WithdrawReasons::all(),
            new_balance.into(),
        )
        .or(Err(Error::<T>::NotEnoughAssets))?;

        // Calculate first and second token amounts depending on liquidity amount to burn
        let (first_asset_amount, second_asset_amount) = Module::<T>::get_burn_amount_reserves(
            first_asset_reserve,
            second_asset_reserve,
            liquidity_asset_id,
            liquidity_asset_amount,
        )?;

        let total_liquidity_assets: Balance =
            <T as Trait>::Currency::total_issuance(liquidity_asset_id.into()).into();

        // If all liquidity assets are being burned then
        // both asset amounts must be equal to their reserve values
        // All storage values related to this pool must be destroyed
        if liquidity_asset_amount == total_liquidity_assets {
            ensure!(
                (first_asset_reserve == first_asset_amount)
                    && (second_asset_reserve == second_asset_amount),
                Error::<T>::UnexpectedFailure
            );
        } else {
            ensure!(
                (first_asset_reserve >= first_asset_amount)
                    && (second_asset_reserve >= second_asset_amount),
                Error::<T>::UnexpectedFailure
            );
        }
        // If all liquidity assets are not being burned then
        // both asset amounts must be less than their reserve values

        // Ensure not withdrawing zero amounts
        ensure!(
            !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
            Error::<T>::ZeroAmount,
        );

        // Transfer withdrawn amounts from vault to user
        <T as Trait>::Currency::transfer(
            first_asset_id.into(),
            &vault,
            &sender,
            first_asset_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;
        <T as Trait>::Currency::transfer(
            second_asset_id.into(),
            &vault,
            &sender,
            second_asset_amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;

        log!(
            info,
            "burn_liquidity: ({:?}, {}, {}, {}) -> ({}, {})",
            sender,
            first_asset_id,
            second_asset_id,
            liquidity_asset_amount,
            first_asset_amount,
            second_asset_amount
        );

        if liquidity_asset_amount == total_liquidity_assets {
            log!(
                info,
                "pool-state: [({}, {}) -> Removed, ({}, {}) -> Removed]",
                first_asset_id,
                second_asset_id,
                second_asset_id,
                first_asset_id,
            );
            Pools::remove((first_asset_id, second_asset_id));
            Pools::remove((second_asset_id, first_asset_id));
            LiquidityAssets::remove((first_asset_id, second_asset_id));
            LiquidityAssets::remove((second_asset_id, first_asset_id));
            LiquidityPools::remove(liquidity_asset_id);
        } else {
            // Apply changes in token pools, removing withdrawn amounts
            // Cannot underflow due to earlier ensure
            // check was executed in get_reserves call
            let first_asset_reserve_updated =
                first_asset_reserve.saturating_sub(first_asset_amount);
            let second_asset_reserve_updated =
                second_asset_reserve.saturating_sub(second_asset_amount);
            Module::<T>::set_reserves(
                first_asset_id,
                first_asset_reserve_updated,
                second_asset_id,
                second_asset_reserve_updated,
            )?;

            log!(
                info,
                "pool-state: [({}, {}) -> {}, ({}, {}) -> {}]",
                first_asset_id,
                second_asset_id,
                first_asset_reserve_updated,
                second_asset_id,
                first_asset_id,
                second_asset_reserve_updated
            );
        }

        Module::<T>::set_liquidity_burning_checkpoint(
            sender.clone(),
            liquidity_asset_id,
            liquidity_asset_amount,
        );

        // Destroying burnt liquidity tokens
        <T as Trait>::Currency::burn_and_settle(
            liquidity_asset_id.into(),
            &sender,
            liquidity_asset_amount.into(),
        )?;

        Module::<T>::deposit_event(RawEvent::LiquidityBurned(
            sender,
            first_asset_id,
            first_asset_amount,
            second_asset_id,
            second_asset_amount,
            liquidity_asset_id,
            liquidity_asset_amount,
        ));

        Ok(())
    }

    fn claim_rewards(
        sender: T::AccountId,
        liquidity_token_id: Self::CurrencyId,
        amount: Self::Balance,
    ) -> DispatchResult {
        let vault = Self::account_id();
        let mangata_id: TokenId = T::NativeCurrencyId::get();

        let current_block_number =
            <frame_system::Module<T>>::block_number().saturated_into::<u32>();
        let (rewards_total, rewards_claimed) = Module::<T>::calculate_rewards_amount(
            sender.clone(),
            liquidity_token_id,
            current_block_number,
        )?;

        let rewards_total_i128: i128 = i128::try_from(rewards_total)
            .map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
        let eligible_to_claim: Balance = u128::try_from(rewards_total_i128 - rewards_claimed)
            .map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
        ensure!(
            amount <= eligible_to_claim,
            Error::<T>::NotEligibleToClaimAmount,
        );

        let rewards_claimed_new = rewards_claimed
            + i128::try_from(amount).map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

        <T as Trait>::Currency::transfer(
            mangata_id.into(),
            &vault,
            &sender,
            amount.into(),
            ExistenceRequirement::KeepAlive,
        )?;

        LiquidityMiningUserClaimed::<T>::insert((sender, liquidity_token_id), rewards_claimed_new);

        //TODO REMOVE CLAIMED MANGATA FROM TOTAL POOL
        Ok(())
    }

    // This function has not been verified
    fn get_tokens_required_for_minting(
        liquidity_asset_id: Self::CurrencyId,
        liquidity_token_amount: Self::Balance,
    ) -> Result<
        (
            Self::CurrencyId,
            Self::Balance,
            Self::CurrencyId,
            Self::Balance,
        ),
        DispatchError,
    > {
        let (first_asset_id, second_asset_id) =
            LiquidityPools::get(liquidity_asset_id).ok_or(Error::<T>::NoSuchLiquidityAsset)?;
        let (first_asset_reserve, second_asset_reserve) =
            Module::<T>::get_reserves(first_asset_id, second_asset_id)?;
        let total_liquidity_assets: Balance =
            <T as Trait>::Currency::total_issuance(liquidity_asset_id.into()).into();

        ensure!(
            !total_liquidity_assets.is_zero(),
            Error::<T>::DivisionByZero
        );
        let second_asset_amount = multiply_by_rational(
            liquidity_token_amount,
            second_asset_reserve,
            total_liquidity_assets,
        )
        .map_err(|_| Error::<T>::UnexpectedFailure)?
        .checked_add(1)
        .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
        let first_asset_amount = multiply_by_rational(
            liquidity_token_amount,
            first_asset_reserve,
            total_liquidity_assets,
        )
        .map_err(|_| Error::<T>::UnexpectedFailure)?
        .checked_add(1)
        .ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

        log!(
            info,
            "get_tokens_required_for_minting: ({}, {}) -> ({}, {}, {}, {})",
            liquidity_asset_id,
            liquidity_token_amount,
            first_asset_id,
            first_asset_amount,
            second_asset_id,
            second_asset_amount,
        );

        Ok((
            first_asset_id,
            first_asset_amount,
            second_asset_id,
            second_asset_amount,
        ))
    }
}

pub trait Valuate {
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

    fn get_liquidity_token_mga_pool(
        liquidity_token_id: Self::CurrencyId,
    ) -> Result<(Self::CurrencyId, Self::CurrencyId), DispatchError>;

    fn valuate_liquidity_token(
        liquidity_token_id: Self::CurrencyId,
        liquidity_token_amount: Self::Balance,
    ) -> Self::Balance;

    fn scale_liquidity_by_mga_valuation(
        mga_valuation: Self::Balance,
        liquidity_token_amount: Self::Balance,
        mga_token_amount: Self::Balance,
    ) -> Self::Balance;
}

impl<T: Trait> Valuate for Module<T> {
    type Balance = Balance;

    type CurrencyId = TokenId;

    fn get_liquidity_token_mga_pool(
        liquidity_token_id: Self::CurrencyId,
    ) -> Result<(Self::CurrencyId, Self::CurrencyId), DispatchError> {
        let (first_token_id, second_token_id) =
            LiquidityPools::get(liquidity_token_id).ok_or(Error::<T>::NoSuchLiquidityAsset)?;
        let native_currency_id = T::NativeCurrencyId::get();
        match native_currency_id {
            _ if native_currency_id == first_token_id => Ok((first_token_id, second_token_id)),
            _ if native_currency_id == second_token_id => Ok((second_token_id, first_token_id)),
            _ => Err(Error::<T>::NotMangataLiquidityAsset.into()),
        }
    }

    fn valuate_liquidity_token(
        liquidity_token_id: Self::CurrencyId,
        liquidity_token_amount: Self::Balance,
    ) -> Self::Balance {
        let (mga_token_id, other_token_id) =
            match Self::get_liquidity_token_mga_pool(liquidity_token_id) {
                Ok(pool) => pool,
                Err(_) => return Default::default(),
            };

        let mga_token_reserve = match Module::<T>::get_reserves(mga_token_id, other_token_id) {
            Ok(reserves) => reserves.0,
            Err(_) => return Default::default(),
        };

        let liquidity_token_reserve: Balance =
            <T as Trait>::Currency::total_issuance(liquidity_token_id.into()).into();

        if liquidity_token_reserve.is_zero() {
            return Default::default();
        }

        multiply_by_rational(
            mga_token_reserve,
            liquidity_token_amount,
            liquidity_token_reserve,
        )
        .unwrap_or_else(|_| Balance::max_value())
    }

    fn scale_liquidity_by_mga_valuation(
        mga_valuation: Self::Balance,
        liquidity_token_amount: Self::Balance,
        mga_token_amount: Self::Balance,
    ) -> Self::Balance {
        if mga_valuation.is_zero() {
            return Default::default();
        }

        multiply_by_rational(liquidity_token_amount, mga_token_amount, mga_valuation)
            .unwrap_or_else(|_| Balance::max_value())
    }
}
