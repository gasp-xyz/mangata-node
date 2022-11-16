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
use crate as pallet_issuance;
use frame_support::{
	assert_ok, construct_runtime, parameter_types,
	traits::{Contains, Everything},
	PalletId,
};
use mangata_types::{Amount, Balance, TokenId};
use orml_traits::parameter_type_with_key;
use sp_runtime::{
	traits::{AccountIdConversion, ConvertInto},
	SaturatedConversion,
};
use sp_std::convert::TryFrom;
use std::{collections::HashMap, sync::Mutex};
pub const MGA_TOKEN_ID: TokenId = 0;
pub(crate) type AccountId = u128;

parameter_types!(
	pub const SomeConst: u64 = 10;
	pub const BlockHashCount: u32 = 250;
);

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = sp_runtime::testing::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::testing::Header;
	type Event = Event;
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
	pub const MgaTokenId: TokenId = 0u32;
	pub const BlocksPerRound: u32 = 5u32;
	pub const HistoryLimit: u32 = 10u32;
}

impl orml_tokens::Config for Test {
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

parameter_types! {
	pub const LiquidityMiningIssuanceVaultId: PalletId = PalletId(*b"py/lqmiv");
	pub LiquidityMiningIssuanceVault: AccountId = LiquidityMiningIssuanceVaultId::get().into_account_truncating();
	pub const StakingIssuanceVaultId: PalletId = PalletId(*b"py/stkiv");
	pub StakingIssuanceVault: AccountId = StakingIssuanceVaultId::get().into_account_truncating();


	pub const TotalCrowdloanAllocation: Balance = 200_000_000;
	pub const IssuanceCap: Balance = 4_000_000_000;
	pub const LinearIssuanceBlocks: u32 = 22_222u32;
	pub const LiquidityMiningSplit: Perbill = Perbill::from_parts(555555556);
	pub const StakingSplit: Perbill = Perbill::from_parts(444444444);
	pub const ImmediateTGEReleasePercent: Percent = Percent::from_percent(20);
	pub const TGEReleasePeriod: u32 = 100u32; // 2 years
	pub const TGEReleaseBegin: u32 = 10u32; // Two weeks into chain start

}

lazy_static::lazy_static! {
	static ref ACTIVATED_POOL: Mutex<HashMap<TokenId, U256>> = {
		let m = HashMap::new();
		Mutex::new(m)
	};
}

pub struct MockPromotedPoolApi;
pub struct MockActivedPoolQueryApi;

#[cfg(test)]

// impl MockActivedPoolQueryApi {
// 	pub fn instance() -> &'static Mutex<HashMap<TokenId, U256>> {
// 		&ACTIVATED_POOL
// 	}
// }

impl ActivedPoolQueryApi for MockActivedPoolQueryApi {
	fn get_pool_activate_amount(liquidity_token_id: TokenId) -> Option<Balance> {
		Some(1_u128)
	}
}

impl pallet_issuance::Config for Test {
	type Event = Event;
	type NativeCurrencyId = MgaTokenId;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type BlocksPerRound = BlocksPerRound;
	type HistoryLimit = HistoryLimit;
	type LiquidityMiningIssuanceVault = LiquidityMiningIssuanceVault;
	type StakingIssuanceVault = StakingIssuanceVault;
	type TotalCrowdloanAllocation = TotalCrowdloanAllocation;
	type IssuanceCap = IssuanceCap;
	type LinearIssuanceBlocks = LinearIssuanceBlocks;
	type LiquidityMiningSplit = LiquidityMiningSplit;
	type StakingSplit = StakingSplit;
	type ImmediateTGEReleasePercent = ImmediateTGEReleasePercent;
	type TGEReleasePeriod = TGEReleasePeriod;
	type TGEReleaseBegin = TGEReleaseBegin;
	type VestingProvider = Vesting;
	type WeightInfo = ();
	// TODO implement unit tests using mock
	type ActivedPoolQueryApiType = MockActivedPoolQueryApi;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100u128;
}

impl pallet_vesting_mangata::Config for Test {
	type Event = Event;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting_mangata::weights::SubstrateWeight<Test>;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	// Should be atleast twice the number of tge recipients
	const MAX_VESTING_SCHEDULES: u32 = 200;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>},
		Vesting: pallet_vesting_mangata::{Pallet, Call, Storage, Event<T>},
		Issuance: pallet_issuance::{Pallet, Event<T>, Storage},
	}
);

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_without_issuance_config() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.expect("Frame system builds valid default genesis config");

	orml_tokens::GenesisConfig::<Test> {
		tokens_endowment: vec![(0u128, 0u32, 2_000_000_000)],
		created_tokens_for_staking: Default::default(),
	}
	.assimilate_storage(&mut t)
	.expect("Tokens storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);

		if !StakeCurrency::exists(MGA_TOKEN_ID) {
			assert_ok!(StakeCurrency::create(&99999, 100));
		}
	});
	ext
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.expect("Frame system builds valid default genesis config");

	orml_tokens::GenesisConfig::<Test> {
		tokens_endowment: vec![(0u128, 0u32, 2_000_000_000)],
		created_tokens_for_staking: Default::default(),
	}
	.assimilate_storage(&mut t)
	.expect("Tokens storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);

		if !StakeCurrency::exists(MGA_TOKEN_ID) {
			assert_ok!(StakeCurrency::create(&99999, 100));
		}

		let current_issuance = StakeCurrency::total_issuance(MGA_TOKEN_ID);
		let target_tge = 2_000_000_000u128;
		assert!(current_issuance <= target_tge);

		assert_ok!(StakeCurrency::mint(MGA_TOKEN_ID, &99999, target_tge - current_issuance));

		assert_ok!(Issuance::finalize_tge(Origin::root()));
		assert_ok!(Issuance::init_issuance_config(Origin::root()));
		assert_ok!(Issuance::calculate_and_store_round_issuance(0u32));
	});
	ext
}

pub type StakeCurrency = orml_tokens::MultiTokenCurrencyAdapter<Test>;

pub(crate) fn roll_to_while_minting(n: u64, expected_amount_minted: Option<Balance>) {
	let mut session_number: u32;
	let mut session_issuance: (Balance, Balance);
	let mut block_issuance: Balance;
	while System::block_number() < n {
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		session_number = System::block_number().saturated_into::<u32>() / BlocksPerRound::get();
		session_issuance = <Issuance as GetIssuance>::get_all_issuance(session_number)
			.expect("session issuance is always populated in advance");
		block_issuance = (session_issuance.0 + session_issuance.1) /
			(BlocksPerRound::get().saturated_into::<u128>());

		if let Some(x) = expected_amount_minted {
			assert_eq!(x, block_issuance);
		}

		// Compute issuance for the next session only after all issuance has been issued is current session
		// To avoid overestimating the missing issuance and overshooting the cap
		if ((System::block_number().saturated_into::<u32>() + 1u32) % BlocksPerRound::get()) == 0 {
			<Issuance as ComputeIssuance>::compute_issuance(session_number + 1u32);
		}
	}
}
