#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

pub use artemis_asset;
pub use artemis_erc20_app;
pub use artemis_eth_app;
use codec::{Decode, Encode};
use frame_support::{
	construct_runtime,
	dispatch::DispatchResult,
	parameter_types,
	traits::{
		tokens::currency::{MultiTokenCurrency, MultiTokenImbalanceWithZeroTrait},
		Contains, EnsureOrigin, EnsureOriginWithArg, Everything, ExistenceRequirement, Get,
		Imbalance, LockIdentifier, Nothing, U128CurrencyToVote, WithdrawReasons,OnRuntimeUpgrade
	},
	unsigned::TransactionValidityError,
	weights::{
		constants::{RocksDbWeight, WEIGHT_PER_MICROS, WEIGHT_PER_MILLIS, WEIGHT_PER_SECOND},
		ConstantMultiplier, DispatchClass, Weight,
	},
	PalletId,
};
#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot,
};
pub use orml_tokens;
use orml_tokens::{MultiTokenCurrencyExtended, TransferDust};
use orml_traits::{
	asset_registry::{AssetMetadata, AssetProcessor},
	parameter_type_with_key,
};
pub use pallet_sudo;
use pallet_transaction_payment::{Multiplier, OnChargeTransaction, TargetedFeeAdjustment};
pub use pallet_verifier;
use pallet_vesting_mangata_rpc_runtime_api::VestingInfosWithLockedAt;
// Polkadot Imports
use polkadot_runtime_common::BlockHashCount;
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, Convert, ConvertInto,
		DispatchInfoOf, PostDispatchInfoOf, Saturating, StaticLookup, Zero,
	},
	transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchError, FixedPointNumber, Percent, Perquintill,
};
pub use sp_runtime::{MultiAddress, Perbill, Permill};
use sp_std::{
	cmp::Ordering,
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	prelude::*,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;
pub use xcm::{latest::prelude::*, VersionedMultiLocation};

pub use constants::{fee::*, parachains::*};
pub use currency::*;
pub use mangata_types::{
	assets::{CustomMetadata, XcmMetadata},
	AccountId, Address, Amount, Balance, BlockNumber, Hash, Index, Signature, TokenId,
};
pub use pallet_bridge;
pub use pallet_issuance::{IssuanceInfo, PoolPromoteApi};
pub use pallet_sudo_origin;
pub use pallet_xyk;
// XCM Imports
use pallet_xyk::AssetMetadataMutationTrait;
use xyk_runtime_api::{RpcAmountsResult, XYKRpcResult};

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub const MGR_TOKEN_ID: TokenId = 0;
pub const ROC_TOKEN_ID: TokenId = 4;
pub const KAR_TOKEN_ID: TokenId = 6;
pub const TUR_TOKEN_ID: TokenId = 7;

pub mod constants;
mod weights;
pub mod xcm_config;
mod migrations;

/// Block header type as expected by this runtime.
pub type Header = generic::HeaderVer<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	MangataMigrations,
>;

pub struct MangataMigrations;
impl OnRuntimeUpgrade for MangataMigrations {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrations::phragmen_elections::PhragmenElectionsMigration::on_runtime_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		migrations::phragmen_elections::PhragmenElectionsMigration::pre_upgrade()
			.expect("try-runtime pre_upgrade for PhragmenElectionsMigration failed!!");
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		migrations::phragmen_elections::PhragmenElectionsMigration::post_upgrade()
			.expect("try-runtime post_upgrade for PhragmenElectionsMigration failed!!");
		Ok(())
	}
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	use sp_runtime::{generic, traits::BlakeTwo256};

	use super::*;

	/// Opaque block header type.
	pub type Header = generic::HeaderVer<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("mangata-parachain"),
	impl_name: create_runtime_str!("mangata-parachain"),
	authoring_version: 10,
	spec_version: 10,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 10,
	state_version: 0,
};

mod currency {
	use super::Balance;

	pub const MILLICENTS: Balance = CENTS / 1000;
	pub const CENTS: Balance = DOLLARS / 100; // assume this is worth about a cent.
	pub const DOLLARS: Balance = super::UNIT;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Unit = the base number of indivisible units for balance
pub const UNIT: Balance = 1_000_000_000_000_000_000;
pub const MILLIUNIT: Balance = 1_000_000_000_000_000;
pub const MICROUNIT: Balance = 1_000_000_000_000;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
/// NOTE: reduced by half comparing to origin impl as we want to fill block only up to 50%
/// so there is room for new extrinsics in the next block
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 4;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// taken from dedicated benchmark (run on reference machine)
	//
	// $ cargo bench --features=disable-execution
	// ...
	// Block production/full block shuffling without executing extrinsics
	//                         time:   [12.025 ms 12.029 ms 12.032 ms]
	//                         change: [-59.043% -59.005% -58.974%] (p = 0.00 < 0.05)
	//
	// ...
	pub const MangataBlockExecutionWeight: Weight = 12 * WEIGHT_PER_MILLIS;

