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
use codec::EncodeLike;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU32, Contains, Everything, Nothing},
};
use mangata_primitives::{Amount, Balance, TokenId};
use mp_multipurpose_liquidity::ActivateKind;
use mp_traits::ActivationReservesProviderTrait;
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyAdapter};
use orml_traits::parameter_type_with_key;
use pallet_xyk::AssetMetadataMutationTrait;
use sp_runtime::{Perbill, Percent};
use sp_std::convert::TryFrom;

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
	pub FakeLiquidityMiningIssuanceVault: AccountId = LiquidityMiningIssuanceVaultId::get().into_account_truncating();
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 0;
}

impl pallet_vesting_mangata::Config for Test {
	type Event = Event;
	type Tokens = MultiTokenCurrencyAdapter<Test>;
	type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting_mangata::weights::SubstrateWeight<Test>;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

pub struct AssetMetadataMutation;
impl AssetMetadataMutationTrait for AssetMetadataMutation {
	fn set_asset_info(
		_asset: TokenId,
		_name: Vec<u8>,
		_symbol: Vec<u8>,
		_decimals: u32,
	) -> DispatchResult {
		Ok(())
	}
}

impl pallet_xyk::Config for Test {
	type Event = Event;
	type ActivationReservesProvider = TokensActivationPassthrough<Test>;
	type Currency = MultiTokenCurrencyAdapter<Test>;
	type NativeCurrencyId = NativeCurrencyId;
	type TreasuryPalletId = TreasuryPalletId;
	type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
	type LiquidityMiningIssuanceVault = FakeLiquidityMiningIssuanceVault;
	type PoolPromoteApi = Issuance;
	type PoolFeePercentage = ConstU128<20>;
	type TreasuryFeePercentage = ConstU128<5>;
	type BuyAndBurnFeePercentage = ConstU128<5>;
	type RewardsDistributionPeriod = ConstU32<10000>;
	type WeightInfo = ();
	type DisallowedPools = Bootstrap;
	type DisabledTokens = Nothing;
	type VestingProvider = Vesting;
	type AssetMetadataMutation = AssetMetadataMutation;
}

impl BootstrapBenchmarkingConfig for Test {}

pub struct TokensActivationPassthrough<T: Config>(PhantomData<T>);
impl<T: Config> ActivationReservesProviderTrait for TokensActivationPassthrough<T>
where
	<T as frame_system::Config>::AccountId: EncodeLike<AccountId>,
{
	type AccountId = T::AccountId;

	fn get_max_instant_unreserve_amount(token_id: TokenId, account_id: &Self::AccountId) -> Balance
	where
		<T as frame_system::Config>::AccountId: EncodeLike<AccountId>,
	{
		Xyk::liquidity_mining_active_user((account_id.clone(), token_id))
	}

	fn can_activate(
		token_id: TokenId,
		account_id: &Self::AccountId,
		amount: Balance,
		_use_balance_from: Option<ActivateKind>,
	) -> bool {
		<T as pallet::Config>::Currency::can_reserve(token_id.into(), account_id, amount.into())
	}

	fn activate(
		token_id: TokenId,
		account_id: &Self::AccountId,
		amount: Balance,
		_use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		<T as pallet::Config>::Currency::reserve(token_id.into(), account_id, amount.into())
	}

	fn deactivate(token_id: TokenId, account_id: &Self::AccountId, amount: Balance) -> Balance {
		<T as pallet::Config>::Currency::unreserve(token_id.into(), account_id, amount.into())
			.into()
	}
}

parameter_types! {
	pub LiquidityMiningIssuanceVault: AccountId = LiquidityMiningIssuanceVaultId::get().into_account_truncating();
	pub const StakingIssuanceVaultId: PalletId = PalletId(*b"py/stkiv");
	pub StakingIssuanceVault: AccountId = StakingIssuanceVaultId::get().into_account_truncating();
	pub const MgaTokenId: TokenId = 0u32;


	pub const TotalCrowdloanAllocation: Balance = 200_000_000;
	pub const IssuanceCap: Balance = 4_000_000_000;
	pub const LinearIssuanceBlocks: u32 = 22_222u32;
	pub const LiquidityMiningSplit: Perbill = Perbill::from_parts(555555556);
	pub const StakingSplit: Perbill = Perbill::from_parts(444444444);
	pub const ImmediateTGEReleasePercent: Percent = Percent::from_percent(20);
	pub const TGEReleasePeriod: u32 = 100u32; // 2 years
	pub const TGEReleaseBegin: u32 = 10u32; // Two weeks into chain start
	pub const BlocksPerRound: u32 = 5u32;
	pub const HistoryLimit: u32 = 10u32;
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
}

mockall::mock! {
	pub PoolCreateApi {}

	impl PoolCreateApi for PoolCreateApi {
		type AccountId = u128;

		fn pool_exists(first: TokenId, second: TokenId) -> bool;
		fn pool_create(account: u128, first: TokenId, first_amount: Balance, second: TokenId, second_amount: Balance) -> Option<(TokenId, Balance)>;

	}
}

mockall::mock! {
	pub RewardsApi {}

	impl RewardsApi for RewardsApi {
		type AccountId = u128;

		fn can_activate(liquidity_asset_id: TokenId) -> bool;

		fn activate_liquidity_tokens(
			user: &u128,
			liquidity_asset_id: TokenId,
			amount: Balance,
		) -> DispatchResult;
	}
}

#[cfg(not(feature = "runtime-benchmarks"))]
// NOTE: use PoolCreateApi mock for unit testing purposes
impl pallet_bootstrap::Config for Test {
	type Event = Event;
	type PoolCreateApi = MockPoolCreateApi;
	type TreasuryPalletId = TreasuryPalletId;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type VestingProvider = Vesting;
	type RewardsApi = MockRewardsApi;
	type WeightInfo = ();
}

#[cfg(feature = "runtime-benchmarks")]
// NOTE: use Xyk as PoolCreateApi for benchmarking purposes
impl pallet_bootstrap::Config for Test {
	type Event = Event;
	type PoolCreateApi = Xyk;
	type TreasuryPalletId = TreasuryPalletId;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type VestingProvider = Vesting;
	type RewardsApi = Xyk;
	type WeightInfo = ();
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
		Xyk: pallet_xyk::{Pallet, Call, Storage, Event<T>, Config<T>},
		Bootstrap: pallet_bootstrap::{Pallet, Call, Storage, Event<T>},
		Vesting: pallet_vesting_mangata::{Pallet, Call, Storage, Event<T>},
		Issuance: pallet_issuance::{Pallet, Event<T>, Storage},
	}
);

impl<T: Config> Pallet<T>
where
	u128: From<<T as frame_system::Config>::AccountId>,
{
	pub fn balance(id: TokenId, who: T::AccountId) -> Balance {
		Tokens::accounts(Into::<u128>::into(who.clone()), Into::<u32>::into(id.clone())).free -
			Tokens::accounts(Into::<u128>::into(who.clone()), Into::<u32>::into(id.clone()))
				.frozen
	}

	pub fn reserved_balance(id: TokenId, who: <T as frame_system::Config>::AccountId) -> Balance {
		Tokens::accounts(Into::<u128>::into(who), Into::<u32>::into(id)).reserved
	}

	pub fn locked_balance(id: TokenId, who: <T as frame_system::Config>::AccountId) -> Balance {
		Tokens::accounts(Into::<u128>::into(who), Into::<u32>::into(id)).frozen
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
