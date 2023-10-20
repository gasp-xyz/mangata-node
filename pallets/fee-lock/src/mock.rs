// Copyright (C) 2020 Mangata team

use super::*;

use crate as pallet_fee_lock;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Contains, Everything},
	weights::constants::RocksDbWeight,
	PalletId,
};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use sp_runtime::{traits::AccountIdConversion, BuildStorage};
use sp_std::convert::TryFrom;

pub const NATIVE_CURRENCY_ID: u32 = 0;
pub(crate) type AccountId = u128;
pub(crate) type Balance = u128;
pub(crate) type TokenId = u32;
pub(crate) type Amount = i128;

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System: frame_system,
		Tokens: orml_tokens,
		FeeLock: pallet_fee_lock,
	}
);

parameter_types!(
	pub const BlockHashCount: u32 = 250;
);

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
	type DbWeight = RocksDbWeight;
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
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub const MaxLocks: u32 = 50;
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
	pub const NativeCurrencyId: u32 = NATIVE_CURRENCY_ID;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
}

pub struct MockPoolReservesProvider<T>(PhantomData<T>);

impl<T: pallet_fee_lock::Config> Valuate<Balance, TokenId> for MockPoolReservesProvider<T> {
	fn get_liquidity_asset(
		_first_asset_id: TokenId,
		_second_asset_id: TokenId,
	) -> Result<TokenId, DispatchError> {
		unimplemented!()
	}

	fn get_liquidity_token_mga_pool(
		_liquidity_token_id: TokenId,
	) -> Result<(TokenId, TokenId), DispatchError> {
		unimplemented!()
	}

	fn valuate_liquidity_token(
		_liquidity_token_id: TokenId,
		_liquidity_token_amount: Balance,
	) -> Balance {
		unimplemented!()
	}

	fn scale_liquidity_by_mga_valuation(
		_mga_valuation: Balance,
		_liquidity_token_amount: Balance,
		_mga_token_amount: Balance,
	) -> Balance {
		unimplemented!()
	}

	fn get_pool_state(_liquidity_token_id: TokenId) -> Option<(Balance, Balance)> {
		unimplemented!()
	}

	fn get_reserves(
		first_asset_id: TokenId,
		second_asset_id: TokenId,
	) -> Result<(Balance, Balance), DispatchError> {
		match (first_asset_id, second_asset_id) {
			(0, 1) => Ok((5000, 10000)),
			(0, 2) => Ok((10000, 5000)),
			(0, 3) => Ok((0, 10000)),
			(0, 4) => Ok((5000, 0)),
			_ => Err(pallet_fee_lock::Error::<T>::UnexpectedFailure.into()),
		}
	}
}

parameter_types! {
	#[derive(PartialEq)]
	pub const MaxCuratedTokens: u32 = 100;
}

impl pallet_fee_lock::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxCuratedTokens = MaxCuratedTokens;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type PoolReservesProvider = MockPoolReservesProvider<Test>;
	type NativeTokenId = NativeCurrencyId;
	type WeightInfo = ();
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

pub struct ExtBuilder {
	ext: sp_io::TestExternalities,
}

impl ExtBuilder {
	pub fn new() -> Self {
		let t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		let ext = sp_io::TestExternalities::new(t);
		Self { ext }
	}

	pub fn create_token(mut self, token_id: TokenId) -> Self {
		self.ext.execute_with(|| {
			while token_id >= Tokens::next_asset_id() {
				Tokens::create(RuntimeOrigin::root(), 0, 0).unwrap();
			}
		});
		return self
	}

	pub fn mint(mut self, who: AccountId, token_id: TokenId, balance: Balance) -> Self {
		self.ext
			.execute_with(|| Tokens::mint(RuntimeOrigin::root(), token_id, who, balance).unwrap());
		return self
	}

	pub fn initialize_fee_locks(mut self, period: u64, lock_amount: u128, threshold: u128) -> Self {
		self.ext.execute_with(|| {
			FeeLock::update_fee_lock_metadata(
				RuntimeOrigin::root(),
				Some(period),
				Some(lock_amount),
				Some(threshold),
				None,
			)
			.unwrap()
		});
		return self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		self.ext
	}
}

pub fn fast_forward_blocks(count: u64) {
	let now = System::block_number();
	for i in 0..count {
		System::set_block_number(now + i + 1);
	}
}
