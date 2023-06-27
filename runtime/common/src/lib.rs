#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	construct_runtime,
	dispatch::{DispatchClass, DispatchResult},
	ensure, parameter_types,
	traits::{
		tokens::currency::{MultiTokenCurrency, MultiTokenImbalanceWithZeroTrait},
		Contains, EnsureOrigin, EnsureOriginWithArg, Everything, ExistenceRequirement, Get,
		Imbalance, InstanceFilter, WithdrawReasons,
	},
	unsigned::TransactionValidityError,
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		ConstantMultiplier, Weight,
	},
	PalletId,
};
#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot,
};
pub use orml_tokens;
use orml_tokens::MultiTokenCurrencyExtended;
use orml_traits::{
	asset_registry::{AssetMetadata, AssetProcessor},
	parameter_type_with_key,
};
pub use pallet_sudo_mangata;
use pallet_transaction_payment_mangata::{ConstFeeMultiplier, Multiplier, OnChargeTransaction};
use pallet_vesting_mangata_rpc_runtime_api::VestingInfosWithLockedAt;
// Polkadot Imports
pub use polkadot_runtime_common::BlockHashCount;
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, Convert, ConvertInto,
		DispatchInfoOf, PostDispatchInfoOf, Saturating, StaticLookup, Zero,
	},
	transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchError, FixedPointNumber, Percent, RuntimeDebug,
};
pub use sp_runtime::{MultiAddress, Perbill, Permill};
use sp_std::{
	cmp::Ordering,
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	prelude::*,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;
pub use xcm::{latest::prelude::*, VersionedMultiLocation};

pub use constants::{fee::*, parachains::*};
pub use currency::*;
use mangata_support::traits::{
	AssetRegistryApi, FeeLockTriggerTrait, PreValidateSwaps, ProofOfStakeRewardsApi,
};
pub use mangata_types::{
	assets::{CustomMetadata, XcmMetadata, XykMetadata},
	AccountId, Address, Amount, Balance, BlockNumber, Hash, Index, Signature, TokenId,
};
pub use pallet_issuance::IssuanceInfo;
pub use pallet_sudo_origin;
pub use pallet_xyk;
// XCM Imports
use pallet_xyk::AssetMetadataMutationTrait;
use xyk_runtime_api::{RpcAmountsResult, XYKRpcResult};

pub mod constants;
mod weights;
// pub mod xcm_config;

pub mod currency {
	use super::Balance;

	pub const MILLICENTS: Balance = CENTS / 1000;
	pub const CENTS: Balance = DOLLARS / 100; // assume this is worth about a cent.
	pub const DOLLARS: Balance = super::consts::UNIT;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 5000 * DOLLARS + (bytes as Balance) * 60 * CENTS
	}
}

pub mod tokens {
	use super::*;
	pub const MGX_TOKEN_ID: TokenId = 0;
	pub const RELAY_TOKEN_ID: TokenId = 4;
	pub const TUR_TOKEN_ID: TokenId = 7;
	parameter_types! {
		pub const MgxTokenId: TokenId = MGX_TOKEN_ID;
		pub const RelayTokenId: TokenId = RELAY_TOKEN_ID;
		pub const TurTokenId: TokenId = TUR_TOKEN_ID;
	}
}



pub mod runtime_types {
	use super::*;

	pub type SignedExtra<Runtime> = (
		frame_system::CheckSpecVersion<Runtime>,
		frame_system::CheckTxVersion<Runtime>,
		frame_system::CheckGenesis<Runtime>,
		frame_system::CheckEra<Runtime>,
		frame_system::CheckNonce<Runtime>,
		frame_system::CheckWeight<Runtime>,
		pallet_transaction_payment_mangata::ChargeTransactionPayment<Runtime>,
	);

