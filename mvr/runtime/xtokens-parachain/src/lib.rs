#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

mod weights;
pub mod xcm_config;

use codec::{Encode, Decode};
use serde::{Serialize, Deserialize};
use frame_support::{BoundedVec, RuntimeDebug};
use scale_info::TypeInfo;
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use sp_runtime::traits::Convert;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	parameter_types,
	traits::{ConstU32, ConstU64, ConstU8, EitherOfDiverse, Everything, Nothing},
	weights::{
		constants::WEIGHT_REF_TIME_PER_SECOND, ConstantMultiplier, Weight, WeightToFeeCoefficient,
		WeightToFeeCoefficients, WeightToFeePolynomial,
	},
	PalletId,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot,
};
use orml_tokens::CurrencyAdapter;
use orml_traits::parameter_type_with_key;
use pallet_xcm::{EnsureXcm, IsVoiceOfBody};
pub use sp_runtime::{MultiAddress, Perbill, Permill};
use xcm_config::{RelayLocation, XcmConfig, XcmOriginToTransactDispatchOrigin};
use cumulus_primitives_core::{
	Concrete,
	GeneralKey,
	Parent,
	AssetId,
	Fungibility::Fungible,
	Instruction::{BuyExecution, DepositAsset, InitiateReserveWithdraw, WithdrawAsset},
	Junction::{self, Parachain},
	Junctions::{Here, X1, X2},
	MultiAsset,
	MultiAssetFilter::Wild,
	MultiAssets, MultiLocation,
	WeightLimit::Unlimited,
	WildMultiAsset::{self, AllCounted},
	Xcm,
};


#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot imports
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

use weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight};

// XCM Imports
use xcm::latest::prelude::BodyId;
use xcm_executor::XcmExecutor;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

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
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
		// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
		let p = UNIT;
		let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

pub const DOT_MGX_SCALE_FACTOR_UNADJUSTED: u128 = 10_000_000_000_u128; // 10_000 as DOT/MGX, with 6 decimals accounted for (12 - DOT, 18 - MGX)

pub fn base_tx_in_mgx() -> Balance {
	UNIT
}

pub fn mgx_per_second() -> u128 {
	let base_weight = Balance::from(ExtrinsicBaseWeight::get().ref_time());
	let base_per_second = (WEIGHT_REF_TIME_PER_SECOND / base_weight as u64) as u128;
	base_per_second * base_tx_in_mgx()
}

pub fn dot_per_second() -> u128 {
	mgx_per_second() / DOT_MGX_SCALE_FACTOR_UNADJUSTED as u128
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

// Unit = the base number of indivisible units for balances
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
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_div(2),
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
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
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = ();
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
	type SystemWeightInfo = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = ();
	/// The maximum length of a block (in bytes).
	type BlockLength = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = ();
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}



parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
	type PriceForSiblingDelivery = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_token_id: CurrencyId| -> u128 {
		0_u128
	};
}

parameter_types! {
	pub const MaxLocks:u32 = 50;
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, codec::MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	/// Relay chain token.
	R,
	/// Parachain A token.
	A,
	/// Parachain A A1 token.
	A1,
	/// Parachain B token.
	B,
	/// Parachain B B1 token
	B1,
	/// Parachain B B2 token
	B2,
	/// Parachain C token
	C,
	/// Parachain D token
	D,
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 10 * MICROUNIT;
	pub const DOTTokenId: CurrencyId = CurrencyId::R;
}

pub type OrmlCurrencyAdapter = orml_tokens::CurrencyAdapter<Runtime, DOTTokenId>;

pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(id: CurrencyId) -> Option<MultiLocation> {
		match id {
			CurrencyId::R => Some(Parent.into()),
			CurrencyId::A => Some(
				(
					Parent,
					Parachain(1),
					Junction::from(BoundedVec::try_from(b"A".to_vec()).unwrap()),
				)
					.into(),
			),
			CurrencyId::A1 => Some(
				(
					Parent,
					Parachain(1),
					Junction::from(BoundedVec::try_from(b"A1".to_vec()).unwrap()),
				)
					.into(),
			),
			CurrencyId::B => Some(
				(
					Parent,
					Parachain(2),
					Junction::from(BoundedVec::try_from(b"B".to_vec()).unwrap()),
				)
					.into(),
			),
			CurrencyId::B1 => Some(
				(
					Parent,
					Parachain(2),
					Junction::from(BoundedVec::try_from(b"B1".to_vec()).unwrap()),
				)
					.into(),
			),
			CurrencyId::B2 => Some(
				(
					Parent,
					Parachain(2),
					Junction::from(BoundedVec::try_from(b"B2".to_vec()).unwrap()),
				)
					.into(),
			),
			CurrencyId::C => Some(
				(
					Parent,
					Parachain(3),
					Junction::from(BoundedVec::try_from(b"C".to_vec()).unwrap()),
				)
					.into(),
			),
			CurrencyId::D => Some(
				(
					Parent,
					Parachain(4),
					Junction::from(BoundedVec::try_from(b"D".to_vec()).unwrap()),
				)
					.into(),
			),
		}
	}
}
impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(l: MultiLocation) -> Option<CurrencyId> {
		let mut a: Vec<u8> = "A".into();
		a.resize(32, 0);
		let mut a1: Vec<u8> = "A1".into();
		a1.resize(32, 0);
		let mut b: Vec<u8> = "B".into();
		b.resize(32, 0);
		let mut b1: Vec<u8> = "B1".into();
		b1.resize(32, 0);
		let mut b2: Vec<u8> = "B2".into();
		b2.resize(32, 0);
		let mut c: Vec<u8> = "C".into();
		c.resize(32, 0);
		let mut d: Vec<u8> = "D".into();
		d.resize(32, 0);
		if l == MultiLocation::parent() {
			return Some(CurrencyId::R);
		}
		match l {
			MultiLocation { parents, interior } if parents == 1 => match interior {
				X2(Parachain(1), GeneralKey { data, .. }) if data.to_vec() == a => Some(CurrencyId::A),
				X2(Parachain(1), GeneralKey { data, .. }) if data.to_vec() == a1 => Some(CurrencyId::A1),
				X2(Parachain(2), GeneralKey { data, .. }) if data.to_vec() == b => Some(CurrencyId::B),
				X2(Parachain(2), GeneralKey { data, .. }) if data.to_vec() == b1 => Some(CurrencyId::B1),
				X2(Parachain(2), GeneralKey { data, .. }) if data.to_vec() == b2 => Some(CurrencyId::B2),
				X2(Parachain(3), GeneralKey { data, .. }) if data.to_vec() == c => Some(CurrencyId::C),
				X2(Parachain(4), GeneralKey { data, .. }) if data.to_vec() == d => Some(CurrencyId::D),
				_ => None,
			},
			MultiLocation { parents, interior } if parents == 0 => match interior {
				X1(GeneralKey { data, .. }) if data.to_vec() == a => Some(CurrencyId::A),
				X1(GeneralKey { data, .. }) if data.to_vec() == b => Some(CurrencyId::B),
				X1(GeneralKey { data, .. }) if data.to_vec() == a1 => Some(CurrencyId::A1),
				X1(GeneralKey { data, .. }) if data.to_vec() == b1 => Some(CurrencyId::B1),
				X1(GeneralKey { data, .. }) if data.to_vec() == b2 => Some(CurrencyId::B2),
				X1(GeneralKey { data, .. }) if data.to_vec() == c => Some(CurrencyId::C),
				X1(GeneralKey { data, .. }) if data.to_vec() == d => Some(CurrencyId::D),
				_ => None,
			},
			_ => None,
		}
	}
}
impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(a: MultiAsset) -> Option<CurrencyId> {
		if let MultiAsset {
			fun: Fungible(_),
			id: Concrete(id),
		} = a
		{
			Self::convert(id)
		} else {
			Option::None
		}
	}
}


impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = i128;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = Nothing;
	type CurrencyHooks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system = 0,
		ParachainSystem: cumulus_pallet_parachain_system = 1,
		ParachainInfo: parachain_info = 3,

		// Monetary stuff.
		Tokens: orml_tokens = 10,
		XTokens: orml_xtokens = 11,


		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue = 30,
		PolkadotXcm: pallet_xcm = 31,
		CumulusXcm: cumulus_pallet_xcm = 32,
		DmpQueue: cumulus_pallet_dmp_queue = 33,
		OrmlXcm: orml_xcm = 34,
	}
);
