#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	construct_runtime,
	dispatch::{DispatchClass, DispatchResult},
	ensure, parameter_types,
	traits::{
		tokens::currency::{MultiTokenCurrency, MultiTokenImbalanceWithZeroTrait},
		Contains, EnsureOrigin, EnsureOriginWithArg, Everything, ExistenceRequirement, Get,
		Imbalance, InstanceFilter, WithdrawReasons,
	},
	unsigned::TransactionValidityError,
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		ConstantMultiplier, Weight,
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
use orml_tokens::MultiTokenCurrencyExtended;
use orml_traits::{
	asset_registry::{AssetMetadata, AssetProcessor},
	parameter_type_with_key,
};
pub use pallet_sudo_mangata;
use pallet_transaction_payment_mangata::{ConstFeeMultiplier, Multiplier, OnChargeTransaction};
use pallet_vesting_mangata_rpc_runtime_api::VestingInfosWithLockedAt;
// Polkadot Imports
pub use polkadot_runtime_common::BlockHashCount;
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
	ApplyExtrinsicResult, DispatchError, FixedPointNumber, Percent, RuntimeDebug,
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
pub use common_runtime::{currency::*, deposit, tokens, runtime_types};

use mangata_support::traits::{
	AssetRegistryApi, FeeLockTriggerTrait, PreValidateSwaps, ProofOfStakeRewardsApi,
};
pub use mangata_types::{
	assets::{CustomMetadata, XcmMetadata, XykMetadata},
	AccountId, Address, Amount, Balance, BlockNumber, Hash, Index, Signature, TokenId,
};
pub use pallet_issuance::IssuanceInfo;
pub use pallet_sudo_origin;
pub use pallet_xyk;
// XCM Imports
use pallet_xyk::AssetMetadataMutationTrait;
use xyk_runtime_api::{RpcAmountsResult, XYKRpcResult};

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod constants;
mod migration;
mod weights;
pub mod xcm_config;

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
	(migration::XykRefactorMigration, migration::AssetRegistryMigration),
	// ()
>;

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
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("mangata-parachain"),
	impl_name: create_runtime_str!("mangata-parachain"),
	authoring_version: 15,
	spec_version: 003100,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 003100,
	state_version: 0,
};

use common_runtime::consts::{DAYS, HOURS, MAXIMUM_BLOCK_WEIGHT, MILLIUNIT, UNIT};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
}


// Configure FRAME pallets to include in runtime.
use common_runtime::config as cfg;

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
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The header type.
	type Header = runtime_types::Header;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
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
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = weights::frame_system_weights::ModuleWeight<Runtime>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = cfg::frame_system::SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	/// The maximum number of consumers allowed on a single account.
	type MaxConsumers = cfg::frame_system::MaxConsumers;
}


impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = cfg::pallet_timestamp::MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp_weights::ModuleWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = ParachainStaking;
}


parameter_types! {
	pub const DefaultPayoutLimit: u32 = 3;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = cfg::pallet_treasury::TreasuryPalletId;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::MgxTokenId>;
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
	<Runtime as orml_tokens::Config>::MaxLocks::get() >= <Runtime as pallet_vesting_mangata::Config>::MAX_VESTING_SCHEDULES
);


impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = TokenId;
	type WeightInfo = weights::orml_tokens_weights::ModuleWeight<Runtime>;
	type ExistentialDeposits = cfg::orml_tokens::ExistentialDeposits;
	type MaxLocks = cfg::orml_tokens::MaxLocks;
	type DustRemovalWhitelist = cfg::orml_tokens::DustRemovalWhitelist<cfg::TreasuryAccountIdOf<Runtime>>;
	type CurrencyHooks = ();
	type MaxReserves = ();
	type ReserveIdentifier = cfg::orml_tokens::ReserveIdentifier;
}



