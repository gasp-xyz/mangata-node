// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! # Parachain Staking
//! Minimal staking pallet that implements collator selection by total backed stake.
//! The main difference between this pallet and `frame/pallet-staking` is that this pallet
//! uses direct delegation. Delegators choose exactly who they delegate and with what stake.
//! This is different from `frame/pallet-staking` where delegators approval vote and run Phragmen.
//!
//! ### Actors
//! There are multiple functions that can be distinguished:
//! - Collator Candidate - depending on managed stake can or can not bo chosen as collator
//! - Delegator - delegates to collator candidate, if that collator candidate will become
//! collator delegator is eligible for proportional part of the rewards that collator receives for
//! building blocks
//! - Aggregator - A collator candiate may choose to aggregate under an aggregator. If this aggregator
//! gets selected then he becomes an author/collator representing the collator candidates aggregating under him.
//! If a collator candidate does not choose to aggregate under an aggregator and gets selected, then he
//! himself becomes the author/collator. Any account that is not delegator or candidate can become
//! aggregator
//!
//! ### Rules
//! There is a new round every `<Round<T>>::get().length` blocks.
//!
//! At the start of every round,
//! * issuance is assigned to collators (and their delegators) for block authoring
//! `T::RewardPaymentDelay` rounds ago, afterwards it can be claimed using dedicated extrinsics
//! * queued collator and delegator exits are executed
//! * a new set of collators is chosen from the candidates
//!
//! To join the set of candidates, call `join_candidates` with `bond >= MinCandidateStk`.
//! To leave the set of candidates, call `schedule_leave_candidates`. If the call succeeds,
//! the collator is removed from the pool of candidates so they cannot be selected for future
//! collator sets, but they are not unbonded until their exit request is executed. Any signed
//! account may trigger the exit `T::LeaveCandidatesDelay` rounds after the round in which the
//! original request was made.
//!
//! To join the set of delegators, call `delegate` and pass in an account that is
//! already a collator candidate and `bond >= MinDelegatorStk`. Each delegator can delegate up to
//! `T::MaxDelegationsPerDelegator` collator candidates by calling `delegate`.
//!
//! To revoke a delegation, call `revoke_delegation` with the collator candidate's account.
//! To leave the set of delegators and revoke all delegations, call `leave_delegators`.
//!
//!
//! # Aggregation
//! Aggregation feature allows accumulating stake in different liquidity tokens under single aggregator account
//! by assosiating several candidates stake with that (aggregator) account. Each candidate needs to bond different
//! liquidity token
//! ```ignore
//!                            ####################
//!               -------------#  Aggregator A    #-------------
//!              |             #                  #             |
//!              |             ####################             |
//!              |                       |                      |
//!              |                       |                      |
//!              |                       |                      |
//!              |                       |                      |
//!              |                       |                      |
//!              |                       |                      |
//!      --------------------  --------------------  --------------------
//!      |  Candidate B     |  |  Candidate C     |  |  Candidate D     |
//!      | token: MGX:TUR   |  | token: MGX:IMBU  |  | token: MGX:MOVR  |
//!      --------------------  --------------------  --------------------
//! ```
//! If candidate decides to aggregate under Aggregator it cannot be chosen to be collator(the
//! candidate), instead aggregator account can be selected (even though its not present on
//! candidates list).
//!
//!
//! Block authors selection algorithm details [`Pallet::select_top_candidates`]
//!
//!```ignore
//!                        candidate B MGX valuation
//! Candidate B rewards = ------------------------------------ * Aggregator A total staking rewards
//!                        candidate ( B + C + D) valuation
//!```
//!
//! Extrinsics:
//! - [`Pallet::aggregator_update_metadata`] - enable/disable candidates for aggregation
//! - [`Pallet::update_candidate_aggregator`] - assign aggregator for candidate
//!
//! Storage entries:
//! - [`CandidateAggregator`]
//! - [`AggregatorMetadata`]
//! - [`RoundAggregatorInfo`]
//! - [`RoundCollatorRewardInfo`]
//!
//! ## Candidate selection mechanism
//! Aggregation feature modifies how collators are selected. Rules are as follows:
//! - Everything is valuated in `MGX` part of staked liquidity token. So if collator A has X MGX:KSM
//! liquidity tokens. And X MGX:KSM liquidity token is convertible to Y `MGX` and Z `KSM`. Then X
//! MGX:KSM tokens has valuation of Y.
//! - If candidate allows for staking native tokens number of native tokens/2 == candidate valuation.
//! - for aggregator(A) each aggregation account (such that aggregates under A) is valuated in MGX and
//! sumed.
//! - Candidates that aggregates under some other account cannot be selected as collators (but the
//! account they aggregate under can)
//! - Candidates with top MGX valuation are selected as collators
//!
//! # Manual payouts
//! Due to big cost of automatic rewards distribution (N transfers where N is total amount of all
//! rewarded collators & delegators) it was decided to switch to manual payouts mechanism. Instead
//! of automatically transferring all the rewards at the end session only rewards amount per account
//! is stored. Then collators & delegators can claim their rewards manually (after T::RewardPaymentDelay).
//!
//! Extrinsics:
//! - [`Pallet::payout_collator_rewards`] - supposed to be called by collator after every round.
//! - [`Pallet::payout_delegator_reward`] - backup solution for withdrawing rewards when collator
//!
//! Storage entries:
//! - [`RoundCollatorRewardInfo`]
//!
//! is not available.
//!
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(doc)]
use aquamarine::aquamarine;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
#[cfg(test)]
#[cfg(not(feature = "runtime-benchmarks"))]
mod mock;
mod set;
#[cfg(test)]
#[cfg(not(feature = "runtime-benchmarks"))]
mod tests;

use crate::set::OrderedSet;
use codec::{Decode, Encode};
use frame_support::{
	pallet,
	pallet_prelude::*,
	traits::{
		tokens::currency::MultiTokenCurrency, EstimateNextSessionRotation, ExistenceRequirement,
		Get,
	},
	transactional,
};
use frame_system::{pallet_prelude::*, RawOrigin};
pub use mangata_support::traits::{
	ComputeIssuance, GetIssuance, PoolCreateApi, ProofOfStakeRewardsApi,
	SequencerStakingProviderTrait, StakingReservesProviderTrait, Valuate, XykFunctionsTrait,
};
pub use mangata_types::multipurpose_liquidity::BondKind;
use orml_tokens::{MultiTokenCurrencyExtended, MultiTokenReservableCurrency};
use pallet_collective_mangata::GetMembers;
use scale_info::TypeInfo;
use sp_arithmetic::per_things::Rounding;
use sp_runtime::{
	helpers_128bit::multiply_by_rational_with_rounding,
	traits::{
		Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, SaturatedConversion,
		Saturating, Zero,
	},
	Perbill, Permill, RuntimeDebug,
};
use sp_staking::SessionIndex;
use sp_std::{
	cmp::Ordering,
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	convert::TryInto,
	prelude::*,
};

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

trait FromInfiniteZeros {
	type Output;
	fn from_zeros() -> Self::Output;
}

impl<D: Decode> FromInfiniteZeros for D {
	type Output = D;
	fn from_zeros() -> Self::Output {
		D::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()
	}
}

#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Debug, Clone)]
pub enum MetadataUpdateAction {
	ExtendApprovedCollators,
	RemoveApprovedCollators,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum RewardKind<AccountId> {
	Collator,
	Delegator(AccountId),
}

#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Debug, Clone)]
pub enum PayoutRounds {
	All,
	Partial(Vec<RoundIndex>),
}

#[pallet]
pub mod pallet {
	pub use super::*;

	/// Pallet for parachain staking
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum PairedOrLiquidityToken<CurrencyId> {
		Paired(CurrencyId),
		Liquidity(CurrencyId),
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Bond<AccountId, Balance, CurrencyId> {
		pub owner: AccountId,
		pub amount: Balance,
		pub liquidity_token: CurrencyId,
	}

	impl<A: Decode, B: Default, C: Default> Default for Bond<A, B, C> {
		fn default() -> Bond<A, B, C> {
			Bond {
				owner: A::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
					.expect("infinite length input; no invalid inputs for type; qed"),
				amount: B::default(),
				liquidity_token: C::default(),
			}
		}
	}

	impl<A, B: Default, C: Default> Bond<A, B, C> {
		pub fn from_owner(owner: A) -> Self {
			Bond { owner, amount: B::default(), liquidity_token: C::default() }
		}
	}

	impl<AccountId: Ord, Balance, CurrencyId> Eq for Bond<AccountId, Balance, CurrencyId> {}

	impl<AccountId: Ord, Balance, CurrencyId> Ord for Bond<AccountId, Balance, CurrencyId> {
		fn cmp(&self, other: &Self) -> Ordering {
			self.owner.cmp(&other.owner)
		}
	}

	impl<AccountId: Ord, Balance, CurrencyId> PartialOrd for Bond<AccountId, Balance, CurrencyId> {
		fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
			Some(self.cmp(other))
		}
	}

	impl<AccountId: Ord, Balance, CurrencyId> PartialEq for Bond<AccountId, Balance, CurrencyId> {
		fn eq(&self, other: &Self) -> bool {
			self.owner == other.owner
		}
	}

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// The activity status of the collator
	pub enum CollatorStatus {
		/// Committed to be online and producing valid blocks (not equivocating)
		Active,
		/// Temporarily inactive and excused for inactivity
		Idle,
		/// Bonded until the inner round
		Leaving(RoundIndex),
	}

	impl Default for CollatorStatus {
		fn default() -> CollatorStatus {
			CollatorStatus::Active
		}
	}

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Snapshot of collator state at the start of the round for which they are selected
	pub struct CollatorSnapshot<AccountId, Balance, CurrencyId> {
		pub bond: Balance,
		pub delegations: Vec<Bond<AccountId, Balance, CurrencyId>>,
		pub total: Balance,
		pub liquidity_token: CurrencyId,
	}

	impl<AccountId, Balance: Default, CurrencyId: Default> Default
		for CollatorSnapshot<AccountId, Balance, CurrencyId>
	{
		fn default() -> CollatorSnapshot<AccountId, Balance, CurrencyId> {
			Self {
				delegations: Default::default(),
				bond: Default::default(),
				total: Default::default(),
				liquidity_token: Default::default(),
			}
		}
	}

	#[derive(PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Changes allowed by an active collator candidate to their self bond
	pub enum CandidateBondChange {
		Increase,
		Decrease,
	}

	#[derive(PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Request scheduled to change the collator candidate self-bond
	pub struct CandidateBondRequest<Balance> {
		pub amount: Balance,
		pub change: CandidateBondChange,
		pub when_executable: RoundIndex,
	}

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Collator candidate state with self bond + delegations
	pub struct CollatorCandidate<AccountId, Balance, CurrencyId> {
		/// The account of this collator
		pub id: AccountId,
		/// This collator's self stake.
		pub bond: Balance,
		/// This is the liquidity_token the collator uses
		pub liquidity_token: CurrencyId,
		/// Set of all delegator AccountIds (to prevent >1 delegation per AccountId)
		pub delegators: OrderedSet<AccountId>,
		/// Top T::MaxDelegatorsPerCollator::get() delegations, ordered greatest to least
		pub top_delegations: Vec<Bond<AccountId, Balance, CurrencyId>>,
		/// Bottom delegations (unbounded), ordered least to greatest
		pub bottom_delegations: Vec<Bond<AccountId, Balance, CurrencyId>>,
		/// Sum of top delegations + self.bond
		pub total_counted: Balance,
		/// Sum of all delegations + self.bond = (total_counted + uncounted)
		pub total_backing: Balance,
		/// Maximum 1 pending request to adjust candidate self bond at any given time
		pub request: Option<CandidateBondRequest<Balance>>,
		/// Current status of the collator
		pub state: CollatorStatus,
	}

	/// Convey relevant information describing if a delegator was added to the top or bottom
	/// Delegations added to the top yield a new total
	#[derive(Clone, Copy, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum DelegatorAdded<Balance> {
		AddedToTop { new_total: Balance },
		AddedToBottom,
	}

