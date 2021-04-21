#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    sp_runtime::ModuleId, weights::Pays, StorageMap
};
use frame_system::ensure_signed;
use pallet_assets as assets;
use log::{info};
use sp_core::U256;
// TODO documentation!
use frame_support::sp_runtime::traits::AccountIdConversion;
use sp_runtime::print;
use sp_runtime::traits::{BlakeTwo256, Hash, One, SaturatedConversion, Zero};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait Trait: assets::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    
}

const PALLET_ID: ModuleId = ModuleId(*b"79b14c96");
// 1/100 %
const TREASURY_PERCENTAGE: u128 = 5; 
const BUYANDBURN_PERCENTAGE:  u128 = 5; 
const SWAPFEE_PERCENTAGE:  u128 = 30; 
const MANGATA_ID:  u128 = 0; 

// const MANGATA_ID: frame_system::AssetId = 0;

decl_error! {
    /// Errors
    pub enum Error for Module<T: Trait> {
        VaultAlreadySet,
        PoolAlreadyExists,
        NotEnoughAssets,
        NoSuchPool,
        NotEnoughReserve,
        ZeroAmount,
        InsufficientInputAmount,
        InsufficientOutputAmount,
        SameAsset,
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = <T as assets::Trait>::Balance,
        AssetId = <T as assets::Trait>::AssetId,
    {
        //TODO add trading events
        PoolCreated(AccountId, AssetId, Balance, AssetId, Balance),
        AssetsSwapped(AccountId, AssetId, Balance, AssetId, Balance),
        LiquidityMinted(
            AccountId,
            AssetId,
            Balance,
            AssetId,
            Balance,
            AssetId,
            Balance,
        ),
        LiquidityBurned(
            AccountId,
            AssetId,
            Balance,
            AssetId,
            Balance,
            AssetId,
            Balance,
        ),
        // LiquidityMinted(AccountId, AssetId, Balance, AssetId, Balance),
        // LiquidityBurned(AccountId, AssetId, Balance, AssetId, Balance),
        // LiquidityAssetsGained(AccountId, AssetId, Balance),
        // LiquidityAssetsBurned(AccountId, AssetId, Balance),
    }
);

// XYK exchange pallet storage.
decl_storage! {
    trait Store for Module<T: Trait> as XykStorage {

        Pools get(fn asset_pool): map hasher(opaque_blake2_256) (T::AssetId, T::AssetId) => T::Balance;

        LiquidityAssets get(fn liquidity_asset): map hasher(opaque_blake2_256) (T::AssetId, T::AssetId) => T::AssetId;

        LiquidityPools get(fn liquidity_pool): map hasher(opaque_blake2_256) T::AssetId => (T::AssetId, T::AssetId);

        Treasury get(fn treasury): map hasher(opaque_blake2_256) T::AssetId => T::Balance;

        TreasuryBurn get(fn treasury_burn): map hasher(opaque_blake2_256) T::AssetId => T::Balance;

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
            first_asset_id: T::AssetId,
            first_asset_amount: T::Balance,
            second_asset_id: T::AssetId,
            second_asset_amount: T::Balance
        ) -> DispatchResult {

            let sender = ensure_signed(origin.clone())?;
            let vault: T::AccountId  = Self::account_id();


            ensure!(
                !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            ensure!(
                !<Pools<T>>::contains_key((first_asset_id, second_asset_id)),
                Error::<T>::PoolAlreadyExists,
            );

            ensure!(
                !<Pools<T>>::contains_key((second_asset_id,first_asset_id)),
                Error::<T>::PoolAlreadyExists,
            );

            ensure!(
                <assets::Module<T>>::balance(first_asset_id, sender.clone()) >= first_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                <assets::Module<T>>::balance(second_asset_id, sender.clone()) >= second_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                first_asset_id != second_asset_id,
                Error::<T>::SameAsset,
            );

            <Pools<T>>::insert(
                (first_asset_id, second_asset_id), first_asset_amount
            );

            <Pools<T>>::insert(
                (second_asset_id, first_asset_id), second_asset_amount
            );

            let liquidity_asset_id = <assets::Module<T>>::assets_next_asset_id();

            <LiquidityAssets<T>>::insert((first_asset_id, second_asset_id), liquidity_asset_id);
            <LiquidityPools<T>>::insert(liquidity_asset_id, (first_asset_id, second_asset_id));

            let initial_liquidity = first_asset_amount + second_asset_amount;
            <assets::Module<T>>::assets_issue(
                &sender,
                &initial_liquidity
            );

            <assets::Module<T>>::assets_transfer(
                &first_asset_id,
                &sender,
                &vault,
                &first_asset_amount
            )?;
            <assets::Module<T>>::assets_transfer(
                &second_asset_id,
                &sender,
                &vault,
                &second_asset_amount
            )?;

            Self::deposit_event(RawEvent::PoolCreated(sender, first_asset_id, first_asset_amount, second_asset_id, second_asset_amount));

            Ok(())
        }

        // you will sell your sold_asset_amount of sold_asset_id to get some amount of bought_asset_id
        #[weight = (10_000, Pays::No)]
        fn sell_asset (
            origin,
            sold_asset_id: T::AssetId,
            bought_asset_id: T::AssetId,
            sold_asset_amount: T::Balance,
            min_amount_out: T::Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            let mangata_id = MANGATA_ID.saturated_into::<T::AssetId>();

            ensure!(
                <Pools<T>>::contains_key((sold_asset_id,bought_asset_id)),
                Error::<T>::NoSuchPool,
            );

            ensure!(
                !sold_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            let input_reserve = <Pools<T>>::get((sold_asset_id, bought_asset_id));
            let output_reserve = <Pools<T>>::get((bought_asset_id, sold_asset_id));
            let bought_asset_amount = Self::calculate_sell_price(
                input_reserve,
                output_reserve,
                sold_asset_amount,
            );

            ensure!(
                <assets::Module<T>>::balance(sold_asset_id, sender.clone()) >= sold_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                bought_asset_amount >= min_amount_out,
                Error::<T>::InsufficientOutputAmount,
            );


           

            let vault = Self::account_id();

            <assets::Module<T>>::assets_transfer(
                &sold_asset_id,
                &sender,
                &vault,
                &sold_asset_amount,
            )?;
            <assets::Module<T>>::assets_transfer(
                &bought_asset_id,
                &vault,
                &sender,
                &bought_asset_amount,
            )?;

            <Pools<T>>::insert(
                (sold_asset_id, bought_asset_id),
                input_reserve + sold_asset_amount,
            );
            <Pools<T>>::insert(
                (bought_asset_id, sold_asset_id),
                output_reserve - bought_asset_amount,
            );

            Self::settle_treasury_and_burn(
                sold_asset_id,
                bought_asset_id,
                sold_asset_amount);


            Self::deposit_event(RawEvent::AssetsSwapped(sender,sold_asset_id, sold_asset_amount, bought_asset_id, bought_asset_amount));

            Ok(())
        }

        #[weight = (10_000, Pays::No)]
        fn buy_asset (
            origin,
            sold_asset_id: T::AssetId,
            bought_asset_id: T::AssetId,
            bought_asset_amount: T::Balance,
            max_amount_in: T::Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            ensure!(
                <Pools<T>>::contains_key((sold_asset_id,bought_asset_id)),
                Error::<T>::NoSuchPool,
            );

            let input_reserve = <Pools<T>>::get((sold_asset_id, bought_asset_id));
            let output_reserve = <Pools<T>>::get((bought_asset_id, sold_asset_id));

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
                <assets::Module<T>>::balance(sold_asset_id, sender.clone()) >= sold_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                sold_asset_amount <= max_amount_in,
                Error::<T>::InsufficientInputAmount,
            );

            let vault = Self::account_id();

            <assets::Module<T>>::assets_transfer(
                &sold_asset_id,
                &sender,
                &vault,
                &sold_asset_amount,
            )?;
            <assets::Module<T>>::assets_transfer(
                &bought_asset_id,
                &vault,
                &sender,
                &bought_asset_amount,
            )?;

            <Pools<T>>::insert(
                (sold_asset_id, bought_asset_id),
                input_reserve + sold_asset_amount,
            );
            <Pools<T>>::insert(
                (bought_asset_id, sold_asset_id),
                output_reserve - bought_asset_amount,
            );

            Self::deposit_event(RawEvent::AssetsSwapped(sender,sold_asset_id, sold_asset_amount, bought_asset_id, bought_asset_amount));

            Ok(())
        }

        #[weight = 10_000]
        fn mint_liquidity (
            origin,
            first_asset_id: T::AssetId,
            second_asset_id: T::AssetId,
            first_asset_amount: T::Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            let vault = Self::account_id();

            //get liquidity_asset_id of selected pool
            let liquidity_asset_id = Self::get_liquidity_asset(
                 first_asset_id,
                 second_asset_id
            );

            ensure!(
                (<Pools<T>>::contains_key((first_asset_id, second_asset_id)) || <Pools<T>>::contains_key((second_asset_id, first_asset_id))),
                Error::<T>::NoSuchPool,
            );

            let first_asset_reserve = <Pools<T>>::get((first_asset_id, second_asset_id));
            let second_asset_reserve = <Pools<T>>::get((second_asset_id, first_asset_id));
            let total_liquidity_assets = <assets::Module<T>>::total_supply(liquidity_asset_id);

            let first_asset_amount_u256: U256 = first_asset_amount.saturated_into::<u128>().into();
            let first_asset_reserve_u256: U256 = first_asset_reserve.saturated_into::<u128>().into();
            let second_asset_reserve_u256: U256 = second_asset_reserve.saturated_into::<u128>().into();
            let total_liquidity_assets_u256: U256 = total_liquidity_assets.saturated_into::<u128>().into();

            let second_asset_amount_u256: U256 = first_asset_amount_u256 * second_asset_reserve_u256 / first_asset_reserve_u256 + 1;
            let liquidity_assets_minted_u256: U256 = first_asset_amount_u256 * total_liquidity_assets_u256 / first_asset_reserve_u256;

            let second_asset_amount = second_asset_amount_u256.saturated_into::<u128>()
                .saturated_into::<T::Balance>();
            let liquidity_assets_minted = liquidity_assets_minted_u256.saturated_into::<u128>()
                .saturated_into::<T::Balance>();

            ensure!(
                !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            ensure!(
                <assets::Module<T>>::balance(first_asset_id, sender.clone()) >= first_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            ensure!(
                <assets::Module<T>>::balance(second_asset_id, sender.clone()) >= second_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            <assets::Module<T>>::assets_transfer(
                &first_asset_id,
                &sender,
                &vault,
                &first_asset_amount,
            )?;
            <assets::Module<T>>::assets_transfer(
                &second_asset_id,
                &sender,
                &vault,
                &second_asset_amount,
            )?;

            <Pools<T>>::insert(
                (&first_asset_id, &second_asset_id),
                first_asset_reserve + first_asset_amount,
            );
            <Pools<T>>::insert(
                (&second_asset_id, &first_asset_id),
                second_asset_reserve + second_asset_amount,
            );

            <assets::Module<T>>::assets_mint(
                &liquidity_asset_id,
                &sender,
                &liquidity_assets_minted
            );

            Self::deposit_event(RawEvent::LiquidityMinted(sender.clone(),first_asset_id, first_asset_amount, second_asset_id, second_asset_amount,liquidity_asset_id, second_asset_amount));
          //  Self::deposit_event(RawEvent::LiquidityAssetsGained(sender.clone(),liquidity_asset_id, second_asset_amount));

            Ok(())
        }

        #[weight = 10_000]
        fn burn_liquidity (
            origin,
            first_asset_id: T::AssetId,
            second_asset_id: T::AssetId,
            first_asset_amount: T::Balance,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            let vault = Self::account_id();

            ensure!(
                <Pools<T>>::contains_key((first_asset_id, second_asset_id)),
                Error::<T>::NoSuchPool,
            );

            let first_asset_reserve = <Pools<T>>::get((first_asset_id, second_asset_id));
            let second_asset_reserve = <Pools<T>>::get((second_asset_id, first_asset_id));
            let liquidity_asset_id = Self::get_liquidity_asset(first_asset_id, second_asset_id);
            let total_liquidity_assets = <assets::Module<T>>::total_supply(liquidity_asset_id);

            let first_asset_amount_u256: U256 = first_asset_amount.saturated_into::<u128>().into();
            let first_asset_reserve_u256: U256 = first_asset_reserve.saturated_into::<u128>().into();
            let second_asset_reserve_u256: U256 = second_asset_reserve.saturated_into::<u128>().into();
            let total_liquidity_assets_u256: U256 = total_liquidity_assets.saturated_into::<u128>().into();
            //get liquidity_asset_amount corresponding to first_asset_amount
            let liquidity_asset_amount_u256: U256 = first_asset_amount_u256 * total_liquidity_assets_u256 / first_asset_reserve_u256;

            let liquidity_asset_amount = liquidity_asset_amount_u256.saturated_into::<u128>()
                .saturated_into::<T::Balance>();

            ensure!(
                <assets::Module<T>>::balance(liquidity_asset_id, sender.clone()) >= liquidity_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            let second_asset_amount_u256 = second_asset_reserve_u256 * liquidity_asset_amount_u256 / total_liquidity_assets_u256;
            let second_asset_amount = second_asset_amount_u256.saturated_into::<u128>()
                .saturated_into::<T::Balance>();

            ensure!(
                !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );

            <assets::Module<T>>::assets_transfer(
                &first_asset_id,
                &vault,
                &sender,
                &first_asset_amount,
            )?;
            <assets::Module<T>>::assets_transfer(
                &second_asset_id,
                &vault,
                &sender,
                &second_asset_amount,
            )?;

            <Pools<T>>::insert(
                (&first_asset_id, &second_asset_id),
                first_asset_reserve - first_asset_amount,
            );
            <Pools<T>>::insert(
                (&second_asset_id, &first_asset_id),
                second_asset_reserve - second_asset_amount,
            );

            if (first_asset_reserve - first_asset_amount == 0.saturated_into::<T::Balance>())
                || (second_asset_reserve - second_asset_amount == 0.saturated_into::<T::Balance>()) {
                <Pools<T>>::remove((first_asset_id, second_asset_id));
                <Pools<T>>::remove((second_asset_id, first_asset_id));
            }

            <assets::Module<T>>::assets_burn(&liquidity_asset_id, &sender, &liquidity_asset_amount);

            Self::deposit_event(RawEvent::LiquidityBurned(sender.clone(),first_asset_id, first_asset_amount, second_asset_id, second_asset_amount,liquidity_asset_id, second_asset_amount));
           // Self::deposit_event(RawEvent::LiquidityAssetsBurned(sender.clone(),liquidity_asset_id, second_asset_amount));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn calculate_sell_price(
        input_reserve: T::Balance,
        output_reserve: T::Balance,
        sell_amount: T::Balance,
    ) -> T::Balance {
        let input_reserve_saturated: U256 = input_reserve.saturated_into::<u128>().into();
        let output_reserve_saturated: U256 = output_reserve.saturated_into::<u128>().into();
        let sell_amount_saturated: U256 = sell_amount.saturated_into::<u128>().into();

        let input_amount_with_fee: U256 = sell_amount_saturated * 997;
        let numerator: U256 = input_amount_with_fee * output_reserve_saturated;
        let denominator: U256 = input_reserve_saturated * 1000 + input_amount_with_fee;
        let result: U256 = numerator / denominator;

        return result
            .saturated_into::<u128>()
            .saturated_into::<T::Balance>();
    }

    pub fn calculate_sell_price_no_fee(
        input_reserve: T::Balance,
        output_reserve: T::Balance,
        sell_amount: T::Balance,
    ) -> T::Balance {
        let input_reserve_saturated: U256 = input_reserve.saturated_into::<u128>().into();
        let output_reserve_saturated: U256 = output_reserve.saturated_into::<u128>().into();
        let sell_amount_saturated: U256 = sell_amount.saturated_into::<u128>().into();

        let numerator: U256 = sell_amount_saturated * output_reserve_saturated;
        let denominator: U256 = input_reserve_saturated + sell_amount_saturated;
        let result: U256 = numerator / denominator;

        return result
            .saturated_into::<u128>()
            .saturated_into::<T::Balance>();
    }

    pub fn calculate_buy_price(
        input_reserve: T::Balance,
        output_reserve: T::Balance,
        buy_amount: T::Balance,
    ) -> T::Balance {
        let input_reserve_saturated: U256 = input_reserve.saturated_into::<u128>().into();
        let output_reserve_saturated: U256 = output_reserve.saturated_into::<u128>().into();
        let buy_amount_saturated: U256 = buy_amount.saturated_into::<u128>().into();

        let numerator: U256 = input_reserve_saturated * buy_amount_saturated * 1000;
        let denominator: U256 = (output_reserve_saturated - buy_amount_saturated) * 997;
        let result: U256 = numerator / denominator + 1;

        return result
            .saturated_into::<u128>()
            .saturated_into::<T::Balance>();
    }

    pub fn get_liquidity_asset(
        first_asset_id: T::AssetId,
        second_asset_id: T::AssetId,
    ) -> T::AssetId {
        if <LiquidityAssets<T>>::contains_key((first_asset_id, second_asset_id)) {
            return <LiquidityAssets<T>>::get((first_asset_id, second_asset_id));
        } else {
            return <LiquidityAssets<T>>::get((second_asset_id, first_asset_id));
        }
    }

    // TODO vault takeout treasury fn
    // TODO parametrize %, mangata id
    // TODO treasury as pallet
    // TODO burn Y not X 
    // TODO burn X if X is mangata
    fn settle_tresury_and_burn(settling_asset_id: T::AssetId, settling_asset_amount: T::Balance) {
        let mangata_id = MANGATA_ID.saturated_into::<T::AssetId>();
        let vault = Self::account_id();
        //parametrize %, mangata id
        let burn_percentage = 5.saturated_into::<T::Balance>();
        let treasury_percentage = 5.saturated_into::<T::Balance>();
        let mut burn_amount = settling_asset_amount * burn_percentage / 10.saturated_into::<T::Balance>();
        let mut treasury_amount = settling_asset_amount * treasury_percentage / 10.saturated_into::<T::Balance>();

        if settling_asset_id == mangata_id {
           

            <Treasury<T>>::insert(
                mangata_id,
                <Treasury<T>>::get(mangata_id) + treasury_amount
            );
            
            <assets::Module<T>>::assets_burn(&mangata_id, &vault, &burn_amount);
        }
       
        else if <Pools<T>>::contains_key((settling_asset_id,mangata_id)){

            //TODO takeout to separate fn
            let input_reserve = <Pools<T>>::get((settling_asset_id, mangata_id));
            let output_reserve = <Pools<T>>::get((mangata_id, settling_asset_id));
           
            //todo, bez slipu amount
            let mangata_amount = Self::calculate_sell_price(
                input_reserve,
                output_reserve,
                settling_asset_amount,
            );
            burn_amount = mangata_amount * burn_percentage / 10.saturated_into::<T::Balance>();
            treasury_amount = mangata_amount * treasury_percentage / 10.saturated_into::<T::Balance>();

            <Pools<T>>::insert(
                (settling_asset_id, mangata_id),
                input_reserve + settling_asset_amount,
            );
            <Pools<T>>::insert(
                (mangata_id, settling_asset_id),
                output_reserve - mangata_amount,
            );

            <Treasury<T>>::insert(
                mangata_id,
                <Treasury<T>>::get(mangata_id) + treasury_amount
            );
            <assets::Module<T>>::assets_burn(&mangata_id, &vault, &burn_amount);

        }
        else {
            <Treasury<T>>::insert(
                settling_asset_id,
                <Treasury<T>>::get(settling_asset_id) + treasury_amount
            );
            <TreasuryBurn<T>>::insert(
                settling_asset_id,
                <TreasuryBurn<T>>::get(settling_asset_id) + burn_amount
            );
        }


    }

//TODO U256 ??    
fn settle_treasury_and_burn( 
    sold_asset_id: T::AssetId,
    bought_asset_id: T::AssetId,
    sold_asset_amount: T::Balance,) {

    let mangata_id = MANGATA_ID.saturated_into::<T::AssetId>();
    let input_reserve = <Pools<T>>::get((sold_asset_id, bought_asset_id));
    let output_reserve = <Pools<T>>::get((bought_asset_id, sold_asset_id));


    let mut settling_asset_id = bought_asset_id;
    let mut treasury_amount = sold_asset_amount * TREASURY_PERCENTAGE.saturated_into::<T::Balance>() / 10000.saturated_into::<T::Balance>();
    let mut burn_amount = sold_asset_amount * BUYANDBURN_PERCENTAGE.saturated_into::<T::Balance>() / 10000.saturated_into::<T::Balance>();
    
    //Check whether to settle treasury and buyburn with sold or bought asset.
    //By default we are using bought, only in case if sold is directly mangata, or is in pair with mangata and bought is not
    if sold_asset_id == mangata_id  || (<Pools<T>>::contains_key((sold_asset_id,mangata_id)) && !<Pools<T>>::contains_key((bought_asset_id,mangata_id))){
        settling_asset_id = sold_asset_id;

        <Pools<T>>::insert(
            (&sold_asset_id, &bought_asset_id),
            input_reserve - burn_amount - treasury_amount,
        );
    }
    //sold amount recalculated to bought asset amount 
    else {
        treasury_amount = treasury_amount * output_reserve / input_reserve; 
        burn_amount = burn_amount * output_reserve / input_reserve; 

        <Pools<T>>::insert(
            (&bought_asset_id, &sold_asset_id),
            output_reserve - treasury_amount - burn_amount,
        );
    }


    if settling_asset_id == mangata_id {

        <Treasury<T>>::insert(
            mangata_id,
            <Treasury<T>>::get(mangata_id) + treasury_amount
        );
        
        <assets::Module<T>>::assets_burn(&mangata_id, &vault, &burn_amount);
    }

    //need to swap our settling asset to mangata
    else if <Pools<T>>::contains_key((settling_asset_id,mangata_id)){
        let input_reserve = <Pools<T>>::get((settling_asset_id, mangata_id));
        let output_reserve = <Pools<T>>::get((mangata_id, settling_asset_id));

        let treasury_amount_in_mangata = Self::calculate_sell_price_no_fee(
            input_reserve,
            output_reserve,
            treasury_amount,
        );
        let burn_amount_in_mangata = Self::calculate_sell_price_no_fee(
            input_reserve,
            output_reserve,
            burn_amount,
        );

        <Pools<T>>::insert(
            (settling_asset_id, mangata_id),
            input_reserve + treasury_amount + burn_amount,
        );

        <Pools<T>>::insert(
            (mangata_id, settling_asset_id),
            output_reserve - treasury_amount_in_mangata - burn_amount_in_mangata,
        );

        <Treasury<T>>::insert(
            mangata_id,
            <Treasury<T>>::get(mangata_id) + treasury_amount_in_mangata
        );
        <assets::Module<T>>::assets_burn(&mangata_id, &vault, &burn_amount_in_mangata);
    }
    else {
        <Treasury<T>>::insert(
            settling_asset_id,
            <Treasury<T>>::get(settling_asset_id) + treasury_amount
        );
        <TreasuryBurn<T>>::insert(
            settling_asset_id,
            <TreasuryBurn<T>>::get(settling_asset_id) + burn_amount
        );
    }


}


    fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
}
