//! # XYK pallet

#![cfg_attr(test, feature(repr128))]
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::convert::TryInto;

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::OnUnbalanced,
	PalletId,
};
use mangata_types::{Balance, TokenId};

use frame_support::{
	dispatch::GetDispatchInfo,
	pallet_prelude::*,
	traits::{
		tokens::currency::MultiTokenCurrency, ExistenceRequirement, UnfilteredDispatchable,
		WithdrawReasons, FindAuthor
	},
};
use frame_system::{pallet_prelude::*, RawOrigin};
use scale_info::TypeInfo;

use sp_core::crypto::Pair as PairT;
use sp_runtime::{traits::{Hash, AccountIdConversion, OpaqueKeys}, KeyTypeId, RuntimeAppPublic};
use sp_std::{boxed::Box, collections::btree_map::BTreeMap, vec::Vec, collections::vec_deque::VecDeque};
use schnorrkel::vrf::{VRFOutput, VRFProof, VRF_PROOF_LENGTH};
use schnorrkel::keys::{PUBLIC_KEY_LENGTH};
use sp_keystore::vrf::VRFSignature;


#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const XXTX_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"xxtx");

#[derive(Encode, Eq, Clone, Copy, Decode, TypeInfo, Debug, PartialEq)]
pub enum Encryption{
	None,
	Single,
	Double,
}

#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct VRFSignatureWrapper{
	proof: [u8; VRF_PROOF_LENGTH],
	output: [u8; PUBLIC_KEY_LENGTH],
}

impl From<VRFSignature> for VRFSignatureWrapper {
    fn from(sig: VRFSignature) -> Self {
        Self {
			proof: sig.proof.to_bytes(),
			output: sig.output.to_bytes(),
		}
    }
}

impl Into<VRFSignature> for VRFSignatureWrapper {
    fn into(self) -> VRFSignature {
        VRFSignature {
			// VVRFSignatureWrapper can only be created from VRFSignature instance
			// so its safe to unwrap here
			proof: VRFProof::from_bytes(&self.proof).unwrap(),
			output: VRFOutput::from_bytes(&self.output).unwrap(),
		}
    }
}

pub mod ecdsa {
	pub mod app_ecdsa {
		use sp_application_crypto::{app_crypto, ed25519};
		use sp_std::convert::TryFrom;
		app_crypto!(ed25519, crate::XXTX_KEY_TYPE_ID);
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

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

const PALLET_ID: PalletId = PalletId(*b"encry_tx");

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct TxnRegistryDetails<AccountId: Parameter, Hash: Parameter> {
	pub id: Hash,
	pub data: Vec<u8>,
	pub encryption: Encryption,
	pub user: AccountId,
	pub weight: Weight,
	pub builder: AccountId,
	pub executor: AccountId,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_aura::Config + pallet_session::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Tokens: MultiTokenCurrency<Self::AccountId>;
		type Call: Parameter
			+ UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ GetDispatchInfo;
		type DoublyEncryptedCallMaxLength: Get<u32>;
		#[pallet::constant]
		type NativeCurrencyId: Get<TokenId>;
	}

	#[pallet::error]
	/// Errors
	pub enum Error<T> {
		IncorrectCallWeight,
		NoMarkedRefund,
		CallDeserilizationFailed,
		DoublyEncryptedCallMaxLengthExceeded,
		NotEnoughtBalance,
		WrongAccount,
		ProofError,
		TxnDoesNotExistsInRegistry,
		UnexpectedError,
		WrongIdentifier,
		Unauthorized,
		EmptyQueue,
		UnknownBlockAuthor,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//TODO add trading events
		Called(DispatchResult, T::AccountId, T::Hash),

		/// Calls were executed
		CallsExecuted(T::AccountId, T::Hash),

		/// A user has submitted a doubly encrypted transaction.
		DoublyEncryptedTxnSubmitted(T::AccountId, T::Hash),

		/// A collator has submitted a singly encrypted transaction.
		SinglyEncryptedTxnSubmitted(T::AccountId, T::Hash),

		/// A collator has submitted a decrypted transaction.
		DecryptedTransactionSubmitted(T::AccountId, T::Hash),

		/// User refunded
		UserRefunded(T::Index, T::AccountId, T::Index, T::Hash, Balance),
	}