	pub type SignedPayload<Runtime, RuntimeCall> = generic::SignedPayload<RuntimeCall, SignedExtra<Runtime>>;
	pub type UncheckedExtrinsic<Runtime, RuntimeCall> = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra<Runtime>>;
	pub type CheckedExtrinsic<Runtime, RuntimeCall> = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra<Runtime>>;
	pub type Header = generic::HeaderVer<BlockNumber, BlakeTwo256>;
	pub type Block<Runtime, RuntimeCall> = generic::Block<Header, UncheckedExtrinsic<Runtime, RuntimeCall>>;
	pub type SignedBlock<Runtime, RuntimeCall> = generic::SignedBlock<Block<Runtime, RuntimeCall>>;
	pub type BlockId<Runtime, RuntimeCall> = generic::BlockId<Block<Runtime, RuntimeCall>>;

	pub type OpaqueBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>;
	pub type OpaqueBlockId = generic::BlockId<OpaqueBlock>;


}


pub mod consts {
	use super::*;
	/// This determines the average expected block time that we are targeting.
	/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
	/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
	/// up by `pallet_aura` to implement `fn slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: u64 = 12000;


	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	// Unit = the base number of indivisible units for balance
	pub const UNIT: Balance = 1_000_000_000_000_000_000;
	pub const MILLIUNIT: Balance = 1_000_000_000_000_000;
	pub const MICROUNIT: Balance = 1_000_000_000_000;


	/// We allow for 0.5 of a second of compute with a 12 second average block time.
	/// NOTE: reduced by half comparing to origin impl as we want to fill block only up to 50%
	/// so there is room for new extrinsics in the next block
	pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
		WEIGHT_REF_TIME_PER_SECOND.saturating_div(4),
		polkadot_primitives::v2::MAX_POV_SIZE as u64,
		);

	/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
	pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

}

pub mod config {
	use super::*;

	pub type TreasuryPalletIdOf<T> = <T as ::pallet_treasury::Config>::PalletId;


	pub struct TreasuryAccountIdOf<T: ::pallet_treasury::Config>(PhantomData<T>);
	impl<T : ::pallet_treasury::Config> Get<AccountId> for TreasuryAccountIdOf<T>{
		fn get() -> AccountId {
			TreasuryPalletIdOf::<T>::get().into_account_truncating()
		}
	}

	pub type ExistentialDepositsOf<T> = <T as ::orml_tokens::Config>::ExistentialDeposits;
	pub type MaxLocksOf<T> = <T as ::orml_tokens::Config>::MaxLocks;
	pub type SessionLenghtOf<T> = <T as ::parachain_staking::Config>::BlocksPerRound;

pub mod frame_system{
	use super::*;

	/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
	/// used to limit the maximal weight of a single extrinsic.
	pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

	/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
	/// `Operational` extrinsics.
	pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

	pub type MaxConsumers = frame_support::traits::ConstU32<16>;

parameter_types! {

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
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
	pub const SS58Prefix: u16 = 42;
}


}

pub mod pallet_timestamp{
	use super::*;

	// NOTE: Currently it is not possible to change the slot duration after the chain has started.
	//       Attempting to do so will brick block production.
	parameter_types! {
		pub const MinimumPeriod: u64 = consts::MILLISECS_PER_BLOCK / 2;
	}
}

pub mod pallet_treasury {
	use super::*;
		parameter_types! {
		pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
		}

	parameter_types! {
		pub const ProposalBond: Permill = Permill::from_percent(5);
		pub const ProposalBondMinimum: Balance = 1 * currency::DOLLARS;
		pub const ProposalBondMaximum: Option<Balance> = None;
		pub const SpendPeriod: BlockNumber = 1 * consts::DAYS;
		pub const Burn: Permill = Permill::from_percent(0);
		pub const MaxApprovals: u32 = 100;
	}

}

pub mod orml_tokens {
	use super::*;
	parameter_types! {
		pub const MaxLocks: u32 = 50;
	}

