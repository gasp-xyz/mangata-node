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
	traits::{ExistenceRequirement, Get, WithdrawReasons},
	transactional, Parameter,
};
use frame_system::pallet_prelude::*;
use mangata_primitives::{Balance, TokenId};
use mp_bootstrap::PoolCreateApi;
use mp_multipurpose_liquidity::ActivateKind;
use mp_traits::{ActivationReservesProviderTrait, XykFunctionsTrait};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use pallet_issuance::{ComputeIssuance, PoolPromoteApi};
use pallet_vesting_mangata::MultiTokenVestingLocks;
use sp_arithmetic::{helpers_128bit::multiply_by_rational_with_rounding, per_things::Rounding};
use sp_runtime::traits::{
	AccountIdConversion, AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member,
	SaturatedConversion, Zero,
};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	prelude::*,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

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
const Q: f64 = 1.06;

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

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ActivationReservesProvider: ActivationReservesProviderTrait<
			AccountId = Self::AccountId,
		>;
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
		type DisallowedPools: Contains<(TokenId, TokenId)>;
		type DisabledTokens: Contains<TokenId>;
		type VestingProvider: MultiTokenVestingLocks<Self::AccountId, Self::BlockNumber>;
		type AssetMetadataMutation: AssetMetadataMutationTrait;
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
		/// Pool considting of passed tokens id is blacklisted
		DisallowedPool,
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
	#[pallet::getter(fn liquidity_mining_user)]
	pub type LiquidityMiningUser<T: Config> =
		StorageMap<_, Blake2_256, (AccountIdOf<T>, TokenId), (u32, U256, U256), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_pool)]
	pub type LiquidityMiningPool<T: Config> =
		StorageMap<_, Blake2_256, TokenId, (u32, U256, U256), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_user_to_be_claimed)]
	pub type LiquidityMiningUserToBeClaimed<T: Config> =
		StorageMap<_, Blake2_256, (AccountIdOf<T>, TokenId), u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_active_user)]
	pub type LiquidityMiningActiveUser<T: Config> =
		StorageMap<_, Twox64Concat, (AccountIdOf<T>, TokenId), u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_active_pool)]
	pub type LiquidityMiningActivePool<T: Config> =
		StorageMap<_, Twox64Concat, TokenId, u128, ValueQuery>;

	// stores user claimed rewards, for future calculation of rewards
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
						assert!(
							created_liquidity_token_id == *liquidity_token_id,
							"Assets not initialized in the expected sequence"
						);
						assert!(
							<Pallet<T> as XykFunctionsTrait<T::AccountId>>::create_pool(
								account_id.clone(),
								*native_token_id,
								*native_token_amount,
								*pooled_token_id,
								*pooled_token_amount
							)
							.is_ok(),
							"Pool creation failed"
						);
					}
				},
			)
		}
	}

	// XYK extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_pool())]
		pub fn create_pool(
			origin: OriginFor<T>,
			first_asset_id: TokenId,
			first_asset_amount: Balance,
			second_asset_id: TokenId,
			second_asset_amount: Balance,
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
		#[pallet::weight(T::WeightInfo::sell_asset())]
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

		#[pallet::weight(T::WeightInfo::buy_asset())]
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

		#[pallet::weight(T::WeightInfo::mint_liquidity_using_vesting_native_tokens())]
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
				Pallet::<T>::get_liquidity_asset(T::NativeCurrencyId::get(), second_asset_id)?;

			ensure!(Self::is_promoted_pool(liquidity_asset_id), Error::<T>::NotAPromotedPool);

			let (unlocked_amount, vesting_starting_block, vesting_ending_block_as_balance): (
				Balance,
				T::BlockNumber,
				Balance,
			) = T::VestingProvider::unlock_tokens_by_vesting_index(
				&sender,
				T::NativeCurrencyId::get().into(),
				native_asset_vesting_index,
				vesting_native_asset_unlock_some_amount_or_all.map(Into::into),
			)
			.map(|x| (x.0.into(), x.1.into(), x.2.into()))?;

			let (liquidity_token_id, liquidity_assets_minted) =
				<Self as XykFunctionsTrait<T::AccountId>>::mint_liquidity(
					sender.clone(),
					T::NativeCurrencyId::get(),
					second_asset_id,
					unlocked_amount,
					expected_second_asset_amount,
					false,
				)?;

			T::VestingProvider::lock_tokens(
				&sender,
				liquidity_token_id.into(),
				liquidity_assets_minted.into(),
				Some(vesting_starting_block.into()),
				vesting_ending_block_as_balance.into(),
			)?;

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_liquidity_using_vesting_native_tokens())]
		#[transactional]
		pub fn mint_liquidity_using_vesting_native_tokens(
			origin: OriginFor<T>,
			vesting_native_asset_amount: Balance,
			second_asset_id: TokenId,
			expected_second_asset_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let liquidity_asset_id =
				Pallet::<T>::get_liquidity_asset(T::NativeCurrencyId::get(), second_asset_id)?;

			ensure!(Self::is_promoted_pool(liquidity_asset_id), Error::<T>::NotAPromotedPool);

			let (vesting_starting_block, vesting_ending_block_as_balance): (
				T::BlockNumber,
				Balance,
			) = T::VestingProvider::unlock_tokens(
				&sender,
				T::NativeCurrencyId::get().into(),
				vesting_native_asset_amount.into(),
			)
			.map(|x| (x.0.into(), x.1.into()))?;

			let (liquidity_token_id, liquidity_assets_minted) =
				<Self as XykFunctionsTrait<T::AccountId>>::mint_liquidity(
					sender.clone(),
					T::NativeCurrencyId::get(),
					second_asset_id,
					vesting_native_asset_amount,
					expected_second_asset_amount,
					false,
				)?;

			T::VestingProvider::lock_tokens(
				&sender,
				liquidity_token_id.into(),
				liquidity_assets_minted.into(),
				Some(vesting_starting_block.into()),
				vesting_ending_block_as_balance.into(),
			)?;

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_liquidity())]
		pub fn mint_liquidity(
			origin: OriginFor<T>,
			first_asset_id: TokenId,
			second_asset_id: TokenId,
			first_asset_amount: Balance,
			expected_second_asset_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(
				!T::DisabledTokens::contains(&first_asset_id) &&
					!T::DisabledTokens::contains(&second_asset_id),
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

		#[pallet::weight(T::WeightInfo::burn_liquidity())]
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
		#[pallet::weight(T::WeightInfo::claim_rewards())]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::claim_rewards(
				sender,
				liquidity_token_id,
				amount,
			)?;

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::promote_pool())]
		pub fn promote_pool(origin: OriginFor<T>, liquidity_token_id: TokenId) -> DispatchResult {
			ensure_root(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::promote_pool(liquidity_token_id)
		}

		#[transactional]
		#[pallet::weight(T::WeightInfo::activate_liquidity())]
		pub fn activate_liquidity(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
			use_balance_from: Option<ActivateKind>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::activate_liquidity(
				sender,
				liquidity_token_id,
				amount,
				use_balance_from,
			)
		}

		#[transactional]
		#[pallet::weight(T::WeightInfo::deactivate_liquidity())]
		pub fn deactivate_liquidity(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as XykFunctionsTrait<T::AccountId>>::deactivate_liquidity(
				sender,
				liquidity_token_id,
				amount,
			)
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

	pub fn is_promoted_pool(liquidity_asset_id: TokenId) -> bool {
		<T as Config>::PoolPromoteApi::get_pool_rewards(liquidity_asset_id).is_some()
	}

	pub fn calculate_rewards_amount(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
	) -> Result<Balance, DispatchError> {
		ensure!(Self::is_promoted_pool(liquidity_asset_id), Error::<T>::NotAPromotedPool);

		let current_time: u32 = <frame_system::Pallet<T>>::block_number().saturated_into::<u32>() /
			T::RewardsDistributionPeriod::get();
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

		let work_user = Self::calculate_work_user(
			user.clone(),
			liquidity_asset_id,
			current_time,
			user_last_checkpoint,
			user_cummulative_work_in_last_checkpoint,
			user_missing_at_last_checkpoint,
		)?;
		let work_pool = Self::calculate_work_pool(
			liquidity_asset_id,
			current_time,
			pool_last_checkpoint,
			pool_cummulative_work_in_last_checkpoint,
			pool_missing_at_last_checkpoint,
		)?;

		let burned_not_claimed_rewards =
			LiquidityMiningUserToBeClaimed::<T>::get((user.clone(), &liquidity_asset_id));
		let current_rewards = Self::calculate_rewards(work_user, work_pool, liquidity_asset_id)?;
		let already_claimed_rewards =
			LiquidityMiningUserClaimed::<T>::get((user, &liquidity_asset_id));

		let total_available_rewards = current_rewards
			.checked_add(burned_not_claimed_rewards)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			.checked_sub(already_claimed_rewards)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		Ok(total_available_rewards)
	}

	pub fn calculate_rewards(
		work_user: U256,
		work_pool: U256,
		liquidity_asset_id: TokenId,
	) -> Result<Balance, DispatchError> {
		let available_rewards_for_pool: U256 = U256::from(
			<T as Config>::PoolPromoteApi::get_pool_rewards(liquidity_asset_id)
				.ok_or_else(|| DispatchError::from(Error::<T>::NotAPromotedPool))?,
		);

		let mut user_mangata_rewards_amount = Balance::try_from(0).unwrap();
		if work_user != U256::from(0) && work_pool != U256::from(0) {
			user_mangata_rewards_amount = Balance::try_from(
				available_rewards_for_pool
					.checked_mul(work_user)
					.ok_or_else(|| DispatchError::from(Error::<T>::NotEnoughtRewardsEarned))?
					.checked_div(work_pool)
					.ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?,
			)
			.map_err(|_| DispatchError::from(Error::<T>::NotEnoughtRewardsEarned))?;
		}

		Ok(user_mangata_rewards_amount)
	}

	// MAX: 0R 0W
	pub fn calculate_work(
		asymptote: Balance,
		time: u32,
		last_checkpoint: u32,
		cummulative_work_in_last_checkpoint: U256,
		missing_at_last_checkpoint: U256,
	) -> Result<U256, DispatchError> {
		let time_passed = time
			.checked_sub(last_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::PastTimeCalculation))?;

		// whole formula: 	missing_at_last_checkpoint*106/6 - missing_at_last_checkpoint*106*precision/6/q_pow
		// q_pow is multiplied by precision, thus there needs to be *precision in numenator as well
		let asymptote_u256: U256 = asymptote.into();
		let cummulative_work_new_max_possible: U256 = asymptote_u256
			.checked_mul(U256::from(time_passed))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let base = missing_at_last_checkpoint
			.checked_mul(U256::from(106))
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))? /
			U256::from(6);

		let precision: u32 = 10000;
		let q_pow = Self::calculate_q_pow(Q, time_passed);

		let cummulative_missing_new = base - base * U256::from(precision) / q_pow;

		let cummulative_work_new = cummulative_work_new_max_possible
			.checked_sub(cummulative_missing_new)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let work_total = cummulative_work_in_last_checkpoint + cummulative_work_new;

		Ok(work_total)
	}

	pub fn calculate_work_pool(
		liquidity_asset_id: TokenId,
		current_time: u32,
		last_checkpoint: u32,
		cummulative_work_in_last_checkpoint: U256,
		missing_at_last_checkpoint: U256,
	) -> Result<U256, DispatchError> {
		let liquidity_assets_amount: Balance =
			LiquidityMiningActivePool::<T>::get(&liquidity_asset_id);

		Self::calculate_work(
			liquidity_assets_amount,
			current_time,
			last_checkpoint,
			cummulative_work_in_last_checkpoint,
			missing_at_last_checkpoint,
		)
	}

	/// 1R
	pub fn calculate_work_user(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
		current_time: u32,
		last_checkpoint: u32,
		cummulative_work_in_last_checkpoint: U256,
		missing_at_last_checkpoint: U256,
	) -> Result<U256, DispatchError> {
		let liquidity_assets_amount: Balance =
			LiquidityMiningActiveUser::<T>::get((&user, &liquidity_asset_id));

		Self::calculate_work(
			liquidity_assets_amount,
			current_time,
			last_checkpoint,
			cummulative_work_in_last_checkpoint,
			missing_at_last_checkpoint,
		)
	}

	pub fn calculate_q_pow(q: f64, pow: u32) -> u128 {
		let precision: u32 = 10000;
		libm::floor(libm::pow(q, pow as f64) * precision as f64) as u128
	}

	/// 0R 0W
	pub fn calculate_missing_at_checkpoint(
		time_passed: u32,
		liquidity_assets_added: Balance,
		missing_at_last_checkpoint: U256,
	) -> Result<U256, DispatchError> {
		let precision: u32 = 10000;
		let q_pow = Self::calculate_q_pow(Q, time_passed);
		let liquidity_assets_added_u256: U256 = liquidity_assets_added.into();

		let missing_at_checkpoint: U256 = liquidity_assets_added_u256 +
			missing_at_last_checkpoint * U256::from(precision) / q_pow;

		Ok(missing_at_checkpoint)
	}

	/// MAX 4R 2W
	pub fn calculate_liquidity_checkpoint(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
		liquidity_assets_added: Balance,
	) -> Result<(u32, U256, U256, U256, U256), DispatchError> {
		let current_time: u32 = <frame_system::Pallet<T>>::block_number().saturated_into::<u32>() /
			T::RewardsDistributionPeriod::get();

		let (
			user_last_checkpoint,
			user_cummulative_work_in_last_checkpoint,
			user_missing_at_last_checkpoint,
		) = LiquidityMiningUser::<T>::try_get((&user, &liquidity_asset_id))
			.unwrap_or_else(|_| (current_time, U256::from(0), U256::from(0)));

		let user_time_passed = current_time
			.checked_sub(user_last_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::PastTimeCalculation))?;
		let user_missing_at_checkpoint = Self::calculate_missing_at_checkpoint(
			user_time_passed,
			liquidity_assets_added,
			user_missing_at_last_checkpoint,
		)?;

		let user_work_total = Self::calculate_work_user(
			user.clone(),
			liquidity_asset_id,
			current_time,
			user_last_checkpoint,
			user_cummulative_work_in_last_checkpoint,
			user_missing_at_last_checkpoint,
		)?;

		let (
			pool_last_checkpoint,
			pool_cummulative_work_in_last_checkpoint,
			pool_missing_at_last_checkpoint,
		) = LiquidityMiningPool::<T>::try_get(&liquidity_asset_id)
			.unwrap_or_else(|_| (current_time, U256::from(0), U256::from(0)));

		let pool_time_passed = current_time
			.checked_sub(pool_last_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::PastTimeCalculation))?;

		let pool_missing_at_checkpoint = Self::calculate_missing_at_checkpoint(
			pool_time_passed,
			liquidity_assets_added,
			pool_missing_at_last_checkpoint,
		)?;
		let pool_work_total = Self::calculate_work_pool(
			liquidity_asset_id,
			current_time,
			pool_last_checkpoint,
			pool_cummulative_work_in_last_checkpoint,
			pool_missing_at_last_checkpoint,
		)?;

		Ok((
			current_time,
			user_work_total,
			user_missing_at_checkpoint,
			pool_work_total,
			pool_missing_at_checkpoint,
		))
	}

	/// MAX: 4R + 4W
	/// 2W + calculate_liquidity_checkpoint(4R+2W)
	pub fn set_liquidity_minting_checkpoint(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
		liquidity_assets_added: Balance,
		use_balance_from: Option<ActivateKind>,
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
		LiquidityMiningPool::<T>::insert(
			&liquidity_asset_id,
			(current_time, pool_work_total, pool_missing_at_checkpoint),
		);

		LiquidityMiningActiveUser::<T>::try_mutate((&user, liquidity_asset_id), |active_amount| {
			if let Some(val) = active_amount.checked_add(liquidity_assets_added) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		LiquidityMiningActivePool::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_add(liquidity_assets_added) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

		// This must not fail due storage edits above
		<T as Config>::ActivationReservesProvider::activate(
			liquidity_asset_id.into(),
			&user,
			liquidity_assets_added.into(),
			use_balance_from,
		)?;

		Ok(())
	}

	pub fn set_liquidity_burning_checkpoint(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
		liquidity_assets_burned: Balance,
	) -> DispatchResult {
		let (
			current_time,
			user_work_total,
			user_missing_at_checkpoint,
			pool_work_total,
			pool_missing_at_checkpoint,
		) = Self::calculate_liquidity_checkpoint(user.clone(), liquidity_asset_id, 0 as u128)?;

		let liquidity_assets_amount: Balance =
			LiquidityMiningActiveUser::<T>::get((&user, &liquidity_asset_id));
		let activated_liquidity_pool: Balance =
			LiquidityMiningActivePool::<T>::get(&liquidity_asset_id);

		let liquidity_assets_burned_u256: U256 = liquidity_assets_burned.into();

		let user_work_burned: U256 = liquidity_assets_burned_u256
			.checked_mul(user_work_total)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			.checked_div(liquidity_assets_amount.into())
			.ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;
		let user_missing_burned: U256 = liquidity_assets_burned_u256
			.checked_mul(user_missing_at_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?
			.checked_div(liquidity_assets_amount.into())
			.ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;

		let user_work_new = user_work_total
			.checked_sub(user_work_burned)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		let user_missing_new = user_missing_at_checkpoint
			.checked_sub(user_missing_burned)
			.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

		let mut pool_work_new = U256::from(0);
		let mut pool_missing_new = U256::from(0);

		if activated_liquidity_pool != liquidity_assets_burned {
			pool_work_new = pool_work_total
				.checked_sub(user_work_burned)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
			pool_missing_new = pool_missing_at_checkpoint
				.checked_sub(user_missing_burned)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		}

		LiquidityMiningUser::<T>::insert(
			(user.clone(), &liquidity_asset_id),
			(current_time, user_work_new, user_missing_new),
		);
		LiquidityMiningPool::<T>::insert(
			&liquidity_asset_id,
			(current_time, pool_work_new, pool_missing_new),
		);

		LiquidityMiningActiveUser::<T>::try_mutate((&user, liquidity_asset_id), |active_amount| {
			if let Some(val) = active_amount.checked_sub(liquidity_assets_burned) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::NotEnoughAssets))?;
		LiquidityMiningActivePool::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_sub(liquidity_assets_burned) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::NotEnoughAssets))?;

		let rewards_burned_not_claimed =
			LiquidityMiningUserToBeClaimed::<T>::get((user.clone(), &liquidity_asset_id));
		let rewards_to_burn =
			Self::calculate_rewards(user_work_burned, pool_work_total, liquidity_asset_id)?;
		let rewards_already_claimed =
			LiquidityMiningUserClaimed::<T>::get((user.clone(), &liquidity_asset_id));

		if rewards_to_burn >= rewards_already_claimed {
			LiquidityMiningUserClaimed::<T>::remove((user.clone(), &liquidity_asset_id));

			let rewards_claimed_new =
				rewards_burned_not_claimed + rewards_to_burn - rewards_already_claimed;

			LiquidityMiningUserToBeClaimed::<T>::insert(
				(&user, liquidity_asset_id),
				rewards_claimed_new,
			);
		} else {
			LiquidityMiningUserClaimed::<T>::insert(
				(user.clone(), &liquidity_asset_id),
				rewards_already_claimed - rewards_to_burn,
			);
		}

		<T as Config>::PoolPromoteApi::claim_pool_rewards(
			liquidity_asset_id.into(),
			rewards_to_burn,
		);

		<T as Config>::ActivationReservesProvider::deactivate(
			liquidity_asset_id.into(),
			&user,
			liquidity_assets_burned.into(),
		);

		Ok(())
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
			return Ok((reserves.0, reserves.1))
		} else if Pools::<T>::contains_key((second_asset_id, first_asset_id)) {
			reserves = Pools::<T>::get((second_asset_id, first_asset_id));
			return Ok((reserves.1, reserves.0))
		} else {
			return Err(DispatchError::from(Error::<T>::NoSuchPool))
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
			return Err(DispatchError::from(Error::<T>::NoSuchPool))
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
		let mangata_id: TokenId = T::NativeCurrencyId::get();
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
				treasury_amount + burn_amount,
			)?;
			let treasury_amount_in_mangata = settle_amount_in_mangata *
				T::TreasuryFeePercentage::get() /
				(T::TreasuryFeePercentage::get() + T::BuyAndBurnFeePercentage::get());

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
			!T::DisabledTokens::contains(&sold_asset_id) &&
				!T::DisabledTokens::contains(&bought_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		let buy_and_burn_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::BuyAndBurnFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)? +
			1;

		let treasury_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::TreasuryFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)? +
			1;

		let pool_fee_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::PoolFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)? +
			1;

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
			return Err(DispatchError::from(Error::<T>::InsufficientOutputAmount))
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
			!T::DisabledTokens::contains(&sold_asset_id) &&
				!T::DisabledTokens::contains(&bought_asset_id),
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
		.ok_or(Error::<T>::UnexpectedFailure)? +
			1;

		let treasury_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::TreasuryFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)? +
			1;

		let pool_fee_amount = multiply_by_rational_with_rounding(
			sold_asset_amount,
			T::PoolFeePercentage::get(),
			10000,
			Rounding::Down,
		)
		.ok_or(Error::<T>::UnexpectedFailure)? +
			1;

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
			return Err(DispatchError::from(Error::<T>::InsufficientInputAmount))
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
			(LiquidityAssets::<T>::contains_key((first_asset_id, second_asset_id)) ||
				LiquidityAssets::<T>::contains_key((second_asset_id, first_asset_id))),
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
		if <T as Config>::PoolPromoteApi::get_pool_rewards(liquidity_asset_id).is_some() &&
			activate_minted_liquidity
		{
			// The reserve from free_balance will not fail the asset were just minted into free_balance
			Pallet::<T>::set_liquidity_minting_checkpoint(
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

	fn burn_liquidity(
		sender: T::AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		liquidity_asset_amount: Self::Balance,
	) -> DispatchResult {
		let vault = Pallet::<T>::account_id();

		ensure!(
			!T::DisabledTokens::contains(&first_asset_id) &&
				!T::DisabledTokens::contains(&second_asset_id),
			Error::<T>::FunctionNotAvailableForThisToken
		);

		let liquidity_asset_id = Pallet::<T>::get_liquidity_asset(first_asset_id, second_asset_id)?;

		// First let's check how much we can actually burn

		let mut max_instant_unreserve_amount = Balance::zero();

		if <T as Config>::PoolPromoteApi::get_pool_rewards(liquidity_asset_id).is_some() {
			max_instant_unreserve_amount =
				T::ActivationReservesProvider::get_max_instant_unreserve_amount(
					liquidity_asset_id,
					&sender,
				);
		} else {
			max_instant_unreserve_amount = Balance::zero();
		}

		// Get token reserves and liquidity asset id
		let (first_asset_reserve, second_asset_reserve) =
			Pallet::<T>::get_reserves(first_asset_id, second_asset_id)?;

		// Ensure user has enought liquidity tokens to burn
		let liquidity_token_available_balance =
			<T as Config>::Currency::available_balance(liquidity_asset_id.into(), &sender).into();

		let liquidity_token_activated_balance =
			LiquidityMiningActiveUser::<T>::get((&sender, &liquidity_asset_id));

		ensure!(
			liquidity_token_available_balance
				.checked_add(max_instant_unreserve_amount)
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
		if !to_be_deactivated.is_zero() {
			Pallet::<T>::set_liquidity_burning_checkpoint(
				sender.clone(),
				liquidity_asset_id,
				to_be_deactivated,
			)?;

			if liquidity_token_activated_balance == to_be_deactivated {
				LiquidityMiningUser::<T>::remove((sender.clone(), liquidity_asset_id));
			}
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
	fn claim_rewards(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		mangata_amount: Self::Balance,
	) -> DispatchResult {
		let mangata_id: TokenId = T::NativeCurrencyId::get();

		let current_rewards =
			Pallet::<T>::calculate_rewards_amount(user.clone(), liquidity_asset_id)?;
		let already_claimed_rewards =
			LiquidityMiningUserClaimed::<T>::get((&user, liquidity_asset_id));
		let burned_not_claimed_rewards =
			LiquidityMiningUserToBeClaimed::<T>::get((&user, liquidity_asset_id));

		ensure!(mangata_amount <= current_rewards, Error::<T>::NotEnoughtRewardsEarned);

		// user is taking out rewards from LP which was already removed from pool
		if mangata_amount <= burned_not_claimed_rewards {
			let burned_not_claimed_rewards_new = burned_not_claimed_rewards - mangata_amount;
			LiquidityMiningUserToBeClaimed::<T>::insert(
				(&user, liquidity_asset_id),
				burned_not_claimed_rewards_new,
			);
		}
		// user is taking out more rewards then rewards from LP which was already removed from pool, additional work needs to be removed from pool and user
		else {
			LiquidityMiningUserToBeClaimed::<T>::insert((&user, liquidity_asset_id), 0 as u128);
			// rewards to claim on top of rewards from LP which was already removed from pool
			let rewards_to_claim = mangata_amount
				.checked_sub(burned_not_claimed_rewards)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
			let new_rewards_claimed = already_claimed_rewards
				.checked_add(rewards_to_claim)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;

			LiquidityMiningUserClaimed::<T>::insert(
				(&user, liquidity_asset_id),
				new_rewards_claimed,
			);
		}

		<T as Config>::Currency::transfer(
			mangata_id.into(),
			&<T as Config>::LiquidityMiningIssuanceVault::get(),
			&user,
			mangata_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		Pallet::<T>::deposit_event(Event::RewardsClaimed(user, liquidity_asset_id, mangata_amount));

		Ok(())
	}

	fn promote_pool(liquidity_token_id: TokenId) -> DispatchResult {
		ensure!(
			<T as Config>::PoolPromoteApi::get_pool_rewards(liquidity_token_id).is_none(),
			Error::<T>::PoolAlreadyPromoted,
		);

		<T as Config>::PoolPromoteApi::promote_pool(liquidity_token_id);

		Pallet::<T>::deposit_event(Event::PoolPromoted(liquidity_token_id));

		Ok(())
	}

	fn activate_liquidity(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		amount: Self::Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		ensure!(Self::is_promoted_pool(liquidity_asset_id), Error::<T>::NotAPromotedPool);
		ensure!(
			<T as Config>::ActivationReservesProvider::can_activate(
				liquidity_asset_id.into(),
				&user,
				amount,
				use_balance_from.clone()
			),
			Error::<T>::NotEnoughAssets
		);

		Pallet::<T>::set_liquidity_minting_checkpoint(
			user.clone(),
			liquidity_asset_id,
			amount,
			use_balance_from,
		)?;

		Pallet::<T>::deposit_event(Event::LiquidityActivated(user, liquidity_asset_id, amount));

		Ok(())
	}

	fn deactivate_liquidity(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult {
		ensure!(Self::is_promoted_pool(liquidity_asset_id), Error::<T>::NotAPromotedPool);
		ensure!(
			LiquidityMiningActiveUser::<T>::get((user.clone(), liquidity_asset_id)) >= amount,
			Error::<T>::NotEnoughAssets
		);

		Pallet::<T>::set_liquidity_burning_checkpoint(user.clone(), liquidity_asset_id, amount)?;

		Pallet::<T>::deposit_event(Event::LiquidityDeactivated(user, liquidity_asset_id, amount));

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

		let mga_token_reserve = match Pallet::<T>::get_reserves(mga_token_id, other_token_id) {
			Ok(reserves) => reserves.0,
			Err(_) => return Default::default(),
		};

		let liquidity_token_reserve: Balance =
			<T as Config>::Currency::total_issuance(liquidity_token_id.into()).into();

		if liquidity_token_reserve.is_zero() {
			return Default::default()
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
			return Default::default()
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
			return None
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

impl<T: Config> mp_bootstrap::RewardsApi for Pallet<T> {
	type AccountId = T::AccountId;

	fn can_activate(liquidity_asset_id: TokenId) -> bool {
		Self::is_promoted_pool(liquidity_asset_id)
	}

	fn activate_liquidity_tokens(
		user: &Self::AccountId,
		liquidity_asset_id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		<Self as XykFunctionsTrait<T::AccountId>>::activate_liquidity(
			user.clone(),
			liquidity_asset_id,
			amount,
			Some(ActivateKind::AvailableBalance),
		)
	}

	fn promote_pool(liquidity_token_id: TokenId) -> bool {
		let promote_pool_result = <T as Config>::PoolPromoteApi::promote_pool(liquidity_token_id);

		Pallet::<T>::deposit_event(Event::PoolPromoted(liquidity_token_id));

		promote_pool_result
	}
}
