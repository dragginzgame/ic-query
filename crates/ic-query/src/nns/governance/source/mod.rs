//! Module: nns::governance::source
//!
//! Responsibility: define the portable direct NNS Governance source boundary.
//! Does not own: CLI parsing, persistence, scheduling, retries, or process output.
//! Boundary: every adapter returns typed data with transport-specific provenance.

#[cfg(all(feature = "canister", target_arch = "wasm32"))]
pub mod canister;
#[cfg(feature = "nns-host")]
pub mod host;

use super::{
    NnsGovernanceEconomics, NnsGovernanceError, NnsGovernanceMaturityModulation,
    NnsGovernanceMetrics, NnsGovernanceRequest, NnsGovernanceRewardEvent,
    NnsGovernanceSourceProvenance,
};
use std::{future::Future, pin::Pin};

#[cfg(all(feature = "canister", target_arch = "wasm32"))]
pub use canister::CanisterNnsSource;

///
/// NnsGovernanceSourceData
///
/// One typed source value and the evidence describing how it was collected.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsGovernanceSourceData<T> {
    /// Typed value returned by Governance.
    pub value: T,
    /// Transport-specific source evidence.
    pub provenance: NnsGovernanceSourceProvenance,
}

impl<T> NnsGovernanceSourceData<T> {
    /// Pair a typed value with its source provenance.
    #[must_use]
    pub const fn new(value: T, provenance: NnsGovernanceSourceProvenance) -> Self {
        Self { value, provenance }
    }
}

///
/// NnsGovernanceSourceFuture
///
/// Boxed caller-runtime future returned by a direct Governance source.
///

pub type NnsGovernanceSourceFuture<'a, T> = Pin<
    Box<dyn Future<Output = Result<NnsGovernanceSourceData<T>, NnsGovernanceError>> + Send + 'a>,
>;

///
/// NnsGovernanceSource
///
/// Portable async source capability for the four bounded Governance point reports.
///

pub trait NnsGovernanceSource: Send + Sync {
    /// Fetch the native network economics parameters.
    fn fetch_economics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceEconomics>;

    /// Fetch the native cached Governance metrics.
    fn fetch_metrics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceMetrics>;

    /// Fetch the latest native voting reward event.
    fn fetch_reward_event<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceRewardEvent>;

    /// Fetch the current native maturity modulation when supplied.
    fn fetch_maturity_modulation<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, Option<NnsGovernanceMaturityModulation>>;
}

#[cfg(any(feature = "nns-host", feature = "canister"))]
pub(super) fn metrics_result(
    result: super::wire::GetMetricsResult,
) -> Result<super::wire::GovernanceCachedMetrics, NnsGovernanceError> {
    match result {
        super::wire::GetMetricsResult::Ok(metrics) => Ok(*metrics),
        super::wire::GetMetricsResult::Err(error) => Err(NnsGovernanceError::Governance {
            error_type: error.error_type,
            message: error.error_message,
        }),
    }
}
