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
	construct_runtime, parameter_types,
	traits::{Contains, Everything},
	PalletId,
};
use mangata_primitives::{Amount, Balance, TokenId};
use orml_traits::parameter_type_with_key;
use sp_runtime::{traits::AccountIdConversion, SaturatedConversion};

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
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account();
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
	pub LiquidityMiningIssuanceVault: AccountId = LiquidityMiningIssuanceVaultId::get().into_account();
	pub const StakingIssuanceVaultId: PalletId = PalletId(*b"py/stkiv");
	pub StakingIssuanceVault: AccountId = StakingIssuanceVaultId::get().into_account();
	pub const CrowdloanIssuanceVaultId: PalletId = PalletId(*b"py/crliv");
	pub CrowdloanIssuanceVault: AccountId = CrowdloanIssuanceVaultId::get().into_account();
}

impl pallet_issuance::Config for Test {
	type Event = Event;
	type NativeCurrencyId = MgaTokenId;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type BlocksPerRound = BlocksPerRound;
	type HistoryLimit = HistoryLimit;
	type LiquidityMiningIssuanceVault = LiquidityMiningIssuanceVault;
	type StakingIssuanceVault = StakingIssuanceVault;
	type CrowdloanIssuanceVault = CrowdloanIssuanceVault;
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
		Issuance: pallet_issuance::{Pallet, Event<T>, Storage, Config},
	}
);

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.expect("Frame system builds valid default genesis config");

	orml_tokens::GenesisConfig::<Test> {
		tokens_endowment: vec![(0u128, 0u32, 2_000_000_000)],
		vesting_tokens: Default::default(),
		created_tokens_for_staking: Default::default(),
	}
	.assimilate_storage(&mut t)
	.expect("Tokens storage can be assimilated");

	GenesisBuild::<Test>::assimilate_storage(
		&pallet_issuance::GenesisConfig {
			issuance_config: IssuanceInfo {
				cap: 4_000_000_000u128,
				tge: 2_000_000_000u128,
				// Only blocks from [0, 22_219] will be considered as the linear period
				// The tokens missing at tge will be attempted to be distributed over this time period
				// Missed opportunities for minting tokens such as at block 0 (genesis block) and or failure to claim will be counted as burned
				linear_issuance_blocks: 22_222u32,
				liquidity_mining_split: Perbill::from_parts(555555556),
				staking_split: Perbill::from_parts(444444444),
				crowdloan_allocation: 200_000_000u128,
			},
		},
		&mut t,
	)
	.expect("pallet-issuance's storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

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
