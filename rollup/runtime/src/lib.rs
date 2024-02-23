#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{alloc::string::String, Decode, Encode, MaxEncodedLen};
use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto,
		DispatchInfoOf, IdentifyAccount, NumberFor, PostDispatchInfoOf, Saturating,
		SignedExtension, StaticLookup, Verify, Zero,
	},
	transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, BoundedVec, DispatchError, FixedPointNumber, MultiAddress,
	MultiSignature, OpaqueExtrinsic, Perbill, Percent, Permill, RuntimeDebug,
};
use sp_std::{
	cmp::Ordering,
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	prelude::*,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use mangata_support::traits::{
	AssetRegistryApi, FeeLockTriggerTrait, PreValidateSwaps, ProofOfStakeRewardsApi,
};
pub use mangata_types::assets::{CustomMetadata, XcmMetadata, XykMetadata};
use sp_api::HeaderT;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};

// A few exports that help ease life for downstream crates.
#[cfg(feature = "runtime-benchmarks")]
pub use frame_support::traits::OriginTrait;
pub use frame_support::{
	construct_runtime,
	dispatch::{DispatchClass, DispatchResult},
	ensure, parameter_types,
	traits::{
		tokens::currency::{MultiTokenCurrency, MultiTokenImbalanceWithZeroTrait},
		ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, Contains, EnsureOrigin,
		EnsureOriginWithArg, Everything, ExistenceRequirement, FindAuthor, Get, Imbalance,
		InstanceFilter, KeyOwnerProofSystem, Randomness, StorageInfo, WithdrawReasons,
	},
	unsigned::TransactionValidityError,
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		ConstantMultiplier, IdentityFee, Weight,
	},
	PalletId, StorageValue,
};
pub use frame_system::{
	limits::{BlockLength, BlockWeights},
	Call as SystemCall, EnsureRoot,
};
pub use orml_tokens::Call as TokensCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use runtime_config::*;
use sp_application_crypto::ByteArray;
use static_assertions::const_assert;
use xyk_runtime_api::RpcAssetMetadata;

pub use orml_tokens::{self, MultiTokenCurrencyExtended};
pub use orml_traits::{
	asset_registry::{AssetMetadata, AssetProcessor},
	parameter_type_with_key,
};
pub use pallet_issuance::IssuanceInfo;
pub use pallet_sudo_mangata;
pub use pallet_sudo_origin;
pub use pallet_transaction_payment_mangata::{ConstFeeMultiplier, Multiplier, OnChargeTransaction};
pub use pallet_xyk::{self, AssetMetadataMutationTrait};
pub use scale_info::TypeInfo;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub mod runtime_config;
pub mod weights;
use runtime_config::config as cfg;
pub use runtime_config::{currency::*, runtime_types, tokens, types::*, CallType};

pub mod constants;

/// Block header type as expected by this runtime.
pub type Header = runtime_types::Header;
/// Block type as expected by this runtime.
pub type Block = runtime_types::Block<Runtime, RuntimeCall>;
/// A Block signed with a Justification
pub type SignedBlock = runtime_types::SignedBlock<Runtime, RuntimeCall>;
/// BlockId type as expected by this runtime.
pub type BlockId = runtime_types::BlockId<Runtime, RuntimeCall>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = runtime_types::SignedExtra<Runtime>;
/// The payload being signed in transactions.
pub type SignedPayload = runtime_types::SignedPayload<Runtime, RuntimeCall>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = runtime_types::UncheckedExtrinsic<Runtime, RuntimeCall>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = runtime_types::CheckedExtrinsic<Runtime, RuntimeCall>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

type Migrations = ();

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	/// Opaque block header type.
	pub type Header = runtime_types::Header;
	/// Opaque block type.
	pub type Block = runtime_types::OpaqueBlock;
	/// Opaque block identifier type.
	pub type BlockId = runtime_types::OpaqueBlockId;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
		pub grandpa: Grandpa,
	}
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("rollup-chain"),
	impl_name: create_runtime_str!("rollup-chain"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 100,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

// Configure FRAME pallets to include in runtime.
impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = cfg::frame_system::RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = cfg::frame_system::RuntimeBlockLength;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The block type.
	type Block = Block;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = cfg::frame_system::BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Runtime version.
	type Version = cfg::frame_system::Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = ();
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = weights::frame_system_weights::ModuleWeight<Runtime>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = cfg::frame_system::SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = ();
	/// The maximum number of consumers allowed on a single account.
	type MaxConsumers = cfg::frame_system::MaxConsumers;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = cfg::pallet_timestamp::MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp_weights::ModuleWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = ParachainStaking;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = cfg::pallet_treasury::TreasuryPalletId;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::RxTokenId>;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = ();
	type ProposalBond = cfg::pallet_treasury::ProposalBond;
	type ProposalBondMinimum = cfg::pallet_treasury::ProposalBondMinimum;
	type ProposalBondMaximum = cfg::pallet_treasury::ProposalBondMaximum;
	type SpendPeriod = cfg::pallet_treasury::SpendPeriod;
	type Burn = cfg::pallet_treasury::Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = weights::pallet_treasury_weights::ModuleWeight<Runtime>;
	type MaxApprovals = cfg::pallet_treasury::MaxApprovals;
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
}

