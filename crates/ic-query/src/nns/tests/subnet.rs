use super::*;

#[test]
fn list_defaults_to_mainnet_ic_catalog() {
    let options = CatalogListOptions::parse([]).expect("parse list");

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
    let options = CatalogListOptions::parse([
        OsString::from("--kind"),
        OsString::from("application"),
        OsString::from("--specialization"),
        OsString::from("fiduciary"),
        OsString::from("--geo"),
        OsString::from("global"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--show-ranges"),
        OsString::from("--verbose"),
        OsString::from("--range-limit"),
        OsString::from("12"),
    ])
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
        CatalogListOptions::parse([OsString::from("--kind"), OsString::from("subnet"),]),
        Err(NnsCommandError::Usage(_))
    ));
    assert!(matches!(
        CatalogListOptions::parse([OsString::from("--range-limit"), OsString::from("0"),]),
        Err(NnsCommandError::Usage(_))
    ));
    assert!(matches!(
        CatalogInfoOptions::parse([
            OsString::from("aaaaa-aa"),
            OsString::from("--as"),
            OsString::from("route"),
        ]),
        Err(NnsCommandError::Usage(_))
    ));
}

#[test]
fn info_usage_names_subnet_lookup_input() {
    let text = info_usage();

    assert!(text.contains("subnet|canister|subnet-prefix"));
    assert!(text.contains("unique subnet prefix"));
    assert!(text.contains("icq nns subnet info <subnet-prefix>"));
    assert!(text.contains("--as <subnet|canister>"));
}

#[test]
fn list_and_info_help_hide_stale_policy_knobs() {
    let list = list_usage();
    let info = info_usage();

    assert!(!list.contains("--stale-after"));
    assert!(!list.contains("--allow-stale-subnet-catalog"));
    assert!(!info.contains("--stale-after"));
    assert!(!info.contains("--allow-stale-subnet-catalog"));
}

#[test]
fn refresh_parses_defaults_and_export_options() {
    let options = CatalogRefreshOptions::parse([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--lock-stale-after"),
        OsString::from("5m"),
        OsString::from("--dry-run"),
        OsString::from("--output"),
        OsString::from("catalog.preview.json"),
    ])
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
fn catalog_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("subnet"),
        OsString::from("list"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns subnet list"));
}

#[test]
fn refresh_is_advertised_as_subnet_command() {
    let text = subnet_usage();

    assert!(text.contains("refresh"));
    assert!(refresh_usage().contains("icq nns subnet refresh"));
}

#[test]
fn nns_namespace_help_mentions_subnet() {
    let text = usage();

    assert!(text.contains("Inspect NNS metadata"));
    assert!(text.contains("subnet"));
    assert!(!text.contains("Inspect cached IC network subnet metadata"));
}
