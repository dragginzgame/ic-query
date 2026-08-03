//! Module: sns::report::model::reports::neurons
//!
//! Responsibility: group SNS neuron report DTOs.
//! Does not own: live neuron fetches, cache storage, sorting, or rendering.
//! Boundary: re-exports serializable neuron report models.

mod detail;
mod diff;
#[cfg(feature = "host")]
mod refresh;
#[cfg(feature = "host")]
mod report;
mod reward;
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
#[cfg(feature = "host")]
pub use refresh::SnsNeuronsRefreshReport;
#[cfg(feature = "host")]
pub use report::SnsNeuronsReport;
pub use reward::{
    SnsRewardCheckpointReport, SnsRewardCheckpointRow, SnsRewardCheckpointValidationError,
    SnsRewardCollectionStatus, SnsRewardEvent, SnsRewardProposalId,
    validate_sns_reward_checkpoint_report,
};
pub(in crate::sns::report) use reward::{
    SnsRewardCheckpointSummary, recompute_reward_checkpoint_summary,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use reward::{
    validate_sns_reward_checkpoint_parameter_evidence, validate_sns_reward_event_evidence,
    validate_sns_reward_running_version_evidence,
};
pub use row::{SnsNeuronDissolveState, SnsNeuronRow};
