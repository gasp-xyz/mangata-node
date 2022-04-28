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
use crate as pallet_bootstrap;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU32, Contains, Everything},
};
use mangata_primitives::{Amount, Balance, TokenId};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyAdapter};
use orml_traits::parameter_type_with_key;
use pallet_issuance::PoolPromoteApi;
// use pallet_xyk::Pallet;

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

parameter_types!(
	pub const MGAId: TokenId = 0;
	pub const KSMId: TokenId = 1;
	pub const KsmToMgaNumerator: u128 = 1;
	pub const KsmToMgaDenominator: u128 = 10_000;
	pub const MaxLocks: u32 = 50;
);

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(_a: &AccountId) -> bool {
		true
	}
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
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = DustRemovalWhitelist;
}

parameter_types! {
	pub const NativeCurrencyId: u32 = 0;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
	pub const LiquidityMiningIssuanceVaultId: PalletId = PalletId(*b"py/lqmiv");
	pub FakeLiquidityMiningIssuanceVault: AccountId = LiquidityMiningIssuanceVaultId::get().into_account();
}

pub struct MockPromotedPoolApi;

impl MockPromotedPoolApi {}

impl PoolPromoteApi for MockPromotedPoolApi {
	fn promote_pool(_liquidity_token_id: TokenId) -> bool {
		false
	}

	fn get_pool_rewards(_liquidity_token_id: TokenId) -> Option<Balance> {
		None
	}

	fn claim_pool_rewards(_liquidity_token_id: TokenId, _claimed_amount: Balance) -> bool {
		false
	}

	fn len() -> usize {
		0
	}
}

impl pallet_xyk::Config for Test {
	type Event = Event;
	type Currency = MultiTokenCurrencyAdapter<Test>;
	type NativeCurrencyId = NativeCurrencyId;
	type TreasuryPalletId = TreasuryPalletId;
	type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
	type LiquidityMiningIssuanceVault = FakeLiquidityMiningIssuanceVault;
	type PoolPromoteApi = MockPromotedPoolApi;
	type PoolFeePercentage = ConstU128<20>;
	type TreasuryFeePercentage = ConstU128<5>;
	type BuyAndBurnFeePercentage = ConstU128<5>;
	type RewardsDistributionPeriod = ConstU32<10000>;
	type WeightInfo = ();
}

mockall::mock! {
	pub PoolCreateApi {}

	impl PoolCreateApi for PoolCreateApi {
		type AccountId = u128;

		fn pool_exists(first: TokenId, second: TokenId) -> bool;
		fn pool_create(account: u128, first: TokenId, first_amount: Balance, second: TokenId, second_amount: Balance) -> Option<(TokenId, Balance)>;
	}
}

#[cfg(not(feature = "runtime-benchmarks"))]
// NOTE: use PoolCreateApi mock for unit testing purposes
impl pallet_bootstrap::Config for Test {
	type Event = Event;
	type MGATokenId = MGAId;
	type KSMTokenId = KSMId;
	type PoolCreateApi = MockPoolCreateApi;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type KsmToMgaRatioNumerator = KsmToMgaNumerator;
	type KsmToMgaRatioDenominator = KsmToMgaDenominator;
}

#[cfg(feature = "runtime-benchmarks")]
// NOTE: use Xyk as PoolCreateApi for benchmarking purposes
impl pallet_bootstrap::Config for Test {
	type Event = Event;
	type MGATokenId = MGAId;
	type KSMTokenId = KSMId;
	type PoolCreateApi = Xyk;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type KsmToMgaRatioNumerator = KsmToMgaNumerator;
	type KsmToMgaRatioDenominator = KsmToMgaDenominator;
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

impl pallet_assets_info::Config for Test {
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
		AssetsInfoModule: pallet_assets_info::{Pallet, Call, Config, Storage, Event<T>},
		Xyk: pallet_xyk::{Pallet, Call, Storage, Event<T>, Config<T>},
		Bootstrap: pallet_bootstrap::{Pallet, Call, Storage, Event<T>},
	}
);

impl<T: Config> Pallet<T> {
	pub fn balance(id: TokenId, who: T::AccountId) -> Balance {
		<T as Config>::Currency::free_balance(id.into(), &who).into()
	}
	pub fn total_supply(id: TokenId) -> Balance {
		<T as Config>::Currency::total_issuance(id.into()).into()
	}
	pub fn transfer(
		currency_id: TokenId,
		source: T::AccountId,
		dest: T::AccountId,
		value: Balance,
	) -> DispatchResult {
		<T as Config>::Currency::transfer(
			currency_id.into(),
			&source,
			&dest,
			value.into(),
			frame_support::traits::ExistenceRequirement::KeepAlive,
		)
	}
	pub fn create_new_token(who: &T::AccountId, amount: Balance) -> TokenId {
		<T as Config>::Currency::create(who, amount.into())
			.expect("Token creation failed")
			.into()
	}
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.expect("Frame system builds valid default genesis config");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