	// taken from dedicated benchmark (run on reference machine)
	//
	// $ cargo bench --features=disable-execution
	// ...
	// avarege execution time of 5067 noop extrinsic : 946200 microseconds => 186
	// ...
	pub const MangataExtrinsicBaseWeight: Weight = 186 * WEIGHT_PER_MICROS;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(MangataBlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = MangataExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 42;
}

parameter_types! {
	pub const MgrTokenId: TokenId = MGR_TOKEN_ID;
	pub const RocTokenId: TokenId = ROC_TOKEN_ID;
	pub const TurTokenId: TokenId = TUR_TOKEN_ID;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::HeaderVer<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = ();
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = Everything;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = weights::frame_system_weights::ModuleWeight<Runtime>;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	/// The maximum number of consumers allowed on a single account.
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp_weights::ModuleWeight<Runtime>;
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = ParachainStaking;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
	pub const ProposalBondMaximum: Option<Balance> = None;
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * DOLLARS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const BountyDepositBase: Balance = 1 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 5 * DOLLARS;
	pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, MgrTokenId>;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type Event = Event;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = weights::pallet_treasury_weights::ModuleWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
}

// TODO: discuiss existential deposit feature
// https://trello.com/c/P5rYYQcS/424-discuiss-orml-tokens-existential-deposit
parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: TokenId| -> Balance {
		0
		// match currency_id {
		// 	&MGR_TOKEN_ID => 100,
		// 	_ => 0,
		// }
	};
}

parameter_types! {
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub const MaxLocks: u32 = 50;
}

// The MaxLocks (on a who-token_id pair) that is allowed by orml_tokens
// must exceed the total possible locks that can be applied to it, ALL pallets considered
// This is because orml_tokens uses BoundedVec for Locks storage item and does not inform on failure
// Balances uses WeakBoundedVec and so does not fail
const_assert!(
	MaxLocks::get() >= <Runtime as pallet_vesting_mangata::Config>::MAX_VESTING_SCHEDULES
);

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		*a == TreasuryAccount::get()
	}
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = TokenId;
	type WeightInfo = weights::orml_tokens_weights::ModuleWeight<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = TransferDust<Runtime, TreasuryAccount>;
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = DustRemovalWhitelist;
}

pub struct AssetMetadataMutation;
impl AssetMetadataMutationTrait for AssetMetadataMutation {
	fn set_asset_info(
		asset: TokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u32,
	) -> DispatchResult {
		let metadata = AssetMetadata {
			name,
			symbol,
			decimals,
			existential_deposit: Default::default(),
			additional: Default::default(),
			location: None,
		};
		orml_asset_registry::Pallet::<Runtime>::do_register_asset_without_asset_processor(
			metadata, asset,
		)?;
		Ok(())
	}
}

impl pallet_xyk::Config for Runtime {
	type Event = Event;
	type ActivationReservesProvider = MultiPurposeLiquidity;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type NativeCurrencyId = MgrTokenId;
	type TreasuryPalletId = TreasuryPalletId;
	type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
	type LiquidityMiningIssuanceVault = LiquidityMiningIssuanceVault;
	type PoolPromoteApi = Issuance;
	type PoolFeePercentage = frame_support::traits::ConstU128<20>;
	type TreasuryFeePercentage = frame_support::traits::ConstU128<5>;
	type BuyAndBurnFeePercentage = frame_support::traits::ConstU128<5>;
	type RewardsDistributionPeriod = frame_support::traits::ConstU32<10000>;
	type VestingProvider = Vesting;
	type DisallowedPools = Bootstrap;
	type DisabledTokens = Nothing;
	type AssetMetadataMutation = AssetMetadataMutation;
	type WeightInfo = weights::pallet_xyk_weights::ModuleWeight<Runtime>;
}

parameter_types! {
	pub const BootstrapUpdateBuffer: BlockNumber = 300;
}

impl pallet_bootstrap::BootstrapBenchmarkingConfig for Runtime {}

impl pallet_bootstrap::Config for Runtime {
	type Event = Event;
	type PoolCreateApi = Xyk;
	type BootstrapUpdateBuffer = BootstrapUpdateBuffer;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type VestingProvider = Vesting;
	type TreasuryPalletId = TreasuryPalletId;
	type RewardsApi = Xyk;
	type WeightInfo = weights::pallet_bootstrap_weights::ModuleWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility_weights::ModuleWeight<Runtime>;
}

