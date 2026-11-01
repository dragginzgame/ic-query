//! Module: sns::commands::options::list
//!
//! Responsibility: parse options for `icq sns list`.
//! Does not own: deployed SNS lookup, report construction, or text output.
//! Boundary: converts clap matches into the SNS list request inputs.

use crate::{
    cli::{
        clap::{required_string, required_typed},
        common::{OutputFormat, output_format},
    },
    sns::commands::spec::SnsListSortArg,
};
use clap::ArgMatches;

pub(in crate::sns::commands) fn list_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, ic_query::sns::SnsListRequest) {
    (
        output_format(matches),
        ic_query::sns::SnsListRequest {
            network: network.to_string(),
            source_endpoint: required_string(matches, "source-endpoint"),
            now_unix_secs,
            all_lifecycles: matches.get_flag("all"),
            verbose: matches.get_flag("verbose"),
            sort: required_typed::<SnsListSortArg>(matches, "sort").into(),
        },
    )
}
