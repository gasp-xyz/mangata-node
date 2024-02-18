#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, Randomness,
		StorageInfo,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		IdentityFee, Weight,
	},
	StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{ConstFeeMultiplier, CurrencyAdapter, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

/// Import the template pallet.
pub use pallet_template;

pub type TokenId = u32;
pub type Balance = u128;
pub type Amount = i128;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::HeaderVer<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("rollup-chain"),
	impl_name: create_runtime_str!("rollup-chain"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 100,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time, with maximum proof size.
/// NOTE: reduced by half comparing to origin impl as we want to fill block only up to 50%
/// so there is room for new extrinsics in the next block
const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND, u64::MAX);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights = BlockWeights::builder()
		.base_block(weights::VerBlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = weights::VerExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * consts::MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(consts::MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				consts::MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * consts::MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// The block type for the runtime.
	type Block = Block;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;

	#[cfg(feature = "experimental")]
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

/// Configure the pallet-template in pallets/template.
impl pallet_template::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_template::weights::SubstrateWeight<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
onstruct_runtime!(
	pub enum Runtime
	{
		// System support stuff.
		System: frame_system = 0,
		Timestamp: pallet_timestamp = 2,
		ParachainInfo: parachain_info = 3,
		Utility: pallet_utility_mangata = 4,
		Proxy: pallet_proxy = 5,
		Maintenance: pallet_maintenance = 6,
		Rolldown: pallet_rolldown = 7,

		// Monetary stuff.
		Tokens: orml_tokens = 10,
		TransactionPayment: pallet_transaction_payment_mangata = 11,

		// Xyk stuff
		Xyk: pallet_xyk = 13,
		ProofOfStake: pallet_proof_of_stake = 14,

		// Fee Locks
		FeeLock: pallet_fee_lock = 15,

		// Vesting
		Vesting: pallet_vesting_mangata = 17,

		// Crowdloan
		Crowdloan: pallet_crowdloan_rewards = 18,

		// Issuance
		Issuance: pallet_issuance = 19,

		// MultiPurposeLiquidity
		MultiPurposeLiquidity: pallet_multipurpose_liquidity = 20,

		// Bootstrap
		Bootstrap: pallet_bootstrap = 21,

		// Collator support. The order of these 4 are important and shall not change.
		SequencerStaking: pallet_sequencer_staking = 29,
		Authorship: pallet_authorship = 30,
		ParachainStaking: parachain_staking = 31,
		Session: pallet_session = 32,
		Aura: pallet_aura = 33,

		AssetRegistry: orml_asset_registry = 53,

		// Governance stuff
		Treasury: pallet_treasury = 60,
		Sudo: pallet_sudo_mangata = 61,
		SudoOrigin: pallet_sudo_origin = 62,
		Council: pallet_collective_mangata::<Instance1> = 63,
		Identity: pallet_identity = 64,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::HeaderVer<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment_mangata::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_session, Session]
		[pallet_timestamp, Timestamp]
		[orml_asset_registry, AssetRegistry]
		[orml_tokens, Tokens]
		[parachain_staking, ParachainStaking]
		[pallet_xyk, Xyk]
		[pallet_treasury, Treasury]
		[pallet_collective_mangata, Council]
		[pallet_bootstrap, Bootstrap]
		[pallet_crowdloan_rewards, Crowdloan]
		[pallet_utility_mangata, Utility]
		[pallet_vesting_mangata, Vesting]
		[pallet_issuance, Issuance]
		[pallet_multipurpose_liquidity, MultiPurposeLiquidity]
		[pallet_fee_lock, FeeLock]
		[pallet_proof_of_stake, ProofOfStake]
	);
}


use codec::alloc::string::String;
use sp_runtime::generic::ExtendedCall;
impl ExtendedCall for RuntimeCall {
	fn context(&self) -> Option<(String, String)> {
		match self {
			RuntimeCall::Xyk(pallet_xyk::Call::sell_asset {
				sold_asset_id,
				sold_asset_amount,
				bought_asset_id,
				min_amount_out,
				..
			}) => {
				let mut buffer = String::new();
				let _ = write!(&mut buffer, "sold_asset_id: {sold_asset_id}\n");
				let _ = write!(&mut buffer, "sold_asset_amount: {sold_asset_amount}\n");
				let _ = write!(&mut buffer, "bought_asset_id: {bought_asset_id}\n");
				let _ = write!(&mut buffer, "min_amount_out: {min_amount_out}\n");
				Some(("xyk::sell_asset".to_string(), buffer))
			},
			RuntimeCall::Xyk(pallet_xyk::Call::buy_asset {
				sold_asset_id,
				bought_asset_amount,
				bought_asset_id,
				max_amount_in,
				..
			}) => {
				let mut buffer = String::new();
				let _ = write!(&mut buffer, "sold_asset_id: {sold_asset_id}\n");
				let _ = write!(&mut buffer, "bought_asset_amount: {bought_asset_amount}\n");
				let _ = write!(&mut buffer, "bought_asset_id: {bought_asset_id}\n");
				let _ = write!(&mut buffer, "max_amount_in: {max_amount_in}\n");
				Some(("xyk::buy_asset".to_string(), buffer))
			},
			RuntimeCall::Tokens(orml_tokens::Call::transfer { dest, currency_id, amount }) => {
				let mut buffer = String::new();
				let _ = write!(&mut buffer, "dest: {dest:?}\n");
				let _ = write!(&mut buffer, "currency_id: {currency_id}\n");
				let _ = write!(&mut buffer, "amount: {amount}\n");
				Some(("orml_tokens::transfer".to_string(), buffer))
			},
			_ => Some(("todo".to_string(), "todo".to_string())),
		}
	}
}


impl_runtime_apis! {
	impl metamask_signature_runtime_api::MetamaskSignatureRuntimeApi<Block> for Runtime {
		fn get_eip712_sign_data(call: Vec<u8>) -> String{
			if let Ok(extrinsic) = UncheckedExtrinsic::decode(& mut call.as_ref()) {
				if let Some((method, params)) = extrinsic.function.context() {
					metamask_signature_runtime_api::eip712_payload(method, params)
				}else{
					Default::default()
				}
			}else{
				Default::default()
			}
		}
	}

	impl rolldown_runtime_api::RolldownRuntimeApi<Block> for Runtime {
		fn get_pending_updates_hash() -> sp_core::H256 {
			pallet_rolldown::Pallet::<Runtime>::pending_updates_proof()
		}
		fn get_pending_updates() -> Vec<u8> {
			pallet_rolldown::Pallet::<Runtime>::l2_update_encoded()
		}
	}

	impl proof_of_stake_runtime_api::ProofOfStakeApi<Block, Balance , TokenId,  AccountId> for Runtime{
		fn calculate_native_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance{
			pallet_proof_of_stake::Pallet::<Runtime>::calculate_native_rewards_amount(user, liquidity_asset_id)
				.unwrap_or_default()
		}

		fn calculate_3rdparty_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
			reward_asset_id: TokenId,
		) -> Balance{
			pallet_proof_of_stake::Pallet::<Runtime>::calculate_3rdparty_rewards_amount(user, liquidity_asset_id, reward_asset_id)
				.unwrap_or_default()
		}

		fn calculate_3rdparty_rewards_all(
			user: AccountId,
		) -> Vec<(TokenId, TokenId, Balance)>{
			pallet_proof_of_stake::Pallet::<Runtime>::calculate_3rdparty_rewards_all(user)
		}
	}

	impl ver_api::VerApi<Block> for Runtime {
		fn get_signer(
			tx: <Block as BlockT>::Extrinsic,
		) -> Option<(sp_runtime::AccountId32, u32)> {
			if let Some(sig) = tx.signature.clone(){
				let nonce: frame_system::CheckNonce<_> = sig.2.4;
				<Runtime as frame_system::Config>::Lookup::lookup(sig.0)
					.ok()
					.and_then(|addr| Some((addr, nonce.0)))
			}else{
				None
			}
		}

		fn is_storage_migration_scheduled() -> bool{
			System::read_events_no_consensus()
				.any(|record|
					matches!(record.event,
						RuntimeEvent::ParachainSystem( cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionApplied{relay_chain_block_num: _})))
		}

		fn store_seed(seed: sp_core::H256){
			// initialize has been called already so we can fetch number from the storage
			System::set_block_seed(&seed);
		}

		fn get_previous_block_txs() -> Vec<Vec<u8>> {
			System::get_previous_blocks_txs()
		}

		fn pop_txs(count: u64) -> Vec<Vec<u8>> {
			System::pop_txs(count as usize)
		}

		fn create_enqueue_txs_inherent(txs: Vec<<Block as BlockT>::Extrinsic>) -> <Block as BlockT>::Extrinsic {
			for t in txs.iter() {
				if let Some((_, _, extra)) = &t.signature {
					let _ = extra.additional_signed();
				}
			}
			UncheckedExtrinsic::new_unsigned(
					RuntimeCall::System(frame_system::Call::enqueue_txs{txs:
						txs.into_iter()
							.map(|tx| (
									tx.signature.clone().and_then(|sig|
										<Runtime as frame_system::Config>::Lookup::lookup(sig.0).ok()
									),
									tx.encode())
							).collect()}))
		}

		fn can_enqueue_txs() -> bool {
			System::can_enqueue_txs()
		}

		fn start_prevalidation() {
			System::set_prevalidation()
		}

		fn account_extrinsic_dispatch_weight(consumed: ver_api::ConsumedWeight, tx: <Block as BlockT>::Extrinsic) -> Result<ver_api::ConsumedWeight, ()> {
			let info = tx.get_dispatch_info();
			let maximum_weight = <Runtime as frame_system::Config>::BlockWeights::get();
			frame_system::calculate_consumed_weight::<RuntimeCall>(maximum_weight, consumed, &info)
			.or(Err(()))
		}

	}

	impl ver_api::VerNonceApi<Block, AccountId> for Runtime {
		fn enqueued_txs_count(acc: AccountId) -> u64 {

			System::enqueued_txs_count(&acc) as u64
		}
	}

	impl xyk_runtime_api::XykApi<Block, Balance, TokenId, AccountId> for Runtime {
		fn calculate_sell_price(
			input_reserve: Balance,
			output_reserve: Balance,
			sell_amount: Balance
		) -> Balance {
			Xyk::calculate_sell_price(input_reserve, output_reserve, sell_amount)
				.map_err(|e|
						 {
							 log::warn!(target:"xyk", "rpc 'XYK::calculate_sell_price' error: '{:?}', returning default value instead", e);
							 e
						 }
						).unwrap_or_default()
		}

		fn calculate_buy_price(
			input_reserve: Balance,
			output_reserve: Balance,
			buy_amount: Balance
		) -> Balance {
				Xyk::calculate_buy_price(input_reserve, output_reserve, buy_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()

		}

		fn calculate_sell_price_id(
			sold_token_id: TokenId,
			bought_token_id: TokenId,
			sell_amount: Balance
		) -> Balance {
				 Xyk::calculate_sell_price_id(sold_token_id, bought_token_id, sell_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_sell_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn calculate_buy_price_id(
			sold_token_id: TokenId,
			bought_token_id: TokenId,
			buy_amount: Balance
		) -> Balance {
				Xyk::calculate_buy_price_id(sold_token_id, bought_token_id, buy_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn get_burn_amount(
			first_asset_id: TokenId,
			second_asset_id: TokenId,
			liquidity_asset_amount: Balance
		) -> (Balance, Balance) {
			Xyk::get_burn_amount(first_asset_id, second_asset_id, liquidity_asset_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn get_max_instant_burn_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance {
			Xyk::get_max_instant_burn_amount(&user, liquidity_asset_id)
		}

		fn get_max_instant_unreserve_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance {
			 Xyk::get_max_instant_unreserve_amount(&user, liquidity_asset_id)
		}

		fn calculate_rewards_amount(
			user: AccountId,
			liquidity_asset_id: TokenId,
		) -> Balance {
			ProofOfStake::calculate_rewards_amount(user, liquidity_asset_id)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_buy_price_id' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn calculate_balanced_sell_amount(
			total_amount: Balance,
			reserve_amount: Balance,
		) -> Balance {
				 Xyk::calculate_balanced_sell_amount(total_amount, reserve_amount)
					.map_err(|e|
						{
							log::warn!(target:"xyk", "rpc 'XYK::calculate_balanced_sell_amount' error: '{:?}', returning default value instead", e);
							e
						}
					).unwrap_or_default()
		}

		fn get_liq_tokens_for_trading() -> Vec<TokenId> {
			Xyk::get_liq_tokens_for_trading()
				.map_err(|e|
					{
						log::warn!(target:"xyk", "rpc 'XYK::get_liq_tokens_for_trading' error: '{:?}', returning default value instead", e);
						e
					}
				).unwrap_or_default()
		}

		fn is_sell_asset_lock_free(
			path: Vec<TokenId>,
			input_amount: Balance,
			) -> Option<bool>{
			match (path.len(), pallet_fee_lock::FeeLockMetadata::<Runtime>::get()) {
				(length, _) if length < 2 => {
					None
				}
				(2, Some(feelock)) => {
					let input = path.get(0)?;
					let output = path.get(1)?;
					let output_amount = Xyk::calculate_sell_price_id(*input, *output, input_amount).ok()?;
					Some(
					FeeHelpers::<
								Runtime,
								orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
								ToAuthor<Runtime>,
								OnChargeTransactionHandler<Runtime>,
								FeeLock,
								>::is_high_value_swap(&feelock, *input, input_amount)
									||
					FeeHelpers::<
								Runtime,
								orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
								ToAuthor<Runtime>,
								OnChargeTransactionHandler<Runtime>,
								FeeLock,
								>::is_high_value_swap(&feelock, *output, output_amount)
								)
				}
				(_,  None) => {
					Some(false)
				}
				(_,  Some(_)) => {
					Some(true)
				}
			}
		}

		fn is_buy_asset_lock_free(
			path: Vec<TokenId>,
			input_amount: Balance,
			) -> Option<bool>{
			match (path.len(), pallet_fee_lock::FeeLockMetadata::<Runtime>::get()) {
				(length, _) if length < 2 => {
					None
				}
				(2, Some(feelock)) => {
					let input = path.get(0)?;
					let output = path.get(1)?;
					let output_amount = Xyk::calculate_buy_price_id(*input, *output, input_amount).ok()?;
					Some(
					FeeHelpers::<
								Runtime,
								orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
								ToAuthor<Runtime>,
								OnChargeTransactionHandler<Runtime>,
								FeeLock,
								>::is_high_value_swap(&feelock, *input, input_amount)
									||
					FeeHelpers::<
								Runtime,
								orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
								ToAuthor<Runtime>,
								OnChargeTransactionHandler<Runtime>,
								FeeLock,
								>::is_high_value_swap(&feelock, *output, output_amount)
								)
				}
				(_, None) => {
					Some(false)
				}
				(_, Some(_)) => {
					Some(true)
				}
			}
		}

		fn get_tradeable_tokens() -> Vec<RpcAssetMetadata<TokenId>> {
			orml_asset_registry::Metadata::<Runtime>::iter()
			.filter_map(|(token_id, metadata)| {
				if !metadata.name.is_empty()
					&& !metadata.symbol.is_empty()
					&& metadata.additional.xyk.as_ref().map_or(true, |xyk| !xyk.operations_disabled)
				{
					let rpc_metadata = RpcAssetMetadata {
						token_id: token_id,
						decimals: metadata.decimals,
						name: metadata.name.to_vec(),
						symbol: metadata.symbol.to_vec(),
					};
					Some(rpc_metadata)
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			let key = Authorship::author(&block).expect("Block should have an author aura digest");
			Executive::execute_block_ver_impl(block, key);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			_authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_mangata_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_mangata_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_mangata::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_mangata_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment_mangata::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_mangata::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// TODO: Maybe checks should be overridden with `frame_try_runtime::UpgradeCheckSelect::All`
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, cfg::frame_system::RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			log::info!(
				target: "node-runtime",
				"try-runtime: executing block {:?} / root checks: {:?} / try-state-select: {:?}",
				block.header.hash(),
				state_root_check,
				select,
			);
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}
}
