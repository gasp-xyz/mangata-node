// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// `construct_runtime defines variant types with different sizes
#![allow(clippy::large_enum_variant)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_api::impl_runtime_apis;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{_1, _2},
    OpaqueMetadata,
};
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::traits::{
    BlakeTwo256, Block as BlockT, Convert, IdentifyAccount, IdentityLookup, NumberFor, OpaqueKeys,
    Saturating, Verify,
};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    transaction_validity::{TransactionSource, TransactionValidity},
    AccountId32, ApplyExtrinsicResult, FixedPointNumber, MultiSignature, Perquintill,
};
use sp_std::prelude::*;
//use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime,
    dispatch::Codec,
    parameter_types,
    traits::{KeyOwnerProofSystem, LockIdentifier, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        IdentityFee, Weight,
    },
    Parameter, StorageValue,
};
use frame_system::EnsureRoot;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};
use static_assertions::const_assert;

pub use mangata_primitives::{Amount, Balance, TokenId};
pub use orml_tokens;
use orml_tokens::MultiTokenCurrency;
pub use pallet_assets_info;

use pallet_session::historical as pallet_session_historical;
pub use pallet_staking::StakerStatus;
pub use pallet_xyk;
use xyk_runtime_api::{RpcAmountsResult, RpcResult};

use frame_system::EnsureOneOf;
pub use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_encrypted_tx::{EncryptedTx, ExtrinsicType};

/// Bridge pallets
pub use bridge;
pub use bridged_asset;
pub use erc20_app;
pub use eth_app;
pub use verifier;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Time unit representation
pub type Moment = u64;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

mod weights;

pub const NATIVE_CURRENCY_ID: u32 = 0;

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub babe: Babe,
        pub xxtx: EncryptedTransactions,
    }
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("mangata"),
    impl_name: create_runtime_str!("mangata"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
pub const EPOCH_DURATION_IN_SLOTS: u64 = {
    const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

    (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub const MaximumBlockWeight: Weight = 100 * WEIGHT_PER_SECOND;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    /// Assume 10% of weight for average on_initialize calls.
    pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
        .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Trait for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = IdentityLookup<AccountId>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Maximum weight of each block.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// The weight of the overhead invoked on the block import process, independent of the
    /// extrinsics included in that block.
    type BlockExecutionWeight = BlockExecutionWeight;
    /// The base weight of any extrinsic processed by the runtime, independent of the
    /// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    /// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
    /// idependent of the logic of that extrinsics. (Roughly max block weight - average on
    /// initialize cost).
    type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
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
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl pallet_babe::Trait for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;

    type KeyOwnerProofSystem = ();

    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::IdentificationTuple;

    type HandleEquivocation = ();

    type WeightInfo = ();
}

pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    pub const SessionsPerEra: u32 = 6;
    pub const BondingDuration: pallet_staking::EraIndex = 24 * 28;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const MaxIterations: u32 = 10;
    // 0.05%. The higher the value, the more strict solution acceptance becomes.
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
    pub const MinStakeAmount: Balance = 1;
}

/// Struct that handles the conversion of Balance -> `u64`. This is used for staking's election
/// calculation.
pub struct CurrencyToVoteHandler;

impl CurrencyToVoteHandler {
    fn factor() -> Balance {
        (orml_tokens::MultiTokenCurrencyAdapter::<Runtime>::total_issuance(NATIVE_CURRENCY_ID)
            / u64::max_value() as Balance)
            .max(1)
    }
}

impl Convert<Balance, u64> for CurrencyToVoteHandler {
    fn convert(x: Balance) -> u64 {
        (x / Self::factor()) as u64
    }
}

impl Convert<u128, Balance> for CurrencyToVoteHandler {
    fn convert(x: u128) -> Balance {
        x * Self::factor()
    }
}

impl pallet_staking::Trait for Runtime {
    type NativeCurrencyId = NativeCurrencyId;
    type MinStakeAmount = MinStakeAmount;
    type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
    type Valuations = Xyk;
    type UnixTime = Timestamp;
    type CurrencyToVote = CurrencyToVoteHandler;
    type RewardRemainder = ();
    type Event = Event;
    type Slash = pallet_treasury::MultiOnUnbalancedWrapper<Treasury>;
    type Reward = (); // rewards are minted from the void
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    /// A super-majority of the council can cancel the slash.
    type SlashCancelOrigin = EnsureRoot<AccountId>;
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type MaxIterations = MaxIterations;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type UnsignedPriority = ();
    type WeightInfo = weights::pallet_staking::WeightInfo;
    #[cfg(feature = "runtime-benchmarks")]
    type Xyk = Xyk;
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Trait for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Trait>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = weights::pallet_session::WeightInfo;
}

impl pallet_session::historical::Trait for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

parameter_types! {
    pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) * MaximumBlockWeight::get();
}

