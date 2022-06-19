
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure, PalletId,
};
use frame_system::ensure_signed;
use sp_core::U256;
// TODO documentation!
use codec::FullCodec;
use frame_support::{
	pallet_prelude::*,
	traits::{ExistenceRequirement, Get, WithdrawReasons},
	transactional, Parameter,
};
use frame_system::pallet_prelude::*;
use mangata_primitives::{Balance, TokenId};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use pallet_assets_info as assets_info;
use pallet_issuance::{ComputeIssuance, PoolPromoteApi};
use pallet_vesting_mangata::MultiTokenVestingLocks;
use sp_arithmetic::helpers_128bit::multiply_by_rational;
use sp_bootstrap::PoolCreateApi;
use sp_runtime::traits::{
	AccountIdConversion, AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member,
	SaturatedConversion, Zero,
};
use sp_std::{convert::TryFrom, fmt::Debug, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub(crate) const LOG_TARGET: &'static str = "mpl";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

const PALLET_ID: PalletId = PalletId(*b"79b14c96");
// Quocient ratio in which liquidity minting curve is rising
const Q: f64 = 1.06;

// Keywords for asset_info
const LIQUIDITY_TOKEN_IDENTIFIER: &[u8] = b"LiquidityPoolToken";
const HEX_INDICATOR: &[u8] = b"0x";
const TOKEN_SYMBOL: &[u8] = b"TKN";
const TOKEN_SYMBOL_SEPARATOR: &[u8] = b"-";
const LIQUIDITY_TOKEN_DESCRIPTION: &[u8] = b"Generated Info for Liquidity Pool Token";
const DEFAULT_DECIMALS: u32 = 18u32;

pub use pallet::*;

mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

#[cfg(not(feature = "enable-trading"))]
fn is_asset_enabled(asset_id: &TokenId) -> bool {
	asset_id < &(2 as u32) || asset_id > &(3 as u32)
}

#[cfg(feature = "enable-trading")]
fn is_asset_enabled(_asset_id: &TokenId) -> bool {
	true
}

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets_info::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MaxVestingSchedules: Get<u32>;
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;
		type NativeCurrencyId: Get<TokenId>;
		type TreasuryPalletId: Get<PalletId>;
		type BnbTreasurySubAccDerive: Get<[u8; 4]>;
		type PoolPromoteApi: ComputeIssuance + PoolPromoteApi;
		#[pallet::constant]
		/// The account id that holds the liquidity mining issuance
		type LiquidityMiningIssuanceVault: Get<Self::AccountId>;
		#[pallet::constant]
		type PoolFeePercentage: Get<u128>;
		#[pallet::constant]
		type TreasuryFeePercentage: Get<u128>;
		#[pallet::constant]
		type BuyAndBurnFeePercentage: Get<u128>;
		#[pallet::constant]
		type RewardsDistributionPeriod: Get<u32>;
		type VestingProvider: MultiTokenVestingLocks<Self::AccountId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// Pool already Exists
		PoolAlreadyExists,
		/// Not enought assets
		NotEnoughAssets,
		/// No such pool exists
		NoSuchPool,
		/// No such liquidity asset exists
		NoSuchLiquidityAsset,
		/// Not enought reserve
		NotEnoughReserve,
		/// Zero amount is not supported
		ZeroAmount,
		/// Insufficient input amount
		InsufficientInputAmount,
		/// Insufficient output amount
		InsufficientOutputAmount,
		/// Asset ids cannot be the same
		SameAsset,
		/// Asset already exists
		AssetAlreadyExists,
		/// Asset does not exists
		AssetDoesNotExists,
		/// Division by zero
		DivisionByZero,
		/// Unexpected failure
		UnexpectedFailure,
		/// Unexpected failure
		NotMangataLiquidityAsset,
		/// Second asset amount exceeded expectations
		SecondAssetAmountExceededExpectations,
		/// Math overflow
		MathOverflow,
		/// Liquidity token creation failed
		LiquidityTokenCreationFailed,
		/// Not enought rewards earned
		NotEnoughtRewardsEarned,
		/// Not a promoted pool
		NotAPromotedPool,
		/// Past time calculation
		PastTimeCalculation,
		/// Pool already promoted
		PoolAlreadyPromoted,
		/// Sold Amount too low
		SoldAmountTooLow,
		/// Asset id is blacklisted
		FunctionNotAvailableForThisToken,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated(T::AccountId, TokenId, Balance, TokenId, Balance),
		AssetsSwapped(T::AccountId, TokenId, Balance, TokenId, Balance),
		LiquidityMinted(T::AccountId, TokenId, Balance, TokenId, Balance, TokenId, Balance),
		LiquidityBurned(T::AccountId, TokenId, Balance, TokenId, Balance, TokenId, Balance),
		PoolPromoted(TokenId),
		LiquidityActivated(T::AccountId, TokenId, Balance),
		LiquidityDeactivated(T::AccountId, TokenId, Balance),
		RewardsClaimed(T::AccountId, TokenId, Balance),
	}

    #[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
    pub struct ReserveStatusInfo {
        staked_unactivated_reserves: Balance,
        activated_unstaked_reserves: Balance,
        staked_and_activated_reserves: Balance,
        unspent_reserves: Balance,
		relock_amount: Balance,
    }

	#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
    pub struct RelockStatusInfo {
        amount: Balance,
        ending_block_as_balance: Balance,
    }

	#[pallet::storage]
	#[pallet::getter(fn get_reserve_status)]
	pub type ReserveStatus<T: Config> =
		StorageMap<_, Blake2_256, T::AccountId, Twox64Concat, TokenId, ReserveStatusInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_relock_status)]
	pub type RelockStatus<T: Config> =
		StorageMap<_, Blake2_256, T::AccountId, Twox64Concat, TokenId, BoundedVec<RelockStatusInfo, T::MaxVestingSchedules::get()>, ValueQuery>;

	
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub reserve_status:
			Vec<(T::AccountId, TokenId, ReserveStatusInfo)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { reserve_status: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// TODO
			// Add genesis build
		}
	}

	// XYK extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[transactional]
		#[pallet::weight(4_000_000_000u64)]
		// This extrinsic has to be transactional
		pub fn reserve_vesting_liquidity_tokens(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			liquidity_token_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(T::Xyk::is_liquidity_token(liquidity_token_id), Error::<T>::NotALiquidityToken);

			let vesting_ending_block_as_balance: Balance = T::VestingProvider::unlock_tokens(
				&sender,
				liquidity_token_id.into(),
				liquidity_token_amount.into(),
			)?
			.into();

			let mut reserve_status = Pallet::<T>::get_reserve_status(sender, liquidity_token_id);

			reserve_status.relock_amount = reserve_status.relock_amount.checked_add(liquidity_token_amount).ok_or(Error::<T>::MathError)?;
			reserve_status.unspent_reserves = reserve_status.unspent_reserves.checked_add(liquidity_token_amount).ok_or(Error::<T>::MathError)?;

			ReserveStatus::<T>::insert(sender, liquidity_token_id, reserve_status);

			RelockStatus::<T>::try_append(sender, liquidity_token_id, RelockStatusInfo{
				amount: liquidity_token_amount,
				ending_block_as_balance: vesting_ending_block_as_balance
			}).ok_or(Error::<T>::RelockCountLimitExceeded)?;

			T::Tokens::reserve(liquidity_token_id.into(), &sender, liquidity_token_amount.into())?;

			Ok(().into())
		}

		#[transactional]
		#[pallet::weight(4_000_000_000u64)]
		// This extrinsic has to be transactional
		pub fn unreserve_and_relock_instance(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			relock_instance_index: u32,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let relock_instances: Vec<RelockStatusInfo> = Self::get_relock_status(sender, liquidity_token_id).ok_or(Error::<T>::NoRelocks)?.into();

			let mut selected_relock_instance: Option<RelockStatusInfo> = None;

			let updated_relock_instances = relock_instances.into_iter().enumerate().filter_map(move |(index, relock_instance)| {
				if index == relock_instance_index {
					selected_relock_instance = relock_instance.clone()
					None
				} else {
					Some(relock_instance)
				}
			}

			selected_relock_instance = selected_relock_instance.ok_or(Error::<T>::RelockInstanceIndexOOB)?;
			
			let mut reserve_status = Pallet::<T>::get_reserve_status(sender, liquidity_token_id);

			reserve_status.relock_amount = reserve_status.relock_amount.checked_sub(selected_relock_instance.amount).ok_or(Error::<T>::MathError)?;
			reserve_status.unspent_reserves = reserve_status.unspent_reserves.checked_sub(selected_relock_instance.amount).ok_or(Error::<T>::NotEnoughUnspentReserves)?;

			ensure!(T::Tokens::unreserve(liquidity_token_id.into(), &sender, selected_relock_instance.amount.into()).is_zero(), Error::<T>::MathError);

			T::VestingProvider::lock_tokens(
				&sender,
				liquidity_token_id.into(),
				selected_relock_instance.amount,
				selected_relock_instance.ending_block_as_balance
			)?;

			ReserveStatus::<T>::insert(sender, liquidity_token_id, reserve_status);

			RelockStatus::<T>::insert(sender, liquidity_token_id,updated_relock_instances);

			Ok(().into())
		}

	}

}


