#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	storage::bounded_btree_set::BoundedBTreeSet,
	traits::{Get, StorageVersion},
	transactional,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use mangata_types::{Balance, TokenId};
use mp_traits::FeeLockTriggerTrait;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use pallet_xyk::Valuate;
use sp_arithmetic::per_things::Rounding;
use sp_runtime::helpers_128bit::multiply_by_rational_with_rounding;

use sp_runtime::{traits::Zero, Saturating};
use sp_std::{convert::TryInto, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

pub(crate) const LOG_TARGET: &'static str = "fee-lock";

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

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[codec(mel_bound(T: Config))]
	#[scale_info(skip_type_params(T))]
	pub struct FeeLockMetadataInfo<T: Config> {
		pub period_length: T::BlockNumber,
		pub fee_lock_amount: Balance,
		pub swap_value_threshold: Balance,
		pub whitelisted_tokens: BoundedBTreeSet<TokenId, T::MaxCuratedTokens>,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_fee_lock_metadata)]
	pub type FeeLockMetadata<T: Config> = StorageValue<_, FeeLockMetadataInfo<T>, OptionQuery>;

	#[derive(
		Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Default,
	)]
	pub struct AccountFeeLockDataInfo<BlockNumber: Default> {
		pub total_fee_lock_amount: Balance,
		pub last_fee_lock_block: BlockNumber,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_account_fee_lock_data)]
	pub type AccountFeeLockData<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		AccountFeeLockDataInfo<T::BlockNumber>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		FeeLockMetadataUpdated,
		FeeLockUnlocked(T::AccountId, Balance),
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// Locks were incorrectly initialized
		FeeLocksIncorrectlyInitialzed,
		/// Lock metadata is invalid
		InvalidFeeLockMetadata,
		/// Locks have not been initialzed
		FeeLocksNotInitialized,
		/// No tokens of the user are fee-locked
		NotFeeLocked,
		/// The lock cannot be unlocked yet
		CantUnlockFeeYet,
		/// The limit on the maximum curated tokens for which there is a swap threshold is exceeded
		MaxCuratedTokensLimitExceeded,
		/// An unexpected failure has occured
		UnexpectedFailure,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type MaxCuratedTokens: Get<u32>;
		type Tokens: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;
		type PoolReservesProvider: Valuate;
		#[pallet::constant]
		type NativeTokenId: Get<TokenId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub period_length: Option<T::BlockNumber>,
		pub fee_lock_amount: Option<Balance>,
		pub swap_value_threshold: Option<Balance>,
		pub whitelisted_tokens: Vec<TokenId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				period_length: Default::default(),
				fee_lock_amount: Default::default(),
				swap_value_threshold: Default::default(),
				whitelisted_tokens: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			match (self.period_length, self.fee_lock_amount, self.swap_value_threshold) {
				(Some(period), Some(amount), Some(threshold)) => {
					let mut tokens: BoundedBTreeSet<TokenId, T::MaxCuratedTokens> =
						Default::default();
					for t in self.whitelisted_tokens.iter() {
						tokens
							.try_insert(*t)
							.expect("list of tokens is <= than T::MaxCuratedTokens");
					}

					FeeLockMetadata::<T>::put(FeeLockMetadataInfo {
						period_length: period,
						fee_lock_amount: amount,
						swap_value_threshold: threshold,
						whitelisted_tokens: tokens,
					});
				},
				(None, None, None) => {},
				_ => {
					panic!("either all or non config parameters should be set");
				},
			};
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// The weight is calculated using MaxCuratedTokens so it is the worst case weight
		#[pallet::call_index(0)]
		#[transactional]
		#[pallet::weight(T::WeightInfo::update_fee_lock_metadata())]
		pub fn update_fee_lock_metadata(
			origin: OriginFor<T>,
			period_length: Option<T::BlockNumber>,
			fee_lock_amount: Option<Balance>,
			swap_value_threshold: Option<Balance>,
			should_be_whitelisted: Option<Vec<(TokenId, bool)>>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let mut fee_lock_metadata =
				Self::get_fee_lock_metadata().unwrap_or(FeeLockMetadataInfo {
					period_length: Default::default(),
					fee_lock_amount: Default::default(),
					swap_value_threshold: Default::default(),
					whitelisted_tokens: Default::default(),
				});

			fee_lock_metadata.period_length =
				period_length.unwrap_or(fee_lock_metadata.period_length);
			fee_lock_metadata.fee_lock_amount =
				fee_lock_amount.unwrap_or(fee_lock_metadata.fee_lock_amount);
			fee_lock_metadata.swap_value_threshold =
				swap_value_threshold.unwrap_or(fee_lock_metadata.swap_value_threshold);

			ensure!(
				!fee_lock_metadata.fee_lock_amount.is_zero(),
				Error::<T>::InvalidFeeLockMetadata
			);
			ensure!(!fee_lock_metadata.period_length.is_zero(), Error::<T>::InvalidFeeLockMetadata);
			ensure!(
				!fee_lock_metadata.swap_value_threshold.is_zero(),
				Error::<T>::InvalidFeeLockMetadata
			);

			if let Some(should_be_whitelisted) = should_be_whitelisted {
				for (token_id, should_be_whitelisted) in should_be_whitelisted.iter() {
					match should_be_whitelisted {
						true => {
							let _ = fee_lock_metadata
								.whitelisted_tokens
								.try_insert(*token_id)
								.map_err(|_| Error::<T>::MaxCuratedTokensLimitExceeded)?;
						},
						false => {
							let _ = fee_lock_metadata.whitelisted_tokens.remove(token_id);
						},
					}
				}
			}

			FeeLockMetadata::<T>::put(fee_lock_metadata);

			Pallet::<T>::deposit_event(Event::FeeLockMetadataUpdated);

			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[transactional]
		#[pallet::weight(T::WeightInfo::unlock_fee())]
		pub fn unlock_fee(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Ok(<Self as FeeLockTriggerTrait<T::AccountId>>::unlock_fee(&who)?.into())
		}
	}
}

