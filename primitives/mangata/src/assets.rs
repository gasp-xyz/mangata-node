use codec::{Decode, Encode};
use scale_info::TypeInfo;

/// Custom metadata for asset-registry
#[derive(TypeInfo, Encode, Decode, Clone, Eq, PartialEq, Debug)]
pub struct EmptyCustomMetadata;
