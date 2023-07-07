#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use common_runtime::{config as cfg, tokens};
use cumulus_primitives_core::ParaId;
use common_runtime::constants::fee::{ksm_per_second, mgx_per_second};
pub use frame_support::{
	match_types, parameter_types,
	traits::{Everything, Get, Nothing},
	weights::Weight,
};
use frame_system::EnsureRoot;
use orml_asset_registry::{AssetRegistryTrader, FixedRateAssetRegistryTrader};
use orml_traits::{
	location::AbsoluteReserveProvider, parameter_type_with_key, FixedConversionRateProvider,
	GetByKey, MultiCurrency,
};
use orml_xcm_support::{IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::traits::ConstU32;
use sp_std::{marker::PhantomData, prelude::*};
use xcm::latest::{prelude::*, Weight as XcmWeight};
use xcm_builder::{
	Account32Hash, AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedRateOfFungible,
	FixedWeightBounds, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
};
use xcm_executor::{traits::DropAssets, Assets, XcmExecutor};

use super::{
	AccountId, AllPalletsWithSystem, AssetMetadataOf, Balance, Convert,
	Maintenance, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeOrigin, TokenId, Tokens, TreasuryAccount, UnknownTokens, XcmpQueue,
};

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = common_runtime::xcm_config::XcmRouter<Runtime>;
	// How to withdraw and deposit an asset.
	type AssetTransactor = common_runtime::xcm_config::LocalAssetTransactor<Runtime>;
	type OriginConverter = common_runtime::xcm_config::XcmOriginToCallOrigin<Runtime, RuntimeOrigin>;
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
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
	type XcmRouter = common_runtime::xcm_config::XcmRouter<Runtime>;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, common_runtime::xcm_config::LocalOriginToLocation<RuntimeOrigin>>;
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
	type ControllerOriginConverter = common_runtime::xcm_config::XcmOriginToTransactDispatchOrigin<Runtime, RuntimeOrigin>;
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

