use alloy_sol_types::sol;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_std::vec::Vec;

pub fn to_eth_u256(value: sp_core::U256) -> alloy_primitives::U256 {
	let mut bytes = [0u8; 32];
	value.to_big_endian(&mut bytes);
	alloy_primitives::U256::from_be_bytes(bytes)
}

pub fn from_eth_u256(value: alloy_primitives::U256) -> U256 {
	let bytes: [u8; 32] = value.to_be_bytes();
	let mut buf = [0u8; 32];
	buf.copy_from_slice(&bytes[..]);
	U256::from_big_endian(&buf)
}

impl<AccountId: Clone> From<crate::Cancel<AccountId>> for Cancel {
	fn from(cancel: crate::Cancel<AccountId>) -> Self {
		Self {
			requestId: cancel.requestId.into(),
			range: cancel.range.into(),
			hash: alloy_primitives::FixedBytes::<32>::from_slice(&cancel.hash[..]),
		}
	}
}

impl From<crate::FailedDepositResolution> for FailedDepositResolution {
	fn from(failed_deposit_resolution: crate::FailedDepositResolution) -> Self {
		Self {
			requestId: failed_deposit_resolution.requestId.into(),
			originRequestId: crate::messages::to_eth_u256(
				failed_deposit_resolution.originRequestId.into(),
			),
			ferry: failed_deposit_resolution.ferry.into(),
		}
	}
}

impl From<crate::Withdrawal> for Withdrawal {
	fn from(withdrawal: crate::Withdrawal) -> Self {
		Self {
			requestId: withdrawal.requestId.into(),
			withdrawalRecipient: withdrawal.withdrawalRecipient.into(),
			tokenAddress: withdrawal.tokenAddress.into(),
			amount: crate::messages::to_eth_u256(withdrawal.amount.into()),
			ferryTip: crate::messages::to_eth_u256(withdrawal.ferryTip.into()),
		}
	}
}

impl From<crate::messages::Chain> for Chain {
	fn from(c: crate::messages::Chain) -> Chain {
		match c {
			crate::messages::Chain::Ethereum => Chain::Ethereum,
			crate::messages::Chain::Arbitrum => Chain::Arbitrum,
		}
	}
}

impl From<crate::messages::Origin> for Origin {
	fn from(c: crate::messages::Origin) -> Origin {
		match c {
			crate::messages::Origin::L1 => Origin::L1,
			crate::messages::Origin::L2 => Origin::L2,
		}
	}
}

impl From<crate::messages::Range> for Range {
	fn from(range: crate::messages::Range) -> Range {
		Range { start: to_eth_u256(range.start.into()), end: to_eth_u256(range.end.into()) }
	}
}

impl From<crate::messages::RequestId> for RequestId {
	fn from(rid: crate::messages::RequestId) -> RequestId {
		RequestId { origin: rid.origin.into(), id: to_eth_u256(U256::from(rid.id)) }
	}
}

impl From<crate::messages::CancelResolution> for CancelResolution {
	fn from(cancel: crate::messages::CancelResolution) -> CancelResolution {
		CancelResolution {
			requestId: cancel.requestId.into(),
			l2RequestId: to_eth_u256(cancel.l2RequestId.into()),
			cancelJustified: cancel.cancelJustified.into(),
			timeStamp: to_eth_u256(cancel.timeStamp),
		}
	}
}

impl From<crate::messages::L1Update> for L1Update {
	fn from(update: crate::messages::L1Update) -> L1Update {
		L1Update {
			chain: update.chain.into(),
			pendingDeposits: update.pendingDeposits.into_iter().map(Into::into).collect::<Vec<_>>(),
			pendingCancelResolutions: update
				.pendingCancelResolutions
				.into_iter()
				.map(Into::into)
				.collect::<Vec<_>>(),
		}
	}
}

impl From<crate::messages::Deposit> for Deposit {
	fn from(deposit: crate::messages::Deposit) -> Deposit {
		Deposit {
			requestId: deposit.requestId.into(),
			depositRecipient: deposit.depositRecipient.into(),
			tokenAddress: deposit.tokenAddress.into(),
			amount: to_eth_u256(deposit.amount),
			timeStamp: to_eth_u256(deposit.timeStamp),
			ferryTip: to_eth_u256(deposit.ferryTip),
		}
	}
}

