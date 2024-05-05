//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;

use rollup_runtime::runtime_config::{
	opaque::Block,
	types::{AccountId, Balance, TokenId},
};

use rolldown_runtime_api::RolldownRuntimeApi;
use sp_runtime::SaturatedConversion;
use xyk_runtime_api::XykRuntimeApi;

use substrate_prometheus_endpoint::{
	MetricSource, Opts, PrometheusError, Registry, SourcedCounter, SourcedGauge,
};

pub fn register(
	registry: Option<&Registry>,
	client: Arc<crate::service::FullClient>,
) -> Result<(), PrometheusError> {
	if let Some(registry) = registry {
		GaspMetricsCounter::register(registry, client.clone())?;
		RolldownLastProccessedRequestOnL2Counter::register(registry, client.clone())?;
		RolldownNumberOfPendingRequestsGauge::register(registry, client.clone())?;
	}
	Ok(())
}

pub struct RolldownLastProccessedRequestOnL2Counter<C>(Arc<C>);

impl<C> Clone for RolldownLastProccessedRequestOnL2Counter<C> {
	fn clone(&self) -> Self {
		RolldownLastProccessedRequestOnL2Counter(self.0.clone())
	}
}

impl<C> RolldownLastProccessedRequestOnL2Counter<C>
where
	C: ProvideRuntimeApi<Block>,
	C: Send + Sync + 'static,
	C: HeaderBackend<Block>,
	C::Api: RolldownRuntimeApi<
		Block,
		pallet_rolldown::messages::L1Update,
		pallet_rolldown::messages::L1,
	>,
{
	/// Registers the `RolldownLastProccessedRequestOnL2Counter` metric whose value is
	/// obtained from the given `C` as in Client that would implement the api we need.
	pub fn register(registry: &Registry, client: Arc<C>) -> Result<(), PrometheusError> {
		substrate_prometheus_endpoint::register(
			SourcedCounter::new(
				&Opts::new(
					"substrate_rolldown_last_processed_request_on_l2",
					"Last Processed Request On L2",
				)
				.variable_label("for_L1"),
				RolldownLastProccessedRequestOnL2Counter(client.clone()),
			)?,
			registry,
		)?;
		Ok(())
	}
}

impl<C> MetricSource for RolldownLastProccessedRequestOnL2Counter<C>
where
	C: ProvideRuntimeApi<Block>,
	C: Send + Sync + 'static,
	C: HeaderBackend<Block>,
	C::Api: RolldownRuntimeApi<
		Block,
		pallet_rolldown::messages::L1Update,
		pallet_rolldown::messages::L1,
	>,
{
	type N = u64;

	fn collect(&self, mut set: impl FnMut(&[&str], Self::N)) {
		let at = self.0.info().best_hash;
		set(
			&["Ethereum"],
			self.0
				.runtime_api()
				.get_last_processed_request_on_l2(at, pallet_rolldown::messages::L1::Ethereum)
				.unwrap_or(None)
				.unwrap_or_default()
				.saturated_into::<u64>(),
		);
	}
}

pub struct RolldownNumberOfPendingRequestsGauge<C>(Arc<C>);

impl<C> Clone for RolldownNumberOfPendingRequestsGauge<C> {
	fn clone(&self) -> Self {
		RolldownNumberOfPendingRequestsGauge(self.0.clone())
	}
}

impl<C> RolldownNumberOfPendingRequestsGauge<C>
where
	C: ProvideRuntimeApi<Block>,
	C: Send + Sync + 'static,
	C: HeaderBackend<Block>,
	C::Api: RolldownRuntimeApi<
		Block,
		pallet_rolldown::messages::L1Update,
		pallet_rolldown::messages::L1,
	>,
{
	/// Registers the `RolldownNumberOfPendingRequestsGauge` metric whose value is
	/// obtained from the given `C` as in Client that would implement the api we need.
	pub fn register(registry: &Registry, client: Arc<C>) -> Result<(), PrometheusError> {
		substrate_prometheus_endpoint::register(
			SourcedCounter::new(
				&Opts::new(
					"substrate_rolldown_number_of_pending_requests",
					"Number Of Pending Requests",
				)
				.variable_label("for_L1"),
				RolldownNumberOfPendingRequestsGauge(client.clone()),
			)?,
			registry,
		)?;
		Ok(())
	}
}

