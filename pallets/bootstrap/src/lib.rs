// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

//! # Bootstrap Module
//!
//! The Bootstrap module provides price discovery mechanism between two tokens. When bootstrap
//! is finished all provisioned tokens are collected and used for new liquidity token(pool) creation.
//! From that moment people that participated in bootstrap can claim their liquidity tokens share using
//! dedicated. Also since that moment its possible to exchange/trade tokens that were bootstraped using `Xyk `pallet.
//!
//!
//! ### Features
//!
//! * Bootstrap pallet is reusable** - after bootstrap between tokens `X` and `Y` is finished the following one can be scheduled (with different pair of tokens).
//! * After bootstrap is finished new liquidity token (`Z`) is created and [`pallet_xyk`] can be used to:
//!        * exchange/trade `X` and `Y` tokens
//!        * mint/burn `Z` tokens
//!
//! * Bootstrap state transition from [`BeforeStart`] -> [`Finished`] happens automatically thanks to substrate framework
//! hooks. **Only transition from `Finished` -> `BeforeStart` needs to be triggered manually because
//! cleaning up storage is complex operation and might not fit in a single block.(as it needs to
//! remove a lot of keys/value pair from the runtime storage)**
//!
//! # How to bootstrap
//! 1. Entity with sudo privileges needs to use [`Pallet::schedule_bootstrap`] to initiate new bootstrap
//!
//! 1.1 [**optional**] depending on fact if [`BootstrapPhase::Whitelist`] is enabled entity
//!   with sudo privileges can whitelist particular users using [`Pallet::whitelist_accounts`]
//!
//! 1.2 [**optional**] [`Pallet::update_promote_bootstrap_pool`] can be used to enable or disable
//!   automatic pool promotion of liquidity pool.
//!
//! 1.3 [**optional**] [`Pallet::cancel_bootstrap`] can be used to cancel bootstrap event
//!
//! 2. When blockchain reaches block that is scheduled as start od the bootstrap participation is
//!    automatically enabled:
//!    * in [`BootstrapPhase::Whitelist`] phase only whitelisted accounts [`Pallet::whitelist_accounts`]
//!    can participate
//!    * in [`BootstrapPhase::Public`] phase everyone can participate
//!
//! 3. When blockchain reaches block:
//! ```ignore
//!  current_block_nr > bootstrap_start_block + whitelist_phase_length + public_phase_length
//! ```
//!
//! Bootstrap is automatically finished and following participations will not be accepted. Also new
//! liquidity pool is created from all the tokens gathered during bootstrap (see [`Valuations`]). `TokenId`
//! of newly created liquidity token as well as amount of minted tokens is persisted into [`MintedLiquidity`]
//! storage item. All the liquidity token minted as a result of pool creation are now stored in
//! bootstrap pallet account.
//!
//! 4. Accounts that participated in bootstrap can claim their liquidity pool share. Share is
//!    calculated proportionally based on provisioned amount. One can use one of below extrinsics to
//!    claim rewards:
//!    * [`Pallet::claim_liquidity_tokens`]
//!    * [`Pallet::claim_and_activate_liquidity_tokens`]
//!
//! 5. When every participant of the bootstrap has claimed their liquidity tokens entity with sudo
//!    rights can [`Pallet::finalize`] whole bootstrap event. If there are some accounts that still
//!    hasnt claim their tokens [`Pallet::claim_liquidity_tokens_for_account`] can be used to do
//!    that in behalf of these accounts. When [`Pallet::finalize`] results with [`Event::BootstrapFinalized`]
//!    Bootstrap is finalized and another bootstrap can be scheduled (as described in 1st point).
//!
//! Bootstrap has specific lifecycle as presented below:
//! ```plantuml
//! @startuml
//! [*] --> BeforeStart
//! BeforeStart --> Whitelist
//! BeforeStart --> Public
//! Whitelist --> Public
//! Public --> Finished
//! Finished --> BeforeStart
//! @enduml
//! ```
//!
//! # API
//!
//! ## Runtime Storage Entries
//!
//! - [`Provisions`] - stores information about who provisioned what (non vested tokens)
//!
//! - [`VestedProvisions`] - stores information about who provisioned what (vested tokens)
//!
//! - [`WhitelistedAccount`] - list of accounts allowed to participate in [`BootstrapPhase::Whitelist`]
//!
//! - [`Phase`] - current state of bootstrap
//!
//! - [`Valuations`] - sum of all provisions in active bootstrap
//!
//! - [`BootstrapSchedule`] - parameters of active bootstrap stored as for more details check [`Pallet::schedule_bootstrap`]
//!
//!  ```ignore
//!  [
//!            block_nr: T::BlockNumber,
//!            first_token_id: u32,
//!            second_token_id: u32,
//!            [
//!                    ratio_numerator:u128,
//!                    ratio_denominator:u128
//!            ]
//!  ]
//!  ```
//!
//! - [`ClaimedRewards`] - how many liquidity tokens has user already **after** bootstrap
//! [`BootstrapPhase::Public`] period has finished.
//!
//! - [`ProvisionAccounts`] - list of participants that hasnt claim their tokens yet
//!
//! - [`ActivePair`] - bootstraped pair of tokens
//!
//! ## Extrinsics
//!
//! * [`Pallet::schedule_bootstrap`]
//! * [`Pallet::whitelist_accounts`]
//! * [`Pallet::update_promote_bootstrap_pool`]
//! * [`Pallet::cancel_bootstrap`]
//! * [`Pallet::provision`]
//! * [`Pallet::claim_liquidity_tokens`]
//! * [`Pallet::claim_liquidity_tokens_for_account`]
//! * [`Pallet::claim_and_activate_liquidity_tokens`]
//! * [`Pallet::finalize`]
//!
//! for more details see [click](#how-to-bootstrap)
//!
//!
use frame_support::pallet_prelude::*;

