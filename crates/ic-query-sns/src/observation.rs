//! Per-read outcomes and explicit observation boundaries.

use serde::Serialize;

/// Stable failure category; diagnostics never determine control flow.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    /// Endpoint, transport, certificate or node-signature verification failed.
    Read,
    /// Whole collection or leaf deadline expired.
    Deadline,
    /// Local Unix time could not be represented.
    Clock,
    /// Request could not be encoded within its bound.
    Encode,
    /// Response could not be decoded within its bound.
    Decode,
    /// Response violated the selected report contract.
    Admission,
    /// Governance returned its explicit error variant.
    Governance,
    /// Governance omitted the result field.
    MissingResult,
}

/// Retained failure with an optional raw Governance error code.
#[derive(Clone, Debug, Serialize, thiserror::Error)]
#[error("{message}")]
pub struct Failure {
    /// Stable classification.
    pub kind: FailureKind,
    /// Bounded source or local diagnostic; not machine-interpreted.
    pub message: String,
    /// Exact Governance error code, including unknown signed codes.
    pub governance_error_type: Option<i32>,
}

impl Failure {
    pub(crate) fn new(kind: FailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            governance_error_type: None,
        }
    }
}

/// One independent read, never an implicit zero or empty success on failure.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Outcome<T> {
    /// Admitted source data.
    Read {
        /// Exact projected value.
        value: T,
    },
    /// Unavailable evidence.
    Failed {
        /// Failure retained for this read.
        error: Failure,
    },
}

/// Local observation times and per-call authentication, distinct from state time.
#[derive(Clone, Debug, Serialize)]
pub struct Observation<T> {
    /// Exact source method.
    pub method: &'static str,
    /// Local start time; absent only on clock failure.
    pub started_at_unix_ms: Option<u64>,
    /// Local finish time; absent only on clock failure.
    pub finished_at_unix_ms: Option<u64>,
    /// True only after Platform returned an authenticated query response.
    pub query_signature_verified: bool,
    /// Query responses do not certify application state.
    pub result_certified: bool,
    /// Independent value or failure.
    pub outcome: Outcome<T>,
}

impl<T> Observation<T> {
    /// Whether this observation yielded admitted data.
    #[must_use]
    pub fn is_read(&self) -> bool {
        matches!(self.outcome, Outcome::Read { .. })
    }

    pub(crate) fn project<U>(self, f: impl FnOnce(T) -> Result<U, Failure>) -> Observation<U> {
        Observation {
            method: self.method,
            started_at_unix_ms: self.started_at_unix_ms,
            finished_at_unix_ms: self.finished_at_unix_ms,
            query_signature_verified: self.query_signature_verified,
            result_certified: self.result_certified,
            outcome: match self.outcome {
                Outcome::Read { value } => match f(value) {
                    Ok(value) => Outcome::Read { value },
                    Err(error) => Outcome::Failed { error },
                },
                Outcome::Failed { error } => Outcome::Failed { error },
            },
        }
    }
}