parameter_types! {
	pub TreasuryAccount: AccountId = cfg::TreasuryPalletIdOf::<Runtime>::get().into_account_truncating();
}

// The MaxLocks (on a who-token_id pair) that is allowed by orml_tokens
// must exceed the total possible locks that can be applied to it, ALL pallets considered
// This is because orml_tokens uses BoundedVec for Locks storage item and does not inform on failure
// Balances uses WeakBoundedVec and so does not fail
const_assert!(
	<Runtime as orml_tokens::Config>::MaxLocks::get() >=
		<Runtime as pallet_vesting_mangata::Config>::MAX_VESTING_SCHEDULES
);

const_assert!(
	<Runtime as pallet_proof_of_stake::Config>::RewardsSchedulesLimit::get() >=
		(<Runtime as pallet_proof_of_stake::Config>::SchedulesPerBlock::get() - 1) *
			<Runtime as parachain_staking::Config>::BlocksPerRound::get()
);

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = TokenId;
	type WeightInfo = weights::orml_tokens_weights::ModuleWeight<Runtime>;
	type ExistentialDeposits = cfg::orml_tokens::ExistentialDeposits;
	type MaxLocks = cfg::orml_tokens::MaxLocks;
	type DustRemovalWhitelist =
		cfg::orml_tokens::DustRemovalWhitelist<cfg::TreasuryAccountIdOf<Runtime>>;
	type CurrencyHooks = ();
	type MaxReserves = ();
	type ReserveIdentifier = cfg::orml_tokens::ReserveIdentifier;
}

impl pallet_xyk::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = Maintenance;
	type ActivationReservesProvider = MultiPurposeLiquidity;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type NativeCurrencyId = tokens::RxTokenId;
	type TreasuryPalletId = cfg::TreasuryPalletIdOf<Runtime>;
	type BnbTreasurySubAccDerive = cfg::pallet_xyk::BnbTreasurySubAccDerive;
	type PoolFeePercentage = cfg::pallet_xyk::PoolFeePercentage;
	type TreasuryFeePercentage = cfg::pallet_xyk::TreasuryFeePercentage;
	type BuyAndBurnFeePercentage = cfg::pallet_xyk::BuyAndBurnFeePercentage;
	type LiquidityMiningRewards = ProofOfStake;
	type VestingProvider = Vesting;
	type DisallowedPools = Bootstrap;
	type DisabledTokens =
		(cfg::pallet_xyk::TestTokensFilter, cfg::pallet_xyk::AssetRegisterFilter<Runtime>);
	type AssetMetadataMutation = cfg::pallet_xyk::AssetMetadataMutation<Runtime>;
	type WeightInfo = weights::pallet_xyk_weights::ModuleWeight<Runtime>;
}

impl pallet_proof_of_stake::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ActivationReservesProvider = MultiPurposeLiquidity;
	type NativeCurrencyId = tokens::RxTokenId;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type LiquidityMiningIssuanceVault = cfg::pallet_issuance::LiquidityMiningIssuanceVault;
	type RewardsDistributionPeriod = cfg::SessionLenghtOf<Runtime>;
	type WeightInfo = weights::pallet_proof_of_stake_weights::ModuleWeight<Runtime>;
	type RewardsSchedulesLimit = cfg::pallet_proof_of_stake::RewardsSchedulesLimit;
	type Min3rdPartyRewardValutationPerSession =
		cfg::pallet_proof_of_stake::Min3rdPartyRewardValutationPerSession;
	type Min3rdPartyRewardVolume = cfg::pallet_proof_of_stake::Min3rdPartyRewardVolume;
	type SchedulesPerBlock = cfg::pallet_proof_of_stake::SchedulesPerBlock;
	type ValuationApi = Xyk;
}

impl pallet_bootstrap::BootstrapBenchmarkingConfig for Runtime {}

impl pallet_bootstrap::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = Maintenance;
	type PoolCreateApi = Xyk;
	type DefaultBootstrapPromotedPoolWeight =
		cfg::pallet_bootstrap::DefaultBootstrapPromotedPoolWeight;
	type BootstrapUpdateBuffer = cfg::pallet_bootstrap::BootstrapUpdateBuffer;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type VestingProvider = Vesting;
	type TreasuryPalletId = cfg::TreasuryPalletIdOf<Runtime>;
	type RewardsApi = ProofOfStake;
	type ClearStorageLimit = cfg::pallet_bootstrap::ClearStorageLimit;
	type WeightInfo = weights::pallet_bootstrap_weights::ModuleWeight<Runtime>;
	type AssetRegistryApi = cfg::pallet_bootstrap::EnableAssetPoolApi<Runtime>;
}

