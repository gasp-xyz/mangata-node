// Copyright 2019-2021 PureStake Inc.
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

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
// use crate::{
// 	BalanceOf, Call, CandidateBondChange, CandidateBondRequest, Config, DelegationChange,
// 	DelegationRequest, Pallet, Range,
// };
use crate::*;
use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_support::traits::{ExistenceRequirement, Get};
use frame_system::RawOrigin;
use itertools::Itertools;
use mangata_support::traits::{ProofOfStakeRewardsApi, XykFunctionsTrait};
use orml_tokens::MultiTokenCurrencyExtended;
use pallet_authorship::EventHandler;
use sp_runtime::Perbill;
use sp_std::vec::Vec;

const DOLLAR: u128 = 1__000_000_000_000_000_000u128;
const MGA_TOKEN_ID: u32 = 0u32;

trait ToBalance {
	fn to_balance<T: Config>(self) -> BalanceOf<T>;
}

impl ToBalance for u128 {
	fn to_balance<T: Config>(self) -> BalanceOf<T> {
		self.try_into().ok().expect("u128 should fit into Balance type")
	}
}

/// We assume
/// Mga token is token id 0
/// Not more than 100 curated tokens
/// Not more than 1000 candidates

/// To maintain simplicity, creating a pool and using resulting liqudity tokens to stake have been separated
/// To do this we mint tokens and create pools using one user, the funding account
/// And then distribute the liquidity tokens to various users
/// For each liquidity token, two additional tokens must be created
/// (Token n, Token n+1) <=> Token n+2
/// Starting from n0=5 as the first 4 are taken by the genesis config, but the starting n0 will be determined at the start of each bench by checking tokens
/// Any set of tokens x, x0=0, will have token_id, (3x+5, 3x+6) <=> 3x+7
/// Since we are creating new tokens every time we can simply just use (v, v+1) as the pooled token amounts, to mint v liquidity tokens

// pub(crate) fn payout_collator_for_round<T: Config + orml_tokens::Config + pallet_xyk::Config>(
// 	n: u32,
// ) {
// 	let dummy_user: T::AccountId = account("dummy", 0u32, 0u32);
// 	let collators: Vec<<T as frame_system::Config>::AccountId> =
// 		RoundCollatorRewardInfo::<T>::iter_key_prefix(n).collect();
// 	for collator in collators.iter() {
// 		Pallet::<T>::payout_collator_rewards(
// 			RawOrigin::Signed(dummy_user.clone()).into(),
// 			n.try_into().unwrap(),
// 			collator.clone(),
// 			<<T as Config>::MaxDelegatorsPerCandidate as Get<u32>>::get(),
// 		);
// 	}
// }

/// Mint v liquidity tokens of token set x to funding account
fn create_non_staking_liquidity_for_funding<
	T: Config<Balance = BalanceOf<T>, CurrencyId = CurrencyIdOf<T>>,
>(
	v: Option<BalanceOf<T>>,
) -> Result<CurrencyIdOf<T>, DispatchError> {
	let funding_account: T::AccountId = account("funding", 0u32, 0u32);
	let x = T::Currency::get_next_currency_id();
	let v = v.unwrap_or((1_000_000_000_000_000_000 * DOLLAR).to_balance::<T>());
	T::Currency::create(&funding_account, v)?;
	T::Currency::create(&funding_account, v + 1u32.into())?;

	assert!(
		T::Xyk::create_pool(funding_account.clone(), x, v, x + 1_u32.into(), v + 1_u32.into(),)
			.is_ok()
	);

	assert_eq!(T::Currency::total_balance(x + 2u32.into(), &funding_account), v);
	Ok(x + 2u32.into())
}

/// Mint v liquidity tokens of token set x to funding account
fn create_staking_liquidity_for_funding<
	T: Config<Balance = BalanceOf<T>, CurrencyId = CurrencyIdOf<T>>,
>(
	v: Option<BalanceOf<T>>,
) -> Result<CurrencyIdOf<T>, DispatchError> {
	let funding_account: T::AccountId = account("funding", 0u32, 0u32);
	let x = T::Currency::get_next_currency_id();
	let v = v.unwrap_or((1_000_000_000_000_000_000 * DOLLAR).to_balance::<T>());
	T::Currency::mint(MGA_TOKEN_ID.into(), &funding_account, v)?;
	T::Currency::create(&funding_account, v + 1u32.into())?;

	assert!(T::Xyk::create_pool(
		funding_account.clone(),
		MGA_TOKEN_ID.into(),
		v,
		x,
		v + 1_u32.into(),
	)
	.is_ok());

	assert_eq!(T::Currency::total_balance(x + 1u32.into(), &funding_account), v);
	Ok(x + 1u32.into())
}

/// Create a funded user.
/// Extra + min_candidate_stk is total minted funds
/// Returns tuple (id, balance)
fn create_funded_user<T: Config<Balance = BalanceOf<T>, CurrencyId = CurrencyIdOf<T>>>(
	string: &'static str,
	n: u32,
	token_id: CurrencyIdOf<T>,
	v: Option<BalanceOf<T>>,
) -> (T::AccountId, CurrencyIdOf<T>, BalanceOf<T>) {
	let funding_account: T::AccountId = account("funding", 0u32, 0u32);
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	log::info!("create user {}-{}-{}", string, n, SEED);
	let v = v.unwrap_or((1_000_000_000 * DOLLAR).to_balance::<T>());
	assert_ok!(T::Currency::transfer(
		token_id.into(),
		&funding_account,
		&user,
		v,
		ExistenceRequirement::AllowDeath
	));
	(user, token_id, v)
}

/// Create a funded delegator.
fn create_funded_delegator<T: Config<Balance = BalanceOf<T>, CurrencyId = CurrencyIdOf<T>>>(
	string: &'static str,
	n: u32,
	collator: T::AccountId,
	collator_token_id: CurrencyIdOf<T>,
	v: Option<BalanceOf<T>>,
	collator_delegator_count: u32,
) -> Result<T::AccountId, &'static str> {
	let (user, _, v) = create_funded_user::<T>(string, n, collator_token_id, v);
	Pallet::<T>::delegate(
		RawOrigin::Signed(user.clone()).into(),
		collator,
		v,
		None,
		collator_delegator_count,
		0u32, // first delegation for all calls
	)?;
	Ok(user)
}

