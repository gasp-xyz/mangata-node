// Copyright (C) 2020 Mangata team
use super::*;

use crate as sequencer_staking;
use core::convert::TryFrom;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{tokens::fungible::Mutate, Everything},
	PalletId,
};
use mangata_support::traits::{ComputeIssuance, GetIssuance};

use orml_traits::parameter_type_with_key;
use sp_runtime::{
	traits::AccountIdConversion, BuildStorage, Perbill, Percent, Permill, Saturating,
};

pub(crate) type AccountId = u64;
pub(crate) type Amount = i128;
pub(crate) type Balance = u128;
pub(crate) type TokenId = u32;
pub(crate) type ChainId = u32;

pub mod consts {
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

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

parameter_types! {
	pub const HistoryLimit: u32 = 10u32;

	pub const LiquidityMiningIssuanceVaultId: PalletId = PalletId(*b"py/lqmiv");
	pub LiquidityMiningIssuanceVault: AccountId = LiquidityMiningIssuanceVaultId::get().into_account_truncating();
	pub const StakingIssuanceVaultId: PalletId = PalletId(*b"py/stkiv");
	pub StakingIssuanceVault: AccountId = StakingIssuanceVaultId::get().into_account_truncating();
	pub const SequencerIssuanceVaultId: PalletId = PalletId(*b"py/seqiv");
	pub SequencerIssuanceVault: AccountId = SequencerIssuanceVaultId::get().into_account_truncating();

	pub const TotalCrowdloanAllocation: Balance = 0;
	pub const IssuanceCap: Balance = 4_000_000_000;
	pub const LinearIssuanceBlocks: u32 = 10_000u32;
	pub const LiquidityMiningSplit: Perbill = Perbill::from_parts(555555556);
	pub const StakingSplit: Perbill = Perbill::from_parts(344444444);
	pub const SequencerSplit: Perbill = Perbill::from_parts(100000000);
	pub const ImmediateTGEReleasePercent: Percent = Percent::from_percent(20);
	pub const TGEReleasePeriod: u32 = 5_256_000u32; // 2 years
	pub const TGEReleaseBegin: u32 = 100_800u32; // Two weeks into chain start
	pub const BlocksPerRound: u32 = 5;
	pub const TargetTge:u128 = 2_000_000_000u128;

}

pub struct MockIssuance;
impl ComputeIssuance for MockIssuance {
	fn initialize() {}
	fn compute_issuance(_n: u32) {
		let issuance = Self::get_sequencer_issuance(_n).unwrap();

		let _ = TokensOf::<Test>::mint_into(&SequencerIssuanceVault::get(), issuance.into());
	}
}

impl GetIssuance<Balance> for MockIssuance {
	fn get_all_issuance(_n: u32) -> Option<(Balance, Balance, Balance)> {
		unimplemented!()
	}
	fn get_liquidity_mining_issuance(_n: u32) -> Option<Balance> {
		unimplemented!()
	}
	fn get_staking_issuance(_n: u32) -> Option<Balance> {
		unimplemented!()
	}
	fn get_sequencer_issuance(_n: u32) -> Option<Balance> {
		let to_be_issued: Balance =
			IssuanceCap::get() - TargetTge::get() - TotalCrowdloanAllocation::get();
		let linear_issuance_sessions: u32 = LinearIssuanceBlocks::get() / BlocksPerRound::get();
		let linear_issuance_per_session = to_be_issued / linear_issuance_sessions as Balance;
		let issuance = SequencerSplit::get() * linear_issuance_per_session;
		Some(issuance)
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
	pub const DefaultPayoutLimit: u32 = 15;
	pub const RewardPaymentDelay: u32 = 2;
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
	type DefaultPayoutLimit = DefaultPayoutLimit;
	type SequencerIssuanceVault = SequencerIssuanceVault;
	type RewardPaymentDelay = RewardPaymentDelay;
	type Issuance = MockIssuance;
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

pub type TokensOf<Test> = <Test as crate::Config>::Currency;
