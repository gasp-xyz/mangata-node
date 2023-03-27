use crate::{
	setup::*,
	xcm::{fee_test::*, kusama_test_net::*},
};

use frame_support::WeakBoundedVec;
use sp_runtime::traits::{AccountIdConversion, ConstU32};
use xcm::VersionedXcm;
use xcm_emulator::TestExt;

pub const MANGATA_ID: u32 = 2110;
pub const SIBLING_ID: u32 = 2000;

fn mgx_location() -> VersionedMultiLocation {
	MultiLocation::new(
		1,
		X2(
			Parachain(MANGATA_ID),
			GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
				NATIVE_ASSET_ID.encode(),
				None,
			)),
		),
	)
	.into()
}

fn reserve_account(id: u32) -> AccountId {
	polkadot_parachain::primitives::Sibling::from(id).into_account_truncating()
}

#[test]
fn transfer_from_relay_chain() {
	KusamaRelay::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::RuntimeOrigin::signed(ALICE.into()),
			Box::new(Parachain(MANGATA_ID).into().into()),
			Box::new(Junction::AccountId32 { id: BOB, network: NetworkId::Any }.into().into()),
			Box::new((Here, unit(12)).into()),
			0
		));
	});

	Mangata::execute_with(|| {
		assert_eq!(
			Tokens::free_balance(RELAY_ASSET_ID, &AccountId::from(BOB)),
			unit(12) - relay_per_second_as_fee(4)
		);
	});
}

#[test]
fn transfer_to_relay_chain() {
	use frame_support::weights::{Weight, WeightToFee as WeightToFeeT};
	use kusama_runtime_constants::fee::WeightToFee;

	let weight: XcmWeight = 298_368_000;
	let fee = WeightToFee::weight_to_fee(&Weight::from_ref_time(weight));
	assert_eq!(103_334_130, fee);

	Mangata::execute_with(|| {
		assert_ok!(XTokens::transfer(
			RuntimeOrigin::signed(ALICE.into()),
			RELAY_ASSET_ID,
			unit(12),
			Box::new(
				MultiLocation::new(
					1,
					X1(Junction::AccountId32 { id: BOB, network: NetworkId::Any })
				)
				.into()
			),
			WeightLimit::Limited(weight)
		));
	});

	KusamaRelay::execute_with(|| {
		assert_eq!(kusama_runtime::Balances::free_balance(&AccountId::from(BOB)), unit(12) - fee);
	});
}

#[test]
fn transfer_asset() {
	TestNet::reset();
	let unit = unit(18);
	let fee = native_per_second_as_fee(4);
	let registered_asset_id = RELAY_ASSET_ID + 1;

	Mangata::execute_with(|| {
		assert_ok!(Tokens::deposit(NATIVE_ASSET_ID, &reserve_account(SIBLING_ID), 50 * unit));
	});

	Sibling::execute_with(|| {
		assert_ok!(AssetRegistry::register_asset(
			RuntimeOrigin::root(),
			AssetMetadataOf {
				decimals: 18,
				name: b"MGX".to_vec(),
				symbol: b"MGX".to_vec(),
				location: None,
				existential_deposit: Default::default(),
				additional: CustomMetadata {
					xcm: Some(XcmMetadata { fee_per_second: mgx_per_second() }),
					..CustomMetadata::default()
				},
			},
			None
		));

		assert_ok!(Tokens::deposit(registered_asset_id, &AccountId::from(ALICE), 100 * unit));

		// no location for asset -> NotCrossChainTransferableCurrency
		assert_noop!(
			XTokens::transfer(
				RuntimeOrigin::signed(ALICE.into()),
				registered_asset_id,
				20 * unit,
				Box::new(
					MultiLocation::new(
						1,
						X2(
							Parachain(MANGATA_ID),
							Junction::AccountId32 { network: NetworkId::Any, id: BOB.into() }
						)
					)
					.into()
				),
				WeightLimit::Limited(600_000_000),
			),
			orml_xtokens::Error::<Runtime>::NotCrossChainTransferableCurrency
		);
		assert_ok!(AssetRegistry::update_asset(
			RuntimeOrigin::root(),
			registered_asset_id,
			None,
			None,
			None,
			None,
			Some(Some(mgx_location())),
			None,
		));

		assert_ok!(XTokens::transfer(
			RuntimeOrigin::signed(ALICE.into()),
			registered_asset_id,
			20 * unit,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Parachain(MANGATA_ID),
						Junction::AccountId32 { network: NetworkId::Any, id: BOB.into() }
					)
				)
				.into()
			),
			WeightLimit::Limited(600_000_000),
		));

		assert_eq!(Tokens::free_balance(registered_asset_id, &AccountId::from(ALICE)), 80 * unit);
	});

	Mangata::execute_with(|| {
		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(BOB)), 20 * unit - fee);
		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &reserve_account(SIBLING_ID)), 30 * unit);

		assert_ok!(XTokens::transfer(
			RuntimeOrigin::signed(BOB.into()),
			NATIVE_ASSET_ID,
			10 * unit,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Parachain(SIBLING_ID),
						Junction::AccountId32 { network: NetworkId::Any, id: ALICE.into() }
					)
				)
				.into()
			),
			WeightLimit::Limited(600_000_000),
		));

		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(BOB)), 10 * unit - fee);
		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &reserve_account(SIBLING_ID)), 40 * unit);
	});

	Sibling::execute_with(|| {
		assert_eq!(
			Tokens::free_balance(registered_asset_id, &AccountId::from(ALICE)),
			90 * unit - fee
		);
	});
}

#[test]
fn send_arbitrary_xcm_fails() {
	TestNet::reset();

	Mangata::execute_with(|| {
		assert_noop!(
			PolkadotXcm::send(
				RuntimeOrigin::signed(ALICE.into()),
				Box::new(MultiLocation::new(1, Here).into()),
				Box::new(VersionedXcm::from(Xcm(vec![WithdrawAsset((Here, 1).into())]))),
			),
			BadOrigin
		);
	});
}