impl StakingReservesProvider for Pallet<T>{

	type AccountId = T::AccountId;

	fn can_bond(token_id: TokenId, account_id: Self::AccountId, amount: Balance, use_balance_from: Option<BondKind>)
	-> bool {
		let reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		match use_balance_from {
			BondKind::FreeBalance => ensure_can_withdraw(token_id.into(), &account_id, amount.into(), Default::default(), Default::default()).is_ok()
				&& reserve_status.staked_unactivated_reserves.checked_add(amount).is_ok(),
			BondKind::ActivatedUnstakedLiquidty => reserve_status.activated_unstaked_reserves.checked_sub(amount).is_ok()
				&& reserve_status.staked_and_activated_reserves.checked_add(amount).is_ok(),
			BondKind::UnspentReserves => reserve_status.unspent_reserves.checked_sub(amount).is_ok()
				&& reserve_status.staked_unactivated_reserves.checked_add(amount).is_ok(),
		}
	}

	fn bond(token_id: TokenId, account_id: Self::AccountId, amount: Balance, use_balance_from: Option<BondKind>)
	-> DispatchResult {
		let mut reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		let use_balance_from = use_balance_from.unwrap_or(BondKind::FreeBalance);

		match use_balance_from {
			BondKind::FreeBalance => {
				reserve_status.staked_unactivated_reserves = reserve_status.staked_unactivated_reserves.checked_add(amount)
				.ok_or(Error::<T>::MathError)?;
				T::Tokens::reserve(token_id.into(), &account_id, amount.into())?
			},
			BondKind::ActivatedUnstakedLiquidty =>{
				reserve_status.activated_unstaked_reserves = reserve_status.activated_unstaked_reserves.checked_sub(amount)
				.ok_or(Error::<T>::NotEnoughTokens)?;
				reserve_status.staked_and_activated_reserves = reserve_status.staked_and_activated_reserves.checked_add(amount)
				.ok_or(Error::<T>::MathError)?;
			},
			BondKind::UnspentReserves =>{
				reserve_status.unspent_reserves = reserve_status.unspent_reserves.checked_sub(amount)
				.ok_or(Error::<T>::NotEnoughTokens)?;
				reserve_status.staked_unactivated_reserves = reserve_status.staked_unactivated_reserves.checked_add(amount)
				.ok_or(Error::<T>::MathError)?;
			},
		}

		ReserveStatus::<T>::insert(account_id, token_id, reserve_status);
		Ok(())
	}

