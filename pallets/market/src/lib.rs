//! # Market Pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		tokens::{
			currency::{MultiTokenCurrency, MultiTokenVestingLocks},
			Balance, CurrencyId,
		},
		Contains,
	},
};
use frame_system::pallet_prelude::*;
use mangata_support::{
	pools::{Inspect, Mutate, SwapResult, TreasuryBurn},
	traits::{
		AssetRegistryProviderTrait, GetMaintenanceStatusTrait, ProofOfStakeRewardsApi,
		XykFunctionsTrait,
	},
};
use mangata_types::multipurpose_liquidity::ActivateKind;

#[cfg(feature = "runtime-benchmarks")]
use mangata_support::traits::ComputeIssuance;

use sp_runtime::traits::{MaybeDisplay, MaybeFromStr, Saturating, Zero};
use sp_std::{convert::TryInto, fmt::Debug, vec, vec::Vec};

use orml_tokens::MultiTokenCurrencyExtended;
use orml_traits::asset_registry::Inspect as AssetRegistryInspect;

pub mod weights;
pub use crate::weights::WeightInfo;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Eq, PartialEq, Debug, Clone, TypeInfo)]
pub enum PoolKind {
	/// Classic XYK invariant
	Xyk,
	/// StableSwap
	StableSwap,
}

#[derive(Clone, Debug)]
pub struct PoolInfo<CurrencyId> {
	pub pool_id: CurrencyId,
	pub kind: PoolKind,
	pub pool: mangata_support::pools::PoolInfo<CurrencyId>,
}

impl<C: PartialEq + Copy> PoolInfo<C> {
	fn same_and_other(&self, same: C) -> Option<(C, C)> {
		if same == self.pool.0 {
			Some(self.pool)
		} else if same == self.pool.1 {
			Some((self.pool.1, self.pool.0))
		} else {
			None
		}
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Debug, Clone, TypeInfo)]
pub struct AtomicSwap<CurrencyId, Balance> {
	pub pool_id: CurrencyId,
	pub kind: PoolKind,
	pub asset_in: CurrencyId,
	pub asset_out: CurrencyId,
	pub amount_in: Balance,
	pub amount_out: Balance,
}

