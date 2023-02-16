#![cfg_attr(not(feature = "std"), no_std)]
#![feature(custom_test_frameworks)]

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

use sp_runtime::{
	traits::{CheckedAdd, Zero},
	Saturating,
};
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
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_idle(now: T::BlockNumber, remaining_weight: Weight) -> Weight {
			let mut consumed_weight: Weight = Default::default();

			// process only up to 80% or remaining weight
			let base_cost = T::DbWeight::get().reads(1)   // UnlockQueueBegin
				+ T::DbWeight::get().reads(1)   // UnlockQueueEnd
				+ T::DbWeight::get().reads(1); // FeeLockMetadata

			let cost_of_single_unlock_iteration = T::WeightInfo::unlock_fee() // cost of unlock action
				+ T::DbWeight::get().reads(1)   // FeeLockMetadataQeueuePosition
				+ T::DbWeight::get().reads(1)   // AccountFeeLockData
				+ T::DbWeight::get().reads(1)   // UnlockQueue
				+ T::DbWeight::get().writes(1); // UnlockQueueBegin

			if (base_cost + cost_of_single_unlock_iteration).ref_time() >
				remaining_weight.ref_time()
			{
				return Weight::from_ref_time(0)
			}

			let metadata = Self::get_fee_lock_metadata();
			let period_length = metadata.map(|meta| meta.period_length);
			let begin = UnlockQueueBegin::<T>::get();
			let end = UnlockQueueEnd::<T>::get();
			consumed_weight += base_cost;

			for i in begin..end {
				consumed_weight += T::DbWeight::get().reads(3); // UnlockQueue, AccountFeeLockData FeeLockMetadataQeueuePosition
				UnlockQueueBegin::<T>::put(i);
				consumed_weight += T::DbWeight::get().writes(1);
				let who = UnlockQueue::<T>::get(i);
				let queue_pos =
					who.as_ref().and_then(|acc| FeeLockMetadataQeueuePosition::<T>::get(acc));

				if matches!(queue_pos, Some(id) if id == i) {
					let lock_info = who.and_then(|who| {
						AccountFeeLockData::<T>::try_get(&who).map(|lock| (who.clone(), lock)).ok()
					});

					match (period_length, lock_info) {
						(Some(period_length), Some((who, lock))) => {
							let unlock_block = lock.last_fee_lock_block.checked_add(&period_length);

							if matches!(unlock_block, Some(unlock) if unlock <= now) {
								UnlockQueueBegin::<T>::put(i + 1);
								consumed_weight += T::WeightInfo::unlock_fee();
								let _ =
									<Self as FeeLockTriggerTrait<T::AccountId>>::unlock_fee(&who);
							} else {
								break
							}
						},
						_ => break,
					};
				} else {
					UnlockQueueBegin::<T>::put(i + 1);
				}

				if cost_of_single_unlock_iteration.ref_time() >
					(remaining_weight.ref_time() - consumed_weight.ref_time())
				{
					break
				}
			}
			consumed_weight
		}
	}

	#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[codec(mel_bound(T: Config))]
	#[scale_info(skip_type_params(T))]
	pub struct FeeLockMetadataInfo<T: Config> {
		pub period_length: T::BlockNumber,
		pub fee_lock_amount: Balance,
		pub swap_value_threshold: Balance,
		pub whitelisted_tokens: BoundedBTreeSet<TokenId, T::MaxCuratedTokens>,
	}

	impl<T: Config> Default for FeeLockMetadataInfo<T> {
		fn default() -> Self {
			Self {
				period_length: Default::default(),
				fee_lock_amount: Default::default(),
				swap_value_threshold: Default::default(),
				whitelisted_tokens: Default::default(),
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn get_fee_lock_metadata)]
	pub type FeeLockMetadata<T: Config> = StorageValue<_, FeeLockMetadataInfo<T>, OptionQuery>;

	#[pallet::storage]
	pub type FeeLockMetadataQeueuePosition<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, u128, OptionQuery>;

	#[pallet::storage]
	pub type UnlockQueue<T: Config> = StorageMap<_, Twox64Concat, u128, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type UnlockQueueBegin<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	pub type UnlockQueueEnd<T: Config> = StorageValue<_, u128, ValueQuery>;

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
impl<T: Config> Pallet<T> {
	pub(crate) fn push_to_the_end_of_unlock_queue(who: &T::AccountId) {
		let mut id = Default::default();
		let id_ref = &mut id;
		UnlockQueueEnd::<T>::mutate(|id| {
			*id_ref = *id;
			*id = *id + 1
		});
		UnlockQueue::<T>::insert(id, who);
		FeeLockMetadataQeueuePosition::<T>::set(who, Some(id));
	}

	pub(crate) fn move_to_the_end_of_unlock_queue(who: &T::AccountId) {
		if let Ok(id) = FeeLockMetadataQeueuePosition::<T>::try_get(who) {
			UnlockQueue::<T>::take(id);
			Self::push_to_the_end_of_unlock_queue(who);
		} else {
			Self::push_to_the_end_of_unlock_queue(who);
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
			Self::move_to_the_end_of_unlock_queue(who);
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
			Self::move_to_the_end_of_unlock_queue(who);
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

		FeeLockMetadataQeueuePosition::<T>::take(&who);
		AccountFeeLockData::<T>::remove(&who);

		Self::deposit_event(Event::FeeLockUnlocked(
			who.clone(),
			account_fee_lock_data.total_fee_lock_amount,
		));

		Ok(())
	}
}
