#![cfg(test)]
pub mod networks;
use cumulus_primitives_core::{
	AssetId,
	Fungibility::Fungible,
	Instruction::{BuyExecution, DepositAsset, InitiateReserveWithdraw, WithdrawAsset},
	Junction::{self, Parachain},
	Junctions::{Here, X1, X2},
	MultiAsset,
	MultiAssetFilter::Wild,
	MultiAssets, MultiLocation,
	WeightLimit::Unlimited,
	WildMultiAsset::{self, AllCounted},
	Xcm,
};
use frame_support::{assert_ok, traits::Currency};
use orml_traits::currency::MultiCurrency;
use xcm::VersionedXcm;
use xcm_emulator::TestExt;

use networks::*;

pub type RelayChainPalletXcm = pallet_xcm::Pallet<polkadot_runtime::Runtime>;
pub type ParachainPalletXcm = pallet_xcm::Pallet<mangata_polkadot_runtime::Runtime>;
pub type XParachainPalletXTokens = orml_xtokens::Pallet<xtokens_parachain::Runtime>;
pub const TRANSFER_AMOUNT: u128 = 50 * unit(12);

#[test]
fn dmp() {
	TestNet::reset();

	networks::Mangata::execute_with(|| {
		sp_tracing::try_init_simple();
		assert_eq!(mangata_polkadot_runtime::Tokens::free_balance(RELAY_ASSET_ID, &BOB), 0);
	});

	networks::PolkadotRelay::execute_with(|| {
		sp_tracing::try_init_simple();

		assert_eq!(polkadot_runtime::Balances::free_balance(&child_account_id(2110)), 0);

		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			polkadot_runtime::RuntimeOrigin::signed(ALICE),
			Box::new(Parachain(2110).into()),
			Box::new(Junction::AccountId32 { network: None, id: BOB.into() }.into()),
			Box::new((Here, TRANSFER_AMOUNT).into()),
			0,
		));

		assert_eq!(
			polkadot_runtime::Balances::free_balance(&child_account_id(2110)),
			TRANSFER_AMOUNT
		);
	});

	networks::Mangata::execute_with(|| {
		sp_tracing::try_init_simple();
		assert!(
			mangata_polkadot_runtime::Tokens::free_balance(RELAY_ASSET_ID, &BOB) > TRANSFER_AMOUNT * 90 / 100
		);
	});
}

#[test]
fn ump() {
	TestNet::reset();

	// deposit funds to sovereign account
	networks::PolkadotRelay::execute_with(|| {
		sp_tracing::try_init_simple();
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			polkadot_runtime::RuntimeOrigin::signed(ALICE),
			Box::new(Parachain(2110).into()),
			Box::new(Junction::AccountId32 { network: None, id: BOB.into() }.into()),
			Box::new((Here, TRANSFER_AMOUNT).into()),
			0,
		));
		assert_eq!(
			polkadot_runtime::Balances::free_balance(&child_account_id(2110)),
			TRANSFER_AMOUNT
		);
	});

	let assets = MultiAssets::from_sorted_and_deduplicated(vec![MultiAsset {
		id: AssetId::Concrete(MultiLocation { parents: 1, interior: Here }),
		fun: Fungible(TRANSFER_AMOUNT),
	}])
	.unwrap();

	networks::Mangata::execute_with(|| {
		sp_tracing::try_init_simple();
		assert_ok!(ParachainPalletXcm::execute(
			mangata_polkadot_runtime::RuntimeOrigin::signed(ALICE),
			Box::new(VersionedXcm::from(Xcm(vec![
				WithdrawAsset(assets),
				InitiateReserveWithdraw {
					assets: Wild(WildMultiAsset::All),
					reserve: MultiLocation { parents: 1, interior: Here },
					xcm: (Xcm(vec![
						BuyExecution {
							fees: MultiAsset {
								id: AssetId::Concrete(MultiLocation { parents: 0, interior: Here }),
								fun: Fungible(TRANSFER_AMOUNT)
							},
							weight_limit: Unlimited
						},
						DepositAsset {
							assets: Wild(AllCounted(1)),
							beneficiary: MultiLocation {
								parents: 0,
								interior: X1(Junction::AccountId32 { id: BOB_RAW, network: None })
							}
						}
					])),
				}
			]))),
			frame_support::weights::Weight::from_parts(u64::MAX, u64::MAX)
		));
	});

	networks::PolkadotRelay::execute_with(|| {
		sp_tracing::try_init_simple();
		assert!(polkadot_runtime::Balances::free_balance(&BOB) > TRANSFER_AMOUNT * 99 / 100,);
		assert_eq!(polkadot_runtime::Balances::free_balance(&child_account_id(2110)), 0);
	});
}

#[test]
fn xtokens_transfer_triggers_asset_trap() {
	TestNet::reset();

	// arrange
	networks::Mangata::execute_with(|| {
		sp_tracing::try_init_simple();
		let _ = mangata_polkadot_runtime::OrmlCurrencyAdapter::deposit_creating(
			&networks::reserve_account(2001),
			INITIAL_BALANCE,
		);
	});

	// act
	networks::XParachain::execute_with(|| {
		sp_tracing::try_init_simple();

		XParachainPalletXTokens::transfer_multiasset(
			xtokens_parachain::RuntimeOrigin::signed(ALICE),
			Box::new(MultiAsset {
			id: AssetId::Concrete(MultiLocation { parents: 1, interior: X1(Parachain(2001)) }),
			fun: Fungible(TRANSFER_AMOUNT),
			}.into()),
			Box::new(
					MultiLocation::new(
						1,
						X2(
							Parachain(2110),
							Junction::AccountId32 { network: None, id: BOB.into() }
						)
					)
					.into()
				),
				Unlimited
		).unwrap();


	});

	// asset
	networks::Mangata::execute_with(|| {
		sp_tracing::try_init_simple();
		assert!(mangata_polkadot_runtime::System::events().into_iter()
				.map(|e| e.event)
				.find(|e| matches!(
					e,
					mangata_polkadot_runtime::RuntimeEvent::PolkadotXcm(
						pallet_xcm::Event::<mangata_polkadot_runtime::Runtime>::AssetsTrapped(..)
						)
					)).is_some());

	});
}
