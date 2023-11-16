#![cfg_attr(not(feature = "std"), no_std)]

//! # Proof of Stake Module
//!
//! The goal of the Proof of Stake module is to reward people for providing liquidity to the Mangata DEX.
//!
//! ## Types of Rewards
//!
//! ### Native Rewards
//!
//! As described in Mangata tokenomics, during each session, some of the rewards are minted and distributed
//! among promoted pools. The council decides which pool to promote, and each promoted pool has a weight
//! assigned that determines how much of the rewards should
//! be distributed to that pool.
//!
//! The final amount of tokens that a user receives depends on:
//! - the amount of activated liquidity - rewards are distributed proportionally to the amount of
//!   activated liquidity.
//! - the liquidity token itself - the more weight is assigned to the pool, the more rewards it receives.
//! - the time of liquidity activation - the longer a user stores liquidity, the more rewards they receive
//!   (based on an asymptotic curve).
//!
//! Activated Liquidity cannot be transferred to another account; it is considered locked. The moment
//! liquidity is unlocked, the user loses the ability to claim rewards for that liquidity.
//!
//! #### Storage entries
//!
//! - [`TotalActivatedLiquidity`] - Stores information about the total amount of activated liquidity for
//! each liquidity token.
//! - [`PromotedPoolRewards`] - Stores information about the total amount of rewards for each liquidity
//! token.
//! - [`RewardsInfo`] - Stores information about rewards for liquidity mining.
//! - [`ThirdPartyActivationKind`] - Wrapper over origin ActivateKind that is used in
//!
//! #### Extrinsics
//!
//! - [`Pallet::activate_liquidity`] - Activates liquidity for liquidity mining rewards.
//! - [`Pallet::deactivate_liquidity_for_native_rewards`] - Deactivates liquidity for liquidity mining rewards.
//! - [`Pallet::claim_native_rewards`] - Claims all rewards for all liquidity tokens.
//! - [`Pallet::update_pool_promotion`] - Enables/disables the pool for liquidity mining rewards.
//!
//! ### 3rd Party Rewards
//!
//! Anyone can provide tokens to reward people who store a particular liquidity token. Any
//! liquidity token can be rewarded with any other token provided by the user. Liquidity can be
//! activated for multiple scheduled rewards related to that liquidity token. Tokens will remain
//! locked (untransferable) as long as there is at least one schedule for which these rewards are
//! activated.
//!
//! #### Storage entries
//!
//! - [`RewardsInfoForScheduleRewards`] - Stores information about rewards for scheduled rewards.
//! - [`ScheduleRewardsTotal`] - Stores the amount of rewards per single liquidity token.
//! - [`RewardsSchedules`] - Stores information about scheduled rewards.
//! - [`ScheduleId`] - Stores the unique id of the schedule.
//! - [`RewardTokensPerPool`] - Stores information about which reward tokens are used for a particular
//! liquidity token.
//! - [`TotalActivatedLiquidityForSchedules`] - Stores information about the total amount of activated
//! liquidity for each schedule.
//! - [`ActivatedLiquidityForSchedules`] - Stores information about how much liquidity was activated for
//! each schedule.
//! - [`ActivatedLockedLiquidityForSchedules`] - Stores information about how much liquidity was activated
//! for each schedule and not yet liquidity mining rewards.
//!
//!
//! #### Extrinsics
//!
//! - [`Pallet::reward_pool`] - Schedules rewards for the selected liquidity token.
//! - [`Pallet::activate_liquidity_for_3rdparty_rewards`] - Activates liquidity for scheduled rewards.
//! - [`Pallet::deactivate_liquidity_for_3rdparty_rewards`] - Deactivates liquidity for scheduled rewards.
//! - [`Pallet::claim_3rdparty_rewards`] - Claims all scheduled rewards for all liquidity tokens.
//!
//! ## Reusing a Single Liquidity Token for Multiple Rewards
//!
//! It may happen that a single liquidity token is rewarded with:
//! - Liquidity Mining Rewards - because the pool was promoted by the council.
//! - Scheduled rewards with token X - because Alice decided to do so.
//! - Scheduled rewards with token Y - because Bob decided to do so.
//!
//! In that case, a single liquidity token can be used to obtain rewards from multiple sources. There are
//! several options to do that:
//!
//! - The user can reuse liquidity used for liquidity mining rewards to claim scheduled rewards. In
//!   this case, [`Pallet::activate_liquidity_for_3rdparty_rewards`] should be used with [`ActivateKind::LiquidityMining`].
//!
//! - The user can reuse liquidity used for scheduled rewards (X) to sign up for rewards from other tokens (provided by Bob). In that case, [`Pallet::activate_liquidity_for_3rdparty_rewards`] should be used with [`ActivateKind::ActivatedLiquidity(X)`].
//!
//! - The user can't directly provide liquidity activated for scheduled rewards to activate it for native rewards. Instead:
//!     * Liquidity used for schedule rewards can be deactivated
//!     [`Pallet::deactivate_liquidity_for_3rdparty_rewards`].
//!     * Liquidity can be activated for liquidity mining rewards [`Pallet::activate_liquidity`].
//!     * Liquidity can be activated for scheduled rewards [`Pallet::activate_liquidity_for_3rdparty_rewards`] with [`ThirdPartyActivationKind::Mining`].

use frame_support::pallet_prelude::*;

pub type ScheduleId = u64;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]

pub struct Schedule<T: Config> {
	scheduled_at: SessionId,
	last_session: SessionId,
	liq_token: CurrencyIdOf<T>,
	reward_token: CurrencyIdOf<T>,
	amount_per_session: BalanceOf<T>,
}

#[derive(Encode, Decode, Default, TypeInfo)]
pub struct SchedulesList {
	pub head: Option<ScheduleId>,
	pub tail: Option<ScheduleId>,
	pub pos: Option<ScheduleId>,
	pub count: u64,
}

use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchResult, PostDispatchInfo},
	ensure,
	storage::bounded_btree_map::BoundedBTreeMap,
	traits::Currency,
};
use frame_system::ensure_signed;
use mangata_support::traits::Valuate;
use mangata_types::multipurpose_liquidity::ActivateKind;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use sp_core::U256;
use sp_runtime::traits::AccountIdConversion;

use frame_support::{
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, ExistenceRequirement, Get},
	transactional,
};

use frame_system::pallet_prelude::*;
use mangata_support::traits::{
	ActivationReservesProviderTrait, LiquidityMiningApi, ProofOfStakeRewardsApi,
};
use sp_std::collections::btree_map::BTreeMap;

use sp_runtime::{
	traits::{CheckedAdd, CheckedSub, SaturatedConversion, Zero},
	DispatchError, Perbill,
};
use sp_std::{convert::TryInto, prelude::*};

