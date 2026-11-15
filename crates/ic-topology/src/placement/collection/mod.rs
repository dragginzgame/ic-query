//! Explicit bounded canister selections; no fleet discovery or deployment authority.
use super::{PlacementError, PlacementReport};
use crate::{ReadError, mainnet};
use ic_platform_types::Principal;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::{collections::BTreeSet, future::Future, time::Duration};
use thiserror::Error;
use tokio::time::{Instant, sleep_until};

/// Largest explicit selection admitted before any lookup.
pub const MAX_CANISTERS: usize = 32;
/// Shared asynchronous budget across sequential leaf lookups.
pub const DEADLINE: Duration = Duration::from_secs(60);

/// Independent placement evidence in the caller's selected order.
#[derive(Debug, Serialize)]
pub struct PlacementCollectionReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// Always false: this source supplies no common Registry version.
    pub atomic_snapshot: bool,
    /// Local collection start, in Unix seconds.
    pub started_at_unix_seconds: u64,
    /// Monotonic duration of collection.
    pub elapsed: Duration,
    /// True only when every selected lookup succeeded with the selected identity.
    pub collection_complete: bool,
    /// Whether the shared budget stopped collection.
    pub overall_deadline_reached: bool,
    /// Exactly one outcome per selected canister, in argument order.
    pub canisters: Vec<CanisterPlacement>,
}

/// One selected lookup; an unavailable placement does not establish absence.
#[derive(Debug, Serialize)]
pub struct CanisterPlacement {
    /// The consumer's selected identity.
    pub canister_id: Principal,
    /// Outcome of this lookup only.
    pub outcome: PlacementOutcome,
}

/// Missing evidence remains explicit and never becomes a fabricated subnet.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PlacementOutcome {
    /// Successful matching lookup with its original source evidence.
    Read {
        /// Independent Registry routing observation.
        report: PlacementReport,
    },
    /// Leaf failure; later selected lookups may still succeed.
    Failed {
        /// Original typed cause; JSON retains its category and Registry rejection.
        #[serde(serialize_with = "serialize_error")]
        error: PlacementError,
    },
    /// Refused a report for a different selected identity.
    IdentityMismatch {
        /// Identity unexpectedly returned by the reader.
        observed_canister_id: Principal,
    },
    /// Lookup started but was cancelled by the shared deadline.
    OverallDeadline,
    /// Shared deadline prevented the lookup from starting.
    NotStarted,
}

/// Admission failures produce no collection and make no network calls.
#[derive(Debug, Error)]
pub enum CollectionError {
    /// Empty or excessive selections are refused rather than truncated.
    #[error("select between 1 and {MAX_CANISTERS} canisters; received {count}")]
    SelectionSize {
        /// Supplied number of identities.
        count: usize,
    },
    /// Duplicate selections are refused rather than silently deduplicated.
    #[error("duplicate selected canister: {canister_id}")]
    Duplicate {
        /// Repeated identity.
        canister_id: Principal,
    },
    /// The local start clock could not be read.
    #[error(transparent)]
    Read(#[from] ReadError),
}

/// Observe 1–32 unique mainnet canister placements in selection order.
///
/// Each sequential lookup retains the leaf's 20-second deadline and reply bounds;
/// all share 60 asynchronous seconds. No retry, spawned task, scan or persistence.
/// Dropping this future cancels the active lookup. Bounded synchronous work is
/// separate from the asynchronous deadline. Requires Tokio IO and time.
/// A successful return can contain only failures; inspect `collection_complete`.
/// Routing evidence proves neither canister existence nor ownership or liveness.
///
/// # Errors
/// Returns selection-size, duplicate-selection or initial clock errors before IO.
pub async fn fetch(
    canister_ids: &[Principal],
) -> Result<PlacementCollectionReport, CollectionError> {
    validate(canister_ids)?;
    let started_at = mainnet::observed_at()?;
    Ok(collect(canister_ids, started_at, super::fetch).await)
}

fn validate(ids: &[Principal]) -> Result<(), CollectionError> {
    if ids.is_empty() || ids.len() > MAX_CANISTERS {
        return Err(CollectionError::SelectionSize { count: ids.len() });
    }
    let mut seen = BTreeSet::new();
    for &canister_id in ids {
        if !seen.insert(canister_id) {
            return Err(CollectionError::Duplicate { canister_id });
        }
    }
    Ok(())
}

async fn collect<F>(
    ids: &[Principal],
    started_at: u64,
    reader: impl Fn(Principal) -> F,
) -> PlacementCollectionReport
where
    F: Future<Output = Result<PlacementReport, PlacementError>>,
{
    let started = Instant::now();
    let deadline = started + DEADLINE;
    let mut canisters: Vec<_> = ids
        .iter()
        .map(|&canister_id| CanisterPlacement {
            canister_id,
            outcome: PlacementOutcome::NotStarted,
        })
        .collect();
    let mut overall_deadline_reached = false;
    for row in &mut canisters {
        if Instant::now() >= deadline {
            overall_deadline_reached = true;
            break;
        }
        row.outcome = PlacementOutcome::OverallDeadline;
        let result = tokio::select! {
            biased;
            _ = sleep_until(deadline) => {
                overall_deadline_reached = true;
                break;
            }
            result = reader(row.canister_id) => result,
        };
        row.outcome = match result {
            Ok(report) if report.canister_id == row.canister_id => {
                PlacementOutcome::Read { report }
            }
            Ok(report) => PlacementOutcome::IdentityMismatch {
                observed_canister_id: report.canister_id,
            },
            Err(error) => PlacementOutcome::Failed { error },
        };
    }
    PlacementCollectionReport {
        schema: 1,
        atomic_snapshot: false,
        started_at_unix_seconds: started_at,
        elapsed: started.elapsed(),
        collection_complete: canisters
            .iter()
            .all(|row| matches!(row.outcome, PlacementOutcome::Read { .. })),
        overall_deadline_reached,
        canisters,
    }
}

fn serialize_error<S: Serializer>(
    error: &PlacementError,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let kind = match error {
        PlacementError::SourceIdentity(_) => "source_identity",
        PlacementError::Read(ReadError::Transport(_)) => "transport_or_verification",
        PlacementError::Read(ReadError::Deadline) => "read_deadline",
        PlacementError::Read(ReadError::Clock(_)) => "clock",
        PlacementError::Encode(_) => "encode",
        PlacementError::Decode(_) => "decode",
        PlacementError::RegistryRejected(_) => "registry_rejected",
        PlacementError::MissingSubnet => "missing_subnet",
    };
    let mut fields = serializer.serialize_struct(
        "PlacementError",
        if matches!(error, PlacementError::RegistryRejected(_)) {
            2
        } else {
            1
        },
    )?;
    fields.serialize_field("kind", kind)?;
    if let PlacementError::RegistryRejected(reason) = error {
        fields.serialize_field("reason", reason)?;
    }
    fields.end()
}

#[cfg(test)]
mod tests;
