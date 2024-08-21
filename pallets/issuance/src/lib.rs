// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, Get, Imbalance, MultiTokenVestingSchedule},
};
use frame_system::pallet_prelude::*;
use mangata_support::traits::{ComputeIssuance, GetIssuance, LiquidityMiningApi};
use orml_tokens::MultiTokenCurrencyExtended;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{CheckedAdd, CheckedDiv, CheckedSub, One, Saturating, Zero},
	Perbill, Percent, RuntimeDebug,
};
use sp_std::{convert::TryInto, prelude::*};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct IssuanceInfo<Balance> {
	// Max number of MGA to target
	pub cap: Balance,
	// MGA created at token generation event
	// We aasume that there is only one tge
	pub issuance_at_init: Balance,
	// Time between which the total issuance of MGA grows to cap, as number of blocks
	pub linear_issuance_blocks: u32,
	// The split of issuance assgined to liquidity_mining
	pub liquidity_mining_split: Perbill,
	// The split of issuance assgined to staking
	pub staking_split: Perbill,
	// The total mga allocated to crowdloan rewards
	pub total_crowdloan_allocation: Balance,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct TgeInfo<AccountId, Balance> {
	// The tge target
	pub who: AccountId,
	// Amount distributed at tge
	pub amount: Balance,
}

/// Weight functions needed for pallet_xyk.
pub trait WeightInfo {
	fn init_issuance_config() -> Weight;
	fn finalize_tge() -> Weight;
	fn execute_tge(x: u32) -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	fn init_issuance_config() -> Weight {
		Weight::from_parts(50_642_000, 0)
	}
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	fn finalize_tge() -> Weight {
		Weight::from_parts(50_830_000, 0)
	}
	// Storage: Vesting Vesting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn execute_tge(l: u32) -> Weight {
		Weight::from_parts(52_151_000, 0)
			// Standard Error: 1_000
			.saturating_add((Weight::from_parts(130_000, 0)).saturating_mul(l as u64))
	}
}

pub use pallet::*;

type BalanceOf<T> =
	<<T as Config>::Tokens as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance;

