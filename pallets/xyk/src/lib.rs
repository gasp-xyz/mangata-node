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
use frame_support::sp_runtime::traits::AccountIdConversion;
use frame_support::traits::{ExistenceRequirement, WithdrawReasons};
use mangata_primitives::{Balance, TokenId};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended};
use sp_runtime::traits::{SaturatedConversion, Zero};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: MultiTokenCurrencyExtended<Self::AccountId>;
}

const PALLET_ID: ModuleId = ModuleId(*b"79b14c96");
// 1/100 %
const TREASURY_PERCENTAGE: u128 = 5; 
const BUYANDBURN_PERCENTAGE: u128 = 5; 
const SWAPFEE_PERCENTAGE: u128 = 30; 
const MANGATA_ID: u128 = 0; 

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

        Pools get(fn asset_pool): map hasher(opaque_blake2_256) (TokenId, TokenId) => Balance;

        LiquidityAssets get(fn liquidity_asset): map hasher(opaque_blake2_256) (TokenId, TokenId) => TokenId;
        LiquidityPools get(fn liquidity_pool): map hasher(opaque_blake2_256) TokenId => (TokenId, TokenId);
        Treasury get(fn treasury): map hasher(opaque_blake2_256) TokenId => Balance;
        TreasuryBurn get(fn treasury_burn): map hasher(opaque_blake2_256) TokenId => Balance;

        Nonce get (fn nonce): u32;

    }
}

