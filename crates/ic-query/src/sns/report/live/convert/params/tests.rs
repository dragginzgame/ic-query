use super::*;
use crate::sns::report::{SnsNeuronId, live::types::SnsGovernanceFollowees};

#[test]
fn complete_parameter_conversion_preserves_default_followees() {
    let parameters = SnsGovernanceParametersWire {
        default_followees: Some(SnsDefaultFolloweesWire {
            followees: vec![
                (
                    7,
                    SnsGovernanceFollowees {
                        followees: vec![SnsNeuronId { id: vec![0xab; 32] }],
                    },
                ),
                (
                    9,
                    SnsGovernanceFollowees {
                        followees: vec![
                            SnsNeuronId { id: vec![0xcd; 32] },
                            SnsNeuronId { id: vec![0xef; 32] },
                        ],
                    },
                ),
            ],
        }),
        ..Default::default()
    };

    let converted = sns_governance_parameters(parameters);
    let defaults = converted.default_followees.expect("default followees");

    assert_eq!(defaults.followees[0].function_id, 7);
    assert_eq!(defaults.followees[0].followee_neuron_ids, ["ab".repeat(32)]);
    assert_eq!(defaults.followees[1].function_id, 9);
    assert_eq!(
        defaults.followees[1].followee_neuron_ids,
        ["cd".repeat(32), "ef".repeat(32)]
    );
}
