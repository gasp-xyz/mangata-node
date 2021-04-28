// Copyright (C) 2020 Mangata team

use crate::{Module, Trait};
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use frame_system as system;
use orml_tokens as assets;
use orml_tokens::MultiTokenCurrencyAdapter;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MinLengthName: usize = 0;
    pub const MaxLengthName: usize = 32;
    pub const MinLengthSymbol: usize = 3;
    pub const MaxLengthSymbol: usize = 8;
    pub const MinLengthDescription: usize = 0;
    pub const MaxLengthDescription: usize = 255;
    pub const MaxDecimals: u32 = 10;
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
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

impl assets::Trait for Test {
    type Event = ();
    type Balance = u128;
    type Amount = i128;
    type CurrencyId = u32;
    type OnReceived = ();
    type WeightInfo = ();
}

impl Trait for Test {
    type Event = ();
    type MinLengthName = MinLengthName;
    type MaxLengthName = MaxLengthName;
    type MinLengthSymbol = MinLengthSymbol;
    type MaxLengthSymbol = MaxLengthSymbol;
    type MinLengthDescription = MinLengthDescription;
    type MaxLengthDescription = MaxLengthDescription;
    type MaxDecimals = MaxDecimals;
    type Currency = MultiTokenCurrencyAdapter<Test>;
}

pub type AssetsInfoModule = Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