use frame_support::{
	codec::{Decode, Encode},
	traits::{
		tokens::currency::MultiTokenCurrency, Contains, ExistenceRequirement, Get, StorageVersion,
	},
	transactional, PalletId,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
use mangata_types::{Balance, TokenId};
use mangata_support::traits::{GetMaintenanceStatusTrait,AssetRegistryApi, PoolCreateApi, RewardsApi};
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use pallet_vesting_mangata::MultiTokenVestingLocks;
use scale_info::TypeInfo;
use sp_arithmetic::{helpers_128bit::multiply_by_rational_with_rounding, per_things::Rounding};
use sp_core::U256;
use sp_io::KillStorageResult;
use sp_runtime::traits::{AccountIdConversion, CheckedAdd, One, SaturatedConversion, Saturating};
use sp_std::{convert::TryInto, prelude::*};

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
	Vested(BlockNrAsBalance, BlockNrAsBalance),
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	#[pallet::storage_version(STORAGE_VERSION)]
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
					let (second_token_valuation, first_token_valuation) = Valuations::<T>::get();

					// one updated takes R:2, W:2; and multiply for two assets
					if !T::AssetRegistryApi::enable_pool_creation((
						Self::first_token_id(),
						Self::second_token_id(),
					)) {
						log!(error, "cannot modify asset registry!");
					}
					// XykFunctionsTrait R: 11 W:12
					// PoolCreateApi::pool_create R:2  +
					// ---------------------------------
					// R: 13 W 12
					if let Some((liq_asset_id, issuance)) = T::PoolCreateApi::pool_create(
						Self::vault_address(),
						Self::first_token_id(),
						first_token_valuation,
						Self::second_token_id(),
						second_token_valuation,
					) {
						MintedLiquidity::<T>::put((liq_asset_id, issuance)); // W:1
						if PromoteBootstrapPool::<T>::get() {
							T::RewardsApi::update_pool_promotion(
								liq_asset_id,
								Some(T::DefaultBootstrapPromotedPoolWeight::get()),
							);
						}
					} else {
						log!(error, "cannot create pool!");
					}
					// TODO: include cost of pool_create call
					T::DbWeight::get().reads_writes(21, 18)
				} else if n >= public_start {
					if phase != BootstrapPhase::Public {
						Phase::<T>::put(BootstrapPhase::Public);
						log!(info, "starting public phase");
						T::DbWeight::get().reads_writes(2, 1)
					} else {
						T::DbWeight::get().reads(2)
					}
				} else if n >= whitelist_start {
					if phase != BootstrapPhase::Whitelist {
						log!(info, "starting whitelist phase");
						Phase::<T>::put(BootstrapPhase::Whitelist);
						T::DbWeight::get().reads_writes(2, 1)
					} else {
						T::DbWeight::get().reads(2)
					}
				} else {
					T::DbWeight::get().reads(2)
				}
			} else {
				T::DbWeight::get().reads(2)
			}
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BootstrapBenchmarkingConfig: pallet_issuance::Config {}

	#[cfg(not(feature = "runtime-benchmarks"))]
	pub trait BootstrapBenchmarkingConfig {}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + BootstrapBenchmarkingConfig {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type MaintenanceStatusProvider: GetMaintenanceStatusTrait;

		/// tokens
		type Currency: MultiTokenCurrencyExtended<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>;

		type PoolCreateApi: PoolCreateApi<AccountId = Self::AccountId>;

		#[pallet::constant]
		type DefaultBootstrapPromotedPoolWeight: Get<u8>;

		#[pallet::constant]
		type BootstrapUpdateBuffer: Get<Self::BlockNumber>;

		#[pallet::constant]
		type TreasuryPalletId: Get<PalletId>;

		type VestingProvider: MultiTokenVestingLocks<Self::AccountId, Self::BlockNumber>;

		type WeightInfo: WeightInfo;

		type RewardsApi: RewardsApi<AccountId = Self::AccountId>;

		type AssetRegistryApi: AssetRegistryApi;
	}

	/// maps ([`frame_system::Config::AccountId`], [`TokenId`]) -> [`Balance`] - identifies how much tokens did account provisioned in active bootstrap
	#[pallet::storage]
	#[pallet::getter(fn provisions)]
	pub type Provisions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, TokenId, Balance, ValueQuery>;

	/// maps ([`frame_system::Config::AccountId`], [`TokenId`]) -> [`Balance`] - identifies how much vested tokens did account provisioned in active bootstrap
	#[pallet::storage]
	#[pallet::getter(fn vested_provisions)]
	pub type VestedProvisions<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		TokenId,
		(Balance, BlockNrAsBalance, BlockNrAsBalance),
		ValueQuery,
	>;

	/// list ([`Vec<AccountId>`]) of whitelisted accounts allowed to participate in [`BootstrapPhase::Whitelist`] phase
	#[pallet::storage]
	#[pallet::getter(fn whitelisted_accounts)]
	pub type WhitelistedAccount<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (), ValueQuery>;

	/// Current state of bootstrap as [`BootstrapPhase`]
	#[pallet::storage]
	#[pallet::getter(fn phase)]
	pub type Phase<T: Config> = StorageValue<_, BootstrapPhase, ValueQuery>;

	/// Total sum of provisions of `first` and `second` token in active bootstrap
	#[pallet::storage]
	#[pallet::getter(fn valuations)]
	pub type Valuations<T: Config> = StorageValue<_, (Balance, Balance), ValueQuery>;

	/// Active bootstrap parameters
	#[pallet::storage]
	#[pallet::getter(fn config)]
	pub type BootstrapSchedule<T: Config> =
		StorageValue<_, (T::BlockNumber, u32, u32, (u128, u128)), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn minted_liquidity)]
	pub type MintedLiquidity<T: Config> = StorageValue<_, (TokenId, Balance), ValueQuery>;

	///  Maps ([`frame_system::Config::AccountId`], [`TokenId`] ) -> [`Balance`] - where [`TokeinId`] is id of the token that user participated with. This storage item is used to identify how much liquidity tokens has been claim by the user. If user participated with 2 tokens there are two entries associated with given account (`Address`, `first_token_id`) and (`Address`, `second_token_id`)
	#[pallet::storage]
	#[pallet::getter(fn claimed_rewards)]
	pub type ClaimedRewards<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, TokenId, Balance, ValueQuery>;

	/// List of accouts that provisioned funds to bootstrap and has not claimed liquidity tokens yet
	#[pallet::storage]
	#[pallet::getter(fn provision_accounts)]
	pub type ProvisionAccounts<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (), OptionQuery>;

	/// Currently bootstraped pair of tokens representaed as [ `first_token_id`, `second_token_id`]
	#[pallet::storage]
	#[pallet::getter(fn pair)]
	pub type ActivePair<T: Config> = StorageValue<_, (TokenId, TokenId), OptionQuery>;

	/// Wheter to automatically promote the pool after [`BootstrapPhase::PublicPhase`] or not.
	#[pallet::storage]
	#[pallet::getter(fn get_promote_bootstrap_pool)]
	pub type PromoteBootstrapPool<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn archived)]
	pub type ArchivedBootstrap<T: Config> =
		StorageValue<_, Vec<(T::BlockNumber, u32, u32, (u128, u128))>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// /// provisions vested/locked tokens into the boostrstrap
		// #[pallet::weight(<<T as Config>::WeightInfo>::provision_vested())]
		// #[transactional]
		// pub fn provision_vested(
		// 	origin: OriginFor<T>,
		// 	token_id: TokenId,
		// 	amount: Balance,
		// ) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;
		//
		// ensure!(!T::MaintenanceStatusProvider::is_maintenance(), Error::<T>::ProvisioningBlockedByMaintenanceMode);
		//
		// 	let (vesting_starting_block, vesting_ending_block_as_balance) =
		// 		<<T as Config>::VestingProvider>::unlock_tokens(
		// 			&sender,
		// 			token_id.into(),
		// 			amount.into(),
		// 		)
		// 		.map_err(|_| Error::<T>::NotEnoughVestedAssets)?;
		// 	Self::do_provision(
		// 		&sender,
		// 		token_id,
		// 		amount,
		// 		ProvisionKind::Vested(
		// 			vesting_starting_block.saturated_into::<BlockNrAsBalance>(),
		// 			vesting_ending_block_as_balance.into(),
		// 		),
		// 	)?;
		// 	ProvisionAccounts::<T>::insert(&sender, ());
		// 	Self::deposit_event(Event::Provisioned(token_id, amount));
		// 	Ok(().into())
		// }

		/// Allows for provisioning one of the tokens from currently bootstrapped pair. Can only be called during:
		/// - [`BootstrapPhase::Whitelist`]
		/// - [`BootstrapPhase::Public`]
		///
		/// phases.
		///
		/// # Args:
		///  - `token_id` - id of the token to provision (should be one of the currently bootstraped pair([`ActivePair`]))
		///  - `amount` - amount of the token to provision
		#[pallet::call_index(0)]
		#[pallet::weight(<<T as Config>::WeightInfo>::provision())]
		#[transactional]
		pub fn provision(
			origin: OriginFor<T>,
			token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				!T::MaintenanceStatusProvider::is_maintenance(),
				Error::<T>::ProvisioningBlockedByMaintenanceMode
			);

			Self::do_provision(&sender, token_id, amount, ProvisionKind::Regular)?;
			ProvisionAccounts::<T>::insert(&sender, ());
			Self::deposit_event(Event::Provisioned(token_id, amount));
			Ok(())
		}

		/// Allows for whitelisting accounts, so they can participate in during whitelist phase. The list of
		/// account is extended with every subsequent call
		#[pallet::call_index(1)]
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
			Ok(())
		}

		/// Used for starting/scheduling new bootstrap
		///
		/// # Args:
		/// - `first_token_id` - first token of the tokens pair
		/// - `second_token_id`: second token of the tokens pair
		/// - `ido_start` - number of block when bootstrap will be started (people will be allowed to participate)
		/// - `whitelist_phase_length`: - length of whitelist phase
		/// - `public_phase_lenght`- length of public phase
		/// - `promote_bootstrap_pool`- whether liquidity pool created by bootstrap should be promoted
		/// - `max_first_to_second_ratio` - represented as (numerator,denominator) - Ratio may be used to limit participations of second token id. Ratio between first and second token needs to be held during whole bootstrap. Whenever user tries to participate (using [`Pallet::provision`] extrinsic) the following conditions is check.
		/// ```ignore
		/// all previous first participations + first token participations             ratio numerator
		/// ----------------------------------------------------------------------- <= ------------------
		/// all previous second token participations + second token participations     ratio denominator
		/// ```
		/// and if it evaluates to `false` extrinsic will fail.
		///
		/// **Because of above equation only participations with first token of a bootstrap pair are limited!**
		///
		/// # Examples
		/// Consider:
		///
		/// - user willing to participate 1000 of first token, when:
		/// 	- ratio set during bootstrap schedule is is set to (1/2)
		/// 	- sum of first token participations - 10_000
		/// 	- sum of second token participations - 20_000
		///
		/// participation extrinsic will **fail** because ratio condition **is not met**
		/// ```ignore
		/// 10_000 + 10_000      1
		/// --------------- <=  ---
		///     20_000           2
		/// ```
		///
		/// - user willing to participate 1000 of first token, when:
		/// 	- ratio set during bootstrap schedule is is set to (1/2)
		/// 	- sum of first token participations - 10_000
		/// 	- sum of second token participations - 40_000
		///
		/// participation extrinsic will **succeed** because ratio condition **is met**
		/// ```ignore
		/// 10_000 + 10_000      1
		/// --------------- <=  ---
		///     40_000           2
		/// ```
		///
		///
		/// **If one doesn't want to limit participations in any way, ratio should be set to (u128::MAX,0) - then ratio requirements are always met**
		///
		/// ```ignore
		/// all previous first participations + first token participations                u128::MAX
		/// ----------------------------------------------------------------------- <= ------------------
		/// all previous second token participations + second token participations            1
		/// ```
		#[pallet::call_index(2)]
		#[pallet::weight(<<T as Config>::WeightInfo>::schedule_bootstrap())]
		#[transactional]
		pub fn schedule_bootstrap(
			origin: OriginFor<T>,
			first_token_id: TokenId,
			second_token_id: TokenId,
			ido_start: T::BlockNumber,
			whitelist_phase_length: Option<u32>,
			public_phase_lenght: u32,
			max_first_to_second_ratio: Option<(u128, u128)>,
			promote_bootstrap_pool: bool,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(Phase::<T>::get() == BootstrapPhase::BeforeStart, Error::<T>::AlreadyStarted);

			if let Some((scheduled_ido_start, _, _, _)) = BootstrapSchedule::<T>::get() {
				let now = <frame_system::Pallet<T>>::block_number();
				ensure!(
					now.saturating_add(T::BootstrapUpdateBuffer::get()) < scheduled_ido_start,
					Error::<T>::TooLateToUpdateBootstrap
				);
			}

			ensure!(first_token_id != second_token_id, Error::<T>::SameToken);

			ensure!(T::Currency::exists(first_token_id.into()), Error::<T>::TokenIdDoesNotExists);
			ensure!(T::Currency::exists(second_token_id.into()), Error::<T>::TokenIdDoesNotExists);

			ensure!(
				ido_start > frame_system::Pallet::<T>::block_number(),
				Error::<T>::BootstrapStartInThePast
			);

			let whitelist_phase_length = whitelist_phase_length.unwrap_or_default();
			let max_first_to_second_ratio =
				max_first_to_second_ratio.unwrap_or((Balance::max_value(), Balance::one()));

			ensure!(max_first_to_second_ratio.0 != 0, Error::<T>::WrongRatio);

			ensure!(max_first_to_second_ratio.1 != 0, Error::<T>::WrongRatio);

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
				!T::PoolCreateApi::pool_exists(first_token_id, second_token_id),
				Error::<T>::PoolAlreadyExists
			);

			ActivePair::<T>::put((first_token_id, second_token_id));
			BootstrapSchedule::<T>::put((
				ido_start,
				whitelist_phase_length,
				public_phase_lenght,
				max_first_to_second_ratio,
			));

			PromoteBootstrapPool::<T>::put(promote_bootstrap_pool);

			Ok(())
		}

		/// Used to cancel active bootstrap. Can only be called before bootstrap is actually started
		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(3, 4).saturating_add(Weight::from_ref_time(1_000_000)))]
		#[transactional]
		pub fn cancel_bootstrap(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;

			// BootstrapSchedule should exist but not after BootstrapUpdateBuffer blocks before start

			let now = <frame_system::Pallet<T>>::block_number();
			let (ido_start, _, _, _) =
				BootstrapSchedule::<T>::get().ok_or(Error::<T>::BootstrapNotSchduled)?;
			ensure!(Phase::<T>::get() == BootstrapPhase::BeforeStart, Error::<T>::AlreadyStarted);

			ensure!(
				now.saturating_add(T::BootstrapUpdateBuffer::get()) < ido_start,
				Error::<T>::TooLateToUpdateBootstrap
			);

			ActivePair::<T>::kill();
			BootstrapSchedule::<T>::kill();
			PromoteBootstrapPool::<T>::kill();
			// Unnecessary
			Phase::<T>::put(BootstrapPhase::BeforeStart);

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 1).saturating_add(Weight::from_ref_time(1_000_000)))]
		#[transactional]
		// can be used to enable or disable automatic pool promotion of liquidity pool. Updates [`PromoteBootstrapPool`]
		pub fn update_promote_bootstrap_pool(
			origin: OriginFor<T>,
			promote_bootstrap_pool: bool,
		) -> DispatchResult {
			ensure_root(origin)?;

			// BootstrapSchedule should exist but not finalized
			// we allow this to go thru if the BootstrapSchedule exists and the phase is before finalized

			ensure!(BootstrapSchedule::<T>::get().is_some(), Error::<T>::BootstrapNotSchduled);
			ensure!(Phase::<T>::get() != BootstrapPhase::Finished, Error::<T>::BootstrapFinished);

			PromoteBootstrapPool::<T>::put(promote_bootstrap_pool);

			Ok(())
		}

		/// When bootstrap is in [`BootstrapPhase::Finished`] state user can claim his part of liquidity tokens.
		#[pallet::call_index(5)]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_and_activate_liquidity_tokens())]
		#[transactional]
		pub fn claim_liquidity_tokens(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_claim_liquidity_tokens(&sender, false)
		}

		/// When bootstrap is in [`BootstrapPhase::Finished`] state user can claim his part of liquidity tokens comparing to `claim_liquidity_tokens` when calling `claim_and_activate_liquidity_tokens` tokens will be automatically activated.
		#[pallet::call_index(6)]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_and_activate_liquidity_tokens())]
		#[transactional]
		pub fn claim_and_activate_liquidity_tokens(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_claim_liquidity_tokens(&sender, true)
		}

		/// Used to reset Bootstrap state and prepare it for running another bootstrap.
		/// It should be called multiple times until it produces [`Event::BootstrapFinalized`] event.
		///
		/// # Args:
		/// * `limit` - limit of storage entries to be removed in single call. Should be set to some
		/// reasonable balue like `100`.
		///
		/// **!!! Cleaning up storage is complex operation and pruning all storage items related to particular
		/// bootstrap might not fit in a single block. As a result tx can be rejected !!!**
		#[pallet::call_index(7)]
		#[pallet::weight(<<T as Config>::WeightInfo>::finalize().saturating_add(T::DbWeight::get().reads_writes(1, 1).saturating_mul(Into::<u64>::into(*limit).saturating_add(u64::one()))))]
		#[transactional]
		pub fn finalize(origin: OriginFor<T>, mut limit: u32) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(Self::phase() == BootstrapPhase::Finished, Error::<T>::NotFinishedYet);

			ensure!(
				ProvisionAccounts::<T>::iter_keys().next().is_none(),
				Error::<T>::BootstrapNotReadyToBeFinished
			);

			match VestedProvisions::<T>::clear(limit, None).into() {
				KillStorageResult::AllRemoved(num_iter) => limit = limit.saturating_sub(num_iter),
				KillStorageResult::SomeRemaining(_) => {
					Self::deposit_event(Event::BootstrapParitallyFinalized);
					return Ok(())
				},
			}

			match WhitelistedAccount::<T>::clear(limit, None).into() {
				KillStorageResult::AllRemoved(num_iter) => limit = limit.saturating_sub(num_iter),
				KillStorageResult::SomeRemaining(_) => {
					Self::deposit_event(Event::BootstrapParitallyFinalized);
					return Ok(())
				},
			}

			match ClaimedRewards::<T>::clear(limit, None).into() {
				KillStorageResult::AllRemoved(num_iter) => limit = limit.saturating_sub(num_iter),
				KillStorageResult::SomeRemaining(_) => {
					Self::deposit_event(Event::BootstrapParitallyFinalized);
					return Ok(())
				},
			}

			match Provisions::<T>::clear(limit, None).into() {
				KillStorageResult::AllRemoved(num_iter) => limit = limit.saturating_sub(num_iter),
				KillStorageResult::SomeRemaining(_) => {
					Self::deposit_event(Event::BootstrapParitallyFinalized);
					return Ok(())
				},
			}

			Phase::<T>::put(BootstrapPhase::BeforeStart);
			let (liq_token_id, _) = MintedLiquidity::<T>::take();
			let balance = T::Currency::free_balance(liq_token_id.into(), &Self::vault_address());
			if balance > 0_u128.into() {
				T::Currency::transfer(
					liq_token_id.into(),
					&Self::vault_address(),
					&T::TreasuryPalletId::get().into_account_truncating(),
					balance,
					ExistenceRequirement::AllowDeath,
				)?;
			}
			Valuations::<T>::kill();
			ActivePair::<T>::kill();
			PromoteBootstrapPool::<T>::kill();

			if let Some(bootstrap) = BootstrapSchedule::<T>::take() {
				ArchivedBootstrap::<T>::mutate(|v| {
					v.push(bootstrap);
				});
			}

			Self::deposit_event(Event::BootstrapFinalized);

			Ok(())
		}

		/// Allows claiming rewards for some account that haven't done that yet. The only difference between
		/// calling [`Pallet::claim_liquidity_tokens_for_account`] by some other account and calling [`Pallet::claim_liquidity_tokens`] directly by that account is account that will be charged for transaction fee.
		/// # Args:
		/// - `other` - account in behalf of which liquidity tokens should be claimed
		#[pallet::call_index(8)]
		#[pallet::weight(<<T as Config>::WeightInfo>::claim_and_activate_liquidity_tokens())]
		#[transactional]
		pub fn claim_liquidity_tokens_for_account(
			origin: OriginFor<T>,
			account: T::AccountId,
			activate_rewards: bool,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Self::do_claim_liquidity_tokens(&account, activate_rewards)
		}
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		/// Only scheduled token pair can be used for provisions
		UnsupportedTokenId,
		/// Not enough funds for provision
		NotEnoughAssets,
		/// Not enough funds for provision (vested)
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
		/// First provision must be in non restricted token
		FirstProvisionInSecondTokenId,
		/// Bootstraped pool already exists
		PoolAlreadyExists,
		/// Cannot claim rewards before bootstrap finish
		NotFinishedYet,
		/// no rewards to claim
		NothingToClaim,
		/// wrong ratio
		WrongRatio,
		/// no rewards to claim
		BootstrapNotReadyToBeFinished,
		/// Tokens used in bootstrap cannot be the same
		SameToken,
		/// Token does not exists
		TokenIdDoesNotExists,
		/// Token activations failed
		TokensActivationFailed,
		/// Bootstrap not scheduled
		BootstrapNotSchduled,
		/// Bootstrap already Finished
		BootstrapFinished,
		/// Bootstrap can only be updated or cancelled
		/// BootstrapUpdateBuffer blocks or more before bootstrap start
		TooLateToUpdateBootstrap,
		/// Bootstrap provisioning blocked by maintenance mode
		ProvisioningBlockedByMaintenanceMode,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Funds provisioned
		Provisioned(TokenId, Balance),
		/// Funds provisioned using vested tokens
		VestedProvisioned(TokenId, Balance),
		/// The activation of the rewards liquidity tokens failed
		RewardsLiquidityAcitvationFailed(T::AccountId, TokenId, Balance),
		/// Rewards claimed
		RewardsClaimed(TokenId, Balance),
		/// account whitelisted
		AccountsWhitelisted,
		/// finalization process tarted
		BootstrapParitallyFinalized,
		/// finalization process finished
		BootstrapFinalized,
	}
}

