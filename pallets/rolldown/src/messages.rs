#![allow(non_snake_case)]

use alloy_primitives::FixedBytes;
use alloy_sol_types::SolValue;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::{
	prelude::{format, string::String},
	TypeInfo,
};
use serde::{Deserialize, Serialize};
use sp_core::{RuntimeDebug, H256, U256};
use sp_runtime::SaturatedConversion;
use sp_std::{
	convert::{TryFrom, TryInto},
	vec,
	vec::Vec,
};

#[repr(u8)]
#[derive(
	Copy,
	Eq,
	PartialEq,
	RuntimeDebug,
	Clone,
	Encode,
	Decode,
	TypeInfo,
	MaxEncodedLen,
	Serialize,
	Deserialize,
	Ord,
	PartialOrd,
)]
pub enum Chain {
	Ethereum,
	Arbitrum,
}

impl Default for Chain {
	fn default() -> Self {
		Chain::Ethereum
	}
}

#[repr(u8)]
#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Copy)]
pub enum Origin {
	L1,
	L2,
}

impl Default for Origin {
	fn default() -> Self {
		Origin::L1
	}
}

impl Into<eth_abi::Origin> for Origin {
	fn into(self) -> eth_abi::Origin {
		match self {
			Origin::L1 => eth_abi::Origin::L1,
			Origin::L2 => eth_abi::Origin::L2,
		}
	}
}

