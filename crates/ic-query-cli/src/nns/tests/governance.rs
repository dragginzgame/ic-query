use super::*;
use crate::cli::clap::render_help;
use ic_query::nns::governance::DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT;

#[test]
fn nns_governance_options_are_shared_across_reports() {
    let defaults = parse_test_options(
        governance_economics_command(),
        &[],
        NnsGovernanceOptions::from_matches,
    )
    .expect("economics defaults");
    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT
    );

    let explicit = parse_test_options(
        governance_metrics_command(),
        &["--json", "--source-endpoint", "https://example.test"],
        NnsGovernanceOptions::from_matches,
    )
    .expect("metrics options");
    assert_eq!(explicit.format, OutputFormat::Json);
    assert_eq!(explicit.source_endpoint, "https://example.test");
}

#[test]
fn nns_governance_help_advertises_native_live_reports() {
    assert!(render_help(command()).contains("governance"));
    let family = render_help(governance_command());
    assert!(family.contains("economics"));
    assert!(family.contains("metrics"));
    assert!(family.contains("reward-event"));
    assert!(family.contains("maturity-modulation"));

    for help in [
        render_help(governance_economics_command()),
        render_help(governance_metrics_command()),
        render_help(governance_reward_event_command()),
        render_help(governance_maturity_modulation_command()),
    ] {
        assert!(help.contains("Collection mode: Live query"));
        assert!(help.contains("--source-endpoint"));
        assert!(help.contains("--json"));
    }
}

#[test]
fn nns_governance_each_report_command_accepts_common_options() {
    for command in [
        governance_economics_command(),
        governance_metrics_command(),
        governance_reward_event_command(),
        governance_maturity_modulation_command(),
    ] {
        let options = parse_test_options(command, &["--json"], NnsGovernanceOptions::from_matches)
            .expect("shared report options");
        assert_eq!(options.format, OutputFormat::Json);
    }
}
