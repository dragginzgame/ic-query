//! Actionable command advice for typed library failures.

use crate::{nns::NnsCommandError, sns::SnsCommandError};
use ic_query::nns::data_center::NnsDataCenterHostError;
use ic_query::nns::neuron::NnsNeuronHostError;
use ic_query::nns::node::NnsNodeHostError;
use ic_query::nns::node_operator::NnsNodeOperatorHostError;
use ic_query::nns::node_provider::NnsNodeProviderHostError;
use ic_query::nns::proposals::NnsProposalHostError;
use ic_query::nns::registry::NnsRegistryHostError;
use ic_query::nns::topology::NnsTopologyHostError;
use ic_query::sns::SnsHostError;

pub const fn nns_advice(error: &NnsCommandError) -> &'static str {
    match error {
        NnsCommandError::DataCenterHost(NnsDataCenterHostError::UnsupportedNetwork { .. }) => {
            "\n\nTry:\n  icq --network ic nns data-center list"
        }
        NnsCommandError::NodeHost(NnsNodeHostError::UnsupportedNetwork { .. }) => {
            "\n\nTry:\n  icq --network ic nns node list"
        }
        NnsCommandError::NodeOperatorHost(NnsNodeOperatorHostError::UnsupportedNetwork {
            ..
        }) => "\n\nTry:\n  icq --network ic nns node-operator list",
        NnsCommandError::NodeProviderHost(NnsNodeProviderHostError::UnsupportedNetwork {
            ..
        }) => "\n\nTry:\n  icq --network ic nns node-provider list",
        NnsCommandError::RegistryHost(NnsRegistryHostError::UnsupportedNetwork { .. }) => {
            "\n\nTry:\n  icq --network ic nns registry version"
        }
        NnsCommandError::TopologyHost(NnsTopologyHostError::UnsupportedNetwork { .. }) => {
            "\n\nTry:\n  icq --network ic nns topology help"
        }
        NnsCommandError::ProposalHost(NnsProposalHostError::MissingProposalCache { .. }) => {
            "\n\nRun `icq nns proposal refresh` to fetch a complete snapshot."
        }
        NnsCommandError::NeuronHost(NnsNeuronHostError::MissingNeuronCache { .. }) => {
            "\n\nRun `icq nns neuron refresh` to fetch a complete snapshot."
        }
        _ => "",
    }
}

pub const fn sns_advice(error: &SnsCommandError) -> &'static str {
    match error {
        SnsCommandError::Host(SnsHostError::UnsupportedNetwork { .. }) => {
            "\n\nTry:\n  icq --network ic sns list"
        }
        SnsCommandError::Host(SnsHostError::MissingNeuronsCache { .. }) => {
            "\n\nRun `icq sns neuron refresh <id|root-principal>` to fetch a complete snapshot before using cache-backed sorting."
        }
        SnsCommandError::Host(SnsHostError::MissingProposalsCache { .. }) => {
            "\n\nRun `icq sns proposal refresh <id|root-principal>` to fetch a complete snapshot."
        }
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IcqCliError;

    #[test]
    fn command_errors_keep_typed_failures_and_add_actionable_advice() {
        for (error, advice) in [
            (
                IcqCliError::Nns(
                    NnsNodeHostError::UnsupportedNetwork {
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
                "icq sns neuron refresh <id|root-principal>",
            ),
        ] {
            assert_eq!(error.exit_code(), 1);
            assert!(error.to_string().contains(advice));
        }
    }
}
