use crate::setup::*;

use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use rollup_runtime::{
	config::{
		pallet_transaction_payment::{OnChargeHandler, ToAuthor},
		TreasuryAccountIdOf,
	},
	fees::FEE_PRECISION,
	OnChargeTransactionHandler,
};

const ASSET_ID_1: u32 = 1;
const ASSET_ID_2: u32 = 2;
// runtime_config::fees::MarketTotalFee
const FEE: u128 = UNIT * 30_000_000 / FEE_PRECISION;

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

fn settle_fee(who: &AccountId, fee: Balance, assets: (TokenId, TokenId), has_failed: bool) {
	let withdrawn = <Handler as OnChargeTransaction<Runtime>>::withdraw_fee(
		&who,
		&RuntimeCall::Market(pallet_market::Call::multiswap_asset {
			swap_pool_list: vec![0],
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
		let who = AccountId::from(ALICE);

		let before = Tokens::accounts(who, NATIVE_ASSET_ID);
		settle_fee(&who, 1000, (ASSET_ID_1, NATIVE_ASSET_ID), false);
		let after = Tokens::accounts(who, NATIVE_ASSET_ID);

		assert_eq!(before.free, after.free);
		// only the init fee lock is emitted
		System::assert_last_event(RuntimeEvent::FeeLock(
			pallet_fee_lock::Event::FeeLockMetadataUpdated,
		));
	})
}

#[test]
fn high_value_swap_should_take_fees_from_native_asset_if_possible() {
	test_env().execute_with(|| {
		init_fee_lock(UNIT);
		let who = AccountId::from(ALICE);
		let fee = 1000;

		let before = Tokens::accounts(who, NATIVE_ASSET_ID);
		settle_fee(&who, fee, (ASSET_ID_1, NATIVE_ASSET_ID), true);
		let after = Tokens::accounts(who, NATIVE_ASSET_ID);

		assert_eq!(before.free, after.free + fee);
		System::assert_last_event(RuntimeEvent::TransactionPayment(
			pallet_transaction_payment::Event::TransactionFeePaid {
				who,
				token_id: NATIVE_ASSET_ID,
				actual_fee: fee,
				tip: 0,
			},
		));
	})
}

#[test]
fn high_value_swap_should_take_fees_from_input_if_no_native_asset_balance() {
	test_env().execute_with(|| {
		init_fee_lock(UNIT);
		// has no native tokens
		let who = AccountId::from(BOB);

		let before_0 = Tokens::accounts(AccountId::from(BOB), NATIVE_ASSET_ID);
		let before_2 = Tokens::accounts(AccountId::from(BOB), ASSET_ID_2);
		settle_fee(&who, 1000, (ASSET_ID_2, NATIVE_ASSET_ID), true);
		let after_2 = Tokens::accounts(AccountId::from(BOB), ASSET_ID_2);

		assert_eq!(before_0.free, 0);
		assert_eq!(before_2.free, after_2.free + FEE);
		System::assert_last_event(RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: ASSET_ID_2,
			from: who,
			to: TreasuryAccountIdOf::<Runtime>::get(),
			amount: FEE,
		}));
	})
}

#[test]
fn high_value_swap_should_take_fees_from_second_fee_asset_if_possible() {
	test_env().execute_with(|| {
		init_fee_lock(UNIT);
		let who = AccountId::from(BOB);
		let fee = 1000;
		// mint some second fee tokens
		Tokens::mint(root(), ASSET_ID_1, who, 100 * UNIT).unwrap();

		let before_0 = Tokens::accounts(AccountId::from(BOB), NATIVE_ASSET_ID);
		let before_1 = Tokens::accounts(AccountId::from(BOB), ASSET_ID_1);
		// TwoCurrencyOnChargeAdapter second token rate rollup/runtime/src/lib.rs:L498
		settle_fee(&who, fee * 30_000, (ASSET_ID_2, NATIVE_ASSET_ID), true);
		let after_1 = Tokens::accounts(AccountId::from(BOB), ASSET_ID_1);

		assert_eq!(before_0.free, 0);
		assert_eq!(before_1.free, after_1.free + fee);
		System::assert_last_event(RuntimeEvent::TransactionPayment(
			pallet_transaction_payment::Event::TransactionFeePaid {
				who,
				token_id: ASSET_ID_1,
				actual_fee: fee,
				tip: 0,
			},
		));
	})
}

