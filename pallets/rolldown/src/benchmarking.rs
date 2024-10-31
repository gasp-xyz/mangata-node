//! Rolldown pallet benchmarking.

use super::*;
use crate::{messages::L1UpdateRequest, Pallet as Rolldown};
use frame_benchmarking::{v2::*, whitelisted_caller};
use frame_support::{
	assert_ok,
	dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
	traits::OriginTrait,
};
use frame_system::RawOrigin;
use hex_literal::hex;
use sp_core::{Get, H256};
use sp_std::{marker::PhantomData, prelude::*};

// TODO
// Dedup this
pub(crate) struct L1UpdateBuilder(Option<u128>, Vec<L1UpdateRequest>);

impl L1UpdateBuilder {
	pub fn new() -> Self {
		Self(None, Default::default())
	}

	pub fn with_offset(mut self, offset: u128) -> Self {
		self.0 = Some(offset);
		self
	}

	pub fn with_requests(mut self, requests: Vec<L1UpdateRequest>) -> Self {
		self.1 = requests;
		self
	}

	pub fn build(self) -> messages::L1Update {
		let mut update = messages::L1Update::default();

		for (id, r) in self.1.into_iter().enumerate() {
			let rid = if let Some(offset) = self.0 { (id as u128) + offset } else { r.id() };
			match r {
				L1UpdateRequest::Deposit(mut d) => {
					d.requestId.id = rid;
					update.pendingDeposits.push(d);
				},
				L1UpdateRequest::CancelResolution(mut c) => {
					c.requestId.id = rid;
					update.pendingCancelResolutions.push(c);
				},
			}
		}
		update
	}
}

impl Default for L1UpdateBuilder {
	fn default() -> Self {
		Self(Some(1u128), Default::default())
	}
}

#[benchmarks(where T::AccountId: From<[u8;20]>, BalanceOf<T>: From<u128>, CurrencyIdOf<T>: From<u32>, <T as pallet_sequencer_staking::Config>::ChainId: From<crate::messages::Chain>)]
mod benchmarks {
	use super::*;

	const MANUAL_BATCH_EXTRA_FEE: u128 = 700u128;

	const TOKEN_ID: u32 = 1u32; // ETH token

	const WITHDRAWAL_AMOUNT: u128 = 1000u128;
	const MINT_AMOUNT: u128 = 1_000_000_000__000_000_000_000_000_000u128;
	const FERRY_TIP_NONE: Option<u128> = None;
	const FERRY_TIP: u128 = 101u128;
	const STAKE_AMOUNT: u128 = 1_000_000__000_000_000_000_000_000u128; // If SlashFineAmount in seq staking is increased this may also need to be
	const DEPOSIT_AMOUNT: u128 = 1_000u128;

	const ETH_RECIPIENT_ACCOUNT: [u8; 20] = hex!("0000000000000000000000000000000000000004");
	const DUMMY_TOKEN_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000030");

	const FERRY_ACCOUNT: [u8; 20] = hex!("0000000000000000000000000000000000000040");
	const USER_ACCOUNT: [u8; 20] = hex!("0000000000000000000000000000000000000005");
	const SEQUENCER_ACCOUNT: [u8; 20] = hex!("0000000000000000000000000000000000000010");
	const SEQUENCER_ACCOUNT_2: [u8; 20] = hex!("0000000000000000000000000000000000000011");
	const SEQUENCER_ALIAS_ACCOUNT: [u8; 20] = hex!("0000000000000000000000000000000000000020");

	const FIRST_SCHEDULED_UPDATE_ID: u128 = 1u128;
	const FIRST_BATCH_ID: u128 = 1u128;
	const FIRST_REQUEST_ID: u128 = 1u128;
	const DUMMY_REQUEST_ID: u128 = 77u128;

	trait ToBalance {
		fn to_balance<T: Config>(self) -> BalanceOf<T>;
	}

	impl ToBalance for u128 {
		fn to_balance<T: Config>(self) -> BalanceOf<T> {
			self.try_into().ok().expect("u128 should fit into Balance type")
		}
	}

	fn setup_whitelisted_account<T: Config>() -> Result<T::AccountId, DispatchErrorWithPostInfo> {
		let caller: T::AccountId = whitelisted_caller();
		T::Tokens::mint(T::NativeCurrencyId::get(), &caller, MINT_AMOUNT.to_balance::<T>())?;
		Ok(caller)
	}

