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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	assert_ok,
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::Contains,
	PalletId,
};
use frame_system::ensure_signed;
use sp_core::U256;
// TODO documentation!
use codec::FullCodec;
use frame_support::{
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, ExistenceRequirement, Get, WithdrawReasons},
	transactional, Parameter,
};
use frame_system::pallet_prelude::*;
use mangata_types::{Balance, TokenId};
use mp_bootstrap::PoolCreateApi;
use mp_multipurpose_liquidity::ActivateKind;
use mp_traits::{ActivationReservesProviderTrait, XykFunctionsTrait};
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use pallet_issuance::{ActivedPoolQueryApi, ComputeIssuance, PoolPromoteApi};
use pallet_vesting_mangata::MultiTokenVestingLocks;
use sp_arithmetic::{helpers_128bit::multiply_by_rational_with_rounding, per_things::Rounding};
use sp_runtime::{
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member,
		SaturatedConversion, Zero,
	},
	Permill,
};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	ops::Div,
	prelude::*,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct RewardInfo {
	pub activated_amount: u128,
	pub rewards_not_yet_claimed: u128,
	pub rewards_already_claimed: u128,
	pub last_checkpoint: u32,
	pub pool_ratio_at_last_checkpoint: U256,
	pub missing_at_last_checkpoint: U256,
}

