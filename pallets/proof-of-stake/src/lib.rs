#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	assert_ok,
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::Contains,
	PalletId,
};
use frame_system::ensure_signed;
use sp_core::U256;
// TODO documentation!
use codec::FullCodec;
use frame_support::{
	pallet_prelude::*,
	traits::{tokens::currency::MultiTokenCurrency, ExistenceRequirement, Get, WithdrawReasons},
	transactional, Parameter,
};
use frame_system::pallet_prelude::*;
use mangata_support::traits::{
	ActivationReservesProviderTrait, ComputeIssuance, CumulativeWorkRewardsApi,
	GetMaintenanceStatusTrait, PoolCreateApi, PreValidateSwaps, ProofOfStakeRewardsApi,
	XykFunctionsTrait,
};
use mangata_types::{multipurpose_liquidity::ActivateKind, Balance, TokenId};
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use pallet_issuance::{ActivatedPoolQueryApi, PoolPromoteApi};
use pallet_vesting_mangata::MultiTokenVestingLocks;
use sp_arithmetic::{helpers_128bit::multiply_by_rational_with_rounding, per_things::Rounding};
use sp_runtime::{
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member,
		SaturatedConversion, Zero,
	},
	Permill,
};
use sp_std::{
	collections::btree_set::BTreeSet,
	convert::{TryFrom, TryInto},
	fmt::Debug,
	ops::Div,
	prelude::*,
	vec::Vec,
};

/// Stores all the information required for non iterative rewards calculation between
/// last_checkpoint and particular subsequent block ('now' in most cases)
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct RewardInfo {
	// amount of activated token
	pub activated_amount: u128,
	// when doing checkpoint we need to store rewards up to this point
	pub rewards_not_yet_claimed: u128,
	// there is no checkpoint during for claim_rewards
	pub rewards_already_claimed: u128,
	// block number of last checkpoint
	pub last_checkpoint: u32,
	// ration betwen rewards : liquidity (as in table)
	pub pool_ratio_at_last_checkpoint: U256,
	// related to the table in the doc
	pub missing_at_last_checkpoint: U256,
}

impl RewardInfo{
    // fn activate_more()
}

pub(crate) const LOG_TARGET: &str = "proof-of-stake";

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

