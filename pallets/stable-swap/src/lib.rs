//! # Stable Pools Pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		tokens::{Balance, CurrencyId},
		Currency, ExistenceRequirement, MultiTokenCurrency, WithdrawReasons,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;

use sp_arithmetic::traits::Unsigned;
use sp_runtime::traits::{
	checked_pow, AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Ensure, One,
	TrailingZeroInput, Zero,
};
use sp_std::{convert::TryInto, vec, vec::Vec};

use orml_tokens::MultiTokenCurrencyExtended;

mod weights;
use crate::weights::WeightInfo;

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(
	TypeInfo,
	Encode,
	Decode,
	CloneNoBound,
	EqNoBound,
	PartialEqNoBound,
	RuntimeDebugNoBound,
	MaxEncodedLen,
	Default,
)]
#[codec(mel_bound(skip_type_params(MaxAssets)))]
#[scale_info(skip_type_params(MaxAssets))]
pub struct PoolInfo<Id: CurrencyId, B: Balance, MaxAssets: Get<u32>> {
	/// Liquidity pool asset
	pub lp_token: Id,
	/// associated asset ids
	pub assets: BoundedVec<Id, MaxAssets>,
	/// amplification coefficient for StableSwap equation
	pub amp_coeff: u128,
	pub rate_multipliers: BoundedVec<B, MaxAssets>,
}

pub type PoolIdOf<T> = <T as Config>::CurrencyId;
pub type PoolInfoOf<T> =
	PoolInfo<<T as Config>::CurrencyId, <T as Config>::Balance, <T as Config>::MaxAssetsInPool>;
pub type AssetIdsOf<T> = BoundedVec<<T as Config>::CurrencyId, <T as Config>::MaxAssetsInPool>;
pub type BalancesOf<T> = BoundedVec<<T as Config>::Balance, <T as Config>::MaxAssetsInPool>;

#[frame_support::pallet]
pub mod pallet {
	use core::fmt::Debug;

	use frame_support::transactional;

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency type that this works on.
		type Currency: MultiTokenCurrencyExtended<
			Self::AccountId,
			Balance = Self::Balance,
			CurrencyId = Self::CurrencyId,
		>;

		/// The `Currency::Balance` type of the currency.
		type Balance: Balance;

		/// A type used for multiplication of `Balance`.
		type HigherPrecisionBalance: Copy
			+ Debug
			+ One
			+ Ensure
			+ Unsigned
			+ From<u32>
			+ From<u128>
			+ From<Self::Balance>
			+ TryInto<Self::Balance>;

		/// Identifier for the assets.
		type CurrencyId: CurrencyId;

		/// Assets that are not allowed to be present in pools.
		// type DisabledTokens: Contains<Self::CurrencyId>;

		/// Treasury pallet id used for fee deposits
		type TreasuryPalletId: Get<PalletId>;

		/// Percentage for swap fee that goes back into the pool.
		#[pallet::constant]
		type PoolFeePercentage: Get<u128>;

		/// Percentage for swap fee that goes into the treasury.
		#[pallet::constant]
		type TreasuryFeePercentage: Get<u128>;

		/// Percentage for swap fee that is burned.
		#[pallet::constant]
		type BuyAndBurnFeePercentage: Get<u128>;

		#[pallet::constant]
		type MaxApmCoeff: Get<u128>;

		#[pallet::constant]
		type MaxAssetsInPool: Get<u32>;

		/// Interface for modifing asset registry when creating new pools
		// type AssetMetadataMutation: AssetMetadataMutationTrait<Self::CurrencyId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Amplification coefficient lower then 1 or too large
		AmpCoeffOutOfRange,
		/// Too many assets for pool creation
		TooManyAssets,
		/// Pool already exists
		PoolAlreadyExists,
		/// Asset does not exist
		AssetDoesNotExist,
		/// No such pool exists
		NoSuchPool,
		/// Provided arguments do not match in length
		ArgumentsLengthMismatch,
		/// Pool is broken, remove liquidity
		PoolInvariantBroken,
		/// Initial liquidity provision needs all assets
		InitialLiquidityZeroAmount,
		/// Asset does not exist in pool.
		NoSuchAssetInPool,
		/// Unexpected failure
		UnexpectedFailure,
		/// Insufficient output amount does not meet min requirements
		InsufficientOutputAmount,
		/// Insufficient input amount
		InsufficientInputAmount,
		/// Excesive output amount does not meet max requirements
		ExcesiveOutputAmount,

		/// Not enought assets
		NotEnoughAssets,
		/// No such liquidity asset exists
		NoSuchLiquidityAsset,
		/// Not enought reserve
		NotEnoughReserve,
		/// Zero amount is not supported
		ZeroAmount,
		/// Asset ids cannot be the same
		SameAsset,
		/// Asset already exists
		AssetAlreadyExists,
		/// Division by zero
		DivisionByZero,
		/// Unexpected failure
		NotPairedWithNativeAsset,
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

	// Pallet's events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A successful call of the `CretaPool` extrinsic will create this event.
		PoolCreated {
			/// The account that created the pool.
			creator: T::AccountId,
			/// The pool id and the account ID of the pool.
			pool_id: PoolIdOf<T>,
			/// The id of the liquidity tokens that will be minted when assets are added to this
			/// pool.
			lp_token: T::CurrencyId,
			/// The asset ids associated with the pool. Note that the order of the assets may not be
			/// the same as the order specified in the create pool extrinsic.
			assets: AssetIdsOf<T>,
		},

