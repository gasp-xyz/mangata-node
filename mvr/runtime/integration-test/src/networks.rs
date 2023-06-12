use cumulus_primitives_core::{Junction, Parachain, Parent};
use frame_support::{traits::GenesisBuild, weights::Weight};
use polkadot_primitives::v4::{BlockNumber, MAX_CODE_SIZE, MAX_POV_SIZE};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_runtime::traits::AccountIdConversion;
use xcm_executor::traits::Convert;

pub const ALICE_RAW: [u8; 32] = [4u8; 32];
pub const BOB_RAW: [u8; 32] = [5u8; 32];
pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new(ALICE_RAW);
pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new(BOB_RAW);

pub const RELAY_ASSET_ID: u32 = 4_u32;
pub const INITIAL_BALANCE: u128 = 100 * unit(12);

pub type Balance = u128;

pub const fn unit(decimals: u32) -> Balance {
	10u128.saturating_pow(decimals)
}

pub fn cent(decimals: u32) -> Balance {
	unit(decimals) / 100
}

pub fn millicent(decimals: u32) -> Balance {
	cent(decimals) / 1000
}

pub fn microcent(decimals: u32) -> Balance {
	millicent(decimals) / 1000
}

use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

decl_test_relay_chain! {
	pub struct PolkadotRelay {
		Runtime = polkadot_runtime::Runtime,
		XcmConfig = polkadot_runtime::xcm_config::XcmConfig,
		new_ext = polkadot_ext(),
	}
}

decl_test_parachain! {
	pub struct Mangata {
		Runtime = mangata_polkadot_runtime::Runtime,
		RuntimeOrigin = mangata_polkadot_runtime::RuntimeOrigin,
		XcmpMessageHandler = mangata_polkadot_runtime::XcmpQueue,
		DmpMessageHandler = mangata_polkadot_runtime::DmpQueue,
		new_ext = para_ext(2110),
	}
}

decl_test_parachain! {
	pub struct Sibling {
		Runtime = mangata_polkadot_runtime::Runtime,
		RuntimeOrigin = mangata_polkadot_runtime::RuntimeOrigin,
		XcmpMessageHandler = mangata_polkadot_runtime::XcmpQueue,
		DmpMessageHandler = mangata_polkadot_runtime::DmpQueue,
		new_ext = para_ext(2000),
	}
}

decl_test_network! {
	pub struct TestNet {
		relay_chain = PolkadotRelay,
		parachains = vec![
			(2000, Sibling),
			(2110, Mangata),
		],
	}
}

fn default_parachains_host_configuration() -> HostConfiguration<BlockNumber> {
	HostConfiguration {
		minimum_validation_upgrade_delay: 5,
		validation_upgrade_cooldown: 5u32,
		validation_upgrade_delay: 5,
		code_retention_period: 1200,
		max_code_size: MAX_CODE_SIZE,
		max_pov_size: MAX_POV_SIZE,
		max_head_data_size: 32 * 1024,
		group_rotation_frequency: 20,
		chain_availability_period: 4,
		thread_availability_period: 4,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024 * 1024,
		max_downward_message_size: 1024,
		ump_service_total_weight: Weight::from_parts(4 * 1_000_000_000, 0),
		max_upward_message_size: 50 * 1024,
		max_upward_message_num_per_candidate: 5,
		hrmp_sender_deposit: 0,
		hrmp_recipient_deposit: 0,
		hrmp_channel_max_capacity: 8,
		hrmp_channel_max_total_size: 8 * 1024,
		hrmp_max_parachain_inbound_channels: 4,
		hrmp_max_parathread_inbound_channels: 4,
		hrmp_channel_max_message_size: 1024 * 1024,
		hrmp_max_parachain_outbound_channels: 4,
		hrmp_max_parathread_outbound_channels: 4,
		hrmp_max_message_num_per_candidate: 5,
		dispute_period: 6,
		no_show_slots: 2,
		n_delay_tranches: 25,
		needed_approvals: 2,
		relay_vrf_modulo_samples: 2,
		zeroth_delay_tranche_width: 0,
		..Default::default()
	}
}

pub fn parent_account_id() -> mangata_polkadot_runtime::AccountId {
	let location = (Parent,);
	mangata_polkadot_runtime::xcm_config::LocationToAccountId::convert(location.into()).unwrap()
}

pub fn child_account_id(
	para: u32,
) -> <polkadot_runtime::Runtime as frame_system::Config>::AccountId {
	let location = (Parachain(para),);
	polkadot_runtime::xcm_config::SovereignAccountOf::convert(location.into()).unwrap()
}

pub fn sibling_account_account_id(
	para: u32,
	who: sp_runtime::AccountId32,
) -> mangata_polkadot_runtime::AccountId {
	let location =
		(Parent, Parachain(para), Junction::AccountId32 { network: None, id: who.into() });
	mangata_polkadot_runtime::xcm_config::LocationToAccountId::convert(location.into()).unwrap()
}

pub fn reserve_account(id: u32) -> mangata_polkadot_runtime::AccountId {
	polkadot_parachain::primitives::Sibling::from(id).into_account_truncating()
}

pub fn polkadot_ext() -> sp_io::TestExternalities {
	use polkadot_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, 1_000_000_000_000 * unit(12)),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
		config: default_parachains_host_configuration(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&pallet_xcm::GenesisConfig { safe_xcm_version: Some(3) },
		&mut t,
	)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn para_ext(parachain_id: u32) -> sp_io::TestExternalities {
	use mangata_polkadot_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	orml_tokens::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, 0, 100 * unit(18)),
			(ALICE, 1, 0),
			(ALICE, 2, 0),
			(ALICE, 3, 0),
			(ALICE, mangata_polkadot_runtime::DOTTokenId::get(), 1_000_000_000 * unit(12)),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	<parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&parachain_info::GenesisConfig { parachain_id: parachain_id.into() },
		&mut t,
	)
	.unwrap();

	<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&pallet_xcm::GenesisConfig { safe_xcm_version: Some(3) },
		&mut t,
	)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
