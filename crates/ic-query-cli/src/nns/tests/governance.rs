use super::*;
use ic_query::nns::governance::DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT;

#[test]
fn nns_governance_options_are_shared_across_reports() {
    let defaults = NnsGovernanceOptions::parse(
        [],
        governance_economics_command(),
        governance_economics_usage,
    )
    .expect("economics defaults");
    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT
    );

    let explicit = NnsGovernanceOptions::parse(
        [
            OsString::from("--format"),
            OsString::from("json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://example.test"),
        ],
        governance_metrics_command(),
        governance_metrics_usage,
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
        assert!(help.contains("--format"));
    }
}

#[test]
fn nns_governance_non_mainnet_is_rejected_before_live_query() {
    for command in [
        "economics",
        "metrics",
        "reward-event",
        "maturity-modulation",
    ] {
        let error = run([
            OsString::from("governance"),
            OsString::from(command),
            OsString::from("--__icq-network"),
            OsString::from("local"),
        ])
        .expect_err("local network must be rejected");

        let message = error.to_string();
        assert!(message.contains("supports only the mainnet `ic` network"));
        assert!(message.contains("icq --network ic nns governance economics"));
    }
}

#[test]
fn nns_governance_each_report_command_accepts_common_options() {
    for (command, usage) in [
        (
            governance_economics_command(),
            governance_economics_usage as fn() -> String,
        ),
        (
            governance_metrics_command(),
            governance_metrics_usage as fn() -> String,
        ),
        (
            governance_reward_event_command(),
            governance_reward_event_usage as fn() -> String,
        ),
        (
            governance_maturity_modulation_command(),
            governance_maturity_modulation_usage as fn() -> String,
        ),
    ] {
        let options = NnsGovernanceOptions::parse(
            [OsString::from("--format"), OsString::from("json")],
            command,
            usage,
        )
        .expect("shared report options");
        assert_eq!(options.format, OutputFormat::Json);
    }
}
