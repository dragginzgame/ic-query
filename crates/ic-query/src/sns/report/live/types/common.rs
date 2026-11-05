//! Native error returned by exact neuron lookup.
use ic_platform::candid::contract::{CandidType, serde::Deserialize};
///
/// SnsGovernanceError
///
/// Candid SNS governance error embedded in exact neuron responses.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::sns::report::live) struct SnsGovernanceError {
    pub(in crate::sns::report::live) error_type: i32,
    pub(in crate::sns::report::live) error_message: String,
}
