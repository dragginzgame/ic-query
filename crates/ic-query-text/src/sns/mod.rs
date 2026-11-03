//! Module: sns
//!
//! Responsibility: group SNS text report renderers.
//! Does not own: report construction, source/cache reads, or JSON output.
//! Boundary: converts already-built report DTOs into human-readable text.

#[cfg(feature = "sns-host")]
mod cache;
mod canisters;
mod common;
mod info;
mod list;
mod metrics;
mod neuron;
#[cfg(feature = "sns-host")]
mod neurons;
mod params;
mod proposals;
#[cfg(feature = "sns-host")]
mod refresh;
mod swap;
mod token;
mod upgrade;

#[cfg(feature = "sns-host")]
pub use crate::sns::cache::sns_neurons_cache_list_report_text;
#[cfg(feature = "sns-host")]
pub use crate::sns::cache::sns_neurons_cache_status_report_text;
#[cfg(feature = "sns-host")]
pub use crate::sns::cache::sns_proposals_cache_list_report_text;
#[cfg(feature = "sns-host")]
pub use crate::sns::cache::sns_proposals_cache_status_report_text;
pub use crate::sns::canisters::sns_canister_report_text;
pub use crate::sns::info::sns_info_report_text;
pub use crate::sns::list::sns_list_report_text;
pub use crate::sns::list::sns_list_report_text_with_metadata_style;
pub use crate::sns::metrics::sns_metrics_report_text;
pub use crate::sns::neuron::sns_neuron_detail_report_text;
#[cfg(feature = "sns-host")]
pub use crate::sns::neurons::sns_neurons_report_text;
pub use crate::sns::params::sns_params_report_text;
pub use crate::sns::proposals::sns_proposal_report_text;
pub use crate::sns::proposals::sns_proposals_report_text;
#[cfg(feature = "sns-host")]
pub use crate::sns::refresh::sns_snapshot_refresh_report_text;

pub use crate::sns::swap::sns_swap_report_text;
pub use crate::sns::token::sns_token_report_text;
pub use crate::sns::upgrade::sns_upgrade_report_text;

pub(crate) fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}

#[cfg(feature = "sns-host")]
mod catalog;
#[cfg(feature = "sns-host")]
pub use catalog::sns_catalog_refresh_report_text;

mod labels;
const COMPACT_PRINCIPAL_CHARS: usize = 5;
