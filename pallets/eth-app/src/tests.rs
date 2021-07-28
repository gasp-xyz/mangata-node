use super::*;
use crate::mock::{new_tester, AccountId, MockEvent, MockRuntime, Origin, System, Tokens, ETH};
use crate::RawEvent;
use codec::Decode;
use frame_support::assert_err;
use frame_support::assert_ok;
use frame_system as system;
use hex_literal::hex;
use orml_tokens::MultiTokenCurrency;
use sp_core::H160;
use sp_core::U256;
use sp_keyring::AccountKeyring as Keyring;

use crate::payload::Payload;

fn last_event() -> MockEvent {
    System::events().pop().expect("Event expected").event
}

const RECIPIENT_ADDR_BYTES: [u8; 32] =
    hex!["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"];

type TestAccountId = <MockRuntime as system::Trait>::AccountId;

#[test]
fn mints_after_handling_ethereum_event() {
    new_tester().execute_with(|| {
        let bob: AccountId = Keyring::Bob.into();
        let id_of_first_minted_token = 0;

        let recipient_addr = TestAccountId::decode(&mut &RECIPIENT_ADDR_BYTES[..]).unwrap();
        let event: Payload<TestAccountId> = Payload {
            sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
            recipient_addr,
            amount: 10.into(),
        };

        // crating token with ID = 0
        assert_ok!(ETH::handle_event(event.clone()));
        assert_eq!(Tokens::free_balance(id_of_first_minted_token, &bob), 10);

        // minting previously created token
        assert_ok!(ETH::handle_event(event));
        assert_eq!(Tokens::free_balance(id_of_first_minted_token, &bob), 20);
    });
}

#[test]
fn burn_should_emit_bridge_event() {
    new_tester().execute_with(|| {
        let recipient = H160::repeat_byte(2);
        let bob: AccountId = Keyring::Bob.into();

        // mint tokens
        let event: Payload<TestAccountId> = Payload {
            sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
            recipient_addr: bob.clone(),
            amount: 500.into(),
        };
        assert_ok!(ETH::handle_event(event.clone()));

        assert_ok!(ETH::burn(Origin::signed(bob.clone()), recipient, 20.into()));

        assert_eq!(
            MockEvent::test_events(RawEvent::Transfer(bob, recipient, 20.into())),
            last_event()
        );
    });
}

#[test]
fn handle_event_should_return_error_on_overflow() {
    new_tester().execute_with(|| {
        let event: Payload<TestAccountId> = Payload {
            sender_addr: H160::repeat_byte(1),
            recipient_addr: Keyring::Bob.into(),
            amount: U256::max_value(),
        };

        assert_err!(
            ETH::handle_event(event.clone()),
            Error::<MockRuntime>::TooBigAmount,
        );
    });
}