impl<T: Config> FeeLockTriggerTrait<T::AccountId> for Pallet<T> {
	fn is_whitelisted(token_id: TokenId) -> bool {
		if let Some(fee_lock_metadata) = Self::get_fee_lock_metadata() {
			if T::NativeTokenId::get() == token_id {
				return true
			}
			fee_lock_metadata.whitelisted_tokens.contains(&token_id)
		} else {
			false
		}
	}

	fn get_swap_valuation_for_token(
		valuating_token_id: TokenId,
		valuating_token_amount: Balance,
	) -> Option<Balance> {
		if T::NativeTokenId::get() == valuating_token_id {
			return Some(valuating_token_amount)
		}
		let (native_token_pool_reserve, valuating_token_pool_reserve) =
			<T::PoolReservesProvider as Valuate>::get_reserves(
				T::NativeTokenId::get(),
				valuating_token_id,
			)
			.ok()?;
		if native_token_pool_reserve.is_zero() || valuating_token_pool_reserve.is_zero() {
			return None
		}
		Some(
			multiply_by_rational_with_rounding(
				valuating_token_amount,
				native_token_pool_reserve,
				valuating_token_pool_reserve,
				Rounding::Down,
			)
			.unwrap_or(Balance::max_value()),
		)
	}

	fn process_fee_lock(who: &T::AccountId) -> DispatchResult {
		let fee_lock_metadata =
			Self::get_fee_lock_metadata().ok_or(Error::<T>::FeeLocksNotInitialized)?;
		let mut account_fee_lock_data = Self::get_account_fee_lock_data(who);
		let now = <frame_system::Pallet<T>>::block_number();

		// This is cause now >= last_fee_lock_block
		ensure!(now >= account_fee_lock_data.last_fee_lock_block, Error::<T>::UnexpectedFailure);

		if now <
			account_fee_lock_data
				.last_fee_lock_block
				.saturating_add(fee_lock_metadata.period_length)
		{
			// First storage edit
			// Cannot fail beyond this point
			// Rerserve additional fee_lock_amount
			<T as pallet::Config>::Tokens::reserve(
				<T as pallet::Config>::NativeTokenId::get().into(),
				who,
				fee_lock_metadata.fee_lock_amount.into(),
			)?;

			// Insert updated account_lock_info into storage
			// This is not expected to fail
			account_fee_lock_data.total_fee_lock_amount = account_fee_lock_data
				.total_fee_lock_amount
				.saturating_add(fee_lock_metadata.fee_lock_amount);
			account_fee_lock_data.last_fee_lock_block = now;
			AccountFeeLockData::<T>::insert(who, account_fee_lock_data);
		} else {
			// We must either reserve more or unreserve
			match (fee_lock_metadata.fee_lock_amount, account_fee_lock_data.total_fee_lock_amount) {
				(x, y) if x > y => <T as pallet::Config>::Tokens::reserve(
					<T as pallet::Config>::NativeTokenId::get().into(),
					who,
					x.saturating_sub(y).into(),
				)?,
				(x, y) if x < y => {
					let unreserve_result = <T as pallet::Config>::Tokens::unreserve(
						<T as pallet::Config>::NativeTokenId::get().into(),
						who,
						y.saturating_sub(x).into(),
					);
					if !unreserve_result.is_zero() {
						log::warn!(
							"Process fee lock unreserve resulted in non-zero unreserve_result {:?}",
							unreserve_result
						);
					}
				},
				_ => {},
			}
			// Insert updated account_lock_info into storage
			// This is not expected to fail
			account_fee_lock_data.total_fee_lock_amount = fee_lock_metadata.fee_lock_amount;
			account_fee_lock_data.last_fee_lock_block = now;
			AccountFeeLockData::<T>::insert(who, account_fee_lock_data);
		}

		Ok(())
	}

