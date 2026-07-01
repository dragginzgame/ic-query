//! Module: sns::report::build
//!
//! Responsibility: group public SNS report builder entry points.
//! Does not own: command parsing, cache file primitives, source models, or rendering.
//! Boundary: exposes builders that coordinate lookup/source/cache reads and assembly.

mod info;
mod list;
mod neurons;
mod params;
mod proposals;
mod token;

pub use info::{build_sns_info_report, build_sns_info_report_with_source};
pub use list::{build_sns_list_report, build_sns_list_report_with_source};
pub use neurons::build_sns_neurons_report;
#[cfg(test)]
pub(in crate::sns::report) use neurons::build_sns_neurons_report_with_source;
pub use params::{build_sns_params_report, build_sns_params_report_with_source};
pub use proposals::{build_sns_proposal_report, build_sns_proposals_report};
#[cfg(test)]
pub(in crate::sns::report) use proposals::{
    build_sns_proposal_report_with_source, build_sns_proposals_report_with_source,
};
pub use token::{build_sns_token_report, build_sns_token_report_with_source};
