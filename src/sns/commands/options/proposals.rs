use super::{common::parse_sns_matches, lookup::SnsLookupOptions};
use crate::{
    cli::clap::{required_typed, typed_option},
    sns::commands::{
        SnsCommandError,
        spec::{
            SnsProposalStatusArg, SnsProposalTopicArg, sns_proposal_command, sns_proposal_usage,
            sns_proposals_command, sns_proposals_usage,
        },
    },
};
use std::ffi::OsString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalsOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) limit: u32,
    pub(in crate::sns::commands) before_proposal_id: Option<u64>,
    pub(in crate::sns::commands) status: SnsProposalStatusArg,
    pub(in crate::sns::commands) topic: SnsProposalTopicArg,
    pub(in crate::sns::commands) verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) proposal_id: u64,
    pub(in crate::sns::commands) verbose: bool,
    pub(in crate::sns::commands) show_ballots: bool,
}

impl SnsProposalsOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_proposals_command(), args, sns_proposals_usage)?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            limit: required_typed(&matches, "limit"),
            before_proposal_id: typed_option::<u64>(&matches, "before"),
            status: required_typed(&matches, "status"),
            topic: required_typed(&matches, "topic"),
            verbose: matches.get_flag("verbose"),
        })
    }
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