impl pallet_utility_mangata::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type DisallowedInBatch = cfg::pallet_utility_mangata::DisallowedInBatch<Runtime>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility_mangata_weights::ModuleWeight<Runtime>;
}

use cfg::pallet_transaction_payment_mangata::{
	FeeHelpers, OnChargeHandler, OneCurrencyOnChargeAdapter, ToAuthor, TriggerEvent,
};

// TODO: renaming foo causes compiler error
pub struct Foo<T>(PhantomData<T>);
impl<T> TriggerEvent<T::AccountId> for Foo<T>
where
	T: frame_system::Config<AccountId = sp_runtime::AccountId32>,
{
	fn trigger(who: T::AccountId, fee: u128, tip: u128) {
		TransactionPayment::deposit_event(
			pallet_transaction_payment_mangata::Event::<Runtime>::TransactionFeePaid {
				who,
				actual_fee: fee,
				tip,
			},
		);
	}
}

impl Into<CallType> for RuntimeCall {
	fn into(self) -> CallType {
		match self {
			RuntimeCall::Xyk(pallet_xyk::Call::sell_asset {
				sold_asset_id,
				sold_asset_amount,
				bought_asset_id,
				min_amount_out,
				..
			}) => CallType::AtomicSell {
				sold_asset_id,
				sold_asset_amount,
				bought_asset_id,
				min_amount_out,
			},
			RuntimeCall::Xyk(pallet_xyk::Call::buy_asset {
				sold_asset_id,
				bought_asset_amount,
				bought_asset_id,
				max_amount_in,
				..
			}) => CallType::AtomicBuy {
				sold_asset_id,
				bought_asset_amount,
				bought_asset_id,
				max_amount_in,
			},
			RuntimeCall::Xyk(pallet_xyk::Call::multiswap_sell_asset {
				swap_token_list,
				sold_asset_amount,
				min_amount_out,
				..
			}) => CallType::MultiSell { swap_token_list, sold_asset_amount, min_amount_out },
			RuntimeCall::Xyk(pallet_xyk::Call::multiswap_buy_asset {
				swap_token_list,
				bought_asset_amount,
				max_amount_in,
				..
			}) => CallType::MultiBuy { swap_token_list, bought_asset_amount, max_amount_in },
			RuntimeCall::Xyk(pallet_xyk::Call::compound_rewards { .. }) =>
				CallType::CompoundRewards,
			RuntimeCall::Xyk(pallet_xyk::Call::provide_liquidity_with_conversion { .. }) =>
				CallType::ProvideLiquidityWithConversion,
			RuntimeCall::FeeLock(pallet_fee_lock::Call::unlock_fee { .. }) => CallType::UnlockFee,
			_ => CallType::Other,
		}
	}
}

use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{generic::ExtendedCall, AccountId32};
use sp_std::{fmt::Write, prelude::*};

impl ExtendedCall for RuntimeCall {
	fn context(&self) -> Option<(String, String)> {
		match self {
			RuntimeCall::Xyk(pallet_xyk::Call::sell_asset {
				sold_asset_id,
				sold_asset_amount,
				bought_asset_id,
				min_amount_out,
				..
			}) => {
				let mut buffer = String::new();
				let _ = write!(&mut buffer, "sold_asset_id: {sold_asset_id}\n");
				let _ = write!(&mut buffer, "sold_asset_amount: {sold_asset_amount}\n");
				let _ = write!(&mut buffer, "bought_asset_id: {bought_asset_id}\n");
				let _ = write!(&mut buffer, "min_amount_out: {min_amount_out}\n");
				Some(("xyk::sell_asset".to_string(), buffer))
			},
			RuntimeCall::Xyk(pallet_xyk::Call::buy_asset {
				sold_asset_id,
				bought_asset_amount,
				bought_asset_id,
				max_amount_in,
				..
			}) => {
				let mut buffer = String::new();
				let _ = write!(&mut buffer, "sold_asset_id: {sold_asset_id}\n");
				let _ = write!(&mut buffer, "bought_asset_amount: {bought_asset_amount}\n");
				let _ = write!(&mut buffer, "bought_asset_id: {bought_asset_id}\n");
				let _ = write!(&mut buffer, "max_amount_in: {max_amount_in}\n");
				Some(("xyk::buy_asset".to_string(), buffer))
			},
			RuntimeCall::Tokens(orml_tokens::Call::transfer { dest, currency_id, amount }) => {
				let mut buffer = String::new();
				let _ = write!(&mut buffer, "dest: {dest:?}\n");
				let _ = write!(&mut buffer, "currency_id: {currency_id}\n");
				let _ = write!(&mut buffer, "amount: {amount}\n");
				Some(("orml_tokens::transfer".to_string(), buffer))
			},
			_ => Some(("todo".to_string(), "todo".to_string())),
		}
	}
}

