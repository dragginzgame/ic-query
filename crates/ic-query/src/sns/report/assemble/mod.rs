//! Module: sns::report::assemble
//!
//! Responsibility: group SNS report DTO assembly helpers.
//! Does not own: command parsing, source/cache reads, view transforms, or text rendering.
//! Boundary: converts resolved source/cache data into serializable report DTOs.

mod canisters;
mod list;
mod metrics;
mod neuron;
mod neurons;
mod params;
mod proposals;
mod provenance;
mod reward;
mod swap;
mod token;
mod upgrade;

pub(super) use canisters::sns_canister_report_from_parts;
pub(super) use list::{sns_info_report_from_list, sns_list_report_from_list};
pub(super) use metrics::sns_metrics_report_from_parts;
pub(super) use neuron::{SnsNeuronDetailReportParts, sns_neuron_detail_report_from_parts};
pub(super) use neurons::{SnsNeuronsLiveReportParts, sns_neurons_report_from_parts};
pub(super) use params::sns_params_report_from_parts;
pub(super) use proposals::{
    SnsProposalReportParts, SnsProposalsReportParts, sns_proposal_report_from_parts,
    sns_proposals_report_from_parts,
};
pub(super) use provenance::SnsReportProvenance;
pub(super) use reward::{SnsRewardCheckpointReportParts, sns_reward_checkpoint_report_from_parts};
pub(super) use swap::sns_swap_report_from_parts;
pub(super) use token::sns_token_report_from_parts;
pub(super) use upgrade::sns_upgrade_report_from_parts;
