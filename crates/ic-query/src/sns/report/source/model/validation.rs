//! Module: sns::report::source::model::validation
//!
//! Responsibility: share capability-aware validation of common SNS source evidence and row limits.
//! Does not own: capability-specific invariants, source DTOs, transport, or report projection.
//! Boundary: preserves each capability label while validating principals, exact values, and row limits.

use crate::sns::report::SnsHostError;

///
/// SnsSourceValidator
///
/// Shared validator for common untrusted SNS source fields.
///

#[derive(Clone, Copy)]
pub(super) struct SnsSourceValidator {
    capability: &'static str,
}

impl SnsSourceValidator {
    pub(super) const fn new(capability: &'static str) -> Self {
        Self { capability }
    }

    pub(super) const fn invalid(self, reason: String) -> SnsHostError {
        SnsHostError::InvalidSourceData {
            capability: self.capability,
            reason,
        }
    }

    pub(super) fn row_limit(self, actual: usize, requested: u32) -> Result<(), SnsHostError> {
        if actual > requested as usize {
            return Err(self.invalid(format!(
                "returned {actual} rows for requested limit {requested}"
            )));
        }
        Ok(())
    }
}
