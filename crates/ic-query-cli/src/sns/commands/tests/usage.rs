use super::*;

#[test]
fn sns_help_is_advertised() {
    let sns = usage();
    let list = sns_list_usage();
    let info = sns_info_usage();
    let token = sns_token_usage();
    let params = sns_params_usage();
    let canister = sns_canister_usage();
    let canister_list = sns_canister_list_usage();
    let proposal = sns_proposal_usage();
    let proposal_info = sns_proposal_info_usage();
    let proposal_list = sns_proposal_list_usage();
    let proposal_cache_list = sns_proposal_cache_list_usage();
    let proposal_cache_status = sns_proposal_cache_status_usage();
    let proposal_refresh = sns_proposal_refresh_usage();
    let neuron = sns_neuron_usage();
    let neuron_list = sns_neuron_list_usage();
    let neuron_cache = sns_neuron_cache_usage();
    let neuron_cache_list = sns_neuron_cache_list_usage();
    let neuron_cache_status = sns_neuron_cache_status_usage();
    let neuron_refresh = sns_neuron_refresh_usage();

    assert!(sns.contains("list"));
    assert!(sns.contains("info"));
    assert!(sns.contains("token"));
    assert!(sns.contains("params"));
    assert!(sns.contains("canister"));
    assert!(sns.contains("proposal"));
    assert!(sns.contains("neuron"));
    assert!(sns.contains("List deployed mainnet SNS instances"));
    assert!(sns.contains("Resolve a deployed SNS"));
    assert!(sns.contains("Show SNS ledger token metadata"));
    assert!(sns.contains("Show SNS governance nervous system parameters"));
    assert!(sns.contains("Inspect SNS Root canister inventory and operational health"));
    assert!(sns.contains("List, inspect, and refresh SNS governance proposals"));
    assert!(sns.contains("List and refresh SNS governance neurons"));
    assert!(list.contains("icq sns list"));
    assert!(list.contains("Collection mode: Live query"));
    assert!(list.contains("--json"));
    assert!(list.contains("--source-endpoint"));
    assert!(list.contains("--sort"));
    assert!(list.contains("--verbose"));
    assert!(info.contains("icq sns info"));
    assert!(info.contains("id|root-principal"));
    assert!(token.contains("icq sns token"));
    assert!(token.contains("id|root-principal"));
    assert!(params.contains("icq sns params"));
    assert!(params.contains("id|root-principal"));
    assert!(canister.contains("icq sns canister"));
    assert!(canister.contains("list"));
    assert!(canister_list.contains("icq sns canister list"));
    assert!(canister_list.contains("Collection mode: Live query"));
    assert!(canister_list.contains("id|root-principal"));
    assert!(canister_list.contains("update_canister_list=false"));
    assert!(canister_list.contains("--source-endpoint"));
    assert!(proposal.contains("icq sns proposal"));
    assert!(proposal.contains("list"));
    assert!(proposal.contains("info"));
    assert!(proposal.contains("refresh"));
    assert!(proposal.contains("cache"));
    assert!(proposal_info.contains("icq sns proposal info"));
    assert!(proposal_info.contains("Collection mode: Cache-preferred read"));
    assert!(proposal_info.contains("id|root-principal"));
    assert!(proposal_info.contains("proposal-id"));
    assert!(proposal_info.contains("--ballots"));
    assert!(proposal_info.contains("--verbose"));
    assert!(proposal_list.contains("icq sns proposal list"));
    assert!(proposal_list.contains("Collection mode: Cache-backed read"));
    assert!(proposal_list.contains("--limit"));
    assert!(proposal_list.contains("--before"));
    assert!(proposal_list.contains("--status"));
    assert!(proposal_list.contains("--topic"));
    assert!(proposal_list.contains("--verbose"));
    assert!(proposal_cache_list.contains("Collection mode: Local cache inspection"));
    assert!(proposal_cache_status.contains("Collection mode: Local cache inspection"));
    assert!(!proposal_cache_list.contains("--source-endpoint"));
    assert!(!proposal_cache_status.contains("--source-endpoint"));
    assert!(proposal_refresh.contains("Collection mode: Forced live refresh"));
    assert!(neuron.contains("icq sns neuron"));
    assert!(neuron.contains("list"));
    assert!(neuron.contains("refresh"));
    assert!(neuron.contains("cache"));
    assert!(neuron_list.contains("icq sns neuron list"));
    assert!(neuron_list.contains("Collection mode: View-dependent read"));
    assert!(neuron_list.contains("--limit"));
    assert!(neuron_list.contains("--owner"));
    assert!(neuron_list.contains("--verbose"));
    assert!(neuron_list.contains("--sort"));
    assert!(neuron_cache.contains("icq sns neuron cache"));
    assert!(neuron_cache.contains("list"));
    assert!(neuron_cache.contains("status"));
    assert!(neuron_cache_list.contains("icq sns neuron cache list"));
    assert!(neuron_cache_list.contains("Collection mode: Local cache inspection"));
    assert!(!neuron_cache_list.contains("--source-endpoint"));
    assert!(neuron_cache_list.contains("--json"));
    assert!(neuron_cache_status.contains("icq sns neuron cache status"));
    assert!(neuron_cache_status.contains("id|root-principal"));
    assert!(neuron_cache_status.contains("Collection mode: Local cache inspection"));
    assert!(!neuron_cache_status.contains("--source-endpoint"));
    assert!(neuron_refresh.contains("icq sns neuron refresh"));
    assert!(neuron_refresh.contains("Collection mode: Forced live refresh"));
    assert!(neuron_refresh.contains("--page-size"));
    assert!(neuron_refresh.contains("--max-pages"));
}

