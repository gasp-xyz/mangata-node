#![cfg_attr(not(feature = "std"), no_std)]
use super::*;
use frame_support::{
	storage::{migration::move_storage_from_pallet, storage_prefix, unhashed::clear_prefix},
	traits::OnRuntimeUpgrade,
};

pub struct XykRefactorMigration;
impl OnRuntimeUpgrade for XykRefactorMigration {
	fn on_runtime_upgrade() -> Weight {
		log::info!(
			target: "proof_of_stake",
			"on_runtime_upgrade: Attempted to apply xyk refactor migration"
		);

		move_storage_from_pallet(b"RewardsInfo", b"Xyk", b"ProofOfStake");
		move_storage_from_pallet(b"LiquidityMiningActivePoolV2", b"Xyk", b"ProofOfStake");

		let storage_items = vec![
			"LiquidityMiningUser",
			"LiquidityMiningPool",
			"LiquidityMiningUserToBeClaimed",
			"LiquidityMiningActiveUser",
			"LiquidityMiningActivePool",
			"LiquidityMiningUserClaimed",
		];

		for storage_item in storage_items {
			let r = clear_prefix(&storage_prefix(b"Xyk", storage_item.as_bytes()), None, None);

			log::info!(
				target: "proof_of_stake",
				"{:?} clear_prefix result: {:?}, {:?}",
				storage_item.as_bytes(), r.maybe_cursor, r.loops
			);
		}

		<Runtime as frame_system::Config>::BlockWeights::get().max_block
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::info!(
			target: "proof_of_stake",
			"pre_upgrade check: proof_of_stake"
		);

		assert!(
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"Xyk", b"RewardsInfo").to_vec(),
				storage_prefix(b"Xyk", b"RewardsInfo").to_vec(),
				|_| Ok(()),
			)
			.count() >= 1
		);

		assert!(
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"Xyk", b"LiquidityMiningActivePoolV2").to_vec(),
				storage_prefix(b"Xyk", b"LiquidityMiningActivePoolV2").to_vec(),
				|_| Ok(()),
			)
			.count() >= 1
		);

		assert!(
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"ProofOfStake", b"RewardsInfo").to_vec(),
				storage_prefix(b"ProofOfStake", b"RewardsInfo").to_vec(),
				|_| Ok(()),
			)
			.count() == 0
		);

		assert!(
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				|_| Ok(()),
			)
			.count() == 0
		);

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
		log::info!(
			target: "proof_of_stake",
			"post_upgrade check: proof_of_stake"
		);

		assert!(
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"Xyk", b"RewardsInfo").to_vec(),
				storage_prefix(b"Xyk", b"RewardsInfo").to_vec(),
				|_| Ok(()),
			)
			.count() == 0
		);

		assert!(
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"Xyk", b"LiquidityMiningActivePoolV2").to_vec(),
				storage_prefix(b"Xyk", b"LiquidityMiningActivePoolV2").to_vec(),
				|_| Ok(()),
			)
			.count() == 0
		);

		assert!(
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"ProofOfStake", b"RewardsInfo").to_vec(),
				storage_prefix(b"ProofOfStake", b"RewardsInfo").to_vec(),
				|_| Ok(()),
			)
			.count() >= 1
		);

		assert!(
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				|_| Ok(()),
			)
			.count() >= 1
		);

		let storage_items = vec![
			"LiquidityMiningUser",
			"LiquidityMiningPool",
			"LiquidityMiningUserToBeClaimed",
			"LiquidityMiningActiveUser",
			"LiquidityMiningActivePool",
			"LiquidityMiningUserClaimed",
		];

		for storage_item in storage_items {
			assert!(
				frame_support::storage::KeyPrefixIterator::new(
					storage_prefix(b"Xyk", storage_item.as_bytes()).to_vec(),
					storage_prefix(b"Xyk", storage_item.as_bytes()).to_vec(),
					|_| Ok(()),
				)
				.count() == 0
			);
		}

		Ok(())
	}
}
