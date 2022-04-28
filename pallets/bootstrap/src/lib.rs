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
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyExtended};
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

pub use pallet::*;
const PALLET_ID: PalletId = PalletId(*b"12345678");

use core::fmt::Debug;
use sp_runtime::traits::MaybeDisplay;

#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: "bootstrap",
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

pub trait PoolCreateApi {
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;

	fn pool_exists(first: TokenId, second: TokenId) -> bool;

	fn pool_create(
		account: Self::AccountId,
		first: TokenId,
		first_amount: Balance,
		second: TokenId,
		second_amount: Balance,
	) -> Option<(TokenId, Balance)>;
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let phase = Phase::<T>::get();
			if phase == BootstrapPhase::Finished {
				return T::DbWeight::get().reads(1)
			}

			if let Some((start, whitelist_length, public_length)) = BootstrapSchedule::<T>::get() {
				// NOTE: arythmetics protected by invariant check in Bootstrap::start_ido
				let whitelist_start = start;
				let public_start = start + whitelist_length.into();
				let finished = start + whitelist_length.into() + public_length.into();

				if n >= finished {
					Phase::<T>::put(BootstrapPhase::Finished);
					log!(info, "bootstrap event finished");
					let (mga_valuation, ksm_valuation) = Valuations::<T>::get();
					if let Some((liq_asset_id, issuance)) = T::PoolCreateApi::pool_create(
						Self::vault_address(),
						T::KSMTokenId::get(),
						ksm_valuation,
						T::MGATokenId::get(),
						mga_valuation,
					) {
						MintedLiquidity::<T>::put((liq_asset_id, issuance));
					} else {
						log!(error, "cannot create pool!");
					}
					// TODO: include cost of pool_create call
					T::DbWeight::get().reads_writes(3, 2)
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
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>;

		type PoolCreateApi: PoolCreateApi<AccountId = Self::AccountId>;

		#[pallet::constant]
		type MGATokenId: Get<TokenId>;

		#[pallet::constant]
		type KSMTokenId: Get<TokenId>;

		#[pallet::constant]
		type KsmToMgaRatioNumerator: Get<u128>;

		#[pallet::constant]
		type KsmToMgaRatioDenominator: Get<u128>;
	}

	#[pallet::storage]
	#[pallet::getter(fn provisions)]
	pub type Provisions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, TokenId, Balance, ValueQuery>;

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
		StorageValue<_, (T::BlockNumber, u32, u32), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn minted_liquidity)]
	pub type MintedLiquidity<T: Config> = StorageValue<_, (TokenId, Balance), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn claimed_rewards)]
	pub type ClaimedRewards<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, TokenId, Balance, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn provision(
			origin: OriginFor<T>,
			token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				token_id == T::KSMTokenId::get() || token_id == T::MGATokenId::get(),
				Error::<T>::UnsupportedTokenId
			);

			let ratio_nominator = T::KsmToMgaRatioNumerator::get();
			let ratio_denominator = T::KsmToMgaRatioDenominator::get();

			ensure!(
				token_id == T::MGATokenId::get() ||
					Phase::<T>::get() == BootstrapPhase::Public ||
					(Phase::<T>::get() == BootstrapPhase::Whitelist &&
						Self::is_whitelisted(&sender)),
				Error::<T>::Unauthorized
			);

			<T as Config>::Currency::transfer(
				token_id.into(),
				&sender,
				&Self::vault_address(),
				amount.into(),
				ExistenceRequirement::KeepAlive,
			)
			.or(Err(Error::<T>::NotEnoughAssets))?;

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

			log!(
				debug,
				"account={:?} successfully provisioned token={} amount={}",
				&sender,
				token_id,
				amount
			);

			Self::deposit_event(Event::Provisioned(token_id, amount));
			Ok(().into())
		}

		#[pallet::weight(0)]
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

		#[pallet::weight(10_000)]
		#[transactional]
		pub fn start_ido(
			origin: OriginFor<T>,
			ido_start: T::BlockNumber,
			whitelist_phase_length: u32,
			public_phase_lenght: u32,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(Phase::<T>::get() == BootstrapPhase::BeforeStart, Error::<T>::AlreadyStarted);

			ensure!(
				ido_start > frame_system::Pallet::<T>::block_number(),
				Error::<T>::BootstrapStartInThePast
			);

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

			BootstrapSchedule::<T>::put((ido_start, whitelist_phase_length, public_phase_lenght));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		#[transactional]
		pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Self::phase() == BootstrapPhase::Finished, Error::<T>::NotFinishedYet);

			let user_ksm_provision = Self::provisions(&sender, T::KSMTokenId::get());
			let user_mga_provision = Self::provisions(&sender, T::MGATokenId::get());

			let (liq_token_id, liquidity) = Self::minted_liquidity();
			let (total_mga_provision, total_ksm_provision) = Self::valuations();

			let ksm_rewards =
				multiply_by_rational(user_ksm_provision, liquidity / 2, total_ksm_provision)
					.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;
			let mga_rewards =
				multiply_by_rational(user_mga_provision, liquidity / 2, total_mga_provision)
					.map_err(|_| DispatchError::from(Error::<T>::MathOverflow))?;

			let ksm_claimed_rewards = ClaimedRewards::<T>::get(&sender, T::KSMTokenId::get());
			let mga_claimed_rewards = ClaimedRewards::<T>::get(&sender, T::MGATokenId::get());

			ensure!(
				ksm_rewards > ksm_claimed_rewards || mga_rewards > mga_claimed_rewards,
				Error::<T>::NothingToClaim
			);

			let mut total_rewards_claimed = 0;

			if ksm_rewards > ksm_claimed_rewards {
				let ksm_to_bo_claimed = ksm_rewards - ksm_claimed_rewards;
				Self::claim_rewards_from_single_currency(
					&sender,
					liq_token_id,
					T::KSMTokenId::get(),
					ksm_to_bo_claimed,
				)?;
				log!(debug, "Rewards from KSM provision: {}", ksm_to_bo_claimed);
				total_rewards_claimed += ksm_to_bo_claimed;
			}

			if mga_rewards > mga_claimed_rewards {
				let mga_to_bo_claimed = mga_rewards - mga_claimed_rewards;
				Self::claim_rewards_from_single_currency(
					&sender,
					liq_token_id,
					T::MGATokenId::get(),
					mga_to_bo_claimed,
				)?;
				log!(debug, "Rewards from MGA provision {}", mga_to_bo_claimed);
				total_rewards_claimed += mga_to_bo_claimed;
			}

			log!(debug, "Rewards claimed token={} amount={}", liq_token_id, total_rewards_claimed);
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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Funds provisioned
		Provisioned(TokenId, Balance),
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
		liq_token_id: TokenId,
		provision_token_id: TokenId,
		to_be_claimed: Balance,
	) -> DispatchResult {
		T::Currency::transfer(
			liq_token_id.into(),
			&Self::vault_address(),
			who,
			to_be_claimed.into(),
			ExistenceRequirement::KeepAlive,
		)?;
		ensure!(
			ClaimedRewards::<T>::try_mutate(who, provision_token_id, |rewards| {
				if let Some(val) = rewards.checked_add(to_be_claimed) {
					*rewards = val;
					Ok(())
				} else {
					Err(())
				}
			})
			.is_ok(),
			Error::<T>::MathOverflow
		);
		Ok(()).into()
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
}
