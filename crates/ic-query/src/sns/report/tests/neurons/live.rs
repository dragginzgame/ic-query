use crate::sns::report::tests::{fixtures::*, *};

#[test]
fn sns_neurons_resolves_list_id_and_renders_governance_neurons() {
    let mut request = neurons_request("1");
    request.owner_principal_id = Some(GOVERNANCE_A.to_string());

    let report = build_sns_neurons_report_with_source(&request, &FixtureSnsNeuronsSource)
        .expect("sns neurons report");
    let text = sns_neurons_report_text(&report);

    assert_eq!(report.schema_version, SNS_NEURONS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.requested_limit, 10);
    assert_eq!(report.owner_principal_id.as_deref(), Some(GOVERNANCE_A));
    assert_eq!(report.neuron_count, 1);
    assert_eq!(report.neurons[0].neuron_id, "0001020304");
    assert_eq!(report.neurons[0].cached_neuron_stake_e8s, 123);
    assert_eq!(report.neurons[0].maturity_e8s_equivalent, 456);
    assert_eq!(report.neurons[0].staked_maturity_e8s_equivalent, Some(789));
    assert_eq!(report.neurons[0].created_at, "2026-06-01T00:00:00Z");
    assert!(text.contains("governance_canister_id: bkyz2-fmaaa-aaaaa-qaaaq-cai"));
    assert!(text.contains("requested_limit: 10"));
    assert!(text.contains("owner_principal_id: bkyz2-fmaaa-aaaaa-qaaaq-cai"));
    assert!(text.contains("00010203"));
    assert!(!text.contains("0001020304"));
    assert!(text.contains("STAKE"));
    assert!(text.contains("MATURITY"));
    assert!(text.contains("STAKED_MATURITY"));
    assert!(!text.contains("STAKE_E8S"));
    assert!(!text.contains("MATURITY_E8S"));
    assert!(text.contains("0.00"));
    assert!(text.contains("2026-06-01T00:00:00Z"));
}

#[test]
fn sns_neurons_text_formats_optional_e8s_as_token_decimals() {
    assert_eq!(text::optional_e8s_decimal_text(None), "-");
    assert_eq!(text::optional_e8s_decimal_text(Some(50_000_000)), "0.50");
}

#[test]
fn sns_neurons_verbose_text_keeps_full_neuron_ids() {
    let mut request = neurons_request("1");
    request.owner_principal_id = Some(GOVERNANCE_A.to_string());
    request.verbose = true;

    let report = build_sns_neurons_report_with_source(&request, &FixtureSnsNeuronsSource)
        .expect("sns neurons report");
    let text = sns_neurons_report_text(&report);

    assert!(report.verbose);
    assert!(text.contains("verbose: yes"));
    assert!(text.contains("0001020304"));
}
