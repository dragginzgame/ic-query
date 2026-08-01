use super::*;
use ic_query::nns::governance::DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT;

#[test]
fn nns_governance_options_are_shared_across_reports() {
    let defaults = NnsGovernanceOptions::parse([], governance_economics_command())
        .expect("economics defaults");
    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT
    );

    let explicit = NnsGovernanceOptions::parse(
        [
            OsString::from("--json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://example.test"),
        ],
        governance_metrics_command(),
    )
    .expect("metrics options");
    assert_eq!(explicit.format, OutputFormat::Json);
    assert_eq!(explicit.source_endpoint, "https://example.test");
}

#[test]
fn nns_governance_help_advertises_native_live_reports() {
    assert!(usage().contains("governance"));
    let family = governance_usage();
    assert!(family.contains("economics"));
    assert!(family.contains("metrics"));
    assert!(family.contains("reward-event"));
    assert!(family.contains("maturity-modulation"));

    for help in [
        governance_economics_usage(),
        governance_metrics_usage(),
        governance_reward_event_usage(),
        governance_maturity_modulation_usage(),
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
        let options = NnsGovernanceOptions::parse([OsString::from("--json")], command)
            .expect("shared report options");
        assert_eq!(options.format, OutputFormat::Json);
    }
}