	fn setup_account<T: Config>(who: T::AccountId) -> DispatchResultWithPostInfo
	where
		T::AccountId: From<[u8; 20]>,
	{
		T::Tokens::mint(T::NativeCurrencyId::get(), &who, MINT_AMOUNT.to_balance::<T>())?;
		T::Tokens::mint(TOKEN_ID.into(), &who, MINT_AMOUNT.to_balance::<T>())?;
		Ok(().into())
	}

	fn setup_and_do_withdrawal<T: Config>(who: T::AccountId) -> DispatchResultWithPostInfo
	where
		T::AccountId: From<[u8; 20]>,
	{
		T::Tokens::mint(TOKEN_ID.into(), &who, MINT_AMOUNT.to_balance::<T>())?;
		let (l1_aset_chain, l1_asset_address) =
			match T::AssetRegistryProvider::get_asset_l1_id(TOKEN_ID.into())
				.ok_or(Error::<T>::L1AssetNotFound)?
			{
				L1Asset::Ethereum(v) => (crate::messages::Chain::Ethereum, v),
				L1Asset::Arbitrum(v) => (crate::messages::Chain::Arbitrum, v),
			};
		Rolldown::<T>::withdraw(
			RawOrigin::Signed(who).into(),
			l1_aset_chain.into(),
			ETH_RECIPIENT_ACCOUNT,
			l1_asset_address,
			WITHDRAWAL_AMOUNT,
			FERRY_TIP_NONE,
		)?;
		Ok(().into())
	}

	fn setup_sequencer<T: Config>(
		sequencer: T::AccountId,
		sequencer_alias: Option<T::AccountId>,
		should_be_active: bool,
		should_be_selected: bool,
	) -> DispatchResultWithPostInfo
	where
		<T as pallet_sequencer_staking::Config>::ChainId: From<messages::Chain>,
		T::AccountId: From<[u8; 20]>,
	{
		T::Tokens::mint(TOKEN_ID.into(), &sequencer, MINT_AMOUNT.to_balance::<T>())?;
		T::Tokens::mint(T::NativeCurrencyId::get(), &sequencer, MINT_AMOUNT.to_balance::<T>())?;
		if let Some(ref sequencer_alias) = sequencer_alias {
			T::Tokens::mint(TOKEN_ID.into(), &sequencer_alias, MINT_AMOUNT.to_balance::<T>())?;
			T::Tokens::mint(
				T::NativeCurrencyId::get(),
				&sequencer_alias,
				MINT_AMOUNT.to_balance::<T>(),
			)?;
		}
		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		pallet_sequencer_staking::Pallet::<T>::provide_sequencer_stake(
			RawOrigin::Signed(sequencer.clone()).into(),
			l1_aset_chain.into(),
			STAKE_AMOUNT.try_into().ok().expect("u128 should fit into Balance type"),
			sequencer_alias,
			pallet_sequencer_staking::StakeAction::StakeOnly,
		)?;

		if should_be_active {
			pallet_sequencer_staking::ActiveSequencers::<T>::try_mutate(|active_sequencers| {
				active_sequencers
					.entry(l1_aset_chain.into())
					.or_default()
					.try_push(sequencer.clone())
					.expect("ActiveSequencers push works");
				Ok::<(), DispatchError>(())
			})?;
			Rolldown::<T>::new_sequencer_active(l1_aset_chain.into(), &sequencer)
		}

		if should_be_selected {
			pallet_sequencer_staking::SelectedSequencer::<T>::mutate(|selected| {
				selected.insert(l1_aset_chain.into(), sequencer.into())
			});
		}

		Ok(().into())
	}

	fn get_chain_and_address_for_asset_id<T: Config>(
		asset_id: CurrencyIdOf<T>,
	) -> Result<(crate::messages::Chain, [u8; 20]), DispatchErrorWithPostInfo> {
		let (l1_aset_chain, l1_asset_address) =
			match T::AssetRegistryProvider::get_asset_l1_id(TOKEN_ID.into())
				.ok_or(Error::<T>::L1AssetNotFound)?
			{
				L1Asset::Ethereum(v) => (crate::messages::Chain::Ethereum, v),
				L1Asset::Arbitrum(v) => (crate::messages::Chain::Arbitrum, v),
			};
		Ok((l1_aset_chain, l1_asset_address))
	}

	#[benchmark]
	fn set_manual_batch_extra_fee() -> Result<(), BenchmarkError> {
		assert_eq!(ManualBatchExtraFee::<T>::get(), BalanceOf::<T>::zero());

		#[extrinsic_call]
		_(RawOrigin::Root, MANUAL_BATCH_EXTRA_FEE.to_balance::<T>());

		assert_eq!(ManualBatchExtraFee::<T>::get(), MANUAL_BATCH_EXTRA_FEE.to_balance::<T>());

		Ok(())
	}