impl pallet_offences::Trait for Runtime {
    type Event = Event;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
    type WeightSoftLimit = OffencesWeightSoftLimit;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 5;
}

impl pallet_authorship::Trait for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = Staking;
}

impl pallet_grandpa::Trait for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProofSystem = ();

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type HandleEquivocation = ();

    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const TransactionByteFee: Balance = 1;
    pub const MGATokenID: TokenId = 0;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_transaction_payment::Trait for Runtime {
    type Currency = orml_tokens::CurrencyAdapter<Runtime, MGATokenID>;
    type OnTransactionPayment = Treasury;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate =
        TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

impl pallet_sudo::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

parameter_types! {
    pub const NativeCurrencyId: u32 = NATIVE_CURRENCY_ID;
}

impl pallet_xyk::Trait for Runtime {
    type Event = Event;
    type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
    type NativeCurrencyId = NativeCurrencyId;
    type TreasuryModuleId = TreasuryModuleId;
    type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
}

// Snowfork traits

impl bridge::Trait for Runtime {
    type Event = Event;
    type Verifier = verifier::Module<Runtime>;
    type AppETH = eth_app::Module<Runtime>;
    type AppERC20 = erc20_app::Module<Runtime>;
}

impl verifier::Trait for Runtime {
    type Event = Event;
}

impl bridged_asset::Trait for Runtime {
    type Event = Event;
    type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
}

impl eth_app::Trait for Runtime {
    type Event = Event;
}

impl erc20_app::Trait for Runtime {
    type Event = Event;
}

mod currency {
    use mangata_primitives::Balance;

    pub const MILLICENTS: Balance = 1_000_000_000;
    pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
    pub const DOLLARS: Balance = 100 * CENTS;
}

parameter_types! {
    pub const TreasuryModuleId: sp_runtime::ModuleId = sp_runtime::ModuleId(*b"py/trsry");
    pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
    pub const ProposalBond: sp_runtime::Permill = sp_runtime::Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 1 * currency::DOLLARS;
    pub const SpendPeriod: BlockNumber = 1 * DAYS;
    pub const Burn: sp_runtime::Permill = sp_runtime::Permill::from_percent(50);
    pub const TipCountdown: BlockNumber = 1 * DAYS;
    pub const TipFindersFee: sp_runtime::Percent = sp_runtime::Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1 * currency::DOLLARS;
    pub const DataDepositPerByte: Balance = 1 * currency::CENTS;
    pub const BountyDepositBase: Balance = 1 * currency::DOLLARS;
    pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
    pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 5 * currency::DOLLARS;
}

impl pallet_treasury::Trait for Runtime {
    type ModuleId = TreasuryModuleId;
    type Currency = orml_tokens::CurrencyAdapter<Runtime, MGATokenID>;
    type ApproveOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        EnsureRoot<AccountId>,
        // TODO: do we want to support pallet_collective?
        // pallet_collective::EnsureMembers<_4, AccountId, CouncilCollective>
    >;
    type RejectOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        EnsureRoot<AccountId>,
        // TODO: do we want to support pallet_collective?
        // pallet_collective::EnsureMembers<_2, AccountId, CouncilCollective>
    >;
    type Tippers = pallet_treasury::NoTippers;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type DataDepositPerByte = DataDepositPerByte;
    type Event = Event;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type BountyCuratorDeposit = BountyCuratorDeposit;
    type BountyValueMinimum = BountyValueMinimum;
    type MaximumReasonLength = MaximumReasonLength;
    type BurnDestination = ();
    type WeightInfo = (); // default weights info
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

impl pallet_assets_info::Trait for Runtime {
    type Event = Event;
    type MinLengthName = MinLengthName;
    type MaxLengthName = MaxLengthName;
    type MinLengthSymbol = MinLengthSymbol;
    type MaxLengthSymbol = MaxLengthSymbol;
    type MinLengthDescription = MinLengthDescription;
    type MaxLengthDescription = MaxLengthDescription;
    type MaxDecimals = MaxDecimals;
    type Currency = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
}

impl orml_tokens::Trait for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = TokenId;
    type OnReceived = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 21 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 15;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Trait<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = weights::pallet_collective::WeightInfo;
}

parameter_types! {
    pub const CandidacyBond: Balance = 10 * currency::DOLLARS;
    pub const VotingBond: Balance = 1 * currency::DOLLARS;
    pub const TermDuration: BlockNumber = 120 * DAYS;
    pub const DesiredMembers: u32 = 9;
    pub const DesiredRunnersUp: u32 = 7;
    pub const ElectionsPhragmenModuleId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Trait for Runtime {
    type Event = Event;
    type ModuleId = ElectionsPhragmenModuleId;
    type Currency = orml_tokens::CurrencyAdapter<Runtime, MGATokenID>;
    type ChangeMembers = Council;
    // NOTE: this implies that council's genesis members cannot be set directly and must come from
    // this module.
    type InitializeMembers = Council;
    type CurrencyToVote = CurrencyToVoteHandler;
    type CandidacyBond = CandidacyBond;
    type VotingBond = VotingBond;
    type LoserCandidate = Treasury;
    type BadReport = Treasury;
    type KickedMember = Treasury;
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    type TermDuration = TermDuration;
    type WeightInfo = weights::pallet_elections_phragmen::WeightInfo;
}

impl pallet_sudo_origin::Trait for Runtime {
    type Event = Event;
    type Call = Call;
    type SudoOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
}

parameter_types! {
    pub const EncryptedTxnsFee: Balance = 1 * currency::DOLLARS;
    pub const DoublyEncryptedCallMaxLength: u32 = 4096;
}

impl pallet_encrypted_transactions::Trait for Runtime {
    type Event = Event;
    type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Runtime>;
    /// The identifier type for an authority.
    type AuthorityId = pallet_encrypted_transactions::ecdsa::AuthorityId;
    type Fee = EncryptedTxnsFee;
    type Treasury = pallet_treasury::MultiOnUnbalancedWrapper<Treasury>;
    type Call = Call;
    type DoublyEncryptedCallMaxLength = DoublyEncryptedCallMaxLength;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        Authorship: pallet_authorship::{Module, Call, Storage, Inherent},
        Babe: pallet_babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned},
        Historical: pallet_session_historical::{Module},
        Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
        Offences: pallet_offences::{Module, Call, Storage, Event},
        // Snowfork pallets
        Bridge: bridge::{Module, Call, Config, Storage, Event},
        Verifier: verifier::{Module, Call, Storage, Event, Config<T>},
        BridgedAsset: bridged_asset::{Module, Call, Config<T>, Storage, Event<T>},
        ETH: eth_app::{Module, Call, Storage, Event<T>},
        ERC20: erc20_app::{Module, Call, Storage, Event<T>},
        AssetsInfo: pallet_assets_info::{Module, Call, Config, Storage, Event},
        Tokens: orml_tokens::{Module, Storage, Call, Event<T>, Config<T>},
        Xyk: pallet_xyk::{Module, Call, Storage, Event<T>, Config<T>},
        Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>, ValidateUnsigned},
        Treasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},
        Council: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        Elections: pallet_elections_phragmen::{Module, Call, Storage, Event<T>, Config<T>},
        SudoOrigin: pallet_sudo_origin::{Module, Call, Event},
        EncryptedTransactions: pallet_encrypted_transactions::{Module, Call, Storage, Config<T>, Event<T>},
    }
);

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
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
    AllModules,