	impl<A, Balance, CurrencyId> CollatorCandidate<A, Balance, CurrencyId>
	where
		A: Ord + Clone + sp_std::fmt::Debug,
		Balance: Default + PartialOrd + CheckedAdd + CheckedSub + Saturating + Ord + Copy,
		CurrencyId: Copy,
	{
		pub fn new(id: A, bond: Balance, liquidity_token: CurrencyId) -> Self {
			CollatorCandidate {
				id,
				bond,
				liquidity_token,
				delegators: OrderedSet::new(),
				top_delegations: Vec::new(),
				bottom_delegations: Vec::new(),
				total_counted: bond,
				total_backing: bond,
				request: None,
				state: CollatorStatus::default(), // default active
			}
		}
		pub fn is_active(&self) -> bool {
			self.state == CollatorStatus::Active
		}
		pub fn is_leaving(&self) -> bool {
			matches!(self.state, CollatorStatus::Leaving(_))
		}
		pub fn can_leave<T: Config>(&self) -> DispatchResult {
			if let CollatorStatus::Leaving(when) = self.state {
				ensure!(<Round<T>>::get().current >= when, Error::<T>::CandidateCannotLeaveYet);
				Ok(())
			} else {
				Err(Error::<T>::CandidateNotLeaving.into())
			}
		}
		/// Schedule executable increase of collator candidate self bond
		/// Returns the round at which the collator can execute the pending request
		pub fn schedule_bond_more<T: Config>(
			&mut self,
			more: Balance,
			use_balance_from: Option<BondKind>,
		) -> Result<RoundIndex, DispatchError>
		where
			T::AccountId: From<A>,
			BalanceOf<T>: From<Balance>,
			CurrencyIdOf<T>: From<CurrencyId>,
		{
			// ensure no pending request
			ensure!(self.request.is_none(), Error::<T>::PendingCandidateRequestAlreadyExists);
			let candidate_id: T::AccountId = self.id.clone().into();
			ensure!(
				<T as pallet::Config>::StakingReservesProvider::can_bond(
					self.liquidity_token.into(),
					&candidate_id,
					more.into(),
					use_balance_from
				),
				Error::<T>::InsufficientBalance
			);
			let when_executable =
				<Round<T>>::get().current.saturating_add(T::CandidateBondDelay::get());
			self.request = Some(CandidateBondRequest {
				change: CandidateBondChange::Increase,
				amount: more,
				when_executable,
			});
			Ok(when_executable)
		}
		/// Schedule executable decrease of collator candidate self bond
		/// Returns the round at which the collator can execute the pending request
		pub fn schedule_bond_less<T: Config>(
			&mut self,
			less: Balance,
		) -> Result<RoundIndex, DispatchError>
		where
			BalanceOf<T>: From<Balance>,
			CurrencyIdOf<T>: From<CurrencyId>,
		{
			// ensure no pending request
			ensure!(self.request.is_none(), Error::<T>::PendingCandidateRequestAlreadyExists);
			// ensure bond above min after decrease
			ensure!(self.bond > less, Error::<T>::CandidateBondBelowMin);

			let bond_valution_after = Pallet::<T>::valuate_bond(
				self.liquidity_token.into(),
				self.bond.checked_sub(&less).unwrap_or_default().into(),
			);
			ensure!(
				bond_valution_after >= T::MinCandidateStk::get(),
				Error::<T>::CandidateBondBelowMin
			);
			let when_executable =
				<Round<T>>::get().current.saturating_add(T::CandidateBondDelay::get());
			self.request = Some(CandidateBondRequest {
				change: CandidateBondChange::Decrease,
				amount: less,
				when_executable,
			});
			Ok(when_executable)
		}
		/// Execute pending request to change the collator self bond
		/// Returns the event to be emitted
		pub fn execute_pending_request<T: Config>(
			&mut self,
			use_balance_from: Option<BondKind>,
		) -> Result<Event<T>, DispatchError>
		where
			T::AccountId: From<A>,
			BalanceOf<T>: From<Balance>,
			CurrencyIdOf<T>: From<CurrencyId>,
		{
			let request = self.request.ok_or(Error::<T>::PendingCandidateRequestsDNE)?;
			ensure!(
				request.when_executable <= <Round<T>>::get().current,
				Error::<T>::PendingCandidateRequestNotDueYet
			);
			let caller: T::AccountId = self.id.clone().into();
			let event = match request.change {
				CandidateBondChange::Increase => {
					self.bond =
						self.bond.checked_add(&request.amount).ok_or(Error::<T>::MathError)?;

					self.total_counted = self
						.total_counted
						.checked_add(&request.amount)
						.ok_or(Error::<T>::MathError)?;
					self.total_backing = self
						.total_backing
						.checked_add(&request.amount)
						.ok_or(Error::<T>::MathError)?;
					<T as pallet::Config>::StakingReservesProvider::bond(
						self.liquidity_token.into(),
						&caller,
						request.amount.into(),
						use_balance_from,
					)?;
					let currency: CurrencyIdOf<T> = self.liquidity_token.into();
					let new_total = <Total<T>>::get(currency).saturating_add(request.amount.into());
					<Total<T>>::insert(currency, new_total);
					Event::CandidateBondedMore(
						self.id.clone().into(),
						request.amount.into(),
						self.bond.into(),
					)
				},
				CandidateBondChange::Decrease => {
					// Arithmetic assumptions are self.bond > less && self.bond - less > CollatorMinBond
					// (assumptions enforced by `schedule_bond_less`; if storage corrupts, must re-verify)
					self.bond =
						self.bond.checked_sub(&request.amount).ok_or(Error::<T>::MathError)?;

					self.total_counted = self
						.total_counted
						.checked_sub(&request.amount)
						.ok_or(Error::<T>::MathError)?;
					self.total_backing = self
						.total_backing
						.checked_sub(&request.amount)
						.ok_or(Error::<T>::MathError)?;
					let debug_amount = <T as pallet::Config>::StakingReservesProvider::unbond(
						self.liquidity_token.into(),
						&caller,
						request.amount.into(),
					);
					if !debug_amount.is_zero() {
						log::warn!("Unbond in staking returned non-zero value {:?}", debug_amount);
					}
					let currency: CurrencyIdOf<T> = self.liquidity_token.into();
					let new_total_staked =
						<Total<T>>::get(currency).saturating_sub(request.amount.into());
					<Total<T>>::insert(currency, new_total_staked);
					Event::CandidateBondedLess(
						self.id.clone().into(),
						request.amount.into(),
						self.bond.into(),
					)
				},
			};
			// reset s.t. no pending request
			self.request = None;
			// update candidate pool value because it must change if self bond changes
			if self.is_active() {
				Pallet::<T>::update_active(
					self.id.clone().into(),
					self.total_counted.into(),
					self.liquidity_token.into(),
				);
			}
			Ok(event)
		}
		/// Cancel pending request to change the collator self bond
		pub fn cancel_pending_request<T: Config>(&mut self) -> Result<Event<T>, DispatchError>
		where
			T::AccountId: From<A>,
			CandidateBondRequest<BalanceOf<T>>: From<CandidateBondRequest<Balance>>,
		{
			let request = self.request.ok_or(Error::<T>::PendingCandidateRequestsDNE)?;
			let event = Event::CancelledCandidateBondChange(self.id.clone().into(), request.into());
			self.request = None;
			Ok(event)
		}
		/// Infallible sorted insertion
		/// caller must verify !self.delegators.contains(delegation.owner) before call
		pub fn add_top_delegation(&mut self, delegation: Bond<A, Balance, CurrencyId>) {
			match self.top_delegations.binary_search_by(|x| delegation.amount.cmp(&x.amount)) {
				Ok(i) => self.top_delegations.insert(i, delegation),
				Err(i) => self.top_delegations.insert(i, delegation),
			}
		}
		/// Infallible sorted insertion
		/// caller must verify !self.delegators.contains(delegation.owner) before call
		pub fn add_bottom_delegation(&mut self, delegation: Bond<A, Balance, CurrencyId>) {
			match self.bottom_delegations.binary_search_by(|x| x.amount.cmp(&delegation.amount)) {
				Ok(i) => self.bottom_delegations.insert(i, delegation),
				Err(i) => self.bottom_delegations.insert(i, delegation),
			}
		}
		/// Sort top delegations from greatest to least
		pub fn sort_top_delegations(&mut self) {
			self.top_delegations.sort_unstable_by(|a, b| b.amount.cmp(&a.amount));
		}
		/// Sort bottom delegations from least to greatest
		pub fn sort_bottom_delegations(&mut self) {
			self.bottom_delegations.sort_unstable_by(|a, b| a.amount.cmp(&b.amount));
		}
		/// Bond account and add delegation. If successful, the return value indicates whether the
		/// delegation is top for the candidate.
		pub fn add_delegation<T: Config>(
			&mut self,
			acc: A,
			amount: Balance,
		) -> Result<DelegatorAdded<BalanceOf<T>>, DispatchError>
		where
			BalanceOf<T>: From<Balance>,
		{
			ensure!(self.delegators.insert(acc.clone()), Error::<T>::DelegatorExists);
			self.total_backing =
				self.total_backing.checked_add(&amount).ok_or(Error::<T>::MathError)?;
			if (self.top_delegations.len() as u32) < T::MaxDelegatorsPerCandidate::get() {
				self.add_top_delegation(Bond {
					owner: acc,
					amount,
					liquidity_token: self.liquidity_token,
				});
				self.total_counted =
					self.total_counted.checked_add(&amount).ok_or(Error::<T>::MathError)?;
				Ok(DelegatorAdded::AddedToTop { new_total: self.total_counted.into() })
			} else {
				// >pop requires push to reset in case isn't pushed to bottom
				let last_delegation_in_top = self
					.top_delegations
					.pop()
					.expect("self.top_delegations.len() >= T::Max exists >= 1 element in top");
				if amount > last_delegation_in_top.amount {
					// update total_counted with positive difference
					self.total_counted = self
						.total_counted
						.checked_add(
							&amount
								.checked_sub(&last_delegation_in_top.amount.into())
								.ok_or(Error::<T>::MathError)?,
						)
						.ok_or(Error::<T>::MathError)?;
					// last delegation already popped from top_delegations
					// insert new delegation into top_delegations
					self.add_top_delegation(Bond {
						owner: acc,
						amount,
						liquidity_token: self.liquidity_token,
					});
					self.add_bottom_delegation(last_delegation_in_top);
					Ok(DelegatorAdded::AddedToTop { new_total: self.total_counted.into() })
				} else {
					// >required push to previously popped last delegation into top_delegations
					self.top_delegations.push(last_delegation_in_top);
					self.add_bottom_delegation(Bond {
						owner: acc,
						amount,
						liquidity_token: self.liquidity_token,
					});
					Ok(DelegatorAdded::AddedToBottom)
				}
			}
		}
		/// Return Ok((if_total_counted_changed, delegation_amount))
		pub fn rm_delegator<T: Config>(
			&mut self,
			delegator: A,
		) -> Result<(bool, Balance), DispatchError> {
			ensure!(self.delegators.remove(&delegator), Error::<T>::DelegatorDNEInDelegatorSet);
			let mut delegation_amt: Option<Balance> = None;
			self.top_delegations = self
				.top_delegations
				.clone()
				.into_iter()
				.filter_map(|d| {
					if d.owner != delegator {
						Some(d)
					} else {
						delegation_amt = Some(d.amount);
						None
					}
				})
				.collect();
			// item removed from the top => highest bottom is popped from bottom and pushed to top
			if let Some(amount) = delegation_amt {
				// last element has largest amount as per ordering
				if let Some(last) = self.bottom_delegations.pop() {
					self.total_counted = self
						.total_counted
						.checked_sub(
							&amount.checked_sub(&last.amount).ok_or(Error::<T>::MathError)?,
						)
						.ok_or(Error::<T>::MathError)?;
					self.add_top_delegation(last);
				} else {
					// no item in bottom delegations so no item from bottom to pop and push up
					self.total_counted =
						self.total_counted.checked_sub(&amount).ok_or(Error::<T>::MathError)?;
				}
				self.total_backing =
					self.total_backing.checked_sub(&amount).ok_or(Error::<T>::MathError)?;
				return Ok((true, amount))
			}
			// else (no item removed from the top)
			self.bottom_delegations = self
				.bottom_delegations
				.clone()
				.into_iter()
				.filter_map(|d| {
					if d.owner != delegator {
						Some(d)
					} else {
						delegation_amt = Some(d.amount);
						None
					}
				})
				.collect();
			// if err, no item with account exists in top || bottom
			let amount = delegation_amt.ok_or(Error::<T>::DelegatorDNEinTopNorBottom)?;
			self.total_backing =
				self.total_backing.checked_sub(&amount).ok_or(Error::<T>::MathError)?;
			Ok((false, amount))
		}
		/// Return true if in_top after call
		/// Caller must verify before call that account is a delegator
		fn increase_delegation<T: Config>(
			&mut self,
			delegator: A,
			more: Balance,
		) -> Result<bool, DispatchError> {
			let mut in_top = false;
			for x in &mut self.top_delegations {
				if x.owner == delegator {
					x.amount = x.amount.checked_add(&more).ok_or(Error::<T>::MathError)?;
					self.total_counted =
						self.total_counted.checked_add(&more).ok_or(Error::<T>::MathError)?;
					self.total_backing =
						self.total_backing.checked_add(&more).ok_or(Error::<T>::MathError)?;
					in_top = true;
					break
				}
			}
			// if delegator was increased in top delegations
			if in_top {
				self.sort_top_delegations();
				return Ok(true)
			}
			// else delegator to increase must exist in bottom
			// >pop requires push later on to reset in case it isn't used
			let lowest_top = self
				.top_delegations
				.pop()
				.expect("any bottom delegations => must exist max top delegations");
			let mut move_2_top = false;
			for x in &mut self.bottom_delegations {
				if x.owner == delegator {
					x.amount = x.amount.checked_add(&more).ok_or(Error::<T>::MathError)?;
					self.total_backing =
						self.total_backing.checked_add(&more).ok_or(Error::<T>::MathError)?;
					move_2_top = x.amount > lowest_top.amount;
					break
				}
			}
			if move_2_top {
				self.sort_bottom_delegations();
				let highest_bottom = self.bottom_delegations.pop().expect("updated => exists");
				self.total_counted = self
					.total_counted
					.checked_add(
						&highest_bottom
							.amount
							.checked_sub(&lowest_top.amount)
							.ok_or(Error::<T>::MathError)?,
					)
					.ok_or(Error::<T>::MathError)?;
				self.add_top_delegation(highest_bottom);
				self.add_bottom_delegation(lowest_top);
				Ok(true)
			} else {
				// >required push to reset top_delegations from earlier pop
				self.top_delegations.push(lowest_top);
				self.sort_bottom_delegations();
				Ok(false)
			}
		}
		/// Return true if in_top after call
		pub fn decrease_delegation<T: Config>(
			&mut self,
			delegator: A,
			less: Balance,
		) -> Result<bool, DispatchError> {
			let mut in_top = false;
			let mut new_lowest_top: Option<Bond<A, Balance, CurrencyId>> = None;
			for x in &mut self.top_delegations {
				if x.owner == delegator {
					x.amount = x.amount.checked_sub(&less).ok_or(Error::<T>::MathError)?;
					// if there is at least 1 delegator in bottom delegators, compare it to check
					// if it should be swapped with lowest top delegation and put in top
					// >pop requires push later on to reset in case it isn't used
					if let Some(highest_bottom) = self.bottom_delegations.pop() {
						if highest_bottom.amount > x.amount {
							new_lowest_top = Some(highest_bottom);
						} else {
							// >required push to reset self.bottom_delegations
							self.bottom_delegations.push(highest_bottom);
						}
					}
					in_top = true;
					break
				}
			}
			if in_top {
				self.sort_top_delegations();
				if let Some(highest_bottom) = new_lowest_top {
					// pop last in top to swap it with top bottom
					let lowest_top = self
						.top_delegations
						.pop()
						.expect("must have >1 item to update, assign in_top = true");
					self.total_counted = self
						.total_counted
						.checked_sub(
							&lowest_top.amount.checked_add(&less).ok_or(Error::<T>::MathError)?,
						)
						.ok_or(Error::<T>::MathError)?;
					self.total_counted = self
						.total_counted
						.checked_add(&highest_bottom.amount)
						.ok_or(Error::<T>::MathError)?;
					self.total_backing =
						self.total_backing.checked_sub(&less).ok_or(Error::<T>::MathError)?;
					self.add_top_delegation(highest_bottom);
					self.add_bottom_delegation(lowest_top);
					return Ok(false)
				} else {
					// no existing bottom delegators so update both counters the same magnitude
					self.total_counted =
						self.total_counted.checked_sub(&less).ok_or(Error::<T>::MathError)?;
					self.total_backing =
						self.total_backing.checked_sub(&less).ok_or(Error::<T>::MathError)?;
					return Ok(true)
				}
			}
			for x in &mut self.bottom_delegations {
				if x.owner == delegator {
					x.amount = x.amount.checked_sub(&less).ok_or(Error::<T>::MathError)?;
					self.total_backing =
						self.total_backing.checked_sub(&less).ok_or(Error::<T>::MathError)?;
					break
				}
			}
			self.sort_bottom_delegations();
			Ok(false)
		}
		pub fn go_offline(&mut self) {
			self.state = CollatorStatus::Idle;
		}
		pub fn go_online(&mut self) {
			self.state = CollatorStatus::Active;
		}
		pub fn leave<T: Config>(&mut self) -> Result<(RoundIndex, RoundIndex), DispatchError> {
			ensure!(!self.is_leaving(), Error::<T>::CandidateAlreadyLeaving);
			let now = <Round<T>>::get().current;
			let when = now.saturating_add(T::LeaveCandidatesDelay::get());
			self.state = CollatorStatus::Leaving(when);
			Ok((now, when))
		}
	}

