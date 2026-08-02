use super::*;

#[test]
fn sns_neurons_rejects_invalid_clap_values() {
    assert!(matches!(
        parse_test_options(
            sns_info_command(),
            &["not-a-principal"],
            SnsLookupOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_test_options(sns_token_command(), &["0"], SnsLookupOptions::from_matches,),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_neuron_list_command(),
            &["1", "--limit", "0"],
            SnsNeuronsOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_neuron_list_command(),
            &["1", "--limit", "101"],
            SnsNeuronsOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_neuron_list_command(),
            &["1", "--owner", "not-a-principal"],
            SnsNeuronsOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_neuron_list_command(),
            &[
                "1",
                "--owner",
                "zqfso-syaaa-aaaaq-aaafq-cai",
                "--sort",
                "stake",
            ],
            SnsNeuronsOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_test_options(
            sns_neuron_refresh_command(),
            &["1", "--page-size", "0"],
            SnsNeuronsRefreshOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_test_options(
            sns_neuron_cache_status_command(),
            &["not-a-principal"],
            SnsNeuronsCacheStatusOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
}

#[test]
fn sns_proposals_reject_invalid_clap_values() {
    assert!(matches!(
        parse_fallible_test_options(
            sns_proposal_list_command(),
            &["1", "--limit", "101"],
            SnsProposalsOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_proposal_list_command(),
            &["1", "--status", "not-a-status"],
            SnsProposalsOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_proposal_list_command(),
            &["1", "--topic", "not-a-topic"],
            SnsProposalsOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_proposal_list_command(),
            &["1", "--asc"],
            SnsProposalsOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_test_options(
            sns_proposal_info_command(),
            &["1", "0"],
            SnsProposalOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
}

#[test]
fn sns_metrics_rejects_invalid_or_excessive_windows() {
    for value in ["0", "1.5h", "366d"] {
        assert!(matches!(
            parse_test_options(
                sns_metrics_command(),
                &["1", "--window", value],
                SnsMetricsOptions::from_matches,
            ),
            Err(SnsCommandError::Usage(_))
        ));
    }
}
