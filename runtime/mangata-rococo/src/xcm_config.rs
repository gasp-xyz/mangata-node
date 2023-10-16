#![cfg_attr(not(feature = "std"), no_std)]

use super::{
	AccountId, AllPalletsWithSystem, Balance, Maintenance, ParachainSystem, PolkadotXcm, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeOrigin, TokenId,
};
use common_runtime::tokens;
pub use frame_support::{
	match_types, parameter_types,
	traits::{Everything, Get, Nothing},
	weights::Weight,
};
use frame_system::EnsureRoot;
use orml_traits::location::AbsoluteReserveProvider;
use orml_xcm_support::MultiNativeAsset;
use sp_runtime::traits::ConstU32;
use xcm_builder::EnsureXcmOrigin;
use xcm_executor::XcmExecutor;

#[cfg(feature = "runtime-benchmarks")]
use xcm::prelude::{MultiLocation, Parent};

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = common_runtime::xcm_config::XcmRouter<Runtime>;
	// How to withdraw and deposit an asset.
	type AssetTransactor = common_runtime::xcm_config::LocalAssetTransactor<Runtime>;
	type OriginConverter =
		common_runtime::xcm_config::XcmOriginToCallOrigin<Runtime, RuntimeOrigin>;
	type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
	// Teleporting is disabled.
	type IsTeleporter = ();
	type UniversalLocation = common_runtime::xcm_config::UniversalLocation<Runtime>;
	type Barrier = common_runtime::xcm_config::Barrier<Runtime>;
	type Weigher = common_runtime::xcm_config::Weigher<RuntimeCall>;
	type Trader = common_runtime::xcm_config::Trader<Runtime>;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = common_runtime::xcm_config::DropAssetsHandler<Runtime>;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetLocker = ();
	type AssetExchanger = ();
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = common_runtime::xcm_config::MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = ();
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
	type XcmRouter = common_runtime::xcm_config::XcmRouter<Runtime>;
	type ExecuteXcmOrigin = EnsureXcmOrigin<
		RuntimeOrigin,
		common_runtime::xcm_config::LocalOriginToLocation<RuntimeOrigin>,
	>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = common_runtime::xcm_config::Weigher<RuntimeCall>;
	type UniversalLocation = common_runtime::xcm_config::UniversalLocation<Runtime>;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::MgxTokenId>;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = common_runtime::xcm_config::LocationToAccountId<Runtime>;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = Maintenance;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter =
		common_runtime::xcm_config::XcmOriginToTransactDispatchOrigin<Runtime, RuntimeOrigin>;
	type WeightInfo = ();
	type PriceForSiblingDelivery = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = Maintenance;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

impl orml_xtokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = TokenId;
	type CurrencyIdConvert = common_runtime::xcm_config::TokenToMultiLocation<Runtime>;
	type AccountIdToMultiLocation = common_runtime::xcm_config::AccountIdToMultiLocation;
	type SelfLocation = common_runtime::xcm_config::SelfLocation<Runtime>;
	type MinXcmFee = common_runtime::xcm_config::ParachainMinFee;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type MultiLocationsFilter = Everything;
	type Weigher = common_runtime::xcm_config::Weigher<RuntimeCall>;
	type BaseXcmWeight = common_runtime::xcm_config::BaseXcmWeight;
	type UniversalLocation = common_runtime::xcm_config::UniversalLocation<Runtime>;
	type MaxAssetsForTransfer = common_runtime::xcm_config::MaxAssetsForTransfer;
	type ReserveProvider = AbsoluteReserveProvider;
}
