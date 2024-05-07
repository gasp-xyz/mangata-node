// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Test utilities
use crate::{self as pallet_crowdloan_rewards, Config};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Contains, Nothing, OnFinalize, OnInitialize, WithdrawReasons},
	PalletId,
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot};
use orml_traits::parameter_type_with_key;
use sp_core::{ecdsa, Pair, H256};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};

use sp_application_crypto::{ecdsa::Public, RuntimePublic};
use sp_runtime::{
	account::AccountId20,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
	BuildStorage, Perbill,
};
use sp_std::{
	convert::{From, TryInto},
	fmt::Write,
};

pub const MGA_TOKEN_ID: TokenId = 0;
pub(crate) type AccountId = u64;
pub(crate) type Balance = u128;
pub(crate) type TokenId = u32;
pub(crate) type Amount = i128;

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>},
		Crowdloan: pallet_crowdloan_rewards::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Storage, Event},
		Vesting: pallet_vesting_mangata::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Nothing;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type OnSetCode = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type MaxConsumers = sp_core::ConstU32<16>;
	type Nonce = u64;
	type Block = Block;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: TokenId| -> Balance {
		0
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
	pub const MgaTokenId: TokenId = MGA_TOKEN_ID;
}

impl orml_tokens::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = TokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type CurrencyHooks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 0;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting_mangata::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting_mangata::weights::SubstrateWeight<Test>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
	pub const TestMaxInitContributors: u32 = 8;
	pub const TestMinimumReward: u128 = 0;
	pub const TestInitialized: bool = false;
	pub const TestInitializationPayment: Perbill = Perbill::from_percent(20);
	pub const TestRewardAddressRelayVoteThreshold: Perbill = Perbill::from_percent(50);
	pub const TestSigantureNetworkIdentifier: &'static [u8] = b"test-";
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Initialized = TestInitialized;
	type InitializationPayment = TestInitializationPayment;
	type MaxInitContributors = TestMaxInitContributors;
	type MinimumReward = TestMinimumReward;
	type NativeTokenId = MgaTokenId;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type RelayChainAccountId = [u8; 20];
	type RewardAddressRelayVoteThreshold = TestRewardAddressRelayVoteThreshold;
	// The origin that is allowed to associate the reward
	type RewardAddressAssociateOrigin = EnsureRoot<Self::AccountId>;
	// The origin that is allowed to change the reward
	type RewardAddressChangeOrigin = EnsureRoot<Self::AccountId>;
	type SignatureNetworkIdentifier = TestSigantureNetworkIdentifier;
	type VestingBlockNumber = BlockNumberFor<Test>;
	type VestingBlockProvider = System;
	type VestingProvider = Vesting;
	type WeightInfo = ();
}

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
	type PalletsOrigin = OriginCaller;
}

fn genesis() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	orml_tokens::GenesisConfig::<Test> {
		tokens_endowment: vec![(0u64, 0u32, 2_000_000_000)],
		created_tokens_for_staking: Default::default(),
	}
	.assimilate_storage(&mut storage)
	.expect("Tokens storage can be assimilated");

	let mut ext = sp_io::TestExternalities::from(storage);
	ext.register_extension(KeystoreExt::new(MemoryKeystore::new()));
	ext.execute_with(|| {
		System::set_block_number(1);
		Crowdloan::set_crowdloan_allocation(RuntimeOrigin::root(), 2500u128).unwrap();
	});
	ext
}

pub type UtilityCall = pallet_utility::Call<Test>;

pub(crate) fn get_ecdsa_pairs(num: u32) -> Vec<ecdsa::Public> {
	let seed: u128 = 12345678901234567890123456789012;
	let mut pairs = Vec::new();
	for i in 0..num {
		pairs.push({
			let mut buffer = String::new();
			let _ = write!(&mut buffer, "//{}", seed + i as u128);
			Public::generate_pair(sp_core::testing::ECDSA, Some(buffer.into_bytes()))
		})
	}
	pairs
}

pub(crate) fn empty() -> sp_io::TestExternalities {
	genesis()
}

pub(crate) fn events() -> Vec<super::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Crowdloan(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>()
}

pub(crate) fn batch_events() -> Vec<pallet_utility::Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Utility(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>()
}

// pub(crate) fn roll_to(n: u64) {
// 	let mut current_block_number = System::block_number();
// 	while current_block_number < n {
// 		Crowdloan::on_finalize(System::block_number());
// 		Tokens::on_finalize(System::block_number());
// 		System::on_finalize(System::block_number());
// 		System::set_block_number(current_block_number);
// 		current_block_number = current_block_number.saturating_add(1);
// 		System::on_initialize(System::block_number());
// 		Tokens::on_initialize(System::block_number());
// 		Crowdloan::on_initialize(System::block_number());
// 	}
// }

pub(crate) fn roll_to(n: u64) {
	while System::block_number() < n {
		Crowdloan::on_finalize(System::block_number());
		Tokens::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Tokens::on_initialize(System::block_number());
		Crowdloan::on_initialize(System::block_number());
	}
}
