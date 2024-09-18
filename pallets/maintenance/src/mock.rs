// Copyright (C) 2020 Mangata team

use super::*;
use crate as pallet_maintenance;
use frame_support::{construct_runtime, derive_impl, parameter_types, traits::Everything};
use frame_system as system;
use sp_runtime::BuildStorage;
use sp_std::convert::TryFrom;

pub(crate) type AccountId = u64;

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System: frame_system,
		Maintenance: pallet_maintenance,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

pub struct FoundationAccountsProvider<T: frame_system::Config>(PhantomData<T>);

impl<T: frame_system::Config> Get<Vec<T::AccountId>> for FoundationAccountsProvider<T>
where
	<T as frame_system::Config>::AccountId: From<u64>,
{
	fn get() -> Vec<T::AccountId> {
		vec![999u64.into()]
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
