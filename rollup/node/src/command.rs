use crate::{
	benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder},
	chain_spec,
	cli::{Cli, Subcommand},
	service,
	service::Block,
};
use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
use futures::executor::block_on;
use sc_cli::SubstrateCli;
use sc_executor::{WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_service::{Deref, PartialComponents};
use sp_keyring::Sr25519Keyring;
use std::{
	convert::TryInto,
	sync::{Arc, Mutex},
	time::Duration,
};

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Rollup Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"support.anonymous.an".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"" | "rollup-local" => Box::new(chain_spec::rollup_local_config(false)),
			"rollup-local-seq" => Box::new(chain_spec::rollup_local_config(true)),
			"rollup-eigen-local" => Box::new(chain_spec::rollup_eigen_local(false)),
			path =>
				Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
		})
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, backend, .. } =
					service::new_partial(&config)?;
				let aux_revert = Box::new(|client, _, blocks| {
					sc_consensus_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				// This switch needs to be in the client, since the client decides
				// which sub-commands it wants to support.
				match cmd {
					BenchmarkCmd::Pallet(cmd) => {
						if !cfg!(feature = "runtime-benchmarks") {
							return Err(
								"Runtime benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
									.into(),
							)
						}

						cmd.run::<Block, ()>(config)
					},
					BenchmarkCmd::Block(cmd) => {
						let PartialComponents { client, .. } = service::new_partial(&config)?;
						cmd.run(client)
					},
					#[cfg(not(feature = "runtime-benchmarks"))]
					BenchmarkCmd::Storage(_) => Err(
						"Storage benchmarking can be enabled with `--features runtime-benchmarks`."
							.into(),
					),
					#[cfg(feature = "runtime-benchmarks")]
					BenchmarkCmd::Storage(cmd) => {
						let PartialComponents { client, backend, .. } =
							service::new_partial(&config)?;
						let db = backend.expose_db();
						let storage = backend.expose_storage();

						cmd.run(config, client, db, storage)
					},
					BenchmarkCmd::Overhead(cmd) => {
						let executor = WasmExecutor::builder()
							.with_execution_method(config.wasm_method)
							.with_onchain_heap_alloc_strategy(DEFAULT_HEAP_ALLOC_STRATEGY)
							.with_offchain_heap_alloc_strategy(DEFAULT_HEAP_ALLOC_STRATEGY)
							.with_max_runtime_instances(config.max_runtime_instances)
							.with_runtime_cache_size(config.runtime_cache_size)
							.build();

						let (c, _, _, _) =
							sc_service::new_full_parts::<Block, rollup_runtime::RuntimeApi, _>(
								&config, None, executor,
							)?;

						let client = Arc::new(Mutex::new(c));
						let ext_builder = RemarkBuilder::new(client.clone());

						let first_block_inherent =
							block_on(inherent_benchmark_data([0u8; 32], Duration::from_millis(0)))
								.unwrap();

						let first_block_seed = sp_ver::extract_inherent_data(&first_block_inherent)
							.map_err(|_| {
								sp_blockchain::Error::Backend(String::from(
									"cannot read random seed from inherents data",
								))
							})?;

						let second_block_inherent = block_on(inherent_benchmark_data(
							first_block_seed.seed.as_bytes().try_into().unwrap(),
							Duration::from_millis(6000),
						))
						.unwrap();

						cmd.run_ver(
							config,
							client,
							(first_block_inherent, second_block_inherent),
							&ext_builder,
						)
					},
					BenchmarkCmd::Extrinsic(cmd) => {
						let executor = WasmExecutor::builder()
							.with_execution_method(config.wasm_method)
							.with_onchain_heap_alloc_strategy(DEFAULT_HEAP_ALLOC_STRATEGY)
							.with_offchain_heap_alloc_strategy(DEFAULT_HEAP_ALLOC_STRATEGY)
							.with_max_runtime_instances(config.max_runtime_instances)
							.with_runtime_cache_size(config.runtime_cache_size)
							.build();

						let (c, _, _, _) =
							sc_service::new_full_parts::<Block, rollup_runtime::RuntimeApi, _>(
								&config, None, executor,
							)?;

						let client = Arc::new(Mutex::new(c));
						// Register the *Remark* and *TKA* builders.
						let ext_factory = ExtrinsicFactory(vec![
							Box::new(RemarkBuilder::new(client.clone())),
							Box::new(TransferKeepAliveBuilder::new(
								client.clone(),
								Sr25519Keyring::Alice.to_account_id(),
								Default::default(),
							)),
						]);

						let first_block_inherent =
							block_on(inherent_benchmark_data([0u8; 32], Duration::from_millis(0)))
								.unwrap();

						let first_block_seed = sp_ver::extract_inherent_data(&first_block_inherent)
							.map_err(|_| {
								sp_blockchain::Error::Backend(String::from(
									"cannot read random seed from inherents data",
								))
							})?;

						let second_block_inherent = block_on(inherent_benchmark_data(
							first_block_seed.seed.as_bytes().try_into().unwrap(),
							Duration::from_millis(6000),
						))
						.unwrap();

						cmd.run_ver(
							client,
							(first_block_inherent, second_block_inherent),
							Vec::new(),
							&ext_factory,
						)
					},
					BenchmarkCmd::Machine(cmd) =>
						cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()),
				}
			})
		},
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime) => Err(try_runtime_cli::DEPRECATION_NOTICE.into()),
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
			.into()),
		Some(Subcommand::ChainInfo(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run::<Block>(&config))
		},
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				service::new_full(config).map_err(sc_cli::Error::Service)
			})
		},
	}
}
