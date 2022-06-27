use super::*;
use frame_support::{
	migration::storage_key_iter,
	traits::{Get, GetStorageVersion, PalletInfoAccess, StorageVersion},
	weights::Weight,
};
use parachain_staking::{CollatorCandidate, Delegator};
use sp_std::collections::btree_map::BTreeMap;

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;

#[cfg(feature = "try-runtime")]
pub fn migrate_from_v0_pre_runtime_upgrade<T: Config, P: GetStorageVersion + PalletInfoAccess>(
) -> Result<(), &'static str> {
	// Check consistency of xyk storage, staking storage and orml reserves
	// Ensure reserve and relock status is zero

	// Get all required storage from xyk and staking

	let collator_storage = storage_key_iter::<
		T::AccountId,
		CollatorCandidate<T::AccountId>,
		Twox64Concat,
	>(b"ParachainStaking", b"CandidateState")
	.collect::<Vec<_>>();

	let delegator_storage =
		storage_key_iter::<T::AccountId, Delegator<T::AccountId>, Twox64Concat>(
			b"ParachainStaking",
			b"DelegatorState",
		)
		.collect::<Vec<_>>();

	let activation_storage = storage_key_iter::<(T::AccountId, TokenId), Balance, Twox64Concat>(
		b"Xyk",
		b"LiquidityMiningActiveUser",
	)
	.collect::<Vec<_>>();

	// Get all users list

	// (Balance, Balance, Balance)
	// represents reserves
	// xyk, staking, total
	let mut user_reserve_info: BTreeMap<(T::AccountId, TokenId), (Balance, Balance, Balance)> =
		BTreeMap::new();

	for (collator_account, collator_info) in collator_storage.iter() {
		if let Some((xyk_reserve, staking_reserve, total_reserve)) =
			user_reserve_info.get_mut(&(collator_account.clone(), collator_info.liquidity_token))
		{
			*staking_reserve = staking_reserve.saturating_add(collator_info.bond);
			*total_reserve = total_reserve.saturating_add(collator_info.bond);
		} else {
			user_reserve_info.insert(
				(collator_account.clone(), collator_info.liquidity_token),
				(Balance::zero(), collator_info.bond, collator_info.bond),
			);
		};
	}

	for (delegator_account, delegator_info) in delegator_storage.iter() {
		for delegation in delegator_info.delegations.0.iter() {
			if let Some((xyk_reserve, staking_reserve, total_reserve)) =
				user_reserve_info.get_mut(&(delegator_account.clone(), delegation.liquidity_token))
			{
				*staking_reserve = staking_reserve.saturating_add(delegation.amount);
				*total_reserve = total_reserve.saturating_add(delegation.amount);
			} else {
				user_reserve_info.insert(
					(delegator_account.clone(), delegation.liquidity_token),
					(Balance::zero(), delegation.amount, delegation.amount),
				);
			};
		}
	}

	for ((account, liquidity_token), amount) in activation_storage.iter() {
		if let Some((xyk_reserve, staking_reserve, total_reserve)) =
			user_reserve_info.get_mut(&(account.clone(), *liquidity_token))
		{
			*xyk_reserve = xyk_reserve.saturating_add(*amount);
			*total_reserve = total_reserve.saturating_add(*amount);
		} else {
			user_reserve_info
				.insert((account.clone(), *liquidity_token), (*amount, Balance::zero(), *amount));
		};
	}

	for ((account, liquidity_token), (xyk_reserve, staking_reserve, total_reserve)) in
		user_reserve_info.iter()
	{
		assert_eq!(xyk_reserve.saturating_add(*staking_reserve), *total_reserve);
		assert_eq!(
			T::Tokens::reserved_balance((*liquidity_token).into(), account).into(),
			*total_reserve
		);
		let reserve_status = Pallet::<T>::get_reserve_status(account, liquidity_token);
		assert!(reserve_status.staked_and_activated_reserves.is_zero());
		assert!(reserve_status.activated_unstaked_reserves.is_zero());
		assert!(reserve_status.staked_unactivated_reserves.is_zero());
		assert!(reserve_status.unspent_reserves.is_zero());
		assert!(reserve_status.relock_amount.is_zero());
		assert!(Pallet::<T>::get_relock_status(account, liquidity_token).is_empty());
	}

	Pallet::<T>::set_temp_storage(user_reserve_info, "user_reserve_info");

	Ok(())
}

