// Copyright (C) 2020 Mangata team

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::storage::child;
use frame_support::traits::OnUnbalanced;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    storage::generator::StorageMap,
    traits::{ExistenceRequirement, Get, UnfilteredDispatchable, WithdrawReasons},
    weights::{GetDispatchInfo, Pays, Weight},
    Parameter,
};
use frame_system::ensure_none;
use frame_system::{ensure_root, ensure_signed, RawOrigin};
use mangata_primitives::Balance;
use orml_tokens::MultiTokenCurrency;
use sp_application_crypto::RuntimeAppPublic;
use sp_core::storage::ChildInfo;
use sp_runtime::traits::Hash;
use sp_runtime::{
    traits::{Member, Zero},
    KeyTypeId, RuntimeDebug,
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;

pub const XXTX_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"xxtx");

pub mod ecdsa {
    mod app_ecdsa {
        use sp_application_crypto::{app_crypto, ecdsa};
        app_crypto!(ecdsa, super::super::XXTX_KEY_TYPE_ID);
    }

    sp_application_crypto::with_pair! {
        /// An xxtx keypair using sr25519 as its crypto.
        pub type AuthorityPair = app_ecdsa::Pair;
    }

    /// An xxtx signature using sr25519 as its crypto.
    pub type AuthoritySignature = app_ecdsa::Signature;

    /// An xxtx identifier using sr25519 as its crypto.
    pub type AuthorityId = app_ecdsa::Public;
}

pub trait Trait: frame_system::Trait + pallet_session::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Tokens: MultiTokenCurrency<Self::AccountId>;
    type AuthorityId: Member + Parameter + RuntimeAppPublic + Default + Ord;
    type Fee: Get<Balance>;
    type Treasury: OnUnbalanced<
        <Self::Tokens as MultiTokenCurrency<Self::AccountId>>::NegativeImbalance,
    >;
    type Call: Parameter
        + UnfilteredDispatchable<Origin = Self::Origin>
        + GetDispatchInfo
        + From<Call<Self>>;
    type DoublyEncryptedCallMaxLength: Get<u32>;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct TxnRegistryDetails<AccountId, Index> {
    pub doubly_encrypted_call: Vec<u8>,
    pub user: AccountId,
    pub nonce: Index,
    pub weight: Weight,
    pub builder: AccountId,
    pub executor: AccountId,
    pub singly_encrypted_call: Option<Vec<u8>>,
    pub decrypted_call: Option<Vec<u8>>,
}

decl_storage! {
    trait Store for Module<T: Trait> as EncryptedTransactions {

        /// The current set of keys that may issue a heartbeat.
        KeyMap get(fn keys): BTreeMap<T::AccountId, T::AuthorityId>;

        /// The registry for all the user submitted transactions
        TxnRegistry get(fn txn_registry): map hasher(blake2_128_concat) T::Hash => Option<TxnRegistryDetails<T::AccountId, T::Index>>;

        /// The queue for the doubly encrypted call indexed by builder
        DoublyEncryptedQueue get(fn doubly_encrypted_queue): map hasher(blake2_128_concat) T::AccountId => Vec<T::Hash>;

        /// The queue for the singly encrypted call indexed by executor
        SinglyEncryptedQueue get(fn singly_encrypted_queue): map hasher(blake2_128_concat) T::AccountId => Vec<T::Hash>;

        /// All transactions indexed by session index and user with txn identifier and fee charged (derived from call weight)
        TxnRecord get(fn txn_record): double_map hasher(blake2_128_concat) T::Index,  hasher(blake2_128_concat) T:: AccountId => BTreeMap<T::Hash, (T::Index, Balance, bool)>;

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
            assert!(
                KeyMap::<T>::get().is_empty(),
                "Keys are already initialized!"
            );
            KeyMap::<T>::put(keys);
        }
    }
}

