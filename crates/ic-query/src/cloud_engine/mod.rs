//! Native CloudEngine control-plane report models, adapters, builders, and renderers.

#[cfg(feature = "cloud-engine-host")]
mod build;
mod model;
#[cfg(feature = "cloud-engine-host")]
mod source;
mod text;
#[cfg(feature = "cloud-engine-host")]
mod wire;

#[cfg(feature = "cloud-engine-host")]
use crate::runtime::RuntimeError;
#[cfg(feature = "cloud-engine-host")]
use thiserror::Error as ThisError;

#[cfg(feature = "cloud-engine-host")]
pub use build::{
    build_cloud_engine_operator_report, build_cloud_engine_operator_report_with_source,
    build_cloud_engine_prices_report, build_cloud_engine_prices_report_with_source,
};
pub use model::{
    CloudEngineNodeType, CloudEngineOperatorReport, CloudEnginePriceRow, CloudEnginePricesReport,
    CloudEngineReportContext,
};
#[cfg(feature = "cloud-engine-host")]
pub use model::{CloudEngineOperatorSourceData, CloudEnginePricesSourceData};
#[cfg(feature = "cloud-engine-host")]
pub use source::{CloudEngineSource, CloudEngineSourceRequest, LiveCloudEngineSource};
pub use text::{cloud_engine_operator_report_text, cloud_engine_prices_report_text};

/// Mainnet CloudEngine control-plane registry canister principal.
pub const MAINNET_CLOUD_ENGINE_CANISTER_ID: &str = "q6cfj-fyaaa-aaaar-qb77q-cai";

/// Default replica endpoint used for live CloudEngine control-plane queries.
pub const DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT: &str = "https://icp-api.io";

/// Maximum marketplace rows accepted from one live response.
pub const MAX_CLOUD_ENGINE_PRICE_ROWS: usize = 1_000;

/// Maximum claimed domains accepted from one engine operator.
pub const MAX_CLOUD_ENGINE_DOMAINS: usize = 100;

/// Maximum decimal digits accepted for one marketplace cycle amount.
pub const MAX_CLOUD_ENGINE_CYCLE_DECIMAL_DIGITS: usize = 256;

#[cfg(feature = "cloud-engine-host")]
const CLOUD_ENGINE_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "cloud-engine-host")]
const CLOUD_ENGINE_AUTHORITY: &str = "cloud_engine_control_plane_canister";

///
/// CloudEngineHostError
///
/// Failure while collecting or validating one live CloudEngine report.
///

#[cfg(feature = "cloud-engine-host")]
#[derive(Debug, ThisError)]
pub enum CloudEngineHostError {
    /// The requested network is not the supported mainnet identity.
    #[error(
        "`icq cloud-engine` supports only the mainnet `ic` network\n\nThese reports query the mainnet CloudEngine control-plane canister.\n\nTry:\n  icq --network ic cloud-engine prices"
    )]
    UnsupportedNetwork {
        /// Rejected network identity.
        network: String,
    },

    /// A requested principal is invalid.
    #[error("invalid {field}: {reason}")]
    InvalidPrincipal {
        /// Principal field being parsed.
        field: &'static str,
        /// Principal parsing failure.
        reason: String,
    },

    /// The IC agent could not be constructed for the requested endpoint.
    #[error("failed to build IC agent for {endpoint}: {reason}")]
    AgentBuild {
        /// Endpoint used to build the agent.
        endpoint: String,
        /// Agent construction failure.
        reason: String,
    },

    /// The built-in CloudEngine control-plane principal could not be parsed.
    #[error("invalid built-in CloudEngine canister principal: {reason}")]
    CanisterId {
        /// Principal parsing failure.
        reason: String,
    },

    /// A CloudEngine control-plane query failed.
    #[error("CloudEngine agent call {method} failed: {reason}")]
    AgentCall {
        /// Canister method being queried.
        method: &'static str,
        /// Agent call failure.
        reason: String,
    },

    /// A CloudEngine query argument could not be Candid encoded.
    #[error("failed to encode Candid {message}: {reason}")]
    CandidEncode {
        /// Candid request type.
        message: &'static str,
        /// Encoding failure.
        reason: String,
    },

    /// A CloudEngine response could not be Candid decoded.
    #[error("failed to decode Candid {message}: {reason}")]
    CandidDecode {
        /// Candid response type.
        message: &'static str,
        /// Decoding failure.
        reason: String,
    },

    /// A custom CloudEngine source returned inconsistent or excessive data.
    #[error("invalid CloudEngine source data: {reason}")]
    InvalidSourceData {
        /// Deterministic source-contract failure.
        reason: String,
    },

    /// The synchronous host runtime could not execute the live query.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

#[cfg(feature = "cloud-engine-host")]
fn enforce_mainnet_network(network: &str) -> Result<(), CloudEngineHostError> {
    crate::network::enforce_mainnet_network_with(network, |network| {
        CloudEngineHostError::UnsupportedNetwork { network }
    })
}

#[cfg(all(test, feature = "cloud-engine-host"))]
mod tests;
