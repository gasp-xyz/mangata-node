// Copyright (C) 2020 Mangata team

use crate::{Module, Trait};
use sp_core::H256;

use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use frame_system as system;
use mangata_primitives::{Amount, Balance, TokenId};
use orml_tokens::{MultiTokenCurrency, MultiTokenCurrencyAdapter, MultiTokenCurrencyExtended};

pub const NATIVE_CURRENCY_ID: u32 = 0;
use crate as encrypted;

impl_outer_origin! {
    pub enum Origin for Test {}
}

impl_outer_event! {
    pub enum TestEvent for Test {
        pallet_session,
        frame_system<T>,
        encrypted<T>,
        orml_tokens<T>,
    }
}
// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

frame_support::impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
        frame_system::System,
        encrypted::EncryptedTX,
	}
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

impl orml_tokens::Trait for Test {
    type Event = TestEvent;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = TokenId;
    type OnReceived = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const NativeCurrencyId: u32 = NATIVE_CURRENCY_ID;
    pub const TreasuryModuleId: sp_runtime::ModuleId = sp_runtime::ModuleId(*b"py/trsry");
    pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
}

mod currency {
    use mangata_primitives::Balance;

    pub const MILLICENTS: Balance = 1_000_000_000;
    pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
    pub const DOLLARS: Balance = 100 * CENTS;
}

parameter_types! {
    pub const EncryptedTxnsFee: Balance = 1 * currency::DOLLARS;
    pub const DoublyEncryptedCallMaxLength: u32 = 4096;
}

use pallet_session::{SessionManager, SessionHandler};
use sp_staking::SessionIndex;

pub struct TestSessionManager;
impl SessionManager<u64> for TestSessionManager {
	fn end_session(_: SessionIndex) {}
	fn start_session(_: SessionIndex) {}
	fn new_session(_: SessionIndex) -> Option<Vec<u64>> {
        None
	}
}

use sp_runtime::testing::UintAuthorityId;
use sp_runtime::traits::OpaqueKeys;

pub struct TestSessionHandler;
use sp_runtime::RuntimeAppPublic;

impl SessionHandler<u128> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
	fn on_genesis_session<T: OpaqueKeys>(_validators: &[(u128, T)]) {}
	fn on_new_session<T: OpaqueKeys>(
		changed: bool,
		validators: &[(u128, T)],
		_queued_validators: &[(u128, T)],
	) {
	}
	fn on_disabled(_validator_index: usize) {
	}
	fn on_before_session_ending() {
	}
}

sp_runtime::impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy: UintAuthorityId,
	}
}

impl From<UintAuthorityId> for MockSessionKeys {
	fn from(dummy: UintAuthorityId) -> Self {
		Self { dummy }
	}
}


impl pallet_session::Trait for Test {
	type Event = TestEvent;
	type ValidatorId = <Self as frame_system::Trait>::AccountId;
	type ValidatorIdOf = sp_runtime::traits::ConvertInto;
	type ShouldEndSession = pallet_session::PeriodicSessions<(), ()>;
	type NextSessionRotation = ();
	type SessionManager = ();
	type SessionHandler = TestSessionHandler;
	type Keys = MockSessionKeys;
	type DisabledValidatorsThreshold = ();
	type WeightInfo = ();
}

impl Trait for Test {
    type Event = TestEvent;
    type Call = Call;
    type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
    type AuthorityId = crate::ecdsa::AuthorityId;
    type Fee = EncryptedTxnsFee; 
    // TODO: figure out if we want to test treasury as well
    type Treasury = ();
    type DoublyEncryptedCallMaxLength = DoublyEncryptedCallMaxLength;
}

pub type EncryptedTX = Module<Test>;
pub type System = system::Module<Test>;

impl<T: Trait> Module<T> {
    // can implement some handy methods here
    // pub fn create_new_token(who: &T::AccountId, amount: Balance) -> TokenId {
    //     <T as Trait>::Currency::create(who, amount.into()).into()
    // }
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