pub(crate) const LOG_TARGET: &'static str = "xyk";

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
const Q: f64 = 1.03;
// Precision used in rewards calculation rounding
const REWARDS_PRECISION: u32 = 10000;

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
#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[cfg(feature = "runtime-benchmarks")]
	pub trait XykBenchmarkingConfig: pallet_issuance::Config {}

	#[cfg(not(feature = "runtime-benchmarks"))]
	pub trait XykBenchmarkingConfig {}

	#[pallet::config]
	pub trait Config: frame_system::Config + XykBenchmarkingConfig {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type ActivationReservesProvider: ActivationReservesProviderTrait<
			AccountId = Self::AccountId,
		>;
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;
		type NativeCurrencyId: Get<TokenId>;
		type TreasuryPalletId: Get<PalletId>;
		type BnbTreasurySubAccDerive: Get<[u8; 4]>;
		// type ActivedPoolQueryApi: ActivedPoolQueryApi;
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
		type DisallowedPools: Contains<(TokenId, TokenId)>;
		type DisabledTokens: Contains<TokenId>;
		type VestingProvider: MultiTokenVestingLocks<Self::AccountId, Self::BlockNumber>;
		type AssetMetadataMutation: AssetMetadataMutationTrait;
		type WeightInfo: WeightInfo;
		#[pallet::constant]
		type RewardsMigrateAccount: Get<Self::AccountId>;
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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated(T::AccountId, TokenId, Balance, TokenId, Balance),
		AssetsSwapped(T::AccountId, TokenId, Balance, TokenId, Balance),
		LiquidityMinted(T::AccountId, TokenId, Balance, TokenId, Balance, TokenId, Balance),
		LiquidityBurned(T::AccountId, TokenId, Balance, TokenId, Balance, TokenId, Balance),
		PoolPromotionUpdated(TokenId, Option<u8>),
		LiquidityActivated(T::AccountId, TokenId, Balance),
		LiquidityDeactivated(T::AccountId, TokenId, Balance),
		RewardsClaimed(T::AccountId, TokenId, Balance),
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_pool)]
	pub type Pools<T: Config> =
		StorageMap<_, Blake2_256, (TokenId, TokenId), (Balance, Balance), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_asset)]
	pub type LiquidityAssets<T: Config> =
		StorageMap<_, Blake2_256, (TokenId, TokenId), Option<TokenId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	pub type LiquidityPools<T: Config> =
		StorageMap<_, Blake2_256, TokenId, Option<(TokenId, TokenId)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_rewards_info)]
	pub type RewardsInfo<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		TokenId,
		RewardInfo,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_active_pool_v2)]
	pub type LiquidityMiningActivePoolV2<T: Config> =
		StorageMap<_, Twox64Concat, TokenId, u128, ValueQuery>;

	// REWARDS V1 to be removed
	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_user)]
	pub type LiquidityMiningUser<T: Config> =
		StorageMap<_, Blake2_256, (AccountIdOf<T>, TokenId), (u32, U256, U256), ValueQuery>;
	// REWARDS V1 to be removed
	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_pool)]
	pub type LiquidityMiningPool<T: Config> =
		StorageMap<_, Blake2_256, TokenId, (u32, U256, U256), ValueQuery>;
	// REWARDS V1 to be removed
	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_user_to_be_claimed)]
	pub type LiquidityMiningUserToBeClaimed<T: Config> =
		StorageMap<_, Blake2_256, (AccountIdOf<T>, TokenId), u128, ValueQuery>;
	// REWARDS V1 to be removed
	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_active_user)]
	pub type LiquidityMiningActiveUser<T: Config> =
		StorageMap<_, Twox64Concat, (AccountIdOf<T>, TokenId), u128, ValueQuery>;
	// REWARDS V1 to be removed
	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_active_pool)]
	pub type LiquidityMiningActivePool<T: Config> =
		StorageMap<_, Twox64Concat, TokenId, u128, ValueQuery>;
	// REWARDS V1 to be removed
	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_user_claimed)]
	pub type LiquidityMiningUserClaimed<T: Config> =
		StorageMap<_, Twox64Concat, (AccountIdOf<T>, TokenId), u128, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub created_pools_for_staking:
			Vec<(T::AccountId, TokenId, Balance, TokenId, Balance, TokenId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { created_pools_for_staking: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
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
							<Pallet<T> as XykFunctionsTrait<T::AccountId>>::mint_liquidity(
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
						let created_liquidity_token_id: TokenId =
							<T as Config>::Currency::get_next_currency_id().into();
						assert_eq!(
							created_liquidity_token_id, *liquidity_token_id,
							"Assets not initialized in the expected sequence",
						);
						assert_ok!(<Pallet<T> as XykFunctionsTrait<T::AccountId>>::create_pool(
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
		#[pallet::weight(<<T as Config>::WeightInfo>::create_pool())]
		pub fn create_pool(
			origin: OriginFor<T>,
			first_asset_id: TokenId,
			first_asset_amount: Balance,
			second_asset_id: TokenId,
			second_asset_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(
				!T::DisabledTokens::contains(&first_asset_id)
					&& !T::DisabledTokens::contains(&second_asset_id),
				Error::<T>::FunctionNotAvailableForThisToken
			);

			ensure!(
				!T::DisallowedPools::contains(&(first_asset_id, second_asset_id)),
				Error::<T>::DisallowedPool,
			);

			<Self as XykFunctionsTrait<T::AccountId>>::create_pool(
				sender,
				first_asset_id,
				first_asset_amount,
				second_asset_id,
				second_asset_amount,
			)?;

			Ok(().into())
		}

		// you will sell your sold_asset_amount of sold_asset_id to get some amount of bought_asset_id
		#[pallet::weight(<<T as Config>::WeightInfo>::sell_asset())]
		pub fn sell_asset(
			origin: OriginFor<T>,
			sold_asset_id: TokenId,
			bought_asset_id: TokenId,
			sold_asset_amount: Balance,
			min_amount_out: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::sell_asset(
				sender,
				sold_asset_id,
				bought_asset_id,
				sold_asset_amount,
				min_amount_out,
			)?;
			Ok(().into())
		}

		#[pallet::weight(<<T as Config>::WeightInfo>::buy_asset())]
		pub fn buy_asset(
			origin: OriginFor<T>,
			sold_asset_id: TokenId,
			bought_asset_id: TokenId,
			bought_asset_amount: Balance,
			max_amount_in: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::buy_asset(
				sender,
				sold_asset_id,
				bought_asset_id,
				bought_asset_amount,
				max_amount_in,
			)?;
			Ok(().into())
		}

		#[pallet::weight(<<T as Config>::WeightInfo>::mint_liquidity_using_vesting_native_tokens())]
		#[transactional]
		pub fn mint_liquidity_using_vesting_native_tokens_by_vesting_index(
			origin: OriginFor<T>,
			native_asset_vesting_index: u32,
			vesting_native_asset_unlock_some_amount_or_all: Option<Balance>,
			second_asset_id: TokenId,
			expected_second_asset_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let liquidity_asset_id =
				Pallet::<T>::get_liquidity_asset(Self::native_token_id(), second_asset_id)?;

			ensure!(
				<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id).is_some(),
				Error::<T>::NotAPromotedPool
			);

			let (unlocked_amount, vesting_starting_block, vesting_ending_block_as_balance): (
				Balance,
				T::BlockNumber,
				Balance,
			) = <<T as Config>::VestingProvider>::unlock_tokens_by_vesting_index(
				&sender,
				Self::native_token_id().into(),
				native_asset_vesting_index,
				vesting_native_asset_unlock_some_amount_or_all.map(Into::into),
			)
			.map(|x| (x.0.into(), x.1.into(), x.2.into()))?;

			let (liquidity_token_id, liquidity_assets_minted) =
				<Self as XykFunctionsTrait<T::AccountId>>::mint_liquidity(
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
				liquidity_assets_minted.into(),
				Some(vesting_starting_block.into()),
				vesting_ending_block_as_balance.into(),
			)?;

			Ok(().into())
		}

		#[pallet::weight(<<T as Config>::WeightInfo>::mint_liquidity_using_vesting_native_tokens())]
		#[transactional]
		pub fn mint_liquidity_using_vesting_native_tokens(
			origin: OriginFor<T>,
			vesting_native_asset_amount: Balance,
			second_asset_id: TokenId,
			expected_second_asset_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let liquidity_asset_id =
				Pallet::<T>::get_liquidity_asset(Self::native_token_id(), second_asset_id)?;

			ensure!(
				<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id).is_some(),
				Error::<T>::NotAPromotedPool
			);

			let (vesting_starting_block, vesting_ending_block_as_balance): (
				T::BlockNumber,
				Balance,
			) = <<T as Config>::VestingProvider>::unlock_tokens(
				&sender,
				Self::native_token_id().into(),
				vesting_native_asset_amount.into(),
			)
			.map(|x| (x.0.into(), x.1.into()))?;

			let (liquidity_token_id, liquidity_assets_minted) =
				<Self as XykFunctionsTrait<T::AccountId>>::mint_liquidity(
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
				liquidity_assets_minted.into(),
				Some(vesting_starting_block.into()),
				vesting_ending_block_as_balance.into(),
			)?;

			Ok(().into())
		}

		#[pallet::weight(<<T as Config>::WeightInfo>::mint_liquidity())]
		pub fn mint_liquidity(
			origin: OriginFor<T>,
			first_asset_id: TokenId,
			second_asset_id: TokenId,
			first_asset_amount: Balance,
			expected_second_asset_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(
				!T::DisabledTokens::contains(&first_asset_id)
					&& !T::DisabledTokens::contains(&second_asset_id),
				Error::<T>::FunctionNotAvailableForThisToken
			);

			<Self as XykFunctionsTrait<T::AccountId>>::mint_liquidity(
				sender,
				first_asset_id,
				second_asset_id,
				first_asset_amount,
				expected_second_asset_amount,
				true,
			)?;

			Ok(().into())
		}

		#[pallet::weight(<<T as Config>::WeightInfo>::compound_rewards())]
		#[transactional]
		pub fn compound_rewards(
			origin: OriginFor<T>,
			liquidity_asset_id: TokenId,
			amount_permille: Permill,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (first_asset_id, second_asset_id) = LiquidityPools::<T>::get(liquidity_asset_id)
				.ok_or(Error::<T>::NoSuchLiquidityAsset)?;

			ensure!(
				!T::DisabledTokens::contains(&first_asset_id)
					&& !T::DisabledTokens::contains(&second_asset_id),
				Error::<T>::FunctionNotAvailableForThisToken
			);

			let rewards_claimed = <Self as XykFunctionsTrait<T::AccountId>>::claim_rewards_all_v2(
				sender.clone(),
				liquidity_asset_id,
			)?;

			let rewards_256 = Into::<U256>::into(rewards_claimed)
				.saturating_mul(amount_permille.deconstruct().into())
				.div(Permill::one().deconstruct());
			let rewards = Balance::try_from(rewards_256)
				.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

			<Self as XykFunctionsTrait<T::AccountId>>::provide_liquidity_with_conversion(
				sender,
				first_asset_id,
				second_asset_id,
				first_asset_id,
				rewards,
				true,
			)?;

			Ok(().into())
		}

		#[pallet::weight(<<T as Config>::WeightInfo>::provide_liquidity_with_conversion())]
		#[transactional]
		pub fn provide_liquidity_with_conversion(
			origin: OriginFor<T>,
			liquidity_asset_id: TokenId,
			provided_asset_id: TokenId,
			provided_asset_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (first_asset_id, second_asset_id) = LiquidityPools::<T>::get(liquidity_asset_id)
				.ok_or(Error::<T>::NoSuchLiquidityAsset)?;

			ensure!(
				!T::DisabledTokens::contains(&first_asset_id)
					&& !T::DisabledTokens::contains(&second_asset_id),
				Error::<T>::FunctionNotAvailableForThisToken
			);

			<Self as XykFunctionsTrait<T::AccountId>>::provide_liquidity_with_conversion(
				sender,
				first_asset_id,
				second_asset_id,
				provided_asset_id,
				provided_asset_amount,
				true,
			)?;

			Ok(().into())
		}

		#[pallet::weight(<<T as Config>::WeightInfo>::burn_liquidity())]
		pub fn burn_liquidity(
			origin: OriginFor<T>,
			first_asset_id: TokenId,
			second_asset_id: TokenId,
			liquidity_asset_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::burn_liquidity(
				sender,
				first_asset_id,
				second_asset_id,
				liquidity_asset_amount,
			)?;

			Ok(().into())
		}

		#[transactional]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_rewards_v2())]
		pub fn claim_rewards_v2(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::claim_rewards_v2(
				sender,
				liquidity_token_id,
				amount,
			)?;

			Ok(().into())
		}

		#[transactional]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_rewards_v2())]
		pub fn claim_rewards_all_v2(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::claim_rewards_all_v2(
				sender,
				liquidity_token_id,
			)?;

			Ok(().into())
		}

		#[pallet::weight(<<T as Config>::WeightInfo>::update_pool_promotion())]
		pub fn update_pool_promotion(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			liquidity_mining_issuance_weight: Option<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::update_pool_promotion(
				liquidity_token_id,
				liquidity_mining_issuance_weight,
			)
		}

		#[transactional]
		#[pallet::weight(<<T as Config>::WeightInfo>::activate_liquidity_v2())]
		pub fn activate_liquidity_v2(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
			use_balance_from: Option<ActivateKind>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::activate_liquidity_v2(
				sender,
				liquidity_token_id,
				amount,
				use_balance_from,
			)
		}

		#[transactional]
		#[pallet::weight(<<T as Config>::WeightInfo>::deactivate_liquidity_v2())]
		pub fn deactivate_liquidity_v2(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::deactivate_liquidity_v2(
				sender,
				liquidity_token_id,
				amount,
			)
		}

		#[transactional]
		#[pallet::weight(<<T as Config>::WeightInfo>::rewards_migrate_v1_to_v2())]
		pub fn rewards_migrate_v1_to_v2(
			origin: OriginFor<T>,
			account: T::AccountId,
			liquidity_token_id: TokenId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(sender == T::RewardsMigrateAccount::get(), Error::<T>::NoRights);

			<Self as XykFunctionsTrait<T::AccountId>>::rewards_migrate_v1_to_v2(
				account,
				liquidity_token_id,
			)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn total_fee() -> u128 {
		T::PoolFeePercentage::get()
			+ T::TreasuryFeePercentage::get()
			+ T::BuyAndBurnFeePercentage::get()
	}

	pub fn calculate_rewards_amount_v2(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
	) -> Result<Balance, DispatchError> {
		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);

		let liquidity_assets_amount: Balance = rewards_info.activated_amount;

		let mut current_rewards = 0;

		if liquidity_assets_amount != 0 {
			let pool_rewards_ratio_current =
				<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id)
					.ok_or_else(|| DispatchError::from(Error::<T>::NotAPromotedPool))?;

			let last_checkpoint = rewards_info.last_checkpoint;
			let pool_ratio_at_last_checkpoint = rewards_info.pool_ratio_at_last_checkpoint;
			let missing_at_checkpoint = rewards_info.missing_at_last_checkpoint;

			current_rewards = Self::calculate_rewards_v2(
				liquidity_assets_amount,
				liquidity_asset_id,
				last_checkpoint,
				pool_ratio_at_last_checkpoint,
				missing_at_checkpoint,
				pool_rewards_ratio_current,
			)?;
		}

		let not_yet_claimed_rewards = rewards_info.rewards_not_yet_claimed;

		let already_claimed_rewards = rewards_info.rewards_already_claimed;

		let total_available_rewards = current_rewards
			.checked_add(not_yet_claimed_rewards)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?
			.checked_sub(already_claimed_rewards)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?;

		Ok(total_available_rewards)
	}

	pub fn calculate_rewards_v2(
		liquidity_assets_amount: u128,
		liquidity_asset_id: TokenId,
		last_checkpoint: u32,
		pool_rewards_ratio: U256,
		missing_at_last_checkpoint: U256,
		pool_rewards_ratio_current: U256,
	) -> Result<Balance, DispatchError> {
		ensure!(
			<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id).is_some(),
			Error::<T>::NotAPromotedPool
		);

		let current_time: u32 = (<frame_system::Pallet<T>>::block_number().saturated_into::<u32>()
			+ 1) / T::RewardsDistributionPeriod::get();

		let time_passed = current_time
			.checked_sub(last_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::PastTimeCalculation))?;

		let pool_rewards_ratio_new = pool_rewards_ratio_current
			.checked_sub(pool_rewards_ratio)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?;

		let user_rewards_base: U256 = U256::from(liquidity_assets_amount)
			.checked_mul(pool_rewards_ratio_new.into()) // TODO: please add UT and link it in this comment
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?
			.checked_div(U256::from(u128::MAX)) // always fit into u128
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?;

		let (cumulative_work, cummulative_work_max_possible) =
			Self::calculate_cumulative_work_max_ratio(
				liquidity_assets_amount,
				time_passed,
				missing_at_last_checkpoint,
			)?;

		let current_rewards = U256::from(user_rewards_base)
			.checked_mul(cumulative_work)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?
			.checked_div(cummulative_work_max_possible)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?
			.try_into()
			.map_err(|_| DispatchError::from(Error::<T>::CalculateRewardsMathError))?;

		Ok(current_rewards)
	}

	// MAX: 0R 0W

	pub fn calculate_cumulative_work_max_ratio(
		liquidity_assets_amount: u128,
		time_passed: u32,
		missing_at_last_checkpoint: U256,
	) -> Result<(U256, U256), DispatchError> {
		let mut cummulative_work = U256::from(0);
		let mut cummulative_work_max_possible_for_ratio = U256::from(1);

		if time_passed != 0 && liquidity_assets_amount != 0 {
			let liquidity_assets_amount_u256: U256 = liquidity_assets_amount.into();

			// whole formula: 	missing_at_last_checkpoint*106/6 - missing_at_last_checkpoint*106*precision/6/q_pow
			// q_pow is multiplied by precision, thus there needs to be *precision in numenator as well

			cummulative_work_max_possible_for_ratio =
				liquidity_assets_amount_u256.checked_mul(U256::from(time_passed)).ok_or_else(
					|| DispatchError::from(Error::<T>::CalculateCumulativeWorkMaxRatioMathError),
				)?;

			// whole formula: 	missing_at_last_checkpoint*Q*100/(Q*100-100) - missing_at_last_checkpoint*Q*100/(Q*100-100)*REWARDS_PRECISION/q_pow
			// q_pow is multiplied by precision, thus there needs to be *precision in numenator as well
			let base = missing_at_last_checkpoint
				.checked_mul(U256::from(libm::floor(Q * 100 as f64) as u128))
				.ok_or_else(|| {
					DispatchError::from(Error::<T>::CalculateCumulativeWorkMaxRatioMathError)
				})?
				.checked_div(U256::from(libm::floor(Q * 100 as f64 - 100 as f64) as u128))
				.ok_or_else(|| {
					DispatchError::from(Error::<T>::CalculateCumulativeWorkMaxRatioMathError)
				})?;

			let q_pow = Self::calculate_q_pow(Q, time_passed + 1);

			let cummulative_missing_new =
				base - base * U256::from(REWARDS_PRECISION) / q_pow - missing_at_last_checkpoint;

			cummulative_work = cummulative_work_max_possible_for_ratio
				.checked_sub(cummulative_missing_new)
				.ok_or_else(|| {
					DispatchError::from(Error::<T>::CalculateCumulativeWorkMaxRatioMathError)
				})?;
		}

		Ok((cummulative_work, cummulative_work_max_possible_for_ratio))
	}

	pub fn calculate_q_pow(q: f64, pow: u32) -> u128 {
		libm::floor(libm::pow(q, pow as f64) * REWARDS_PRECISION as f64) as u128
	}

	/// 0R 0W
	pub fn calculate_missing_at_checkpoint_v2(
		time_passed: u32,
		missing_at_last_checkpoint: U256,
	) -> Result<U256, DispatchError> {
		let q_pow = Self::calculate_q_pow(Q, time_passed);

		let missing_at_checkpoint: U256 =
			missing_at_last_checkpoint * U256::from(REWARDS_PRECISION) / q_pow;

		Ok(missing_at_checkpoint)
	}

	pub fn set_liquidity_minting_checkpoint_v2(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
		liquidity_assets_added: Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		let current_time: u32 = (<frame_system::Pallet<T>>::block_number().saturated_into::<u32>()
			+ 1) / T::RewardsDistributionPeriod::get();
		let mut pool_ratio_current =
			<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id)
				.ok_or_else(|| DispatchError::from(Error::<T>::NotAPromotedPool))?;

		let rewards_info = RewardsInfo::<T>::try_get(user.clone(), liquidity_asset_id)
			.unwrap_or_else(|_| RewardInfo {
				activated_amount: 0 as u128,
				rewards_not_yet_claimed: 0 as u128,
				rewards_already_claimed: 0 as u128,
				last_checkpoint: current_time,
				pool_ratio_at_last_checkpoint: pool_ratio_current,
				missing_at_last_checkpoint: U256::from(liquidity_assets_added),
			});

		let last_checkpoint = rewards_info.last_checkpoint;
		let pool_ratio_at_last_checkpoint = rewards_info.pool_ratio_at_last_checkpoint;
		let missing_at_last_checkpoint = rewards_info.missing_at_last_checkpoint;
		let liquidity_assets_amount: Balance = rewards_info.activated_amount;

		let time_passed = current_time
			.checked_sub(last_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::PastTimeCalculation))?;

		if time_passed == 0 {
			pool_ratio_current = pool_ratio_at_last_checkpoint;
		}

		let missing_at_checkpoint_new = if liquidity_assets_amount == 0 {
			U256::from(liquidity_assets_added)
		} else {
			Self::calculate_missing_at_checkpoint_v2(time_passed, missing_at_last_checkpoint)?
				.checked_add(U256::from(liquidity_assets_added))
				.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?
		};

		let user_current_rewards = if liquidity_assets_amount == 0 {
			0
		} else {
			Self::calculate_rewards_v2(
				liquidity_assets_amount,
				liquidity_asset_id,
				last_checkpoint,
				pool_ratio_at_last_checkpoint,
				missing_at_last_checkpoint,
				pool_ratio_current,
			)?
		};

		let activated_amount_new = liquidity_assets_amount
			.checked_add(liquidity_assets_added)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		let total_available_rewards = user_current_rewards
			.checked_add(rewards_info.rewards_not_yet_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?
			.checked_sub(rewards_info.rewards_already_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		let rewards_info_new: RewardInfo = RewardInfo {
			activated_amount: activated_amount_new,
			rewards_not_yet_claimed: total_available_rewards,
			rewards_already_claimed: 0_u128,
			last_checkpoint: current_time,
			pool_ratio_at_last_checkpoint: pool_ratio_current,
			missing_at_last_checkpoint: missing_at_checkpoint_new,
		};

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info_new);

		LiquidityMiningActivePoolV2::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_add(liquidity_assets_added) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		// This must not fail due storage edits above
		<T as Config>::ActivationReservesProvider::activate(
			liquidity_asset_id.into(),
			&user,
			liquidity_assets_added.into(),
			use_balance_from,
		)?;

		Ok(())
	}

	pub fn set_liquidity_burning_checkpoint_v2(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
		liquidity_assets_burned: Balance,
	) -> DispatchResult {
		let current_time: u32 = (<frame_system::Pallet<T>>::block_number().saturated_into::<u32>()
			+ 1) / T::RewardsDistributionPeriod::get();

		let mut pool_ratio_current =
			<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id)
				.ok_or_else(|| DispatchError::from(Error::<T>::NotAPromotedPool))?;

		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);

		let last_checkpoint = rewards_info.last_checkpoint;
		let pool_ratio_at_last_checkpoint = rewards_info.pool_ratio_at_last_checkpoint;
		let missing_at_last_checkpoint = rewards_info.missing_at_last_checkpoint;
		let liquidity_assets_amount: Balance = rewards_info.activated_amount;

		let time_passed = current_time
			.checked_sub(last_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::PastTimeCalculation))?;

		if time_passed == 0 {
			pool_ratio_current = pool_ratio_at_last_checkpoint;
		}

		let missing_at_checkpoint_new =
			Self::calculate_missing_at_checkpoint_v2(time_passed, missing_at_last_checkpoint)?;

		let user_current_rewards = Self::calculate_rewards_v2(
			liquidity_assets_amount,
			liquidity_asset_id,
			last_checkpoint,
			pool_ratio_at_last_checkpoint,
			missing_at_last_checkpoint,
			pool_ratio_current,
		)?;

		let activated_amount_new = liquidity_assets_amount
			.checked_sub(liquidity_assets_burned)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		let activated_amount_new_u256: U256 = activated_amount_new.into();

		let missing_at_checkpoint_after_burn: U256 = activated_amount_new_u256
			.checked_mul(missing_at_checkpoint_new)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?
			.checked_div(liquidity_assets_amount.into())
			.ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;

		let total_available_rewards = user_current_rewards
			.checked_add(rewards_info.rewards_not_yet_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?
			.checked_sub(rewards_info.rewards_already_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		let rewards_info_new: RewardInfo = RewardInfo {
			activated_amount: activated_amount_new,
			rewards_not_yet_claimed: total_available_rewards,
			rewards_already_claimed: 0_u128,
			last_checkpoint: current_time,
			pool_ratio_at_last_checkpoint: pool_ratio_current,
			missing_at_last_checkpoint: missing_at_checkpoint_after_burn,
		};

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info_new);

		LiquidityMiningActivePoolV2::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_sub(liquidity_assets_burned) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		<T as Config>::ActivationReservesProvider::deactivate(
			liquidity_asset_id.into(),
			&user,
			liquidity_assets_burned.into(),
		);

		Ok(())
	}

	pub fn get_max_instant_burn_amount(
		user: &AccountIdOf<T>,
		liquidity_asset_id: TokenId,
	) -> Balance {
		Self::get_max_instant_unreserve_amount(user, liquidity_asset_id).saturating_add(
			<T as Config>::Currency::available_balance(liquidity_asset_id.into(), user).into(),
		)
	}

	pub fn get_max_instant_unreserve_amount(
		user: &AccountIdOf<T>,
		liquidity_asset_id: TokenId,
	) -> Balance {
		T::ActivationReservesProvider::get_max_instant_unreserve_amount(liquidity_asset_id, user)
	}

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
		input_reserve: Balance,
		output_reserve: Balance,
		sell_amount: Balance,
	) -> Result<Balance, DispatchError> {
		let after_fee_percentage: u128 = 10000 - Self::total_fee();
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
		let after_fee_percentage: u128 = 10000 - Self::total_fee();
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

	pub fn calculate_balanced_sell_amount(
		total_amount: Balance,
		reserve_amount: Balance,
	) -> Result<Balance, DispatchError> {
		let multiplier: U256 = 10_000.into();
		let multiplier_sq: U256 = multiplier.pow(2.into());
		let non_pool_fees: U256 = (Self::total_fee() - T::PoolFeePercentage::get()).into(); // npf
		let total_fee: U256 = Self::total_fee().into(); // tf
		let total_amount_saturated: U256 = total_amount.into(); // z
		let reserve_amount_saturated: U256 = reserve_amount.into(); // a

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
		let result = Balance::try_from(result_u256)
			.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		Ok(result)
	}

	// MAX: 2R
	pub fn get_liquidity_asset(
		first_asset_id: TokenId,
		second_asset_id: TokenId,
	) -> Result<TokenId, DispatchError> {
		if LiquidityAssets::<T>::contains_key((first_asset_id, second_asset_id)) {
			LiquidityAssets::<T>::get((first_asset_id, second_asset_id))
				.ok_or_else(|| Error::<T>::UnexpectedFailure.into())
		} else {
			LiquidityAssets::<T>::get((second_asset_id, first_asset_id))
				.ok_or_else(|| Error::<T>::NoSuchPool.into())
		}
	}

	pub fn calculate_sell_price_id(
		sold_token_id: TokenId,
		bought_token_id: TokenId,
		sell_amount: Balance,
	) -> Result<Balance, DispatchError> {
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_token_id, bought_token_id)?;

		Self::calculate_sell_price(input_reserve, output_reserve, sell_amount)
	}

	pub fn calculate_buy_price_id(
		sold_token_id: TokenId,
		bought_token_id: TokenId,
		buy_amount: Balance,
	) -> Result<Balance, DispatchError> {
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_token_id, bought_token_id)?;

		Self::calculate_buy_price(input_reserve, output_reserve, buy_amount)
	}

	pub fn get_reserves(
		first_asset_id: TokenId,
		second_asset_id: TokenId,
	) -> Result<(Balance, Balance), DispatchError> {
		let mut reserves = Pools::<T>::get((first_asset_id, second_asset_id));

		if Pools::<T>::contains_key((first_asset_id, second_asset_id)) {
			return Ok((reserves.0, reserves.1));
		} else if Pools::<T>::contains_key((second_asset_id, first_asset_id)) {
			reserves = Pools::<T>::get((second_asset_id, first_asset_id));
			return Ok((reserves.1, reserves.0));
		} else {
			return Err(DispatchError::from(Error::<T>::NoSuchPool));
		}
	}

	/// worst case scenario
	/// MAX: 2R 1W
	pub fn set_reserves(
		first_asset_id: TokenId,
		first_asset_amount: Balance,
		second_asset_id: TokenId,
		second_asset_amount: Balance,
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
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;

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
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();

		// Calculate first and second token amount to be withdrawn
		ensure!(!total_liquidity_assets.is_zero(), Error::<T>::DivisionByZero);
		let first_asset_amount = multiply_by_rational_with_rounding(
			first_asset_reserve,
			liquidity_asset_amount,
			total_liquidity_assets,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?;
		let second_asset_amount = multiply_by_rational_with_rounding(
			second_asset_reserve,
			liquidity_asset_amount,
			total_liquidity_assets,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?;

		Ok((first_asset_amount, second_asset_amount))
	}

	//TODO if pool contains key !
	fn settle_treasury_and_burn(
		sold_asset_id: TokenId,
		burn_amount: Balance,
		treasury_amount: Balance,
	) -> DispatchResult {
		let vault = Self::account_id();
		let mangata_id: TokenId = Self::native_token_id();
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
				burn_amount.into(),
			)?;
		}
		//If settling token is connected to mangata, token is swapped in corresponding pool to mangata without fee
		else if Pools::<T>::contains_key((sold_asset_id, mangata_id))
			|| Pools::<T>::contains_key((mangata_id, sold_asset_id))
		{
			// MAX: 2R (from if cond)

			// Getting token reserves
			let (input_reserve, output_reserve) =
				Pallet::<T>::get_reserves(sold_asset_id, mangata_id)?;

			// Calculating swapped mangata amount
			let settle_amount_in_mangata = Self::calculate_sell_price_no_fee(
				input_reserve,
				output_reserve,
				treasury_amount + burn_amount,
			)?;
			let treasury_amount_in_mangata = settle_amount_in_mangata
				* T::TreasuryFeePercentage::get()
				/ (T::TreasuryFeePercentage::get() + T::BuyAndBurnFeePercentage::get());

			let burn_amount_in_mangata = settle_amount_in_mangata - treasury_amount_in_mangata;

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
				treasury_amount.into(),
				ExistenceRequirement::KeepAlive,
			)?;

			<T as Config>::Currency::transfer(
				mangata_id.into(),
				&vault,
				&treasury_account,
				treasury_amount_in_mangata.into(),
				ExistenceRequirement::KeepAlive,
			)?;

			<T as Config>::Currency::transfer(
				sold_asset_id.into(),
				&bnb_treasury_account,
				&vault,
				burn_amount.into(),
				ExistenceRequirement::KeepAlive,
			)?;

			// Mangata burned from pool
			<T as Config>::Currency::burn_and_settle(
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
		PALLET_ID.into_account_truncating()
	}

	fn treasury_account_id() -> T::AccountId {
		T::TreasuryPalletId::get().into_account_truncating()
	}

	fn bnb_treasury_account_id() -> T::AccountId {
		T::TreasuryPalletId::get().into_sub_account_truncating(T::BnbTreasurySubAccDerive::get())
	}

	fn native_token_id() -> TokenId {
		<T as Config>::NativeCurrencyId::get()
	}
}