	#[pallet::storage]
	#[pallet::getter(fn keys)]
	pub type KeyMap<T: Config> =
		StorageValue<_, BTreeMap<T::AccountId, T::AuthorityId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn doubly_encrypted_queue)]
	pub type EnqueuedTxs<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, VecDeque<TxnRegistryDetails<T::AccountId, T::Hash>>, ValueQuery>;

	#[pallet::storage]
	pub type UniqueId<T: Config> = StorageValue<_, u128, ValueQuery>;

	// stores new data
	#[pallet::storage]
	pub type ProcessedTx<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	// stores proof for new data
	#[pallet::storage]
	pub type Proof<T: Config> = StorageValue<_,  <sp_core::sr25519::Pair as PairT>::Signature, OptionQuery>;
	
	// stores proof for new data
	#[pallet::storage]
	pub type BlockAuthor<T: Config> = StorageValue<_,  <T as pallet_aura::Config>::AuthorityId, OptionQuery>;

	#[pallet::storage]
	pub type Authorities<T:Config> = StorageValue<_,
	BoundedVec<T::AuthorityId, <T as pallet_aura::Config>::MaxAuthorities>,
	ValueQuery
	>; 

	// #[pallet::storage]
	// #[pallet::getter(fn txn_record)]
	// pub type TxnRecord<T: Config> = StorageDoubleMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	T::Index,
	// 	Blake2_128Concat,
	// 	T::AccountId,
	// 	BTreeMap<T::Hash, (T::Index, Balance, bool)>,
	// 	ValueQuery,
	// >;

	#[pallet::storage]
	#[pallet::getter(fn execd_txn_record)]
	pub type ExecutedTxnRecord<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::Index,
		Blake2_128Concat,
		T::AccountId,
		Vec<T::Hash>,
		ValueQuery,
	>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// dummy `on_initialize` to return the weight used in `on_finalize`.
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight
		{

			let digest = frame_system::Pallet::<T>::digest();
			let author = pallet_aura::Pallet::<T>::find_author(
				digest.logs().iter().filter_map(|d| d.as_pre_runtime()),
			).expect("Could not find AuRa author index!");
			let authorities = pallet_aura::Pallet::<T>::authorities();
			let authority_id = authorities.get(author as usize).expect("authority with given id exists");
			BlockAuthor::<T>::put(authority_id);
				
			// weight of `on_finalize`
			// T::WeightInfo::on_finalize()
			Default::default()
		}

		/// # <weight>
		/// - `O(1)`
		/// - 1 storage deletion (codec `O(1)`).
		/// # </weight>
		fn on_finalize(_n: BlockNumberFor<T>) {
			let data = ProcessedTx::<T>::take();
			let maybe_proof = Proof::<T>::get();

			if !data.is_empty() {
				let proof = maybe_proof.expect("proof is provided when txs are being processed");
				let author = BlockAuthor::<T>::get().expect("block author is known");
				let public_key = author.to_raw_vec();

				let signature = schnorrkel::Signature::from_bytes(proof.as_ref()).unwrap();
				let pub_key = schnorrkel::PublicKey::from_bytes(public_key.as_ref()).unwrap();
				pub_key.verify_simple(b"substrate", data.as_ref(), &signature).expect("proof has been passed");

			}
			

			// assert if block builder did not use any
		}
	}


	// XYK extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn submit_doubly_encrypted_transaction(
			origin: OriginFor<T>,
			doubly_encrypted_call: Vec<u8>,
			fee: Balance,
			weight: Weight,
			builder: T::AccountId,
			executor: T::AccountId,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			ensure!(
				doubly_encrypted_call.len() <= T::DoublyEncryptedCallMaxLength::get() as usize,
				Error::<T>::DoublyEncryptedCallMaxLengthExceeded
			);

			T::Tokens::transfer(Self::native_token_id().into(), &user, &Self::account_id(), fee.into(), ExistenceRequirement::KeepAlive)
				.map_err(|_| Error::<T>::NotEnoughtBalance)?;

			let cnt = UniqueId::<T>::mutate(|id| {
				let prev = *id;
				*id+=1;
				prev		
			});

			let identifier: T::Hash = Self::calculate_unique_id(&user, cnt, &doubly_encrypted_call);

			let txn_registry_details = TxnRegistryDetails {
				id: identifier.clone(),
				data: doubly_encrypted_call,
				encryption: Encryption::Double,
				user: user.clone(),
				weight,
				builder: builder.clone(),
				executor,
			};

			EnqueuedTxs::<T>::mutate(&builder, |queue| queue.push_back(txn_registry_details));
			// TxnRecord::<T>::mutate(
			// 	T::Index::from(<pallet_session::Pallet<T>>::current_index()),
			// 	&user,
			// 	|tree_record| tree_record.insert(identifier, (nonce, fee_charged, false)),
			// );
			Self::deposit_event(Event::DoublyEncryptedTxnSubmitted(user, identifier));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn dummy_tx(
			origin: OriginFor<T>,
			_data: Vec<u8>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn submit_singly_encrypted_transaction(
			origin: OriginFor<T>,
			identifier: T::Hash,
			singly_encrypted_call: Vec<u8>,
		) -> DispatchResult {
			let builder = ensure_signed(origin)?;

			//asset builder is actually block builder
			// ensure!(details.id == identifier, Error::<T>::Unauthorized);

			let mut txs = EnqueuedTxs::<T>::get(&builder);
			let mut details = txs.pop_front().ok_or(Error::<T>::EmptyQueue)?;

			ensure!(details.id == identifier, Error::<T>::WrongIdentifier);
			ensure!(details.encryption == Encryption::Double, Error::<T>::WrongAccount);

			ProcessedTx::<T>::mutate(|data| data.extend_from_slice(&singly_encrypted_call) );

			details.encryption = Encryption::Single;
			details.data = singly_encrypted_call;
			txs.push_back(details);

			Ok(())
		}

		#[pallet::weight(10_000)]
		// //TODO: make use of _weight parameter, collator should precalculate weight of decrypted
		// //transactions
		pub fn submit_decrypted_transaction(
			origin: OriginFor<T>,
			identifier: T::Hash,
			decrypted_call: Vec<u8>,
			_weight: Weight,
		) -> DispatchResult {
			ensure_none(origin)?;

			// let mut txn_registry_details = TxnRegistry::<T>::get(identifier)
			// 	.ok_or_else(|| Error::<T>::TxnDoesNotExistsInRegistry)?;
			// SinglyEncryptedQueue::<T>::mutate(&txn_registry_details.executor, |vec_hash| {
			// 	vec_hash.retain(|x| *x != identifier)
			// });
            //
			// ExecutedTxnRecord::<T>::mutate(
			// 	T::Index::from(<pallet_session::Pallet<T>>::current_index()),
			// 	&txn_registry_details.user,
			// 	|vec_hash| vec_hash.push(identifier),
			// );
            //
			// txn_registry_details.decrypted_call = Some(decrypted_call.clone());
            //
			// TxnRegistry::<T>::insert(identifier, txn_registry_details.clone());
            //
			// Self::deposit_event(Event::DecryptedTransactionSubmitted(
			// 	txn_registry_details.user.clone(),
			// 	identifier,
			// ));
            //
			// let calls: Vec<Box<<T as Config>::Call>> = Decode::decode(&mut &decrypted_call[..])
			// 	.map_err(|_| DispatchError::from(Error::<T>::CallDeserilizationFailed))?;
            //
			// Pallet::<T>::execute_calls(
			// 	RawOrigin::Root.into(),
			// 	calls,
			// 	txn_registry_details.user,
			// 	identifier,
			// 	txn_registry_details.weight,
			// )?;
            //
			Ok(())
		}

		#[pallet::weight(10_000)]
		// #[weight = (weight.saturating_add(10_000), Pays::No)]
		pub fn execute_calls(
			origin: OriginFor<T>,
			calls: Vec<Box<<T as Config>::Call>>,
			user_account: T::AccountId,
			identifier: T::Hash,
			weight: Weight,
		) -> DispatchResult {
			ensure_root(origin)?;

			let mut calls_weight = 0_u128;
			for call in calls.iter() {
				calls_weight =
					calls_weight.saturating_add(call.get_dispatch_info().weight.ref_time() as u128);
			}
			if calls_weight > weight.ref_time() as u128 {
				return Err(DispatchError::from(Error::<T>::IncorrectCallWeight))
			}

			for call in calls {
				let res = call.dispatch_bypass_filter(
					frame_system::RawOrigin::Signed(user_account.clone()).into(),
				);
				Self::deposit_event(Event::Called(
					res.map(|_| ()).map_err(|e| e.error),
					user_account.clone(),
					identifier,
				));
			}

			Self::deposit_event(Event::CallsExecuted(user_account, identifier));

			Ok(())
		}

	// 	#[pallet::weight(10_000)]
	// 	pub fn refund_user(origin: OriginFor<T>, identifier: T::Hash) -> DispatchResult {
	// 		let user = ensure_signed(origin)?;
	// 		let current_session_index = <pallet_session::Pallet<T>>::current_index();
	// 		let previous_session_index: T::Index = current_session_index
	// 			.checked_sub(1u8.into())
	// 			.ok_or_else(|| DispatchError::from(Error::<T>::NoMarkedRefund))?
	// 			.into();
    //
	// 		if ExecutedTxnRecord::<T>::get(previous_session_index, &user).contains(&identifier) {
	// 			return Err(DispatchError::from(Error::<T>::NoMarkedRefund))
	// 		} else {
	// 			let (nonce, fee_charged, already_refunded) =
	// 				TxnRecord::<T>::get(previous_session_index, &user)
	// 					.get(&identifier)
	// 					.ok_or_else(|| DispatchError::from(Error::<T>::NoMarkedRefund))?
	// 					.clone();
    //
	// 			ensure!(!already_refunded, Error::<T>::NoMarkedRefund);
    //
	// 			// TODO
	// 			// Refund fee
	// 			TxnRecord::<T>::mutate(
	// 				T::Index::from(<pallet_session::Pallet<T>>::current_index()),
	// 				&user,
	// 				|tree_record| tree_record.insert(identifier, (nonce, fee_charged, true)),
	// 			);
    //
	// 			Self::deposit_event(Event::UserRefunded(
	// 				previous_session_index,
	// 				user,
	// 				nonce,
	// 				identifier,
	// 				fee_charged,
	// 			));
	// 		}
    //
	// 		Ok(())
	// 	}
	}
}

