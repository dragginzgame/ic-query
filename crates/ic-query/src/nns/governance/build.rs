//! Module: nns::governance::build
//!
//! Responsibility: assemble direct NNS Governance reports with shared provenance.
//! Does not own: live transport, Candid wire types, persistence, or text rendering.
//! Boundary: validates request and returned source evidence around one source invocation.

use super::{
    NNS_GOVERNANCE_REPORT_SCHEMA_VERSION, NnsGovernanceEconomicsReport, NnsGovernanceError,
    NnsGovernanceMaturityModulationReport, NnsGovernanceMetricsReport, NnsGovernanceReportContext,
    NnsGovernanceRequest, NnsGovernanceRewardEventReport, NnsGovernanceSource,
    NnsGovernanceSourceProvenance,
    validation::{
        validate_governance_metrics, validate_governance_request, validate_source_provenance,
    },
};
use crate::nns::MAINNET_GOVERNANCE_CANISTER_ID;
#[cfg(feature = "nns-host")]
use crate::{nns::LiveNnsSource, runtime::block_on_current_thread};

#[cfg(feature = "nns-host")]
use super::NnsGovernanceHostError;

/// Build one live NNS Governance economics report through the native replica adapter.
#[cfg(feature = "nns-host")]
pub fn build_nns_governance_economics_report(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceEconomicsReport, NnsGovernanceHostError> {
    Ok(block_on_current_thread(
        build_nns_governance_economics_report_with_source(request, &LiveNnsSource),
    )??)
}

/// Build one NNS Governance economics report from a caller-owned async source.
pub async fn build_nns_governance_economics_report_with_source(
    request: &NnsGovernanceRequest,
    source: &dyn NnsGovernanceSource,
) -> Result<NnsGovernanceEconomicsReport, NnsGovernanceError> {
    validate_governance_request(request)?;
    let data = source.fetch_economics(request).await?;
    validate_source_provenance(&request.source, &data.provenance)?;
    Ok(NnsGovernanceEconomicsReport {
        context: report_context(request, data.provenance),
        economics: data.value,
    })
}

/// Build one live NNS Governance metrics report through the native replica adapter.
#[cfg(feature = "nns-host")]
pub fn build_nns_governance_metrics_report(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceMetricsReport, NnsGovernanceHostError> {
    Ok(block_on_current_thread(
        build_nns_governance_metrics_report_with_source(request, &LiveNnsSource),
    )??)
}

/// Build one NNS Governance metrics report from a caller-owned async source.
pub async fn build_nns_governance_metrics_report_with_source(
    request: &NnsGovernanceRequest,
    source: &dyn NnsGovernanceSource,
) -> Result<NnsGovernanceMetricsReport, NnsGovernanceError> {
    validate_governance_request(request)?;
    let data = source.fetch_metrics(request).await?;
    validate_source_provenance(&request.source, &data.provenance)?;
    validate_governance_metrics(&data.value)?;
    Ok(NnsGovernanceMetricsReport {
        context: report_context(request, data.provenance),
        metrics: data.value,
    })
}

/// Build one live latest reward-event report through the native replica adapter.
#[cfg(feature = "nns-host")]
pub fn build_nns_governance_reward_event_report(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceRewardEventReport, NnsGovernanceHostError> {
    Ok(block_on_current_thread(
        build_nns_governance_reward_event_report_with_source(request, &LiveNnsSource),
    )??)
}

/// Build one latest reward-event report from a caller-owned async source.
pub async fn build_nns_governance_reward_event_report_with_source(
    request: &NnsGovernanceRequest,
    source: &dyn NnsGovernanceSource,
) -> Result<NnsGovernanceRewardEventReport, NnsGovernanceError> {
    validate_governance_request(request)?;
    let data = source.fetch_reward_event(request).await?;
    validate_source_provenance(&request.source, &data.provenance)?;
    Ok(NnsGovernanceRewardEventReport {
        context: report_context(request, data.provenance),
        reward_event: data.value,
    })
}

/// Build one live maturity-modulation report through the native replica adapter.
#[cfg(feature = "nns-host")]
pub fn build_nns_governance_maturity_modulation_report(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceMaturityModulationReport, NnsGovernanceHostError> {
    Ok(block_on_current_thread(
        build_nns_governance_maturity_modulation_report_with_source(request, &LiveNnsSource),
    )??)
}

/// Build one maturity-modulation report from a caller-owned async source.
pub async fn build_nns_governance_maturity_modulation_report_with_source(
    request: &NnsGovernanceRequest,
    source: &dyn NnsGovernanceSource,
) -> Result<NnsGovernanceMaturityModulationReport, NnsGovernanceError> {
    validate_governance_request(request)?;
    let data = source.fetch_maturity_modulation(request).await?;
    validate_source_provenance(&request.source, &data.provenance)?;
    Ok(NnsGovernanceMaturityModulationReport {
        context: report_context(request, data.provenance),
        maturity_modulation: data.value,
    })
}

fn report_context(
    request: &NnsGovernanceRequest,
    source: NnsGovernanceSourceProvenance,
) -> NnsGovernanceReportContext {
    NnsGovernanceReportContext {
        schema_version: NNS_GOVERNANCE_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        source,
    }
}
