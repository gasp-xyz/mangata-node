#![allow(non_snake_case)]
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::Serialize;
use sp_core::{RuntimeDebug, H256, U256};
use sp_std::vec::Vec;

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
pub struct Deposit {
	pub depositRecipient: [u8; 20],
	pub tokenAddress: [u8; 20],
	pub amount: sp_core::U256,
}

impl Into<eth_abi::Deposit> for Deposit {
	fn into(self) -> eth_abi::Deposit {
		eth_abi::Deposit {
			depositRecipient: self.depositRecipient.into(),
			tokenAddress: self.tokenAddress.into(),
			amount: to_eth_u256(self.amount),
		}
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
#[allow(non_snake_case)]
pub struct Withdraw {
	pub depositRecipient: [u8; 20],
	pub tokenAddress: [u8; 20],
	pub amount: sp_core::U256,
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
pub struct L2UpdatesToRemove {
	pub l2UpdatesToRemove: Vec<sp_core::U256>,
}

impl Into<eth_abi::L2UpdatesToRemove> for L2UpdatesToRemove {
	fn into(self) -> eth_abi::L2UpdatesToRemove {
		eth_abi::L2UpdatesToRemove {
			l2UpdatesToRemove: self
				.l2UpdatesToRemove
				.into_iter()
				.map(|req_id| to_eth_u256(req_id))
				.collect(),
		}
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
pub struct CancelResolution {
	pub l2RequestId: sp_core::U256,
	pub cancelJustified: bool,
}

impl Into<eth_abi::CancelResolution> for CancelResolution {
	fn into(self) -> eth_abi::CancelResolution {
		eth_abi::CancelResolution {
			l2RequestId: to_eth_u256(self.l2RequestId),
			cancelJustified: self.cancelJustified.into(),
		}
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
#[repr(u8)]
pub enum PendingRequestType {
	DEPOSIT,
	CANCEL_RESOLUTION,
	L2_UPDATES_TO_REMOVE,
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
pub struct L1Update {
	pub lastProccessedRequestOnL1: sp_core::U256,
	pub lastAcceptedRequestOnL1: sp_core::U256,
	pub offset: sp_core::U256,
	pub order: Vec<PendingRequestType>,
	pub pendingDeposits: Vec<Deposit>,
	pub pendingCancelResultions: Vec<CancelResolution>,
	pub pendingL2UpdatesToRemove: Vec<L2UpdatesToRemove>,
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
pub enum L1UpdateRequest {
	Deposit(Deposit),
	Cancel(CancelResolution),
	Remove(L2UpdatesToRemove),
}

impl Into<eth_abi::PendingRequestType> for PendingRequestType {
	fn into(self) -> eth_abi::PendingRequestType {
		match self {
			PendingRequestType::DEPOSIT => eth_abi::PendingRequestType::DEPOSIT,
			PendingRequestType::CANCEL_RESOLUTION => eth_abi::PendingRequestType::CANCEL_RESOLUTION,
			PendingRequestType::L2_UPDATES_TO_REMOVE =>
				eth_abi::PendingRequestType::L2_UPDATES_TO_REMOVE,
		}
	}
}

impl L1Update {
	pub fn into_requests(self) -> Vec<(sp_core::U256, L1UpdateRequest)> {
		let L1Update {
			offset,
			order,
			mut pendingDeposits,
			mut pendingCancelResultions,
			mut pendingL2UpdatesToRemove,
			..
		} = self;

		order
			.into_iter()
			.enumerate()
			.map(|(request_id, request_type)| {
				(
					sp_core::U256::from(request_id) + offset,
					match request_type {
						PendingRequestType::DEPOSIT =>
							L1UpdateRequest::Deposit(pendingDeposits.pop().unwrap()),
						PendingRequestType::CANCEL_RESOLUTION =>
							L1UpdateRequest::Cancel(pendingCancelResultions.pop().unwrap()),
						PendingRequestType::L2_UPDATES_TO_REMOVE =>
							L1UpdateRequest::Remove(pendingL2UpdatesToRemove.pop().unwrap()),
					},
				)
			})
			.collect()
	}
}

pub fn to_eth_u256(value: U256) -> alloy_primitives::U256 {
	let mut bytes = [0u8; 32];
	value.to_big_endian(&mut bytes);
	alloy_primitives::U256::from_be_bytes(bytes)
}

impl Into<eth_abi::L1Update> for L1Update {
	fn into(self) -> eth_abi::L1Update {
		eth_abi::L1Update {
			lastProccessedRequestOnL1: to_eth_u256(self.lastProccessedRequestOnL1),
			lastAcceptedRequestOnL1: to_eth_u256(self.lastAcceptedRequestOnL1),
			offset: to_eth_u256(self.offset),
			order: self.order.into_iter().map(Into::into).collect::<Vec<_>>(),
			pendingDeposits: self.pendingDeposits.into_iter().map(Into::into).collect::<Vec<_>>(),
			pendingCancelResultions: self
				.pendingCancelResultions
				.into_iter()
				.map(Into::into)
				.collect::<Vec<_>>(),
			pendingL2UpdatesToRemove: self
				.pendingL2UpdatesToRemove
				.into_iter()
				.map(Into::into)
				.collect::<Vec<_>>(),
		}
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
pub struct Cancel<AccountId> {
	pub updater: AccountId,
	pub canceler: AccountId,
	pub lastProccessedRequestOnL1: U256,
	pub lastAcceptedRequestOnL1: U256,
	pub hash: H256,
}

pub use eth_abi::{L2Update, UpdateType};

pub mod eth_abi {
	use alloy_sol_types::sol;
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;
	sol! {
		// L1 to L2
		struct Deposit {
			address depositRecipient;
			address tokenAddress;
			uint256 amount;
		}

		struct L2UpdatesToRemove {
			uint256[] l2UpdatesToRemove;
		}

		struct CancelResolution {
			uint256 l2RequestId;
			bool cancelJustified;
		}

		enum PendingRequestType{ DEPOSIT, CANCEL_RESOLUTION, L2_UPDATES_TO_REMOVE}

		struct L1Update {
			uint256 lastProccessedRequestOnL1;
			uint256 lastAcceptedRequestOnL1;
			uint256 offset;
			PendingRequestType[] order;
			Deposit[] pendingDeposits;
			CancelResolution[] pendingCancelResultions;
			L2UpdatesToRemove[] pendingL2UpdatesToRemove;
		}


		#[derive(Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
		enum UpdateType{ DEPOSIT, WITHDRAWAL, INDEX_UPDATE, CANCEL_RESOLUTION}

		// L2 to L1
		#[derive(Debug, PartialEq)]
		struct RequestResult {
			uint256 requestId;
			UpdateType updateType;
			bool status;
		}

		#[derive(Debug, PartialEq)]
		struct Withdrawal {
			uint256 l2RequestId;
			address withdrawalRecipient;
			address tokenAddress;
			uint256 amount;
		}

		#[derive(Debug, PartialEq)]
		struct Cancel {
			uint256 l2RequestId;
			uint256 lastProccessedRequestOnL1;
			uint256 lastAcceptedRequestOnL1;
			bytes32 hash;
		}

		#[derive(Debug, PartialEq)]
		struct L2Update {
			Cancel[] cancels;
			Withdrawal[] withdrawals;
			RequestResult[] results;
		}
	}
}
