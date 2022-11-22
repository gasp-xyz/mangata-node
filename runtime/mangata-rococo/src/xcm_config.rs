#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use cumulus_primitives_core::ParaId;
use frame_support::{
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
use sp_runtime::{traits::ConstU32, WeakBoundedVec};
use sp_std::{marker::PhantomData, prelude::*};
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedRateOfFungible,
	FixedWeightBounds, LocationInverter, ParentIsPreset, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
};
use xcm_executor::{traits::DropAssets, Assets, XcmExecutor};

use super::{
	constants::{fee::*, parachains},
	AccountId, AssetMetadataOf, Balance, RuntimeOrigin, RuntimeCall, RuntimeEvent, Convert, ExistentialDeposits,
	ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, TokenId, Tokens, TreasuryAccount,
	UnknownTokens, XcmpQueue, MGR_TOKEN_ID, ROC_TOKEN_ID,
};

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

parameter_types! {
	pub RocLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin = (
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
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

match_types! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
	};
}

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
	// ^^^ Parent and its exec plurality get free execution
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
);

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
	fn take_revenue(revenue: MultiAsset) {
		if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = revenue {
			if let Some(currency_id) = TokenIdConvert::convert(location) {
				// Ensure AcalaTreasuryAccount have ed requirement for native asset, but don't need
				// ed requirement for cross-chain asset because it's one of whitelist accounts.
				// Ignore the result.
				let _ = Tokens::deposit(currency_id, &TreasuryAccount::get(), amount);
			}
		}
	}
}

parameter_types! {
	// regular transfer is ~400M weight, xcm transfer weight is ~4*UnitWeightCost
	pub UnitWeightCost: u64 = 150_000_000;
	pub const MaxInstructions: u32 = 100;

	pub RocPerSecond: (AssetId, u128) = (MultiLocation::parent().into(), roc_per_second());
	pub MgrPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			0,
			X1(GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			MGR_TOKEN_ID.encode(),
			None,
		))),
		).into(),
		mgr_per_second()
	);
	pub KarPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::karura::ID), GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			parachains::karura::KAR_KEY.to_vec(),
			None,
		))),
		).into(),
		// KAR:KSM 100:1
		roc_per_second() * 100
	);
	pub KusdPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::karura::ID), GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			parachains::karura::KUSD_KEY.to_vec(),
			None,
		))),
		).into(),
		// KUSD:KSM 50:1
		roc_per_second() * 50
	);
	pub LksmPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::karura::ID), GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			parachains::karura::LKSM_KEY.to_vec(),
			None,
		))),
		).into(),
		// LKSM:KSM 10:1
		roc_per_second() * 10
	);
	pub TurPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X1(Parachain(parachains::turing::ID)),
		).into(),
		// TUR:KSM 100:1 & 10:12 decimals
		roc_per_second()
	);
	pub ImbuPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::imbue::ID), GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			parachains::imbue::IMBU_KEY.to_vec(),
			None,
		))),
		).into(),
		// IMBU:KSM 50:1
		roc_per_second() * 50
	);
	pub PhaPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X1(Parachain(parachains::phala::ID)),
		).into(),
		// PHA:KSM = 400:1
		roc_per_second() * 400
	);
	pub BncPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::bifrost::ID), GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			parachains::bifrost::BNC_KEY.to_vec(),
			None,
		))),
		).into(),
		// BNC:KSM = 80:1
		roc_per_second() * 80
	);
	pub VsksmPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::bifrost::ID), GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			parachains::bifrost::VSKSM_KEY.to_vec(),
			None,
		))),
		).into(),
		// VSKSM:KSM = 1:1
		roc_per_second()
	);
	pub VksmPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::bifrost::ID), GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			parachains::bifrost::VKSM_KEY.to_vec(),
			None,
		))),
		).into(),
		// VKSM:KSM = 1:1
		roc_per_second()
	);

	pub BaseRate: u128 = mgr_per_second();
}

type AssetRegistryOf<T> = orml_asset_registry::Pallet<T>;

pub struct FeePerSecondProvider;
impl FixedConversionRateProvider for FeePerSecondProvider {
	fn get_fee_per_second(location: &MultiLocation) -> Option<u128> {
		if let Some(asset_id) = AssetRegistryOf::<Runtime>::location_to_asset_id(location) {
			if let Some(xcm_meta) = AssetRegistryOf::<Runtime>::metadata(asset_id)
				.and_then(|metadata: AssetMetadataOf| metadata.additional.xcm)
			{
				let fee_per_second: u128 = xcm_meta.fee_per_second;
				log::debug!(
					target: "xcm::weight", "fee_per_second: asset: {:?}, fps:{:?}",
					asset_id, fee_per_second
				);
				return Some(fee_per_second)
			}
		}
		None
	}
}

