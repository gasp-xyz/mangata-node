#![allow(non_snake_case)]

pub mod eth_abi;

use self::eth_abi::to_eth_u256;
use crate::L2Request;
use alloy_sol_types::SolValue;
use codec::{Decode, Encode, MaxEncodedLen};
use eth_abi::from_eth_u256;
use scale_info::{
	prelude::{format, string::String},
	TypeInfo,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use sp_core::{RuntimeDebug, H256, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	vec::Vec,
};

pub trait NativeToEthMapping {
	type EthType: SolValue;
}

pub trait EthAbi {
	fn abi_encode(&self) -> Vec<u8>;
}

pub trait EthAbiHash {
	fn abi_encode_hash(&self) -> H256;
}

impl<T> EthAbiHash for T
where
	T: EthAbi,
{
	fn abi_encode_hash(&self) -> H256 {
		let encoded = self.abi_encode();
		let hash: [u8; 32] = Keccak256::digest(&encoded[..]).into();
		H256::from(hash)
	}
}

impl<T> EthAbi for T
where
	T: Clone,
	T: NativeToEthMapping,
	T: Into<T::EthType>,
	T::EthType: SolValue,
{
	fn abi_encode(&self) -> Vec<u8> {
		let eth_type: <T as NativeToEthMapping>::EthType = (self.clone()).into();
		eth_type.abi_encode()
	}
}

impl<AccountId: Clone> EthAbi for L2Request<AccountId> {
	fn abi_encode(&self) -> Vec<u8> {
		match self {
			L2Request::FailedDepositResolution(deposit) => deposit.abi_encode(),
			L2Request::Cancel(cancel) => cancel.abi_encode(),
			L2Request::Withdrawal(withdrawal) => withdrawal.abi_encode(),
		}
	}
}

#[repr(u8)]
#[derive(
	Default,
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
	#[default]
	Ethereum,
	Arbitrum,
}

#[repr(u8)]
#[derive(
	Default, Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Copy,
)]
pub enum Origin {
	#[default]
	L1,
	L2,
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Copy)]
pub struct Range {
	pub start: u128,
	pub end: u128,
}

impl From<(u128, u128)> for Range {
	fn from((start, end): (u128, u128)) -> Range {
		Range { start, end }
	}
}

#[derive(
	Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Default, Copy,
)]
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

// L2 to L1

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Copy)]
pub struct FailedDepositResolution {
	pub requestId: RequestId,
	pub originRequestId: u128,
}

#[derive(
	Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Default, Copy,
)]
pub struct Withdrawal {
	pub requestId: RequestId,
	pub withdrawalRecipient: [u8; 20],
	pub tokenAddress: [u8; 20],
	pub amount: U256,
	pub ferryTip: U256,
}

impl NativeToEthMapping for Withdrawal {
	type EthType = eth_abi::Withdrawal;
}

impl<AccountId: Clone> From<Withdrawal> for crate::L2Request<AccountId> {
	fn from(w: Withdrawal) -> crate::L2Request<AccountId> {
		crate::L2Request::Withdrawal(w)
	}
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Copy)]
pub struct Cancel<AccountId: Clone> {
	pub requestId: RequestId,
	pub updater: AccountId,
	pub canceler: AccountId,
	pub range: Range,
	pub hash: H256,
}

impl<AccountId> NativeToEthMapping for Cancel<AccountId>
where
	Self: Clone,
	AccountId: Clone,
{
	type EthType = eth_abi::Cancel;
}

impl<AccountId: Clone> From<Cancel<AccountId>> for crate::L2Request<AccountId> {
	fn from(cancel: Cancel<AccountId>) -> crate::L2Request<AccountId> {
		crate::L2Request::Cancel(cancel)
	}
}

// L1 to L2 messages
#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize, Default)]
pub struct Deposit {
	pub requestId: RequestId,
	pub depositRecipient: [u8; 20],
	pub tokenAddress: [u8; 20],
	pub amount: U256,
	pub timeStamp: U256,
	pub ferryTip: U256,
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
pub struct CancelResolution {
	pub requestId: RequestId,
	pub l2RequestId: u128,
	pub cancelJustified: bool,
	pub timeStamp: sp_core::U256,
}

impl NativeToEthMapping for FailedDepositResolution {
	type EthType = eth_abi::FailedDepositResolution;
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, Default, TypeInfo, Serialize)]
pub struct L1Update {
	pub chain: Chain,
	pub pendingDeposits: Vec<Deposit>,
	pub pendingCancelResolutions: Vec<CancelResolution>,
}

