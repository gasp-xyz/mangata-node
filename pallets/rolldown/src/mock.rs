// Copyright (C) 2020 Mangata team

use super::*;

use crate as rolldown;
use core::convert::TryFrom;
use frame_support::{construct_runtime, parameter_types, traits::Everything};

use frame_support::traits::ConstU128;
pub use mangata_support::traits::ProofOfStakeRewardsApi;
use orml_traits::parameter_type_with_key;
use sp_runtime::{traits::ConvertBack, BuildStorage, Saturating};

pub(crate) type AccountId = u64;
pub(crate) type Amount = i128;
pub(crate) type Balance = u128;
pub(crate) type TokenId = u32;

pub mod consts {
	pub const MILLION: u128 = 1_000_000;
	pub const ALICE: u64 = 2;
	pub const BOB: u64 = 3;
	pub const CHARLIE: u64 = 4;
	pub const EVE: u64 = 5;
}

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System: frame_system,
		Tokens: orml_tokens,
		Rolldown: rolldown
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

mockall::mock! {
	pub SequencerStakingProviderApi {}

	impl SequencerStakingProviderTrait<AccountId, Balance> for SequencerStakingProviderApi {
		fn is_active_sequencer(sequencer: &AccountId) -> bool;
		fn slash_sequencer<'a>(to_be_slashed: &AccountId, maybe_to_reward: Option<&'a AccountId>) -> DispatchResult;
		fn is_selected_sequencer(sequencer: &AccountId) -> bool;
	}
}

mockall::mock! {
	pub AssetRegistryProviderApi {}
	impl AssetRegistryProviderTrait<TokenId> for AssetRegistryProviderApi {
		fn get_l1_asset_id(l1_asset: L1Asset) -> Option<TokenId>;
		fn create_l1_asset(l1_asset: L1Asset) -> Result<TokenId, DispatchError>;
	}
}

pub struct DummyAddressConverter();

impl Convert<[u8; 20], AccountId> for DummyAddressConverter {
	fn convert(account: [u8; 20]) -> AccountId {
		let mut bytes = [0u8; 8];
		bytes.copy_from_slice(&account[0..8]);
		AccountId::from_be_bytes(bytes)
	}
}

impl ConvertBack<[u8; 20], AccountId> for DummyAddressConverter {
	fn convert_back(account: AccountId) -> [u8; 20] {
		let mut address = [0u8; 20];
		let bytes: Vec<u8> = account
			.to_be_bytes()
			.iter()
			.cloned()
			.chain(std::iter::repeat(0u8).take(12))
			.into_iter()
			.collect();

		address.copy_from_slice(&bytes[..]);
		address
	}
}

impl rolldown::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type SequencerStakingProvider = MockSequencerStakingProviderApi;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type AssetRegistryProvider = MockAssetRegistryProviderApi;
	type AddressConverter = DummyAddressConverter;
	type DisputePeriodLength = ConstU128<5>;
	type RequestsPerBlock = ConstU128<10>;
}

pub struct ExtBuilder {
	ext: sp_io::TestExternalities,
}

impl ExtBuilder {
	pub fn new() -> Self {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		rolldown::GenesisConfig::<Test> { _phantom: Default::default() }
			.assimilate_storage(&mut t)
			.expect("Tokens storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);

		ext.execute_with(|| {
			for s in vec![consts::ALICE, consts::BOB, consts::CHARLIE].iter() {
				Pallet::<Test>::new_sequencer_active(s);
			}
		});

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

	pub fn execute_with_default_mocks<R>(mut self, f: impl FnOnce() -> R) -> R {
		self.ext.execute_with(|| {
			let is_liquidity_token_mock =
				MockSequencerStakingProviderApi::is_active_sequencer_context();
			is_liquidity_token_mock.expect().return_const(true);

			let is_selected_sequencer_mock =
				MockSequencerStakingProviderApi::is_selected_sequencer_context();
			is_selected_sequencer_mock.expect().return_const(true);

			let get_l1_asset_id_mock = MockAssetRegistryProviderApi::get_l1_asset_id_context();
			get_l1_asset_id_mock.expect().return_const(crate::tests::ETH_TOKEN_ADDRESS_MGX);

			f()
		})
	}
}

pub fn forward_to_block<T>(n: BlockNumberFor<T>)
where
	T: frame_system::Config,
	T: rolldown::Config,
{
	while frame_system::Pallet::<T>::block_number() < n {
		let new_block_number =
			frame_system::Pallet::<T>::block_number().saturating_add(1u32.into());
		frame_system::Pallet::<T>::set_block_number(new_block_number);

		frame_system::Pallet::<T>::on_initialize(new_block_number);
		rolldown::Pallet::<T>::on_initialize(new_block_number);
		rolldown::Pallet::<T>::on_finalize(new_block_number);
		frame_system::Pallet::<T>::on_finalize(new_block_number);
	}
}

pub(crate) fn events() -> Vec<pallet::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Rolldown(inner) = e { Some(inner) } else { None })
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
