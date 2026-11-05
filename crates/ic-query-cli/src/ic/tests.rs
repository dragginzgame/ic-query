use super::*;
use crate::cli::clap::render_help;
use std::ffi::OsString;

#[test]
fn usage_discloses_dashboard_command_families() {
    let usage = render_help(command());

    assert!(usage.contains("Usage: ic [COMMAND]"));
    assert!(usage.contains("api-boundary-node"));
    assert!(usage.contains("canister"));
    assert!(usage.contains("metrics"));
    assert!(usage.contains("network"));
    assert!(usage.contains("replica-version"));
}

#[tokio::test(flavor = "current_thread")]
async fn family_and_nested_help_return_without_network_calls() {
    for args in [
        &["ic", "--help"][..],
        &["ic", "api-boundary-node", "--help"],
        &["ic", "api-boundary-node", "list", "--help"],
        &["ic", "canister", "--help"],
        &["ic", "canister", "info", "--help"],
        &["ic", "canister", "count", "--help"],
        &["ic", "canister", "page", "--help"],
        &["ic", "metrics", "--help"],
        &["ic", "network", "--help"],
        &["ic", "network", "boundary-node-data-centers", "--help"],
        &["ic", "network", "daily-stats", "--help"],
        &["ic", "replica-version", "--help"],
        &["ic", "replica-version", "info", "--help"],
        &["ic", "replica-version", "list", "--help"],
    ] {
        assert!(crate::run(args.iter().map(OsString::from)).await.is_ok());
    }
}

pub(super) fn parse_test_options<Options>(
    command: ClapCommand,
    args: &[&str],
    from_matches: fn(&ArgMatches) -> Options,
) -> Result<Options, IcCommandError> {
    let matches = crate::cli::parse_test_args(command, args)
        .map_err(|error| IcCommandError::Usage(error.to_string()))?;
    Ok(from_matches(&matches))
}