pub type Trader = (
	FixedRateOfFungible<MgrPerSecond, ToTreasury>,
	AssetRegistryTrader<FixedRateAssetRegistryTrader<FeePerSecondProvider>, ToTreasury>,
	FixedRateOfFungible<RocPerSecond, ToTreasury>,
	FixedRateOfFungible<KarPerSecond, ToTreasury>,
	FixedRateOfFungible<KusdPerSecond, ToTreasury>,
	FixedRateOfFungible<TurPerSecond, ToTreasury>,
	FixedRateOfFungible<BncPerSecond, ToTreasury>,
);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall= RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToCallOrigin;
	type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
	// Teleporting is disabled.
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = Trader;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap =
		MangataDropAssets<PolkadotXcm, ToTreasury, TokenIdConvert, ExistentialDeposits>;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent= RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type RuntimeOrigin= RuntimeOrigin;
	type RuntimeCall= RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent= RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

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
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent= RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent= RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		X1(AccountId32 { network: NetworkId::Any, id: account.into() }).into()
	}
}

parameter_types! {
	pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::get().into())));
	pub const BaseXcmWeight: u64 = 100_000_000; // TODO: recheck this
	pub const MaxAssetsForTransfer:usize = 2;
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		None
	};
}

impl orml_xtokens::Config for Runtime {
	type RuntimeEvent= RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = TokenId;
	type CurrencyIdConvert = TokenIdConvert;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type MinXcmFee = ParachainMinFee;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type MultiLocationsFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type ReserveProvider = AbsoluteReserveProvider;
}

pub type LocalAssetTransactor = MultiCurrencyAdapter<
	Tokens,
	UnknownTokens,
	IsNativeConcrete<TokenId, TokenIdConvert>,
	AccountId,
	LocationToAccountId,
	TokenId,
	TokenIdConvert,
	orml_xcm_support::DepositToAlternative<TreasuryAccount, Tokens, TokenId, AccountId, Balance>,
>;

/// `DropAssets` implementation support asset amount lower thant ED handled by `TakeRevenue`.
///
/// parameters type:
/// - `NC`: native currency_id type.
/// - `NB`: the ExistentialDeposit amount of native currency_id.
/// - `GK`: the ExistentialDeposit amount of tokens.
pub struct MangataDropAssets<X, T, C, GK>(PhantomData<(X, T, C, GK)>);
impl<X, T, C, GK> DropAssets for MangataDropAssets<X, T, C, GK>
where
	X: DropAssets,
	T: TakeRevenue,
	C: Convert<MultiLocation, Option<TokenId>>,
	GK: GetByKey<TokenId, Balance>,
{
	fn drop_assets(origin: &MultiLocation, assets: Assets) -> u64 {
		let multi_assets: Vec<MultiAsset> = assets.into();
		let mut asset_traps: Vec<MultiAsset> = vec![];
		for asset in multi_assets {
			if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = asset.clone() {
				let currency_id = C::convert(location);
				// burn asset(do nothing here) if convert result is None
				if let Some(currency_id) = currency_id {
					let ed = GK::get(&currency_id);
					if amount < ed {
						T::take_revenue(asset);
					} else {
						asset_traps.push(asset);
					}
				}
			}
		}
		if !asset_traps.is_empty() {
			X::drop_assets(origin, asset_traps.into());
		}
		0
	}
}

pub struct TokenIdConvert;
impl Convert<TokenId, Option<MultiLocation>> for TokenIdConvert {
	fn convert(id: TokenId) -> Option<MultiLocation> {
		if id == ROC_TOKEN_ID {
			return Some(MultiLocation::parent())
		}

		match AssetRegistryOf::<Runtime>::multilocation(&id) {
			Ok(Some(multi_location)) => Some(multi_location),
			_ => Some(MultiLocation::new(
				1,
				X2(
					Parachain(ParachainInfo::get().into()),
					GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(id.encode(), None)),
				),
			)),
		}
	}
}
impl Convert<MultiLocation, Option<TokenId>> for TokenIdConvert {
	fn convert(location: MultiLocation) -> Option<TokenId> {
		if location == MultiLocation::parent() {
			return Some(ROC_TOKEN_ID)
		}

		match location {
			MultiLocation { parents: 1, interior: X2(Parachain(para_id), GeneralKey(key)) }
				if ParaId::from(para_id) == ParachainInfo::get() =>
				TokenId::decode(&mut &(*key)[..]).ok(),

			MultiLocation { parents: 0, interior: X1(GeneralKey(key)) } =>
				TokenId::decode(&mut &(*key)[..]).ok(),

			_ => AssetRegistryOf::<Runtime>::location_to_asset_id(location.clone()),
		}
	}
}
impl Convert<MultiAsset, Option<TokenId>> for TokenIdConvert {
	fn convert(asset: MultiAsset) -> Option<TokenId> {
		if let MultiAsset { id: Concrete(location), .. } = asset {
			Self::convert(location)
		} else {
			None
		}
	}
}
