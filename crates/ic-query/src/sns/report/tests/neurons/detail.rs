use crate::sns::report::tests::{fixtures::*, *};

struct NoCallSnsNeuronSource;

impl SnsDiscoverySource for NoCallSnsNeuronSource {
    fn fetch_sns_inventory(
        &self,
        _request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        unreachable!("invalid neuron id must fail before discovery")
    }

    fn fetch_sns_metadata(
        &self,
        _request: &SnsSourceRequest,
        _targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        unreachable!("invalid neuron id must fail before metadata")
    }
}

impl SnsNeuronSource for NoCallSnsNeuronSource {
    fn fetch_sns_neuron(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        _neuron_id: &str,
    ) -> Result<MainnetSnsNeuron, SnsHostError> {
        unreachable!("invalid neuron id must fail before exact lookup")
    }
}

struct InvalidPolicySnsNeuronSource;

delegate_sns_discovery!(InvalidPolicySnsNeuronSource);

impl SnsNeuronSource for InvalidPolicySnsNeuronSource {
    fn fetch_sns_neuron(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        _neuron_id: &str,
    ) -> Result<MainnetSnsNeuron, SnsHostError> {
        let mut neuron = fixture_sns_neuron();
        neuron.detail.maturity_mint_conversion_observed_disabled =
            SnsPolicyObservationStatus::ObservedSatisfied;
        Ok(neuron)
    }
}

#[test]
fn exact_neuron_detail_preserves_native_permission_and_disbursement_evidence() {
    let report =
        build_sns_neuron_detail_report_with_source(&neuron_request("1"), &FixtureSnsNeuronSource)
            .expect("exact neuron detail");
    let text = sns_neuron_detail_report_text(&report);

    assert_eq!(
        report.schema_version,
        SNS_NEURON_DETAIL_REPORT_SCHEMA_VERSION
    );
    assert_eq!(report.neuron_id, NEURON_A);
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.data_source, "live");
    assert_eq!(report.detail.neuron.neuron_id, NEURON_A);
    assert_eq!(report.detail.permissions[0].permission_types[4].code, 9);
    assert_eq!(
        report.detail.permissions[0].permission_types[4].name,
        "stake_maturity"
    );
    assert_eq!(
        report.detail.maturity_mint_conversion_observed_disabled,
        SnsPolicyObservationStatus::Violated
    );
    assert_eq!(
        report.detail.manual_maturity_staking_observed_disabled,
        SnsPolicyObservationStatus::Violated
    );
    let destination = report.detail.disburse_maturity_in_progress[0]
        .account_to_disburse_to
        .as_ref()
        .expect("destination account");
    assert_eq!(destination.owner.as_deref(), Some(ROOT_A));
    assert_eq!(
        destination.subaccount_hex.as_deref(),
        Some(&*"ab".repeat(32))
    );
    assert_eq!(
        report.detail.followees[0].followee_neuron_ids[0],
        "11".repeat(32)
    );
    assert_eq!(
        report.detail.topic_followees.as_ref().expect("topic rows")[0].followees[0]
            .alias
            .as_deref(),
        Some("governance lead")
    );
    assert!(text.contains("maturity_mint_conversion_observed_disabled: violated"));
    assert!(text.contains("9:stake_maturity"));
    assert!(text.contains(ROOT_A));
}

#[test]
fn exact_neuron_detail_rejects_invalid_input_before_source_calls() {
    let mut request = neuron_request("1");
    request.neuron_id = "AB".repeat(32);

    assert!(matches!(
        build_sns_neuron_detail_report_with_source(&request, &NoCallSnsNeuronSource),
        Err(SnsHostError::InvalidNeuronIdText { neuron_id }) if neuron_id == "AB".repeat(32)
    ));
}

#[test]
fn exact_neuron_detail_recomputes_custom_source_policy_observations() {
    let error = build_sns_neuron_detail_report_with_source(
        &neuron_request("1"),
        &InvalidPolicySnsNeuronSource,
    )
    .expect_err("tampered derived status rejected");

    assert!(matches!(
        error,
        SnsHostError::InvalidSourceData {
            capability: "SNS neuron detail",
            reason,
        } if reason.contains("does not match raw evidence")
    ));
}
