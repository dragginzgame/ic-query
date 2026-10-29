//! Module: sns::report::assemble
//!
//! Responsibility: group SNS report DTO assembly helpers.
//! Does not own: command parsing, source/cache reads, view algorithms, or text rendering.
//! Boundary: applies report views and converts resolved source/cache data into report DTOs.

mod list;
mod neurons;
mod proposals;
mod provenance;

pub(super) use list::sns_list_report_from_list;
pub(super) use neurons::sns_neurons_report_from_rows;
pub(super) use proposals::{sns_proposal_report_from_parts, sns_proposals_report_from_parts};
pub(super) use provenance::SnsReportContext;
