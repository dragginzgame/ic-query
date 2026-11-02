//! Module: nns::proposals::options
//!
//! Responsibility: parse NNS proposal command options.
//! Does not own: clap command construction, live queries, or output.
//! Boundary: constructs public requests from clap matches.

use crate::{
    cli::{
        clap::{required_typed, string_option, typed_option},
        common::{output_format, resolve_proposal_sort_direction},
    },
    nns::{
        NnsCommandError, OutputFormat,
        leaf::NnsCommonOptions,
        proposals::values::{
            NNS_PROPOSAL_BALLOTS_FLAG, NNS_PROPOSAL_ID_ARG, NNS_PROPOSAL_LIST_REWARD_STATUS_ARG,
            NNS_PROPOSAL_VERBOSE_FLAG,
        },
    },
};
use clap::ArgMatches;
use ic_query::nns::proposals::{
    NnsProposalActivityRequest, NnsProposalListRequest, NnsProposalListSort, NnsProposalRequest,
    NnsProposalRewardStatusFilter, NnsProposalStatusFilter, NnsProposalTopicFilter,
};

pub(in crate::nns) fn proposal_activity_request(
    matches: &ArgMatches,
) -> (OutputFormat, NnsProposalActivityRequest) {
    (
        output_format(matches),
        NnsProposalActivityRequest {
            from_proposal_timestamp_seconds: typed_option(matches, "from"),
            until_proposal_timestamp_seconds: typed_option(matches, "until"),
        },
    )
}

pub(in crate::nns) fn proposal_list_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> Result<(OutputFormat, NnsProposalListRequest), NnsCommandError> {
    let common = NnsCommonOptions::from_matches(matches, network);
    let sort = required_typed::<NnsProposalListSort>(matches, "sort");
    let sort_direction = resolve_proposal_sort_direction(
        matches,
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
            limit: required_typed(matches, "limit"),
            before_proposal_id: typed_option(matches, "before"),
            status: required_typed::<NnsProposalStatusFilter>(matches, "status"),
            reward_status: required_typed::<NnsProposalRewardStatusFilter>(
                matches,
                NNS_PROPOSAL_LIST_REWARD_STATUS_ARG,
            ),
            topic: required_typed::<NnsProposalTopicFilter>(matches, "topic"),
            proposer_neuron_id: typed_option(matches, "proposer"),
            query: string_option(matches, "query"),
            sort,
            sort_direction,
            verbose: matches.get_flag(NNS_PROPOSAL_VERBOSE_FLAG),
        },
    ))
}

pub(in crate::nns) fn proposal_info_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, NnsProposalRequest) {
    let (format, governance) =
        NnsCommonOptions::from_matches(matches, network).into_governance_request(now_unix_secs);
    (
        format,
        NnsProposalRequest {
            governance,
            proposal_id: required_typed(matches, NNS_PROPOSAL_ID_ARG),
            show_ballots: matches.get_flag(NNS_PROPOSAL_BALLOTS_FLAG),
            verbose: matches.get_flag(NNS_PROPOSAL_VERBOSE_FLAG),
        },
    )
}
