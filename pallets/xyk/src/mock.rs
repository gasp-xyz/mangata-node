use crate::{Module, Trait};
use sp_core::H256;

use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

use frame_support::traits::ExistenceRequirement;
use frame_support::{
    dispatch::DispatchResult, impl_outer_event, impl_outer_origin, parameter_types, weights::Weight,
};
use frame_system as system;
use mangata_primitives::{Amount, Balance, TokenId};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyAdapter, MultiTokenCurrencyExtended};
use pallet_assets_info as assets_info;

pub const NATIVE_CURRENCY_ID: u32 = 0;
mod xyk {
    pub use crate::Event;
}

impl_outer_origin! {
    pub enum Origin for Test {}
}

impl_outer_event! {
    pub enum TestEvent for Test {
        assets_info,
        frame_system<T>,
        xyk<T>,
        orml_tokens<T>,
    }
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
    type AccountId = u128;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
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

impl orml_tokens::Trait for Test {
    type Event = TestEvent;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = TokenId;
    type OnReceived = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const MinLengthName: usize = 1;
    pub const MaxLengthName: usize = 255;
    pub const MinLengthSymbol: usize = 1;
    pub const MaxLengthSymbol: usize = 255;
    pub const MinLengthDescription: usize = 1;
    pub const MaxLengthDescription: usize = 255;
    pub const MaxDecimals: u32 = 255;
}

impl assets_info::Trait for Test {
    type Event = TestEvent;
    type MinLengthName = MinLengthName;
    type MaxLengthName = MaxLengthName;
    type MinLengthSymbol = MinLengthSymbol;
    type MaxLengthSymbol = MaxLengthSymbol;
    type MinLengthDescription = MinLengthDescription;
    type MaxLengthDescription = MaxLengthDescription;
    type MaxDecimals = MaxDecimals;
    type Currency = orml_tokens::MultiTokenCurrencyAdapter<Test>;
}

parameter_types! {
    pub const NativeCurrencyId: u32 = NATIVE_CURRENCY_ID;
    pub const TreasuryModuleId: sp_runtime::ModuleId = sp_runtime::ModuleId(*b"py/trsry");
    pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
}

impl Trait for Test {
    type Event = TestEvent;
    type Currency = MultiTokenCurrencyAdapter<Test>;
    type NativeCurrencyId = NativeCurrencyId;
    type TreasuryModuleId = TreasuryModuleId;
    type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
}

pub type XykStorage = Module<Test>;
pub type System = system::Module<Test>;

impl<T: Trait> Module<T> {
    pub fn balance(id: TokenId, who: T::AccountId) -> Balance {
        <T as Trait>::Currency::free_balance(id.into(), &who).into()
    }
    pub fn transfer(
        currency_id: TokenId,
        source: T::AccountId,
        dest: T::AccountId,
        value: Balance,
    ) -> DispatchResult {
        <T as Trait>::Currency::transfer(
            currency_id.into(),
            &source,
            &dest,
            value.into(),
            ExistenceRequirement::KeepAlive,
        )
        .into()
    }

    pub fn total_supply(id: TokenId) -> Balance {
        <T as Trait>::Currency::total_issuance(id.into()).into()
    }
    pub fn create_new_token(who: &T::AccountId, amount: Balance) -> TokenId {
        <T as Trait>::Currency::create(who, amount.into()).into()
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
