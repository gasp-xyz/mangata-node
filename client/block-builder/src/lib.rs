// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// Copyright (C) 2020 Mangata team
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate block builder
//!
//! This crate provides the [`BlockBuilder`] utility and the corresponding runtime api
//! [`BlockBuilder`](sp_block_builder::BlockBuilder).Error
//!
//! The block builder utility is used in the node as an abstraction over the runtime api to
//! initialize a block, to push extrinsics and to finalize a block.

#![warn(missing_docs)]

use codec::Encode;
use log::info;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::seq::SliceRandom;

use sp_api::{
	ApiErrorFor, ApiExt, ApiRef, Core, ProvideRuntimeApi, StorageChanges, StorageProof,
	TransactionOutcome,
};
use sp_blockchain::{Backend, Error};
use sp_consensus::RecordProof;
use sp_core::{ExecutionContext, H256};
use sp_runtime::{
	generic::BlockId,
	traits::{
		BlakeTwo256, Block as BlockT, DigestFor, Hash, HashFor, Header as HeaderT, NumberFor, One,
	},
};
use std::collections::VecDeque;
use sp_core::crypto::Ss58Codec;

pub use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_runtime::AccountId32;
use std::collections::HashMap;

use sc_client_api::backend;

/// A block that was build by [`BlockBuilder`] plus some additional data.
///
/// This additional data includes the `storage_changes`, these changes can be applied to the
/// backend to get the state of the block. Furthermore an optional `proof` is included which
/// can be used to proof that the build block contains the expected data. The `proof` will
/// only be set when proof recording was activated.
pub struct BuiltBlock<Block: BlockT, StateBackend: backend::StateBackend<HashFor<Block>>> {
	/// The actual block that was build.
	pub block: Block,
	/// The changes that need to be applied to the backend to get the state of the build block.
	pub storage_changes: StorageChanges<StateBackend, Block>,
	/// An optional proof that was recorded while building the block.
	pub proof: Option<StorageProof>,
}

impl<Block: BlockT, StateBackend: backend::StateBackend<HashFor<Block>>>
	BuiltBlock<Block, StateBackend>
{
	/// Convert into the inner values.
	pub fn into_inner(
		self,
	) -> (
		Block,
		StorageChanges<StateBackend, Block>,
		Option<StorageProof>,
	) {
		(self.block, self.storage_changes, self.proof)
	}
}

/// Block builder provider
pub trait BlockBuilderProvider<B, Block, RA>
where
	Block: BlockT,
	B: backend::Backend<Block>,
	Self: Sized,
	RA: ProvideRuntimeApi<Block>,
{
	/// Create a new block, built on top of `parent`.
	///
	/// When proof recording is enabled, all accessed trie nodes are saved.
	/// These recorded trie nodes can be used by a third party to proof the
	/// output of this block builder without having access to the full storage.
	fn new_block_at<R: Into<RecordProof>>(
		&self,
		parent: &BlockId<Block>,
		inherent_digests: DigestFor<Block>,
		record_proof: R,
	) -> sp_blockchain::Result<BlockBuilder<Block, RA, B>>;

	/// Create a new block, built on the head of the chain.
	fn new_block(
		&self,
		inherent_digests: DigestFor<Block>,
	) -> sp_blockchain::Result<BlockBuilder<Block, RA, B>>;
}

/// Extrinsic unique id calculated from its bytes
pub type ExtrinsicId = u64;

/// Utility for building new (valid) blocks from a stream of extrinsics.
pub struct BlockBuilder<'a, Block: BlockT, A: ProvideRuntimeApi<Block>, B> {
	extrinsics: Vec<Block::Extrinsic>,
	api: ApiRef<'a, A::Api>,
	block_id: BlockId<Block>,
	parent_hash: Block::Hash,
	backend: &'a B,
}

