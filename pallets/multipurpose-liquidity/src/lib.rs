#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{
		tokens::currency::{MultiTokenCurrency, MultiTokenVestingLocks},
		Get, StorageVersion, WithdrawReasons,
	},
	transactional,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use mangata_support::traits::{
	ActivationReservesProviderTrait, StakingReservesProviderTrait, XykFunctionsTrait,
};
use mangata_types::multipurpose_liquidity::{ActivateKind, BondKind};
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use sp_runtime::traits::{Bounded, CheckedAdd, CheckedSub, Saturating, Zero};
use sp_std::{convert::TryInto, prelude::*};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

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

pub use pallet::*;

type BalanceOf<T> =
	<<T as Config>::Tokens as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance;

type CurrencyIdOf<T> = <<T as Config>::Tokens as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MaxRelocks: Get<u32>;
		type Tokens: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;
		type NativeCurrencyId: Get<CurrencyIdOf<Self>>;
		type VestingProvider: MultiTokenVestingLocks<
			Self::AccountId,
			Currency = Self::Tokens,
			Moment = BlockNumberFor<Self>,
		>;
		type Xyk: XykFunctionsTrait<Self::AccountId, BalanceOf<Self>, CurrencyIdOf<Self>>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// The token is not a liquidity token
		NotALiquidityToken,
		/// The limit on the maximum number of relocks was exceeded
		RelockCountLimitExceeded,
		/// Provided index for relock is out of bounds
		RelockInstanceIndexOOB,
		/// Not enough unspend reserves
		NotEnoughUnspentReserves,
		/// Not enough tokens
		NotEnoughTokens,
		/// Math error
		MathError,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VestingTokensReserved(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>),
		TokensRelockedFromReserve(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
	}

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(
		Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Default,
	)]
	pub struct ReserveStatusInfo<Balance> {
		pub staked_unactivated_reserves: Balance,
		pub activated_unstaked_reserves: Balance,
		pub staked_and_activated_reserves: Balance,
		pub unspent_reserves: Balance,
		pub relock_amount: Balance,
	}

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(
		Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Default,
	)]
	pub struct RelockStatusInfo<Balance, BlockNumber> {
		pub amount: Balance,
		pub starting_block: BlockNumber,
		pub ending_block_as_balance: Balance,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_reserve_status)]
	pub type ReserveStatus<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		CurrencyIdOf<T>,
		ReserveStatusInfo<BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_relock_status)]
	pub type RelockStatus<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		CurrencyIdOf<T>,
		BoundedVec<RelockStatusInfo<BalanceOf<T>, BlockNumberFor<T>>, T::MaxRelocks>,
		ValueQuery,
	>;

	// MPL extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[transactional]
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::reserve_vesting_liquidity_tokens())]
		/// Migrates vested liquidity tokens from Vested pallet to MPL. Information about
		/// unlock schedule is preserved, so whenever one decides to move tokens back to
		/// Vested pallet tokens can be unlocked.
		pub fn reserve_vesting_liquidity_tokens_by_vesting_index(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			liquidity_token_vesting_index: u32,
			liquidity_token_unlock_some_amount_or_all: Option<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(T::Xyk::is_liquidity_token(liquidity_token_id), Error::<T>::NotALiquidityToken);
			Self::do_reserve_tokens_by_vesting_index(
				sender,
				liquidity_token_id,
				liquidity_token_vesting_index,
				liquidity_token_unlock_some_amount_or_all,
			)
		}

		#[transactional]
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::reserve_vesting_liquidity_tokens())]
		/// Migrates vested MGX from Vested pallet to MPL. Information about unlock schedule is
		/// preserved, so whenever one decides to move tokens back to Vested pallet tokens can be
		/// unlocked.
		pub fn reserve_vesting_native_tokens_by_vesting_index(
			origin: OriginFor<T>,
			liquidity_token_vesting_index: u32,
			liquidity_token_unlock_some_amount_or_all: Option<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			Self::do_reserve_tokens_by_vesting_index(
				sender,
				T::NativeCurrencyId::get(),
				liquidity_token_vesting_index,
				liquidity_token_unlock_some_amount_or_all,
			)
		}

		#[transactional]
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::reserve_vesting_liquidity_tokens())]
		// This extrinsic has to be transactional
		pub fn reserve_vesting_liquidity_tokens(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			liquidity_token_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(T::Xyk::is_liquidity_token(liquidity_token_id), Error::<T>::NotALiquidityToken);

			let (vesting_starting_block, vesting_ending_block_as_balance): (
				BlockNumberFor<T>,
				BalanceOf<T>,
			) = T::VestingProvider::unlock_tokens(&sender, liquidity_token_id, liquidity_token_amount)
				.map(|x| (x.0, x.1))?;

			let mut reserve_status = Pallet::<T>::get_reserve_status(&sender, liquidity_token_id);

			reserve_status.relock_amount = reserve_status
				.relock_amount
				.checked_add(&liquidity_token_amount)
				.ok_or(Error::<T>::MathError)?;
			reserve_status.unspent_reserves = reserve_status
				.unspent_reserves
				.checked_add(&liquidity_token_amount)
				.ok_or(Error::<T>::MathError)?;

			ReserveStatus::<T>::insert(&sender, liquidity_token_id, reserve_status);

			RelockStatus::<T>::try_append(
				&sender,
				liquidity_token_id,
				RelockStatusInfo {
					amount: liquidity_token_amount,
					starting_block: vesting_starting_block,
					ending_block_as_balance: vesting_ending_block_as_balance,
				},
			)
			.map_err(|_| Error::<T>::RelockCountLimitExceeded)?;

			T::Tokens::reserve(liquidity_token_id.into(), &sender, liquidity_token_amount)?;

			Pallet::<T>::deposit_event(Event::VestingTokensReserved(
				sender,
				liquidity_token_id,
				liquidity_token_amount,
			));

			Ok(().into())
		}

		#[transactional]
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::unreserve_and_relock_instance())]
		// This extrinsic has to be transactional
		pub fn unreserve_and_relock_instance(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			relock_instance_index: u32,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let relock_instances: Vec<RelockStatusInfo<BalanceOf<T>, BlockNumberFor<T>>> =
				Self::get_relock_status(&sender, liquidity_token_id).into();

			let selected_relock_instance: RelockStatusInfo<BalanceOf<T>, BlockNumberFor<T>> =
				relock_instances
					.get(relock_instance_index as usize)
					.ok_or(Error::<T>::RelockInstanceIndexOOB)?
					.clone();

			let updated_relock_instances: BoundedVec<
				RelockStatusInfo<BalanceOf<T>, BlockNumberFor<T>>,
				T::MaxRelocks,
			> = relock_instances
				.into_iter()
				.enumerate()
				.filter_map(move |(index, relock_instance)| {
					if index == relock_instance_index as usize {
						None
					} else {
						Some(relock_instance)
					}
				})
				.collect::<Vec<_>>()
				.try_into()
				.map_err(|_| Error::<T>::RelockCountLimitExceeded)?;

			let mut reserve_status = Pallet::<T>::get_reserve_status(&sender, liquidity_token_id);

			reserve_status.relock_amount = reserve_status
				.relock_amount
				.checked_sub(&selected_relock_instance.amount)
				.ok_or(Error::<T>::MathError)?;
			reserve_status.unspent_reserves = reserve_status
				.unspent_reserves
				.checked_sub(&selected_relock_instance.amount)
				.ok_or(Error::<T>::NotEnoughUnspentReserves)?;

			ensure!(
				T::Tokens::unreserve(
					liquidity_token_id.into(),
					&sender,
					selected_relock_instance.amount
				)
				.is_zero(),
				Error::<T>::MathError
			);

			T::VestingProvider::lock_tokens(
				&sender,
				liquidity_token_id.into(),
				selected_relock_instance.amount,
				Some(selected_relock_instance.starting_block.into()),
				selected_relock_instance.ending_block_as_balance,
			)?;

			ReserveStatus::<T>::insert(&sender, liquidity_token_id, reserve_status);

			RelockStatus::<T>::insert(&sender, liquidity_token_id, updated_relock_instances);

			Pallet::<T>::deposit_event(Event::TokensRelockedFromReserve(
				sender,
				liquidity_token_id,
				selected_relock_instance.amount,
				selected_relock_instance.ending_block_as_balance,
			));

			Ok(().into())
		}
	}
}

