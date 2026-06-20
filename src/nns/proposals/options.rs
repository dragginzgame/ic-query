//! Module: nns::proposals::options
//!
//! Responsibility: parse NNS proposal command options.
//! Does not own: clap command construction, report requests, or live queries.
//! Boundary: converts clap matches into command-local option structs.

use super::commands::{
    nns_proposal_cache_list_command, nns_proposal_cache_list_usage_for_error,
    nns_proposal_cache_status_command, nns_proposal_cache_status_usage_for_error,
    nns_proposal_info_command, nns_proposal_info_usage_for_error, nns_proposal_list_command,
    nns_proposal_list_usage_for_error, nns_proposal_refresh_command,
    nns_proposal_refresh_usage_for_error,
};
use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option},
        common::FORMAT_ARG,
    },
    nns::{
        NnsCommandError, OutputFormat,
        leaf::NnsCommonOptions,
        parse_nns_matches,
        proposals::{
            report::{
                NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_DESC_LABEL, NnsProposalListSort,
                NnsProposalRewardStatusFilter, NnsProposalSortDirection, NnsProposalStatusFilter,
                NnsProposalTopicFilter,
            },
            values::{
                NNS_PROPOSAL_BALLOTS_FLAG, NNS_PROPOSAL_ID_ARG,
                NNS_PROPOSAL_LIST_LOCAL_SORT_VALUE_NAME, NNS_PROPOSAL_LIST_REWARD_STATUS_ARG,
                NNS_PROPOSAL_VERBOSE_FLAG, NnsProposalListSortArg, NnsProposalRewardStatusArg,
                NnsProposalStatusArg, NnsProposalTopicArg,
            },
        },
    },
};
use clap::ArgMatches;
use clap::Command as ClapCommand;
use std::ffi::OsString;

///
/// NnsProposalListOptions
///
/// Options accepted by `icq nns proposal list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalListOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) limit: u32,
    pub(in crate::nns) before_proposal_id: Option<u64>,
    pub(in crate::nns) status: NnsProposalStatusFilter,
    pub(in crate::nns) reward_status: NnsProposalRewardStatusFilter,
    pub(in crate::nns) topic: NnsProposalTopicFilter,
    pub(in crate::nns) proposer_neuron_id: Option<u64>,
    pub(in crate::nns) sort: NnsProposalListSort,
    pub(in crate::nns) sort_direction: NnsProposalSortDirection,
    pub(in crate::nns) verbose: bool,
}

impl NnsProposalListOptions {
    pub(in crate::nns) fn parse_list<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        Self::parse_with(
            args,
            nns_proposal_list_command(),
            nns_proposal_list_usage_for_error,
        )
    }

    fn parse_with<I>(
        args: I,
        command: ClapCommand,
        usage: impl FnOnce() -> String,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(command, args, usage)?;
        let common = NnsCommonOptions::from_matches(&matches);
        let sort = required_typed::<NnsProposalListSortArg>(&matches, "sort");
        let sort_direction = proposal_sort_direction(&matches, sort)?;
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            limit: required_typed(&matches, "limit"),
            before_proposal_id: typed_option(&matches, "before"),
            status: required_typed::<NnsProposalStatusArg>(&matches, "status").into(),
            reward_status: required_typed::<NnsProposalRewardStatusArg>(
                &matches,
                NNS_PROPOSAL_LIST_REWARD_STATUS_ARG,
            )
            .into(),
            topic: required_typed::<NnsProposalTopicArg>(&matches, "topic").into(),
            proposer_neuron_id: typed_option(&matches, "proposer"),
            sort: sort.into(),
            sort_direction,
            verbose: matches.get_flag(NNS_PROPOSAL_VERBOSE_FLAG),
        })
    }
}

fn proposal_sort_direction(
    matches: &ArgMatches,
    sort: NnsProposalListSortArg,
) -> Result<NnsProposalSortDirection, NnsCommandError> {
    let sort = NnsProposalListSort::from(sort);
    if matches.get_flag(NNS_PROPOSAL_SORT_ASC_LABEL) {
        return explicit_proposal_sort_direction(
            sort,
            NnsProposalSortDirection::Asc,
            NNS_PROPOSAL_SORT_ASC_LABEL,
        );
    }
    if matches.get_flag(NNS_PROPOSAL_SORT_DESC_LABEL) {
        return explicit_proposal_sort_direction(
            sort,
            NnsProposalSortDirection::Desc,
            NNS_PROPOSAL_SORT_DESC_LABEL,
        );
    }
    Ok(sort.default_direction())
}

fn explicit_proposal_sort_direction(
    sort: NnsProposalListSort,
    direction: NnsProposalSortDirection,
    flag: &'static str,
) -> Result<NnsProposalSortDirection, NnsCommandError> {
    if !sort.uses_local_direction() {
        return Err(NnsCommandError::Usage(format!(
            "--{flag} requires --sort {NNS_PROPOSAL_LIST_LOCAL_SORT_VALUE_NAME}"
        )));
    }
    Ok(direction)
}

///
/// NnsProposalOptions
///
/// Options accepted by `icq nns proposal info`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) proposal_id: u64,
    pub(in crate::nns) show_ballots: bool,
    pub(in crate::nns) verbose: bool,
}

impl NnsProposalOptions {
    pub(in crate::nns) fn parse_info<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        Self::parse_with(
            args,
            nns_proposal_info_command(),
            nns_proposal_info_usage_for_error,
        )
    }

    fn parse_with<I>(
        args: I,
        command: ClapCommand,
        usage: impl FnOnce() -> String,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(command, args, usage)?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            proposal_id: required_typed(&matches, NNS_PROPOSAL_ID_ARG),
            show_ballots: matches.get_flag(NNS_PROPOSAL_BALLOTS_FLAG),
            verbose: matches.get_flag(NNS_PROPOSAL_VERBOSE_FLAG),
        })
    }
}

///
/// NnsProposalRefreshOptions
///
/// Options accepted by `icq nns proposal refresh`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalRefreshOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) page_size: u32,
    pub(in crate::nns) max_pages: Option<u32>,
}

impl NnsProposalRefreshOptions {
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(
            nns_proposal_refresh_command(),
            args,
            nns_proposal_refresh_usage_for_error,
        )?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            page_size: required_typed(&matches, "page-size"),
            max_pages: typed_option(&matches, "max-pages"),
        })
    }
}

///
/// NnsProposalCacheOptions
///
/// Options accepted by `icq nns proposal cache list/status`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalCacheOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
}

impl NnsProposalCacheOptions {
    pub(in crate::nns) fn parse_list<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        Self::parse_with(
            args,
            nns_proposal_cache_list_command(),
            nns_proposal_cache_list_usage_for_error,
        )
    }

    pub(in crate::nns) fn parse_status<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        Self::parse_with(
            args,
            nns_proposal_cache_status_command(),
            nns_proposal_cache_status_usage_for_error,
        )
    }

    fn parse_with<I>(
        args: I,
        command: ClapCommand,
        usage: impl FnOnce() -> String,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(command, args, usage)?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, FORMAT_ARG),
        })
    }
}