// XYK extrinsics.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 10_000]
        fn create_pool(
            origin,
            first_asset_id: TokenId,
            first_asset_amount: Balance,
            second_asset_id: TokenId,
            second_asset_amount: Balance
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            let vault: T::AccountId  = Self::account_id();

            ensure!(
                !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            ensure!(
                !Pools::contains_key((first_asset_id, second_asset_id)),
                Error::<T>::PoolAlreadyExists,
            );

            ensure!(
                !Pools::contains_key((second_asset_id,first_asset_id)),
                Error::<T>::PoolAlreadyExists,
            );

            let first_asset_free_balance: Balance = T::Currency::free_balance(first_asset_id.into(), &sender).into();
            let second_asset_free_balance: Balance = T::Currency::free_balance(second_asset_id.into(), &sender).into();

            ensure!(
                first_asset_free_balance >= first_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                second_asset_free_balance >= second_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            T::Currency::ensure_can_withdraw(first_asset_id.into(), &sender, first_asset_amount.into(),
                WithdrawReasons::all()
                ,{first_asset_free_balance - first_asset_amount}.into() ).or(Err(Error::<T>::NotEnoughAssets))?;

            T::Currency::ensure_can_withdraw(second_asset_id.into(), &sender, second_asset_amount.into(),
                WithdrawReasons::all()
                ,{second_asset_free_balance - second_asset_amount}.into()).or(Err(Error::<T>::NotEnoughAssets))?;


            ensure!(
                first_asset_id != second_asset_id,
                Error::<T>::SameAsset,
            );

            let initial_liquidity = first_asset_amount + second_asset_amount;

            Pools::insert(
                (first_asset_id, second_asset_id), first_asset_amount
            );

            Pools::insert(
                (second_asset_id, first_asset_id), second_asset_amount
            );

            T::Currency::transfer(
                first_asset_id.into(),
                &sender,
                &vault,
                first_asset_amount.into(),
                ExistenceRequirement::AllowDeath
            )?;

            T::Currency::transfer(
                second_asset_id.into(),
                &sender,
                &vault,
                second_asset_amount.into(),
                ExistenceRequirement::AllowDeath
            )?;

            let liquidity_asset_id: TokenId = T::Currency::create(&sender, initial_liquidity.into()).into();

            LiquidityAssets::insert((first_asset_id, second_asset_id), liquidity_asset_id);
            LiquidityPools::insert(liquidity_asset_id, (first_asset_id, second_asset_id));


            Self::deposit_event(RawEvent::PoolCreated(sender, first_asset_id, first_asset_amount, second_asset_id, second_asset_amount));

            Ok(())

        }

        // you will sell your sold_asset_amount of sold_asset_id to get some amount of bought_asset_id
        #[weight = (10_000, Pays::No)]
        fn sell_asset (
            origin,
            sold_asset_id: TokenId,
            bought_asset_id: TokenId,
            sold_asset_amount: Balance,
            min_amount_out: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            ensure!(
                Pools::contains_key((sold_asset_id,bought_asset_id)),
                Error::<T>::NoSuchPool,
            );

            ensure!(
                !sold_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            let input_reserve = Pools::get((sold_asset_id, bought_asset_id));
            let output_reserve = Pools::get((bought_asset_id, sold_asset_id));
            let bought_asset_amount = Self::calculate_sell_price(
                input_reserve,
                output_reserve,
                sold_asset_amount,
            )?;

            ensure!(
                T::Currency::free_balance(sold_asset_id.into(), &sender).into() >= sold_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                bought_asset_amount >= min_amount_out,
                Error::<T>::InsufficientOutputAmount,
            );

            let vault = Self::account_id();

            T::Currency::transfer(
                sold_asset_id.into(),
                &sender,
                &vault,
                sold_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;
            T::Currency::transfer(
                bought_asset_id.into(),
                &vault,
                &sender,
                bought_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;

            Pools::insert(
                (sold_asset_id, bought_asset_id),
                input_reserve + sold_asset_amount,
            );
            Pools::insert(
                (bought_asset_id, sold_asset_id),
                output_reserve - bought_asset_amount,
            );

            Self::deposit_event(RawEvent::AssetsSwapped(sender,sold_asset_id, sold_asset_amount, bought_asset_id, bought_asset_amount));

            Ok(())
        }

        #[weight = (10_000, Pays::No)]
        fn buy_asset (
            origin,
            sold_asset_id: TokenId,
            bought_asset_id: TokenId,
            bought_asset_amount: Balance,
            max_amount_in: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            ensure!(
                Pools::contains_key((sold_asset_id,bought_asset_id)),
                Error::<T>::NoSuchPool,
            );

            let input_reserve = Pools::get((sold_asset_id, bought_asset_id));
            let output_reserve = Pools::get((bought_asset_id, sold_asset_id));

            ensure!(
                output_reserve > bought_asset_amount,
                Error::<T>::NotEnoughReserve,
            );

            ensure!(
                !bought_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            let sold_asset_amount = Self::calculate_buy_price(
                input_reserve,
                output_reserve,
                bought_asset_amount,
            );

            ensure!(
                T::Currency::free_balance(sold_asset_id.into(), &sender).into() >= sold_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                sold_asset_amount <= max_amount_in,
                Error::<T>::InsufficientInputAmount,
            );

            let vault = Self::account_id();

            T::Currency::transfer(
                sold_asset_id.into(),
                &sender,
                &vault,
                sold_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;
            T::Currency::transfer(
                bought_asset_id.into(),
                &vault,
                &sender,
                bought_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;

            Pools::insert(
                (sold_asset_id, bought_asset_id),
                input_reserve + sold_asset_amount,
            );
            Pools::insert(
                (bought_asset_id, sold_asset_id),
                output_reserve - bought_asset_amount,
            );

            Self::deposit_event(RawEvent::AssetsSwapped(sender,sold_asset_id, sold_asset_amount, bought_asset_id, bought_asset_amount));

            Ok(())
        }

        #[weight = 10_000]
        fn mint_liquidity (
            origin,
            first_asset_id: TokenId,
            second_asset_id: TokenId,
            first_asset_amount: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            let vault = Self::account_id();

            ensure!(
                (LiquidityAssets::contains_key((first_asset_id, second_asset_id)) || LiquidityAssets::contains_key((second_asset_id, first_asset_id))),
                Error::<T>::NoSuchPool,
            );

            let liquidity_asset_id = Self::get_liquidity_asset(
                 first_asset_id,
                 second_asset_id
            );

            ensure!(
                (Pools::contains_key((first_asset_id, second_asset_id)) || Pools::contains_key((second_asset_id, first_asset_id))),
                Error::<T>::NoSuchPool,
            );

            let first_asset_reserve = Pools::get((first_asset_id, second_asset_id));
            let second_asset_reserve = Pools::get((second_asset_id, first_asset_id));
            let total_liquidity_assets: Balance = T::Currency::total_issuance(liquidity_asset_id.into()).into();

            let first_asset_amount_u256: U256 = first_asset_amount.saturated_into::<u128>().into();
            let first_asset_reserve_u256: U256 = first_asset_reserve.saturated_into::<u128>().into();
            let second_asset_reserve_u256: U256 = second_asset_reserve.saturated_into::<u128>().into();
            let total_liquidity_assets_u256: U256 = total_liquidity_assets.saturated_into::<u128>().into();

            let second_asset_amount_u256: U256 = first_asset_amount_u256 * second_asset_reserve_u256 / first_asset_reserve_u256 + 1;
            let liquidity_assets_minted_u256: U256 = first_asset_amount_u256 * total_liquidity_assets_u256 / first_asset_reserve_u256;

            let second_asset_amount = second_asset_amount_u256.saturated_into::<u128>()
                .saturated_into::<Balance>();
            let liquidity_assets_minted = liquidity_assets_minted_u256.saturated_into::<u128>()
                .saturated_into::<Balance>();

            ensure!(
                !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            ensure!(
                T::Currency::free_balance(first_asset_id.into(), &sender).into() >= first_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                T::Currency::free_balance(second_asset_id.into(), &sender).into() >= second_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            T::Currency::transfer(
                first_asset_id.into(),
                &sender,
                &vault,
                first_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;
            T::Currency::transfer(
                second_asset_id.into(),
                &sender,
                &vault,
                second_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;

            T::Currency::mint(liquidity_asset_id.into(),&sender, liquidity_assets_minted.into())?;

            Pools::insert(
                (&first_asset_id, &second_asset_id),
                first_asset_reserve + first_asset_amount,
            );
            Pools::insert(
                (&second_asset_id, &first_asset_id),
                second_asset_reserve + second_asset_amount,
            );


           Self::deposit_event(RawEvent::LiquidityMinted(sender,first_asset_id, first_asset_amount, second_asset_id, second_asset_amount,liquidity_asset_id, second_asset_amount));

            Ok(())
        }

        #[weight = 10_000]
        fn burn_liquidity (
            origin,
            first_asset_id: TokenId,
            second_asset_id: TokenId,
            liquidity_asset_amount: Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            let vault = Self::account_id();

            ensure!(
                Pools::contains_key((first_asset_id, second_asset_id)),
                Error::<T>::NoSuchPool,
            );

            let first_asset_reserve = Pools::get((first_asset_id, second_asset_id));
            let second_asset_reserve = Pools::get((second_asset_id, first_asset_id));
            let liquidity_asset_id = Self::get_liquidity_asset(first_asset_id, second_asset_id);

            ensure!(
                T::Currency::can_slash(liquidity_asset_id.into(), &sender, liquidity_asset_amount.into()),
                Error::<T>::NotEnoughAssets,
            );
            let new_balance: Balance = T::Currency::free_balance(liquidity_asset_id.into(), &sender).into() - liquidity_asset_amount;

            T::Currency::ensure_can_withdraw(liquidity_asset_id.into(),
                &sender,
                liquidity_asset_amount.into(),
                WithdrawReasons::all(),
                new_balance.into()).or(Err(Error::<T>::NotEnoughAssets))?;

            let (first_asset_amount, second_asset_amount) =  Self::get_burn_amount(first_asset_id, second_asset_id, liquidity_asset_amount);

            ensure!(
                !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            T::Currency::transfer(
                first_asset_id.into(),
                &vault,
                &sender,
                first_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;
            T::Currency::transfer(
                second_asset_id.into(),
                &vault,
                &sender,
                second_asset_amount.into(),
                ExistenceRequirement::KeepAlive,
            )?;
            Pools::insert(
                (&first_asset_id, &second_asset_id),
                first_asset_reserve - first_asset_amount,
            );
            Pools::insert(
                (&second_asset_id, &first_asset_id),
                second_asset_reserve - second_asset_amount,
            );

            if (first_asset_reserve - first_asset_amount == 0.saturated_into::<Balance>())
                || (second_asset_reserve - second_asset_amount == 0.saturated_into::<Balance>()) {
                Pools::remove((first_asset_id, second_asset_id));
                Pools::remove((second_asset_id, first_asset_id));
            }

            T::Currency::burn_and_settle(liquidity_asset_id.into(), &sender, liquidity_asset_amount.into())?;

            Self::deposit_event(RawEvent::LiquidityBurned(sender, first_asset_id, first_asset_amount, second_asset_id, second_asset_amount,liquidity_asset_id, second_asset_amount));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn calculate_sell_price(
        input_reserve: Balance,
        output_reserve: Balance,
        sell_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let input_reserve_saturated: U256 = input_reserve.saturated_into::<u128>().into();
        let output_reserve_saturated: U256 = output_reserve.saturated_into::<u128>().into();
        let sell_amount_saturated: U256 = sell_amount.saturated_into::<u128>().into();

        let input_amount_with_fee: U256 = sell_amount_saturated * 997;
        let numerator: U256 = input_amount_with_fee * output_reserve_saturated;
        let denominator: U256 = input_reserve_saturated * 1000 + input_amount_with_fee;
        let result = numerator
            .checked_div(denominator)
            .ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;
        Ok(result.saturated_into::<u128>().saturated_into::<Balance>())
    }

    pub fn calculate_sell_price_no_fee(
        input_reserve: Balance,
        output_reserve: Balance,
        sell_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let input_reserve_saturated: U256 = input_reserve.saturated_into::<u128>().into();
        let output_reserve_saturated: U256 = output_reserve.saturated_into::<u128>().into();
        let sell_amount_saturated: U256 = sell_amount.saturated_into::<u128>().into();

        let numerator: U256 = sell_amount_saturated * output_reserve_saturated;
        let denominator: U256 = input_reserve_saturated + sell_amount_saturated;
        let result = numerator
            .checked_div(denominator)
            .ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;
        Ok(result.saturated_into::<u128>().saturated_into::<Balance>())
    }

    pub fn calculate_buy_price(
        input_reserve: Balance,
        output_reserve: Balance,
        buy_amount: Balance,
    ) -> Balance {
        let input_reserve_saturated: U256 = input_reserve.saturated_into::<u128>().into();
        let output_reserve_saturated: U256 = output_reserve.saturated_into::<u128>().into();
        let buy_amount_saturated: U256 = buy_amount.saturated_into::<u128>().into();

        let numerator: U256 = input_reserve_saturated * buy_amount_saturated * 1000;
        let denominator: U256 = (output_reserve_saturated - buy_amount_saturated) * 997;
        let result: U256 = numerator / denominator + 1;

        result.saturated_into::<u128>().saturated_into::<Balance>()
    }

    pub fn get_liquidity_asset(first_asset_id: TokenId, second_asset_id: TokenId) -> TokenId {
        if LiquidityAssets::contains_key((first_asset_id, second_asset_id)) {
            LiquidityAssets::get((first_asset_id, second_asset_id))
        } else {
            LiquidityAssets::get((second_asset_id, first_asset_id))
        }
    }

    pub fn get_burn_amount(
        first_asset_id: TokenId,
        second_asset_id: TokenId,
        liquidity_asset_amount: Balance,
    ) -> (Balance, Balance) {
        let liquidity_asset_id = Self::get_liquidity_asset(first_asset_id, second_asset_id);
        let first_asset_reserve_u256: U256 = Pools::get((first_asset_id, second_asset_id))
            .saturated_into::<u128>()
            .into();
        let second_asset_reserve_u256: U256 = Pools::get((second_asset_id, first_asset_id))
            .saturated_into::<u128>()
            .into();
        let total_liquidity_assets_u256: U256 =
            T::Currency::total_issuance(liquidity_asset_id.into())
                .saturated_into::<u128>()
                .into();
        let liquidity_asset_amount_u256: U256 =
            liquidity_asset_amount.saturated_into::<u128>().into();

        let first_asset_amount_u256 =
            first_asset_reserve_u256 * liquidity_asset_amount_u256 / total_liquidity_assets_u256;
        let second_asset_amount_u256 =
            second_asset_reserve_u256 * liquidity_asset_amount_u256 / total_liquidity_assets_u256;
        let second_asset_amount = second_asset_amount_u256
            .saturated_into::<u128>()
            .saturated_into::<Balance>();
        let first_asset_amount = first_asset_amount_u256
            .saturated_into::<u128>()
            .saturated_into::<Balance>();

        (first_asset_amount, second_asset_amount)
    }

    fn settle_treasury_and_burn( 
        sold_asset_id: TokenId,
        bought_asset_id: TokenId,
        sold_asset_amount: Balance,) -> DispatchResult
        {
    
        let vault = Self::account_id();
        let mangata_id = MANGATA_ID.saturated_into::<TokenId>();
        let input_reserve = Pools::get((sold_asset_id, bought_asset_id));
        let output_reserve = Pools::get((bought_asset_id, sold_asset_id));
    
        let mut settling_asset_id = bought_asset_id;
        let mut treasury_amount = sold_asset_amount * TREASURY_PERCENTAGE.saturated_into::<Balance>() / 10000.saturated_into::<Balance>();
        let mut burn_amount = sold_asset_amount * BUYANDBURN_PERCENTAGE.saturated_into::<Balance>() / 10000.saturated_into::<Balance>();
        
        //Check whether to settle treasury and buyburn with sold or bought asset.
        //By default we are using bought, only in case if sold is directly mangata, or is in pair with mangata and bought is not
        if sold_asset_id == mangata_id  || (Pools::contains_key((sold_asset_id,mangata_id)) && !Pools::contains_key((bought_asset_id,mangata_id))){
            settling_asset_id = sold_asset_id;
    
            Pools::insert(
                (&sold_asset_id, &bought_asset_id),
                input_reserve - burn_amount - treasury_amount,
            );
        }
        //sold amount recalculated to bought asset amount 
        else {
            treasury_amount = treasury_amount * output_reserve / input_reserve; 
            burn_amount = burn_amount * output_reserve / input_reserve; 
    
            Pools::insert(
                (&bought_asset_id, &sold_asset_id),
                output_reserve - treasury_amount - burn_amount,
            );
        }
    
        if settling_asset_id == mangata_id {
    
            Treasury::insert(
                mangata_id,
                Treasury::get(mangata_id) + treasury_amount
            );
            
            T::Currency::burn_and_settle(mangata_id.into(), &vault, burn_amount.into())?;
        }
    
        //swap settling asset to mangata
        else if Pools::contains_key((settling_asset_id,mangata_id)){
            let input_reserve = Pools::get((settling_asset_id, mangata_id));
            let output_reserve = Pools::get((mangata_id, settling_asset_id));
    
            let treasury_amount_in_mangata = Self::calculate_sell_price_no_fee(
                input_reserve,
                output_reserve,
                treasury_amount,
            )?;
            let burn_amount_in_mangata = Self::calculate_sell_price_no_fee(
                input_reserve,
                output_reserve,
                burn_amount,
            )?;
    
            Pools::insert(
                (settling_asset_id, mangata_id),
                input_reserve + treasury_amount + burn_amount,
            );    
            Pools::insert(
                (mangata_id, settling_asset_id),
                output_reserve - treasury_amount_in_mangata - burn_amount_in_mangata,
            );
            Treasury::insert(
                mangata_id,
                Treasury::get(mangata_id) + treasury_amount_in_mangata
            );

            T::Currency::burn_and_settle(mangata_id.into(), &vault, burn_amount.into())?;
        }
        // if settling token has no mangata connection, settling token is added to treasuries
        else {
            Treasury::insert(
                settling_asset_id,
                Treasury::get(settling_asset_id) + treasury_amount
            );
            TreasuryBurn::insert(
                settling_asset_id,
                TreasuryBurn::get(settling_asset_id) + burn_amount
            );
        }
        Ok(())
    }
    
    fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
}