impl pallet_xyk::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = Maintenance;
	type ActivationReservesProvider = MultiPurposeLiquidity;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type NativeCurrencyId = tokens::MgxTokenId;
	type TreasuryPalletId = cfg::TreasuryPalletIdOf<Runtime>;
	type BnbTreasurySubAccDerive = cfg::pallet_xyk::BnbTreasurySubAccDerive;
	type PoolFeePercentage = cfg::pallet_xyk::PoolFeePercentage;
	type TreasuryFeePercentage = cfg::pallet_xyk::TreasuryFeePercentage;
	type BuyAndBurnFeePercentage = cfg::pallet_xyk::BuyAndBurnFeePercentage;
	type LiquidityMiningRewards = ProofOfStake;
	type VestingProvider = Vesting;
	type DisallowedPools = Bootstrap;
	type DisabledTokens = (cfg::pallet_xyk::TestTokensFilter, cfg::pallet_xyk::AssetRegisterFilter<Runtime>);
	type AssetMetadataMutation = cfg::pallet_xyk::AssetMetadataMutation<Runtime>;
	type WeightInfo = weights::pallet_xyk_weights::ModuleWeight<Runtime>;
}

impl pallet_proof_of_stake::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ActivationReservesProvider = MultiPurposeLiquidity;
	type NativeCurrencyId = tokens::MgxTokenId;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	//TODO: fix
	type LiquidityMiningIssuanceVault = cfg::pallet_issuance::LiquidityMiningIssuanceVault;
	type RewardsDistributionPeriod = cfg::SessionLenghtOf<Runtime>;
	type WeightInfo = weights::pallet_proof_of_stake_weights::ModuleWeight<Runtime>;
}

impl pallet_bootstrap::BootstrapBenchmarkingConfig for Runtime {}

impl pallet_bootstrap::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = Maintenance;
	type PoolCreateApi = Xyk;
	type DefaultBootstrapPromotedPoolWeight = cfg::pallet_bootstrap::DefaultBootstrapPromotedPoolWeight;
	type BootstrapUpdateBuffer = cfg::pallet_bootstrap::BootstrapUpdateBuffer;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type VestingProvider = Vesting;
	type TreasuryPalletId = cfg::TreasuryPalletIdOf<Runtime>;
	type RewardsApi = ProofOfStake;
	type ClearStorageLimit = cfg::pallet_bootstrap::ClearStorageLimit;
	type WeightInfo = weights::pallet_bootstrap_weights::ModuleWeight<Runtime>;
	type AssetRegistryApi = cfg::pallet_bootstrap::EnableAssetPoolApi<Runtime>;
}


impl TippingCheck for RuntimeCall {
	fn can_be_tipped(&self, ) -> bool {
		match self {
			RuntimeCall::Xyk(pallet_xyk::Call::sell_asset { .. }) |
			RuntimeCall::Xyk(pallet_xyk::Call::buy_asset { .. }) |
			RuntimeCall::Xyk(pallet_xyk::Call::multiswap_sell_asset { .. }) |
			RuntimeCall::Xyk(pallet_xyk::Call::multiswap_buy_asset { .. }) => false,
			_ => true,
		}
	}
}

#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo,
)]
pub struct DisallowedInBatch;
impl Contains<RuntimeCall> for DisallowedInBatch {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::Xyk(pallet_xyk::Call::sell_asset { .. }) |
			RuntimeCall::Xyk(pallet_xyk::Call::buy_asset { .. }) |
			RuntimeCall::Xyk(pallet_xyk::Call::multiswap_sell_asset { .. }) |
			RuntimeCall::Xyk(pallet_xyk::Call::multiswap_buy_asset { .. }) |
			RuntimeCall::Xyk(pallet_xyk::Call::compound_rewards { .. }) |
			RuntimeCall::Xyk(pallet_xyk::Call::provide_liquidity_with_conversion { .. }) => true,
			_ => false,
		}
	}
}

impl pallet_utility_mangata::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type DisallowedInBatch = DisallowedInBatch;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility_mangata_weights::ModuleWeight<Runtime>;
}

use cfg::pallet_transaction_payment_mangata::{
	OnMultiTokenUnbalanced, ToAuthor, ORMLCurrencyAdapterNegativeImbalance,
	LiquidityInfoEnum, FeeHelpers, OnChargeHandler, ThreeCurrencyOnChargeAdapter, TippingCheck,
	TriggerEvent, CallType
};



