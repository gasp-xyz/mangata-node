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
	traits::{
		tokens::currency::MultiTokenCurrency, ConstU128, ConstU32, Contains, Everything, Nothing,
	},
};
use mangata_support::traits::ActivationReservesProviderTrait;
use mangata_types::{multipurpose_liquidity::ActivateKind, Amount, Balance, TokenId};
use orml_tokens::MultiTokenCurrencyAdapter;
use orml_traits::parameter_type_with_key;
use pallet_xyk::AssetMetadataMutationTrait;
use sp_runtime::{Perbill, Percent};
use sp_std::convert::TryFrom;
use std::sync::Mutex;

pub(crate) type AccountId = u128;

parameter_types!(
	pub const SomeConst: u64 = 10;
	pub const BlockHashCount: u32 = 250;
);

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
	pub ExistentialDeposits: |_currency_id: TokenId| -> Balance {
		0
	};
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
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type CurrencyHooks = ();
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
	type RuntimeEvent = RuntimeEvent;
	type Tokens = MultiTokenCurrencyAdapter<Test>;
	type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting_mangata::weights::SubstrateWeight<Test>;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

pub struct RewardsMigrateAccountProvider<T: frame_system::Config>(PhantomData<T>);
impl<T: frame_system::Config> Get<T::AccountId> for RewardsMigrateAccountProvider<T> {
	fn get() -> T::AccountId {
		T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
			.expect("Infinite input; no dead input space; qed")
	}
}

impl pallet_xyk::XykBenchmarkingConfig for Test {}

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
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = MockMaintenanceStatusProvider;
	type ActivationReservesProvider = TokensActivationPassthrough<Test>;
	type Currency = MultiTokenCurrencyAdapter<Test>;
	type NativeCurrencyId = NativeCurrencyId;
	type TreasuryPalletId = TreasuryPalletId;
	type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
	type LiquidityMiningRewards = ProofOfStake;
	type PoolFeePercentage = ConstU128<20>;
	type TreasuryFeePercentage = ConstU128<5>;
	type BuyAndBurnFeePercentage = ConstU128<5>;
	type WeightInfo = ();
	type DisallowedPools = Bootstrap;
	type DisabledTokens = Nothing;
	type VestingProvider = Vesting;
	type AssetMetadataMutation = AssetMetadataMutation;
	type RewardsMigrateAccount = RewardsMigrateAccountProvider<Test>;
}

impl pallet_proof_of_stake::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ActivationReservesProvider = TokensActivationPassthrough<Test>;
	type NativeCurrencyId = NativeCurrencyId;
	type Currency = MultiTokenCurrencyAdapter<Test>;
	type LiquidityMiningIssuanceVault = FakeLiquidityMiningIssuanceVault;
	type RewardsDistributionPeriod = ConstU32<10000>;
	type WeightInfo = ();
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
		ProofOfStake::get_rewards_info(account_id.clone(), token_id).activated_amount
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

	impl ProofOfStakeRewardsApi<AccountId> for RewardsApi {
		type Balance = <Test as orml_tokens::Config>::Balance;
		type CurrencyId = <Test as orml_tokens::Config>::CurrencyId;

	fn enable(liquidity_token_id: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::CurrencyId, weight: u8);

	fn disable(liquidity_token_id: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::CurrencyId);

	fn is_enabled(
		liquidity_token_id: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::CurrencyId,
	) -> bool;

	fn claim_rewards_all(
		sender: AccountId,
		liquidity_token_id: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::CurrencyId,
	) -> Result<<mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::Balance, DispatchError>;

	// Activation & deactivation should happen in PoS
	fn activate_liquidity(
		sender: AccountId,
		liquidity_token_id: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::CurrencyId,
		amount: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult;

	// Activation & deactivation should happen in PoS
	fn deactivate_liquidity(
		sender: AccountId,
		liquidity_token_id: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::CurrencyId,
		amount: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::Balance,
	) -> DispatchResult;

	fn calculate_rewards_amount(
		user: AccountId,
		liquidity_asset_id: <mock::MockRewardsApi as ProofOfStakeRewardsApi<AccountId>>::CurrencyId,
	) -> Result<Balance, DispatchError>;

	}
}