impl<T: Config> StakingReservesProviderTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>
	for Pallet<T>
{
	fn can_bond(
		token_id: CurrencyIdOf<T>,
		account_id: &T::AccountId,
		amount: BalanceOf<T>,
		use_balance_from: Option<BondKind>,
	) -> bool {
		let reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		let use_balance_from = use_balance_from.unwrap_or(BondKind::AvailableBalance);

		match use_balance_from {
			BondKind::AvailableBalance => {
				T::Tokens::ensure_can_withdraw(
					token_id.into(),
					&account_id,
					amount,
					WithdrawReasons::all(),
					Default::default(),
				)
				.is_ok() && reserve_status
					.staked_unactivated_reserves
					.checked_add(&amount)
					.is_some()
			},
			BondKind::ActivatedUnstakedReserves => {
				reserve_status.activated_unstaked_reserves.checked_sub(&amount).is_some()
					&& reserve_status.staked_and_activated_reserves.checked_add(&amount).is_some()
			},
			BondKind::UnspentReserves => {
				reserve_status.unspent_reserves.checked_sub(&amount).is_some()
					&& reserve_status.staked_unactivated_reserves.checked_add(&amount).is_some()
			},
		}
	}

	fn bond(
		token_id: CurrencyIdOf<T>,
		account_id: &T::AccountId,
		amount: BalanceOf<T>,
		use_balance_from: Option<BondKind>,
	) -> DispatchResult {
		let mut reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		let use_balance_from = use_balance_from.unwrap_or(BondKind::AvailableBalance);

		match use_balance_from {
			BondKind::AvailableBalance => {
				reserve_status.staked_unactivated_reserves = reserve_status
					.staked_unactivated_reserves
					.checked_add(&amount)
					.ok_or(Error::<T>::MathError)?;
				T::Tokens::reserve(token_id.into(), &account_id, amount)?;
			},
			BondKind::ActivatedUnstakedReserves => {
				reserve_status.activated_unstaked_reserves = reserve_status
					.activated_unstaked_reserves
					.checked_sub(&amount)
					.ok_or(Error::<T>::NotEnoughTokens)?;
				reserve_status.staked_and_activated_reserves = reserve_status
					.staked_and_activated_reserves
					.checked_add(&amount)
					.ok_or(Error::<T>::MathError)?;
			},
			BondKind::UnspentReserves => {
				reserve_status.unspent_reserves = reserve_status
					.unspent_reserves
					.checked_sub(&amount)
					.ok_or(Error::<T>::NotEnoughTokens)?;
				reserve_status.staked_unactivated_reserves = reserve_status
					.staked_unactivated_reserves
					.checked_add(&amount)
					.ok_or(Error::<T>::MathError)?;
			},
		}

		ReserveStatus::<T>::insert(account_id, token_id, reserve_status);
		Ok(())
	}

	fn unbond(
		token_id: CurrencyIdOf<T>,
		account_id: &T::AccountId,
		amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		// From staked_unactivated_reserves goes to either free balance or unspent reserves depending on relock_amount

		// From staked_and_activated_reserves goes to activated always.

		let mut reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);
		let mut working_amount = amount;
		let mut unreserve_amount = working_amount.min(reserve_status.staked_unactivated_reserves);

		working_amount = working_amount.saturating_sub(unreserve_amount);
		reserve_status.staked_unactivated_reserves =
			reserve_status.staked_unactivated_reserves.saturating_sub(unreserve_amount);

		let mut move_reserve = working_amount.min(reserve_status.staked_and_activated_reserves);
		// This is just to prevent overflow.
		move_reserve = BalanceOf::<T>::max_value()
			.saturating_sub(reserve_status.activated_unstaked_reserves)
			.min(move_reserve);
		reserve_status.staked_and_activated_reserves =
			reserve_status.staked_and_activated_reserves.saturating_sub(move_reserve);
		reserve_status.activated_unstaked_reserves =
			reserve_status.activated_unstaked_reserves.saturating_add(move_reserve);
		working_amount = working_amount.saturating_sub(move_reserve);

		// Now we will attempt to unreserve the amount on the basis of the relock_amount
		let total_remaining_reserve = reserve_status
			.staked_unactivated_reserves
			.saturating_add(reserve_status.activated_unstaked_reserves)
			.saturating_add(reserve_status.staked_and_activated_reserves)
			.saturating_add(reserve_status.unspent_reserves);

		let mut add_to_unspent =
			reserve_status.relock_amount.saturating_sub(total_remaining_reserve);
		if add_to_unspent > unreserve_amount {
			log::warn!(
				"Unbond witnessed prior state of relock_amount being higher than mpl reserves {:?} {:?}",
				add_to_unspent,
				unreserve_amount
			);
		}
		add_to_unspent = add_to_unspent.min(unreserve_amount);
		unreserve_amount = unreserve_amount.saturating_sub(add_to_unspent);
		reserve_status.unspent_reserves =
			reserve_status.unspent_reserves.saturating_add(add_to_unspent);

		let unreserve_result: BalanceOf<T> =
			T::Tokens::unreserve(token_id.into(), account_id, unreserve_amount);

		if !unreserve_result.is_zero() {
			log::warn!("Unbond resulted in non-zero unreserve_result {:?}", unreserve_result);
		}

		if !working_amount.is_zero() {
			log::warn!("Unbond resulted in left-over amount {:?}", working_amount);
		}

		ReserveStatus::<T>::insert(account_id, token_id, reserve_status);
		working_amount.saturating_add(unreserve_result)
	}
}