#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Debug)]
pub enum BootstrapPhase {
	/// Waiting for another bootstrap to be scheduled using [`Pallet::schedule_bootstrap`]
	BeforeStart,
	// Phase where only whitelisted accounts (see [`Bootstrap::whitelist_accounts`]) can participate with both tokens as long as particular accounts are whitelisted and ratio after participation is below enforced ratio.
	Whitelist,
	/// Anyone can participate as long as ratio after participation is below enforced ratio
	Public,
	/// Bootstrap has finished. At this phase users that participated in bootstrap during previous phases can claim their share of minted `liquidity tokens`. `Bootstrap::finalize` can be call to reset pallet state schedule following bootstrap again.
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
		PALLET_ID.into_account_truncating()
	}

	fn claim_liquidity_tokens_from_single_currency(
		who: &T::AccountId,
		provision_token_id: &TokenId,
		rewards: Balance,
		rewards_vested: Balance,
		lock: (BlockNrAsBalance, BlockNrAsBalance),
	) -> DispatchResult {
		let (liq_token_id, _) = Self::minted_liquidity();
		let total_rewards = rewards.checked_add(rewards_vested).ok_or(Error::<T>::MathOverflow)?;
		if total_rewards == 0 {
			return Ok(())
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
			<<T as Config>::VestingProvider>::lock_tokens(
				who,
				liq_token_id.into(),
				rewards_vested.into(),
				Some(lock.0.saturated_into()),
				lock.1.into(),
			)?;
		}

		Ok(())
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
		let (second_token_valuation, first_token_valuation) = Valuations::<T>::get();
		let left = U256::from(first_token_valuation) * U256::from(ratio_denominator);
		let right = U256::from(ratio_nominator) * U256::from(second_token_valuation);
		left <= right
	}

	pub fn do_provision(
		sender: &T::AccountId,
		token_id: TokenId,
		amount: Balance,
		is_vested: ProvisionKind,
	) -> DispatchResult {
		let is_first_token = token_id == Self::first_token_id();
		let is_second_token = token_id == Self::second_token_id();
		let is_public_phase = Phase::<T>::get() == BootstrapPhase::Public;
		let is_whitelist_phase = Phase::<T>::get() == BootstrapPhase::Whitelist;
		let am_i_whitelisted = Self::is_whitelisted(sender);

		ensure!(is_first_token || is_second_token, Error::<T>::UnsupportedTokenId);

		ensure!(
			is_public_phase || (is_whitelist_phase && (am_i_whitelisted || is_second_token)),
			Error::<T>::Unauthorized
		);

		let schedule = BootstrapSchedule::<T>::get();
		ensure!(schedule.is_some(), Error::<T>::Unauthorized);
		let (_, _, _, (ratio_nominator, ratio_denominator)) = schedule.unwrap();

		<T as Config>::Currency::transfer(
			token_id.into(),
			sender,
			&Self::vault_address(),
			amount.into(),
			ExistenceRequirement::KeepAlive,
		)
		.or(Err(Error::<T>::NotEnoughAssets))?;

		match is_vested {
			ProvisionKind::Regular => {
				ensure!(
					Provisions::<T>::try_mutate(sender, token_id, |provision| {
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
			ProvisionKind::Vested(provision_start_block, provision_end_block) => {
				ensure!(
					VestedProvisions::<T>::try_mutate(
						sender,
						token_id,
						|(provision, start_block, end_block)| {
							if let Some(val) = provision.checked_add(amount) {
								*provision = val;
								*start_block = (*start_block).max(provision_start_block);
								*end_block = (*end_block).max(provision_end_block);
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

		let (pre_second_token_valuation, _) = Valuations::<T>::get();
		ensure!(
			token_id != Self::first_token_id() || pre_second_token_valuation != 0,
			Error::<T>::FirstProvisionInSecondTokenId
		);

		ensure!(
			Valuations::<T>::try_mutate(
				|(second_token_valuation, first_token_valuation)| -> Result<(), ()> {
					if token_id == Self::second_token_id() {
						*second_token_valuation =
							second_token_valuation.checked_add(amount).ok_or(())?;
					}
					if token_id == Self::first_token_id() {
						*first_token_valuation =
							first_token_valuation.checked_add(amount).ok_or(())?;
					}
					Ok(())
				}
			)
			.is_ok(),
			Error::<T>::MathOverflow
		);

		if token_id == Self::first_token_id() {
			ensure!(
				Self::is_ratio_kept(ratio_nominator, ratio_denominator),
				Error::<T>::ValuationRatio
			);
		}
		Ok(())
	}

	fn get_valuation(token_id: &TokenId) -> Balance {
		if *token_id == Self::first_token_id() {
			Self::valuations().1
		} else if *token_id == Self::second_token_id() {
			Self::valuations().0
		} else {
			0
		}
	}

	fn calculate_rewards(
		who: &T::AccountId,
		token_id: &TokenId,
	) -> Result<(Balance, Balance, (BlockNrAsBalance, BlockNrAsBalance)), Error<T>> {
		let valuation = Self::get_valuation(token_id);
		let provision = Self::provisions(who, token_id);
		let (vested_provision, lock_start, lock_end) = Self::vested_provisions(who, token_id);
		let (_, liquidity) = Self::minted_liquidity();
		let rewards =
			multiply_by_rational_with_rounding(liquidity / 2, provision, valuation, Rounding::Down)
				.ok_or(Error::<T>::MathOverflow)?;
		let vested_rewards = multiply_by_rational_with_rounding(
			liquidity / 2,
			vested_provision,
			valuation,
			Rounding::Down,
		)
		.ok_or(Error::<T>::MathOverflow)?;
		Ok((rewards, vested_rewards, (lock_start, lock_end)))
	}

	fn do_claim_liquidity_tokens(who: &T::AccountId, activate_rewards: bool) -> DispatchResult {
		ensure!(Self::phase() == BootstrapPhase::Finished, Error::<T>::NotFinishedYet);

		let (liq_token_id, _) = Self::minted_liquidity();

		// for backward compatibility
		if !Self::archived().is_empty() {
			ensure!(ProvisionAccounts::<T>::get(who).is_some(), Error::<T>::NothingToClaim);
		} else {
			ensure!(
				!ClaimedRewards::<T>::contains_key(&who, &Self::first_token_id()),
				Error::<T>::NothingToClaim
			);
			ensure!(
				!ClaimedRewards::<T>::contains_key(&who, &Self::second_token_id()),
				Error::<T>::NothingToClaim
			);
		}

		let (first_token_rewards, first_token_rewards_vested, first_token_lock) =
			Self::calculate_rewards(who, &Self::first_token_id())?;
		let (second_token_rewards, second_token_rewards_vested, second_token_lock) =
			Self::calculate_rewards(who, &Self::second_token_id())?;

		let total_rewards_claimed = second_token_rewards
			.checked_add(second_token_rewards_vested)
			.ok_or(Error::<T>::MathOverflow)?
			.checked_add(first_token_rewards)
			.ok_or(Error::<T>::MathOverflow)?
			.checked_add(first_token_rewards_vested)
			.ok_or(Error::<T>::MathOverflow)?;

		Self::claim_liquidity_tokens_from_single_currency(
			who,
			&Self::second_token_id(),
			second_token_rewards,
			second_token_rewards_vested,
			second_token_lock,
		)?;
		log!(
			info,
			"Second token rewards (non-vested, vested) = ({}, {})",
			second_token_rewards,
			second_token_rewards_vested,
		);

		Self::claim_liquidity_tokens_from_single_currency(
			who,
			&Self::first_token_id(),
			first_token_rewards,
			first_token_rewards_vested,
			first_token_lock,
		)?;
		log!(
			info,
			"First token rewards (non-vested, vested) = ({}, {})",
			first_token_rewards,
			first_token_rewards_vested,
		);

		ProvisionAccounts::<T>::remove(who);

		if activate_rewards && <T as Config>::RewardsApi::can_activate(liq_token_id) {
			let non_vested_rewards = second_token_rewards
				.checked_add(first_token_rewards)
				.ok_or(Error::<T>::MathOverflow)?;
			if non_vested_rewards > 0 {
				let activate_result = <T as Config>::RewardsApi::activate_liquidity_tokens(
					who,
					liq_token_id,
					non_vested_rewards,
				);
				if let Err(err) = activate_result {
					log!(
						error,
						"Activating liquidity tokens failed upon bootstrap claim rewards = ({:?}, {}, {}, {:?})",
						who,
						liq_token_id,
						non_vested_rewards,
						err
					);

					Self::deposit_event(Event::RewardsLiquidityAcitvationFailed(
						who.clone(),
						liq_token_id,
						non_vested_rewards,
					));
				};
			}
		}

		Self::deposit_event(Event::RewardsClaimed(liq_token_id, total_rewards_claimed));

		Ok(())
	}

	fn first_token_id() -> TokenId {
		ActivePair::<T>::get().map(|(first, _)| first).unwrap_or(4_u32)
	}

	fn second_token_id() -> TokenId {
		ActivePair::<T>::get().map(|(_, second)| second).unwrap_or(0_u32)
	}
}

impl<T: Config> Contains<(TokenId, TokenId)> for Pallet<T> {
	fn contains(pair: &(TokenId, TokenId)) -> bool {
		pair == &(Self::first_token_id(), Self::second_token_id()) ||
			pair == &(Self::second_token_id(), Self::first_token_id())
	}
}
