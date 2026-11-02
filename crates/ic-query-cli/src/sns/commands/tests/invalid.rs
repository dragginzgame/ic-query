use super::*;

#[test]
fn sns_neurons_rejects_invalid_clap_values() {
    for neuron_id in ["01", &"AB".repeat(32), &"gg".repeat(32)] {
        assert!(matches!(
            parse_fallible_test_options(sns_neuron_info_command(), &["1", neuron_id], |m, n| {
                neuron_request(m, n, 1_780_531_200)
            },),
            Err(SnsCommandError::Usage(_))
        ));
    }
    assert!(matches!(
        parse_fallible_test_options(sns_info_command(), &["not-a-principal"], |m, n| {
            lookup_request(m, n, 1_780_531_200)
        },),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(sns_token_command(), &["0"], |m, n| lookup_request(
            m,
            n,
            1_780_531_200
        ),),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(sns_neuron_list_command(), &["1", "--limit", "0"], |m, n| {
            neurons_request(m, n, 1_780_531_200, || {
                Ok(std::path::PathBuf::from("fixture-cache"))
            })
        },),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_neuron_list_command(),
            &["1", "--limit", "101"],
            |m, n| neurons_request(m, n, 1_780_531_200, || Ok(std::path::PathBuf::from(
                "fixture-cache"
            ))),
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_neuron_list_command(),
            &["1", "--owner", "not-a-principal"],
            |m, n| neurons_request(m, n, 1_780_531_200, || Ok(std::path::PathBuf::from(
                "fixture-cache"
            ))),
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
            |m, n| neurons_request(m, n, 1_780_531_200, || Ok(std::path::PathBuf::from(
                "fixture-cache"
            ))),
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_neuron_refresh_command(),
            &["1", "--page-size", "0"],
            |m, n| snapshot_refresh_request(
                m,
                n,
                std::path::PathBuf::from("fixture-cache"),
                1_780_531_200
            ),
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_test_options(
            sns_neuron_cache_status_command(),
            &["not-a-principal"],
            |m, n| cache_status_request(m, n, std::path::PathBuf::from("fixture-cache")),
        ),
        Err(SnsCommandError::Usage(_))
    ));
}

#[test]
fn sns_proposals_reject_invalid_clap_values() {
    assert!(matches!(
        parse_fallible_test_options(
            sns_proposal_list_command(),
            &["be2us-64aaa-aaaaa-qaabq-cai", "--limit", "101"],
            |m, n| proposals_request(m, n, 1_780_531_200, || Ok(std::path::PathBuf::from(
                "fixture-cache"
            ))),
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_proposal_list_command(),
            &["be2us-64aaa-aaaaa-qaabq-cai", "--status", "not-a-status"],
            |m, n| proposals_request(m, n, 1_780_531_200, || Ok(std::path::PathBuf::from(
                "fixture-cache"
            ))),
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_proposal_list_command(),
            &["be2us-64aaa-aaaaa-qaabq-cai", "--topic", "not-a-topic"],
            |m, n| proposals_request(m, n, 1_780_531_200, || Ok(std::path::PathBuf::from(
                "fixture-cache"
            ))),
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(
            sns_proposal_list_command(),
            &["be2us-64aaa-aaaaa-qaabq-cai", "--asc"],
            |m, n| proposals_request(m, n, 1_780_531_200, || Ok(std::path::PathBuf::from(
                "fixture-cache"
            ))),
        ),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        parse_fallible_test_options(sns_proposal_info_command(), &["1", "0"], |m, n| {
            proposal_request(m, n, 1_780_531_200, || {
                Ok(std::path::PathBuf::from("fixture-cache"))
            })
        },),
        Err(SnsCommandError::Usage(_))
    ));
}

#[test]
fn sns_metrics_rejects_invalid_or_excessive_windows() {
    for value in ["0", "1.5h", "366d"] {
        assert!(matches!(
            parse_fallible_test_options(
                sns_metrics_command(),
                &["1", "--window", value],
                |m, n| { metrics_request(m, n, 1_780_531_200) },
            ),
            Err(SnsCommandError::Usage(_))
        ));
    }
}