sol! {
	// L1 to L2
	#[derive(Debug)]
	struct Deposit {
		RequestId requestId;
		address depositRecipient;
		address tokenAddress;
		uint256 amount;
		uint256 timeStamp;
		uint256 ferryTip;
	}


	#[derive(Debug)]
	struct CancelResolution {
		RequestId requestId;
		uint256 l2RequestId;
		bool cancelJustified;
		uint256 timeStamp;
	}

	#[derive(Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
	enum Chain{ Ethereum, Arbitrum }

	#[derive(Debug)]
	struct L1Update {
		Chain chain;
		Deposit[] pendingDeposits;
		CancelResolution[] pendingCancelResolutions;
	}

	#[derive(Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
	enum Origin{ L1, L2 }

	#[derive(Debug, Eq, PartialEq)]
	struct RequestId {
		Origin origin;
		uint256 id;
	}

	#[derive(Debug, PartialEq)]
	struct FailedDepositResolution {
		RequestId requestId;
		uint256 originRequestId;
		address ferry;
	}

	#[derive(Debug, PartialEq)]
	struct Withdrawal {
		RequestId requestId;
		address withdrawalRecipient;
		address tokenAddress;
		uint256 amount;
		uint256 ferryTip;
	}

	#[derive(Debug, PartialEq)]
	struct Range{
		uint256 start;
		uint256 end;
	}

	#[derive(Debug, PartialEq)]
	struct Cancel {
		RequestId requestId;
		Range range;
		bytes32 hash;
	}

}

#[cfg(test)]
mod test {
	use super::*;
	use alloy_sol_types::SolValue;
	use hex_literal::hex;
	use serial_test::serial;
	use sp_crypto_hashing::keccak_256;

	pub trait Keccak256Hash {
		fn keccak256_hash(&self) -> [u8; 32];
	}

	impl<T> Keccak256Hash for T
	where
		T: SolValue,
	{
		fn keccak256_hash(&self) -> [u8; 32] {
			Into::<[u8; 32]>::into(keccak_256(&self.abi_encode()[..]))
		}
	}

	#[test]
	fn test_conversion_u256__small() {
		let val = sp_core::U256::from(1u8);
		let eth_val = alloy_primitives::U256::from(1u8);
		assert_eq!(to_eth_u256(val), eth_val);
	}

	#[test]
	fn test_conversion_u256__big() {
		let val = sp_core::U256::from([u8::MAX; 32]);
		assert_eq!(from_eth_u256(to_eth_u256(val)), val);
	}

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// NOTE: Below hash values should not be ever chaned, there are comaptible test implemented in eigen monorepo
	/// to ensure abi compatibility between L1 & L2
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	#[test]
	#[serial]
	// this test ensures that the hash calculated on rust side matches hash calculated in contract
	fn test_l1_update_hash_compare_with_solidty() {
		assert_eq!(
			L1Update {
				chain: Chain::Ethereum,
				pendingDeposits: vec![Deposit {
					requestId: RequestId {
						origin: Origin::L1,
						id: alloy_primitives::U256::from(1)
					},
					depositRecipient: hex!("1111111111111111111111111111111111111111").into(),
					tokenAddress: hex!("2222222222222222222222222222222222222222").into(),
					amount: alloy_primitives::U256::from(123456),
					timeStamp: alloy_primitives::U256::from(987),
					ferryTip: alloy_primitives::U256::from(321987)
				}],
				pendingCancelResolutions: vec![CancelResolution {
					requestId: RequestId {
						origin: Origin::L1,
						id: alloy_primitives::U256::from(123)
					},
					l2RequestId: alloy_primitives::U256::from(123456),
					cancelJustified: true,
					timeStamp: alloy_primitives::U256::from(987)
				}],
			}
			.keccak256_hash(),
			hex!("663fa3ddfe64659f67b2728637936fa8d21f18ef96c07dec110cdd8f45be6fee"),
		);
	}

	#[test]
	#[serial]
	fn test_calculate_chain_hash() {
		assert_eq!(
			Chain::Ethereum.keccak256_hash(),
			hex!("290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563"),
		);

		assert_eq!(
			Chain::Arbitrum.keccak256_hash(),
			hex!("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6"),
		);
	}

	#[test]
	#[serial]
	fn test_calculate_withdrawal_hash() {
		assert_eq!(
			Withdrawal {
				requestId: RequestId { origin: Origin::L2, id: alloy_primitives::U256::from(123) },
				withdrawalRecipient: hex!("ffffffffffffffffffffffffffffffffffffffff").into(),
				tokenAddress: hex!("1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f").into(),
				amount: alloy_primitives::U256::from(123456),
				ferryTip: alloy_primitives::U256::from(465789)
			}
			.keccak256_hash(),
			hex!("a931da68c445f23b06a72768d07a3513f85c0118ff80f6e284117a221869ae8b"),
		);
	}

	#[test]
	#[serial]
	fn test_calculate_failed_deposit_resolution_hash() {
		assert_eq!(
			FailedDepositResolution {
				requestId: RequestId { origin: Origin::L1, id: alloy_primitives::U256::from(123) },
				originRequestId: alloy_primitives::U256::from(1234),
				ferry: hex!("b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5").into()
			}
			.keccak256_hash(),
			hex!("d3def31efb42dd99500c389f59115f0eef5e008db0ee0a81562ef3acbe02eece"),
		);
	}
}
