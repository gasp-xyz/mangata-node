#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::weights::DispatchInfo;
use frame_system::Trait;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
};

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct CheckNonce<T: Trait>(#[codec(compact)] pub T::Index);

impl<T: Trait> CheckNonce<T> {
    /// utility constructor. Used only in client/factory code.
    pub fn from(nonce: T::Index) -> Self {
        Self(nonce)
    }
}

impl<T: Trait> sp_std::fmt::Debug for CheckNonce<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "CheckNonce({})", self.0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Trait> SignedExtension for CheckNonce<T>
where
    T::Call: Dispatchable<Info = DispatchInfo>,
{
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "CheckNonce";

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<(), TransactionValidityError> {
        frame_system::CheckNonce::<T>::from(self.0).pre_dispatch(who, call, info, len)
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        frame_system::CheckNonce::<T>::from(self.0).validate(who, call, info, len)
    }
}
