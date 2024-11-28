use super::*;
use crate as market;

use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{
		tokens::currency::MultiTokenCurrency, ConstU128, ConstU32, Contains, ExistenceRequirement,
		GetDefault, Nothing, WithdrawReasons,
	},
	PalletId,
};
use frame_system as system;
use mangata_support::traits::{ActivationReservesProviderTrait, ComputeIssuance};
use mangata_types::assets::L1Asset;
use sp_runtime::{traits::AccountIdConversion, BuildStorage};
use std::convert::TryFrom;

pub use orml_tokens::{MultiTokenCurrencyAdapter, MultiTokenCurrencyExtended};
use orml_traits::{asset_registry::AssetMetadata, parameter_type_with_key};

use pallet_xyk::AssetMetadataMutationTrait;

pub(crate) type AccountId = u128;
pub(crate) type Amount = i128;
pub(crate) type Balance = u128;
pub(crate) type TokenId = u32;

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System: frame_system,
		Tokens: orml_tokens,
		Vesting: pallet_vesting_mangata,
		StableSwap: pallet_stable_swap,
		Xyk: pallet_xyk,
		Market: market,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = sp_runtime::traits::IdentityLookup<AccountId>;
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
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub const MaxLocks: u32 = 50;
	pub const NativeCurrencyId: u32 = 0_u32;
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
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 0;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
	WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting_mangata::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Tokens = MultiTokenCurrencyAdapter<Test>;
	type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting_mangata::weights::SubstrateWeight<Test>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	type BlockNumberProvider = System;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

impl pallet_stable_swap::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = MultiTokenCurrencyAdapter<Test>;
	type Balance = Balance;
	type HigherPrecisionBalance = sp_core::U256;
	type CurrencyId = TokenId;
	type TreasuryPalletId = TreasuryPalletId;
	type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
	type MarketTotalFee = ConstU128<30_000_000>;
	type MarketTreasuryFeePart = ConstU128<3_333_333_334>;
	type MarketBnBFeePart = ConstU128<5_000_000_000>;
	type MaxApmCoeff = ConstU128<1_000_000>;
	type DefaultApmCoeff = ConstU128<1_000>;
	type MaxAssetsInPool = ConstU32<8>;
	type WeightInfo = ();
}

#[cfg(not(feature = "runtime-benchmarks"))]
mod mocks {
	use super::*;

	mockall::mock! {
		pub MaintenanceStatusProviderApi {}
		impl GetMaintenanceStatusTrait for MaintenanceStatusProviderApi {
			fn is_maintenance() -> bool;
			fn is_upgradable() -> bool;
		}
	}

	mockall::mock! {
		pub ActivationReservesApi {}
		impl ActivationReservesProviderTrait<AccountId, Balance, TokenId> for ActivationReservesApi {
			fn get_max_instant_unreserve_amount(token_id: TokenId, account_id: &AccountId) -> Balance;

			fn can_activate(
				token_id: TokenId,
				account_id: &AccountId,
				amount: Balance,
				use_balance_from: Option<ActivateKind>,
			) -> bool;

			fn activate(
				token_id: TokenId,
				account_id: &AccountId,
				amount: Balance,
				use_balance_from: Option<ActivateKind>,
			) -> DispatchResult;

			fn deactivate(token_id: TokenId, account_id: &AccountId, amount: Balance) -> Balance;
		}
	}

	mockall::mock! {
		pub RewardsApi {}

		impl ProofOfStakeRewardsApi<AccountId, Balance, TokenId> for RewardsApi {

			fn enable(liquidity_token_id: TokenId, weight: u8);

			fn disable(liquidity_token_id: TokenId);

			fn is_enabled(
				liquidity_token_id: TokenId,
			) -> bool;

			fn claim_rewards_all(
				sender: AccountId,
				liquidity_token_id: TokenId,
			) -> Result<Balance, DispatchError>;

			fn activate_liquidity(
				sender: AccountId,
				liquidity_token_id: TokenId,
				amount: Balance,
				use_balance_from: Option<ActivateKind>,
			) -> DispatchResult;

			fn deactivate_liquidity(
				sender: AccountId,
				liquidity_token_id: TokenId,
				amount: Balance,
			) -> DispatchResult;

			fn calculate_rewards_amount(
				user: AccountId,
				liquidity_asset_id: TokenId,
			) -> Result<Balance, DispatchError>;

			fn rewards_period() -> u32;
		}
	}

