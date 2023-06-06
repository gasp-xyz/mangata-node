pub mod networks;
use cumulus_primitives_core::{Instruction::{TransferReserveAsset, BuyExecution, DepositAsset, ClearOrigin, ReserveAssetDeposited}, MultiAssets, MultiAsset, AssetId, MultiLocation, Junctions::{X2, Here, X1}, Junction::{Parachain, self}, Xcm, WeightLimit::Unlimited, Parent, WildMultiAsset::{All, self}, Fungibility::Fungible, NetworkId::{self, Polkadot}, MultiAssetFilter::Wild};
use xcm::IntoVersion;
use frame_support::assert_ok;
use xcm_emulator::TestExt;
use orml_traits::currency::MultiCurrency;
use networks::*;

pub type RelayChainPalletXcm = pallet_xcm::Pallet<polkadot_runtime::Runtime>;
pub type ParachainPalletXcm = pallet_xcm::Pallet<mangata_polkadot_runtime::Runtime>;
pub const INITIAL_BALANCE: u128 = 1_000_000_000;

#[test]
fn pallet_xcm_reserve_transfer_works() {
	TestNet::reset();

	let withdraw_amount = 123 * unit(12);

	networks::PolkadotRelay::execute_with(|| {
		sp_tracing::try_init_simple();
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
				polkadot_runtime::RuntimeOrigin::signed(ALICE),
				Box::new(Parachain(2110).into()),
				Box::new(Junction::AccountId32 { network: None, id: BOB.into() }.into()),
				Box::new((Here, withdraw_amount).into()),
				0,
				));
	});

	networks::Mangata::execute_with(|| {
		sp_tracing::try_init_simple();
		assert_eq!(
			mangata_polkadot_runtime::Tokens::free_balance(RELAY_ASSET_ID, &BOB),
			withdraw_amount
			);
	});
}

fn wrap_version<RuntimeCall>(
	xcm: impl Into<xcm::VersionedXcm<RuntimeCall>>,
) -> Result<xcm::VersionedXcm<RuntimeCall>, ()> {
	xcm.into().into_version(3)
}


