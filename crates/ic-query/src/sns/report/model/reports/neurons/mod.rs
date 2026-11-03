//! Module: sns::report::model::reports::neurons
//!
//! Responsibility: group SNS neuron report DTOs.
//! Does not own: live neuron fetches, cache storage, sorting, or rendering.
//! Boundary: re-exports serializable neuron report models.

pub(in crate::sns::report) mod detail;
#[cfg(feature = "sns-host")]
mod report;
mod row;

pub use detail::{
    SnsMaturityDisbursementRow, SnsNeuronAccount, SnsNeuronDetail, SnsNeuronDetailReport,
    SnsNeuronFolloweeRow, SnsNeuronFolloweesRow, SnsNeuronPermissionRow,
    SnsNeuronTopicFolloweesRow, SnsPolicyObservationStatus, sns_neuron_permission_name,
};
#[cfg(feature = "sns-host")]
pub use report::SnsNeuronsReport;

pub use row::{SnsNeuronDissolveState, SnsNeuronRow};
