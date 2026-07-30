//! Module: nns::governance::source
//!
//! Responsibility: query and project direct NNS Governance reports.
//! Does not own: CLI parsing, caching, or process output.
//! Boundary: adapts four native Governance query methods into stable public values.

use super::{
    NnsGovernanceEconomics, NnsGovernanceHostError, NnsGovernanceMaturityModulation,
    NnsGovernanceMetrics, NnsGovernanceRewardEvent, enforce_mainnet_network,
    validate_governance_metrics,
    wire::{
        GetMaturityModulationRequest, GetMaturityModulationResponse, GetMetricsResult,
        GovernanceCachedMetrics,
    },
};
use crate::{
    nns::{
        LiveNnsSource, NnsSourceRequest,
        governance_query::{query_nns_governance, query_nns_governance_no_args},
    },
    runtime::block_on_current_thread,
};

///
/// NnsGovernanceSource
///
/// Source capability for direct NNS Governance economics, metrics, and reward reports.
///

pub trait NnsGovernanceSource {
    /// Fetch the native network economics parameters.
    fn fetch_economics(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceEconomics, NnsGovernanceHostError>;

    /// Fetch the native cached Governance metrics.
    fn fetch_metrics(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceMetrics, NnsGovernanceHostError>;

    /// Fetch the latest native voting reward event.
    fn fetch_reward_event(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceRewardEvent, NnsGovernanceHostError>;

    /// Fetch the current native maturity modulation when supplied.
    fn fetch_maturity_modulation(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<Option<NnsGovernanceMaturityModulation>, NnsGovernanceHostError>;
}

impl NnsGovernanceSource for LiveNnsSource {
    fn fetch_economics(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceEconomics, NnsGovernanceHostError> {
        enforce_mainnet_network(&request.network)?;
        let economics: NnsGovernanceEconomics =
            block_on_current_thread(query_nns_governance_no_args(
                request,
                "get_network_economics_parameters",
                "NetworkEconomics",
            ))??;
        Ok(economics)
    }

    fn fetch_metrics(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceMetrics, NnsGovernanceHostError> {
        enforce_mainnet_network(&request.network)?;
        let result: GetMetricsResult = block_on_current_thread(query_nns_governance_no_args(
            request,
            "get_metrics",
            "GetMetricsResult",
        ))??;
        let metrics = NnsGovernanceMetrics::from(metrics_result(result)?);
        validate_governance_metrics(&metrics)?;
        Ok(metrics)
    }

    fn fetch_reward_event(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceRewardEvent, NnsGovernanceHostError> {
        enforce_mainnet_network(&request.network)?;
        let reward_event: NnsGovernanceRewardEvent = block_on_current_thread(
            query_nns_governance_no_args(request, "get_latest_reward_event", "RewardEvent"),
        )??;
        Ok(reward_event)
    }

    fn fetch_maturity_modulation(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<Option<NnsGovernanceMaturityModulation>, NnsGovernanceHostError> {
        enforce_mainnet_network(&request.network)?;
        let response: GetMaturityModulationResponse =
            block_on_current_thread(query_nns_governance(
                request,
                "get_maturity_modulation",
                "GetMaturityModulationRequest",
                "GetMaturityModulationResponse",
                &GetMaturityModulationRequest {},
            ))??;
        Ok(response.maturity_modulation)
    }
}

pub(super) fn metrics_result(
    result: GetMetricsResult,
) -> Result<GovernanceCachedMetrics, NnsGovernanceHostError> {
    match result {
        GetMetricsResult::Ok(metrics) => Ok(*metrics),
        GetMetricsResult::Err(error) => Err(NnsGovernanceHostError::Governance {
            error_type: error.error_type,
            message: error.error_message,
        }),
    }
}
