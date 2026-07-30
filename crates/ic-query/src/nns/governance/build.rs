//! Module: nns::governance::build
//!
//! Responsibility: assemble direct NNS Governance reports with shared provenance.
//! Does not own: live transport, Candid wire types, or text rendering.
//! Boundary: validates target identity before invoking any source capability.

use super::{
    NNS_GOVERNANCE_REPORT_SCHEMA_VERSION, NnsGovernanceEconomicsReport, NnsGovernanceHostError,
    NnsGovernanceMaturityModulationReport, NnsGovernanceMetricsReport, NnsGovernanceReportContext,
    NnsGovernanceRewardEventReport, NnsGovernanceSource, enforce_mainnet_network,
    validate_governance_metrics,
};
use crate::{
    ic_registry::MAINNET_GOVERNANCE_CANISTER_ID,
    nns::{LiveNnsSource, NnsSourceRequest},
};

/// Build one live NNS Governance economics report.
pub fn build_nns_governance_economics_report(
    request: &NnsSourceRequest,
) -> Result<NnsGovernanceEconomicsReport, NnsGovernanceHostError> {
    build_nns_governance_economics_report_with_source(request, &LiveNnsSource)
}

/// Build one NNS Governance economics report from a custom source.
pub fn build_nns_governance_economics_report_with_source(
    request: &NnsSourceRequest,
    source: &dyn NnsGovernanceSource,
) -> Result<NnsGovernanceEconomicsReport, NnsGovernanceHostError> {
    enforce_mainnet_network(&request.network)?;
    Ok(NnsGovernanceEconomicsReport {
        context: report_context(request),
        economics: source.fetch_economics(request)?,
    })
}

/// Build one live NNS Governance metrics report.
pub fn build_nns_governance_metrics_report(
    request: &NnsSourceRequest,
) -> Result<NnsGovernanceMetricsReport, NnsGovernanceHostError> {
    build_nns_governance_metrics_report_with_source(request, &LiveNnsSource)
}

/// Build one NNS Governance metrics report from a custom source.
pub fn build_nns_governance_metrics_report_with_source(
    request: &NnsSourceRequest,
    source: &dyn NnsGovernanceSource,
) -> Result<NnsGovernanceMetricsReport, NnsGovernanceHostError> {
    enforce_mainnet_network(&request.network)?;
    let metrics = source.fetch_metrics(request)?;
    validate_governance_metrics(&metrics)?;
    Ok(NnsGovernanceMetricsReport {
        context: report_context(request),
        metrics,
    })
}

/// Build one live latest NNS Governance reward-event report.
pub fn build_nns_governance_reward_event_report(
    request: &NnsSourceRequest,
) -> Result<NnsGovernanceRewardEventReport, NnsGovernanceHostError> {
    build_nns_governance_reward_event_report_with_source(request, &LiveNnsSource)
}

/// Build one latest NNS Governance reward-event report from a custom source.
pub fn build_nns_governance_reward_event_report_with_source(
    request: &NnsSourceRequest,
    source: &dyn NnsGovernanceSource,
) -> Result<NnsGovernanceRewardEventReport, NnsGovernanceHostError> {
    enforce_mainnet_network(&request.network)?;
    Ok(NnsGovernanceRewardEventReport {
        context: report_context(request),
        reward_event: source.fetch_reward_event(request)?,
    })
}

/// Build one live NNS Governance maturity-modulation report.
pub fn build_nns_governance_maturity_modulation_report(
    request: &NnsSourceRequest,
) -> Result<NnsGovernanceMaturityModulationReport, NnsGovernanceHostError> {
    build_nns_governance_maturity_modulation_report_with_source(request, &LiveNnsSource)
}

/// Build one NNS Governance maturity-modulation report from a custom source.
pub fn build_nns_governance_maturity_modulation_report_with_source(
    request: &NnsSourceRequest,
    source: &dyn NnsGovernanceSource,
) -> Result<NnsGovernanceMaturityModulationReport, NnsGovernanceHostError> {
    enforce_mainnet_network(&request.network)?;
    Ok(NnsGovernanceMaturityModulationReport {
        context: report_context(request),
        maturity_modulation: source.fetch_maturity_modulation(request)?,
    })
}

fn report_context(request: &NnsSourceRequest) -> NnsGovernanceReportContext {
    NnsGovernanceReportContext {
        schema_version: NNS_GOVERNANCE_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        source_endpoint: request.endpoint.clone(),
        fetched_by: request.fetched_by.clone(),
    }
}
