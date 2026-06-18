//! Module: sns::report::assemble
//!
//! Responsibility: group SNS report DTO assembly helpers.
//! Does not own: command parsing, source/cache reads, view transforms, or text rendering.
//! Boundary: converts resolved source/cache data into serializable report DTOs.

mod list;
mod neurons;
mod params;
mod proposals;
mod provenance;
mod token;

pub(super) use list::{sns_info_report_from_list, sns_list_report_from_list};
pub(super) use neurons::{SnsNeuronsLiveReportParts, sns_neurons_report_from_parts};
pub(super) use params::sns_params_report_from_parts;
pub(super) use proposals::{
    SnsProposalReportParts, SnsProposalsReportParts, sns_proposal_report_from_parts,
    sns_proposals_report_from_parts,
};
pub(super) use provenance::SnsReportProvenance;
pub(super) use token::sns_token_report_from_parts;
