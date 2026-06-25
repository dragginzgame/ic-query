//! Module: sns::report::text
//!
//! Responsibility: group SNS text report renderers.
//! Does not own: report construction, source/cache reads, or JSON output.
//! Boundary: converts already-built report DTOs into human-readable text.

mod common;
mod info;
mod list;
mod neurons;
mod params;
mod proposals;
mod token;

#[cfg(test)]
pub(super) use common::optional_e8s_decimal_text;
pub use info::sns_info_report_text;
pub use list::sns_list_report_text;
pub use neurons::{
    sns_neurons_cache_list_report_text, sns_neurons_cache_status_report_text,
    sns_neurons_refresh_report_text, sns_neurons_report_text,
};
pub use params::sns_params_report_text;
pub use proposals::{
    sns_proposal_report_text, sns_proposals_cache_list_report_text,
    sns_proposals_cache_status_report_text, sns_proposals_refresh_report_text,
    sns_proposals_report_text,
};
pub use token::sns_token_report_text;
