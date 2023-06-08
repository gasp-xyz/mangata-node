pub mod networks;
use cumulus_primitives_core::{Instruction::{TransferReserveAsset, BuyExecution, DepositAsset, ClearOrigin, ReserveAssetDeposited, Transact, WithdrawAsset, InitiateReserveWithdraw}, MultiAssets, MultiAsset, AssetId, MultiLocation, Junctions::{X2, Here, X1}, Junction::{Parachain, self}, Xcm, WeightLimit::Unlimited, Parent, WildMultiAsset::{All, self, AllCounted}, Fungibility::Fungible, NetworkId::{self, Polkadot}, MultiAssetFilter::Wild, OriginKind};
use xcm::{IntoVersion, VersionedXcm};
use frame_support::assert_ok;
use xcm_emulator::TestExt;
use orml_traits::currency::MultiCurrency;
use frame_support::weights::Weight;
use networks::*;
use xcm_emulator::Encode;

pub type RelayChainPalletXcm = pallet_xcm::Pallet<polkadot_runtime::Runtime>;
pub type ParachainPalletXcm = pallet_xcm::Pallet<mangata_polkadot_runtime::Runtime>;
pub const INITIAL_BALANCE: u128 = 1_000_000_000;


#[test]
fn dot_reserve_transfer_assets_works() {
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


#[test]
fn ump() {
	TestNet::reset();

	let remark = polkadot_runtime::RuntimeCall::System(
		frame_system::Call::<polkadot_runtime::Runtime>::remark_with_event { remark: vec![1, 2, 3] },
	);
	let withdraw_amount = 123 * unit(12);
	let mut assets = MultiAssets::new();
	assets.push( MultiAsset { id: AssetId::Concrete(MultiLocation { parents: 1, interior: Here }), fun: Fungible(withdraw_amount) });

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
									  fees: MultiAsset { id: AssetId::Concrete(MultiLocation { parents: 0, interior: Here }), fun: Fungible(withdraw_amount) },
									  weight_limit: Unlimited
								  },
								  DepositAsset {
									  assets: Wild(AllCounted(1)),
									  beneficiary: MultiLocation { parents: 0, interior: X1(Junction::AccountId32 { id: BOB_RAW, network: None }) }
								  }
						])),
					}
				]))),
				frame_support::weights::Weight::from_parts(u64::MAX, u64::MAX)
				));
	});

	networks::PolkadotRelay::execute_with(|| {
		sp_tracing::try_init_simple();
		use polkadot_runtime::{RuntimeEvent, System};
		assert!(
			polkadot_runtime::Balances::free_balance(&BOB) > withdraw_amount * 99 / 100,
			);

	});
}
