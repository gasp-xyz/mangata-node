#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
// use mangata_traits::{TxWrapper, EncryptedTX, ExecuteEncryptedExtrinsic};

use codec::Encode;
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use frame_support::{traits::UnfilteredDispatchable, weights::GetDispatchInfo, Parameter};

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
    type Call: Parameter + UnfilteredDispatchable<Origin = Self::Origin> + GetDispatchInfo;
}

decl_storage! {
    trait Store for Module<T: Trait> as XykStorage {
        /// Transactions & TransactionsId serves as FIFO queue for storing encrypted transactions

        /// block_builder, transaction_id => transaction
        Transactions: double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u32 => (Vec<u8>, H256);

        /// stores current index of transaction per block builder
        TransactionsId get(fn txs): map hasher(blake2_128_concat) T::AccountId => u32;

    }
}

decl_event!(
    pub enum Event {
        /// Asset info stored. [assetId, info]
        ExtrinsicDecoded,
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Target application not found.
        AppNotFound,
        DeserializationError,
        /// Updated AppId is the same as current
        DifferentAppIdRequired
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;

        /// thats only for development purposes - should be replaced by RPC call or offline api
        /// that allows for generating encrypted payloads
        #[weight = 10_000]
        pub fn dev_encrypt_tx (
            origin,
            block_builder_id: T::AccountId,
            call: Box<<T as Trait>::Call>,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin.clone())?;
            let bytes = call.encode();

            // should be real proof
            let proof = BlakeTwo256::hash(&bytes[..]);
            Self::submit_encrypted_tx(origin, block_builder_id, bytes, proof)
        }

        /// use for submitting encrypted TX
        #[weight = 10_000]
        pub fn submit_encrypted_tx (
            origin,
            block_builder_id: T::AccountId,
            encrypted_call: Vec<u8>,
            encrypted_call_proof: H256,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            let id: u32 = TransactionsId::<T>::get(block_builder_id.clone());
            Transactions::<T>::insert(block_builder_id.clone(), id, (encrypted_call, encrypted_call_proof));
            TransactionsId::<T>::mutate(block_builder_id, |n| *n += 1);
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {}
