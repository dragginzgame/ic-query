//! Module: nns::proposals::options
//!
//! Responsibility: parse NNS proposal command options.
//! Does not own: clap command construction, live queries, or output.
//! Boundary: constructs public requests from clap matches.

use super::commands::{ProposalActivityOptions, ProposalInfoOptions, ProposalListOptions};
use crate::cli::clap::parsed_options;

use crate::{
    cli::common::resolve_proposal_sort_direction,
    nns::{NnsCommandError, OutputFormat, leaf::NnsCommonOptions},
};
use clap::ArgMatches;
use ic_query::nns::proposals::NnsProposalActivityRequest;
use ic_query::nns::proposals::NnsProposalListRequest;
use ic_query::nns::proposals::NnsProposalListSort;
use ic_query::nns::proposals::NnsProposalRequest;

pub(in crate::nns) fn proposal_activity_request(
    matches: &ArgMatches,
) -> (OutputFormat, NnsProposalActivityRequest) {
    let options: ProposalActivityOptions = parsed_options(matches);
    (
        OutputFormat::from_json(options.json),
        NnsProposalActivityRequest {
            from_proposal_timestamp_seconds: options.from,
            until_proposal_timestamp_seconds: options.until,
        },
    )
}

pub(in crate::nns) fn proposal_list_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> Result<(OutputFormat, NnsProposalListRequest), NnsCommandError> {
    let options: ProposalListOptions = parsed_options(matches);
    let common = NnsCommonOptions::from_source(options.source, network);
    let sort = options.sort;
    let sort_direction = resolve_proposal_sort_direction(
        options.asc,
        options.desc,
        sort.uses_local_direction(),
        sort.default_direction(),
        || {
            NnsProposalListSort::ALL
                .iter()
                .copied()
                .filter(|sort| *sort != NnsProposalListSort::Api)
                .map(NnsProposalListSort::as_str)
                .collect::<Vec<_>>()
                .join("|")
        },
    )
    .map_err(NnsCommandError::Usage)?;
    let (format, governance) = common.into_governance_request(now_unix_secs);
    Ok((
        format,
        NnsProposalListRequest {
            governance,
            limit: options.limit,
            before_proposal_id: options.before,
            status: options.status,
            reward_status: options.reward_status,
            topic: options.topic,
            proposer_neuron_id: options.proposer,
            query: options.query,
            sort,
            sort_direction,
        },
    ))
}

pub(in crate::nns) fn proposal_info_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, NnsProposalRequest) {
    let options: ProposalInfoOptions = parsed_options(matches);
    let (format, governance) = NnsCommonOptions::from_source(options.source, network)
        .into_governance_request(now_unix_secs);
    (
        format,
        NnsProposalRequest {
            governance,
            proposal_id: options.proposal_id,
        },
    )
}