impl Into<eth_abi::Chain> for Chain {
	fn into(self) -> eth_abi::Chain {
		match self {
			Chain::Ethereum => eth_abi::Chain::Ethereum,
			Chain::Arbitrum => eth_abi::Chain::Arbitrum,
		}
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
pub struct Range {
	pub start: u128,
	pub end: u128,
}

impl From<(u128, u128)> for Range {
	fn from((start, end): (u128, u128)) -> Range {
		Range { start, end }
	}
}

impl Into<eth_abi::Range> for Range {
	fn into(self) -> eth_abi::Range {
		eth_abi::Range { start: to_eth_u256(self.start.into()), end: to_eth_u256(self.end.into()) }
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Default)]
pub struct RequestId {
	pub origin: Origin,
	pub id: u128,
}

impl RequestId {
	pub fn new(origin: Origin, id: u128) -> Self {
		Self { origin, id }
	}
}

impl From<(Origin, u128)> for RequestId {
	fn from((origin, id): (Origin, u128)) -> RequestId {
		RequestId { origin, id }
	}
}

impl Into<eth_abi::RequestId> for RequestId {
	fn into(self) -> eth_abi::RequestId {
		eth_abi::RequestId { origin: self.origin.into(), id: to_eth_u256(U256::from(self.id)) }
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Default)]
pub struct Deposit {
	pub requestId: RequestId,
	pub depositRecipient: [u8; 20],
	pub tokenAddress: [u8; 20],
	pub amount: sp_core::U256,
	pub timeStamp: sp_core::U256,
}

impl Into<eth_abi::Deposit> for Deposit {
	fn into(self) -> eth_abi::Deposit {
		eth_abi::Deposit {
			requestId: self.requestId.into(),
			depositRecipient: self.depositRecipient.into(),
			tokenAddress: self.tokenAddress.into(),
			amount: to_eth_u256(self.amount),
			timeStamp: to_eth_u256(self.timeStamp),
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


#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
pub struct CancelResolution {
	pub requestId: RequestId,
	pub l2RequestId: u128,
	pub cancelJustified: bool,
	pub timeStamp: sp_core::U256,
}

impl Into<eth_abi::CancelResolution> for CancelResolution {
	fn into(self) -> eth_abi::CancelResolution {
		eth_abi::CancelResolution {
			requestId: self.requestId.into(),
			l2RequestId: to_eth_u256(self.l2RequestId.into()),
			cancelJustified: self.cancelJustified.into(),
			timeStamp: to_eth_u256(self.timeStamp),
		}
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Default, Serialize)]
pub struct FailedWithdrawalResolution {
	pub requestId: RequestId,
	pub l2RequestId: u128,
	pub timeStamp: sp_core::U256,
}

impl Into<eth_abi::FailedWithdrawalResolution> for FailedWithdrawalResolution {
	fn into(self) -> eth_abi::FailedWithdrawalResolution {
		eth_abi::FailedWithdrawalResolution {
			requestId: self.requestId.into(),
			l2RequestId: to_eth_u256(self.l2RequestId.into()),
			timeStamp: to_eth_u256(self.timeStamp),
		}
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, Default, TypeInfo, Serialize)]
pub struct L1Update {
	pub chain: Chain,
	pub pendingDeposits: Vec<Deposit>,
	pub pendingCancelResolutions: Vec<CancelResolution>,
	pub pendingFailedWithdrawalResolutions: Vec<FailedWithdrawalResolution>,
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
pub enum L1UpdateRequest {
	Deposit(Deposit),
	CancelResolution(CancelResolution),
	FailedWithdrawalResolution(FailedWithdrawalResolution),
}

impl L1UpdateRequest {
	pub fn request_id(&self) -> RequestId {
		match self {
			L1UpdateRequest::Deposit(deposit) => deposit.requestId.clone(),
			L1UpdateRequest::CancelResolution(cancel) => cancel.requestId.clone(),
			L1UpdateRequest::FailedWithdrawalResolution(withdrawal) => withdrawal.requestId.clone(),
		}
	}

	pub fn id(&self) -> u128 {
		match self {
			L1UpdateRequest::Deposit(deposit) => deposit.requestId.id.clone(),
			L1UpdateRequest::CancelResolution(cancel) => cancel.requestId.id.clone(),
			L1UpdateRequest::FailedWithdrawalResolution(withdrawal) => withdrawal.requestId.id.clone(),
		}
	}

	pub fn origin(&self) -> Origin {
		match self {
			L1UpdateRequest::Deposit(deposit) => deposit.requestId.origin.clone(),
			L1UpdateRequest::CancelResolution(cancel) => cancel.requestId.origin.clone(),
			L1UpdateRequest::FailedWithdrawalResolution(withdrawal) =>
				withdrawal.requestId.origin.clone(),
		}
	}
}

impl L1Update {
	pub fn range(&self) -> Option<Range> {
		let first = [
			self.pendingDeposits.first().map(|v| v.requestId.id),
			self.pendingCancelResolutions.first().map(|v| v.requestId.id),
			self.pendingFailedWithdrawalResolutions.first().map(|v| v.requestId.id),
		]
		.into_iter()
		.cloned()
		.filter_map(|v| v)
		.min();

		let last = [
			self.pendingDeposits.last().map(|v| v.requestId.id),
			self.pendingCancelResolutions.last().map(|v| v.requestId.id),
			self.pendingFailedWithdrawalResolutions.last().map(|v| v.requestId.id),
		]
		.into_iter()
		.cloned()
		.filter_map(|v| v)
		.max();
		if let (Some(first), Some(last)) = (first, last) {
			Some(Range { start: first, end: last })
		} else {
			None
		}
	}

	pub fn into_requests(self) -> Vec<L1UpdateRequest> {
		let mut result: Vec<L1UpdateRequest> = Default::default();

		let L1Update {
			pendingDeposits,
			pendingCancelResolutions,
			pendingFailedWithdrawalResolutions,
			chain,
		} = self;

		let mut deposits_it = pendingDeposits.into_iter().peekable();
		let mut cancel_it = pendingCancelResolutions.into_iter().peekable();
		let mut withdrawal_it = pendingFailedWithdrawalResolutions.into_iter().peekable();

		loop {
			let min = [
				deposits_it.peek().map(|v| v.requestId.id),
				cancel_it.peek().map(|v| v.requestId.id),
				withdrawal_it.peek().map(|v| v.requestId.id),
			]
			.into_iter()
			.cloned()
			.filter_map(|v| v)
			.min();

			match (
				deposits_it.peek(),
				cancel_it.peek(),
				withdrawal_it.peek(),
				min,
			) {
				(Some(deposit), _, _, Some(min)) if deposit.requestId.id == min => {
					if let Some(elem) = deposits_it.next() {
						result.push(L1UpdateRequest::Deposit(elem.clone()));
					}
				},
				(_, Some(cancel), _, Some(min)) if cancel.requestId.id == min => {
					if let Some(elem) = cancel_it.next() {
						result.push(L1UpdateRequest::CancelResolution(elem.clone()));
					}
				},
				(_, _, Some(withdrawal), Some(min)) if withdrawal.requestId.id == min => {
					if let Some(elem) = withdrawal_it.next() {
						result.push(L1UpdateRequest::FailedWithdrawalResolution(elem.clone()));
					}
				},
				_ => break,
			}
		}
		result
	}
}

pub fn to_eth_u256(value: U256) -> alloy_primitives::U256 {
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

impl Into<eth_abi::L1Update> for L1Update {
	fn into(self) -> eth_abi::L1Update {
		eth_abi::L1Update {
			chain: self.chain.into(),
			pendingDeposits: self.pendingDeposits.into_iter().map(Into::into).collect::<Vec<_>>(),
			pendingCancelResolutions: self
				.pendingCancelResolutions
				.into_iter()
				.map(Into::into)
				.collect::<Vec<_>>(),
			pendingFailedWithdrawalResolutions: self
				.pendingFailedWithdrawalResolutions
				.into_iter()
				.map(Into::into)
				.collect::<Vec<_>>(),
		}
	}
}

impl TryFrom<eth_abi::FailedWithdrawalResolution> for FailedWithdrawalResolution {
	type Error = String;

	fn try_from(value: eth_abi::FailedWithdrawalResolution) -> Result<Self, Self::Error> {
		let request_id = value
			.requestId
			.try_into()
			.map_err(|e| format!("Error converting requestId: {}", e))?;
		let l2RequestId = value
			.l2RequestId
			.try_into()
			.map_err(|e| format!("Error converting l2RequestId: {}", e))?;

		Ok(Self {
			requestId: request_id,
			l2RequestId,
			timeStamp: from_eth_u256(value.timeStamp),
		})
	}
}

impl TryFrom<eth_abi::Deposit> for Deposit {
	type Error = String;

	fn try_from(deposit: eth_abi::Deposit) -> Result<Self, Self::Error> {
		let request_id = deposit.requestId.try_into();
		let deposit_recipient = deposit.depositRecipient.try_into();
		let token_address = deposit.tokenAddress.try_into();

		Ok(Self {
			requestId: request_id.map_err(|e| format!("Error converting requestId: {}", e))?,
			depositRecipient: deposit_recipient
				.map_err(|e| format!("Error converting depositRecipient: {}", e))?,
			tokenAddress: token_address
				.map_err(|e| format!("Error converting tokenAddress: {}", e))?,
			amount: from_eth_u256(deposit.amount),
			timeStamp: from_eth_u256(deposit.timeStamp),
		})
	}
}

impl TryFrom<eth_abi::L1Update> for L1Update {
	type Error = String;

	fn try_from(update: eth_abi::L1Update) -> Result<Self, Self::Error> {
		let pending_deposits: Result<Vec<_>, _> =
			update.pendingDeposits.into_iter().map(|d| d.try_into()).collect();
		let pending_cancel_resultions: Result<Vec<_>, _> =
			update.pendingCancelResolutions.into_iter().map(|c| c.try_into()).collect();
		let pending_withdrawal_resolutions: Result<Vec<_>, _> =
			update.pendingFailedWithdrawalResolutions.into_iter().map(|u| u.try_into()).collect();

		Ok(Self {
			chain: update.chain.try_into()?,
			pendingDeposits: pending_deposits
				.map_err(|e| format!("Error converting pendingDeposits: {}", e))?,
			pendingCancelResolutions: pending_cancel_resultions
				.map_err(|e| format!("Error converting pendingCancelResolutions: {}", e))?,
			pendingFailedWithdrawalResolutions: pending_withdrawal_resolutions
				.map_err(|e| format!("Error converting pendingFailedWithdrawalResolutions: {}", e))?,
		})
	}
}

impl TryFrom<eth_abi::RequestId> for RequestId {
	type Error = String; // Change to appropriate error type

	fn try_from(request_id: eth_abi::RequestId) -> Result<Self, Self::Error> {
		let origin = request_id.origin.try_into();
		let id: Result<u128, _> = request_id.id.try_into();

		Ok(Self {
			origin: origin.map_err(|e| format!("Error converting origin: {}", e))?,
			id: id.map_err(|e| format!("Error converting id: {}", e))?,
		})
	}
}

impl TryFrom<eth_abi::Origin> for Origin {
	type Error = String;

	fn try_from(origin: eth_abi::Origin) -> Result<Self, Self::Error> {
		match origin {
			eth_abi::Origin::L1 => Ok(Origin::L1),
			eth_abi::Origin::L2 => Ok(Origin::L2),
			_ => Err(String::from("Invalid origin type")),
		}
	}
}

impl TryFrom<eth_abi::Chain> for Chain {
	type Error = String;

	fn try_from(origin: eth_abi::Chain) -> Result<Self, Self::Error> {
		match origin {
			eth_abi::Chain::Ethereum => Ok(Chain::Ethereum),
			eth_abi::Chain::Arbitrum => Ok(Chain::Arbitrum),
			_ => Err(String::from("Invalid origin type")),
		}
	}
}

impl TryFrom<eth_abi::CancelResolution> for CancelResolution {
	type Error = String;

	fn try_from(value: eth_abi::CancelResolution) -> Result<Self, Self::Error> {
		let request_id: RequestId = value
			.requestId
			.try_into()
			.map_err(|e| format!("Error converting requestId: {}", e))?;
		let l2_request_id = value.l2RequestId.try_into();

		Ok(Self {
			requestId: request_id,
			l2RequestId: l2_request_id
				.map_err(|e| format!("Error converting l2_request_id: {}", e))?,
			cancelJustified: value.cancelJustified,
			timeStamp: from_eth_u256(value.timeStamp),
		})
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

impl Into<eth_abi::Cancel> for Cancel<[u8; 20]> {
	fn into(self) -> eth_abi::Cancel {
		todo!()
	}
}


pub mod eth_abi {
	use alloy_sol_types::sol;
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;
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
		struct FailedDeposit {
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
}
