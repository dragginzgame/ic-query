use super::super::{
    SnsHostError, SnsProposalReport, SnsProposalRequest, SnsProposalsReport, SnsProposalsRequest,
    assemble::{
        SnsProposalReportParts, SnsProposalsReportParts, sns_proposal_report_from_parts,
        sns_proposals_report_from_parts,
    },
    live::LiveSnsSource,
    lookup::{lookup_request_from_parts, resolve_sns_lookup},
    proposals_cache::{
        build_sns_proposal_report_from_cache, build_sns_proposals_report_from_cache_or_refresh,
    },
    source::{SnsProposalSource, SnsProposalsSource},
};

pub fn build_sns_proposal_report(
    request: &SnsProposalRequest,
) -> Result<SnsProposalReport, SnsHostError> {
    build_sns_proposal_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_proposals_report(
    request: &SnsProposalsRequest,
) -> Result<SnsProposalsReport, SnsHostError> {
    build_sns_proposals_report_with_source(request, &LiveSnsSource)
}

pub(in crate::sns::report) fn build_sns_proposal_report_with_source(
    request: &SnsProposalRequest,
    source: &dyn SnsProposalSource,
) -> Result<SnsProposalReport, SnsHostError> {
    if let Some(icp_root) = request.icp_root.as_ref()
        && let Some(report) = build_sns_proposal_report_from_cache(request, icp_root)?
    {
        return Ok(report);
    }
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let proposal =
        source.fetch_sns_proposal(&lookup.fetch_request, &lookup.sns, request.proposal_id)?;
    Ok(sns_proposal_report_from_parts(SnsProposalReportParts {
        list: lookup.list,
        id: lookup.id,
        sns: lookup.sns,
        proposal_id: request.proposal_id,
        verbose: request.verbose,
        show_ballots: request.show_ballots,
        proposal,
    }))
}

pub(in crate::sns::report) fn build_sns_proposals_report_with_source(
    request: &SnsProposalsRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    if let Some(icp_root) = request.icp_root.as_ref()
        && request_can_use_proposals_cache(request)
    {
        return build_sns_proposals_report_from_cache_or_refresh(request, icp_root, source);
    }
    build_sns_proposals_report_live(request, source)
}

fn build_sns_proposals_report_live(
    request: &SnsProposalsRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let include_status = request
        .status
        .governance_status_code()
        .into_iter()
        .collect::<Vec<_>>();
    let proposals = source.fetch_sns_proposals(
        &lookup.fetch_request,
        &lookup.sns,
        request.limit,
        request.before_proposal_id,
        &include_status,
        request.topic,
    )?;
    Ok(sns_proposals_report_from_parts(SnsProposalsReportParts {
        list: lookup.list,
        id: lookup.id,
        sns: lookup.sns,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        topic: request.topic,
        verbose: request.verbose,
        proposals,
    }))
}

const fn request_can_use_proposals_cache(request: &SnsProposalsRequest) -> bool {
    if !matches!(request.topic, super::super::SnsProposalTopicFilter::Any) {
        return false;
    }
    matches!(
        request.status,
        super::super::SnsProposalStatusFilter::Any
            | super::super::SnsProposalStatusFilter::Open
            | super::super::SnsProposalStatusFilter::Executed
            | super::super::SnsProposalStatusFilter::Failed
    )
}