		/// A successful call of the `AddLiquidity` extrinsic will create this event.
		LiquidityMinted {
			/// The account that the liquidity was taken from.
			who: T::AccountId,
			/// The id of the pool that the liquidity was added to.
			pool_id: PoolIdOf<T>,
			/// The amount of the first asset that was added to the pool.
			amounts_provided: BalancesOf<T>,
			/// The id of the LP token that was minted.
			lp_token: T::CurrencyId,
			/// The amount of lp tokens that were minted of that id.
			lp_token_minted: T::Balance,
			/// The new total supply of the associated LP token.
			total_supply: T::Balance,
			/// The fees taken into treasury.
			fees: BalancesOf<T>,
		},

		/// Assets have been swapped, a successfull call to `Swap` will create this event.
		AssetsSwapped {
			/// Which account was the instigator of the swap.
			who: T::AccountId,
			/// The id of the pool where assets were swapped.
			pool_id: PoolIdOf<T>,
			/// The id of the asset that was swapped.
			asset_in: T::CurrencyId,
			/// The amount of the asset that was swapped.
			amount_in: T::Balance,
			/// The id of the asset that was received.
			asset_out: T::CurrencyId,
			/// The amount of the asset that was received.
			amount_out: T::Balance,
		},
		/// A successful call of the `RemoveLiquidityOneAsset` extrinsic will create this event.
		LiquidityBurnedOne {
			/// Which account was the instigator of the swap.
			who: T::AccountId,
			/// The id of the pool where assets were swapped.
			pool_id: PoolIdOf<T>,
			/// The id of the asset that was received.
			asset_id: T::CurrencyId,
			/// The amount of the asset that was received.
			amount: T::Balance,
			/// The amount of the associated LP token that was burned.
			burned_amount: T::Balance,
			/// The new total supply of the associated LP token.
			total_supply: T::Balance,
		},
		/// A successful call of the `RemoveLiquidityImbalanced` & `RemoveLiquidity` extrinsic will create this event.
		LiquidityBurned {
			/// Which account was the instigator of the swap.
			who: T::AccountId,
			/// The id of the pool where assets were swapped.
			pool_id: PoolIdOf<T>,
			/// The amount of the asset that was received.
			amounts: BalancesOf<T>,
			/// The amount of the associated LP token that was burned.
			burned_amount: T::Balance,
			/// The new total supply of the associated LP token.
			total_supply: T::Balance,
			/// The fees taken into treasury.
			fees: BalancesOf<T>,
		},
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_pool)]
	pub type Pools<T: Config> = StorageMap<_, Identity, PoolIdOf<T>, PoolInfoOf<T>, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			assert!(
				T::MaxAssetsInPool::get() > 1,
				"the `MaxAssetsInPool` should be greater than 1",
			);
		}
	}

	/// Pallet's callable functions.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a liquidity pool and an associated new `lp_token` asset
		/// (the id of which is returned in the `Event::PoolCreated` event).
		/// Tokens can have arbitrary decimals (<=18).
		///
		/// * `assets` - An array of asset ids in pool
		/// * `rates` - An array of: [10 ** (36 - _coins[n].decimals()), ... for n in range(N_COINS)]
		/// * `amp_coeff` - Amplification co-efficient - a lower value here means less tolerance for imbalance within the pool's assets.
		/// 				Suggested values include:
		///    					* Uncollateralized algorithmic stablecoins: 5-10
		///    					* Non-redeemable, collateralized assets: 100
		///   					* Redeemable assets: 200-400
		///
		/// Initial liquidity amounts needs to be provided with [`Pallet::add_liquidity`].
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			assets: Vec<T::CurrencyId>,
			rates: Vec<T::Balance>,
			amp_coeff: u128,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				1 <= amp_coeff && amp_coeff <= T::MaxApmCoeff::get(),
				Error::<T>::AmpCoeffOutOfRange
			);

			let assets_in_len = assets.len();
			ensure!(
				assets_in_len <= T::MaxAssetsInPool::get().try_into().unwrap_or_default(),
				Error::<T>::TooManyAssets
			);
			ensure!(
				rates.len() <= T::MaxAssetsInPool::get().try_into().unwrap_or_default(),
				Error::<T>::TooManyAssets
			);
			ensure!(rates.len() == assets_in_len, Error::<T>::ArgumentsLengthMismatch);

			let mut assets = assets.clone();
			assets.sort();
			assets.dedup();
			ensure!(assets_in_len == assets.len(), Error::<T>::SameAsset);

			for id in assets.clone() {
				ensure!(T::Currency::exists(id), Error::<T>::AssetDoesNotExist)
			}

			let pool_account = Self::get_pool_account(&assets);
			ensure!(
				!frame_system::Pallet::<T>::account_exists(&pool_account),
				Error::<T>::PoolAlreadyExists
			);
			frame_system::Pallet::<T>::inc_providers(&pool_account);

			let lp_token: T::CurrencyId = T::Currency::create(&sender, T::Balance::zero())
				.map_err(|_| Error::<T>::LiquidityTokenCreationFailed)?
				.into();

			let assets = AssetIdsOf::<T>::truncate_from(assets);
			let rates = BalancesOf::<T>::truncate_from(rates);
			let pool_info = PoolInfo {
				lp_token: lp_token.clone(),
				assets: assets.clone(),
				amp_coeff,
				rate_multipliers: rates,
			};
			Pools::<T>::insert(lp_token.clone(), pool_info);

			Self::deposit_event(Event::PoolCreated {
				creator: sender,
				pool_id: lp_token,
				lp_token,
				assets,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn swap(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			asset_in: T::CurrencyId,
			asset_out: T::CurrencyId,
			dx: T::Balance,
			min_dy: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// ensure same asset
			// ensure can withdraw dx
			// ensure pool exists
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool.assets);

			// ensure assets in pool
			let i = pool
				.assets
				.iter()
				.position(|x| *x == asset_in)
				.ok_or(Error::<T>::NoSuchAssetInPool)?;
			let j = pool
				.assets
				.iter()
				.position(|x| *x == asset_out)
				.ok_or(Error::<T>::NoSuchAssetInPool)?;

			// old balances
			let (_, xp) = Self::get_balances_xp_pool(&pool_account, &pool)?;

			// get tokens in
			T::Currency::transfer(
				asset_in,
				&sender,
				&pool_account,
				dx,
				ExistenceRequirement::AllowDeath,
			)?;

			// invariant
			let x = Self::checked_mul_div_u128(
				&T::HigherPrecisionBalance::from(dx),
				&T::HigherPrecisionBalance::from(pool.rate_multipliers[i]),
				Self::PRECISION,
			)?
			.checked_add(&xp[i])
			.ok_or(Error::<T>::MathOverflow)?;
			let d = Self::get_invariant(&xp, pool.amp_coeff)?;
			let y = Self::get_y(i, j, &x, &xp, pool.amp_coeff, &d)?;
			// -1 in case of rounding error
			let dy = xp[j]
				.checked_sub(&y)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_sub(&One::one())
				.ok_or(Error::<T>::MathOverflow)?;

			// fees
			let (fee, trsy_fee, m) = Self::fees();
			let dyn_fee = Self::dynamic_fee(
				Self::checked_add_div_2(&xp[i], &x)?,
				Self::checked_add_div_2(&xp[j], &y)?,
				fee,
				m,
			)?;
			let dy_fee = Self::checked_mul_div_u128(&dy, &dyn_fee, Self::FEE_DENOMINATOR)?;
			let to_treasury = Self::checked_mul_div_to_balance(
				&Self::checked_mul_div_u128(&dy_fee, &trsy_fee, Self::FEE_DENOMINATOR)?,
				pool.rate_multipliers[j],
			)?
			.try_into()
			.map_err(|_| Error::<T>::MathOverflow)?;

			T::Currency::transfer(
				pool.assets[i],
				&pool_account,
				&Self::treasury_account_id(),
				to_treasury,
				ExistenceRequirement::AllowDeath,
			)?;

			// real units
			let dy = Self::checked_mul_div_to_balance(&dy, pool.rate_multipliers[j])?;
			ensure!(dy >= min_dy, Error::<T>::InsufficientOutputAmount);

			T::Currency::transfer(
				asset_out,
				&pool_account,
				&sender,
				dy,
				ExistenceRequirement::AllowDeath,
			)?;

			Self::deposit_event(Event::AssetsSwapped {
				who: sender,
				pool_id,
				asset_in,
				amount_in: dx,
				asset_out,
				amount_out: dy,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			amounts: Vec<T::Balance>,
			min_amount_lp_tokens: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			ensure!(amounts.len() == pool.assets.len(), Error::<T>::ArgumentsLengthMismatch);
			let pool_account = Self::get_pool_account(&pool.assets);
			let asset_amounts = pool.assets.iter().zip(amounts.iter());
			let total_supply = T::Currency::total_issuance(pool.lp_token);
			let n = T::HigherPrecisionBalance::from(pool.assets.len() as u128);

			// check user's asset balances
			for (id, amount) in asset_amounts.clone() {
				ensure!(
					!(total_supply == Zero::zero() && *amount == Zero::zero()),
					Error::<T>::InitialLiquidityZeroAmount
				);
				T::Currency::ensure_can_withdraw(
					*id,
					&sender,
					*amount,
					WithdrawReasons::all(),
					Default::default(),
				)?;
			}

			// get initial invariant
			let (balances_0, d_0) = Self::get_invariant_pool(&pool_account, &pool)?;

			// transfer from user account
			for (id, amount) in asset_amounts {
				T::Currency::transfer(
					*id,
					&sender,
					&pool_account,
					*amount,
					ExistenceRequirement::AllowDeath,
				)?;
			}

			// check new invariant
			let (balances_1, d_1) = Self::get_invariant_pool(&pool_account, &pool)?;
			ensure!(d_1 > d_0, Error::<T>::PoolInvariantBroken);

			let mut fees_b: Vec<T::Balance> = vec![];
			// LPs also incur fees. A swap between A & B would pay roughly the same amount of fees as depositing A into the pool and then withdrawing B.
			let mint_amount = if total_supply > Zero::zero() {
				let (d_1, fees) = Self::handle_imbalanced_liquidity_fees(
					&pool,
					&pool_account,
					&n,
					&d_0,
					&d_1,
					&balances_0,
					&balances_1,
				)?;

				for f in fees {
					fees_b.push(f.try_into().map_err(|_| Error::<T>::MathOverflow)?)
				}

				d_1.checked_sub(&d_0)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_mul(&T::HigherPrecisionBalance::from(total_supply))
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(&d_0)
					.ok_or(Error::<T>::MathOverflow)?
					.try_into()
					.map_err(|_| Error::<T>::MathOverflow)?
			} else {
				// no fees on intial liquidity deposit
				d_1.try_into().map_err(|_| Error::<T>::MathOverflow)?
			};

			ensure!(mint_amount >= min_amount_lp_tokens, Error::<T>::InsufficientOutputAmount);

			T::Currency::mint(pool.lp_token, &sender, mint_amount)?;

			let total_supply = T::Currency::total_issuance(pool.lp_token);
			Self::deposit_event(Event::LiquidityMinted {
				who: sender,
				pool_id,
				amounts_provided: BoundedVec::truncate_from(amounts),
				lp_token: pool.lp_token,
				lp_token_minted: mint_amount,
				total_supply,
				fees: BoundedVec::truncate_from(fees_b),
			});

			Ok(())
		}

		/// Withdraw a single asset from the pool
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::remove_liquidity_one_asset())]
		#[transactional]
		pub fn remove_liquidity_one_asset(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			asset_id: T::CurrencyId,
			burn_amount: T::Balance,
			min_amount_out: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(burn_amount > Zero::zero(), Error::<T>::InsufficientInputAmount);

			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool.assets);

			let (dy, trsy_fee) =
				Self::calc_withdraw_one(&pool_account, &pool, asset_id, burn_amount)?;

			ensure!(dy > min_amount_out, Error::<T>::InsufficientOutputAmount);

			T::Currency::transfer(
				asset_id,
				&sender,
				&Self::treasury_account_id(),
				trsy_fee,
				ExistenceRequirement::AllowDeath,
			)?;

			T::Currency::burn_and_settle(pool.lp_token, &sender, burn_amount)?;

			T::Currency::transfer(
				asset_id,
				&pool_account,
				&sender,
				dy,
				ExistenceRequirement::AllowDeath,
			)?;

			let total_supply = T::Currency::total_issuance(pool.lp_token);
			Self::deposit_event(Event::LiquidityBurnedOne {
				who: sender,
				pool_id,
				asset_id,
				amount: dy,
				burned_amount: burn_amount,
				total_supply,
			});

			Ok(())
		}

		/// Withdraw assets from the pool in an imbalanced amounts
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::remove_liquidity_imbalanced())]
		#[transactional]
		pub fn remove_liquidity_imbalanced(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			amounts: Vec<T::Balance>,
			max_burn_amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			ensure!(amounts.len() == pool.assets.len(), Error::<T>::ArgumentsLengthMismatch);
			let pool_account = Self::get_pool_account(&pool.assets);
			let asset_amounts = pool.assets.iter().zip(amounts.iter());
			let total_supply = T::Currency::total_issuance(pool.lp_token);
			let n = T::HigherPrecisionBalance::from(pool.assets.len() as u128);

			let (balances_0, d_0) = Self::get_invariant_pool(&pool_account, &pool)?;

			// transfer to user account
			for (id, amount) in asset_amounts {
				T::Currency::transfer(
					*id,
					&pool_account,
					&sender,
					*amount,
					ExistenceRequirement::AllowDeath,
				)?;
			}

			let (balances_1, d_1) = Self::get_invariant_pool(&pool_account, &pool)?;
			let (d_1, fees) = Self::handle_imbalanced_liquidity_fees(
				&pool,
				&pool_account,
				&n,
				&d_0,
				&d_1,
				&balances_0,
				&balances_1,
			)?;

			let burn_amount = d_0
				.checked_sub(&d_1)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_mul(&T::HigherPrecisionBalance::from(total_supply))
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(&d_0)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_add(&One::one())
				.ok_or(Error::<T>::MathOverflow)?
				.try_into()
				.map_err(|_| Error::<T>::MathOverflow)?;

			ensure!(burn_amount > One::one(), Error::<T>::InsufficientInputAmount);
			ensure!(burn_amount <= max_burn_amount, Error::<T>::ExcesiveOutputAmount);

			T::Currency::burn_and_settle(pool.lp_token, &sender, burn_amount)?;

			let mut fees_b: Vec<T::Balance> = vec![];
			for f in fees {
				fees_b.push(f.try_into().map_err(|_| Error::<T>::MathOverflow)?)
			}

			let total_supply = T::Currency::total_issuance(pool.lp_token);
			Self::deposit_event(Event::LiquidityBurned {
				who: sender,
				pool_id,
				amounts: BoundedVec::truncate_from(amounts),
				burned_amount: burn_amount,
				total_supply,
				fees: BoundedVec::truncate_from(fees_b),
			});

			Ok(())
		}

		/// Withdraw assets from the pool in an imbalanced amounts
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_liquidity())]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			burn_amount: T::Balance,
			min_amounts: Vec<T::Balance>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			ensure!(min_amounts.len() == pool.assets.len(), Error::<T>::ArgumentsLengthMismatch);
			let pool_account = Self::get_pool_account(&pool.assets);
			// let asset_amounts = pool.assets.iter().zip(amounts.iter());
			let total_supply = T::Currency::total_issuance(pool.lp_token);
			// let n = T::HigherPrecisionBalance::from(pool.assets.len() as u128);

			let (balances, _) = Self::get_balances_xp_pool(&pool_account, &pool)?;

			let mut amounts = vec![];
			for i in 0..pool.assets.len() {
				let value = balances[i]
					.checked_mul(&T::HigherPrecisionBalance::from(burn_amount))
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(&T::HigherPrecisionBalance::from(total_supply))
					.ok_or(Error::<T>::MathOverflow)?
					.try_into()
					.map_err(|_| Error::<T>::MathOverflow)?;
				amounts.push(value);
				T::Currency::transfer(
					pool.assets[i],
					&pool_account,
					&sender,
					value,
					ExistenceRequirement::AllowDeath,
				)?;
			}

			T::Currency::burn_and_settle(pool.lp_token, &sender, burn_amount)?;

			let total_supply = T::Currency::total_issuance(pool.lp_token);
			Self::deposit_event(Event::LiquidityBurned {
				who: sender,
				pool_id,
				amounts: BoundedVec::truncate_from(amounts),
				burned_amount: burn_amount,
				total_supply,
				fees: BoundedVec::new(),
			});

			Ok(())
		}

		// #[pallet::call_index(1)]
		// #[pallet::weight((<<T as Config>::WeightInfo>::multiswap_sell_asset(swap_token_list.len() as u32), DispatchClass::Operational, Pays::No))]
		// pub fn multiswap_sell_asset(
		// 	origin: OriginFor<T>,
		// 	swap_token_list: Vec<T::CurrencyId>,
		// 	sold_asset_amount: T::Balance,
		// 	min_amount_out: T::Balance,
		// ) -> DispatchResultWithPostInfo {
		// 	let sender = ensure_signed(origin)?;

		// 	Ok(Pays::No.into())
		// }

		// #[pallet::call_index(2)]
		// #[pallet::weight((<<T as Config>::WeightInfo>::multiswap_buy_asset(swap_token_list.len() as u32), DispatchClass::Operational, Pays::No))]
		// pub fn multiswap_buy_asset(
		// 	origin: OriginFor<T>,
		// 	swap_token_list: Vec<T::CurrencyId>,
		// 	bought_asset_amount: T::Balance,
		// 	max_amount_in: T::Balance,
		// ) -> DispatchResultWithPostInfo {
		// 	let sender = ensure_signed(origin)?;

		// 	Ok(Pays::No.into())
		// }

		// #[pallet::call_index(3)]
		// #[pallet::weight(<<T as Config>::WeightInfo>::mint_liquidity())]
		// pub fn mint_liquidity(
		// 	origin: OriginFor<T>,
		// 	first_asset_id: T::CurrencyId,
		// 	second_asset_id: T::CurrencyId,
		// 	first_asset_amount: T::Balance,
		// 	expected_second_asset_amount: T::Balance,
		// ) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;

		// 	Ok(())
		// }

		// #[pallet::call_index(4)]
		// #[pallet::weight(<<T as Config>::WeightInfo>::burn_liquidity())]
		// pub fn burn_liquidity(
		// 	origin: OriginFor<T>,
		// 	first_asset_id: T::CurrencyId,
		// 	second_asset_id: T::CurrencyId,
		// 	liquidity_asset_amount: T::Balance,
		// ) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;
		// 	Ok(())
		// }
	}

	impl<T: Config> Pallet<T> {
		const PRECISION: u128 = 10_u128.pow(18);
		const FEE_DENOMINATOR: u128 = 10_u128.pow(10);
		const A_PRECISION: u128 = 100;

		/// The account ID of the pool.
		///
		/// This actually does computation. If you need to keep using it, then make sure you cache
		/// the value and only call this once.
		pub fn get_pool_account(assets: &Vec<T::CurrencyId>) -> T::AccountId {
			let mut sorted = assets.clone();
			sorted.sort();
			let encoded_pool_id = sp_io::hashing::blake2_256(&Encode::encode(&sorted));

			Decode::decode(&mut TrailingZeroInput::new(encoded_pool_id.as_ref()))
				.expect("infinite length input; no invalid inputs for type; qed")
		}

		/// The current virtual price of the pool LP token, useful for calculating profits.
		/// Returns LP token virtual price normalized to 1e18.
		pub fn get_virtual_price(pool_id: &PoolIdOf<T>) -> Result<T::Balance, Error<T>> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool.assets);

			let total_supply: <T as Config>::Balance = T::Currency::total_issuance(pool.lp_token);
			let (_, d) = Self::get_invariant_pool(&pool_account, pool)?;

			d.checked_mul(&T::HigherPrecisionBalance::from(Self::PRECISION))
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(&T::HigherPrecisionBalance::from(total_supply))
				.ok_or(Error::<T>::MathOverflow)?
				.try_into()
				.map_err(|_| Error::<T>::MathOverflow)
		}

		// 0.3% total fee, 50% of fee to treasury, dyn fee 2* mul
		fn fees(
		) -> (T::HigherPrecisionBalance, T::HigherPrecisionBalance, T::HigherPrecisionBalance) {
			(
				T::HigherPrecisionBalance::from(30_000_000_u128),
				T::HigherPrecisionBalance::from(5_000_000_000_u128),
				T::HigherPrecisionBalance::from(20_000_000_000_u128),
			)
		}

		fn base_fee(
			n: &T::HigherPrecisionBalance,
		) -> Result<<T as Config>::HigherPrecisionBalance, Error<T>> {
			let (fee, _, _) = Self::fees();
			fee.checked_mul(n)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(
					&n.checked_sub(&One::one())
						.ok_or(Error::<T>::MathOverflow)?
						.checked_mul(&T::HigherPrecisionBalance::from(4_u32))
						.ok_or(Error::<T>::MathOverflow)?,
				)
				.ok_or(Error::<T>::MathOverflow)
		}

		fn treasury_account_id() -> T::AccountId {
			T::TreasuryPalletId::get().into_account_truncating()
		}

		fn account_id() -> T::AccountId {
			PalletId(*b"py/stbsw").into_account_truncating()
		}

		// common
		fn handle_imbalanced_liquidity_fees(
			pool: &PoolInfoOf<T>,
			pool_account: &T::AccountId,
			n: &T::HigherPrecisionBalance,
			d_0: &T::HigherPrecisionBalance,
			d_1: &T::HigherPrecisionBalance,
			balances_0: &Vec<T::HigherPrecisionBalance>,
			balances_1: &Vec<T::HigherPrecisionBalance>,
		) -> Result<(T::HigherPrecisionBalance, Vec<T::HigherPrecisionBalance>), DispatchError> {
			let mut fees = vec![];
			let (fee, trsy_fee, m) = Self::fees();
			let base_fee = Self::base_fee(n)?;
			let ys = d_0
				.checked_add(d_1)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(n)
				.ok_or(Error::<T>::MathOverflow)?;

			let mut balances_mint = balances_1.clone();

			for i in 0..pool.assets.len() {
				let ideal_balance = d_1
					.checked_mul(&balances_0[i])
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(d_0)
					.ok_or(Error::<T>::MathOverflow)?;

				let diff = if ideal_balance > balances_1[i] {
					ideal_balance - balances_1[i]
				} else {
					balances_1[i] - ideal_balance
				};

				let xs = Self::checked_mul_div_u128(
					&balances_0[i].checked_add(&balances_1[i]).ok_or(Error::<T>::MathOverflow)?,
					&T::HigherPrecisionBalance::from(pool.rate_multipliers[i]),
					Self::PRECISION,
				)?;

				let dyn_fee = Self::dynamic_fee(xs, ys, base_fee, m)?;
				fees.push(Self::checked_mul_div_u128(&diff, &dyn_fee, Self::FEE_DENOMINATOR)?);
				let to_treasury =
					Self::checked_mul_div_u128(&fees[i], &trsy_fee, Self::FEE_DENOMINATOR)?
						.try_into()
						.map_err(|_| Error::<T>::MathOverflow)?;

				balances_mint[i] =
					balances_mint[i].checked_sub(&fees[i]).ok_or(Error::<T>::MathOverflow)?;

				T::Currency::transfer(
					pool.assets[i],
					&pool_account,
					&Self::treasury_account_id(),
					to_treasury,
					ExistenceRequirement::AllowDeath,
				)?;
			}

			let xp = Self::xp(&pool.rate_multipliers, &balances_mint)?;
			let d_1 = Self::get_invariant(&xp, pool.amp_coeff)?;

			Ok((d_1, fees))
		}

		// stable swap maths

		// https://www.desmos.com/calculator/zhrwbvcipo
		fn dynamic_fee(
			xpi: T::HigherPrecisionBalance,
			xpj: T::HigherPrecisionBalance,
			fee: T::HigherPrecisionBalance,
			m: T::HigherPrecisionBalance,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			let den = T::HigherPrecisionBalance::from(Self::FEE_DENOMINATOR);
			if m < den {
				return Ok(fee);
			}

			let xps2 = checked_pow(xpi.checked_add(&xpj).ok_or(Error::<T>::MathOverflow)?, 2)
				.ok_or(Error::<T>::MathOverflow)?;

			let res = fee
				.checked_mul(&m)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(
					&m.checked_sub(&den)
						.ok_or(Error::<T>::MathOverflow)?
						.checked_mul(&T::HigherPrecisionBalance::from(4_u32))
						.ok_or(Error::<T>::MathOverflow)?
						.checked_mul(&xpi)
						.ok_or(Error::<T>::MathOverflow)?
						.checked_mul(&xpj)
						.ok_or(Error::<T>::MathOverflow)?
						.checked_div(&xps2)
						.ok_or(Error::<T>::MathOverflow)?
						.checked_add(&den)
						.ok_or(Error::<T>::MathOverflow)?,
				)
				.ok_or(Error::<T>::MathOverflow)?;

			Ok(res)
		}

		fn get_balances_xp_pool(
			pool_account: &T::AccountId,
			pool: &PoolInfoOf<T>,
		) -> Result<(Vec<T::HigherPrecisionBalance>, Vec<T::HigherPrecisionBalance>), Error<T>> {
			let reserves: Vec<T::HigherPrecisionBalance> = pool
				.assets
				.iter()
				.map(|&id| T::Currency::available_balance(id, pool_account))
				.map(T::HigherPrecisionBalance::from)
				.collect();

			let xp = Self::xp(&pool.rate_multipliers, &reserves)?;

			Ok((reserves, xp))
		}

		fn get_invariant_pool(
			pool_account: &T::AccountId,
			pool: &PoolInfoOf<T>,
		) -> Result<(Vec<T::HigherPrecisionBalance>, T::HigherPrecisionBalance), Error<T>> {
			let amp = pool.amp_coeff;
			let (reserves, xp) = Self::get_balances_xp_pool(pool_account, pool)?;
			let d = Self::get_invariant(&xp, amp)?;
			Ok((reserves, d))
		}

		fn xp(
			rates: &BalancesOf<T>,
			balances: &Vec<T::HigherPrecisionBalance>,
		) -> Result<Vec<T::HigherPrecisionBalance>, Error<T>> {
			let mut xp = vec![];
			for (&balance, &rate) in balances.iter().zip(rates.iter()) {
				xp.push(Self::checked_mul_div_u128(
					&T::HigherPrecisionBalance::from(balance),
					&T::HigherPrecisionBalance::from(rate),
					Self::PRECISION,
				)?);
			}
			Ok(xp)
		}

		/// Computes the Stable Swap invariant (D).
		///
		/// The invariant is defined as follows:
		///
		/// ```text
		/// A * sum(x_i) * n**n + D = A * D * n**n + D**(n+1) / (n**n * prod(x_i))
		/// ```
		/// Converging solution:
		/// ```text
		/// D[j+1] = (A * n**n * sum(x_i) - D[j]**(n+1) / (n**n prod(x_i))) / (A * n**n - 1)
		/// ```
		fn get_invariant(
			xp: &Vec<T::HigherPrecisionBalance>,
			amp: u128,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			let n = T::HigherPrecisionBalance::from(xp.len() as u128);
			let amp = T::HigherPrecisionBalance::from(amp);
			let mut sum = T::HigherPrecisionBalance::zero();
			for balance in xp.iter() {
				sum = sum.checked_add(balance).ok_or(Error::<T>::MathOverflow)?;
			}

			if sum == Zero::zero() {
				return Ok(Zero::zero());
			}

			let mut d = sum;
			// len will allways be less then u32::MAX
			let ann = amp.checked_mul(&n).ok_or(Error::<T>::MathOverflow)?;

			for _ in 0..256 {
				let mut d_p = d;
				for b in xp.iter() {
					d_p = d_p
						.checked_mul(&d)
						.ok_or(Error::<T>::MathOverflow)?
						.checked_div(b)
						.ok_or(Error::<T>::MathOverflow)?;
				}
				let nn = checked_pow(n, xp.len()).ok_or(Error::<T>::MathOverflow)?;
				d_p = d_p.checked_div(&nn).ok_or(Error::<T>::MathOverflow)?;

				let d_prev = d;
				// (Ann * S / A_PRECISION + D_P * N_COINS) * D / ((Ann - A_PRECISION) * D / A_PRECISION + (N_COINS + 1) * D_P)
				d = Self::checked_mul_div_u128(&ann, &sum, Self::A_PRECISION)?
					.checked_add(&d_p.checked_mul(&n).ok_or(Error::<T>::MathOverflow)?)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_mul(&d)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(
						&Self::checked_mul_div_u128(
							&ann.checked_sub(&T::HigherPrecisionBalance::from(Self::A_PRECISION))
								.ok_or(Error::<T>::MathOverflow)?,
							&d,
							Self::A_PRECISION,
						)?
						.checked_add(
							&n.checked_add(&One::one())
								.ok_or(Error::<T>::MathOverflow)?
								.checked_mul(&d_p)
								.ok_or(Error::<T>::MathOverflow)?,
						)
						.ok_or(Error::<T>::MathOverflow)?,
					)
					.ok_or(Error::<T>::MathOverflow)?;

				if d.checked_sub(&d_prev).map_or(false, |diff| diff.le(&One::one())) {
					return Ok(d);
				}
				if d_prev.checked_sub(&d).map_or(false, |diff| diff.le(&One::one())) {
					return Ok(d);
				}
			}

			// converges in few iters, should not happen
			// if it does, pool is broken, users should remove liquidity
			Err(Error::<T>::PoolInvariantBroken)
		}

		/// Calculate x[j] if one makes x[i] = x
		///
		/// Done by solving quadratic equation iteratively.
		/// x_1**2 + x_1 * (sum' - (A*n**n - 1) * D / (A * n**n)) = D ** (n + 1) / (n ** (2 * n) * prod' * A)
		/// x_1**2 + b*x_1 = c
		/// x_1 = (x_1**2 + c) / (2*x_1 + b)
		///
		/// x in the input is converted to the same price/precision
		fn get_y(
			i: usize,
			j: usize,
			x: &T::HigherPrecisionBalance,
			xp: &Vec<T::HigherPrecisionBalance>,
			amp: u128,
			d: &T::HigherPrecisionBalance,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			// ensure 0 < i,j < max assets, i != j, ...should not happen due previous checks
			ensure!(
				i != j && j >= 0 as usize && j < T::MaxAssetsInPool::get() as usize,
				Error::<T>::UnexpectedFailure
			);
			ensure!(
				i >= 0 as usize && i < T::MaxAssetsInPool::get() as usize,
				Error::<T>::UnexpectedFailure
			);

			let n = T::HigherPrecisionBalance::from(xp.len() as u128);
			let amp = T::HigherPrecisionBalance::from(amp);
			let ann = amp.checked_mul(&n).ok_or(Error::<T>::MathOverflow)?;

			let mut sum = T::HigherPrecisionBalance::zero();
			let mut c = *d;

			for _i in 0..xp.len() {
				let mut _x = Zero::zero();
				if _i == i {
					_x = *x
				} else if _i != j {
					_x = xp[_i]
				} else {
					continue
				};

				sum = sum.checked_add(&_x).ok_or(Error::<T>::MathOverflow)?;
				c = c
					.checked_mul(&d)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(&_x.checked_mul(&n).ok_or(Error::<T>::MathOverflow)?)
					.ok_or(Error::<T>::MathOverflow)?;
			}

			Self::solve_y(&n, &ann, d, &c, &sum)
		}

		/// Calculate x[i] if one reduces D from being calculated for xp to D
		///
		/// x in the input is converted to the same price/precision
		fn get_y_d(
			i: usize,
			xp: &Vec<T::HigherPrecisionBalance>,
			amp: u128,
			d: &T::HigherPrecisionBalance,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			ensure!(
				i >= 0 as usize && i < T::MaxAssetsInPool::get() as usize,
				Error::<T>::UnexpectedFailure
			);

			let n = T::HigherPrecisionBalance::from(xp.len() as u128);
			let amp = T::HigherPrecisionBalance::from(amp);
			let ann = amp.checked_mul(&n).ok_or(Error::<T>::MathOverflow)?;

			let mut sum = T::HigherPrecisionBalance::zero();
			let mut c = *d;

			for _i in 0..xp.len() {
				let mut _x = Zero::zero();
				if _i != i {
					_x = xp[_i]
				} else {
					continue
				};

				sum = sum.checked_add(&_x).ok_or(Error::<T>::MathOverflow)?;
				c = c
					.checked_mul(d)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(&_x.checked_mul(&n).ok_or(Error::<T>::MathOverflow)?)
					.ok_or(Error::<T>::MathOverflow)?;
			}
			Self::solve_y(&n, &ann, d, &c, &sum)
		}

		/// Done by solving quadratic equation iteratively.
		/// x_1**2 + x_1 * (sum' - (A*n**n - 1) * D / (A * n**n)) = D ** (n + 1) / (n ** (2 * n) * prod' * A)
		/// x_1**2 + b*x_1 = c
		/// x_1 = (x_1**2 + c) / (2*x_1 + b)
		fn solve_y(
			n: &T::HigherPrecisionBalance,
			ann: &T::HigherPrecisionBalance,
			d: &T::HigherPrecisionBalance,
			c: &T::HigherPrecisionBalance,
			sum: &T::HigherPrecisionBalance,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			let c = c
				.checked_mul(d)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_mul(&T::HigherPrecisionBalance::from(Self::A_PRECISION))
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(&ann.checked_mul(n).ok_or(Error::<T>::MathOverflow)?)
				.ok_or(Error::<T>::MathOverflow)?;
			let b = sum
				.checked_add(
					&d.checked_mul(&T::HigherPrecisionBalance::from(Self::A_PRECISION))
						.ok_or(Error::<T>::MathOverflow)?
						.checked_div(ann)
						.ok_or(Error::<T>::MathOverflow)?,
				)
				.ok_or(Error::<T>::MathOverflow)?;
			let mut y = *d;

			for _ in 0..256 {
				let y_prev = y;
				y = y
					.checked_mul(&y)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_add(&c)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(
						&y.checked_mul(&T::HigherPrecisionBalance::from(2_u32))
							.ok_or(Error::<T>::MathOverflow)?
							.checked_add(&b)
							.ok_or(Error::<T>::MathOverflow)?
							.checked_sub(d)
							.ok_or(Error::<T>::MathOverflow)?,
					)
					.ok_or(Error::<T>::MathOverflow)?;

				if y.checked_sub(&y_prev).map_or(false, |diff| diff.le(&One::one())) {
					return Ok(y);
				}
				if y_prev.checked_sub(&y).map_or(false, |diff| diff.le(&One::one())) {
					return Ok(y);
				}
			}

			Err(Error::<T>::UnexpectedFailure)
		}

		fn calc_withdraw_one(
			pool_account: &T::AccountId,
			pool: &PoolInfoOf<T>,
			asset_id: T::CurrencyId,
			burn_amount: T::Balance,
		) -> Result<(T::Balance, T::Balance), Error<T>> {
			let n = T::HigherPrecisionBalance::from(pool.assets.len() as u128);
			let i = pool
				.assets
				.iter()
				.position(|x| *x == asset_id)
				.ok_or(Error::<T>::NoSuchAssetInPool)?;

			let (_, xp) = Self::get_balances_xp_pool(pool_account, pool)?;
			let (_, d_0) = Self::get_invariant_pool(pool_account, pool)?;
			let total_supply = T::Currency::total_issuance(pool.lp_token);

			let d_1 = d_0
				.checked_sub(
					&T::HigherPrecisionBalance::from(burn_amount)
						.checked_mul(&d_0)
						.ok_or(Error::<T>::MathOverflow)?
						.checked_div(&T::HigherPrecisionBalance::from(total_supply))
						.ok_or(Error::<T>::MathOverflow)?,
				)
				.ok_or(Error::<T>::MathOverflow)?;

			let new_y = Self::get_y_d(i, &xp, pool.amp_coeff, &d_1)?;
			let base_fee = Self::base_fee(&n)?;

			let ys = d_0
				.checked_add(&d_1)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(
					&n.checked_mul(&T::HigherPrecisionBalance::from(2_u32))
						.ok_or(Error::<T>::MathOverflow)?,
				)
				.ok_or(Error::<T>::MathOverflow)?;

			let mut xp_reduced = vec![];
			for j in 0..pool.assets.len() {
				let mut xavg = Zero::zero();
				let mut dx_exp = Zero::zero();
				let xpjdd = xp[j]
					.checked_mul(&d_1)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(&d_0)
					.ok_or(Error::<T>::MathOverflow)?;
				if i == j {
					dx_exp = xpjdd.checked_sub(&new_y).ok_or(Error::<T>::MathOverflow)?;
					xavg = Self::checked_add_div_2(&xp[j], &new_y)?;
				} else {
					dx_exp = xp[j].checked_sub(&xpjdd).ok_or(Error::<T>::MathOverflow)?;
					xavg = xp[j]
				}
				let dyn_fee = Self::dynamic_fee(xavg, ys, base_fee, Self::fees().2)?;
				xp_reduced.push(
					xp[j]
						.checked_sub(&Self::checked_mul_div_u128(
							&dyn_fee,
							&dx_exp,
							Self::FEE_DENOMINATOR,
						)?)
						.ok_or(Error::<T>::MathOverflow)?,
				)
			}

			let dy = xp_reduced[i]
				.checked_sub(&Self::get_y_d(i, &xp_reduced, pool.amp_coeff, &d_1)?)
				.ok_or(Error::<T>::MathOverflow)?;
			let dy_0 = Self::checked_mul_div_to_balance(
				&xp[i].checked_sub(&new_y).ok_or(Error::<T>::MathOverflow)?,
				pool.rate_multipliers[i],
			)?;
			let dy = Self::checked_mul_div_to_balance(
				&dy.checked_sub(&One::one()) // less for roudning errors
					.ok_or(Error::<T>::MathOverflow)?,
				pool.rate_multipliers[i],
			)?;
			let fee = T::HigherPrecisionBalance::from(
				dy_0.checked_sub(&dy).ok_or(Error::<T>::MathOverflow)?,
			);
			let trsy_fee =
				Self::checked_mul_div_u128(&fee, &Self::fees().1, Self::FEE_DENOMINATOR)?
					.try_into()
					.map_err(|_| Error::<T>::MathOverflow)?;

			Ok((dy, trsy_fee))
		}

		// math
		fn checked_add_div_2(
			a: &T::HigherPrecisionBalance,
			b: &T::HigherPrecisionBalance,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			a.checked_add(b)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(&T::HigherPrecisionBalance::from(2_u32))
				.ok_or(Error::<T>::MathOverflow)
		}

		fn checked_mul_div_u128(
			a: &T::HigherPrecisionBalance,
			b: &T::HigherPrecisionBalance,
			d: u128,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			a.checked_mul(b)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(&T::HigherPrecisionBalance::from(d))
				.ok_or(Error::<T>::MathOverflow)
		}

		fn checked_mul_div_to_balance(
			a: &T::HigherPrecisionBalance,
			rate: T::Balance,
		) -> Result<T::Balance, Error<T>> {
			a.checked_mul(&T::HigherPrecisionBalance::from(Self::PRECISION))
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(&T::HigherPrecisionBalance::from(rate))
				.ok_or(Error::<T>::MathOverflow)?
				.try_into()
				.map_err(|_| Error::<T>::MathOverflow)
		}
	}
}
