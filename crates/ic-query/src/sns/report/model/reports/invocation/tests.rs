use super::*;

#[test]
fn canister_call_type_labels_round_trip() {
    for (call_type, label) in [
        (SnsCanisterCallType::Query, "query"),
        (SnsCanisterCallType::CompositeQuery, "composite_query"),
        (SnsCanisterCallType::IngressUpdate, "ingress_update"),
    ] {
        assert_json_label(call_type, label);
        assert_eq!(call_type.as_str(), label);
    }
}

#[test]
fn canister_method_labels_round_trip() {
    for (method, label) in [
        (SnsCanisterMethod::ListDeployedSnses, "list_deployed_snses"),
        (SnsCanisterMethod::GetMetadata, "get_metadata"),
        (SnsCanisterMethod::ListSnsCanisters, "list_sns_canisters"),
        (
            SnsCanisterMethod::GetSnsCanistersSummary,
            "get_sns_canisters_summary",
        ),
        (SnsCanisterMethod::GetMetrics, "get_metrics"),
        (
            SnsCanisterMethod::GetNervousSystemParameters,
            "get_nervous_system_parameters",
        ),
        (SnsCanisterMethod::ListProposals, "list_proposals"),
        (SnsCanisterMethod::GetProposal, "get_proposal"),
        (SnsCanisterMethod::ListNeurons, "list_neurons"),
        (SnsCanisterMethod::GetNeuron, "get_neuron"),
        (
            SnsCanisterMethod::GetLatestRewardEvent,
            "get_latest_reward_event",
        ),
        (SnsCanisterMethod::GetLifecycle, "get_lifecycle"),
        (SnsCanisterMethod::GetSaleParameters, "get_sale_parameters"),
        (SnsCanisterMethod::GetDerivedState, "get_derived_state"),
        (
            SnsCanisterMethod::GetRunningSnsVersion,
            "get_running_sns_version",
        ),
        (SnsCanisterMethod::GetNextSnsVersion, "get_next_sns_version"),
    ] {
        assert_json_label(method, label);
        assert_eq!(method.as_str(), label);
    }
}

fn assert_json_label<T>(value: T, label: &str)
where
    T: Copy + std::fmt::Debug + Eq + Serialize + serde::de::DeserializeOwned,
{
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        format!("\"{label}\"")
    );
    assert_eq!(
        serde_json::from_str::<T>(&format!("\"{label}\"")).unwrap(),
        value
    );
}
