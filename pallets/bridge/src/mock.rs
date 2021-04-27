use super::*;
use crate::{Module, Trait};
use frame_support::{
    impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types, weights::Weight,
};
use orml_tokens::MultiTokenCurrencyAdapter;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    MultiSignature, Perbill,
};
use sp_std::convert::From;

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        frame_system::SystemModule,
        bridge::BridgeModule,
        pallet_verifier::VerifierModule,
        artemis_eth_app::AppETHModule,
        artemis_erc20_app::AppERC20Module,
        artemis_asset::ArtemisAssetModule,
        orml_tokens::TokensModule,
    }
}

impl_outer_origin! {
    pub enum Origin for Test {
    }
}

mod bridge_event {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum Event for Test {
        bridge_event,
        frame_system<T>,
        pallet_verifier,
        artemis_eth_app<T>,
        artemis_erc20_app<T>,
        artemis_asset<T>,
        orml_tokens<T>,
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

impl pallet_verifier::Trait for Test {
    type Event = Event;
}

impl artemis_eth_app::Trait for Test {
    type Event = Event;
}

impl artemis_erc20_app::Trait for Test {
    type Event = Event;
}

impl artemis_asset::Trait for Test {
    type Event = Event;
    type Currency = MultiTokenCurrencyAdapter<Test>;
}

impl orml_tokens::Trait for Test {
    type Event = Event;
    type OnReceived = ();
    type WeightInfo = ();
}

impl Trait for Test {
    type Event = Event;
    type Verifier = pallet_verifier::Module<Test>;
    type AppETH = artemis_eth_app::Module<Test>;
    type AppERC20 = artemis_erc20_app::Module<Test>;
}

pub type BridgeModule = Module<Test>;
pub type SystemModule = frame_system::Module<Test>;
pub type VerifierModule = pallet_verifier::Module<Test>;
pub type AppETHModule = artemis_eth_app::Module<Test>;
pub type AppERC20Module = artemis_erc20_app::Module<Test>;
pub type ArtemisAssetModule = artemis_asset::Module<Test>;
pub type TokensModule = orml_tokens::Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