// Quocient ratio in which liquidity minting curve is rising
const Q: f64 = 1.03;
// Precision used in rewards calculation rounding
const REWARDS_PRECISION: u32 = 10000;

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type ActivationReservesProvider: ActivationReservesProviderTrait<
			AccountId = Self::AccountId,
		>;
		type NativeCurrencyId: Get<TokenId>;
		type Xyk: XykFunctionsTrait<Self::AccountId>;
		type PoolPromoteApi: ComputeIssuance + PoolPromoteApi;
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;
		#[pallet::constant]
		/// The account id that holds the liquidity mining issuance
		type LiquidityMiningIssuanceVault: Get<Self::AccountId>;
		#[pallet::constant]
		type RewardsDistributionPeriod: Get<u32>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// Not enought assets
		NotEnoughAssets,
		/// Division by zero
		DivisionByZero,
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
		CalculateCumulativeWorkMaxRatioMathError,
		CalculateRewardsAllMathError,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolPromotionUpdated(TokenId, Option<u8>),
		LiquidityActivated(T::AccountId, TokenId, Balance),
		LiquidityDeactivated(T::AccountId, TokenId, Balance),
		RewardsClaimed(T::AccountId, TokenId, Balance),
	}

	#[pallet::storage]
	#[pallet::getter(fn get_rewards_info)]
	pub type RewardsInfo<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		TokenId,
		RewardInfo,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_mining_active_pool_v2)]
	pub type LiquidityMiningActivePoolV2<T: Config> =
		StorageMap<_, Twox64Concat, TokenId, u128, ValueQuery>;

	// XYK extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<<T as Config>::WeightInfo>::proof_of_stake_compound_rewards())]
		#[transactional]
		pub fn compound_rewards(
			origin: OriginFor<T>,
			liquidity_asset_id: TokenId,
			amount_permille: Permill,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<T::Xyk as XykFunctionsTrait<T::AccountId>>::do_compound_rewards(
				sender,
				liquidity_asset_id.into(),
				amount_permille,
			)?;

			Ok(().into())
		}

		#[transactional]
		#[pallet::call_index(1)]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_rewards_v2())]
		pub fn claim_rewards_v2(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as ProofOfStakeRewardsApi<T::AccountId>>::claim_rewards_v2(
				sender,
				liquidity_token_id,
				amount,
			)?;

			Ok(())
		}

		#[transactional]
		#[pallet::call_index(2)]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_rewards_v2())]
		pub fn claim_rewards_all_v2(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as ProofOfStakeRewardsApi<T::AccountId>>::claim_rewards_all_v2(
				sender,
				liquidity_token_id,
			)?;

			Ok(())
		}

		// Disabled pool demotion
		#[pallet::call_index(3)]
		#[pallet::weight(<<T as Config>::WeightInfo>::update_pool_promotion())]
		pub fn update_pool_promotion(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			liquidity_mining_issuance_weight: u8,
		) -> DispatchResult {
			ensure_root(origin)?;

			<Self as ProofOfStakeRewardsApi<T::AccountId>>::update_pool_promotion(
				liquidity_token_id,
				Some(liquidity_mining_issuance_weight),
			)
		}

		#[transactional]
		#[pallet::call_index(4)]
		#[pallet::weight(<<T as Config>::WeightInfo>::activate_liquidity_v2())]
		pub fn activate_liquidity_v2(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
			use_balance_from: Option<ActivateKind>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as ProofOfStakeRewardsApi<T::AccountId>>::activate_liquidity_v2(
				sender,
				liquidity_token_id,
				amount,
				use_balance_from,
			)
		}

		#[transactional]
		#[pallet::call_index(5)]
		#[pallet::weight(<<T as Config>::WeightInfo>::deactivate_liquidity_v2())]
		pub fn deactivate_liquidity_v2(
			origin: OriginFor<T>,
			liquidity_token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as ProofOfStakeRewardsApi<T::AccountId>>::deactivate_liquidity_v2(
				sender,
				liquidity_token_id,
				amount,
			)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn native_token_id() -> TokenId {
		<T as Config>::NativeCurrencyId::get()
	}

	fn get_pool_rewards(liquidity_asset_id: TokenId) -> Result<U256, sp_runtime::DispatchError> {
		<T as Config>::PoolPromoteApi::get_pool_rewards_v2(liquidity_asset_id)
				.ok_or( DispatchError::from(Error::<T>::NotAPromotedPool))
	}

	fn get_current_rewards_time() -> Result<u32, sp_runtime::DispatchError>{
		<frame_system::Pallet<T>>::block_number()
			.saturated_into::<u32>()
			.checked_add(1)
			.and_then(|v| v.checked_div(T::RewardsDistributionPeriod::get()))
			.ok_or(DispatchError::from(Error::<T>::CalculateRewardsMathError))
	}

	fn ensure_is_promoted_pool(liquidity_asset_id: TokenId) -> Result<(), DispatchError> {
		if Self::get_pool_rewards(liquidity_asset_id).is_ok(){
			Ok(())
		}else{
			Err(DispatchError::from(Error::<T>::NotAPromotedPool))
		}
	}

	// remove   liquidity_asset_id
	fn calculate_rewards_v2(
		liquidity_assets_amount: u128,
		last_checkpoint: u32,
		pool_rewards_ratio: U256,
		missing_at_last_checkpoint: U256,
		pool_rewards_ratio_current: U256,
	) -> Result<Balance, DispatchError> {

		let current_time: u32 = Self::get_current_rewards_time()?;

		let time_passed = current_time
			.checked_sub(last_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::PastTimeCalculation))?;

		let pool_rewards_ratio_new = pool_rewards_ratio_current
			.checked_sub(pool_rewards_ratio)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?;

		let user_rewards_base: U256 = U256::from(liquidity_assets_amount)
			.checked_mul(pool_rewards_ratio_new) // TODO: please add UT and link it in this comment
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?
			.checked_div(U256::from(u128::MAX)) // always fit into u128
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?;

		let (cumulative_work, cummulative_work_max_possible) =
			Self::calculate_cumulative_work_max_ratio(
				liquidity_assets_amount,
				time_passed,
				missing_at_last_checkpoint,
			)?;

		let current_rewards = user_rewards_base
			.checked_mul(cumulative_work)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?
			.checked_div(cummulative_work_max_possible)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?
			.try_into()
			.map_err(|_| DispatchError::from(Error::<T>::CalculateRewardsMathError))?;

		Ok(current_rewards)
	}

	// MAX: 0R 0W

	fn calculate_cumulative_work_max_ratio(
		liquidity_assets_amount: u128,
		time_passed: u32,
		missing_at_last_checkpoint: U256,
	) -> Result<(U256, U256), DispatchError> {
		let mut cummulative_work = U256::from(0);
		let mut cummulative_work_max_possible_for_ratio = U256::from(1);

		if time_passed != 0 && liquidity_assets_amount != 0 {
			let liquidity_assets_amount_u256: U256 = liquidity_assets_amount.into();

			// whole formula: 	missing_at_last_checkpoint*106/6 - missing_at_last_checkpoint*106*precision/6/q_pow
			// q_pow is multiplied by precision, thus there needs to be *precision in numenator as well

			cummulative_work_max_possible_for_ratio =
				liquidity_assets_amount_u256.checked_mul(U256::from(time_passed)).ok_or_else(
					|| DispatchError::from(Error::<T>::CalculateCumulativeWorkMaxRatioMathError),
				)?;

			// whole formula: 	missing_at_last_checkpoint*Q*100/(Q*100-100) - missing_at_last_checkpoint*Q*100/(Q*100-100)*REWARDS_PRECISION/q_pow
			// q_pow is multiplied by precision, thus there needs to be *precision in numenator as well
			let base = missing_at_last_checkpoint
				.checked_mul(U256::from(libm::floor(Q * 100_f64) as u128))
				.ok_or_else(|| {
					DispatchError::from(Error::<T>::CalculateCumulativeWorkMaxRatioMathError)
				})?
				.checked_div(U256::from(libm::floor(Q * 100_f64 - 100_f64) as u128))
				.ok_or_else(|| {
					DispatchError::from(Error::<T>::CalculateCumulativeWorkMaxRatioMathError)
				})?;

			let q_pow = Self::calculate_q_pow(
				Q,
				time_passed
					.checked_add(1)
					.ok_or(Error::<T>::CalculateCumulativeWorkMaxRatioMathError)?,
			);

			let cummulative_missing_new =
				base - base * U256::from(REWARDS_PRECISION) / q_pow - missing_at_last_checkpoint;

			cummulative_work = cummulative_work_max_possible_for_ratio
				.checked_sub(cummulative_missing_new)
				.ok_or_else(|| {
					DispatchError::from(Error::<T>::CalculateCumulativeWorkMaxRatioMathError)
				})?;
		}

		Ok((cummulative_work, cummulative_work_max_possible_for_ratio))
	}

	fn calculate_q_pow(q: f64, pow: u32) -> u128 {
		libm::floor(libm::pow(q, pow as f64) * REWARDS_PRECISION as f64) as u128
	}

	/// 0R 0W
	fn calculate_missing_at_checkpoint_v2(
		time_passed: u32,
		missing_at_last_checkpoint: U256,
	) -> Result<U256, DispatchError> {
		let q_pow = Self::calculate_q_pow(Q, time_passed);

		let missing_at_checkpoint: U256 =
			missing_at_last_checkpoint * U256::from(REWARDS_PRECISION) / q_pow;

		Ok(missing_at_checkpoint)
	}
}