// TODO: renaming foo causes compiler error
pub struct Foo<T>(PhantomData<T>);
impl<T> TriggerEvent<T::AccountId> for Foo<T>
where
	T: frame_system::Config<AccountId = sp_runtime::AccountId32>
{
	fn trigger(who: T::AccountId, fee: u128, tip: u128){
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
				min_amount_out
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
				max_amount_in
			},
			RuntimeCall::Xyk(pallet_xyk::Call::multiswap_sell_asset {
				swap_token_list,
				sold_asset_amount,
				min_amount_out,
				..
			}) => CallType::MultiSell {
				swap_token_list,
				sold_asset_amount,
				min_amount_out,
			},
			RuntimeCall::Xyk(pallet_xyk::Call::multiswap_buy_asset {
				swap_token_list,
				bought_asset_amount,
				max_amount_in,
				..
			}) => CallType::MultiBuy {
				swap_token_list,
				bought_asset_amount,
				max_amount_in,
			},
			RuntimeCall::FeeLock(pallet_fee_lock::Call::unlock_fee { .. }) => CallType::UnlockFee,
			_ => CallType::Other
		}
	}
}

pub type OnChargeTransactionHandler<T> = ThreeCurrencyOnChargeAdapter<
	orml_tokens::MultiTokenCurrencyAdapter<T>,
	ToAuthor<T>,
	tokens::MgxTokenId,
	tokens::RelayTokenId,
	tokens::TurTokenId,
	frame_support::traits::ConstU128<{common_runtime::constants::fee::RELAY_MGX_SCALE_FACTOR}>,
	frame_support::traits::ConstU128<{common_runtime::constants::fee::TUR_MGR_SCALE_FACTOR}>,
	Foo<T>
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
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = cfg::pallet_transaction_payment_mangata::FeeMultiplierUpdate;
	type OperationalFeeMultiplier = cfg::pallet_transaction_payment_mangata::OperationalFeeMultiplier;
}

parameter_types! {
	pub const MaxCuratedTokens: u32 = 100;
}

impl pallet_fee_lock::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxCuratedTokens = cfg::pallet_fee_lock::MaxCuratedTokens;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type PoolReservesProvider = Xyk;
	type NativeTokenId = tokens::MgxTokenId;
	type WeightInfo = weights::pallet_fee_lock_weights::ModuleWeight<Runtime>;
}


impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = Maintenance;
	type OnSystemEvent = ();
	type SelfParaId = ParachainInfo;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = cfg::cumulus_pallet_parachain_system::ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = cfg::cumulus_pallet_parachain_system::ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::AnyRelayNumber;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}


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
}

impl pallet_sudo_mangata::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
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
	type NativeTokenId = tokens::MgxTokenId;
	type StakingLiquidityTokenValuator = Xyk;
	type Issuance = Issuance;
	type StakingIssuanceVault = cfg::parachain_staking::StakingIssuanceVaultOf<Runtime>;
	type FallbackProvider = Council;
	type WeightInfo = weights::parachain_staking_weights::ModuleWeight<Runtime>;
	type DefaultPayoutLimit = DefaultPayoutLimit;
}

impl parachain_staking::StakingBenchmarkConfig for Runtime {
	#[cfg(feature = "runtime-benchmarks")]
	type RewardsApi = ProofOfStake;
	#[cfg(feature = "runtime-benchmarks")]
	type Xyk = Xyk;
}

impl pallet_xyk::XykBenchmarkingConfig for Runtime {}

// Issuance history must be kept for atleast the staking reward delay
const_assert!(<Runtime as parachain_staking::Config>::RewardPaymentDelay::get() <= <Runtime as pallet_issuance::Config>::HistoryLimit::get() );

impl pallet_issuance::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NativeCurrencyId = tokens::MgxTokenId;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	//TODO
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

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * DOLLARS;
}

