//! Certified Cycle Minting Canister ICP/XDR and cycles reports.

#[cfg(feature = "cmc-host")]
mod build;
mod model;
#[cfg(feature = "cmc-host")]
mod source;
mod text;
#[cfg(feature = "cmc-host")]
mod wire;

#[cfg(feature = "cmc-host")]
use crate::runtime::RuntimeError;
#[cfg(feature = "cmc-host")]
use thiserror::Error as ThisError;

#[cfg(feature = "cmc-host")]
pub use build::{
    build_cmc_cycles_report, build_cmc_cycles_report_with_source, build_cmc_xdr_report,
    build_cmc_xdr_report_with_source,
};
#[cfg(feature = "cmc-host")]
pub use model::CmcCertifiedRate;
pub use model::{
    CmcCertification, CmcCyclesReport, CmcIcpXdrConversionRate, CmcReportContext, CmcXdrReport,
};
#[cfg(feature = "cmc-host")]
pub use source::{CmcSource, CmcSourceRequest, LiveCmcSource};
pub use text::{cmc_cycles_report_text, cmc_xdr_report_text};

/// Mainnet Cycle Minting Canister principal.
pub const MAINNET_CMC_CANISTER_ID: &str = "rkp4c-7iaaa-aaaaa-aaaca-cai";

/// Default replica endpoint used for live CMC queries.
pub const DEFAULT_CMC_SOURCE_ENDPOINT: &str = "https://icp-api.io";

/// IC protocol conversion constant: one XDR corresponds to one trillion cycles.
pub const CYCLES_PER_XDR: u128 = 1_000_000_000_000;

#[cfg(feature = "cmc-host")]
const ICP_XDR_PERMYRIAD_DENOMINATOR: u128 = 10_000;
#[cfg(feature = "cmc-host")]
const CMC_REPORT_SCHEMA_VERSION: u32 = 1;

///
/// CmcHostError
///
/// Failure while collecting or validating one live CMC report.
///

#[cfg(feature = "cmc-host")]
#[derive(Debug, ThisError)]
pub enum CmcHostError {
    /// The requested network is not the supported mainnet identity.
    #[error(
        "`icq system` supports only the mainnet `ic` network\n\nThese reports query the Internet Computer mainnet Cycle Minting Canister.\n\nTry:\n  icq --network ic system xdr"
    )]
    UnsupportedNetwork {
        /// Rejected network identity.
        network: String,
    },

    /// The IC agent could not be constructed for the requested endpoint.
    #[error("failed to build IC agent for {endpoint}: {reason}")]
    AgentBuild {
        /// Endpoint used to build the agent.
        endpoint: String,
        /// Agent construction failure.
        reason: String,
    },

    /// The built-in mainnet CMC principal could not be parsed.
    #[error("invalid built-in CMC canister principal: {reason}")]
    CanisterId {
        /// Principal parsing failure.
        reason: String,
    },

    /// The CMC query call failed.
    #[error("CMC agent call {method} failed: {reason}")]
    AgentCall {
        /// CMC method being queried.
        method: &'static str,
        /// Agent call failure.
        reason: String,
    },

    /// The CMC query argument could not be Candid encoded.
    #[error("failed to encode Candid {message}: {reason}")]
    CandidEncode {
        /// Candid request type.
        message: &'static str,
        /// Encoding failure.
        reason: String,
    },

    /// The CMC response could not be Candid decoded.
    #[error("failed to decode Candid {message}: {reason}")]
    CandidDecode {
        /// Candid response type.
        message: &'static str,
        /// Decoding failure.
        reason: String,
    },

    /// The CMC certificate signature or delegation did not authenticate.
    #[error("CMC certified-rate authentication failed: {reason}")]
    CertificateAuthentication {
        /// Authentication failure detail from the IC agent.
        reason: String,
    },

    /// The CMC certified-rate witness did not prove the returned rate.
    #[error("invalid CMC certified-rate evidence: {reason}")]
    InvalidCertifiedRate {
        /// Deterministic witness validation failure.
        reason: String,
    },

    /// A custom CMC source returned structurally inconsistent evidence.
    #[error("invalid CMC source data: {reason}")]
    InvalidSourceData {
        /// Deterministic source-contract failure.
        reason: String,
    },

    /// The synchronous host runtime could not execute the live query.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

#[cfg(feature = "cmc-host")]
fn enforce_mainnet_network(network: &str) -> Result<(), CmcHostError> {
    crate::network::enforce_mainnet_network_with(network, |network| {
        CmcHostError::UnsupportedNetwork { network }
    })
}

#[cfg(all(test, feature = "cmc-host"))]
mod tests;
