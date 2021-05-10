use sp_runtime::traits::Block as BlockT;
use sp_runtime::AccountId32;
use codec::{Decode, Encode};

/// Information about extrinsic fetched from runtime API
#[derive(Encode, Decode, PartialEq)]
pub struct ExtrinsicInfo{
    /// extrinsic signer
    pub who: AccountId32,
    /// nonce
    pub nonce: u32
}

sp_api::decl_runtime_apis! {
	/// The `TaggedTransactionQueue` api trait for interfering with the transaction queue.
	pub trait ExtrinsicInfoRuntimeApi {
		/// Validate the transaction.
		// #[changed_in(2)]
		// fn get_info(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity;

		/// Validate the transaction.
		///
		/// This method is invoked by the transaction pool to learn details about given transaction.
		/// The implementation should make sure to verify the correctness of the transaction
		/// against current state.
		/// Note that this call may be performed by the pool multiple times and transactions
		/// might be verified in any possible order.
		fn get_info(
			tx: <Block as BlockT>::Extrinsic,
		) -> Option<ExtrinsicInfo>;
	}
}
