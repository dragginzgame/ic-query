use super::*;
use crate::cli::clap::render_help;

#[test]
fn sns_reward_checkpoint_options_preserve_live_controls() {
    let options = parse_test_options(
        sns_reward_checkpoint_command(),
        &[
            "1",
            "--max-pages",
            "7",
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
        ],
        SnsRewardCheckpointOptions::from_matches,
    )
    .expect("reward checkpoint options");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.max_pages, Some(7));
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
}

#[test]
fn sns_reward_help_is_explicit_about_bounded_collection() {
    let sns = render_help(sns_command());
    let reward = render_help(sns_reward_command());
    let checkpoint = render_help(sns_reward_checkpoint_command());
    let diff = render_help(sns_reward_diff_command());

    assert!(sns.contains("reward"));
    assert!(sns.contains("Collect and compare SNS maturity reward evidence"));
    assert!(reward.contains("checkpoint"));
    assert!(reward.contains("diff"));
    assert!(checkpoint.contains("icq sns reward checkpoint"));
    assert!(checkpoint.contains("Collection mode: Live query"));
    assert!(checkpoint.contains("--max-pages"));
    assert!(checkpoint.contains("--json"));
    assert!(checkpoint.contains("--source-endpoint"));
    assert!(checkpoint.contains("N + 8 client queries"));
    assert!(checkpoint.contains("does not query"));
    assert!(diff.contains("icq sns reward diff"));
    assert!(diff.contains("Collection mode: Local-only file inspection"));
    assert!(diff.contains("--json"));
    assert!(!diff.contains("--source-endpoint"));
    assert!(!diff.contains("--max-pages"));
    assert!(diff.contains("performs no live calls"));
}

#[test]
fn sns_reward_diff_options_preserve_local_paths_and_format() {
    let matches = parse_test_matches(
        sns_reward_diff_command(),
        &["before.json", "after.json", "--json"],
    )
    .expect("reward diff matches");
    let options = SnsRewardDiffOptions::from_matches(&matches);

    assert_eq!(
        options.before_checkpoint,
        std::path::Path::new("before.json")
    );
    assert_eq!(options.after_checkpoint, std::path::Path::new("after.json"));
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn sns_reward_checkpoint_rejects_zero_page_cap() {
    assert!(matches!(
        parse_test_options(
            sns_reward_checkpoint_command(),
            &["1", "--max-pages", "0"],
            SnsRewardCheckpointOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
}