	parameter_type_with_key! {
		pub ExistentialDeposits: |_currency_id: TokenId| -> Balance {
			0
		};
	}

	pub struct DustRemovalWhitelist<T: Get<AccountId>>(PhantomData<T>);
	impl<T : Get<AccountId>> Contains<AccountId> for DustRemovalWhitelist<T> {
		fn contains(a: &AccountId) -> bool {
			*a == T::get()
		}
	}

	pub type ReserveIdentifier = [u8; 8];

}

pub mod pallet_xyk {
	use codec::EncodeLike;

	use super::*;
	parameter_types! {
		pub const BnbTreasurySubAccDerive: [u8; 4] = *b"bnbt";
	}
	pub type PoolFeePercentage = frame_support::traits::ConstU128<20>;
	pub type TreasuryFeePercentage = frame_support::traits::ConstU128<5>;
	pub type BuyAndBurnFeePercentage = frame_support::traits::ConstU128<5>;

	pub struct TestTokensFilter;
	impl Contains<TokenId> for TestTokensFilter {
		fn contains(token_id: &TokenId) -> bool {
			// we dont want to allow doing anything with dummy assets previously
			// used for testing
			*token_id == 2 || *token_id == 3
		}
	}

	pub struct AssetRegisterFilter<Runtime>(PhantomData<Runtime>);
	impl<T> Contains<TokenId> for AssetRegisterFilter<T> where
		T : ::orml_asset_registry::Config<CustomMetadata=CustomMetadata, AssetId=TokenId, Balance=Balance>,
		{
			fn contains(t: &TokenId) -> bool {
				let meta: Option<_> = ::orml_asset_registry::Metadata::<T>::get(*t);
				if let Some(xyk) = meta.and_then(|m| m.additional.xyk) {
					return xyk.operations_disabled
				}
				return false
			}
		}

pub struct AssetMetadataMutation<Runtime>(PhantomData<Runtime>);

impl<T> AssetMetadataMutationTrait for AssetMetadataMutation<T> where
		T : ::orml_asset_registry::Config<CustomMetadata=CustomMetadata, AssetId=TokenId, Balance=Balance>,
{
	fn set_asset_info(
		asset: TokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u32,
	) -> DispatchResult {
		let metadata = AssetMetadata {
			name,
			symbol,
			decimals,
			existential_deposit: Default::default(),
			additional: Default::default(),
			location: None,
		};
		orml_asset_registry::Pallet::<T>::do_register_asset_without_asset_processor(
			metadata, asset,
		)?;
		Ok(())
	}
}

}

pub mod pallet_bootstrap {
	use super::*;

	parameter_types! {
		pub const BootstrapUpdateBuffer: BlockNumber = 300;
		pub const DefaultBootstrapPromotedPoolWeight: u8 = 0u8;
		pub const ClearStorageLimit: u32 = 100u32;
	}

	pub struct EnableAssetPoolApi<Runtime>(PhantomData<Runtime>);
	impl<T> AssetRegistryApi for EnableAssetPoolApi<T> where
		T : ::orml_asset_registry::Config<CustomMetadata=CustomMetadata, AssetId=TokenId, Balance=Balance>,
	{
		fn enable_pool_creation(assets: (TokenId, TokenId)) -> bool {
			for &asset in [assets.0, assets.1].iter() {
				let meta_maybe: Option<_> =
					orml_asset_registry::Metadata::<T>::get(asset);
				if let Some(xyk) = meta_maybe.clone().and_then(|m| m.additional.xyk) {
					let mut additional = meta_maybe.unwrap().additional;
					if xyk.operations_disabled {
						additional.xyk = Some(XykMetadata { operations_disabled: false });
						match orml_asset_registry::Pallet::<T>::do_update_asset(
							asset,
							None,
							None,
							None,
							None,
							None,
							Some(additional),
							) {
							Ok(_) => {},
							Err(e) => {
								log::error!(target: "bootstrap", "cannot modify {} asset: {:?}!", asset, e);
								return false
							},
						}
					}
				}
			}
			true
		}
	}

}

pub mod pallet_transaction_payment_mangata{
	use super::*;

