use super::{NnsNeuronError, map_neuron_info_error};

#[test]
fn neuron_info_maps_only_the_native_not_found_error() {
    let not_found = map_neuron_info_error(
        NnsNeuronError::GovernanceResponse {
            error_type: 4,
            message: "wording is not part of the contract".to_string(),
        },
        42,
    );
    assert!(matches!(
        not_found,
        NnsNeuronError::NeuronNotFound { neuron_id: 42 }
    ));

    let unrelated = map_neuron_info_error(
        NnsNeuronError::GovernanceResponse {
            error_type: 12,
            message: "not found text must not override the code".to_string(),
        },
        42,
    );
    assert!(matches!(
        unrelated,
        NnsNeuronError::GovernanceResponse { error_type: 12, .. }
    ));
}