mod reward_info;
use reward_info::{RewardInfo, RewardsCalculator};

mod schedule_rewards_calculator;
use schedule_rewards_calculator::{
	ActivatedLiquidityPerSchedule, ScheduleRewards, ScheduleRewardsCalculator,
};

mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(all(test, not(feature = "runtime-benchmarks")))]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod utils;

pub(crate) const LOG_TARGET: &str = "pos";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type BalanceOf<T> = <<T as Config>::Currency as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

type CurrencyIdOf<T> = <<T as Config>::Currency as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

/// - `LiquidityMining` - already activated liquidity (for liquidity mining rewards)
#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ThirdPartyActivationKind<CurrencyId> {
	ActivateKind(Option<ActivateKind>),

	ActivatedLiquidity(CurrencyId),
	NativeRewardsLiquidity,
}

const PALLET_ID: frame_support::PalletId = frame_support::PalletId(*b"rewards!");
pub type SessionId = u32;
#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			let session_id = Self::session_index();

			// NOTE: 1R
			if Self::is_new_session() {
				SchedulesListMetadata::<T>::mutate(|s| s.pos = None);
				return Default::default()
			}

			for _ in 0..T::SchedulesPerBlock::get() {
				// READS PER ITERTION
				//
				// 			ON VALID SCHEDULE  (AVERAGE)                        ====> 1 RW + N*R + N*W
				// 				1 x READ/WRITE SCHEDULE META(HEAD,TAIL,POS)            : ALWAYS
				// 				PER ITER:
				// 					- READ RewardsSchedulesList   : ALWAYS
				// 					- WRITE ScheduleRewardsTotal  : ALWAYS (pesemisitic)

				// 			ON OUTDATED SCHEDULE (PESIMITIC)                   =====> 1 RW + (N-1)*W + 1W
				// 				1 x READ/WRITE SCHEDULE META(HEAD,TAIL,POS)            : ALWAYS
				// 				REMOVE N-1 SCHEDULES IN THE MIDDDLE
				// 					- 1 x WRITE update previous schedudle `next`  : ALWAYS (pesemisitic)
				// 				REMOVE LAST ELEM:
				// 					- 1 x write update list tail (already counted in)
				// 					- 1 x WRITE update elem before last : ALWAYS (pesemisitic)

				// NOTE: 1R
				let s = SchedulesListMetadata::<T>::get();
				let last_valid = s.pos;
				// NOTE: 1R
				let pos = match (last_valid, s.head) {
					(Some(pos), _) => {
						if let Some((_schedule, next)) = RewardsSchedulesList::<T>::get(pos) {
							next
						} else {
							None
						}
					},
					(None, Some(head)) => Some(head),
					_ => None,
				};

				if let Some(pos_val) = pos {
					// NOTE: 1R
					if let Some((schedule, next)) = RewardsSchedulesList::<T>::get(pos_val) {
						if schedule.last_session >= session_id {
							if schedule.scheduled_at < session_id {
								// NOTE: 1R 1W
								ScheduleRewardsTotal::<T>::mutate(
									(schedule.liq_token, schedule.reward_token),
									|s| s.provide_rewards(session_id, schedule.amount_per_session),
								);
							}
							// NOTE: 1W
							SchedulesListMetadata::<T>::mutate(|s| s.pos = Some(pos_val));
						} else {
							// NOTE: 2R
							//
							let meta = Self::list_metadata();
							match (meta.head, meta.tail) {
								(Some(head), Some(tail)) if head == pos_val && head != tail =>
									if let Some(next) = next {
										// NOTE: 1W
										SchedulesListMetadata::<T>::mutate(|s| {
											s.head = Some(next);
											s.count -= 1;
										});
									},
								(Some(head), Some(tail)) if tail == pos_val && head == tail => {
									// NOTE: 3W
									SchedulesListMetadata::<T>::mutate(|s| {
										s.tail = None;
										s.head = None;
										s.pos = None;
										s.count = 0;
									});
								},
								(Some(head), Some(tail)) if tail == pos_val && head != tail =>
									if let Some(last_valid) = last_valid {
										// NOTE: 1W
										SchedulesListMetadata::<T>::mutate(|s| {
											s.tail = Some(last_valid);
											s.count -= 1;
										});
										// NOTE: 1R 1W
										RewardsSchedulesList::<T>::mutate(last_valid, |data| {
											if let Some((_schedule, next)) = data.as_mut() {
												*next = None
											}
										});
									},
								(Some(_head), Some(_tail)) =>
									if let Some(last_valid) = last_valid {
										SchedulesListMetadata::<T>::mutate(|s| {
											s.count -= 1;
										});
										// NOTE: 1R 1W
										RewardsSchedulesList::<T>::mutate(last_valid, |data| {
											if let Some((_schedule, prev_next)) = data.as_mut() {
												*prev_next = next
											}
										});
									},
								_ => {},
							}
						}
					}
				} else {
					break
				}
			}

			// always use same amount of block space even if no schedules were processed
			T::DbWeight::get().reads(1) +
				T::DbWeight::get().writes(1) +
				T::DbWeight::get().reads(T::SchedulesPerBlock::get().into()) +
				T::DbWeight::get().writes(T::SchedulesPerBlock::get().into())
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub trait PoSBenchmarkingConfig: pallet_issuance::Config {}
	#[cfg(feature = "runtime-benchmarks")]
	impl<T: pallet_issuance::Config> PoSBenchmarkingConfig for T {}

	#[cfg(not(feature = "runtime-benchmarks"))]
	pub trait PoSBenchmarkingConfig {}
	#[cfg(not(feature = "runtime-benchmarks"))]
	impl<T> PoSBenchmarkingConfig for T {}

	#[cfg(not(feature = "runtime-benchmarks"))]
	pub trait ValutationApiTrait<T: Config>: Valuate<BalanceOf<T>, CurrencyIdOf<T>> {}

	#[cfg(feature = "runtime-benchmarks")]
	pub trait ValutationApiTrait<T: Config>:
		Valuate<BalanceOf<T>, CurrencyIdOf<T>>
		+ XykFunctionsTrait<AccountIdOf<T>, BalanceOf<T>, CurrencyIdOf<T>>
	{
	}

	#[cfg(not(feature = "runtime-benchmarks"))]
	impl<T, C> ValutationApiTrait<C> for T
	where
		C: Config,
		T: Valuate<BalanceOf<C>, CurrencyIdOf<C>>,
	{
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<T, C> ValutationApiTrait<C> for T
	where
		C: Config,
		T: Valuate<BalanceOf<C>, CurrencyIdOf<C>>,
		T: XykFunctionsTrait<AccountIdOf<C>, BalanceOf<C>, CurrencyIdOf<C>>,
	{
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + PoSBenchmarkingConfig {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type ActivationReservesProvider: ActivationReservesProviderTrait<
			Self::AccountId,
			BalanceOf<Self>,
			CurrencyIdOf<Self>,
		>;
		type NativeCurrencyId: Get<CurrencyIdOf<Self>>;
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;
		#[pallet::constant]
		/// The account id that holds the liquidity mining issuance
		type LiquidityMiningIssuanceVault: Get<Self::AccountId>;
		#[pallet::constant]
		type RewardsDistributionPeriod: Get<u32>;
		/// The maximum number of schedules that can be active at one moment
		type RewardsSchedulesLimit: Get<u32>;
		/// The minimum number of rewards per session for schedule rewards
		type Min3rdPartyRewardValutationPerSession: Get<u128>;
		type Min3rdPartyRewardVolume: Get<u128>;
		type SchedulesPerBlock: Get<u32>;

		type WeightInfo: WeightInfo;
		type ValuationApi: ValutationApiTrait<Self>;
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// Not enought assets
		NotEnoughAssets,
		/// Math overflow
		MathOverflow,
		/// Not enough rewards earned
		NotEnoughRewardsEarned,
		/// Not a promoted pool
		NotAPromotedPool,
		/// Past time calculation
		PastTimeCalculation,
		LiquidityCheckpointMathError,
		CalculateRewardsMathError,
		MathError,
		CalculateRewardsAllMathError,
		MissingRewardsInfoError,
		DeprecatedExtrinsic,
		/// Cannot schedule rewards in past
		CannotScheduleRewardsInPast,
		/// Pool does not exist
		PoolDoesNotExist,
		/// Too many schedules
		TooManySchedules,
		/// Too little rewards per session
		TooLittleRewards,
		/// Too small volume of the pool
		TooSmallVolume,
		// Liquidity is reused for 3rdparty rewards
		LiquidityLockedIn3rdpartyRewards,
		// No rewards to claim
		NoThirdPartyPartyRewardsToClaim,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolPromotionUpdated(CurrencyIdOf<T>, Option<u8>),
		LiquidityActivated(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>),
		LiquidityDeactivated(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>),
		RewardsClaimed(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>),
		ThirdPartyRewardsClaimed(T::AccountId, CurrencyIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn get_rewards_info)]
	pub type RewardsInfo<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		RewardInfo<BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// Stores information about pool weight and accumulated rewards. The accumulated
	/// rewards amount is the number of rewards that can be claimed per liquidity
	/// token. Here is tracked the number of rewards per liquidity token relationship.
	/// Expect larger values when the number of liquidity tokens are smaller.
	pub type PromotedPoolRewards<T: Config> =
		StorageValue<_, BTreeMap<CurrencyIdOf<T>, PromotedPools>, ValueQuery>;

	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
	/// Information about single token rewards
	pub struct PromotedPools {
		// Weight of the pool, each of the activated tokens has its weight assignedd
		// Liquidityt Mining Rewards are distributed based on that weight
		pub weight: u8,
		/// **Cumulative** number of rewards amount that can be claimed for single
		/// activted liquidity token
		pub rewards: U256,
	}

	#[pallet::storage]
	#[pallet::getter(fn total_activated_amount)]
	pub type TotalActivatedLiquidity<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BalanceOf<T>, ValueQuery>;

	// //////////////////////////////////////////////////////////////////////////////////////////////
	// 3rd Party Rewards
	// //////////////////////////////////////////////////////////////////////////////////////////////

	/// Stores information about pool weight and accumulated rewards
	#[pallet::storage]
	pub type RewardsInfoForScheduleRewards<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		(CurrencyIdOf<T>, CurrencyIdOf<T>),
		RewardInfo<BalanceOf<T>>,
		ValueQuery,
	>;

	/// How much scheduled rewards per single liquidty_token should be distribute_rewards
	/// the **value is multiplied by u128::MAX** to avoid floating point arithmetic
	#[pallet::storage]
	pub type ScheduleRewardsTotal<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(CurrencyIdOf<T>, CurrencyIdOf<T>),
		ScheduleRewards<BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type ScheduleRewardsPerLiquidity<T: Config> =
		StorageMap<_, Twox64Concat, (CurrencyIdOf<T>, CurrencyIdOf<T>), (U256, u64), ValueQuery>;

	/// List of activated schedules sorted by expiry date
	#[pallet::storage]
	#[pallet::getter(fn schedules)]
	pub type RewardsSchedules<T: Config> = StorageValue<
		_,
		BoundedBTreeMap<
			(BlockNumberFor<T>, CurrencyIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>, u64),
			(),
			T::RewardsSchedulesLimit,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn list_metadata)]
	pub type SchedulesListMetadata<T: Config> = StorageValue<_, SchedulesList, ValueQuery>;

	#[pallet::storage]
	pub type RewardsSchedulesList<T: Config> =
		StorageMap<_, Twox64Concat, ScheduleId, (Schedule<T>, Option<ScheduleId>), OptionQuery>;

	/// Maps liquidity token to list of tokens that it ever was rewarded with
	#[pallet::storage]
	pub type RewardTokensPerPool<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		CurrencyIdOf<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		(),
		ValueQuery,
	>;

	/// Tracks number of activated liquidity per schedule. It is used for calculation of
	/// "cumulative rewrds amount" per 1 liquidity token. Therefore activation/deactivation needs
	/// to be deffered same way as schedule rewards are delayed.
	#[pallet::storage]
	pub type TotalActivatedLiquidityForSchedules<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		CurrencyIdOf<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		ActivatedLiquidityPerSchedule<BalanceOf<T>>,
		ValueQuery,
	>;

	/// Tracks how much liquidity user activated for particular (liq token, reward token) pair
	/// StorageNMap was used because it only require single read to know if user deactivated all
	/// liquidity associated with particular liquidity_token that is rewarded. If so part of the
	/// liquididty tokens can be unlocked.
	#[pallet::storage]
	pub type ActivatedLiquidityForSchedules<T> = StorageNMap<
		_,
		(
			NMapKey<Twox64Concat, AccountIdOf<T>>,
			NMapKey<Twox64Concat, CurrencyIdOf<T>>,
			NMapKey<Twox64Concat, CurrencyIdOf<T>>,
		),
		BalanceOf<T>,
		OptionQuery,
	>;

	/// Tracks how much of the liquidity was activated for schedule rewards and not yet
	/// liquidity mining rewards. That information is essential to properly handle token unlcocks
	/// when liquidity is deactivated.
	#[pallet::storage]
	pub type ActivatedLockedLiquidityForSchedules<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Tracks how much of the liquidity was activated for schedule rewards and not yet
	/// liquidity mining rewards. That information is essential to properly handle token unlcocks
	/// when liquidity is deactivated.
	#[pallet::storage]
	pub type ActivatedNativeRewardsLiq<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		BalanceOf<T>,
		ValueQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claims liquidity mining rewards
		#[transactional]
		#[pallet::call_index(0)]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_native_rewards())]
		#[deprecated(note = "claim_native_rewards should be used instead")]
		pub fn claim_rewards_all(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as ProofOfStakeRewardsApi<_, _, _>>::claim_rewards_all(
				sender,
				liquidity_token_id,
			)?;

			Ok(())
		}

		/// Enables/disables pool for liquidity mining rewards
		#[pallet::call_index(1)]
		#[pallet::weight(<<T as Config>::WeightInfo>::update_pool_promotion())]
		pub fn update_pool_promotion(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			liquidity_mining_issuance_weight: u8,
		) -> DispatchResult {
			ensure_root(origin)?;

			if liquidity_mining_issuance_weight > 0 {
				<Self as ProofOfStakeRewardsApi<_, _, _>>::enable(
					liquidity_token_id,
					liquidity_mining_issuance_weight,
				);
			} else {
				<Self as ProofOfStakeRewardsApi<_, _, _>>::disable(liquidity_token_id);
			}
			Ok(())
		}

		/// Increases number of tokens used for liquidity mining purposes.
		///
		/// Parameters:
		/// - liquidity_token_id - id of the token
		/// - amount - amount of the token
		/// - use_balance_from - where from tokens should be used
		#[transactional]
		#[pallet::call_index(2)]
		#[pallet::weight(<<T as Config>::WeightInfo>::activate_liquidity_for_native_rewards())]
		#[deprecated(note = "activate_liquidity_for_native_rewards should be used instead")]
		pub fn activate_liquidity(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			use_balance_from: Option<ActivateKind>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as ProofOfStakeRewardsApi<_, _, _>>::activate_liquidity(
				sender,
				liquidity_token_id,
				amount,
				use_balance_from,
			)
		}

		/// Decreases number of tokens used for liquidity mining purposes
		#[transactional]
		#[pallet::call_index(3)]
		#[pallet::weight(<<T as Config>::WeightInfo>::deactivate_liquidity_for_native_rewards())]
		#[deprecated(note = "deactivate_liquidity_for_native_rewards should be used instead")]
		pub fn deactivate_liquidity(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::deactivate_liquidity_for_native_rewards_impl(sender, liquidity_token_id, amount)
		}

		/// Schedules rewards for selected liquidity token
		/// - tokens - pair of tokens
		/// - amount - amount of the token
		/// - schedule_end - id of the last rewarded seession. Rewards will be distributedd equally between sessions in range (now ..
		/// schedule_end). Distribution starts from the *next* session till `schedule_end`.
		#[transactional]
		#[pallet::call_index(4)]
		#[pallet::weight(<<T as Config>::WeightInfo>::reward_pool())]
		pub fn reward_pool(
			origin: OriginFor<T>,
			pool: (CurrencyIdOf<T>, CurrencyIdOf<T>),
			token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			schedule_end: SessionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::reward_pool_impl(sender, pool, token_id, amount, schedule_end)
		}

		/// Increases number of tokens used for liquidity mining purposes.
		///
		/// Parameters:
		/// - liquidity_token_id - id of the token
		/// - amount - amount of the token
		/// - use_balance_from - where from tokens should be used. If set to `None` then tokens will
		/// be taken from available balance
		#[transactional]
		#[pallet::call_index(5)]
		#[pallet::weight(<<T as Config>::WeightInfo>::activate_liquidity_for_3rdparty_rewards())]
		pub fn activate_liquidity_for_3rdparty_rewards(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			reward_token: CurrencyIdOf<T>,
			use_balance_from: Option<ThirdPartyActivationKind<CurrencyIdOf<T>>>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			Self::activate_liquidity_for_3rdparty_rewards_impl(
				sender,
				liquidity_token_id,
				amount,
				use_balance_from.unwrap_or(ThirdPartyActivationKind::ActivateKind(None)),
				reward_token,
			)
			.map_err(|err| DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(
						<<T as Config>::WeightInfo>::activate_liquidity_for_3rdparty_rewards(),
					),
					pays_fee: Pays::Yes,
				},
				error: err,
			})?;
			Ok(Pays::No.into())
		}

		/// Decreases number of tokens used for liquidity mining purposes.
		///
		/// Parameters:
		/// - liquidity_token_id - id of the token
		/// - amount - amount of the token
		/// - use_balance_from - where from tokens should be used
		#[transactional]
		#[pallet::call_index(6)]
		#[pallet::weight(<<T as Config>::WeightInfo>::deactivate_liquidity_for_3rdparty_rewards())]
		pub fn deactivate_liquidity_for_3rdparty_rewards(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,

			reward_token: CurrencyIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			Self::deactivate_liquidity_for_3rdparty_rewards_impl(
				sender,
				liquidity_token_id,
				amount,
				reward_token,
			)
			.map_err(|err| DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(
						<<T as Config>::WeightInfo>::activate_liquidity_for_3rdparty_rewards(),
					),
					pays_fee: Pays::Yes,
				},
				error: err,
			})?;
			Ok(Pays::No.into())
		}

		/// Claims liquidity mining rewards
		/// - tokens - pair of tokens
		/// - amount - amount of the token
		/// - reward_token - id of the token that is rewarded
		#[transactional]
		#[pallet::call_index(7)]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_3rdparty_rewards())]
		pub fn claim_3rdparty_rewards(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			reward_token: CurrencyIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ScheduleRewardsCalculator::<T>::update_cumulative_rewards(
				liquidity_token_id,
				reward_token,
			);
			Self::claim_schedule_rewards_all_impl(sender, liquidity_token_id, reward_token)
				.map_err(|err| DispatchErrorWithPostInfo {
					post_info: PostDispatchInfo {
						actual_weight: Some(<<T as Config>::WeightInfo>::claim_3rdparty_rewards()),
						pays_fee: Pays::Yes,
					},
					error: err,
				})?;
			Ok(Pays::No.into())
		}

		/// Increases number of tokens used for liquidity mining purposes.
		///
		/// Parameters:
		/// - liquidity_token_id - id of the token
		/// - amount - amount of the token
		/// - use_balance_from - where from tokens should be used
		#[transactional]
		#[pallet::call_index(8)]
		#[pallet::weight(<<T as Config>::WeightInfo>::activate_liquidity_for_native_rewards())]
		pub fn activate_liquidity_for_native_rewards(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
			use_balance_from: Option<ActivateKind>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::activate_liquidity_for_native_rewards_impl(
				sender,
				liquidity_token_id,
				amount,
				use_balance_from,
			)
		}

		/// Decreases number of tokens used for liquidity mining purposes
		#[transactional]
		#[pallet::call_index(9)]
		#[pallet::weight(<<T as Config>::WeightInfo>::deactivate_liquidity_for_native_rewards())]
		pub fn deactivate_liquidity_for_native_rewards(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::deactivate_liquidity_for_native_rewards_impl(sender, liquidity_token_id, amount)
		}

		#[transactional]
		#[pallet::call_index(10)]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_native_rewards())]
		#[deprecated(note = "claim_native_rewards should be used instead")]
		pub fn claim_native_rewards(
			origin: OriginFor<T>,
			liquidity_token_id: CurrencyIdOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as ProofOfStakeRewardsApi<_, _, _>>::claim_rewards_all(
				sender,
				liquidity_token_id,
			)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn activate_liquidity_for_native_rewards_impl(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		Self::ensure_native_rewards_enabled(liquidity_asset_id)?;

		ensure!(
			<T as Config>::ActivationReservesProvider::can_activate(
				liquidity_asset_id,
				&user,
				amount,
				use_balance_from.clone()
			),
			Error::<T>::NotEnoughAssets
		);

		Self::set_liquidity_minting_checkpoint(user.clone(), liquidity_asset_id, amount)?;

		<T as Config>::ActivationReservesProvider::activate(
			liquidity_asset_id,
			&user,
			amount,
			use_balance_from,
		)?;
		Pallet::<T>::deposit_event(Event::LiquidityActivated(user, liquidity_asset_id, amount));

		Ok(())
	}

	pub fn calculate_native_rewards_amount(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		Self::ensure_native_rewards_enabled(liquidity_asset_id)?;
		let rewards_info = RewardsInfo::<T>::try_get(user.clone(), liquidity_asset_id)
			.or(Err(DispatchError::from(Error::<T>::MissingRewardsInfoError)))?;

		let current_rewards = if rewards_info.activated_amount.is_zero() {
			BalanceOf::<T>::zero()
		} else {
			let calc = RewardsCalculator::<_, BalanceOf<T>>::mining_rewards::<T>(
				user.clone(),
				liquidity_asset_id,
			)?;
			calc.calculate_rewards().map_err(|err| Into::<Error<T>>::into(err))?
		};

		Ok(current_rewards
			.checked_add(&rewards_info.rewards_not_yet_claimed)
			.and_then(|v| v.checked_sub(&rewards_info.rewards_already_claimed))
			.ok_or(Error::<T>::CalculateRewardsMathError)?)
	}

	fn deactivate_liquidity_for_native_rewards_impl(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		if amount > BalanceOf::<T>::zero() {
			Self::set_liquidity_burning_checkpoint(user.clone(), liquidity_asset_id, amount)?;
			Pallet::<T>::deposit_event(Event::LiquidityDeactivated(
				user,
				liquidity_asset_id,
				amount,
			));
		}
		Ok(())
	}

	fn activate_liquidity_for_3rdparty_rewards_impl(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
		use_balance_from: ThirdPartyActivationKind<CurrencyIdOf<T>>,
		reward_token: CurrencyIdOf<T>,
	) -> DispatchResult {
		Self::ensure_3rdparty_rewards_enabled(liquidity_asset_id)?;

		match use_balance_from {
			// 1R 1W
			ThirdPartyActivationKind::ActivateKind(ref use_balance_from) => {
				ensure!(
					<T as Config>::ActivationReservesProvider::can_activate(
						liquidity_asset_id,
						&user,
						amount,
						use_balance_from.clone(),
					),
					Error::<T>::NotEnoughAssets
				);
				ActivatedLockedLiquidityForSchedules::<T>::mutate(
					user.clone(),
					liquidity_asset_id,
					|val| *val += amount,
				);
			},
			// 2R
			ThirdPartyActivationKind::ActivatedLiquidity(token_id) => {
				let already_activated_amount = RewardsInfoForScheduleRewards::<T>::get(
					user.clone(),
					(liquidity_asset_id, reward_token),
				)
				.activated_amount;
				let available_amount = RewardsInfoForScheduleRewards::<T>::get(
					user.clone(),
					(liquidity_asset_id, token_id),
				)
				.activated_amount;
				ensure!(
					already_activated_amount + amount <= available_amount,
					Error::<T>::NotEnoughAssets
				);
			},
			ThirdPartyActivationKind::NativeRewardsLiquidity => {
				let already_activated_amount = RewardsInfoForScheduleRewards::<T>::get(
					user.clone(),
					(liquidity_asset_id, reward_token),
				)
				.activated_amount;
				let available_amount =
					RewardsInfo::<T>::get(user.clone(), liquidity_asset_id).activated_amount;
				ensure!(
					already_activated_amount + amount <= available_amount,
					Error::<T>::NotEnoughAssets
				);
				ActivatedNativeRewardsLiq::<T>::mutate(user.clone(), liquidity_asset_id, |val| {
					*val += amount
				});
			},
		}

		Self::set_liquidity_minting_checkpoint_3rdparty(
			user.clone(),
			liquidity_asset_id,
			amount,
			reward_token,
		)?;

		match use_balance_from {
			ThirdPartyActivationKind::ActivateKind(use_balance_from) => {
				<T as Config>::ActivationReservesProvider::activate(
					liquidity_asset_id,
					&user,
					amount,
					use_balance_from,
				)?;
			},
			_ => {},
		}

		Pallet::<T>::deposit_event(Event::LiquidityActivated(user, liquidity_asset_id, amount));

		Ok(())
	}

	fn deactivate_liquidity_for_3rdparty_rewards_impl(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
		rewards_asset_id: CurrencyIdOf<T>,
	) -> DispatchResult {
		if amount > BalanceOf::<T>::zero() {
			Self::set_liquidity_burning_checkpoint_for_schedule(
				user.clone(),
				liquidity_asset_id,
				amount,
				rewards_asset_id,
			)?;
			Pallet::<T>::deposit_event(Event::LiquidityDeactivated(
				user,
				liquidity_asset_id,
				amount,
			));
		}
		Ok(())
	}

	pub fn calculate_3rdparty_rewards_all(
		user: AccountIdOf<T>,
	) -> Vec<(CurrencyIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>)> {
		let result = RewardsInfoForScheduleRewards::<T>::iter_prefix(user.clone())
			.map(|((liq_token, reward_token), _)| {
				Self::calculate_3rdparty_rewards_amount(user.clone(), liq_token, reward_token)
					.map(|amount| (liq_token, reward_token, amount))
			})
			.collect::<Result<Vec<_>, _>>();
		let mut result = result.unwrap_or_default();
		result.sort();
		result
	}

	pub fn calculate_3rdparty_rewards_amount(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		rewards_asset_id: CurrencyIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		Self::ensure_3rdparty_rewards_enabled(liquidity_asset_id)?;

		if let Ok(info) = RewardsInfoForScheduleRewards::<T>::try_get(
			user.clone(),
			(liquidity_asset_id, rewards_asset_id),
		) {
			let current_rewards = if info.activated_amount == BalanceOf::<T>::zero() {
				BalanceOf::<T>::zero()
			} else {
				let calc = RewardsCalculator::schedule_rewards::<T>(
					user.clone(),
					liquidity_asset_id,
					rewards_asset_id,
				)?;
				calc.calculate_rewards().map_err(|err| Into::<Error<T>>::into(err))?
			};

			Ok(current_rewards
				.checked_add(&info.rewards_not_yet_claimed)
				.and_then(|v| v.checked_sub(&info.rewards_already_claimed))
				.ok_or(Error::<T>::CalculateRewardsMathError)?)
		} else {
			Ok(BalanceOf::<T>::zero())
		}
	}

	fn pallet_account() -> T::AccountId {
		PALLET_ID.into_account_truncating()
	}

	pub fn session_index() -> u32 {
		Self::get_current_rewards_time().unwrap_or_default()
	}

	pub fn rewards_period() -> u32 {
		T::RewardsDistributionPeriod::get()
	}

	pub fn is_new_session() -> bool {
		let block_nr = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();
		(block_nr + 1) % Self::rewards_period() == 0u32
	}

	fn native_token_id() -> CurrencyIdOf<T> {
		<T as Config>::NativeCurrencyId::get()
	}

	fn get_pool_rewards(liquidity_asset_id: CurrencyIdOf<T>) -> Result<U256, DispatchError> {
		Ok(PromotedPoolRewards::<T>::get()
			.get(&liquidity_asset_id)
			.map(|v| v.rewards)
			.ok_or(Error::<T>::NotAPromotedPool)?)
	}

	fn get_current_rewards_time() -> Result<u32, DispatchError> {
		<frame_system::Pallet<T>>::block_number()
			.saturated_into::<u32>()
			.checked_add(1)
			.and_then(|v| v.checked_div(T::RewardsDistributionPeriod::get()))
			.ok_or(DispatchError::from(Error::<T>::CalculateRewardsMathError))
	}

	fn ensure_native_rewards_enabled(
		liquidity_asset_id: CurrencyIdOf<T>,
	) -> Result<(), DispatchError> {
		ensure!(Self::get_pool_rewards(liquidity_asset_id).is_ok(), Error::<T>::NotAPromotedPool);
		Ok(())
	}

	fn ensure_3rdparty_rewards_enabled(
		liquidity_asset_id: CurrencyIdOf<T>,
	) -> Result<(), DispatchError> {
		ensure!(
			RewardTokensPerPool::<T>::iter_prefix_values(liquidity_asset_id)
				.next()
				.is_some(),
			Error::<T>::NotAPromotedPool
		);
		Ok(())
	}

	fn set_liquidity_minting_checkpoint(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_added: BalanceOf<T>,
	) -> DispatchResult {
		Self::ensure_native_rewards_enabled(liquidity_asset_id)?;

		{
			let calc = RewardsCalculator::mining_rewards::<T>(user.clone(), liquidity_asset_id)?;
			let rewards_info = calc
				.activate_more(liquidity_assets_added)
				.map_err(|err| Into::<Error<T>>::into(err))?;

			RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info);
		}

		TotalActivatedLiquidity::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_add(&liquidity_assets_added) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		Ok(())
	}

	fn set_liquidity_minting_checkpoint_3rdparty(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_added: BalanceOf<T>,
		liquidity_assets_reward: CurrencyIdOf<T>,
	) -> DispatchResult {
		Self::ensure_3rdparty_rewards_enabled(liquidity_asset_id)?;

		ScheduleRewardsCalculator::<T>::update_cumulative_rewards(
			liquidity_asset_id,
			liquidity_assets_reward,
		);
		{
			let calc = RewardsCalculator::schedule_rewards::<T>(
				user.clone(),
				liquidity_asset_id,
				liquidity_assets_reward,
			)?;
			let rewards_info = calc
				.activate_more(liquidity_assets_added)
				.map_err(|err| Into::<Error<T>>::into(err))?;
			RewardsInfoForScheduleRewards::<T>::insert(
				user.clone(),
				(liquidity_asset_id, liquidity_assets_reward),
				rewards_info,
			);
		}

		ActivatedLiquidityForSchedules::<T>::try_mutate_exists(
			(user.clone(), liquidity_asset_id, liquidity_assets_reward),
			|v| {
				match v {
					Some(x) => {
						v.as_mut().map(|a| *a += liquidity_assets_added);
					},
					None => {
						*v = Some(liquidity_assets_added);
					},
				};
				Ok::<(), Error<T>>(())
			},
		)?;

		ScheduleRewardsCalculator::<T>::update_total_activated_liqudity(
			liquidity_asset_id,
			liquidity_assets_reward,
			liquidity_assets_added,
			true,
		);

		Ok(())
	}

	fn set_liquidity_burning_checkpoint(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_burned: BalanceOf<T>,
	) -> DispatchResult {
		Self::ensure_native_rewards_enabled(liquidity_asset_id)?;

		let rewards_info = RewardsInfo::<T>::try_get(user.clone(), liquidity_asset_id)
			.or(Err(DispatchError::from(Error::<T>::MissingRewardsInfoError)))?;
		ensure!(
			rewards_info.activated_amount >= liquidity_assets_burned,
			Error::<T>::NotEnoughAssets
		);

		ensure!(
			rewards_info
				.activated_amount
				.checked_sub(&ActivatedNativeRewardsLiq::<T>::get(user.clone(), liquidity_asset_id))
				.ok_or(Error::<T>::MathOverflow)? >=
				liquidity_assets_burned,
			Error::<T>::LiquidityLockedIn3rdpartyRewards
		);

		let calc = RewardsCalculator::mining_rewards::<T>(user.clone(), liquidity_asset_id)?;
		let rewards_info = calc
			.activate_less(liquidity_assets_burned)
			.map_err(|err| Into::<Error<T>>::into(err))?;

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info);

		TotalActivatedLiquidity::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_sub(&liquidity_assets_burned) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		<T as Config>::ActivationReservesProvider::deactivate(
			liquidity_asset_id,
			&user,
			liquidity_assets_burned,
		);

		Ok(())
	}

	fn set_liquidity_burning_checkpoint_for_schedule(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
		liquidity_assets_burned: BalanceOf<T>,
		reward_token: CurrencyIdOf<T>,
	) -> DispatchResult {
		Self::ensure_3rdparty_rewards_enabled(liquidity_asset_id)?;
		ScheduleRewardsCalculator::<T>::update_cumulative_rewards(liquidity_asset_id, reward_token);

		let calc = RewardsCalculator::schedule_rewards::<T>(
			user.clone(),
			liquidity_asset_id,
			reward_token,
		)?;

		let rewards_info = calc
			.activate_less(liquidity_assets_burned)
			.map_err(|err| Into::<Error<T>>::into(err))?;

		RewardsInfoForScheduleRewards::<T>::insert(
			user.clone(),
			(liquidity_asset_id, reward_token),
			rewards_info,
		);

		ScheduleRewardsCalculator::<T>::update_total_activated_liqudity(
			liquidity_asset_id,
			reward_token,
			liquidity_assets_burned,
			false,
		);

		ActivatedLiquidityForSchedules::<T>::try_mutate_exists(
			(user.clone(), liquidity_asset_id, reward_token),
			|v| {
				v.and_then(|a| {
					a.checked_sub(&liquidity_assets_burned).and_then(|val| {
						if val > BalanceOf::<T>::zero() {
							*v = Some(val);
						} else {
							*v = None;
						}
						Some(val)
					})
				})
				.ok_or(Error::<T>::MathOverflow)
			},
		)?;

		if let None = ActivatedLiquidityForSchedules::<T>::iter_prefix_values((
			user.clone(),
			liquidity_asset_id,
		))
		.next()
		{
			let amount = ActivatedLockedLiquidityForSchedules::<T>::mutate(
				user.clone(),
				liquidity_asset_id,
				|val| {
					let prev = *val;
					*val = BalanceOf::<T>::zero();
					prev
				},
			);

			<T as Config>::ActivationReservesProvider::deactivate(
				liquidity_asset_id,
				&user,
				amount,
			);

			let _ =
				ActivatedNativeRewardsLiq::<T>::mutate(user.clone(), liquidity_asset_id, |val| {
					let prev = *val;
					*val = BalanceOf::<T>::zero();
					prev
				});
		}

		Ok(())
	}

	fn claim_schedule_rewards_all_impl(
		user: T::AccountId,
		liquidity_asset_id: CurrencyIdOf<T>,
		reward_token: CurrencyIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		Self::ensure_3rdparty_rewards_enabled(liquidity_asset_id)?;

		let calc = RewardsCalculator::schedule_rewards::<T>(
			user.clone(),
			liquidity_asset_id,
			reward_token,
		)?;
		let (rewards_info, total_available_rewards) =
			calc.claim_rewards().map_err(|err| Into::<Error<T>>::into(err))?;

		ensure!(
			total_available_rewards > Zero::zero(),
			Error::<T>::NoThirdPartyPartyRewardsToClaim
		);

		<T as Config>::Currency::transfer(
			reward_token.into(),
			&Self::pallet_account(),
			&user,
			total_available_rewards,
			ExistenceRequirement::KeepAlive,
		)?;

		RewardsInfoForScheduleRewards::<T>::insert(
			user.clone(),
			(liquidity_asset_id, reward_token),
			rewards_info,
		);

		Pallet::<T>::deposit_event(Event::ThirdPartyRewardsClaimed(
			user,
			liquidity_asset_id,
			reward_token,
			total_available_rewards,
		));

		Ok(total_available_rewards)
	}

	pub(crate) fn reward_pool_impl(
		sender: T::AccountId,

		pool: (CurrencyIdOf<T>, CurrencyIdOf<T>),
		token_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
		schedule_end: SessionId,
	) -> DispatchResult {
		let liquidity_token_id = <T as Config>::ValuationApi::get_liquidity_asset(pool.0, pool.1)
			.map_err(|_| Error::<T>::PoolDoesNotExist)?;

		let current_session = Self::session_index();
		ensure!(
			schedule_end.saturated_into::<u32>() > current_session,
			Error::<T>::CannotScheduleRewardsInPast
		);

		let amount_per_session: BalanceOf<T> = schedule_end
			.saturated_into::<u32>()
			.checked_sub(current_session)
			.and_then(|v| Into::<u128>::into(amount).checked_div(v.into()))
			.ok_or(Error::<T>::MathOverflow)?
			.try_into()
			.or(Err(Error::<T>::MathOverflow))?;

		ensure!(
			Self::verify_rewards_min_amount(token_id, amount_per_session),
			Error::<T>::TooLittleRewards
		);

		ensure!(Self::verify_rewards_min_volume(token_id), Error::<T>::TooSmallVolume);

		RewardTokensPerPool::<T>::insert(liquidity_token_id, token_id, ());

		T::Currency::transfer(
			token_id.into(),
			&sender,
			&Self::pallet_account(),
			amount,
			ExistenceRequirement::KeepAlive,
		)?;

		let head = SchedulesListMetadata::<T>::get().head;
		let tail = SchedulesListMetadata::<T>::get().tail;

		let schedule = Schedule {
			scheduled_at: Self::session_index(),
			last_session: schedule_end,
			liq_token: liquidity_token_id,
			reward_token: token_id,
			amount_per_session,
		};

		match (head, tail) {
			(None, None) => {
				// first schedule
				RewardsSchedulesList::<T>::insert(0, (schedule, None::<ScheduleId>));
				SchedulesListMetadata::<T>::mutate(|s| {
					s.head = Some(0);
					s.tail = Some(0);
					s.count = 1;
				});
			},
			(Some(_head), Some(tail)) => {
				RewardsSchedulesList::<T>::mutate(tail, |info| {
					if let Some((_schedule, next)) = info.as_mut() {
						*next = Some(tail + 1u64)
					}
				});
				RewardsSchedulesList::<T>::insert(tail + 1, (schedule, None::<ScheduleId>));
				SchedulesListMetadata::<T>::try_mutate(|s| {
					if s.count < T::RewardsSchedulesLimit::get().into() {
						s.tail = Some(tail + 1);
						s.count += 1;
						Ok(s.count)
					} else {
						Err(Error::<T>::TooManySchedules)
					}
				})?;
			},
			_ => {}, // invariant assures this will never happen
		}

		Ok(())
	}

	fn verify_rewards_min_amount(
		token_id: CurrencyIdOf<T>,
		amount_per_session: BalanceOf<T>,
	) -> bool {
		if <T as Config>::ValuationApi::valuate_liquidity_token(token_id, amount_per_session).into() >=
			T::Min3rdPartyRewardValutationPerSession::get()
		{
			return true
		}

		if token_id == Self::native_token_id() &&
			amount_per_session.into() >= T::Min3rdPartyRewardValutationPerSession::get()
		{
			return true
		}

		if <T as Config>::ValuationApi::valuate_non_liquidity_token(token_id, amount_per_session)
			.into() >= T::Min3rdPartyRewardValutationPerSession::get()
		{
			return true
		}

		return false
	}

	fn verify_rewards_min_volume(token_id: CurrencyIdOf<T>) -> bool {
		if token_id == Self::native_token_id() {
			return true
		}

		if let Some((mga_reserves, _)) = <T as Config>::ValuationApi::get_pool_state(token_id) {
			return mga_reserves.into() >= T::Min3rdPartyRewardVolume::get()
		}

		if let Ok((mga_reserves, _)) =
			<T as Config>::ValuationApi::get_reserves(Self::native_token_id(), token_id)
		{
			return mga_reserves.into() >= T::Min3rdPartyRewardVolume::get()
		}

		return false
	}
}