impl<T: Config> ProofOfStakeRewardsApi<T::AccountId> for Pallet<T> {
	type Balance = Balance;

	type CurrencyId = TokenId;

	// MAX: 3R 2W
	fn claim_rewards_v2(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		mangata_amount: Self::Balance,
	) -> DispatchResult {
		let mangata_id: TokenId = Self::native_token_id();
		let claimable_rewards =
			<Self as ProofOfStakeRewardsApi<T::AccountId>>::calculate_rewards_amount_v2(
				user.clone(),
				liquidity_asset_id,
			)?;

		ensure!(mangata_amount <= claimable_rewards, Error::<T>::NotEnoughRewardsEarned);

		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);

		let mut not_yet_claimed_rewards = rewards_info.rewards_not_yet_claimed;
		let mut already_claimed_rewards = rewards_info.rewards_already_claimed;

		if mangata_amount <= not_yet_claimed_rewards {
			not_yet_claimed_rewards = not_yet_claimed_rewards
				.checked_sub(mangata_amount)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
		}
		// user is taking out more rewards then rewards from LP which was already removed from pool, additional work needs to be removed from pool and user
		else {
			// rewards to claim on top of rewards from LP which was already removed from pool
			let rewards_to_claim = mangata_amount
				.checked_sub(not_yet_claimed_rewards)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
			already_claimed_rewards = already_claimed_rewards
				.checked_add(rewards_to_claim)
				.ok_or_else(|| DispatchError::from(Error::<T>::MathOverflow))?;
			not_yet_claimed_rewards = 0_u128;
		}

