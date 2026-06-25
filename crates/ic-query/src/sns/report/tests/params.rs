use super::{fixtures::*, *};

#[test]
fn sns_params_resolves_list_id_and_renders_governance_parameters() {
    let request = params_request("1");

    let report = build_sns_params_report_with_source(&request, &FixtureSnsParamsSource)
        .expect("sns params report");
    let text = sns_params_report_text(&report);

    assert_eq!(report.schema_version, SNS_PARAMS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(
        report.parameters.neuron_minimum_stake_e8s,
        Some(100_000_000)
    );
    assert_eq!(report.parameters.transaction_fee_e8s, Some(10_000));
    assert_eq!(
        report
            .parameters
            .voting_rewards_parameters
            .as_ref()
            .and_then(|rewards| rewards.initial_reward_rate_basis_points),
        Some(1000)
    );
    assert!(text.contains("governance_canister_id: bkyz2-fmaaa-aaaaa-qaaaq-cai"));
    assert!(text.contains("neuron_minimum_stake"));
    assert!(text.contains("transaction_fee"));
    assert!(text.contains("max_dissolve_delay"));
    assert!(text.contains("voting_reward_initial_rate"));
    assert!(text.contains("automatically_advance_target_version"));
    assert!(text.contains("1.00"));
    assert!(text.contains("0.00"));
    assert!(text.contains("2922d"));
    assert!(text.contains("10.00%"));
    assert!(text.contains("yes"));
    assert!(text.contains("1,2,3"));
}
