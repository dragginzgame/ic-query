//! Module: nns::governance::build
//!
//! Responsibility: assemble direct NNS Governance reports with shared provenance.
//! Does not own: live transport, Candid wire types, persistence, or text rendering.
//! Boundary: validates request and returned source evidence around one source invocation.

use super::{
    NnsGovernanceEconomicsReport, NnsGovernanceError, NnsGovernanceMaturityModulationReport,
    NnsGovernanceMetricsReport, NnsGovernanceReportContext, NnsGovernanceRequest,
    NnsGovernanceRewardEventReport, NnsGovernanceSource,
    validation::{
        validate_governance_metrics, validate_governance_request, validate_source_provenance,
    },
};
#[cfg(feature = "nns-host")]
use crate::nns::LiveNnsSource;

/// Build one live NNS Governance economics report through the native replica adapter.
#[cfg(feature = "nns-host")]
pub async fn build_nns_governance_economics_report(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceEconomicsReport, NnsGovernanceError> {
    build_nns_governance_economics_report_with_source(request, &LiveNnsSource::default()).await
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
        context: NnsGovernanceReportContext::from_source(request, data.provenance),
        economics: data.value,
    })
}

/// Build one live NNS Governance metrics report through the native replica adapter.
#[cfg(feature = "nns-host")]
pub async fn build_nns_governance_metrics_report(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceMetricsReport, NnsGovernanceError> {
    build_nns_governance_metrics_report_with_source(request, &LiveNnsSource::default()).await
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
        context: NnsGovernanceReportContext::from_source(request, data.provenance),
        metrics: data.value,
    })
}

/// Build one live latest reward-event report through the native replica adapter.
#[cfg(feature = "nns-host")]
pub async fn build_nns_governance_reward_event_report(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceRewardEventReport, NnsGovernanceError> {
    build_nns_governance_reward_event_report_with_source(request, &LiveNnsSource::default()).await
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
        context: NnsGovernanceReportContext::from_source(request, data.provenance),
        reward_event: data.value,
    })
}

/// Build one live maturity-modulation report through the native replica adapter.
#[cfg(feature = "nns-host")]
pub async fn build_nns_governance_maturity_modulation_report(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceMaturityModulationReport, NnsGovernanceError> {
    build_nns_governance_maturity_modulation_report_with_source(request, &LiveNnsSource::default())
        .await
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
        context: NnsGovernanceReportContext::from_source(request, data.provenance),
        maturity_modulation: data.value,
    })
}
