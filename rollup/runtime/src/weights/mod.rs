#![allow(clippy::unnecessary_cast)]

mod block_weights;
mod extrinsic_weights;

pub use block_weights::BlockExecutionWeight as VerBlockExecutionWeight;
pub use extrinsic_weights::ExtrinsicBaseWeight as VerExtrinsicBaseWeight;

pub mod frame_system;
pub mod orml_asset_registry;
pub mod orml_tokens;
pub mod pallet_bootstrap;
pub mod pallet_collective_mangata;
pub mod pallet_crowdloan_rewards;
pub mod pallet_fee_lock;
pub mod pallet_issuance;
pub mod pallet_multipurpose_liquidity;
pub mod pallet_proof_of_stake;
pub mod pallet_session;
pub mod pallet_timestamp;
pub mod pallet_treasury;
pub mod pallet_utility_mangata;
pub mod pallet_vesting_mangata;
pub mod pallet_xyk;
pub mod parachain_staking;

pub use self::{
	frame_system as frame_system_weights, orml_asset_registry as orml_asset_registry_weights,
	orml_tokens as orml_tokens_weights, pallet_bootstrap as pallet_bootstrap_weights,
	pallet_collective_mangata as pallet_collective_mangata_weights,
	pallet_crowdloan_rewards as pallet_crowdloan_rewards_weights,
	pallet_fee_lock as pallet_fee_lock_weights, pallet_issuance as pallet_issuance_weights,
	pallet_multipurpose_liquidity as pallet_multipurpose_liquidity_weights,
	pallet_proof_of_stake as pallet_proof_of_stake_weights,
	pallet_session as pallet_session_weights, pallet_timestamp as pallet_timestamp_weights,
	pallet_treasury as pallet_treasury_weights,
	pallet_utility_mangata as pallet_utility_mangata_weights,
	pallet_vesting_mangata as pallet_vesting_mangata_weights, pallet_xyk as pallet_xyk_weights,
	parachain_staking as parachain_staking_weights,
};