#[test]
fn sns_upgrade_help_is_advertised() {
    let sns = usage();
    let upgrade = sns_upgrade_usage();

    assert!(sns.contains("upgrade"));
    assert!(sns.contains("Show the running SNS version"));
    assert!(upgrade.contains("icq sns upgrade"));
    assert!(upgrade.contains("id|root-principal"));
    assert!(upgrade.contains("at most four live calls"));
    assert!(upgrade.contains("get_running_sns_version"));
    assert!(upgrade.contains("get_next_sns_version"));
    assert!(upgrade.contains("--json"));
}

#[test]
fn sns_metrics_help_is_bounded_and_explicit() {
    let sns = usage();
    let metrics = sns_metrics_usage();

    assert!(sns.contains("metrics"));
    assert!(sns.contains("Show bounded native Governance metrics"));
    assert!(metrics.contains("icq sns metrics"));
    assert!(metrics.contains("id|root-principal"));
    assert!(metrics.contains("--window <duration>"));
    assert!(metrics.contains("[default: 30d]"));
    assert!(metrics.contains("get_metrics composite query"));
    assert!(metrics.contains("three live calls"));
    assert!(metrics.contains("one internal"));
    assert!(metrics.contains("Does not scan transactions"));
    assert!(metrics.contains("--json"));
}

#[test]
fn sns_swap_help_is_advertised() {
    let sns = usage();
    let swap = sns_swap_usage();

    assert!(sns.contains("swap"));
    assert!(sns.contains("Show bounded SNS swap lifecycle"));
    assert!(swap.contains("icq sns swap"));
    assert!(swap.contains("id|root-principal"));
    assert!(swap.contains("Collection mode: Live query"));
    assert!(swap.contains("--json"));
}

#[test]
fn sns_list_usage_snapshot() {
    let expected = "\
List deployed mainnet SNS instances

Usage: icq sns list [OPTIONS]

Options:
      --json                   Print raw JSON instead of human-readable text
      --source-endpoint <url>  IC API endpoint used for SNS-W and governance metadata queries [default: https://icp-api.io]
      --verbose                Show full canister IDs in text output
      --sort <id|name>         Text/JSON row order; ids follow the SNS-W response order [default: id] [possible values: id, name]
  -h, --help                   Print help

Collection mode: Live query; does not read or write a report cache.

Examples:
  icq sns list
  icq sns list --sort name
  icq sns list --verbose
  icq --network ic sns list --json
  icq sns list --source-endpoint https://icp-api.io
";

    assert_snapshot("sns list usage", &sns_list_usage(), expected);
}