impl<T: Config> ProofOfStakeRewardsApi<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>> for Pallet<T> {
	#[cfg(feature = "runtime-benchmarks")]
	fn enable_3rdparty_rewards(
		account: T::AccountId,
		pool: (CurrencyIdOf<T>, CurrencyIdOf<T>),
		reward_token_id: CurrencyIdOf<T>,
		last_session: u32,
		amount: BalanceOf<T>,
	) {
		let liquidity_token_id =
			<T as Config>::ValuationApi::get_liquidity_asset(pool.0, pool.1).expect("pool exist");
		Pallet::<T>::reward_pool_impl(
			account.clone(),
			pool,
			reward_token_id,
			amount,
			last_session.into(),
		)
		.expect("call should pass");
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn activate_liquidity_for_3rdparty_rewards(
		account: T::AccountId,
		liquidity_token: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
		reward_token_id: CurrencyIdOf<T>,
	) {
		Pallet::<T>::activate_liquidity_for_3rdparty_rewards_impl(
			account,
			liquidity_token,
			amount,
			ThirdPartyActivationKind::ActivateKind(None),
			reward_token_id,
		)
		.expect("call should pass")
	}

	fn enable(liquidity_token_id: CurrencyIdOf<T>, weight: u8) {
		PromotedPoolRewards::<T>::mutate(|promoted_pools| {
			promoted_pools
				.entry(liquidity_token_id)
				.and_modify(|info| info.weight = weight)
				.or_insert(PromotedPools { weight, rewards: U256::zero() });
		});
		Pallet::<T>::deposit_event(Event::PoolPromotionUpdated(liquidity_token_id, Some(weight)));
	}

	fn disable(liquidity_token_id: CurrencyIdOf<T>) {
		PromotedPoolRewards::<T>::mutate(|promoted_pools| {
			if let Some(info) = promoted_pools.get_mut(&liquidity_token_id) {
				info.weight = 0;
			}
		});
		Pallet::<T>::deposit_event(Event::PoolPromotionUpdated(liquidity_token_id, None));
	}

	fn is_enabled(liquidity_token_id: CurrencyIdOf<T>) -> bool {
		PromotedPoolRewards::<T>::get().contains_key(&liquidity_token_id)
	}

	fn claim_rewards_all(
		user: T::AccountId,
		liquidity_asset_id: CurrencyIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		Self::ensure_native_rewards_enabled(liquidity_asset_id)?;

		let calc = RewardsCalculator::mining_rewards::<T>(user.clone(), liquidity_asset_id)?;
		let (rewards_info, total_available_rewards) =
			calc.claim_rewards().map_err(|err| Into::<Error<T>>::into(err))?;

		<T as Config>::Currency::transfer(
			Self::native_token_id().into(),
			&<T as Config>::LiquidityMiningIssuanceVault::get(),
			&user,
			total_available_rewards,
			ExistenceRequirement::KeepAlive,
		)?;

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info);

		Pallet::<T>::deposit_event(Event::RewardsClaimed(
			user,
			liquidity_asset_id,
			total_available_rewards,
		));

		Ok(total_available_rewards)
	}

