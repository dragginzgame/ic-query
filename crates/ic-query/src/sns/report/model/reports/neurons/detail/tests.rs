use super::*;

#[test]
fn permission_names_cover_native_and_unknown_codes() {
    for (code, expected) in [
        (0, "unspecified"),
        (1, "configure_dissolve_state"),
        (2, "manage_principals"),
        (3, "submit_proposal"),
        (4, "vote"),
        (5, "disburse"),
        (6, "split"),
        (7, "merge_maturity"),
        (8, "disburse_maturity"),
        (9, "stake_maturity"),
        (10, "manage_voting_permission"),
        (11, "unknown"),
        (-1, "unknown"),
    ] {
        assert_eq!(sns_neuron_permission_name(code), expected, "code {code}");
    }
}

#[test]
fn policy_observations_fail_closed_and_prioritize_known_violations() {
    let mut detail = SnsNeuronDetail {
        neuron: SnsNeuronRow {
            neuron_id: "00".repeat(32),
            cached_neuron_stake_e8s: 0,
            maturity_e8s_equivalent: 0,
            staked_maturity_e8s_equivalent: None,
            created_timestamp_seconds: 0,
            source_nns_neuron_id: None,
            auto_stake_maturity: None,
            aging_since_timestamp_seconds: 0,
            dissolve_state: None,
            voting_power_percentage_multiplier: 100,
            vesting_period_seconds: None,
            neuron_fees_e8s: 0,
        },
        permissions: vec![SnsNeuronPermissionRow {
            principal: Some("aaaaa-aa".to_string()),
            permission_types: vec![11],
        }],
        disburse_maturity_in_progress: Vec::new(),
        followees: Vec::new(),
        topic_followees: None,
    };

    assert_eq!(
        detail.derived_policy_observations(),
        (
            SnsPolicyObservationStatus::Unassessable,
            SnsPolicyObservationStatus::Unassessable,
        )
    );

    detail.permissions[0].permission_types.push(7);
    assert_eq!(
        detail.derived_policy_observations(),
        (
            SnsPolicyObservationStatus::Violated,
            SnsPolicyObservationStatus::Unassessable,
        )
    );
}