impl pallet_vesting_mangata::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type RuntimeEvent = RuntimeEvent;
	type Initialized = Initialized;
	type InitializationPayment = InitializationPayment;
	type MaxInitContributors = MaxInitContributorsBatchSizes;
	type MinimumReward = MinimumReward;
	type RewardAddressRelayVoteThreshold = RelaySignaturesThreshold;
	type NativeTokenId = tokens::MgxTokenId;
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
	type RuntimeEvent = RuntimeEvent;
	type MaxRelocks = cfg::MaxLocksOf<Runtime>;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
	type NativeCurrencyId = tokens::MgxTokenId;
	type VestingProvider = Vesting;
	type Xyk = Xyk;
	type WeightInfo = weights::pallet_multipurpose_liquidity_weights::ModuleWeight<Runtime>;
}

impl orml_unknown_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

impl orml_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
impl EnsureOriginWithArg<RuntimeOrigin, Option<u32>> for AssetAuthority {
	type Success = ();

	fn try_origin(
		origin: RuntimeOrigin,
		_asset_id: &Option<u32>,
	) -> Result<Self::Success, RuntimeOrigin> {
		EnsureRoot::try_origin(origin)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_asset_id: &Option<u32>) -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::root())
	}
}

impl orml_asset_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CustomMetadata = CustomMetadata;
	type AssetId = TokenId;
	type AuthorityOrigin = AssetAuthority;
	type AssetProcessor = SequentialIdWithCreation<Runtime>;
	type Balance = Balance;
	type WeightInfo = weights::orml_asset_registry_weights::ModuleWeight<Runtime>;
}

// Proxy Pallet
/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo,
)]
pub enum ProxyType {
	AutoCompound,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::AutoCompound
	}
}

parameter_types! {
	pub const ProxyDepositBase: Balance = deposit(1, 16);
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 16);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 68);
}

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
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::MgxTokenId>;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = frame_support::traits::ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = frame_support::traits::ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}
parameter_types! {
	// Add item in storage and take 270 bytes, Registry { [], Balance, Info { [], [u8,32] * 7, [u8,20] }}
	pub const BasicDeposit: Balance = deposit(1, 270);
	// No item in storage, extra field takes 66 bytes, ([u8,32], [u8,32])
	pub const FieldDeposit: Balance = deposit(0, 66);
	// Add item in storage, and takes 97 bytes, AccountId + (AccountId, [u8,32])
	pub const SubAccountDeposit: Balance = deposit(1, 97);
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

type IdentityForceOrigin = EnsureRoot<AccountId>;
type IdentityRegistrarOrigin = EnsureRoot<AccountId>;

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::MgxTokenId>;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type ForceOrigin = IdentityForceOrigin;
	type RegistrarOrigin = IdentityRegistrarOrigin;
	type Slashed = Treasury;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

impl pallet_maintenance::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type FoundationAccountsProvider = cfg::pallet_maintenance::FoundationAccountsProvider<Runtime>;
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
		Utility: pallet_utility_mangata::{Pallet, Call, Event} = 4,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 5,
		Maintenance: pallet_maintenance::{Pallet, Call, Storage, Event<T>} = 6,

		// Monetary stuff.
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>} = 10,
		TransactionPayment: pallet_transaction_payment_mangata::{Pallet, Storage, Event<Runtime>} = 11,

		// Xyk stuff
		Xyk: pallet_xyk::{Pallet, Call, Storage, Event<T>, Config<T>} = 13,
		ProofOfStake: pallet_proof_of_stake::{Pallet, Call, Storage, Event<T>} = 14,

		// Fee Locks
		FeeLock: pallet_fee_lock::{Pallet, Storage, Call, Event<T>, Config<T>} = 15,

		// Vesting
		Vesting: pallet_vesting_mangata::{Pallet, Call, Storage, Event<T>} = 17,

		// Crowdloan
		Crowdloan: pallet_crowdloan_rewards::{Pallet, Call, Storage, Event<T>} = 18,

		// Issuance
		Issuance: pallet_issuance::{Pallet, Event<T>, Storage, Call} = 19,

		// MultiPurposeLiquidity
		MultiPurposeLiquidity: pallet_multipurpose_liquidity::{Pallet, Call, Storage, Event<T>} = 20,

		// Bootstrap
		Bootstrap: pallet_bootstrap::{Pallet, Call, Storage, Event<T>} = 21,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Storage} = 30,
		ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 31,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 32,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 33,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 34,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 40,
		PolkadotXcm: pallet_xcm::{Pallet, Storage, Call, Event<T>, Origin, Config} = 41,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 42,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 43,

		// ORML XCM
		XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 50,
		UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 51,
		OrmlXcm: orml_xcm::{Pallet, Call, Event<T>} = 52,
		AssetRegistry: orml_asset_registry::{Pallet, Call, Storage, Event<T>, Config<T>} = 53,

		// Governance stuff
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 60,
		Sudo: pallet_sudo_mangata::{Pallet, Call, Config<T>, Storage, Event<T>} = 61,
		SudoOrigin: pallet_sudo_origin::{Pallet, Call, Event<T>} = 62,
		Council: pallet_collective_mangata::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 63,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 64,
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

