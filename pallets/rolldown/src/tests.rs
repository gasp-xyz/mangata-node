use frame_support::{assert_ok, assert_err};
use sp_core::crypto::Ss58Codec;

use crate::*;


#[test]
fn parse_ethereum_address_from_20bytes_hex_string() {
	assert_ok!(EthereumAddressConverter::try_convert("0x0000000000000000000000000000000000000000".to_string()));
	assert_ok!(EthereumAddressConverter::try_convert("0xbd0f320b5343c5d52f18b85dd1a1d0f6844fbb1a".to_string()));
}

#[test]
fn parse_ethereum_address_from_20bytes_hex_string_without_prefix() {
	assert_ok!(EthereumAddressConverter::try_convert("0000000000000000000000000000000000000000".to_string()));
	assert_ok!(EthereumAddressConverter::try_convert("bd0f320b5343c5d52f18b85dd1a1d0f6844fbb1a".to_string()));
}

#[test]
fn parse_ethereum_address_from_20bytes_hex_string_without_and_without_capital_leters() {
	assert_ok!(EthereumAddressConverter::try_convert("0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string()));
	assert_ok!(EthereumAddressConverter::try_convert("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()));
}

#[test]
fn parse_ethereum_address_from_too_short_string_fails() {
	assert!(
		EthereumAddressConverter::try_convert("0x000000000000000000000000000000000000000".to_string()).is_err()
	);
}

#[test]
fn parse_ethereum_address_from_too_long_string_fails() {
	assert!(
		EthereumAddressConverter::try_convert("0x000000000000000000000000000000000000000000".to_string()).is_err()
	);
}

#[test]
fn ethereum_address_is_convertible_to_dot_address() {
	let account = EthereumAddressConverter::try_convert("0xbd0f320b5343c5d52f18b85dd1a1d0f6844fbb1a".to_string()).unwrap();

	assert_eq!(
		account.to_ss58check(),
		"5EawKqk4DuwYfbZzTdTBAtx7Srxupto6Y2tvdw5kzLfANXNQ".to_string()
	)
}


