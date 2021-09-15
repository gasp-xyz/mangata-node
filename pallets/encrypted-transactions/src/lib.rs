// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

// TODO
// Add dependencies
// Update Chain spec
// Add pallet to runtime
// Add KEy type to runtime

use sp_application_crypto::RuntimeAppPublic;
use codec::{Encode, Decode};
use sp_core::offchain::OpaqueNetworkState;
use sp_std::prelude::*;
use sp_std::convert::TryInto;
use pallet_session::historical::IdentificationTuple;
use sp_runtime::{
	offchain::storage::StorageValueRef,
	RuntimeDebug,
	traits::{Convert, Member, Saturating, AtLeast32BitUnsigned}, Perbill,
	transaction_validity::{
		TransactionValidity, ValidTransaction, InvalidTransaction, TransactionSource,
		TransactionPriority,
	},
};
use sp_staking::{
	SessionIndex,
	offence::{ReportOffence, Offence, Kind},
};
use frame_support::{
	decl_module, decl_event, decl_storage, Parameter, debug, decl_error,
	traits::Get,
	weights::Weight,
};
use frame_system::ensure_none;
use frame_system::offchain::{
	SendTransactionTypes,
	SubmitTransaction,
};

pub mod ecdsa {
	mod app_ecdsa {
		use sp_application_crypto::{app_crypto, KeyTypeId(b"xxtx"), ecdsa};
		app_crypto!(ecdsa, IM_ONLINE);
	}

	sp_application_crypto::with_pair! {
		/// An i'm online keypair using sr25519 as its crypto.
		pub type AuthorityPair = app_ecdsa::Pair;
	}

	/// An i'm online signature using sr25519 as its crypto.
	pub type AuthoritySignature = app_ecdsa::Signature;

	/// An i'm online identifier using sr25519 as its crypto.
	pub type AuthorityId = app_ecdsa::Public;
}

pub trait Trait {
	/// The identifier type for an authority.
	type AuthorityId: Member + Parameter + RuntimeAppPublic + Default + Ord;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Trait> as EncryptedTransactions {

		/// The current set of keys that may issue a heartbeat.
		Keys get(fn keys): Vec<(T::AccountId, T::AuthorityId)>;

	}
	add_extra_genesis {
		config(keys): Vec<(T::AccountId, T::AuthorityId)>;
		build(|config| Module::<T>::initialize_keys(&config.keys))
	}
}

impl<T: Trait> Module<T> {
    fn initialize_keys(keys: &[(T::AccountId, T::AuthorityId)]) {
		if !keys.is_empty() {
			assert!(Keys::<T>::get().is_empty(), "Keys are already initialized!");
			Keys::<T>::put(keys);
		}
	}
}

impl<T: Trait> sp_runtime::BoundToRuntimeAppPublic for Module<T> {
	type Public = T::AuthorityId;
}

impl<T: Trait> pallet_session::OneSessionHandler<T::AccountId> for Module<T> {
	type Key = T::AuthorityId;

	fn on_genesis_session<'a, I: 'a>(validators: I)
		where I: Iterator<Item=(&'a T::AccountId, T::AuthorityId)>
	{
		let keys = validators.collect::<Vec<_>>();
		Self::initialize_keys(&keys);
	}

	fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
		where I: Iterator<Item=(&'a T::AccountId, T::AuthorityId)>
	{
		// Remember who the authorities are for the new session.
		Keys::<T>::put(validators.collect::<Vec<_>>());
	}

	fn on_before_session_ending() {
        //ignore
	}

	fn on_disabled(_i: usize) {
		// ignore
	}
}