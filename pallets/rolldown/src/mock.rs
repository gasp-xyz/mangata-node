// Copyright (C) 2020 Mangata team

use super::*;

use crate as rolldown;
use core::convert::TryFrom;
use frame_support::{construct_runtime, derive_impl, parameter_types, traits::Everything};
use sp_runtime::traits::One;
use std::collections::HashSet;

use frame_support::traits::ConstU128;
pub use mangata_support::traits::ProofOfStakeRewardsApi;
use mangata_types::assets::L1Asset;
use orml_traits::parameter_type_with_key;
use sp_runtime::{traits::ConvertBack, BuildStorage, Saturating};

pub(crate) type AccountId = u64;
pub(crate) type Amount = i128;
pub(crate) type Balance = u128;
pub(crate) type TokenId = u32;

pub mod consts {
	pub const MILLION: u128 = 1_000_000;
	pub const THOUSAND: u128 = 1_000;
	pub const ALICE: u64 = 2;
	pub const BOB: u64 = 3;
	pub const CHARLIE: u64 = 4;
	pub const CHAIN: crate::messages::Chain = crate::messages::Chain::Ethereum;
}

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System: frame_system,
		Tokens: orml_tokens,
		Rolldown: rolldown
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
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

	impl SequencerStakingProviderTrait<AccountId, Balance, messages::Chain> for SequencerStakingProviderApi {
		fn is_active_sequencer(chain: messages::Chain, sequencer: &AccountId) -> bool;
		fn is_active_sequencer_alias(chain: messages::Chain, sequencer: &AccountId, alias: &AccountId) -> bool;
		fn slash_sequencer<'a>(chain: messages::Chain, to_be_slashed: &AccountId, maybe_to_reward: Option<&'a AccountId>) -> DispatchResult;
		fn is_selected_sequencer(chain: messages::Chain, sequencer: &AccountId) -> bool;
		fn selected_sequencer(chain: messages::Chain) -> Option<AccountId>;
	}
}

mockall::mock! {
	pub AssetRegistryProviderApi {}
	impl AssetRegistryProviderTrait<TokenId> for AssetRegistryProviderApi {
		fn get_l1_asset_id(l1_asset: L1Asset) -> Option<TokenId>;
		fn create_l1_asset(l1_asset: L1Asset) -> Result<TokenId, DispatchError>;
	}
}

mockall::mock! {
	pub MaintenanceStatusProviderApi {}

	impl GetMaintenanceStatusTrait for MaintenanceStatusProviderApi {
		fn is_maintenance() -> bool;
		fn is_upgradable() -> bool;
	}

	impl SetMaintenanceModeOn for MaintenanceStatusProviderApi {
		fn trigger_maintanance_mode();
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

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"rolldown");
	pub const NativeCurrencyId: u32 = 0;
}

impl rolldown::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type SequencerStakingProvider = MockSequencerStakingProviderApi;
	type Tokens = orml_tokens::MultiTokenCurrencyAdapter<Test>;
	type AssetRegistryProvider = MockAssetRegistryProviderApi;
	type AddressConverter = DummyAddressConverter;
	type DisputePeriodLength = ConstU128<5>;
	type RequestsPerBlock = ConstU128<10>;
	type MaintenanceStatusProvider = MockMaintenanceStatusProviderApi;
	type ChainId = messages::Chain;
	type RightsMultiplier = ConstU128<1>;
	type AssetAddressConverter = crate::MultiEvmChainAddressConverter;
	type MerkleRootAutomaticBatchSize = ConstU128<10>;
	type MerkleRootAutomaticBatchPeriod = ConstU128<25>;
	type TreasuryPalletId = TreasuryPalletId;
	type NativeCurrencyId = NativeCurrencyId;
	type SequencerStakingRewards = ();
	type WithdrawFee = ConstU128<500>;
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
				Pallet::<Test>::new_sequencer_active(consts::CHAIN, s);
			}
		});

		Self { ext }
	}

	pub fn new_without_default_sequencers() -> Self {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		rolldown::GenesisConfig::<Test> { _phantom: Default::default() }
			.assimilate_storage(&mut t)
			.expect("Tokens storage can be assimilated");

		let ext = sp_io::TestExternalities::new(t);

		Self { ext }
	}

	pub fn single_sequencer(_seq: AccountId) -> Self {
		let t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		let ext = sp_io::TestExternalities::new(t);
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

	pub fn all_mocks() -> HashSet<Mocks> {
		[
			Mocks::IsActiveSequencer,
			Mocks::IsSelectedSequencer,
			Mocks::SelectedSequencer,
			Mocks::GetL1AssetId,
			Mocks::MaintenanceMode,
		]
		.iter()
		.cloned()
		.collect()
	}

	pub fn execute_with_default_mocks<R>(self, f: impl FnOnce() -> R) -> R {
		self.execute_with_mocks(Self::all_mocks(), f)
	}

	pub fn execute_without_mocks<R>(
		self,
		disabled: impl IntoIterator<Item = Mocks> + Clone,
		f: impl FnOnce() -> R,
	) -> R {
		let disabled: HashSet<Mocks> = disabled.into_iter().collect();
		let difference: HashSet<Mocks> = Self::all_mocks().difference(&disabled).cloned().collect();
		self.execute_with_mocks(difference, f)
	}

	pub fn execute_with_mocks<R>(mut self, mocks: HashSet<Mocks>, f: impl FnOnce() -> R) -> R {
		self.ext.execute_with(|| {
			let is_liquidity_token_mock =
				MockSequencerStakingProviderApi::is_active_sequencer_context();
			let is_selected_sequencer_mock =
				MockSequencerStakingProviderApi::is_selected_sequencer_context();
			let get_l1_asset_id_mock = MockAssetRegistryProviderApi::get_l1_asset_id_context();
			let is_maintenance_mock = MockMaintenanceStatusProviderApi::is_maintenance_context();
			let selected_sequencer_mock =
				MockSequencerStakingProviderApi::selected_sequencer_context();

			if mocks.contains(&Mocks::IsActiveSequencer) {
				is_liquidity_token_mock.expect().return_const(true);
			}

			if mocks.contains(&Mocks::IsSelectedSequencer) {
				is_selected_sequencer_mock.expect().return_const(true);
			}

			if mocks.contains(&Mocks::GetL1AssetId) {
				get_l1_asset_id_mock.expect().return_const(crate::tests::ETH_TOKEN_ADDRESS_MGX);
			}

			if mocks.contains(&Mocks::SelectedSequencer) {
				selected_sequencer_mock.expect().return_const(None);
			}

			if mocks.contains(&Mocks::MaintenanceMode) {
				is_maintenance_mock.expect().return_const(false);
			}

			f()
		})
	}
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum Mocks {
	IsActiveSequencer,
	IsSelectedSequencer,
	SelectedSequencer,
	GetL1AssetId,
	MaintenanceMode,
}

pub fn forward_to_next_block<T>()
where
	T: frame_system::Config,
	T: rolldown::Config,
{
	forward_to_block::<T>(frame_system::Pallet::<T>::block_number() + One::one());
}

pub fn forward_to_block<T>(n: BlockNumberFor<T>)
where
	T: frame_system::Config,
	T: rolldown::Config,
{
	while frame_system::Pallet::<T>::block_number() < n {
		rolldown::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
		frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
		let new_block_number =
			frame_system::Pallet::<T>::block_number().saturating_add(1u32.into());
		frame_system::Pallet::<T>::set_block_number(new_block_number);

		frame_system::Pallet::<T>::on_initialize(new_block_number);
		rolldown::Pallet::<T>::on_initialize(new_block_number);
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