#[test]
fn reserve_transfer_asserts_build_from_scratch_works() {
	TestNet::reset();
	networks::PolkadotRelay::execute_with(|| {
		sp_tracing::try_init_simple();
		log::info!("     <<< RELAY >>>");
		let withdraw_amount = 123 * unit(12);
		let assets = MultiAssets::from_sorted_and_deduplicated_skip_checks(vec![MultiAsset{
			id: AssetId::Concrete( MultiLocation { parents: 0, interior: Here }),
			fun: Fungible(withdraw_amount) }]
		);

		let xcm = Xcm(vec![
						  TransferReserveAsset {
								  assets: assets.clone(),
								  dest: MultiLocation::new(
									  0,
									  X1(Parachain(2110))
								  )
								  , xcm: Xcm(vec![
											 BuyExecution {
												 fees: MultiAsset { id: AssetId::Concrete(MultiLocation { parents: 1, interior: Here }), fun: Fungible(withdraw_amount) },
												 weight_limit: Unlimited
											 },
											 DepositAsset {
												 assets: Wild(WildMultiAsset::AllCounted(1)),
												 beneficiary: MultiLocation {
													 parents: 0,
													 interior: X1(Junction::AccountId32 { network: None, id: BOB_RAW })
												 }
											 }
								  ])
						  },
							ReserveAssetDeposited(assets),
							ClearOrigin
		]);

		// let x = wrap_version::<polkadot_runtime::RuntimeCall>(xcm).unwrap();
// ::<xcm::VersionedXcm>::().into_versioned(3);

					// let mut message = vec![ReserveAssetDeposited(assets), ClearOrigin];
					// message.extend(xcm.0.into_iter());
					// let versioned_dest = Box::new(cumulus_primitives_core::Junction::Parachain(2110).into_versioned());
					// let dest = MultiLocation::try_from(*versioned_dest).expect("convertable");

					// let versioned_dest = Box::new(cumulus_primitives_core::Junction::Parachain(2110).into_versioned());
					// let versioned_message = Box::new(xcm::VersionedXcm::from(xcm.clone()));
                //
				// polkadot_runtime::RuntimeOrigin::signed(ALICE),
				// Box::new(Junction::AccountId32 { network: None, id: ALICE.into() }.into()),
				// Box::new((Here, withdraw_amount).into()),
				// 	assert_ok!(polkadot_runtime::XcmPallet::send(
				// 			polkadot_runtime::RuntimeOrigin::root(),
				// 			versioned_dest,
				// 			versioned_message
				// 			));
                // //

		// <polkadot_runtime::Runtime as pallet_xcm::Config>::XcmExecutor::

					assert_ok!(pallet_xcm::Pallet::<polkadot_runtime::Runtime>::send_xcm(
							Junction::AccountId32 { network: None, id: ALICE_RAW },
							Parachain(2110),
							xcm
					));
                    //


					// polkadot_runtime::XcmPallet::send_xcm(Here, dest, xcm.clone()).unwrap();
					// polkadot_runtime::XcmPallet::send_xcm(Here, dest, xcm.clone()).unwrap();
					// assert_ok!(polkadot_runtime::XcmPallet::send_xcm(Here, Parent, xcm.clone()));
					// assert_ok!(polkadot_runtime::XcmPallet::reserve_transfer_assets(
					// 		polkadot_runtime::RuntimeOrigin::signed(ALICE.into()),
					// 		Box::new(cumulus_primitives_core::Junction::Parachain(2110).into_versioned()),
					// 		Box::new(cumulus_primitives_core::Junction::AccountId32 { id: BOB, network: None }.into_versioned()),
					// 		Box::new((cumulus_primitives_core::Junctions::Here, unit(12)).into()),
					// 		0
					// 		));
					// println!("BOB: {}", sp_runtime::AccountId32::from(BOB));
					// println!("BOB: {}", sp_runtime::AccountId32::from(BOB));
					// println!("BOB: {}", sp_runtime::AccountId32::from(BOB));
                    //
					// assert_eq!(
					// 	mangata_polkadot_runtime::Tokens::free_balance(RELAY_ASSET_ID, &BOB),
					// 	withdraw_amount
					// 	);
					// for e in polkadot_runtime::System::events(){
					// 	println!("{e:?}");
					// }

	});

	networks::Mangata::execute_with(|| {
		sp_tracing::try_init_simple();
		log::info!("   !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!  <<< PARACHAIN >>>");
		assert_eq!(
			mangata_polkadot_runtime::Tokens::free_balance(RELAY_ASSET_ID, &sp_runtime::AccountId32::from(BOB)),
			unit(12) /*- relay_per_second_as_fee(4)*/
		);
	});
}


