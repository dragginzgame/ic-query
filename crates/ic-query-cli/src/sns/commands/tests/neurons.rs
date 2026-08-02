use super::*;

#[test]
fn sns_neurons_parses_owner_limit_and_json_format() {
    let options = parse_fallible_test_options(
        sns_neuron_list_command(),
        &[
            "1",
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--limit",
            "10",
            "--owner",
            "bkyz2-fmaaa-aaaaa-qaaaq-cai",
            "--sort",
            "api",
            "--verbose",
        ],
        SnsNeuronsOptions::from_matches,
    )
    .expect("parse neurons");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.limit, 10);
    assert_eq!(
        options.owner_principal_id.as_deref(),
        Some("bkyz2-fmaaa-aaaaa-qaaaq-cai")
    );
    assert_eq!(options.sort, SnsNeuronsSortArg::Api);
    assert!(options.verbose);
}

#[test]
fn sns_neurons_allows_large_limits_for_cached_sorts() {
    let options = parse_fallible_test_options(
        sns_neuron_list_command(),
        &["22", "--limit", "500", "--sort", "stake"],
        SnsNeuronsOptions::from_matches,
    )
    .expect("parse cached neurons sort");

    assert_eq!(options.lookup.input, "22");
    assert_eq!(options.limit, 500);
    assert_eq!(options.sort, SnsNeuronsSortArg::Stake);
}

#[test]
fn sns_neurons_refresh_parses_page_controls() {
    let options = parse_test_options(
        sns_neuron_refresh_command(),
        &[
            "1",
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--page-size",
            "50",
            "--max-pages",
            "3",
        ],
        SnsNeuronsRefreshOptions::from_matches,
    )
    .expect("parse neurons refresh");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.page_size, 50);
    assert_eq!(options.max_pages, Some(3));
}

#[test]
fn sns_neurons_cache_parses_list_and_status_options() {
    let list = parse_test_options(
        sns_neuron_cache_list_command(),
        &["--json"],
        SnsNeuronsCacheListOptions::from_matches,
    )
    .expect("parse cache list");

    assert_eq!(list.network, "ic");
    assert_eq!(list.format, OutputFormat::Json);

    let status = parse_test_options(
        sns_neuron_cache_status_command(),
        &["1", "--json"],
        SnsNeuronsCacheStatusOptions::from_matches,
    )
    .expect("parse cache status");

    assert_eq!(status.input, "1");
    assert_eq!(status.network, "ic");
    assert_eq!(status.format, OutputFormat::Json);
}
