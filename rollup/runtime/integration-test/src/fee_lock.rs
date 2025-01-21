use crate::setup::*;

use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use pallet_market::PoolKind;
use rollup_runtime::{
	config::{
		pallet_transaction_payment::{OnChargeHandler, ToAuthor},
		BnbAccountIdOf, TreasuryAccountIdOf,
	},
	fees::FEE_PRECISION,
	OnChargeTransactionHandler,
};

const ASSET_ID_1: u32 = 1;
const ASSET_ID_2: u32 = 2;
const POOL_ID: u32 = 3;
// runtime_config::fees::MarketTotalFee
const FEE: u128 = UNIT * 30_000_000 / FEE_PRECISION;
const TRSY_FEE_P: u128 = FEE * 3_333_333_334 / FEE_PRECISION;
const BNB_FEE: u128 = TRSY_FEE_P * 5_000_000_000 / FEE_PRECISION;
const TRSY_FEE: u128 = TRSY_FEE_P - BNB_FEE;
const POOL_FEE: u128 = FEE - TRSY_FEE - BNB_FEE;

type Market = pallet_market::Pallet<Runtime>;

fn test_env() -> TestExternalities {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_2, 100 * UNIT),
			(AccountId::from(BOB), ASSET_ID_2, 100 * UNIT),
		],
		assets: vec![],
		..ExtBuilder::default()
	}
	.build()
}

// NATIVE_ASSET_ID & ASSET_ID_1 are used in TwoCurrencyOnChargeHandler for native fees
type Handler = OnChargeHandler<
	orml_tokens::MultiTokenCurrencyAdapter<Runtime>,
	ToAuthor<Runtime>,
	OnChargeTransactionHandler<Runtime>,
	FeeLock,
>;

fn origin() -> RuntimeOrigin {
	RuntimeOrigin::signed(AccountId::from(ALICE))
}

fn root() -> RuntimeOrigin {
	frame_system::RawOrigin::Root.into()
}

fn init_fee_lock(amount: Balance) {
	FeeLock::update_fee_lock_metadata(root(), Some(10), Some(amount), Some(1), Some(vec![]))
		.unwrap();
}

pub(crate) fn events() -> Vec<RuntimeEvent> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| match e {
			RuntimeEvent::FeeLock(_) |
			RuntimeEvent::Tokens(_) |
			RuntimeEvent::TransactionPayment(_) => Some(e),
			_ => None,
		})
		.collect::<Vec<_>>()
}

fn settle_fee(
	who: &AccountId,
	fee: Balance,
	assets: (TokenId, TokenId),
	has_failed: bool,
	multi: bool,
) {
	let swap_pool_list = if multi { vec![POOL_ID, 0] } else { vec![POOL_ID] };
	let withdrawn = <Handler as OnChargeTransaction<Runtime>>::withdraw_fee(
		&who,
		&RuntimeCall::Market(pallet_market::Call::multiswap_asset {
			swap_pool_list,
			asset_id_in: assets.0,
			asset_amount_in: UNIT,
			asset_id_out: assets.1,
			min_amount_out: UNIT,
		}),
		&DispatchInfo::default(),
		fee,
		0,
	)
	.unwrap();

	let pays_fee = if has_failed { Pays::Yes } else { Pays::No };

	let _ = <Handler as OnChargeTransaction<Runtime>>::correct_and_deposit_fee(
		&who,
		&DispatchInfo::default(),
		&PostDispatchInfo { actual_weight: None, pays_fee },
		fee,
		0,
		withdrawn,
	);
}

#[test]
fn high_value_swap_should_be_free() {
	test_env().execute_with(|| {
		init_fee_lock(UNIT);
		Market::create_pool(origin(), PoolKind::Xyk, ASSET_ID_1, UNIT, NATIVE_ASSET_ID, UNIT)
			.unwrap();
		let who = AccountId::from(ALICE);

		let before = Tokens::accounts(who, NATIVE_ASSET_ID);
		settle_fee(&who, 1000, (ASSET_ID_1, NATIVE_ASSET_ID), false, false);
		let after = Tokens::accounts(who, NATIVE_ASSET_ID);

		assert_eq!(before.free, after.free);
	})
}

#[test]
fn high_value_swap_should_take_fees_from_input_xyk() {
	test_env().execute_with(|| {
		init_fee_lock(UNIT);
		Market::create_pool(origin(), PoolKind::Xyk, ASSET_ID_2, UNIT, NATIVE_ASSET_ID, UNIT)
			.unwrap();
		let who = AccountId::from(ALICE);

		let before_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		let before_2 = Tokens::accounts(AccountId::from(ALICE), ASSET_ID_2);
		settle_fee(&who, 1000, (ASSET_ID_2, NATIVE_ASSET_ID), true, false);
		let after_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		let after_2 = Tokens::accounts(AccountId::from(ALICE), ASSET_ID_2);

		assert_eq!(before_0.free, after_0.free);
		assert_eq!(before_2.free, after_2.free + FEE);
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_2,
			from: who,
			to: TreasuryAccountIdOf::<Runtime>::get(),
			amount: TRSY_FEE,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_2,
			from: who,
			to: BnbAccountIdOf::<Runtime>::get(),
			amount: BNB_FEE,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_2,
			from: who,
			to: Xyk::account_id(),
			amount: POOL_FEE,
		}));
	})
}

