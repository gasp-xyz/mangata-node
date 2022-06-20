#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;

#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum ActivateKind {
		FreeBalance,
		StakedUnactivatedLiquidty,
		UnspentReserves,
	}

#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum BondKind {
		FreeBalance,
		ActivatedUnstakedLiquidty,
		UnspentReserves,
	}