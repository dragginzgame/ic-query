//! Module: sns::commands::spec::commands::neurons::refresh
//!
//! Responsibility: build the clap spec for SNS neuron snapshot refresh.
//! Does not own: refresh execution, progress reporting, or cache storage.
//! Boundary: defines refresh limits, source endpoint option, and examples.

use crate::cli::clap::with_options;
use clap::Command as ClapCommand;

const SNS_NEURONS_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq sns neuron refresh <governance-canister-id>
  icq sns neuron refresh <governance-canister-id>
  icq sns neuron refresh <governance-canister-id> --page-size 100
  icq --network ic sns neuron refresh <governance-canister-id> --json";

pub(in crate::sns::commands) fn sns_neuron_refresh_command() -> ClapCommand {
    with_options::<crate::sns::commands::options::SnsRefreshOptions>(
        ClapCommand::new("refresh")
            .about("Force-refresh and cache a complete SNS governance neuron snapshot"),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT)
            .help("IC API endpoint used for Governance neuron queries")
    })
    .after_help(crate::cli::common::collection_help(
        crate::cli::common::COLLECTION_MODE_FORCE_REFRESH,
        SNS_NEURONS_REFRESH_HELP_AFTER,
    ))
}
