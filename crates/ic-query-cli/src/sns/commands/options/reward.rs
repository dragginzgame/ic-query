//! Parse live reward-checkpoint requests at the CLI boundary.

use crate::{
    cli::{clap::typed_option, common::OutputFormat},
    sns::commands::options::SnsLookupOptions,
};
use clap::ArgMatches;
use ic_query::sns::SnsRewardCheckpointRequest;

pub(in crate::sns::commands) fn reward_checkpoint_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, SnsRewardCheckpointRequest) {
    let (format, lookup) =
        SnsLookupOptions::from_matches(matches, network).into_request(now_unix_secs);
    (
        format,
        SnsRewardCheckpointRequest::new(lookup).with_max_pages(typed_option(matches, "max-pages")),
    )
}