type CurrencyIdOf<T> = <<T as Config>::Tokens as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// MGA currency to check total_issuance
		type NativeCurrencyId: Get<CurrencyIdOf<Self>>;
		/// Tokens
		type Tokens: MultiTokenCurrencyExtended<Self::AccountId>;
		/// Number of blocks per session/round
		#[pallet::constant]
		type BlocksPerRound: Get<u32>;
		/// Number of sessions to store issuance history for
		#[pallet::constant]
		type HistoryLimit: Get<u32>;
		#[pallet::constant]
		/// The account id that holds the liquidity mining issuance
		type LiquidityMiningIssuanceVault: Get<Self::AccountId>;
		#[pallet::constant]
		/// The account id that holds the staking issuance
		type StakingIssuanceVault: Get<Self::AccountId>;
		#[pallet::constant]
		/// The total mga allocated for crowdloans
		type TotalCrowdloanAllocation: Get<BalanceOf<Self>>;
		#[pallet::constant]
		/// The maximum amount of Mangata tokens
		type ImmediateTGEReleasePercent: Get<Percent>;
		#[pallet::constant]
		/// The maximum amount of Mangata tokens
		type IssuanceCap: Get<BalanceOf<Self>>;
		#[pallet::constant]
		/// The number of blocks the issuance is linear
		type LinearIssuanceBlocks: Get<u32>;
		#[pallet::constant]
		/// The split of issuance for liquidity mining rewards
		type LiquidityMiningSplit: Get<Perbill>;
		#[pallet::constant]
		/// The split of issuance for staking rewards
		type StakingSplit: Get<Perbill>;
		#[pallet::constant]
		/// The number of blocks the tge tokens vest for
		type TGEReleasePeriod: Get<u32>;
		#[pallet::constant]
		/// The block at which the tge tokens begin to vest
		type TGEReleaseBegin: Get<u32>;
		/// The vesting pallet
		type VestingProvider: MultiTokenVestingSchedule<
			Self::AccountId,
			Currency = Self::Tokens,
			Moment = BlockNumberFor<Self>,
		>;
		type WeightInfo: WeightInfo;
		type LiquidityMiningApi: LiquidityMiningApi<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_issuance_config)]
	pub type IssuanceConfigStore<T: Config> =
		StorageValue<_, IssuanceInfo<BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_tge_total)]
	pub type TGETotal<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_tge_finalized)]
	pub type IsTGEFinalized<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_session_issuance)]
	pub type SessionIssuance<T: Config> =
		StorageMap<_, Twox64Concat, u32, Option<(BalanceOf<T>, BalanceOf<T>)>, ValueQuery>;

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// The issuance config has already been initialized
		IssuanceConfigAlreadyInitialized,
		/// The issuance config has not been initialized
		IssuanceConfigNotInitialized,
		/// TGE must be finalized before issuance config is inti
		TGENotFinalized,
		/// The TGE is already finalized
		TGEIsAlreadyFinalized,
		/// The issuance config is invalid
		IssuanceConfigInvalid,
		/// An underflow or an overflow has occured
		MathError,
		/// unknown pool
		UnknownPool,
	}

	// XYK extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::init_issuance_config())]
		pub fn init_issuance_config(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			Self::do_init_issuance_config()
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::finalize_tge())]
		pub fn finalize_tge(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(!IsTGEFinalized::<T>::get(), Error::<T>::TGEIsAlreadyFinalized);

			IsTGEFinalized::<T>::put(true);

			Pallet::<T>::deposit_event(Event::TGEFinalized);

			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::execute_tge(tge_infos.len() as u32))]
		pub fn execute_tge(
			origin: OriginFor<T>,
			tge_infos: Vec<TgeInfo<T::AccountId, BalanceOf<T>>>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(!IsTGEFinalized::<T>::get(), Error::<T>::TGEIsAlreadyFinalized);

			ensure!(!T::TGEReleasePeriod::get().is_zero(), Error::<T>::MathError);

			let lock_percent: Percent = Percent::from_percent(100)
				.checked_sub(&T::ImmediateTGEReleasePercent::get())
				.ok_or(Error::<T>::MathError)?;

			for tge_info in tge_infos {
				let locked: BalanceOf<T> = (lock_percent * tge_info.amount).max(One::one());
				let per_block: BalanceOf<T> =
					(locked / T::TGEReleasePeriod::get().into()).max(One::one());

				if T::VestingProvider::can_add_vesting_schedule(
					&tge_info.who,
					locked,
					per_block,
					T::TGEReleaseBegin::get().into(),
					T::NativeCurrencyId::get(),
				)
				.is_ok()
				{
					let imb = T::Tokens::deposit_creating(
						T::NativeCurrencyId::get().into(),
						&tge_info.who.clone(),
						tge_info.amount,
					);

					if !tge_info.amount.is_zero() && imb.peek().is_zero() {
						Pallet::<T>::deposit_event(Event::TGEInstanceFailed(tge_info));
					} else {
						let _ = T::VestingProvider::add_vesting_schedule(
							&tge_info.who,
							locked,
							per_block,
							T::TGEReleaseBegin::get().into(),
							T::NativeCurrencyId::get().into(),
						);
						TGETotal::<T>::mutate(|v| *v = v.saturating_add(tge_info.amount));
						Pallet::<T>::deposit_event(Event::TGEInstanceSucceeded(tge_info));
					}
				} else {
					Pallet::<T>::deposit_event(Event::TGEInstanceFailed(tge_info));
				}
			}

			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Issuance for upcoming session issued
		SessionIssuanceIssued(u32, BalanceOf<T>, BalanceOf<T>),
		/// Issuance for upcoming session calculated and recorded
		SessionIssuanceRecorded(u32, BalanceOf<T>, BalanceOf<T>),
		/// Issuance configuration has been finalized
		IssuanceConfigInitialized(IssuanceInfo<BalanceOf<T>>),
		/// TGE has been finalized
		TGEFinalized,
		/// A TGE instance has failed
		TGEInstanceFailed(TgeInfo<T::AccountId, BalanceOf<T>>),
		/// A TGE instance has succeeded
		TGEInstanceSucceeded(TgeInfo<T::AccountId, BalanceOf<T>>),
	}
}

