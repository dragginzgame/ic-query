//! Module: sns::commands::options::proposals
//!
//! Responsibility: parse SNS proposal detail, list, refresh, and cache options.
//! Does not own: proposal command specs, proposal cache policy, or reports.
//! Boundary: validates clap matches into proposal command request inputs.

use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option},
        common::OutputFormat,
    },
    sns::commands::{
        SnsCommandError,
        options::{common::parse_sns_matches, lookup::SnsLookupOptions},
        spec::{
            SNS_PROPOSALS_LOCAL_SORT_VALUE_NAME, SnsProposalStatusArg, SnsProposalTopicArg,
            SnsProposalsSortArg, sns_proposal_command, sns_proposal_usage,
            sns_proposals_cache_list_command, sns_proposals_cache_list_usage,
            sns_proposals_cache_status_command, sns_proposals_cache_status_usage,
            sns_proposals_command, sns_proposals_refresh_command, sns_proposals_refresh_usage,
            sns_proposals_usage,
        },
    },
    sns::report::{SnsProposalSortDirection, SnsProposalsSort},
};
use clap::ArgMatches;
use std::ffi::OsString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalsOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) limit: u32,
    pub(in crate::sns::commands) before_proposal_id: Option<u64>,
    pub(in crate::sns::commands) status: SnsProposalStatusArg,
    pub(in crate::sns::commands) topic: SnsProposalTopicArg,
    pub(in crate::sns::commands) sort: SnsProposalsSortArg,
    pub(in crate::sns::commands) sort_direction: SnsProposalSortDirection,
    pub(in crate::sns::commands) verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) proposal_id: u64,
    pub(in crate::sns::commands) verbose: bool,
    pub(in crate::sns::commands) show_ballots: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalsCacheListOptions {
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalsCacheStatusOptions {
    pub(in crate::sns::commands) input: String,
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalsRefreshOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) page_size: u32,
    pub(in crate::sns::commands) max_pages: Option<u32>,
}

impl SnsProposalsOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_proposals_command(), args, sns_proposals_usage)?;
        let status = required_typed(&matches, "status");
        let sort = required_typed(&matches, "sort");
        let sort_direction = proposal_sort_direction(&matches, sort)?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            limit: required_typed(&matches, "limit"),
            before_proposal_id: typed_option::<u64>(&matches, "before"),
            status,
            topic: required_typed(&matches, "topic"),
            sort,
            sort_direction,
            verbose: matches.get_flag("verbose"),
        })
    }
}

fn proposal_sort_direction(
    matches: &ArgMatches,
    sort: SnsProposalsSortArg,
) -> Result<SnsProposalSortDirection, SnsCommandError> {
    if matches.get_flag("asc") {
        return explicit_proposal_sort_direction(sort, SnsProposalSortDirection::Asc, "--asc");
    }
    if matches.get_flag("desc") {
        return explicit_proposal_sort_direction(sort, SnsProposalSortDirection::Desc, "--desc");
    }
    Ok(SnsProposalsSort::from(sort).default_direction())
}

fn explicit_proposal_sort_direction(
    sort: SnsProposalsSortArg,
    direction: SnsProposalSortDirection,
    flag: &'static str,
) -> Result<SnsProposalSortDirection, SnsCommandError> {
    if !SnsProposalsSort::from(sort).uses_local_direction() {
        return Err(SnsCommandError::Usage(format!(
            "{flag} requires --sort {SNS_PROPOSALS_LOCAL_SORT_VALUE_NAME}"
        )));
    }
    Ok(direction)
}

impl SnsProposalOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_proposal_command(), args, sns_proposal_usage)?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            proposal_id: required_typed(&matches, "proposal-id"),
            verbose: matches.get_flag("verbose"),
            show_ballots: matches.get_flag("ballots"),
        })
    }
}

impl SnsProposalsCacheListOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(
            sns_proposals_cache_list_command(),
            args,
            sns_proposals_cache_list_usage,
        )?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
        })
    }
}

impl SnsProposalsCacheStatusOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(
            sns_proposals_cache_status_command(),
            args,
            sns_proposals_cache_status_usage,
        )?;
        Ok(Self {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
        })
    }
}

impl SnsProposalsRefreshOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(
            sns_proposals_refresh_command(),
            args,
            sns_proposals_refresh_usage,
        )?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            page_size: required_typed(&matches, "page-size"),
            max_pages: typed_option::<u32>(&matches, "max-pages"),
        })
    }
}
