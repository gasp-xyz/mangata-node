// Copyright (C) 2020 Mangata team

use super::*;

use crate as pallet_fee_lock;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{Contains, Nothing},
	weights::constants::RocksDbWeight,
	PalletId,
};
use frame_system as system;
use mangata_support::pools::{PoolInfo, Valuate, ValuateFor};
use orml_traits::parameter_type_with_key;
use sp_runtime::{traits::AccountIdConversion, BuildStorage};
use sp_std::convert::TryFrom;

pub const NATIVE_CURRENCY_ID: u32 = 0;
pub(crate) type AccountId = u64;
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

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type DbWeight = RocksDbWeight;
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
	type NontransferableTokens = Nothing;
	type NontransferableTokensAllowList = Nothing;
}

parameter_types! {
	pub const NativeCurrencyId: u32 = NATIVE_CURRENCY_ID;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
}

pub struct MockValuateForNative {}
impl ValuateFor<NativeCurrencyId> for MockValuateForNative {}
impl Valuate for MockValuateForNative {
	type Balance = Balance;
	type CurrencyId = TokenId;

	fn find_paired_pool(
		base_id: Self::CurrencyId,
		asset_id: Self::CurrencyId,
	) -> Result<PoolInfo<Self::CurrencyId, Self::Balance>, DispatchError> {
		unimplemented!()
	}

	fn check_can_valuate(_: Self::CurrencyId, _: Self::CurrencyId) -> Result<(), DispatchError> {
		unimplemented!()
	}

	fn check_pool_exist(pool_id: Self::CurrencyId) -> Result<(), DispatchError> {
		unimplemented!()
	}

	fn get_reserve_and_lp_supply(
		_: Self::CurrencyId,
		pool_id: Self::CurrencyId,
	) -> Option<(Self::Balance, Self::Balance)> {
		unimplemented!()
	}

	fn get_valuation_for_paired(
		_: Self::CurrencyId,
		_: Self::CurrencyId,
		amount: Self::Balance,
	) -> Self::Balance {
		unimplemented!()
	}

	fn find_valuation(
		_: Self::CurrencyId,
		_: Self::CurrencyId,
		_: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		Ok(2000_u128)
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
	type ValuateForNative = MockValuateForNative;
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
