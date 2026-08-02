use super::*;
use crate::cli::clap::render_help;
use ic_query::nns::neuron::{DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, NNS_NEURON_MAX_PAGE_SIZE};

#[test]
fn nns_neuron_options_parse_defaults_and_explicit_values() {
    let list = parse_test_options(
        neuron_list_command(),
        &[],
        NnsNeuronListOptions::from_matches,
    )
    .expect("list defaults");
    assert_eq!(list.network, MAINNET_NETWORK);
    assert_eq!(list.format, OutputFormat::Text);
    assert_eq!(list.source_endpoint, DEFAULT_NNS_NEURON_SOURCE_ENDPOINT);
    assert_eq!(list.limit, 25);
    assert_eq!(list.start_neuron_id, None);
    assert!(!list.verbose);

    let max_page_size = NNS_NEURON_MAX_PAGE_SIZE.to_string();
    let list = parse_test_options(
        neuron_list_command(),
        &[
            "--limit",
            &max_page_size,
            "--start-neuron-id",
            "123",
            "--verbose",
            "--json",
        ],
        NnsNeuronListOptions::from_matches,
    )
    .expect("explicit list options");
    assert_eq!(list.limit, NNS_NEURON_MAX_PAGE_SIZE);
    assert_eq!(list.start_neuron_id, Some(123));
    assert!(list.verbose);
    assert_eq!(list.format, OutputFormat::Json);

    let info = parse_test_options(
        neuron_info_command(),
        &["456", "--verbose"],
        NnsNeuronInfoOptions::from_matches,
    )
    .expect("info options");
    assert_eq!(info.neuron_id, 456);
    assert!(info.verbose);

    let refresh = parse_test_options(
        neuron_refresh_command(),
        &["--page-size", "100", "--max-pages", "2"],
        NnsNeuronRefreshOptions::from_matches,
    )
    .expect("refresh options");
    assert_eq!(refresh.page_size, 100);
    assert_eq!(refresh.max_pages, Some(2));

    let cache = parse_test_options(
        neuron_cache_status_command(),
        &["--json"],
        NnsNeuronCacheOptions::from_matches,
    )
    .expect("cache options");
    assert_eq!(cache.network, MAINNET_NETWORK);
    assert_eq!(cache.format, OutputFormat::Json);
}

#[test]
fn nns_neuron_rejects_invalid_numeric_values() {
    assert!(
        parse_test_options(
            neuron_list_command(),
            &["--limit", "0"],
            NnsNeuronListOptions::from_matches,
        )
        .is_err()
    );
    assert!(
        parse_test_options(
            neuron_refresh_command(),
            &["--page-size", "301"],
            NnsNeuronRefreshOptions::from_matches,
        )
        .is_err()
    );
    assert!(
        parse_test_options(
            neuron_info_command(),
            &["0"],
            NnsNeuronInfoOptions::from_matches,
        )
        .is_err()
    );
}

#[test]
fn nns_neuron_help_advertises_collection_modes_and_commands() {
    assert!(render_help(command()).contains("neuron"));
    let family = render_help(neuron_command());
    assert!(family.contains("list"));
    assert!(family.contains("info"));
    assert!(family.contains("refresh"));
    assert!(family.contains("cache"));
    assert!(render_help(neuron_list_command()).contains("Cache-preferred read"));
    assert!(render_help(neuron_info_command()).contains("Cache-preferred read"));
    assert!(render_help(neuron_refresh_command()).contains("Forced live refresh"));
    assert!(render_help(neuron_cache_command()).contains("Local cache inspection"));
    assert!(render_help(neuron_cache_status_command()).contains("does not make a network request"));
}
