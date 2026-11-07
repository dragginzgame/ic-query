//! Retained neuron access for token migration and stake evidence.
mod model;
pub use model::*;
#[cfg(feature = "sns-host")]
mod assemble;
#[cfg(feature = "sns-host")]
mod build;
#[cfg(feature = "sns-host")]
mod cache_attempt;
#[cfg(feature = "sns-host")]
mod cache_paths;
#[cfg(feature = "sns-host")]
mod cache_status;
#[cfg(feature = "sns-host")]
mod cache_storage;
#[cfg(feature = "sns-host")]
mod cache_summary;
#[cfg(feature = "sns-host")]
mod live;
#[cfg(feature = "sns-host")]
mod neurons_cache;
#[cfg(feature = "sns-host")]
mod source;
#[cfg(feature = "sns-host")]
mod target;
#[cfg(feature = "sns-host")]
mod view;
#[cfg(feature = "sns-host")]
use crate::hex::hex_bytes;
#[cfg(feature = "sns-host")]
pub use build::*;
#[cfg(feature = "sns-host")]
pub use cache_status::build_sns_neurons_cache_status_report;
#[cfg(feature = "sns-host")]
pub use cache_summary::build_sns_neurons_cache_list_report;
#[cfg(feature = "sns-host")]
pub use live::LiveSnsSource;
#[cfg(feature = "sns-host")]
pub use neurons_cache::*;
#[cfg(feature = "sns-host")]
pub use source::{MainnetSnsNeuronPage, SnsNeuronId, SnsNeuronSource, SnsNeuronsSource};
/// Default replica endpoint for direct Governance neuron queries.
pub const DEFAULT_SNS_SOURCE_ENDPOINT: &str = "https://icp-api.io";
/// Maximum rows accepted in one Governance neuron page.
pub const SNS_REFRESH_MAX_PAGE_SIZE: u32 = 100;
#[cfg(feature = "sns-host")]
pub(super) const SNS_CACHE_COMPONENT: &str = "sns-neuron";
#[cfg(feature = "sns-host")]
pub(super) const SNS_NEURONS_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "sns-host")]
pub(super) const SNS_NEURON_DETAIL_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "sns-host")]
fn enforce_mainnet_network(network: &str) -> Result<(), SnsHostError> {
    crate::network::enforce_mainnet_network_with(network, |network| {
        SnsHostError::UnsupportedNetwork { network }
    })
}

#[cfg(all(test, feature = "sns-host"))]
mod tests;
