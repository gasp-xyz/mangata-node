//! # XYK pallet

//! Provides functions for token operations, swapping tokens, creating token pools, minting and burning liquidity and supporting public functions
//!
//! ### Token operation functions:
//! - create_pool
//! - mint_liquidity
//! - burn_liquidity
//! - sell_asset
//! - buy_asset
//! - compound_rewards
//! - provide_liquidity_with_conversion
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
//! - calculate_balanced_sell_amount
//! - get_liq_tokens_for_trading
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
//! # fn compound_rewards
//! - Claims a specified portion of rewards, and provides them back into the selected pool.
//! - Wraps claim_rewards, sell_asset and mint_liquidity, so that there is minimal surplus of reward asset left after operation.
//! - Current impl assumes a MGX-ASSET pool & rewards in MGX asset
//!
//! ### arguments
//! `origin` - sender of a fn, user claiming rewards and providing liquidity to the pool
//!
//! liquidity_asset_id - the pool where we provide the liquidity
//!
//! amount_permille - portion of rewards to claim
//!
//! ### Example
//! ```ignore
//! compound_rewards (
//!    Origin::signed(1),
//!    2,
//!    1_000,
//!)
//! ```
//! Claim all of the rewards, currently in MGX, and use them to provide liquidity for the pool with asset id 2
//!
//! ### Errors
//! - inherits all of the errors from `claim_rewards`, `sell_asset` and `mint_liquidity`
//!
//! `NoSuchLiquidityAsset` - pool with given asset id does not exist
//!
//! `FunctionNotAvailableForThisToken` - not available for this asset id
//!
//! `NotEnoughRewardsEarned` - not enough rewards available
//!
//! # fn provide_liquidity_with_conversion
//! - Given one of the liquidity pool asset, computes balanced sell amount and provides liquidity into the pool
//! - Wraps sell_asset and mint_liquidity
//!
//! ### arguments
//! `origin` - sender of a fn, user claiming rewards and providing liquidity to the pool
//!
//! liquidity_asset_id - the pool where we provide the liquidity
//!
//! provided_asset_id - which asset of the pool
//!
//! provided_asset_amount - amount of the provided asset to use
//!
//! ### Example
//! ```ignore
//! provide_liquidity_with_conversion (
//!    Origin::signed(1),
//!    2,
//!    1,
//!    1_000_000,
//!)
//! ```
//! Given the liquidity pool with asset id 2, we assume that asset id 1 is one of the pool's pair, compute balanced swap and provide liquidity into the pool
//!
//! ### Errors
//! - inherits all of the errors from `sell_asset` and `mint_liquidity`
//!
//! `NoSuchLiquidityAsset` - pool wiht given asset id does not exist
//!
//! `FunctionNotAvailableForThisToken` - not available for this asset id
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
//! # calculate_balanced_sell_amount
//! - Supporting public function accessible through rpc call which calculates how much amount x we need to swap from total_amount, so that after `y = swap(x)`, the resulting balance equals `(total_amount - x) / y = pool_x / pool_y`
//! - the resulting amounts can then be used to `mint_liquidity` with minimal leftover after operation
//! # get_liq_tokens_for_trading
//! - Supporting public function accessible through rpc call which lists all of the liquidity pool token ids that are available for trading

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	assert_ok,
	dispatch::{DispatchErrorWithPostInfo, DispatchResult, PostDispatchInfo},
	ensure,
	traits::Contains,
	PalletId,
};
use frame_system::ensure_signed;
use sp_core::U256;

