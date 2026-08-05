//! Module: ic::model::requests::metrics
//!
//! Responsibility: bounded official Dashboard Metrics API request contracts.
//! Does not own: network resources, canister discovery, transport, or reports.
//! Boundary: captures one selected aggregate metric and its explicit observation window.

use serde::Serialize;
use std::{fmt, str::FromStr};

///
/// IcMetricKind
///
/// One bounded network metric exposed by the official Dashboard Metrics API.
///

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IcMetricKind {
    /// Network instruction execution rate.
    InstructionRate,
    /// Network message execution rate.
    MessageExecutionRate,
    /// Network cycle burn rate.
    CycleBurnRate,
    /// Network block ingestion rate.
    BlockRate,
    /// Total and currently up node counts.
    IcNodeCount,
    /// Total Subnet count.
    IcSubnetTotal,
    /// Running and stopped canister counts.
    RegisteredCanistersCount,
    /// Total estimated IC energy-consumption rate in kWh.
    TotalIcEnergyConsumptionRateKwh,
    /// Active boundary-node count.
    BoundaryNodesCount,
}

impl IcMetricKind {
    /// Return every metric supported by the bounded report adapter.
    #[must_use]
    pub const fn all() -> [Self; 9] {
        [
            Self::InstructionRate,
            Self::MessageExecutionRate,
            Self::CycleBurnRate,
            Self::BlockRate,
            Self::IcNodeCount,
            Self::IcSubnetTotal,
            Self::RegisteredCanistersCount,
            Self::TotalIcEnergyConsumptionRateKwh,
            Self::BoundaryNodesCount,
        ]
    }

    /// Return the official Dashboard Metrics API path name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InstructionRate => "instruction-rate",
            Self::MessageExecutionRate => "message-execution-rate",
            Self::CycleBurnRate => "cycle-burn-rate",
            Self::BlockRate => "block-rate",
            Self::IcNodeCount => "ic-node-count",
            Self::IcSubnetTotal => "ic-subnet-total",
            Self::RegisteredCanistersCount => "registered-canisters-count",
            Self::TotalIcEnergyConsumptionRateKwh => "total-ic-energy-consumption-rate-kwh",
            Self::BoundaryNodesCount => "boundary-nodes-count",
        }
    }

    #[cfg(feature = "dashboard-host")]
    pub(crate) const fn series_names(self) -> &'static [&'static str] {
        match self {
            Self::InstructionRate => &["instruction_rate"],
            Self::MessageExecutionRate => &["message_execution_rate"],
            Self::CycleBurnRate => &["cycle_burn_rate"],
            Self::BlockRate => &["block_rate"],
            Self::IcNodeCount => &["total_nodes", "up_nodes"],
            Self::IcSubnetTotal => &["ic_subnet_total"],
            Self::RegisteredCanistersCount => &["running_canisters", "stopped_canisters"],
            Self::TotalIcEnergyConsumptionRateKwh => &["energy_consumption_rate"],
            Self::BoundaryNodesCount => &["boundary_nodes_count"],
        }
    }
}

impl fmt::Display for IcMetricKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for IcMetricKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::all()
            .into_iter()
            .find(|metric| metric.as_str() == value)
            .ok_or_else(|| format!("unsupported IC Dashboard metric {value:?}"))
    }
}

///
/// IcMetricQuery
///
/// One explicitly bounded official Dashboard metric time-series query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcMetricQuery {
    /// Official Dashboard metric to retrieve.
    pub metric: IcMetricKind,
    /// Inclusive query start as Unix seconds.
    pub start_unix_secs: u64,
    /// Inclusive query end as Unix seconds.
    pub end_unix_secs: u64,
    /// Requested observation interval in seconds.
    pub step_secs: u32,
}

impl IcMetricQuery {
    /// Construct one explicit metric query window.
    #[must_use]
    pub const fn new(
        metric: IcMetricKind,
        start_unix_secs: u64,
        end_unix_secs: u64,
        step_secs: u32,
    ) -> Self {
        Self {
            metric,
            start_unix_secs,
            end_unix_secs,
            step_secs,
        }
    }
}

///
/// IcMetricRequest
///
/// Request accepted by the bounded official Dashboard metric report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcMetricRequest {
    /// Dashboard Metrics API base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Explicitly bounded metric query.
    pub query: IcMetricQuery,
}

impl IcMetricRequest {
    /// Construct one bounded live Dashboard metric request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        query: IcMetricQuery,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            query,
        }
    }
}