	parameter_types! {
		pub const OperationalFeeMultiplier: u8 = 5;
		pub const TransactionByteFee: Balance = 5 * consts::MILLIUNIT;
	pub ConstFeeMultiplierValue: Multiplier = Multiplier::saturating_from_rational(1, 1);
	}

	pub type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	pub type FeeMultiplierUpdate = ConstFeeMultiplier<ConstFeeMultiplierValue>;

// #[derive(Encode, Decode, TypeInfo)]
// pub enum LiquidityInfoEnum<C: MultiTokenCurrency<T::AccountId>, T: ::frame_system::Config> {
// 	Imbalance((TokenId, NegativeImbalanceOf<C, T>)),
// 	FeeLock,
// }
//
// #[derive(Encode, Decode, Clone, TypeInfo)]
// pub struct ThreeCurrencyOnChargeAdapter<C, OU, T1, T2, T3, SF2, SF3>(
// 	PhantomData<(C, OU, T1, T2, T3, SF2, SF3)>,
// );
//
// type NegativeImbalanceOf<C, T> =
// 	<C as MultiTokenCurrency<<T as ::frame_system::Config>::AccountId>>::NegativeImbalance;
//
// pub trait OnMultiTokenUnbalanced<
// 	Imbalance: frame_support::traits::TryDrop + MultiTokenImbalanceWithZeroTrait<TokenId>,
// >
// {
// 	/// Handler for some imbalances. The different imbalances might have different origins or
// 	/// meanings, dependent on the context. Will default to simply calling on_unbalanced for all
// 	/// of them. Infallible.
// 	fn on_unbalanceds<B>(token_id: TokenId, amounts: impl Iterator<Item = Imbalance>)
// 	where
// 		Imbalance: frame_support::traits::Imbalance<B>,
// 	{
// 		Self::on_unbalanced(amounts.fold(Imbalance::from_zero(token_id), |i, x| x.merge(i)))
// 	}
//
// 	/// Handler for some imbalance. Infallible.
// 	fn on_unbalanced(amount: Imbalance) {
// 		amount.try_drop().unwrap_or_else(Self::on_nonzero_unbalanced)
// 	}
//
// 	/// Actually handle a non-zero imbalance. You probably want to implement this rather than
// 	/// `on_unbalanced`.
// 	fn on_nonzero_unbalanced(amount: Imbalance) {
// 		drop(amount);
// 	}
// }
//
// pub struct ToAuthor<Runtime>(PhantomData<Runtime>);
//
// impl<Runtime> OnMultiTokenUnbalanced<ORMLCurrencyAdapterNegativeImbalance<Runtime>> for ToAuthor<Runtime>
// where
// Runtime: ::pallet_authorship::Config<AccountId= sp_runtime::AccountId32>,
// Runtime: ::orml_tokens::Config<CurrencyId = u32>,
// {
// 	fn on_nonzero_unbalanced(amount: ORMLCurrencyAdapterNegativeImbalance<Runtime>) {
// 		if let Some(author) = ::pallet_authorship::Pallet::<Runtime>::author() {
// 			<::orml_tokens::MultiTokenCurrencyAdapter<Runtime> as MultiTokenCurrency<
// 				AccountId,
// 			>>::resolve_creating(amount.0, &author, amount);
// 		}
// 	}
// }
//
// type ORMLCurrencyAdapterNegativeImbalance<Runtime> =
// 	<::orml_tokens::MultiTokenCurrencyAdapter<Runtime> as MultiTokenCurrency<
// 		AccountId,
// 	>>::NegativeImbalance;
//
// /// Default implementation for a Currency and an OnUnbalanced handler.
// ///
// /// The unbalance handler is given 2 unbalanceds in [`OnUnbalanced::on_unbalanceds`]: fee and
// /// then tip.
// impl<T, C, OU, T1, T2, T3, SF2, SF3> OnChargeTransaction<T>
// 	for ThreeCurrencyOnChargeAdapter<C, OU, T1, T2, T3, SF2, SF3>
// where
// 	T: ::pallet_transaction_payment_mangata::Config,
// 	T::LengthToFee: ::frame_support::weights::WeightToFee<
// 		Balance = <C as MultiTokenCurrency<<T as ::frame_system::Config>::AccountId>>::Balance,
// 	>,
// 	C: MultiTokenCurrency<<T as ::frame_system::Config>::AccountId>,
// 	C::PositiveImbalance: Imbalance<
// 		<C as MultiTokenCurrency<<T as ::frame_system::Config>::AccountId>>::Balance,
// 		Opposite = C::NegativeImbalance,
// 	>,
// 	C::NegativeImbalance: Imbalance<
// 		<C as MultiTokenCurrency<<T as ::frame_system::Config>::AccountId>>::Balance,
// 		Opposite = C::PositiveImbalance,
// 	>,
// 	OU: OnMultiTokenUnbalanced<NegativeImbalanceOf<C, T>>,
// 	NegativeImbalanceOf<C, T>: MultiTokenImbalanceWithZeroTrait<TokenId>,
// 	<C as MultiTokenCurrency<<T as ::frame_system::Config>::AccountId>>::Balance:
// 		scale_info::TypeInfo,
// 	T1: Get<TokenId>,
// 	T2: Get<TokenId>,
// 	T3: Get<TokenId>,
// 	SF2: Get<u128>,
// 	SF3: Get<u128>,
// 	Balance: From<<C as MultiTokenCurrency<<T as ::frame_system::Config>::AccountId>>::Balance>,
// 	sp_runtime::AccountId32: From<<T as ::frame_system::Config>::AccountId>,
// {
// 	type LiquidityInfo = Option<LiquidityInfoEnum<C, T>>;
// 	type Balance = <C as MultiTokenCurrency<<T as ::frame_system::Config>::AccountId>>::Balance;
//
// 	/// Withdraw the predicted fee from the transaction origin.
// 	///
// 	/// Note: The `fee` already includes the `tip`.
// 	fn withdraw_fee(
// 		who: &T::AccountId,
// 		_call: &T::RuntimeCall,
// 		_info: &DispatchInfoOf<T::RuntimeCall>,
// 		fee: Self::Balance,
// 		tip: Self::Balance,
// 	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
// 		if fee.is_zero() {
// 			return Ok(None)
// 		}
//
// 		let withdraw_reason = if tip.is_zero() {
// 			WithdrawReasons::TRANSACTION_PAYMENT
// 		} else {
// 			WithdrawReasons::TRANSACTION_PAYMENT | WithdrawReasons::TIP
// 		};
//
// 		match C::withdraw(
// 			T1::get().into(),
// 			who,
// 			fee,
// 			withdraw_reason,
// 			ExistenceRequirement::KeepAlive,
// 		) {
// 			Ok(imbalance) => Ok(Some(LiquidityInfoEnum::Imbalance((T1::get(), imbalance)))),
// 			// TODO make sure atleast 1 planck KSM is charged
// 			Err(_) => match C::withdraw(
// 				T2::get().into(),
// 				who,
// 				fee / SF2::get().into(),
// 				withdraw_reason,
// 				ExistenceRequirement::KeepAlive,
// 			) {
// 				Ok(imbalance) => Ok(Some(LiquidityInfoEnum::Imbalance((T2::get(), imbalance)))),
// 				Err(_) => match C::withdraw(
// 					T3::get().into(),
// 					who,
// 					fee / SF3::get().into(),
// 					withdraw_reason,
// 					ExistenceRequirement::KeepAlive,
// 				) {
// 					Ok(imbalance) => Ok(Some(LiquidityInfoEnum::Imbalance((T3::get(), imbalance)))),
// 					Err(_) => Err(InvalidTransaction::Payment.into()),
// 				},
// 			},
// 		}
// 	}
//
// 	/// Hand the fee and the tip over to the `[OnUnbalanced]` implementation.
// 	/// Since the predicted fee might have been too high, parts of the fee may
// 	/// be refunded.
// 	///
// 	/// Note: The `corrected_fee` already includes the `tip`.
// 	fn correct_and_deposit_fee(
// 		who: &T::AccountId,
// 		_dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
// 		_post_info: &PostDispatchInfoOf<T::RuntimeCall>,
// 		corrected_fee: Self::Balance,
// 		tip: Self::Balance,
// 		already_withdrawn: Self::LiquidityInfo,
// 	) -> Result<(), TransactionValidityError> {
// 		if let Some(LiquidityInfoEnum::Imbalance((token_id, paid))) = already_withdrawn {
// 			let (corrected_fee, tip) = if token_id == T3::get() {
// 				(corrected_fee / SF3::get().into(), tip / SF3::get().into())
// 			} else if token_id == T2::get() {
// 				(corrected_fee / SF2::get().into(), tip / SF2::get().into())
// 			} else {
// 				(corrected_fee, tip)
// 			};
// 			// Calculate how much refund we should return
// 			let refund_amount = paid.peek().saturating_sub(corrected_fee);
// 			// refund to the the account that paid the fees. If this fails, the
// 			// account might have dropped below the existential balance. In
// 			// that case we don't refund anything.
// 			let refund_imbalance = C::deposit_into_existing(token_id.into(), &who, refund_amount)
// 				.unwrap_or_else(|_| C::PositiveImbalance::from_zero(token_id.into()));
// 			// merge the imbalance caused by paying the fees and refunding parts of it again.
// 			let adjusted_paid = paid
// 				.offset(refund_imbalance)
// 				.same()
// 				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
// 			// Call someone else to handle the imbalance (fee and tip separately)
// 			let (tip_imb, fee) = adjusted_paid.split(tip);
// 			OU::on_unbalanceds(token_id, Some(fee).into_iter().chain(Some(tip_imb)));
// 			::pallet_transaction_payment_mangata::Pallet::<T>::deposit_event(
// 				::pallet_transaction_payment_mangata::Event::<T>::TransactionFeePaid {
// 					who: who.clone(),
// 					actual_fee: Default::default(),
// 					tip: Default::default(),
// 					// actual_fee: corrected_fee.into(),
// 					// tip: Balance::from(tip),
// 				},
// 			);
// 		}
// 		Ok(())
// 	}
// }
// 	// on-chain fees are 10x more expensive then ~real rate
// 	pub const KSM_MGX_SCALE_FACTOR: u128 = 1000_000_000_u128; // 1000 as KSM/MGX, with 6 decimals accounted for (12 - KSM, 18 - MGX)
// 	pub const KAR_MGX_SCALE_FACTOR: u128 = KSM_MGX_SCALE_FACTOR / 100; // 100 as KAR/KSM
// 	pub const TUR_MGX_SCALE_FACTOR: u128 = KSM_MGX_SCALE_FACTOR; // 100 as TUR/KSM, with 2 decimals accounted for (10 - TUR, 12 - KSM)
//
// pub type OnChargeTransactionHandler<Runtime> = ThreeCurrencyOnChargeAdapter<
// 	::orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
// 	ToAuthor<Runtime>,
// 	tokens::MgxTokenId,
// 	tokens::RelayTokenId,
// 	tokens::TurTokenId,
// 	::frame_support::traits::ConstU128<KSM_MGX_SCALE_FACTOR>,
// 	::frame_support::traits::ConstU128<TUR_MGX_SCALE_FACTOR>,
// >;
}

}