type ORMLCurrencyAdapterNegativeImbalance =
	<orml_tokens::MultiTokenCurrencyAdapter<Runtime> as MultiTokenCurrency<
		AccountId,
	>>::NegativeImbalance;

pub trait OnMultiTokenUnbalanced<
	Imbalance: frame_support::traits::TryDrop + MultiTokenImbalanceWithZeroTrait<TokenId>,
>
{
	/// Handler for some imbalances. The different imbalances might have different origins or
	/// meanings, dependent on the context. Will default to simply calling on_unbalanced for all
	/// of them. Infallible.
	fn on_unbalanceds<B>(token_id: TokenId, amounts: impl Iterator<Item = Imbalance>)
	where
		Imbalance: frame_support::traits::Imbalance<B>,
	{
		Self::on_unbalanced(amounts.fold(Imbalance::from_zero(token_id), |i, x| x.merge(i)))
	}

	/// Handler for some imbalance. Infallible.
	fn on_unbalanced(amount: Imbalance) {
		amount.try_drop().unwrap_or_else(Self::on_nonzero_unbalanced)
	}

	/// Actually handle a non-zero imbalance. You probably want to implement this rather than
	/// `on_unbalanced`.
	fn on_nonzero_unbalanced(amount: Imbalance) {
		drop(amount);
	}
}

pub struct ToAuthor;
impl OnMultiTokenUnbalanced<ORMLCurrencyAdapterNegativeImbalance> for ToAuthor {
	fn on_nonzero_unbalanced(amount: ORMLCurrencyAdapterNegativeImbalance) {
		if let Some(author) = Authorship::author() {
			<orml_tokens::MultiTokenCurrencyAdapter<Runtime> as MultiTokenCurrency<
				AccountId,
			>>::resolve_creating(amount.0, &author, amount);
		}
	}
}

parameter_types! {
	pub const TransactionByteFee: Balance = 5 * MILLIUNIT;
	pub const OperationalFeeMultiplier: u8 = 5;
}

#[derive(Encode, Decode, Clone, TypeInfo)]
pub struct ThreeCurrencyOnChargeAdapter<C, OU, T1, T2, T3, SF2, SF3>(
	PhantomData<(C, OU, T1, T2, T3, SF2, SF3)>,
);

type NegativeImbalanceOf<C, T> =
	<C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

/// Default implementation for a Currency and an OnUnbalanced handler.
///
/// The unbalance handler is given 2 unbalanceds in [`OnUnbalanced::on_unbalanceds`]: fee and
/// then tip.
impl<T, C, OU, T1, T2, T3, SF2, SF3> OnChargeTransaction<T>
	for ThreeCurrencyOnChargeAdapter<C, OU, T1, T2, T3, SF2, SF3>
