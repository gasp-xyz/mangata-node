// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

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

//! # Crowdloan Rewards Pallet
//!
//! This pallet issues rewards to citizens who participated in a crowdloan on the backing relay
//! chain (eg Kusama) in order to help this parrachain acquire a parachain slot.
//!
//! ## Monetary Policy
//!
//! This is simple and mock for now. We can do whatever we want.
//! This pallet stores a constant  "reward ratio" which is the number of reward tokens to pay per
//! contributed token. In simple cases this can be 1, but needs to be customizeable to allow for
//! vastly differing absolute token supplies between relay and para.
//! Vesting is also linear. No tokens are vested at genesis and they unlock linearly until a
//! predecided block number. Vesting computations happen on demand when payouts are requested. So
//! no block weight is ever wasted on this, and there is no "base-line" cost of updating vestings.
//! Like I said, we can anything we want there. Even a non-linear reward curve to disincentivize
//! whales.
//!
//! ## Payout Mechanism
//!
//! The current payout mechanism requires contributors to claim their payouts. Because they are
//! paying the transaction fees for this themselves, they can do it as often as every block, or
//! wait and claim the entire thing once it is fully vested. We could consider auto payouts if we
//! want.
//!
//! ## Sourcing Contribution Information
//!
//! The pallet can learn about the crowdloan contributions in several ways.
//!
//! * **Through the initialize_reward_vec extrinsic*
//!
//! The simplest way is to call the initialize_reward_vec through a democracy proposal/sudo call.
//! This makes sense in a scenario where the crowdloan took place entirely offchain.
//! This extrinsic initializes the associated and unassociated stoerage with the provided data
//!
//! * **ReadingRelayState**
//!
//! The most elegant, but most complex solution would be for the para to read the contributions
//! directly from the relay state. Blocked by https://github.com/paritytech/cumulus/issues/320 so
//! I won't pursue it further right now. I can't decide whether that would really add security /
//! trustlessness, or is just a sexy blockchain thing to do. Contributors can always audit the
//! democracy proposal and make sure their contribution is in it, so in that sense reading relay state
//! isn't necessary. But if a single contribution is left out, the rest of the contributors might
//! not care enough to delay network launch. The little guy might get censored.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet,
	pallet_prelude::*,
	traits::{MultiTokenCurrency, MultiTokenVestingLocks, OnRuntimeUpgrade},
};
use frame_system::pallet_prelude::*;
use orml_tokens::MultiTokenCurrencyExtended;
use sp_runtime::{
	account::EthereumSignature,
	traits::{AtLeast32BitUnsigned, BlockNumberProvider, CheckedSub, Saturating, Verify},
	AccountId20, Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

pub use pallet::*;
#[cfg(feature = "try-runtime")]
pub use sp_runtime::TryRuntimeError;
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: "crowdloan",
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

pub type BalanceOf<T> =
	<<T as Config>::Tokens as MultiTokenCurrency<<T as frame_system::Config>::AccountId>>::Balance;

pub type TokenIdOf<T> = <<T as Config>::Tokens as MultiTokenCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

#[pallet]
pub mod pallet {
	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::storage_version(STORAGE_VERSION)]
	// The crowdloan rewards pallet
	pub struct Pallet<T>(PhantomData<T>);

	// The wrapper around which the reward changing message needs to be wrapped
	pub const WRAPPED_BYTES_PREFIX: &[u8] = b"<Bytes>";
	pub const WRAPPED_BYTES_POSTFIX: &[u8] = b"</Bytes>";

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Checker for the reward vec, is it initalized already?
		type Initialized: Get<bool>;
		/// Percentage to be payed at initialization
		#[pallet::constant]
		type InitializationPayment: Get<Perbill>;
		// Max number of contributors that can be inserted at once in initialize_reward_vec
		#[pallet::constant]
		type MaxInitContributors: Get<u32>;
		/// The minimum contribution to which rewards will be paid.
		type MinimumReward: Get<BalanceOf<Self>>;
		/// A fraction representing the percentage of proofs
		/// that need to be presented to change a reward address through the relay keys
		#[pallet::constant]
		type RewardAddressRelayVoteThreshold: Get<Perbill>;
		/// MGA token Id
		#[pallet::constant]
		type NativeTokenId: Get<TokenIdOf<Self>>;
		/// The currency in which the rewards will be paid (probably the parachain native currency)
		type Tokens: MultiTokenCurrency<Self::AccountId>
			+ MultiTokenCurrencyExtended<Self::AccountId>;
		/// The AccountId type contributors used on the relay chain.
		type RelayChainAccountId: Parameter
			//TODO these AccountId20 bounds feel a little extraneous. I wonder if we can remove them.
			// Since our "relaychain" is now ethereum
			+ Into<AccountId20>
			+ From<AccountId20>
			+ Ord;

		// The origin that is allowed to change the reward address with relay signatures
		type RewardAddressChangeOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Network Identifier to be appended into the signatures for reward address change/association
		/// Prevents replay attacks from one network to the other
		#[pallet::constant]
		type SignatureNetworkIdentifier: Get<&'static [u8]>;

		// The origin that is allowed to change the reward address with relay signatures
		type RewardAddressAssociateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The type that will be used to track vesting progress
		type VestingBlockNumber: AtLeast32BitUnsigned + Parameter + Default + Into<BalanceOf<Self>>;

		/// The notion of time that will be used for vesting. Probably
		/// either the relay chain or sovereignchain block number.
		type VestingBlockProvider: BlockNumberProvider<BlockNumber = Self::VestingBlockNumber>;

		/// Vesting provider for paying out vested rewards
		type VestingProvider: MultiTokenVestingLocks<
			Self::AccountId,
			Moment = Self::VestingBlockNumber,
			Currency = Self::Tokens,
		>;

		type WeightInfo: WeightInfo;
	}

	/// Stores info about the rewards owed as well as how much has been vested so far.
	/// For a primer on this kind of design, see the recipe on compounding interest
	/// https://substrate.dev/recipes/fixed-point.html#continuously-compounding
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct RewardInfo<T: Config> {
		pub total_reward: BalanceOf<T>,
		pub claimed_reward: BalanceOf<T>,
		pub contributed_relay_addresses: Vec<T::RelayChainAccountId>,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Associate a native rewards_destination identity with a crowdloan contribution.
		///
		/// The caller needs to provide the unassociated relay account and a proof to succeed
		/// with the association
		/// The proof is nothing but a signature over the reward_address using the relay keys
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::associate_native_identity())]
		pub fn associate_native_identity(
			origin: OriginFor<T>,
			reward_account: T::AccountId,
			relay_account: T::RelayChainAccountId,
			proof: EthereumSignature,
		) -> DispatchResultWithPostInfo {
			// Check that the origin is the one able to asociate the reward addrss
			T::RewardAddressChangeOrigin::ensure_origin(origin)?;

			// Check the proof:
			// 1. Is signed by an actual unassociated contributor
			// 2. Signs a valid native identity
			// Check the proof. The Proof consists of a Signature of the rewarded account with the
			// claimer key

			// The less costly checks will go first

			// The relay account should be unassociated
			let reward_info =
				UnassociatedContributions::<T>::get(CrowdloanId::<T>::get(), &relay_account)
					.ok_or(Error::<T>::NoAssociatedClaim)?;

			// We ensure the relay chain id wast not yet associated to avoid multi-claiming
			// We dont need this right now, as it will always be true if the above check is true
			ensure!(
				ClaimedRelayChainIds::<T>::get(CrowdloanId::<T>::get(), &relay_account).is_none(),
				Error::<T>::AlreadyAssociated
			);

			// For now I prefer that we dont support providing an existing account here
			ensure!(
				AccountsPayable::<T>::get(CrowdloanId::<T>::get(), &reward_account).is_none(),
				Error::<T>::AlreadyAssociated
			);

			// b"<Bytes>" "SignatureNetworkIdentifier" + "new_account" + b"</Bytes>"
			let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
			payload.append(&mut T::SignatureNetworkIdentifier::get().to_vec());
			payload.append(&mut reward_account.encode());
			payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

			// Check the signature
			Self::verify_signatures(
				vec![(relay_account.clone(), proof)],
				reward_info.clone(),
				payload,
			)?;

			// Insert on payable
			AccountsPayable::<T>::insert(CrowdloanId::<T>::get(), &reward_account, &reward_info);

			// Remove from unassociated
			<UnassociatedContributions<T>>::remove(CrowdloanId::<T>::get(), &relay_account);

			// Insert in mapping
			ClaimedRelayChainIds::<T>::insert(CrowdloanId::<T>::get(), &relay_account, ());

			// Emit Event
			Self::deposit_event(Event::NativeIdentityAssociated(
				relay_account,
				reward_account,
				reward_info.total_reward,
			));

			Ok(Default::default())
		}

		/// Change reward account by submitting proofs from relay accounts
		///
		/// The number of valid proofs needs to be bigger than 'RewardAddressRelayVoteThreshold'
		/// The account to be changed needs to be submitted as 'previous_account'

		/// Origin must be RewardAddressChangeOrigin
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::change_association_with_relay_keys(proofs.len() as u32))]
		pub fn change_association_with_relay_keys(
			origin: OriginFor<T>,
			reward_account: T::AccountId,
			previous_account: T::AccountId,
			proofs: Vec<(T::RelayChainAccountId, EthereumSignature)>,
		) -> DispatchResultWithPostInfo {
			// Check that the origin is the one able to change the reward addrss
			T::RewardAddressChangeOrigin::ensure_origin(origin)?;

			// For now I prefer that we dont support providing an existing account here
			ensure!(
				AccountsPayable::<T>::get(CrowdloanId::<T>::get(), &reward_account).is_none(),
				Error::<T>::AlreadyAssociated
			);

			// To avoid replay attacks, we make sure the payload contains the previous address too
			// I am assuming no rational user will go back to a previously changed reward address
			// b"<Bytes>" + "SignatureNetworkIdentifier" + new_account" + "previous_account" + b"</Bytes>"
			let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
			payload.append(&mut T::SignatureNetworkIdentifier::get().to_vec());
			payload.append(&mut reward_account.encode());
			payload.append(&mut previous_account.encode());
			payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

			// Get the reward info for the account to be changed
			let reward_info = AccountsPayable::<T>::get(CrowdloanId::<T>::get(), &previous_account)
				.ok_or(Error::<T>::NoAssociatedClaim)?;

			Self::verify_signatures(proofs, reward_info.clone(), payload)?;

			// Remove fromon payable
			AccountsPayable::<T>::remove(CrowdloanId::<T>::get(), &previous_account);

			// Insert on payable
			AccountsPayable::<T>::insert(CrowdloanId::<T>::get(), &reward_account, &reward_info);

			// Emit Event
			Self::deposit_event(Event::RewardAddressUpdated(previous_account, reward_account));

			Ok(Default::default())
		}

		/// Collect rewards from particular crowdloan.
		/// If crowdloan_id is not set current [`CrowdloanId`] id will be used.
		/// Caller is instantly rewarded with [`InitializationPayment`] % of available rewards,
		/// remaining funds are locked according to schedule(using `pallet_mangata_vesting` configured
		/// by [`Pallet::<T>::complete_initialization`] call.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::claim())]
		pub fn claim(
			origin: OriginFor<T>,
			crowdloan_id: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let payee = ensure_signed(origin)?;

			let current_crowdloan_id = CrowdloanId::<T>::get();
			let crowdloan_id = crowdloan_id.unwrap_or(current_crowdloan_id);

			ensure!(
				<Initialized<T>>::get() || crowdloan_id < current_crowdloan_id,
				Error::<T>::RewardVecNotFullyInitializedYet
			);
			// Calculate the veted amount on demand.
			let mut info = AccountsPayable::<T>::get(crowdloan_id, &payee)
				.ok_or(Error::<T>::NoAssociatedClaim)?;
			ensure!(info.claimed_reward < info.total_reward, Error::<T>::RewardsAlreadyClaimed);

			// Get the current block used for vesting purposes
			let _now = T::VestingBlockProvider::current_block_number();

			// How much should the contributor have already claimed by this block?
			// By multiplying first we allow the conversion to integer done with the biggest number
			let amount: BalanceOf<T> = info
				.total_reward
				.checked_sub(&info.claimed_reward)
				.ok_or(Error::<T>::MathOverflow)?;
			info.claimed_reward += amount;

			T::Tokens::mint(T::NativeTokenId::get(), &payee, amount)?;

			let period = CrowdloanPeriod::<T>::get(crowdloan_id).ok_or(Error::<T>::PeriodNotSet)?;

			T::VestingProvider::lock_tokens(
				&payee,
				T::NativeTokenId::get(),
				amount - T::InitializationPayment::get() * amount,
				Some(period.0),
				period.1.into(),
			)?;

			Self::deposit_event(Event::RewardsPaid(payee.clone(), amount));
			AccountsPayable::<T>::insert(crowdloan_id, &payee, &info);

			Ok(Default::default())
		}

		/// Update reward address, proving that the caller owns the current native key
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_reward_address())]
		pub fn update_reward_address(
			origin: OriginFor<T>,
			new_reward_account: T::AccountId,
			crowdloan_id: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let signer = ensure_signed(origin)?;

			let crowdloan_id = crowdloan_id.unwrap_or(CrowdloanId::<T>::get());

			// Calculate the veted amount on demand.
			let info = AccountsPayable::<T>::get(crowdloan_id, &signer)
				.ok_or(Error::<T>::NoAssociatedClaim)?;

			// For now I prefer that we dont support providing an existing account here
			ensure!(
				AccountsPayable::<T>::get(crowdloan_id, &new_reward_account).is_none(),
				Error::<T>::AlreadyAssociated
			);

			// Remove previous rewarded account
			AccountsPayable::<T>::remove(crowdloan_id, &signer);

			// Update new rewarded acount
			AccountsPayable::<T>::insert(crowdloan_id, &new_reward_account, &info);

			// Emit event
			Self::deposit_event(Event::RewardAddressUpdated(signer, new_reward_account));

			Ok(Default::default())
		}

		/// This extrinsic completes the initialization if some checks are fullfiled. These checks are:
		///  -The reward contribution money matches the crowdloan pot
		///  -The end vesting block is higher than the init vesting block
		///  -The initialization has not complete yet
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::complete_initialization())]
		pub fn complete_initialization(
			origin: OriginFor<T>,
			lease_start_block: T::VestingBlockNumber,
			lease_ending_block: T::VestingBlockNumber,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let initialized = <Initialized<T>>::get();

			// This ensures there was no prior initialization
			ensure!(!initialized, Error::<T>::RewardVecAlreadyInitialized);

			// This ensures the end vesting block (when all funds are fully vested)
			// is bigger than the init vesting block
			ensure!(lease_ending_block > lease_start_block, Error::<T>::VestingPeriodNonValid);

			let total_initialized_rewards =
				InitializedRewardAmount::<T>::get(CrowdloanId::<T>::get());

			let reward_difference = Pallet::<T>::get_crowdloan_allocation(CrowdloanId::<T>::get())
				.saturating_sub(total_initialized_rewards);

			// Ensure the difference is not bigger than the total number of contributors
			ensure!(
				reward_difference < TotalContributors::<T>::get(CrowdloanId::<T>::get()).into(),
				Error::<T>::RewardsDoNotMatchFund
			);

			<CrowdloanPeriod<T>>::insert(
				CrowdloanId::<T>::get(),
				(lease_start_block, lease_ending_block),
			);
			<Initialized<T>>::put(true);

			Ok(Default::default())
		}

		/// Initialize the reward distribution storage. It shortcuts whenever an error is found

		/// Sets crowdloan allocation for:
		/// - current round of crowdloan - if it has not been completed (`[Pallet::<T>::complete_initialization]`)
		/// - following round of crowdloan rewards payment if previous one has been already
		/// completed
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::set_crowdloan_allocation())]
		pub fn set_crowdloan_allocation(
			origin: OriginFor<T>,
			crowdloan_allocation_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			if <Initialized<T>>::get() {
				let zero: BalanceOf<T> = 0_u32.into();
				<CrowdloanId<T>>::mutate(|val| *val += 1);
				<Initialized<T>>::put(false);
				<InitializedRewardAmount<T>>::insert(CrowdloanId::<T>::get(), zero);
			}

			ensure!(
				crowdloan_allocation_amount >=
					InitializedRewardAmount::<T>::get(CrowdloanId::<T>::get()),
				Error::<T>::AllocationDoesNotMatch
			);

			CrowdloanAllocation::<T>::insert(CrowdloanId::<T>::get(), crowdloan_allocation_amount);
			Ok(Default::default())
		}

		/// Initialize the reward distribution storage. It shortcuts whenever an error is found

		/// This does not enforce any checks other than making sure we dont go over funds
		/// complete_initialization should perform any additional
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::initialize_reward_vec(rewards.len() as u32))]
		pub fn initialize_reward_vec(
			origin: OriginFor<T>,
			rewards: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let initialized = <Initialized<T>>::get();
			ensure!(!initialized, Error::<T>::RewardVecAlreadyInitialized);

			// Ensure we are below the max number of contributors
			ensure!(
				rewards.len() as u32 <= T::MaxInitContributors::get(),
				Error::<T>::TooManyContributors
			);

			// What is the amount initialized so far?
			let mut total_initialized_rewards =
				InitializedRewardAmount::<T>::get(CrowdloanId::<T>::get());

			// Total number of contributors
			let mut total_contributors = TotalContributors::<T>::get(CrowdloanId::<T>::get());

			let incoming_rewards: BalanceOf<T> = rewards
				.iter()
				.fold(0_u32.into(), |acc: BalanceOf<T>, (_, _, reward)| acc + *reward);

			// Ensure we dont go over funds
			ensure!(
				total_initialized_rewards + incoming_rewards <=
					Pallet::<T>::get_crowdloan_allocation(CrowdloanId::<T>::get()),
				Error::<T>::BatchBeyondFundPot
			);

			for (relay_account, native_account, reward) in &rewards {
				if ClaimedRelayChainIds::<T>::get(CrowdloanId::<T>::get(), relay_account).is_some() ||
					UnassociatedContributions::<T>::get(CrowdloanId::<T>::get(), relay_account)
						.is_some()
				{
					// Dont fail as this is supposed to be called with batch calls and we
					// dont want to stall the rest of the contributions
					Self::deposit_event(Event::InitializedAlreadyInitializedAccount(
						relay_account.clone(),
						native_account.clone(),
						*reward,
					));
					continue
				}

				total_initialized_rewards += *reward;
				total_contributors += 1;

				if *reward < T::MinimumReward::get() {
					// Don't fail as this is supposed to be called with batch calls and we
					// dont want to stall the rest of the contributions
					Self::deposit_event(Event::InitializedAccountWithNotEnoughContribution(
						relay_account.clone(),
						native_account.clone(),
						*reward,
					));
					continue
				}

				if let Some(native_account) = native_account {
					AccountsPayable::<T>::mutate(CrowdloanId::<T>::get(), native_account, |info| {
						let result = info.as_mut().map_or(
							RewardInfo {
								total_reward: *reward,
								claimed_reward: 0_u32.into(),
								contributed_relay_addresses: vec![relay_account.clone()],
							},
							|i| {
								i.total_reward += *reward;
								i.contributed_relay_addresses.push(relay_account.clone());
								i.clone()
							},
						);
						*info = Some(result);
					});
					ClaimedRelayChainIds::<T>::insert(CrowdloanId::<T>::get(), relay_account, ());
				} else {
					UnassociatedContributions::<T>::insert(
						CrowdloanId::<T>::get(),
						relay_account,
						RewardInfo {
							total_reward: *reward,
							claimed_reward: 0_u32.into(),
							contributed_relay_addresses: vec![relay_account.clone()],
						},
					);
				}
			}
			InitializedRewardAmount::<T>::insert(
				CrowdloanId::<T>::get(),
				total_initialized_rewards,
			);
			TotalContributors::<T>::insert(CrowdloanId::<T>::get(), total_contributors);

			Ok(Default::default())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn total_contributors() -> u32 {
			TotalContributors::<T>::get(CrowdloanId::<T>::get())
		}
		/// Verify a set of signatures made with relay chain accounts
		/// We are verifying all the signatures, and then counting
		/// We could do something more efficient like count as we verify
		/// In any of the cases the weight will need to account for all the signatures,
		/// as we dont know beforehand whether they will be valid
		fn verify_signatures(
			proofs: Vec<(T::RelayChainAccountId, EthereumSignature)>,
			reward_info: RewardInfo<T>,
			payload: Vec<u8>,
		) -> DispatchResult {
			// The proofs should
			// 1. be signed by contributors to this address, otherwise they are not counted
			// 2. Signs a valid native identity
			// 3. The sum of the valid proofs needs to be bigger than InsufficientNumberOfValidProofs

			// I use a map here for faster lookups
			let mut voted: BTreeMap<T::RelayChainAccountId, ()> = BTreeMap::new();
			for (relay_account, signature) in proofs {
				// We just count votes that we have not seen
				if voted.get(&relay_account).is_none() {
					// Maybe I should not error here?
					ensure!(
						reward_info.contributed_relay_addresses.contains(&relay_account),
						Error::<T>::NonContributedAddressProvided
					);

					// I am erroring here as I think it is good to know the reason in the single-case
					// signature
					ensure!(
						signature.verify(payload.as_slice(), &relay_account.clone().into()),
						Error::<T>::InvalidClaimSignature
					);
					voted.insert(relay_account, ());
				}
			}

			// Ensure the votes are sufficient
			ensure!(
				Perbill::from_rational(
					voted.len() as u32,
					reward_info.contributed_relay_addresses.len() as u32
				) >= T::RewardAddressRelayVoteThreshold::get(),
				Error::<T>::InsufficientNumberOfValidProofs
			);
			Ok(())
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// User trying to associate a native identity with a relay chain identity for posterior
		/// reward claiming provided an already associated relay chain identity
		AlreadyAssociated,
		/// Trying to introduce a batch that goes beyond the limits of the funds
		BatchBeyondFundPot,
		/// First claim already done
		FirstClaimAlreadyDone,
		/// The contribution is not high enough to be eligible for rewards
		RewardNotHighEnough,
		/// User trying to associate a native identity with a relay chain identity for posterior
		/// reward claiming provided a wrong signature
		InvalidClaimSignature,
		/// User trying to claim the first free reward provided the wrong signature
		InvalidFreeClaimSignature,
		/// User trying to claim an award did not have an claim associated with it. This may mean
		/// they did not contribute to the crowdloan, or they have not yet associated a native id
		/// with their contribution
		NoAssociatedClaim,
		/// User trying to claim rewards has already claimed all rewards associated with its
		/// identity and contribution
		RewardsAlreadyClaimed,
		/// Reward vec has already been initialized
		RewardVecAlreadyInitialized,
		/// Reward vec has not yet been fully initialized
		RewardVecNotFullyInitializedYet,
		/// Rewards should match funds of the pallet
		RewardsDoNotMatchFund,
		/// Initialize_reward_vec received too many contributors
		TooManyContributors,
		/// Provided vesting period is not valid
		VestingPeriodNonValid,
		/// User provided a signature from a non-contributor relay account
		NonContributedAddressProvided,
		/// User submitted an unsifficient number of proofs to change the reward address
		InsufficientNumberOfValidProofs,
		/// The mint operation during claim has resulted in err.
		/// This is expected when claiming less than existential desposit on a non-existent account
		/// Please consider waiting until the EndVestingBlock to attempt this
		ClaimingLessThanED,
		/// Math overflow
		MathOverflow,
		/// Period not set
		PeriodNotSet,
		/// Trying to introduce a batch that goes beyond the limits of the funds
		AllocationDoesNotMatch,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_crowdloan_allocation)]
	pub type CrowdloanAllocation<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, BalanceOf<T>, ValueQuery>;

	/// Id of current crowdloan rewards distribution, automatically incremented by
	/// [`Pallet::<T>::complete_initialization`]
	#[pallet::storage]
	#[pallet::getter(fn get_crowdloan_id)]
	pub type CrowdloanId<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn accounts_payable)]
	pub type AccountsPayable<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, RewardInfo<T>>;

	#[pallet::storage]
	#[pallet::getter(fn crowdloan_period)]
	pub type CrowdloanPeriod<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, (T::VestingBlockNumber, T::VestingBlockNumber)>;

	#[pallet::storage]
	#[pallet::getter(fn claimed_relay_chain_ids)]
	pub type ClaimedRelayChainIds<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, T::RelayChainAccountId, ()>;
	#[pallet::storage]
	#[pallet::getter(fn unassociated_contributions)]
	pub type UnassociatedContributions<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Blake2_128Concat,
		T::RelayChainAccountId,
		RewardInfo<T>,
	>;
	#[pallet::storage]
	#[pallet::getter(fn initialized)]
	pub type Initialized<T: Config> = StorageValue<_, bool, ValueQuery, T::Initialized>;

	// #[pallet::storage]
	// #[pallet::storage_prefix = "InitRelayBlock"]
	// #[pallet::getter(fn init_vesting_block)]
	// /// Vesting block height at the initialization of the pallet
	// type InitVestingBlock<T: Config> = StorageValue<_, T::VestingBlockNumber, ValueQuery>;
	//
	// #[pallet::storage]
	// #[pallet::storage_prefix = "EndRelayBlock"]
	// #[pallet::getter(fn end_vesting_block)]
	// /// Vesting block height at the initialization of the pallet
	// type EndVestingBlock<T: Config> = StorageValue<_, T::VestingBlockNumber, ValueQuery>;
	//
	#[pallet::storage]
	#[pallet::getter(fn init_reward_amount)]
	/// Total initialized amount so far. We store this to make pallet funds == contributors reward
	/// check easier and more efficient
	pub(crate) type InitializedRewardAmount<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_contributors_by_id)]
	/// Total number of contributors to aid hinting benchmarking
	pub(crate) type TotalContributors<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// The initial payment of InitializationPayment % was paid
		InitialPaymentMade(T::AccountId, BalanceOf<T>),
		/// Someone has proven they made a contribution and associated a native identity with it.
		/// Data is the relay account,  native account and the total amount of _rewards_ that will be paid
		NativeIdentityAssociated(T::RelayChainAccountId, T::AccountId, BalanceOf<T>),
		/// A contributor has claimed some rewards.
		/// Data is the account getting paid and the amount of rewards paid.
		RewardsPaid(T::AccountId, BalanceOf<T>),
		/// A contributor has updated the reward address.
		RewardAddressUpdated(T::AccountId, T::AccountId),
		/// When initializing the reward vec an already initialized account was found
		InitializedAlreadyInitializedAccount(
			T::RelayChainAccountId,
			Option<T::AccountId>,
			BalanceOf<T>,
		),
		/// When initializing the reward vec an already initialized account was found
		InitializedAccountWithNotEnoughContribution(
			T::RelayChainAccountId,
			Option<T::AccountId>,
			BalanceOf<T>,
		),
	}
}
