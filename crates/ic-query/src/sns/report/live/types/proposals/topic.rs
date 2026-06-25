//! Module: sns::report::live::types::proposals::topic
//!
//! Responsibility: SNS governance proposal topic filter wire types.
//! Does not own: CLI value parsing, request construction, or rendering.
//! Boundary: models Candid topic selectors for proposal list calls.

use candid::{CandidType, Deserialize};

///
/// SnsTopicSelector
///
/// Candid wrapper for one SNS governance proposal topic selector.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsTopicSelector {
    pub(in crate::sns::report::live) topic: Option<SnsTopic>,
}

///
/// SnsTopic
///
/// Candid SNS governance proposal topic enum used by list filters.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum SnsTopic {
    DaoCommunitySettings,
    SnsFrameworkManagement,
    DappCanisterManagement,
    ApplicationBusinessLogic,
    Governance,
    TreasuryAssetManagement,
    CriticalDappOperations,
}
