use super::*;
use crate::test_support::assert_snapshot;

#[test]
fn sns_list_parses_defaults_and_json_format() {
    let defaults = SnsListOptions::parse([]).expect("parse defaults");
    assert_eq!(defaults.network, "ic");
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
    assert_eq!(defaults.sort, SnsListSortArg::Id);
    assert!(!defaults.verbose);

    let options = SnsListOptions::parse([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--sort"),
        OsString::from("name"),
        OsString::from("--verbose"),
    ])
    .expect("parse list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.sort, SnsListSortArg::Name);
    assert!(options.verbose);
}

#[test]
fn sns_info_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--format"),
            OsString::from("json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_info_command,
        sns_info_usage,
    )
    .expect("parse info");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_token_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--format"),
            OsString::from("json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_token_command,
        sns_token_usage,
    )
    .expect("parse token");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_params_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--format"),
            OsString::from("json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_params_command,
        sns_params_usage,
    )
    .expect("parse params");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

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

#[test]
fn sns_neurons_rejects_invalid_clap_values() {
    assert!(matches!(
        SnsLookupOptions::parse(
            [OsString::from("not-a-principal")],
            sns_info_command,
            sns_info_usage
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsLookupOptions::parse([OsString::from("0")], sns_token_command, sns_token_usage),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsOptions::parse([
            OsString::from("1"),
            OsString::from("--limit"),
            OsString::from("0"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsOptions::parse([
            OsString::from("1"),
            OsString::from("--limit"),
            OsString::from("101"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsOptions::parse([
            OsString::from("1"),
            OsString::from("--owner"),
            OsString::from("not-a-principal"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsRefreshOptions::parse([
            OsString::from("1"),
            OsString::from("--page-size"),
            OsString::from("0"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsCacheStatusOptions::parse([OsString::from("not-a-principal")]),
        Err(SnsCommandError::Usage(_))
    ));
}

#[test]
fn sns_help_is_advertised() {
    let sns = usage();
    let list = sns_list_usage();
    let info = sns_info_usage();
    let token = sns_token_usage();
    let params = sns_params_usage();
    let neurons = sns_neurons_usage();
    let neurons_cache = sns_neurons_cache_usage();
    let neurons_cache_list = sns_neurons_cache_list_usage();
    let neurons_cache_status = sns_neurons_cache_status_usage();
    let neurons_refresh = sns_neurons_refresh_usage();

    assert!(sns.contains("list"));
    assert!(sns.contains("info"));
    assert!(sns.contains("token"));
    assert!(sns.contains("params"));
    assert!(sns.contains("neurons"));
    assert!(sns.contains("List deployed mainnet SNS instances"));
    assert!(sns.contains("Resolve a deployed SNS"));
    assert!(sns.contains("Show SNS ledger token metadata"));
    assert!(sns.contains("Show SNS governance nervous system parameters"));
    assert!(sns.contains("List and refresh SNS governance neurons"));
    assert!(list.contains("icq sns list"));
    assert!(list.contains("--format json"));
    assert!(list.contains("--source-endpoint"));
    assert!(list.contains("--sort"));
    assert!(list.contains("--verbose"));
    assert!(info.contains("icq sns info"));
    assert!(info.contains("id|root-principal"));
    assert!(token.contains("icq sns token"));
    assert!(token.contains("id|root-principal"));
    assert!(params.contains("icq sns params"));
    assert!(params.contains("id|root-principal"));
    assert!(neurons.contains("icq sns neurons"));
    assert!(neurons.contains("--limit"));
    assert!(neurons.contains("--owner"));
    assert!(neurons.contains("--verbose"));
    assert!(neurons.contains("--sort"));
    assert!(neurons.contains("refresh"));
    assert!(neurons.contains("cache"));
    assert!(neurons_cache.contains("icq sns neurons cache"));
    assert!(neurons_cache.contains("list"));
    assert!(neurons_cache.contains("status"));
    assert!(neurons_cache_list.contains("icq sns neurons cache list"));
    assert!(neurons_cache_list.contains("--format json"));
    assert!(neurons_cache_status.contains("icq sns neurons cache status"));
    assert!(neurons_cache_status.contains("id|root-principal"));
    assert!(neurons_refresh.contains("icq sns neurons refresh"));
    assert!(neurons_refresh.contains("--page-size"));
    assert!(neurons_refresh.contains("--max-pages"));
}

#[test]
fn sns_list_usage_snapshot() {
    let expected = "\
List deployed mainnet SNS instances

Usage: icq sns list [OPTIONS]

Options:
      --format <text|json>     Output format; defaults to text [default: text] [possible values: text, json]
      --source-endpoint <url>  IC API endpoint used for SNS-W and governance metadata queries [default: https://icp-api.io]
      --verbose                Show full canister IDs in text output
      --sort <id|name>         Text/JSON row order; ids follow the SNS-W response order [default: id] [possible values: id, name]

Examples:
  icq sns list
  icq sns list --sort name
  icq sns list --verbose
  icq --network ic sns list --format json
  icq sns list --source-endpoint https://icp-api.io
";

    assert_snapshot("sns list usage", &sns_list_usage(), expected);
}
