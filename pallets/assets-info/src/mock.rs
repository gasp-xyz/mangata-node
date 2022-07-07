// Copyright (C) 2020 Mangata team

use frame_support::{
	construct_runtime, parameter_types,
	traits::{Contains, Everything},
	PalletId,
};
use frame_system as system;
use mangata_primitives::{Amount, Balance, TokenId};
use orml_tokens as assets;
use orml_tokens::MultiTokenCurrencyAdapter;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};

use super::*;

use crate as assets_info;

pub(crate) type AccountId = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Tokens: assets::{Pallet, Storage, Call, Event<T>, Config<T>},
		AssetsInfoModule: assets_info::{Pallet, Call, Config, Storage, Event<T>},
	}
);

// Configure a mock runtime to test the pallet.

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MinLengthName: usize = 1;
	pub const MaxLengthName: usize = 8;
	pub const MinLengthSymbol: usize = 1;
	pub const MaxLengthSymbol: usize = 8;
	pub const MinLengthDescription: usize = 1;
	pub const MaxLengthDescription: usize = 8;
	pub const MaxDecimals: u32 = 10;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
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
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: TokenId| -> Balance {
		match currency_id {
			_ => 0,
		}
	};
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		*a == TreasuryAccount::get()
	}
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub const MaxLocks: u32 = 50;
}

impl assets::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = TokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = DustRemovalWhitelist;
}

impl Config for Test {
	type Event = Event;
	type MinLengthName = MinLengthName;
	type MaxLengthName = MaxLengthName;
	type MinLengthSymbol = MinLengthSymbol;
	type MaxLengthSymbol = MaxLengthSymbol;
	type MinLengthDescription = MinLengthDescription;
	type MaxLengthDescription = MaxLengthDescription;
	type MaxDecimals = MaxDecimals;
	type Currency = MultiTokenCurrencyAdapter<Test>;
	type RelayNativeTokensValueScaleFactor = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