#[derive(Eq, PartialEq, RuntimeDebug, Clone, Encode, Decode, TypeInfo, Serialize)]
pub enum L1UpdateRequest {
	Deposit(Deposit),
	CancelResolution(CancelResolution),
}

impl L1UpdateRequest {
	pub fn request_id(&self) -> RequestId {
		match self {
			L1UpdateRequest::Deposit(deposit) => deposit.requestId.clone(),
			L1UpdateRequest::CancelResolution(cancel) => cancel.requestId.clone(),
		}
	}

	pub fn id(&self) -> u128 {
		match self {
			L1UpdateRequest::Deposit(deposit) => deposit.requestId.id.clone(),
			L1UpdateRequest::CancelResolution(cancel) => cancel.requestId.id.clone(),
		}
	}

	pub fn origin(&self) -> Origin {
		match self {
			L1UpdateRequest::Deposit(deposit) => deposit.requestId.origin.clone(),
			L1UpdateRequest::CancelResolution(cancel) => cancel.requestId.origin.clone(),
		}
	}
}

impl L1Update {
	pub fn range(&self) -> Option<Range> {
		let first = [
			self.pendingDeposits.first().map(|v| v.requestId.id),
			self.pendingCancelResolutions.first().map(|v| v.requestId.id),
		]
		.iter()
		.cloned()
		.filter_map(|v| v)
		.min();

		let last = [
			self.pendingDeposits.last().map(|v| v.requestId.id),
			self.pendingCancelResolutions.last().map(|v| v.requestId.id),
		]
		.iter()
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

		let L1Update { chain, pendingDeposits, pendingCancelResolutions } = self;
		let _ = chain;

		let mut deposits_it = pendingDeposits.into_iter().peekable();
		let mut cancel_it = pendingCancelResolutions.into_iter().peekable();

		loop {
			let min = [
				deposits_it.peek().map(|v| v.requestId.id),
				cancel_it.peek().map(|v| v.requestId.id),
			]
			.iter()
			.cloned()
			.filter_map(|v| v)
			.min();

			match (deposits_it.peek(), cancel_it.peek(), min) {
				(Some(deposit), _, Some(min)) if deposit.requestId.id == min => {
					if let Some(elem) = deposits_it.next() {
						result.push(L1UpdateRequest::Deposit(elem.clone()));
					}
				},
				(_, Some(cancel), Some(min)) if cancel.requestId.id == min => {
					if let Some(elem) = cancel_it.next() {
						result.push(L1UpdateRequest::CancelResolution(elem.clone()));
					}
				},
				_ => break,
			}
		}
		result
	}
}

impl TryFrom<eth_abi::Deposit> for Deposit {
	type Error = String;

	fn try_from(deposit: eth_abi::Deposit) -> Result<Self, Self::Error> {
		let requestId = deposit.requestId.try_into()?;
		let depositRecipient = deposit
			.depositRecipient
			.try_into()
			.map_err(|e| format!("Error converting requestId: {}", e))?;
		let tokenAddress = deposit
			.tokenAddress
			.try_into()
			.map_err(|e| format!("Error converting tokenAddress: {}", e))?;

		Ok(Self {
			requestId,
			depositRecipient,
			tokenAddress,
			amount: from_eth_u256(deposit.amount),
			timeStamp: from_eth_u256(deposit.timeStamp),
			ferryTip: from_eth_u256(deposit.ferryTip),
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

		Ok(Self {
			chain: update.chain.try_into()?,
			pendingDeposits: pending_deposits
				.map_err(|e| format!("Error converting pendingDeposits: {}", e))?,
			pendingCancelResolutions: pending_cancel_resultions
				.map_err(|e| format!("Error converting pendingCancelResolutions: {}", e))?,
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