	fn can_unlock_fee(who: &T::AccountId) -> DispatchResult {
		// Check if total_fee_lock_amount is non-zero
		// THEN Check is period is greater than last

		let account_fee_lock_data = Self::get_account_fee_lock_data(&who);

		ensure!(!account_fee_lock_data.total_fee_lock_amount.is_zero(), Error::<T>::NotFeeLocked);

		let fee_lock_metadata =
			Self::get_fee_lock_metadata().ok_or(Error::<T>::FeeLocksNotInitialized)?;

		let now = <frame_system::Pallet<T>>::block_number();

		ensure!(
			now >= account_fee_lock_data
				.last_fee_lock_block
				.saturating_add(fee_lock_metadata.period_length),
			Error::<T>::CantUnlockFeeYet
		);

		Ok(())
	}

	fn unlock_fee(who: &T::AccountId) -> DispatchResult {
		// Check if total_fee_lock_amount is non-zero
		// THEN Check is period is greater than last

		let account_fee_lock_data = Self::get_account_fee_lock_data(&who);

		ensure!(!account_fee_lock_data.total_fee_lock_amount.is_zero(), Error::<T>::NotFeeLocked);

		let fee_lock_metadata =
			Self::get_fee_lock_metadata().ok_or(Error::<T>::FeeLocksNotInitialized)?;

		let now = <frame_system::Pallet<T>>::block_number();

		ensure!(
			now >= account_fee_lock_data
				.last_fee_lock_block
				.saturating_add(fee_lock_metadata.period_length),
			Error::<T>::CantUnlockFeeYet
		);

		let unreserve_result = <T as pallet::Config>::Tokens::unreserve(
			<T as pallet::Config>::NativeTokenId::get().into(),
			&who,
			account_fee_lock_data.total_fee_lock_amount.into(),
		);
		if !unreserve_result.is_zero() {
			log::warn!(
				"Unlock lock unreserve resulted in non-zero unreserve_result {:?}",
				unreserve_result
			);
		}

		AccountFeeLockData::<T>::remove(who.clone());

		Self::deposit_event(Event::FeeLockUnlocked(
			who.clone(),
			account_fee_lock_data.total_fee_lock_amount,
		));

		Ok(())
	}
}
