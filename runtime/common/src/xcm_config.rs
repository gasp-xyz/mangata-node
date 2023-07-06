#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
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

// use super::{
// 	constants::fee::*, AccountId, AllPalletsWithSystem, AssetMetadataOf, Balance, Convert,
// 	ExistentialDeposits, Maintenance, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime,
// 	RuntimeCall, RuntimeEvent, RuntimeOrigin, TokenId, Tokens, TreasuryAccount, UnknownTokens,
// 	XcmpQueue, MGR_TOKEN_ID, ROC_TOKEN_ID,
// };

pub fn general_key(key: &[u8]) -> Junction {
	let mut data = [0u8; 32];
	data[..key.len()].copy_from_slice(&key[..]);
	GeneralKey { length: key.len() as u8, data }
}

parameter_types! {
	pub KsmLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	// pub RelayChainOrigin<T>: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	// pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
}

pub struct RelayChainOrigin<T>(PhantomData<T>);
impl<T> Get<T::RuntimeOrigin> for RelayChainOrigin<T> where
T: frame_system::Config,
<T as frame_system::Config>::RuntimeOrigin: From<cumulus_pallet_xcm::Origin>
{
	fn get() -> T::RuntimeOrigin {
		cumulus_pallet_xcm::Origin::Relay.into()
	}
}

pub struct UniversalLocation<T>(PhantomData<T>);
impl<T> Get<InteriorMultiLocation> for UniversalLocation<T> where
T: parachain_info::Config,
{
	fn get() -> InteriorMultiLocation {
		X2(GlobalConsensus(RelayNetwork::get()), Parachain(parachain_info::Pallet::<T>::parachain_id().into()))
	}
}

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type RuntimeOriginOf<T> = <T as frame_system::Config>::AccountId;

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId<T> = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountIdOf<T>>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountIdOf<T>>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountIdOf<T>>,
	// Create hash of `AccountId32` used for proxy accounts
	Account32Hash<RelayNetwork, AccountIdOf<T>>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin<T, RuntimeOrigin> = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId<T>, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin<T>, RuntimeOrigin>,
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

pub type Barrier<T> = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
	// ^^^ Parent and its exec plurality get free execution
	// Expected responses are OK.
	AllowKnownQueryResponses<pallet_xcm::Pallet<T>>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
);
type AssetRegistryOf<T> = orml_asset_registry::Pallet<T>;

use sp_runtime::traits::Convert;

pub struct TokenIdConvert<T>(PhantomData<T>);
impl<T> Convert<T::AssetId, Option<MultiLocation>> for TokenIdConvert<T> where
T: parachain_info::Config,
T: orml_asset_registry::Config,
<T as orml_asset_registry::Config>::AssetId: From<u32>
{
	fn convert(id: T::AssetId) -> Option<MultiLocation> {
		None
	}
}

impl<T> Convert<MultiLocation, Option<T::AssetId>> for TokenIdConvert<T> where
T: parachain_info::Config,
T: orml_asset_registry::Config,
<T as orml_asset_registry::Config>::AssetId: From<u32>{
	fn convert(location: MultiLocation) -> Option<TokenId> {
		None
	}
}

// impl Convert<MultiAsset, Option<TokenId>> for TokenIdConvert {
// 	fn convert(asset: MultiAsset) -> Option<TokenId> {
// 		if let MultiAsset { id: Concrete(location), .. } = asset {
// 			Self::convert(location)
// 		} else {
// 			None
// 		}
// 	}
// }
// // pub struct ToTreasury;
// // impl TakeRevenue for ToTreasury {
// // 	fn take_revenue(revenue: MultiAsset) {
// // 		if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = revenue {
// // 			if let Some(currency_id) = TokenIdConvert::convert(location) {
// // 				// Ensure AcalaTreasuryAccount have ed requirement for native asset, but don't need
// // 				// ed requirement for cross-chain asset because it's one of whitelist accounts.
// // 				// Ignore the result.
// // 				let _ = Tokens::deposit(currency_id, &TreasuryAccount::get(), amount);
// // 			}
// // 		}
// // 	}
// // }