#[test]
fn high_value_swap_should_take_fees_from_input_ss() {
	test_env().execute_with(|| {
		init_fee_lock(UNIT);
		Market::create_pool(
			origin(),
			PoolKind::StableSwap,
			ASSET_ID_2,
			UNIT,
			NATIVE_ASSET_ID,
			UNIT,
		)
		.unwrap();
		let who = AccountId::from(ALICE);

		let before_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		let before_2 = Tokens::accounts(AccountId::from(ALICE), ASSET_ID_2);
		settle_fee(&who, 1000, (ASSET_ID_2, NATIVE_ASSET_ID), true, false);
		let after_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		let after_2 = Tokens::accounts(AccountId::from(ALICE), ASSET_ID_2);

		assert_eq!(before_0.free, after_0.free);
		assert_eq!(before_2.free, after_2.free + FEE);
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_2,
			from: who,
			to: TreasuryAccountIdOf::<Runtime>::get(),
			amount: TRSY_FEE,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_2,
			from: who,
			to: BnbAccountIdOf::<Runtime>::get(),
			amount: BNB_FEE,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_2,
			from: who,
			to: StableSwap::get_pool_account(&POOL_ID),
			amount: POOL_FEE,
		}));
	})
}

#[test]
fn low_value_swap_should_lock_fees_in_native() {
	test_env().execute_with(|| {
		let fee_lock = 5 * UNIT;
		init_fee_lock(fee_lock);
		let who = AccountId::from(ALICE);
		let fee = 1000;
		Market::create_pool(origin(), PoolKind::Xyk, ASSET_ID_1, UNIT, ASSET_ID_2, UNIT).unwrap();

		let before_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		settle_fee(&who, fee, (ASSET_ID_1, ASSET_ID_2), false, true);
		let after_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);

		assert_eq!(before_0.free, after_0.free + fee_lock);
		assert_eq!(after_0.reserved, fee_lock);
		// fees locked
		System::assert_last_event(RuntimeEvent::FeeLock(pallet_fee_lock::Event::FeeLocked {
			who,
			lock_amount: fee_lock,
			total_locked: fee_lock,
		}));
	})
}

#[test]
fn low_value_swap_should_lock_fees_in_native_and_take_fees_from_input_xyk() {
	test_env().execute_with(|| {
		let fee_lock = 5 * UNIT;
		init_fee_lock(fee_lock);
		let who = AccountId::from(ALICE);
		let fee = 1000;
		Market::create_pool(origin(), PoolKind::Xyk, ASSET_ID_1, UNIT, ASSET_ID_2, UNIT).unwrap();

		let before_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		let before_1 = Tokens::accounts(AccountId::from(ALICE), ASSET_ID_1);
		settle_fee(&who, fee, (ASSET_ID_1, ASSET_ID_2), true, true);
		let after_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		let after_1 = Tokens::accounts(AccountId::from(ALICE), ASSET_ID_1);

		assert_eq!(before_0.free, after_0.free + fee_lock);
		assert_eq!(after_0.reserved, fee_lock);
		assert_eq!(before_1.free, after_1.free + FEE);
		// fees locked
		System::assert_has_event(RuntimeEvent::FeeLock(pallet_fee_lock::Event::FeeLocked {
			who,
			lock_amount: fee_lock,
			total_locked: fee_lock,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_1,
			from: who,
			to: TreasuryAccountIdOf::<Runtime>::get(),
			amount: TRSY_FEE,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_1,
			from: who,
			to: BnbAccountIdOf::<Runtime>::get(),
			amount: BNB_FEE,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_1,
			from: who,
			to: Xyk::account_id(),
			amount: POOL_FEE,
		}));
	})
}

#[test]
fn low_value_swap_should_lock_fees_in_native_and_take_fees_from_input_ss() {
	test_env().execute_with(|| {
		let fee_lock = 5 * UNIT;
		init_fee_lock(fee_lock);
		let who = AccountId::from(ALICE);
		let fee = 1000;
		Market::create_pool(origin(), PoolKind::StableSwap, ASSET_ID_1, UNIT, ASSET_ID_2, UNIT)
			.unwrap();

		let before_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		let before_1 = Tokens::accounts(AccountId::from(ALICE), ASSET_ID_1);
		settle_fee(&who, fee, (ASSET_ID_1, ASSET_ID_2), true, true);
		let after_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		let after_1 = Tokens::accounts(AccountId::from(ALICE), ASSET_ID_1);

		assert_eq!(before_0.free, after_0.free + fee_lock);
		assert_eq!(after_0.reserved, fee_lock);
		assert_eq!(before_1.free, after_1.free + FEE);
		// fees locked
		System::assert_has_event(RuntimeEvent::FeeLock(pallet_fee_lock::Event::FeeLocked {
			who,
			lock_amount: fee_lock,
			total_locked: fee_lock,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_1,
			from: who,
			to: TreasuryAccountIdOf::<Runtime>::get(),
			amount: TRSY_FEE,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_1,
			from: who,
			to: BnbAccountIdOf::<Runtime>::get(),
			amount: BNB_FEE,
		}));
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_1,
			from: who,
			to: StableSwap::get_pool_account(&POOL_ID),
			amount: POOL_FEE,
		}));
	})
}