	fn unbond(token_id: TokenId, account_id: Self::AccountId, amount: Balance) -> Balance {
		// From staked_unactivated_reserves goes to either free balance or unspent reserves depending on relock_amount

		// From staked_and_activated_reserves goes to activated always.

		let mut reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);
		let mut working_amount = amount;
		let mut unreserve_amount = Balance::zero();

		unreserve_amount = working_amount.min(reserve_status.staked_unactivated_reserves);
		working_amount = working_amount.saturating_sub(unreserve_amount);
		reserve_status.staked_unactivated_reserves = reserve_status.staked_unactivated_reserves.saturating_sub(unreserve_amount);


		let mut move_reserve = working_amount.min(reserve_status.staked_and_activated_reserves);
		// This is just to prevent overflow.
		move_reserve = Balance::max_value().saturating_sub(reserve_status.activated_unstaked_reserves).min(move_reserve);
		reserve_status.staked_and_activated_reserves = reserve_status.staked_and_activated_reserves.saturating_sub(move_reserve);
		reserve_status.activated_unstaked_reserves = reserve_status.activated_unstaked_reserves.saturating_add(move_reserve);
		working_amount = working_amount.saturating_sub(move_reserve);

		// Now we will attempt to unreserve the amount on the basis of the relock_amount
		let total_remaining_reserve = reserve_status.staked_unactivated_reserves.saturating_add(
        reserve_status.activated_unstaked_reserves).saturating_add(
        reserve_status.staked_and_activated_reserves).saturating_add(
        reserve_status.unspent_reserves);

