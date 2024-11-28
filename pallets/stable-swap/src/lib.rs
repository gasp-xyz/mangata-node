//! # Stable Pools Pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	ensure, fail,
	pallet_prelude::*,
	traits::{
		tokens::{Balance, CurrencyId},
		ExistenceRequirement, MultiTokenCurrency,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;

use mangata_support::pools::{Inspect, Mutate, SwapResult};
use sp_arithmetic::traits::Unsigned;
use sp_runtime::traits::{
	checked_pow, AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Ensure, One,
	TrailingZeroInput, Zero,
};
use sp_std::{convert::TryInto, fmt::Debug, vec, vec::Vec};

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
impl<Id: CurrencyId, B: Balance, MaxAssets: Get<u32>> PoolInfo<Id, B, MaxAssets> {
	fn get_asset_index<T: Config>(&self, id: Id) -> Result<usize, Error<T>> {
		self.assets.iter().position(|x| *x == id).ok_or(Error::<T>::NoSuchAssetInPool)
	}
}

pub type PoolIdOf<T> = <T as Config>::CurrencyId;
pub type PoolInfoOf<T> =
	PoolInfo<<T as Config>::CurrencyId, <T as Config>::Balance, <T as Config>::MaxAssetsInPool>;
pub type AssetIdsOf<T> = BoundedVec<<T as Config>::CurrencyId, <T as Config>::MaxAssetsInPool>;
pub type BalancesOf<T> = BoundedVec<<T as Config>::Balance, <T as Config>::MaxAssetsInPool>;

#[frame_support::pallet]
pub mod pallet {
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

		/// Treasury pallet id used for fee deposits.
		type TreasuryPalletId: Get<PalletId>;

		/// Treasury sub-account used for buy & burn.
		type BnbTreasurySubAccDerive: Get<[u8; 4]>;

		/// Total fee applied to a swap.
		/// Part goes back to pool, part to treasury, and part is burned.
		#[pallet::constant]
		type MarketTotalFee: Get<u128>;

		/// Percentage of total fee that goes into the treasury.
		#[pallet::constant]
		type MarketTreasuryFeePart: Get<u128>;

		/// Percentage of treasury fee that gets burned if possible.
		#[pallet::constant]
		type MarketBnBFeePart: Get<u128>;

		#[pallet::constant]
		type MaxApmCoeff: Get<u128>;

		#[pallet::constant]
		type DefaultApmCoeff: Get<u128>;

		#[pallet::constant]
		type MaxAssetsInPool: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Amplification coefficient lower then 1 or too large
		AmpCoeffOutOfRange,
		/// Initial pool rate multipliers are too large
		InitialPoolRateOutOfRange,
		/// Too many assets for pool creation
		TooManyAssets,
		/// Pool already exists
		PoolAlreadyExists,
		/// Asset does not exist
		AssetDoesNotExist,
		/// Asset ids cannot be the same
		SameAsset,
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
		/// Math overflow
		MathOverflow,
		/// Liquidity token creation failed
		LiquidityTokenCreationFailed,
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
			/// The amounts of the assets that were added to the pool.
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
			/// The account that the liquidity token was taken from.
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
			/// The account that the liquidity token was taken from.
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
		/// 			A custom rate can also be used & values needs to be multiplied by 1e18, with max value of 1e36
		/// 			eg. for a rate 2:1 it's [2e18, 1e18]
		/// * `amp_coeff` - Amplification co-efficient - a lower value here means less tolerance for imbalance within the pool's assets.
		/// 				Suggested values include:
		///    					* Uncollateralized algorithmic stablecoins: 5-10
		///    					* Non-redeemable, collateralized assets: 100
		///   					* Redeemable assets: 200-400
		///
		/// Initial liquidity amounts needs to be provided with [`Pallet::add_liquidity`].
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_pool())]
		pub fn create_pool(
			origin: OriginFor<T>,
			assets: Vec<T::CurrencyId>,
			rates: Vec<T::Balance>,
			amp_coeff: u128,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let info = Self::do_create_pool(&sender, assets, rates, amp_coeff)?;

			Self::deposit_event(Event::PoolCreated {
				creator: sender,
				pool_id: info.lp_token,
				lp_token: info.lp_token,
				assets: info.assets,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::add_liquidity())]
		pub fn swap(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			asset_in: T::CurrencyId,
			asset_out: T::CurrencyId,
			dx: T::Balance,
			min_dy: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// we ignore the buy&burn, funds stay in treasury
			let SwapResult { amount_out, .. } =
				Self::do_swap(&sender, pool_id, asset_in, asset_out, dx, min_dy)?;

			Self::deposit_event(Event::AssetsSwapped {
				who: sender,
				pool_id,
				asset_in,
				amount_in: dx,
				asset_out,
				amount_out,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::add_liquidity())]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			amounts: Vec<T::Balance>,
			min_amount_lp_tokens: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (mint_amount, fees) =
				Self::do_add_liquidity(&sender, pool_id, amounts.clone(), min_amount_lp_tokens)?;

			let total_supply = T::Currency::total_issuance(pool_id);
			Self::deposit_event(Event::LiquidityMinted {
				who: sender,
				pool_id,
				amounts_provided: BoundedVec::truncate_from(amounts),
				lp_token: pool_id,
				lp_token_minted: mint_amount,
				total_supply,
				fees,
			});

			Ok(())
		}

		/// Withdraw a single asset from the pool
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::remove_liquidity_one_asset())]
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
			let pool_account = Self::get_pool_account(&pool_id);

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
			let pool_account = Self::get_pool_account(&pool_id);
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
			let (d_1, fees) = Self::calc_imbalanced_liquidity_fees(
				&pool,
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
			for (&id, &f) in pool.assets.iter().zip(fees.iter()) {
				let to_treasury =
					Self::checked_mul_div_u128(&f, &Self::get_trsy_fee(), Self::FEE_DENOMINATOR)?
						.try_into()
						.map_err(|_| Error::<T>::MathOverflow)?;

				T::Currency::transfer(
					id,
					&pool_account,
					&Self::treasury_account_id(),
					to_treasury,
					ExistenceRequirement::AllowDeath,
				)?;

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

		/// Withdraw balanced assets from the pool given LP tokens amount to burn
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_liquidity())]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			burn_amount: T::Balance,
			min_amounts: Vec<T::Balance>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let amounts = Self::do_remove_liquidity(&sender, pool_id, burn_amount, min_amounts)?;

			let total_supply = T::Currency::total_issuance(pool_id);
			Self::deposit_event(Event::LiquidityBurned {
				who: sender,
				pool_id,
				amounts,
				burned_amount: burn_amount,
				total_supply,
				fees: BoundedVec::new(),
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub const FEE_DENOMINATOR: u128 = 10_u128.pow(10);
		pub(crate) const PRECISION_EXP: u32 = 18;
		const PRECISION: u128 = 10_u128.pow(Self::PRECISION_EXP);
		const A_PRECISION: u128 = 100;

		// calls impl
		pub fn do_create_pool(
			sender: &T::AccountId,
			assets: Vec<T::CurrencyId>,
			rates: Vec<T::Balance>,
			amp_coeff: u128,
		) -> Result<PoolInfoOf<T>, DispatchError> {
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

			let mut to_sort: Vec<(T::CurrencyId, T::Balance)> =
				assets.into_iter().zip(rates.into_iter()).collect();
			to_sort.sort_by_key(|&(a, _)| a);
			let (mut assets, rates): (Vec<T::CurrencyId>, Vec<T::Balance>) =
				to_sort.into_iter().unzip();
			assets.dedup();
			ensure!(assets_in_len == assets.len(), Error::<T>::SameAsset);

			for id in assets.clone() {
				ensure!(T::Currency::exists(id), Error::<T>::AssetDoesNotExist)
			}

			let lp_token: T::CurrencyId = T::Currency::create(&sender, T::Balance::zero())
				.map_err(|_| Error::<T>::LiquidityTokenCreationFailed)?
				.into();
			let pool_account = Self::get_pool_account(&lp_token);
			ensure!(
				!frame_system::Pallet::<T>::account_exists(&pool_account),
				Error::<T>::PoolAlreadyExists
			);
			frame_system::Pallet::<T>::inc_providers(&pool_account);

			let assets = AssetIdsOf::<T>::truncate_from(assets);
			let rates = BalancesOf::<T>::truncate_from(rates);
			let amp_coeff = amp_coeff * Self::A_PRECISION;
			let pool_info = PoolInfo {
				lp_token: lp_token.clone(),
				assets: assets.clone(),
				amp_coeff,
				rate_multipliers: rates,
			};
			Pools::<T>::insert(lp_token.clone(), pool_info.clone());

			Ok(pool_info)
		}

		pub fn do_swap(
			sender: &T::AccountId,
			pool_id: PoolIdOf<T>,
			asset_in: T::CurrencyId,
			asset_out: T::CurrencyId,
			dx: T::Balance,
			min_dy: T::Balance,
		) -> Result<SwapResult<T::Balance>, DispatchError> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);

			// ensure assets in pool
			let i = pool.get_asset_index::<T>(asset_in)?;
			let j = pool.get_asset_index::<T>(asset_out)?;

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

			// get dy for dx
			let (dy, dy_fee) = Self::calc_dy(
				i,
				j,
				T::HigherPrecisionBalance::from(dx),
				pool.amp_coeff,
				&xp,
				pool.rate_multipliers.to_vec(),
			)?;

			let fee = Self::checked_mul_div_to_balance(&dy_fee, pool.rate_multipliers[j])?;
			let to_treasury = Self::checked_mul_div_to_balance(
				&Self::checked_mul_div_u128(&dy_fee, &Self::get_trsy_fee(), Self::FEE_DENOMINATOR)?,
				pool.rate_multipliers[j],
			)?;

			let to_bnb = Self::checked_mul_div_to_balance(
				&Self::checked_mul_div_u128(
					&T::HigherPrecisionBalance::from(to_treasury),
					&Self::get_bnb_fee(),
					Self::FEE_DENOMINATOR,
				)?,
				pool.rate_multipliers[j],
			)?;

			let to_treasury = to_treasury - to_bnb;

			T::Currency::transfer(
				pool.assets[j],
				&pool_account,
				&Self::treasury_account_id(),
				to_treasury,
				ExistenceRequirement::AllowDeath,
			)?;

			T::Currency::transfer(
				pool.assets[j],
				&pool_account,
				&Self::bnb_treasury_account_id(),
				to_bnb,
				ExistenceRequirement::AllowDeath,
			)?;

			// real units
			let dy = Self::checked_mul_div_to_balance(
				&dy.checked_sub(&dy_fee).ok_or(Error::<T>::MathOverflow)?,
				pool.rate_multipliers[j],
			)?;
			ensure!(dy >= min_dy, Error::<T>::InsufficientOutputAmount);

			T::Currency::transfer(
				asset_out,
				&pool_account,
				&sender,
				dy,
				ExistenceRequirement::AllowDeath,
			)?;

			Self::deposit_event(Event::AssetsSwapped {
				who: sender.clone(),
				pool_id,
				asset_in,
				amount_in: dx,
				asset_out,
				amount_out: dy,
			});

			Ok(SwapResult {
				amount_out: dy,
				total_fee: fee,
				treasury_fee: to_treasury,
				bnb_fee: to_bnb,
			})
		}

		pub fn do_add_liquidity(
			sender: &T::AccountId,
			pool_id: PoolIdOf<T>,
			amounts: Vec<T::Balance>,
			min_amount_lp_tokens: T::Balance,
		) -> Result<(T::Balance, BalancesOf<T>), DispatchError> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			ensure!(amounts.len() == pool.assets.len(), Error::<T>::ArgumentsLengthMismatch);
			let pool_account = Self::get_pool_account(&pool_id);
			let asset_amounts = pool.assets.iter().zip(amounts.iter());
			let total_supply = T::Currency::total_issuance(pool.lp_token);
			let n = T::HigherPrecisionBalance::from(pool.assets.len() as u128);

			// check initial amounts
			for amount in amounts.clone() {
				ensure!(
					!(total_supply == Zero::zero() && amount == Zero::zero()),
					Error::<T>::InitialLiquidityZeroAmount
				);
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
				let (d_1, fees) = Self::calc_imbalanced_liquidity_fees(
					&pool,
					&n,
					&d_0,
					&d_1,
					&balances_0,
					&balances_1,
				)?;

				for (&id, &f) in pool.assets.iter().zip(fees.iter()) {
					let to_treasury = Self::checked_mul_div_u128(
						&f,
						&Self::get_trsy_fee(),
						Self::FEE_DENOMINATOR,
					)?
					.try_into()
					.map_err(|_| Error::<T>::MathOverflow)?;

					T::Currency::transfer(
						id,
						&pool_account,
						&Self::treasury_account_id(),
						to_treasury,
						ExistenceRequirement::AllowDeath,
					)?;

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

			Ok((mint_amount, BoundedVec::truncate_from(fees_b)))
		}

		pub fn do_remove_liquidity(
			sender: &T::AccountId,
			pool_id: PoolIdOf<T>,
			burn_amount: T::Balance,
			min_amounts: Vec<T::Balance>,
		) -> Result<BalancesOf<T>, DispatchError> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			ensure!(min_amounts.len() == pool.assets.len(), Error::<T>::ArgumentsLengthMismatch);
			let pool_account = Self::get_pool_account(&pool_id);
			let total_supply = T::Currency::total_issuance(pool.lp_token);

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

				ensure!(value >= min_amounts[i], Error::<T>::InsufficientOutputAmount);

				T::Currency::transfer(
					pool.assets[i],
					&pool_account,
					&sender,
					value,
					ExistenceRequirement::AllowDeath,
				)?;
			}

			T::Currency::burn_and_settle(pool.lp_token, &sender, burn_amount)?;

			Ok(BoundedVec::truncate_from(amounts))
		}

		/// The account of the pool that store asset balances.
		///
		/// This actually does computation. If you need to keep using it, then make sure you cache
		/// the value and only call this once.
		pub fn get_pool_account(pool_id: &PoolIdOf<T>) -> T::AccountId {
			let encoded_pool_id = sp_io::hashing::blake2_256(&Encode::encode(&pool_id));

			Decode::decode(&mut TrailingZeroInput::new(encoded_pool_id.as_ref()))
				.expect("infinite length input; no invalid inputs for type; qed")
		}

		pub fn get_pool_reserves(pool_id: &PoolIdOf<T>) -> Result<Vec<T::Balance>, Error<T>> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);

			Ok(pool
				.assets
				.iter()
				.map(|&id| T::Currency::available_balance(id, &pool_account))
				.collect())
		}

		/// The current virtual price of the pool LP token, useful for calculating profits.
		/// Returns LP token virtual price normalized to 1e18.
		pub fn get_virtual_price(pool_id: &PoolIdOf<T>) -> Result<T::Balance, Error<T>> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);

			let total_supply: <T as Config>::Balance = T::Currency::total_issuance(pool.lp_token);
			let (_, d) = Self::get_invariant_pool(&pool_account, pool)?;

			d.checked_mul(&T::HigherPrecisionBalance::from(Self::PRECISION))
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(&T::HigherPrecisionBalance::from(total_supply))
				.ok_or(Error::<T>::MathOverflow)?
				.try_into()
				.map_err(|_| Error::<T>::MathOverflow)
		}

		/// Calculate the current input dx given output dy.
		pub fn get_dx(
			pool_id: &PoolIdOf<T>,
			asset_in: T::CurrencyId,
			asset_out: T::CurrencyId,
			dy: T::Balance,
		) -> Result<T::Balance, Error<T>> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);
			let (_, xp) = Self::get_balances_xp_pool(&pool_account, &pool)?;
			let i = pool.get_asset_index::<T>(asset_in)?;
			let j = pool.get_asset_index::<T>(asset_out)?;

			Self::get_dx_xp(&pool, i, j, dy, xp)
		}

		pub fn get_dx_with_impact(
			pool_id: &PoolIdOf<T>,
			asset_in: T::CurrencyId,
			asset_out: T::CurrencyId,
			dy: T::Balance,
		) -> Result<(T::Balance, T::Balance), Error<T>> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);
			let (reserves, xp) = Self::get_balances_xp_pool(&pool_account, &pool)?;
			let i = pool.get_asset_index::<T>(asset_in)?;
			let j = pool.get_asset_index::<T>(asset_out)?;

			let dx = Self::get_dx_xp(&pool, i, j, dy, xp)?;

			let mut reserves = reserves.clone();
			reserves[i] = reserves[i] + T::HigherPrecisionBalance::from(dx);
			reserves[j] = reserves[j] - T::HigherPrecisionBalance::from(dy);
			let xp = Self::xp(&pool.rate_multipliers, &reserves)?;

			let dx2 = Self::get_dx_xp(&pool, i, j, dy, xp)?;

			Ok((dx, dx2))
		}

		/// Calculate the output dy given input dx.
		pub fn get_dy(
			pool_id: &PoolIdOf<T>,
			asset_in: T::CurrencyId,
			asset_out: T::CurrencyId,
			dx: T::Balance,
		) -> Result<T::Balance, Error<T>> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);
			let (_, xp) = Self::get_balances_xp_pool(&pool_account, &pool)?;
			let i = pool.get_asset_index::<T>(asset_in)?;
			let j = pool.get_asset_index::<T>(asset_out)?;

			log::debug!(target: "", "{:?} {:?} {:?} {:?}", i, j, dx, xp);
			Self::get_dy_xp(&pool, i, j, dx, xp)
		}

		pub fn get_dy_with_impact(
			pool_id: &PoolIdOf<T>,
			asset_in: T::CurrencyId,
			asset_out: T::CurrencyId,
			dx: T::Balance,
		) -> Result<(T::Balance, T::Balance), Error<T>> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);
			let (reserves, xp) = Self::get_balances_xp_pool(&pool_account, &pool)?;
			let i = pool.get_asset_index::<T>(asset_in)?;
			let j = pool.get_asset_index::<T>(asset_out)?;

			let dy = Self::get_dy_xp(&pool, i, j, dx, xp)?;

			let mut reserves = reserves.clone();
			reserves[i] = reserves[i] + T::HigherPrecisionBalance::from(dx);
			reserves[j] = reserves[j] - T::HigherPrecisionBalance::from(dy);
			let xp = Self::xp(&pool.rate_multipliers, &reserves)?;

			let dy2 = Self::get_dy_xp(&pool, i, j, dx, xp)?;

			Ok((dy, dy2))
		}

		pub fn calc_lp_token_amount(
			pool_id: &PoolIdOf<T>,
			amounts: Vec<T::Balance>,
			is_deposit: bool,
		) -> Result<T::Balance, Error<T>> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);

			let n = T::HigherPrecisionBalance::from(pool.assets.len() as u128);
			let total_supply: <T as Config>::Balance = T::Currency::total_issuance(pool.lp_token);

			let (balances_0, d_0) = Self::get_invariant_pool(&pool_account, &pool)?;

			let mut balances_1 = vec![Zero::zero(); pool.assets.len()];
			for i in 0..balances_0.len() {
				let amount = T::HigherPrecisionBalance::from(amounts[i]);
				balances_1[i] = if is_deposit {
					balances_0[i].checked_add(&amount).ok_or(Error::<T>::MathOverflow)?
				} else {
					balances_0[i].checked_sub(&amount).ok_or(Error::<T>::MathOverflow)?
				}
			}
			let xp_1 = Self::xp(&pool.rate_multipliers, &balances_1)?;
			let d_1 = Self::get_invariant(&xp_1, pool.amp_coeff)?;

			let d_2 = if total_supply > Zero::zero() {
				let (d, _) = Self::calc_imbalanced_liquidity_fees(
					&pool,
					&n,
					&d_0,
					&d_1,
					&balances_0,
					&balances_1,
				)?;
				d
			} else {
				d_1
			};

			let diff = if is_deposit {
				d_2.checked_sub(&d_0).ok_or(Error::<T>::MathOverflow)?
			} else {
				d_0.checked_sub(&d_2).ok_or(Error::<T>::MathOverflow)?
			};

			let r = diff
				.checked_mul(&T::HigherPrecisionBalance::from(total_supply))
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(&d_0)
				.ok_or(Error::<T>::MathOverflow)?
				.try_into()
				.map_err(|_| Error::<T>::MathOverflow)?;

			Ok(r)
		}

		pub fn get_burn_amounts(
			pool_id: &PoolIdOf<T>,
			burn_amount: T::Balance,
		) -> Result<Vec<T::Balance>, DispatchError> {
			let maybe_pool = Pools::<T>::get(pool_id.clone());
			let pool = maybe_pool.as_ref().ok_or(Error::<T>::NoSuchPool)?;
			let pool_account = Self::get_pool_account(&pool_id);
			let total_supply = T::Currency::total_issuance(pool.lp_token);

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
			}

			Ok(amounts)
		}

		fn treasury_account_id() -> T::AccountId {
			T::TreasuryPalletId::get().into_account_truncating()
		}

		fn bnb_treasury_account_id() -> T::AccountId {
			T::TreasuryPalletId::get()
				.into_sub_account_truncating(T::BnbTreasurySubAccDerive::get())
		}

		fn get_fee() -> T::HigherPrecisionBalance {
			T::HigherPrecisionBalance::from(T::MarketTotalFee::get())
		}

		fn get_trsy_fee() -> T::HigherPrecisionBalance {
			T::HigherPrecisionBalance::from(T::MarketTreasuryFeePart::get())
		}

		fn get_bnb_fee() -> T::HigherPrecisionBalance {
			T::HigherPrecisionBalance::from(T::MarketBnBFeePart::get())
		}

		// dyn fee 2* mul
		fn get_fee_dyn_mul() -> T::HigherPrecisionBalance {
			T::HigherPrecisionBalance::from(20_000_000_000_u128)
			// T::HigherPrecisionBalance::from(1_u128)
		}

		fn has_dynamic_fee() -> bool {
			let m = Self::get_fee_dyn_mul();
			let den = T::HigherPrecisionBalance::from(Self::FEE_DENOMINATOR);
			return m > den
		}

		// stable swap maths

		fn base_fee(
			n: &T::HigherPrecisionBalance,
		) -> Result<<T as Config>::HigherPrecisionBalance, Error<T>> {
			let fee = Self::get_fee();
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

		fn dynamic_fee(
			xpi: &T::HigherPrecisionBalance,
			xpj: &T::HigherPrecisionBalance,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			let fee = Self::get_fee();
			let mul = Self::get_fee_dyn_mul();
			Self::calc_dynamic_fee(xpi, xpj, &fee, &mul)
		}

		fn dynamic_fee_base(
			xpi: &T::HigherPrecisionBalance,
			xpj: &T::HigherPrecisionBalance,
			n: &T::HigherPrecisionBalance,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			let fee = Self::base_fee(n)?;
			let mul = Self::get_fee_dyn_mul();
			Self::calc_dynamic_fee(xpi, xpj, &fee, &mul)
		}

		// https://www.desmos.com/calculator/zhrwbvcipo
		fn calc_dynamic_fee(
			xpi: &T::HigherPrecisionBalance,
			xpj: &T::HigherPrecisionBalance,
			fee: &T::HigherPrecisionBalance,
			m: &T::HigherPrecisionBalance,
		) -> Result<T::HigherPrecisionBalance, Error<T>> {
			let den = T::HigherPrecisionBalance::from(Self::FEE_DENOMINATOR);
			if *m <= den {
				return Ok(*fee);
			}

			let xps2 = checked_pow(xpi.checked_add(xpj).ok_or(Error::<T>::MathOverflow)?, 2)
				.ok_or(Error::<T>::MathOverflow)?;

			let res = fee
				.checked_mul(m)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(
					&m.checked_sub(&den)
						.ok_or(Error::<T>::MathOverflow)?
						.checked_mul(&T::HigherPrecisionBalance::from(4_u32))
						.ok_or(Error::<T>::MathOverflow)?
						.checked_mul(xpi)
						.ok_or(Error::<T>::MathOverflow)?
						.checked_mul(xpj)
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

		pub fn get_dx_xp(
			pool: &PoolInfoOf<T>,
			i: usize,
			j: usize,
			dy: T::Balance,
			xp: Vec<T::HigherPrecisionBalance>,
		) -> Result<T::Balance, Error<T>> {
			let d = Self::get_invariant(&xp, pool.amp_coeff)?;

			let mut x = xp[i];
			let mut y = xp[j];
			for _i in 0..255 {
				let x_prev = x;
				let dyn_fee = Self::dynamic_fee(
					&Self::checked_add_div_2(&xp[i], &x)?,
					&Self::checked_add_div_2(&xp[j], &y)?,
				)?;
				let dy_fee = Self::checked_mul_div_u128(
					&T::HigherPrecisionBalance::from(dy),
					&T::HigherPrecisionBalance::from(pool.rate_multipliers[j]),
					Self::PRECISION,
				)?
				.checked_add(&One::one())
				.ok_or(Error::<T>::MathOverflow)?
				.checked_mul(&T::HigherPrecisionBalance::from(Self::FEE_DENOMINATOR))
				.ok_or(Error::<T>::MathOverflow)?
				.checked_div(
					&T::HigherPrecisionBalance::from(Self::FEE_DENOMINATOR)
						.checked_sub(&dyn_fee)
						.ok_or(Error::<T>::MathOverflow)?,
				)
				.ok_or(Error::<T>::MathOverflow)?;

				y = xp[j].checked_sub(&dy_fee).ok_or(Error::<T>::MathOverflow)?;
				x = Self::get_y(j, i, &y, &xp, pool.amp_coeff, &d)?;

				// if we don't have dynamic fee we can return immediatelly, otherwise loop with adjusted fee
				if !Self::has_dynamic_fee() || Self::check_diff_le_one(&x, &x_prev) {
					return Ok(Self::checked_mul_div_to_balance(
						&x.checked_sub(&xp[i]).ok_or(Error::<T>::MathOverflow)?,
						pool.rate_multipliers[i],
					)?);
				}
			}

			Err(Error::<T>::UnexpectedFailure)
		}

		fn get_dy_xp(
			pool: &PoolInfoOf<T>,
			i: usize,
			j: usize,
			dx: T::Balance,
			xp: Vec<T::HigherPrecisionBalance>,
		) -> Result<T::Balance, Error<T>> {
			let (dy, dy_fee) = Self::calc_dy(
				i,
				j,
				T::HigherPrecisionBalance::from(dx),
				pool.amp_coeff,
				&xp,
				pool.rate_multipliers.to_vec(),
			)?;

			Self::checked_mul_div_to_balance(
				&dy.checked_sub(&dy_fee).ok_or(Error::<T>::MathOverflow)?,
				pool.rate_multipliers[j],
			)
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

				if Self::check_diff_le_one(&d, &d_prev) {
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

				if Self::check_diff_le_one(&y, &y_prev) {
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
			let i = pool.get_asset_index(asset_id)?;

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
				let xpjdd = xp[j]
					.checked_mul(&d_1)
					.ok_or(Error::<T>::MathOverflow)?
					.checked_div(&d_0)
					.ok_or(Error::<T>::MathOverflow)?;
				let xavg: T::HigherPrecisionBalance;
				let dx_exp: T::HigherPrecisionBalance;
				if i == j {
					dx_exp = xpjdd.checked_sub(&new_y).ok_or(Error::<T>::MathOverflow)?;
					xavg = Self::checked_add_div_2(&xp[j], &new_y)?;
				} else {
					dx_exp = xp[j].checked_sub(&xpjdd).ok_or(Error::<T>::MathOverflow)?;
					xavg = xp[j]
				}
				let dyn_fee = Self::dynamic_fee_base(&xavg, &ys, &n)?;
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
				Self::checked_mul_div_u128(&fee, &Self::get_trsy_fee(), Self::FEE_DENOMINATOR)?
					.try_into()
					.map_err(|_| Error::<T>::MathOverflow)?;

			Ok((dy, trsy_fee))
		}

		fn calc_dy(
			i: usize,
			j: usize,
			dx: T::HigherPrecisionBalance,
			amp: u128,
			xp: &Vec<T::HigherPrecisionBalance>,
			rates: Vec<T::Balance>,
		) -> Result<(T::HigherPrecisionBalance, T::HigherPrecisionBalance), Error<T>> {
			let x = Self::checked_mul_div_u128(
				&dx,
				&T::HigherPrecisionBalance::from(rates[i]),
				Self::PRECISION,
			)?
			.checked_add(&xp[i])
			.ok_or(Error::<T>::MathOverflow)?;

			let d = Self::get_invariant(&xp, amp)?;
			let y = Self::get_y(i, j, &x, xp, amp, &d)?;
			// -1 in case of rounding error
			let dy = xp[j]
				.checked_sub(&y)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_sub(&One::one())
				.ok_or(Error::<T>::MathOverflow)?;

			// fees
			let dyn_fee = Self::dynamic_fee(
				&Self::checked_add_div_2(&xp[i], &x)?,
				&Self::checked_add_div_2(&xp[j], &y)?,
			)?;
			let dy_fee = Self::checked_mul_div_u128(&dy, &dyn_fee, Self::FEE_DENOMINATOR)?;
			Ok((dy, dy_fee))
		}

		fn calc_imbalanced_liquidity_fees(
			pool: &PoolInfoOf<T>,
			n: &T::HigherPrecisionBalance,
			d_0: &T::HigherPrecisionBalance,
			d_1: &T::HigherPrecisionBalance,
			balances_0: &Vec<T::HigherPrecisionBalance>,
			balances_1: &Vec<T::HigherPrecisionBalance>,
		) -> Result<(T::HigherPrecisionBalance, Vec<T::HigherPrecisionBalance>), Error<T>> {
			let mut fees = vec![];
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

				let dyn_fee = Self::dynamic_fee_base(&xs, &ys, n)?;
				fees.push(Self::checked_mul_div_u128(&diff, &dyn_fee, Self::FEE_DENOMINATOR)?);

				// this can fail if the fee is bigger then available balance
				// eg. pool initialized with low liquidity and adding big amount for single coin
				// would cause fees on other assets larger then initial liquidity
				balances_mint[i] =
					balances_mint[i].checked_sub(&fees[i]).ok_or(Error::<T>::MathOverflow)?;
			}

			let xp = Self::xp(&pool.rate_multipliers, &balances_mint)?;
			let d_1 = Self::get_invariant(&xp, pool.amp_coeff)?;

			Ok((d_1, fees))
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

		fn check_diff_le_one(a: &T::HigherPrecisionBalance, b: &T::HigherPrecisionBalance) -> bool {
			if a.checked_sub(&b).map_or(false, |diff| diff.le(&One::one())) {
				return true;
			}
			if b.checked_sub(&a).map_or(false, |diff| diff.le(&One::one())) {
				return true;
			}
			return false;
		}
	}
}

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
	type CurrencyId = T::CurrencyId;
	type Balance = T::Balance;

	fn get_pool_info(
		pool_id: Self::CurrencyId,
	) -> Option<mangata_support::pools::PoolInfo<Self::CurrencyId>> {
		let info = Pools::<T>::get(pool_id)?;
		let asset1 = info.assets.get(0)?;
		let asset2 = info.assets.get(1)?;
		Some((*asset1, *asset2))
	}

	fn get_pool_reserves(
		pool_id: Self::CurrencyId,
	) -> Option<mangata_support::pools::PoolReserves<Self::Balance>> {
		let reserves = Self::get_pool_reserves(&pool_id).ok()?;
		let balance1 = reserves.get(0)?;
		let balance2 = reserves.get(1)?;
		Some((*balance1, *balance2))
	}

	fn get_dy(
		pool_id: Self::CurrencyId,
		asset_in: Self::CurrencyId,
		asset_out: Self::CurrencyId,
		dx: Self::Balance,
	) -> Option<Self::Balance> {
		Self::get_dy(&pool_id, asset_in, asset_out, dx).ok()
	}

	fn get_dy_with_impact(
		pool_id: Self::CurrencyId,
		asset_in: Self::CurrencyId,
		asset_out: Self::CurrencyId,
		dx: Self::Balance,
	) -> Option<(Self::Balance, Self::Balance)> {
		Self::get_dy_with_impact(&pool_id, asset_in, asset_out, dx).ok()
	}

	fn get_dx(
		pool_id: Self::CurrencyId,
		asset_in: Self::CurrencyId,
		asset_out: Self::CurrencyId,
		dy: Self::Balance,
	) -> Option<Self::Balance> {
		Self::get_dx(&pool_id, asset_in, asset_out, dy).ok()
	}

	fn get_dx_with_impact(
		pool_id: Self::CurrencyId,
		asset_in: Self::CurrencyId,
		asset_out: Self::CurrencyId,
		dy: Self::Balance,
	) -> Option<(Self::Balance, Self::Balance)> {
		Self::get_dx_with_impact(&pool_id, asset_in, asset_out, dy).ok()
	}

	fn get_burn_amounts(
		pool_id: Self::CurrencyId,
		lp_burn_amount: Self::Balance,
	) -> Option<(Self::Balance, Self::Balance)> {
		let amounts = Self::get_burn_amounts(&pool_id, lp_burn_amount).ok()?;
		let asset1 = amounts.get(0)?;
		let asset2 = amounts.get(1)?;
		Some((*asset1, *asset2))
	}

	fn get_non_empty_pools() -> Option<Vec<Self::CurrencyId>> {
		let result = Pools::<T>::iter_values()
			.map(|v| v.lp_token)
			.filter(|v| !T::Currency::total_issuance((*v).into()).is_zero())
			.collect();

		Some(result)
	}

	fn get_mint_amount(
		pool_id: Self::CurrencyId,
		amounts: (Self::Balance, Self::Balance),
	) -> Option<Self::Balance> {
		Self::calc_lp_token_amount(&pool_id, vec![amounts.0, amounts.1], true).ok()
	}
}

impl<T: Config> Mutate<T::AccountId> for Pallet<T> {
	fn create_pool(
		sender: &T::AccountId,
		asset_1: Self::CurrencyId,
		amount_1: Self::Balance,
		asset_2: Self::CurrencyId,
		amount_2: Self::Balance,
	) -> Result<Self::CurrencyId, DispatchError> {
		let assets = vec![asset_1, asset_2];
		let ten = T::Balance::from(10_u32);

		let exp = |amount: Self::Balance| {
			let mut i = 0_usize;
			let mut pow_10 = T::Balance::one() * ten;
			while amount % pow_10 == Zero::zero() {
				i += 1;
				pow_10 *= ten;
			}
			i
		};

		let exp1 = exp(amount_1);
		let exp2 = exp(amount_2);
		let min = exp1.min(exp2);
		let exp = checked_pow(ten, min).ok_or(Error::<T>::MathOverflow)?;

		let rate_1_mul = amount_1 / exp;
		let rate_2_mul = amount_2 / exp;

		// the max rate cannot be more than 1e18
		let precision =
			checked_pow(ten, Self::PRECISION_EXP as usize).ok_or(Error::<T>::MathOverflow)?;
		ensure!(rate_1_mul <= precision, Error::<T>::InitialPoolRateOutOfRange);
		ensure!(rate_2_mul <= precision, Error::<T>::InitialPoolRateOutOfRange);

		let rate_1 = rate_2_mul.checked_mul(&precision).ok_or(Error::<T>::MathOverflow)?;
		let rate_2 = rate_1_mul.checked_mul(&precision).ok_or(Error::<T>::MathOverflow)?;

		let rates = vec![rate_1, rate_2];
		let info = Self::do_create_pool(sender, assets, rates, T::DefaultApmCoeff::get())?;

		// asset ids are ordered
		let asset_pool_1 = *info.assets.get(0).ok_or(Error::<T>::UnexpectedFailure)?;
		let amounts = if asset_pool_1 == asset_1 {
			vec![amount_1, amount_2]
		} else {
			vec![amount_2, amount_1]
		};

		let _ = Self::do_add_liquidity(&sender, info.lp_token, amounts, Zero::zero())?;

		Ok(info.lp_token)
	}

	fn add_liquidity(
		sender: &T::AccountId,
		pool_id: Self::CurrencyId,
		amounts: (Self::Balance, Self::Balance),
		min_amount_lp_tokens: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		let amounts = vec![amounts.0, amounts.1];
		let (minted, _) = Self::do_add_liquidity(&sender, pool_id, amounts, min_amount_lp_tokens)?;
		Ok(minted)
	}

	fn remove_liquidity(
		sender: &T::AccountId,
		pool_id: Self::CurrencyId,
		liquidity_asset_amount: Self::Balance,
		min_asset_amounts_out: (Self::Balance, Self::Balance),
	) -> Result<(T::Balance, T::Balance), DispatchError> {
		let min_amounts = vec![min_asset_amounts_out.0, min_asset_amounts_out.1];
		let amounts =
			Self::do_remove_liquidity(&sender, pool_id, liquidity_asset_amount, min_amounts)?;
		let asset1 = amounts.get(0).ok_or(Error::<T>::UnexpectedFailure)?;
		let asset2 = amounts.get(1).ok_or(Error::<T>::UnexpectedFailure)?;
		Ok((*asset1, *asset2))
	}

	fn swap(
		sender: &T::AccountId,
		pool_id: Self::CurrencyId,
		asset_in: Self::CurrencyId,
		asset_out: Self::CurrencyId,
		amount_in: Self::Balance,
		min_amount_out: Self::Balance,
	) -> Result<SwapResult<Self::Balance>, DispatchError> {
		Self::do_swap(sender, pool_id, asset_in, asset_out, amount_in, min_amount_out)
	}
}
