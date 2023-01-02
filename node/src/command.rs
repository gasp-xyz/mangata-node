use crate::{
	chain_spec,
	cli::{Cli, RelayChainCli, Subcommand},
	command_helper::{inherent_benchmark_data, BenchmarkExtrinsicBuilder},
	service,
	service::new_partial,
};
use codec::Encode;
use cumulus_client_cli::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use log::info;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::{
	config::{BasePath, PrometheusConfig},
	PartialComponents,
};

use sp_core::hexdisplay::HexDisplay;

use sp_runtime::traits::{AccountIdConversion, Block as BlockT};
use std::{convert::TryInto, io::Write, net::SocketAddr, sync::Arc, time::Duration};

fn load_spec(id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match id {
		#[cfg(feature = "mangata-kusama")]
		"dev" => Box::new(chain_spec::mangata_kusama::development_config()),
		#[cfg(feature = "mangata-kusama")]
		"" | "local" => Box::new(chain_spec::mangata_kusama::local_config()),
		#[cfg(feature = "mangata-kusama")]
		"kusama-mainnet" => Box::new(chain_spec::mangata_kusama::kusama_mainnet_config()),

		#[cfg(feature = "mangata-rococo")]
		"public-testnet" => Box::new(chain_spec::mangata_rococo::public_testnet_config()),

		#[cfg(feature = "mangata-rococo")]
		"mangata-rococo-local-testnet" =>
			Box::new(chain_spec::mangata_rococo::mangata_rococo_local_config()),

		path => {
			let path = std::path::PathBuf::from(path);

			let chain_spec =
				Box::new(crate::chain_spec::DummyChainSpec::from_json_file(path.clone())?)
					as Box<dyn service::ChainSpec>;

			if chain_spec.is_mangata_kusama() {
				#[cfg(feature = "mangata-kusama")]
				{
					Box::new(chain_spec::mangata_kusama::ChainSpec::from_json_file(path)?)
				}

				#[cfg(not(feature = "mangata-kusama"))]
				return Err(service::MANGATA_KUSAMA_RUNTIME_NOT_AVAILABLE.into())
			} else if chain_spec.is_mangata_rococo() {
				#[cfg(feature = "mangata-rococo")]
				{
					Box::new(chain_spec::mangata_rococo::ChainSpec::from_json_file(path)?)
				}
				#[cfg(not(feature = "mangata-rococo"))]
				return Err(service::MANGATA_ROCOCO_RUNTIME_NOT_AVAILABLE.into())
			} else {
				return Err("The id of the chainspec does not match the enabled feature".into())
			}
		},
	})
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Mangata Parachain Collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Mangata Parachain Collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relay chain node.\n\n\
		parachain-collator <parachain-args> -- <relay-chain-args>"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/cumulus/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2020
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		match spec {
			#[cfg(feature = "mangata-kusama")]
			spec if spec.is_mangata_kusama() => return &service::mangata_kusama_runtime::VERSION,
			#[cfg(feature = "mangata-rococo")]
			spec if spec.is_mangata_rococo() => return &service::mangata_rococo_runtime::VERSION,
			_ => panic!("invalid chain spec"),
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Mangata Parachain Collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Mangata Parachain Collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relay chain node.\n\n\
		parachain-collator <parachain-args> -- <relay-chain-args>"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/cumulus/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2020
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

#[allow(clippy::borrowed_box)]
fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

/// Can be called for a `Configuration` to check if it is a configuration for
/// the `Mangata` network.
pub trait IdentifyVariant {
	/// Returns `true` if this is a configuration for the `Moonbase` network.
	fn is_mangata_kusama(&self) -> bool;

	/// Returns `true` if this is a configuration for the `Moonbeam` network.
	fn is_mangata_rococo(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_mangata_kusama(&self) -> bool {
		!(self.id().starts_with("mangata_public_testnet") ||
			self.id().starts_with("mangata_rococo_local"))
	}

	fn is_mangata_rococo(&self) -> bool {
		self.id().starts_with("mangata_public_testnet") ||
			self.id().starts_with("mangata_rococo_local")
	}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);

				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		},
		Some(Subcommand::Key(cmd)) => Ok(cmd.run(&cli)?),
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, backend, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, backend, None), task_manager))
			})
		},
		Some(Subcommand::ExportGenesisState(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let spec = load_spec(&params.shared_params.chain.clone().unwrap_or_default())?;

			let output_buf = match spec {
				#[cfg(feature = "mangata-kusama")]
				spec if spec.is_mangata_kusama() => {
					let state_version = Cli::native_runtime_version(&spec).state_version();
					let block: service::mangata_kusama_runtime::Block =
						generate_genesis_block(&*spec, state_version)?;
					let raw_header = block.header().encode();
					let output_buf = if params.raw {
						raw_header
					} else {
						format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
					};
					output_buf
				},
				#[cfg(feature = "mangata-rococo")]
				spec if spec.is_mangata_rococo() => {
					let state_version = Cli::native_runtime_version(&spec).state_version();
					let block: service::mangata_rococo_runtime::Block =
						generate_genesis_block(&*spec, state_version)?;
					let raw_header = block.header().encode();
					let output_buf = if params.raw {
						raw_header
					} else {
						format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
					};
					output_buf
				},
				_ => panic!("invalid chain spec"),
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		},
		Some(Subcommand::ExportGenesisWasm(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let raw_wasm_blob = extract_genesis_wasm(
				&cli.load_spec(&params.shared_params.chain.clone().unwrap_or_default())?,
			)?;
			let output_buf = if params.raw {
				raw_wasm_blob
			} else {
				format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			match chain_spec {
				#[cfg(feature = "mangata-kusama")]
				spec if spec.is_mangata_kusama() => match cmd {
					BenchmarkCmd::Pallet(cmd) =>
						if cfg!(feature = "runtime-benchmarks") {
							runner.sync_run(|config| {
								cmd.run::<service::mangata_kusama_runtime::Block, service::MangataKusamaRuntimeExecutor>(config)
							})
						} else {
							Err("Benchmarking wasn't enabled when building the node. \
						You can enable it with `--features runtime-benchmarks`."
								.into())
						},
					BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
						let partials = new_partial::<
							service::mangata_kusama_runtime::RuntimeApi,
							service::MangataKusamaRuntimeExecutor,
						>(&config)?;
						cmd.run(partials.client)
					}),
					#[cfg(not(feature = "runtime-benchmarks"))]
					BenchmarkCmd::Storage(_) =>
						return Err(sc_cli::Error::Input(
							"Compile with --features=runtime-benchmarks \
						to enable storage benchmarks."
								.into(),
						)
						.into()),
					#[cfg(feature = "runtime-benchmarks")]
					BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
						let partials = new_partial::<
							service::mangata_kusama_runtime::RuntimeApi,
							service::MangataKusamaRuntimeExecutor,
						>(&config)?;
						let db = partials.backend.expose_db();
						let storage = partials.backend.expose_storage();

						cmd.run(config, partials.client.clone(), db, storage)
					}),
					BenchmarkCmd::Extrinsic(_) => Err("Unsupported benchmarking command".into()),
					BenchmarkCmd::Overhead(cmd) => runner.sync_run(|config| {
						let PartialComponents { client, task_manager: _, .. } = new_partial::<
							service::mangata_kusama_runtime::RuntimeApi,
							service::MangataKusamaRuntimeExecutor,
						>(&config)?;
						let ext_builder = BenchmarkExtrinsicBuilder::new(client.clone());

						let first_block_inherent =
							inherent_benchmark_data([0u8; 32], Duration::from_millis(0))?;

						let first_block_seed = sp_ver::extract_inherent_data(&first_block_inherent)
							.map_err(|_| {
								sp_blockchain::Error::Backend(String::from(
									"cannot read random seed from inherents data",
								))
							})?;

						let second_block_inherent = inherent_benchmark_data(
							first_block_seed.seed.as_bytes().try_into().unwrap(),
							Duration::from_millis(12000),
						)?;

						cmd.run_ver(
							config,
							client.clone(),
							(first_block_inherent, second_block_inherent),
							&ext_builder,
						)
					}),
					BenchmarkCmd::Machine(cmd) => runner
						.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
				},
				#[cfg(feature = "mangata-rococo")]
				spec if spec.is_mangata_rococo() => match cmd {
					BenchmarkCmd::Pallet(cmd) =>
						if cfg!(feature = "runtime-benchmarks") {
							runner.sync_run(|config| {
								cmd.run::<service::mangata_rococo_runtime::Block, service::MangataRococoRuntimeExecutor>(config)
							})
						} else {
							Err("Benchmarking wasn't enabled when building the node. \
						You can enable it with `--features runtime-benchmarks`."
								.into())
						},
					BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
						let partials = new_partial::<
							service::mangata_rococo_runtime::RuntimeApi,
							service::MangataRococoRuntimeExecutor,
						>(&config)?;
						cmd.run(partials.client)
					}),
					#[cfg(not(feature = "runtime-benchmarks"))]
					BenchmarkCmd::Storage(_) =>
						return Err(sc_cli::Error::Input(
							"Compile with --features=runtime-benchmarks \
						to enable storage benchmarks."
								.into(),
						)
						.into()),
					#[cfg(feature = "runtime-benchmarks")]
					BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
						let partials = new_partial::<
							service::mangata_rococo_runtime::RuntimeApi,
							service::MangataRococoRuntimeExecutor,
						>(&config)?;
						let db = partials.backend.expose_db();
						let storage = partials.backend.expose_storage();

						cmd.run(config, partials.client.clone(), db, storage)
					}),
					BenchmarkCmd::Overhead(_) => Err("Unsupported benchmarking command".into()),
					// TODO: not sure what to do with this Extrinsic
					BenchmarkCmd::Extrinsic(_) => Err("Unsupported benchmarking command".into()),
					BenchmarkCmd::Machine(cmd) => runner
						.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
				},
				_ => panic!("invalid chain spec"),
			}
		},
		Some(Subcommand::TryRuntime(cmd)) =>
			if cfg!(feature = "try-runtime") {
				let runner = cli.create_runner(cmd)?;
				let chain_spec = &runner.config().chain_spec;

				match chain_spec {
					#[cfg(feature = "mangata-kusama")]
					spec if spec.is_mangata_kusama() => runner.async_run(|config| {
						let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
						let task_manager =
							sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
								.map_err(|e| {
									sc_cli::Error::Service(sc_service::Error::Prometheus(e))
								})?;

						Ok((
								cmd.run::<service::mangata_kusama_runtime::Block, service::MangataKusamaRuntimeExecutor>(config),
								task_manager,
							))
					}),
					#[cfg(feature = "mangata-rococo")]
					spec if spec.is_mangata_rococo() => runner.async_run(|config| {
						let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
						let task_manager =
							sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
								.map_err(|e| {
									sc_cli::Error::Service(sc_service::Error::Prometheus(e))
								})?;

						Ok((
								cmd.run::<service::mangata_rococo_runtime::Block, service::MangataRococoRuntimeExecutor>(
									config,
								),
								task_manager,
							))
					}),
					_ => panic!("invalid chain spec"),
				}
			} else {
				Err("Try-runtime must be enabled by `--features try-runtime`.".into())
			},
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();

			runner.run_node_until_exit(|config| async move {
				let hwbench = if !cli.no_hardware_benchmarks {
					config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(&database_path);
						sc_sysinfo::gather_hwbench(Some(database_path))
					})
				} else {
					None
				};
				let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
					.map(|e| e.para_id)
					.ok_or_else(|| "Could not find parachain ID in chain-spec.")?;

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);

				let id = ParaId::from(para_id);

				let parachain_account =
					AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account_truncating(&id);

				let genesis_state = match &config.chain_spec {
					#[cfg(feature = "mangata-kusama")]
					spec if spec.is_mangata_kusama() => {
						let state_version = Cli::native_runtime_version(&spec).state_version();
						let block: service::mangata_kusama_runtime::Block =
							generate_genesis_block(&*config.chain_spec, state_version)
								.map_err(|e| format!("{:?}", e))?;
						format!("0x{:?}", HexDisplay::from(&block.header().encode()))
					},
					#[cfg(feature = "mangata-rococo")]
					spec if spec.is_mangata_rococo() => {
						let state_version = Cli::native_runtime_version(&spec).state_version();
						let block: service::mangata_rococo_runtime::Block =
							generate_genesis_block(&*config.chain_spec, state_version)
								.map_err(|e| format!("{:?}", e))?;
						format!("0x{:?}", HexDisplay::from(&block.header().encode()))
					},
					_ => panic!("invalid chain spec"),
				};

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", id);
				info!("Parachain Account: {}", parachain_account);
				info!("Parachain genesis state: {}", genesis_state);
				info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

				match &config.chain_spec {
					#[cfg(feature = "mangata-kusama")]
					spec if spec.is_mangata_kusama() => crate::service::start_parachain_node::<
						service::mangata_kusama_runtime::RuntimeApi,
						service::MangataKusamaRuntimeExecutor,
					>(config, polkadot_config, collator_options, id, hwbench)
					.await
					.map(|r| r.0)
					.map_err(Into::into),
					#[cfg(feature = "mangata-rococo")]
					spec if spec.is_mangata_rococo() => crate::service::start_parachain_node::<
						service::mangata_rococo_runtime::RuntimeApi,
						service::MangataRococoRuntimeExecutor,
					>(config, polkadot_config, collator_options, id, hwbench)
					.await
					.map(|r| r.0)
					.map_err(Into::into),
					_ => panic!("invalid chain spec"),
				}
			})
		},
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		self.shared_params().base_path()
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool(is_dev)
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}
}
