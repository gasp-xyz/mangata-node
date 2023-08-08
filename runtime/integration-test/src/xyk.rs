use crate::setup::*;

const ASSET_ID_1: u32 = 1;

fn test_env(xyk_metadata: Option<XykMetadata>) -> TestExternalities {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 100 * UNIT),
			(AccountId::from(ALICE), ASSET_ID_1, 100 * UNIT),
		],
		assets: vec![(
			ASSET_ID_1,
			AssetMetadataOf {
				decimals: 18,
				name: b"Asset".to_vec(),
				symbol: b"Asset".to_vec(),
				location: None,
				existential_deposit: Default::default(),
				additional: CustomMetadata { xyk: xyk_metadata, ..CustomMetadata::default() },
			},
		)],
		..ExtBuilder::default()
	}
	.build()
}

fn create_pool() -> DispatchResultWithPostInfo {
	pallet_xyk::Pallet::<Runtime>::create_pool(
		RuntimeOrigin::signed(AccountId::from(ALICE)),
		NATIVE_ASSET_ID,
		10_ * UNIT,
		ASSET_ID_1,
		10 * UNIT,
	)
}

#[test]
fn create_pool_works_meta_allowed() {
	test_env(Some(XykMetadata { operations_disabled: false })).execute_with(|| {
		assert_ok!(create_pool());
	});
}

#[test]
fn create_pool_works_no_meta() {
	test_env(None).execute_with(|| {
		assert_ok!(create_pool());
	});
}

#[test]
fn create_pool_disabled_meta_disabled() {
	test_env(Some(XykMetadata { operations_disabled: true })).execute_with(|| {
		assert_err!(create_pool(), pallet_xyk::Error::<Runtime>::FunctionNotAvailableForThisToken);
	});
}


#[test]
fn swap_tx_does_not_charge_fee() {
	test_env(Some(XykMetadata { operations_disabled: true })).execute_with(|| {
		let call = RuntimeCall::Xyk(XykCall::sell_assset {
			sold_asset_id: 0u32,
			bought_asset_id: 4u32,
			sold_asset_amount: 1u128,
			min_amount_out: 0u128,
		});
		// assert_err!(create_pool(), pallet_xyk::Error::<Runtime>::FunctionNotAvailableForThisToken);
	// 	let xt = TestXt::new(call.clone(), Some((origin, extra)));
	});
}


	// fn query_info_and_fee_details_works() {
	// 	let call = RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 69 });
	// 	let origin = 111111;
	// 	let extra = ();
	// 	let xt = TestXt::new(call.clone(), Some((origin, extra)));
	// 	let info = xt.get_dispatch_info();
	// 	let ext = xt.encode();
	// 	let len = ext.len() as u32;
    //
	// 	let unsigned_xt = TestXt::<_, ()>::new(call, None);
	// 	let unsigned_xt_info = unsigned_xt.get_dispatch_info();
    //
	// 	ExtBuilder::default()
	// 		.base_weight(Weight::from_parts(5, 0))
	// 		.weight_fee(2)
	// 		.build()
	// 		.execute_with(|| {
	// 			// all fees should be x1.5
	// 			<NextFeeMultiplier<Runtime>>::put(Multiplier::saturating_from_rational(3, 2));
    //
	// 			assert_eq!(
	// 				TransactionPayment::query_info(xt.clone(), len),
	// 				RuntimeDispatchInfo {
	// 					weight: info.weight,
	// 					class: info.class,
	// 					partial_fee: 5 * 2 /* base * weight_fee */
	// 					+ len as u64  /* len * 1 */
	// 					+ info.weight.min(BlockWeights::get().max_block).ref_time() as u64 * 2 * 3 / 2 /* weight */
	// 				},
	// 			);
    //
	// 			assert_eq!(
	// 				TransactionPayment::query_info(unsigned_xt.clone(), len),
	// 				RuntimeDispatchInfo {
	// 					weight: unsigned_xt_info.weight,
	// 					class: unsigned_xt_info.class,
	// 					partial_fee: 0,
	// 				},
	// 			);
    //
	// 			assert_eq!(
	// 				TransactionPayment::query_fee_details(xt, len),
	// 				FeeDetails {
	// 					inclusion_fee: Some(InclusionFee {
	// 						base_fee: 5 * 2,
	// 						len_fee: len as u64,
	// 						adjusted_weight_fee: info
	// 							.weight
	// 							.min(BlockWeights::get().max_block)
	// 							.ref_time() as u64 * 2 * 3 / 2
	// 					}),
	// 					tip: 0,
	// 				},
	// 			);
    //
	// 			assert_eq!(
	// 				TransactionPayment::query_fee_details(unsigned_xt, len),
	// 				FeeDetails { inclusion_fee: None, tip: 0 },
	// 			);
	// 		});
	// }