	impl<A: Clone, Balance, CurrencyId> From<CollatorCandidate<A, Balance, CurrencyId>>
		for CollatorSnapshot<A, Balance, CurrencyId>
	{
		fn from(
			other: CollatorCandidate<A, Balance, CurrencyId>,
		) -> CollatorSnapshot<A, Balance, CurrencyId> {
			CollatorSnapshot {
				bond: other.bond,
				delegations: other.top_delegations,
				total: other.total_counted,
				liquidity_token: other.liquidity_token,
			}
		}
	}

	#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum DelegatorStatus {
		/// Active with no scheduled exit
		Active,
		/// Schedule exit to revoke all ongoing delegations
		Leaving(RoundIndex),
	}

	impl Default for DelegatorStatus {
		fn default() -> DelegatorStatus {
			DelegatorStatus::Active
		}
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Delegator state
	pub struct Delegator<AccountId, Balance, CurrencyId> {
		/// Delegator account
		pub id: AccountId,
		/// All current delegations
		pub delegations: OrderedSet<Bond<AccountId, Balance, CurrencyId>>,
		/// Requests to change delegations, relevant iff active
		pub requests: PendingDelegationRequests<AccountId, Balance>,
		/// Status for this delegator
		pub status: DelegatorStatus,
	}

	impl<AccountId: Decode + Ord, Balance: Default, CurrencyId: Default> Default
		for Delegator<AccountId, Balance, CurrencyId>
	{
		fn default() -> Self {
			Self {
				id: AccountId::from_zeros(),
				delegations: Default::default(),
				requests: Default::default(),
				status: Default::default(),
			}
		}
	}

	impl<A, Balance, CurrencyId> Delegator<A, Balance, CurrencyId>
	where
		A: Ord + Clone,
		Balance: Ord + Copy + Saturating + CheckedAdd + CheckedSub,
		CurrencyId: Copy,
	{
		pub fn new(id: A, collator: A, amount: Balance, liquidity_token: CurrencyId) -> Self {
			Delegator {
				id,
				delegations: OrderedSet::from(vec![Bond {
					owner: collator,
					amount,
					liquidity_token,
				}]),
				requests: PendingDelegationRequests::new(),
				status: DelegatorStatus::Active,
			}
		}

		pub fn requests(&self) -> BTreeMap<A, DelegationRequest<A, Balance>> {
			self.requests.requests.clone()
		}

		pub fn is_active(&self) -> bool {
			matches!(self.status, DelegatorStatus::Active)
		}

		pub fn is_leaving(&self) -> bool {
			matches!(self.status, DelegatorStatus::Leaving(_))
		}

		/// Can only leave if the current round is less than or equal to scheduled execution round
		/// - returns None if not in leaving state
		pub fn can_execute_leave<T: Config>(&self, delegation_weight_hint: u32) -> DispatchResult {
			ensure!(
				delegation_weight_hint >= (self.delegations.0.len() as u32),
				Error::<T>::TooLowDelegationCountToLeaveDelegators
			);
			if let DelegatorStatus::Leaving(when) = self.status {
				ensure!(<Round<T>>::get().current >= when, Error::<T>::DelegatorCannotLeaveYet);
				Ok(())
			} else {
				Err(Error::<T>::DelegatorNotLeaving.into())
			}
		}

		/// Set status to leaving
		pub(crate) fn set_leaving(&mut self, when: RoundIndex) {
			self.status = DelegatorStatus::Leaving(when);
		}

		/// Schedule status to exit
		pub fn schedule_leave<T: Config>(&mut self) -> (RoundIndex, RoundIndex) {
			let now = <Round<T>>::get().current;
			let when = now.saturating_add(T::LeaveDelegatorsDelay::get());
			self.set_leaving(when);
			(now, when)
		}

		/// Set delegator status to active
		pub fn cancel_leave(&mut self) {
			self.status = DelegatorStatus::Active
		}

		pub fn add_delegation(&mut self, bond: Bond<A, Balance, CurrencyId>) -> bool {
			if self.delegations.insert(bond) {
				true
			} else {
				false
			}
		}

		// Return Some(remaining balance), must be more than MinDelegatorStk
		// Return None if delegation not found
		pub fn rm_delegation(&mut self, collator: A) -> Option<usize> {
			let mut amt: Option<Balance> = None;
			let delegations = self
				.delegations
				.0
				.iter()
				.filter_map(|x| {
					if x.owner == collator {
						amt = Some(x.amount);
						None
					} else {
						Some(x.clone())
					}
				})
				.collect();
			if let Some(_) = amt {
				self.delegations = OrderedSet::from(delegations);
				Some(self.delegations.0.len())
			} else {
				None
			}
		}

		/// Schedule increase delegation
		pub fn schedule_increase_delegation<T: Config>(
			&mut self,
			collator: A,
			more: Balance,
			use_balance_from: Option<BondKind>,
		) -> Result<RoundIndex, DispatchError>
		where
			T::AccountId: From<A>,
			BalanceOf<T>: From<Balance>,
			CurrencyIdOf<T>: From<CurrencyId>,
		{
			let Bond { liquidity_token, .. } = self
				.delegations
				.0
				.iter()
				.find(|b| b.owner == collator)
				.ok_or(Error::<T>::DelegationDNE)?;
			let delegator_id: T::AccountId = self.id.clone().into();
			ensure!(
				<T as pallet::Config>::StakingReservesProvider::can_bond(
					(*liquidity_token).into(),
					&delegator_id,
					more.into(),
					use_balance_from
				),
				Error::<T>::InsufficientBalance
			);
			let when = <Round<T>>::get().current.saturating_add(T::DelegationBondDelay::get());
			self.requests.bond_more::<T>(collator, more, when)?;
			Ok(when)
		}

		/// Schedule decrease delegation
		pub fn schedule_decrease_delegation<T: Config>(
			&mut self,
			collator: A,
			less: Balance,
		) -> Result<RoundIndex, DispatchError>
		where
			BalanceOf<T>: Into<Balance>,
		{
			// get delegation amount
			let Bond { amount, .. } = self
				.delegations
				.0
				.iter()
				.find(|b| b.owner == collator)
				.ok_or(Error::<T>::DelegationDNE)?;
			ensure!(
				*amount >= T::MinDelegation::get().into().saturating_add(less),
				Error::<T>::DelegationBelowMin
			);
			let when = <Round<T>>::get().current.saturating_add(T::DelegationBondDelay::get());
			self.requests.bond_less::<T>(collator, less, when)?;
			Ok(when)
		}

		/// Schedule revocation for the given collator
		pub fn schedule_revoke<T: Config>(
			&mut self,
			collator: A,
		) -> Result<(RoundIndex, RoundIndex), DispatchError> {
			// get delegation amount
			let Bond { amount, .. } = self
				.delegations
				.0
				.iter()
				.find(|b| b.owner == collator)
				.ok_or(Error::<T>::DelegationDNE)?;
			let now = <Round<T>>::get().current;
			let when = now.saturating_add(T::RevokeDelegationDelay::get());
			// add revocation to pending requests
			self.requests.revoke::<T>(collator, *amount, when)?;
			Ok((now, when))
		}

		/// Execute pending delegation change request
		pub fn execute_pending_request<T: Config>(
			&mut self,
			candidate: A,
			use_balance_from: Option<BondKind>,
		) -> DispatchResult
		where
			T::AccountId: From<A>,
			BalanceOf<T>: From<Balance> + Into<Balance>,
			CurrencyIdOf<T>: From<CurrencyId>,
			Delegator<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>:
				From<Delegator<A, Balance, CurrencyId>>,
		{
			let now = <Round<T>>::get().current;
			let DelegationRequest { amount, action, when_executable, .. } = self
				.requests
				.requests
				.remove(&candidate)
				.ok_or(Error::<T>::PendingDelegationRequestDNE)?;
			ensure!(when_executable <= now, Error::<T>::PendingDelegationRequestNotDueYet);
			let (balance_amt, candidate_id, delegator_id): (Balance, T::AccountId, T::AccountId) =
				(amount.into(), candidate.clone().into(), self.id.clone().into());
			match action {
				DelegationChange::Revoke => {
					// revoking last delegation => leaving set of delegators
					let leaving = if self.delegations.0.len() == 1usize { true } else { false };
					// remove delegation from delegator state
					self.rm_delegation(candidate.clone());
					// remove delegation from collator state delegations
					Pallet::<T>::delegator_leaves_collator(
						delegator_id.clone(),
						candidate_id.clone(),
					)?;
					Pallet::<T>::deposit_event(Event::DelegationRevoked(
						delegator_id.clone(),
						candidate_id,
						balance_amt.into(),
					));
					if leaving {
						<DelegatorState<T>>::remove(&delegator_id);
						Pallet::<T>::deposit_event(Event::DelegatorLeft(
							delegator_id,
							balance_amt.into(),
						));
					} else {
						let nom_st: Delegator<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>> =
							self.clone().into();
						<DelegatorState<T>>::insert(&delegator_id, nom_st);
					}
					Ok(())
				},
				DelegationChange::Increase => {
					// increase delegation
					for x in &mut self.delegations.0 {
						if x.owner == candidate {
							x.amount =
								x.amount.checked_add(&amount).ok_or(Error::<T>::MathError)?;
							// update collator state delegation
							let mut collator_state = <CandidateState<T>>::get(&candidate_id)
								.ok_or(Error::<T>::CandidateDNE)?;
							<T as pallet::Config>::StakingReservesProvider::bond(
								x.liquidity_token.into(),
								&self.id.clone().into(),
								balance_amt.into(),
								use_balance_from,
							)?;
							let before = collator_state.total_counted;
							let in_top = collator_state.increase_delegation::<T>(
								self.id.clone().into(),
								balance_amt.into(),
							)?;
							let after = collator_state.total_counted;
							if collator_state.is_active() && (before != after) {
								Pallet::<T>::update_active(
									candidate_id.clone(),
									after,
									collator_state.liquidity_token,
								);
							}
							let new_total_staked = <Total<T>>::get(collator_state.liquidity_token)
								.saturating_add(balance_amt.into());
							<Total<T>>::insert(collator_state.liquidity_token, new_total_staked);
							<CandidateState<T>>::insert(&candidate_id, collator_state);
							let nom_st: Delegator<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>> =
								self.clone().into();
							<DelegatorState<T>>::insert(&delegator_id, nom_st);
							Pallet::<T>::deposit_event(Event::DelegationIncreased(
								delegator_id,
								candidate_id,
								balance_amt.into(),
								in_top,
							));
							return Ok(())
						}
					}
					Err(Error::<T>::DelegationDNE.into())
				},
				DelegationChange::Decrease => {
					// decrease delegation
					for x in &mut self.delegations.0 {
						if x.owner == candidate {
							if x.amount > amount.saturating_add(T::MinDelegation::get().into()) {
								x.amount =
									x.amount.checked_sub(&amount).ok_or(Error::<T>::MathError)?;
								let mut collator = <CandidateState<T>>::get(&candidate_id)
									.ok_or(Error::<T>::CandidateDNE)?;
								let debug_amount =
									<T as pallet::Config>::StakingReservesProvider::unbond(
										x.liquidity_token.into(),
										&delegator_id,
										balance_amt.into(),
									);
								if !debug_amount.is_zero() {
									log::warn!(
										"Unbond in staking returned non-zero value {:?}",
										debug_amount
									);
								}
								let before = collator.total_counted;
								// need to go into decrease_delegation
								let in_top = collator.decrease_delegation::<T>(
									delegator_id.clone(),
									balance_amt.into(),
								)?;
								let after = collator.total_counted;
								if collator.is_active() && (before != after) {
									Pallet::<T>::update_active(
										candidate_id.clone(),
										after,
										collator.liquidity_token,
									);
								}
								let new_total_staked = <Total<T>>::get(collator.liquidity_token)
									.saturating_sub(balance_amt.into());
								<Total<T>>::insert(collator.liquidity_token, new_total_staked);
								<CandidateState<T>>::insert(&candidate_id, collator);
								let nom_st: Delegator<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>> =
									self.clone().into();
								<DelegatorState<T>>::insert(&delegator_id, nom_st);
								Pallet::<T>::deposit_event(Event::DelegationDecreased(
									delegator_id,
									candidate_id,
									balance_amt.into(),
									in_top,
								));
								return Ok(())
							} else {
								// must rm entire delegation if x.amount <= less or cancel request
								return Err(Error::<T>::DelegationBelowMin.into())
							}
						}
					}
					Err(Error::<T>::DelegationDNE.into())
				},
			}
		}

		/// Cancel pending delegation change request
		pub fn cancel_pending_request<T: Config>(
			&mut self,
			candidate: A,
		) -> Result<DelegationRequest<A, Balance>, DispatchError> {
			let order = self
				.requests
				.requests
				.remove(&candidate)
				.ok_or(Error::<T>::PendingDelegationRequestDNE)?;
			Ok(order)
		}
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Changes requested by the delegator
	/// - limit of 1 ongoing change per delegation
	/// - no changes allowed if delegator is leaving
	pub enum DelegationChange {
		Revoke,
		Increase,
		Decrease,
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct DelegationRequest<AccountId, Balance> {
		pub collator: AccountId,
		pub amount: Balance,
		pub when_executable: RoundIndex,
		pub action: DelegationChange,
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Pending requests to mutate delegations for each delegator
	pub struct PendingDelegationRequests<AccountId, Balance> {
		/// Map from collator -> Request (enforces at most 1 pending request per delegation)
		pub requests: BTreeMap<AccountId, DelegationRequest<AccountId, Balance>>,
	}

	impl<A: Ord, Balance> Default for PendingDelegationRequests<A, Balance> {
		fn default() -> PendingDelegationRequests<A, Balance> {
			PendingDelegationRequests { requests: BTreeMap::new() }
		}
	}

	impl<A: Ord + Clone, Balance> PendingDelegationRequests<A, Balance> {
		/// New default (empty) pending requests
		pub fn new() -> PendingDelegationRequests<A, Balance> {
			PendingDelegationRequests::default()
		}
		/// Add bond more order to pending requests
		pub fn bond_more<T: Config>(
			&mut self,
			collator: A,
			amount: Balance,
			when_executable: RoundIndex,
		) -> DispatchResult {
			ensure!(
				self.requests.get(&collator).is_none(),
				Error::<T>::PendingDelegationRequestAlreadyExists
			);
			self.requests.insert(
				collator.clone(),
				DelegationRequest {
					collator,
					amount,
					when_executable,
					action: DelegationChange::Increase,
				},
			);
			Ok(())
		}
		/// Add bond less order to pending requests, only succeeds if returns true
		/// - limit is the maximum amount allowed that can be subtracted from the delegation
		/// before it would be below the minimum delegation amount
		pub fn bond_less<T: Config>(
			&mut self,
			collator: A,
			amount: Balance,
			when_executable: RoundIndex,
		) -> DispatchResult {
			ensure!(
				self.requests.get(&collator).is_none(),
				Error::<T>::PendingDelegationRequestAlreadyExists
			);
			self.requests.insert(
				collator.clone(),
				DelegationRequest {
					collator,
					amount,
					when_executable,
					action: DelegationChange::Decrease,
				},
			);
			Ok(())
		}
		/// Add revoke order to pending requests
		/// - limit is the maximum amount allowed that can be subtracted from the delegation
		/// before it would be below the minimum delegation amount
		pub fn revoke<T: Config>(
			&mut self,
			collator: A,
			amount: Balance,
			when_executable: RoundIndex,
		) -> DispatchResult {
			ensure!(
				self.requests.get(&collator).is_none(),
				Error::<T>::PendingDelegationRequestAlreadyExists
			);
			self.requests.insert(
				collator.clone(),
				DelegationRequest {
					collator,
					amount,
					when_executable,
					action: DelegationChange::Revoke,
				},
			);
			Ok(())
		}
	}

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// The current round index and transition information
	pub struct RoundInfo<BlockNumber> {
		/// Current round index
		pub current: RoundIndex,
		/// The first block of the current round
		pub first: BlockNumber,
		/// The length of the current round in number of blocks
		pub length: u32,
	}
	impl<
			B: Copy
				+ sp_std::ops::Add<Output = B>
				+ sp_std::ops::Sub<Output = B>
				+ From<u32>
				+ PartialOrd
				+ One
				+ Zero
				+ Saturating
				+ CheckedMul,
		> RoundInfo<B>
	{
		pub fn new(current: RoundIndex, first: B, length: u32) -> RoundInfo<B> {
			RoundInfo { current, first, length }
		}
		/// Check if the round should be updated
		pub fn should_update(&self, now: B) -> bool {
			now.saturating_add(One::one()) >= self.first.saturating_add(self.length.into())
		}
		/// New round
		pub fn update(&mut self, now: B) {
			self.current = self.current.saturating_add(1u32);
			self.first = now;
		}
	}
	impl<
			B: Copy
				+ sp_std::ops::Add<Output = B>
				+ sp_std::ops::Sub<Output = B>
				+ From<u32>
				+ PartialOrd
				+ One
				+ Zero
				+ Saturating
				+ CheckedMul,
		> Default for RoundInfo<B>
	{
		fn default() -> RoundInfo<B> {
			RoundInfo::new(0u32, Zero::zero(), 20u32)
		}
	}

	pub(crate) type RoundIndex = u32;
	type RewardPoint = u32;

	#[cfg(feature = "runtime-benchmarks")]
	pub trait StakingBenchmarkConfig: pallet_session::Config + pallet_issuance::Config {
		type Balance;
		type CurrencyId;
		type RewardsApi: ProofOfStakeRewardsApi<Self::AccountId, Self::Balance, Self::CurrencyId>;
		type Xyk: XykFunctionsTrait<Self::AccountId, Self::Balance, Self::CurrencyId>;
	}

	#[cfg(not(feature = "runtime-benchmarks"))]
	pub trait StakingBenchmarkConfig {}

	pub type BalanceOf<T> = <<T as Config>::Currency as MultiTokenCurrency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	pub type CurrencyIdOf<T> = <<T as Config>::Currency as MultiTokenCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + StakingBenchmarkConfig {
		/// Overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Multipurpose-liquidity
		type StakingReservesProvider: StakingReservesProviderTrait<
			Self::AccountId,
			BalanceOf<Self>,
			CurrencyIdOf<Self>,
		>;
		/// The currency type
		type Currency: MultiTokenCurrency<Self::AccountId>
			+ MultiTokenReservableCurrency<Self::AccountId>
			+ MultiTokenCurrencyExtended<Self::AccountId>;
		/// The origin for monetary governance
		type MonetaryGovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Default number of blocks per round at genesis
		#[pallet::constant]
		type BlocksPerRound: Get<u32>;
		/// Number of rounds that candidates remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveCandidatesDelay: Get<RoundIndex>;
		/// Number of rounds that candidate requests to adjust self-bond must wait to be executable
		#[pallet::constant]
		type CandidateBondDelay: Get<RoundIndex>;
		/// Number of rounds that delegators remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveDelegatorsDelay: Get<RoundIndex>;
		/// Number of rounds that delegations remain bonded before revocation request is executable
		#[pallet::constant]
		type RevokeDelegationDelay: Get<RoundIndex>;
		/// Number of rounds that delegation {more, less} requests must wait before executable
		#[pallet::constant]
		type DelegationBondDelay: Get<RoundIndex>;
		/// Number of rounds after which block authors are rewarded
		#[pallet::constant]
		type RewardPaymentDelay: Get<RoundIndex>;
		/// Minimum number of selected candidates every round
		#[pallet::constant]
		type MinSelectedCandidates: Get<u32>;
		/// Maximum collator candidates allowed
		#[pallet::constant]
		type MaxCollatorCandidates: Get<u32>;
		/// Maximum delegators allowed per candidate
		#[pallet::constant]
		type MaxTotalDelegatorsPerCandidate: Get<u32>;
		/// Maximum delegators counted per candidate
		#[pallet::constant]
		type MaxDelegatorsPerCandidate: Get<u32>;
		#[pallet::constant]
		type DefaultPayoutLimit: Get<u32>;
		/// Maximum delegations per delegator
		#[pallet::constant]
		type MaxDelegationsPerDelegator: Get<u32>;
		/// Default commission due to collators, is `CollatorCommission` storage value in genesis
		#[pallet::constant]
		type DefaultCollatorCommission: Get<Perbill>;
		/// Minimum stake required for any candidate to be in `SelectedCandidates` for the round
		#[pallet::constant]
		type MinCollatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be a collator candidate
		#[pallet::constant]
		type MinCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to delegate
		#[pallet::constant]
		type MinDelegation: Get<BalanceOf<Self>>;
		/// The native token used for payouts
		#[pallet::constant]
		type NativeTokenId: Get<CurrencyIdOf<Self>>;
		/// The valuator for our staking liquidity tokens, i.e., XYK
		/// This should never return (_, Zero::zero())
		type StakingLiquidityTokenValuator: Valuate<BalanceOf<Self>, CurrencyIdOf<Self>>;
		/// The module used for computing and getting issuance
		type Issuance: ComputeIssuance + GetIssuance<BalanceOf<Self>>;
		#[pallet::constant]
		/// The account id that holds the liquidity mining issuance
		type StakingIssuanceVault: Get<Self::AccountId>;
		/// The module that provides the set of fallback accounts
		type FallbackProvider: GetMembers<Self::AccountId>;
		/// The module that provides the info and processing for the sequencer stakes
		type SequencerStakingProvider: SequencerStakingProviderTrait<
			Self::AccountId,
			BalanceOf<Self>,
		>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		DelegatorDNE,
		DelegatorDNEinTopNorBottom,
		DelegatorDNEInDelegatorSet,
		CandidateDNE,
		DelegationDNE,
		DelegatorExists,
		CandidateExists,
		CandidateBondBelowMin,
		InsufficientBalance,
		DelegationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		DelegatorAlreadyLeaving,
		DelegatorNotLeaving,
		DelegatorCannotLeaveYet,
		CannotDelegateIfLeaving,
		CandidateAlreadyLeaving,
		CandidateNotLeaving,
		CandidateCannotLeaveYet,
		CannotGoOnlineIfLeaving,
		ExceedMaxDelegationsPerDelegator,
		AlreadyDelegatedCandidate,
		InvalidSchedule,
		CannotSetBelowMin,
		NoWritingSameValue,
		TooLowCandidateCountWeightHintJoinCandidates,
		TooLowCandidateCountWeightHintCancelLeaveCandidates,
		TooLowCandidateCountToLeaveCandidates,
		TooLowDelegationCountToDelegate,
		TooLowCandidateDelegationCountToDelegate,
		TooLowDelegationCountToLeaveDelegators,
		PendingCandidateRequestsDNE,
		PendingCandidateRequestAlreadyExists,
		PendingCandidateRequestNotDueYet,
		PendingDelegationRequestDNE,
		PendingDelegationRequestAlreadyExists,
		PendingDelegationRequestNotDueYet,
		StakingLiquidityTokenNotListed,
		TooLowCurrentStakingLiquidityTokensCount,
		StakingLiquidityTokenAlreadyListed,
		ExceedMaxCollatorCandidates,
		ExceedMaxTotalDelegatorsPerCandidate,
		CandidateNotAggregating,
		CandidateNotAggregatingUnderAggregator,
		CandidateAlreadyApprovedByAggregator,
		AggregatorExists,
		CollatorRoundRewardsDNE,
		DelegatorRewardsDNE,
		AggregatorDNE,
		TargettedAggregatorSameAsCurrent,
		CandidateNotApprovedByAggregator,
		AggregatorLiquidityTokenTaken,
		IncorrectRewardDelegatorCount,
		MathError,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Starting Block, Round, Number of Collators Selected, Total Balance
		NewRound(BlockNumberFor<T>, RoundIndex, u32, BalanceOf<T>),
		/// Account, Amount Locked, New Total Amt Locked
		JoinedCollatorCandidates(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Round, Collator Account, Total Exposed Amount (includes all delegations)
		CollatorChosen(RoundIndex, T::AccountId, BalanceOf<T>),
		/// Candidate, Amount To Increase, Round at which request can be executed by caller
		CandidateBondMoreRequested(T::AccountId, BalanceOf<T>, RoundIndex),
		/// Candidate, Amount To Decrease, Round at which request can be executed by caller
		CandidateBondLessRequested(T::AccountId, BalanceOf<T>, RoundIndex),
		/// Candidate, Amount, New Bond Total
		CandidateBondedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Candidate, Amount, New Bond
		CandidateBondedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Round Offline, Candidate
		CandidateWentOffline(RoundIndex, T::AccountId),
		/// Round Online, Candidate
		CandidateBackOnline(RoundIndex, T::AccountId),
		/// Round At Which Exit Is Allowed, Candidate, Scheduled Exit
		CandidateScheduledExit(RoundIndex, T::AccountId, RoundIndex),
		/// Candidate
		CancelledCandidateExit(T::AccountId),
		/// Candidate, Cancelled Request
		CancelledCandidateBondChange(T::AccountId, CandidateBondRequest<BalanceOf<T>>),
		/// Ex-Candidate, Amount Unlocked, New Total Amt Locked
		CandidateLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Delegator, Candidate, Amount to be increased, Round at which can be executed
		DelegationIncreaseScheduled(T::AccountId, T::AccountId, BalanceOf<T>, RoundIndex),
		/// Delegator, Candidate, Amount to be decreased, Round at which can be executed
		DelegationDecreaseScheduled(T::AccountId, T::AccountId, BalanceOf<T>, RoundIndex),
		// Delegator, Candidate, Amount, If in top delegations for candidate after increase
		DelegationIncreased(T::AccountId, T::AccountId, BalanceOf<T>, bool),
		// Delegator, Candidate, Amount, If in top delegations for candidate after decrease
		DelegationDecreased(T::AccountId, T::AccountId, BalanceOf<T>, bool),
		/// Round, Delegator, Scheduled Exit
		DelegatorExitScheduled(RoundIndex, T::AccountId, RoundIndex),
		/// Round, Delegator, Candidate, Scheduled Exit
		DelegationRevocationScheduled(RoundIndex, T::AccountId, T::AccountId, RoundIndex),
		/// Delegator, Amount Unstaked
		DelegatorLeft(T::AccountId, BalanceOf<T>),
		/// Delegator, Candidate, Amount Unstaked
		DelegationRevoked(T::AccountId, T::AccountId, BalanceOf<T>),
		/// Delegator
		DelegatorExitCancelled(T::AccountId),
		/// Delegator, Cancelled Request
		CancelledDelegationRequest(T::AccountId, DelegationRequest<T::AccountId, BalanceOf<T>>),
		/// Delegator, Amount Locked, Candidate, Delegator Position with New Total Counted if in Top
		Delegation(T::AccountId, BalanceOf<T>, T::AccountId, DelegatorAdded<BalanceOf<T>>),
		/// Delegator, Candidate, Amount Unstaked, New Total Amt Staked for Candidate
		DelegatorLeftCandidate(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Session index, Delegator, Collator, Due reward (as per counted delegation for collator)
		DelegatorDueReward(RoundIndex, T::AccountId, T::AccountId, BalanceOf<T>),
		/// Paid the account (delegator or collator) the balance as liquid rewards
		Rewarded(RoundIndex, T::AccountId, BalanceOf<T>),
		/// Notify about reward periods that has been paid (collator, payout rounds, any rewards left)
		CollatorRewardsDistributed(T::AccountId, PayoutRounds),
		/// Staking expectations set
		StakeExpectationsSet(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// Set total selected candidates to this value [old, new]
		TotalSelectedSet(u32, u32),
		/// Set collator commission to this value [old, new]
		CollatorCommissionSet(Perbill, Perbill),
		/// A candidate updated aggregator
		CandidateAggregatorUpdated(T::AccountId, Option<T::AccountId>),
		/// An agggregator's metadata has been updated
		AggregatorMetadataUpdated(T::AccountId),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(_now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
			// some extra offset on top
			let claim_cost = <T as Config>::WeightInfo::payout_collator_rewards();
			if remaining_weight.ref_time() > claim_cost.ref_time() {
				if let Some((collator, _round)) = RoundCollatorRewardInfo::<T>::iter_keys().next() {
					let _ = Self::do_payout_collator_rewards(collator, Some(1));
				}
			}

			claim_cost
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn collator_commission)]
	/// Commission percent taken off of rewards for all collators
	type CollatorCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_selected)]
	/// The total candidates selected every round
	pub(crate) type TotalSelected<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	pub(crate) type Round<T: Config> = StorageValue<_, RoundInfo<BlockNumberFor<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn delegator_state)]
	/// Get delegator state associated with an account if account is delegating else None
	pub(crate) type DelegatorState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegator<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_state)]
	/// Get collator candidate state associated with an account if account is a candidate else None
	pub(crate) type CandidateState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		CollatorCandidate<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn selected_candidates)]
	/// The collator candidates selected for the current round
	/// Block authors selection algorithm details [`Pallet::select_top_candidates`]
	type SelectedCandidates<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total)]
	/// Total capital locked by this staking pallet
	type Total<T: Config> = StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	/// The pool of collator candidates, each with their total backing stake
	type CandidatePool<T: Config> =
		StorageValue<_, OrderedSet<Bond<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	/// Snapshot of collator delegation stake at the start of the round
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		CollatorSnapshot<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn points)]
	/// Total points awarded to collators for block production in the round
	pub type Points<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, RewardPoint, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn awarded_pts)]
	/// Points for each collator per round
	pub type AwardedPts<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		RewardPoint,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn staking_liquidity_tokens)]
	pub type StakingLiquidityTokens<T: Config> = StorageValue<
		_,
		BTreeMap<CurrencyIdOf<T>, Option<(BalanceOf<T>, BalanceOf<T>)>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_candidate_aggregator)]
	/// Maps collator to its aggregator
	pub type CandidateAggregator<T: Config> =
		StorageValue<_, BTreeMap<T::AccountId, T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_aggregator_metadata)]
	/// Stores information about approved candidates for aggregation
	pub type AggregatorMetadata<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		AggregatorMetadataType<T::AccountId, CurrencyIdOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_round_aggregator_info)]
	/// Stored once per session, maps aggregator to list of assosiated candidates
	pub type RoundAggregatorInfo<T: Config> = StorageMap<
		_,
		Twox64Concat,
		RoundIndex,
		BTreeMap<T::AccountId, BTreeMap<T::AccountId, BalanceOf<T>>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_round_collator_reward_info)]
	/// Stores information about rewards per each session
	pub type RoundCollatorRewardInfo<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		RoundIndex,
		RoundCollatorRewardInfoType<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct RoundCollatorRewardInfoType<AccountId, Balance> {
		pub collator_reward: Balance,
		pub delegator_rewards: BTreeMap<AccountId, Balance>,
	}

	impl<AccountId, Balance: Default> Default for RoundCollatorRewardInfoType<AccountId, Balance> {
		fn default() -> RoundCollatorRewardInfoType<AccountId, Balance> {
			Self { collator_reward: Default::default(), delegator_rewards: Default::default() }
		}
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct AggregatorMetadataType<AccountId, CurrencyId> {
		pub token_collator_map: BTreeMap<CurrencyId, AccountId>,
		pub approved_candidates: BTreeSet<AccountId>,
	}

	impl<AccountId, CurrencyId: Default> Default for AggregatorMetadataType<AccountId, CurrencyId> {
		fn default() -> AggregatorMetadataType<AccountId, CurrencyId> {
			Self { token_collator_map: Default::default(), approved_candidates: Default::default() }
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub candidates: Vec<(T::AccountId, BalanceOf<T>, CurrencyIdOf<T>)>,
		pub delegations: Vec<(T::AccountId, T::AccountId, BalanceOf<T>)>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { candidates: vec![], delegations: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let mut liquidity_token_list: Vec<CurrencyIdOf<T>> = self
				.candidates
				.iter()
				.cloned()
				.map(|(_, _, l)| l)
				.collect::<Vec<CurrencyIdOf<T>>>();
			liquidity_token_list.sort();
			liquidity_token_list.dedup();
			let liquidity_token_count: u32 = liquidity_token_list.len().try_into().unwrap();
			for (i, liquidity_token) in liquidity_token_list.iter().enumerate() {
				if let Err(error) = <Pallet<T>>::add_staking_liquidity_token(
					RawOrigin::Root.into(),
					PairedOrLiquidityToken::Liquidity(*liquidity_token),
					i as u32,
				) {
					log::warn!(
						"Adding staking liquidity token failed in genesis with error {:?}",
						error
					);
				}
			}
			let mut candidate_count = 0u32;
			// Initialize the candidates
			for &(ref candidate, balance, liquidity_token) in &self.candidates {
				assert!(
					<T as pallet::Config>::Currency::available_balance(
						liquidity_token.into(),
						candidate
					) >= balance,
					"Account does not have enough balance to bond as a candidate."
				);
				candidate_count = candidate_count.saturating_add(1u32);
				if let Err(error) = <Pallet<T>>::join_candidates(
					T::RuntimeOrigin::from(Some(candidate.clone()).into()),
					balance,
					liquidity_token,
					None,
					candidate_count,
					liquidity_token_count,
				) {
					log::warn!("Join candidates failed in genesis with error {:?}", error);
				} else {
					candidate_count = candidate_count.saturating_add(1u32);
				}
			}
			let mut col_delegator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			let mut del_delegation_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			// Initialize the delegations
			for &(ref delegator, ref target, balance) in &self.delegations {
				let associated_collator = self.candidates.iter().find(|b| b.0 == *target);
				let collator_liquidity_token =
					associated_collator.expect("Delegation to non-existant collator").2;
				assert!(
					<T as pallet::Config>::Currency::available_balance(
						collator_liquidity_token.into(),
						delegator
					) >= balance,
					"Account does not have enough balance to place delegation."
				);
				let cd_count =
					if let Some(x) = col_delegator_count.get(target) { *x } else { 0u32 };
				let dd_count =
					if let Some(x) = del_delegation_count.get(delegator) { *x } else { 0u32 };
				if let Err(error) = <Pallet<T>>::delegate(
					T::RuntimeOrigin::from(Some(delegator.clone()).into()),
					target.clone(),
					balance,
					None,
					cd_count,
					dd_count,
				) {
					log::warn!("Delegate failed in genesis with error {:?}", error);
				} else {
					if let Some(x) = col_delegator_count.get_mut(target) {
						*x = x.saturating_add(1u32);
					} else {
						col_delegator_count.insert(target.clone(), 1u32);
					};
					if let Some(x) = del_delegation_count.get_mut(delegator) {
						*x = x.saturating_add(1u32);
					} else {
						del_delegation_count.insert(delegator.clone(), 1u32);
					};
				}
			}
			// Set collator commission to default config
			<CollatorCommission<T>>::put(T::DefaultCollatorCommission::get());
			// Set total selected candidates to minimum config
			<TotalSelected<T>>::put(T::MinSelectedCandidates::get());
			// Choose top TotalSelected collator candidates
			let (v_count, _, total_relevant_exposure) = <Pallet<T>>::select_top_candidates(1u32);
			// Start Round 0 at Block 0
			let round: RoundInfo<BlockNumberFor<T>> =
				RoundInfo::new(0u32, 0u32.into(), <T as pallet::Config>::BlocksPerRound::get());
			<Round<T>>::put(round);
			// So that round 0 can be rewarded
			for atstake in <AtStake<T>>::iter_prefix(1u32) {
				<AtStake<T>>::insert(0u32, atstake.0, atstake.1);
			}
			<Pallet<T>>::deposit_event(Event::NewRound(
				BlockNumberFor::<T>::zero(),
				0u32,
				v_count,
				total_relevant_exposure,
			));
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::set_total_selected())]
		/// Set the total number of collator candidates selected per round
		/// - changes are not applied until the start of the next round
		pub fn set_total_selected(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(new >= T::MinSelectedCandidates::get(), Error::<T>::CannotSetBelowMin);
			let old = <TotalSelected<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<TotalSelected<T>>::put(new);
			Self::deposit_event(Event::TotalSelectedSet(old, new));
			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::set_collator_commission())]
		/// Set the commission for all collators
		pub fn set_collator_commission(
			origin: OriginFor<T>,
			new: Perbill,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let old = <CollatorCommission<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<CollatorCommission<T>>::put(new);
			Self::deposit_event(Event::CollatorCommissionSet(old, new));
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::join_candidates(*candidate_count, *liquidity_token_count))]
		/// Join the set of collator candidates
		pub fn join_candidates(
			origin: OriginFor<T>,
			bond: BalanceOf<T>,
			liquidity_token: CurrencyIdOf<T>,
			use_balance_from: Option<BondKind>,
			candidate_count: u32,
			liquidity_token_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			ensure!(!Self::is_delegator(&acc), Error::<T>::DelegatorExists);
			ensure!(!Self::is_aggregator(&acc), Error::<T>::AggregatorExists);
			let staking_liquidity_tokens = <StakingLiquidityTokens<T>>::get();

			ensure!(
				liquidity_token_count as usize >= staking_liquidity_tokens.len(),
				Error::<T>::TooLowCurrentStakingLiquidityTokensCount
			);
			ensure!(
				staking_liquidity_tokens.contains_key(&liquidity_token) ||
					liquidity_token == T::NativeTokenId::get(),
				Error::<T>::StakingLiquidityTokenNotListed
			);

			ensure!(
				Self::valuate_bond(liquidity_token, bond) >= T::MinCandidateStk::get(),
				Error::<T>::CandidateBondBelowMin
			);
			let mut candidates = <CandidatePool<T>>::get();
			let old_count = candidates.0.len() as u32;
			// This is a soft check
			// Reinforced by similar check in go_online and cancel_leave_candidates
			ensure!(
				old_count < T::MaxCollatorCandidates::get(),
				Error::<T>::ExceedMaxCollatorCandidates
			);
			ensure!(
				candidate_count >= old_count,
				Error::<T>::TooLowCandidateCountWeightHintJoinCandidates
			);
			ensure!(
				candidates.insert(Bond { owner: acc.clone(), amount: bond, liquidity_token }),
				Error::<T>::CandidateExists
			);
			// reserve must be called before storage changes
			// and before any unsafe math operations with `bond: Balance`
			<T as pallet::Config>::StakingReservesProvider::bond(
				liquidity_token,
				&acc,
				bond,
				use_balance_from,
			)?;
			let candidate = CollatorCandidate::new(acc.clone(), bond, liquidity_token);
			<CandidateState<T>>::insert(&acc, candidate);
			<CandidatePool<T>>::put(candidates);
			let new_total = <Total<T>>::get(liquidity_token).saturating_add(bond);
			<Total<T>>::insert(liquidity_token, new_total);
			Self::deposit_event(Event::JoinedCollatorCandidates(acc, bond, new_total));
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_leave_candidates(*candidate_count))]
		/// Request to leave the set of candidates. If successful, the account is immediately
		/// removed from the candidate pool to prevent selection as a collator.
		pub fn schedule_leave_candidates(
			origin: OriginFor<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let (now, when) = state.leave::<T>()?;
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidate_count >= candidates.0.len() as u32,
				Error::<T>::TooLowCandidateCountToLeaveCandidates
			);
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateScheduledExit(now, collator, when));
			Ok(().into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::execute_leave_candidates(*candidate_delegation_count))]
		/// Execute leave candidates request
		pub fn execute_leave_candidates(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			candidate_delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let state = <CandidateState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(
				state.delegators.0.len() <= candidate_delegation_count as usize,
				Error::<T>::TooLowCandidateCountToLeaveCandidates
			);
			state.can_leave::<T>()?;

			let return_stake = |bond: Bond<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>| {
				let debug_amount = <T as pallet::Config>::StakingReservesProvider::unbond(
					bond.liquidity_token.into(),
					&bond.owner,
					bond.amount,
				);
				if !debug_amount.is_zero() {
					log::warn!("Unbond in staking returned non-zero value {:?}", debug_amount);
				}
				// remove delegation from delegator state
				let mut delegator = DelegatorState::<T>::get(&bond.owner).expect(
					"Collator state and delegator state are consistent.
						Collator state has a record of this delegation. Therefore,
						Delegator state also has a record. qed.",
				);
				if let Some(remaining_delegations) = delegator.rm_delegation(candidate.clone()) {
					if remaining_delegations.is_zero() {
						<DelegatorState<T>>::remove(&bond.owner);
					} else {
						let _ = delegator.requests.requests.remove(&candidate);
						<DelegatorState<T>>::insert(&bond.owner, delegator);
					}
				}
			};
			// return all top delegations
			for bond in state.top_delegations {
				return_stake(bond);
			}
			// return all bottom delegations
			for bond in state.bottom_delegations {
				return_stake(bond);
			}
			// return stake to collator
			let debug_amount = <T as pallet::Config>::StakingReservesProvider::unbond(
				state.liquidity_token.into(),
				&state.id,
				state.bond,
			);
			if !debug_amount.is_zero() {
				log::warn!("Unbond in staking returned non-zero value {:?}", debug_amount);
			}

			let res = Self::do_update_candidate_aggregator(&candidate, None);
			match res {
				Err(e) if e == DispatchError::from(Error::<T>::CandidateNotAggregating) => {},
				Err(_) => {
					log::error!("do_update_candidate_aggregator failed with error {:?}", res);
				},
				Ok(_) => {},
			}

			<CandidateState<T>>::remove(&candidate);
			let new_total_staked =
				<Total<T>>::get(state.liquidity_token).saturating_sub(state.total_backing);
			<Total<T>>::insert(state.liquidity_token, new_total_staked);
			Self::deposit_event(Event::CandidateLeft(
				candidate,
				state.total_backing,
				new_total_staked,
			));
			Ok(().into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_leave_candidates(*candidate_count))]
		/// Cancel open request to leave candidates
		/// - only callable by collator account
		/// - result upon successful call is the candidate is active in the candidate pool
		pub fn cancel_leave_candidates(
			origin: OriginFor<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_leaving(), Error::<T>::CandidateNotLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			// Reinforcement for the soft check in join_candiates
			ensure!(
				candidates.0.len() < T::MaxCollatorCandidates::get() as usize,
				Error::<T>::ExceedMaxCollatorCandidates
			);
			ensure!(
				candidates.0.len() as u32 <= candidate_count,
				Error::<T>::TooLowCandidateCountWeightHintCancelLeaveCandidates
			);
			ensure!(
				candidates.insert(Bond {
					owner: collator.clone(),
					amount: state.total_counted,
					liquidity_token: state.liquidity_token
				}),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CancelledCandidateExit(collator));
			Ok(().into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::go_offline())]
		/// Temporarily leave the set of collator candidates without unbonding
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_active(), Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateWentOffline(<Round<T>>::get().current, collator));
			Ok(().into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::go_online())]
		/// Rejoin the set of collator candidates if previously had called `go_offline`
		pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_active(), Error::<T>::AlreadyActive);
			ensure!(!state.is_leaving(), Error::<T>::CannotGoOnlineIfLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			// Reinforcement for the soft check in join_candiates
			ensure!(
				candidates.0.len() < T::MaxCollatorCandidates::get() as usize,
				Error::<T>::ExceedMaxCollatorCandidates
			);
			ensure!(
				candidates.insert(Bond {
					owner: collator.clone(),
					amount: state.total_counted,
					liquidity_token: state.liquidity_token
				}),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBackOnline(<Round<T>>::get().current, collator));
			Ok(().into())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_candidate_bond_more())]
		/// Request by collator candidate to increase self bond by `more`
		pub fn schedule_candidate_bond_more(
			origin: OriginFor<T>,
			more: BalanceOf<T>,
			use_balance_from: Option<BondKind>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let when = state.schedule_bond_more::<T>(more, use_balance_from)?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBondMoreRequested(collator, more, when));
			Ok(().into())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_candidate_bond_less())]
		/// Request by collator candidate to decrease self bond by `less`
		pub fn schedule_candidate_bond_less(
			origin: OriginFor<T>,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let when = state.schedule_bond_less::<T>(less)?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBondLessRequested(collator, less, when));
			Ok(().into())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(<T as Config>::WeightInfo::execute_candidate_bond_more())]
		/// Execute pending request to adjust the collator candidate self bond
		pub fn execute_candidate_bond_request(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			use_balance_from: Option<BondKind>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward this if caller != candidate
			let mut state = <CandidateState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let event = state.execute_pending_request::<T>(use_balance_from)?;
			<CandidateState<T>>::insert(&candidate, state);
			Self::deposit_event(event);
			Ok(().into())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_candidate_bond_more())]
		/// Cancel pending request to adjust the collator candidate self bond
		pub fn cancel_candidate_bond_request(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let event = state.cancel_pending_request::<T>()?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(event);
			Ok(().into())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(
			<T as Config>::WeightInfo::delegate(
				*candidate_delegation_count,
				*delegation_count,
			)
		)]
		/// If caller is not a delegator and not a collator, then join the set of delegators
		/// If caller is a delegator, then makes delegation to change their delegation state
		pub fn delegate(
			origin: OriginFor<T>,
			collator: T::AccountId,
			amount: BalanceOf<T>,
			use_balance_from: Option<BondKind>,
			candidate_delegation_count: u32,
			delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_aggregator(&acc), Error::<T>::AggregatorExists);
			let mut collator_state =
				<CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let delegator_state = if let Some(mut state) = <DelegatorState<T>>::get(&acc) {
				ensure!(state.is_active(), Error::<T>::CannotDelegateIfLeaving);
				// delegation after first
				ensure!(
					Self::valuate_bond(collator_state.liquidity_token, amount) >=
						T::MinDelegation::get(),
					Error::<T>::DelegationBelowMin
				);
				ensure!(
					delegation_count >= state.delegations.0.len() as u32,
					Error::<T>::TooLowDelegationCountToDelegate
				);
				ensure!(
					(state.delegations.0.len() as u32) < T::MaxDelegationsPerDelegator::get(),
					Error::<T>::ExceedMaxDelegationsPerDelegator
				);
				ensure!(
					state.add_delegation(Bond {
						owner: collator.clone(),
						amount,
						liquidity_token: collator_state.liquidity_token,
					}),
					Error::<T>::AlreadyDelegatedCandidate
				);
				state
			} else {
				ensure!(amount >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);
				ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
				Delegator::new(
					acc.clone(),
					collator.clone(),
					amount,
					collator_state.liquidity_token,
				)
			};
			// This check is hard
			// There is no other way to add to a collators delegation count
			ensure!(
				collator_state.delegators.0.len() <
					T::MaxTotalDelegatorsPerCandidate::get() as usize,
				Error::<T>::ExceedMaxTotalDelegatorsPerCandidate
			);
			ensure!(
				candidate_delegation_count >= collator_state.delegators.0.len() as u32,
				Error::<T>::TooLowCandidateDelegationCountToDelegate
			);
			let delegator_position = collator_state.add_delegation::<T>(acc.clone(), amount)?;
			<T as pallet::Config>::StakingReservesProvider::bond(
				collator_state.liquidity_token.into(),
				&acc,
				amount,
				use_balance_from,
			)?;
			if let DelegatorAdded::AddedToTop { new_total } = delegator_position {
				if collator_state.is_active() {
					// collator in candidate pool
					Self::update_active(
						collator.clone(),
						new_total,
						collator_state.liquidity_token,
					);
				}
			}
			let new_total_locked =
				<Total<T>>::get(collator_state.liquidity_token).saturating_add(amount);
			<Total<T>>::insert(collator_state.liquidity_token, new_total_locked);
			<CandidateState<T>>::insert(&collator, collator_state);
			<DelegatorState<T>>::insert(&acc, delegator_state);
			Self::deposit_event(Event::Delegation(acc, amount, collator, delegator_position));
			Ok(().into())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_leave_delegators())]
		/// Request to leave the set of delegators. If successful, the caller is scheduled
		/// to be allowed to exit. Success forbids future delegator actions until the request is
		/// invoked or cancelled.
		pub fn schedule_leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::DelegatorAlreadyLeaving);
			let (now, when) = state.schedule_leave::<T>();
			<DelegatorState<T>>::insert(&acc, state);
			Self::deposit_event(Event::DelegatorExitScheduled(now, acc, when));
			Ok(().into())
		}

		#[pallet::call_index(14)]
		#[pallet::weight(<T as Config>::WeightInfo::execute_leave_delegators(*delegation_count))]
		/// Execute the right to exit the set of delegators and revoke all ongoing delegations.
		pub fn execute_leave_delegators(
			origin: OriginFor<T>,
			delegator: T::AccountId,
			delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			state.can_execute_leave::<T>(delegation_count)?;
			let mut amount_unstaked: BalanceOf<T> = Zero::zero();
			for bond in state.delegations.0 {
				amount_unstaked = amount_unstaked.saturating_add(bond.amount);
				if let Err(error) =
					Self::delegator_leaves_collator(delegator.clone(), bond.owner.clone())
				{
					log::warn!(
						"STORAGE CORRUPTED \nDelegator leaving collator failed with error: {:?}",
						error
					);
				}
			}
			<DelegatorState<T>>::remove(&delegator);
			Self::deposit_event(Event::DelegatorLeft(delegator, amount_unstaked));
			Ok(().into())
		}

		#[pallet::call_index(15)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_leave_delegators())]
		/// Cancel a pending request to exit the set of delegators. Success clears the pending exit
		/// request (thereby resetting the delay upon another `leave_delegators` call).
		pub fn cancel_leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			// ensure delegator state exists
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			// ensure state is leaving
			ensure!(state.is_leaving(), Error::<T>::DelegatorDNE);
			// cancel exit request
			state.cancel_leave();
			<DelegatorState<T>>::insert(&delegator, state);
			Self::deposit_event(Event::DelegatorExitCancelled(delegator));
			Ok(().into())
		}

		#[pallet::call_index(16)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_revoke_delegation())]
		/// Request to revoke an existing delegation. If successful, the delegation is scheduled
		/// to be allowed to be revoked via the `execute_delegation_request` extrinsic.
		pub fn schedule_revoke_delegation(
			origin: OriginFor<T>,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			let (now, when) = state.schedule_revoke::<T>(collator.clone())?;
			<DelegatorState<T>>::insert(&delegator, state);
			Self::deposit_event(Event::DelegationRevocationScheduled(
				now, delegator, collator, when,
			));
			Ok(().into())
		}

		#[pallet::call_index(17)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_delegator_bond_more())]
		/// Request to bond more for delegators wrt a specific collator candidate.
		pub fn schedule_delegator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
			use_balance_from: Option<BondKind>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			let when = state.schedule_increase_delegation::<T>(
				candidate.clone(),
				more,
				use_balance_from,
			)?;
			<DelegatorState<T>>::insert(&delegator, state);
			Self::deposit_event(Event::DelegationIncreaseScheduled(
				delegator, candidate, more, when,
			));
			Ok(().into())
		}

		#[pallet::call_index(18)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_delegator_bond_less())]
		/// Request bond less for delegators wrt a specific collator candidate.
		pub fn schedule_delegator_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&caller).ok_or(Error::<T>::DelegatorDNE)?;
			let when = state.schedule_decrease_delegation::<T>(candidate.clone(), less)?;
			<DelegatorState<T>>::insert(&caller, state);
			Self::deposit_event(Event::DelegationDecreaseScheduled(caller, candidate, less, when));
			Ok(().into())
		}

		#[pallet::call_index(19)]
		#[pallet::weight(<T as Config>::WeightInfo::execute_delegator_bond_more())]
		/// Execute pending request to change an existing delegation
		pub fn execute_delegation_request(
			origin: OriginFor<T>,
			delegator: T::AccountId,
			candidate: T::AccountId,
			use_balance_from: Option<BondKind>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward caller if caller != delegator
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			state.execute_pending_request::<T>(candidate, use_balance_from)?;
			Ok(().into())
		}

		#[pallet::call_index(20)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_delegator_bond_more())]
		/// Cancel request to change an existing delegation.
		pub fn cancel_delegation_request(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			let request = state.cancel_pending_request::<T>(candidate)?;
			<DelegatorState<T>>::insert(&delegator, state);
			Self::deposit_event(Event::CancelledDelegationRequest(delegator, request));
			Ok(().into())
		}

		#[pallet::call_index(21)]
		#[pallet::weight(<T as Config>::WeightInfo::add_staking_liquidity_token(*current_liquidity_tokens))]
		/// Enables new staking token to be used for staking. Only tokens paired with MGX can be
		/// used. Caller can pass the id of token for which MGX paired pool already exists or
		/// liquidity token id itself. **Root only**
		pub fn add_staking_liquidity_token(
			origin: OriginFor<T>,
			paired_or_liquidity_token: PairedOrLiquidityToken<CurrencyIdOf<T>>,
			current_liquidity_tokens: u32,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let added_liquidity_token: CurrencyIdOf<T> = match paired_or_liquidity_token {
				PairedOrLiquidityToken::Paired(x) =>
					T::StakingLiquidityTokenValuator::get_liquidity_asset(
						x.into(),
						T::NativeTokenId::get().into(),
					)?,
				PairedOrLiquidityToken::Liquidity(x) => {
					T::StakingLiquidityTokenValuator::get_liquidity_token_mga_pool(x.into())?;
					x
				},
			};

			StakingLiquidityTokens::<T>::try_mutate(
				|staking_liquidity_tokens| -> DispatchResult {
					ensure!(
						current_liquidity_tokens as usize >= staking_liquidity_tokens.len(),
						Error::<T>::TooLowCurrentStakingLiquidityTokensCount
					);
					ensure!(
						staking_liquidity_tokens.insert(added_liquidity_token, None).is_none(),
						Error::<T>::StakingLiquidityTokenAlreadyListed
					);

					Ok(())
				},
			)?;
			Ok(().into())
		}

		#[pallet::call_index(22)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_staking_liquidity_token(*current_liquidity_tokens))]
		/// Removes previously added liquidity token
		pub fn remove_staking_liquidity_token(
			origin: OriginFor<T>,
			paired_or_liquidity_token: PairedOrLiquidityToken<CurrencyIdOf<T>>,
			current_liquidity_tokens: u32,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let removed_liquidity_token: CurrencyIdOf<T> = match paired_or_liquidity_token {
				PairedOrLiquidityToken::Paired(x) =>
					T::StakingLiquidityTokenValuator::get_liquidity_asset(
						x.into(),
						T::NativeTokenId::get().into(),
					)?,
				PairedOrLiquidityToken::Liquidity(x) => x,
			};

			StakingLiquidityTokens::<T>::try_mutate(
				|staking_liquidity_tokens| -> DispatchResult {
					ensure!(
						current_liquidity_tokens as usize >= staking_liquidity_tokens.len(),
						Error::<T>::TooLowCurrentStakingLiquidityTokensCount
					);
					ensure!(
						staking_liquidity_tokens.remove(&removed_liquidity_token).is_some(),
						Error::<T>::StakingLiquidityTokenNotListed
					);

					Ok(())
				},
			)?;
			Ok(().into())
		}

		#[pallet::call_index(23)]
		#[pallet::weight(T::DbWeight::get().reads_writes(20, 20))]
		#[transactional]
		/// Modifies aggregator metadata by extending or reducing list of approved candidates
		/// Account may only become aggregator only if its not collator or delegator at the moment
		pub fn aggregator_update_metadata(
			origin: OriginFor<T>,
			collator_candidates: Vec<T::AccountId>,
			action: MetadataUpdateAction,
		) -> DispatchResultWithPostInfo {
			let aggregator = ensure_signed(origin)?;

			ensure!(!Self::is_candidate(&aggregator), Error::<T>::CandidateExists);
			ensure!(!Self::is_delegator(&aggregator), Error::<T>::DelegatorExists);

			AggregatorMetadata::<T>::try_mutate_exists(
				aggregator.clone(),
				|maybe_aggregator_metadata| -> DispatchResult {
					let mut aggregator_metadata =
						maybe_aggregator_metadata.take().unwrap_or_default();

					match action {
						MetadataUpdateAction::ExtendApprovedCollators => {
							collator_candidates.iter().try_for_each(
								|collator| -> DispatchResult {
									Self::add_approved_candidate_for_collator_metadata(
										collator,
										&mut aggregator_metadata,
									)
								},
							)?;
						},
						MetadataUpdateAction::RemoveApprovedCollators => {
							collator_candidates.iter().try_for_each(
								|collator| -> DispatchResult {
									Self::remove_approved_candidates_from_collator_metadata(
										collator,
										&aggregator,
										&mut aggregator_metadata,
									)
								},
							)?;
						},
					}

					if !aggregator_metadata.approved_candidates.is_empty() {
						*maybe_aggregator_metadata = Some(aggregator_metadata);
					}

					Ok(())
				},
			)?;

			Self::deposit_event(Event::AggregatorMetadataUpdated(aggregator));
			Ok(().into())
		}

		#[pallet::call_index(24)]
		#[pallet::weight(T::DbWeight::get().reads_writes(20, 20))]
		/// Assigns/replaces the candidate that given collator wants to aggregate under
		#[transactional]
		pub fn update_candidate_aggregator(
			origin: OriginFor<T>,
			maybe_aggregator: Option<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let candidate = ensure_signed(origin)?;

			Self::do_update_candidate_aggregator(&candidate, maybe_aggregator.clone())?;

			Ok(().into())
		}

		/// This extrinsic should be used to distribute rewards for collator and assodiated
		/// delegators. As round rewards are processed in random order its impossible predict
		/// how many delegators (and assodiated transfer extrinsic calls) will be required so
		/// worst case scenario (delegators_count = MaxCollatorCandidates) is assumed.
		///
		/// params:
		/// - collator - account id
		/// - limit - number of rewards periods that should be processed within extrinsic. Note
		/// that limit assumes worst case scenario of (delegators_count = MaxCollatorCandidates)
		/// so as a result, `limit` or more session round rewards may be distributed
		#[pallet::call_index(25)]
		#[pallet::weight(number_of_sesisons.unwrap_or(T::DefaultPayoutLimit::get()) * <T as Config>::WeightInfo::payout_collator_rewards())]
		#[transactional]
		pub fn payout_collator_rewards(
			origin: OriginFor<T>,
			collator: T::AccountId,
			number_of_sesisons: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let _caller = ensure_signed(origin)?;
			Self::do_payout_collator_rewards(collator, number_of_sesisons)
		}

		// TODO: use more precise benchmark
		#[pallet::call_index(26)]
		#[pallet::weight(<T as Config>::WeightInfo::payout_delegator_reward())]
		#[transactional]
		/// Payout delegator rewards only for particular round. Collators should rather use
		/// [`Pallet::payout_collator_rewards`] but if collator is inresponsive one can claim
		/// particular delegator rewards manually.
		pub fn payout_delegator_reward(
			origin: OriginFor<T>,
			round: RoundIndex,
			collator: T::AccountId,
			delegator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let _caller = ensure_signed(origin)?;

			RoundCollatorRewardInfo::<T>::try_mutate(
				collator.clone(),
				round,
				|maybe_collator_payout_info| -> DispatchResult {
					let collator_payout_info = maybe_collator_payout_info
						.as_mut()
						.ok_or(Error::<T>::CollatorRoundRewardsDNE)?;
					let delegator_reward = collator_payout_info
						.delegator_rewards
						.remove(&delegator)
						.ok_or(Error::<T>::DelegatorRewardsDNE)?;
					Self::payout_reward(
						round,
						delegator,
						delegator_reward,
						RewardKind::Delegator(collator),
					)?;
					Ok(())
				},
			)?;

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_approved_candidate_for_collator_metadata(
			collator_candidate: &T::AccountId,
			aggregator_metadata: &mut AggregatorMetadataType<T::AccountId, CurrencyIdOf<T>>,
		) -> DispatchResult {
			ensure!(Self::is_candidate(&collator_candidate), Error::<T>::CandidateDNE);
			ensure!(
				aggregator_metadata.approved_candidates.insert(collator_candidate.clone()),
				Error::<T>::CandidateAlreadyApprovedByAggregator
			);
			Ok(())
		}

		fn remove_approved_candidates_from_collator_metadata(
			collator_candidate: &T::AccountId,
			aggregator: &T::AccountId,
			aggregator_metadata: &mut AggregatorMetadataType<T::AccountId, CurrencyIdOf<T>>,
		) -> DispatchResult {
			// Do not propagate the error if there's an error here
			// Then it means that the aggregator wasn't aggregating for this candidate
			// Or that the target is no longer a candidate, which can happen if the candidate has left
			// removing all his aggregation details except for approvals from various aggregators
			let _ = CandidateAggregator::<T>::try_mutate(
				|candidate_aggregator_map| -> DispatchResult {
					let collator_candidate_state = <CandidateState<T>>::get(&collator_candidate)
						.ok_or(Error::<T>::CandidateDNE)?;
					ensure!(
						Some(aggregator.clone()) ==
							candidate_aggregator_map.remove(collator_candidate),
						Error::<T>::CandidateNotAggregatingUnderAggregator
					);

					// If CandidateAggregator has the aggregator listed under this candidate then
					// the aggregator metadata will have this candidate listed under its liquidity token
					let res = aggregator_metadata
						.token_collator_map
						.remove(&collator_candidate_state.liquidity_token);
					if res != Some(collator_candidate.clone()) {
						log::error!(
							"Inconsistent aggregator metadata: candidate - {:?}, aggregator - {:?}",
							collator_candidate,
							aggregator
						);
					}

					Ok(())
				},
			);

			ensure!(
				aggregator_metadata.approved_candidates.remove(collator_candidate),
				Error::<T>::CandidateNotApprovedByAggregator
			);

			Ok(())
		}

		pub fn payout_reward(
			round: RoundIndex,
			to: T::AccountId,
			amt: BalanceOf<T>,
			kind: RewardKind<T::AccountId>,
		) -> DispatchResult {
			let _ = <T as pallet::Config>::Currency::transfer(
				T::NativeTokenId::get().into(),
				&<T as pallet::Config>::StakingIssuanceVault::get(),
				&to,
				amt,
				ExistenceRequirement::AllowDeath,
			)?;
			match kind {
				RewardKind::Collator =>
					Self::deposit_event(Event::Rewarded(round, to.clone(), amt)),
				RewardKind::Delegator(collator) => Self::deposit_event(Event::DelegatorDueReward(
					round,
					collator.clone(),
					to.clone(),
					amt,
				)),
			};
			Ok(())
		}
		pub fn is_delegator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}
		pub fn is_candidate(acc: &T::AccountId) -> bool {
			<CandidateState<T>>::get(acc).is_some()
		}
		pub fn is_aggregator(acc: &T::AccountId) -> bool {
			<AggregatorMetadata<T>>::get(acc).is_some()
		}
		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}

		fn remove_aggregator_for_collator(candidate: &T::AccountId) -> DispatchResult {
			CandidateAggregator::<T>::try_mutate(|candidate_aggregator_info| -> DispatchResult {
				let detached_aggregator = candidate_aggregator_info
					.remove(&candidate)
					.ok_or(Error::<T>::CandidateNotAggregating)?;

				AggregatorMetadata::<T>::try_mutate(
					detached_aggregator.clone(),
					|maybe_aggregator_metadata| -> DispatchResult {
						let aggregator_metadata =
							maybe_aggregator_metadata.as_mut().ok_or(Error::<T>::AggregatorDNE)?;
						let candidate_state =
							<CandidateState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
						let res = aggregator_metadata
							.token_collator_map
							.remove(&candidate_state.liquidity_token);
						if res != Some(candidate.clone()) {
							log::error!(
								"Inconsistent aggregator metadata: candidate - {:?}, aggregator - {:?}",
								candidate,
								detached_aggregator
								);
						}

						Ok(())
					},
				)?;

				Ok(())
			})?;
			Ok(())
		}

		fn corelate_collator_with_aggregator(
			candidate: &T::AccountId,
			new_aggregator: T::AccountId,
		) -> DispatchResult {
			AggregatorMetadata::<T>::try_mutate(
				new_aggregator,
				|maybe_aggregator_metadata| -> DispatchResult {
					let aggregator_metadata =
						maybe_aggregator_metadata.as_mut().ok_or(Error::<T>::AggregatorDNE)?;
					ensure!(
						aggregator_metadata.approved_candidates.contains(candidate),
						Error::<T>::CandidateNotApprovedByAggregator
					);
					let candidate_state =
						<CandidateState<T>>::get(candidate).ok_or(Error::<T>::CandidateDNE)?;
					ensure!(
						aggregator_metadata
							.token_collator_map
							.insert(candidate_state.liquidity_token, candidate.clone())
							.is_none(),
						Error::<T>::AggregatorLiquidityTokenTaken
					);

					Ok(())
				},
			)?;
			Ok(())
		}

		fn replace_aggregator_for_collator(
			candidate: &T::AccountId,
			new_aggregator: T::AccountId,
			prev_aggregator: T::AccountId,
		) -> DispatchResult {
			ensure!(
				prev_aggregator != new_aggregator,
				Error::<T>::TargettedAggregatorSameAsCurrent
			);

			AggregatorMetadata::<T>::try_mutate(
				prev_aggregator.clone(),
				|maybe_prev_aggregator_metadata| -> DispatchResult {
					let prev_aggregator_metadata =
						maybe_prev_aggregator_metadata.as_mut().ok_or(Error::<T>::AggregatorDNE)?;
					let candidate_state =
						<CandidateState<T>>::get(candidate).ok_or(Error::<T>::CandidateDNE)?;
					let res = prev_aggregator_metadata
						.token_collator_map
						.remove(&candidate_state.liquidity_token);
					if res != Some(candidate.clone()) {
						log::error!(
							"Inconsistent aggregator metadata: candidate - {:?}, aggregator - {:?}",
							candidate,
							prev_aggregator
						);
					}

					Self::corelate_collator_with_aggregator(candidate, new_aggregator)?;
					Ok(())
				},
			)?;

			Ok(())
		}

		fn assign_aggregator_for_collator(
			candidate: &T::AccountId,
			new_aggregator: T::AccountId,
		) -> DispatchResult {
			CandidateAggregator::<T>::try_mutate(|candidate_aggregator_info| -> DispatchResult {
				match candidate_aggregator_info.insert(candidate.clone(), new_aggregator.clone()) {
					Some(prev_aggregator) => {
						Self::replace_aggregator_for_collator(
							candidate,
							new_aggregator,
							prev_aggregator,
						)?;
					},
					None => {
						Self::corelate_collator_with_aggregator(candidate, new_aggregator)?;
					},
				}
				Ok(())
			})?;
			Ok(())
		}

		pub fn do_update_candidate_aggregator(
			candidate: &T::AccountId,
			maybe_aggregator: Option<T::AccountId>,
		) -> DispatchResult {
			ensure!(Self::is_candidate(candidate), Error::<T>::CandidateDNE);

			if let Some(ref new_aggregator) = maybe_aggregator {
				Self::assign_aggregator_for_collator(candidate, new_aggregator.clone())?;
			} else {
				Self::remove_aggregator_for_collator(candidate)?;
			}

			Self::deposit_event(Event::CandidateAggregatorUpdated(
				candidate.clone(),
				maybe_aggregator,
			));
			Ok(())
		}
		/// Caller must ensure candidate is active before calling
		fn update_active(
			candidate: T::AccountId,
			total: BalanceOf<T>,
			candidate_liquidity_token: CurrencyIdOf<T>,
		) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.remove(&Bond::from_owner(candidate.clone()));
			candidates.insert(Bond {
				owner: candidate,
				amount: total,
				liquidity_token: candidate_liquidity_token,
			});
			<CandidatePool<T>>::put(candidates);
		}
		fn delegator_leaves_collator(
			delegator: T::AccountId,
			collator: T::AccountId,
		) -> DispatchResult {
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let (total_changed, delegator_stake) = state.rm_delegator::<T>(delegator.clone())?;
			let debug_amount = <T as pallet::Config>::StakingReservesProvider::unbond(
				state.liquidity_token.into(),
				&delegator,
				delegator_stake,
			);
			if !debug_amount.is_zero() {
				log::warn!("Unbond in staking returned non-zero value {:?}", debug_amount);
			}
			if state.is_active() && total_changed {
				Self::update_active(collator.clone(), state.total_counted, state.liquidity_token);
			}
			let new_total_locked =
				<Total<T>>::get(state.liquidity_token).saturating_sub(delegator_stake);
			<Total<T>>::insert(state.liquidity_token, new_total_locked);
			let new_total = state.total_counted;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::DelegatorLeftCandidate(
				delegator,
				collator,
				delegator_stake,
				new_total,
			));
			Ok(())
		}

		fn process_collator_with_rewards(
			round_to_payout: u32,
			collator: T::AccountId,
			reward: BalanceOf<T>,
		) {
			let state = <AtStake<T>>::take(round_to_payout, &collator);
			let collator_commission_perbill = <CollatorCommission<T>>::get();
			let mut collator_payout_info =
				RoundCollatorRewardInfoType::<T::AccountId, BalanceOf<T>>::default();
			if state.delegations.is_empty() {
				// solo collator with no delegators
				collator_payout_info.collator_reward = reward;
				RoundCollatorRewardInfo::<T>::insert(
					collator,
					round_to_payout,
					collator_payout_info,
				);
			} else {
				let collator_commission = collator_commission_perbill.mul_floor(reward);
				let reward_less_commission = reward.saturating_sub(collator_commission);

				let collator_perbill = Perbill::from_rational(state.bond, state.total);
				let collator_reward_less_commission =
					collator_perbill.mul_floor(reward_less_commission);

				collator_payout_info.collator_reward =
					collator_reward_less_commission.saturating_add(collator_commission);

				match state
					.delegations
					.iter()
					.cloned()
					.try_fold(state.bond, |acc, x| acc.checked_add(&x.amount))
				{
					Some(total) if total <= state.total => {
						state.delegations.iter().for_each(|delegator_bond| {
							collator_payout_info.delegator_rewards.insert(
								delegator_bond.owner.clone(),
								multiply_by_rational_with_rounding(
									reward_less_commission.into(),
									delegator_bond.amount.into(),
									state.total.into(),
									Rounding::Down,
								)
								.map(SaturatedConversion::saturated_into)
								.unwrap_or(BalanceOf::<T>::zero()),
							);
						});
					},
					_ => {
						// unexpected overflow has occured and rewards will now distributed evenly amongst
						let delegator_count = state.delegations.len() as u32;
						let delegator_reward = reward
							.saturating_sub(collator_payout_info.collator_reward)
							.checked_div(&delegator_count.into())
							.unwrap_or(Zero::zero());
						state.delegations.iter().for_each(|delegator_bond| {
							collator_payout_info
								.delegator_rewards
								.insert(delegator_bond.owner.clone(), delegator_reward);
						});
					},
				}

				RoundCollatorRewardInfo::<T>::insert(
					collator,
					round_to_payout,
					collator_payout_info,
				);
			}
		}

		fn process_aggregator_with_rewards_and_dist(
			round_to_payout: u32,
			_aggregator: T::AccountId,
			author_rewards: BalanceOf<T>,
			distribution: &BTreeMap<T::AccountId, BalanceOf<T>>,
		) {
			match distribution
				.values()
				.cloned()
				.try_fold(BalanceOf::<T>::zero(), |acc, x| acc.checked_add(&x))
			{
				Some(aggregator_total_valuation) => {
					distribution.iter().for_each(|(collator, contribution)| {
						Self::process_collator_with_rewards(
							round_to_payout,
							collator.clone(),
							multiply_by_rational_with_rounding(
								author_rewards.into(),
								contribution.clone().into(),
								aggregator_total_valuation.into(),
								Rounding::Down,
							)
							.map(SaturatedConversion::saturated_into)
							.unwrap_or(BalanceOf::<T>::zero()),
						)
					});
				},
				None => {
					// unexpected overflow has occured and rewards will now distributed evenly amongst
					let collator_count = distribution.keys().cloned().count() as u32;
					let collator_reward = author_rewards
						.checked_div(&collator_count.into())
						.unwrap_or(BalanceOf::<T>::zero());
					distribution.keys().for_each(|collator| {
						Self::process_collator_with_rewards(
							round_to_payout,
							collator.clone(),
							collator_reward,
						)
					});
				},
			}
		}

		fn pay_stakers(now: RoundIndex) {
			// payout is now - duration rounds ago => now - duration > 0 else return early
			let duration = T::RewardPaymentDelay::get();
			if now < duration {
				return
			}
			let round_to_payout = now.saturating_sub(duration);
			let total = <Points<T>>::take(round_to_payout);
			if total.is_zero() {
				return
			}
			let total_issuance =
				T::Issuance::get_staking_issuance(round_to_payout).unwrap_or(Zero::zero());

			// unwrap_or_default here is to ensure backward compatibility during the upgrade
			let round_aggregator_info =
				RoundAggregatorInfo::<T>::take(round_to_payout).unwrap_or_default();

			for (author, pts) in <AwardedPts<T>>::drain_prefix(round_to_payout) {
				let author_issuance_perbill = Perbill::from_rational(pts, total);

				let author_rewards = author_issuance_perbill.mul_floor(total_issuance);

				match round_aggregator_info.get(&author) {
					Some(aggregator_distribution) =>
						Self::process_aggregator_with_rewards_and_dist(
							round_to_payout,
							author,
							author_rewards,
							aggregator_distribution,
						),
					None =>
						Self::process_collator_with_rewards(round_to_payout, author, author_rewards),
				}
			}
		}

		pub fn calculate_collators_valuations<'a, I>(
			valuated_bond_it: I,
		) -> BTreeMap<T::AccountId, BalanceOf<T>>
		where
			I: Iterator<
				Item = (&'a Bond<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>, BalanceOf<T>),
			>,
			I: Clone,
		{
			let aggregator_info = <CandidateAggregator<T>>::get();
			// collect aggregated bonds
			let mut valuated_bonds = valuated_bond_it
				.clone()
				.filter_map(|(bond, valuation)| {
					aggregator_info.get(&bond.owner).map(|aggregator| (bond, valuation, aggregator))
				})
				.fold(
					BTreeMap::<T::AccountId, BalanceOf<T>>::new(),
					|mut acc, (_bond, valuation, aggregator)| {
						acc.entry(aggregator.clone())
							.and_modify(|total| *total = total.saturating_add(valuation))
							.or_insert_with(|| valuation);
						acc
					},
				);

			// extend with non agregated bonds
			valuated_bonds.extend(valuated_bond_it.filter_map(|(bond, valuation)| {
				if let None = aggregator_info.get(&bond.owner) {
					Some((bond.owner.clone(), valuation))
				} else {
					None
				}
			}));

			valuated_bonds
		}

		pub fn calculate_aggregators_collator_info<'a, I>(
			valuated_bond_it: I,
		) -> BTreeMap<T::AccountId, BTreeMap<T::AccountId, BalanceOf<T>>>
		where
			I: Iterator<
				Item = (&'a Bond<T::AccountId, BalanceOf<T>, CurrencyIdOf<T>>, BalanceOf<T>),
			>,
		{
			let aggregator_info = <CandidateAggregator<T>>::get();

			valuated_bond_it
				.filter_map(|(bond, valuation)| {
					aggregator_info.get(&bond.owner).map(|aggregator| (bond, valuation, aggregator))
				})
				.fold(
					BTreeMap::<T::AccountId, BTreeMap<T::AccountId, BalanceOf<T>>>::new(),
					|mut acc, (bond, valuation, aggregator)| {
						acc.entry(aggregator.clone())
							.and_modify(|x| {
								x.insert(bond.owner.clone(), valuation);
							})
							.or_insert(BTreeMap::from([(bond.owner.clone(), valuation)]));
						acc
					},
				)
		}

		pub fn calculate_valuations_and_aggregation_info() -> (
			BTreeMap<T::AccountId, BalanceOf<T>>,
			BTreeMap<T::AccountId, BTreeMap<T::AccountId, BalanceOf<T>>>,
		) {
			let candidates = <CandidatePool<T>>::get().0;

			let liq_token_to_pool = <StakingLiquidityTokens<T>>::get();
			let valuated_bond_it = candidates.iter().filter_map(|bond| {
				if bond.liquidity_token == T::NativeTokenId::get() {
					Some((bond, bond.amount.checked_div(&2_u32.into()).unwrap_or_default()))
				} else {
					match liq_token_to_pool.get(&bond.liquidity_token) {
						Some(Some((reserve1, reserve2))) if !reserve1.is_zero() =>
							multiply_by_rational_with_rounding(
								bond.amount.into(),
								(*reserve1).into(),
								(*reserve2).into(),
								Rounding::Down,
							)
							.map(SaturatedConversion::saturated_into)
							.map(|val| (bond, val))
							.or(Some((bond, BalanceOf::<T>::max_value()))),
						_ => None,
					}
				}
			});

			(
				Self::calculate_collators_valuations(valuated_bond_it.clone()),
				Self::calculate_aggregators_collator_info(valuated_bond_it.clone()),
			)
		}
		//
		/// Compute the top `TotalSelected` candidates in the CandidatePool and return
		/// a vec of their AccountIds (in the order of selection)
		pub fn compute_top_candidates(
			now: RoundIndex,
		) -> (
			Vec<(T::AccountId, BalanceOf<T>)>,
			Vec<(T::AccountId, BalanceOf<T>)>,
			BTreeMap<T::AccountId, BTreeMap<T::AccountId, BalanceOf<T>>>,
		) {
			let (valuated_author_candidates_btreemap, aggregators_collator_info) =
				Self::calculate_valuations_and_aggregation_info();

			let mut valuated_author_candidates_vec: Vec<(T::AccountId, BalanceOf<T>)> =
				valuated_author_candidates_btreemap.into_iter().collect::<_>();

			let mut filtered_authors: Vec<_> = valuated_author_candidates_vec
				.into_iter()
				.filter(|x| x.1 >= T::MinCollatorStk::get())
				.collect::<_>();

			// order candidates by stake (least to greatest so requires `rev()`)
			filtered_authors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
			let top_n = <TotalSelected<T>>::get() as usize;
			// choose the top TotalSelected qualified candidates, ordered by stake
			let mut selected_authors: Vec<(T::AccountId, BalanceOf<T>)> =
				filtered_authors.into_iter().rev().take(top_n).collect::<_>();
			selected_authors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

			let mut all_selected_collators = Vec::<(T::AccountId, BalanceOf<T>)>::new();
			for selected_author in selected_authors.iter() {
				if let Some(aggregator_collator_info) =
					aggregators_collator_info.get(&selected_author.0)
				{
					all_selected_collators.extend_from_slice(
						&{
							aggregator_collator_info
								.into_iter()
								.map(|(a, b)| (a.clone(), b.clone()))
								.collect::<Vec<(T::AccountId, BalanceOf<T>)>>()
						}[..],
					);
				} else {
					all_selected_collators.push(selected_author.clone());
				}
			}

			(selected_authors, all_selected_collators, aggregators_collator_info)
		}

		pub fn staking_liquidity_tokens_snapshot() {
			let mut staking_liquidity_tokens = <StakingLiquidityTokens<T>>::get();

			for (token, valuation) in staking_liquidity_tokens.iter_mut() {
				*valuation = T::StakingLiquidityTokenValuator::get_pool_state((*token).into());
			}

			<StakingLiquidityTokens<T>>::put(staking_liquidity_tokens);
		}

		#[aquamarine::aquamarine]
		/// Best as in most cumulatively supported in terms of stake
		/// Returns [collator_count, delegation_count, total staked]
		/// ```mermaid
		/// flowchart
		///    A[Start] --> B{for all candidates}
		///    B -- Is aggregating under Aggregator? --> C[increase Aggreagator valuation]
		///    B -- Is solo collator? --> D[increase collator valuation]
		///    C --> E[collect final valuations of solo collators and aggregators]
		///    D --> E
		///    E -- list of solo collators and aggregators only--> F[pick top N valuated accounts]
		///    F --> G{for every block author}
		///    G -- author --> Z[persist into SelectedCandidates runtime storage]
		///    G -- author --> Y{Is solo collator or Aggregator}
		///    Y -- is solo collator --> I[emit CollatorChosen event]
		///    Y -- is aggregator --> H{for every associated collator}
		///    H --> I
		/// ```
		pub fn select_top_candidates(now: RoundIndex) -> (u32, u32, BalanceOf<T>) {
			let (mut collator_count, mut delegation_count, mut total_relevant_exposure) =
				(0u32, 0u32, BalanceOf::<T>::zero());
			Self::staking_liquidity_tokens_snapshot();
			// choose the top TotalSelected qualified candidates, ordered by stake
			let (selected_authors, all_selected_collators, aggregators_collator_info) =
				Self::compute_top_candidates(now);

			RoundAggregatorInfo::<T>::insert(now, aggregators_collator_info);

			// snapshot exposure for round for weighting reward distribution
			for collator in all_selected_collators.iter() {
				let state = <CandidateState<T>>::get(&collator.0)
					.expect("all members of CandidateQ must be candidates");
				collator_count = collator_count.saturating_add(1u32);
				delegation_count = delegation_count.saturating_add(state.delegators.0.len() as u32);
				let amount = collator.1;
				total_relevant_exposure = total_relevant_exposure.saturating_add(amount);
				let collator_snaphot: CollatorSnapshot<
					T::AccountId,
					BalanceOf<T>,
					CurrencyIdOf<T>,
				> = state.into();
				<AtStake<T>>::insert(now, collator.0.clone(), collator_snaphot);
				Self::deposit_event(Event::CollatorChosen(now, collator.0.clone(), amount));
			}

			// insert canonical collator set
			<SelectedCandidates<T>>::put(
				selected_authors.iter().cloned().map(|x| x.0).collect::<Vec<T::AccountId>>(),
			);
			(collator_count, delegation_count, total_relevant_exposure)
		}

		fn valuate_bond(liquidity_token: CurrencyIdOf<T>, bond: BalanceOf<T>) -> BalanceOf<T> {
			if liquidity_token == T::NativeTokenId::get() {
				bond.checked_div(&2_u32.into()).unwrap_or_default()
			} else {
				T::StakingLiquidityTokenValuator::valuate_liquidity_token(
					liquidity_token.into(),
					bond,
				)
			}
		}

		fn do_payout_collator_rewards(
			collator: T::AccountId,
			number_of_sesisons: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let mut rounds = Vec::<RoundIndex>::new();

			let limit = number_of_sesisons.unwrap_or(T::DefaultPayoutLimit::get());
			let mut payouts_left = limit * (T::MaxDelegationsPerDelegator::get() + 1);

			for (id, (round, info)) in
				RoundCollatorRewardInfo::<T>::iter_prefix(collator.clone()).enumerate()
			{
				if payouts_left < (info.delegator_rewards.len() as u32 + 1u32) {
					break
				}

				Self::payout_reward(
					round,
					collator.clone(),
					info.collator_reward,
					RewardKind::Collator,
				)?;
				payouts_left -= 1u32;

				let _ = info.delegator_rewards.iter().try_for_each(|(d, r)| {
					Self::payout_reward(
						round,
						d.clone(),
						r.clone(),
						RewardKind::Delegator(collator.clone()),
					)
				})?;
				RoundCollatorRewardInfo::<T>::remove(collator.clone(), round);
				rounds.push(round);

				payouts_left = payouts_left
					.checked_sub(info.delegator_rewards.len() as u32)
					.unwrap_or_default();

				if (id as u32).checked_add(1u32).unwrap_or(u32::MAX) > limit {
					// We can optimize number of rounds that can be processed as extrinsic weight
					// was benchmarked assuming that collator have delegators count == T::MaxDelegatorsPerCandidate
					// so if there are less or no delegators, we can use remaining weight for
					// processing following block, the only extra const is single iteration that
					// consumes 1 storage read. We can compensate that by sacrificing single transfer tx
					//
					// esitmated weight or payout_collator_rewards extrinsic with limit parm set to 1 is
					// 1 storage read + (MaxDelegatorsPerCollator + 1) * transfer weight
					//
					// so if collator does not have any delegators only 1 transfer was actually
					// executed leaving MaxDelegatorsPerCollator spare. We can use that remaining
					// weight to payout rewards for following rounds. Each extra round requires:
					// - 1 storage read (iteration)
					// - N <= MaxDelegatorsPerCandidate transfers (depending on collators count)
					//
					// we can compansate storage read with 1 transfer and try to process following
					// round if there are enought transfers left
					payouts_left = payouts_left.checked_sub(1).unwrap_or_default();
				}
			}

			ensure!(!rounds.is_empty(), Error::<T>::CollatorRoundRewardsDNE);

			if let Some(_) = RoundCollatorRewardInfo::<T>::iter_prefix(collator.clone()).next() {
				Self::deposit_event(Event::CollatorRewardsDistributed(
					collator,
					PayoutRounds::Partial(rounds),
				));
			} else {
				Self::deposit_event(Event::CollatorRewardsDistributed(collator, PayoutRounds::All));
			}

			// possibly use PostDispatchInfo.actual_weight to return refund caller if delegators
			// count was lower than assumed upper bound
			Ok(().into())
		}
	}

	/// Add reward points to block authors:
	/// * 20 points to the block producer for producing a block in the chain
	impl<T: Config> pallet_authorship::EventHandler<T::AccountId, BlockNumberFor<T>> for Pallet<T> {
		fn note_author(author: T::AccountId) {
			let now = <Round<T>>::get().current;
			let score_plus_20 = <AwardedPts<T>>::get(now, &author).saturating_add(20);
			<AwardedPts<T>>::insert(now, author, score_plus_20);
			<Points<T>>::mutate(now, |x| *x = x.saturating_add(20));
		}
	}

	impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
		fn new_session(_: SessionIndex) -> Option<Vec<T::AccountId>> {
			let selected_canidates = Self::selected_candidates();
			if !selected_canidates.is_empty() {
				Some(selected_canidates)
			} else {
				let fallback_canidates = T::FallbackProvider::get_members();
				if !fallback_canidates.is_empty() {
					Some(fallback_canidates)
				} else {
					None
				}
			}
		}
		fn start_session(session_index: SessionIndex) {
			if !session_index.is_zero() {
				let n = <frame_system::Pallet<T>>::block_number().saturating_add(One::one());
				let mut round = <Round<T>>::get();
				// println!("ROUND FINISHED {}", round.current.saturated_into::<u64>());
				// mutate round
				round.update(n);
				// pay all stakers for T::RewardPaymentDelay rounds ago
				Self::pay_stakers(round.current);
				// select top collator candidates for next round
				let (collator_count, _delegation_count, total_relevant_exposure) =
					Self::select_top_candidates(round.current.saturating_add(One::one()));
				// Calculate the issuance for next round
				// No issuance must happen after this point
				T::Issuance::compute_issuance(round.current);
				// start next round
				<Round<T>>::put(round);
				// Emit new round event
				Self::deposit_event(Event::NewRound(
					round.first,
					round.current,
					collator_count,
					total_relevant_exposure,
				));
			}
		}
		fn end_session(_: SessionIndex) {
			// ignore
		}
	}

	impl<T: Config> pallet_session::ShouldEndSession<BlockNumberFor<T>> for Pallet<T> {
		fn should_end_session(now: BlockNumberFor<T>) -> bool {
			let round = <Round<T>>::get();
			round.should_update(now)
		}
	}

	impl<T: Config> EstimateNextSessionRotation<BlockNumberFor<T>> for Pallet<T> {
		fn average_session_length() -> BlockNumberFor<T> {
			<Round<T>>::get().length.into()
		}

		fn estimate_current_session_progress(now: BlockNumberFor<T>) -> (Option<Permill>, Weight) {
			let round = <Round<T>>::get();
			let passed_blocks = now.saturating_sub(round.first).saturating_add(One::one());

			(
				Some(Permill::from_rational(passed_blocks, round.length.into())),
				// One read for the round info, blocknumber is read free
				T::DbWeight::get().reads(1),
			)
		}

		fn estimate_next_session_rotation(
			_now: BlockNumberFor<T>,
		) -> (Option<BlockNumberFor<T>>, Weight) {
			let round = <Round<T>>::get();

			(
				Some(round.first.saturating_add(round.length.saturating_sub(One::one()).into())),
				// One read for the round info, blocknumber is read free
				T::DbWeight::get().reads(1),
			)
		}
	}
}
