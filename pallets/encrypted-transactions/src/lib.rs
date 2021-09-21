// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use sp_application_crypto::RuntimeAppPublic;
use codec::{Encode, Decode};
use sp_core::offchain::OpaqueNetworkState;
use sp_std::prelude::*;
use sp_std::convert::TryInto;
use sp_std::collections::btree_map::BTreeMap;
use pallet_session::historical::IdentificationTuple;
use sp_runtime::{
	offchain::storage::StorageValueRef,
	RuntimeDebug,
	traits::{Convert, Member, Saturating, AtLeast32BitUnsigned, Zero}, Perbill,
	transaction_validity::{
		TransactionValidity, ValidTransaction, InvalidTransaction, TransactionSource,
		TransactionPriority,
	},
    KeyTypeId,
};
use sp_staking::{
	SessionIndex,
	offence::{ReportOffence, Offence, Kind},
};
use frame_support::{
	storage::generator::StorageMap,
	decl_module, decl_event, decl_storage, Parameter, debug, decl_error,
	traits::{Get, WithdrawReasons, ExistenceRequirement, UnfilteredDispatchable},
	weights::{Weight, Pays, GetDispatchInfo},
	dispatch::{DispatchError, DispatchResult},
};
use frame_system::ensure_none;
use frame_system::offchain::{
	SendTransactionTypes,
	SubmitTransaction,
};
use sp_core::storage::ChildInfo;
use frame_support::storage::child;
use mangata_primitives::{Balance, TokenId};
// use sp_runtime::{DispatchResult};
use frame_support::traits::OnUnbalanced;
use orml_tokens::{MultiTokenCurrency, MultiTokenNegativeImbalance};
use frame_system::{ensure_signed, ensure_root, RawOrigin};
use sp_runtime::traits::Hash;

pub const xxtx_key_type_id: KeyTypeId = KeyTypeId(*b"xxtx");

