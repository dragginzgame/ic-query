//! Module: sns::commands::spec::commands::neurons::refresh
//!
//! Responsibility: build the clap spec for SNS neuron snapshot refresh.
//! Does not own: refresh execution, progress reporting, or cache storage.
//! Boundary: defines refresh limits, source endpoint option, and examples.

use crate::sns::commands::spec::commands::args::sns_snapshot_refresh_command;
use clap::Command as ClapCommand;

const SNS_NEURONS_REFRESH_DEFAULT_PAGE_SIZE: &str = "100";

const SNS_NEURONS_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq sns neuron refresh 1
  icq sns neuron refresh be2us-64aaa-aaaaa-qaabq-cai
  icq sns neuron refresh 1 --page-size 100
  icq --network ic sns neuron refresh 1 --json";

pub(in crate::sns::commands) fn sns_neuron_refresh_command() -> ClapCommand {
    sns_snapshot_refresh_command(
        ClapCommand::new("refresh")
            .bin_name("icq sns neuron refresh")
            .about("Force-refresh and cache a complete SNS governance neuron snapshot"),
        SNS_NEURONS_REFRESH_DEFAULT_PAGE_SIZE,
        "Maximum neurons to request per SNS governance page",
        SNS_NEURONS_REFRESH_HELP_AFTER,
    )
}
