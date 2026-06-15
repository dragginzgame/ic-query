use super::*;

#[test]
fn sns_neurons_parses_owner_limit_and_json_format() {
    let options = SnsNeuronsOptions::parse([
        OsString::from("1"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--limit"),
        OsString::from("10"),
        OsString::from("--owner"),
        OsString::from("bkyz2-fmaaa-aaaaa-qaaaq-cai"),
        OsString::from("--sort"),
        OsString::from("api"),
        OsString::from("--verbose"),
    ])
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
    let options = SnsNeuronsOptions::parse([
        OsString::from("22"),
        OsString::from("--limit"),
        OsString::from("500"),
        OsString::from("--sort"),
        OsString::from("stake"),
    ])
    .expect("parse cached neurons sort");

    assert_eq!(options.lookup.input, "22");
    assert_eq!(options.limit, 500);
    assert_eq!(options.sort, SnsNeuronsSortArg::Stake);
}

#[test]
fn sns_neurons_refresh_parses_page_controls() {
    let options = SnsNeuronsRefreshOptions::parse([
        OsString::from("1"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--page-size"),
        OsString::from("50"),
        OsString::from("--max-pages"),
        OsString::from("3"),
    ])
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
    let list =
        SnsNeuronsCacheListOptions::parse([OsString::from("--format"), OsString::from("json")])
            .expect("parse cache list");

    assert_eq!(list.network, "ic");
    assert_eq!(list.format, OutputFormat::Json);

    let status = SnsNeuronsCacheStatusOptions::parse([
        OsString::from("1"),
        OsString::from("--format"),
        OsString::from("json"),
    ])
    .expect("parse cache status");

    assert_eq!(status.input, "1");
    assert_eq!(status.network, "ic");
    assert_eq!(status.format, OutputFormat::Json);
}
