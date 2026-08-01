//! CLI surface for direct NNS Governance reports.

mod commands;
mod options;
mod run;

#[cfg(test)]
pub(in crate::nns) use commands::{
    governance_economics_command, governance_economics_usage,
    governance_maturity_modulation_command, governance_maturity_modulation_usage,
    governance_metrics_command, governance_metrics_usage, governance_reward_event_command,
    governance_reward_event_usage, governance_usage,
};
#[cfg(test)]
pub(in crate::nns) use options::NnsGovernanceOptions;
pub(super) use run::{command, run};
