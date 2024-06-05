// Copyright (C) 2020 Mangata team

use self::consts::DEFAULT_CHAIN_ID;

use super::*;

use crate as sequencer_staking;
use core::convert::TryFrom;
use frame_support::{construct_runtime, parameter_types, traits::Everything};

use frame_support::traits::ConstU128;
pub use mangata_support::traits::ProofOfStakeRewardsApi;
use mockall::automock;
use orml_traits::parameter_type_with_key;
use sp_runtime::{traits::ConvertBack, BuildStorage, Permill, Saturating};

pub(crate) type AccountId = u64;
pub(crate) type Amount = i128;
pub(crate) type Balance = u128;
pub(crate) type TokenId = u32;
pub(crate) type ChainId = u32;

pub mod consts {
	pub const MILLION: u128 = 1_000_000;
	pub const ALICE: u64 = 2;
	pub const BOB: u64 = 3;
	pub const CHARLIE: u64 = 4;
	pub const DAVE: u64 = 5;
	pub const EVE: u64 = 6;

	pub const NATIVE_TOKEN_ID: super::TokenId = 0;
	pub const DEFAULT_CHAIN_ID: u32 = 1;

	pub const TOKENS_ENDOWED: super::Balance = 10_000;
	pub const MINIMUM_STAKE: super::Balance = 1000;
	pub const SLASH_AMOUNT: super::Balance = 100;
}

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System: frame_system,
		Tokens: orml_tokens,
		SequencerStaking: sequencer_staking
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
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

impl orml_tokens::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = TokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = ();
	type DustRemovalWhitelist = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type CurrencyHooks = ();
}

parameter_types! {
	pub const CancellerRewardPercentage: Permill = Permill::from_percent(20);
	pub const NativeTokenId: TokenId = 0;
}

impl sequencer_staking::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = orml_tokens::CurrencyAdapter<Test, NativeTokenId>;
	type MinimumSequencers = frame_support::traits::ConstU32<2>;
	type RolldownProvider = MockRolldownProviderApi;
	type NoOfPastSessionsForEligibility = frame_support::traits::ConstU32<2>;
	type MaxSequencers = frame_support::traits::ConstU32<11>;
	type BlocksForSequencerUpdate = frame_support::traits::ConstU32<2>;
	type CancellerRewardPercentage = CancellerRewardPercentage;
	type ChainId = ChainId;
}

mockall::mock! {
	pub RolldownProviderApi {}

	impl RolldownProviderTrait<ChainId, AccountId> for RolldownProviderApi {
		fn new_sequencer_active(chain: ChainId, sequencer: &AccountId);
		fn sequencer_unstaking(chain: ChainId, sequencer: &AccountId)->DispatchResult;
		fn handle_sequencer_deactivations(chain: ChainId, deactivated_sequencers: Vec<AccountId>);
	}
}

pub struct ExtBuilder {
	ext: sp_io::TestExternalities,
}

impl ExtBuilder {
	pub fn new() -> Self {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		orml_tokens::GenesisConfig::<Test> {
			tokens_endowment: vec![
				(consts::ALICE, consts::NATIVE_TOKEN_ID, consts::TOKENS_ENDOWED),
				(consts::BOB, consts::NATIVE_TOKEN_ID, consts::TOKENS_ENDOWED),
				(consts::CHARLIE, consts::NATIVE_TOKEN_ID, consts::TOKENS_ENDOWED),
				(consts::DAVE, consts::NATIVE_TOKEN_ID, consts::TOKENS_ENDOWED),
			],
			created_tokens_for_staking: Default::default(),
		}
		.assimilate_storage(&mut t)
		.expect("Tokens storage can be assimilated");

		sequencer_staking::GenesisConfig::<Test> {
			minimal_stake_amount: consts::MINIMUM_STAKE,
			slash_fine_amount: consts::SLASH_AMOUNT,
			sequencers_stake: vec![
				(consts::ALICE, consts::DEFAULT_CHAIN_ID, consts::MINIMUM_STAKE),
				(consts::BOB, consts::DEFAULT_CHAIN_ID, consts::MINIMUM_STAKE),
			],
		}
		.assimilate_storage(&mut t)
		.expect("SequencerStaking storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);

		Self { ext }
	}

	fn create_if_does_not_exists(&mut self, token_id: TokenId) {
		self.ext.execute_with(|| {
			while token_id >= Tokens::next_asset_id() {
				Tokens::create(RuntimeOrigin::root(), 0, 0).unwrap();
			}
		});
	}

	pub fn issue(mut self, who: AccountId, token_id: TokenId, balance: Balance) -> Self {
		self.create_if_does_not_exists(token_id);
		self.ext
			.execute_with(|| Tokens::mint(RuntimeOrigin::root(), token_id, who, balance).unwrap());
		return self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		self.ext
	}
}

#[macro_use]
macro_rules! set_default_mocks {
	() => {
		let new_sequencer_active_mock = MockRolldownProviderApi::new_sequencer_active_context();
		new_sequencer_active_mock.expect().times(2).returning(|_, _| ());
	};
}
pub(crate) use set_default_mocks;

pub fn forward_to_block<T>(n: BlockNumberFor<T>)
where
	T: frame_system::Config,
	T: sequencer_staking::Config,
{
	while frame_system::Pallet::<T>::block_number() < n {
		sequencer_staking::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
		frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
		let new_block_number =
			frame_system::Pallet::<T>::block_number().saturating_add(1u32.into());
		frame_system::Pallet::<T>::set_block_number(new_block_number);

		frame_system::Pallet::<T>::on_initialize(new_block_number);
		sequencer_staking::Pallet::<T>::on_initialize(new_block_number);
	}
}

pub(crate) fn events() -> Vec<pallet::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(
			|e| if let RuntimeEvent::SequencerStaking(inner) = e { Some(inner) } else { None },
		)
		.collect::<Vec<_>>()
}

#[macro_export]
macro_rules! assert_eq_events {
	($events:expr) => {
		match &$events {
			e => similar_asserts::assert_eq!(*e, $crate::mock::events()),
		}
	};
}

#[macro_export]
macro_rules! assert_event_emitted {
	($event:expr) => {
		match &$event {
			e => {
				assert!(
					$crate::mock::events().iter().find(|x| *x == e).is_some(),
					"Event {:?} was not found in events: \n {:?}",
					e,
					crate::mock::events()
				);
			},
		}
	};
}
