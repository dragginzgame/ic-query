use super::super::{
    SnsHostError, SnsProposalReport, SnsProposalRequest, SnsProposalsReport, SnsProposalsRequest,
    assemble::{
        SnsProposalReportParts, SnsProposalsReportParts, sns_proposal_report_from_parts,
        sns_proposals_report_from_parts,
    },
    live::LiveSnsSource,
    lookup::{lookup_request_from_parts, resolve_sns_lookup},
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