	// Note that even though r/w on L2RequestsBatchLast would count as 1, since the value is unbounded each r/w might be far more expensive than expected
	#[benchmark]
	fn create_batch() -> Result<(), BenchmarkError> {
		setup_account::<T>(USER_ACCOUNT.into())?;
		setup_and_do_withdrawal::<T>(USER_ACCOUNT.into())?;
		setup_sequencer::<T>(
			SEQUENCER_ACCOUNT.into(),
			Some(SEQUENCER_ALIAS_ACCOUNT.into()),
			false,
			false,
		)?;

		ManualBatchExtraFee::<T>::set(MANUAL_BATCH_EXTRA_FEE.to_balance::<T>());

		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		assert!(
			L2RequestsBatch::<T>::get::<(<T as Config>::ChainId, _)>((
				l1_aset_chain.into(),
				FIRST_BATCH_ID
			))
			.is_none(),
			"BEFORE L2RequestsBatch {:?} chain {:?} batch should be uninit",
			l1_aset_chain,
			FIRST_BATCH_ID
		);
		assert!(
			L2RequestsBatchLast::<T>::get().get(&l1_aset_chain.into()).is_none(),
			"BEFORE L2RequestsBatchLast {:?} chain should be uninit",
			l1_aset_chain
		);

		let acc: T::AccountId = SEQUENCER_ALIAS_ACCOUNT.into();
		whitelist_account!(acc);
		#[extrinsic_call]
		_(
			RawOrigin::Signed(SEQUENCER_ALIAS_ACCOUNT.into()),
			l1_aset_chain.into(),
			Some(SEQUENCER_ACCOUNT.into()),
		);

		assert!(
			L2RequestsBatch::<T>::get::<(<T as Config>::ChainId, _)>((
				l1_aset_chain.into(),
				FIRST_BATCH_ID
			))
			.is_some(),
			"AFTER L2RequestsBatch {:?} chain {:?} batch should be init",
			l1_aset_chain,
			FIRST_BATCH_ID
		);
		assert!(
			L2RequestsBatchLast::<T>::get().get(&l1_aset_chain.into()).is_some(),
			"BEFORE L2RequestsBatchLast {:?} chain should be init",
			l1_aset_chain
		);

		Ok(())
	}

	// Note that even though r/w on L2RequestsBatchLast would count as 1, since the value is unbounded each r/w might be far more expensive than expected
	#[benchmark]
	fn force_create_batch() -> Result<(), BenchmarkError> {
		setup_account::<T>(USER_ACCOUNT.into())?;
		setup_and_do_withdrawal::<T>(USER_ACCOUNT.into())?;
		setup_sequencer::<T>(
			SEQUENCER_ACCOUNT.into(),
			Some(SEQUENCER_ALIAS_ACCOUNT.into()),
			false,
			false,
		)?;

		ManualBatchExtraFee::<T>::put(MANUAL_BATCH_EXTRA_FEE.to_balance::<T>());

		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		assert!(
			L2RequestsBatch::<T>::get::<(<T as Config>::ChainId, _)>((
				l1_aset_chain.into(),
				FIRST_BATCH_ID
			))
			.is_none(),
			"BEFORE L2RequestsBatch {:?} chain {:?} batch should be uninit",
			l1_aset_chain,
			FIRST_BATCH_ID
		);
		assert!(
			L2RequestsBatchLast::<T>::get().get(&l1_aset_chain.into()).is_none(),
			"BEFORE L2RequestsBatchLast {:?} chain should be uninit",
			l1_aset_chain
		);

		#[extrinsic_call]
		_(RawOrigin::Root, l1_aset_chain.into(), (1u128, 1u128), SEQUENCER_ACCOUNT.into());

		assert!(
			L2RequestsBatch::<T>::get::<(<T as Config>::ChainId, _)>((
				l1_aset_chain.into(),
				FIRST_BATCH_ID
			))
			.is_some(),
			"AFTER L2RequestsBatch {:?} chain {:?} batch should be init",
			l1_aset_chain,
			FIRST_BATCH_ID
		);
		assert!(
			L2RequestsBatchLast::<T>::get().get(&l1_aset_chain.into()).is_some(),
			"BEFORE L2RequestsBatchLast {:?} chain should be init",
			l1_aset_chain
		);

		Ok(())
	}