impl<T: Config> XykFunctionsTrait<T::AccountId> for Pallet<T> {
	type Balance = Balance;

	type CurrencyId = TokenId;

	fn create_pool(
		sender: T::AccountId,
		first_asset_id: Self::CurrencyId,
		first_asset_amount: Self::Balance,
		second_asset_id: Self::CurrencyId,
		second_asset_amount: Self::Balance,
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
			first_asset_amount.into(),
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		<T as Config>::Currency::ensure_can_withdraw(
			second_asset_id.into(),
			&sender,
			second_asset_amount.into(),
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		// Ensure pool is not created with same token in pair
		ensure!(first_asset_id != second_asset_id, Error::<T>::SameAsset,);

		// Liquidity token amount calculation
		let mut initial_liquidity = first_asset_amount / 2 + second_asset_amount / 2;
		if initial_liquidity == 0 {
			initial_liquidity = 1
		}

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
			first_asset_amount.into(),
			ExistenceRequirement::AllowDeath,
		)?;

		<T as Config>::Currency::transfer(
			second_asset_id.into(),
			&sender,
			&vault,
			second_asset_amount.into(),
			ExistenceRequirement::AllowDeath,
		)?;

		// Creating new liquidity token and transfering it to user
		let liquidity_asset_id: Self::CurrencyId =
			<T as Config>::Currency::create(&sender, initial_liquidity.into())
				.map_err(|_| Error::<T>::LiquidityTokenCreationFailed)?
				.into();

		// Adding info about liquidity asset
		LiquidityAssets::<T>::insert((first_asset_id, second_asset_id), Some(liquidity_asset_id));
		LiquidityPools::<T>::insert(liquidity_asset_id, Some((first_asset_id, second_asset_id)));

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

	fn sell_asset(
		sender: T::AccountId,
		sold_asset_id: Self::CurrencyId,
		bought_asset_id: Self::CurrencyId,
		sold_asset_amount: Self::Balance,
		min_amount_out: Self::Balance,
	) -> DispatchResult {
		// Ensure not selling zero amount
		ensure!(!sold_asset_amount.is_zero(), Error::<T>::ZeroAmount,);

		ensure!(
			!T::DisabledTokens::contains(&sold_asset_id)
				&& !T::DisabledTokens::contains(&bought_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		let buy_and_burn_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::BuyAndBurnFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
			+ 1;

		let treasury_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::TreasuryFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
			+ 1;

		let pool_fee_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::PoolFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
			+ 1;

		// for future implementation of min fee if necessary
		// let min_fee: u128 = 0;
		// if buy_and_burn_amount + treasury_amount + pool_fee_amount < min_fee {
		//     buy_and_burn_amount = min_fee * Self::total_fee() / T::BuyAndBurnFeePercentage::get();
		//     treasury_amount = min_fee * Self::total_fee() / T::TreasuryFeePercentage::get();
		//     pool_fee_amount = min_fee - buy_and_burn_amount - treasury_amount;
		// }

		// Get token reserves

		// MAX: 2R
		let (input_reserve, output_reserve) =
			Pallet::<T>::get_reserves(sold_asset_id, bought_asset_id)?;

		ensure!(input_reserve.checked_add(sold_asset_amount).is_some(), Error::<T>::MathOverflow);

		// Calculate bought asset amount to be received by paying sold asset amount
		let bought_asset_amount =
			Pallet::<T>::calculate_sell_price(input_reserve, output_reserve, sold_asset_amount)?;

		// Ensure user has enough tokens to sell
		<T as Config>::Currency::ensure_can_withdraw(
			sold_asset_id.into(),
			&sender,
			sold_asset_amount.into(),
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		let vault = Pallet::<T>::account_id();
		let treasury_account: T::AccountId = Self::treasury_account_id();
		let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

		// Transfer of fees, before tx can fail on min amount out
		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&vault,
			pool_fee_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&treasury_account,
			treasury_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&bnb_treasury_account,
			buy_and_burn_amount.into(),
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
				(sold_asset_amount
					.checked_sub(buy_and_burn_amount + treasury_amount + pool_fee_amount)
					.ok_or_else(|| DispatchError::from(Error::<T>::SoldAmountTooLow))?)
				.into(),
				ExistenceRequirement::KeepAlive,
			)?;
			<T as Config>::Currency::transfer(
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

			// MAX 2R 1W
			Pallet::<T>::set_reserves(
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

			Pallet::<T>::deposit_event(Event::AssetsSwapped(
				sender,
				sold_asset_id,
				sold_asset_amount,
				bought_asset_id,
				bought_asset_amount,
			));
		}

		// Settle tokens which goes to treasury and for buy and burn purpose
		Pallet::<T>::settle_treasury_and_burn(sold_asset_id, buy_and_burn_amount, treasury_amount)?;

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
		ensure!(
			!T::DisabledTokens::contains(&sold_asset_id)
				&& !T::DisabledTokens::contains(&bought_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

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

		let buy_and_burn_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::BuyAndBurnFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
			+ 1;

		let treasury_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::TreasuryFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
			+ 1;

		let pool_fee_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::PoolFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
			+ 1;

		// for future implementation of min fee if necessary
		// let min_fee: u128 = 0;
		// if buy_and_burn_amount + treasury_amount + pool_fee_amount < min_fee {
		//     buy_and_burn_amount = min_fee * Self::total_fee() / T::BuyAndBurnFeePercentage::get();
		//     treasury_amount = min_fee * Self::total_fee() / T::TreasuryFeePercentage::get();
		//     pool_fee_amount = min_fee - buy_and_burn_amount - treasury_amount;
		// }

		ensure!(input_reserve.checked_add(sold_asset_amount).is_some(), Error::<T>::MathOverflow);

		// Ensure user has enough tokens to sell
		<T as Config>::Currency::ensure_can_withdraw(
			sold_asset_id.into(),
			&sender,
			sold_asset_amount.into(),
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		let vault = Pallet::<T>::account_id();
		let treasury_account: T::AccountId = Self::treasury_account_id();
		let bnb_treasury_account: T::AccountId = Self::bnb_treasury_account_id();

		// Transfer of fees, before tx can fail on min amount out
		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&vault,
			pool_fee_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&treasury_account,
			treasury_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		<T as Config>::Currency::transfer(
			sold_asset_id.into(),
			&sender,
			&bnb_treasury_account,
			buy_and_burn_amount.into(),
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
				(sold_asset_amount
					.checked_sub(buy_and_burn_amount + treasury_amount + pool_fee_amount)
					.ok_or_else(|| DispatchError::from(Error::<T>::SoldAmountTooLow))?)
				.into(),
				ExistenceRequirement::KeepAlive,
			)?;
			<T as Config>::Currency::transfer(
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
			Pallet::<T>::set_reserves(
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

			Pallet::<T>::deposit_event(Event::AssetsSwapped(
				sender,
				sold_asset_id,
				sold_asset_amount,
				bought_asset_id,
				bought_asset_amount,
			));
		}
		// Settle tokens which goes to treasury and for buy and burn purpose
		Pallet::<T>::settle_treasury_and_burn(sold_asset_id, buy_and_burn_amount, treasury_amount)?;

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
		activate_minted_liquidity: bool,
	) -> Result<(Self::CurrencyId, Self::Balance), DispatchError> {
		let vault = Pallet::<T>::account_id();

		// Ensure pool exists
		ensure!(
			(LiquidityAssets::<T>::contains_key((first_asset_id, second_asset_id))
				|| LiquidityAssets::<T>::contains_key((second_asset_id, first_asset_id))),
			Error::<T>::NoSuchPool,
		);

		// TODO move ensure in get_liq_asset ?
		// Get liquidity token id
		let liquidity_asset_id = Pallet::<T>::get_liquidity_asset(first_asset_id, second_asset_id)?;

		// Get token reserves
		let (first_asset_reserve, second_asset_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;
		let total_liquidity_assets: Self::Balance =
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();

		// Calculation of required second asset amount and received liquidity token amount
		ensure!(!first_asset_reserve.is_zero(), Error::<T>::DivisionByZero);
		let second_asset_amount = multiply_by_rational_with_rounding(
			first_asset_amount,
			second_asset_reserve,
			first_asset_reserve,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let liquidity_assets_minted = multiply_by_rational_with_rounding(
			first_asset_amount,
			total_liquidity_assets,
			first_asset_reserve,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?;

		ensure!(
			second_asset_amount <= expected_second_asset_amount,
			Error::<T>::SecondAssetAmountExceededExpectations,
		);

		// Ensure minting amounts are not zero
		ensure!(
			!first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
			Error::<T>::ZeroAmount,
		);

		// Ensure user has enough withdrawable tokens to create pool in amounts required

		<T as Config>::Currency::ensure_can_withdraw(
			first_asset_id.into(),
			&sender,
			first_asset_amount.into(),
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		<T as Config>::Currency::ensure_can_withdraw(
			second_asset_id.into(),
			&sender,
			second_asset_amount.into(),
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		// Transfer of token amounts from user to vault
		<T as Config>::Currency::transfer(
			first_asset_id.into(),
			&sender,
			&vault,
			first_asset_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;
		<T as Config>::Currency::transfer(
			second_asset_id.into(),
			&sender,
			&vault,
			second_asset_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		// Creating new liquidity tokens to user
		<T as Config>::Currency::mint(
			liquidity_asset_id.into(),
			&sender,
			liquidity_assets_minted.into(),
		)?;

		// Liquidity minting functions not triggered on not promoted pool
		if <T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id).is_some()
			&& activate_minted_liquidity
		{
			// The reserve from free_balance will not fail the asset were just minted into free_balance
			Pallet::<T>::set_liquidity_minting_checkpoint_v2(
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

	fn provide_liquidity_with_conversion(
		sender: T::AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		provided_asset_id: Self::CurrencyId,
		provided_asset_amount: Self::Balance,
		activate_minted_liquidity: bool,
	) -> Result<(Self::CurrencyId, Self::Balance), DispatchError> {
		// checks
		ensure!(!provided_asset_amount.is_zero(), Error::<T>::ZeroAmount,);

		let (first_reserve, second_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;

		let (reserve, other_reserve, other_asset_id) = if provided_asset_id == first_asset_id {
			(first_reserve, second_reserve, second_asset_id)
		} else if provided_asset_id == second_asset_id {
			(second_reserve, first_reserve, first_asset_id)
		} else {
			return Err(DispatchError::from(Error::<T>::FunctionNotAvailableForThisToken));
		};

		// Ensure user has enough tokens to sell
		<T as Config>::Currency::ensure_can_withdraw(
			provided_asset_id.into(),
			&sender,
			provided_asset_amount.into(),
			WithdrawReasons::all(),
			// Does not fail due to earlier ensure
			Default::default(),
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		// calculate sell
		let swap_amount =
			Pallet::<T>::calculate_balanced_sell_amount(provided_asset_amount, reserve)?;

		let bought_amount = Pallet::<T>::calculate_sell_price(reserve, other_reserve, swap_amount)?;

		<Self as XykFunctionsTrait<T::AccountId>>::sell_asset(
			sender.clone(),
			provided_asset_id,
			other_asset_id,
			swap_amount,
			bought_amount,
		)?;

		let mint_amount = provided_asset_amount
			.checked_sub(swap_amount)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		log!(
			info,
			"provide_liquidity_with_conversion: ({:?}, {}, {}, {}, {}) -> ({}, {})",
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
		<Self as XykFunctionsTrait<T::AccountId>>::mint_liquidity(
			sender,
			other_asset_id,
			provided_asset_id,
			bought_amount,
			Self::Balance::MAX,
			activate_minted_liquidity,
		)
	}

	fn burn_liquidity(
		sender: T::AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		liquidity_asset_amount: Self::Balance,
	) -> DispatchResult {
		let vault = Pallet::<T>::account_id();

		ensure!(
			!T::DisabledTokens::contains(&first_asset_id)
				&& !T::DisabledTokens::contains(&second_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		let liquidity_asset_id = Pallet::<T>::get_liquidity_asset(first_asset_id, second_asset_id)?;

		// First let's check how much we can actually burn
		let max_instant_unreserve_amount =
			T::ActivationReservesProvider::get_max_instant_unreserve_amount(
				liquidity_asset_id,
				&sender,
			);

		// Get token reserves and liquidity asset id
		let (first_asset_reserve, second_asset_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;

		// Ensure user has enought liquidity tokens to burn
		let liquidity_token_available_balance =
			<T as Config>::Currency::available_balance(liquidity_asset_id.into(), &sender).into();

		ensure!(
			liquidity_token_available_balance
				.checked_add(max_instant_unreserve_amount)
				.ok_or(Error::<T>::MathOverflow)?
				>= liquidity_asset_amount,
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
		if !to_be_deactivated.is_zero() {
			Pallet::<T>::set_liquidity_burning_checkpoint_v2(
				sender.clone(),
				liquidity_asset_id,
				to_be_deactivated,
			)?;
		}

		// Calculate first and second token amounts depending on liquidity amount to burn
		let (first_asset_amount, second_asset_amount) = Pallet::<T>::get_burn_amount_reserves(
			first_asset_reserve,
			second_asset_reserve,
			liquidity_asset_id,
			liquidity_asset_amount,
		)?;

		let total_liquidity_assets: Balance =
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();

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
		<T as Config>::Currency::transfer(
			first_asset_id.into(),
			&vault,
			&sender,
			first_asset_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;
		<T as Config>::Currency::transfer(
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
			Pools::<T>::remove((first_asset_id, second_asset_id));
			Pools::<T>::remove((second_asset_id, first_asset_id));
			LiquidityAssets::<T>::remove((first_asset_id, second_asset_id));
			LiquidityAssets::<T>::remove((second_asset_id, first_asset_id));
			LiquidityPools::<T>::remove(liquidity_asset_id);
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
				"pool-state: [({}, {}) -> {}, ({}, {}) -> {}]",
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
			liquidity_asset_amount.into(),
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

	// MAX: 3R 2W
	fn claim_rewards_v2(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		mangata_amount: Self::Balance,
	) -> DispatchResult {
		let mangata_id: TokenId = Self::native_token_id();
		let claimable_rewards =
			Pallet::<T>::calculate_rewards_amount_v2(user.clone(), liquidity_asset_id)?;

		ensure!(mangata_amount <= claimable_rewards, Error::<T>::NotEnoughRewardsEarned);

		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);

		let mut not_yet_claimed_rewards = rewards_info.rewards_not_yet_claimed;
		let mut already_claimed_rewards = rewards_info.rewards_already_claimed;

		if mangata_amount <= not_yet_claimed_rewards {
			not_yet_claimed_rewards = not_yet_claimed_rewards - mangata_amount;
		}
		// user is taking out more rewards then rewards from LP which was already removed from pool, additional work needs to be removed from pool and user
		else {
			// rewards to claim on top of rewards from LP which was already removed from pool
			let rewards_to_claim = mangata_amount
				.checked_sub(not_yet_claimed_rewards)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
			already_claimed_rewards = already_claimed_rewards
				.checked_add(rewards_to_claim)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
			not_yet_claimed_rewards = 0 as u128;
		}

		let rewards_info_new: RewardInfo = RewardInfo {
			activated_amount: rewards_info.activated_amount,
			rewards_not_yet_claimed: not_yet_claimed_rewards,
			rewards_already_claimed: already_claimed_rewards,
			last_checkpoint: rewards_info.last_checkpoint,
			pool_ratio_at_last_checkpoint: rewards_info.pool_ratio_at_last_checkpoint,
			missing_at_last_checkpoint: rewards_info.missing_at_last_checkpoint,
		};

		<T as Config>::Currency::transfer(
			mangata_id.into(),
			&<T as Config>::LiquidityMiningIssuanceVault::get(),
			&user,
			mangata_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info_new);

		Pallet::<T>::deposit_event(Event::RewardsClaimed(user, liquidity_asset_id, mangata_amount));

		Ok(())
	}

	fn claim_rewards_all_v2(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
	) -> Result<Self::Balance, DispatchError> {
		let mangata_id: TokenId = Self::native_token_id();

		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);

		let pool_rewards_ratio_current =
			<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id)
				.ok_or_else(|| DispatchError::from(Error::<T>::NotAPromotedPool))?;

		let current_rewards = Pallet::<T>::calculate_rewards_v2(
			rewards_info.activated_amount,
			liquidity_asset_id,
			rewards_info.last_checkpoint,
			rewards_info.pool_ratio_at_last_checkpoint,
			rewards_info.missing_at_last_checkpoint,
			pool_rewards_ratio_current,
		)?;

		let rewards_not_yet_claimed = rewards_info.rewards_not_yet_claimed;
		let rewards_already_claimed = rewards_info.rewards_already_claimed;

		let total_available_rewards = current_rewards
			.checked_add(rewards_not_yet_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsAllMathError))?
			.checked_sub(rewards_already_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsAllMathError))?;

		let rewards_info_new: RewardInfo = RewardInfo {
			activated_amount: rewards_info.activated_amount,
			rewards_not_yet_claimed: 0 as u128,
			rewards_already_claimed: current_rewards,
			last_checkpoint: rewards_info.last_checkpoint,
			pool_ratio_at_last_checkpoint: rewards_info.pool_ratio_at_last_checkpoint,
			missing_at_last_checkpoint: rewards_info.missing_at_last_checkpoint,
		};

		<T as Config>::Currency::transfer(
			mangata_id.into(),
			&<T as Config>::LiquidityMiningIssuanceVault::get(),
			&user,
			total_available_rewards.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info_new.clone());

		Pallet::<T>::deposit_event(Event::RewardsClaimed(
			user,
			liquidity_asset_id,
			total_available_rewards,
		));

		Ok(total_available_rewards)
	}

	fn update_pool_promotion(
		liquidity_token_id: TokenId,
		liquidity_mining_issuance_weight: Option<u8>,
	) -> DispatchResult {
		<T as Config>::PoolPromoteApi::update_pool_promotion(
			liquidity_token_id,
			liquidity_mining_issuance_weight,
		);

		Pallet::<T>::deposit_event(Event::PoolPromotionUpdated(
			liquidity_token_id,
			liquidity_mining_issuance_weight,
		));

		Ok(())
	}

	fn activate_liquidity_v2(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		amount: Self::Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		ensure!(
			<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id).is_some(),
			Error::<T>::NotAPromotedPool
		);

		ensure!(
			<T as Config>::ActivationReservesProvider::can_activate(
				liquidity_asset_id.into(),
				&user,
				amount,
				use_balance_from.clone()
			),
			Error::<T>::NotEnoughAssets
		);

		Pallet::<T>::set_liquidity_minting_checkpoint_v2(
			user.clone(),
			liquidity_asset_id,
			amount,
			use_balance_from,
		)?;

		Pallet::<T>::deposit_event(Event::LiquidityActivated(user, liquidity_asset_id, amount));

		Ok(())
	}

	fn deactivate_liquidity_v2(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult {
		ensure!(
			<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id).is_some(),
			Error::<T>::NotAPromotedPool
		);

		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);

		ensure!(rewards_info.activated_amount >= amount, Error::<T>::NotEnoughAssets);

		Pallet::<T>::set_liquidity_burning_checkpoint_v2(user.clone(), liquidity_asset_id, amount)?;

		Pallet::<T>::deposit_event(Event::LiquidityDeactivated(user, liquidity_asset_id, amount));

		Ok(())
	}

	fn rewards_migrate_v1_to_v2(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
	) -> DispatchResult {
		let mangata_id: TokenId = Self::native_token_id();

		ensure!(
			<T as Config>::PoolPromoteApi::get_pool_rewards(liquidity_asset_id).is_some(),
			Error::<T>::NotAPromotedPool
		);
		//READING ALL NECESSARY STORAGE
		let pool_ratio_current =
			<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id)
				.ok_or_else(|| DispatchError::from(Error::<T>::NotAPromotedPool))?;
		let available_rewards_for_pool: U256 = U256::from(
			<T as Config>::PoolPromoteApi::get_pool_rewards(liquidity_asset_id)
				.ok_or_else(|| DispatchError::from(Error::<T>::NotAPromotedPool))?,
		);
		let liquidity_assets_amount: Balance =
			LiquidityMiningActiveUser::<T>::get((&user, &liquidity_asset_id));
		let pool_activated_amount: Balance =
			LiquidityMiningActivePool::<T>::get(&liquidity_asset_id);
		let burned_not_claimed_rewards =
			LiquidityMiningUserToBeClaimed::<T>::get((user.clone(), &liquidity_asset_id));
		let already_claimed_rewards =
			LiquidityMiningUserClaimed::<T>::get((user.clone(), &liquidity_asset_id));
		let current_time: u32 = <frame_system::Pallet<T>>::block_number().saturated_into::<u32>()
			/ T::RewardsDistributionPeriod::get();
		let (
			user_last_checkpoint,
			user_cummulative_work_in_last_checkpoint,
			user_missing_at_last_checkpoint,
		) = LiquidityMiningUser::<T>::try_get((&user, &liquidity_asset_id))
			.unwrap_or_else(|_| (current_time, U256::from(0), U256::from(0)));
		let (
			pool_last_checkpoint,
			pool_cummulative_work_in_last_checkpoint,
			pool_missing_at_last_checkpoint,
		) = LiquidityMiningPool::<T>::try_get(&liquidity_asset_id)
			.unwrap_or_else(|_| (current_time, U256::from(0), U256::from(0)));

		//CALCULATE REWARDS
		let time_passed_user = current_time - user_last_checkpoint;
		let asymptote_u256_user: U256 = liquidity_assets_amount.into();
		let cummulative_work_new_max_possible_user: U256 = asymptote_u256_user
			.checked_mul(U256::from(time_passed_user))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let base_user = user_missing_at_last_checkpoint
			.checked_mul(U256::from(106))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			/ U256::from(6);
		let q_pow_user = Self::calculate_q_pow(1.06, time_passed_user);
		let cummulative_missing_new_user = base_user - base_user * REWARDS_PRECISION / q_pow_user;
		let cummulative_work_new_user = cummulative_work_new_max_possible_user
			.checked_sub(cummulative_missing_new_user)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let work_user = user_cummulative_work_in_last_checkpoint + cummulative_work_new_user;
		let time_passed_pool = current_time - pool_last_checkpoint;
		let asymptote_u256_pool: U256 = pool_activated_amount.into();
		let cummulative_work_new_max_possible_pool: U256 = asymptote_u256_pool
			.checked_mul(U256::from(time_passed_pool))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let base_pool = pool_missing_at_last_checkpoint
			.checked_mul(U256::from(106))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			/ U256::from(6);
		let q_pow_pool = Self::calculate_q_pow(1.06, time_passed_pool);
		let cummulative_missing_new_pool = base_pool - base_pool * REWARDS_PRECISION / q_pow_pool;
		let cummulative_work_new_pool = cummulative_work_new_max_possible_pool
			.checked_sub(cummulative_missing_new_pool)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let work_pool = pool_cummulative_work_in_last_checkpoint + cummulative_work_new_pool;

		let mut current_rewards = Balance::try_from(0).unwrap();
		if work_user != U256::from(0) && work_pool != U256::from(0) {
			current_rewards = Balance::try_from(
				available_rewards_for_pool
					.checked_mul(work_user)
					.ok_or_else(|| DispatchError::from(Error::<T>::NotEnoughRewardsEarned))?
					.checked_div(work_pool)
					.ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?,
			)
			.map_err(|_| DispatchError::from(Error::<T>::NotEnoughRewardsEarned))?;
		}

		let total_rewards = current_rewards
			.checked_add(burned_not_claimed_rewards)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			.checked_sub(already_claimed_rewards)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		//TRANSFER ALL REWARDS TO USER
		<T as Config>::Currency::transfer(
			mangata_id.into(),
			&<T as Config>::LiquidityMiningIssuanceVault::get(),
			&user,
			total_rewards.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		Pallet::<T>::deposit_event(Event::RewardsClaimed(
			user.clone(),
			liquidity_asset_id,
			total_rewards,
		));

		// MODIFY REW1 POOL VALUES
		let q_pow_pool = Self::calculate_q_pow(1.06, time_passed_pool);
		let pool_missing_at_checkpoint: U256 =
			pool_missing_at_last_checkpoint * U256::from(REWARDS_PRECISION) / q_pow_pool;

		let user_missing_at_checkpoint: U256 =
			user_missing_at_last_checkpoint * U256::from(REWARDS_PRECISION) / q_pow_user;

		if pool_activated_amount != liquidity_assets_amount {
			let pool_work_new = work_pool
				.checked_sub(work_user)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
			let pool_missing_new = pool_missing_at_checkpoint
				.checked_sub(user_missing_at_checkpoint)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
			LiquidityMiningPool::<T>::insert(
				&liquidity_asset_id,
				(current_time, pool_work_new, pool_missing_new),
			);

			LiquidityMiningActivePool::<T>::try_mutate(liquidity_asset_id, |active_amount| {
				if let Some(val) = active_amount.checked_sub(liquidity_assets_amount) {
					*active_amount = val;
					Ok(())
				} else {
					Err(())
				}
			})
			.map_err(|_| DispatchError::from(Error::<T>::NotEnoughAssets))?;

			<T as Config>::PoolPromoteApi::claim_pool_rewards(
				liquidity_asset_id.into(),
				total_rewards,
			);
		} else {
			// IF LAST REMOVE POOL STORAGE
			LiquidityMiningActivePool::<T>::remove(&liquidity_asset_id);
			LiquidityMiningPool::<T>::remove(&liquidity_asset_id);
			<T as Config>::PoolPromoteApi::claim_pool_rewards(
				liquidity_asset_id.into(),
				total_rewards,
			);
			// THERE WILL BE SOME REWARD LEFT
		}

		// REMOVE USER REW1 STORAGE
		LiquidityMiningUser::<T>::remove((user.clone(), &liquidity_asset_id));
		LiquidityMiningUserToBeClaimed::<T>::remove((user.clone(), &liquidity_asset_id));
		LiquidityMiningActiveUser::<T>::remove((user.clone(), &liquidity_asset_id));
		LiquidityMiningUserClaimed::<T>::remove((user.clone(), &liquidity_asset_id));

		// ADD USER INFO TO REW2 STORAGE
		let rewards_info_new: RewardInfo = RewardInfo {
			activated_amount: liquidity_assets_amount,
			rewards_not_yet_claimed: 0_u128,
			rewards_already_claimed: 0_u128,
			last_checkpoint: current_time,
			pool_ratio_at_last_checkpoint: pool_ratio_current,
			missing_at_last_checkpoint: user_missing_at_checkpoint,
		};

		LiquidityMiningActivePoolV2::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_add(liquidity_assets_amount) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info_new);

		Ok(())
	}

	// This function has not been verified
	fn get_tokens_required_for_minting(
		liquidity_asset_id: Self::CurrencyId,
		liquidity_token_amount: Self::Balance,
	) -> Result<(Self::CurrencyId, Self::Balance, Self::CurrencyId, Self::Balance), DispatchError> {
		let (first_asset_id, second_asset_id) =
			LiquidityPools::<T>::get(liquidity_asset_id).ok_or(Error::<T>::NoSuchLiquidityAsset)?;
		let (first_asset_reserve, second_asset_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;
		let total_liquidity_assets: Balance =
			<T as Config>::Currency::total_issuance(liquidity_asset_id.into()).into();

		ensure!(!total_liquidity_assets.is_zero(), Error::<T>::DivisionByZero);
		let second_asset_amount = multiply_by_rational_with_rounding(
			liquidity_token_amount,
			second_asset_reserve,
			total_liquidity_assets,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
		.checked_add(1)
		.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let first_asset_amount = multiply_by_rational_with_rounding(
			liquidity_token_amount,
			first_asset_reserve,
			total_liquidity_assets,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)?
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

		Ok((first_asset_id, first_asset_amount, second_asset_id, second_asset_amount))
	}

	fn is_liquidity_token(liquidity_asset_id: TokenId) -> bool {
		LiquidityPools::<T>::get(liquidity_asset_id).is_some()
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

	fn get_liquidity_asset(
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
	) -> Result<TokenId, DispatchError>;

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

	fn get_pool_state(liquidity_token_id: Self::CurrencyId) -> Option<(Balance, Balance)>;
}

pub trait AssetMetadataMutationTrait {
	fn set_asset_info(
		asset: TokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u32,
	) -> DispatchResult;
}

impl<T: Config> Valuate for Pallet<T> {
	type Balance = Balance;

	type CurrencyId = TokenId;

	fn get_liquidity_asset(
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
	) -> Result<TokenId, DispatchError> {
		Pallet::<T>::get_liquidity_asset(first_asset_id, second_asset_id)
	}

	fn get_liquidity_token_mga_pool(
		liquidity_token_id: Self::CurrencyId,
	) -> Result<(Self::CurrencyId, Self::CurrencyId), DispatchError> {
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
		liquidity_token_id: Self::CurrencyId,
		liquidity_token_amount: Self::Balance,
	) -> Self::Balance {
		let (mga_token_id, other_token_id) =
			match Self::get_liquidity_token_mga_pool(liquidity_token_id) {
				Ok(pool) => pool,
				Err(_) => return Default::default(),
			};

		let mga_token_reserve = match Pallet::<T>::get_reserves(mga_token_id, other_token_id) {
			Ok(reserves) => reserves.0,
			Err(_) => return Default::default(),
		};

		let liquidity_token_reserve: Balance =
			<T as Config>::Currency::total_issuance(liquidity_token_id.into()).into();

		if liquidity_token_reserve.is_zero() {
			return Default::default();
		}

		multiply_by_rational_with_rounding(
			mga_token_reserve,
			liquidity_token_amount,
			liquidity_token_reserve,
			Rounding::Down,
		)
		.unwrap_or(Balance::max_value())
	}

	fn scale_liquidity_by_mga_valuation(
		mga_valuation: Self::Balance,
		liquidity_token_amount: Self::Balance,
		mga_token_amount: Self::Balance,
	) -> Self::Balance {
		if mga_valuation.is_zero() {
			return Default::default();
		}

		multiply_by_rational_with_rounding(
			liquidity_token_amount,
			mga_token_amount,
			mga_valuation,
			Rounding::Down,
		)
		.unwrap_or(Balance::max_value())
	}

	fn get_pool_state(
		liquidity_token_id: Self::CurrencyId,
	) -> Option<(Self::Balance, Self::Balance)> {
		let (mga_token_id, other_token_id) =
			match Self::get_liquidity_token_mga_pool(liquidity_token_id) {
				Ok(pool) => pool,
				Err(_) => return None,
			};

		let mga_token_reserve = match Pallet::<T>::get_reserves(mga_token_id, other_token_id) {
			Ok(reserves) => reserves.0,
			Err(_) => return None,
		};

		let liquidity_token_reserve: Balance =
			<T as Config>::Currency::total_issuance(liquidity_token_id.into()).into();

		if liquidity_token_reserve.is_zero() {
			return None;
		}

		Some((mga_token_reserve, liquidity_token_reserve))
	}
}

impl<T: Config> PoolCreateApi for Pallet<T> {
	type AccountId = T::AccountId;

	fn pool_exists(first: TokenId, second: TokenId) -> bool {
		Pools::<T>::contains_key((first, second)) || Pools::<T>::contains_key((second, first))
	}

	fn pool_create(
		account: Self::AccountId,
		first: TokenId,
		first_amount: Balance,
		second: TokenId,
		second_amount: Balance,
	) -> Option<(TokenId, Balance)> {
		match <Self as XykFunctionsTrait<Self::AccountId>>::create_pool(
			account,
			first,
			first_amount,
			second,
			second_amount,
		) {
			Ok(_) => LiquidityAssets::<T>::get((first, second)).map(|asset_id| {
				(asset_id, <T as Config>::Currency::total_issuance(asset_id.into()).into())
			}),
			Err(e) => {
				log!(error, "cannot create pool {:?}!", e);
				None
			},
		}
	}
}

impl<T: Config> ActivedPoolQueryApi for Pallet<T> {
	fn get_pool_activate_amount(liquidity_token_id: TokenId) -> Option<Balance> {
		LiquidityMiningActivePoolV2::<T>::try_get(liquidity_token_id).ok()
	}
}

impl<T: Config> mp_bootstrap::RewardsApi for Pallet<T> {
	type AccountId = T::AccountId;

	fn can_activate(liquidity_asset_id: TokenId) -> bool {
		<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id).is_some()
	}

	fn activate_liquidity_tokens(
		user: &Self::AccountId,
		liquidity_asset_id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		<Self as XykFunctionsTrait<T::AccountId>>::activate_liquidity_v2(
			user.clone(),
			liquidity_asset_id,
			amount,
			Some(ActivateKind::AvailableBalance),
		)
	}

	fn update_pool_promotion(
		liquidity_token_id: TokenId,
		liquidity_mining_issuance_weight: Option<u8>,
	) {
		<T as Config>::PoolPromoteApi::update_pool_promotion(
			liquidity_token_id,
			liquidity_mining_issuance_weight,
		);

		Pallet::<T>::deposit_event(Event::PoolPromotionUpdated(
			liquidity_token_id,
			liquidity_mining_issuance_weight,
		));
	}
}
