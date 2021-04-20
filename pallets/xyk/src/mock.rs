// Copyright (C) 2020 Mangata team

use crate::{Module, Trait};
use sp_core::H256;

use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use frame_system as system;
use orml_tokens::MultiCurrencyMintable;
use orml_traits::MultiCurrency;

impl_outer_origin! {
    pub enum Origin for Test {}
}

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
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

pub type Tokens = orml_tokens::Module<Test>;

impl orml_tokens::Trait for Test {
    type Event = ();
    type Balance = u128;
    type Amount = i128;
    type CurrencyId = u32;
    type OnReceived = ();
    type WeightInfo = ();
}

impl Trait for Test {
    type Event = ();
    type MultiCurrency = Tokens;
}

pub type XykStorage = Module<Test>;

type BalanceOf<T> =
    <<T as Trait>::MultiCurrency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;

type CurrencyIdOf<T> = <<T as Trait>::MultiCurrency as MultiCurrency<
    <T as frame_system::Trait>::AccountId,
>>::CurrencyId;

impl<T: Trait> Module<T> {
    pub fn balance(id: CurrencyIdOf<T>, who: T::AccountId) -> BalanceOf<T> {
        T::MultiCurrency::free_balance(id, &who)
    }
    pub fn total_supply(id: CurrencyIdOf<T>) -> BalanceOf<T> {
        T::MultiCurrency::total_issuance(id)
    }
    pub fn create_new_token(who: &T::AccountId, id: BalanceOf<T>) -> CurrencyIdOf<T> {
        T::MultiCurrency::issue(who, id)
    }
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