decl_error! {
    /// Errors
    pub enum Error for Module<T: Trait>
    {
        IncorrectCallWeight,
        NoMarkedRefund,
        CallDeserilizationFailed,
        DoublyEncryptedCallMaxLengthExceeded,
        TxnDoesNotExistsInRegistry,
        UnexpectedError
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Index = <T as frame_system::Trait>::Index,
        Hash = <T as frame_system::Trait>::Hash,
    {
        /// A called was called. \[result\]
        Called(DispatchResult, AccountId, Index, Hash),

        /// Calls were executed
        CallsExecuted(AccountId, Index, Hash),

        /// A user has submitted a doubly encrypted transaction.
        DoublyEncryptedTxnSubmitted(AccountId, Index, Hash),

        /// A collator has submitted a singly encrypted transaction.
        SinglyEncryptedTxnSubmitted(AccountId, Index, Hash),

        /// A collator has submitted a decrypted transaction.
        DecryptedTransactionSubmitted(AccountId, Index, Hash),

        /// User refunded
        UserRefunded(Index, AccountId, Index, Hash, Balance),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 10_000]
        fn submit_doubly_encrypted_transaction(origin, doubly_encrypted_call: Vec<u8>, nonce: T::Index, weight: Weight, builder: T::AccountId, executor: T::AccountId) -> DispatchResult{
            let user = ensure_signed(origin)?;

            ensure!(doubly_encrypted_call.len() <= T::DoublyEncryptedCallMaxLength::get() as usize, Error::<T>::DoublyEncryptedCallMaxLengthExceeded);

            let fee_charged = T::Fee::get();

            T::Tokens::ensure_can_withdraw(0u8.into(), &user, fee_charged.into(), WithdrawReasons::all(), Default::default())?;
            let negative_imbalance = T::Tokens::withdraw(0u8.into(), &user, fee_charged.into(), WithdrawReasons::all(), ExistenceRequirement::AllowDeath)?;
            T::Treasury::on_unbalanced(negative_imbalance);

            let mut identifier_vec: Vec<u8> = Vec::<u8>::new();
            identifier_vec.extend_from_slice(&doubly_encrypted_call[..]);
            identifier_vec.extend_from_slice(&Encode::encode(&user)[..]);
            identifier_vec.extend_from_slice(&Encode::encode(&nonce)[..]);

            let identifier: T::Hash = T::Hashing::hash(&identifier_vec[..]);

            let txn_registry_details = TxnRegistryDetails{
                doubly_encrypted_call: doubly_encrypted_call,
                user: user.clone(),
                nonce: nonce,
                weight: weight,
                builder: builder.clone(),
                executor: executor,
                singly_encrypted_call: None,
                decrypted_call: None,
            };

            TxnRegistry::<T>::insert(identifier, txn_registry_details);
            DoublyEncryptedQueue::<T>::mutate(&builder, |vec_hash| {vec_hash.push(identifier)});
            TxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &user, |tree_record| tree_record.insert(identifier, (nonce, fee_charged, false)));
            Self::deposit_event(RawEvent::DoublyEncryptedTxnSubmitted(user, nonce, identifier));
            Ok(())
        }

        #[weight = 10_000]
        fn submit_singly_encrypted_transaction(origin, identifier: T::Hash, singly_encrypted_call: Vec<u8>) -> DispatchResult{
            ensure_none(origin)?;
            TxnRegistry::<T>::try_mutate(identifier, |txn_registry_details_option| -> DispatchResult {
                if let Some (ref mut txn_registry_details) = txn_registry_details_option{

                    DoublyEncryptedQueue::<T>::mutate(&txn_registry_details.builder, |vec_hash| {vec_hash.retain(|x| *x!=identifier)});
                    SinglyEncryptedQueue::<T>::mutate(&txn_registry_details.executor, |vec_hash| {vec_hash.push(identifier)});
                    txn_registry_details.singly_encrypted_call = Some(singly_encrypted_call);

                    Self::deposit_event(RawEvent::SinglyEncryptedTxnSubmitted(txn_registry_details.user.clone(), txn_registry_details.nonce, identifier));

                    Ok(())

                }else{

                    Err(DispatchError::from(Error::<T>::TxnDoesNotExistsInRegistry))
                }
            })
        }

        #[weight = 10_000]
        //TODO: make use of _weight parameter, collator should precalculate weight of decrypted
        //transactions
        fn submit_decrypted_transaction(origin, identifier: T::Hash, decrypted_call: Vec<u8>, _weight: Weight) -> DispatchResult{
            ensure_none(origin)?;

            let mut txn_registry_details = TxnRegistry::<T>::get(identifier).ok_or_else(|| Error::<T>::TxnDoesNotExistsInRegistry)?;
            SinglyEncryptedQueue::<T>::mutate(&txn_registry_details.executor, |vec_hash| {vec_hash.retain(|x| *x!=identifier)});
            ExecutedTxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &txn_registry_details.user, |vec_hash| {vec_hash.push(identifier)});

            txn_registry_details.decrypted_call = Some(decrypted_call.clone());

            TxnRegistry::<T>::insert(identifier, &txn_registry_details);

            Self::deposit_event(RawEvent::DecryptedTransactionSubmitted(txn_registry_details.user.clone(), txn_registry_details.nonce, identifier));

            let calls: Vec<Box<<T as Trait>::Call>> = Decode::decode(&mut &decrypted_call[..]).map_err(|_| DispatchError::from(Error::<T>::CallDeserilizationFailed))?;

            Module::<T>::execute_calls(RawOrigin::Root.into(), calls, txn_registry_details.user, identifier, txn_registry_details.nonce, txn_registry_details.weight)?;

            Ok(())
        }

        #[weight = (weight.saturating_add(10_000), Pays::No)]
        fn execute_calls(origin, calls: Vec<Box<<T as Trait>::Call>>, user_account: T::AccountId, identifier: T::Hash, nonce: T::Index, weight: Weight) -> DispatchResult{

            ensure_root(origin)?;

            let mut calls_weight: u128 = u128::zero();
            for call in calls.iter() {
                calls_weight = calls_weight.saturating_add(call.get_dispatch_info().weight.into());
            }
            if calls_weight > weight.into(){
                return Err(DispatchError::from(Error::<T>::IncorrectCallWeight));
            }

            for call in calls {
                let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Signed(user_account.clone()).into());
                Self::deposit_event(RawEvent::Called(res.map(|_| ()).map_err(|e| e.error), user_account.clone(), nonce, identifier));
            }

            Self::deposit_event(RawEvent::CallsExecuted(user_account, nonce, identifier));

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
                let (nonce, fee_charged, already_refunded) = TxnRecord::<T>::get(previous_session_index, &user).get(&identifier).ok_or_else(|| DispatchError::from(Error::<T>::NoMarkedRefund))?.clone();

                ensure!(!already_refunded, Error::<T>::NoMarkedRefund);

                // TODO
                // Refund fee
                TxnRecord::<T>::mutate(T::Index::from(<pallet_session::Module<T>>::current_index()), &user, |tree_record| tree_record.insert(identifier, (nonce, fee_charged, true)));

                Self::deposit_event(RawEvent::UserRefunded(previous_session_index, user, nonce, identifier, fee_charged));
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
    where
        I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
    {
        let keys = validators
            .map(|(x, y)| (x.clone(), y))
            .collect::<BTreeMap<_, _>>();
        Self::initialize_keys(&keys);
    }

    fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
    where
        I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
    {
        // Remember who the authorities are for the new session.
        KeyMap::<T>::put(validators.collect::<BTreeMap<_, _>>());
    }

    fn on_before_session_ending() {
        KeyMap::<T>::kill();

        child::kill_storage(&ChildInfo::new_default_from_vec(
            TxnRegistry::<T>::prefix_hash(),
        ));
        child::kill_storage(&ChildInfo::new_default_from_vec(
            DoublyEncryptedQueue::<T>::prefix_hash(),
        ));
        child::kill_storage(&ChildInfo::new_default_from_vec(
            SinglyEncryptedQueue::<T>::prefix_hash(),
        ));

        let session_index = <pallet_session::Module<T>>::current_index();

        if let Some(previous_session_index) = session_index.checked_sub(1u8.into()) {
            TxnRecord::<T>::remove_prefix(T::Index::from(previous_session_index));
            ExecutedTxnRecord::<T>::remove_prefix(T::Index::from(previous_session_index));
        }
    }

    fn on_disabled(_i: usize) {
        // ignore
    }
}
