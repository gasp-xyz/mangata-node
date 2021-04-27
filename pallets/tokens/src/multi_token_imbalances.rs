// wrapping these imbalances in a private module is necessary to ensure absolute
// privacy of the inner member.
use crate::{TotalIssuance, Trait};
use frame_support::storage::StorageMap;
use frame_support::traits::{Imbalance, TryDrop};
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{mem, result};
use mangata_primitives::{TokenId, Balance};

/// Opaque, move-only struct with private fields that serves as a token
/// denoting that funds have been created without any equal and opposite
/// accounting.
#[must_use]
pub struct PositiveImbalance(
    TokenId,
	Balance,
);

impl PositiveImbalance {
	/// Create a new positive imbalance from a balance.
	pub fn new(currency_id: TokenId, amount: Balance) -> Self {
		PositiveImbalance(currency_id, amount)
	}

	pub fn zero(currency_id: TokenId) -> Self {
		PositiveImbalance(currency_id, Zero::zero())
	}
}

/// Opaque, move-only struct with private fields that serves as a token
/// denoting that funds have been destroyed without any equal and opposite
/// accounting.
#[must_use]
pub struct NegativeImbalance(
    TokenId,
	Balance,
);

impl NegativeImbalance {
	/// Create a new negative imbalance from a balance.
	pub fn new(currency_id: TokenId, amount: Balance) -> Self {
		NegativeImbalance(currency_id, amount)
	}

	pub fn zero(currency_id: TokenId) -> Self {
		NegativeImbalance(currency_id, Zero::zero())
	}

}

impl TryDrop for PositiveImbalance {
	fn try_drop(self) -> result::Result<(), Self> {
		self.drop_zero()
	}
}

impl Imbalance<Balance> for PositiveImbalance {
	type Opposite = NegativeImbalance;

	fn zero() -> Self {
        unimplemented!("PositiveImbalance::zero is not implemented");
	}

	fn drop_zero(self) -> result::Result<(), Self> {
		if self.1.is_zero() {
			Ok(())
		} else {
			Err(self)
		}
	}
	fn split(self, amount: Balance) -> (Self, Self) {
		let first = self.1.min(amount);
		let second = self.1 - first;
        let currency_id = self.0;

		mem::forget(self);
		(Self::new(currency_id, first), Self::new(currency_id, second))
	}
	fn merge(mut self, other: Self) -> Self {
        assert_eq!(self.0, other.0);
        self.1 = self.1.saturating_add(other.1);
        mem::forget(other);
        self
	}
	fn subsume(&mut self, other: Self) {
        assert_eq!(self.0, other.0);
		self.1 = self.1.saturating_add(other.1);
		mem::forget(other);
	}
	fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
        assert_eq!(self.0, other.0);
        let (a, b) = (self.1, other.1);
        let currency_id = self.0;
        mem::forget((self, other));

        if a >= b {
            Ok(Self::new(currency_id, a - b))
        } else {
            Err(NegativeImbalance::new(currency_id, b - a))
        }
	}
	fn peek(&self) -> Balance {
		self.1
	}
}

impl TryDrop for NegativeImbalance {
	fn try_drop(self) -> result::Result<(), Self> {
		self.drop_zero()
	}
}

impl Imbalance<Balance> for NegativeImbalance {
	type Opposite = PositiveImbalance;

	fn zero() -> Self {
        unimplemented!("NegativeImbalance::zero is not implemented");
	}
	fn drop_zero(self) -> result::Result<(), Self> {
		if self.1.is_zero() {
			Ok(())
		} else {
			Err(self)
		}
	}
	fn split(self, amount: Balance) -> (Self, Self) {
		let first = self.1.min(amount);
		let second = self.1 - first;
        let currency_id = self.0;

		mem::forget(self);
		(Self::new( currency_id, first), Self::new(currency_id, second))
	}
	fn merge(mut self, other: Self) -> Self {
        assert_eq!(self.0, other.0);
        self.1 = self.1.saturating_add(other.1);
        mem::forget(other);
        self
	}
	fn subsume(&mut self, other: Self) {
        assert_eq!(self.0, other.0);
        self.1 = self.1.saturating_add(other.1);
        mem::forget(other);
	}
	fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
        assert_eq!(self.0, other.0);
        let (a, b) = (self.1, other.1);
        let currency_id = self.0;
        mem::forget((self, other));
        if a >= b {
            Ok(Self::new(currency_id, a - b))
        } else {
            Err(PositiveImbalance::new(currency_id, b - a))
        }
	}
	fn peek(&self) -> Balance {
		self.1
	}
}

impl Drop for PositiveImbalance {
	/// Basic drop handler will just square up the total issuance.
	fn drop(&mut self) {
		TotalIssuance::mutate(self.0, |v| *v = v.saturating_add(self.1));
	}
}

impl Drop for NegativeImbalance {
	/// Basic drop handler will just square up the total issuance.
	fn drop(&mut self) {
		TotalIssuance::mutate(self.0, |v| *v = v.saturating_sub(self.1));
	}
}
