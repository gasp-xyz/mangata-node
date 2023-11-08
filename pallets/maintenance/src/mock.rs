// Copyright (C) 2020 Mangata team

use super::*;
use crate as pallet_maintenance;
use frame_support::{construct_runtime, parameter_types, traits::Everything};
use frame_system as system;
use sp_runtime::BuildStorage;
use sp_std::convert::TryFrom;

pub(crate) type AccountId = u128;

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System: frame_system,
		Maintenance: pallet_maintenance,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = sp_runtime::testing::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type Block = Block;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
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
	system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