use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::currency::{MultiTokenCurrency, MultiTokenVestingLocks},
		ExistenceRequirement, Get, WithdrawReasons,
	},
	transactional,
};
use frame_system::pallet_prelude::*;
use mangata_support::traits::{
	ActivationReservesProviderTrait, GetMaintenanceStatusTrait, PoolCreateApi, PreValidateSwaps,
	ProofOfStakeRewardsApi, Valuate, XykFunctionsTrait,
};
use mangata_types::multipurpose_liquidity::ActivateKind;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use sp_arithmetic::{helpers_128bit::multiply_by_rational_with_rounding, per_things::Rounding};
use sp_runtime::{
	traits::{
		AccountIdConversion, Bounded, CheckedAdd, CheckedDiv, CheckedSub, One, Saturating, Zero,
	},
	DispatchError, ModuleError, Permill, SaturatedConversion,
};
use sp_std::{
	collections::btree_set::BTreeSet,
	convert::{TryFrom, TryInto},
	ops::Div,
	prelude::*,
	vec,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub(crate) const LOG_TARGET: &str = "xyk";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

const PALLET_ID: PalletId = PalletId(*b"79b14c96");

// Keywords for asset_info
const LIQUIDITY_TOKEN_IDENTIFIER: &[u8] = b"LiquidityPoolToken";
const HEX_INDICATOR: &[u8] = b"0x";
const TOKEN_SYMBOL: &[u8] = b"TKN";
const TOKEN_SYMBOL_SEPARATOR: &[u8] = b"-";
const DEFAULT_DECIMALS: u32 = 18u32;

pub use pallet::*;

mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type BalanceOf<T> = <<T as pallet::Config>::Currency as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

pub type CurrencyIdOf<T> = <<T as pallet::Config>::Currency as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

// type LiquidityMiningRewardsOf<T> = <T as ::Config>::AccountId;
#[derive(Eq, PartialEq, Encode, Decode)]
pub enum SwapKind {
	Sell,
	Buy,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::DispatchClass;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[cfg(feature = "runtime-benchmarks")]
	pub trait XykBenchmarkingConfig:
		pallet_issuance::Config + pallet_proof_of_stake::Config
	{
	}

	#[cfg(not(feature = "runtime-benchmarks"))]
	pub trait XykBenchmarkingConfig {}

	// #[cfg(feature = "runtime-benchmarks")]
	// pub trait XykRewardsApi<AccountIdT>: ProofOfStakeRewardsApi<AccountIdT, Balance = Balance, CurrencyId = CurrencyIdOf<T>> + LiquidityMiningApi{}
	// #[cfg(feature = "runtime-benchmarks")]
	// impl<K,AccountIdT> XykRewardsApi<AccountIdT> for K where
	// 	K: ProofOfStakeRewardsApi<AccountIdT, Balance = Balance, CurrencyId = CurrencyIdOf<T>>,
	// 	K: LiquidityMiningApi,
	// {
	// }
	//
	// #[cfg(not(feature = "runtime-benchmarks"))]
	// pub trait XykRewardsApi<AccountIdT>: ProofOfStakeRewardsApi<AccountIdT, Balance = Balance, CurrencyId = CurrencyIdOf<T>>{}
	// #[cfg(not(feature = "runtime-benchmarks"))]
	// impl<K,AccountIdT> XykRewardsApi<AccountIdT> for K where
	// 	K: ProofOfStakeRewardsApi<AccountIdT, Balance = Balance, CurrencyId = CurrencyIdOf<T>>,
	// {
	// }

	#[pallet::config]
	pub trait Config: frame_system::Config + XykBenchmarkingConfig {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MaintenanceStatusProvider: GetMaintenanceStatusTrait;
		type ActivationReservesProvider: ActivationReservesProviderTrait<
			Self::AccountId,
			BalanceOf<Self>,
			CurrencyIdOf<Self>,
		>;
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;
		type NativeCurrencyId: Get<CurrencyIdOf<Self>>;
		type TreasuryPalletId: Get<PalletId>;
		type BnbTreasurySubAccDerive: Get<[u8; 4]>;
		type LiquidityMiningRewards: ProofOfStakeRewardsApi<
			Self::AccountId,
			BalanceOf<Self>,
			CurrencyIdOf<Self>,
		>;
		#[pallet::constant]
		type PoolFeePercentage: Get<u128>;
		#[pallet::constant]
		type TreasuryFeePercentage: Get<u128>;
		#[pallet::constant]
		type BuyAndBurnFeePercentage: Get<u128>;
		type DisallowedPools: Contains<(CurrencyIdOf<Self>, CurrencyIdOf<Self>)>;
		type DisabledTokens: Contains<CurrencyIdOf<Self>>;
		type VestingProvider: MultiTokenVestingLocks<
			Self::AccountId,
			Currency = <Self as pallet::Config>::Currency,
			Moment = BlockNumberFor<Self>,
		>;
		type AssetMetadataMutation: AssetMetadataMutationTrait<CurrencyIdOf<Self>>;
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
		/// Not enough rewards earned
		NotEnoughRewardsEarned,
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
		/// Pool considting of passed tokens id is blacklisted
		DisallowedPool,
		LiquidityCheckpointMathError,
		CalculateRewardsMathError,
		CalculateCumulativeWorkMaxRatioMathError,
		CalculateRewardsAllMathError,
		NoRights,
		MultiswapShouldBeAtleastTwoHops,
		MultiBuyAssetCantHaveSamePoolAtomicSwaps,
		MultiSwapCantHaveSameTokenConsequetively,
		/// Trading blocked by maintenance mode
		TradingBlockedByMaintenanceMode,
		PoolIsEmpty,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
		AssetsSwapped(T::AccountId, Vec<CurrencyIdOf<T>>, BalanceOf<T>, BalanceOf<T>),
		SellAssetFailedDueToSlippage(
			T::AccountId,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
		),
		BuyAssetFailedDueToSlippage(
			T::AccountId,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
		),
		LiquidityMinted(
			T::AccountId,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			BalanceOf<T>,
		),
		LiquidityBurned(
			T::AccountId,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			BalanceOf<T>,
		),
		PoolPromotionUpdated(CurrencyIdOf<T>, Option<u8>),
		LiquidityActivated(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>),
		LiquidityDeactivated(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>),
		RewardsClaimed(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>),
		MultiSwapAssetFailedOnAtomicSwap(
			T::AccountId,
			Vec<CurrencyIdOf<T>>,
			BalanceOf<T>,
			ModuleError,
		),
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_pool)]
	pub type Pools<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(CurrencyIdOf<T>, CurrencyIdOf<T>),
		(BalanceOf<T>, BalanceOf<T>),
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_asset)]
	pub type LiquidityAssets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(CurrencyIdOf<T>, CurrencyIdOf<T>),
		Option<CurrencyIdOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	pub type LiquidityPools<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CurrencyIdOf<T>,
		Option<(CurrencyIdOf<T>, CurrencyIdOf<T>)>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_number_of_swaps)]
	pub type TotalNumberOfSwaps<T: Config> =
		StorageValue<_, u32, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub created_pools_for_staking: Vec<(
			T::AccountId,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
		)>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { created_pools_for_staking: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			self.created_pools_for_staking.iter().for_each(
				|(
					account_id,
					native_token_id,
					native_token_amount,
					pooled_token_id,
					pooled_token_amount,
					liquidity_token_id,
				)| {
					if <T as Config>::Currency::exists({ *liquidity_token_id }.into()) {
						assert!(
							<Pallet<T> as XykFunctionsTrait<
								T::AccountId,
								BalanceOf<T>,
								CurrencyIdOf<T>,
							>>::mint_liquidity(
								account_id.clone(),
								*native_token_id,
								*pooled_token_id,
								*native_token_amount,
								*pooled_token_amount,
								true,
							)
							.is_ok(),
							"Pool mint failed"
						);
					} else {
						let created_liquidity_token_id: CurrencyIdOf<T> =
							<T as Config>::Currency::get_next_currency_id().into();
						assert_eq!(
							created_liquidity_token_id, *liquidity_token_id,
							"Assets not initialized in the expected sequence",
						);
						assert_ok!(<Pallet<T> as XykFunctionsTrait<
							T::AccountId,
							BalanceOf<T>,
							CurrencyIdOf<T>,
						>>::create_pool(
							account_id.clone(),
							*native_token_id,
							*native_token_amount,
							*pooled_token_id,
							*pooled_token_amount
						));
					}
				},
			)
		}
	}

	// XYK extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<<T as Config>::WeightInfo>::create_pool())]
		pub fn create_pool(
			origin: OriginFor<T>,
			first_asset_id: CurrencyIdOf<T>,
			first_asset_amount: BalanceOf<T>,
			second_asset_id: CurrencyIdOf<T>,
			second_asset_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(
				!T::DisabledTokens::contains(&first_asset_id) &&
					!T::DisabledTokens::contains(&second_asset_id),
				Error::<T>::FunctionNotAvailableForThisToken
			);

			ensure!(
				!T::DisallowedPools::contains(&(first_asset_id, second_asset_id)),
				Error::<T>::DisallowedPool,
			);

			<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::create_pool(
				sender,
				first_asset_id,
				first_asset_amount,
				second_asset_id,
				second_asset_amount,
			)?;

			Ok(().into())
		}

		/// Executes sell_asset swap.
		/// First the swap is prevalidated, if it is successful then the extrinsic is accepted. Beyond this point the exchange commission will be charged.
		/// The sold amount of the sold asset is used to determine the bought asset amount.
		/// If the bought asset amount is lower than the min_amount_out then it will fail on slippage.
		/// The percentage exchange commission is still charged even if the swap fails on slippage. Though the swap itself will be a no-op.
		/// The slippage is calculated based upon the sold_asset_amount.
		/// Upon slippage failure, the extrinsic is marked "successful", but an event for the failure is emitted
		///
		///
		/// # Args:
		/// - `sold_asset_id` - The token being sold
		/// - `bought_asset_id` - The token being bought
		/// - `sold_asset_amount`: The amount of the sold token being sold
		/// - `min_amount_out` - The minimum amount of bought asset that must be bought in order to not fail on slippage. Slippage failures still charge exchange commission.
		#[pallet::call_index(1)]
		#[pallet::weight((<<T as Config>::WeightInfo>::sell_asset(), DispatchClass::Operational, Pays::No))]
		#[deprecated(note = "multiswap_sell_asset should be used instead")]
		pub fn sell_asset(
			origin: OriginFor<T>,
			sold_asset_id: CurrencyIdOf<T>,
			bought_asset_id: CurrencyIdOf<T>,
			sold_asset_amount: BalanceOf<T>,
			min_amount_out: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::sell_asset(
				sender,
				sold_asset_id,
				bought_asset_id,
				sold_asset_amount,
				min_amount_out,
				false,
			)
			.map_err(|err| DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(<<T as Config>::WeightInfo>::sell_asset()),
					pays_fee: Pays::Yes,
				},
				error: err,
			})?;
			TotalNumberOfSwaps::<T>::mutate(|v|{*v=v.saturating_add(One::one())});
			Ok(Pays::No.into())
		}

		/// Executes a multiswap sell asset in a series of sell asset atomic swaps.
		///
		/// Multiswaps must fee lock instead of paying transaction fees.
		///
		/// First the multiswap is prevalidated, if it is successful then the extrinsic is accepted
		/// and the exchange commission will be charged upon execution on the **first** swap using **sold_asset_amount**.
		///
		/// Upon failure of an atomic swap or bad slippage, all the atomic swaps are reverted and the exchange commission is charged.
		/// Upon such a failure, the extrinsic is marked "successful", but an event for the failure is emitted
		///
		/// # Args:
		/// - `swap_token_list` - This list of tokens is the route of the atomic swaps, starting with the asset sold and ends with the asset finally bought
		/// - `sold_asset_amount`: The amount of the first asset sold
		/// - `min_amount_out` - The minimum amount of last asset that must be bought in order to not fail on slippage. Slippage failures still charge exchange commission.
		#[pallet::call_index(2)]
		#[pallet::weight((<<T as Config>::WeightInfo>::multiswap_sell_asset(swap_token_list.len() as u32), DispatchClass::Operational, Pays::No))]
		pub fn multiswap_sell_asset(
			origin: OriginFor<T>,
			swap_token_list: Vec<CurrencyIdOf<T>>,
			sold_asset_amount: BalanceOf<T>,
			min_amount_out: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			if let (Some(sold_asset_id), Some(bought_asset_id), 2) =
				(swap_token_list.get(0), swap_token_list.get(1), swap_token_list.len())
			{
				<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::sell_asset(
					sender,
					*sold_asset_id,
					*bought_asset_id,
					sold_asset_amount,
					min_amount_out,
					false,
				)
			} else {
				<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::multiswap_sell_asset(
					sender,
					swap_token_list.clone(),
					sold_asset_amount,
					min_amount_out,
					false,
					false,
				)
			}
			.map_err(|err| DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(<<T as Config>::WeightInfo>::multiswap_sell_asset(
						swap_token_list.len() as u32,
					)),
					pays_fee: Pays::Yes,
				},
				error: err,
			})?;
			Ok(Pays::No.into())
		}

		/// Executes buy_asset swap.
		/// First the swap is prevalidated, if it is successful then the extrinsic is accepted. Beyond this point the exchange commission will be charged.
		/// The bought of the bought asset is used to determine the sold asset amount.
		/// If the sold asset amount is higher than the max_amount_in then it will fail on slippage.
		/// The percentage exchange commission is still charged even if the swap fails on slippage. Though the swap itself will be a no-op.
		/// The slippage is calculated based upon the sold asset amount.
		/// Upon slippage failure, the extrinsic is marked "successful", but an event for the failure is emitted
		///
		///
		/// # Args:
		/// - `sold_asset_id` - The token being sold
		/// - `bought_asset_id` - The token being bought
		/// - `bought_asset_amount`: The amount of the bought token being bought
		/// - `max_amount_in` - The maximum amount of sold asset that must be sold in order to not fail on slippage. Slippage failures still charge exchange commission.
		#[pallet::call_index(3)]
		#[pallet::weight((<<T as Config>::WeightInfo>::buy_asset(), DispatchClass::Operational, Pays::No))]
		#[deprecated(note = "multiswap_buy_asset should be used instead")]
		pub fn buy_asset(
			origin: OriginFor<T>,
			sold_asset_id: CurrencyIdOf<T>,
			bought_asset_id: CurrencyIdOf<T>,
			bought_asset_amount: BalanceOf<T>,
			max_amount_in: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::buy_asset(
				sender,
				sold_asset_id,
				bought_asset_id,
				bought_asset_amount,
				max_amount_in,
				false,
			)
			.map_err(|err| DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(<<T as Config>::WeightInfo>::buy_asset()),
					pays_fee: Pays::Yes,
				},
				error: err,
			})?;
			TotalNumberOfSwaps::<T>::mutate(|v|{*v=v.saturating_add(One::one())});
			Ok(Pays::No.into())
		}

		/// Executes a multiswap buy asset in a series of buy asset atomic swaps.
		///
		/// Multiswaps must fee lock instead of paying transaction fees.
		///
		/// First the multiswap is prevalidated, if it is successful then the extrinsic is accepted
		/// and the exchange commission will be charged upon execution on the *first* swap using *max_amount_in*.
		/// multiswap_buy_asset cannot have two (or more) atomic swaps on the same pool.
		/// multiswap_buy_asset prevaildation only checks for whether there are enough funds to pay for the exchange commission.
		/// Failure to have the required amount of first asset funds will result in failure (and charging of the exchange commission).
		///
		/// Upon failure of an atomic swap or bad slippage, all the atomic swaps are reverted and the exchange commission is charged.
		/// Upon such a failure, the extrinsic is marked "successful", but an event for the failure is emitted
		///
		/// # Args:
		/// - `swap_token_list` - This list of tokens is the route of the atomic swaps, starting with the asset sold and ends with the asset finally bought
		/// - `bought_asset_amount`: The amount of the last asset bought
		/// - `max_amount_in` - The maximum amount of first asset that can be sold in order to not fail on slippage. Slippage failures still charge exchange commission.
		#[pallet::call_index(4)]
		#[pallet::weight((<<T as Config>::WeightInfo>::multiswap_buy_asset(swap_token_list.len() as u32), DispatchClass::Operational, Pays::No))]
		pub fn multiswap_buy_asset(
			origin: OriginFor<T>,
			swap_token_list: Vec<CurrencyIdOf<T>>,
			bought_asset_amount: BalanceOf<T>,
			max_amount_in: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			if let (Some(sold_asset_id), Some(bought_asset_id), 2) =
				(swap_token_list.get(0), swap_token_list.get(1), swap_token_list.len())
			{
				<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::buy_asset(
					sender,
					*sold_asset_id,
					*bought_asset_id,
					bought_asset_amount,
					max_amount_in,
					false,
				)
			} else {
				<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::multiswap_buy_asset(
					sender,
					swap_token_list.clone(),
					bought_asset_amount,
					max_amount_in,
					false,
					false,
				)
			}
			.map_err(|err| DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(<<T as Config>::WeightInfo>::multiswap_buy_asset(
						swap_token_list.len() as u32,
					)),
					pays_fee: Pays::Yes,
				},
				error: err,
			})?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<<T as Config>::WeightInfo>::mint_liquidity_using_vesting_native_tokens())]
		#[transactional]
		pub fn mint_liquidity_using_vesting_native_tokens_by_vesting_index(
			origin: OriginFor<T>,
			native_asset_vesting_index: u32,
			vesting_native_asset_unlock_some_amount_or_all: Option<BalanceOf<T>>,
			second_asset_id: CurrencyIdOf<T>,
			expected_second_asset_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let liquidity_asset_id =
				Pallet::<T>::get_liquidity_asset(Self::native_token_id(), second_asset_id)?;

			ensure!(
				<T::LiquidityMiningRewards as ProofOfStakeRewardsApi<
					T::AccountId,
					BalanceOf<T>,
					CurrencyIdOf<T>,
				>>::is_enabled(liquidity_asset_id),
				Error::<T>::NotAPromotedPool
			);

			let (unlocked_amount, vesting_starting_block, vesting_ending_block_as_balance): (
				BalanceOf<T>,
				BlockNumberFor<T>,
				BalanceOf<T>,
			) = <<T as Config>::VestingProvider>::unlock_tokens_by_vesting_index(
				&sender,
				Self::native_token_id().into(),
				native_asset_vesting_index,
				vesting_native_asset_unlock_some_amount_or_all,
			)
			.map(|x| (x.0, x.1, x.2))?;

			let (liquidity_token_id, liquidity_assets_minted) = <Self as XykFunctionsTrait<
				T::AccountId,
				BalanceOf<T>,
				CurrencyIdOf<T>,
			>>::mint_liquidity(
				sender.clone(),
				Self::native_token_id(),
				second_asset_id,
				unlocked_amount,
				expected_second_asset_amount,
				false,
			)?;

			<<T as Config>::VestingProvider>::lock_tokens(
				&sender,
				liquidity_token_id.into(),
				liquidity_assets_minted,
				Some(vesting_starting_block),
				vesting_ending_block_as_balance,
			)?;

			Ok(().into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<<T as Config>::WeightInfo>::mint_liquidity_using_vesting_native_tokens())]
		#[transactional]
		pub fn mint_liquidity_using_vesting_native_tokens(
			origin: OriginFor<T>,
			vesting_native_asset_amount: BalanceOf<T>,
			second_asset_id: CurrencyIdOf<T>,
			expected_second_asset_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let liquidity_asset_id =
				Pallet::<T>::get_liquidity_asset(Self::native_token_id(), second_asset_id)?;

			ensure!(
				<T::LiquidityMiningRewards as ProofOfStakeRewardsApi<
					T::AccountId,
					BalanceOf<T>,
					CurrencyIdOf<T>,
				>>::is_enabled(liquidity_asset_id),
				Error::<T>::NotAPromotedPool
			);

			let (vesting_starting_block, vesting_ending_block_as_balance): (
				BlockNumberFor<T>,
				BalanceOf<T>,
			) = <<T as Config>::VestingProvider>::unlock_tokens(
				&sender,
				Self::native_token_id().into(),
				vesting_native_asset_amount,
			)
			.map(|x| (x.0, x.1))?;

			let (liquidity_token_id, liquidity_assets_minted) = <Self as XykFunctionsTrait<
				T::AccountId,
				BalanceOf<T>,
				CurrencyIdOf<T>,
			>>::mint_liquidity(
				sender.clone(),
				Self::native_token_id(),
				second_asset_id,
				vesting_native_asset_amount,
				expected_second_asset_amount,
				false,
			)?;

			<<T as Config>::VestingProvider>::lock_tokens(
				&sender,
				liquidity_token_id.into(),
				liquidity_assets_minted,
				Some(vesting_starting_block),
				vesting_ending_block_as_balance,
			)?;

			Ok(().into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<<T as Config>::WeightInfo>::mint_liquidity())]
		pub fn mint_liquidity(
			origin: OriginFor<T>,
			first_asset_id: CurrencyIdOf<T>,
			second_asset_id: CurrencyIdOf<T>,
			first_asset_amount: BalanceOf<T>,
			expected_second_asset_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(
				!T::DisabledTokens::contains(&first_asset_id) &&
					!T::DisabledTokens::contains(&second_asset_id),
				Error::<T>::FunctionNotAvailableForThisToken
			);

			<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::mint_liquidity(
				sender,
				first_asset_id,
				second_asset_id,
				first_asset_amount,
				expected_second_asset_amount,
				true,
			)?;

			Ok(().into())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(<<T as Config>::WeightInfo>::compound_rewards())]
		#[transactional]
		pub fn compound_rewards(
			origin: OriginFor<T>,
			liquidity_asset_id: CurrencyIdOf<T>,
			amount_permille: Permill,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::do_compound_rewards(
				sender,
				liquidity_asset_id,
				amount_permille,
			)?;

			Ok(().into())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(<<T as Config>::WeightInfo>::provide_liquidity_with_conversion())]
		#[transactional]
		pub fn provide_liquidity_with_conversion(
			origin: OriginFor<T>,
			liquidity_asset_id: CurrencyIdOf<T>,
			provided_asset_id: CurrencyIdOf<T>,
			provided_asset_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (first_asset_id, second_asset_id) = LiquidityPools::<T>::get(liquidity_asset_id)
				.ok_or(Error::<T>::NoSuchLiquidityAsset)?;

			ensure!(
				!T::DisabledTokens::contains(&first_asset_id) &&
					!T::DisabledTokens::contains(&second_asset_id),
				Error::<T>::FunctionNotAvailableForThisToken
			);

			<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::provide_liquidity_with_conversion(
				sender,
				first_asset_id,
				second_asset_id,
				provided_asset_id,
				provided_asset_amount,
				true,
			)?;

			Ok(().into())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(<<T as Config>::WeightInfo>::burn_liquidity())]
		pub fn burn_liquidity(
			origin: OriginFor<T>,
			first_asset_id: CurrencyIdOf<T>,
			second_asset_id: CurrencyIdOf<T>,
			liquidity_asset_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::burn_liquidity(
				sender,
				first_asset_id,
				second_asset_id,
				liquidity_asset_amount,
			)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn total_fee() -> u128 {
		T::PoolFeePercentage::get() +
			T::TreasuryFeePercentage::get() +
			T::BuyAndBurnFeePercentage::get()
	}

	pub fn get_max_instant_burn_amount(
		user: &AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
	) -> BalanceOf<T> {
		Self::get_max_instant_unreserve_amount(user, liquidity_asset_id).saturating_add(
			<T as Config>::Currency::available_balance(liquidity_asset_id.into(), user),
		)
	}

	pub fn get_max_instant_unreserve_amount(
		user: &AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
	) -> BalanceOf<T> {
		<T as pallet::Config>::ActivationReservesProvider::get_max_instant_unreserve_amount(
			liquidity_asset_id,
			user,
		)
	}

	// Sets the liquidity token's info
	// May fail if liquidity_asset_id does not exsist
	// Should not fail otherwise as the parameters for the max and min length in pallet_assets_info should be set appropriately
	pub fn set_liquidity_asset_info(
		liquidity_asset_id: CurrencyIdOf<T>,
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
	) -> DispatchResult {
		let mut name: Vec<u8> = Vec::<u8>::new();
		name.extend_from_slice(LIQUIDITY_TOKEN_IDENTIFIER);
		name.extend_from_slice(HEX_INDICATOR);
		for bytes in liquidity_asset_id.saturated_into::<u32>().to_be_bytes().iter() {
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
		for bytes in first_asset_id.saturated_into::<u32>().to_be_bytes().iter() {
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
		for bytes in second_asset_id.saturated_into::<u32>().to_be_bytes().iter() {
			match (bytes >> 4) as u8 {
				x @ 0u8..=9u8 => symbol.push(x.saturating_add(48u8)),
				x => symbol.push(x.saturating_add(55u8)),
			}
			match (bytes & 0b0000_1111) as u8 {
				x @ 0u8..=9u8 => symbol.push(x.saturating_add(48u8)),
				x => symbol.push(x.saturating_add(55u8)),
			}
		}

		T::AssetMetadataMutation::set_asset_info(
			liquidity_asset_id,
			name,
			symbol,
			DEFAULT_DECIMALS,
		)?;
		Ok(())
	}

	// Calculate amount of tokens to be bought by sellling sell_amount
	pub fn calculate_sell_price(
		input_reserve: BalanceOf<T>,
		output_reserve: BalanceOf<T>,
		sell_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let after_fee_percentage: u128 = 10000_u128
			.checked_sub(Self::total_fee())
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let input_reserve_saturated: U256 = input_reserve.into().into();
		let output_reserve_saturated: U256 = output_reserve.into().into();
		let sell_amount_saturated: U256 = sell_amount.into().into();

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

		let result_u128 = u128::try_from(result_u256)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		let result = BalanceOf::<T>::try_from(result_u128)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
		log!(
			info,
			"calculate_sell_price: ({:?}, {:?}, {:?}) -> {:?}",
			input_reserve,
			output_reserve,
			sell_amount,
			result
		);
		Ok(result)
	}

	pub fn calculate_sell_price_no_fee(
		// Callculate amount of tokens to be received by sellling sell_amount, without fee
		input_reserve: BalanceOf<T>,
		output_reserve: BalanceOf<T>,
		sell_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let input_reserve_saturated: U256 = input_reserve.into().into();
		let output_reserve_saturated: U256 = output_reserve.into().into();
		let sell_amount_saturated: U256 = sell_amount.into().into();

		let numerator: U256 = sell_amount_saturated.saturating_mul(output_reserve_saturated);
		let denominator: U256 = input_reserve_saturated.saturating_add(sell_amount_saturated);
		let result_u256 = numerator
			.checked_div(denominator)
			.ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;
		let result_u128 = u128::try_from(result_u256)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
		let result = BalanceOf::<T>::try_from(result_u128)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
		log!(
			info,
			"calculate_sell_price_no_fee: ({:?}, {:?}, {:?}) -> {:?}",
			input_reserve,
			output_reserve,
			sell_amount,
			result
		);
		Ok(result)
	}

	// Calculate amount of tokens to be paid, when buying buy_amount
	pub fn calculate_buy_price(
		input_reserve: BalanceOf<T>,
		output_reserve: BalanceOf<T>,
		buy_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let after_fee_percentage: u128 = 10000_u128
			.checked_sub(Self::total_fee())
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let input_reserve_saturated: U256 = input_reserve.into().into();
		let output_reserve_saturated: U256 = output_reserve.into().into();
		let buy_amount_saturated: U256 = buy_amount.into().into();

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

		let result_u128 = u128::try_from(result_u256)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		let result = BalanceOf::<T>::try_from(result_u128)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
		log!(
			info,
			"calculate_buy_price: ({:?}, {:?}, {:?}) -> {:?}",
			input_reserve,
			output_reserve,
			buy_amount,
			result
		);
		Ok(result)
	}

	pub fn calculate_balanced_sell_amount(
		total_amount: BalanceOf<T>,
		reserve_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let multiplier: U256 = 10_000.into();
		let multiplier_sq: U256 = multiplier.pow(2.into());
		let non_pool_fees: U256 = Self::total_fee()
			.checked_sub(T::PoolFeePercentage::get())
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			.into(); // npf
		let total_fee: U256 = Self::total_fee().into(); // tf
		let total_amount_saturated: U256 = total_amount.into().into(); // z
		let reserve_amount_saturated: U256 = reserve_amount.into().into(); // a

		// n: 2*10_000^2*a - 10_000*tf*a - sqrt( (-2*10_000^2*a + 10_000*tf*a)^2 - 4*10_000^2*a*z*(10_000tf - npf*tf + 10_000npf - 10_000^2) )
		// d: 2 * (10_000tf - npf*tf + 10_000npf - 10_000^2)
		// x = n / d

		// fee_rate: 2*10_000^2 - 10_000*tf
		// simplify n: fee_rate*a - sqrt( fee_rate^2*s^2 - 2*10_000^2*(-d)*a*z )
		let fee_rate = multiplier_sq
			.saturating_mul(2.into())
			.checked_sub(total_fee.saturating_mul(multiplier))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		// 2*(10_000tf - npf*tf + 10_000npf - 10_000^2) -> negative number
		// change to: d = (10_000^2 + npf*tf - 10_000tf - 10_000npf) * 2
		let denominator_negative = multiplier_sq
			.checked_add(non_pool_fees.saturating_mul(total_fee))
			.and_then(|v| v.checked_sub(total_fee.saturating_mul(multiplier)))
			.and_then(|v| v.checked_sub(non_pool_fees.saturating_mul(multiplier)))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			.saturating_mul(2.into());

		// fee_rate^2*a^2 - 2*10_000^2*(-den)*a*z
		let sqrt_arg = reserve_amount_saturated
			.checked_pow(2.into())
			.and_then(|v| v.checked_mul(fee_rate.pow(2.into())))
			.and_then(|v| {
				v.checked_add(
					total_amount_saturated
						.saturating_mul(reserve_amount_saturated)
						.saturating_mul(denominator_negative)
						.saturating_mul(multiplier_sq)
						.saturating_mul(2.into()),
				)
			})
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		// n: fee_rate*a - sqrt(...) -> negative
		// sqrt(..) - fee_rate*a
		let numerator_negative = sqrt_arg
			.integer_sqrt()
			.checked_sub(reserve_amount_saturated.saturating_mul(fee_rate))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		// -n/-d == n/d
		let result_u256 = numerator_negative
			.checked_div(denominator_negative)
			.ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;
		let result_u128 = u128::try_from(result_u256)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
		let result = BalanceOf::<T>::try_from(result_u128)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		Ok(result)
	}

	pub fn get_liq_tokens_for_trading() -> Result<Vec<CurrencyIdOf<T>>, DispatchError> {
		let result = LiquidityAssets::<T>::iter_values()
			.filter_map(|v| v)
			.filter(|v| !<T as Config>::Currency::total_issuance((*v).into()).is_zero())
			.collect();

		Ok(result)
	}

	// MAX: 2R
	pub fn get_liquidity_asset(
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
	) -> Result<CurrencyIdOf<T>, DispatchError> {
		if LiquidityAssets::<T>::contains_key((first_asset_id, second_asset_id)) {
			LiquidityAssets::<T>::get((first_asset_id, second_asset_id))
				.ok_or_else(|| Error::<T>::UnexpectedFailure.into())
		} else {
			LiquidityAssets::<T>::get((second_asset_id, first_asset_id))
				.ok_or_else(|| Error::<T>::NoSuchPool.into())
		}
	}

	pub fn calculate_sell_price_id(
		sold_token_id: CurrencyIdOf<T>,
		bought_token_id: CurrencyIdOf<T>,
		sell_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_token_id, bought_token_id)?;

		ensure!(!(Self::is_pool_empty(sold_token_id, bought_token_id)?), Error::<T>::PoolIsEmpty);

		Self::calculate_sell_price(input_reserve, output_reserve, sell_amount)
	}

	pub fn calculate_buy_price_id(
		sold_token_id: CurrencyIdOf<T>,
		bought_token_id: CurrencyIdOf<T>,
		buy_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_token_id, bought_token_id)?;

		ensure!(!(Self::is_pool_empty(sold_token_id, bought_token_id)?), Error::<T>::PoolIsEmpty);

		Self::calculate_buy_price(input_reserve, output_reserve, buy_amount)
	}

	pub fn get_reserves(
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		let mut reserves = Pools::<T>::get((first_asset_id, second_asset_id));

		if Pools::<T>::contains_key((first_asset_id, second_asset_id)) {
			Ok((reserves.0, reserves.1))
		} else if Pools::<T>::contains_key((second_asset_id, first_asset_id)) {
			reserves = Pools::<T>::get((second_asset_id, first_asset_id));
			Ok((reserves.1, reserves.0))
		} else {
			Err(DispatchError::from(Error::<T>::NoSuchPool))
		}
	}

	/// worst case scenario
	/// MAX: 2R 1W
	pub fn set_reserves(
		first_asset_id: CurrencyIdOf<T>,
		first_asset_amount: BalanceOf<T>,
		second_asset_id: CurrencyIdOf<T>,
		second_asset_amount: BalanceOf<T>,
	) -> DispatchResult {
		if Pools::<T>::contains_key((first_asset_id, second_asset_id)) {
			Pools::<T>::insert(
				(first_asset_id, second_asset_id),
				(first_asset_amount, second_asset_amount),
			);
		} else if Pools::<T>::contains_key((second_asset_id, first_asset_id)) {
			Pools::<T>::insert(
				(second_asset_id, first_asset_id),
				(second_asset_amount, first_asset_amount),
			);
		} else {
			return Err(DispatchError::from(Error::<T>::NoSuchPool))
		}

		Ok(())
	}

	// Calculate first and second token amounts depending on liquidity amount to burn
	pub fn get_burn_amount(
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
		liquidity_asset_amount: BalanceOf<T>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		// Get token reserves and liquidity asset id
		let liquidity_asset_id = Self::get_liquidity_asset(first_asset_id, second_asset_id)?;
		let (first_asset_reserve, second_asset_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;

		ensure!(!(Self::is_pool_empty(first_asset_id, second_asset_id)?), Error::<T>::PoolIsEmpty);

		let (first_asset_amount, second_asset_amount) = Self::get_burn_amount_reserves(
			first_asset_reserve,
			second_asset_reserve,
			liquidity_asset_id,
			liquidity_asset_amount,
		)?;

		log!(
			info,
			"get_burn_amount: ({:?}, {:?}, {:?}) -> ({:?}, {:?})",
			first_asset_id,
			second_asset_id,
			liquidity_asset_amount,
			first_asset_amount,
			second_asset_amount
		);

		Ok((first_asset_amount, second_asset_amount))
	}

	pub fn get_burn_amount_reserves(
		first_asset_reserve: BalanceOf<T>,
		second_asset_reserve: BalanceOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_asset_amount: BalanceOf<T>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		// Get token reserves and liquidity asset id

		let total_liquidity_assets: BalanceOf<T> =
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into());

		// Calculate first and second token amount to be withdrawn
		ensure!(!total_liquidity_assets.is_zero(), Error::<T>::DivisionByZero);
		let first_asset_amount = multiply_by_rational_with_rounding(
			first_asset_reserve.into(),
			liquidity_asset_amount.into(),
			total_liquidity_assets.into(),
			Rounding::Down,
		)
		.map(SaturatedConversion::saturated_into)
		.ok_or(Error::<T>::UnexpectedFailure)?;
		let second_asset_amount = multiply_by_rational_with_rounding(
			second_asset_reserve.into(),
			liquidity_asset_amount.into(),
			total_liquidity_assets.into(),
			Rounding::Down,
		)
		.map(SaturatedConversion::saturated_into)
		.ok_or(Error::<T>::UnexpectedFailure)?;

		Ok((first_asset_amount, second_asset_amount))
	}

	fn settle_treasury_and_burn(
		sold_asset_id: CurrencyIdOf<T>,
		burn_amount: BalanceOf<T>,
		treasury_amount: BalanceOf<T>,
	) -> DispatchResult {
		let vault = Self::account_id();
		let mangata_id: CurrencyIdOf<T> = Self::native_token_id();
		let treasury_account: T::AccountId = Self::treasury_account_id();
		let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

		// If settling token is mangata, treasury amount is added to treasury and burn amount is burned from corresponding pool
		if sold_asset_id == mangata_id {
			// treasury_amount of MGA is already in treasury at this point

			// MGA burned from bnb_treasury_account
			// MAX: 3R 1W
			<T as Config>::Currency::burn_and_settle(
				sold_asset_id.into(),
				&bnb_treasury_account,
				burn_amount,
			)?;
		}
		//If settling token is connected to mangata, token is swapped in corresponding pool to mangata without fee
		else if Pools::<T>::contains_key((sold_asset_id, mangata_id)) ||
			Pools::<T>::contains_key((mangata_id, sold_asset_id))
		{
			// MAX: 2R (from if cond)

			// Getting token reserves
			let (input_reserve, output_reserve) =
				Pallet::<T>::get_reserves(sold_asset_id, mangata_id)?;

			// Calculating swapped mangata amount
			let settle_amount_in_mangata = Self::calculate_sell_price_no_fee(
				input_reserve,
				output_reserve,
				treasury_amount
					.checked_add(&burn_amount)
					.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?,
			)?;

			let treasury_amount_in_mangata: BalanceOf<T> = settle_amount_in_mangata
				.into()
				.checked_mul(T::TreasuryFeePercentage::get())
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
				.checked_div(
					T::TreasuryFeePercentage::get()
						.checked_add(T::BuyAndBurnFeePercentage::get())
						.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?,
				)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
				.try_into()
				.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

			let burn_amount_in_mangata: BalanceOf<T> = settle_amount_in_mangata
				.into()
				.checked_sub(treasury_amount_in_mangata.into())
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
				.try_into()
				.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

			// Apply changes in token pools, adding treasury and burn amounts of settling token, removing  treasury and burn amounts of mangata

			// MAX: 2R 1W
			Pallet::<T>::set_reserves(
				sold_asset_id,
				input_reserve.saturating_add(treasury_amount).saturating_add(burn_amount),
				mangata_id,
				output_reserve
					.saturating_sub(treasury_amount_in_mangata)
					.saturating_sub(burn_amount_in_mangata),
			)?;

			<T as Config>::Currency::transfer(
				sold_asset_id.into(),
				&treasury_account,
				&vault,
				treasury_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			<T as Config>::Currency::transfer(
				mangata_id.into(),
				&vault,
				&treasury_account,
				treasury_amount_in_mangata,
				ExistenceRequirement::KeepAlive,
			)?;

			<T as Config>::Currency::transfer(
				sold_asset_id.into(),
				&bnb_treasury_account,
				&vault,
				burn_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			// Mangata burned from pool
			<T as Config>::Currency::burn_and_settle(
				mangata_id.into(),
				&vault,
				burn_amount_in_mangata,
			)?;
		}
		// Settling token has no mangata connection, settling token is added to treasuries
		else {
			// Both treasury_amount and buy_and_burn_amount of sold_asset are in their respective treasuries
		}
		Ok(())
	}

	fn account_id() -> T::AccountId {
		PALLET_ID.into_account_truncating()
	}

	fn treasury_account_id() -> T::AccountId {
		T::TreasuryPalletId::get().into_account_truncating()
	}

	fn bnb_treasury_account_id() -> T::AccountId {
		T::TreasuryPalletId::get().into_sub_account_truncating(T::BnbTreasurySubAccDerive::get())
	}

	fn native_token_id() -> CurrencyIdOf<T> {
		<T as Config>::NativeCurrencyId::get()
	}

	fn calculate_initial_liquidity(
		first_asset_amount: BalanceOf<T>,
		second_asset_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let initial_liquidity = first_asset_amount
			.checked_div(&2_u32.into())
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			.checked_add(
				&second_asset_amount
					.checked_div(&2_u32.into())
					.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?,
			)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		return Ok(if initial_liquidity == BalanceOf::<T>::zero() {
			BalanceOf::<T>::one()
		} else {
			initial_liquidity
		})
	}

	fn is_pool_empty(
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
	) -> Result<bool, DispatchError> {
		let liquidity_asset_id = Pallet::<T>::get_liquidity_asset(first_asset_id, second_asset_id)?;
		let total_liquidity_assets: BalanceOf<T> =
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into());

		return Ok(total_liquidity_assets.is_zero())
	}
}

impl<T: Config> PreValidateSwaps<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>> for Pallet<T> {
	fn pre_validate_sell_asset(
		sender: &T::AccountId,
		sold_asset_id: CurrencyIdOf<T>,
		bought_asset_id: CurrencyIdOf<T>,
		sold_asset_amount: BalanceOf<T>,
		_min_amount_out: BalanceOf<T>,
	) -> Result<
		(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
		DispatchError,
	> {
		ensure!(
			!T::MaintenanceStatusProvider::is_maintenance(),
			Error::<T>::TradingBlockedByMaintenanceMode
		);

		// Ensure not selling zero amount
		ensure!(!sold_asset_amount.is_zero(), Error::<T>::ZeroAmount,);

		ensure!(
			!T::DisabledTokens::contains(&sold_asset_id) &&
				!T::DisabledTokens::contains(&bought_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		ensure!(!(Self::is_pool_empty(sold_asset_id, bought_asset_id)?), Error::<T>::PoolIsEmpty);

		let buy_and_burn_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::BuyAndBurnFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let treasury_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::TreasuryFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let pool_fee_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::PoolFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let total_fees: BalanceOf<T> = buy_and_burn_amount
			.checked_add(&treasury_amount)
			.and_then(|v| v.checked_add(&pool_fee_amount))
			.ok_or(Error::<T>::MathOverflow)?;

		// MAX: 2R
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_asset_id, bought_asset_id)?;

		ensure!(input_reserve.checked_add(&sold_asset_amount).is_some(), Error::<T>::MathOverflow);

		// Calculate bought asset amount to be received by paying sold asset amount
		let bought_asset_amount =
			Pallet::<T>::calculate_sell_price(input_reserve, output_reserve, sold_asset_amount)?;

		// Ensure user has enough tokens to sell
		<T as Config>::Currency::ensure_can_withdraw(
			sold_asset_id.into(),
			sender,
			total_fees,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		Ok((
			buy_and_burn_amount,
			treasury_amount,
			pool_fee_amount,
			input_reserve,
			output_reserve,
			bought_asset_amount,
		))
	}

	/// We only validate the first atomic swap's ability to accept fees
	fn pre_validate_multiswap_sell_asset(
		sender: &T::AccountId,
		swap_token_list: Vec<CurrencyIdOf<T>>,
		sold_asset_amount: BalanceOf<T>,
		_min_amount_out: BalanceOf<T>,
	) -> Result<
		(
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			CurrencyIdOf<T>,
		),
		DispatchError,
	> {
		ensure!(
			!T::MaintenanceStatusProvider::is_maintenance(),
			Error::<T>::TradingBlockedByMaintenanceMode
		);
		ensure!(swap_token_list.len() > 2_usize, Error::<T>::MultiswapShouldBeAtleastTwoHops);
		let sold_asset_id =
			*swap_token_list.get(0).ok_or(Error::<T>::MultiswapShouldBeAtleastTwoHops)?;
		let bought_asset_id =
			*swap_token_list.get(1).ok_or(Error::<T>::MultiswapShouldBeAtleastTwoHops)?;

		// Ensure not selling zero amount
		ensure!(!sold_asset_amount.is_zero(), Error::<T>::ZeroAmount,);

		let atomic_pairs: Vec<(CurrencyIdOf<T>, CurrencyIdOf<T>)> = swap_token_list
			.clone()
			.into_iter()
			.zip(swap_token_list.clone().into_iter().skip(1))
			.collect();

		for (x, y) in atomic_pairs.iter() {
			ensure!(!(Self::is_pool_empty(*x, *y)?), Error::<T>::PoolIsEmpty);

			if x == y {
				return Err(Error::<T>::MultiSwapCantHaveSameTokenConsequetively.into())
			}
		}

		ensure!(
			!T::DisabledTokens::contains(&sold_asset_id) &&
				!T::DisabledTokens::contains(&bought_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		let buy_and_burn_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::BuyAndBurnFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let treasury_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::TreasuryFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let pool_fee_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::PoolFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let total_fees = buy_and_burn_amount
			.checked_add(&treasury_amount)
			.and_then(|v| v.checked_add(&pool_fee_amount))
			.ok_or(Error::<T>::MathOverflow)?;

		// Get token reserves

		// MAX: 2R
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_asset_id, bought_asset_id)?;

		ensure!(input_reserve.checked_add(&pool_fee_amount).is_some(), Error::<T>::MathOverflow);

		// Ensure user has enough tokens to sell
		<T as Config>::Currency::ensure_can_withdraw(
			sold_asset_id.into(),
			sender,
			total_fees,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		Ok((
			buy_and_burn_amount,
			treasury_amount,
			pool_fee_amount,
			input_reserve,
			output_reserve,
			sold_asset_id,
			bought_asset_id,
		))
	}

	fn pre_validate_buy_asset(
		sender: &T::AccountId,
		sold_asset_id: CurrencyIdOf<T>,
		bought_asset_id: CurrencyIdOf<T>,
		bought_asset_amount: BalanceOf<T>,
		_max_amount_in: BalanceOf<T>,
	) -> Result<
		(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
		DispatchError,
	> {
		ensure!(
			!T::MaintenanceStatusProvider::is_maintenance(),
			Error::<T>::TradingBlockedByMaintenanceMode
		);

		ensure!(
			!T::DisabledTokens::contains(&sold_asset_id) &&
				!T::DisabledTokens::contains(&bought_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		ensure!(!(Self::is_pool_empty(sold_asset_id, bought_asset_id)?), Error::<T>::PoolIsEmpty);

		// Get token reserves
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_asset_id, bought_asset_id)?;

		// Ensure there are enough tokens in reserves
		ensure!(output_reserve > bought_asset_amount, Error::<T>::NotEnoughReserve,);

		// Ensure not buying zero amount
		ensure!(!bought_asset_amount.is_zero(), Error::<T>::ZeroAmount,);

		// Calculate amount to be paid from bought amount
		let sold_asset_amount =
			Pallet::<T>::calculate_buy_price(input_reserve, output_reserve, bought_asset_amount)?;

		let buy_and_burn_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::BuyAndBurnFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let treasury_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::TreasuryFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let pool_fee_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			sold_asset_amount.into(),
			T::PoolFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		// for future implementation of min fee if necessary
		// let min_fee: u128 = 0;
		// if buy_and_burn_amount + treasury_amount + pool_fee_amount < min_fee {
		//     buy_and_burn_amount = min_fee * Self::total_fee() / T::BuyAndBurnFeePercentage::get();
		//     treasury_amount = min_fee * Self::total_fee() / T::TreasuryFeePercentage::get();
		//     pool_fee_amount = min_fee - buy_and_burn_amount - treasury_amount;
		// }

		ensure!(input_reserve.checked_add(&sold_asset_amount).is_some(), Error::<T>::MathOverflow);

		// Ensure user has enough tokens to sell
		<T as Config>::Currency::ensure_can_withdraw(
			sold_asset_id.into(),
			sender,
			sold_asset_amount,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		Ok((
			buy_and_burn_amount,
			treasury_amount,
			pool_fee_amount,
			input_reserve,
			output_reserve,
			sold_asset_amount,
		))
	}

	/// We only validate the first atomic swap's ability to accept fees
	fn pre_validate_multiswap_buy_asset(
		sender: &T::AccountId,
		swap_token_list: Vec<CurrencyIdOf<T>>,
		final_bought_asset_amount: BalanceOf<T>,
		max_amount_in: BalanceOf<T>,
	) -> Result<
		(
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			CurrencyIdOf<T>,
			CurrencyIdOf<T>,
		),
		DispatchError,
	> {
		ensure!(
			!T::MaintenanceStatusProvider::is_maintenance(),
			Error::<T>::TradingBlockedByMaintenanceMode
		);
		ensure!(swap_token_list.len() > 2_usize, Error::<T>::MultiswapShouldBeAtleastTwoHops);
		// Ensure not buying zero amount
		ensure!(!final_bought_asset_amount.is_zero(), Error::<T>::ZeroAmount,);
		ensure!(!max_amount_in.is_zero(), Error::<T>::ZeroAmount,);

		// Unwraps are fine due to above ensure
		let sold_asset_id =
			*swap_token_list.get(0).ok_or(Error::<T>::MultiswapShouldBeAtleastTwoHops)?;
		let bought_asset_id =
			*swap_token_list.get(1).ok_or(Error::<T>::MultiswapShouldBeAtleastTwoHops)?;

		ensure!(
			!T::DisabledTokens::contains(&sold_asset_id) &&
				!T::DisabledTokens::contains(&bought_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		// Cannot use multiswap twice on the same pool
		let atomic_pairs: Vec<(CurrencyIdOf<T>, CurrencyIdOf<T>)> = swap_token_list
			.clone()
			.into_iter()
			.zip(swap_token_list.clone().into_iter().skip(1))
			.collect();

		let mut atomic_pairs_hashset = BTreeSet::new();

		for (x, y) in atomic_pairs.iter() {
			ensure!(!(Self::is_pool_empty(*x, *y)?), Error::<T>::PoolIsEmpty);

			if x == y {
				return Err(Error::<T>::MultiSwapCantHaveSameTokenConsequetively.into())
			} else if x > y {
				if !atomic_pairs_hashset.insert((x, y)) {
					return Err(Error::<T>::MultiBuyAssetCantHaveSamePoolAtomicSwaps.into())
				};
			} else
			// x < y
			{
				if !atomic_pairs_hashset.insert((y, x)) {
					return Err(Error::<T>::MultiBuyAssetCantHaveSamePoolAtomicSwaps.into())
				};
			}
		}

		// Get token reserves
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_asset_id, bought_asset_id)?;

		let buy_and_burn_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			max_amount_in.into(),
			T::BuyAndBurnFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let treasury_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			max_amount_in.into(),
			T::TreasuryFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let pool_fee_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			max_amount_in.into(),
			T::PoolFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or(Error::<T>::MathOverflow)?
		.try_into()
		.map_err(|_| Error::<T>::MathOverflow)?;

		let total_fees = buy_and_burn_amount
			.checked_add(&treasury_amount)
			.and_then(|v| v.checked_add(&pool_fee_amount))
			.ok_or(Error::<T>::MathOverflow)?;

		ensure!(input_reserve.checked_add(&pool_fee_amount).is_some(), Error::<T>::MathOverflow);

		// Ensure user has enough tokens to sell
		<T as Config>::Currency::ensure_can_withdraw(
			sold_asset_id.into(),
			sender,
			total_fees,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		Ok((
			buy_and_burn_amount,
			treasury_amount,
			pool_fee_amount,
			input_reserve,
			output_reserve,
			sold_asset_id,
			bought_asset_id,
		))
	}
}

impl<T: Config> XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>> for Pallet<T> {
	fn create_pool(
		sender: T::AccountId,
		first_asset_id: CurrencyIdOf<T>,
		first_asset_amount: BalanceOf<T>,
		second_asset_id: CurrencyIdOf<T>,
		second_asset_amount: BalanceOf<T>,
	) -> DispatchResult {
		let vault: T::AccountId = Pallet::<T>::account_id();

		// Ensure pool is not created with zero amount
		ensure!(
			!first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
			Error::<T>::ZeroAmount,
		);

		// Ensure pool does not exists yet
		ensure!(
			!Pools::<T>::contains_key((first_asset_id, second_asset_id)),
			Error::<T>::PoolAlreadyExists,
		);

		// Ensure pool does not exists yet
		ensure!(
			!Pools::<T>::contains_key((second_asset_id, first_asset_id)),
			Error::<T>::PoolAlreadyExists,
		);

		// Ensure user has enough withdrawable tokens to create pool in amounts required

		<T as Config>::Currency::ensure_can_withdraw(
			first_asset_id.into(),
			&sender,
			first_asset_amount,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		<T as Config>::Currency::ensure_can_withdraw(
			second_asset_id.into(),
			&sender,
			second_asset_amount,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		// Ensure pool is not created with same token in pair
		ensure!(first_asset_id != second_asset_id, Error::<T>::SameAsset,);

		// Liquidity token amount calculation
		let initial_liquidity =
			Pallet::<T>::calculate_initial_liquidity(first_asset_amount, second_asset_amount)?;

		Pools::<T>::insert(
			(first_asset_id, second_asset_id),
			(first_asset_amount, second_asset_amount),
		);

		// Pools::insert((second_asset_id, first_asset_id), second_asset_amount);

		// Moving tokens from user to vault
		<T as Config>::Currency::transfer(
			first_asset_id.into(),
			&sender,
			&vault,
			first_asset_amount,
			ExistenceRequirement::AllowDeath,
		)?;

		<T as Config>::Currency::transfer(
			second_asset_id.into(),
			&sender,
			&vault,
			second_asset_amount,
			ExistenceRequirement::AllowDeath,
		)?;

		// Creating new liquidity token and transfering it to user
		let liquidity_asset_id: CurrencyIdOf<T> =
			<T as Config>::Currency::create(&sender, initial_liquidity)
				.map_err(|_| Error::<T>::LiquidityTokenCreationFailed)?
				.into();

		// Adding info about liquidity asset
		LiquidityAssets::<T>::insert((first_asset_id, second_asset_id), Some(liquidity_asset_id));
		LiquidityPools::<T>::insert(liquidity_asset_id, Some((first_asset_id, second_asset_id)));

		log!(
			info,
			"create_pool: ({:?}, {:?}, {:?}, {:?}, {:?}) -> ({:?}, {:?})",
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
			"pool-state: [({:?}, {:?}) -> {:?}, ({:?}, {:?}) -> {:?}]",
			first_asset_id,
			second_asset_id,
			first_asset_amount,
			second_asset_id,
			first_asset_id,
			second_asset_amount
		);
		// This, will and should, never fail
		Pallet::<T>::set_liquidity_asset_info(liquidity_asset_id, first_asset_id, second_asset_id)?;

		Pallet::<T>::deposit_event(Event::PoolCreated(
			sender,
			first_asset_id,
			first_asset_amount,
			second_asset_id,
			second_asset_amount,
		));

		Ok(())
	}

	// To put it comprehensively the only reason that the user should lose out on swap fee
	// is if the mistake is theirs, which in the context of swaps is bad slippage besides pre_validation.
	// In this implementation, once pre_validation passes the swap fee mechanism that follows should suceed.
	// And if the function fails beyond pre_validation but not on slippage then the user is free from blame.
	// Further internals calls, might determine the slippage themselves before calling this swap function,
	// in which case again the user must not be charged the swap fee.
	fn sell_asset(
		sender: T::AccountId,
		sold_asset_id: CurrencyIdOf<T>,
		bought_asset_id: CurrencyIdOf<T>,
		sold_asset_amount: BalanceOf<T>,
		min_amount_out: BalanceOf<T>,
		err_upon_bad_slippage: bool,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (
			buy_and_burn_amount,
			treasury_amount,
			pool_fee_amount,
			input_reserve,
			output_reserve,
			bought_asset_amount,
		) = <Pallet<T> as PreValidateSwaps<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::pre_validate_sell_asset(
			&sender,
			sold_asset_id,
			bought_asset_id,
			sold_asset_amount,
			min_amount_out,
		)?;

		let vault = Pallet::<T>::account_id();
		let treasury_account: T::AccountId = Self::treasury_account_id();
		let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

		// Transfer of fees, before tx can fail on min amount out
		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&vault,
			pool_fee_amount,
			ExistenceRequirement::KeepAlive,
		)?;

		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&treasury_account,
			treasury_amount,
			ExistenceRequirement::KeepAlive,
		)?;

		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&bnb_treasury_account,
			buy_and_burn_amount,
			ExistenceRequirement::KeepAlive,
		)?;

		// Add pool fee to pool
		// 2R 1W
		Pallet::<T>::set_reserves(
			sold_asset_id,
			input_reserve.saturating_add(pool_fee_amount),
			bought_asset_id,
			output_reserve,
		)?;

		// Ensure bought token amount is higher then requested minimal amount
		if bought_asset_amount >= min_amount_out {
			// Transfer the rest of sold token amount from user to vault and bought token amount from vault to user
			<T as Config>::Currency::transfer(
				sold_asset_id.into(),
				&sender,
				&vault,
				sold_asset_amount
					.checked_sub(
						&buy_and_burn_amount
							.checked_add(&treasury_amount)
							.and_then(|v| v.checked_add(&pool_fee_amount))
							.ok_or_else(|| DispatchError::from(Error::<T>::SoldAmountTooLow))?,
					)
					.ok_or_else(|| DispatchError::from(Error::<T>::SoldAmountTooLow))?,
				ExistenceRequirement::KeepAlive,
			)?;

			<T as Config>::Currency::transfer(
				bought_asset_id.into(),
				&vault,
				&sender,
				bought_asset_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			// Apply changes in token pools, adding sold amount and removing bought amount
			// Neither should fall to zero let alone underflow, due to how pool destruction works
			// Won't overflow due to earlier ensure
			let input_reserve_updated = input_reserve.saturating_add(
				sold_asset_amount
					.checked_sub(&treasury_amount)
					.and_then(|v| v.checked_sub(&buy_and_burn_amount))
					.ok_or_else(|| DispatchError::from(Error::<T>::SoldAmountTooLow))?,
			);
			let output_reserve_updated = output_reserve.saturating_sub(bought_asset_amount);

			// MAX 2R 1W
			Pallet::<T>::set_reserves(
				sold_asset_id,
				input_reserve_updated,
				bought_asset_id,
				output_reserve_updated,
			)?;

			log!(
				info,
				"sell_asset: ({:?}, {:?}, {:?}, {:?}, {:?}) -> {:?}",
				sender,
				sold_asset_id,
				bought_asset_id,
				sold_asset_amount,
				min_amount_out,
				bought_asset_amount
			);

			log!(
				info,
				"pool-state: [({:?}, {:?}) -> {:?}, ({:?}, {:?}) -> {:?}]",
				sold_asset_id,
				bought_asset_id,
				input_reserve_updated,
				bought_asset_id,
				sold_asset_id,
				output_reserve_updated
			);

			Pallet::<T>::deposit_event(Event::AssetsSwapped(
				sender.clone(),
				vec![sold_asset_id, bought_asset_id],
				sold_asset_amount,
				bought_asset_amount,
			));
		}

		// Settle tokens which goes to treasury and for buy and burn purpose
		Pallet::<T>::settle_treasury_and_burn(sold_asset_id, buy_and_burn_amount, treasury_amount)?;

		if bought_asset_amount < min_amount_out {
			if err_upon_bad_slippage {
				return Err(DispatchError::from(Error::<T>::InsufficientOutputAmount))
			} else {
				Pallet::<T>::deposit_event(Event::SellAssetFailedDueToSlippage(
					sender,
					sold_asset_id,
					sold_asset_amount,
					bought_asset_id,
					bought_asset_amount,
					min_amount_out,
				));
				return Ok(Default::default())
			}
		}

		Ok(bought_asset_amount)
	}

	fn do_multiswap_sell_asset(
		sender: T::AccountId,
		swap_token_list: Vec<CurrencyIdOf<T>>,
		sold_asset_amount: BalanceOf<T>,
		min_amount_out: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		frame_support::storage::with_storage_layer(|| -> Result<BalanceOf<T>, DispatchError> {
			// Ensure user has enough tokens to sell
			<T as Config>::Currency::ensure_can_withdraw(
				// Naked unwrap is fine due to pre validation len check
				{ *swap_token_list.get(0).ok_or(Error::<T>::MultiswapShouldBeAtleastTwoHops)? }
					.into(),
				&sender,
				sold_asset_amount,
				WithdrawReasons::all(),
				Default::default(),
			)
			.or(Err(Error::<T>::NotEnoughAssets))?;

			// pre_validate has already confirmed that swap_token_list.len()>1
			let atomic_pairs: Vec<(CurrencyIdOf<T>, CurrencyIdOf<T>)> = swap_token_list
				.clone()
				.into_iter()
				.zip(swap_token_list.clone().into_iter().skip(1))
				.collect();

			let mut atomic_sold_asset_amount = sold_asset_amount;
			let mut atomic_bought_asset_amount = BalanceOf::<T>::zero();

			for (atomic_sold_asset, atomic_bought_asset) in atomic_pairs.iter() {
				atomic_bought_asset_amount = <Self as XykFunctionsTrait<
					T::AccountId,
					BalanceOf<T>,
					CurrencyIdOf<T>,
				>>::sell_asset(
					sender.clone(),
					*atomic_sold_asset,
					*atomic_bought_asset,
					atomic_sold_asset_amount,
					BalanceOf::<T>::zero(),
					// We using most possible slippage so this should be irrelevant
					true,
				)?;

				// Prep the next loop
				atomic_sold_asset_amount = atomic_bought_asset_amount;
			}

			// fail/error and revert if bad final slippage
			if atomic_bought_asset_amount < min_amount_out {
				return Err(Error::<T>::InsufficientOutputAmount.into())
			} else {
				return Ok(atomic_bought_asset_amount)
			}
		})
	}

	fn multiswap_sell_asset(
		sender: T::AccountId,
		swap_token_list: Vec<CurrencyIdOf<T>>,
		sold_asset_amount: BalanceOf<T>,
		min_amount_out: BalanceOf<T>,
		_err_upon_bad_slippage: bool,
		_err_upon_non_slippage_fail: bool,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (
			fee_swap_buy_and_burn_amount,
			fee_swap_treasury_amount,
			fee_swap_pool_fee_amount,
			fee_swap_input_reserve,
			fee_swap_output_reserve,
			fee_swap_sold_asset_id,
			fee_swap_bought_asset_id,
		) = <Pallet<T> as PreValidateSwaps<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::pre_validate_multiswap_sell_asset(
			&sender,
			swap_token_list.clone(),
			sold_asset_amount,
			min_amount_out,
		)?;

		// First execute all atomic swaps in a storage layer
		// And then the finally bought amount is compared
		// The bool in error represents if the fail is due to bad final slippage
		match <Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::do_multiswap_sell_asset(
			sender.clone(),
			swap_token_list.clone(),
			sold_asset_amount,
			min_amount_out,
		) {
			Ok(bought_asset_amount) => {
				Pallet::<T>::deposit_event(Event::AssetsSwapped(
					sender.clone(),
					swap_token_list.clone(),
					sold_asset_amount,
					bought_asset_amount,
				));
				TotalNumberOfSwaps::<T>::mutate(|v|{*v=v.saturating_add(One::one())});
				Ok(bought_asset_amount)
			},
			Err(e) => {
				// Charge fee

				let vault = Pallet::<T>::account_id();
				let treasury_account: T::AccountId = Self::treasury_account_id();
				let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

				// Transfer of fees, before tx can fail on min amount out
				<T as Config>::Currency::transfer(
					fee_swap_sold_asset_id,
					&sender,
					&vault,
					fee_swap_pool_fee_amount,
					ExistenceRequirement::KeepAlive,
				)?;

				<T as Config>::Currency::transfer(
					fee_swap_sold_asset_id,
					&sender,
					&treasury_account,
					fee_swap_treasury_amount,
					ExistenceRequirement::KeepAlive,
				)?;

				<T as Config>::Currency::transfer(
					fee_swap_sold_asset_id,
					&sender,
					&bnb_treasury_account,
					fee_swap_buy_and_burn_amount,
					ExistenceRequirement::KeepAlive,
				)?;

				// Add pool fee to pool
				// 2R 1W
				Pallet::<T>::set_reserves(
					fee_swap_sold_asset_id,
					fee_swap_input_reserve.saturating_add(fee_swap_pool_fee_amount),
					fee_swap_bought_asset_id,
					fee_swap_output_reserve,
				)?;

				// Settle tokens which goes to treasury and for buy and burn purpose
				Pallet::<T>::settle_treasury_and_burn(
					fee_swap_sold_asset_id,
					fee_swap_buy_and_burn_amount,
					fee_swap_treasury_amount,
				)?;

				if let DispatchError::Module(module_err) = e {
					Pallet::<T>::deposit_event(Event::MultiSwapAssetFailedOnAtomicSwap(
						sender.clone(),
						swap_token_list.clone(),
						sold_asset_amount,
						module_err,
					));
					Ok(Default::default())
				} else {
					Err(DispatchError::from(Error::<T>::UnexpectedFailure))
				}
			},
		}
	}

	// To put it comprehensively the only reason that the user should lose out on swap fee
	// is if the mistake is theirs, which in the context of swaps is bad slippage besides pre_validation.
	// In this implementation, once pre_validation passes the swap fee mechanism that follows should suceed.
	// And if the function fails beyond pre_validation but not on slippage then the user is free from blame.
	// Further internals calls, might determine the slippage themselves before calling this swap function,
	// in which case again the user must not be charged the swap fee.
	fn buy_asset(
		sender: T::AccountId,
		sold_asset_id: CurrencyIdOf<T>,
		bought_asset_id: CurrencyIdOf<T>,
		bought_asset_amount: BalanceOf<T>,
		max_amount_in: BalanceOf<T>,
		err_upon_bad_slippage: bool,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (
			buy_and_burn_amount,
			treasury_amount,
			pool_fee_amount,
			input_reserve,
			output_reserve,
			sold_asset_amount,
		) = <Pallet<T> as PreValidateSwaps<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::pre_validate_buy_asset(
			&sender,
			sold_asset_id,
			bought_asset_id,
			bought_asset_amount,
			max_amount_in,
		)?;

		let vault = Pallet::<T>::account_id();
		let treasury_account: T::AccountId = Self::treasury_account_id();
		let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

		// Transfer of fees, before tx can fail on min amount out
		<T as Config>::Currency::transfer(
			sold_asset_id,
			&sender,
			&vault,
			pool_fee_amount,
			ExistenceRequirement::KeepAlive,
		)?;

		<T as Config>::Currency::transfer(
			sold_asset_id,
			&sender,
			&treasury_account,
			treasury_amount,
			ExistenceRequirement::KeepAlive,
		)?;

		<T as Config>::Currency::transfer(
			sold_asset_id,
			&sender,
			&bnb_treasury_account,
			buy_and_burn_amount,
			ExistenceRequirement::KeepAlive,
		)?;

		// Add pool fee to pool
		Pallet::<T>::set_reserves(
			sold_asset_id,
			input_reserve.saturating_add(pool_fee_amount),
			bought_asset_id,
			output_reserve,
		)?;

		// Ensure paid amount is less then maximum allowed price
		if sold_asset_amount <= max_amount_in {
			// Transfer sold token amount from user to vault and bought token amount from vault to user
			<T as Config>::Currency::transfer(
				sold_asset_id.into(),
				&sender,
				&vault,
				sold_asset_amount
					.checked_sub(
						&buy_and_burn_amount
							.checked_add(&treasury_amount)
							.and_then(|v| v.checked_add(&pool_fee_amount))
							.ok_or_else(|| DispatchError::from(Error::<T>::SoldAmountTooLow))?,
					)
					.ok_or_else(|| DispatchError::from(Error::<T>::SoldAmountTooLow))?,
				ExistenceRequirement::KeepAlive,
			)?;
			<T as Config>::Currency::transfer(
				bought_asset_id,
				&vault,
				&sender,
				bought_asset_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			// Apply changes in token pools, adding sold amount and removing bought amount
			// Neither should fall to zero let alone underflow, due to how pool destruction works
			// Won't overflow due to earlier ensure
			let input_reserve_updated = input_reserve.saturating_add(
				sold_asset_amount
					.checked_sub(&treasury_amount)
					.and_then(|v| v.checked_sub(&buy_and_burn_amount))
					.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?,
			);

			let output_reserve_updated = output_reserve.saturating_sub(bought_asset_amount);
			Pallet::<T>::set_reserves(
				sold_asset_id,
				input_reserve_updated,
				bought_asset_id,
				output_reserve_updated,
			)?;

			log!(
				info,
				"buy_asset: ({:?}, {:?}, {:?}, {:?}, {:?}) -> {:?}",
				sender,
				sold_asset_id,
				bought_asset_id,
				bought_asset_amount,
				max_amount_in,
				sold_asset_amount
			);

			log!(
				info,
				"pool-state: [({:?}, {:?}) -> {:?}, ({:?}, {:?}) -> {:?}]",
				sold_asset_id,
				bought_asset_id,
				input_reserve_updated,
				bought_asset_id,
				sold_asset_id,
				output_reserve_updated
			);

			Pallet::<T>::deposit_event(Event::AssetsSwapped(
				sender.clone(),
				vec![sold_asset_id, bought_asset_id],
				sold_asset_amount,
				bought_asset_amount,
			));
		}
		// Settle tokens which goes to treasury and for buy and burn purpose
		Pallet::<T>::settle_treasury_and_burn(sold_asset_id, buy_and_burn_amount, treasury_amount)?;

		if sold_asset_amount > max_amount_in {
			if err_upon_bad_slippage {
				return Err(DispatchError::from(Error::<T>::InsufficientInputAmount))
			} else {
				Pallet::<T>::deposit_event(Event::BuyAssetFailedDueToSlippage(
					sender,
					sold_asset_id,
					sold_asset_amount,
					bought_asset_id,
					bought_asset_amount,
					max_amount_in,
				));
			}
		}

		Ok(sold_asset_amount)
	}

	fn do_multiswap_buy_asset(
		sender: T::AccountId,
		swap_token_list: Vec<CurrencyIdOf<T>>,
		bought_asset_amount: BalanceOf<T>,
		max_amount_in: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		frame_support::storage::with_storage_layer(|| -> Result<BalanceOf<T>, DispatchError> {
			// pre_validate has already confirmed that swap_token_list.len()>1
			let atomic_pairs: Vec<(CurrencyIdOf<T>, CurrencyIdOf<T>)> = swap_token_list
				.clone()
				.into_iter()
				.zip(swap_token_list.clone().into_iter().skip(1))
				.collect();

			let mut atomic_sold_asset_amount = BalanceOf::<T>::zero();
			let mut atomic_bought_asset_amount = bought_asset_amount;

			let mut atomic_swap_buy_amounts_rev: Vec<BalanceOf<T>> = Default::default();
			// Calc
			// We can do this using calculate_buy_price_id chain due to the check in pre_validation
			// that ensures that no pool is touched twice. So the reserves in question are consistent
			for (atomic_sold_asset, atomic_bought_asset) in atomic_pairs.iter().rev() {
				atomic_sold_asset_amount = Self::calculate_buy_price_id(
					*atomic_sold_asset,
					*atomic_bought_asset,
					atomic_bought_asset_amount,
				)?;

				atomic_swap_buy_amounts_rev.push(atomic_bought_asset_amount);
				// Prep the next loop
				atomic_bought_asset_amount = atomic_sold_asset_amount;
			}

			ensure!(atomic_sold_asset_amount <= max_amount_in, Error::<T>::InsufficientInputAmount);

			// Ensure user has enough tokens to sell
			<T as Config>::Currency::ensure_can_withdraw(
				// Naked unwrap is fine due to pre validation len check
				{ *swap_token_list.get(0).ok_or(Error::<T>::MultiswapShouldBeAtleastTwoHops)? }
					.into(),
				&sender,
				atomic_sold_asset_amount,
				WithdrawReasons::all(),
				Default::default(),
			)
			.or(Err(Error::<T>::NotEnoughAssets))?;

			// Execute here
			for ((atomic_sold_asset, atomic_bought_asset), atomic_swap_buy_amount) in
				atomic_pairs.iter().zip(atomic_swap_buy_amounts_rev.iter().rev())
			{
				let _ = <Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::buy_asset(
					sender.clone(),
					*atomic_sold_asset,
					*atomic_bought_asset,
					*atomic_swap_buy_amount,
					BalanceOf::<T>::max_value(),
					// We using most possible slippage so this should be irrelevant
					true,
				)?;
			}

			return Ok(atomic_sold_asset_amount)
		})
	}

	fn multiswap_buy_asset(
		sender: T::AccountId,
		swap_token_list: Vec<CurrencyIdOf<T>>,
		bought_asset_amount: BalanceOf<T>,
		max_amount_in: BalanceOf<T>,
		_err_upon_bad_slippage: bool,
		_err_upon_non_slippage_fail: bool,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (
			fee_swap_buy_and_burn_amount,
			fee_swap_treasury_amount,
			fee_swap_pool_fee_amount,
			fee_swap_input_reserve,
			fee_swap_output_reserve,
			fee_swap_sold_asset_id,
			fee_swap_bought_asset_id,
		) = <Pallet<T> as PreValidateSwaps<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::pre_validate_multiswap_buy_asset(
			&sender,
			swap_token_list.clone(),
			bought_asset_amount,
			max_amount_in,
		)?;

		// First execute all atomic swaps in a storage layer
		// And then the finally sold amount is compared
		// The bool in error represents if the fail is due to bad final slippage
		match <Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::do_multiswap_buy_asset(
			sender.clone(),
			swap_token_list.clone(),
			bought_asset_amount,
			max_amount_in,
		) {
			Ok(sold_asset_amount) => {
				Pallet::<T>::deposit_event(Event::AssetsSwapped(
					sender.clone(),
					swap_token_list.clone(),
					sold_asset_amount,
					bought_asset_amount,
				));
				TotalNumberOfSwaps::<T>::mutate(|v|{*v=v.saturating_add(One::one())});
				Ok(sold_asset_amount)
			},
			Err(e) => {
				let vault = Pallet::<T>::account_id();
				let treasury_account: T::AccountId = Self::treasury_account_id();
				let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

				// Transfer of fees, before tx can fail on min amount out
				<T as Config>::Currency::transfer(
					fee_swap_sold_asset_id,
					&sender,
					&vault,
					fee_swap_pool_fee_amount,
					ExistenceRequirement::KeepAlive,
				)?;

				<T as Config>::Currency::transfer(
					fee_swap_sold_asset_id,
					&sender,
					&treasury_account,
					fee_swap_treasury_amount,
					ExistenceRequirement::KeepAlive,
				)?;

				<T as Config>::Currency::transfer(
					fee_swap_sold_asset_id,
					&sender,
					&bnb_treasury_account,
					fee_swap_buy_and_burn_amount,
					ExistenceRequirement::KeepAlive,
				)?;

				// Add pool fee to pool
				// 2R 1W
				Pallet::<T>::set_reserves(
					fee_swap_sold_asset_id,
					fee_swap_input_reserve.saturating_add(fee_swap_pool_fee_amount),
					fee_swap_bought_asset_id,
					fee_swap_output_reserve,
				)?;

				// Settle tokens which goes to treasury and for buy and burn purpose
				Pallet::<T>::settle_treasury_and_burn(
					fee_swap_sold_asset_id,
					fee_swap_buy_and_burn_amount,
					fee_swap_treasury_amount,
				)?;

				if let DispatchError::Module(module_err) = e {
					Pallet::<T>::deposit_event(Event::MultiSwapAssetFailedOnAtomicSwap(
						sender.clone(),
						swap_token_list.clone(),
						bought_asset_amount,
						module_err,
					));
					Ok(Default::default())
				} else {
					Err(DispatchError::from(Error::<T>::UnexpectedFailure))
				}
			},
		}
	}

	fn mint_liquidity(
		sender: T::AccountId,
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
		first_asset_amount: BalanceOf<T>,
		expected_second_asset_amount: BalanceOf<T>,
		activate_minted_liquidity: bool,
	) -> Result<(CurrencyIdOf<T>, BalanceOf<T>), DispatchError> {
		let vault = Pallet::<T>::account_id();

		// Ensure pool exists
		ensure!(
			(LiquidityAssets::<T>::contains_key((first_asset_id, second_asset_id)) ||
				LiquidityAssets::<T>::contains_key((second_asset_id, first_asset_id))),
			Error::<T>::NoSuchPool,
		);

		// Get liquidity token id
		let liquidity_asset_id = Pallet::<T>::get_liquidity_asset(first_asset_id, second_asset_id)?;

		// Get token reserves
		let (first_asset_reserve, second_asset_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;
		let total_liquidity_assets: BalanceOf<T> =
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into());

		// The pool is empty and we are basically creating a new pool and reusing the existing one
		let second_asset_amount = if !(first_asset_reserve.is_zero() &&
			second_asset_reserve.is_zero()) &&
			!total_liquidity_assets.is_zero()
		{
			// Calculation of required second asset amount and received liquidity token amount
			ensure!(!first_asset_reserve.is_zero(), Error::<T>::DivisionByZero);

			multiply_by_rational_with_rounding(
				first_asset_amount.into(),
				second_asset_reserve.into(),
				first_asset_reserve.into(),
				Rounding::Down,
			)
			.ok_or(Error::<T>::UnexpectedFailure)?
			.checked_add(1)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			.try_into()
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?
		} else {
			expected_second_asset_amount
		};

		ensure!(
			second_asset_amount <= expected_second_asset_amount,
			Error::<T>::SecondAssetAmountExceededExpectations,
		);

		// Ensure minting amounts are not zero
		ensure!(
			!first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
			Error::<T>::ZeroAmount,
		);

		// We calculate the required liquidity token amount and also validate asset amounts
		let liquidity_assets_minted = if total_liquidity_assets.is_zero() {
			Pallet::<T>::calculate_initial_liquidity(first_asset_amount, second_asset_amount)?
		} else {
			multiply_by_rational_with_rounding(
				first_asset_amount.into(),
				total_liquidity_assets.into(),
				first_asset_reserve.into(),
				Rounding::Down,
			)
			.ok_or(Error::<T>::UnexpectedFailure)?
			.try_into()
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?
		};

		// Ensure user has enough withdrawable tokens to create pool in amounts required

		<T as Config>::Currency::ensure_can_withdraw(
			first_asset_id.into(),
			&sender,
			first_asset_amount,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		<T as Config>::Currency::ensure_can_withdraw(
			second_asset_id,
			&sender,
			second_asset_amount,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		// Transfer of token amounts from user to vault
		<T as Config>::Currency::transfer(
			first_asset_id,
			&sender,
			&vault,
			first_asset_amount,
			ExistenceRequirement::KeepAlive,
		)?;
		<T as Config>::Currency::transfer(
			second_asset_id,
			&sender,
			&vault,
			second_asset_amount,
			ExistenceRequirement::KeepAlive,
		)?;

		// Creating new liquidity tokens to user
		<T as Config>::Currency::mint(liquidity_asset_id, &sender, liquidity_assets_minted)?;

		if <T::LiquidityMiningRewards as ProofOfStakeRewardsApi<
			T::AccountId,
			BalanceOf<T>,
			CurrencyIdOf<T>,
		>>::is_enabled(liquidity_asset_id) &&
			activate_minted_liquidity
		{
			// The reserve from free_balance will not fail the asset were just minted into free_balance
			<T::LiquidityMiningRewards as ProofOfStakeRewardsApi<
				T::AccountId,
				BalanceOf<T>,
				CurrencyIdOf<T>,
			>>::activate_liquidity(
				sender.clone(),
				liquidity_asset_id,
				liquidity_assets_minted,
				Some(ActivateKind::AvailableBalance),
			)?;
		}

		// Apply changes in token pools, adding minted amounts
		// Won't overflow due earlier ensure
		let first_asset_reserve_updated = first_asset_reserve.saturating_add(first_asset_amount);
		let second_asset_reserve_updated = second_asset_reserve.saturating_add(second_asset_amount);
		Pallet::<T>::set_reserves(
			first_asset_id,
			first_asset_reserve_updated,
			second_asset_id,
			second_asset_reserve_updated,
		)?;

		log!(
			info,
			"mint_liquidity: ({:?}, {:?}, {:?}, {:?}) -> ({:?}, {:?}, {:?})",
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
			"pool-state: [({:?}, {:?}) -> {:?}, ({:?}, {:?}) -> {:?}]",
			first_asset_id,
			second_asset_id,
			first_asset_reserve_updated,
			second_asset_id,
			first_asset_id,
			second_asset_reserve_updated
		);

		Pallet::<T>::deposit_event(Event::LiquidityMinted(
			sender,
			first_asset_id,
			first_asset_amount,
			second_asset_id,
			second_asset_amount,
			liquidity_asset_id,
			liquidity_assets_minted,
		));

		Ok((liquidity_asset_id, liquidity_assets_minted))
	}

	fn do_compound_rewards(
		sender: T::AccountId,
		liquidity_asset_id: CurrencyIdOf<T>,
		amount_permille: Permill,
	) -> DispatchResult {
		let (first_asset_id, second_asset_id) =
			LiquidityPools::<T>::get(liquidity_asset_id).ok_or(Error::<T>::NoSuchLiquidityAsset)?;

		ensure!(
			!T::DisabledTokens::contains(&first_asset_id) &&
				!T::DisabledTokens::contains(&second_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		let rewards_id: CurrencyIdOf<T> = Self::native_token_id();
		ensure!(
			first_asset_id == rewards_id || second_asset_id == rewards_id,
			Error::<T>::FunctionNotAvailableForThisToken
		);

		let rewards_claimed = <T::LiquidityMiningRewards as ProofOfStakeRewardsApi<
			T::AccountId,
			BalanceOf<T>,
			CurrencyIdOf<T>,
		>>::claim_rewards_all(sender.clone(), liquidity_asset_id)?;

		let rewards_256 = U256::from(rewards_claimed.into())
			.saturating_mul(amount_permille.deconstruct().into())
			.div(Permill::one().deconstruct());
		let rewards_128 = u128::try_from(rewards_256)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
		let rewards = BalanceOf::<T>::try_from(rewards_128)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::provide_liquidity_with_conversion(
			sender,
			first_asset_id,
			second_asset_id,
			rewards_id,
			rewards,
			true,
		)?;

		Ok(())
	}

	fn provide_liquidity_with_conversion(
		sender: T::AccountId,
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
		provided_asset_id: CurrencyIdOf<T>,
		provided_asset_amount: BalanceOf<T>,
		activate_minted_liquidity: bool,
	) -> Result<(CurrencyIdOf<T>, BalanceOf<T>), DispatchError> {
		// checks
		ensure!(!provided_asset_amount.is_zero(), Error::<T>::ZeroAmount,);

		ensure!(!(Self::is_pool_empty(first_asset_id, second_asset_id)?), Error::<T>::PoolIsEmpty);

		let (first_reserve, second_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;

		let (reserve, other_reserve, other_asset_id) = if provided_asset_id == first_asset_id {
			(first_reserve, second_reserve, second_asset_id)
		} else if provided_asset_id == second_asset_id {
			(second_reserve, first_reserve, first_asset_id)
		} else {
			return Err(DispatchError::from(Error::<T>::FunctionNotAvailableForThisToken))
		};

		// Ensure user has enough tokens to sell
		<T as Config>::Currency::ensure_can_withdraw(
			provided_asset_id,
			&sender,
			provided_asset_amount,
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		// calculate sell
		let swap_amount =
			Pallet::<T>::calculate_balanced_sell_amount(provided_asset_amount, reserve)?;

		let bought_amount = Pallet::<T>::calculate_sell_price(reserve, other_reserve, swap_amount)?;

		let _ =
			<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::sell_asset(
				sender.clone(),
				provided_asset_id,
				other_asset_id,
				swap_amount,
				bought_amount,
				true,
			)?;

		let mint_amount = provided_asset_amount
			.checked_sub(&swap_amount)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		log!(
			info,
			"provide_liquidity_with_conversion: ({:?}, {:?}, {:?}, {:?}, {:?}) -> ({:?}, {:?})",
			sender,
			first_asset_id,
			second_asset_id,
			provided_asset_id,
			provided_asset_amount,
			mint_amount,
			bought_amount
		);

		// we swap the order of the pairs to handle rounding
		// we spend all of the Y
		// and have some surplus amount of X that equals to the rounded part of Y
		<Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::mint_liquidity(
			sender,
			other_asset_id,
			provided_asset_id,
			bought_amount,
			BalanceOf::<T>::max_value(),
			activate_minted_liquidity,
		)
	}

	fn burn_liquidity(
		sender: T::AccountId,
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
		liquidity_asset_amount: BalanceOf<T>,
	) -> DispatchResult {
		let vault = Pallet::<T>::account_id();

		ensure!(
			!T::DisabledTokens::contains(&first_asset_id) &&
				!T::DisabledTokens::contains(&second_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		let liquidity_asset_id = Pallet::<T>::get_liquidity_asset(first_asset_id, second_asset_id)?;

		// First let's check how much we can actually burn
		let max_instant_unreserve_amount =
			<T as pallet::Config>::ActivationReservesProvider::get_max_instant_unreserve_amount(
				liquidity_asset_id,
				&sender,
			);

		// Get token reserves and liquidity asset id
		let (first_asset_reserve, second_asset_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;

		// Ensure user has enought liquidity tokens to burn
		let liquidity_token_available_balance =
			<T as Config>::Currency::available_balance(liquidity_asset_id.into(), &sender);

		ensure!(
			liquidity_token_available_balance
				.checked_add(&max_instant_unreserve_amount)
				.ok_or(Error::<T>::MathOverflow)? >=
				liquidity_asset_amount,
			Error::<T>::NotEnoughAssets,
		);

		// Given the above ensure passes we only need to know how much to deactivate before burning
		// Because once deactivated we will be burning the entire liquidity_asset_amount from available balance
		// to_be_deactivated will ofcourse then also be greater than max_instant_unreserve_amount
		// If pool is not promoted max_instant_unreserve_amount is 0, so liquidity_token_available_balance >= liquidity_asset_amount
		// which would mean to_be_deactivated is 0, skipping deactivation
		let to_be_deactivated =
			liquidity_asset_amount.saturating_sub(liquidity_token_available_balance);

		// deactivate liquidity
		<T::LiquidityMiningRewards as ProofOfStakeRewardsApi<
			T::AccountId,
			BalanceOf<T>,
			CurrencyIdOf<T>,
		>>::deactivate_liquidity(sender.clone(), liquidity_asset_id, to_be_deactivated)?;

		// Calculate first and second token amounts depending on liquidity amount to burn
		let (first_asset_amount, second_asset_amount) = Pallet::<T>::get_burn_amount_reserves(
			first_asset_reserve,
			second_asset_reserve,
			liquidity_asset_id,
			liquidity_asset_amount,
		)?;

		let total_liquidity_assets: BalanceOf<T> =
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into());

		// If all liquidity assets are being burned then
		// both asset amounts must be equal to their reserve values
		// All storage values related to this pool must be destroyed
		if liquidity_asset_amount == total_liquidity_assets {
			ensure!(
				(first_asset_reserve == first_asset_amount) &&
					(second_asset_reserve == second_asset_amount),
				Error::<T>::UnexpectedFailure
			);
		} else {
			ensure!(
				(first_asset_reserve >= first_asset_amount) &&
					(second_asset_reserve >= second_asset_amount),
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
		<T as Config>::Currency::transfer(
			first_asset_id,
			&vault,
			&sender,
			first_asset_amount,
			ExistenceRequirement::KeepAlive,
		)?;
		<T as Config>::Currency::transfer(
			second_asset_id,
			&vault,
			&sender,
			second_asset_amount,
			ExistenceRequirement::KeepAlive,
		)?;

		log!(
			info,
			"burn_liquidity: ({:?}, {:?}, {:?}, {:?}) -> ({:?}, {:?})",
			sender,
			first_asset_id,
			second_asset_id,
			liquidity_asset_amount,
			first_asset_amount,
			second_asset_amount
		);

		// Is liquidity asset amount empty?
		if liquidity_asset_amount == total_liquidity_assets {
			log!(
				info,
				"pool-state: [({:?}, {:?}) -> Removed, ({:?}, {:?}) -> Removed]",
				first_asset_id,
				second_asset_id,
				second_asset_id,
				first_asset_id,
			);
			Pallet::<T>::set_reserves(
				first_asset_id,
				BalanceOf::<T>::zero(),
				second_asset_id,
				BalanceOf::<T>::zero(),
			)?;
		} else {
			// Apply changes in token pools, removing withdrawn amounts
			// Cannot underflow due to earlier ensure
			// check was executed in get_reserves call
			let first_asset_reserve_updated =
				first_asset_reserve.saturating_sub(first_asset_amount);
			let second_asset_reserve_updated =
				second_asset_reserve.saturating_sub(second_asset_amount);
			Pallet::<T>::set_reserves(
				first_asset_id,
				first_asset_reserve_updated,
				second_asset_id,
				second_asset_reserve_updated,
			)?;

			log!(
				info,
				"pool-state: [({:?}, {:?}) -> {:?}, ({:?}, {:?}) -> {:?}]",
				first_asset_id,
				second_asset_id,
				first_asset_reserve_updated,
				second_asset_id,
				first_asset_id,
				second_asset_reserve_updated
			);
		}

		// Destroying burnt liquidity tokens
		// MAX: 3R 1W
		<T as Config>::Currency::burn_and_settle(
			liquidity_asset_id.into(),
			&sender,
			liquidity_asset_amount,
		)?;

		Pallet::<T>::deposit_event(Event::LiquidityBurned(
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

	// This function has not been verified
	fn get_tokens_required_for_minting(
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_token_amount: BalanceOf<T>,
	) -> Result<(CurrencyIdOf<T>, BalanceOf<T>, CurrencyIdOf<T>, BalanceOf<T>), DispatchError> {
		let (first_asset_id, second_asset_id) =
			LiquidityPools::<T>::get(liquidity_asset_id).ok_or(Error::<T>::NoSuchLiquidityAsset)?;
		let (first_asset_reserve, second_asset_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;
		let total_liquidity_assets: BalanceOf<T> =
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into());

		ensure!(!total_liquidity_assets.is_zero(), Error::<T>::DivisionByZero);
		let second_asset_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			liquidity_token_amount.into(),
			second_asset_reserve.into(),
			total_liquidity_assets.into(),
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
		.try_into()
		.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		let first_asset_amount: BalanceOf<T> = multiply_by_rational_with_rounding(
			liquidity_token_amount.into(),
			first_asset_reserve.into(),
			total_liquidity_assets.into(),
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
		.try_into()
		.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		log!(
			info,
			"get_tokens_required_for_minting: ({:?}, {:?}) -> ({:?}, {:?}, {:?}, {:?})",
			liquidity_asset_id,
			liquidity_token_amount,
			first_asset_id,
			first_asset_amount,
			second_asset_id,
			second_asset_amount,
		);

		Ok((first_asset_id, first_asset_amount, second_asset_id, second_asset_amount))
	}

	fn is_liquidity_token(liquidity_asset_id: CurrencyIdOf<T>) -> bool {
		LiquidityPools::<T>::get(liquidity_asset_id).is_some()
	}
}

pub trait AssetMetadataMutationTrait<CurrencyId> {
	fn set_asset_info(
		asset: CurrencyId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u32,
	) -> DispatchResult;
}

impl<T: Config> Valuate<BalanceOf<T>, CurrencyIdOf<T>> for Pallet<T> {
	fn get_liquidity_asset(
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
	) -> Result<CurrencyIdOf<T>, DispatchError> {
		Pallet::<T>::get_liquidity_asset(first_asset_id, second_asset_id)
	}

	fn get_liquidity_token_mga_pool(
		liquidity_token_id: CurrencyIdOf<T>,
	) -> Result<(CurrencyIdOf<T>, CurrencyIdOf<T>), DispatchError> {
		let (first_token_id, second_token_id) =
			LiquidityPools::<T>::get(liquidity_token_id).ok_or(Error::<T>::NoSuchLiquidityAsset)?;
		let native_currency_id = Self::native_token_id();
		match native_currency_id {
			_ if native_currency_id == first_token_id => Ok((first_token_id, second_token_id)),
			_ if native_currency_id == second_token_id => Ok((second_token_id, first_token_id)),
			_ => Err(Error::<T>::NotMangataLiquidityAsset.into()),
		}
	}

	fn valuate_liquidity_token(
		liquidity_token_id: CurrencyIdOf<T>,
		liquidity_token_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let (mga_token_id, other_token_id) =
			match Self::get_liquidity_token_mga_pool(liquidity_token_id) {
				Ok(pool) => pool,
				Err(_) => return Default::default(),
			};

		let mga_token_reserve = match Pallet::<T>::get_reserves(mga_token_id, other_token_id) {
			Ok(reserves) => reserves.0,
			Err(_) => return Default::default(),
		};

		let liquidity_token_reserve: BalanceOf<T> =
			<T as Config>::Currency::total_issuance(liquidity_token_id.into());

		if liquidity_token_reserve.is_zero() {
			return Default::default()
		}

		multiply_by_rational_with_rounding(
			mga_token_reserve.into(),
			liquidity_token_amount.into(),
			liquidity_token_reserve.into(),
			Rounding::Down,
		)
		.map(SaturatedConversion::saturated_into)
		.unwrap_or(BalanceOf::<T>::max_value())
	}

	fn valuate_non_liquidity_token(
		non_liquidity_token_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let native_token_id = Pallet::<T>::native_token_id();

		let (native_reserves, token_reserves) =
			match Pallet::<T>::get_reserves(native_token_id, non_liquidity_token_id) {
				Ok(reserves) => reserves,
				Err(_) => return Default::default(),
			};
		Pallet::<T>::calculate_sell_price_no_fee(token_reserves, native_reserves, amount)
			.unwrap_or_default()
	}

	fn scale_liquidity_by_mga_valuation(
		mga_valuation: BalanceOf<T>,
		liquidity_token_amount: BalanceOf<T>,
		mga_token_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		if mga_valuation.is_zero() {
			return Default::default()
		}

		multiply_by_rational_with_rounding(
			liquidity_token_amount.into(),
			mga_token_amount.into(),
			mga_valuation.into(),
			Rounding::Down,
		)
		.map(SaturatedConversion::saturated_into)
		.unwrap_or(BalanceOf::<T>::max_value())
	}

	fn get_pool_state(liquidity_token_id: CurrencyIdOf<T>) -> Option<(BalanceOf<T>, BalanceOf<T>)> {
		let (mga_token_id, other_token_id) =
			match Self::get_liquidity_token_mga_pool(liquidity_token_id) {
				Ok(pool) => pool,
				Err(_) => return None,
			};

		let mga_token_reserve = match Pallet::<T>::get_reserves(mga_token_id, other_token_id) {
			Ok(reserves) => reserves.0,
			Err(_) => return None,
		};

		let liquidity_token_reserve: BalanceOf<T> =
			<T as Config>::Currency::total_issuance(liquidity_token_id.into());

		if liquidity_token_reserve.is_zero() {
			return None
		}

		Some((mga_token_reserve, liquidity_token_reserve))
	}

	fn get_reserves(
		first_asset_id: CurrencyIdOf<T>,
		second_asset_id: CurrencyIdOf<T>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		Pallet::<T>::get_reserves(first_asset_id, second_asset_id)
	}

	fn is_liquidity_token(liquidity_asset_id: CurrencyIdOf<T>) -> bool {
		LiquidityPools::<T>::get(liquidity_asset_id).is_some()
	}
}

impl<T: Config> PoolCreateApi<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>> for Pallet<T> {
	fn pool_exists(first: CurrencyIdOf<T>, second: CurrencyIdOf<T>) -> bool {
		Pools::<T>::contains_key((first, second)) || Pools::<T>::contains_key((second, first))
	}

	fn pool_create(
		account: T::AccountId,
		first: CurrencyIdOf<T>,
		first_amount: BalanceOf<T>,
		second: CurrencyIdOf<T>,
		second_amount: BalanceOf<T>,
	) -> Option<(CurrencyIdOf<T>, BalanceOf<T>)> {
		match <Self as XykFunctionsTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>::create_pool(
			account,
			first,
			first_amount,
			second,
			second_amount,
		) {
			Ok(_) => LiquidityAssets::<T>::get((first, second)).map(|asset_id| {
				(asset_id, <T as Config>::Currency::total_issuance(asset_id.into()))
			}),
			Err(e) => {
				log!(error, "cannot create pool {:?}!", e);
				None
			},
		}
	}
}
