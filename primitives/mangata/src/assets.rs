use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use crate::Balance;

/// A type describing our custom additional metadata stored in the orml-asset-registry.
#[derive(
	Clone,
	Copy,
	Default,
	PartialOrd,
	Ord,
	PartialEq,
	Eq,
	Debug,
	Encode,
	Decode,
	TypeInfo,
	MaxEncodedLen,
)]
pub struct CustomMetadata {
	/// XCM-related metadata.
	pub xcm: XcmMetadata,
}

#[derive(
	Clone,
	Copy,
	Default,
	PartialOrd,
	Ord,
	PartialEq,
	Eq,
	Debug,
	Encode,
	Decode,
	TypeInfo,
	MaxEncodedLen,
)]
pub struct XcmMetadata {
	/// defines amount equivalent to 1 Unit of native asset,
	/// used to compute the fee charged for every second that an XCM message takes to execute.
	pub proportional_amount_in_native_asset: Balance,
}
