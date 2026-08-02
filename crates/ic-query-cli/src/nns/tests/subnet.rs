use super::*;
use crate::cli::clap::render_help;

#[test]
fn list_defaults_to_mainnet_ic_catalog() {
    let options = parse_test_options(list_command(), &[], CatalogListOptions::from_matches)
        .expect("parse list");

    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Text);
    assert_eq!(
        options.source_endpoint,
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT
    );
    assert_eq!(options.range_limit, DEFAULT_RANGE_LIMIT);
    assert!(!options.verbose);
}

#[test]
fn list_parses_filters_and_json_format() {
    let options = parse_test_options(
        list_command(),
        &[
            "--kind",
            "application",
            "--specialization",
            "fiduciary",
            "--geo",
            "global",
            "--json",
            "--show-ranges",
            "--verbose",
            "--range-limit",
            "12",
        ],
        CatalogListOptions::from_matches,
    )
    .expect("parse list");

    assert_eq!(options.filters.kind, Some(SubnetKind::Application));
    assert_eq!(
        options.filters.specialization,
        Some(SubnetSpecialization::Fiduciary)
    );
    assert_eq!(
        options.filters.geographic_scope,
        Some(GeographicScope::Global)
    );
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(
        options.source_endpoint,
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT
    );
    assert!(options.show_ranges);
    assert!(options.verbose);
    assert_eq!(options.range_limit, 12);
}

#[test]
fn clap_rejects_invalid_nns_option_values() {
    assert!(matches!(
        parse_test_options(
            list_command(),
            &["--kind", "subnet"],
            CatalogListOptions::from_matches,
        ),
        Err(NnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_test_options(
            list_command(),
            &["--range-limit", "0"],
            CatalogListOptions::from_matches,
        ),
        Err(NnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_test_options(
            info_command(),
            &["aaaaa-aa", "--as", "route"],
            CatalogInfoOptions::from_matches,
        ),
        Err(NnsCommandError::Usage(_))
    ));
}

#[test]
fn info_usage_names_subnet_lookup_input() {
    let text = render_help(info_command());

    assert!(text.contains("subnet|canister|subnet-prefix"));
    assert!(text.contains("unique subnet prefix"));
    assert!(text.contains("icq nns subnet info <subnet-prefix>"));
    assert!(text.contains("--as <subnet|canister>"));
}

#[test]
fn list_and_info_help_hide_stale_policy_knobs() {
    let list = render_help(list_command());
    let info = render_help(info_command());

    assert!(list.contains("Collection mode: Cache-backed read"));
    assert!(info.contains("Collection mode: Cache-backed read"));
    assert!(!list.contains("--stale-after"));
    assert!(!list.contains("--allow-stale-subnet-catalog"));
    assert!(!info.contains("--stale-after"));
    assert!(!info.contains("--allow-stale-subnet-catalog"));
}

#[test]
fn refresh_parses_defaults_and_export_options() {
    let options = parse_test_options(
        refresh_command(),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--lock-stale-after",
            "5m",
            "--dry-run",
            "--output",
            "catalog.preview.json",
        ],
        CatalogRefreshOptions::from_matches,
    )
    .expect("parse refresh");

    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.lock_stale_after_seconds, 300);
    assert!(options.dry_run);
    assert_eq!(
        options.output_path,
        Some(PathBuf::from("catalog.preview.json"))
    );
}

#[test]
fn refresh_is_advertised_as_subnet_command() {
    let text = render_help(subnet_command());
    let refresh = render_help(refresh_command());

    assert!(text.contains("refresh"));
    assert!(refresh.contains("icq nns subnet refresh"));
    assert!(refresh.contains("Collection mode: Forced live refresh"));
}

#[test]
fn nns_namespace_help_mentions_subnet() {
    let text = usage();

    assert!(text.contains("Inspect NNS metadata"));
    assert!(text.contains("subnet"));
    assert!(!text.contains("Inspect cached IC network subnet metadata"));
}