impl<T: Config> Pallet<T> {

	fn account_id() -> T::AccountId {
		PALLET_ID.into_account_truncating()
	}

	fn native_token_id() -> TokenId {
		<T as Config>::NativeCurrencyId::get()
	}

	fn initialize_keys(keys: &BTreeMap<T::AccountId, T::AuthorityId>) {
		if !keys.is_empty() {
			assert!(KeyMap::<T>::get().is_empty(), "Keys are already initialized!");
			KeyMap::<T>::put(keys);
		}
	}

	fn calculate_unique_id(account: &T::AccountId, cnt: u128, call: &Vec<u8>) -> T::Hash{
			T::Hashing::hash_of(
				&[&call[..],
				&Encode::encode(account)[..],
				&Encode::encode(&cnt)
				]
			)
	}


	fn decrypt_txs(who: &T::AccountId, pub_key: [u8; 32], priv_key: [u8; 64]) -> Result<(), Error<T>>{

		if BlockAuthor::<T>::get().is_none() {
			return Err(Error::<T>::UnknownBlockAuthor);
		}

		if matches!(BlockAuthor::<T>::get(), Some(public) if public.to_raw_vec() != pub_key.to_vec()) {
			return Err(Error::<T>::WrongAccount);
		}

		let pair = schnorrkel::Keypair{
			secret: schnorrkel::SecretKey::from_bytes(&priv_key[..]).expect("that should not fail"),
			public: schnorrkel::PublicKey::from_bytes(&pub_key[..]).expect("that should not fail"),
		};

		let pair = sp_core::sr25519::Pair::from(pair);
		let mut queue = EnqueuedTxs::<T>::get(who);
		let len = queue.len();

		let mut processed_data : Vec<u8> = Default::default();

		while len > 0 {
			let tx = queue.pop_front().ok_or(Error::<T>::EmptyQueue)?;
			if &tx.builder !=  who {
				return Err(Error::<T>::EmptyQueue);
			}
			processed_data.extend_from_slice(&tx.data);
		}

		let signature = pair.sign(&processed_data[..]);
		Proof::<T>::set(Some(signature));

		Ok(())

	}
}
//
// impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
// 	type Public = T::AuthorityId;
// }
//
// impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
// 	type Key = T::AuthorityId;
//
// 	fn on_genesis_session<'a, I: 'a>(validators: I)
// 	where
// 		I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
// 	{
// 		let keys = validators.map(|(x, y)| (x.clone(), y)).collect::<BTreeMap<_, _>>();
// 		Self::initialize_keys(&keys);
// 	}
//
// 	fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
// 	where
// 		I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
// 	{
// 		// Remember who the authorities are for the new session.
// 		KeyMap::<T>::put(validators.collect::<BTreeMap<_, _>>());
// 	}
//
// 	fn on_before_session_ending() {
//
// 		// !!!
// 		// IF THERE ARE ANY TX THEY STILL SHOULD BE CLEANED UP BY THE collators
// 		// !!!
//
// 		// KeyMap::<T>::kill();
// 		// child::kill_storage(
// 		// 	&ChildInfo::new_default_from_vec(TxnRegistry::<T>::prefix_hash()),
// 		// 	None,
// 		// );
// 		// child::kill_storage(
// 		// 	&ChildInfo::new_default_from_vec(DoublyEncryptedQueue::<T>::prefix_hash()),
// 		// 	None,
// 		// );
// 		// child::kill_storage(
// 		// 	&ChildInfo::new_default_from_vec(SinglyEncryptedQueue::<T>::prefix_hash()),
// 		// 	None,
// 		// );
//         //
// 		// let session_index = <pallet_session::Pallet<T>>::current_index();
//         //
// 		// if let Some(previous_session_index) = session_index.checked_sub(1u8.into()) {
// 		// 	TxnRecord::<T>::remove_prefix(T::Index::from(previous_session_index), None);
// 		// 	ExecutedTxnRecord::<T>::remove_prefix(T::Index::from(previous_session_index), None);
// 		// }
// 	}
//
// 	fn on_disabled(_i: u32) {
// 		// ignore
// 	}
// }
