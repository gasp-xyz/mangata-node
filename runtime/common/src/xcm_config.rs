#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use cumulus_primitives_core::ParaId;
pub use frame_support::{
	match_types, parameter_types,
	traits::{Everything, Get, Nothing},
	weights::Weight,
};

use orml_asset_registry::{AssetRegistryTrader, FixedRateAssetRegistryTrader};
use orml_traits::{
	parameter_type_with_key, FixedConversionRateProvider,
	GetByKey, MultiCurrency,
};
use orml_xcm_support::{IsNativeConcrete, MultiCurrencyAdapter};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;

use sp_std::{marker::PhantomData, prelude::*};
use xcm::latest::{prelude::*, Weight as XcmWeight};
use xcm_builder::{
	Account32Hash, AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, FixedRateOfFungible,
	FixedWeightBounds, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
};
use xcm_executor::{traits::DropAssets, Assets};

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
pub type TokensIdOf<T> = <T as orml_tokens::Config>::CurrencyId;

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
	AllowSubscriptionsFrom<Everything>,
);
type AssetRegistryOf<T> = orml_asset_registry::Pallet<T>;

use sp_runtime::traits::Convert;

pub struct TokenToMultiLocation<T>(PhantomData<T>);
impl<T> Convert<T::AssetId, Option<MultiLocation>> for TokenToMultiLocation<T> where
T: parachain_info::Config,
T: orml_asset_registry::Config,
<T as orml_asset_registry::Config>::AssetId: From<u32>
{
	fn convert(id: T::AssetId) -> Option<MultiLocation> {
		// allow relay asset
		if id == crate::tokens::RelayTokenId::get().into() {
			return Some(MultiLocation::parent())
		}
		// allow native asset
		if id == crate::tokens::MgxTokenId::get().into() {
			return Some(MultiLocation::new(
				1,
				X2(Parachain(parachain_info::Pallet::<T>::get().into()), general_key(&id.encode())),
			))
		}
		// allow assets in registry with location set
		AssetRegistryOf::<T>::multilocation(&id).unwrap_or(None)
	}
}

pub struct MultiLocationToToken<T>(PhantomData<T>);
impl<T> Convert<MultiLocation, Option<T::AssetId>> for MultiLocationToToken<T> where
T: parachain_info::Config,
T: orml_asset_registry::Config,
<T as orml_asset_registry::Config>::AssetId: From<u32>
{
	fn convert(location: MultiLocation) -> Option<T::AssetId> {
		// allow relay asset
		if location == MultiLocation::parent() {
			return Some(crate::tokens::RelayTokenId::get().into())
		}

		match location {
			// allow native asset
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(para_id), GeneralKey { length, data }),
			} if ParaId::from(para_id) == parachain_info::Pallet::<T>::get() => {
				let token_id = T::AssetId::decode(&mut &data[..(length as usize)]).ok();
				token_id.filter(|tid| *tid == crate::tokens::MGX_TOKEN_ID.into())
			},
			MultiLocation { parents: 0, interior: X1(GeneralKey { length, data }) } =>
			{
				let token_id = T::AssetId::decode(&mut &data[..(length as usize)]).ok();
				token_id.filter(|tid| *tid == crate::tokens::MGX_TOKEN_ID.into())
			},

			// allow assets in registry with location set
			_ => AssetRegistryOf::<T>::location_to_asset_id(location.clone()),
		}
	}
}

pub struct MultiAssetToToken<T>(PhantomData<T>);
impl<T> Convert<MultiAsset, Option<T::AssetId>> for MultiAssetToToken<T> where
T: parachain_info::Config,
T: orml_asset_registry::Config,
<T as orml_asset_registry::Config>::AssetId: From<u32>
{
	fn convert(asset: MultiAsset) -> Option<T::AssetId> {
		if let MultiAsset { id: Concrete(location), .. } = asset {
			MultiLocationToToken::<T>::convert(location)
		} else {
			None
		}
	}
}


pub type LocalAssetTransactor<Runtime> = MultiCurrencyAdapter<
	orml_tokens::Pallet<Runtime>,
	orml_unknown_tokens::Pallet<Runtime>,
	IsNativeConcrete<TokensIdOf<Runtime>, MultiLocationToToken<Runtime>>,
	AccountIdOf<Runtime>,
	LocationToAccountId<Runtime>,
	TokensIdOf<Runtime>,
	MultiAssetToToken<Runtime>,
	orml_xcm_support::DepositToAlternative<crate::config::TreasuryAccountIdOf<Runtime>, orml_tokens::Pallet<Runtime>, TokensIdOf<Runtime>, AccountIdOf<Runtime>, <Runtime as orml_tokens::Config>::Balance>,