where
	T: pallet_transaction_payment::Config,
	T::LengthToFee: frame_support::weights::WeightToFee<
		Balance = <C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance,
	>,
	C: MultiTokenCurrency<<T as frame_system::Config>::AccountId>,
	C::PositiveImbalance: Imbalance<
		<C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::NegativeImbalance,
	>,
	C::NegativeImbalance: Imbalance<
		<C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::PositiveImbalance,
	>,
	OU: OnMultiTokenUnbalanced<NegativeImbalanceOf<C, T>>,
	NegativeImbalanceOf<C, T>: MultiTokenImbalanceWithZeroTrait<TokenId>,
	<C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance:
		scale_info::TypeInfo,
	T1: Get<TokenId>,
	T2: Get<TokenId>,
	T3: Get<TokenId>,
	SF2: Get<u128>,
	SF3: Get<u128>,
{
	type LiquidityInfo = Option<(TokenId, NegativeImbalanceOf<C, T>)>;
	type Balance = <C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Withdraw the predicted fee from the transaction origin.
	///
	/// Note: The `fee` already includes the `tip`.
	fn withdraw_fee(
		who: &T::AccountId,
		_call: &T::Call,
		_info: &DispatchInfoOf<T::Call>,
		fee: Self::Balance,
		tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
		if fee.is_zero() {
			return Ok(None)
		}

		let withdraw_reason = if tip.is_zero() {
			WithdrawReasons::TRANSACTION_PAYMENT
		} else {
			WithdrawReasons::TRANSACTION_PAYMENT | WithdrawReasons::TIP
		};

		match C::withdraw(
			T1::get().into(),
			who,
			fee,
			withdraw_reason,
			ExistenceRequirement::KeepAlive,
		) {
			Ok(imbalance) => Ok(Some((T1::get(), imbalance))),
			// TODO make sure atleast 1 planck KSM is charged
			Err(_) => match C::withdraw(
				T2::get().into(),
				who,
				fee / SF2::get().into(),
				withdraw_reason,
				ExistenceRequirement::KeepAlive,
			) {
				Ok(imbalance) => Ok(Some((T2::get(), imbalance))),
				Err(_) => match C::withdraw(
					T3::get().into(),
					who,
					fee / SF3::get().into(),
					withdraw_reason,
					ExistenceRequirement::KeepAlive,
				) {
					Ok(imbalance) => Ok(Some((T3::get(), imbalance))),
					Err(_) => Err(InvalidTransaction::Payment.into()),
				},
			},
		}
	}

	/// Hand the fee and the tip over to the `[OnUnbalanced]` implementation.
	/// Since the predicted fee might have been too high, parts of the fee may
	/// be refunded.
	///
	/// Note: The `corrected_fee` already includes the `tip`.
	fn correct_and_deposit_fee(
		who: &T::AccountId,
		_dispatch_info: &DispatchInfoOf<T::Call>,
		_post_info: &PostDispatchInfoOf<T::Call>,
		corrected_fee: Self::Balance,
		tip: Self::Balance,
		already_withdrawn: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError> {
		if let Some((token_id, paid)) = already_withdrawn {
			let (corrected_fee, tip) = if token_id == T3::get() {
				(corrected_fee / SF3::get().into(), tip / SF3::get().into())
			} else if token_id == T2::get() {
				(corrected_fee / SF2::get().into(), tip / SF2::get().into())
			} else {
				(corrected_fee, tip)
			};
			// Calculate how much refund we should return
			let refund_amount = paid.peek().saturating_sub(corrected_fee);
			// refund to the the account that paid the fees. If this fails, the
			// account might have dropped below the existential balance. In
			// that case we don't refund anything.
			let refund_imbalance = C::deposit_into_existing(token_id.into(), &who, refund_amount)
				.unwrap_or_else(|_| C::PositiveImbalance::from_zero(token_id.into()));
			// merge the imbalance caused by paying the fees and refunding parts of it again.
			let adjusted_paid = paid
				.offset(refund_imbalance)
				.same()
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
			// Call someone else to handle the imbalance (fee and tip separately)
			let (tip, fee) = adjusted_paid.split(tip);
			OU::on_unbalanceds(token_id, Some(fee).into_iter().chain(Some(tip)));
		}
		Ok(())
	}
}

parameter_types! {
	// We want no variability, the other parameters are superfluous
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(0, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1);
}

impl pallet_transaction_payment::Config for Runtime {
	type Event = Event;
	type OnChargeTransaction = ThreeCurrencyOnChargeAdapter<
		orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
		ToAuthor,
		MgrTokenId,
		RocTokenId,
		TurTokenId,
		frame_support::traits::ConstU128<ROC_MGR_SCALE_FACTOR>,
		frame_support::traits::ConstU128<TUR_MGR_SCALE_FACTOR>,
	>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate =
		TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnSystemEvent = ();
	type SelfParaId = ParachainInfo;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::AnyRelayNumber;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
	pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = ParachainStaking;
	type NextSessionRotation = ParachainStaking;
	type SessionManager = ParachainStaking;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = weights::pallet_session_weights::ModuleWeight<Runtime>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const MinLengthName: usize = 1;
	pub const MaxLengthName: usize = 255;
	pub const MinLengthSymbol: usize = 1;
	pub const MaxLengthSymbol: usize = 255;
	pub const MinLengthDescription: usize = 1;
	pub const MaxLengthDescription: usize = 255;
	pub const MaxDecimals: u32 = 255;
}

impl pallet_bridge::Config for Runtime {
	type Event = Event;
	type Verifier = pallet_verifier::Pallet<Runtime>;
	type AppETH = artemis_eth_app::Module<Runtime>;
	type AppERC20 = artemis_erc20_app::Module<Runtime>;
}

impl pallet_verifier::Config for Runtime {
	type Event = Event;
}

impl artemis_asset::Config for Runtime {
	type Event = Event;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
}

impl artemis_eth_app::Config for Runtime {
	type Event = Event;
}

impl artemis_erc20_app::Config for Runtime {
	type Event = Event;
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

impl pallet_sudo_origin::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type SudoOrigin =
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective_weights::ModuleWeight<Runtime>;
}

parameter_types! {
	/// Default BlocksPerRound is every 4 hours (1200 * 12 second block times)
	pub const BlocksPerRound: u32 = 4 * HOURS;
	/// Collator candidate exit delay (number of rounds)
	pub const LeaveCandidatesDelay: u32 = 2;
	/// Collator candidate bond increases/decreases delay (number of rounds)
	pub const CandidateBondDelay: u32 = 2;
	/// Delegator exit delay (number of rounds)
	pub const LeaveDelegatorsDelay: u32 = 2;
	/// Delegation revocations delay (number of rounds)
	pub const RevokeDelegationDelay: u32 = 2;
	/// Delegation bond increases/decreases delay (number of rounds)
	pub const DelegationBondDelay: u32 = 2;
	/// Reward payments delay (number of rounds)
	pub const RewardPaymentDelay: u32 = 2;
	/// Minimum collators selected per round, default at genesis and minimum forever after
	pub const MinSelectedCandidates: u32 = 25;
	/// Maximum collator candidates allowed
	pub const MaxCollatorCandidates: u32 = 35;
	/// Maximum delegators allowed per candidate
	pub const MaxTotalDelegatorsPerCandidate: u32 = 25;
	/// Maximum delegators counted per candidate
	pub const MaxDelegatorsPerCandidate: u32 = 12;
	/// Maximum delegations per delegator
	pub const MaxDelegationsPerDelegator: u32 = 30;
	/// Default fixed percent a collator takes off the top of due rewards
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	/// Minimum stake required to become a collator
	pub const MinCollatorStk: u128 = 10 * DOLLARS;
	/// Minimum stake required to be reserved to be a candidate
	pub const MinCandidateStk: u128 = 1 * DOLLARS;
	/// Minimum stake required to be reserved to be a delegator
	pub const MinDelegatorStk: u128 = 1 * CENTS;
}

// To ensure that BlocksPerRound is not zero, breaking issuance calculations
// Also since 1 block is used for session change, atleast 1 block more needed for extrinsics to work
const_assert!(BlocksPerRound::get() >= 2);

impl parachain_staking::Config for Runtime {
	type Event = Event;
	type StakingReservesProvider = MultiPurposeLiquidity;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
	type BlocksPerRound = BlocksPerRound;
	type LeaveCandidatesDelay = LeaveCandidatesDelay;
	type CandidateBondDelay = CandidateBondDelay;
	type LeaveDelegatorsDelay = LeaveDelegatorsDelay;
	type RevokeDelegationDelay = RevokeDelegationDelay;
	type DelegationBondDelay = DelegationBondDelay;
	type RewardPaymentDelay = RewardPaymentDelay;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxCollatorCandidates = MaxCollatorCandidates;
	type MaxTotalDelegatorsPerCandidate = MaxTotalDelegatorsPerCandidate;
	type MaxDelegatorsPerCandidate = MaxDelegatorsPerCandidate;
	type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
	type DefaultCollatorCommission = DefaultCollatorCommission;
	type MinCollatorStk = MinCollatorStk;
	type MinCandidateStk = MinCandidateStk;
	type MinDelegation = MinDelegatorStk;
	type NativeTokenId = MgrTokenId;
	type StakingLiquidityTokenValuator = Xyk;
	type Issuance = Issuance;
	type StakingIssuanceVault = StakingIssuanceVault;
	type FallbackProvider = Council;
	type WeightInfo = weights::parachain_staking_weights::ModuleWeight<Runtime>;
}

impl parachain_staking::StakingBenchmarkConfig for Runtime {}

parameter_types! {
	pub const HistoryLimit: u32 = 10u32;

	pub const LiquidityMiningIssuanceVaultId: PalletId = PalletId(*b"py/lqmiv");
	pub LiquidityMiningIssuanceVault: AccountId = LiquidityMiningIssuanceVaultId::get().into_account_truncating();
	pub const StakingIssuanceVaultId: PalletId = PalletId(*b"py/stkiv");
	pub StakingIssuanceVault: AccountId = StakingIssuanceVaultId::get().into_account_truncating();

	pub const TotalCrowdloanAllocation: Balance = 330_000_000 * DOLLARS;
	pub const IssuanceCap: Balance = 4_000_000_000 * DOLLARS;
	pub const LinearIssuanceBlocks: u32 = 13_140_000u32; // 5 years
	pub const LiquidityMiningSplit: Perbill = Perbill::from_parts(555555556);
	pub const StakingSplit: Perbill = Perbill::from_parts(444444444);
	pub const ImmediateTGEReleasePercent: Percent = Percent::from_percent(20);
	pub const TGEReleasePeriod: u32 = 5_256_000u32; // 2 years
	pub const TGEReleaseBegin: u32 = 100_800u32; // Two weeks into chain start
}

// Issuance history must be kept for atleast the staking reward delay
const_assert!(RewardPaymentDelay::get() <= HistoryLimit::get());

impl pallet_issuance::Config for Runtime {
	type Event = Event;
	type NativeCurrencyId = MgrTokenId;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type BlocksPerRound = BlocksPerRound;
	type HistoryLimit = HistoryLimit;
	type LiquidityMiningIssuanceVault = LiquidityMiningIssuanceVault;
	type StakingIssuanceVault = StakingIssuanceVault;
	type TotalCrowdloanAllocation = TotalCrowdloanAllocation;
	type IssuanceCap = IssuanceCap;
	type LinearIssuanceBlocks = LinearIssuanceBlocks;
	type LiquidityMiningSplit = LiquidityMiningSplit;
	type StakingSplit = StakingSplit;
	type ImmediateTGEReleasePercent = ImmediateTGEReleasePercent;
	type TGEReleasePeriod = TGEReleasePeriod;
	type TGEReleaseBegin = TGEReleaseBegin;
	type VestingProvider = Vesting;
	type WeightInfo = weights::pallet_issuance_weights::ModuleWeight<Runtime>;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * DOLLARS;
}

impl pallet_vesting_mangata::Config for Runtime {
	type Event = Event;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = weights::pallet_vesting_mangata_weights::ModuleWeight<Runtime>;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 50;
}

parameter_types! {
	pub const Initialized: bool = false;
	pub const InitializationPayment: Perbill = Perbill::from_parts(214285700);
	pub const MaxInitContributorsBatchSizes: u32 = 100;
	pub const MinimumReward: Balance = 0;
	pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
	pub const SigantureNetworkIdentifier: &'static [u8] = b"mangata-";
}

impl pallet_crowdloan_rewards::Config for Runtime {
	type Event = Event;
	type Initialized = Initialized;
	type InitializationPayment = InitializationPayment;
	type MaxInitContributors = MaxInitContributorsBatchSizes;
	type MinimumReward = MinimumReward;
	type RewardAddressRelayVoteThreshold = RelaySignaturesThreshold;
	type NativeTokenId = MgrTokenId;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type RelayChainAccountId = sp_runtime::AccountId32;
	type RewardAddressChangeOrigin = EnsureRoot<AccountId>;
	type SignatureNetworkIdentifier = SigantureNetworkIdentifier;
	type RewardAddressAssociateOrigin = EnsureRoot<AccountId>;
	type VestingBlockNumber = BlockNumber;
	type VestingBlockProvider = System;
	type WeightInfo = weights::pallet_crowdloan_rewards_weights::ModuleWeight<Runtime>;
}

impl pallet_multipurpose_liquidity::Config for Runtime {
	type Event = Event;
	type MaxRelocks = MaxLocks;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type NativeCurrencyId = MgrTokenId;
	type VestingProvider = Vesting;
	type Xyk = Xyk;
	type WeightInfo = weights::pallet_multipurpose_liquidity_weights::ModuleWeight<Runtime>;
}

impl orml_unknown_tokens::Config for Runtime {
	type Event = Event;
}

impl orml_xcm::Config for Runtime {
	type Event = Event;
	type SovereignOrigin = EnsureRoot<AccountId>;
}

pub type AssetMetadataOf = AssetMetadata<Balance, CustomMetadata>;
type CurrencyAdapter = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;

pub struct SequentialIdWithCreation<T>(PhantomData<T>);
impl<T: orml_asset_registry::Config> AssetProcessor<TokenId, AssetMetadataOf>
	for SequentialIdWithCreation<T>
{
	fn pre_register(
		id: Option<TokenId>,
		asset_metadata: AssetMetadataOf,
	) -> Result<(TokenId, AssetMetadataOf), DispatchError> {
		let next_id = CurrencyAdapter::get_next_currency_id();
		let asset_id = id.unwrap_or(next_id);
		match asset_id.cmp(&next_id) {
			Ordering::Equal => CurrencyAdapter::create(&TreasuryAccount::get(), Default::default())
				.and_then(|created_asset_id| match created_asset_id.cmp(&asset_id) {
					Ordering::Equal => Ok((asset_id, asset_metadata)),
					_ => Err(orml_asset_registry::Error::<T>::InvalidAssetId.into()),
				}),
			Ordering::Less => Ok((asset_id, asset_metadata)),
			_ => Err(orml_asset_registry::Error::<T>::InvalidAssetId.into()),
		}
	}
}

pub struct AssetAuthority;
impl EnsureOriginWithArg<Origin, Option<u32>> for AssetAuthority {
	type Success = ();

	fn try_origin(origin: Origin, _asset_id: &Option<u32>) -> Result<Self::Success, Origin> {
		EnsureRoot::try_origin(origin)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin(_asset_id: &Option<u32>) -> Origin {
		EnsureRoot::successful_origin()
	}
}

impl orml_asset_registry::Config for Runtime {
	type Event = Event;
	type CustomMetadata = CustomMetadata;
	type AssetId = TokenId;
	type AuthorityOrigin = AssetAuthority;
	type AssetProcessor = SequentialIdWithCreation<Runtime>;
	type Balance = Balance;
	type WeightInfo = weights::orml_asset_registry_weights::ModuleWeight<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
		} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 3,

		// Snowbridge stuff
		Bridge: pallet_bridge::{Pallet, Call, Config, Storage, Event} = 4,
		Verifier: pallet_verifier::{Pallet, Call, Storage, Event, Config<T>} = 5,
		BridgedAsset: artemis_asset::{Pallet, Call, Config<T>, Storage, Event<T>} = 6,
		ETH: artemis_eth_app::{Pallet, Call, Storage, Event<T>} = 7,
		ERC20: artemis_erc20_app::{Pallet, Call, Storage, Event<T>} = 8,

		// Monetary stuff.
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<Runtime>} = 11,

		// Xyk stuff
		Xyk: pallet_xyk::{Pallet, Call, Storage, Event<T>, Config<T>} = 13,

		// Vesting
		Vesting: pallet_vesting_mangata::{Pallet, Call, Storage, Event<T>} = 17,

		// Crowdloan
		Crowdloan: pallet_crowdloan_rewards::{Pallet, Call, Storage, Event<T>} = 18,

		// Issuance
		Issuance: pallet_issuance::{Pallet, Event<T>, Storage, Call} = 19,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
		ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,

		// MultiPurposeLiquidity
		MultiPurposeLiquidity: pallet_multipurpose_liquidity::{Pallet, Call, Storage, Event<T>} = 25,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 30,
		PolkadotXcm: pallet_xcm::{Pallet, Storage, Call, Event<T>, Origin, Config} = 31,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,

		// ORML XCM
		XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 35,
		UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 36,
		OrmlXcm: orml_xcm::{Pallet, Call, Event<T>} = 37,
		AssetRegistry: orml_asset_registry::{Pallet, Call, Storage, Event<T>, Config<T>} = 38,

		// Governance stuff
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 41,
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 49,
		SudoOrigin: pallet_sudo_origin::{Pallet, Call, Event<T>} = 50,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 51,

		// Bootstrap
		Bootstrap: pallet_bootstrap::{Pallet, Call, Storage, Event<T>} = 53,
		Utility: pallet_utility::{Pallet, Call, Event} = 54,
	}
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[orml_tokens, Tokens]
		[orml_asset_registry, AssetRegistry]
		[parachain_staking, ParachainStaking]
		[pallet_xyk, Xyk]
		[pallet_treasury, Treasury]
		[pallet_collective, Council]
		[pallet_bootstrap, Bootstrap]
		[pallet_crowdloan_rewards, Crowdloan]
		[pallet_utility, Utility]
		[pallet_vesting_mangata, Vesting]
		[pallet_issuance, Issuance]
		[pallet_multipurpose_liquidity, MultiPurposeLiquidity]
	);
}