impl<T: Config> ComputeIssuance for Pallet<T> {
	fn initialize() {
		IsTGEFinalized::<T>::put(true);
		Self::do_init_issuance_config().unwrap();
	}

	fn compute_issuance(n: u32) {
		let _ = Pallet::<T>::calculate_and_store_round_issuance(n);
		let _ = Pallet::<T>::clear_round_issuance_history(n);
	}
}

pub trait ProvideTotalCrowdloanRewardAllocation<T: Config> {
	fn get_total_crowdloan_allocation() -> Option<BalanceOf<T>>;
}

impl<T: Config> ProvideTotalCrowdloanRewardAllocation<T> for Pallet<T> {
	fn get_total_crowdloan_allocation() -> Option<BalanceOf<T>> {
		IssuanceConfigStore::<T>::get()
			.map(|issuance_config| issuance_config.total_crowdloan_allocation)
	}
}

impl<T: Config> GetIssuance<BalanceOf<T>> for Pallet<T> {
	fn get_all_issuance(n: u32) -> Option<(BalanceOf<T>, BalanceOf<T>)> {
		SessionIssuance::<T>::get(n)
	}
	fn get_liquidity_mining_issuance(n: u32) -> Option<BalanceOf<T>> {
		SessionIssuance::<T>::get(n).map(|(x, _)| x)
	}
	fn get_staking_issuance(n: u32) -> Option<BalanceOf<T>> {
		SessionIssuance::<T>::get(n).map(|(_, x)| x)
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_init_issuance_config() -> DispatchResultWithPostInfo {
		ensure!(
			IssuanceConfigStore::<T>::get().is_none(),
			Error::<T>::IssuanceConfigAlreadyInitialized
		);
		ensure!(IsTGEFinalized::<T>::get(), Error::<T>::TGENotFinalized);

		let issuance_config: IssuanceInfo<BalanceOf<T>> = IssuanceInfo {
			cap: T::IssuanceCap::get(),
			issuance_at_init: T::Tokens::total_issuance(T::NativeCurrencyId::get().into()),
			linear_issuance_blocks: T::LinearIssuanceBlocks::get(),
			liquidity_mining_split: T::LiquidityMiningSplit::get(),
			staking_split: T::StakingSplit::get(),
			total_crowdloan_allocation: T::TotalCrowdloanAllocation::get(),
		};

		Pallet::<T>::build_issuance_config(issuance_config.clone())?;

		Pallet::<T>::deposit_event(Event::IssuanceConfigInitialized(issuance_config));

		Ok(().into())
	}

	pub fn build_issuance_config(issuance_config: IssuanceInfo<BalanceOf<T>>) -> DispatchResult {
		ensure!(
			issuance_config
				.liquidity_mining_split
				.checked_add(&issuance_config.staking_split)
				.ok_or(Error::<T>::IssuanceConfigInvalid)? ==
				Perbill::from_percent(100),
			Error::<T>::IssuanceConfigInvalid
		);
		ensure!(
			issuance_config.cap >=
				issuance_config
					.issuance_at_init
					.checked_add(&issuance_config.total_crowdloan_allocation)
					.ok_or(Error::<T>::IssuanceConfigInvalid)?,
			Error::<T>::IssuanceConfigInvalid
		);
		ensure!(
			issuance_config.linear_issuance_blocks != u32::zero(),
			Error::<T>::IssuanceConfigInvalid
		);
		ensure!(
			issuance_config.linear_issuance_blocks > T::BlocksPerRound::get(),
			Error::<T>::IssuanceConfigInvalid
		);
		ensure!(T::BlocksPerRound::get() != u32::zero(), Error::<T>::IssuanceConfigInvalid);
		IssuanceConfigStore::<T>::put(issuance_config);
		Ok(())
	}

	pub fn calculate_and_store_round_issuance(current_round: u32) -> DispatchResult {
		let issuance_config =
			IssuanceConfigStore::<T>::get().ok_or(Error::<T>::IssuanceConfigNotInitialized)?;
		let to_be_issued: BalanceOf<T> = issuance_config
			.cap
			.checked_sub(&issuance_config.issuance_at_init)
			.ok_or(Error::<T>::MathError)?
			.checked_sub(&issuance_config.total_crowdloan_allocation)
			.ok_or(Error::<T>::MathError)?;
		let linear_issuance_sessions: u32 = issuance_config
			.linear_issuance_blocks
			.checked_div(T::BlocksPerRound::get())
			.ok_or(Error::<T>::MathError)?;
		let linear_issuance_per_session = to_be_issued
			.checked_div(&linear_issuance_sessions.into())
			.ok_or(Error::<T>::MathError)?;

		let current_round_issuance: BalanceOf<T>;
		// We do not want issuance to overshoot
		// Sessions begin from 0 and linear_issuance_sessions is the total number of linear sessions including 0
		// So we stop before that
		if current_round < linear_issuance_sessions {
			current_round_issuance = linear_issuance_per_session;
		} else {
			let current_mga_total_issuance: BalanceOf<T> =
				T::Tokens::total_issuance(T::NativeCurrencyId::get().into());
			if issuance_config.cap > current_mga_total_issuance {
				// TODO
				// Here we assume that the crowdloan ends before linear issuance period ends
				// We could get the amount that the crowdloan rewards still need to mint and account for that
				// But that largely depends on on how the next crowdloan will be implemented
				// Not very useful for the first crowdloan, as we know that it will end before linear issuance period ends and we check for this
				current_round_issuance = linear_issuance_per_session.min(
					issuance_config
						.cap
						.checked_sub(&current_mga_total_issuance)
						.ok_or(Error::<T>::MathError)?,
				)
			} else {
				current_round_issuance = Zero::zero();
			}
		}

		let liquidity_mining_issuance =
			issuance_config.liquidity_mining_split * current_round_issuance;

		let staking_issuance = issuance_config.staking_split * current_round_issuance;

		T::LiquidityMiningApi::distribute_rewards(liquidity_mining_issuance);

		{
			let liquidity_mining_issuance_issued = T::Tokens::deposit_creating(
				T::NativeCurrencyId::get().into(),
				&T::LiquidityMiningIssuanceVault::get(),
				liquidity_mining_issuance,
			);
			let staking_issuance_issued = T::Tokens::deposit_creating(
				T::NativeCurrencyId::get().into(),
				&T::StakingIssuanceVault::get(),
				staking_issuance,
			);
			Self::deposit_event(Event::SessionIssuanceIssued(
				current_round,
				liquidity_mining_issuance_issued.peek(),
				staking_issuance_issued.peek(),
			));
		}

		SessionIssuance::<T>::insert(
			current_round,
			Some((liquidity_mining_issuance, staking_issuance)),
		);

		Pallet::<T>::deposit_event(Event::SessionIssuanceRecorded(
			current_round,
			liquidity_mining_issuance,
			staking_issuance,
		));

		Ok(())
	}

	pub fn clear_round_issuance_history(current_round: u32) -> DispatchResult {
		if current_round >= T::HistoryLimit::get() {
			SessionIssuance::<T>::remove(current_round - T::HistoryLimit::get());
		}
		Ok(())
	}
}
