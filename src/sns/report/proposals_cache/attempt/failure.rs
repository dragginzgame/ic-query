//! Module: sns::report::proposals_cache::attempt::failure
//!
//! Responsibility: persist failed proposal refresh-attempt metadata.
//! Does not own: live proposal paging, cache publication, or error rendering.
//! Boundary: records the current refresh failure as best-effort attempt state.

use super::{
    model::{SnsProposalsAttemptContext, SnsProposalsAttemptProgress},
    write::write_attempt_status,
};
use crate::sns::report::SnsHostError;

/// Best-effort write of failed proposal refresh-attempt metadata.
pub(in crate::sns::report::proposals_cache) fn write_failed_attempt(
    context: SnsProposalsAttemptContext<'_>,
    err: &SnsHostError,
) {
    let _ = write_attempt_status(
        context,
        "failed",
        SnsProposalsAttemptProgress::starting(),
        Some(err.to_string()),
    );
}
