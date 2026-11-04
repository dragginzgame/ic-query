//! Module: sns::report::model::reports::invocation
//!
//! Responsibility: shared typed provenance for native SNS canister invocations.
//! Does not own: transport, source validation, report assembly, or rendering.
//! Boundary: defines the closed call types and methods used by bounded SNS reports.

use serde::{Deserialize, Serialize};

///
/// SnsCanisterCallType
///
/// Invocation mode used for a native SNS canister method.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsCanisterCallType {
    /// A non-replicated query call.
    Query,
    /// A non-replicated composite query call.
    CompositeQuery,
    /// A replicated update submitted through ingress.
    IngressUpdate,
}

impl SnsCanisterCallType {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::CompositeQuery => "composite_query",
            Self::IngressUpdate => "ingress_update",
        }
    }
}

///
/// SnsCanisterMethod
///
/// Native canister method invoked or recorded by a typed SNS report adapter.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsCanisterMethod {
    /// SNS-W deployed-instance inventory query.
    ListDeployedSnses,
    /// SNS Governance metadata query.
    GetMetadata,
    /// SNS Root inventory query.
    ListSnsCanisters,
    /// SNS Root operational-health update.
    GetSnsCanistersSummary,
    /// SNS Governance cached-metrics composite query.
    GetMetrics,
    /// SNS Governance nervous-system-parameters query.
    GetNervousSystemParameters,
    /// SNS Governance bounded proposal-list query.
    ListProposals,
    /// SNS Governance exact proposal query.
    GetProposal,
    /// SNS Governance bounded neuron-list query.
    ListNeurons,
    /// SNS Governance exact neuron query.
    GetNeuron,
    /// SNS Governance latest-reward-event query.
    GetLatestRewardEvent,
    /// SNS swap lifecycle query.
    GetLifecycle,
    /// SNS swap sale-parameters query.
    GetSaleParameters,
    /// SNS swap derived-state query.
    GetDerivedState,
    /// SNS Governance running-version query.
    GetRunningSnsVersion,
    /// SNS-W next-blessed-version query.
    GetNextSnsVersion,
}

impl SnsCanisterMethod {
    /// Return the exact native Candid method name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ListDeployedSnses => "list_deployed_snses",
            Self::GetMetadata => "get_metadata",
            Self::ListSnsCanisters => "list_sns_canisters",
            Self::GetSnsCanistersSummary => "get_sns_canisters_summary",
            Self::GetMetrics => "get_metrics",
            Self::GetNervousSystemParameters => "get_nervous_system_parameters",
            Self::ListProposals => "list_proposals",
            Self::GetProposal => "get_proposal",
            Self::ListNeurons => "list_neurons",
            Self::GetNeuron => "get_neuron",
            Self::GetLatestRewardEvent => "get_latest_reward_event",
            Self::GetLifecycle => "get_lifecycle",
            Self::GetSaleParameters => "get_sale_parameters",
            Self::GetDerivedState => "get_derived_state",
            Self::GetRunningSnsVersion => "get_running_sns_version",
            Self::GetNextSnsVersion => "get_next_sns_version",
        }
    }
}

#[cfg(test)]
mod tests;