	#[benchmark]
	fn update_l2_from_l1(x: Linear<2, 200>) -> Result<(), BenchmarkError> {
		setup_sequencer::<T>(
			SEQUENCER_ACCOUNT.into(),
			Some(SEQUENCER_ALIAS_ACCOUNT.into()),
			true,
			true,
		)?;
		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		let x_deposits: usize = (x as usize) / 2;
		let x_cancel_resolution: usize = (x as usize) - x_deposits;
		let update = L1UpdateBuilder::default()
			.with_requests(
				[
					vec![L1UpdateRequest::Deposit(Default::default()); x_deposits],
					vec![
						L1UpdateRequest::CancelResolution(Default::default());
						x_cancel_resolution
					],
				]
				.concat(),
			)
			.build();
		let update_hash = update.abi_encode_hash();

		<frame_system::Pallet<T>>::set_block_number(1u32.into());
		let dispute_period_end = <frame_system::Pallet<T>>::block_number().saturated_into::<u128>() +
			Rolldown::<T>::get_dispute_period();
		assert!(
			PendingSequencerUpdates::<T>::get(dispute_period_end, l1_chain).is_none(),
			"BEFORE PendingSequencerUpdates {:?} dispute_period_end {:?} l1_chain should be uninit",
			dispute_period_end,
			l1_chain
		);

		let acc: T::AccountId = SEQUENCER_ACCOUNT.into();
		whitelist_account!(acc);
		#[extrinsic_call]
		_(RawOrigin::Signed(SEQUENCER_ACCOUNT.into()), update, update_hash);

		assert!(
			PendingSequencerUpdates::<T>::get(dispute_period_end, l1_chain).is_some(),
			"AFTER PendingSequencerUpdates {:?} dispute_period_end {:?} l1_chain should be init",
			dispute_period_end,
			l1_chain
		);
		Ok(())
	}

	#[benchmark]
	fn update_l2_from_l1_unsafe(x: Linear<2, 200>) -> Result<(), BenchmarkError> {
		setup_sequencer::<T>(
			SEQUENCER_ACCOUNT.into(),
			Some(SEQUENCER_ALIAS_ACCOUNT.into()),
			true,
			true,
		)?;
		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		let x_deposits: usize = (x as usize) / 2;
		let x_cancel_resolution: usize = (x as usize) - x_deposits;
		let update = L1UpdateBuilder::default()
			.with_requests(
				[
					vec![L1UpdateRequest::Deposit(Default::default()); x_deposits],
					vec![
						L1UpdateRequest::CancelResolution(Default::default());
						x_cancel_resolution
					],
				]
				.concat(),
			)
			.build();

		<frame_system::Pallet<T>>::set_block_number(1u32.into());
		let dispute_period_end = <frame_system::Pallet<T>>::block_number().saturated_into::<u128>() +
			Rolldown::<T>::get_dispute_period();
		assert!(
			PendingSequencerUpdates::<T>::get(dispute_period_end, l1_chain).is_none(),
			"BEFORE PendingSequencerUpdates {:?} dispute_period_end {:?} l1_chain should be uninit",
			dispute_period_end,
			l1_chain
		);

		let acc: T::AccountId = SEQUENCER_ACCOUNT.into();
		whitelist_account!(acc);
		#[extrinsic_call]
		_(RawOrigin::Signed(SEQUENCER_ACCOUNT.into()), update);

		assert!(
			PendingSequencerUpdates::<T>::get(dispute_period_end, l1_chain).is_some(),
			"AFTER PendingSequencerUpdates {:?} dispute_period_end {:?} l1_chain should be init",
			dispute_period_end,
			l1_chain
		);
		Ok(())
	}

	#[benchmark]
	fn force_update_l2_from_l1(x: Linear<2, 200>) -> Result<(), BenchmarkError> {
		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		let x_deposits: usize = (x as usize) / 2;
		let x_cancel_resolution: usize = (x as usize) - x_deposits;
		let update = L1UpdateBuilder::default()
			.with_requests(
				[
					vec![L1UpdateRequest::Deposit(Default::default()); x_deposits],
					vec![
						L1UpdateRequest::CancelResolution(Default::default());
						x_cancel_resolution
					],
				]
				.concat(),
			)
			.build();

		assert!(
			MaxAcceptedRequestIdOnl2::<T>::get(l1_chain).is_zero(),
			"BEFORE MaxAcceptedRequestIdOnl2 {:?} l1_chain should be zero",
			l1_chain
		);

		#[extrinsic_call]
		_(RawOrigin::Root, update);

		assert!(
			!MaxAcceptedRequestIdOnl2::<T>::get(l1_chain).is_zero(),
			"AFTER MaxAcceptedRequestIdOnl2 {:?} l1_chain should NOT be zero",
			l1_chain
		);
		Ok(())
	}

