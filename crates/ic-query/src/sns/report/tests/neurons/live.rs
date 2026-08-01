use crate::sns::report::tests::{fixtures::*, *};

struct InvalidBoundedSnsNeuronsSource;

delegate_sns_discovery!(InvalidBoundedSnsNeuronsSource);

impl SnsNeuronsSource for InvalidBoundedSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        let mut neurons =
            FixtureSnsNeuronsSource.fetch_sns_neurons(request, sns, limit, owner_principal_id)?;
        neurons.neurons[0].neuron_id = "0A".to_string();
        Ok(neurons)
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _start_page_at: Option<&SnsNeuronId>,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        unreachable!("bounded validation fixture does not refresh")
    }
}

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
    assert_eq!(report.neurons[0].source_nns_neuron_id, Some(42));
    assert_eq!(report.neurons[0].auto_stake_maturity, Some(true));
    assert_eq!(
        report.neurons[0].aging_since_timestamp_seconds,
        1_780_272_100
    );
    assert_eq!(
        report.neurons[0].dissolve_state,
        Some(SnsNeuronDissolveState::DissolveDelaySeconds(31_536_000))
    );
    assert_eq!(report.neurons[0].voting_power_percentage_multiplier, 100);
    assert_eq!(report.neurons[0].vesting_period_seconds, Some(63_072_000));
    assert_eq!(report.neurons[0].neuron_fees_e8s, 10);
    assert!(text.contains("governance_canister_id: bkyz2-fmaaa-aaaaa-qaaaq-cai"));
    assert!(text.contains("requested_limit: 10"));
    assert!(text.contains("owner_principal_id: bkyz2-fmaaa-aaaaa-qaaaq-cai"));
    assert!(text.contains("00010203"));
    assert!(!text.contains("0001020304"));
    assert!(text.contains("STAKE"));
    assert!(text.contains("MATURITY"));
    assert!(text.contains("STAKED_MATURITY"));
    assert!(text.contains("FEES"));
    assert!(text.contains("DISSOLVE"));
    assert!(text.contains("delay:31536000"));
    assert!(text.contains("AUTO_STAKE"));
    assert!(text.contains("VOTING_%"));
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
fn sns_neurons_builder_rejects_noncanonical_custom_source_rows() {
    let mut request = neurons_request("1");
    request.owner_principal_id = Some(GOVERNANCE_A.to_string());

    let error = build_sns_neurons_report_with_source(&request, &InvalidBoundedSnsNeuronsSource)
        .expect_err("invalid custom-source row rejected");

    assert!(matches!(
        error,
        SnsHostError::InvalidSourceData {
            capability: "SNS neurons",
            reason,
        } if reason.contains("canonical lowercase hexadecimal")
    ));
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
