//! Parachain runtime mock.

use frame_support::{
	construct_runtime, parameter_types,
	traits::{EnsureOrigin, Everything, EverythingBut, Nothing},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
};

// use codec::{Encode,Decode};
use sp_runtime::traits::IdentityLookup;
use frame_system::EnsureRoot;
use sp_core::{ConstU32, H256};
use sp_runtime::{
	AccountId32,
};

use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::{
	DmpMessageHandler, Id as ParaId, Sibling, XcmpMessageFormat, XcmpMessageHandler,
};
use xcm::{latest::prelude::*, VersionedXcm};
use xcm_builder::{
	Account32Hash, AccountId32Aliases, AllowUnpaidExecutionFrom, ConvertedConcreteId,
	CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
	IsConcrete, NativeAsset, NoChecking, ParentIsPreset,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation,
};
use xcm_executor::{
	traits::{Convert, JustTry},
	Config, XcmExecutor,
};

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u32;
	type BlockNumber = u32;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub ExistentialDeposit: Balance = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<0>;
	type MaxFreezes = ConstU32<0>;
}

// parameter_types! {
// 	pub const ReservedXcmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
// 	pub const ReservedDmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
// }
//
// parameter_types! {
// 	pub const KsmLocation: MultiLocation = MultiLocation::parent();
// 	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
// 	pub UniversalLocation: InteriorMultiLocation = Parachain(MsgQueue::parachain_id().into()).into();
// }
//
// pub type LocationToAccountId = (
// 	ParentIsPreset<AccountId>,
// 	SiblingParachainConvertsVia<Sibling, AccountId>,
// 	AccountId32Aliases<RelayNetwork, AccountId>,
// 	Account32Hash<(), AccountId>,
// );
//
// pub type XcmOriginToCallOrigin = (
// 	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
// 	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
// 	XcmPassthrough<RuntimeOrigin>,
// );
//
// parameter_types! {
// 	pub const UnitWeightCost: Weight = Weight::from_parts(1, 1);
// 	pub KsmPerSecondPerByte: (AssetId, u128, u128) = (Concrete(Parent.into()), 1, 1);
// 	pub const MaxInstructions: u32 = 100;
// 	pub const MaxAssetsIntoHolding: u32 = 64;
// 	pub ForeignPrefix: MultiLocation = (Parent,).into();
// }
//
// pub type LocalAssetTransactor = (
// 	XcmCurrencyAdapter<Balances, IsConcrete<KsmLocation>, LocationToAccountId, AccountId, ()>,
// );
//
// pub type XcmRouter = (
// 	// Two routers - use UMP to communicate with the relay chain:
// 	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
// 	// ..and XCMP to communicate with the sibling chains.
// 	XcmpQueue,
// );
// pub type Barrier = AllowUnpaidExecutionFrom<Everything>;
//
// parameter_types! {
// 	pub NftCollectionOne: MultiAssetFilter
// 		= Wild(AllOf { fun: WildNonFungible, id: Concrete((Parent, GeneralIndex(1)).into()) });
// 	pub NftCollectionOneForRelay: (MultiAssetFilter, MultiLocation)
// 		= (NftCollectionOne::get(), (Parent,).into());
// }
// pub type TrustedTeleporters = xcm_builder::Case<NftCollectionOneForRelay>;
// pub type TrustedReserves = EverythingBut<xcm_builder::Case<NftCollectionOneForRelay>>;
//
// pub struct XcmConfig;
// impl Config for XcmConfig {
// 	type RuntimeCall = RuntimeCall;
// 	type XcmSender = XcmRouter;
// 	type AssetTransactor = LocalAssetTransactor;
// 	type OriginConverter = XcmOriginToCallOrigin;
// 	type IsReserve = (NativeAsset, TrustedReserves);
// 	type IsTeleporter = TrustedTeleporters;
// 	type UniversalLocation = UniversalLocation;
// 	type Barrier = Barrier;
// 	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
// 	type Trader = FixedRateOfFungible<KsmPerSecondPerByte, ()>;
// 	type ResponseHandler = ();
// 	type AssetTrap = ();
// 	type AssetLocker = ();
// 	type AssetExchanger = ();
// 	type AssetClaims = ();
// 	type SubscriptionService = ();
// 	type PalletInstancesInfo = ();
// 	type FeeManager = ();
// 	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
// 	type MessageExporter = ();
// 	type UniversalAliases = Nothing;
// 	type CallDispatcher = RuntimeCall;
// 	type SafeCallFilter = Everything;
// }
//
// pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;
//
// #[cfg(feature = "runtime-benchmarks")]
// parameter_types! {
// 	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
// }
//
// impl cumulus_pallet_xcmp_queue::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type XcmExecutor = xcm_config::ExecutorWrapper<XcmExecutor<XcmConfig>>;
// 	type ChannelInfo = ParachainSystem;
// 	type VersionWrapper = ();
// 	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
// 	type ControllerOrigin = EnsureRoot<AccountId>;
// 	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
// 	type WeightInfo = ();
// 	type PriceForSiblingDelivery = ();
// }
//
// impl cumulus_pallet_dmp_queue::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type XcmExecutor = XcmExecutor<XcmConfig>;
// 	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
// }
//
// impl pallet_xcm::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
// 	type XcmRouter = XcmRouter;
// 	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
// 	type XcmExecuteFilter = Everything;
// 	type XcmExecutor = XcmExecutor<XcmConfig>;
// 	type XcmTeleportFilter = Nothing;
// 	type XcmReserveTransferFilter = Everything;
// 	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
// 	type UniversalLocation = UniversalLocation;
// 	type RuntimeOrigin = RuntimeOrigin;
// 	type RuntimeCall = RuntimeCall;
// 	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
// 	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
// 	type Currency = Balances;
// 	type CurrencyMatcher = ();
// 	type TrustedLockers = ();
// 	type SovereignAccountOf = LocationToAccountId;
// 	type MaxLockers = ConstU32<8>;
// 	type WeightInfo = pallet_xcm::TestWeightInfo;
// 	#[cfg(feature = "runtime-benchmarks")]
// 	type ReachableDest = ReachableDest;
// 	type AdminOrigin = EnsureRoot<AccountId>;
// }
//
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
);

