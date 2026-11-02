//! Parse live reward-checkpoint requests at the CLI boundary.

use crate::{
    cli::{clap::typed_option, common::OutputFormat},
    sns::commands::options::lookup_request,
};
use clap::ArgMatches;
use ic_query::sns::SnsRewardCheckpointRequest;

pub(in crate::sns::commands) fn reward_checkpoint_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> Result<(OutputFormat, SnsRewardCheckpointRequest), crate::sns::commands::SnsCommandError> {
    let (format, lookup) = lookup_request(matches, network, now_unix_secs)?;
    Ok((
        format,
        SnsRewardCheckpointRequest::new(lookup).with_max_pages(typed_option(matches, "max-pages")),
    ))
}
