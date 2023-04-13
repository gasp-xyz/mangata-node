#![cfg_attr(not(feature = "std"), no_std)]
use super::*;
use frame_support::{
	storage::{
		migration::{move_prefix, move_storage_from_pallet},
		storage_prefix, unhashed,
	},
	traits::OnRuntimeUpgrade,
};

pub fn move_storage_from_pallet_with_rename(
	old_storage_name: &[u8],
	new_storage_name: &[u8],
	old_pallet_name: &[u8],
	new_pallet_name: &[u8],
) {
	let new_prefix = storage_prefix(new_pallet_name, new_storage_name);
	let old_prefix = storage_prefix(old_pallet_name, old_storage_name);

	move_prefix(&old_prefix, &new_prefix);

	if let Some(value) = unhashed::get_raw(&old_prefix) {
		unhashed::put_raw(&new_prefix, &value);
		unhashed::kill(&old_prefix);
	}
}

pub struct XykRefactorMigration;
impl OnRuntimeUpgrade for XykRefactorMigration {
	fn on_runtime_upgrade() -> Weight {
		log::info!(
			target: "proof_of_stake",
			"on_runtime_upgrade: Attempted to apply xyk refactor migration"
		);

		move_storage_from_pallet_with_rename(
			b"PromotedPoolRewardsV2",
			b"PromotedPoolRewards",
			b"Issuance",
			b"ProofOfStake",
		);
		move_storage_from_pallet_with_rename(
			b"LiquidityMiningActivePoolV2",
			b"RewardsInfo",
			b"ProofOfStake",
			b"ProofOfStake",
		);

		<Runtime as frame_system::Config>::BlockWeights::get().max_block
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::info!(
			target: "migration",
			"pre_upgrade check: proof_of_stake"
		);

		let pos__liquidity_mining_avitvate_pool_v2_count =
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				|_| Ok(()),
			)
			.count();

		let issuance__promoted_pool_reards_v2 = frame_support::storage::KeyPrefixIterator::new(
			storage_prefix(b"Issuance", b"PromotedPoolRewardsV2").to_vec(),
			storage_prefix(b"Issuance", b"PromotedPoolRewardsV2").to_vec(),
			|_| Ok(()),
		)
		.count();

		log::info!(target: "migration", "PRE ProofOfStake::LiquidityMiningActivePoolV2 count  :{}", pos__liquidity_mining_avitvate_pool_v2_count);
		log::info!(target: "migration", "PRE Issuance::PromotedPoolRewardsV2           count  :{}", issuance__promoted_pool_reards_v2);

		assert!(pos__liquidity_mining_avitvate_pool_v2_count >= 1);
		assert!(issuance__promoted_pool_reards_v2 >= 1);

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
		log::info!(
			target: "proof_of_stake",
			"post_upgrade check: proof_of_stake"
		);

		let pos__liquidity_mining_avitvate_pool_v2_count =
			frame_support::storage::KeyPrefixIterator::new(
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				storage_prefix(b"ProofOfStake", b"LiquidityMiningActivePoolV2").to_vec(),
				|_| Ok(()),
			)
			.count();

		let issuance__promoted_pool_reards_v2 = frame_support::storage::KeyPrefixIterator::new(
			storage_prefix(b"Issuance", b"PromotedPoolRewardsV2").to_vec(),
			storage_prefix(b"Issuance", b"PromotedPoolRewardsV2").to_vec(),
			|_| Ok(()),
		)
		.count();

		let pos__rewards_info_count = frame_support::storage::KeyPrefixIterator::new(
			storage_prefix(b"ProofOfStake", b"RewardsInfo").to_vec(),
			storage_prefix(b"ProofOfStake", b"RewardsInfo").to_vec(),
			|_| Ok(()),
		)
		.count();

		let pos__promoted_pool_rewards_count = frame_support::storage::KeyPrefixIterator::new(
			storage_prefix(b"ProofOfStake", b"PromotedPoolRewards").to_vec(),
			storage_prefix(b"ProofOfStake", b"PromotedPoolRewards").to_vec(),
			|_| Ok(()),
		)
		.count();

		log::info!(target: "migration", "POST ProofOfStake::LiquidityMiningActivePoolV2 count :{}", pos__liquidity_mining_avitvate_pool_v2_count);
		log::info!(target: "migration", "POST Issuance::PromotedPoolRewardsV2 count           :{}", issuance__promoted_pool_reards_v2);
		log::info!(target: "migration", "POST ProofOfStake::RewardsInfo count                 :{}", pos__rewards_info_count);
		log::info!(target: "migration", "POST Issuance::PromotedPoolRewards count             :{}", pos__promoted_pool_rewards_count);

		assert!(pos__liquidity_mining_avitvate_pool_v2_count >= 1);
		assert!(issuance__promoted_pool_reards_v2 >= 1);

		Ok(())
	}
}