pub mod ecdsa {
	mod app_ecdsa {
		use sp_application_crypto::{app_crypto, ecdsa};
		app_crypto!(ecdsa, super::super::xxtx_key_type_id);
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

pub trait Trait: frame_system::Trait + pallet_session::Trait{
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Tokens: MultiTokenCurrency<Self::AccountId>;
	/// The identifier type for an authority.
	type AuthorityId: Member + Parameter + RuntimeAppPublic + Default + Ord;
	type Fee: Get<Balance>;
	type Treasury: OnUnbalanced<<Self::Tokens as MultiTokenCurrency<Self::AccountId>>::NegativeImbalance>;
	type Call: Parameter + UnfilteredDispatchable<Origin=Self::Origin> + GetDispatchInfo;
}

decl_storage! {
	trait Store for Module<T: Trait> as EncryptedTransactions {

		/// The current set of keys that may issue a heartbeat.
		KeyMap get(fn keys): BTreeMap<T::AccountId, T::AuthorityId>;

		/// The registry for all the user submitted transactions
		TxnRegistry get(fn txn_registry): map hasher(blake2_128_concat) T::Hash => (Vec<u8>, T::AccountId, T::Index, Weight, T::AccountId, T::AccountId, Option<Vec<u8>>);
		
		/// The queue for the doubly encrypted call indexed by builder
		DoublyEncryptedQueue get(fn doubly_encrypted_queue): map hasher(blake2_128_concat) T::AccountId => Vec<T::Hash>;

		/// The queue for the singly encrypted call indexed by executor
		SinglyEncryptedQueue get(fn singly_encrypted_queue): map hasher(blake2_128_concat) T::AccountId => Vec<T::Hash>;

		/// All transactions indexed by session index and user with txn identifier and fee charged (derived from call weight)
		TxnRecord get(fn txn_record): double_map hasher(blake2_128_concat) T::Index,  hasher(blake2_128_concat) T:: AccountId => BTreeMap<T::Hash, (Balance, bool)>;

		/// Executed transactions indexed by session index and user with txn identifier
		ExecutedTxnRecord get(fn execd_txn_record): double_map hasher(blake2_128_concat) T::Index,  hasher(blake2_128_concat) T:: AccountId => Vec<T::Hash>;
	}
	add_extra_genesis {
		config(keys): Vec<(T::AccountId, T::AuthorityId)>;
		build(|config| Module::<T>::initialize_keys(&config.keys.iter().map(|(x,y)| {(x.clone(), y.clone())}).collect::<BTreeMap<_,_>>()))
	}
}

impl<T: Trait> Module<T> {
    fn initialize_keys(keys: &BTreeMap<T::AccountId, T::AuthorityId>) {
		if !keys.is_empty() {
			assert!(KeyMap::<T>::get().is_empty(), "Keys are already initialized!");
			KeyMap::<T>::put(keys);
		}
	}

}

decl_error! {
    /// Errors
    pub enum Error for Module<T: Trait> {
        IncorrectCallWeight,
		NoMarkedRefund,
		CallDeserilizationFailed
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
		Index = <T as frame_system::Trait>::Index,
    {
    	/// A called was called. \[result\]
		Called(DispatchResult),
		DoublyEncryptedTxnSubmitted(AccountId, Index),
		
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// TOOD
		// Remember Author Index

		// with weight!!!!!!!!!!!!

		// Actual call weight should match. Call should be executed as "user"

		// Refund user

		fn deposit_event() = default;

		#[weight = 10_000]
		fn submit_doubly_encrypted_transaction(origin, doubly_encrypted_call: Vec<u8>, nonce: T::Index, weight: Weight, builder: T::AccountId, executor: T::AccountId) -> DispatchResult{
			let user = ensure_signed(origin)?;

			let fee_charged = T::Fee::get();

			T::Tokens::ensure_can_withdraw(0u8.into(), &user, fee_charged.into(), WithdrawReasons::all(), Default::default())?;
			let negative_imbalance = T::Tokens::withdraw(0u8.into(), &user, fee_charged.into(), WithdrawReasons::all(), ExistenceRequirement::AllowDeath)?;
			T::Treasury::on_unbalanced(negative_imbalance);

			let mut identifier_vec: Vec<u8> = Vec::<u8>::new();
			identifier_vec.extend_from_slice(&doubly_encrypted_call[..]);
			identifier_vec.extend_from_slice(&Encode::encode(&user)[..]);
			identifier_vec.extend_from_slice(&Encode::encode(&nonce)[..]);

			let identifier: T::Hash = T::Hashing::hash(&identifier_vec[..]);

			TxnRegistry::<T>::insert(identifier, (doubly_encrypted_call, &user, nonce, weight, &builder, executor, None::<Vec<u8>>));
			DoublyEncryptedQueue::<T>::mutate(&builder, |vec_hash| {vec_hash.push(identifier)});
			TxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &user, |tree_record| tree_record.insert(identifier, (fee_charged, false)));
			Self::deposit_event(RawEvent::DoublyEncryptedTxnSubmitted(user, nonce));
			Ok(())
		}

		#[weight = 10_000]
		fn submit_singly_encrypted_transaction(origin, identifier: T::Hash, singly_encrypted_call: Vec<u8>) -> DispatchResult{
			ensure_none(origin)?;
			let (doubly_encrypted_call, user, nonce, weight, builder, executor, _) = TxnRegistry::<T>::get(identifier);
			DoublyEncryptedQueue::<T>::mutate(&builder, |vec_hash| {vec_hash.retain(|x| *x!=identifier)});
			SinglyEncryptedQueue::<T>::mutate(&executor, |vec_hash| {vec_hash.push(identifier)});
			TxnRegistry::<T>::insert(identifier, (doubly_encrypted_call, user, nonce, weight, &builder, &executor, Some(singly_encrypted_call)));
			Ok(())
		}

		#[weight = 10_000]
		fn submit_decrypted_transaction(origin, identifier: T::Hash, decrypted_call: Vec<u8>, weight: Weight) -> DispatchResult{
			ensure_none(origin)?;
			let (doubly_encrypted_call, user, nonce, weight, builder, executor, singly_encrypted_call) = TxnRegistry::<T>::get(identifier);
			SinglyEncryptedQueue::<T>::mutate(builder, |vec_hash| {vec_hash.retain(|x| *x!=identifier)});
			ExecutedTxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &user, |vec_hash| {vec_hash.push(identifier)});

			let calls: Vec<Box<<T as Trait>::Call>> = Decode::decode(&mut &decrypted_call[..]).map_err(|_| DispatchError::from(Error::<T>::CallDeserilizationFailed))?;

			Module::<T>::execute_calls(RawOrigin::Root.into(), calls, user, nonce, weight)?;

			Ok(())
		}

		#[weight = (*weight, Pays::No)]
		fn execute_calls(origin, calls: Vec<Box<<T as Trait>::Call>>, user_account: T::AccountId, nonce: T::Index, weight: Weight) -> DispatchResult{

			ensure_root(origin)?;

			let calls_weight: u128 = u128::zero();
			for call in calls.iter() {
				calls_weight.saturating_add(call.get_dispatch_info().weight.into());
			}
			if calls_weight > weight.into(){
				return Err(DispatchError::from(Error::<T>::IncorrectCallWeight));
			}
	
			for call in calls {
				let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Signed(user_account.clone()).into());
				Self::deposit_event(RawEvent::Called(res.map(|_| ()).map_err(|e| e.error)));
			}
	
			Ok(())
	
		}

