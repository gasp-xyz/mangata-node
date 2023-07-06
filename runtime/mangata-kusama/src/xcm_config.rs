#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use common_runtime::{config as cfg, tokens};
use cumulus_primitives_core::ParaId;
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
use common_runtime::constants::fee::{ksm_per_second, mgx_per_second};
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

pub fn general_key(key: &[u8]) -> Junction {
	let mut data = [0u8; 32];
	data[..key.len()].copy_from_slice(&key[..]);
	GeneralKey { length: key.len() as u8, data }
}

use common_runtime::xcm_config::{
	RelayChainOrigin, KsmLocation, RelayNetwork, UniversalLocation, LocationToAccountId, RuntimeOriginOf,
	ParentOrParentsExecutivePlurality, Barrier,
};


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
	pub UnitWeightCost: XcmWeight = XcmWeight::from_parts(150_000_000, 0);
	pub const MaxInstructions: u32 = 100;

	pub KsmPerSecond: (AssetId, u128, u128) = (MultiLocation::parent().into(), ksm_per_second(), ksm_per_second());
	pub MgxPerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			0,
			X1(general_key(&tokens::MgxTokenId::get().encode())),
		).into(),
		mgx_per_second(),
		mgx_per_second(),
	);
	pub const MaxAssetsIntoHolding: u32 = 64;
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
	FixedRateOfFungible<MgxPerSecond, ToTreasury>,
	AssetRegistryTrader<FixedRateAssetRegistryTrader<FeePerSecondProvider>, ToTreasury>,
	FixedRateOfFungible<KsmPerSecond, ToTreasury>,
);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = common_runtime::xcm_config::XcmOriginToCallOrigin<Runtime, RuntimeOrigin>;
	type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
	// Teleporting is disabled.
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation<Runtime>;
	type Barrier = Barrier<Runtime>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = Trader;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = MangataDropAssets<
		PolkadotXcm,
		ToTreasury,
		TokenIdConvert<>,
		cfg::ExistentialDepositsOf<Runtime>,
	>;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetLocker = ();
	type AssetExchanger = ();
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = ();
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation<Runtime>;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = orml_tokens::CurrencyAdapter<Runtime, tokens::MgxTokenId>;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId<Runtime>;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId<Runtime>, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin<Runtime>, RuntimeOrigin>,
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
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = Maintenance;
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
	type MaintenanceStatusProvider = Maintenance;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		X1(AccountId32 { network: None, id: account.into() }).into()
	}
}

parameter_types! {
	pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::get().into())));
	pub const BaseXcmWeight: XcmWeight = XcmWeight::from_parts(100_000_000, 0); // TODO: recheck this
	pub const MaxAssetsForTransfer:usize = 2;
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		None
	};
}

impl orml_xtokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = TokenId;
	type CurrencyIdConvert = common_runtime::xcm_config::TokenIdConvert<Runtime>;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type MinXcmFee = ParachainMinFee;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type MultiLocationsFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type UniversalLocation = UniversalLocation<Runtime>;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type ReserveProvider = AbsoluteReserveProvider;
}

pub type LocalAssetTransactor = MultiCurrencyAdapter<
	Tokens,
	UnknownTokens,
	IsNativeConcrete<TokenId, TokenIdConvert>,
	AccountId,
	LocationToAccountId<Runtime>,
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
	fn drop_assets(
		origin: &MultiLocation,
		assets: Assets,
		context: &XcmContext,
	) -> sp_weights::Weight {
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
			X::drop_assets(origin, asset_traps.into(), context);
		}
		XcmWeight::from_parts(0, 0)
	}
}

pub struct TokenIdConvert;
impl Convert<TokenId, Option<MultiLocation>> for TokenIdConvert {
	fn convert(id: TokenId) -> Option<MultiLocation> {
		// allow relay asset
		if id == tokens::RelayTokenId::get() {
			return Some(MultiLocation::parent())
		}
		// allow native asset
		if id == tokens::MgxTokenId::get() {
			return Some(MultiLocation::new(
				1,
				X2(Parachain(ParachainInfo::get().into()), general_key(&id.encode())),
			))
		}
		// allow assets in registry with location set
		AssetRegistryOf::<Runtime>::multilocation(&id).unwrap_or(None)
	}
}

impl Convert<MultiLocation, Option<TokenId>> for TokenIdConvert {
	fn convert(location: MultiLocation) -> Option<TokenId> {
		// allow relay asset
		if location == MultiLocation::parent() {
			return Some(tokens::RelayTokenId::get())
		}

		match location {
			// allow native asset
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(para_id), GeneralKey { length, data }),
			} if ParaId::from(para_id) == ParachainInfo::get() =>
				match TokenId::decode(&mut &data[..(length as usize)]) {
					Ok(tokens::MGX_TOKEN_ID) => Some(tokens::MgxTokenId::get()),
					_ => None,
				},

			// allow native asset
			MultiLocation { parents: 0, interior: X1(GeneralKey { length, data }) } =>
				match TokenId::decode(&mut &data[..(length as usize)]) {
					Ok(tokens::MGX_TOKEN_ID) => Some(tokens::MgxTokenId::get()),
					_ => None,
				},

			// allow assets in registry with location set
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
