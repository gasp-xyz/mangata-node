// wrapping these imbalances in a private module is necessary to ensure absolute
// privacy of the inner member.
use crate::{TotalIssuance, Trait};
use frame_support::storage::StorageMap;
use frame_support::traits::{Get, Imbalance, TryDrop};
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{marker, mem, result};
use mangata_primitives::{TokenId, Balance};

/// Opaque, move-only struct with private fields that serves as a token
/// denoting that funds have been created without any equal and opposite
/// accounting.
#[must_use]
pub struct PositiveImbalance<GetCurrencyId: Get<TokenId>>(
	Balance,
	marker::PhantomData<GetCurrencyId>,
);

impl<GetCurrencyId: Get<TokenId>> PositiveImbalance<GetCurrencyId> {
	/// Create a new positive imbalance from a balance.
	pub fn new(amount: Balance) -> Self {
		PositiveImbalance(amount, marker::PhantomData::<GetCurrencyId>)
	}
}

/// Opaque, move-only struct with private fields that serves as a token
/// denoting that funds have been destroyed without any equal and opposite
/// accounting.
#[must_use]
pub struct NegativeImbalance<GetCurrencyId: Get<TokenId>>(
	Balance,
	marker::PhantomData<GetCurrencyId>,
);

impl<GetCurrencyId: Get<TokenId>> NegativeImbalance<GetCurrencyId> {
	/// Create a new negative imbalance from a balance.
	pub fn new(amount: Balance) -> Self {
		NegativeImbalance(amount, marker::PhantomData::<GetCurrencyId>)
	}
}

impl<GetCurrencyId: Get<TokenId>> TryDrop for PositiveImbalance<GetCurrencyId> {
	fn try_drop(self) -> result::Result<(), Self> {
		self.drop_zero()
	}
}

impl<GetCurrencyId: Get<TokenId>> Imbalance<Balance> for PositiveImbalance<GetCurrencyId> {
	type Opposite = NegativeImbalance<GetCurrencyId>;

	fn zero() -> Self {
		Self::new(Zero::zero())
	}
	fn drop_zero(self) -> result::Result<(), Self> {
		if self.0.is_zero() {
			Ok(())
		} else {
			Err(self)
		}
	}
	fn split(self, amount: Balance) -> (Self, Self) {
		let first = self.0.min(amount);
		let second = self.0 - first;

		mem::forget(self);
		(Self::new(first), Self::new(second))
	}
	fn merge(mut self, other: Self) -> Self {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);

		self
	}
	fn subsume(&mut self, other: Self) {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);
	}
	fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
		let (a, b) = (self.0, other.0);
		mem::forget((self, other));

		if a >= b {
			Ok(Self::new(a - b))
		} else {
			Err(NegativeImbalance::new(b - a))
		}
	}
	fn peek(&self) -> Balance {
		self.0
	}
}

impl<GetCurrencyId: Get<TokenId>> TryDrop for NegativeImbalance<GetCurrencyId> {
	fn try_drop(self) -> result::Result<(), Self> {
		self.drop_zero()
	}
}

impl<GetCurrencyId: Get<TokenId>> Imbalance<Balance> for NegativeImbalance<GetCurrencyId> {
	type Opposite = PositiveImbalance<GetCurrencyId>;

	fn zero() -> Self {
		Self::new(Zero::zero())
	}
	fn drop_zero(self) -> result::Result<(), Self> {
		if self.0.is_zero() {
			Ok(())
		} else {
			Err(self)
		}
	}
	fn split(self, amount: Balance) -> (Self, Self) {
		let first = self.0.min(amount);
		let second = self.0 - first;

		mem::forget(self);
		(Self::new(first), Self::new(second))
	}
	fn merge(mut self, other: Self) -> Self {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);

		self
	}
	fn subsume(&mut self, other: Self) {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);
	}
	fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
		let (a, b) = (self.0, other.0);
		mem::forget((self, other));

		if a >= b {
			Ok(Self::new(a - b))
		} else {
			Err(PositiveImbalance::new(b - a))
		}
	}
	fn peek(&self) -> Balance {
		self.0
	}
}

impl<GetCurrencyId: Get<TokenId>> Drop for PositiveImbalance<GetCurrencyId> {
	/// Basic drop handler will just square up the total issuance.
	fn drop(&mut self) {
		TotalIssuance::mutate(GetCurrencyId::get(), |v| *v = v.saturating_add(self.0));
	}
}

impl<GetCurrencyId: Get<TokenId>> Drop for NegativeImbalance<GetCurrencyId> {
	/// Basic drop handler will just square up the total issuance.
	fn drop(&mut self) {
		TotalIssuance::mutate(GetCurrencyId::get(), |v| *v = v.saturating_sub(self.0));
	}
}