pub type OnChargeTransactionHandler<T> = OneCurrencyOnChargeAdapter<
	orml_tokens::MultiTokenCurrencyAdapter<T>,
	ToAuthor<T>,
	tokens::RxTokenId,
	Foo<T>,
>;

impl pallet_transaction_payment_mangata::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = OnChargeHandler<
		orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
		ToAuthor<Runtime>,
		OnChargeTransactionHandler<Runtime>,
		FeeLock,
	>;
	type LengthToFee = cfg::pallet_transaction_payment_mangata::LengthToFee;
	type WeightToFee = constants::fee::WeightToFee;
	type FeeMultiplierUpdate = cfg::pallet_transaction_payment_mangata::FeeMultiplierUpdate;
	type OperationalFeeMultiplier =
		cfg::pallet_transaction_payment_mangata::OperationalFeeMultiplier;
}

parameter_types! {
	pub const MaxCuratedTokens: u32 = 100;
}

impl pallet_fee_lock::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxCuratedTokens = cfg::pallet_fee_lock::MaxCuratedTokens;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type PoolReservesProvider = Xyk;
	type NativeTokenId = tokens::RxTokenId;
	type WeightInfo = weights::pallet_fee_lock_weights::ModuleWeight<Runtime>;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type MaxAuthorities = cfg::pallet_aura::MaxAuthorities;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;

	#[cfg(feature = "experimental")]
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_sudo_mangata::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
}

impl pallet_sudo_origin::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type SudoOrigin = cfg::pallet_sudo_origin::SudoOrigin<CouncilCollective>;
}

type CouncilCollective = pallet_collective_mangata::Instance1;
impl pallet_collective_mangata::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = cfg::pallet_collective_mangata::CouncilMotionDuration;
	type ProposalCloseDelay = cfg::pallet_collective_mangata::CouncilProposalCloseDelay;
	type MaxProposals = cfg::pallet_collective_mangata::CouncilMaxProposals;
	type MaxMembers = cfg::pallet_collective_mangata::CouncilMaxMembers;
	type FoundationAccountsProvider = cfg::pallet_maintenance::FoundationAccountsProvider<Runtime>;
	type DefaultVote = pallet_collective_mangata::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective_mangata_weights::ModuleWeight<Runtime>;
	type SetMembersOrigin = cfg::pallet_collective_mangata::SetMembersOrigin<Self::AccountId>;
	type MaxProposalWeight = cfg::pallet_collective_mangata::MaxProposalWeight;
}

// To ensure that BlocksPerRound is not zero, breaking issuance calculations
// Also since 1 block is used for session change, atleast 1 block more needed for extrinsics to work
const_assert!(<Runtime as parachain_staking::Config>::BlocksPerRound::get() >= 2);

impl parachain_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StakingReservesProvider = MultiPurposeLiquidity;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
	type BlocksPerRound = cfg::parachain_staking::BlocksPerRound;
	type LeaveCandidatesDelay = cfg::parachain_staking::LeaveCandidatesDelay;
	type CandidateBondDelay = cfg::parachain_staking::CandidateBondDelay;
	type LeaveDelegatorsDelay = cfg::parachain_staking::LeaveDelegatorsDelay;
	type RevokeDelegationDelay = cfg::parachain_staking::RevokeDelegationDelay;
	type DelegationBondDelay = cfg::parachain_staking::DelegationBondDelay;
	type RewardPaymentDelay = cfg::parachain_staking::RewardPaymentDelay;
	type MinSelectedCandidates = cfg::parachain_staking::MinSelectedCandidates;
	type MaxCollatorCandidates = cfg::parachain_staking::MaxCollatorCandidates;
	type MaxTotalDelegatorsPerCandidate = cfg::parachain_staking::MaxTotalDelegatorsPerCandidate;
	type MaxDelegatorsPerCandidate = cfg::parachain_staking::MaxDelegatorsPerCandidate;
	type MaxDelegationsPerDelegator = cfg::parachain_staking::MaxDelegationsPerDelegator;
	type DefaultCollatorCommission = cfg::parachain_staking::DefaultCollatorCommission;
	type MinCollatorStk = cfg::parachain_staking::MinCollatorStk;
	type MinCandidateStk = cfg::parachain_staking::MinCandidateStk;
	type MinDelegation = cfg::parachain_staking::MinDelegatorStk;
	type NativeTokenId = tokens::RxTokenId;
	type StakingLiquidityTokenValuator = Xyk;
	type Issuance = Issuance;
	type StakingIssuanceVault = cfg::parachain_staking::StakingIssuanceVaultOf<Runtime>;
	type FallbackProvider = Council;
	type WeightInfo = weights::parachain_staking_weights::ModuleWeight<Runtime>;
	type DefaultPayoutLimit = cfg::parachain_staking::DefaultPayoutLimit;
}