		let mut add_to_unspent = reserve_status.relock_amount.saturating_sub(total_remaining_reserve);
		if add_to_unspent > unreserve_amount{
			log::warn!(
				"Unbond witnessed prior state of relock_amount being higher than mpl reserves {:?} {:?}",
				add_to_unspent,
				unreserve_amount
			);
		}
		add_to_unspent = add_to_unspent.min(unreserve_amount);
		unreserve_amount = unreserve_amount.saturating_sub(add_to_unspent);
		reserve_status.unspent_reserves = reserve_status.unspent_reserves.saturating_add(add_to_unspent);

		let unreserve_result = T::Tokens::unreserve(token_id.into(), account_id, unreserve_amount.into());

		if !unreserve_result.is_zero(){
			log::warn!(
				"Unbond resulted in non-zero unreserve_result {:?}",
				unreserve_result
			);
		}

		if !working_amount.is_zero(){
			log::warn!(
				"Unbond resulted in left-over amount {:?}",
				working_amount
			);
		}

		ReserveStatus::<T>::insert(account_id, token_id, reserve_status);
		working_amount.saturating_add(unreserve_result)
	}
}

impl XykReservesProvider for Pallet<T>{

	type AccountId = T::AccountId;

	fn get_max_instant_unreserve_amount(token_id: TokenId, account_id: Self::AccountId)
	-> DispatchResult {
		let reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		let total_remaining_reserve = reserve_status.staked_unactivated_reserves.saturating_add(
			reserve_status.staked_and_activated_reserves).saturating_add(
			reserve_status.unspent_reserves);

		let amount_held_back_by_relock = reserve_status.relock_amount.saturating_sub(total_remaining_reserve);

		// We assume here that the actual unreserve will ofcoures go fine returning 0.
		reserve_status.activated_unstaked_reserves.saturating_sub(amount_held_back_by_relock)
	}

