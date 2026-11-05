use super::*;
use crate::IcqCliError;

#[test]
fn command_errors_keep_typed_failures_and_add_actionable_advice() {
    for (error, advice) in [
        (
            IcqCliError::Nns(
                NnsInventoryError::UnsupportedNetwork {
                    collection: NnsInventoryCollection::Node,
                    network: "local".into(),
                }
                .into(),
            ),
            "icq --network ic nns node list",
        ),
        (
            IcqCliError::Nns(
                NnsTopologyHostError::UnsupportedNetwork {
                    network: "local".into(),
                }
                .into(),
            ),
            "icq --network ic nns topology help",
        ),
        (
            IcqCliError::Nns(
                NnsProposalHostError::MissingProposalCache {
                    path: "missing.json".into(),
                }
                .into(),
            ),
            "icq nns proposal refresh",
        ),
        (
            IcqCliError::Sns(
                SnsHostError::MissingNeuronsCache {
                    path: "missing.json".into(),
                }
                .into(),
            ),
            "icq sns neuron refresh <governance-canister-id>",
        ),
    ] {
        assert_eq!(error.exit_code(), 1);
        assert!(error.to_string().contains(advice));
    }
}