mockall::mock! {
	pub AssetRegistryApi {}

	impl AssetRegistryApi for AssetRegistryApi {
		fn enable_pool_creation(assets: (TokenId, TokenId)) -> bool;
	}
}

pub struct AssetRegistry;
impl AssetRegistryApi for AssetRegistry {
	fn enable_pool_creation(_assets: (TokenId, TokenId)) -> bool {
		true
	}
}

pub struct MockMaintenanceStatusProvider;

lazy_static::lazy_static! {
	static ref MAINTENANCE_STATUS: Mutex<bool> = {
		let m: bool = false;
		Mutex::new(m)
	};
}

#[cfg(test)]
impl MockMaintenanceStatusProvider {
	pub fn instance() -> &'static Mutex<bool> {
		&MAINTENANCE_STATUS
	}
}

impl MockMaintenanceStatusProvider {
	pub fn set_maintenance(value: bool) {
		let mut mutex = Self::instance().lock().unwrap();
		*mutex = value;
	}
}

impl GetMaintenanceStatusTrait for MockMaintenanceStatusProvider {
	fn is_maintenance() -> bool {
		let mutex = Self::instance().lock().unwrap();
		*mutex
	}

	fn is_upgradable() -> bool {
		unimplemented!()
	}
}

parameter_types! {
	pub const BootstrapUpdateBuffer: <Test as frame_system::Config>::BlockNumber = 10;
	pub const DefaultBootstrapPromotedPoolWeight: u8 = 1u8;
	pub const ClearStorageLimit: u32 = 10u32;
}

#[cfg(not(feature = "runtime-benchmarks"))]
impl pallet_bootstrap::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = MockMaintenanceStatusProvider;
	type PoolCreateApi = MockPoolCreateApi;
	type DefaultBootstrapPromotedPoolWeight = DefaultBootstrapPromotedPoolWeight;
	type BootstrapUpdateBuffer = BootstrapUpdateBuffer;
	type TreasuryPalletId = TreasuryPalletId;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type VestingProvider = Vesting;
	type RewardsApi = MockRewardsApi;
	type ClearStorageLimit = ClearStorageLimit;
	type WeightInfo = ();
	type AssetRegistryApi = MockAssetRegistryApi;
}

#[cfg(feature = "runtime-benchmarks")]
impl pallet_bootstrap::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = MockMaintenanceStatusProvider;
	type PoolCreateApi = Xyk;
	type DefaultBootstrapPromotedPoolWeight = DefaultBootstrapPromotedPoolWeight;
	type BootstrapUpdateBuffer = BootstrapUpdateBuffer;
	type TreasuryPalletId = TreasuryPalletId;
	type Currency = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type VestingProvider = Vesting;
	type RewardsApi = ProofOfStake;
	type ClearStorageLimit = ClearStorageLimit;
	type WeightInfo = ();
	type AssetRegistryApi = AssetRegistry;
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
		ProofOfStake: pallet_proof_of_stake::{Pallet, Call, Storage, Event<T>},
	}
);

impl<T: Config> Pallet<T>
where
	u128: From<<T as frame_system::Config>::AccountId>,
{
	pub fn balance(id: TokenId, who: T::AccountId) -> Balance {
		Tokens::accounts(Into::<u128>::into(who.clone()), Into::<u32>::into(id)).free -
			Tokens::accounts(Into::<u128>::into(who), Into::<u32>::into(id)).frozen
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
	ext.execute_with(|| {
		System::set_block_number(1);
		MockMaintenanceStatusProvider::set_maintenance(false);
	});
	ext
}