	#[benchmark]
	fn cancel_requests_from_l1() -> Result<(), BenchmarkError> {
		setup_sequencer::<T>(SEQUENCER_ACCOUNT.into(), None, true, false)?;
		setup_sequencer::<T>(SEQUENCER_ACCOUNT_2.into(), None, true, false)?;
		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		let update = L1UpdateBuilder::default()
			.with_requests(vec![L1UpdateRequest::Deposit(Default::default())])
			.build();

		let sequencer_account: T::AccountId = SEQUENCER_ACCOUNT.into();
		PendingSequencerUpdates::<T>::insert(
			DUMMY_REQUEST_ID,
			l1_chain,
			(sequencer_account, update, H256::default()),
		);

		assert!(
			L2Requests::<T>::get(l1_chain, RequestId::from((Origin::L2, FIRST_REQUEST_ID)))
				.is_none(),
			"BEFORE L2Requests {:?} l1_chain {:?} requestId should be none",
			l1_chain,
			FIRST_REQUEST_ID
		);

		let acc: T::AccountId = SEQUENCER_ACCOUNT_2.into();
		whitelist_account!(acc);
		#[extrinsic_call]
		_(RawOrigin::Signed(SEQUENCER_ACCOUNT_2.into()), l1_aset_chain.into(), DUMMY_REQUEST_ID);

		assert!(
			L2Requests::<T>::get(l1_chain, RequestId::from((Origin::L2, FIRST_REQUEST_ID)))
				.is_some(),
			"AFTER L2Requests {:?} l1_chain {:?} requestId should be some",
			l1_chain,
			FIRST_REQUEST_ID
		);
		Ok(())
	}

	#[benchmark]
	fn force_cancel_requests_from_l1() -> Result<(), BenchmarkError> {
		setup_sequencer::<T>(SEQUENCER_ACCOUNT.into(), None, true, true)?;
		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		let sequencer_account: T::AccountId = SEQUENCER_ACCOUNT.into();
		PendingSequencerUpdates::<T>::insert(
			DUMMY_REQUEST_ID,
			l1_chain,
			(sequencer_account, messages::L1Update::default(), H256::default()),
		);

		assert!(
			T::SequencerStakingProvider::is_selected_sequencer(l1_chain, &SEQUENCER_ACCOUNT.into()),
			"suboptimal code path"
		);
		assert!(
			PendingSequencerUpdates::<T>::get(DUMMY_REQUEST_ID, l1_chain).is_some(),
			"BEFORE PendingSequencerUpdates {:?} requestId {:?} l1_chain should be some",
			DUMMY_REQUEST_ID,
			l1_chain
		);

		#[extrinsic_call]
		_(RawOrigin::Root, l1_aset_chain.into(), DUMMY_REQUEST_ID);

		assert!(
			PendingSequencerUpdates::<T>::get(DUMMY_REQUEST_ID, l1_chain).is_none(),
			"AFTER PendingSequencerUpdates {:?} requestId {:?} l1_chain should be none",
			DUMMY_REQUEST_ID,
			l1_chain
		);
		Ok(())
	}

	#[benchmark]
	fn withdraw() -> Result<(), BenchmarkError> {
		setup_account::<T>(USER_ACCOUNT.into())?;

		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		assert!(
			!<<T as Config>::WithdrawFee>::convert(l1_chain).is_zero(),
			"withdraw fee should not be zero, suboptimal code path"
		);
		assert!(
			L2Requests::<T>::get(l1_chain, RequestId::from((Origin::L2, FIRST_REQUEST_ID)))
				.is_none(),
			"BEFORE L2Requests {:?} l1_chain {:?} requestId should be none",
			l1_chain,
			DUMMY_REQUEST_ID
		);

		let acc: T::AccountId = USER_ACCOUNT.into();
		whitelist_account!(acc);
		#[extrinsic_call]
		_(
			RawOrigin::Signed(USER_ACCOUNT.into()),
			l1_chain.into(),
			ETH_RECIPIENT_ACCOUNT,
			l1_asset_address,
			WITHDRAWAL_AMOUNT,
			FERRY_TIP_NONE,
		);

		assert!(
			L2Requests::<T>::get(l1_chain, RequestId::from((Origin::L2, FIRST_REQUEST_ID)))
				.is_some(),
			"AFTER L2Requests {:?} l1_chain {:?} requestId should be some",
			l1_chain,
			DUMMY_REQUEST_ID
		);
		Ok(())
	}