impl_runtime_apis! {

	impl ver_api::VerApi<Block> for Runtime {
		fn get_signer(
			tx: <Block as BlockT>::Extrinsic,
		) -> Option<(sp_runtime::AccountId32, u32)> {
			if let Some(sig) = tx.signature.clone(){
				let nonce: frame_system::CheckNonce<_> = sig.2.4;
				<Runtime as frame_system::Config>::Lookup::lookup(sig.0)
					.map(|addr| Some((addr, nonce.0))).expect("unknown address for signed extrinsic")
			}else{
				None
			}
		}

		fn is_storage_migration_scheduled() -> bool{
			System::read_events_no_consensus()
				.iter()
				.any(|record|
					matches!(record.event,
						Event::ParachainSystem( cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionApplied{relay_chain_block_num: _})))
		}

		fn store_seed(seed: sp_core::H256){
			// initialize has been called already so we can fetch number from the storage
			System::set_block_seed(&seed);
		}
	}

	impl xyk_runtime_api::XykApi<Block, Balance, TokenId, AccountId> for Runtime {
		fn calculate_sell_price(
			input_reserve: Balance,
			output_reserve: Balance,
			sell_amount: Balance
		) -> XYKRpcResult<Balance> {
			XYKRpcResult {
				price: Xyk::calculate_sell_price(input_reserve, output_reserve, sell_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_sell_price' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
			}
		}

		fn calculate_buy_price(
			input_reserve: Balance,
			output_reserve: Balance,
			buy_amount: Balance
		) -> XYKRpcResult<Balance> {
			XYKRpcResult {
				price: Xyk::calculate_buy_price(input_reserve, output_reserve, buy_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()

			}
		}

		fn calculate_sell_price_id(
			sold_token_id: TokenId,
			bought_token_id: TokenId,
			sell_amount: Balance
		) -> XYKRpcResult<Balance> {
			XYKRpcResult {
				price: Xyk::calculate_sell_price_id(sold_token_id, bought_token_id, sell_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_sell_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
			}
		}

		fn calculate_buy_price_id(
			sold_token_id: TokenId,
			bought_token_id: TokenId,
			buy_amount: Balance
		) -> XYKRpcResult<Balance> {
			XYKRpcResult {
				price: Xyk::calculate_buy_price_id(sold_token_id, bought_token_id, buy_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
			}
		}

		fn get_burn_amount(
			first_asset_id: TokenId,
			second_asset_id: TokenId,
			liquidity_asset_amount: Balance
		) -> RpcAmountsResult<Balance> {
			match Xyk::get_burn_amount(first_asset_id, second_asset_id, liquidity_asset_amount){
				Ok((first_asset_amount, second_asset_amount)) => RpcAmountsResult{
																	first_asset_amount,
																	second_asset_amount
																},
				Err(e) => {
					log::warn!(target:"xyk", "rpc 'XYK::get_burn_amount' error: '{:?}', returning default value instead", e);
					Default::default()
				},
			}
		}

		fn calculate_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> XYKRpcResult<Balance> {
			match Xyk::calculate_rewards_amount(user, liquidity_asset_id){
				Ok(claimable_rewards) => XYKRpcResult{
					price:claimable_rewards
				},
				Err(e) => {
						log::warn!(target:"xyk", "rpc 'XYK::calculate_rewards_amount' error: '{:?}', returning default value instead", e);
						Default::default()
				},
			}
		}

		fn get_max_instant_burn_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance {
			Xyk::get_max_instant_burn_amount(&user, liquidity_asset_id)
		}

		fn get_max_instant_unreserve_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance {
			Xyk::get_max_instant_unreserve_amount(&user, liquidity_asset_id)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl pallet_vesting_mangata_rpc_runtime_api::VestingMangataApi<Block, AccountId, TokenId, Balance, BlockNumber> for Runtime {
		fn get_vesting_locked_at(who: AccountId, token_id: TokenId, at_block_number: Option<BlockNumber>) -> VestingInfosWithLockedAt<Balance, BlockNumber>
		{
			match Vesting::get_vesting_locked_at(&who, token_id, at_block_number){
				Ok(vesting_infos_with_locked_at) => VestingInfosWithLockedAt{
					vesting_infos_with_locked_at: vesting_infos_with_locked_at
				},
				Err(e) => {
						log::warn!(target:"vesting", "rpc 'Vesting::get_vesting_locked_at' error: '{:?}', returning default value instead", e);
						Default::default()
				},
			}
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			let key = cumulus_pallet_aura_ext::get_block_signer_pub_key::<Runtime,Block>(&block);
			Executive::execute_block_ver_impl(block, key);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade parachain-template.");
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block_no_check(block: Block) -> Weight {
			Executive::execute_block_no_check(block)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");
		inherent_data.check_extrinsics(block)
	}
}

// replace validate block function with its expanded version
#[doc(hidden)]
mod parachain_validate_block {
	use super::*;

	#[no_mangle]
	#[cfg(not(feature = "std"))]
	unsafe fn validate_block(arguments: *const u8, arguments_len: usize) -> u64 {
		let params =
			cumulus_pallet_parachain_system::validate_block::polkadot_parachain::load_params(
				arguments,
				arguments_len,
			);
		let res =
            cumulus_pallet_parachain_system::validate_block::implementation::validate_block::<<Runtime
                                                                                              as
                                                                                              cumulus_pallet_parachain_system::validate_block::GetRuntimeBlockType>::RuntimeBlock,
                                                                                              cumulus_pallet_aura_ext::BlockExecutorVer<Runtime, Executive>,
                                                                                              Runtime,
                                                                                              CheckInherents>(params);
		cumulus_pallet_parachain_system::validate_block::polkadot_parachain::write_result(&res)
	}
}
