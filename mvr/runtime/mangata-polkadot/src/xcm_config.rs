use super::{
	AccountId, AllPalletsWithSystem, OrmlCurrencyAdapter, ParachainInfo, ParachainSystem,
	PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, XcmpQueue,
};

use core::marker::PhantomData;
use frame_support::{
	log, match_types, parameter_types,
	traits::{ConstU32, Contains, Everything, Nothing, ProcessMessageError},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;

use cumulus_primitives_core::MultiLocation;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowUnpaidExecutionFrom,
	CreateMatcher, CurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
	IsConcrete, MatchXcm, NativeAsset, ParentIsPreset, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{traits::ShouldExecute, XcmExecutor};
// use cumulus_primitives_core::Instruction::*;

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: Option<NetworkId> = None;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = CurrencyAdapter<
	// Use this currency:
	OrmlCurrencyAdapter,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

pub struct AllowSiblingParachainReserveTransferAssetTrap<T>(PhantomData<T>);
impl<T: Contains<MultiLocation>> ShouldExecute
	for AllowSiblingParachainReserveTransferAssetTrap<T>
{
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ProcessMessageError> {
		log::trace!(
		target: "xcm::barriers",
		"AllowSiblingParachainReserveTransferAssetTrap origin: {:?}, instructions: {:?}, max_weight: {:?}, weight_credit: {:?}",
		origin, instructions, max_weight, _weight_credit,
		);
		frame_support::ensure!(T::contains(origin), ProcessMessageError::Unsupported);
		let end = instructions.len().min(4);
		instructions[..end]
			.matcher()
			.match_next_inst(|inst| match inst {
				ReserveAssetDeposited(..) => Ok(()), // to be trapped
				_ => Err(ProcessMessageError::BadFormat),
			})?
			.match_next_inst(|inst| match inst {
				WithdrawAsset(..) => Ok(()),
				_ => Err(ProcessMessageError::BadFormat),
			})?
			.skip_inst_while(|inst| matches!(inst, ClearOrigin))?
			.match_next_inst(|inst| match inst {
				BuyExecution { weight_limit, .. } if *weight_limit == Unlimited => Ok(()),
				_ => Err(ProcessMessageError::Overweight(max_weight)),
			})?;
		Ok(())
	}
}

match_types! {
	pub type RelayNetworkOnly: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here }
	};
}

match_types! {
	pub type SibilingParachain: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: X1(Parachain (_) )}
	};
}

pub type Barrier = (
	TakeWeightCredit,
	AllowUnpaidExecutionFrom<RelayNetworkOnly>,
	AllowSiblingParachainReserveTransferAssetTrap<SibilingParachain>,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
);

use frame_support::weights::constants::{ExtrinsicBaseWeight, WEIGHT_REF_TIME_PER_SECOND};

pub fn dot_per_second() -> u128 {
	let base_weight = crate::Balance::from(ExtrinsicBaseWeight::get().ref_time());
	let base_per_second = (WEIGHT_REF_TIME_PER_SECOND / base_weight as u64) as u128;
	base_per_second * 1_000_000_000_000_u128 / 100 // 0.01 DOT
}

parameter_types! {
	pub DotPerSecondPerByte: (AssetId, u128, u128) = (MultiLocation::parent().into(), dot_per_second(), dot_per_second());
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = NativeAsset;
	type IsTeleporter = (); // Teleporting is disabled.
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = FixedRateOfFungible<DotPerSecondPerByte, ()>;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Nothing;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

pub struct ExecutorWrapper<Executor, FeeAmount>(PhantomData<(Executor, FeeAmount)>);

impl<Executor, RCall, FeeAmount> ExecuteXcm<RCall> for ExecutorWrapper<Executor, FeeAmount>
where
	Executor: ExecuteXcm<RCall>,
	FeeAmount: sp_runtime::traits::Get<u128>,
{
	type Prepared = Executor::Prepared;

	fn prepare(message: Xcm<RCall>) -> core::result::Result<Self::Prepared, Xcm<RCall>> {
		Executor::prepare(message)
	}

	fn execute(
		origin: impl Into<MultiLocation>,
		pre: Self::Prepared,
		hash: XcmHash,
		weight_credit: Weight,
	) -> Outcome {
		Executor::execute(origin, pre, hash, weight_credit)
	}

	fn charge_fees(location: impl Into<MultiLocation>, fees: MultiAssets) -> XcmResult {
		Executor::charge_fees(location, fees)
	}

	fn execute_xcm(
		origin: impl Into<MultiLocation>,
		message: Xcm<RCall>,
		hash: XcmHash,
		weight_limit: Weight,
	) -> Outcome {
		let mut it = message.0.iter();
		let msg = if let (
			Some(ReserveAssetDeposited(deposited_assets)),
			Some(ClearOrigin),
			Some(BuyExecution { fees: _, weight_limit: _ }),
			Some(DepositAsset { assets: _, beneficiary: _ }),
		) = (it.next(), it.next(), it.next(), it.next())
		{
			let amount = FeeAmount::get();
			let location = MultiLocation { parents: 1, interior: Here };
			let fee_asset = MultiAsset { id: AssetId::Concrete(location), fun: Fungible(amount) };
			let withdraw_assets =
				MultiAssets::from_sorted_and_deduplicated_skip_checks(vec![fee_asset.clone()]);

			Xcm(vec![
				ReserveAssetDeposited(deposited_assets.clone()),
				WithdrawAsset(withdraw_assets),
				ClearOrigin,
				BuyExecution { fees: fee_asset, weight_limit: Unlimited },
			])
		} else {
			message
		};

		Executor::execute_xcm(origin, msg, hash, weight_limit)
	}

	fn execute_xcm_in_credit(
		origin: impl Into<MultiLocation>,
		message: Xcm<RCall>,
		hash: XcmHash,
		weight_limit: Weight,
		weight_credit: Weight,
	) -> Outcome {
		Executor::execute_xcm_in_credit(origin, message, hash, weight_limit, weight_credit)
	}
}

/// allow for InitiateReserveWithdraw transfers to relay network
pub struct StrictXcmExecuteFilter;
impl Contains<(MultiLocation, Xcm<RuntimeCall>)> for StrictXcmExecuteFilter {
	fn contains(t: &(MultiLocation, Xcm<RuntimeCall>)) -> bool {
		match t {
			(MultiLocation { parents: 0, interior: X1(AccountId32 { .. }) }, msg)
				if msg.len() == 2 =>
			{
				let mut it = msg.inner().iter();
				if let (Some(WithdrawAsset(..)), Some(InitiateReserveWithdraw { .. })) =
					(it.next(), it.next())
				{
					true
				} else {
					false
				}
			},
			_ => false,
		}
	}
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = StrictXcmExecuteFilter;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Nothing;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = OrmlCurrencyAdapter;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
	type AdminOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
