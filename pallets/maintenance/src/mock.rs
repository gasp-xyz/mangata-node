// Copyright (C) 2020 Mangata team

use super::*;
use sp_std::convert::TryFrom;

use sp_core::H256;

use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};

use crate as pallet_maintenance;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, Contains, Everything},
	PalletId,
};
use frame_system as system;
use mangata_types::Amount;
use orml_traits::parameter_type_with_key;

pub const NATIVE_CURRENCY_ID: u32 = 0;

pub(crate) type AccountId = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Maintenance: pallet_maintenance::{Pallet, Storage, Call, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl system::Config for Test {
	type BaseCallFilter = Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
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
	type MaxConsumers = ConstU32<16>;
}

pub struct FoundationAccountsProvider<T: frame_system::Config>(PhantomData<T>);

impl<T: frame_system::Config> Get<Vec<T::AccountId>> for FoundationAccountsProvider<T>
where
	<T as frame_system::Config>::AccountId: From<u128>,
{
	fn get() -> Vec<T::AccountId> {
		vec![999u128.into()]
	}
}

impl pallet_maintenance::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type FoundationAccountsProvider = FoundationAccountsProvider<Self>;
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
