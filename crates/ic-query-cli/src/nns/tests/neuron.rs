use super::*;
use ic_query::nns::neuron::{DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, NNS_NEURON_MAX_PAGE_SIZE};

#[test]
fn nns_neuron_options_parse_defaults_and_explicit_values() {
    let list = NnsNeuronListOptions::parse([]).expect("list defaults");
    assert_eq!(list.network, MAINNET_NETWORK);
    assert_eq!(list.format, OutputFormat::Text);
    assert_eq!(list.source_endpoint, DEFAULT_NNS_NEURON_SOURCE_ENDPOINT);
    assert_eq!(list.limit, 25);
    assert_eq!(list.start_neuron_id, None);
    assert!(!list.verbose);

    let list = NnsNeuronListOptions::parse([
        OsString::from("--limit"),
        OsString::from(NNS_NEURON_MAX_PAGE_SIZE.to_string()),
        OsString::from("--start-neuron-id"),
        OsString::from("123"),
        OsString::from("--verbose"),
        OsString::from("--json"),
    ])
    .expect("explicit list options");
    assert_eq!(list.limit, NNS_NEURON_MAX_PAGE_SIZE);
    assert_eq!(list.start_neuron_id, Some(123));
    assert!(list.verbose);
    assert_eq!(list.format, OutputFormat::Json);

    let info = NnsNeuronInfoOptions::parse([OsString::from("456"), OsString::from("--verbose")])
        .expect("info options");
    assert_eq!(info.neuron_id, 456);
    assert!(info.verbose);

    let refresh = NnsNeuronRefreshOptions::parse([
        OsString::from("--page-size"),
        OsString::from("100"),
        OsString::from("--max-pages"),
        OsString::from("2"),
    ])
    .expect("refresh options");
    assert_eq!(refresh.page_size, 100);
    assert_eq!(refresh.max_pages, Some(2));

    let cache = NnsNeuronCacheOptions::parse([OsString::from("--json")]).expect("cache options");
    assert_eq!(cache.network, MAINNET_NETWORK);
    assert_eq!(cache.format, OutputFormat::Json);
}

#[test]
fn nns_neuron_rejects_invalid_numeric_values() {
    assert!(
        NnsNeuronListOptions::parse([OsString::from("--limit"), OsString::from("0"),]).is_err()
    );
    assert!(
        NnsNeuronRefreshOptions::parse([OsString::from("--page-size"), OsString::from("301"),])
            .is_err()
    );
    assert!(NnsNeuronInfoOptions::parse([OsString::from("0")]).is_err());
}

#[test]
fn nns_neuron_help_advertises_collection_modes_and_commands() {
    assert!(usage().contains("neuron"));
    let family = neuron_usage();
    assert!(family.contains("list"));
    assert!(family.contains("info"));
    assert!(family.contains("refresh"));
    assert!(family.contains("cache"));
    assert!(neuron_list_usage().contains("Cache-preferred read"));
    assert!(neuron_info_usage().contains("Cache-preferred read"));
    assert!(neuron_refresh_usage().contains("Forced live refresh"));
    assert!(neuron_cache_usage().contains("Local cache inspection"));
    assert!(neuron_cache_status_usage().contains("does not make a network request"));
}