	#[benchmark]
	fn refund_failed_deposit() -> Result<(), BenchmarkError> {
		setup_account::<T>(USER_ACCOUNT.into())?;

		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		FailedL1Deposits::<T>::insert(
			(l1_chain, DUMMY_REQUEST_ID),
			(<T::AccountId as From<[u8; 20]>>::from(USER_ACCOUNT), H256::default()),
		);

		assert!(
			L2Requests::<T>::get(l1_chain, RequestId::from((Origin::L2, FIRST_REQUEST_ID)))
				.is_none(),
			"BEFORE L2Requests {:?} l1_chain {:?} requestId should be none",
			l1_chain,
			DUMMY_REQUEST_ID
		);

		let acc: T::AccountId = USER_ACCOUNT.into();
		whitelist_account!(acc);
		#[extrinsic_call]
		_(RawOrigin::Signed(USER_ACCOUNT.into()), l1_chain, DUMMY_REQUEST_ID);

		assert!(
			L2Requests::<T>::get(l1_chain, RequestId::from((Origin::L2, FIRST_REQUEST_ID)))
				.is_some(),
			"AFTER L2Requests {:?} l1_chain {:?} requestId should be some",
			l1_chain,
			DUMMY_REQUEST_ID
		);
		Ok(())
	}

	#[benchmark]
	fn ferry_deposit() -> Result<(), BenchmarkError> {
		setup_account::<T>(FERRY_ACCOUNT.into())?;

		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		let deposit = messages::Deposit {
			depositRecipient: ETH_RECIPIENT_ACCOUNT.into(),
			requestId: RequestId::from((Origin::L1, DUMMY_REQUEST_ID)),
			tokenAddress: l1_asset_address,
			amount: DEPOSIT_AMOUNT.into(),
			timeStamp: Default::default(),
			ferryTip: FERRY_TIP.into(),
		};
		let deposit_hash = deposit.abi_encode_hash();

		assert!(
			FerriedDeposits::<T>::get((l1_chain, deposit_hash)).is_none(),
			"BEFORE FerriedDeposits {:?} l1_chain {:?} deposit_hash should be none",
			l1_chain,
			deposit_hash
		);

		let acc: T::AccountId = FERRY_ACCOUNT.into();
		whitelist_account!(acc);
		#[extrinsic_call]
		_(
			RawOrigin::Signed(FERRY_ACCOUNT.into()),
			l1_chain,
			RequestId::from((Origin::L1, DUMMY_REQUEST_ID)),
			ETH_RECIPIENT_ACCOUNT.into(),
			l1_asset_address,
			DEPOSIT_AMOUNT.into(),
			Default::default(),
			FERRY_TIP.into(),
			deposit_hash,
		);

		assert!(
			FerriedDeposits::<T>::get((l1_chain, deposit_hash)).is_some(),
			"AFTER FerriedDeposits {:?} l1_chain {:?} deposit_hash should be some",
			l1_chain,
			deposit_hash
		);
		Ok(())
	}

	#[benchmark]
	fn ferry_deposit_unsafe() -> Result<(), BenchmarkError> {
		setup_account::<T>(FERRY_ACCOUNT.into())?;

		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();
		let deposit = messages::Deposit {
			depositRecipient: ETH_RECIPIENT_ACCOUNT.into(),
			requestId: RequestId::from((Origin::L1, DUMMY_REQUEST_ID)),
			tokenAddress: l1_asset_address,
			amount: DEPOSIT_AMOUNT.into(),
			timeStamp: Default::default(),
			ferryTip: FERRY_TIP.into(),
		};
		let deposit_hash = deposit.abi_encode_hash();

		assert!(
			FerriedDeposits::<T>::get((l1_chain, deposit_hash)).is_none(),
			"BEFORE FerriedDeposits {:?} l1_chain {:?} deposit_hash should be none",
			l1_chain,
			deposit_hash
		);

		let acc: T::AccountId = FERRY_ACCOUNT.into();
		whitelist_account!(acc);
		#[extrinsic_call]
		_(
			RawOrigin::Signed(FERRY_ACCOUNT.into()),
			l1_chain,
			RequestId::from((Origin::L1, DUMMY_REQUEST_ID)),
			ETH_RECIPIENT_ACCOUNT.into(),
			l1_asset_address,
			DEPOSIT_AMOUNT.into(),
			Default::default(),
			FERRY_TIP.into(),
		);

		assert!(
			FerriedDeposits::<T>::get((l1_chain, deposit_hash)).is_some(),
			"AFTER FerriedDeposits {:?} l1_chain {:?} deposit_hash should be some",
			l1_chain,
			deposit_hash
		);
		Ok(())
	}

