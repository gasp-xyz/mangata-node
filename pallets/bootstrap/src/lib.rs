// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;

use frame_support::{
	codec::{Decode, Encode},
	traits::{ExistenceRequirement, Get},
	transactional, PalletId,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
use mangata_primitives::{Balance, TokenId};
use mp_bootstrap::PoolCreateApi;
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use pallet_vesting_mangata::MultiTokenVestingLocks;
use scale_info::TypeInfo;
use sp_arithmetic::helpers_128bit::multiply_by_rational;
use sp_core::U256;
use sp_runtime::traits::{AccountIdConversion, CheckedAdd};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

mod benchmarking;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::WeightInfo;

pub use pallet::*;
const PALLET_ID: PalletId = PalletId(*b"bootstrp");

use core::fmt::Debug;

#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: "bootstrap",
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

pub type BlockNrAsBalance = Balance;

pub enum ProvisionKind {
	Regular,
	Vested(BlockNrAsBalance),
}

#[frame_support::pallet]
pub mod pallet {

	use frame_benchmarking::Zero;

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let phase = Phase::<T>::get(); // R:1
			if phase == BootstrapPhase::Finished {
				return T::DbWeight::get().reads(1)
			}

			if let Some((start, whitelist_length, public_length, _)) = BootstrapSchedule::<T>::get()
			{
				// R:1
				// NOTE: arythmetics protected by invariant check in Bootstrap::start_ido
				let whitelist_start = start;
				let public_start = start + whitelist_length.into();
				let finished = start + whitelist_length.into() + public_length.into();

				if n >= finished {
					Phase::<T>::put(BootstrapPhase::Finished); // 1 WRINTE
					log!(info, "bootstrap event finished");
					let (mga_valuation, ksm_valuation) = Valuations::<T>::get();
					// XykFunctionsTrait R: 11 W:12
					// PoolCreateApi::pool_create R:2  +
					// ---------------------------------
					// R: 13 W 12
					if let Some((liq_asset_id, issuance)) = T::PoolCreateApi::pool_create(
						Self::vault_address(),
						T::KSMTokenId::get(),
						ksm_valuation,
						T::MGATokenId::get(),
						mga_valuation,
					) {
						MintedLiquidity::<T>::put((liq_asset_id, issuance)); // W:1
					} else {
						log!(error, "cannot create pool!");
					}
					// TODO: include cost of pool_create call
					T::DbWeight::get().reads_writes(15, 13)
				} else if n >= public_start {
					Phase::<T>::put(BootstrapPhase::Public);
					log!(info, "starting public phase");
					T::DbWeight::get().reads_writes(2, 1)
				} else if n >= whitelist_start {
					log!(info, "starting whitelist phase");
					Phase::<T>::put(BootstrapPhase::Whitelist);
					T::DbWeight::get().reads_writes(2, 1)
				} else {
					T::DbWeight::get().reads(2)
				}
			} else {
				T::DbWeight::get().reads(2)
			}
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// tokens
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;

		type PoolCreateApi: PoolCreateApi<AccountId = Self::AccountId>;

		#[pallet::constant]
		type MGATokenId: Get<TokenId>;

		#[pallet::constant]
		type KSMTokenId: Get<TokenId>;

		type VestingProvider: MultiTokenVestingLocks<Self::AccountId>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn provisions)]
	pub type Provisions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, TokenId, Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vested_provisions)]
	pub type VestedProvisions<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		TokenId,
		(Balance, BlockNrAsBalance),
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn whitelisted_accounts)]
	pub type WhitelistedAccount<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn phase)]
	pub type Phase<T: Config> = StorageValue<_, BootstrapPhase, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn valuations)]
	pub type Valuations<T: Config> = StorageValue<_, (Balance, Balance), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn config)]
	pub type BootstrapSchedule<T: Config> =
		StorageValue<_, (T::BlockNumber, u32, u32, (u128, u128)), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn minted_liquidity)]
	pub type MintedLiquidity<T: Config> = StorageValue<_, (TokenId, Balance), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn claimed_rewards)]
	pub type ClaimedRewards<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, TokenId, Balance, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// provisions vested/locked tokens into the boostrstrap
		#[pallet::weight(T::WeightInfo::provision_vested())]
		#[transactional]
		pub fn provision_vested(
			origin: OriginFor<T>,
			token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let vesting_ending_block_as_balance: Balance =
				T::VestingProvider::unlock_tokens(&sender, token_id.into(), amount.into())
					.map_err(|_| Error::<T>::NotEnoughVestedAssets)?
					.into();
			Self::do_provision(
				sender,
				token_id,
				amount,
				ProvisionKind::Vested(vesting_ending_block_as_balance),
			)?;
			Self::deposit_event(Event::Provisioned(token_id, amount));
			Ok(().into())
		}

		/// provisions non-vested/non-locked tokens into the boostrstrap
		#[pallet::weight(T::WeightInfo::provision())]
		#[transactional]
		pub fn provision(
			origin: OriginFor<T>,
			token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_provision(sender, token_id, amount, ProvisionKind::Regular)?;
			Self::deposit_event(Event::Provisioned(token_id, amount));
			Ok(().into())
		}

		/// provides a list of whitelisted accounts, list is extended with every call
		#[pallet::weight(T::DbWeight::get().writes(1) * (accounts.len() as u64))]
		#[transactional]
		pub fn whitelist_accounts(
			origin: OriginFor<T>,
			accounts: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			for account in accounts {
				WhitelistedAccount::<T>::insert(&account, ());
			}
			Self::deposit_event(Event::AccountsWhitelisted);
			Ok(().into())
		}

		/// schedules start of an bootstrap event where
		/// - ido_start - number of block when bootstrap event should be started
		/// - whitelist_phase_length - length of whitelist phase in blocks.
		/// - public_phase_length - length of public phase in blocks
		/// - max_ksm_to_mgx_ratio - maximum tokens ratio that is held by the pallet during bootstrap event
		///
		/// max_ksm_to_mgx_ratio[0]       KSM VALUATION
		/// ----------------------- < ---------------------
		/// max_ksm_to_mgx_ratio[1]       MGX VALUATION
		///
		/// bootstrap phases:
		/// - BeforeStart - blocks 0..ido_start
		/// - WhitelistPhase - blocks ido_start..(ido_start + whitelist_phase_length)
		/// - PublicPhase - blocks (ido_start + whitelist_phase_length)..(ido_start + whitelist_phase_length  + public_phase_lenght)
		#[pallet::weight(T::WeightInfo::start_ido())]
		#[transactional]
		pub fn start_ido(
			origin: OriginFor<T>,
			ido_start: T::BlockNumber,
			whitelist_phase_length: u32,
			public_phase_lenght: u32,
			max_ksm_to_mgx_ratio: (u128, u128),
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(Phase::<T>::get() == BootstrapPhase::BeforeStart, Error::<T>::AlreadyStarted);

			ensure!(
				ido_start > frame_system::Pallet::<T>::block_number(),
				Error::<T>::BootstrapStartInThePast
			);

			ensure!(max_ksm_to_mgx_ratio.0 != 0, Error::<T>::WrongRatio);

			ensure!(max_ksm_to_mgx_ratio.1 != 0, Error::<T>::WrongRatio);

			ensure!(whitelist_phase_length > 0, Error::<T>::PhaseLengthCannotBeZero);

			ensure!(public_phase_lenght > 0, Error::<T>::PhaseLengthCannotBeZero);

			ensure!(
				ido_start
					.checked_add(&whitelist_phase_length.into())
					.and_then(|whiteslist_start| whiteslist_start
						.checked_add(&public_phase_lenght.into()))
					.is_some(),
				Error::<T>::MathOverflow
			);

			ensure!(
				ido_start.checked_add(&whitelist_phase_length.into()).is_some(),
				Error::<T>::MathOverflow
			);

			ensure!(
				!T::PoolCreateApi::pool_exists(T::KSMTokenId::get(), T::MGATokenId::get()),
				Error::<T>::PoolAlreadyExists
			);

			BootstrapSchedule::<T>::put((
				ido_start,
				whitelist_phase_length,
				public_phase_lenght,
				max_ksm_to_mgx_ratio,
			));

			Ok(().into())
		}

		/// claim liquidity tokens from pool created as a result of bootstrap event finish
		#[pallet::weight(T::WeightInfo::claim_rewards())]
		#[transactional]
		pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Self::phase() == BootstrapPhase::Finished, Error::<T>::NotFinishedYet);

			let (liq_token_id, _) = Self::minted_liquidity();

			ensure!(
				ClaimedRewards::<T>::get(&sender, T::KSMTokenId::get()) == 0 &&
					ClaimedRewards::<T>::get(&sender, T::MGATokenId::get()) == 0,
				Error::<T>::NothingToClaim
			);

			let (ksm_rewards, ksm_rewards_vested, ksm_lock) =
				Self::calculate_rewards(&sender, &T::KSMTokenId::get())?;
			let (mga_rewards, mga_rewards_vested, mga_lock) =
				Self::calculate_rewards(&sender, &T::MGATokenId::get())?;

			let total_rewards_claimed = mga_rewards
				.checked_add(mga_rewards_vested)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_add(ksm_rewards)
				.ok_or(Error::<T>::MathOverflow)?
				.checked_add(ksm_rewards_vested)
				.ok_or(Error::<T>::MathOverflow)?;

			if total_rewards_claimed.is_zero() {
				return Err(Error::<T>::NothingToClaim.into())
			}

			Self::claim_rewards_from_single_currency(
				&sender,
				&T::MGATokenId::get(),
				mga_rewards,
				mga_rewards_vested,
				mga_lock,
			)?;
			log!(
				info,
				"MGA rewards (non-vested, vested, total) = ({}, {}, {})",
				mga_rewards,
				mga_rewards_vested,
				mga_rewards + mga_rewards_vested
			);

			Self::claim_rewards_from_single_currency(
				&sender,
				&T::KSMTokenId::get(),
				ksm_rewards,
				ksm_rewards_vested,
				ksm_lock,
			)?;
			log!(
				info,
				"KSM rewards (non-vested, vested, total) = ({}, {}, {})",
				ksm_rewards,
				ksm_rewards_vested,
				ksm_rewards + ksm_rewards_vested
			);

			Self::deposit_event(Event::RewardsClaimed(liq_token_id, total_rewards_claimed));

			Ok(().into())
		}
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// Only MGA & KSM can be used for provisions
		UnsupportedTokenId,
		/// Not enought funds for provisio
		NotEnoughAssets,
		/// Not enought funds for provisio (vested)
		NotEnoughVestedAssets,
		/// Math problem
		MathOverflow,
		/// User cannot participate at this moment
		Unauthorized,
		/// Bootstrap cant be scheduled in past
		BootstrapStartInThePast,
		/// Bootstarap phases cannot lasts 0 blocks
		PhaseLengthCannotBeZero,
		/// Bootstrate event already started
		AlreadyStarted,
		/// Valuation ratio exceeded
		ValuationRatio,
		/// First provision must be in MGA/MGX
		FirstProvisionInMga,
		/// Bootstraped pool already exists
		PoolAlreadyExists,
		/// Cannot claim rewards before bootstrap finish
		NotFinishedYet,
		/// no rewards to claim
		NothingToClaim,
		/// no rewards to claim
		WrongRatio,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Funds provisioned
		Provisioned(TokenId, Balance),
		/// Funds provisioned using vested tokens
		VestedProvisioned(TokenId, Balance),
		/// Rewards claimed
		RewardsClaimed(TokenId, Balance),
		/// account whitelisted
		AccountsWhitelisted,
	}
}

