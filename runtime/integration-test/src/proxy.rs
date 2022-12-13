use crate::setup::*;

type SystemError = frame_system::Error<Runtime>;

const ASSET_ID_1: u32 = 1;
const LP_ASSET_ID: u32 = 2;

#[test]
fn proxy_behavior_correct() {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 1000 * UNIT),
			(AccountId::from(BOB), NATIVE_ASSET_ID, 1000 * UNIT),
		],
		..ExtBuilder::default()
	}
	.build()
	.execute_with(|| {
		// proxy fails for account with no NATIVE_ASSET_ID
		assert_noop!(
			Proxy::add_proxy(
				Origin::signed(AccountId::from([21; 32])),
				AccountId::from(ALICE).into(),
				ProxyType::Any,
				0
			),
			orml_tokens::Error::<Runtime>::BalanceTooLow
		);
		let call = Box::new(Call::Tokens(orml_tokens::Call::transfer {
			dest: AccountId::from(ALICE).into(),
			currency_id: NATIVE_ASSET_ID,
			amount: 10 * UNIT,
		}));

		// Alice has all Bob's permissions now
		assert_ok!(Proxy::add_proxy(
			Origin::signed(AccountId::from(BOB)),
			AccountId::from(ALICE).into(),
			ProxyType::Any,
			0
		));
		// takes deposit from bobs account for proxy
		assert!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(BOB)) < 1000 * UNIT);

		// alice can now make calls for bob's account
		assert_ok!(Proxy::proxy(
			Origin::signed(AccountId::from(ALICE)),
			AccountId::from(BOB).into(),
			None,
			call.clone()
		));
		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(ALICE)), 1010 * UNIT);

		// alice cannot make calls for bob's account anymore
		assert_ok!(Proxy::remove_proxy(
			Origin::signed(AccountId::from(BOB)),
			AccountId::from(ALICE).into(),
			ProxyType::Any,
			0
		));
		assert_noop!(
			Proxy::proxy(
				Origin::signed(AccountId::from(ALICE)),
				AccountId::from(BOB).into(),
				None,
				call.clone()
			),
			pallet_proxy::Error::<Runtime>::NotProxy
		);
		// bob's deposit is returned
		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(BOB)), 990 * UNIT);
	});
}

#[test]
fn proxy_permissions_correct() {
	ExtBuilder {
		balances: vec![
			(AccountId::from(ALICE), NATIVE_ASSET_ID, 1000 * UNIT),
			(AccountId::from(BOB), NATIVE_ASSET_ID, 1000 * UNIT),
			(AccountId::from(BOB), ASSET_ID_1, 1000 * UNIT),
		],
		..ExtBuilder::default()
	}
	.build()
	.execute_with(|| {
		assert_ok!(Xyk::create_pool(
			Origin::signed(AccountId::from(BOB)),
			NATIVE_ASSET_ID,
			10_ * UNIT,
			ASSET_ID_1,
			10 * UNIT,
		));
		// Alice has all Bob's permissions now
		assert_ok!(Proxy::add_proxy(
			Origin::signed(AccountId::from(BOB)),
			AccountId::from(ALICE).into(),
			ProxyType::Any,
			0
		));
		let root_call = Box::new(Call::Tokens(orml_tokens::Call::set_balance {
			who: AccountId::from(ALICE).into(),
			currency_id: NATIVE_ASSET_ID,
			new_free: 1 * UNIT,
			new_reserved: 0,
		}));
		let transfer_call = Box::new(Call::Tokens(orml_tokens::Call::transfer {
			dest: AccountId::from(BOB).into(),
			currency_id: NATIVE_ASSET_ID,
			amount: 10 * UNIT,
		}));
		let provide_liquidity_call =
			Box::new(Call::Xyk(pallet_xyk::Call::provide_liquidity_with_conversion {
				liquidity_asset_id: LP_ASSET_ID,
				provided_asset_id: NATIVE_ASSET_ID,
				provided_asset_amount: 10 * UNIT,
			}));
		let compound_call = Box::new(Call::Xyk(pallet_xyk::Call::compound_rewards {
			liquidity_asset_id: LP_ASSET_ID,
			amount_permille: Permill::one(),
		}));
		let create_pool_call = Box::new(Call::Xyk(pallet_xyk::Call::create_pool {
			first_asset_id: NATIVE_ASSET_ID,
			first_asset_amount: 100,
			second_asset_id: ASSET_ID_1,
			second_asset_amount: 100,
		}));

		// Proxy calls do not bypass root permission
		assert_ok!(Proxy::proxy(
			Origin::signed(AccountId::from(ALICE)),
			AccountId::from(BOB).into(),
			None,
			root_call.clone()
		));
		// while the proxy call executes the call being proxied fails
		assert_eq!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(ALICE)), 1000 * UNIT);

		// Alice's gives compound permissions to Bob
		assert_ok!(Proxy::add_proxy(
			Origin::signed(AccountId::from(ALICE)),
			AccountId::from(BOB).into(),
			ProxyType::AutoCompound,
			0
		));
		// Bob can be a proxy for alice compound calls
		assert_ok!(Proxy::proxy(
			Origin::signed(AccountId::from(BOB)),
			AccountId::from(ALICE).into(),
			Some(ProxyType::AutoCompound),
			provide_liquidity_call.clone()
		));
		assert_eq!(
			Xyk::asset_pool((NATIVE_ASSET_ID, ASSET_ID_1)),
			(19995851032790690753, 10 * UNIT)
		);

		// Bob can be a proxy for alice compound calls
		assert_ok!(Proxy::proxy(
			Origin::signed(AccountId::from(BOB)),
			AccountId::from(ALICE).into(),
			Some(ProxyType::AutoCompound),
			compound_call.clone()
		));
		// assert NoRewardsEvent

		// Bob can't proxy for alice in a non compound call, once again proxy call works but nested call fails
		assert_ok!(Proxy::proxy(
			Origin::signed(AccountId::from(BOB)),
			AccountId::from(ALICE).into(),
			Some(ProxyType::AutoCompound),
			transfer_call.clone()
		));
		// the transfer call fails as Bob only had compound permission for alice
		assert!(Tokens::free_balance(NATIVE_ASSET_ID, &AccountId::from(BOB)) < 1000 * UNIT);

		// create pool call is part of the Xyk but is not in the AutoCompound ProxyType filter
		assert_ok!(Proxy::proxy(
			Origin::signed(AccountId::from(BOB)),
			AccountId::from(ALICE).into(),
			Some(ProxyType::AutoCompound),
			create_pool_call.clone()
		));
		// hence the failure
		System::assert_last_event(
			pallet_proxy::Event::ProxyExecuted { result: Err(SystemError::CallFiltered.into()) }
				.into(),
		);

		// remove proxy works
		assert_ok!(Proxy::remove_proxy(
			Origin::signed(AccountId::from(ALICE)),
			AccountId::from(BOB).into(),
			ProxyType::AutoCompound,
			0
		));
		assert_noop!(
			Proxy::proxy(
				Origin::signed(AccountId::from(BOB)),
				AccountId::from(ALICE).into(),
				Some(ProxyType::AutoCompound),
				provide_liquidity_call.clone()
			),
			pallet_proxy::Error::<Runtime>::NotProxy
		);
	});
}
