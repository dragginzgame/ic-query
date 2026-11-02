use super::fixtures::*;
use super::*;

#[tokio::test(flavor = "current_thread")]
async fn sns_params_resolves_list_id_and_renders_governance_parameters() {
    let request = params_request("1");

    let report = build_sns_params_report_with_source(&request, &FixtureSnsParamsSource)
        .await
        .expect("sns params report");

    assert_eq!(report.schema_version, SNS_PARAMS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.context.instance.id, 1);
    assert_eq!(report.context.instance.name, "Fixture SNS");
    assert_eq!(report.context.instance.root_canister_id, ROOT_A);
    assert_eq!(report.context.governance_canister_id, GOVERNANCE_A);
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
}