	mockall::mock! {
		pub AssetRegApi {}

		impl AssetRegistryProviderTrait<TokenId> for AssetRegApi {
			fn get_l1_asset_id(l1_asset: L1Asset) -> Option<TokenId>;
			fn create_l1_asset(l1_asset: L1Asset) -> Result<TokenId, DispatchError>;
			fn get_asset_l1_id(asset_id: TokenId) -> Option<L1Asset>;
			fn create_pool_asset(
				lp_asset: TokenId,
				asset_1: TokenId,
				asset_2: TokenId,
			) -> DispatchResult;
		}

		impl orml_traits::asset_registry::Inspect for AssetRegApi {
			type AssetId = TokenId;
			type Balance = Balance;
			type CustomMetadata = ();
			type StringLimit = ConstU32<10>;

			fn metadata(asset_id: &TokenId) -> Option<AssetMetadata<Balance, (), ConstU32<10>>>;
		}

		impl AssetMetadataMutationTrait<TokenId> for AssetRegApi {
			fn set_asset_info(
				asset: TokenId,
				name: Vec<u8>,
				symbol: Vec<u8>,
				decimals: u32,
			) -> DispatchResult;
		}
	}

	mockall::mock! {
		pub Issuance {}

		impl ComputeIssuance for Issuance {
			fn initialize() {}
			fn compute_issuance(n: u32);
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
mod mocks {
	use super::*;

	pub struct MockMaintenanceStatusProviderApi;

	impl GetMaintenanceStatusTrait for MockMaintenanceStatusProviderApi {
		fn is_maintenance() -> bool {
			false
		}

		fn is_upgradable() -> bool {
			unimplemented!()
		}
	}
	pub struct MockActivationReservesApi;

	impl ActivationReservesProviderTrait<AccountId, Balance, TokenId> for MockActivationReservesApi {
		fn get_max_instant_unreserve_amount(_: TokenId, _: &AccountId) -> Balance {
			Zero::zero()
		}

		fn can_activate(_: TokenId, _: &AccountId, _: Balance, _: Option<ActivateKind>) -> bool {
			unimplemented!()
		}

		fn activate(
			_: TokenId,
			_: &AccountId,
			_: Balance,
			_: Option<ActivateKind>,
		) -> DispatchResult {
			unimplemented!()
		}

		fn deactivate(_: TokenId, _: &AccountId, _: Balance) -> Balance {
			unimplemented!()
		}
	}

	pub struct MockRewardsApi;

	impl ProofOfStakeRewardsApi<AccountId, Balance, TokenId> for MockRewardsApi {
		fn enable(_: TokenId, _: u8) {}

		fn disable(_: TokenId) {
			unimplemented!()
		}

		fn is_enabled(_: TokenId) -> bool {
			true
		}

		fn claim_rewards_all(_: AccountId, _: TokenId) -> Result<Balance, DispatchError> {
			unimplemented!()
		}

		fn activate_liquidity(
			_: AccountId,
			_: TokenId,
			_: Balance,
			_: Option<ActivateKind>,
		) -> DispatchResult {
			Ok(()).into()
		}

		fn deactivate_liquidity(_: AccountId, _: TokenId, _: Balance) -> DispatchResult {
			Ok(()).into()
		}

		fn calculate_rewards_amount(_: AccountId, _: TokenId) -> Result<Balance, DispatchError> {
			unimplemented!()
		}

		fn rewards_period() -> u32 {
			10
		}
	}

	pub struct MockAssetRegApi;

	impl AssetRegistryProviderTrait<TokenId> for MockAssetRegApi {
		fn get_l1_asset_id(_: L1Asset) -> Option<TokenId> {
			unimplemented!()
		}
		fn create_l1_asset(_: L1Asset) -> Result<TokenId, DispatchError> {
			unimplemented!()
		}
		fn get_asset_l1_id(_: TokenId) -> Option<L1Asset> {
			unimplemented!()
		}
		fn create_pool_asset(_: TokenId, _: TokenId, _: TokenId) -> DispatchResult {
			Ok(()).into()
		}
	}

	impl orml_traits::asset_registry::Inspect for MockAssetRegApi {
		type AssetId = TokenId;
		type Balance = Balance;
		type CustomMetadata = ();
		type StringLimit = ConstU32<10>;

		fn metadata(_: &TokenId) -> Option<AssetMetadata<Balance, (), ConstU32<10>>> {
			Some(AssetMetadata {
				decimals: 18,
				name: BoundedVec::new(),
				symbol: BoundedVec::new(),
				existential_deposit: Zero::zero(),
				additional: (),
			})
		}
	}

	impl AssetMetadataMutationTrait<TokenId> for MockAssetRegApi {
		fn set_asset_info(_: TokenId, _: Vec<u8>, _: Vec<u8>, _: u32) -> DispatchResult {
			Ok(()).into()
		}
	}

	pub struct MockIssuance;

	impl ComputeIssuance for MockIssuance {
		fn initialize() {
			unimplemented!()
		}
		fn compute_issuance(_: u32) {}
	}
}

impl pallet_xyk::XykBenchmarkingConfig for Test {}

impl pallet_xyk::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaintenanceStatusProvider = mocks::MockMaintenanceStatusProviderApi;
	type ActivationReservesProvider = mocks::MockActivationReservesApi;
	type Currency = MultiTokenCurrencyAdapter<Test>;
	type NativeCurrencyId = NativeCurrencyId;
	type TreasuryPalletId = TreasuryPalletId;
	type BnbTreasurySubAccDerive = BnbTreasurySubAccDerive;
	type PoolFeePercentage = ConstU128<20>;
	type TreasuryFeePercentage = ConstU128<5>;
	type BuyAndBurnFeePercentage = ConstU128<5>;
	type LiquidityMiningRewards = mocks::MockRewardsApi;
	type WeightInfo = ();
	type VestingProvider = Vesting;
	type DisallowedPools = Nothing;
	type DisabledTokens = Nothing;
	type AssetMetadataMutation = mocks::MockAssetRegApi;
	type FeeLockWeight = ();
}

impl market::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = MultiTokenCurrencyAdapter<Test>;
	type Balance = Balance;
	type CurrencyId = TokenId;
	type NativeCurrencyId = NativeCurrencyId;
	type Xyk = Xyk;
	type StableSwap = StableSwap;
	type Rewards = mocks::MockRewardsApi;
	type Vesting = Vesting;
	type AssetRegistry = mocks::MockAssetRegApi;
	type DisabledTokens = Nothing;
	type DisallowedPools = Nothing;
	type MaintenanceStatusProvider = mocks::MockMaintenanceStatusProviderApi;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type ComputeIssuance = mocks::MockIssuance;
	type NontransferableTokens = Nothing;
	type FoundationAccountsProvider = GetDefault;
	type ArbitrageBot = Nothing;
}

impl<T: Config> Pallet<T>
where
	<T as pallet::Config>::Currency:
		MultiTokenCurrencyExtended<AccountId, CurrencyId = TokenId, Balance = Balance>,
{
	pub fn balance(id: TokenId, who: AccountId) -> Balance {
		<T as Config>::Currency::free_balance(id.into(), &who).into()
	}
	pub fn total_supply(id: TokenId) -> Balance {
		<T as Config>::Currency::total_issuance(id.into()).into()
	}
	pub fn transfer(
		currency_id: TokenId,
		source: AccountId,
		dest: AccountId,
		value: Balance,
	) -> DispatchResult {
		<T as Config>::Currency::transfer(
			currency_id,
			&source,
			&dest,
			value,
			ExistenceRequirement::KeepAlive,
		)
	}
	pub fn create_new_token(who: &AccountId, amount: Balance) -> TokenId {
		<T as Config>::Currency::create(who, amount).expect("Token creation failed")
	}

	pub fn mint_token(token_id: TokenId, who: &AccountId, amount: Balance) {
		<T as Config>::Currency::mint(token_id, who, amount).expect("Token minting failed")
	}
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities =
		system::GenesisConfig::<Test>::default().build_storage().unwrap().into();

	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

pub(crate) fn events() -> Vec<pallet::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Market(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>()
}

/// Compares the system events with passed in events
/// Prints highlighted diff iff assert_eq fails
#[macro_export]
macro_rules! assert_eq_events {
	($events:expr) => {
		match &$events {
			e => similar_asserts::assert_eq!(*e, $crate::mock::events()),
		}
	};
}

/// Panics if an event is not found in the system log of events
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
