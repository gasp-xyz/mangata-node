#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	storage::bounded_btree_map::BoundedBTreeMap,
	traits::{Get, StorageVersion},
	transactional,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use mangata_types::{Balance, TokenId};
use mp_traits::TimeoutTriggerTrait;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};

use sp_runtime::traits::{CheckedDiv, Zero};
use sp_std::{convert::TryInto, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

pub(crate) const LOG_TARGET: &'static str = "token-timeouts";

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

	#[derive(
		Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo, Default,
	)]
	#[codec(mel_bound(T: Config))]
	#[scale_info(skip_type_params(T))]
	pub struct TimeoutMetadataInfo<T: Config> {
		pub period_length: T::BlockNumber,
		pub timeout_amount: Balance,
		pub swap_value_threshold: BoundedBTreeMap<TokenId, Balance, T::MaxCuratedTokens>,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_timeout_metadata)]
	pub type TimeoutMetadata<T: Config> = StorageValue<_, TimeoutMetadataInfo<T>, OptionQuery>;

	#[derive(
		Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Default,
	)]
	pub struct AccountTimeoutDataInfo<BlockNumber: Default> {
		pub total_timeout_amount: Balance,
		pub last_timeout_block: BlockNumber,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_account_timeout_data)]
	pub type AccountTimeoutData<T: Config> =
		StorageMap<_, Blake2_256, T::AccountId, AccountTimeoutDataInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TimeoutMetadataUpdated,
		TimeoutReleased(T::AccountId, Balance),
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// Timeouts were incorrectly initialized
		TimeoutsIncorrectlyInitialzed,
		/// Timeout metadata is invalid
		InvalidTimeoutMetadata,
		/// Timeouts have not been initialzed
		TimeoutsNotInitialized,
		/// No tokens of the user are timedout
		NotTimedout,
		/// The timeout cannot be released yet
		CantReleaseYet,
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
		#[pallet::constant]
		type NativeTokenId: Get<TokenId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// The weight is calculated using MaxCuratedTokens so it is the worst case weight
		#[transactional]
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::update_timeout_metadata())]
		pub fn update_timeout_metadata(
			origin: OriginFor<T>,
			period_length: Option<T::BlockNumber>,
			timeout_amount: Option<Balance>,
			swap_value_thresholds: Option<Vec<(TokenId, Option<Balance>)>>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let mut timeout_metadata =
				Self::get_timeout_metadata().unwrap_or(TimeoutMetadataInfo {
					period_length: Default::default(),
					timeout_amount: Default::default(),
					swap_value_threshold: BoundedBTreeMap::new(),
				});

			timeout_metadata.period_length =
				period_length.unwrap_or(timeout_metadata.period_length);
			timeout_metadata.timeout_amount =
				timeout_amount.unwrap_or(timeout_metadata.timeout_amount);

			ensure!(!timeout_metadata.timeout_amount.is_zero(), Error::<T>::InvalidTimeoutMetadata);
			ensure!(!timeout_metadata.period_length.is_zero(), Error::<T>::InvalidTimeoutMetadata);

			if let Some(swap_value_thresholds) = swap_value_thresholds {
				for (token_id, maybe_threshold) in swap_value_thresholds.iter() {
					match maybe_threshold {
						Some(threshold) => {
							let _ = timeout_metadata
								.swap_value_threshold
								.try_insert(*token_id, *threshold)
								.map_err(|_| Error::<T>::MaxCuratedTokensLimitExceeded)?;
						},
						None => {
							let _ = timeout_metadata.swap_value_threshold.remove(token_id);
						},
					}
				}
			}

			TimeoutMetadata::<T>::put(timeout_metadata);

			Pallet::<T>::deposit_event(Event::TimeoutMetadataUpdated);

			Ok(().into())
		}

		#[transactional]
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::release_timeout())]
		pub fn release_timeout(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if total_timeout_amount is non-zero
			// THEN Check is period is greater than last

			let account_timeout_data = Self::get_account_timeout_data(&who);

			ensure!(!account_timeout_data.total_timeout_amount.is_zero(), Error::<T>::NotTimedout);

			let timeout_metadata =
				Self::get_timeout_metadata().ok_or(Error::<T>::TimeoutsNotInitialized)?;

			let now = <frame_system::Pallet<T>>::block_number();

			let current_period = now
				.checked_div(&timeout_metadata.period_length)
				.ok_or(Error::<T>::TimeoutsIncorrectlyInitialzed)?;
			let last_timeout_block_period = account_timeout_data
				.last_timeout_block
				.checked_div(&timeout_metadata.period_length)
				.ok_or(Error::<T>::TimeoutsIncorrectlyInitialzed)?;

			ensure!(current_period > last_timeout_block_period, Error::<T>::CantReleaseYet);

			let unreserve_result = <T as pallet::Config>::Tokens::unreserve(
				<T as pallet::Config>::NativeTokenId::get().into(),
				&who,
				account_timeout_data.total_timeout_amount.into(),
			);
			if !unreserve_result.is_zero() {
				log::warn!(
					"Release timeout unreserve resulted in non-zero unreserve_result {:?}",
					unreserve_result
				);
			}

			AccountTimeoutData::<T>::remove(who.clone());

			Pallet::<T>::deposit_event(Event::TimeoutReleased(
				who,
				account_timeout_data.total_timeout_amount,
			));

			Ok(().into())
		}
	}
}