	fn activate(token_id: TokenId, account_id: Self::AccountId, amount: Balance, use_balance_from: Option<ActivateKind>)
	-> DispatchResult {
		let mut reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		let use_balance_from = use_balance_from.unwrap_or(ActivateKind::FreeBalance);

		match use_balance_from {
			ActivateKind::FreeBalance => {
				reserve_status.activated_unstaked_reserves = reserve_status.activated_unstaked_reserves.checked_add(amount)
				.ok_or(Error::<T>::MathError)?;
				T::Tokens::reserve(token_id.into(), &account_id, amount.into())?;
			},
			ActivateKind::StakedUnactivatedLiquidty =>{
				reserve_status.staked_unactivated_reserves = reserve_status.staked_unactivated_reserves.checked_sub(amount)
				.ok_or(Error::<T>::NotEnoughTokens)?;
				reserve_status.staked_and_activated_reserves = reserve_status.staked_and_activated_reserves.checked_add(amount)
				.ok_or(Error::<T>::MathError)?;
			},
			ActivateKind::UnspentReserves =>{
				reserve_status.unspent_reserves = reserve_status.unspent_reserves.checked_sub(amount)
				.ok_or(Error::<T>::NotEnoughTokens)?;
				reserve_status.activated_unstaked_reserves = reserve_status.activated_unstaked_reserves.checked_add(amount)
				.ok_or(Error::<T>::MathError)?;
			},
		}

		ReserveStatus::<T>::insert(account_id, token_id, reserve_status);
		Ok(())
	}

	fn deactivate(token_id: TokenId, account_id: Self::AccountId, amount: Balance) -> Balance {
		// From ActivatedUnstakedLiquidity goes to either free balance or unspent reserves depending on relock_amount
		// From staked_and_activated_reserves goes to staked always.

		let mut reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);
		let mut working_amount = amount;
		let mut unreserve_amount = Balance::zero();

		unreserve_amount = working_amount.min(reserve_status.activated_unstaked_reserves);
		working_amount = working_amount.saturating_sub(unreserve_amount);
		reserve_status.activated_unstaked_reserves = reserve_status.activated_unstaked_reserves.saturating_sub(unreserve_amount);


		let mut move_reserve = working_amount.min(reserve_status.staked_and_activated_reserves);
		// This is just to prevent overflow.
		move_reserve = Balance::max_value().saturating_sub(reserve_status.staked_unactivated_reserves).min(move_reserve);
		reserve_status.staked_and_activated_reserves = reserve_status.staked_and_activated_reserves.saturating_sub(move_reserve);
		reserve_status.staked_unactivated_reserves = reserve_status.staked_unactivated_reserves.saturating_add(move_reserve);
		working_amount = working_amount.saturating_sub(move_reserve);

		// Now we will attempt to unreserve the amount on the basis of the relock_amount
		let total_remaining_reserve = reserve_status.staked_unactivated_reserves.saturating_add(
        reserve_status.activated_unstaked_reserves).saturating_add(
        reserve_status.staked_and_activated_reserves).saturating_add(
        reserve_status.unspent_reserves);

		let mut add_to_unspent = reserve_status.relock_amount.saturating_sub(total_remaining_reserve);
		if add_to_unspent > unreserve_amount{
			log::warn!(
				"Unbond witnessed prior state of relock_amount being higher than mpl reserves {:?} {:?}",
				add_to_unspent,
				unreserve_amount
			);
		}
		add_to_unspent = add_to_unspent.min(unreserve_amount);
		unreserve_amount = unreserve_amount.saturating_sub(add_to_unspent);
		reserve_status.unspent_reserves = reserve_status.unspent_reserves.saturating_add(add_to_unspent);

		let unreserve_result = T::Tokens::unreserve(token_id.into(), account_id, unreserve_amount.into());

		if !unreserve_result.is_zero(){
			log::warn!(
				"Unbond resulted in non-zero unreserve_result {:?}",
				unreserve_result
			);
		}

		if !working_amount.is_zero(){
			log::warn!(
				"Unbond resulted in left-over amount {:?}",
				working_amount
			);
		}

		ReserveStatus::<T>::insert(account_id, token_id, reserve_status);
		working_amount.saturating_add(unreserve_result)
	}
}

// TODO Xyk stuff
// TODO linkers here
// VEsting stuff here
// Tests
// Benchmarking
// Genesis
// Storage Migration

// Vesting
// Xyk fix
// Compile