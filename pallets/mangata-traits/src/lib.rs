#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	Parameter,
};
use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member
	},
};
use codec::FullCodec;

pub trait Trait: frame_system::Trait {
    /// TokenId of a token.
    type TokenId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + Default + AtLeast32BitUnsigned + FullCodec;
}