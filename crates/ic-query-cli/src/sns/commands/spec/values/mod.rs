//! CLI selector for live API ordering or locally ordered SNS neuron caches.

///
/// SnsNeuronsSortArg
///
/// CLI selector for live API ordering or a locally ordered complete cache.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, clap::ValueEnum)]
pub(in crate::sns::commands) enum SnsNeuronsSortArg {
    #[default]
    Api,
    Id,
    Stake,
    Maturity,
    Created,
}