impl parachain_staking::StakingBenchmarkConfig for Runtime {
	#[cfg(feature = "runtime-benchmarks")]
	type Balance = Balance;
	#[cfg(feature = "runtime-benchmarks")]
	type CurrencyId = TokenId;
	#[cfg(feature = "runtime-benchmarks")]
	type RewardsApi = ProofOfStake;
	#[cfg(feature = "runtime-benchmarks")]
	type Xyk = Xyk;
}

impl pallet_xyk::XykBenchmarkingConfig for Runtime {}

// Issuance history must be kept for atleast the staking reward delay
const_assert!(
	<Runtime as parachain_staking::Config>::RewardPaymentDelay::get() <=
		<Runtime as pallet_issuance::Config>::HistoryLimit::get()
);

impl pallet_issuance::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NativeCurrencyId = tokens::RxTokenId;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type BlocksPerRound = cfg::parachain_staking::BlocksPerRound;
	type HistoryLimit = cfg::pallet_issuance::HistoryLimit;
	type LiquidityMiningIssuanceVault = cfg::pallet_issuance::LiquidityMiningIssuanceVault;
	type StakingIssuanceVault = cfg::pallet_issuance::StakingIssuanceVault;
	type TotalCrowdloanAllocation = cfg::pallet_issuance::TotalCrowdloanAllocation;
	type IssuanceCap = cfg::pallet_issuance::IssuanceCap;
	type LinearIssuanceBlocks = cfg::pallet_issuance::LinearIssuanceBlocks;
	type LiquidityMiningSplit = cfg::pallet_issuance::LiquidityMiningSplit;
	type StakingSplit = cfg::pallet_issuance::StakingSplit;
	type ImmediateTGEReleasePercent = cfg::pallet_issuance::ImmediateTGEReleasePercent;
	type TGEReleasePeriod = cfg::pallet_issuance::TGEReleasePeriod;
	type TGEReleaseBegin = cfg::pallet_issuance::TGEReleaseBegin;
	type VestingProvider = Vesting;
	type WeightInfo = weights::pallet_issuance_weights::ModuleWeight<Runtime>;
	type LiquidityMiningApi = ProofOfStake;
}

impl pallet_vesting_mangata::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = cfg::pallet_vesting_mangata::MinVestedTransfer;
	type WeightInfo = weights::pallet_vesting_mangata_weights::ModuleWeight<Runtime>;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 50;
	type UnvestedFundsAllowedWithdrawReasons =
		cfg::pallet_vesting_mangata::UnvestedFundsAllowedWithdrawReasons;
}

impl pallet_crowdloan_rewards::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Initialized = cfg::pallet_crowdloan_rewards::Initialized;
	type InitializationPayment = cfg::pallet_crowdloan_rewards::InitializationPayment;
	type MaxInitContributors = cfg::pallet_crowdloan_rewards::MaxInitContributorsBatchSizes;
	type MinimumReward = cfg::pallet_crowdloan_rewards::MinimumReward;
	type RewardAddressRelayVoteThreshold = cfg::pallet_crowdloan_rewards::RelaySignaturesThreshold;
	type NativeTokenId = tokens::RxTokenId;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type RelayChainAccountId = sp_runtime::AccountId32;
	type RewardAddressChangeOrigin = EnsureRoot<AccountId>;
	type SignatureNetworkIdentifier = cfg::pallet_crowdloan_rewards::SigantureNetworkIdentifier;
	type RewardAddressAssociateOrigin = EnsureRoot<AccountId>;
	type VestingBlockNumber = BlockNumber;
	type VestingBlockProvider = System;
	type VestingProvider = Vesting;
	type WeightInfo = weights::pallet_crowdloan_rewards_weights::ModuleWeight<Runtime>;
}

impl pallet_multipurpose_liquidity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxRelocks = cfg::MaxLocksOf<Runtime>;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type NativeCurrencyId = tokens::RxTokenId;
	type VestingProvider = Vesting;
	type Xyk = Xyk;
	type WeightInfo = weights::pallet_multipurpose_liquidity_weights::ModuleWeight<Runtime>;
}

impl orml_asset_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CustomMetadata = CustomMetadata;
	type AssetId = TokenId;
	type AuthorityOrigin = cfg::orml_asset_registry::AssetAuthority<Runtime>;
	type AssetProcessor = cfg::orml_asset_registry::SequentialIdWithCreation<Runtime>;
	type Balance = Balance;
	type WeightInfo = weights::orml_asset_registry_weights::ModuleWeight<Runtime>;
	type StringLimit = cfg::orml_asset_registry::StringLimit;
}