		let rewards_info_new: RewardInfo = RewardInfo {
			activated_amount: rewards_info.activated_amount,
			rewards_not_yet_claimed: not_yet_claimed_rewards,
			rewards_already_claimed: already_claimed_rewards,
			last_checkpoint: rewards_info.last_checkpoint,
			pool_ratio_at_last_checkpoint: rewards_info.pool_ratio_at_last_checkpoint,
			missing_at_last_checkpoint: rewards_info.missing_at_last_checkpoint,
		};

		<T as Config>::Currency::transfer(
			mangata_id.into(),
			&<T as Config>::LiquidityMiningIssuanceVault::get(),
			&user,
			mangata_amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info_new);

		Pallet::<T>::deposit_event(Event::RewardsClaimed(user, liquidity_asset_id, mangata_amount));

		Ok(())
	}

	fn claim_rewards_all_v2(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
	) -> Result<Self::Balance, DispatchError> {
		let mangata_id: TokenId = Self::native_token_id();

		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);
		let pool_rewards_ratio_current = Self::get_pool_rewards(liquidity_asset_id)?;

		Self::ensure_is_promoted_pool(liquidity_asset_id)?;
		let current_rewards = Self::calculate_rewards_v2(
			rewards_info.activated_amount,
			rewards_info.last_checkpoint,
			rewards_info.pool_ratio_at_last_checkpoint,
			rewards_info.missing_at_last_checkpoint,
			pool_rewards_ratio_current,
		)?;

		let rewards_not_yet_claimed = rewards_info.rewards_not_yet_claimed;
		let rewards_already_claimed = rewards_info.rewards_already_claimed;

		let total_available_rewards = current_rewards
			.checked_add(rewards_not_yet_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsAllMathError))?
			.checked_sub(rewards_already_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsAllMathError))?;

		let rewards_info_new: RewardInfo = RewardInfo {
			activated_amount: rewards_info.activated_amount,
			rewards_not_yet_claimed: 0_u128,
			rewards_already_claimed: current_rewards,
			last_checkpoint: rewards_info.last_checkpoint,
			pool_ratio_at_last_checkpoint: rewards_info.pool_ratio_at_last_checkpoint,
			missing_at_last_checkpoint: rewards_info.missing_at_last_checkpoint,
		};

		<T as Config>::Currency::transfer(
			mangata_id.into(),
			&<T as Config>::LiquidityMiningIssuanceVault::get(),
			&user,
			total_available_rewards.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info_new);

		Pallet::<T>::deposit_event(Event::RewardsClaimed(
			user,
			liquidity_asset_id,
			total_available_rewards,
		));

		Ok(total_available_rewards)
	}

	fn update_pool_promotion(
		liquidity_token_id: TokenId,
		liquidity_mining_issuance_weight: Option<u8>,
	) -> DispatchResult {
		<T as Config>::PoolPromoteApi::update_pool_promotion(
			liquidity_token_id,
			liquidity_mining_issuance_weight,
		);

		Pallet::<T>::deposit_event(Event::PoolPromotionUpdated(
			liquidity_token_id,
			liquidity_mining_issuance_weight,
		));

		Ok(())
	}

	fn activate_liquidity_v2(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		amount: Self::Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		ensure_is_promoted_pool(liquidity_asset_id)?;

		ensure!(
			<T as Config>::ActivationReservesProvider::can_activate(
				liquidity_asset_id,
				&user,
				amount,
				use_balance_from.clone()
			),
			Error::<T>::NotEnoughAssets
		);

		<Self as ProofOfStakeRewardsApi<T::AccountId>>::set_liquidity_minting_checkpoint_v2(
			user.clone(),
			liquidity_asset_id,
			amount,
			use_balance_from,
		)?;

		Pallet::<T>::deposit_event(Event::LiquidityActivated(user, liquidity_asset_id, amount));

		Ok(())
	}

	fn deactivate_liquidity_v2(
		user: T::AccountId,
		liquidity_asset_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult {
		Self::ensure_is_promoted_pool(liquidity_asset_id)?;

		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);

		ensure!(rewards_info.activated_amount >= amount, Error::<T>::NotEnoughAssets);

		<Self as ProofOfStakeRewardsApi<T::AccountId>>::set_liquidity_burning_checkpoint_v2(
			user.clone(),
			liquidity_asset_id,
			amount,
		)?;

		Pallet::<T>::deposit_event(Event::LiquidityDeactivated(user, liquidity_asset_id, amount));

		Ok(())
	}

	fn current_rewards_time() -> Option<u32> {
		Self::get_current_rewards_time().ok()	
	}

	fn calculate_rewards_amount_v2(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
	) -> Result<Balance, DispatchError> {
		let rewards_info: RewardInfo = Self::get_rewards_info(user, liquidity_asset_id);

		let liquidity_assets_amount: Balance = rewards_info.activated_amount;

		let mut current_rewards = 0;

		if liquidity_assets_amount != 0 {
			let pool_rewards_ratio_current = Self::get_pool_rewards(liquidity_asset_id)?;

			let last_checkpoint = rewards_info.last_checkpoint;
			let pool_ratio_at_last_checkpoint = rewards_info.pool_ratio_at_last_checkpoint;
			let missing_at_checkpoint = rewards_info.missing_at_last_checkpoint;

			Self::ensure_is_promoted_pool(liquidity_asset_id)?;
			current_rewards = Self::calculate_rewards_v2(
				liquidity_assets_amount,
				last_checkpoint,
				pool_ratio_at_last_checkpoint,
				missing_at_checkpoint,
				pool_rewards_ratio_current,
			)?;
		}

		let not_yet_claimed_rewards = rewards_info.rewards_not_yet_claimed;

		let already_claimed_rewards = rewards_info.rewards_already_claimed;

		let total_available_rewards = current_rewards
			.checked_add(not_yet_claimed_rewards)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?
			.checked_sub(already_claimed_rewards)
			.ok_or_else(|| DispatchError::from(Error::<T>::CalculateRewardsMathError))?;

		Ok(total_available_rewards)
	}

	fn set_liquidity_minting_checkpoint_v2(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
		liquidity_assets_added: Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		// last_checkpoint
		let current_time: u32 = Self::get_current_rewards_time()?;

		// pool_ratio_current
		let pool_ratio_current = Self::get_pool_rewards(liquidity_asset_id)?;

		let RewardInfo {
			last_checkpoint,
			pool_ratio_at_last_checkpoint,
			missing_at_last_checkpoint,
			activated_amount,
			rewards_not_yet_claimed,
			rewards_already_claimed,
		} = RewardsInfo::<T>::try_get(user.clone(), liquidity_asset_id)
			.unwrap_or(RewardInfo {
				activated_amount: 0_u128,
				rewards_not_yet_claimed: 0_u128,
				rewards_already_claimed: 0_u128,
				last_checkpoint: current_time,
				pool_ratio_at_last_checkpoint: pool_ratio_current,
				missing_at_last_checkpoint: U256::from(0u128),
			});


		// ACTIVATED
		let activated_amount_new = activated_amount
			.checked_add(liquidity_assets_added)
			.ok_or(Error::<T>::LiquidityCheckpointMathError)?;

		let time_passed = current_time
			.checked_sub(last_checkpoint)
			.ok_or(DispatchError::from(Error::<T>::PastTimeCalculation))?;
		let missing_at_checkpoint_new = Self::calculate_missing_at_checkpoint_v2(time_passed, missing_at_last_checkpoint)?
				.checked_add(U256::from(liquidity_assets_added))
				.ok_or(Error::<T>::LiquidityCheckpointMathError)?;

		let user_current_rewards = Self::calculate_rewards_v2(
				activated_amount,
				last_checkpoint,
				pool_ratio_at_last_checkpoint,
				missing_at_last_checkpoint,
				pool_ratio_current,
		)?;

		let total_available_rewards = user_current_rewards
			.checked_add(rewards_not_yet_claimed)
			.and_then(|v| v.checked_sub(rewards_already_claimed))
			.ok_or(Error::<T>::LiquidityCheckpointMathError)?;

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, RewardInfo {
			pool_ratio_at_last_checkpoint: pool_ratio_current,
			activated_amount: activated_amount_new,
			rewards_already_claimed: 0_u128,
			missing_at_last_checkpoint: missing_at_checkpoint_new,
			rewards_not_yet_claimed: total_available_rewards,
			last_checkpoint: current_time,
		});

		LiquidityMiningActivePoolV2::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_add(liquidity_assets_added) {
				*active_amount = val;
				Ok(())
			} else {
				Err(())
			}
		})
		.map_err(|_| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		// This must not fail due storage edits above
		<T as Config>::ActivationReservesProvider::activate(
			liquidity_asset_id,
			&user,
			liquidity_assets_added,
			use_balance_from,
		)?;

		Ok(())
	}

	fn set_liquidity_burning_checkpoint_v2(
		user: AccountIdOf<T>,
		liquidity_asset_id: TokenId,
		liquidity_assets_burned: Balance,
	) -> DispatchResult {
		let current_time: u32 = Self::get_current_rewards_time()?;
		let mut pool_ratio_current = Self::get_pool_rewards(liquidity_asset_id)?;

		let rewards_info: RewardInfo = Self::get_rewards_info(user.clone(), liquidity_asset_id);

		let last_checkpoint = rewards_info.last_checkpoint;
		let pool_ratio_at_last_checkpoint = rewards_info.pool_ratio_at_last_checkpoint;
		let missing_at_last_checkpoint = rewards_info.missing_at_last_checkpoint;
		let liquidity_assets_amount: Balance = rewards_info.activated_amount;

		let time_passed = current_time
			.checked_sub(last_checkpoint)
			.ok_or_else(|| DispatchError::from(Error::<T>::PastTimeCalculation))?;

		if time_passed == 0 {
			pool_ratio_current = pool_ratio_at_last_checkpoint;
		}

		let missing_at_checkpoint_new =
			Self::calculate_missing_at_checkpoint_v2(time_passed, missing_at_last_checkpoint)?;

		Self::ensure_is_promoted_pool(liquidity_asset_id)?;
		let user_current_rewards = Self::calculate_rewards_v2(
			liquidity_assets_amount,
			last_checkpoint,
			pool_ratio_at_last_checkpoint,
			missing_at_last_checkpoint,
			pool_ratio_current,
		)?;

		let activated_amount_new = liquidity_assets_amount
			.checked_sub(liquidity_assets_burned)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		let activated_amount_new_u256: U256 = activated_amount_new.into();

		let missing_at_checkpoint_after_burn: U256 = activated_amount_new_u256
			.checked_mul(missing_at_checkpoint_new)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?
			.checked_div(liquidity_assets_amount.into())
			.ok_or_else(|| DispatchError::from(Error::<T>::DivisionByZero))?;

		let total_available_rewards = user_current_rewards
			.checked_add(rewards_info.rewards_not_yet_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?
			.checked_sub(rewards_info.rewards_already_claimed)
			.ok_or_else(|| DispatchError::from(Error::<T>::LiquidityCheckpointMathError))?;

		let rewards_info_new: RewardInfo = RewardInfo {
			activated_amount: activated_amount_new,
			rewards_not_yet_claimed: total_available_rewards,
			rewards_already_claimed: 0_u128,
			last_checkpoint: current_time,
			pool_ratio_at_last_checkpoint: pool_ratio_current,
			missing_at_last_checkpoint: missing_at_checkpoint_after_burn,
		};

		RewardsInfo::<T>::insert(user.clone(), liquidity_asset_id, rewards_info_new);

		LiquidityMiningActivePoolV2::<T>::try_mutate(liquidity_asset_id, |active_amount| {
			if let Some(val) = active_amount.checked_sub(liquidity_assets_burned) {
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
}

impl<T: Config> CumulativeWorkRewardsApi for Pallet<T> {
	type AccountId = T::AccountId;

	fn get_pool_rewards_v2(liquidity_asset_id: TokenId) -> Option<U256> {
		Self::get_pool_rewards(liquidity_asset_id).ok()
	}

	fn claim_rewards_all_v2(
		user: Self::AccountId,
		liquidity_asset_id: TokenId,
	) -> Result<Balance, DispatchError> {
		<Self as ProofOfStakeRewardsApi<T::AccountId>>::claim_rewards_all_v2(
			user,
			liquidity_asset_id,
		)
	}

	fn set_liquidity_minting_checkpoint_v2(
		user: Self::AccountId,
		liquidity_asset_id: TokenId,
		liquidity_assets_added: Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		<Self as ProofOfStakeRewardsApi<T::AccountId>>::set_liquidity_minting_checkpoint_v2(
			user,
			liquidity_asset_id,
			liquidity_assets_added,
			use_balance_from,
		)
	}

	fn set_liquidity_burning_checkpoint_v2(
		user: Self::AccountId,
		liquidity_asset_id: TokenId,
		liquidity_assets_burned: Balance,
	) -> DispatchResult {
		<Self as ProofOfStakeRewardsApi<T::AccountId>>::set_liquidity_burning_checkpoint_v2(
			user,
			liquidity_asset_id,
			liquidity_assets_burned,
		)
	}
}

impl<T: Config> ActivatedPoolQueryApi for Pallet<T> {
	fn get_pool_activate_amount(liquidity_token_id: TokenId) -> Option<Balance> {
		LiquidityMiningActivePoolV2::<T>::try_get(liquidity_token_id).ok()
	}
}

impl<T: Config> mangata_support::traits::RewardsApi for Pallet<T> {
	type AccountId = T::AccountId;

	fn can_activate(liquidity_asset_id: TokenId) -> bool {
		Self::get_pool_rewards(liquidity_asset_id).is_ok()
	}

	fn activate_liquidity_tokens(
		user: &Self::AccountId,
		liquidity_asset_id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		<Self as ProofOfStakeRewardsApi<T::AccountId>>::activate_liquidity_v2(
			user.clone(),
			liquidity_asset_id,
			amount,
			Some(ActivateKind::AvailableBalance),
		)
	}

	fn update_pool_promotion(
		liquidity_token_id: TokenId,
		liquidity_mining_issuance_weight: Option<u8>,
	) {
		<T as Config>::PoolPromoteApi::update_pool_promotion(
			liquidity_token_id,
			liquidity_mining_issuance_weight,
		);

		Pallet::<T>::deposit_event(Event::PoolPromotionUpdated(
			liquidity_token_id,
			liquidity_mining_issuance_weight,
		));
	}
}
