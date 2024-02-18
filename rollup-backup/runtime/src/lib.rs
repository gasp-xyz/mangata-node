#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, Randomness,
		StorageInfo,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		IdentityFee, Weight,
	},
	StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{ConstFeeMultiplier, CurrencyAdapter, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

/// Import the template pallet.
pub use pallet_template;

pub type TokenId = u32;
pub type Balance = u128;
pub type Amount = i128;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::HeaderVer<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment_mangata::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::HeaderVer<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
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

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time, with maximum proof size.
/// NOTE: reduced by half comparing to origin impl as we want to fill block only up to 50%
/// so there is room for new extrinsics in the next block
const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND, u64::MAX);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights = BlockWeights::builder()
		.base_block(weights::VerBlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = weights::VerExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * consts::MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(consts::MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				consts::MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * consts::MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

pub const RX_TOKEN_ID: TokenId = 0;

parameter_types! {
	pub const RxTokenId: TokenId = RX_TOKEN_ID;
}

// Configure FRAME pallets to include in runtime.
mod runtime_config;
use runtime_config as cfg;
impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// The block type for the runtime.
	type Block = Block;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = weights::frame_system_weights::ModuleWeight<Runtime>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = weights::pallet_timestamp_weights::ModuleWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = ParachainStaking;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = cfg::pallet_treasury::TreasuryPalletId;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, RxTokenId>;
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
	type NativeCurrencyId = tokens::MgxTokenId;
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
	type NativeCurrencyId = tokens::MgxTokenId;
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
	FeeHelpers, OnChargeHandler, ThreeCurrencyOnChargeAdapter, ToAuthor, TriggerEvent,
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
	type WeightToFee = common_runtime::constants::fee::WeightToFee;
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
	type NativeTokenId = tokens::MgxTokenId;
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
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
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
	type NativeTokenId = RxTokenId;
	type StakingLiquidityTokenValuator = Xyk;
	type Issuance = Issuance;
	type StakingIssuanceVault = cfg::parachain_staking::StakingIssuanceVaultOf<Runtime>;
	type FallbackProvider = Council;
	type SequencerStakingProvider = SequencerStaking;
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
	type NativeCurrencyId = tokens::MgxTokenId;
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
	type NativeTokenId = tokens::MgxTokenId;
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
	type NativeCurrencyId = tokens::MgxTokenId;
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
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::MgxTokenId>;
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
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::MgxTokenId>;
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
	pub enum Runtime
	{
		// System support stuff.
		System: frame_system = 0,
		Timestamp: pallet_timestamp = 2,
		ParachainInfo: parachain_info = 3,
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
		[frame_system, SystemBench::<Runtime>]
		[pallet_session, Session]
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
			let key = Authorship::author(&block).expect("Block should have an author aura digest");
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
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
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
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
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
}

parameter_types! {
	pub const OperationalFeeMultiplier: u8 = 5;
	pub const TransactionByteFee: Balance = 5 * consts::MILLIUNIT;
pub ConstFeeMultiplierValue: Multiplier = Multiplier::saturating_from_rational(1, 1);
}

pub type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
pub type FeeMultiplierUpdate = ConstFeeMultiplier<ConstFeeMultiplierValue>;

pub type ORMLCurrencyAdapterNegativeImbalance<Runtime> =
	<::orml_tokens::MultiTokenCurrencyAdapter<Runtime> as MultiTokenCurrency<
		<Runtime as ::frame_system::Config>::AccountId,
	>>::NegativeImbalance;

pub trait OnMultiTokenUnbalanced<
	TokenIdType,
	Imbalance: ::frame_support::traits::TryDrop + MultiTokenImbalanceWithZeroTrait<TokenIdType>,
>
{
	/// Handler for some imbalances. The different imbalances might have different origins or
	/// meanings, dependent on the context. Will default to simply calling on_unbalanced for all
	/// of them. Infallible.
	fn on_unbalanceds<B>(token_id: TokenIdType, amounts: impl Iterator<Item = Imbalance>)
	where
		Imbalance: ::frame_support::traits::Imbalance<B>,
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

pub struct ToAuthor<Runtime>(PhantomData<Runtime>);
impl<T: ::orml_tokens::Config + ::pallet_authorship::Config>
	OnMultiTokenUnbalanced<T::CurrencyId, ORMLCurrencyAdapterNegativeImbalance<T>> for ToAuthor<T>
{
	fn on_nonzero_unbalanced(amount: ORMLCurrencyAdapterNegativeImbalance<T>) {
		if let Some(author) = ::pallet_authorship::Pallet::<T>::author() {
			<::orml_tokens::MultiTokenCurrencyAdapter<T> as MultiTokenCurrency<
				<T as ::frame_system::Config>::AccountId,
			>>::resolve_creating(amount.0, &author, amount);
		}
	}
}

#[derive(Encode, Decode, TypeInfo)]
pub enum LiquidityInfoEnum<C: MultiTokenCurrency<T::AccountId>, T: frame_system::Config> {
	Imbalance((C::CurrencyId, NegativeImbalanceOf<C, T>)),
	FeeLock,
}

pub struct FeeHelpers<T, C, OU, OCA, OFLA>(PhantomData<(T, C, OU, OCA, OFLA)>);
impl<T, C, OU, OCA, OFLA> FeeHelpers<T, C, OU, OCA, OFLA>
where
	T: pallet_transaction_payment_mangata::Config
		+ pallet_xyk::Config<Currency = C>
		+ pallet_fee_lock::Config<Tokens = C>,
	T::LengthToFee: frame_support::weights::WeightToFee<
		Balance = <C as MultiTokenCurrency<T::AccountId>>::Balance,
	>,
	C: MultiTokenCurrency<T::AccountId, Balance = Balance, CurrencyId = TokenId>,
	C::PositiveImbalance: Imbalance<
		<C as MultiTokenCurrency<T::AccountId>>::Balance,
		Opposite = C::NegativeImbalance,
	>,
	C::NegativeImbalance: Imbalance<
		<C as MultiTokenCurrency<T::AccountId>>::Balance,
		Opposite = C::PositiveImbalance,
	>,
	OU: OnMultiTokenUnbalanced<C::CurrencyId, NegativeImbalanceOf<C, T>>,
	NegativeImbalanceOf<C, T>: MultiTokenImbalanceWithZeroTrait<C::CurrencyId>,
	OCA: OnChargeTransaction<
		T,
		LiquidityInfo = Option<LiquidityInfoEnum<C, T>>,
		Balance = <C as MultiTokenCurrency<T::AccountId>>::Balance,
	>,
	OFLA: FeeLockTriggerTrait<
		T::AccountId,
		<C as MultiTokenCurrency<T::AccountId>>::Balance,
		<C as MultiTokenCurrency<T::AccountId>>::CurrencyId,
	>,
	// T: frame_system::Config<RuntimeCall = RuntimeCall>,
	T::AccountId: From<sp_runtime::AccountId32> + Into<sp_runtime::AccountId32>,
	sp_runtime::AccountId32: From<T::AccountId>,
{
	pub fn handle_sell_asset(
		who: &T::AccountId,
		fee_lock_metadata: pallet_fee_lock::FeeLockMetadataInfo<T>,
		sold_asset_id: TokenId,
		sold_asset_amount: Balance,
		bought_asset_id: TokenId,
		min_amount_out: Balance,
	) -> Result<Option<LiquidityInfoEnum<C, T>>, TransactionValidityError> {
		if fee_lock_metadata.is_whitelisted(sold_asset_id) ||
			fee_lock_metadata.is_whitelisted(bought_asset_id)
		{
			let (_, _, _, _, _, bought_asset_amount) =
				<pallet_xyk::Pallet<T> as PreValidateSwaps<
					T::AccountId,
					Balance,
					TokenId,
				>>::pre_validate_sell_asset(
					&who.clone(),
					sold_asset_id,
					bought_asset_id,
					sold_asset_amount,
					min_amount_out,
				)
				.map_err(|_| {
					TransactionValidityError::Invalid(
						InvalidTransaction::SwapPrevalidation.into(),
					)
				})?;
			if Self::is_high_value_swap(
				&fee_lock_metadata,
				sold_asset_id,
				sold_asset_amount,
			) || Self::is_high_value_swap(
				&fee_lock_metadata,
				bought_asset_id,
				bought_asset_amount,
			) {
				let _ = OFLA::unlock_fee(who);
			} else {
				OFLA::process_fee_lock(who).map_err(|_| {
					TransactionValidityError::Invalid(
						InvalidTransaction::ProcessFeeLock.into(),
					)
				})?;
			}
		} else {
			OFLA::process_fee_lock(who).map_err(|_| {
				TransactionValidityError::Invalid(InvalidTransaction::ProcessFeeLock.into())
			})?;
		}
		Ok(Some(LiquidityInfoEnum::FeeLock))
	}

	pub fn is_high_value_swap(
		fee_lock_metadata: &pallet_fee_lock::FeeLockMetadataInfo<T>,
		asset_id: u32,
		asset_amount: u128,
	) -> bool {
		if let (true, Some(valuation)) = (
			fee_lock_metadata.is_whitelisted(asset_id),
			OFLA::get_swap_valuation_for_token(asset_id, asset_amount),
		) {
			valuation >= fee_lock_metadata.swap_value_threshold
		} else {
			false
		}
	}

	pub fn handle_buy_asset(
		who: &T::AccountId,
		fee_lock_metadata: pallet_fee_lock::FeeLockMetadataInfo<T>,
		sold_asset_id: TokenId,
		bought_asset_amount: Balance,
		bought_asset_id: TokenId,
		max_amount_in: Balance,
	) -> Result<Option<LiquidityInfoEnum<C, T>>, TransactionValidityError> {
		if fee_lock_metadata.is_whitelisted(sold_asset_id) ||
			fee_lock_metadata.is_whitelisted(bought_asset_id)
		{
			let (_, _, _, _, _, sold_asset_amount) =
				<pallet_xyk::Pallet<T> as PreValidateSwaps<
					T::AccountId,
					Balance,
					TokenId,
				>>::pre_validate_buy_asset(
					&who.clone(),
					sold_asset_id,
					bought_asset_id,
					bought_asset_amount,
					max_amount_in,
				)
				.map_err(|_| {
					TransactionValidityError::Invalid(
						InvalidTransaction::SwapPrevalidation.into(),
					)
				})?;
			if Self::is_high_value_swap(
				&fee_lock_metadata,
				sold_asset_id,
				sold_asset_amount,
			) || Self::is_high_value_swap(
				&fee_lock_metadata,
				bought_asset_id,
				bought_asset_amount,
			) {
				let _ = OFLA::unlock_fee(who);
			} else {
				OFLA::process_fee_lock(who).map_err(|_| {
					TransactionValidityError::Invalid(
						InvalidTransaction::ProcessFeeLock.into(),
					)
				})?;
			}
		} else {
			// "swap on non-curated token" branch
			OFLA::process_fee_lock(who).map_err(|_| {
				TransactionValidityError::Invalid(InvalidTransaction::ProcessFeeLock.into())
			})?;
		}
		Ok(Some(LiquidityInfoEnum::FeeLock))
	}

	pub fn handle_multiswap_buy_asset(
		who: &T::AccountId,
		_fee_lock_metadata: pallet_fee_lock::FeeLockMetadataInfo<T>,
		swap_token_list: Vec<TokenId>,
		bought_asset_amount: Balance,
		max_amount_in: Balance,
	) -> Result<Option<LiquidityInfoEnum<C, T>>, TransactionValidityError> {
		// ensure swap cannot fail
		// This is to ensure that xyk swap fee is always charged
		// We also ensure that the user has enough funds to transact
		let _ = <pallet_xyk::Pallet<T> as PreValidateSwaps<
			T::AccountId,
			Balance,
			TokenId,
		>>::pre_validate_multiswap_buy_asset(
			&who.clone(),
			swap_token_list,
			bought_asset_amount,
			max_amount_in,
		)
		.map_err(|_| {
			TransactionValidityError::Invalid(InvalidTransaction::SwapPrevalidation.into())
		})?;

		// This is the "low value swap on curated token" branch
		OFLA::process_fee_lock(who).map_err(|_| {
			TransactionValidityError::Invalid(InvalidTransaction::ProcessFeeLock.into())
		})?;
		Ok(Some(LiquidityInfoEnum::FeeLock))
	}

	pub fn handle_multiswap_sell_asset(
		who: &<T>::AccountId,
		_fee_lock_metadata: pallet_fee_lock::FeeLockMetadataInfo<T>,
		swap_token_list: Vec<TokenId>,
		sold_asset_amount: Balance,
		min_amount_out: Balance,
	) -> Result<Option<LiquidityInfoEnum<C, T>>, TransactionValidityError> {
		// ensure swap cannot fail
		// This is to ensure that xyk swap fee is always charged
		// We also ensure that the user has enough funds to transact
		let _ = <pallet_xyk::Pallet<T> as PreValidateSwaps<
			T::AccountId,
			Balance,
			TokenId,
		>>::pre_validate_multiswap_sell_asset(
			&who.clone(),
			swap_token_list.clone(),
			sold_asset_amount,
			min_amount_out,
		)
		.map_err(|_| {
			TransactionValidityError::Invalid(InvalidTransaction::SwapPrevalidation.into())
		})?;

		// This is the "low value swap on curated token" branch
		OFLA::process_fee_lock(who).map_err(|_| {
			TransactionValidityError::Invalid(InvalidTransaction::ProcessFeeLock.into())
		})?;
		Ok(Some(LiquidityInfoEnum::FeeLock))
	}
}

const SINGLE_HOP_MULTISWAP: usize = 2;
#[derive(Encode, Decode, Clone, TypeInfo)]
pub struct OnChargeHandler<C, OU, OCA, OFLA>(PhantomData<(C, OU, OCA, OFLA)>);

/// Default implementation for a Currency and an OnUnbalanced handler.
///
/// The unbalance handler is given 2 unbalanceds in [`OnUnbalanced::on_unbalanceds`]: fee and
/// then tip.
impl<T, C, OU, OCA, OFLA> OnChargeTransaction<T> for OnChargeHandler<C, OU, OCA, OFLA>
where
	T: pallet_transaction_payment_mangata::Config
		+ pallet_xyk::Config<Currency = C>
		+ pallet_fee_lock::Config<Tokens = C>,
	<T as frame_system::Config>::RuntimeCall: Into<crate::CallType>,
	T::LengthToFee: frame_support::weights::WeightToFee<
		Balance = <C as MultiTokenCurrency<T::AccountId>>::Balance,
	>,
	C: MultiTokenCurrency<T::AccountId, Balance = Balance, CurrencyId = TokenId>,
	C::PositiveImbalance: Imbalance<
		<C as MultiTokenCurrency<T::AccountId>>::Balance,
		Opposite = C::NegativeImbalance,
	>,
	C::NegativeImbalance: Imbalance<
		<C as MultiTokenCurrency<T::AccountId>>::Balance,
		Opposite = C::PositiveImbalance,
	>,
	OU: OnMultiTokenUnbalanced<C::CurrencyId, NegativeImbalanceOf<C, T>>,
	NegativeImbalanceOf<C, T>: MultiTokenImbalanceWithZeroTrait<TokenId>,
	OCA: OnChargeTransaction<
		T,
		LiquidityInfo = Option<LiquidityInfoEnum<C, T>>,
		Balance = <C as MultiTokenCurrency<T::AccountId>>::Balance,
	>,
	OFLA: FeeLockTriggerTrait<
		T::AccountId,
		<C as MultiTokenCurrency<T::AccountId>>::Balance,
		<C as MultiTokenCurrency<T::AccountId>>::CurrencyId,
	>,
	// T: frame_system::Config<RuntimeCall = RuntimeCall>,
	T::AccountId: From<sp_runtime::AccountId32> + Into<sp_runtime::AccountId32>,
	Balance: From<<C as MultiTokenCurrency<T::AccountId>>::Balance>,
	sp_runtime::AccountId32: From<T::AccountId>,
{
	type LiquidityInfo = Option<LiquidityInfoEnum<C, T>>;
	type Balance = <C as MultiTokenCurrency<T::AccountId>>::Balance;

	/// Withdraw the predicted fee from the transaction origin.
	///
	/// Note: The `fee` already includes the `tip`.
	fn withdraw_fee(
		who: &T::AccountId,
		call: &T::RuntimeCall,
		info: &DispatchInfoOf<T::RuntimeCall>,
		fee: Self::Balance,
		tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
		let call_type: crate::CallType = (*call).clone().into();

		match call_type {
			crate::CallType::MultiSell { .. } |
			crate::CallType::MultiBuy { .. } |
			crate::CallType::AtomicBuy { .. } |
			crate::CallType::AtomicSell { .. } => {
				ensure!(
					tip.is_zero(),
					TransactionValidityError::Invalid(
						InvalidTransaction::TippingNotAllowedForSwaps.into(),
					)
				);
			},
			_ => {},
		};

		// call.is_unlock_fee();

		// THIS IS NOT PROXY PALLET COMPATIBLE, YET
		// Also ugly implementation to keep it maleable for now
		match (call_type, pallet_fee_lock::FeeLockMetadata::<T>::get()) {
			(
				crate::CallType::AtomicSell {
					sold_asset_id,
					sold_asset_amount,
					bought_asset_id,
					min_amount_out,
				},
				Some(fee_lock_metadata),
			) => FeeHelpers::<T, C, OU, OCA, OFLA>::handle_sell_asset(
				who,
				fee_lock_metadata,
				sold_asset_id,
				sold_asset_amount,
				bought_asset_id,
				min_amount_out,
			),
			(
				crate::CallType::AtomicBuy {
					sold_asset_id,
					bought_asset_amount,
					bought_asset_id,
					max_amount_in,
				},
				Some(fee_lock_metadata),
			) => FeeHelpers::<T, C, OU, OCA, OFLA>::handle_buy_asset(
				who,
				fee_lock_metadata,
				sold_asset_id,
				bought_asset_amount,
				bought_asset_id,
				max_amount_in,
			),
			(
				crate::CallType::MultiBuy {
					swap_token_list,
					bought_asset_amount,
					max_amount_in,
				},
				Some(fee_lock_metadata),
			) if swap_token_list.len() == SINGLE_HOP_MULTISWAP => {
				let sold_asset_id =
					swap_token_list.get(0).ok_or(TransactionValidityError::Invalid(
						InvalidTransaction::SwapPrevalidation.into(),
					))?;
				let bought_asset_id =
					swap_token_list.get(1).ok_or(TransactionValidityError::Invalid(
						InvalidTransaction::SwapPrevalidation.into(),
					))?;
				FeeHelpers::<T, C, OU, OCA, OFLA>::handle_buy_asset(
					who,
					fee_lock_metadata,
					*sold_asset_id,
					bought_asset_amount,
					*bought_asset_id,
					max_amount_in,
				)
			},
			(
				crate::CallType::MultiBuy {
					swap_token_list,
					bought_asset_amount,
					max_amount_in,
				},
				Some(fee_lock_metadata),
			) => FeeHelpers::<T, C, OU, OCA, OFLA>::handle_multiswap_buy_asset(
				who,
				fee_lock_metadata,
				swap_token_list.clone(),
				bought_asset_amount,
				max_amount_in,
			),
			(
				crate::CallType::MultiSell {
					swap_token_list,
					sold_asset_amount,
					min_amount_out,
				},
				Some(fee_lock_metadata),
			) if swap_token_list.len() == SINGLE_HOP_MULTISWAP => {
				let sold_asset_id =
					swap_token_list.get(0).ok_or(TransactionValidityError::Invalid(
						InvalidTransaction::SwapPrevalidation.into(),
					))?;
				let bought_asset_id =
					swap_token_list.get(1).ok_or(TransactionValidityError::Invalid(
						InvalidTransaction::SwapPrevalidation.into(),
					))?;
				FeeHelpers::<T, C, OU, OCA, OFLA>::handle_sell_asset(
					who,
					fee_lock_metadata,
					*sold_asset_id,
					sold_asset_amount,
					*bought_asset_id,
					min_amount_out,
				)
			},
			(
				crate::CallType::MultiSell {
					swap_token_list,
					sold_asset_amount,
					min_amount_out,
				},
				Some(fee_lock_metadata),
			) => FeeHelpers::<T, C, OU, OCA, OFLA>::handle_multiswap_sell_asset(
				who,
				fee_lock_metadata,
				swap_token_list.clone(),
				sold_asset_amount,
				min_amount_out,
			),
			(crate::CallType::UnlockFee, _) => {
				let imb = C::withdraw(
					tokens::MgxTokenId::get().into(),
					who,
					tip,
					WithdrawReasons::TIP,
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| {
					TransactionValidityError::Invalid(InvalidTransaction::Payment.into())
				})?;

				OU::on_unbalanceds(tokens::MgxTokenId::get().into(), Some(imb).into_iter());
				OFLA::can_unlock_fee(who).map_err(|_| {
					TransactionValidityError::Invalid(InvalidTransaction::UnlockFee.into())
				})?;
				Ok(Some(LiquidityInfoEnum::FeeLock))
			},
			_ => OCA::withdraw_fee(who, call, info, fee, tip),
		}
	}

	/// Hand the fee and the tip over to the `[OnUnbalanced]` implementation.
	/// Since the predicted fee might have been too high, parts of the fee may
	/// be refunded.
	///
	/// Note: The `corrected_fee` already includes the `tip`.
	fn correct_and_deposit_fee(
		who: &T::AccountId,
		dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		post_info: &PostDispatchInfoOf<T::RuntimeCall>,
		corrected_fee: Self::Balance,
		tip: Self::Balance,
		already_withdrawn: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError> {
		match already_withdrawn {
			Some(LiquidityInfoEnum::Imbalance(_)) => OCA::correct_and_deposit_fee(
				who,
				dispatch_info,
				post_info,
				corrected_fee,
				tip,
				already_withdrawn,
			),
			Some(LiquidityInfoEnum::FeeLock) => Ok(()),
			None => Ok(()),
		}
	}
}

#[derive(Encode, Decode, Clone, TypeInfo)]
pub struct OneCurrencyOnChargeAdapter<C, OU, T1, TE>(
	PhantomData<(C, OU, T1, TE)>,
);

type NegativeImbalanceOf<C, T> =
	<C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

pub trait TriggerEvent<AccountIdT> {
	fn trigger(who: AccountIdT, fee: u128, tip: u128);
}

/// Default implementation for a Currency and an OnUnbalanced handler.
///
/// The unbalance handler is given 2 unbalanceds in [`OnUnbalanced::on_unbalanceds`]: fee and
/// then tip.
impl<T, C, OU, T1, TE> OnChargeTransaction<T>
for OneCurrencyOnChargeAdapter<C, OU, T1, TE>
where
T: pallet_transaction_payment_mangata::Config,
// TE: TriggerEvent<<T as frame_system::Config>::AccountId>,
TE: TriggerEvent<<T as frame_system::Config>::AccountId>,
// <<T as pallet_transaction_payment_mangata::Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance : From<u128>,
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
OU: OnMultiTokenUnbalanced<C::CurrencyId, NegativeImbalanceOf<C, T>>,
NegativeImbalanceOf<C, T>: MultiTokenImbalanceWithZeroTrait<TokenId>,
<C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance:
scale_info::TypeInfo,
T1: Get<C::CurrencyId>,
// Balance: From<<C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance>,
// Balance: From<TokenId>,
// sp_runtime::AccountId32: From<<T as frame_system::Config>::AccountId>,
{
type LiquidityInfo = Option<LiquidityInfoEnum<C, T>>;
type Balance = <C as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance;
/// Withdraw the predicted fee from the transaction origin.
///
/// Note: The `fee` already includes the `tip`.
fn withdraw_fee(
who: &T::AccountId,
_call: &T::RuntimeCall,
_info: &DispatchInfoOf<T::RuntimeCall>,
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
	T1::get(),
	who,
	fee,
	withdraw_reason,
	ExistenceRequirement::KeepAlive,
) {
	Ok(imbalance) => Ok(Some(LiquidityInfoEnum::Imbalance((T1::get(), imbalance)))),
	// TODO make sure atleast 1 planck KSM is charged
	Err(_) => Err(InvalidTransaction::Payment.into()),
}
}

/// Hand the fee and the tip over to the `[OnUnbalanced]` implementation.
/// Since the predicted fee might have been too high, parts of the fee may
/// be refunded.
///
/// Note: The `corrected_fee` already includes the `tip`.
fn correct_and_deposit_fee(
who: &T::AccountId,
_dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
_post_info: &PostDispatchInfoOf<T::RuntimeCall>,
corrected_fee: Self::Balance,
tip: Self::Balance,
already_withdrawn: Self::LiquidityInfo,
) -> Result<(), TransactionValidityError> {
if let Some(LiquidityInfoEnum::Imbalance((token_id, paid))) = already_withdrawn {
	// Calculate how much refund we should return
	let refund_amount = paid.peek().saturating_sub(corrected_fee);
	// refund to the the account that paid the fees. If this fails, the
	// account might have dropped below the existential balance. In
	// that case we don't refund anything.
	let refund_imbalance = C::deposit_into_existing(token_id, &who, refund_amount)
		.unwrap_or_else(|_| C::PositiveImbalance::from_zero(token_id));
	// merge the imbalance caused by paying the fees and refunding parts of it again.
	let adjusted_paid = paid
		.offset(refund_imbalance)
		.same()
		.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
	// Call someone else to handle the imbalance (fee and tip separately)
	let (tip_imb, fee) = adjusted_paid.split(tip);
	OU::on_unbalanceds(token_id, Some(fee).into_iter().chain(Some(tip_imb)));

	TE::trigger(who.clone(), corrected_fee.into(), tip.into());
}
Ok(())
}
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		let p = base_tx_in_rx();
		let q = Balance::from(VerExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

pub fn base_tx_in_rx() -> Balance {
	UNIT
}

pub enum CallType {
	AtomicSell {
		sold_asset_id: TokenId,
		sold_asset_amount: Balance,
		bought_asset_id: TokenId,
		min_amount_out: Balance,
	},
	AtomicBuy {
		sold_asset_id: TokenId,
		bought_asset_amount: Balance,
		bought_asset_id: TokenId,
		max_amount_in: Balance,
	},
	MultiSell {
		swap_token_list: Vec<TokenId>,
		sold_asset_amount: Balance,
		min_amount_out: Balance,
	},
	MultiBuy {
		swap_token_list: Vec<TokenId>,
		bought_asset_amount: Balance,
		max_amount_in: Balance,
	},
	CompoundRewards,
	ProvideLiquidityWithConversion,
	UnlockFee,
	UtilityInnerCall,
	Other,
}