	#[benchmark]
	fn process_deposit() -> Result<(), BenchmarkError> {
		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();
		let deposit = messages::Deposit {
			depositRecipient: ETH_RECIPIENT_ACCOUNT.into(),
			requestId: RequestId::from((Origin::L1, DUMMY_REQUEST_ID)),
			tokenAddress: l1_asset_address,
			amount: DEPOSIT_AMOUNT.into(),
			timeStamp: Default::default(),
			ferryTip: FERRY_TIP.into(),
		};
		let deposit_hash = deposit.abi_encode_hash();

		FerriedDeposits::<T>::insert(
			(l1_chain, deposit_hash),
			&<T::AccountId as From<[u8; 20]>>::from(ETH_RECIPIENT_ACCOUNT),
		);

		let balance_before =
			T::Tokens::free_balance(TOKEN_ID.into(), &ETH_RECIPIENT_ACCOUNT.into());

		#[block]
		{
			Rolldown::<T>::process_deposit(l1_chain, &deposit);
		}

		let balance_after = T::Tokens::free_balance(TOKEN_ID.into(), &ETH_RECIPIENT_ACCOUNT.into());
		assert_eq!(
			balance_after,
			balance_before + DEPOSIT_AMOUNT.into(),
			"Balance should increase by {:?}",
			DEPOSIT_AMOUNT
		);
		Ok(())
	}

	// This benchmark doesn't consider the size of ActiveSequencers and SequencerRights L1 BtreeMap Entry
	// This should be fine since the impact should be low...
	// Also there is no way reasonable way to acquire that value when this function is actually called in the on_initiliaze hook (we don't wanna read these values everytime)...
	#[benchmark]
	fn process_cancel_resolution() -> Result<(), BenchmarkError> {
		setup_sequencer::<T>(SEQUENCER_ACCOUNT.into(), None, true, true)?;
		setup_sequencer::<T>(SEQUENCER_ACCOUNT_2.into(), None, true, false)?;

		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		let cancel_request = messages::Cancel {
			requestId: RequestId::from((Origin::L2, Default::default())),
			updater: SEQUENCER_ACCOUNT.into(),
			canceler: SEQUENCER_ACCOUNT_2.into(),
			range: messages::Range { start: u128::default(), end: u128::default() },
			hash: H256::default(),
		};

		L2Requests::<T>::insert(
			l1_chain,
			RequestId::from((Origin::L2, DUMMY_REQUEST_ID)),
			(L2Request::Cancel(cancel_request.clone()), cancel_request.abi_encode_hash()),
		);
		AwaitingCancelResolution::<T>::mutate(l1_chain, |v| {
			v.insert((SEQUENCER_ACCOUNT.into(), DUMMY_REQUEST_ID, DisputeRole::Submitter))
		});
		AwaitingCancelResolution::<T>::mutate(l1_chain, |v| {
			v.insert((SEQUENCER_ACCOUNT_2.into(), DUMMY_REQUEST_ID, DisputeRole::Canceler))
		});

		let cancel_resolution = messages::CancelResolution {
			requestId: RequestId::from((Origin::L1, Default::default())),
			l2RequestId: DUMMY_REQUEST_ID,
			cancelJustified: true,
			timeStamp: Default::default(),
		};

		let slash_fine_amount = pallet_sequencer_staking::SlashFineAmount::<T>::get();
		pallet_sequencer_staking::SequencerStake::<T>::insert(
			(
				<T::AccountId as From<[u8; 20]>>::from(SEQUENCER_ACCOUNT),
				Into::<<T as pallet_sequencer_staking::Config>::ChainId>::into(l1_aset_chain),
			),
			slash_fine_amount,
		);
		assert!(
			pallet_sequencer_staking::SequencerStake::<T>::get((
				<T::AccountId as From<[u8; 20]>>::from(SEQUENCER_ACCOUNT),
				Into::<<T as pallet_sequencer_staking::Config>::ChainId>::into(l1_aset_chain)
			)) < pallet_sequencer_staking::MinimalStakeAmount::<T>::get() + slash_fine_amount,
			"suboptimal code path"
		);

		assert!(
			T::SequencerStakingProvider::is_active_sequencer(l1_chain, &SEQUENCER_ACCOUNT.into()),
			"suboptimal code path"
		);
		assert!(
			T::SequencerStakingProvider::is_active_sequencer(l1_chain, &SEQUENCER_ACCOUNT_2.into()),
			"suboptimal code path"
		);
		assert!(
			T::SequencerStakingProvider::is_selected_sequencer(l1_chain, &SEQUENCER_ACCOUNT.into()),
			"suboptimal code path"
		);

		assert!(
			AwaitingCancelResolution::<T>::get(l1_chain)
				.get(&(SEQUENCER_ACCOUNT.into(), DUMMY_REQUEST_ID, DisputeRole::Submitter))
				.is_some(),
			"BEFORE AwaitingCancelResolution {:?} l1_chain {:?} sequencer should be some",
			l1_chain,
			SEQUENCER_ACCOUNT
		);
		assert!(
			AwaitingCancelResolution::<T>::get(l1_chain)
				.get(&(SEQUENCER_ACCOUNT_2.into(), DUMMY_REQUEST_ID, DisputeRole::Canceler))
				.is_some(),
			"BEFORE AwaitingCancelResolution {:?} l1_chain {:?} sequencer should be some",
			l1_chain,
			SEQUENCER_ACCOUNT_2
		);

		let balance_before =
			T::Tokens::total_balance(T::NativeCurrencyId::get(), &SEQUENCER_ACCOUNT.into());

		#[block]
		{
			Rolldown::<T>::process_cancel_resolution(l1_chain, &cancel_resolution);
		}

		let balance_after =
			T::Tokens::total_balance(T::NativeCurrencyId::get(), &SEQUENCER_ACCOUNT.into());
		assert_eq!(
			balance_after +
				TryInto::<u128>::try_into(slash_fine_amount)
					.ok()
					.expect("should work")
					.to_balance::<T>(),
			balance_before,
			"Balance should decrease by {:?}",
			slash_fine_amount
		);
		assert!(
			AwaitingCancelResolution::<T>::get(l1_chain)
				.get(&(SEQUENCER_ACCOUNT.into(), DUMMY_REQUEST_ID, DisputeRole::Submitter))
				.is_none(),
			"AFTER AwaitingCancelResolution {:?} l1_chain {:?} sequencer should be none",
			l1_chain,
			SEQUENCER_ACCOUNT
		);
		assert!(
			AwaitingCancelResolution::<T>::get(l1_chain)
				.get(&(SEQUENCER_ACCOUNT_2.into(), DUMMY_REQUEST_ID, DisputeRole::Canceler))
				.is_none(),
			"AFTER AwaitingCancelResolution {:?} l1_chain {:?} sequencer should be none",
			l1_chain,
			SEQUENCER_ACCOUNT_2
		);

		Ok(())
	}