		#[weight = 10_000]
		fn refund_user(origin, identifier: T::Hash) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let current_session_index = <pallet_session::Module<T>>::current_index();
			let previous_session_index: T::Index = current_session_index.checked_sub(1u8.into()).ok_or_else(|| DispatchError::from(Error::<T>::NoMarkedRefund))?.into();

			if ExecutedTxnRecord::<T>::get(previous_session_index, &user).contains(&identifier){
				return Err(DispatchError::from(Error::<T>::NoMarkedRefund));
			}
			else {
				let (fee_charged, already_refunded) = TxnRecord::<T>::get(previous_session_index, &user).get(&identifier).ok_or_else(|| DispatchError::from(Error::<T>::NoMarkedRefund))?.clone();

				if already_refunded{
					return Err(DispatchError::from(Error::<T>::NoMarkedRefund));
				}
				// TODO
				// Refund fee
				TxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &user, |tree_record| tree_record.insert(identifier, (fee_charged, true)));
			}

			Ok(())

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
		let keys = validators.map(|(x,y)| {(x.clone(),y)}).collect::<BTreeMap<_,_>>();
		Self::initialize_keys(&keys);
	}

	fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
		where I: Iterator<Item=(&'a T::AccountId, T::AuthorityId)>
	{
		// Remember who the authorities are for the new session.
		KeyMap::<T>::put(validators.collect::<BTreeMap<_,_>>());
	}

	fn on_before_session_ending() {

		KeyMap::<T>::kill();

		child::kill_storage(&ChildInfo::new_default_from_vec(TxnRegistry::<T>::prefix_hash()));
		child::kill_storage(&ChildInfo::new_default_from_vec(DoublyEncryptedQueue::<T>::prefix_hash()));
		child::kill_storage(&ChildInfo::new_default_from_vec(SinglyEncryptedQueue::<T>::prefix_hash()));

		let session_index = <pallet_session::Module<T>>::current_index();

		if let Some(previous_session_index) = session_index.checked_sub(1u8.into()){
			TxnRecord::<T>::remove_prefix(T::Index::from(previous_session_index));
			ExecutedTxnRecord::<T>::remove_prefix(T::Index::from(previous_session_index));
		}
	}

	fn on_disabled(_i: usize) {
		// ignore
	}
}