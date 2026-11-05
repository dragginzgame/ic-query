//! Native neuron source contracts and admission.
pub(in crate::sns::report) mod model;
mod traits;
pub use model::{MainnetSnsNeuronPage, SnsNeuronId};
pub(in crate::sns::report) use model::{
    sns_neuron_id_from_text, validate_mainnet_sns_neuron, validate_mainnet_sns_neuron_page,
    validate_mainnet_sns_neurons, validate_sns_neuron_rows,
};
pub use traits::{SnsNeuronSource, SnsNeuronsSource};