/// Migrate the pallet storage to v1.
pub fn migrate_from_v0<T: Config, P: GetStorageVersion + PalletInfoAccess>(
) -> frame_support::weights::Weight {
	let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();

	if on_chain_storage_version != 0 {
		log::info!(
			target: "mpl",
			"Attempted to apply xyk-staking-mpl consistency migration to mpl but failed because storage version is {:?}, and not 0",
			on_chain_storage_version,
		);
		return T::DbWeight::get().reads(1)
	}

	// Apply storage migration from StorageVersion 0 to 1

	//////////////////////////////////////////////////////

	let collator_storage = storage_key_iter::<
		T::AccountId,
		CollatorCandidate<T::AccountId>,
		Twox64Concat,
	>(b"ParachainStaking", b"CandidateState")
	.collect::<Vec<_>>();

	let delegator_storage =
		storage_key_iter::<T::AccountId, Delegator<T::AccountId>, Twox64Concat>(
			b"ParachainStaking",
			b"DelegatorState",
		)
		.collect::<Vec<_>>();

	let activation_storage = storage_key_iter::<(T::AccountId, TokenId), Balance, Twox64Concat>(
		b"Xyk",
		b"LiquidityMiningActiveUser",
	)
	.collect::<Vec<_>>();

	// Now we have all the info in memory

	let mut collators_processed_count = u32::zero();

	let mut delegations_processed_count = u32::zero();

	let mut activation_processed_count = u32::zero();

	for (collator_account, collator_info) in collator_storage.iter() {
		let mut reserve_status =
			Pallet::<T>::get_reserve_status(collator_account, collator_info.liquidity_token);
		reserve_status.staked_unactivated_reserves =
			reserve_status.staked_unactivated_reserves.saturating_add(collator_info.bond);

		log::info!(
			target: "mpl",
			"Reserve status after processing collator: {:?} on liquidity token: {:?} with amount: {:?} := {:?}",
			collator_account,
			collator_info.liquidity_token,
			collator_info.bond,
			reserve_status,
		);

		collators_processed_count = collators_processed_count.saturating_add(1u32);

		ReserveStatus::<T>::insert(collator_account, collator_info.liquidity_token, reserve_status);
	}

	for (delegator_account, delegator_info) in delegator_storage.iter() {
		for delegation in delegator_info.delegations.0.iter() {
			let mut reserve_status =
				Pallet::<T>::get_reserve_status(delegator_account, delegation.liquidity_token);

			reserve_status.staked_unactivated_reserves =
				reserve_status.staked_unactivated_reserves.saturating_add(delegation.amount);

			log::info!(
				target: "mpl",
				"Reserve status after processing delegation of delegator: {:?} on liquidity token: {:?} with amount: {:?} := {:?}",
				delegator_account,
				delegation.liquidity_token,
				delegation.amount,
				reserve_status,
			);

			delegations_processed_count = delegations_processed_count.saturating_add(1u32);

			ReserveStatus::<T>::insert(
				delegator_account,
				delegation.liquidity_token,
				reserve_status,
			);
		}
	}

	for ((account, liquidity_token), amount) in activation_storage.iter() {
		let mut reserve_status = Pallet::<T>::get_reserve_status(account, liquidity_token);

		reserve_status.activated_unstaked_reserves =
			reserve_status.activated_unstaked_reserves.saturating_add(*amount);

		log::info!(
			target: "mpl",
			"Reserve status after processing activation of account: {:?} on liquidity token: {:?} with amount: {:?} := {:?}",
			account,
			liquidity_token,
			amount,
			reserve_status,
		);

		activation_processed_count = activation_processed_count.saturating_add(1u32);

		ReserveStatus::<T>::insert(account, liquidity_token, reserve_status);
	}

	StorageVersion::new(1).put::<P>();

	let reads_to_collect_info = collator_storage
		.len()
		.saturating_add(delegator_storage.len())
		.saturating_add(activation_storage.len());
	let reads_to_modify_state = collators_processed_count
		.saturating_add(delegations_processed_count)
		.saturating_add(activation_processed_count);
	let writes_to_modify_state = collators_processed_count
		.saturating_add(delegations_processed_count)
		.saturating_add(activation_processed_count);

	T::DbWeight::get().reads_writes(
		reads_to_collect_info.saturating_add(reads_to_modify_state as usize) as Weight + 1,
		writes_to_modify_state as Weight + 1,
	)
}

#[cfg(feature = "try-runtime")]
pub fn migrate_from_v0_post_runtime_upgrade<T: Config, P: GetStorageVersion + PalletInfoAccess>(
) -> Result<(), &'static str> {
	// Check consistency of xyk storage, staking storage and orml reserves

	let user_reserve_info: BTreeMap<(T::AccountId, TokenId), (Balance, Balance, Balance)> =
		Pallet::<T>::get_temp_storage("user_reserve_info").expect("temp storage was set");

	for ((account, liquidity_token), (xyk_reserve, staking_reserve, total_reserve)) in
		user_reserve_info.iter()
	{
		assert_eq!(xyk_reserve.saturating_add(*staking_reserve), *total_reserve);
		assert_eq!(
			T::Tokens::reserved_balance({ *liquidity_token }.into(), account).into(),
			*total_reserve
		);
		let reserve_status = Pallet::<T>::get_reserve_status(account, liquidity_token);
		assert!(reserve_status.staked_and_activated_reserves.is_zero());
		assert_eq!(reserve_status.activated_unstaked_reserves, *xyk_reserve);
		assert_eq!(reserve_status.staked_unactivated_reserves, *staking_reserve);
		assert!(reserve_status.unspent_reserves.is_zero());
		assert!(reserve_status.relock_amount.is_zero());
		assert!(Pallet::<T>::get_relock_status(account, liquidity_token).is_empty());
	}

	Ok(())
}
