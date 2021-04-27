use frame_support::{
	Parameter,
};
use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member
	},
};
use codec::FullCodec;
use sp_std::*;

pub trait MangataPrimitives {
    /// TokenId of a token.
    type TokenId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + Default + AtLeast32BitUnsigned + FullCodec;
}