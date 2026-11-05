//! Neuron row, permission, identity and pagination admission.
mod neurons;
mod validation;
pub use neurons::{MainnetSnsNeuronPage, SnsNeuronId};
pub(in crate::sns::report) use neurons::{
    sns_neuron_id_from_text, validate_mainnet_sns_neuron, validate_mainnet_sns_neuron_page,
    validate_mainnet_sns_neurons, validate_sns_neuron_rows,
};
