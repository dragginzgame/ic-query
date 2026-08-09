//! CLI surface for direct NNS Governance reports.

mod commands;
mod options;
mod run;

pub(in crate::nns) use commands::governance_command;
#[cfg(test)]
pub(in crate::nns) use commands::{
    governance_economics_command, governance_maturity_modulation_command,
    governance_metrics_command, governance_reward_event_command,
};
#[cfg(test)]
pub(in crate::nns) use options::NnsGovernanceOptions;
pub(super) use run::run;
