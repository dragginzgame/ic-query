//! Module: sns::report::model::reports::neurons
//!
//! Responsibility: group SNS neuron report DTOs.
//! Does not own: live neuron fetches, cache storage, sorting, or rendering.
//! Boundary: re-exports serializable neuron report models.

pub(in crate::sns::report) mod detail;
mod diff;
#[cfg(feature = "sns-host")]
mod report;
pub(in crate::sns::report) mod reward;
mod row;

pub use detail::{
    SnsMaturityDisbursementRow, SnsNeuronAccount, SnsNeuronDetail, SnsNeuronDetailReport,
    SnsNeuronFolloweeRow, SnsNeuronFolloweesRow, SnsNeuronPermissionRow, SnsNeuronPermissionValue,
    SnsNeuronTopicFolloweesRow, SnsPolicyObservationStatus, sns_neuron_permission_name,
};
pub use diff::{
    SnsRewardAllocationStatus, SnsRewardDiffCheckpointRef, SnsRewardDiffInvalidReason,
    SnsRewardDiffInvalidReasonKind, SnsRewardDiffReport, SnsRewardDiffRow,
};
#[cfg(feature = "sns-host")]
pub use report::SnsNeuronsReport;
pub use reward::{
    SnsRewardCheckpointReport, SnsRewardCheckpointRow, SnsRewardCheckpointSummary,
    SnsRewardCheckpointValidationError, SnsRewardCollectionStatus, SnsRewardEvent,
    SnsRewardProposalId, validate_sns_reward_checkpoint_report,
};
pub use row::{SnsNeuronDissolveState, SnsNeuronRow};