use cfg::pallet_proxy::ProxyType;

// TODO: ideally should be moved to common runtime
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			_ if matches!(c, RuntimeCall::Utility(..)) => true,
			ProxyType::AutoCompound => {
				matches!(
					c,
					RuntimeCall::Xyk(pallet_xyk::Call::provide_liquidity_with_conversion { .. }) |
						RuntimeCall::Xyk(pallet_xyk::Call::compound_rewards { .. })
				)
			},
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::RxTokenId>;
	type ProxyType = cfg::pallet_proxy::ProxyType;
	type ProxyDepositBase = cfg::pallet_proxy::ProxyDepositBase;
	type ProxyDepositFactor = cfg::pallet_proxy::ProxyDepositFactor;
	type MaxProxies = frame_support::traits::ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = frame_support::traits::ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = cfg::pallet_proxy::AnnouncementDepositBase;
	type AnnouncementDepositFactor = cfg::pallet_proxy::AnnouncementDepositFactor;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::RxTokenId>;
	type BasicDeposit = cfg::pallet_identity::BasicDeposit;
	type FieldDeposit = cfg::pallet_identity::FieldDeposit;
	type SubAccountDeposit = cfg::pallet_identity::SubAccountDeposit;
	type MaxSubAccounts = cfg::pallet_identity::MaxSubAccounts;
	type MaxAdditionalFields = cfg::pallet_identity::MaxAdditionalFields;
	type MaxRegistrars = cfg::pallet_identity::MaxRegistrars;
	type ForceOrigin = cfg::pallet_identity::IdentityForceOrigin;
	type RegistrarOrigin = cfg::pallet_identity::IdentityRegistrarOrigin;
	type Slashed = Treasury;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

impl pallet_maintenance::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type FoundationAccountsProvider = cfg::pallet_maintenance::FoundationAccountsProvider<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub struct Runtime {
		// System support stuff.
		System: frame_system = 0,
		Timestamp: pallet_timestamp = 2,
		Utility: pallet_utility_mangata = 4,
		Proxy: pallet_proxy = 5,
		Maintenance: pallet_maintenance = 6,

		// Monetary stuff.
		Tokens: orml_tokens = 10,
		TransactionPayment: pallet_transaction_payment_mangata = 11,

		// Xyk stuff
		Xyk: pallet_xyk = 13,
		ProofOfStake: pallet_proof_of_stake = 14,

		// Fee Locks
		FeeLock: pallet_fee_lock = 15,

		// Vesting
		Vesting: pallet_vesting_mangata = 17,

		// Crowdloan
		Crowdloan: pallet_crowdloan_rewards = 18,

		// Issuance
		Issuance: pallet_issuance = 19,

		// MultiPurposeLiquidity
		MultiPurposeLiquidity: pallet_multipurpose_liquidity = 20,

		// Bootstrap
		Bootstrap: pallet_bootstrap = 21,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship = 30,
		ParachainStaking: parachain_staking = 31,
		Session: pallet_session = 32,
		Aura: pallet_aura = 33,
		Grandpa: pallet_grandpa = 34,

		AssetRegistry: orml_asset_registry = 53,

		// Governance stuff
		Treasury: pallet_treasury = 60,
		Sudo: pallet_sudo_mangata = 61,
		SudoOrigin: pallet_sudo_origin = 62,
		Council: pallet_collective_mangata::<Instance1> = 63,
		Identity: pallet_identity = 64,
	}
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[orml_asset_registry, AssetRegistry]
		[orml_tokens, Tokens]
		[parachain_staking, ParachainStaking]
		[pallet_xyk, Xyk]
		[pallet_treasury, Treasury]
		[pallet_collective_mangata, Council]
		[pallet_bootstrap, Bootstrap]
		[pallet_crowdloan_rewards, Crowdloan]
		[pallet_utility_mangata, Utility]
		[pallet_vesting_mangata, Vesting]
		[pallet_issuance, Issuance]
		[pallet_multipurpose_liquidity, MultiPurposeLiquidity]
		[pallet_fee_lock, FeeLock]
		[pallet_proof_of_stake, ProofOfStake]
	);
}
use codec::alloc::string::ToString;

use frame_support::dispatch::GetDispatchInfo;