// #[test]
// fn transfer_to_relay_chain() {
// 	use frame_support::weights::{Weight, WeightToFee as WeightToFeeT};
// 	use kusama_runtime_constants::fee::WeightToFee;
//
// 	let weight: XcmWeight = Weight::from_parts(299_506_000, 0);
// 	let fee = WeightToFee::weight_to_fee(&weight);
// 	assert_eq!(94_172_727, fee);
//
// 	Mangata::execute_with(|| {
// 		assert_ok!(XTokens::transfer(
// 			RuntimeOrigin::signed(ALICE.into()),
// 			RELAY_ASSET_ID,
// 			unit(12),
// 			Box::new(
// 				Junction::AccountId32 { id: BOB, network: None }
// 					.into_exterior(1)
// 					.into_versioned()
// 			),
// 			WeightLimit::Limited(weight)
// 		));
// 	});
//
// 	KusamaRelay::execute_with(|| {
// 		assert_eq!(kusama_runtime::Balances::free_balance(&AccountId::from(BOB)), unit(12) - fee);
// 	});
// }
//
// #[test]
// fn transfer_asset() {
// 	TestNet::reset();
// 	let unit = unit(18);
// 	let fee = native_per_second_as_fee(4);
// 	let registered_asset_id = RELAY_ASSET_ID + 1;
//
// 	Mangata::execute_with(|| {
// 		assert_ok!(Tokens::deposit(NATIVE_ASSET_ID, &reserve_account(SIBLING_ID), 50 * unit));
// 	});
//
// 	Sibling::execute_with(|| {
// 		assert_ok!(AssetRegistry::register_asset(
// 			RuntimeOrigin::root(),
// 			AssetMetadataOf {
// 				decimals: 18,
// 				name: b"MGX".to_vec(),
// 				symbol: b"MGX".to_vec(),
// 				location: None,
// 				existential_deposit: Default::default(),
// 				additional: CustomMetadata {
// 					xcm: Some(XcmMetadata { fee_per_second: mgx_per_second() }),
// 					..CustomMetadata::default()
// 				},
// 			},
// 			None
// 		));
//
// 		assert_ok!(Tokens::deposit(registered_asset_id, &AccountId::from(ALICE), 100 * unit));
//
// 		// no location for asset -> NotCrossChainTransferableCurrency
// 		assert_noop!(
// 			XTokens::transfer(
// 				RuntimeOrigin::signed(ALICE.into()),
// 				registered_asset_id,
// 				20 * unit,
// 				Box::new(
// 					MultiLocation::new(
// 						1,
// 						X2(
// 							Parachain(MANGATA_ID),
// 							Junction::AccountId32 { network: None, id: BOB.into() }
// 						)
// 					)
// 					.into()
// 				),
// 				WeightLimit::Limited(Weight::from_parts(600_000_000, 0)),
// 			),
// 			orml_xtokens::Error::<Runtime>::NotCrossChainTransferableCurrency
// 		);
// 		assert_ok!(AssetRegistry::update_asset(
// 			RuntimeOrigin::root(),
// 			registered_asset_id,
// 			None,
// 			None,
// 			None,
// 			None,
// 			Some(Some(mgx_location())),
// 			None,
// 		));
//
// 		assert_ok!(XTokens::transfer(
// 			RuntimeOrigin::signed(ALICE.into()),
// 			registered_asset_id,
// 			20 * unit,
// 			Box::new(
// 				MultiLocation::new(
// 					1,
// 					X2(
// 						Parachain(MANGATA_ID),
// 						Junction::AccountId32 { network: None, id: BOB.into() }
// 					)
// 				)
// 				.into()
// 			),
// 			WeightLimit::Limited(Weight::from_parts(600_000_000, 0)),
// 		));
//
// 		assert_eq!(Tokens::free_balance(registered_asset_id, &AccountId::from(ALICE)), 80 * unit);
// 	});
//
// 	Mangata::execute_with(|| {
// 		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(BOB)), 20 * unit - fee);
// 		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &reserve_account(SIBLING_ID)), 30 * unit);
//
// 		assert_ok!(XTokens::transfer(
// 			RuntimeOrigin::signed(BOB.into()),
// 			NATIVE_ASSET_ID,
// 			10 * unit,
// 			Box::new(
// 				MultiLocation::new(
// 					1,
// 					X2(
// 						Parachain(SIBLING_ID),
// 						Junction::AccountId32 { network: None, id: ALICE.into() }
// 					)
// 				)
// 				.into()
// 			),
// 			WeightLimit::Limited(Weight::from_parts(600_000_000, 0)),
// 		));
//
// 		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(BOB)), 10 * unit - fee);
// 		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &reserve_account(SIBLING_ID)), 40 * unit);
//
// 	});
//
// 	Sibling::execute_with(|| {
// 		assert_eq!(
// 			Tokens::free_balance(registered_asset_id, &AccountId::from(ALICE)),
// 			90 * unit - fee
// 		);
// 	});
// }

