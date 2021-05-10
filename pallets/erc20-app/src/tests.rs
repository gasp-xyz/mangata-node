use crate::mock::{new_tester, AccountId, MockEvent, MockRuntime, Origin, System, ERC20, Tokens};
use frame_support::assert_ok;
use frame_system as system;
use hex_literal::hex;
use mangata_primitives::TokenId;
use orml_tokens::MultiTokenCurrency;
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

use crate::RawEvent;

use crate::payload::Payload;

type TestAccountId = <MockRuntime as system::Trait>::AccountId;

fn last_event() -> MockEvent {
    System::events().pop().expect("Event expected").event
}

#[test]
fn mints_after_handling_ethereum_event() {
    new_tester().execute_with(|| {
        let token_addr = H160::repeat_byte(1);
        let id_of_first_minted_token: TokenId = 0;
        let bob: AccountId = Keyring::Bob.into();

        let event: Payload<TestAccountId> = Payload {
            sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
            recipient_addr: bob.clone(),
            token_addr,
            amount: 10.into(),
        };

        // crating token with ID = 0
        assert_ok!(ERC20::handle_event(event.clone()));
        assert_eq!(
            Tokens::free_balance(id_of_first_minted_token, &bob),
            10
        );

        // minting previously created token
        assert_ok!(ERC20::handle_event(event));
        assert_eq!(
            Tokens::free_balance(id_of_first_minted_token, &bob),
            20
        );
    });
}

#[test]
fn burn_should_emit_bridge_event() {
    new_tester().execute_with(|| {
        let token_addr = H160::repeat_byte(1);
        let recipient = H160::repeat_byte(2);
        let bob: AccountId = Keyring::Bob.into();

        let event: Payload<TestAccountId> = Payload {
            sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
            recipient_addr: bob.clone(),
            token_addr,
            amount: 20.into(),
        };
        assert_ok!(ERC20::handle_event(event.clone()));

        assert_ok!(ERC20::burn(
            Origin::signed(bob.clone()),
            token_addr,
            recipient,
            20.into()
        ));

        assert_eq!(
            MockEvent::test_events(RawEvent::Transfer(token_addr, bob, recipient, 20.into())),
            last_event()
        );
    });
}