#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Debug)]
pub enum BootstrapPhase {
	BeforeStart,
	Whitelist,
	Public,
	Finished,
}

impl Default for BootstrapPhase {
	fn default() -> Self {
		BootstrapPhase::BeforeStart
	}
}

impl<T: Config> Pallet<T> {
	fn is_whitelisted(account: &T::AccountId) -> bool {
		WhitelistedAccount::<T>::try_get(account).is_ok()
	}

	fn vault_address() -> T::AccountId {
		PALLET_ID.into_account()
	}

	fn claim_rewards_from_single_currency(
		who: &T::AccountId,
		provision_token_id: &TokenId,
		rewards: Balance,
		rewards_vested: Balance,
		lock: BlockNrAsBalance,
	) -> DispatchResult {
		let (liq_token_id, _) = Self::minted_liquidity();
		let total_rewards = rewards.checked_add(rewards_vested).ok_or(Error::<T>::MathOverflow)?;
		if total_rewards == 0 {
			return Ok(().into())
		}

		T::Currency::transfer(
			liq_token_id.into(),
			&Self::vault_address(),
			who,
			total_rewards.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		ClaimedRewards::<T>::try_mutate(who, provision_token_id, |rewards| {
			if let Some(val) = rewards.checked_add(total_rewards) {
				*rewards = val;
				Ok(())
			} else {
				Err(Error::<T>::MathOverflow)
			}
		})?;

		if rewards_vested > 0 {
			T::VestingProvider::lock_tokens(
				&who,
				liq_token_id.into(),
				rewards_vested.into(),
				lock.into(),
			)?;
		}

		Ok(().into())
	}

	///
	/// assures that
	///
	/// actual_nominator              expected_nominator
	/// --------------------   <=     ------------------
	/// actual_denominator            expected_denominator
	///
	/// actual_nominator * expected_denominator     expected_nominator * actual_denominator
	/// ---------------------------------------- <= ----------------------------------------
	/// actual_denominator * expected_denominator    expected_denominator * actual_nominator
	fn is_ratio_kept(ratio_nominator: u128, ratio_denominator: u128) -> bool {
		let (mga_valuation, ksm_valuation) = Valuations::<T>::get();
		let left = U256::from(ksm_valuation) * U256::from(ratio_denominator);
		let right = U256::from(ratio_nominator) * U256::from(mga_valuation);
		left <= right
	}

	pub fn do_provision(
		sender: T::AccountId,
		token_id: TokenId,
		amount: Balance,
		is_vested: ProvisionKind,
	) -> DispatchResult {
		let is_ksm = token_id == T::KSMTokenId::get();
		let is_mga = token_id == T::MGATokenId::get();
		let is_public_phase = Phase::<T>::get() == BootstrapPhase::Public;
		let is_whitelist_phase = Phase::<T>::get() == BootstrapPhase::Whitelist;
		let am_i_whitelisted = Self::is_whitelisted(&sender);

		ensure!(is_ksm || is_mga, Error::<T>::UnsupportedTokenId);

		ensure!(
			is_public_phase || (is_whitelist_phase && (am_i_whitelisted || is_mga)),
			Error::<T>::Unauthorized
		);

		let schedule = BootstrapSchedule::<T>::get();
		ensure!(schedule.is_some(), Error::<T>::Unauthorized);
		let (_, _, _, (ratio_nominator, ratio_denominator)) = schedule.unwrap();

		<T as Config>::Currency::transfer(
			token_id.into(),
			&sender,
			&Self::vault_address(),
			amount.into(),
			ExistenceRequirement::KeepAlive,
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		match is_vested {
			ProvisionKind::Regular => {
				ensure!(
					Provisions::<T>::try_mutate(sender.clone(), token_id, |provision| {
						if let Some(val) = provision.checked_add(amount) {
							*provision = val;
							Ok(())
						} else {
							Err(())
						}
					})
					.is_ok(),
					Error::<T>::MathOverflow
				);
			},
			ProvisionKind::Vested(nr) => {
				ensure!(
					VestedProvisions::<T>::try_mutate(
						sender.clone(),
						token_id,
						|(provision, block_nr)| {
							if let Some(val) = provision.checked_add(amount) {
								*provision = val;
								*block_nr = (*block_nr).max(nr);
								Ok(())
							} else {
								Err(())
							}
						}
					)
					.is_ok(),
					Error::<T>::MathOverflow
				);
			},
		}

		let (pre_mga_valuation, _) = Valuations::<T>::get();
		ensure!(
			token_id != T::KSMTokenId::get() || pre_mga_valuation != 0,
			Error::<T>::FirstProvisionInMga
		);

		ensure!(
			Valuations::<T>::try_mutate(|(mga_valuation, ksm_valuation)| -> Result<(), ()> {
				if token_id == T::MGATokenId::get() {
					*mga_valuation = mga_valuation.checked_add(amount).ok_or(())?;
				}
				if token_id == T::KSMTokenId::get() {
					*ksm_valuation = ksm_valuation.checked_add(amount).ok_or(())?;
				}
				Ok(())
			})
			.is_ok(),
			Error::<T>::MathOverflow
		);

		if token_id == T::KSMTokenId::get() {
			ensure!(
				Self::is_ratio_kept(ratio_nominator, ratio_denominator),
				Error::<T>::ValuationRatio
			);
		}
		Ok(().into())
	}

	fn get_valuation(token_id: &TokenId) -> Balance {
		if *token_id == T::KSMTokenId::get() {
			Self::valuations().1
		} else if *token_id == T::MGATokenId::get() {
			Self::valuations().0
		} else {
			0
		}
	}

	fn calculate_rewards(
		who: &T::AccountId,
		token_id: &TokenId,
	) -> Result<(Balance, Balance, BlockNrAsBalance), Error<T>> {
		let valuation = Self::get_valuation(token_id);
		let provision = Self::provisions(who, token_id);
		let (vested_provision, lock) = Self::vested_provisions(who, token_id);
		let (_, liquidity) = Self::minted_liquidity();
		let rewards = multiply_by_rational(liquidity / 2, provision, valuation)
			.map_err(|_| Error::<T>::MathOverflow)?;
		let vested_rewards = multiply_by_rational(liquidity / 2, vested_provision, valuation)
			.map_err(|_| Error::<T>::MathOverflow)?;
		Ok((rewards, vested_rewards, lock))
	}
}
