//! Module: sns::report::reward_checkpoint_file
//!
//! Responsibility: load local SNS reward-checkpoint JSON for reconciliation.
//! Does not own: process arguments, output, live calls, or checkpoint collection.
//! Boundary: filesystem parsing is host-only while reconciliation remains pure.

use crate::sns::report::{
    SnsHostError, SnsRewardCheckpointReport, SnsRewardDiffReport, build_sns_reward_diff_report,
};
use std::{fs, path::Path};

/// Load one strict SNS reward-checkpoint JSON document from a caller-selected path.
pub fn load_sns_reward_checkpoint(path: &Path) -> Result<SnsRewardCheckpointReport, SnsHostError> {
    let data = fs::read_to_string(path).map_err(|source| SnsHostError::ReadRewardCheckpoint {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&data).map_err(|source| SnsHostError::ParseRewardCheckpoint {
        path: path.to_path_buf(),
        source,
    })
}

/// Load and purely reconcile two caller-selected SNS reward-checkpoint files.
pub fn build_sns_reward_diff_report_from_paths(
    before_path: &Path,
    after_path: &Path,
) -> Result<SnsRewardDiffReport, SnsHostError> {
    let before = load_sns_reward_checkpoint(before_path)?;
    let after = load_sns_reward_checkpoint(after_path)?;
    Ok(build_sns_reward_diff_report(&before, &after))
}
