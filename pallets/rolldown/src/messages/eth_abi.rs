use alloy_sol_types::{sol, SolType, SolValue};
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
			originRequestId: crate::messages::to_eth_u256(failed_deposit_resolution.originRequestId.into()),
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
	fn from(range: crate::messages::Range) -> Range{
		Range { start: to_eth_u256(range.start.into()), end: to_eth_u256(range.end.into()) }
	}
}


impl From<crate::messages::RequestId> for RequestId {
	fn from(rid: crate::messages::RequestId) -> RequestId {
		RequestId {
			origin: rid.origin.into(),
			id: to_eth_u256(U256::from(rid.id))
		}
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

impl From<crate::messages::FailedWithdrawalResolution> for FailedWithdrawalResolution {
	fn from(resolution: crate::messages::FailedWithdrawalResolution) -> FailedWithdrawalResolution {
		FailedWithdrawalResolution {
			requestId: resolution.requestId.into(),
			l2RequestId: to_eth_u256(resolution.l2RequestId.into()),
			timeStamp: to_eth_u256(resolution.timeStamp),
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
			pendingFailedWithdrawalResolutions: update
				.pendingFailedWithdrawalResolutions
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
	}

	#[derive(Debug)]
	struct FailedWithdrawalResolution {
		RequestId requestId;
		uint256 l2RequestId;
		uint256 timeStamp;
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
		FailedWithdrawalResolution[] pendingFailedWithdrawalResolutions;
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
	}

	#[derive(Debug, PartialEq)]
	struct Withdrawal {
		RequestId requestId;
		address withdrawalRecipient;
		address tokenAddress;
		uint256 amount;
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
