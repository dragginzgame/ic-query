//! Complete-neuron view ordering.
#[cfg(feature = "sns-host")]
mod neurons;
#[cfg(feature = "sns-host")]
pub use neurons::SnsNeuronsCacheSort;
