// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use crate as pallet_encrypted_tx;
use codec::EncodeLike;
use orml_traits::parameter_type_with_key;
use mangata_types::BlockNumber;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		tokens::currency::MultiTokenCurrency, ConstU128, ConstU32, Contains, Everything, Nothing, OnUnbalanced
	},
};
use mangata_types::{Amount, Balance, TokenId};
use sp_runtime::{Perbill, Percent, impl_opaque_keys, traits::ConvertInto, testing::UintAuthorityId};
use sp_runtime::traits::OpaqueKeys;
use sp_std::convert::TryFrom;

pub(crate) type AccountId = u128;

impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy: UintAuthorityId,
	}
}

impl From<UintAuthorityId> for MockSessionKeys{
    fn from(value: UintAuthorityId) -> Self {
        Self {dummy: value}
    }
}

parameter_types!(
	pub const SomeConst: u64 = 10;
	pub const BlockHashCount: u32 = 250;
);

pub struct MockShouldEndSession;
impl pallet_session::ShouldEndSession<u64> for MockShouldEndSession{
    fn should_end_session(now: u64) -> bool {
        todo!()
    }
}

pub struct MockSessionHandler;
impl pallet_session::SessionHandler<u128> for MockSessionHandler {
    const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];

    fn on_genesis_session<T: OpaqueKeys>(validators: &[(u128, T)]) {
        todo!()
    }

    fn on_new_session<T: OpaqueKeys>(
		changed: bool,
		validators: &[(u128, T)],
		queued_validators: &[(u128, T)],
	) {
        todo!()
    }

    fn on_disabled(validator_index: u32) {
        todo!()
    }
}

impl pallet_session::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = MockShouldEndSession;
	type NextSessionRotation = ();
	type SessionManager = ();
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = MockSessionHandler;
	type Keys = MockSessionKeys;
	type WeightInfo = ();
}


parameter_types!(
	pub const MaxAuthorities: u32 = 1_000_000;
);

impl pallet_aura::Config for Test {
	type AuthorityId = UintAuthorityId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}


parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Test {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}


impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = sp_runtime::testing::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::testing::Header;
	type RuntimeEvent = RuntimeEvent;
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

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(_a: &AccountId) -> bool {
		true
	}
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: TokenId| -> Balance {
		0
	};
}

parameter_types!(
	pub const MaxLocks: u32 = 50;
);

impl orml_tokens::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = TokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type OnSlash = ();
	type OnDeposit = ();
	type OnTransfer = ();
	type MaxReserves = ();
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types!(
	pub const DoublyEncryptedCallMaxLength: u32 = 10000;
	pub const Fee: Balance = 10000;
	pub const NativeCurrencyId: u32 = 0;
);

// NOTE: use PoolCreateApi mock for unit testing purposes
impl pallet_encrypted_tx::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	// type Treasury = OnDustRemoval;
	type Call = RuntimeCall;
	type DoublyEncryptedCallMaxLength = DoublyEncryptedCallMaxLength;
	type AuthorityId = UintAuthorityId;
	type NativeCurrencyId = NativeCurrencyId;
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

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;


type NegativeImbalanceOf<T> = <<T as pallet_encrypted_tx::Config>::Tokens as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

pub struct OnDustRemoval;
impl OnUnbalanced<NegativeImbalanceOf<Test>> for OnDustRemoval {
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<Test>) {
		unimplemented!()
	}
}

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
		EncryptedTx: pallet_encrypted_tx::{Pallet, Call, Storage, Event<T>},
		OrmlTokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>},
		Aura: pallet_aura::{Pallet, Storage, Config<T>},
	}
);

pub struct ExtBuilder{
	ext: sp_io::TestExternalities,
}

impl ExtBuilder{
	pub fn new() -> Self {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		Self {ext}
	}

	pub fn create_token(mut self, token_id: TokenId) -> Self{
		self.ext
			.execute_with(|| 
						  {
							  while token_id >= OrmlTokens::next_asset_id(){
								  OrmlTokens::create(
									  RuntimeOrigin::root(),
									  0,
									  0
									  ).unwrap();
							  }
						  }
						 );
		return self
	}

	pub fn mint(mut self, who: AccountId, token_id: TokenId, balance: Balance) -> Self{
		self.ext
			.execute_with(|| 
						  OrmlTokens::mint(
							  RuntimeOrigin::root(),
							  token_id,
							  who,
							  balance,
							  ).unwrap()
		);
		return self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		self.ext
	}
}


