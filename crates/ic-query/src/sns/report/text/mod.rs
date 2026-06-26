//! Module: sns::report::text
//!
//! Responsibility: group SNS text report renderers.
//! Does not own: report construction, source/cache reads, or JSON output.
//! Boundary: converts already-built report DTOs into human-readable text.

#[cfg(feature = "host")]
mod common;
#[cfg(feature = "host")]
mod info;
mod list;
#[cfg(feature = "host")]
mod neurons;
#[cfg(feature = "host")]
mod params;
#[cfg(feature = "host")]
mod proposals;
#[cfg(feature = "host")]
mod token;

#[cfg(all(test, feature = "host"))]
pub(super) use common::optional_e8s_decimal_text;
#[cfg(feature = "host")]
pub use info::sns_info_report_text;
pub use list::sns_list_report_text;
#[cfg(feature = "host")]
pub use neurons::{
    sns_neurons_cache_list_report_text, sns_neurons_cache_status_report_text,
    sns_neurons_refresh_report_text, sns_neurons_report_text,
};
#[cfg(feature = "host")]
pub use params::sns_params_report_text;
#[cfg(feature = "host")]
pub use proposals::{
    sns_proposal_report_text, sns_proposals_cache_list_report_text,
    sns_proposals_cache_status_report_text, sns_proposals_refresh_report_text,
    sns_proposals_report_text,
};
#[cfg(feature = "host")]
pub use token::sns_token_report_text;
