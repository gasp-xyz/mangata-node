use crate as bridge;
use super::*;

use frame_support::{
    construct_runtime, parameter_types,
};
use mangata_primitives::{Amount, Balance, TokenId};
use orml_tokens::MultiTokenCurrencyAdapter;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    MultiSignature,
};
use sp_std::convert::From;
use orml_traits::parameter_type_with_key;


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
        BridgedAssets: artemis_asset::{Module, Call, Storage, Config<T>, Event<T>},
		Tokens: orml_tokens::{Module, Storage, Call, Event<T>, Config<T>},
        ERC20: artemis_erc20_app::{Module, Storage, Call, Event<T>},
        ETH: artemis_eth_app::{Module, Storage, Call, Event<T>},
        Verifier: pallet_verifier::{Module, Storage, Call, Event, Config<T>},
        BridgeModule: bridge::{Module, Storage, Call, Event, Config},
	}
);


parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl system::Config for Test {
	type BaseCallFilter = ();
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
}

impl pallet_verifier::Config for Test {
    type Event = Event;
}

impl artemis_eth_app::Config for Test {
    type Event = Event;
}

impl artemis_erc20_app::Config for Test {
    type Event = Event;
}

impl artemis_asset::Config for Test {
    type Event = Event;
    type Currency = MultiTokenCurrencyAdapter<Test>;
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
}

impl Config for Test {
    type Event = Event;
    type Verifier = pallet_verifier::Module<Test>;
    type AppETH = artemis_eth_app::Module<Test>;
    type AppERC20 = artemis_erc20_app::Module<Test>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