impl<C> MetricSource for RolldownNumberOfPendingRequestsGauge<C>
where
	C: ProvideRuntimeApi<Block>,
	C: Send + Sync + 'static,
	C: HeaderBackend<Block>,
	C::Api: RolldownRuntimeApi<
		Block,
		pallet_rolldown::messages::L1Update,
		pallet_rolldown::messages::L1,
	>,
{
	type N = u64;

	fn collect(&self, mut set: impl FnMut(&[&str], Self::N)) {
		let at = self.0.info().best_hash;
		set(
			&["Ethereum"],
			self.0
				.runtime_api()
				.get_number_of_pending_requests(at, pallet_rolldown::messages::L1::Ethereum)
				.unwrap_or(None)
				.unwrap_or_default()
				.saturated_into::<u64>(),
		);
	}
}

pub struct GaspMetricsCounter<C>(Arc<C>);

impl<C> Clone for GaspMetricsCounter<C> {
	fn clone(&self) -> Self {
		GaspMetricsCounter(self.0.clone())
	}
}

impl<C> GaspMetricsCounter<C>
where
	C: ProvideRuntimeApi<Block>,
	C: Send + Sync + 'static,
	C: HeaderBackend<Block>,
	// use the concrete type sp_runtime::AccountId20 here because whatever implements MetricSource needs to be
	// Send + Sync + Clone and the AccountId abstraction we get from Signer and IdentifyAccount do not provide these bounds
	C::Api: XykRuntimeApi<Block, Balance, TokenId, sp_runtime::AccountId20>,
	C::Api: RolldownRuntimeApi<
		Block,
		pallet_rolldown::messages::L1Update,
		pallet_rolldown::messages::L1,
	>,
{
	/// Registers the `GaspMetricsCounter` metric whose value is
	/// obtained from the given `C` as in Client that would implement the api we need.
	pub fn register(registry: &Registry, client: Arc<C>) -> Result<(), PrometheusError> {
		substrate_prometheus_endpoint::register(
			SourcedCounter::new(
				&Opts::new("substrate_gasp_metrics", "Gasp Metrics").variable_label("Counter_for"),
				GaspMetricsCounter(client.clone()),
			)?,
			registry,
		)?;
		Ok(())
	}
}

impl<C> MetricSource for GaspMetricsCounter<C>
where
	C: ProvideRuntimeApi<Block>,
	C: Send + Sync + 'static,
	C: HeaderBackend<Block>,
	// use the concrete type sp_runtime::AccountId20 here because whatever implements MetricSource needs to be
	// Send + Sync + Clone and the AccountId abstraction we get from Signer and IdentifyAccount do not provide these bounds
	C::Api: XykRuntimeApi<Block, Balance, TokenId, sp_runtime::AccountId20>,
	C::Api: RolldownRuntimeApi<
		Block,
		pallet_rolldown::messages::L1Update,
		pallet_rolldown::messages::L1,
	>,
{
	type N = u64;

	fn collect(&self, mut set: impl FnMut(&[&str], Self::N)) {
		let at = self.0.info().best_hash;
		set(
			&["Total Deposits"],
			self.0
				.runtime_api()
				.get_total_number_of_deposits(at)
				.unwrap_or_default()
				.saturated_into::<u64>(),
		);
		set(
			&["Total Withdrawals"],
			self.0
				.runtime_api()
				.get_total_number_of_withdrawals(at)
				.unwrap_or_default()
				.saturated_into::<u64>(),
		);
		set(
			&["Total Swaps"],
			self.0
				.runtime_api()
				.get_total_number_of_swaps(at)
				.unwrap_or_default()
				.saturated_into::<u64>(),
		);
	}
}