impl<T: Config> ActivationReservesProviderTrait<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>
	for Pallet<T>
{
	fn get_max_instant_unreserve_amount(
		token_id: CurrencyIdOf<T>,
		account_id: &T::AccountId,
	) -> BalanceOf<T> {
		let reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		let total_remaining_reserve = reserve_status
			.staked_unactivated_reserves
			.saturating_add(reserve_status.staked_and_activated_reserves)
			.saturating_add(reserve_status.unspent_reserves);

		let amount_held_back_by_relock =
			reserve_status.relock_amount.saturating_sub(total_remaining_reserve);

		// We assume here that the actual unreserve will ofcoures go fine returning 0.
		reserve_status
			.activated_unstaked_reserves
			.saturating_sub(amount_held_back_by_relock)
	}

	fn can_activate(
		token_id: CurrencyIdOf<T>,
		account_id: &T::AccountId,
		amount: BalanceOf<T>,
		use_balance_from: Option<ActivateKind>,
	) -> bool {
		let reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		let use_balance_from = use_balance_from.unwrap_or(ActivateKind::AvailableBalance);

		match use_balance_from {
			ActivateKind::AvailableBalance => {
				T::Tokens::ensure_can_withdraw(
					token_id.into(),
					&account_id,
					amount,
					WithdrawReasons::all(),
					Default::default(),
				)
				.is_ok() && reserve_status
					.activated_unstaked_reserves
					.checked_add(&amount)
					.is_some()
			},
			ActivateKind::StakedUnactivatedReserves => {
				reserve_status.staked_unactivated_reserves.checked_sub(&amount).is_some()
					&& reserve_status.staked_and_activated_reserves.checked_add(&amount).is_some()
			},
			ActivateKind::UnspentReserves => {
				reserve_status.unspent_reserves.checked_sub(&amount).is_some()
					&& reserve_status.activated_unstaked_reserves.checked_add(&amount).is_some()
			},
		}
	}

	fn activate(
		token_id: CurrencyIdOf<T>,
		account_id: &T::AccountId,
		amount: BalanceOf<T>,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		let mut reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);

		let use_balance_from = use_balance_from.unwrap_or(ActivateKind::AvailableBalance);

		match use_balance_from {
			ActivateKind::AvailableBalance => {
				reserve_status.activated_unstaked_reserves = reserve_status
					.activated_unstaked_reserves
					.checked_add(&amount)
					.ok_or(Error::<T>::MathError)?;
				T::Tokens::reserve(token_id.into(), &account_id, amount)?;
			},
			ActivateKind::StakedUnactivatedReserves => {
				reserve_status.staked_unactivated_reserves = reserve_status
					.staked_unactivated_reserves
					.checked_sub(&amount)
					.ok_or(Error::<T>::NotEnoughTokens)?;
				reserve_status.staked_and_activated_reserves = reserve_status
					.staked_and_activated_reserves
					.checked_add(&amount)
					.ok_or(Error::<T>::MathError)?;
			},
			ActivateKind::UnspentReserves => {
				reserve_status.unspent_reserves = reserve_status
					.unspent_reserves
					.checked_sub(&amount)
					.ok_or(Error::<T>::NotEnoughTokens)?;
				reserve_status.activated_unstaked_reserves = reserve_status
					.activated_unstaked_reserves
					.checked_add(&amount)
					.ok_or(Error::<T>::MathError)?;
			},
		}

		ReserveStatus::<T>::insert(account_id, token_id, reserve_status);
		Ok(())
	}

	fn deactivate(
		token_id: CurrencyIdOf<T>,
		account_id: &T::AccountId,
		amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		// From ActivatedUnstakedReserves goes to either free balance or unspent reserves depending on relock_amount
		// From staked_and_activated_reserves goes to staked always.

		let mut reserve_status = Pallet::<T>::get_reserve_status(account_id, token_id);
		let mut working_amount = amount;
		let mut unreserve_amount = working_amount.min(reserve_status.activated_unstaked_reserves);

		working_amount = working_amount.saturating_sub(unreserve_amount);
		reserve_status.activated_unstaked_reserves =
			reserve_status.activated_unstaked_reserves.saturating_sub(unreserve_amount);

		let mut move_reserve = working_amount.min(reserve_status.staked_and_activated_reserves);
		// This is just to prevent overflow.
		move_reserve = BalanceOf::<T>::max_value()
			.saturating_sub(reserve_status.staked_unactivated_reserves)
			.min(move_reserve);
		reserve_status.staked_and_activated_reserves =
			reserve_status.staked_and_activated_reserves.saturating_sub(move_reserve);
		reserve_status.staked_unactivated_reserves =
			reserve_status.staked_unactivated_reserves.saturating_add(move_reserve);
		working_amount = working_amount.saturating_sub(move_reserve);

		// Now we will attempt to unreserve the amount on the basis of the relock_amount
		let total_remaining_reserve = reserve_status
			.staked_unactivated_reserves
			.saturating_add(reserve_status.activated_unstaked_reserves)
			.saturating_add(reserve_status.staked_and_activated_reserves)
			.saturating_add(reserve_status.unspent_reserves);

		let mut add_to_unspent =
			reserve_status.relock_amount.saturating_sub(total_remaining_reserve);
		if add_to_unspent > unreserve_amount {
			log::warn!(
				"Unbond witnessed prior state of relock_amount being higher than mpl reserves {:?} {:?}",
				add_to_unspent,
				unreserve_amount
			);
		}
		add_to_unspent = add_to_unspent.min(unreserve_amount);
		unreserve_amount = unreserve_amount.saturating_sub(add_to_unspent);
		reserve_status.unspent_reserves =
			reserve_status.unspent_reserves.saturating_add(add_to_unspent);

		let unreserve_result: BalanceOf<T> =
			T::Tokens::unreserve(token_id.into(), account_id, unreserve_amount);

		if !unreserve_result.is_zero() {
			log::warn!("Unbond resulted in non-zero unreserve_result {:?}", unreserve_result);
		}

		if !working_amount.is_zero() {
			log::warn!("Unbond resulted in left-over amount {:?}", working_amount);
		}

		ReserveStatus::<T>::insert(account_id, token_id, reserve_status);
		working_amount.saturating_add(unreserve_result)
	}
}