// use LP token as pool id, extra type for readability
pub type PoolIdOf<T> = <T as Config>::CurrencyId;
// pools are composed of a pair of assets
pub type PoolInfoOf<T> = PoolInfo<<T as Config>::CurrencyId>;
pub type AssetPairOf<T> = (<T as Config>::CurrencyId, <T as Config>::CurrencyId);
pub type BalancePairOf<T> = (<T as Config>::Balance, <T as Config>::Balance);
pub type AtomicSwapOf<T> = AtomicSwap<<T as Config>::CurrencyId, <T as Config>::Balance>;

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

		/// Identifier for the assets.
		type CurrencyId: CurrencyId;

		/// Native currency
		type NativeCurrencyId: Get<Self::CurrencyId>;

		/// Xyk pools
		type Xyk: XykFunctionsTrait<Self::AccountId, Self::Balance, Self::CurrencyId>
			+ TreasuryBurn<Self::AccountId, CurrencyId = Self::CurrencyId, Balance = Self::Balance>;

		/// StableSwap pools
		type StableSwap: Mutate<
			Self::AccountId,
			CurrencyId = Self::CurrencyId,
			Balance = Self::Balance,
		>;

		/// Reward apis for native asset LP tokens activation
		type Rewards: ProofOfStakeRewardsApi<Self::AccountId, Self::Balance, Self::CurrencyId>;

		/// Vesting apis for providing native vested liquidity
		type Vesting: MultiTokenVestingLocks<
			Self::AccountId,
			Moment = BlockNumberFor<Self>,
			Currency = Self::Currency,
		>;

		/// Apis for LP asset creation in asset registry
		type AssetRegistry: AssetRegistryProviderTrait<Self::CurrencyId>
			+ AssetRegistryInspect<AssetId = Self::CurrencyId>;

		/// List of tokens ids that are not allowed to be used at all
		type DisabledTokens: Contains<Self::CurrencyId>;

		/// List of assets that are not allowed to form a pool
		type DisallowedPools: Contains<AssetPairOf<Self>>;

		/// Disable trading with maintenance mode
		type MaintenanceStatusProvider: GetMaintenanceStatusTrait;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Tokens which cannot be transfered by extrinsics/user or use in pool, unless foundation override
		type NontransferableTokens: Contains<Self::CurrencyId>;

		/// A list of Foundation members with elevated rights
		type FoundationAccountsProvider: Get<Vec<Self::AccountId>>;

		/// A special account used for nontransferable tokens to allow 'selling' to balance pools
		type ArbitrageBot: Contains<Self::AccountId>;

		#[cfg(feature = "runtime-benchmarks")]
		type ComputeIssuance: ComputeIssuance;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No such pool exists
		NoSuchPool,
		/// Asset id is not allowed
		FunctionNotAvailableForThisToken,
		/// Asset ids are not allowed to create a pool
		DisallowedPool,
		/// Insufficient output amount does not meet min requirements
		InsufficientOutputAmount,
		/// Excesive input amount does not meet max requirements
		ExcesiveInputAmount,
		/// Pool is not paired with native currency id
		NotPairedWithNativeAsset,
		/// Not a promoted pool
		NotAPromotedPool,
		/// Asset does not exists
		AssetDoesNotExists,
		/// Operation not available for such pool type
		FunctionNotAvailableForThisPoolKind,
		/// Trading blocked by maintenance mode
		TradingBlockedByMaintenanceMode,
		/// Multi swap path contains repetive pools
		MultiSwapSamePool,
		/// Input asset id is not connected with output asset id for given pools
		MultiSwapPathInvalid,
		/// Asset cannot be used to create or modify a pool
		NontransferableToken,
	}

	// Pallet's events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Assets were swapped successfully
		AssetsSwapped {
			/// The account that initiated the swap
			who: T::AccountId,
			/// List of the atomic asset swaps
			swaps: Vec<AtomicSwapOf<T>>,
		},

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
			assets: AssetPairOf<T>,
		},

		/// A successful call of the `AddLiquidity` extrinsic will create this event.
		LiquidityMinted {
			/// The account that the liquidity was taken from.
			who: T::AccountId,
			/// The id of the pool that the liquidity was added to.
			pool_id: PoolIdOf<T>,
			/// The amounts of the assets that were added to the pool.
			amounts_provided: BalancePairOf<T>,
			/// The id of the LP token that was minted.
			lp_token: T::CurrencyId,
			/// The amount of lp tokens that were minted of that id.
			lp_token_minted: T::Balance,
			/// The new total supply of the associated LP token.
			total_supply: T::Balance,
		},

		/// A successful call of the `RemoveLiquidity` extrinsic will create this event.
		LiquidityBurned {
			/// The account that the liquidity token was taken from.
			who: T::AccountId,
			/// The id of the pool that the liquidity was taken from.
			pool_id: PoolIdOf<T>,
			/// The amount of the asset that was received.
			amounts: BalancePairOf<T>,
			/// The amount of the associated LP token that was burned.
			burned_amount: T::Balance,
			/// The new total supply of the associated LP token.
			total_supply: T::Balance,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			assert!(true, "template",);
		}
	}

	/// Pallet's callable functions.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a liquidity pool and an associated new `lp_token` asset
		/// For a StableSwap pool, the "stable" rate is computed from the ratio of input amounts, max rate is 1e18:1
		#[pallet::call_index(0)]
		#[pallet::weight(
			T::WeightInfo::create_pool_xyk().max(
			T::WeightInfo::create_pool_sswap()
		))]
		pub fn create_pool(
			origin: OriginFor<T>,
			kind: PoolKind,
			first_asset_id: T::CurrencyId,
			first_asset_amount: T::Balance,
			second_asset_id: T::CurrencyId,
			second_asset_amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// check assets id, or the foundation has a veto
			ensure!(
				(!T::NontransferableTokens::contains(&first_asset_id) &&
					!T::NontransferableTokens::contains(&second_asset_id)) ||
					T::FoundationAccountsProvider::get().contains(&sender),
				Error::<T>::NontransferableToken
			);

			Self::check_assets_allowed((first_asset_id, second_asset_id))?;

			ensure!(
				!T::DisallowedPools::contains(&(first_asset_id, second_asset_id)),
				Error::<T>::DisallowedPool,
			);

			let lp_token = match kind {
				PoolKind::Xyk => {
					let lp_token = T::Xyk::create_pool(
						sender.clone(),
						first_asset_id,
						first_asset_amount,
						second_asset_id,
						second_asset_amount,
					)?;
					lp_token
				},
				PoolKind::StableSwap => {
					let lp_token = T::StableSwap::create_pool(
						&sender,
						first_asset_id,
						first_asset_amount,
						second_asset_id,
						second_asset_amount,
					)?;

					T::AssetRegistry::create_pool_asset(lp_token, first_asset_id, second_asset_id)?;
					lp_token
				},
			};

			Self::deposit_event(Event::PoolCreated {
				creator: sender.clone(),
				pool_id: lp_token,
				lp_token,
				assets: (first_asset_id, second_asset_id),
			});

			let lp_supply = T::Currency::total_issuance(lp_token);
			Self::deposit_event(Event::LiquidityMinted {
				who: sender,
				pool_id: lp_token,
				amounts_provided: (first_asset_amount, second_asset_amount),
				lp_token,
				lp_token_minted: lp_supply,
				total_supply: lp_supply,
			});

			Ok(())
		}

		/// Provide liquidity into the pool of `pool_id`, suitable for Xyk pools.
		/// An optimal amount of the other asset will be calculated on current rate,
		/// a maximum amount should be provided to limit possible rate slippage.
		/// For a StableSwap pool a rate of 1:1 is used.
		/// Liquidity tokens that represent this share of the pool will be sent to origin.
		#[pallet::call_index(1)]
		#[pallet::weight(
			T::WeightInfo::mint_liquidity_xyk().max(
			T::WeightInfo::mint_liquidity_sswap()
		))]
		pub fn mint_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			asset_id: T::CurrencyId,
			asset_amount: T::Balance,
			max_other_asset_amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let pool_info = Self::get_pool_info(pool_id)?;
			// check assets id, foundation has no veto
			ensure!(
				!T::NontransferableTokens::contains(&pool_info.pool.0) &&
					!T::NontransferableTokens::contains(&pool_info.pool.1),
				Error::<T>::NontransferableToken
			);
			Self::check_assets_allowed(pool_info.pool)?;

			let (lp_amount, other_asset_amount) = Self::do_mint_liquidity(
				&sender,
				pool_info,
				asset_id,
				asset_amount,
				max_other_asset_amount,
				true,
			)?;

			let lp_supply = T::Currency::total_issuance(pool_id);
			Self::deposit_event(Event::LiquidityMinted {
				who: sender,
				pool_id,
				amounts_provided: (asset_amount, other_asset_amount),
				lp_token: pool_id,
				lp_token_minted: lp_amount,
				total_supply: lp_supply,
			});

			Ok(())
		}

		/// Provide fixed liquidity into the pool of `pool_id`, suitable for StableSwap pools.
		/// For Xyk pools, if a single amount is defined, it will swap internally to to match current rate,
		/// setting both values results in error.
		/// Liquidity tokens that represent this share of the pool will be sent to origin.
		#[pallet::call_index(2)]
		#[pallet::weight(
			T::WeightInfo::mint_liquidity_fixed_amounts_xyk().max(
			T::WeightInfo::mint_liquidity_fixed_amounts_sswap()
		))]
		pub fn mint_liquidity_fixed_amounts(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			amounts: (T::Balance, T::Balance),
			min_amount_lp_tokens: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let pool_info = Self::get_pool_info(pool_id)?;
			// check assets id, foundation has no veto
			ensure!(
				!T::NontransferableTokens::contains(&pool_info.pool.0) &&
					!T::NontransferableTokens::contains(&pool_info.pool.1),
				Error::<T>::NontransferableToken
			);
			Self::check_assets_allowed(pool_info.pool)?;

			let lp_amount = match pool_info.kind {
				PoolKind::Xyk => {
					ensure!(
						amounts.0 == Zero::zero() || amounts.1 == Zero::zero(),
						Error::<T>::FunctionNotAvailableForThisPoolKind
					);

					let (id, amount) = if amounts.1 == Zero::zero() {
						(pool_info.pool.0, amounts.0)
					} else {
						(pool_info.pool.1, amounts.1)
					};

					let (_, lp_amount) = T::Xyk::provide_liquidity_with_conversion(
						sender.clone(),
						pool_info.pool.0,
						pool_info.pool.1,
						id,
						amount,
						true,
					)?;
					ensure!(lp_amount > min_amount_lp_tokens, Error::<T>::InsufficientOutputAmount);
					lp_amount
				},
				PoolKind::StableSwap => {
					let amount = T::StableSwap::add_liquidity(
						&sender,
						pool_id,
						amounts,
						min_amount_lp_tokens,
					)?;
					if T::Rewards::native_rewards_enabled(pool_info.pool_id) {
						T::Rewards::activate_liquidity(
							sender.clone(),
							pool_id,
							amount,
							Some(ActivateKind::AvailableBalance),
						)?;
					}
					amount
				},
			};

			let lp_supply = T::Currency::total_issuance(pool_id);
			Self::deposit_event(Event::LiquidityMinted {
				who: sender,
				pool_id,
				amounts_provided: amounts,
				lp_token: pool_id,
				lp_token_minted: lp_amount,
				total_supply: lp_supply,
			});

			Ok(())
		}

		/// Provides liquidity from vested native asset. Tokens are added to pool and
		/// minted LP tokens are then vested instead.
		/// Only pools paired with native asset are allowed.
		#[pallet::call_index(3)]
		#[pallet::weight(
			T::WeightInfo::mint_liquidity_using_vesting_native_tokens_by_vesting_index_xyk().max(
			T::WeightInfo::mint_liquidity_using_vesting_native_tokens_by_vesting_index_sswap()
		))]
		pub fn mint_liquidity_using_vesting_native_tokens_by_vesting_index(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			native_asset_vesting_index: u32,
			vesting_native_asset_unlock_some_amount_or_all: Option<T::Balance>,
			max_other_asset_amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let pool_info = Self::get_pool_info(pool_id)?;
			// check assets id, foundation has no veto
			ensure!(
				!T::NontransferableTokens::contains(&pool_info.pool.0) &&
					!T::NontransferableTokens::contains(&pool_info.pool.1),
				Error::<T>::NontransferableToken
			);
			Self::check_assets_allowed(pool_info.pool)?;

			let native_id = T::NativeCurrencyId::get();
			ensure!(
				native_id == pool_info.pool.0 || native_id == pool_info.pool.1,
				Error::<T>::NotPairedWithNativeAsset
			);

			ensure!(T::Rewards::native_rewards_enabled(pool_id), Error::<T>::NotAPromotedPool);

			let (unlocked_amount, vesting_starting_block, vesting_ending_block_as_balance) =
				T::Vesting::unlock_tokens_by_vesting_index(
					&sender,
					native_id,
					native_asset_vesting_index,
					vesting_native_asset_unlock_some_amount_or_all,
				)?;

			let (lp_amount, other_asset_amount) = Self::do_mint_liquidity(
				&sender,
				pool_info,
				native_id,
				unlocked_amount,
				max_other_asset_amount,
				false,
			)?;

			T::Vesting::lock_tokens(
				&sender,
				pool_id,
				lp_amount,
				Some(vesting_starting_block),
				vesting_ending_block_as_balance,
			)?;

			let lp_supply = T::Currency::total_issuance(pool_id);
			Self::deposit_event(Event::LiquidityMinted {
				who: sender,
				pool_id,
				amounts_provided: (unlocked_amount, other_asset_amount),
				lp_token: pool_id,
				lp_token_minted: lp_amount,
				total_supply: lp_supply,
			});

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(
			T::WeightInfo::mint_liquidity_using_vesting_native_tokens_xyk().max(
			T::WeightInfo::mint_liquidity_using_vesting_native_tokens_sswap()
		))]
		pub fn mint_liquidity_using_vesting_native_tokens(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			native_asset_vesting_amount: T::Balance,
			max_other_asset_amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let pool_info = Self::get_pool_info(pool_id)?;
			// check assets id, foundation has no veto
			ensure!(
				!T::NontransferableTokens::contains(&pool_info.pool.0) &&
					!T::NontransferableTokens::contains(&pool_info.pool.1),
				Error::<T>::NontransferableToken
			);
			Self::check_assets_allowed(pool_info.pool)?;

			let native_id = T::NativeCurrencyId::get();
			ensure!(
				native_id == pool_info.pool.0 || native_id == pool_info.pool.1,
				Error::<T>::NotPairedWithNativeAsset
			);

			ensure!(T::Rewards::native_rewards_enabled(pool_id), Error::<T>::NotAPromotedPool);

			let (vesting_starting_block, vesting_ending_block_as_balance) =
				T::Vesting::unlock_tokens(&sender, native_id, native_asset_vesting_amount)?;

			let (lp_amount, other_asset_amount) = Self::do_mint_liquidity(
				&sender,
				pool_info,
				native_id,
				native_asset_vesting_amount,
				max_other_asset_amount,
				false,
			)?;

			T::Vesting::lock_tokens(
				&sender,
				pool_id,
				lp_amount,
				Some(vesting_starting_block),
				vesting_ending_block_as_balance,
			)?;

			let lp_supply = T::Currency::total_issuance(pool_id);
			Self::deposit_event(Event::LiquidityMinted {
				who: sender,
				pool_id,
				amounts_provided: (native_asset_vesting_amount, other_asset_amount),
				lp_token: pool_id,
				lp_token_minted: lp_amount,
				total_supply: lp_supply,
			});

			Ok(())
		}

		/// Allows you to remove liquidity by providing the `lp_burn_amount` tokens that will be
		/// burned in the process. The usage of `min_first_asset_amount`/`min_second_asset_amount`
		/// controls the min amount of returned tokens.
		#[pallet::call_index(5)]
		#[pallet::weight(
			T::WeightInfo::burn_liquidity_xyk().max(
			T::WeightInfo::burn_liquidity_sswap()
		))]
		pub fn burn_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			liquidity_burn_amount: T::Balance,
			min_first_asset_amount: T::Balance,
			min_second_asset_amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let pool_info = Self::get_pool_info(pool_id)?;
			// check assets id, or the foundation has a veto
			ensure!(
				(!T::NontransferableTokens::contains(&pool_info.pool.0) &&
					!T::NontransferableTokens::contains(&pool_info.pool.1)) ||
					T::FoundationAccountsProvider::get().contains(&sender),
				Error::<T>::NontransferableToken
			);
			Self::check_assets_allowed(pool_info.pool)?;

			let amounts = match pool_info.kind {
				PoolKind::Xyk => {
					let amounts = T::Xyk::burn_liquidity(
						sender.clone(),
						pool_info.pool.0,
						pool_info.pool.1,
						liquidity_burn_amount,
					)?;
					ensure!(
						amounts.0 >= min_first_asset_amount,
						Error::<T>::InsufficientOutputAmount
					);
					ensure!(
						amounts.1 >= min_second_asset_amount,
						Error::<T>::InsufficientOutputAmount
					);
					amounts
				},
				PoolKind::StableSwap => {
					// deactivate liquidity if low balance
					let balance = T::Currency::available_balance(pool_id, &sender);
					let deactivate = liquidity_burn_amount.saturating_sub(balance);
					// noop on zero amount
					T::Rewards::deactivate_liquidity(sender.clone(), pool_id, deactivate)?;

					let amounts = T::StableSwap::remove_liquidity(
						&sender,
						pool_id,
						liquidity_burn_amount,
						(min_first_asset_amount, min_second_asset_amount),
					)?;
					amounts
				},
			};

			let lp_supply = T::Currency::total_issuance(pool_id);
			Self::deposit_event(Event::LiquidityBurned {
				who: sender,
				pool_id,
				amounts,
				burned_amount: liquidity_burn_amount,
				total_supply: lp_supply,
			});

			Ok(())
		}

		/// Executes a multiswap asset in a series of swap asset atomic swaps.
		///
		/// Multiswaps must fee lock instead of paying transaction fees.
		/// For a single atomic swap, both `asset_amount_in` and `min_amount_out` are considered to allow free execution without locks.
		///
		/// # Args:
		/// - `swap_token_list` - This list of tokens is the route of the atomic swaps, starting with the asset sold and ends with the asset finally bought
		/// - `asset_id_in`: The id of the asset sold
		/// - `asset_amount_in`: The amount of the asset sold
		/// - `asset_id_out`: The id of the asset received
		/// - `min_amount_out` - The minimum amount of requested asset that must be bought in order to not fail on slippage, use RPC calls to calc expected value
		// This call is part of the fee lock mechanism, which allows free execution on success
		// in case of an error & no native asset to cover fees, a fixed % is subtracted from input swap asset to avoid DOS attacks
		// `OnChargeTransaction` impl should check whether the sender has funds to cover such fee
		// or consider transaction invalid
		#[pallet::call_index(6)]
		#[pallet::weight(
			T::WeightInfo::multiswap_asset_xyk(swap_pool_list.len() as u32).max(
			T::WeightInfo::multiswap_asset_sswap(swap_pool_list.len() as u32)
		))]
		pub fn multiswap_asset(
			origin: OriginFor<T>,
			swap_pool_list: Vec<PoolIdOf<T>>,
			asset_id_in: T::CurrencyId,
			asset_amount_in: T::Balance,
			asset_id_out: T::CurrencyId,
			min_amount_out: T::Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// ensure maintenance mode
			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::TradingBlockedByMaintenanceMode
			);

			let (pools, path) = Self::get_valid_path(&swap_pool_list, asset_id_in, asset_id_out)?;

			let swaps =
				Self::do_swaps(&sender, pools, path.clone(), asset_amount_in, min_amount_out)?;

			Self::deposit_event(Event::AssetsSwapped { who: sender.clone(), swaps });

			// total swaps inc

			Ok(Pays::No.into())
		}

		/// Executes a multiswap asset in a series of swap asset atomic swaps.
		/// The precise output amount is provided instead.
		///
		/// Multiswaps must fee lock instead of paying transaction fees.
		/// For a single atomic swap, both `asset_amount_out` and `max_amount_in` are considered to allow free execution without locks.
		///
		/// # Args:
		/// - `swap_token_list` - This list of tokens is the route of the atomic swaps, starting with the asset sold and ends with the asset finally bought
		/// - `asset_id_out`: The id of the asset received
		/// - `asset_amount_out`: The amount of the asset received
		/// - `asset_id_in`: The id of the asset sold
		/// - `max_amount_in` - The maximum amount of sold asset in order to not fail on slippage, use RPC calls to calc expected value
		// This call is part of the fee lock mechanism, which allows free execution on success
		// in case of an error & no native asset to cover fees, a fixed % is subtracted from input swap asset to avoid DOS attacks
		// `OnChargeTransaction` impl should check whether the sender has funds to cover such fee
		// or consider transaction invalid
		#[pallet::call_index(7)]
		#[pallet::weight(
			T::WeightInfo::multiswap_asset_buy_xyk(swap_pool_list.len() as u32).max(
			T::WeightInfo::multiswap_asset_buy_sswap(swap_pool_list.len() as u32)
		))]
		pub fn multiswap_asset_buy(
			origin: OriginFor<T>,
			swap_pool_list: Vec<PoolIdOf<T>>,
			asset_id_out: T::CurrencyId,
			asset_amount_out: T::Balance,
			asset_id_in: T::CurrencyId,
			max_amount_in: T::Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// ensure maintenance mode
			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::TradingBlockedByMaintenanceMode
			);

			let (pools, path) = Self::get_valid_path(&swap_pool_list, asset_id_in, asset_id_out)?;
			// calc input amount
			let mut id = asset_id_out;
			let mut amount_in = asset_amount_out;
			for (pool, swap) in pools.iter().rev().zip(path.iter().rev()) {
				amount_in = Self::calculate_buy_price(pool.pool_id, id, amount_in)
					.ok_or(Error::<T>::ExcesiveInputAmount)?;
				id = if id == swap.0 { swap.1 } else { swap.0 };
			}

			ensure!(amount_in < max_amount_in, Error::<T>::ExcesiveInputAmount);

			let swaps = Self::do_swaps(&sender, pools, path.clone(), amount_in, asset_amount_out)?;

			Self::deposit_event(Event::AssetsSwapped { who: sender.clone(), swaps });

			// total swaps inc

			Ok(Pays::No.into())
		}
	}

	impl<T: Config> Pallet<T> {
		// impl for runtime apis, rather do the composition here with traits then in runtime with pallets
		pub fn calculate_sell_price(
			pool_id: T::CurrencyId,
			sell_asset_id: T::CurrencyId,
			sell_amount: T::Balance,
		) -> Option<T::Balance> {
			let pool_info = Self::get_pool_info(pool_id).ok()?;
			let (_, other) = pool_info.same_and_other(sell_asset_id)?;
			match pool_info.kind {
				PoolKind::Xyk => T::Xyk::get_dy(pool_id, sell_asset_id, other, sell_amount),
				PoolKind::StableSwap =>
					T::StableSwap::get_dy(pool_id, sell_asset_id, other, sell_amount),
			}
		}

		pub fn calculate_sell_price_with_impact(
			pool_id: T::CurrencyId,
			sell_asset_id: T::CurrencyId,
			sell_amount: T::Balance,
		) -> Option<(T::Balance, T::Balance)> {
			let pool_info = Self::get_pool_info(pool_id).ok()?;
			let (_, other) = pool_info.same_and_other(sell_asset_id)?;
			match pool_info.kind {
				PoolKind::Xyk =>
					T::Xyk::get_dy_with_impact(pool_id, sell_asset_id, other, sell_amount),
				PoolKind::StableSwap =>
					T::StableSwap::get_dy_with_impact(pool_id, sell_asset_id, other, sell_amount),
			}
		}

		pub fn calculate_buy_price(
			pool_id: T::CurrencyId,
			bought_asset_id: T::CurrencyId,
			buy_amount: T::Balance,
		) -> Option<T::Balance> {
			let pool_info = Self::get_pool_info(pool_id).ok()?;
			let (_, other) = pool_info.same_and_other(bought_asset_id)?;
			match pool_info.kind {
				PoolKind::Xyk => T::Xyk::get_dx(pool_id, other, bought_asset_id, buy_amount),
				PoolKind::StableSwap =>
					T::StableSwap::get_dx(pool_id, other, bought_asset_id, buy_amount),
			}
		}

		pub fn calculate_buy_price_with_impact(
			pool_id: T::CurrencyId,
			bought_asset_id: T::CurrencyId,
			buy_amount: T::Balance,
		) -> Option<(T::Balance, T::Balance)> {
			let pool_info = Self::get_pool_info(pool_id).ok()?;
			let (_, other) = pool_info.same_and_other(bought_asset_id)?;
			match pool_info.kind {
				PoolKind::Xyk =>
					T::Xyk::get_dx_with_impact(pool_id, other, bought_asset_id, buy_amount),
				PoolKind::StableSwap =>
					T::StableSwap::get_dx_with_impact(pool_id, other, bought_asset_id, buy_amount),
			}
		}

		pub fn get_burn_amount(
			pool_id: T::CurrencyId,
			lp_burn_amount: T::Balance,
		) -> Option<(T::Balance, T::Balance)> {
			T::Xyk::get_burn_amounts(pool_id, lp_burn_amount)
				.or_else(|| T::StableSwap::get_burn_amounts(pool_id, lp_burn_amount))
		}

		pub fn get_pools_for_trading() -> Vec<T::CurrencyId> {
			let mut assets = vec![];
			if let Some(pools) = T::Xyk::get_non_empty_pools() {
				assets.extend(pools.iter());
			}
			if let Some(pools) = T::StableSwap::get_non_empty_pools() {
				assets.extend(pools.iter());
			}
			assets
		}

		pub fn get_pools(pool_id: Option<T::CurrencyId>) -> Vec<(PoolInfoOf<T>, BalancePairOf<T>)> {
			let pool_ids = if pool_id.is_some() {
				vec![pool_id.unwrap()]
			} else {
				Self::get_pools_for_trading()
			};
			let mut pools = vec![];
			for id in pool_ids.into_iter() {
				if let Some(info) = Self::get_pool_info(id).ok() {
					let balances = match info.kind {
						PoolKind::Xyk => T::Xyk::get_pool_reserves(info.pool_id),
						PoolKind::StableSwap => T::StableSwap::get_pool_reserves(info.pool_id),
					};
					pools.push((info, balances.unwrap_or_default()))
				}
			}
			pools
		}

		pub fn calculate_expected_amount_for_minting(
			pool_id: PoolIdOf<T>,
			asset_id: T::CurrencyId,
			amount: T::Balance,
		) -> Option<T::Balance> {
			let pool_info = Self::get_pool_info(pool_id).ok()?;
			match pool_info.kind {
				PoolKind::Xyk => T::Xyk::expected_amount_for_minting(pool_id, asset_id, amount),
				PoolKind::StableSwap => Some(amount),
			}
		}

		pub fn calculate_expected_lp_minted(
			pool_id: PoolIdOf<T>,
			amounts: BalancePairOf<T>,
		) -> Option<T::Balance> {
			let pool_info = Self::get_pool_info(pool_id).ok()?;
			match pool_info.kind {
				PoolKind::Xyk => T::Xyk::get_mint_amount(pool_id, amounts),
				PoolKind::StableSwap => T::StableSwap::get_mint_amount(pool_id, amounts),
			}
		}

		fn get_pool_info(pool_id: PoolIdOf<T>) -> Result<PoolInfoOf<T>, Error<T>> {
			if let Some(pool) = T::Xyk::get_pool_info(pool_id) {
				return Ok(PoolInfo { pool_id, kind: PoolKind::Xyk, pool })
			}
			if let Some(pool) = T::StableSwap::get_pool_info(pool_id) {
				return Ok(PoolInfo { pool_id, kind: PoolKind::StableSwap, pool })
			}

			return Err(Error::<T>::NoSuchPool);
		}

		fn check_assets_allowed(assets: AssetPairOf<T>) -> Result<(), Error<T>> {
			ensure!(
				!T::DisabledTokens::contains(&assets.0) && !T::DisabledTokens::contains(&assets.1),
				Error::<T>::FunctionNotAvailableForThisToken
			);
			Ok(())
		}

		fn get_valid_path(
			swap_pool_list: &Vec<PoolIdOf<T>>,
			asset_in: T::CurrencyId,
			asset_out: T::CurrencyId,
		) -> Result<(Vec<PoolInfoOf<T>>, Vec<AssetPairOf<T>>), Error<T>> {
			// at least one swap
			ensure!(swap_pool_list.len() > 0, Error::<T>::NoSuchPool);

			// check pools repetition
			let mut dedup = swap_pool_list.clone();
			dedup.sort();
			dedup.dedup();
			ensure!(dedup.len() == swap_pool_list.len(), Error::<T>::MultiSwapSamePool);

			let mut path: Vec<AssetPairOf<T>> = vec![];
			let mut pools: Vec<PoolInfoOf<T>> = vec![];
			for &pool_id in swap_pool_list.iter() {
				let pool_info = Self::get_pool_info(pool_id)?;
				pools.push(pool_info.clone());
				// function not available for tokens
				Self::check_assets_allowed(pool_info.pool)?;

				// check pools' asset connection
				// first is asset_id_in, last is asset_id_out
				let prev_asset_id = if let Some(&last) = path.last() { last.1 } else { asset_in };

				let pool = pool_info
					.same_and_other(prev_asset_id)
					.ok_or(Error::<T>::MultiSwapPathInvalid)?;
				path.push(pool);
			}

			ensure!(
				path.last().is_some_and(|&l| l.1 == asset_out),
				Error::<T>::MultiSwapPathInvalid
			);

			Ok((pools, path))
		}

		fn do_mint_liquidity(
			sender: &T::AccountId,
			pool_info: PoolInfoOf<T>,
			asset_id: T::CurrencyId,
			amount: T::Balance,
			max_amount: T::Balance,
			activate: bool,
		) -> Result<(T::Balance, T::Balance), DispatchError> {
			let (asset_with_amount, asset_other) =
				pool_info.same_and_other(asset_id).ok_or(Error::<T>::MultiSwapPathInvalid)?;

			let amounts = match pool_info.kind {
				PoolKind::Xyk => {
					let (_, lp_amount, second_asset_withdrawn) = T::Xyk::mint_liquidity(
						sender.clone(),
						asset_with_amount,
						asset_other,
						amount,
						max_amount,
						activate,
					)?;
					(lp_amount, second_asset_withdrawn)
				},
				PoolKind::StableSwap => {
					// use 1:1 rate for amounts
					let lp_amount = T::StableSwap::add_liquidity(
						&sender,
						pool_info.pool_id,
						(amount, amount),
						Zero::zero(),
					)?;
					if activate && T::Rewards::native_rewards_enabled(pool_info.pool_id) {
						T::Rewards::activate_liquidity(
							sender.clone(),
							pool_info.pool_id,
							amount,
							Some(ActivateKind::AvailableBalance),
						)?;
					}
					(lp_amount, amount)
				},
			};

			Ok(amounts)
		}

		fn do_swaps(
			sender: &T::AccountId,
			pools: Vec<PoolInfoOf<T>>,
			path: Vec<AssetPairOf<T>>,
			amount_in: T::Balance,
			min_amount_out: T::Balance,
		) -> Result<Vec<AtomicSwapOf<T>>, DispatchError> {
			let mut swaps: Vec<AtomicSwapOf<T>> = vec![];
			let mut amount_out = amount_in;
			for (pool, swap) in pools.iter().zip(path.into_iter()) {
				// check input asset id, or the foundation has a veto
				ensure!(
					!T::NontransferableTokens::contains(&swap.0) ||
						T::ArbitrageBot::contains(sender),
					Error::<T>::NontransferableToken
				);

				let amount_in = amount_out;
				amount_out = match pool.kind {
					PoolKind::StableSwap => {
						let SwapResult { amount_out, treasury_fee, bnb_fee, .. } =
							T::StableSwap::swap(
								sender,
								pool.pool_id,
								swap.0,
								swap.1,
								amount_in,
								Zero::zero(),
							)?;

						T::Xyk::settle_treasury_and_burn(swap.1, bnb_fee, treasury_fee)?;

						amount_out
					},
					PoolKind::Xyk => T::Xyk::sell_asset(
						sender.clone(),
						swap.0,
						swap.1,
						amount_in,
						Zero::zero(),
						true,
					)?,
				};

				swaps.push(AtomicSwap {
					pool_id: pool.pool_id,
					kind: pool.kind.clone(),
					asset_in: swap.0,
					asset_out: swap.1,
					amount_in,
					amount_out,
				});
			}

			ensure!(amount_out >= min_amount_out, Error::<T>::InsufficientOutputAmount);

			Ok(swaps)
		}
	}
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct RpcAssetMetadata<TokenId> {
	pub token_id: TokenId,
	pub decimals: u32,
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct RpcPoolInfo<TokenId, Balance> {
	pub pool_id: TokenId,
	pub kind: PoolKind,
	pub lp_token_id: TokenId,
	pub assets: Vec<TokenId>,
	pub reserves: Vec<Balance>,
}

sp_api::decl_runtime_apis! {
	 /// This runtime api allows people to query the size of the liquidity pools
	 /// and quote prices for swaps.
	pub trait MarketRuntimeApi<Balance, AssetId>
	where
		Balance: Codec + MaybeDisplay + MaybeFromStr,
		AssetId: Codec + MaybeDisplay + MaybeFromStr,
	{
		fn calculate_sell_price(
			pool_id: AssetId,
			sell_asset_id: AssetId,
			sell_amount: Balance
		) -> Option<Balance>;

		fn calculate_sell_price_with_impact(
			pool_id: AssetId,
			sell_asset_id: AssetId,
			sell_amount: Balance
		) -> Option<(Balance, Balance)>;

		fn calculate_buy_price(
			pool_id: AssetId,
			buy_asset_id: AssetId,
			buy_amount: Balance
		) -> Option<Balance>;

		fn calculate_buy_price_with_impact(
			pool_id: AssetId,
			buy_asset_id: AssetId,
			buy_amount: Balance
		) -> Option<(Balance, Balance)>;

		fn get_burn_amount(
			pool_id: AssetId,
			lp_burn_amount: Balance,
		) -> Option<(Balance, Balance)>;

		fn calculate_expected_amount_for_minting(
			pool_id: AssetId,
			asset_id: AssetId,
			amount: Balance,
		) -> Option<Balance>;

		fn calculate_expected_lp_minted(
			pool_id: AssetId,
			amounts: (Balance, Balance),
		) -> Option<Balance>;

// 		fn get_max_instant_burn_amount(
// 			user: AccountId,
// 			liquidity_asset_id: AssetId,
// 		) -> Balance;

// 		fn get_max_instant_unreserve_amount(
// 			user: AccountId,
// 			liquidity_asset_id: AssetId,
// 		) -> Balance;

// 		fn calculate_rewards_amount(
// 			user: AccountId,
// 			liquidity_asset_id: AssetId,
// 		) -> Balance;

// 		fn calculate_balanced_sell_amount(
// 			total_amount: Balance,
// 			reserve_amount: Balance,
// 		) -> Balance;

// 		fn is_buy_asset_lock_free(
// 			path: sp_std::vec::Vec<AssetId>,
// 			input_amount: Balance,
// 		) -> Option<bool>;

// 		fn is_sell_asset_lock_free(
// 			path: sp_std::vec::Vec<AssetId>,
// 			input_amount: Balance,
// 		) -> Option<bool>;

		fn get_tradeable_tokens() -> Vec<RpcAssetMetadata<AssetId>>;

		fn get_pools_for_trading() -> Vec<AssetId>;

		fn get_pools(pool_id: Option<AssetId>) -> Vec<RpcPoolInfo<AssetId, Balance>>;

// 		fn get_total_number_of_swaps() -> u128;
	}
}