/// Create a funded collator.
fn create_funded_collator<T: Config<Balance = BalanceOf<T>, CurrencyId = CurrencyIdOf<T>>>(
	string: &'static str,
	n: u32,
	token_id: CurrencyIdOf<T>,
	v: Option<BalanceOf<T>>,
	candidate_count: u32,
	liquidity_token_count: u32,
) -> Result<T::AccountId, &'static str> {
	let (user, token_id, v) = create_funded_user::<T>(string, n, token_id, v);
	Pallet::<T>::join_candidates(
		RawOrigin::Signed(user.clone()).into(),
		v,
		token_id,
		None,
		candidate_count,
		liquidity_token_count,
	)?;
	Ok(user)
}

pub(crate) fn roll_to_round_and_author<T: Config + pallet_session::Config>(
	n: u32,
	author: Option<T::AccountId>,
) {
	let current_round: u32 = Pallet::<T>::round().current;

	while !(Pallet::<T>::round().current >= n + current_round as u32 + 1u32) {
		<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<
			T,
		>>::block_number());
		<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(
			<frame_system::Pallet<T>>::block_number(),
		);
		<frame_system::Pallet<T>>::set_block_number(
			<frame_system::Pallet<T>>::block_number() + 1u32.into(),
		);
		<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(
			<frame_system::Pallet<T>>::block_number(),
		);
		if author.clone().is_some() {
			Pallet::<T>::note_author(author.clone().unwrap().clone());
		}
		<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(
			<frame_system::Pallet<T>>::block_number(),
		);
		if <Pallet<T> as pallet_session::ShouldEndSession<_>>::should_end_session(
			<frame_system::Pallet<T>>::block_number(),
		) {
			// This doesn't really use pallet_session::Pallet::<T>::current_index()
			// especially since pallet_session's on_initialize is not triggered (session index will always be 0)
			// But Staking's start session doesn't care as long as it isn't session 0
			<Pallet<T> as pallet_session::SessionManager<_>>::start_session(
				pallet_session::Pallet::<T>::current_index() as u32 + 1u32,
			);
		}
	}

	// Assumes round is atleast 3 blocks
	// Roll to 2 blocks into the given round
	for _i in 0..2 {
		<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<
			T,
		>>::block_number());
		<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(
			<frame_system::Pallet<T>>::block_number(),
		);
		<frame_system::Pallet<T>>::set_block_number(
			<frame_system::Pallet<T>>::block_number() + 1u32.into(),
		);
		<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(
			<frame_system::Pallet<T>>::block_number(),
		);
		if author.clone().is_some() {
			Pallet::<T>::note_author(author.clone().unwrap().clone());
		}
		<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(
			<frame_system::Pallet<T>>::block_number(),
		);
	}
}

const USER_SEED: u32 = 999666;
const DUMMY_COUNT: u32 = 999666;

