use crate::setup::*;

#[test]
fn identity_permissions_correct() {
	ExtBuilder {
		balances: vec![(AccountId::from(ALICE), NATIVE_ASSET_ID, 10000 * UNIT)],
		..ExtBuilder::default()
	}
	.build()
	.execute_with(|| {
		assert_noop!(
			Identity::add_registrar(
				RuntimeOrigin::signed(AccountId::from(ALICE)),
				AccountId::from(BOB).into()
			),
			BadOrigin
		);
		assert_ok!(Identity::add_registrar(RuntimeOrigin::root(), AccountId::from(BOB).into()));

		assert_noop!(
			Identity::kill_identity(
				RuntimeOrigin::signed(AccountId::from(ALICE)),
				AccountId::from(BOB).into()
			),
			BadOrigin
		);
		// origin passes, but fails on no identity set
		assert_noop!(
			Identity::kill_identity(RuntimeOrigin::root(), AccountId::from(BOB).into()),
			pallet_identity::Error::<Runtime>::NotNamed
		);
	});
}