impl_runtime_apis! {

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
			System::read_events_no_consensus()
				.any(|record|
					matches!(record.event,
						RuntimeEvent::ParachainSystem( cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionApplied{relay_chain_block_num: _})))
		}

		fn store_seed(seed: sp_core::H256){
			// initialize has been called already so we can fetch number from the storage
			System::set_block_seed(&seed);
		}


		fn get_previous_block_txs() -> Vec<Vec<u8>> {
			System::get_previous_blocks_txs()
		}

		fn pop_txs(count: u64) -> Vec<Vec<u8>>{
			System::pop_txs(count as usize)
		}

		fn create_enqueue_txs_inherent(txs: Vec<<Block as BlockT>::Extrinsic>) -> <Block as BlockT>::Extrinsic{
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

		fn can_enqueue_txs() -> bool{
			System::can_enqueue_txs()
		}

		fn start_prevalidation() {
			System::set_prevalidation()
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

		fn get_max_instant_burn_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> XYKRpcResult<Balance> {
			XYKRpcResult { price: Xyk::get_max_instant_burn_amount(&user, liquidity_asset_id) }
		}

		fn get_max_instant_unreserve_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> XYKRpcResult<Balance> {
			XYKRpcResult { price: Xyk::get_max_instant_unreserve_amount(&user, liquidity_asset_id) }
		}

		fn calculate_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> XYKRpcResult<Balance> {
			match ProofOfStake::calculate_rewards_amount(user, liquidity_asset_id){
				Ok(claimable_rewards) => XYKRpcResult{
					price:claimable_rewards
				},
				Err(e) => {
						log::warn!(target:"xyk", "rpc 'XYK::calculate_rewards_amount' error: '{:?}', returning default value instead", e);
						Default::default()
				},
			}
		}

		fn calculate_balanced_sell_amount(
			total_amount: Balance,
			reserve_amount: Balance,
		) -> XYKRpcResult<Balance> {
			XYKRpcResult {
				price: Xyk::calculate_balanced_sell_amount(total_amount, reserve_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_balanced_sell_amount' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
			}
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
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
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
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
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
	unsafe fn validate_block(arguments: *mut u8, arguments_len: usize) -> u64 {
		let args = cumulus_pallet_parachain_system::validate_block::sp_std::boxed::Box::from_raw(
			cumulus_pallet_parachain_system::validate_block::sp_std::slice::from_raw_parts_mut(
				arguments,
				arguments_len,
			),
		);
		let args = cumulus_pallet_parachain_system::validate_block::bytes::Bytes::from(args);

		// Then we decode from these bytes the `MemoryOptimizedValidationParams`.
		let params = cumulus_pallet_parachain_system::validate_block::decode_from_bytes::<
			cumulus_pallet_parachain_system::validate_block::MemoryOptimizedValidationParams,
		>(args)
		.expect("Invalid arguments to `validate_block`.");

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
