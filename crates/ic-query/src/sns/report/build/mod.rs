//! Module: sns::report::build
//!
//! Responsibility: group public SNS report builder entry points.
//! Does not own: command parsing, cache file primitives, source models, or rendering.
//! Boundary: exposes builders that coordinate lookup/source/cache reads and assembly.

mod canisters;
mod info;
mod list;
mod metrics;
mod neuron;
mod neurons;
mod params;
mod proposals;
mod reward;
mod swap;
mod token;
mod upgrade;

pub use canisters::{build_sns_canister_report, build_sns_canister_report_with_source};
pub use info::{build_sns_info_report, build_sns_info_report_with_source};
pub use list::{build_sns_list_report, build_sns_list_report_with_source};
pub use metrics::{build_sns_metrics_report, build_sns_metrics_report_with_source};
pub use neuron::{build_sns_neuron_detail_report, build_sns_neuron_detail_report_with_source};
pub use neurons::{build_sns_neurons_report, build_sns_neurons_report_with_source};
pub use params::{build_sns_params_report, build_sns_params_report_with_source};
pub use proposals::{
    build_sns_proposal_report, build_sns_proposal_report_with_source, build_sns_proposals_report,
    build_sns_proposals_report_with_progress, build_sns_proposals_report_with_source,
};
pub use reward::{
    build_sns_reward_checkpoint_report, build_sns_reward_checkpoint_report_with_source,
};
pub use swap::{build_sns_swap_report, build_sns_swap_report_with_source};
pub use token::{build_sns_token_report, build_sns_token_report_with_source};
pub use upgrade::{build_sns_upgrade_report, build_sns_upgrade_report_with_source};