benchmarks! {
	where_clause {  where T: Config<Balance = BalanceOf<T>, CurrencyId = CurrencyIdOf<T>> }
	// ROOT DISPATCHABLES

	set_total_selected {}: _(RawOrigin::Root, 100u32)
	verify {
		assert_eq!(Pallet::<T>::total_selected(), 100u32);
	}

	set_collator_commission {}: _(RawOrigin::Root, Perbill::from_percent(33))
	verify {
		assert_eq!(Pallet::<T>::collator_commission(), Perbill::from_percent(33));
	}

	// USER DISPATCHABLES

	join_candidates {
		let x in 3_u32..(<<T as Config>::MaxCollatorCandidates as Get<u32>>::get() - 1u32);
		let y in 3_u32..100;

		// Worst Case Complexity is search into a list so \exists full list before call
		let liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();
		assert!(y > liquidity_token_count);
		for i in liquidity_token_count..(y - 1u32){
			let liquidity_token_id = create_staking_liquidity_for_funding::<T>(Some(T::MinCandidateStk::get())).unwrap();
			Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(liquidity_token_id), i)?;
		}

		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(Some(T::MinCandidateStk::get() * x.into())).unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), y - 1));


		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();
		assert!(x >= candidate_count);

		// Worst Case Complexity is insertion into an ordered list so \exists full list before call

		for i in candidate_count..x {
			let seed = USER_SEED - i;
			let res = create_funded_collator::<T>(
				"collator",
				seed,
				created_liquidity_token,
				Some(T::MinCandidateStk::get()),
				candidate_count + i,
				y
			);
			if res.is_err(){
				let res_str: &str = res.unwrap_err().try_into().unwrap();
				log::info!("res_str: {:?}", res_str);
			} else {
				let collator = res.unwrap();
			}
		}
		let (caller, _, _) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, Some(T::MinCandidateStk::get()));
	}: _(RawOrigin::Signed(caller.clone()), T::MinCandidateStk::get(), created_liquidity_token, None, x, y)
	verify {
		assert!(Pallet::<T>::is_candidate(&caller));
	}

	// This call schedules the collator's exit and removes them from the candidate pool
	// -> it retains the self-bond and delegator bonds
	schedule_leave_candidates {
		let x in 3..(<<T as Config>::MaxCollatorCandidates as Get<u32>>::get() - 1u32);

		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();
		assert!(x >= candidate_count);

		// Worst Case Complexity is insertion into an ordered list so \exists full list before call

		for i in candidate_count..(x - 1u32) {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				created_liquidity_token,
				None,
				i,
				liquidity_token_count
			)?;
		}
		let caller = create_funded_collator::<T>("caller", USER_SEED, created_liquidity_token, None, x - 1u32, liquidity_token_count)?;

	}: _(RawOrigin::Signed(caller.clone()), x)
	verify {
		assert!(Pallet::<T>::candidate_state(&caller).unwrap().is_leaving());
	}

	execute_leave_candidates {
		// x is total number of delegations for the candidate
		let x in 2..(<<T as Config>::MaxTotalDelegatorsPerCandidate as Get<u32>>::get());

		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let candidate: T::AccountId = create_funded_collator::<T>(
			"unique_caller",
			USER_SEED - 100,
			created_liquidity_token,
			None,
			candidate_count + 1u32,
			liquidity_token_count,
		)?;
		// 2nd delegation required for all delegators to ensure DelegatorState updated not removed
		let second_candidate: T::AccountId = create_funded_collator::<T>(
			"unique__caller",
			USER_SEED - 99,
			created_liquidity_token,
			None,
			candidate_count + 2u32,
			liquidity_token_count,
		)?;

		let mut delegators: Vec<T::AccountId> = Vec::new();
		let mut col_del_count = 0u32;
		for i in 1..x {
			let seed = USER_SEED + i;
			let delegator = create_funded_delegator::<T>(
				"delegator",
				seed,
				candidate.clone(),
				created_liquidity_token,
				None,
				col_del_count,
			)?;
			assert_ok!(T::Currency::transfer(created_liquidity_token, &account("funding", 0u32, 0u32), &delegator, (100*DOLLAR).to_balance::<T>(), ExistenceRequirement::AllowDeath));
			assert_ok!(Pallet::<T>::delegate(
				RawOrigin::Signed(delegator.clone()).into(),
				second_candidate.clone(),
				(100*DOLLAR).to_balance::<T>(),
				None,
				col_del_count,
				1u32
			));
			assert_ok!(Pallet::<T>::schedule_revoke_delegation(
				RawOrigin::Signed(delegator.clone()).into(),
				candidate.clone()
			));
			delegators.push(delegator);
			col_del_count += 1u32;
		}
		assert_ok!(Pallet::<T>::schedule_leave_candidates(
			RawOrigin::Signed(candidate.clone()).into(),
			candidate_count + 3u32
		));
		roll_to_round_and_author::<T>(2, Some(candidate.clone()));
	}: _(RawOrigin::Signed(candidate.clone()), candidate.clone(), col_del_count)
	verify {
		assert!(Pallet::<T>::candidate_state(&candidate).is_none());
		assert!(Pallet::<T>::candidate_state(&second_candidate).is_some());
		for delegator in delegators {
			assert!(Pallet::<T>::is_delegator(&delegator));
		}
	}

	cancel_leave_candidates {
		let x in 3..(<<T as Config>::MaxCollatorCandidates as Get<u32>>::get() - 1u32);
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		// Worst Case Complexity is removal from an ordered list so \exists full list before call
		let mut candidate_count = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				created_liquidity_token,
				None,
				candidate_count,
				liquidity_token_count,
			)?;
			candidate_count += 1u32;
		}
		let caller: T::AccountId = create_funded_collator::<T>(
			"caller",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		candidate_count += 1u32;
		Pallet::<T>::schedule_leave_candidates(
			RawOrigin::Signed(caller.clone()).into(),
			candidate_count
		)?;
		candidate_count -= 1u32;
	}: _(RawOrigin::Signed(caller.clone()), candidate_count)
	verify {
		assert!(Pallet::<T>::candidate_state(&caller).unwrap().is_active());
	}

	go_offline {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;
		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Pallet::<T>::candidate_state(&caller).unwrap().is_active());
	}

	go_online {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		Pallet::<T>::go_offline(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Pallet::<T>::candidate_state(&caller).unwrap().is_active());
	}

	schedule_candidate_bond_more {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let more = (10*DOLLAR).to_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		assert_ok!(T::Currency::transfer(created_liquidity_token, &account("funding", 0u32, 0u32), &caller, more, ExistenceRequirement::AllowDeath));

	}: _(RawOrigin::Signed(caller.clone()), more, None)
	verify {
		let state = Pallet::<T>::candidate_state(&caller).expect("request bonded more so exists");
		assert_eq!(
			state.request,
			Some(CandidateBondRequest {
				amount: more,
				change: CandidateBondChange::Increase,
				when_executable: 2,
			})
		);
	}

	schedule_candidate_bond_less {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let less = (10*DOLLAR).to_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
	}: _(RawOrigin::Signed(caller.clone()), less)
	verify {
		let state = Pallet::<T>::candidate_state(&caller).expect("request bonded less so exists");
		assert_eq!(
			state.request,
			Some(CandidateBondRequest {
				amount: less,
				change: CandidateBondChange::Decrease,
				when_executable: 2,
			})
		);
	}

	execute_candidate_bond_more {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let more = (10*DOLLAR).to_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		assert_ok!(T::Currency::transfer(created_liquidity_token, &account("funding", 0u32, 0u32), &caller, more, ExistenceRequirement::AllowDeath));

		Pallet::<T>::schedule_candidate_bond_more(
			RawOrigin::Signed(caller.clone()).into(),
			more,
			None
		)?;
		roll_to_round_and_author::<T>(2, Some(caller.clone()));
	}: {
		Pallet::<T>::execute_candidate_bond_request(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone(),
			None
		)?;
	} verify {
		let expected_bond = 1000000010* DOLLAR;
		assert_eq!(<T as pallet::Config>::Currency::reserved_balance(created_liquidity_token, &caller).into(), expected_bond);
	}

	execute_candidate_bond_less {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let less = (10*DOLLAR).to_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		Pallet::<T>::schedule_candidate_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			less
		)?;
		roll_to_round_and_author::<T>(2, Some(caller.clone()));
	}: {
		Pallet::<T>::execute_candidate_bond_request(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone(),
			None
		)?;
	} verify {
		assert_eq!(<T as pallet::Config>::Currency::reserved_balance(created_liquidity_token.into(), &caller).into(), 999999990*DOLLAR);
	}

	cancel_candidate_bond_more {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let more = (10*DOLLAR).to_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		assert_ok!(T::Currency::transfer(created_liquidity_token, &account("funding", 0u32, 0u32), &caller, more, ExistenceRequirement::AllowDeath));

		Pallet::<T>::schedule_candidate_bond_more(
			RawOrigin::Signed(caller.clone()).into(),
			more,
			None
		)?;
	}: {
		Pallet::<T>::cancel_candidate_bond_request(
			RawOrigin::Signed(caller.clone()).into(),
		)?;
	} verify {
		assert!(
			Pallet::<T>::candidate_state(&caller).unwrap().request.is_none()
		);
	}

	cancel_candidate_bond_less {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let less = (10*DOLLAR).to_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		Pallet::<T>::schedule_candidate_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			less
		)?;
	}: {
		Pallet::<T>::cancel_candidate_bond_request(
			RawOrigin::Signed(caller.clone()).into(),
		)?;
	} verify {
		assert!(
			Pallet::<T>::candidate_state(&caller).unwrap().request.is_none()
		);
	}

	delegate {
		let x in 3..(<<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get().min(<<T as Config>::MaxCollatorCandidates as Get<u32>>::get() - 2u32));
		let y in 2..<<T as Config>::MaxDelegatorsPerCandidate as Get<u32>>::get();

		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		// Worst Case is full of delegations before calling `delegate`
		let mut collators: Vec<T::AccountId> = Vec::new();
		// Initialize MaxDelegationsPerDelegator collator candidates
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				created_liquidity_token,
				None,
				collators.len() as u32 + candidate_count,
				liquidity_token_count,
			)?;
			collators.push(collator.clone());
		}

		let (caller, _, _) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, Some((100 * DOLLAR * (collators.len() as u128 + 1u128) + 1u128).to_balance::<T>()));
		// Delegation count
		let mut del_del_count = 0u32;
		// Nominate MaxDelegationsPerDelegators collator candidates
		for col in collators.clone() {
			Pallet::<T>::delegate(
				RawOrigin::Signed(caller.clone()).into(), col, (100 * DOLLAR).to_balance::<T>(), None, 0u32, del_del_count
			)?;
			del_del_count += 1u32;
		}
		// Last collator to be delegated
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			collators.len() as u32 + candidate_count + 1,
			liquidity_token_count,
		)?;
		// Worst Case Complexity is insertion into an almost full collator
		let mut col_del_count = 0u32;
		for i in 1..y {
			let seed = USER_SEED + i;
			let _ = create_funded_delegator::<T>(
				"delegator",
				seed,
				collator.clone(),
				created_liquidity_token,
				None,
				col_del_count,
			)?;
			col_del_count += 1u32;
		}
	}: _(RawOrigin::Signed(caller.clone()), collator, (100*DOLLAR + 1u128).to_balance::<T>(), None, col_del_count, del_del_count)
	verify {
		assert!(Pallet::<T>::is_delegator(&caller));
	}

	schedule_leave_delegators {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, _) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			(100*DOLLAR).to_balance::<T>(),
			None,
			0u32,
			0u32
		)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Pallet::<T>::delegator_state(&caller).unwrap().is_leaving());
	}

	execute_leave_delegators {
		let x in 2..(<<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get().min(<<T as Config>::MaxCollatorCandidates as Get<u32>>::get() - 2u32));
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		// Worst Case is full of delegations before execute exit
		let mut collators: Vec<T::AccountId> = Vec::new();
		// Initialize MaxDelegationsPerDelegator collator candidates
		for i in 1..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				created_liquidity_token,
				None,
				collators.len() as u32 + candidate_count,
				liquidity_token_count
			)?;
			collators.push(collator.clone());
		}
		// Fund the delegator
		let (caller, _, _) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, Some((100 * DOLLAR * (collators.len() as u128)).to_balance::<T>()));
		// Delegation count
		let mut delegation_count = 0u32;
		let author = collators[0].clone();
		// Nominate MaxDelegationsPerDelegators collator candidates
		for col in collators {
			Pallet::<T>::delegate(
				RawOrigin::Signed(caller.clone()).into(),
				col,
				(100*DOLLAR).to_balance::<T>(),
				None,
				0u32,
				delegation_count
			)?;
			delegation_count += 1u32;
		}
		Pallet::<T>::schedule_leave_delegators(RawOrigin::Signed(caller.clone()).into())?;
		roll_to_round_and_author::<T>(2, Some(author));
	}: _(RawOrigin::Signed(caller.clone()), caller.clone(), delegation_count)
	verify {
		assert!(Pallet::<T>::delegator_state(&caller).is_none());
	}

	cancel_leave_delegators {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			v,
			None,
			0u32,
			0u32
		)?;
		Pallet::<T>::schedule_leave_delegators(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Pallet::<T>::delegator_state(&caller).unwrap().is_active());
	}

	schedule_revoke_delegation {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			v,
			None,
			0u32,
			0u32
		)?;
	}: _(RawOrigin::Signed(caller.clone()), collator.clone())
	verify {
		assert_eq!(
			Pallet::<T>::delegator_state(&caller).unwrap().requests().get(&collator),
			Some(&DelegationRequest {
				collator,
				amount: v,
				when_executable: 2,
				action: DelegationChange::Revoke
			})
		);
	}

	schedule_delegator_bond_more {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);
		Pallet::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			v - (10*DOLLAR).to_balance::<T>(),
			None,
			0u32,
			0u32
		)?;
	}: _(RawOrigin::Signed(caller.clone()), collator.clone(), (10*DOLLAR).to_balance::<T>(), None)
	verify {
		let state = Pallet::<T>::delegator_state(&caller)
			.expect("just request bonded less so exists");
		assert_eq!(
			state.requests().get(&collator),
			Some(&DelegationRequest {
				collator,
				amount: (10*DOLLAR).to_balance::<T>(),
				when_executable: 2,
				action: DelegationChange::Increase
			})
		);
	}

	schedule_delegator_bond_less {

		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			v,
			None,
			0u32,
			0u32
		)?;
	}: _(RawOrigin::Signed(caller.clone()), collator.clone(), (10*DOLLAR).to_balance::<T>())
	verify {
		let state = Pallet::<T>::delegator_state(&caller)
			.expect("just request bonded less so exists");
		assert_eq!(
			state.requests().get(&collator),
			Some(&DelegationRequest {
				collator,
				amount: (10*DOLLAR).to_balance::<T>(),
				when_executable: 2,
				action: DelegationChange::Decrease
			})
		);
	}

	execute_revoke_delegation {

		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);

		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			v,
			None,
			0u32,
			0u32
		)?;
		Pallet::<T>::schedule_revoke_delegation(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone()
		)?;
		roll_to_round_and_author::<T>(2, Some(collator.clone()));
	}: {
		Pallet::<T>::execute_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone(),
			collator.clone(),
			None
		)?;
	} verify {
		assert!(
			!Pallet::<T>::is_delegator(&caller)
		);
	}

	execute_delegator_bond_more {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);

		Pallet::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			v - (10*DOLLAR).to_balance::<T>(),
			None,
			0u32,
			0u32
		)?;
		Pallet::<T>::schedule_delegator_bond_more(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			(10*DOLLAR).to_balance::<T>(),
			None
		)?;
		roll_to_round_and_author::<T>(2, Some(collator.clone()));
	}: {
		Pallet::<T>::execute_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone(),
			collator.clone(),
			None
		)?;
	} verify {
		let expected_bond = 1000000000* DOLLAR;
		assert_eq!(<T as pallet::Config>::Currency::reserved_balance(created_liquidity_token.into(), &caller).into(), expected_bond);
	}

	execute_delegator_bond_less {

		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			v,
			None,
			0u32,
			0u32
		)?;
		let bond_less = (10*DOLLAR).to_balance::<T>();
		Pallet::<T>::schedule_delegator_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			bond_less
		)?;
		roll_to_round_and_author::<T>(2, Some(collator.clone()));
	}: {
		Pallet::<T>::execute_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone(),
			collator.clone(),
			None
		)?;
	} verify {
		let expected = v - bond_less;
		assert_eq!(<T as pallet::Config>::Currency::reserved_balance(created_liquidity_token.into(), &caller), expected);
	}

	cancel_revoke_delegation {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);

		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			v,
			None,
			0u32,
			0u32
		)?;
		Pallet::<T>::schedule_revoke_delegation(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone()
		)?;
	}: {
		Pallet::<T>::cancel_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone()
		)?;
	} verify {
		assert!(
			Pallet::<T>::delegator_state(&caller).unwrap().requests().get(&collator).is_none()
		);
	}

	cancel_delegator_bond_more {

		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, v) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);

		Pallet::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			v - (10*DOLLAR).to_balance::<T>(),
			None,
			0u32,
			0u32
		)?;
		Pallet::<T>::schedule_delegator_bond_more(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			(10*DOLLAR).to_balance::<T>(),
			None
		)?;
		roll_to_round_and_author::<T>(2, Some(collator.clone()));
	}: {
		Pallet::<T>::cancel_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone()
		)?;
	} verify {
		assert!(
			Pallet::<T>::delegator_state(&caller)
				.unwrap()
				.requests()
				.get(&collator)
				.is_none()
		);
	}

	cancel_delegator_bond_less {
		let created_liquidity_token =
			create_staking_liquidity_for_funding::<T>(None).unwrap();

		let mut liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), liquidity_token_count));

		liquidity_token_count = liquidity_token_count + 1u32;

		let candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			created_liquidity_token,
			None,
			candidate_count,
			liquidity_token_count,
		)?;
		let (caller, _, total) = create_funded_user::<T>("caller", USER_SEED, created_liquidity_token, None);
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			total - (10*DOLLAR).to_balance::<T>(),
			None,
			0u32,
			0u32
		)?;
		let bond_less = (10*DOLLAR).to_balance::<T>();
		Pallet::<T>::schedule_delegator_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			bond_less
		)?;
		roll_to_round_and_author::<T>(2, Some(collator.clone()));
	}: {
		Pallet::<T>::cancel_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone()
		)?;
	} verify {
		assert!(
			Pallet::<T>::delegator_state(&caller)
				.unwrap()
				.requests()
				.get(&collator)
				.is_none()
		);
	}

	add_staking_liquidity_token {
		let x in 3..100;

		let liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();
		assert!(x > liquidity_token_count);
		for i in liquidity_token_count..(x){
			let liquidity_token_id = create_staking_liquidity_for_funding::<T>(Some(T::MinCandidateStk::get())).unwrap();
			assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(liquidity_token_id), i));
		}

		let liquidity_token_id = create_staking_liquidity_for_funding::<T>(Some(T::MinCandidateStk::get())).unwrap();

	}: _(RawOrigin::Root, PairedOrLiquidityToken::Liquidity(liquidity_token_id), x)
	verify {
		assert!(
			Pallet::<T>::staking_liquidity_tokens()
				.contains_key(&liquidity_token_id)
		);
	}

	remove_staking_liquidity_token {
		let x in 3..100;

		let liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();
		assert!(x > liquidity_token_count);
		for i in liquidity_token_count..(x - 1u32){
			let token_id = create_staking_liquidity_for_funding::<T>(Some(T::MinCandidateStk::get())).unwrap();
			assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(token_id), i));
		}

		let token_id = create_staking_liquidity_for_funding::<T>(Some(T::MinCandidateStk::get())).unwrap();
		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(token_id), x - 1u32));

	}: _(RawOrigin::Root, PairedOrLiquidityToken::Liquidity(token_id), x)
	verify {
		assert!(
			!Pallet::<T>::staking_liquidity_tokens()
				.contains_key(&(token_id))
		);
	}

	aggregator_update_metadata {

		let x = <<T as Config>::MaxCollatorCandidates as Get<u32>>::get() - 2u32; // to account for the two candidates we start with


		let start_liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		let initial_candidates: Vec<T::AccountId> = Pallet::<T>::candidate_pool().0.into_iter().map(|x| x.owner).collect::<_>();
		let base_candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();
		assert_eq!(base_candidate_count, 2);

		let mut candidates: Vec<T::AccountId> = Vec::<T::AccountId>::new();


		const SEED: u32 = 0;

		for i in 0u32..x{

			let created_liquidity_token =
				create_staking_liquidity_for_funding::<T>(None).unwrap();

			assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), 1000));

			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				created_liquidity_token,
				None,
				candidates.len() as u32 + base_candidate_count,
				1000
			)?;

			candidates.push(collator.clone());

		}

		let aggregator: T::AccountId = account("aggregator", 0u32, SEED);
		assert_ok!(Pallet::<T>::aggregator_update_metadata(RawOrigin::Signed(
			aggregator.clone()).into(),
			candidates.clone(),
			MetadataUpdateAction::ExtendApprovedCollators
		));

		for i in 0u32..(x){

			let seed = USER_SEED - i;

			let collator: T::AccountId = account("collator", seed, SEED);
			assert_ok!(Pallet::<T>::update_candidate_aggregator(RawOrigin::Signed(
				collator.clone()).into(),
				Some(aggregator.clone()),
			));

		}

		for i in 0u32..(x){

			let seed = USER_SEED - i;

			let collator: T::AccountId = account("collator", seed, SEED);
			assert_eq!(CandidateAggregator::<T>::get()
				.get(&collator).cloned(),
				Some(aggregator.clone()),
			);

		}

		assert_eq!(AggregatorMetadata::<T>::get(&aggregator).unwrap().token_collator_map.len(), x as usize);
		assert_eq!(AggregatorMetadata::<T>::get(&aggregator).unwrap().approved_candidates.len(), x as usize);

	}: _(RawOrigin::Signed(aggregator.clone()), candidates.clone(), MetadataUpdateAction::RemoveApprovedCollators)
	verify {

		for i in 0u32..(x){

			let seed = USER_SEED - i;

			let collator = account("collator", seed, SEED);
			assert_eq!(CandidateAggregator::<T>::get()
				.get(&collator).cloned(),
				None,
			);

		}

		assert_eq!(AggregatorMetadata::<T>::get(&aggregator), None);
		assert_eq!(AggregatorMetadata::<T>::get(&aggregator), None);

	}

	update_candidate_aggregator {

		let x = <<T as Config>::MaxCollatorCandidates as Get<u32>>::get() - 2u32; // to account for the two candidates we start with


		let start_liquidity_token_count: u32 = Pallet::<T>::staking_liquidity_tokens().len().try_into().unwrap();

		let initial_candidates: Vec<T::AccountId> = Pallet::<T>::candidate_pool().0.into_iter().map(|x| x.owner).collect::<_>();
		let base_candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();
		assert_eq!(base_candidate_count, 2);

		let mut candidates: Vec<T::AccountId> = Vec::<T::AccountId>::new();

		const SEED: u32 = 0;

		for i in 0u32..x{

			let created_liquidity_token =
				create_staking_liquidity_for_funding::<T>(None).unwrap();

			assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), 1000));

			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				created_liquidity_token,
				None,
				candidates.len() as u32 + base_candidate_count,
				1000
			)?;

			candidates.push(collator.clone());

		}

		let aggregator: T::AccountId = account("aggregator", 0u32, SEED);
		assert_ok!(Pallet::<T>::aggregator_update_metadata(RawOrigin::Signed(
			aggregator.clone()).into(),
			candidates.clone(),
			MetadataUpdateAction::ExtendApprovedCollators
		));

		for i in 1u32..(x){

			let seed = USER_SEED - i;

			let collator: T::AccountId = account("collator", seed, SEED);
			assert_ok!(Pallet::<T>::update_candidate_aggregator(RawOrigin::Signed(
				collator.clone()).into(),
				Some(aggregator.clone()),
			));

		}

		for i in 1u32..(x){

			let seed = USER_SEED - i;

			let collator: T::AccountId = account("collator", seed, SEED);
			assert_eq!(CandidateAggregator::<T>::get()
				.get(&collator).cloned(),
				Some(aggregator.clone()),
			);

		}

		let collator_switching: T::AccountId = account("collator", USER_SEED, SEED);
		let aggregator_old: T::AccountId = account("aggregator", 1u32, SEED);
		assert_ok!(Pallet::<T>::aggregator_update_metadata(RawOrigin::Signed(
			aggregator_old.clone()).into(),
			vec![collator_switching.clone()],
			MetadataUpdateAction::ExtendApprovedCollators
		));

		assert_ok!(Pallet::<T>::update_candidate_aggregator(RawOrigin::Signed(
			collator_switching.clone()).into(),
			Some(aggregator_old.clone()),
		));

		assert_eq!(CandidateAggregator::<T>::get()
				.get(&collator_switching).cloned(),
				Some(aggregator_old.clone()),
			);

		assert_eq!(AggregatorMetadata::<T>::get(&aggregator).unwrap().token_collator_map.len(), (x - 1) as usize);
		assert_eq!(AggregatorMetadata::<T>::get(&aggregator).unwrap().approved_candidates.len(), x as usize);
		assert_eq!(AggregatorMetadata::<T>::get(&aggregator_old).unwrap().token_collator_map.len(), 1 as usize);
		assert_eq!(AggregatorMetadata::<T>::get(&aggregator_old).unwrap().approved_candidates.len(), 1 as usize);

	}: _(RawOrigin::Signed(collator_switching.clone()), Some(aggregator.clone()))
	verify {

		for i in 0u32..(x){

			let seed = USER_SEED - i;

			let collator = account("collator", seed, SEED);
			assert_eq!(CandidateAggregator::<T>::get()
				.get(&collator).cloned(),
				Some(aggregator.clone()),
			);

		}

		assert_eq!(AggregatorMetadata::<T>::get(&aggregator).unwrap().token_collator_map.len(), x as usize);
		assert_eq!(AggregatorMetadata::<T>::get(&aggregator).unwrap().approved_candidates.len(), x as usize);
		assert_eq!(AggregatorMetadata::<T>::get(&aggregator_old).unwrap().token_collator_map.len(), 0 as usize);
		assert_eq!(AggregatorMetadata::<T>::get(&aggregator_old).unwrap().approved_candidates.len(), 1 as usize);

	}

	payout_collator_rewards {

		let funding_account: T::AccountId = account("funding", 0u32, 0u32);
		assert_ok!(T::Currency::mint(MGA_TOKEN_ID.into(), &<<T as Config>::StakingIssuanceVault as Get<T::AccountId>>::get(), (1_000_000*DOLLAR).to_balance::<T>()));

		const SEED: u32 = 0;
		let collator: T::AccountId = account("collator", 0u32, SEED);

		let mut round_collator_reward_info = RoundCollatorRewardInfoType::<T::AccountId, BalanceOf<T>>::default();
		round_collator_reward_info.collator_reward = (1*DOLLAR).to_balance::<T>();

		for i in 0u32..<<T as Config>::MaxDelegatorsPerCandidate as Get<u32>>::get() {
			let delegator: T::AccountId = account("delegator", USER_SEED - i, SEED);
			round_collator_reward_info.delegator_rewards.insert(delegator, (1*DOLLAR).to_balance::<T>());
		}

		RoundCollatorRewardInfo::<T>::insert(collator.clone(), 1000, round_collator_reward_info);

		assert_eq!(RoundCollatorRewardInfo::<T>::get(&collator, 1000).unwrap().collator_reward, (1*DOLLAR).to_balance::<T>());

	}: _(RawOrigin::Signed(collator.clone()), collator.clone(), Some(1))
	verify {

		assert_eq!(RoundCollatorRewardInfo::<T>::get(&collator, 1000), None);

	}


	payout_delegator_reward {

		let funding_account: T::AccountId = account("funding", 0u32, 0u32);
		assert_ok!(T::Currency::mint(MGA_TOKEN_ID.into(), &<<T as Config>::StakingIssuanceVault as Get<T::AccountId>>::get(), (1_000_000*DOLLAR).to_balance::<T>()));

		const SEED: u32 = 0;
		let collator: T::AccountId = account("collator", 0u32, SEED);

		let mut round_collator_reward_info = RoundCollatorRewardInfoType::<T::AccountId, BalanceOf<T>>::default();
		round_collator_reward_info.collator_reward = (1*DOLLAR).to_balance::<T>();

		for i in 0u32..(<<T as Config>::MaxDelegatorsPerCandidate as Get<u32>>::get()) {
			let delegator: T::AccountId = account("delegator", USER_SEED - i, SEED);
			round_collator_reward_info.delegator_rewards.insert(delegator, (1*DOLLAR).to_balance::<T>());
		}

		RoundCollatorRewardInfo::<T>::insert(collator.clone(), 1000, round_collator_reward_info);

		assert_eq!(RoundCollatorRewardInfo::<T>::get(&collator, 1000).unwrap().collator_reward, (1*DOLLAR).to_balance::<T>());
		assert_eq!(RoundCollatorRewardInfo::<T>::get(&collator, 1000).unwrap().delegator_rewards.len(), <<T as Config>::MaxDelegatorsPerCandidate as Get<u32>>::get() as usize);

		let delegator_target: T::AccountId = account("delegator", USER_SEED, SEED);

	}: _(RawOrigin::Signed(delegator_target.clone()), 1000, collator.clone(), delegator_target.clone())
	verify {

		assert_eq!(RoundCollatorRewardInfo::<T>::get(&collator, 1000).unwrap().collator_reward, (1*DOLLAR).to_balance::<T>());
		assert_eq!(RoundCollatorRewardInfo::<T>::get(&collator, 1000).unwrap().delegator_rewards.len(), (<<T as Config>::MaxDelegatorsPerCandidate as Get<u32>>::get() - 1) as usize);

	}

	// Session Change

	// The session pallet's on initialize is called but should_end_session returns false
	// This essentially just benhcmarks should_end_session
	passive_session_change {
		// Move on by a block
		// Assuming we start at (say) 0, and that round is atleast 3 blocks.

		<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
		<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
		<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
		<frame_system::Pallet<T>>::set_block_number(<frame_system::Pallet<T>>::block_number() + 1u32.into());
		<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
		<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());

		assert_eq!(pallet_session::Pallet::<T>::current_index() as u32, 0u32);

		assert!(!<Pallet::<T> as pallet_session::ShouldEndSession<_>>::should_end_session(<frame_system::Pallet<T>>::block_number()));

	}: {<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());}
	verify {
		assert_eq!(pallet_session::Pallet::<T>::current_index() as u32, 0u32);
	}

	active_session_change {

		// liquidity tokens
		let x in 3..100;
		// candidate_count
		let y in (<<T as Config>::MinSelectedCandidates as Get<u32>>::get() + 1u32)..(<<T as Config>::MaxCollatorCandidates as Get<u32>>::get() - 2u32); // to account for the two candidates we start with
		// MaxDelegatorsPerCandidate
		let z in 3..<<T as Config>::MaxDelegatorsPerCandidate as Get<u32>>::get();

		// // Since now an aggregator can have multiple collators each of whose rewards will be written to the storage individually
		// // Total selected
		let w = <<T as Config>::MinSelectedCandidates as Get<u32>>::get() + 1u32;

		// // liquidity tokens
		// let x = 100;
		// // candidate_count
		// let y = 190;
		// // MaxDelegatorsPerCandidate
		// let z = 200;
		// // Total selected
		// let w = 190;

		assert_ok!(<pallet_issuance::Pallet<T>>::finalize_tge(RawOrigin::Root.into()));
		assert_ok!(<pallet_issuance::Pallet<T>>::init_issuance_config(RawOrigin::Root.into()));
		assert_ok!(<pallet_issuance::Pallet<T>>::calculate_and_store_round_issuance(0u32));

		assert_ok!(Pallet::<T>::set_total_selected(RawOrigin::Root.into(), w));

		// We will prepare `x-1` liquidity tokens in loop and then another after

		let start_liquidity_token = Pallet::<T>::staking_liquidity_tokens();
		let start_liquidity_token_count: u32 = start_liquidity_token.len().try_into().unwrap();
		for (token,_) in start_liquidity_token {
			// <pallet_issuance::Pallet<T> as PoolPromoteApi>::update_pool_promotion(token, Some(1));
			T::RewardsApi::enable(token, 1);
		}

		assert!(x > start_liquidity_token_count);
		// create X - 1 Tokens now and then remaining one
		for i in start_liquidity_token_count..(x-1){
			let created_liquidity_token = create_staking_liquidity_for_funding::<T>(Some(T::MinCandidateStk::get())).unwrap();
			Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), i).unwrap();
			// <pallet_issuance::Pallet<T> as PoolPromoteApi>::update_pool_promotion(created_liquidity_token, Some(1));
			T::RewardsApi::enable(created_liquidity_token, 1);
		}

		// Now to prepare the liquidity token we will use for collator and delegators
		let amount = ((z*(y+1)) as u128 * 100 * DOLLAR).to_balance::<T>() + T::MinCandidateStk::get() * DOLLAR.to_balance::<T>();
		let created_liquidity_token = create_staking_liquidity_for_funding::<T>(Some(amount)).unwrap();
		assert_ok!(Pallet::<T>::add_staking_liquidity_token(RawOrigin::Root.into(), PairedOrLiquidityToken::Liquidity(created_liquidity_token), x));
		// <pallet_issuance::Pallet<T> as PoolPromoteApi>::update_pool_promotion(created_liquidity_token, Some(1));
		T::RewardsApi::enable(created_liquidity_token, 1);


		// Now we will create y funded collators
		let initial_candidates: Vec<T::AccountId> = Pallet::<T>::candidate_pool().0.into_iter().map(|x| x.owner).collect::<_>();
		let base_candidate_count: u32 = Pallet::<T>::candidate_pool().0.len().try_into().unwrap();

		assert_eq!(base_candidate_count, 2);
		assert_eq!(x as usize , StakingLiquidityTokens::<T>::get().len());

		// let pool_rewards = pallet_issuance::PromotedPoolsRewardsV2::<T>::get();
		// assert_eq!(pool_rewards.len(), x as usize);


		let mut candidates = (0u32..y)
			.map(|i|{
				create_funded_collator::<T>(
				"collator",
				USER_SEED - i,
				created_liquidity_token,
				Some(T::MinCandidateStk::get()),
				i + base_candidate_count,
				x
				)
			}).collect::<Result<Vec<T::AccountId>, &'static str>>()?;

		// create one aggregator per candidate
		for (id, c) in candidates.iter().enumerate() {
			let aggregator: T::AccountId = account("aggregator", id as u32, 0);
			assert_ok!(Pallet::<T>::aggregator_update_metadata(RawOrigin::Signed(
				aggregator.clone()).into(),
				vec![c.clone()],
				MetadataUpdateAction::ExtendApprovedCollators,
			));

			assert_ok!(Pallet::<T>::update_candidate_aggregator(RawOrigin::Signed(
				c.clone()).into(),
				Some(aggregator.clone()),
			));
		}

		assert_eq!(candidates.len(), y as usize);
		//
		// // Now we will create `z*y` delegators each with `100*DOLLAR` created_liquidity_token tokens
		//
		let delegators_count = z*y;
		let delegators: Vec<_> = (0u32..delegators_count)
		.map(|i|
			create_funded_user::<T>("delegator", USER_SEED-i, created_liquidity_token, Some((100*DOLLAR).to_balance::<T>()))
		).map(|(account, _token_id, _amount)| account)
		.collect();
		assert_eq!(delegators.len(), (z*y) as usize);

		for (delegators, candidate) in delegators.iter().chunks(z as usize).into_iter()
			.zip(candidates.clone())
		{

			for (count, delegator) in delegators.into_iter().enumerate() {
				Pallet::<T>::delegate(RawOrigin::Signed(
					delegator.clone()).into(),
					candidate.clone().into(),
					(100*DOLLAR).to_balance::<T>(),
					None,
					count as u32,
					0u32,
				).unwrap();
			}

			assert_eq!(Pallet::<T>::candidate_state(candidate.clone()).unwrap().delegators.0.len() , z as usize);
			assert_eq!(Pallet::<T>::candidate_state(candidate.clone()).unwrap().top_delegations.len() , z as usize);
			assert_eq!(Pallet::<T>::candidate_state(candidate.clone()).unwrap().bottom_delegations.len() ,  0usize);

		}


		//
		// Remove the initial two collators so that they do not get selected
		// We do this as the two collators do not have max delegators and would not be worst case

		for initial_candidate in initial_candidates{
			assert_ok!(Pallet::<T>::go_offline(RawOrigin::Signed(
				initial_candidate.clone()).into()));
		}



		// We would like to move on to the end of round 4
		let session_to_reach = 4u32;

		// Moves to the end of the round
		// Infinite loop that breaks when should_end_session is true
		loop {
			<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<frame_system::Pallet<T>>::set_block_number(<frame_system::Pallet<T>>::block_number() + 1u32.into());
			<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			if Pallet::<T>::round().current == session_to_reach {
				for i in 0..2{
					<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
					<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
					<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
					<frame_system::Pallet<T>>::set_block_number(<frame_system::Pallet<T>>::block_number() + 1u32.into());
					<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
					<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
					<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
				}
				break;
			}
		}

		let selected_author = Pallet::<T>::selected_candidates();


		// We would like to move on to the end of round 1
		let session_to_reach = 5u32;

		// Moves to the end of the round 0
		// Infinite loop that breaks when should_end_session is true
		loop {
			<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<frame_system::Pallet<T>>::set_block_number(<frame_system::Pallet<T>>::block_number() + 1u32.into());
			<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			if Pallet::<T>::round().current == session_to_reach {
				for i in 0..2{
					<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
					<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
					<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
					<frame_system::Pallet<T>>::set_block_number(<frame_system::Pallet<T>>::block_number() + 1u32.into());
					<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
					<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
					<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
				}
				break;
			}
		}


		assert_eq!(pallet_session::Pallet::<T>::current_index() as u32, 5u32);
		assert_eq!(Pallet::<T>::round().current as u32, 5u32);
		assert_eq!(selected_author.len(), (w as usize).min(Pallet::<T>::candidate_pool().0.len() as usize));


		let candidate_pool_state = Pallet::<T>::candidate_pool().0;

		for (i, candidate_bond) in candidate_pool_state.into_iter().enumerate() {
			if candidate_bond.liquidity_token == created_liquidity_token {
				assert_eq!(candidate_bond.amount.into(), (1+(z as u128)*100)*DOLLAR);
			}
		}

		for author in selected_author.clone() {
			Pallet::<T>::note_author(author.clone());
		}

		// We would like to move on to the end of round 1
		let end_of_session_to_reach = 6u32;
		// let pool_rewards = pallet_issuance::PromotedPoolsRewardsV2::<T>::get();
		// assert_eq!(pool_rewards.len(), x as usize);

		// Moves to the end of the round 0
		// Infinite loop that breaks when should_end_session is true
		loop {
			<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_finalize(<frame_system::Pallet<T>>::block_number());
			<frame_system::Pallet<T>>::set_block_number(<frame_system::Pallet<T>>::block_number() + 1u32.into());
			<frame_system::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			<pallet::Pallet<T> as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			if <Pallet::<T> as pallet_session::ShouldEndSession<_>>::should_end_session(<frame_system::Pallet<T>>::block_number())
				&& (Pallet::<T>::round().current == end_of_session_to_reach) {
				break;
			} else {
				<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());
			}
		}


		assert_eq!(pallet_session::Pallet::<T>::current_index() as u32, 6u32);
		assert_eq!(Pallet::<T>::round().current as u32, 6u32);

		assert!(<Pallet::<T> as pallet_session::ShouldEndSession<_>>::should_end_session(<frame_system::Pallet<T>>::block_number()));

		for author in selected_author.clone() {
			for candidate in AggregatorMetadata::<T>::get(&author).unwrap().token_collator_map.iter().map(|x| x.1){
			assert!(T::Currency::total_balance(MGA_TOKEN_ID.into(), &candidate).is_zero());
			}
		}

	}: {<pallet_session::Pallet::<T>  as frame_support::traits::Hooks<_>>::on_initialize(<frame_system::Pallet<T>>::block_number());}
	verify {
		assert_eq!(pallet_session::Pallet::<T>::current_index() as u32, 7u32);
		assert_eq!(Pallet::<T>::round().current as u32, 7u32);
		assert_eq!(w as usize, candidates.iter().filter_map(|c| RoundCollatorRewardInfo::<T>::get(c.clone(), 5u32)).count());
	}

}