impl_runtime_apis! {
	impl metamask_signature_runtime_api::MetamaskSignatureRuntimeApi<Block> for Runtime {
		fn get_eip712_sign_data(call: Vec<u8>) -> String{
			if let Ok(extrinsic) = UncheckedExtrinsic::decode(& mut call.as_ref()) {
				if let Some((method, params)) = extrinsic.function.context() {
					metamask_signature_runtime_api::eip712_payload(method, params)
				}else{
					Default::default()
				}
			}else{
				Default::default()
			}
		}
	}

	impl proof_of_stake_runtime_api::ProofOfStakeApi<Block, Balance , TokenId,  AccountId> for Runtime{
		fn calculate_native_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance{
			pallet_proof_of_stake::Pallet::<Runtime>::calculate_native_rewards_amount(user, liquidity_asset_id)
				.unwrap_or_default()
		}

		fn calculate_3rdparty_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
			reward_asset_id: TokenId,
		) -> Balance{
			pallet_proof_of_stake::Pallet::<Runtime>::calculate_3rdparty_rewards_amount(user, liquidity_asset_id, reward_asset_id)
				.unwrap_or_default()
		}

		fn calculate_3rdparty_rewards_all(
			user: AccountId,
		) -> Vec<(TokenId, TokenId, Balance)>{
			pallet_proof_of_stake::Pallet::<Runtime>::calculate_3rdparty_rewards_all(user)
		}
	}

	impl ver_api::VerApi<Block> for Runtime {
		fn get_signer(
			tx: <Block as BlockT>::Extrinsic,
		) -> Option<(sp_runtime::AccountId32, u32)> {
			if let Some(sig) = tx.signature.clone(){
				let nonce: frame_system::CheckNonce<_> = sig.2.4;
				<Runtime as frame_system::Config>::Lookup::lookup(sig.0)
					.ok()
					.and_then(|addr| Some((addr, nonce.0)))
			}else{
				None
			}
		}

		fn is_storage_migration_scheduled() -> bool{
			Executive::runtime_upgraded_peek()
		}

		fn store_seed(seed: sp_core::H256){
			// initialize has been called already so we can fetch number from the storage
			System::set_block_seed(&seed);
		}

		fn get_previous_block_txs() -> Vec<Vec<u8>> {
			System::get_previous_blocks_txs()
		}

		fn pop_txs(count: u64) -> Vec<Vec<u8>> {
			System::pop_txs(count as usize)
		}

		fn create_enqueue_txs_inherent(txs: Vec<<Block as BlockT>::Extrinsic>) -> <Block as BlockT>::Extrinsic {
			for t in txs.iter() {
				if let Some((_, _, extra)) = &t.signature {
					let _ = extra.additional_signed();
				}
			}
			UncheckedExtrinsic::new_unsigned(
					RuntimeCall::System(frame_system::Call::enqueue_txs{txs:
						txs.into_iter()
							.map(|tx| (
									tx.signature.clone().and_then(|sig|
										<Runtime as frame_system::Config>::Lookup::lookup(sig.0).ok()
									),
									tx.encode())
							).collect()}))
		}

		fn can_enqueue_txs() -> bool {
			System::can_enqueue_txs()
		}

		fn start_prevalidation() {
			System::set_prevalidation()
		}

		fn account_extrinsic_dispatch_weight(consumed: ver_api::ConsumedWeight, tx: <Block as BlockT>::Extrinsic) -> Result<ver_api::ConsumedWeight, ()> {
			let info = tx.get_dispatch_info();
			let maximum_weight = <Runtime as frame_system::Config>::BlockWeights::get();
			frame_system::calculate_consumed_weight::<RuntimeCall>(maximum_weight, consumed, &info)
			.or(Err(()))
		}

	}

	impl ver_api::VerNonceApi<Block, AccountId> for Runtime {
		fn enqueued_txs_count(acc: AccountId) -> u64 {

			System::enqueued_txs_count(&acc) as u64
		}
	}

	impl xyk_runtime_api::XykApi<Block, Balance, TokenId, AccountId> for Runtime {
		fn calculate_sell_price(
			input_reserve: Balance,
			output_reserve: Balance,
			sell_amount: Balance
		) -> Balance {
			Xyk::calculate_sell_price(input_reserve, output_reserve, sell_amount)
				.map_err(|e|
						 {
							 log::warn!(target:"xyk", "rpc 'XYK::calculate_sell_price' error: '{:?}', returning default value instead", e);
							 e
						 }
						).unwrap_or_default()
		}

		fn calculate_buy_price(
			input_reserve: Balance,
			output_reserve: Balance,
			buy_amount: Balance
		) -> Balance {
				Xyk::calculate_buy_price(input_reserve, output_reserve, buy_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()

		}

		fn calculate_sell_price_id(
			sold_token_id: TokenId,
			bought_token_id: TokenId,
			sell_amount: Balance
		) -> Balance {
				 Xyk::calculate_sell_price_id(sold_token_id, bought_token_id, sell_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_sell_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn calculate_buy_price_id(
			sold_token_id: TokenId,
			bought_token_id: TokenId,
			buy_amount: Balance
		) -> Balance {
				Xyk::calculate_buy_price_id(sold_token_id, bought_token_id, buy_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn get_burn_amount(
			first_asset_id: TokenId,
			second_asset_id: TokenId,
			liquidity_asset_amount: Balance
		) -> (Balance, Balance) {
			Xyk::get_burn_amount(first_asset_id, second_asset_id, liquidity_asset_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
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

		fn calculate_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance {
			ProofOfStake::calculate_rewards_amount(user, liquidity_asset_id)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn calculate_balanced_sell_amount(
			total_amount: Balance,
			reserve_amount: Balance,
		) -> Balance {
				 Xyk::calculate_balanced_sell_amount(total_amount, reserve_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_balanced_sell_amount' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn get_liq_tokens_for_trading() -> Vec<TokenId> {
			Xyk::get_liq_tokens_for_trading()
				.map_err(|e|
					{
						log::warn!(target:"xyk", "rpc 'XYK::get_liq_tokens_for_trading' error: '{:?}', returning default value instead", e);
						e
					}
				).unwrap_or_default()
		}

		fn is_sell_asset_lock_free(
			path: Vec<TokenId>,
			input_amount: Balance,
			) -> Option<bool>{
			match (path.len(), pallet_fee_lock::FeeLockMetadata::<Runtime>::get()) {
				(length, _) if length < 2 => {
					None
				}
				(2, Some(feelock)) => {
					let input = path.get(0)?;
					let output = path.get(1)?;
					let output_amount = Xyk::calculate_sell_price_id(*input, *output, input_amount).ok()?;
					Some(
					FeeHelpers::<
								Runtime,
								orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
								ToAuthor<Runtime>,
								OnChargeTransactionHandler<Runtime>,
								FeeLock,
								>::is_high_value_swap(&feelock, *input, input_amount)
									||
					FeeHelpers::<
								Runtime,
								orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
								ToAuthor<Runtime>,
								OnChargeTransactionHandler<Runtime>,
								FeeLock,
								>::is_high_value_swap(&feelock, *output, output_amount)
								)
				}
				(_,  None) => {
					Some(false)
				}
				(_,  Some(_)) => {
					Some(true)
				}
			}
		}

		fn is_buy_asset_lock_free(
			path: Vec<TokenId>,
			input_amount: Balance,
			) -> Option<bool>{
			match (path.len(), pallet_fee_lock::FeeLockMetadata::<Runtime>::get()) {
				(length, _) if length < 2 => {
					None
				}
				(2, Some(feelock)) => {
					let input = path.get(0)?;
					let output = path.get(1)?;
					let output_amount = Xyk::calculate_buy_price_id(*input, *output, input_amount).ok()?;
					Some(
					FeeHelpers::<
								Runtime,
								orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
								ToAuthor<Runtime>,
								OnChargeTransactionHandler<Runtime>,
								FeeLock,
								>::is_high_value_swap(&feelock, *input, input_amount)
									||
					FeeHelpers::<
								Runtime,
								orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
								ToAuthor<Runtime>,
								OnChargeTransactionHandler<Runtime>,
								FeeLock,
								>::is_high_value_swap(&feelock, *output, output_amount)
								)
				}
				(_, None) => {
					Some(false)
				}
				(_, Some(_)) => {
					Some(true)
				}
			}
		}

		fn get_tradeable_tokens() -> Vec<RpcAssetMetadata<TokenId>> {
			orml_asset_registry::Metadata::<Runtime>::iter()
			.filter_map(|(token_id, metadata)| {
				if !metadata.name.is_empty()
					&& !metadata.symbol.is_empty()
					&& metadata.additional.xyk.as_ref().map_or(true, |xyk| !xyk.operations_disabled)
				{
					let rpc_metadata = RpcAssetMetadata {
						token_id: token_id,
						decimals: metadata.decimals,
						name: metadata.name.to_vec(),
						symbol: metadata.symbol.to_vec(),
					};
					Some(rpc_metadata)
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			let header = block.header();
			let author =
				// since execute_block_ver_impl is called at the end here
				// and execute_block_ver_impl initializes the block
				// That means that at this point the block is not initialized
				// That means session was not on_initialize
				// That means we can use session validator set
				// as they have not been overwritten yet 
				
				pallet_session::FindAccountFromAuthorIndex::<Runtime, Aura>::find_author(
					header.digest().logs().iter().filter_map(|d| d.as_pre_runtime())
				).expect("Could not find AuRa author index!")
				.to_raw_vec();
			Executive::execute_block_ver_impl(block, author);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
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

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
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

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			_authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_mangata_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_mangata_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_mangata::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_mangata_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment_mangata::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment_mangata::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// TODO: Maybe checks should be overridden with `frame_try_runtime::UpgradeCheckSelect::All`
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, cfg::frame_system::RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			log::info!(
				target: "node-runtime",
				"try-runtime: executing block {:?} / root checks: {:?} / try-state-select: {:?}",
				block.header.hash(),
				state_root_check,
				select,
			);
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}
}
