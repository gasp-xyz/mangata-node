use super::*;
use frame_support::{
	traits::{Get, GetStorageVersion, PalletInfoAccess, StorageVersion},
	weights::Weight,
};
use parachain_staking::{CollatorCandidate, Delegator};
use frame_support::migration::storage_key_iter;

/// Migrate the pallet storage to v1.
pub fn migrate_to_v1<T: Config, P: GetStorageVersion + PalletInfoAccess>(
) -> frame_support::weights::Weight {
	let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();

    if on_chain_storage_version != 0{ 
        log::info!(
            target: "mpl",
            "Attempted to apply xyk-staking-mpl consistency migration to mpl but failed because storage version is {:?}, and not 0",
            on_chain_storage_version,
        );
        return T::DbWeight::get().reads(1)
    }

    // Apply storage migration from StorageVersion 0 to 1

    //////////////////////////////////////////////////////

    let collator_storage = storage_key_iter::<T::AccountId, CollatorCandidate<T::AccountId>, Twox64Concat>(b"ParachainStaking", b"CandidateState")
        .collect::<Vec<_>>();

    let delegator_storage = storage_key_iter::<T::AccountId, Delegator<T::AccountId>, Twox64Concat>(b"ParachainStaking", b"DelegatorState")
        .collect::<Vec<_>>();

    let activation_storage = storage_key_iter::<(T::AccountId, TokenId), Balance, Twox64Concat>(b"Xyk", b"LiquidityMiningActiveUser")
        .collect::<Vec<_>>();

    // Now we have all the info in memory

    let mut collators_processed_count = u32::zero();

    let mut delegations_processed_count = u32::zero(); 

    let mut activation_processed_count = u32::zero(); 

    for (collator_account, collator_info) in collator_storage.iter(){
        let mut reserve_status = Pallet::<T>::get_reserve_status(collator_account, collator_info.liquidity_token);
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

    for (delegator_account, delegator_info) in delegator_storage.iter(){
        for delegation in delegator_info.delegations.0.iter(){

            let mut reserve_status = Pallet::<T>::get_reserve_status(delegator_account, delegation.liquidity_token);

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

            ReserveStatus::<T>::insert(delegator_account, delegation.liquidity_token, reserve_status);
        }
    }

    for ( (account, liquidity_token), amount) in activation_storage.iter(){
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

    let reads_to_collect_info = collator_storage.len().saturating_add(delegator_storage.len()).saturating_add(activation_storage.len());
    let reads_to_modify_state = collators_processed_count.saturating_add(delegations_processed_count).saturating_add(activation_processed_count);
    let writes_to_modify_state = collators_processed_count.saturating_add(delegations_processed_count).saturating_add(activation_processed_count);

    T::DbWeight::get().reads_writes(reads_to_collect_info.saturating_add(reads_to_modify_state as usize) as Weight + 1, writes_to_modify_state as Weight + 1)
}
