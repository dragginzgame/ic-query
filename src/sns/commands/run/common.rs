use crate::{cli::common::current_unix_secs, project::icp_root, sns::commands::SnsCommandError};
use std::path::PathBuf;

pub(super) fn command_unix_secs() -> Result<u64, SnsCommandError> {
    current_unix_secs().map_err(SnsCommandError::Clock)
}

pub(super) fn command_icp_root() -> Result<PathBuf, SnsCommandError> {
    icp_root().map_err(|err| SnsCommandError::Usage(err.to_string()))
}