impl<T: Config> TimeoutTriggerTrait<T::AccountId> for Pallet<T> {
	fn process_timeout(who: &T::AccountId) -> DispatchResult {
		let timeout_metadata =
			Self::get_timeout_metadata().ok_or(Error::<T>::TimeoutsNotInitialized)?;
		let mut account_timeout_data = Self::get_account_timeout_data(who);
		let now = <frame_system::Pallet<T>>::block_number();

		let current_period = now
			.checked_div(&timeout_metadata.period_length)
			.ok_or(Error::<T>::TimeoutsIncorrectlyInitialzed)?;
		let last_timeout_block_period = account_timeout_data
			.last_timeout_block
			.checked_div(&timeout_metadata.period_length)
			.ok_or(Error::<T>::TimeoutsIncorrectlyInitialzed)?;

		// This is cause now >= last_timeout_block
		ensure!(current_period >= last_timeout_block_period, Error::<T>::UnexpectedFailure);

		if current_period == last_timeout_block_period {
			// First storage edit
			// Cannot fail beyond this point
			// Rerserve additional timeout_amount
			<T as pallet::Config>::Tokens::reserve(
				<T as pallet::Config>::NativeTokenId::get().into(),
				who,
				timeout_metadata.timeout_amount.into(),
			)?;

			// Insert updated account_timeout_info into storage
			// This is not expected to fail
			account_timeout_data.total_timeout_amount = account_timeout_data
				.total_timeout_amount
				.saturating_add(timeout_metadata.timeout_amount);
			account_timeout_data.last_timeout_block = now;
			AccountTimeoutData::<T>::insert(who, account_timeout_data);
		} else {
			// We must either reserve more or unreserve
			match (timeout_metadata.timeout_amount, account_timeout_data.total_timeout_amount) {
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
							"Process timeout unreserve resulted in non-zero unreserve_result {:?}",
							unreserve_result
						);
					}
				},
				_ => {},
			}
			// Insert updated account_timeout_info into storage
			// This is not expected to fail
			account_timeout_data.total_timeout_amount = timeout_metadata.timeout_amount;
			account_timeout_data.last_timeout_block = now;
			AccountTimeoutData::<T>::insert(who, account_timeout_data);
		}

		Ok(())
	}

	fn can_release_timeout(who: &T::AccountId) -> DispatchResult {
		// Check if total_timeout_amount is non-zero
		// THEN Check is period is greater than last

		let account_timeout_data = Self::get_account_timeout_data(&who);

		ensure!(!account_timeout_data.total_timeout_amount.is_zero(), Error::<T>::NotTimedout);

		let timeout_metadata =
			Self::get_timeout_metadata().ok_or(Error::<T>::TimeoutsNotInitialized)?;

		let now = <frame_system::Pallet<T>>::block_number();

		let current_period = now
			.checked_div(&timeout_metadata.period_length)
			.ok_or(Error::<T>::TimeoutsIncorrectlyInitialzed)?;
		let last_timeout_block_period = account_timeout_data
			.last_timeout_block
			.checked_div(&timeout_metadata.period_length)
			.ok_or(Error::<T>::TimeoutsIncorrectlyInitialzed)?;

		ensure!(current_period > last_timeout_block_period, Error::<T>::CantReleaseYet);

		Ok(())
	}
}