>;


pub struct ToTreasury<T>(PhantomData<T>);

impl<T> TakeRevenue for ToTreasury<T> where
T: orml_tokens::Config<AccountId = sp_runtime::AccountId32, CurrencyId = mangata_types::TokenId>,
T: pallet_treasury::Config,
T: parachain_info::Config,
T: orml_asset_registry::Config<AssetId=mangata_types::TokenId>,
{
	fn take_revenue(revenue: MultiAsset) {
		if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = revenue {
			if let Some(currency_id) = MultiLocationToToken::<T>::convert(location) {
				// Ensure AcalaTreasuryAccount have ed requirement for native asset, but don't need
				// ed requirement for cross-chain asset because it's one of whitelist accounts.
				// Ignore the result.
				let _ = orml_tokens::Pallet::<T>::deposit(currency_id.into(), &crate::config::TreasuryAccountIdOf::<T>::get(), amount.into());
			}
		}
	}
}

use crate::constants::fee::{ksm_per_second, mgx_per_second};

parameter_types! {
	// regular transfer is ~400M weight, xcm transfer weight is ~4*UnitWeightCost
	pub UnitWeightCost: XcmWeight = XcmWeight::from_parts(150_000_000, 0);
	pub const MaxInstructions: u32 = 100;

	pub KsmPerSecond: (AssetId, u128, u128) = (MultiLocation::parent().into(), ksm_per_second(), ksm_per_second());
	pub MgxPerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			0,
			X1(general_key(&crate::tokens::MgxTokenId::get().encode())),
		).into(),
		mgx_per_second(),
		mgx_per_second(),
	);
	pub const MaxAssetsIntoHolding: u32 = 64;
}


pub struct FeePerSecondProvider<T>(PhantomData<T>);
impl<T> FixedConversionRateProvider for FeePerSecondProvider<T> where
T: orml_asset_registry::Config<CustomMetadata = mangata_types::assets::CustomMetadata>,
{
	fn get_fee_per_second(location: &MultiLocation) -> Option<u128> {
		if let Some(asset_id) = AssetRegistryOf::<T>::location_to_asset_id(location) {
			if let Some(xcm_meta) = AssetRegistryOf::<T>::metadata(asset_id.clone())
				.and_then(|metadata| metadata.additional.xcm)
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


pub type Trader<Runtime> = (
	FixedRateOfFungible<MgxPerSecond, ToTreasury<Runtime>>,
	AssetRegistryTrader<FixedRateAssetRegistryTrader<FeePerSecondProvider<Runtime>>, ToTreasury<Runtime>>,
	FixedRateOfFungible<KsmPerSecond, ToTreasury<Runtime>>,
);

pub type Weigher<RuntimeCall> = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter<Runtime> = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<cumulus_pallet_parachain_system::Pallet<Runtime>, pallet_xcm::Pallet<Runtime>, ()>,
	// ..and XCMP to communicate with the sibling chains.
	cumulus_pallet_xcmp_queue::Pallet<Runtime>,
);

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
	C: Convert<MultiLocation, Option<mangata_types::TokenId>>,
	GK: GetByKey<mangata_types::TokenId, mangata_types::Balance>,
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


pub type DropAssetsHandler<T> = MangataDropAssets<
	pallet_xcm::Pallet<T>,
	ToTreasury<T>,
	MultiLocationToToken<T>,
	crate::config::ExistentialDepositsOf<T>,
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin<Runtime, RuntimeOrigin> = (
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


parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		None
	};
}

pub struct AccountIdToMultiLocation;
impl Convert<mangata_types::AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: mangata_types::AccountId) -> MultiLocation {
		X1(AccountId32 { network: None, id: account.into() }).into()
	}
}

parameter_types! {
	pub const BaseXcmWeight: XcmWeight = XcmWeight::from_parts(100_000_000, 0); // TODO: recheck this
	pub const MaxAssetsForTransfer:usize = 2;
}

pub struct SelfLocation<T>(PhantomData<T>);
impl<T> Get<MultiLocation> for SelfLocation<T> where
T: parachain_info::Config,
{
	fn get() -> MultiLocation {
		MultiLocation::new(1, X1(Parachain(parachain_info::Pallet::<T>::get().into())))
	}
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation<RuntimeOrigin> = SignedToAccountId32<RuntimeOrigin, mangata_types::AccountId, RelayNetwork>;
