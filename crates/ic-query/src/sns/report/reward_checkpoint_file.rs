//! Module: sns::report::reward_checkpoint_file
//!
//! Responsibility: load local SNS reward-checkpoint JSON for reconciliation.
//! Does not own: process arguments, output, live calls, or checkpoint collection.
//! Boundary: filesystem parsing is host-only while reconciliation remains pure.

use crate::sns::report::{
    SnsHostError, SnsRewardCheckpointReport, SnsRewardDiffReport, build_sns_reward_diff_report,
};
use ic_host_fs::read::read_file;
use std::{io, path::Path};

const CHECKPOINT_BYTES: usize = 64 * 1024 * 1024;

/// Load one strict SNS reward-checkpoint JSON document, at most 64 MiB, from a selected path.
pub fn load_sns_reward_checkpoint(path: &Path) -> Result<SnsRewardCheckpointReport, SnsHostError> {
    let data =
        read_file(path, CHECKPOINT_BYTES).map_err(|source| SnsHostError::ReadRewardCheckpoint {
            path: path.to_path_buf(),
            source: io::Error::from(source),
        })?;
    serde_json::from_slice(&data).map_err(|source| SnsHostError::ParseRewardCheckpoint {
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
