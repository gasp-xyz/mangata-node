use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions needed for pallet_stable_pools.
pub trait WeightInfo {
	fn create_pool() -> Weight;
	fn mint_liquidity() -> Weight;
	fn burn_liquidity() -> Weight;
	fn multiswap_asset(x: u32) -> Weight;
}

impl WeightInfo for () {
	fn create_pool() -> Weight {
		Weight::from_parts(0, 0)
	}

	fn mint_liquidity() -> Weight {
		Weight::from_parts(0, 0)
	}
	fn burn_liquidity() -> Weight {
		Weight::from_parts(0, 0)
	}
	fn multiswap_asset(x: u32) -> Weight {
		Weight::from_parts(0, 0)
	}
}