	fn activate_liquidity(
		user: T::AccountId,
		liquidity_asset_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		Self::activate_liquidity_for_native_rewards_impl(
			user,
			liquidity_asset_id,
			amount,
			use_balance_from,
		)
	}

	fn deactivate_liquidity(
		user: T::AccountId,
		liquidity_asset_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Self::deactivate_liquidity_for_native_rewards_impl(user, liquidity_asset_id, amount)
	}

	fn calculate_rewards_amount(
		user: AccountIdOf<T>,
		liquidity_asset_id: CurrencyIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		Self::calculate_native_rewards_amount(user, liquidity_asset_id)
	}
}

impl<T: Config> LiquidityMiningApi<BalanceOf<T>> for Pallet<T> {
	/// Distributs liquidity mining rewards between all the activated tokens based on their weight
	fn distribute_rewards(liquidity_mining_rewards: BalanceOf<T>) {
		let _ = PromotedPoolRewards::<T>::try_mutate(|promoted_pools| -> DispatchResult {
			// benchmark with max of X prom pools
			let activated_pools: Vec<_> = promoted_pools
				.clone()
				.into_iter()
				.filter_map(|(token_id, info)| {
					let activated_amount = Self::total_activated_amount(token_id);
					if activated_amount > BalanceOf::<T>::zero() && info.weight > 0 {
						Some((token_id, info.weight, info.rewards, activated_amount))
					} else {
						None
					}
				})
				.collect();

			let maybe_total_weight = activated_pools.iter().try_fold(
				0u64,
				|acc, &(_token_id, weight, _rewards, _activated_amount)| {
					acc.checked_add(weight.into())
				},
			);

			for (token_id, weight, rewards, activated_amount) in activated_pools {
				let liquidity_mining_issuance_for_pool = match maybe_total_weight {
					Some(total_weight) if !total_weight.is_zero() =>
						Perbill::from_rational(weight.into(), total_weight)
							.mul_floor(liquidity_mining_rewards),
					_ => BalanceOf::<T>::zero(),
				};

				let rewards_for_liquidity: U256 =
					U256::from(liquidity_mining_issuance_for_pool.into())
						.checked_mul(U256::from(u128::MAX))
						.and_then(|x| x.checked_div(activated_amount.into().into()))
						.and_then(|x| x.checked_add(rewards))
						.ok_or(Error::<T>::MathError)?;

				promoted_pools
					.entry(token_id.clone())
					.and_modify(|info| info.rewards = rewards_for_liquidity);
			}
			Ok(())
		});
	}
}