#[test]
fn low_value_swap_should_lock_fees_in_native() {
	test_env().execute_with(|| {
		let fee_lock = 5 * UNIT;
		init_fee_lock(fee_lock);
		let who = AccountId::from(ALICE);
		let fee = 1000;

		let before_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		settle_fee(&who, fee, (ASSET_ID_1, ASSET_ID_2), false);
		let after_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);

		println!("{:#?}", before_0);
		println!("{:#?}", after_0);
		println!("{:#?}", events());

		assert_eq!(before_0.free, after_0.free + fee_lock);
		assert_eq!(after_0.reserved, fee_lock);
		// max(fee_lock, fee) is withdrawn, all is refunded
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Deposited {
			currency_id: NATIVE_ASSET_ID,
			who,
			amount: fee_lock,
		}));
		// zero fee was paid
		System::assert_has_event(RuntimeEvent::TransactionPayment(
			pallet_transaction_payment::Event::TransactionFeePaid {
				who,
				token_id: NATIVE_ASSET_ID,
				actual_fee: 0,
				tip: 0,
			},
		));
		// fees locked
		System::assert_last_event(RuntimeEvent::FeeLock(pallet_fee_lock::Event::FeeLocked {
			who,
			lock_amount: fee_lock,
			total_locked: fee_lock,
		}));
	})
}

#[test]
fn low_value_swap_should_take_fees_in_native_if_fails() {
	test_env().execute_with(|| {
		let fee_lock = 5 * UNIT;
		init_fee_lock(fee_lock);
		let who = AccountId::from(ALICE);
		let fee = 1000;

		let before_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		settle_fee(&who, fee, (ASSET_ID_1, ASSET_ID_2), true);
		let after_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);

		println!("{:#?}", before_0);
		println!("{:#?}", after_0);
		println!("{:#?}", events());

		assert_eq!(before_0.free, after_0.free + fee);
		// max(fee_lock, fee) is withdrawn, so fee_lock-fee is refunded
		System::assert_has_event(RuntimeEvent::Tokens(orml_tokens::Event::Deposited {
			currency_id: NATIVE_ASSET_ID,
			who,
			amount: fee_lock - fee,
		}));

		System::assert_last_event(RuntimeEvent::TransactionPayment(
			pallet_transaction_payment::Event::TransactionFeePaid {
				who,
				token_id: NATIVE_ASSET_ID,
				actual_fee: fee,
				tip: 0,
			},
		));
	})
}

#[test]
fn low_value_swap_should_take_fees_in_native_if_fails_big_fee() {
	test_env().execute_with(|| {
		let fee_lock = 5 * UNIT;
		init_fee_lock(fee_lock);
		let who = AccountId::from(ALICE);
		let fee = 10 * fee_lock;

		let before_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);
		settle_fee(&who, fee, (ASSET_ID_1, ASSET_ID_2), true);
		let after_0 = Tokens::accounts(AccountId::from(ALICE), NATIVE_ASSET_ID);

		println!("{:#?}", before_0);
		println!("{:#?}", after_0);
		println!("{:#?}", events());

		assert_eq!(before_0.free, after_0.free + fee);
		// max(fee_lock, fee) is withdrawn, nothing is refunded
		assert!(events().iter().all(|event| match *event {
			RuntimeEvent::Tokens(orml_tokens::Event::Deposited { .. }) => false,
			_ => true,
		}));

		System::assert_last_event(RuntimeEvent::TransactionPayment(
			pallet_transaction_payment::Event::TransactionFeePaid {
				who,
				token_id: NATIVE_ASSET_ID,
				actual_fee: fee,
				tip: 0,
			},
		));
	})
}
