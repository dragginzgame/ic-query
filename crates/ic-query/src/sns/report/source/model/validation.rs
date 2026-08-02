//! Module: sns::report::source::model::validation
//!
//! Responsibility: share capability-aware validation of common SNS source evidence.
//! Does not own: capability-specific invariants, source DTOs, transport, or report projection.
//! Boundary: preserves each capability label while validating canonical principals and exact values.

use crate::sns::report::SnsHostError;
use candid::Principal;

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

    pub(super) fn exact(
        self,
        field: &'static str,
        expected: &str,
        actual: &str,
    ) -> Result<(), SnsHostError> {
        if actual != expected {
            return Err(self.invalid(format!("{field} is {actual:?}, expected {expected:?}")));
        }
        Ok(())
    }

    pub(super) fn canonical_principal(
        self,
        field: &'static str,
        value: &str,
    ) -> Result<(), SnsHostError> {
        let principal = Principal::from_text(value)
            .map_err(|error| self.invalid(format!("{field} {value:?} is invalid: {error}")))?;
        if principal.to_text() != value {
            return Err(self.invalid(format!("{field} {value:?} is not canonical principal text")));
        }
        Ok(())
    }
}