>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            let authors :Vec<_> = block.extrinsics().iter().map(
                |tx| tx.clone().signature.map(|info| info.0) ).collect();

            Executive::execute_block(block, authors)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
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

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl extrinsic_info_runtime_api::runtime_api::ExtrinsicInfoRuntimeApi<Block> for Runtime {
        fn get_info(
            tx: <Block as BlockT>::Extrinsic,
        ) -> Option<extrinsic_info_runtime_api::ExtrinsicInfo> {
            tx.signature.clone().map(|sig|
                {
                    let nonce: frame_system::CheckNonce<_> = sig.2.4;
                    extrinsic_info_runtime_api::ExtrinsicInfo{
                        who: sig.0,
                        nonce: nonce.0,
                    }
                }
            )
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::SlotNumber {
            Babe::current_epoch_start()
        }

        fn generate_key_ownership_proof(
            _slot_number: sp_consensus_babe::SlotNumber,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
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

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
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
    }

    impl xyk_runtime_api::XykApi<Block, Balance, TokenId> for Runtime {
        fn calculate_sell_price(
            input_reserve: Balance,
            output_reserve: Balance,
            sell_amount: Balance
        ) -> RpcResult<Balance> {
            RpcResult {
                price: Xyk::calculate_sell_price(input_reserve, output_reserve, sell_amount).unwrap_or_default()
            }
        }

        fn calculate_buy_price(
            input_reserve: Balance,
            output_reserve: Balance,
            buy_amount: Balance
        ) -> RpcResult<Balance> {
            RpcResult {
                price: Xyk::calculate_buy_price(input_reserve, output_reserve, buy_amount).unwrap_or_default()
            }
        }

        fn calculate_sell_price_id(
            sold_token_id: TokenId,
            bought_token_id: TokenId,
            sell_amount: Balance
        ) -> RpcResult<Balance> {
            RpcResult {
                price: Xyk::calculate_sell_price_id(sold_token_id, bought_token_id, sell_amount).unwrap_or_default()
            }
        }

        fn calculate_buy_price_id(
            sold_token_id: TokenId,
            bought_token_id: TokenId,
            buy_amount: Balance
        ) -> RpcResult<Balance> {
            RpcResult {
                price: Xyk::calculate_buy_price_id(sold_token_id, bought_token_id, buy_amount).unwrap_or_default()
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
                Err(_) => RpcAmountsResult{
                    first_asset_amount: 0u32.into(),
                    second_asset_amount: 0u32.into()
                },
            }
        }
        fn calculate_rewards_amount(
            user: AccountIdOf<T>,
            liquidity_asset_id: TokenId,
            block_number: u32
        ) -> RpcRewardsResult<Balance> {
            match Xyk::calculate_rewards_amount(user, liquidity_asset_id, block_number){
                Ok((total_rewards, already_claimed)) => RpcRewardsResult{
                                                                    total_rewards,
                                                                    already_claimed
                                                                },
                Err(_) => RpcRewardsResult{
                    total_rewards: 0u32.into(),
                    already_claimed: 0u32.into()
                },
            }
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Module as SystemBench;

            impl frame_system_benchmarking::Trait for Runtime {}

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

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, bridge, Bridge);
            add_benchmark!(params, batches, pallet_staking, Staking);
            add_benchmark!(params, batches, pallet_treasury, Treasury);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }

    impl sp_encrypted_tx::EncryptedTxApi<Block> for Runtime {

        fn create_submit_singly_encrypted_transaction(identifier: <Block as BlockT>::Hash, singly_encrypted_call: Vec<u8>) -> <Block as BlockT>::Extrinsic{
            UncheckedExtrinsic::new_unsigned(
                    Call::EncryptedTransactions(pallet_encrypted_transactions::Call::submit_singly_encrypted_transaction(identifier, singly_encrypted_call)))
        }

        fn create_submit_decrypted_transaction(identifier: <Block as BlockT>::Hash, decrypted_call: Vec<u8>, _weight: Weight) -> <Block as BlockT>::Extrinsic{
            UncheckedExtrinsic::new_unsigned(
                    Call::EncryptedTransactions(pallet_encrypted_transactions::Call::submit_decrypted_transaction(identifier, decrypted_call, Default::default())))
        }

        fn get_type(extrinsic: <Block as BlockT>::Extrinsic) -> ExtrinsicType<<Block as BlockT>::Hash>{
            match extrinsic.function{
                Call::EncryptedTransactions(pallet_encrypted_transactions::Call::submit_singly_encrypted_transaction(identifier, singly_encrypted_call)) => {
                    ExtrinsicType::<<Block as BlockT>::Hash>::SinglyEncryptedTx{identifier, singly_encrypted_call}
                },
                Call::EncryptedTransactions(pallet_encrypted_transactions::Call::submit_decrypted_transaction(identifier, decrypted_call, _)) => {
                    ExtrinsicType::<<Block as BlockT>::Hash>::DecryptedTx{identifier, decrypted_call}
                },
                _ => { ExtrinsicType::<<Block as BlockT>::Hash>::Other }
            }
        }

        fn get_double_encrypted_transactions(block_builder_id: &AccountId32) -> Vec<EncryptedTx<<Block as BlockT>::Hash>>{
            let transactions = EncryptedTransactions::doubly_encrypted_queue(block_builder_id);
            transactions.into_iter().filter_map(|tx_hash|
                EncryptedTransactions::txn_registry(tx_hash)
                .map(|tx_details|
                    EncryptedTx{
                        tx_id: tx_hash,
                        data: tx_details.doubly_encrypted_call,
                    }
                )
            ).collect()
        }

        fn get_singly_encrypted_transactions(block_builder_id: &AccountId32) -> Vec<EncryptedTx<<Block as BlockT>::Hash>>{
            let transactions = EncryptedTransactions::singly_encrypted_queue(block_builder_id);
            transactions.into_iter().filter_map(|tx_hash|
                match EncryptedTransactions::txn_registry(tx_hash){
                    Some(pallet_encrypted_transactions::TxnRegistryDetails{singly_encrypted_call: Some(call), ..}) =>
                        Some(EncryptedTx{
                            tx_id: tx_hash,
                            data: call
                        }),
                    _ => None
                }
            ).collect()
        }

        // fetches address assigned to authority id
        fn get_account_id(collator_id: u32) -> Option<AccountId32>{
            Session::validators().get(collator_id as usize).map(|e| e.clone())
        }

        // use autority id to identify public key (from encrypted transactions apllet)
        fn get_authority_public_key(authority_id: &AccountId32) -> Option<sp_core::ecdsa::Public>{
            EncryptedTransactions::keys().get(authority_id).map(
                |key| {
                    let mut key_array = [0u8;33];
                    key_array.copy_from_slice(key.as_ref());
                    sp_core::ecdsa::Public::from_raw(key_array)
                }
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use codec::{Decode, Encode};

    #[test]
    fn test_serialize_deserialize_call() {
        let call: Call = Call::Xyk(pallet_xyk::Call::sell_asset(0, 0, 0, 0));
        let call_bytes = call.encode();
        let deserialized_call: Call = Decode::decode(&mut &call_bytes[..]).unwrap();
        assert_eq!(call, deserialized_call);
    }
}