impl<T: Config> Pallet<T> {
	fn do_reserve_tokens_by_vesting_index(
		account: T::AccountId,
		liquidity_token_id: CurrencyIdOf<T>,
		liquidity_token_vesting_index: u32,
		liquidity_token_unlock_some_amount_or_all: Option<BalanceOf<T>>,
	) -> DispatchResultWithPostInfo {
		let (unlocked_amount, vesting_starting_block, vesting_ending_block_as_balance): (
			BalanceOf<T>,
			BlockNumberFor<T>,
			BalanceOf<T>,
		) = T::VestingProvider::unlock_tokens_by_vesting_index(
			&account,
			liquidity_token_id,
			liquidity_token_vesting_index,
			liquidity_token_unlock_some_amount_or_all,
		)
		.map(|x| (x.0, x.1, x.2))?;

		let mut reserve_status = Pallet::<T>::get_reserve_status(&account, liquidity_token_id);

		reserve_status.relock_amount = reserve_status
			.relock_amount
			.checked_add(&unlocked_amount)
			.ok_or(Error::<T>::MathError)?;
		reserve_status.unspent_reserves = reserve_status
			.unspent_reserves
			.checked_add(&unlocked_amount)
			.ok_or(Error::<T>::MathError)?;

		ReserveStatus::<T>::insert(&account, liquidity_token_id, reserve_status);

		RelockStatus::<T>::try_append(
			&account,
			liquidity_token_id,
			RelockStatusInfo {
				amount: unlocked_amount,
				starting_block: vesting_starting_block,
				ending_block_as_balance: vesting_ending_block_as_balance,
			},
		)
		.map_err(|_| Error::<T>::RelockCountLimitExceeded)?;

		T::Tokens::reserve(liquidity_token_id.into(), &account, unlocked_amount)?;

		Pallet::<T>::deposit_event(Event::VestingTokensReserved(
			account,
			liquidity_token_id,
			unlocked_amount,
		));
		Ok(().into())
	}
}
