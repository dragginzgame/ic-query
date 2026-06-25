use super::*;

#[test]
fn sns_help_is_advertised() {
    let sns = usage();
    let list = sns_list_usage();
    let info = sns_info_usage();
    let token = sns_token_usage();
    let params = sns_params_usage();
    let proposal = sns_proposal_usage();
    let proposals = sns_proposals_usage();
    let neurons = sns_neurons_usage();
    let neurons_cache = sns_neurons_cache_usage();
    let neurons_cache_list = sns_neurons_cache_list_usage();
    let neurons_cache_status = sns_neurons_cache_status_usage();
    let neurons_refresh = sns_neurons_refresh_usage();

    assert!(sns.contains("list"));
    assert!(sns.contains("info"));
    assert!(sns.contains("token"));
    assert!(sns.contains("params"));
    assert!(sns.contains("proposal"));
    assert!(sns.contains("proposals"));
    assert!(sns.contains("neurons"));
    assert!(sns.contains("List deployed mainnet SNS instances"));
    assert!(sns.contains("Resolve a deployed SNS"));
    assert!(sns.contains("Show SNS ledger token metadata"));
    assert!(sns.contains("Show SNS governance nervous system parameters"));
    assert!(sns.contains("Show one SNS governance proposal"));
    assert!(sns.contains("List SNS governance proposals"));
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
    assert!(proposal.contains("icq sns proposal"));
    assert!(proposal.contains("id|root-principal"));
    assert!(proposal.contains("proposal-id"));
    assert!(proposal.contains("--ballots"));
    assert!(proposal.contains("--verbose"));
    assert!(proposals.contains("icq sns proposals"));
    assert!(proposals.contains("--limit"));
    assert!(proposals.contains("--before"));
    assert!(proposals.contains("--status"));
    assert!(proposals.contains("--topic"));
    assert!(proposals.contains("--verbose"));
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
