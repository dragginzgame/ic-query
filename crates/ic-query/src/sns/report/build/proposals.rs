//! Module: sns::report::build::proposals
//!
//! Responsibility: build SNS proposal list and detail reports.
//! Does not own: command parsing, cache file primitives, proposal row conversion, or rendering.
//! Boundary: routes cache-backed requests through cache reports and live reads through sources.

use crate::report::ReportProvenance;
use crate::{
    QueryProgress,
    progress::IgnoreQueryProgress,
    sns::report::{
        SnsHostError, SnsProposalReport, SnsProposalRequest, SnsProposalStatusFilter,
        SnsProposalsReport, SnsProposalsRequest,
        assemble::{
            SnsReportContext, sns_proposal_report_from_parts, sns_proposals_report_from_parts,
        },
        enforce_mainnet_network,
        live::LiveSnsSource,
        lookup::resolve_sns_lookup,
        proposals_cache::{
            build_sns_proposal_report_from_cache, build_sns_proposals_report_from_cache_or_refresh,
        },
        source::{
            SnsProposalSource, SnsProposalsSource, validate_mainnet_sns_proposal,
            validate_mainnet_sns_proposals,
        },
        view::apply_sns_proposal_view,
    },
};

pub async fn build_sns_proposal_report(
    request: &SnsProposalRequest,
) -> Result<SnsProposalReport, SnsHostError> {
    build_sns_proposal_report_with_source(request, &LiveSnsSource::default()).await
}

pub async fn build_sns_proposals_report(
    request: &SnsProposalsRequest,
) -> Result<SnsProposalsReport, SnsHostError> {
    build_sns_proposals_report_with_source(request, &LiveSnsSource::default()).await
}

/// Build an SNS proposal list report and emit cache-refresh progress when needed.
pub async fn build_sns_proposals_report_with_progress(
    request: &SnsProposalsRequest,
    progress: &mut dyn QueryProgress,
) -> Result<SnsProposalsReport, SnsHostError> {
    build_sns_proposals_report_with_source_and_progress(
        request,
        &LiveSnsSource::default(),
        progress,
    )
    .await
}

pub async fn build_sns_proposal_report_with_source(
    request: &SnsProposalRequest,
    source: &dyn SnsProposalSource,
) -> Result<SnsProposalReport, SnsHostError> {
    enforce_mainnet_network(&request.lookup.network)?;
    if let Some(cache_root) = request.cache_root.as_ref()
        && let Some(report) = build_sns_proposal_report_from_cache(request, cache_root)?
    {
        return Ok(report);
    }
    let lookup = resolve_sns_lookup(&request.lookup, source).await?;
    let proposal = source
        .fetch_sns_proposal(&lookup.fetch_request, &lookup.sns, request.proposal_id)
        .await?;
    validate_mainnet_sns_proposal(&proposal, request.proposal_id)?;
    Ok(sns_proposal_report_from_parts(
        request,
        SnsReportContext::from_live(lookup.fetch_request, lookup.sns),
        ReportProvenance::live(),
        proposal,
    ))
}

pub async fn build_sns_proposals_report_with_source(
    request: &SnsProposalsRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    let mut progress = IgnoreQueryProgress;
    build_sns_proposals_report_with_source_and_progress(request, source, &mut progress).await
}

async fn build_sns_proposals_report_with_source_and_progress(
    request: &SnsProposalsRequest,
    source: &dyn SnsProposalsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsProposalsReport, SnsHostError> {
    validate_sns_proposals_request(request)?;
    if let Some(cache_root) = request.cache_root.as_ref() {
        return build_sns_proposals_report_from_cache_or_refresh(
            request, cache_root, source, progress,
        )
        .await;
    }
    build_sns_proposals_report_live(request, source).await
}

fn validate_sns_proposals_request(request: &SnsProposalsRequest) -> Result<(), SnsHostError> {
    enforce_mainnet_network(&request.lookup.network)?;
    if request.status == SnsProposalStatusFilter::Decided && request.cache_root.is_none() {
        return Err(SnsHostError::UnsupportedProposalView {
            reason: "`--status decided` requires a complete proposal cache".to_string(),
        });
    }
    Ok(())
}

async fn build_sns_proposals_report_live(
    request: &SnsProposalsRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    let lookup = resolve_sns_lookup(&request.lookup, source).await?;
    let include_status = request
        .status
        .governance_status_code()
        .into_iter()
        .collect::<Vec<_>>();
    let mut proposals = source
        .fetch_sns_proposals(
            &lookup.fetch_request,
            &lookup.sns,
            request.limit,
            request.before_proposal_id,
            &include_status,
            request.topic,
        )
        .await?;
    validate_mainnet_sns_proposals(&proposals, request.limit)?;
    apply_sns_proposal_view(&mut proposals, request);
    Ok(sns_proposals_report_from_parts(
        request,
        SnsReportContext::from_live(lookup.fetch_request, lookup.sns),
        ReportProvenance::live(),
        proposals,
    ))
}