impl<'a, Block, A, B> BlockBuilder<'a, Block, A, B>
where
	Block: BlockT,
	A: ProvideRuntimeApi<Block> + 'a,
	A::Api: BlockBuilderApi<Block, Error = Error>
		+ ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>>,
	B: backend::Backend<Block>,
{
	/// Create a new instance of builder based on the given `parent_hash` and `parent_number`.
	///
	/// While proof recording is enabled, all accessed trie nodes are saved.
	/// These recorded trie nodes can be used by a third party to prove the
	/// output of this block builder without having access to the full storage.
	pub fn new(
		api: &'a A,
		parent_hash: Block::Hash,
		parent_number: NumberFor<Block>,
		record_proof: RecordProof,
		inherent_digests: DigestFor<Block>,
		backend: &'a B,
	) -> Result<Self, ApiErrorFor<A, Block>> {
		let header = <<Block as BlockT>::Header as HeaderT>::new(
			parent_number + One::one(),
			Default::default(),
			Default::default(),
			parent_hash,
			inherent_digests,
		);

		let mut api = api.runtime_api();

		if record_proof.yes() {
			api.record_proof();
		}

		let block_id = BlockId::Hash(parent_hash);

		info!("Api call to initialize the block");
		api.initialize_block_with_context(&block_id, ExecutionContext::BlockConstruction, &header)?;

		Ok(Self {
			parent_hash,
			extrinsics: Vec::new(),
			api,
			block_id,
			backend,
		})
	}

	/// Push onto the block's list of extrinsics.
	///
	/// This will ensure the extrinsic can be validly executed (by executing it).
	pub fn push(&mut self, xt: <Block as BlockT>::Extrinsic) -> Result<(), ApiErrorFor<A, Block>> {
		// let block_id = &self.block_id;
		// let extrinsics = &mut self.extrinsics;

		//FIXME add test execution with state rejection
		info!("Pushing transactions without execution");
		self.extrinsics.push(xt);
		Ok(())

		// info!("Going to call api tx execution");
		// self.api.execute_in_transaction(|api| {
		// 	match api.apply_extrinsic_with_context(
		// 		block_id,
		// 		ExecutionContext::BlockConstruction,
		// 		xt.clone(),
		// 	) {
		// 		Ok(Ok(_)) => {
		// 			extrinsics.push(xt);
		// 			TransactionOutcome::Commit(Ok(()))
		// 		}
		// 		Ok(Err(tx_validity)) => {
		// 			TransactionOutcome::Rollback(
		// 				Err(ApplyExtrinsicFailed::Validity(tx_validity).into()),
		// 			)
		// 		},
		// 		Err(e) => TransactionOutcome::Rollback(Err(e)),
		// 	}
		// })
	}

	/// Consume the builder to build a valid `Block` containing all pushed extrinsics.
	///
	/// Returns the build `Block`, the changes to the storage and an optional `StorageProof`
	/// supplied by `self.api`, combined as [`BuiltBlock`].
	/// The storage proof will be `Some(_)` when proof recording was enabled.
	pub fn build(
		mut self,
		context_provider: &dyn Fn(H256) -> Option<(AccountId32, u32)>
	) -> Result<(BuiltBlock<Block, backend::StateBackendFor<B, Block>>, Vec<H256>), ApiErrorFor<A, Block>> {
		let block_id = &self.block_id;

		let extrinsics = self.extrinsics.clone();
		let parent_hash = self.parent_hash;
		let extrinsics_hash = BlakeTwo256::hash(&extrinsics.encode());

		//FIXME
		let consumed_extrinsics: Vec<H256> = match self
			.backend
			.blockchain()
			.body(BlockId::Hash(parent_hash))
			.unwrap()
		{
			Some(previous_block_extrinsics) => {
				if previous_block_extrinsics.is_empty() {
					info!("No extrinsics found for previous block");
					extrinsics.into_iter().for_each(|xt| {
						self.api.execute_in_transaction(|api| {
							match api.apply_extrinsic_with_context(
								block_id,
								ExecutionContext::BlockConstruction,
								xt.clone(),
							) {
								Ok(Ok(_)) => TransactionOutcome::Commit(()),
								Ok(Err(_tx_validity)) => TransactionOutcome::Rollback(()),
								Err(_e) => TransactionOutcome::Rollback(()),
							}
						})
					});
					vec![]
				} else {
					info!("transaction count {}", previous_block_extrinsics.len());
					info!("seed: {:?}", extrinsics_hash.to_fixed_bytes());


                    // group extrinsics by author - extrinsics are already received in order
					let mut grouped_extrinsics: HashMap<Option<AccountId32>, VecDeque<_>> = previous_block_extrinsics
						.into_iter()
						.fold(HashMap::new(), |mut groups, tx| {
							let tx_hash = BlakeTwo256::hash(&tx.encode());
							let who = context_provider(tx_hash).map(|x| Some(x.0)).unwrap_or(None);

							log::debug!(target: "block_shuffler", "who:{:32}  extrinsic:{:?}",who.clone().map(|x| x.to_ss58check()).unwrap_or_else(|| String::from("None")), tx_hash);

							groups.entry(who).or_insert(VecDeque::new()).push_back(tx);
							groups
						});

					let mut rng: StdRng = SeedableRng::from_seed(extrinsics_hash.to_fixed_bytes());

                    // generate exact number of slots for each account
                    // [ Alice, Alice, Alice, ... , Bob, Bob, Bob, ... ]
                    let mut slots: Vec<_> = grouped_extrinsics
                        .iter()
                        .map(|(who, txs)| vec![who;txs.len()])
                        .flatten()
                        .map(|elem| elem.to_owned())
                        .collect();

                    // shuffle slots
					slots.shuffle(&mut rng);

                    // fill slots using extrinsics in order
                    // [ Alice, Bob, ... , Alice, Bob ]
                    //              ↓↓↓
                    // [ AliceExtrinsic1, BobExtrinsic1, ... , AliceExtrinsicN, BobExtrinsicN ]
                    let shuffled_extrinsics: Vec<_> = slots
                        .into_iter()
                        .map(|who| grouped_extrinsics.get_mut(&who).unwrap().pop_front().unwrap())
                        .collect();


                    for xt in shuffled_extrinsics.iter() {
                        log::debug!(target: "block_shuffler", "executing extrinsic :{:?}", BlakeTwo256::hash(&xt.encode()));
						self.api.execute_in_transaction(|api| {
							match api.apply_extrinsic_with_context(
								block_id,
								ExecutionContext::BlockConstruction,
								xt.clone(),
							) {
								Ok(Ok(_)) => TransactionOutcome::Commit(()),
								Ok(Err(_tx_validity)) => TransactionOutcome::Rollback(()),
								Err(_e) => TransactionOutcome::Rollback(()),
							}
						})
					};

					shuffled_extrinsics.into_iter().map(|tx| BlakeTwo256::hash(&tx.encode())).collect()
				}
			}
			None => {
				info!("No extrinsics found for previous block");
				vec![]
			}
		};

		let header = self
			.api
			.finalize_block_with_context(&self.block_id, ExecutionContext::BlockConstruction)?;

		debug_assert_eq!(
			header.extrinsics_root().clone(),
			HashFor::<Block>::ordered_trie_root(
				self.extrinsics.iter().map(Encode::encode).collect(),
			),
		);

		let proof = self.api.extract_proof();

		let state = self.backend.state_at(self.block_id)?;
		let changes_trie_state = backend::changes_tries_state_at_block(
			&self.block_id,
			self.backend.changes_trie_storage(),
		)?;

		let storage_changes =
			self.api
				.into_storage_changes(&state, changes_trie_state.as_ref(), parent_hash)?;

		Ok(
			(BuiltBlock {
			block: <Block as BlockT>::new(header, self.extrinsics),
			storage_changes,
			proof,
			},
			consumed_extrinsics)
		)
	}

	/// Create the inherents for the block.
	///
	/// Returns the inherents created by the runtime or an error if something failed.
	pub fn create_inherents(
		&mut self,
		inherent_data: sp_inherents::InherentData,
	) -> Result<Vec<Block::Extrinsic>, ApiErrorFor<A, Block>> {
		let block_id = self.block_id;
		self.api.execute_in_transaction(move |api| {
			// `create_inherents` should not change any state, to ensure this we always rollback
			// the transaction.
			TransactionOutcome::Rollback(api.inherent_extrinsics_with_context(
				&block_id,
				ExecutionContext::BlockConstruction,
				inherent_data,
			))
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_blockchain::HeaderBackend;
	use sp_core::Blake2Hasher;
	use sp_state_machine::Backend;
	use substrate_test_runtime_client::{DefaultTestClientBuilderExt, TestClientBuilderExt};

	#[test]
	fn block_building_storage_proof_does_not_include_runtime_by_default() {
		let builder = substrate_test_runtime_client::TestClientBuilder::new();
		let backend = builder.backend();
		let client = builder.build();

		let block = BlockBuilder::new(
			&client,
			client.info().best_hash,
			client.info().best_number,
			RecordProof::Yes,
			Default::default(),
			&*backend,
		)
		.unwrap()
		.build()
		.unwrap();

		let proof = block.proof.expect("Proof is build on request");

		let backend = sp_state_machine::create_proof_check_backend::<Blake2Hasher>(
			block.storage_changes.transaction_storage_root,
			proof,
		)
		.unwrap();

		assert!(backend
			.storage(&sp_core::storage::well_known_keys::CODE)
			.unwrap_err()
			.contains("Database missing expected key"),);
	}
}