	#[benchmark]
	fn schedule_requests(x: Linear<2, 200>) -> Result<(), BenchmarkError> {
		let (l1_aset_chain, l1_asset_address) =
			get_chain_and_address_for_asset_id::<T>(TOKEN_ID.into())?;
		let l1_chain: <T as Config>::ChainId = l1_aset_chain.into();

		let x_deposits: usize = (x as usize) / 2;
		let x_cancel_resolution: usize = (x as usize) - x_deposits;
		let mut update = L1UpdateBuilder::default()
			.with_requests(
				[
					vec![L1UpdateRequest::Deposit(Default::default()); x_deposits],
					vec![
						L1UpdateRequest::CancelResolution(Default::default());
						x_cancel_resolution
					],
				]
				.concat(),
			)
			.build();

		assert!(
			MaxAcceptedRequestIdOnl2::<T>::get(l1_chain).is_zero(),
			"BEFORE MaxAcceptedRequestIdOnl2 {:?} chain should be zero",
			l1_chain
		);
		assert!(
			UpdatesExecutionQueue::<T>::get(FIRST_SCHEDULED_UPDATE_ID).is_none(),
			"BEFORE UpdatesExecutionQueue {:?} scheduled update id should be none",
			FIRST_SCHEDULED_UPDATE_ID
		);

		#[block]
		{
			Rolldown::<T>::schedule_requests(BlockNumberFor::<T>::default(), l1_chain, update);
		}

		assert!(
			!MaxAcceptedRequestIdOnl2::<T>::get(l1_chain).is_zero(),
			"AFTER MaxAcceptedRequestIdOnl2 {:?} chain should NOT be zero",
			l1_chain
		);
		assert!(
			UpdatesExecutionQueue::<T>::get(FIRST_SCHEDULED_UPDATE_ID).is_some(),
			"AFTER UpdatesExecutionQueue {:?} scheduled update id should be some",
			FIRST_SCHEDULED_UPDATE_ID
		);
		Ok(())
	}
}