pub type BlockNumber = u32;
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Header = sp_runtime::generic::Header<BlockNumber, sp_runtime::traits::BlakeTwo256>;
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type Signature = sp_runtime::MultiSignature;
pub type UncheckedExtrinsic =
	sp_runtime::generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		// XcmpQueue: cumulus_pallet_xcmp_queue,
		// PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
		// CumulusXcm: cumulus_pallet_xcm,
		// DmpQueue: cumulus_pallet_dmp_queue,
	}
);
//
//
// pub fn para_ext(parachain_id: u32) -> sp_io::TestExternalities {
// 	use mangata_polkadot_runtime::{Runtime, System};
//
// 	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
//
// 	orml_tokens::GenesisConfig::<Runtime> {
// 		balances: vec![
// 			// (ALICE, 0, 100 * unit(18)),
// 			// (ALICE, 1, 0),
// 			// (ALICE, 2, 0),
// 			// (ALICE, 3, 0),
// 			// (ALICE, mangata_polkadot_runtime::DOTTokenId::get(), INITIAL_BALANCE),
// 		],
// 	}
// 	.assimilate_storage(&mut t)
// 	.unwrap();
//
// 	<parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
// 		&parachain_info::GenesisConfig { parachain_id: parachain_id.into() },
// 		&mut t,
// 	)
// 	.unwrap();
//
// 	<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
// 		&pallet_xcm::GenesisConfig { safe_xcm_version: Some(3) },
// 		&mut t,
// 	)
// 	.unwrap();
//
// 	let mut ext = sp_io::TestExternalities::new(t);
// 	ext.execute_with(|| System::set_block_number(1));
// 	ext
// }
