// Copyright (C) 2020 Mangata team

use super::*;

use sp_core::H256;

use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};

use crate as pallet_multipurpose_liquidity;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{tokens::currency::MultiTokenCurrency, ConstU128, ConstU32, Contains, Everything},
	PalletId,
};
use frame_system as system;
use mangata_types::{Amount, Balance, TokenId};
use orml_tokens::{MultiTokenCurrencyAdapter, MultiTokenCurrencyExtended};
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
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>},
		Vesting: pallet_vesting_mangata::{Pallet, Call, Storage, Event<T>},
		MultiPurposeLiquidity: pallet_multipurpose_liquidity::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl system::Config for Test {
	type BaseCallFilter = Everything;
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
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
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub const MaxLocks: u32 = 50;
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
	pub const NativeCurrencyId: u32 = NATIVE_CURRENCY_ID;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
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
	const MAX_VESTING_SCHEDULES: u32 = 50;
}

impl Config for Test {
	type Event = Event;
	type MaxRelocks = MaxLocks;
	type Tokens = MultiTokenCurrencyAdapter<Test>;
	type NativeCurrencyId = NativeCurrencyId;
	type VestingProvider = Vesting;
	type Xyk = MockXyk<Test>;
	type WeightInfo = ();
}

pub struct MockXyk<T>(PhantomData<T>);
impl<T: Config> XykFunctionsTrait<T::AccountId> for MockXyk<T> {
	type Balance = Balance;

	type CurrencyId = TokenId;

	fn create_pool(
		sender: T::AccountId,
		first_asset_id: Self::CurrencyId,
		first_asset_amount: Self::Balance,
		second_asset_id: Self::CurrencyId,
		second_asset_amount: Self::Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn sell_asset(
		sender: T::AccountId,
		sold_asset_id: Self::CurrencyId,
		bought_asset_id: Self::CurrencyId,
		sold_asset_amount: Self::Balance,
		min_amount_out: Self::Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn buy_asset(
		sender: T::AccountId,
		sold_asset_id: Self::CurrencyId,
		bought_asset_id: Self::CurrencyId,
		bought_asset_amount: Self::Balance,
		max_amount_in: Self::Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn mint_liquidity(
		sender: T::AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		first_asset_amount: Self::Balance,
		expected_second_asset_amount: Self::Balance,
		activate_minted_liquidity: bool,
	) -> Result<(Self::CurrencyId, Self::Balance), DispatchError> {
		unimplemented!()
	}

	fn burn_liquidity(
		sender: T::AccountId,
		first_asset_id: Self::CurrencyId,
		second_asset_id: Self::CurrencyId,
		liquidity_asset_amount: Self::Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn get_tokens_required_for_minting(
		liquidity_asset_id: Self::CurrencyId,
		liquidity_token_amount: Self::Balance,
	) -> Result<(Self::CurrencyId, Self::Balance, Self::CurrencyId, Self::Balance), DispatchError> {
		unimplemented!()
	}

	fn update_pool_promotion(liquidity_token_id: TokenId, liquidity_mining_issuance_percent: Option<Percent>) -> DispatchResult {
		unimplemented!()
	}

	fn is_liquidity_token(liquidity_asset_id: TokenId) -> bool {
		true
	}

	fn claim_rewards_v2(
		sender: T::AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn claim_rewards_all_v2(
		sender: T::AccountId,
		liquidity_token_id: Self::CurrencyId,
	) -> DispatchResult {
		unimplemented!()
	}

	fn activate_liquidity_v2(
		sender: T::AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
		use_balance_from: Option<ActivateKind>,
	) -> DispatchResult {
		unimplemented!()
	}

	fn deactivate_liquidity_v2(
		sender: T::AccountId,
		liquidity_token_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn rewards_migrate_v1_to_v2(
		account: T::AccountId,
		liquidity_token_id: Self::CurrencyId,
	) -> DispatchResult {
		unimplemented!()
	}
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
