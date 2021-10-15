// Copyright (C) 2020 Mangata team

use super::*;

use sp_core::H256;

use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use frame_system as system;
use mangata_primitives::{Amount, Balance, TokenId};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyAdapter, MultiTokenCurrencyExtended};
use pallet_assets_info as assets_info;
use frame_support::{construct_runtime, parameter_types};
use orml_traits::parameter_type_with_key;
use crate as xyk;

pub const NATIVE_CURRENCY_ID: u32 = 0;


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		Tokens: orml_tokens::{Module, Storage, Call, Event<T>, Config<T>},
        AssetsInfoModule: assets_info::{Module, Call, Config, Storage, Event<T>},
		XykStorage: xyk::{Module, Call, Storage, Event<T>, Config<T>},
	}
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}
impl system::Config for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
	type PalletInfo = PalletInfo;
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: TokenId| -> Balance {
		match currency_id {
			_ => 0,
		}
	};
}

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = TokenId;
    type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
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

impl assets_info::Config for Test {
    type Event = Event;
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

impl Config for Test {
    type Event = Event;
    type Currency = MultiTokenCurrencyAdapter<Test>;
    type NativeCurrencyId = NativeCurrencyId;
    type TreasuryModuleId = TreasuryModuleId;
    type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
}

impl<T: Config> Module<T> {
    pub fn balance(id: TokenId, who: T::AccountId) -> Balance {
        <T as Config>::Currency::free_balance(id.into(), &who).into()
    }
    pub fn total_supply(id: TokenId) -> Balance {
        <T as Config>::Currency::total_issuance(id.into()).into()
    }
    pub fn create_new_token(who: &T::AccountId, amount: Balance) -> TokenId {
        <T as Config>::Currency::create(who, amount.into()).into()
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
