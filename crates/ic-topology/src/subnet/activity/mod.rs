//! Two certified metrics reads for bounded activity evidence, without an uptime verdict.

use std::time::Duration;

use ic_platform_types::Principal;
use serde::Serialize;
use thiserror::Error;

use super::metrics::{self, SubnetMetricsError, SubnetMetricsReport};

/// A comparison of two independently certified observations, in acquisition order.
#[derive(Debug, Serialize)]
pub struct SubnetActivityReport {
    /// Query-owned report schema; remains 1 before 1.0.
    pub schema: u8,
    /// Requested wait after the first read; not a certified observation window.
    pub interval_seconds: u64,
    /// First complete metrics observation and its evidence.
    pub before: SubnetMetricsReport,
    /// Second complete metrics observation and its evidence.
    pub after: SubnetMetricsReport,
    /// Difference between observed replicated-message totals, not throughput.
    pub message_change: MessageChange,
}

/// Counter ordering across the two observations, without inferring uptime or resets.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MessageChange {
    /// The second observed total is larger.
    Increased {
        /// Exact counter difference; no certified time interval is established.
        by: u64,
    },
    /// Totals are equal; low traffic or repeated state may explain this.
    Unchanged,
    /// The second total is lower; cause and certified-state ordering are unknown.
    Decreased,
}

/// Unavailable activity evidence; no failure establishes subnet downtime.
#[derive(Debug, Error)]
pub enum SubnetActivityError {
    /// Reject an unbounded or zero wait before constructing a reader.
    #[error("activity interval must be between 1 and 60 seconds")]
    InvalidInterval,
    /// The first read failed; the second was not attempted.
    #[error("first activity observation failed: {0}")]
    First(#[source] SubnetMetricsError),
    /// The second read failed; the first observation remains available to callers.
    #[error("second activity observation failed after retaining the first: {source}")]
    Second {
        /// Successful first observation; not a complete activity report.
        before: Box<SubnetMetricsReport>,
        /// Original typed read failure.
        #[source]
        source: SubnetMetricsError,
    },
}

/// Read one subnet twice, waiting 1–60 seconds after the first completes.
///
/// Requires a Tokio runtime with IO and time. Each read uses the metrics reader's
/// 20-second asynchronous deadline and 1 MiB HTTP ceiling. No retries or tasks
/// are spawned. Two samples do not establish uptime, rates or certified elapsed
/// time; repeated or differently aged certified state remains possible.
///
/// # Errors
/// Refuses an invalid interval before network IO. Read failures preserve their
/// typed causes and, for a second-read failure, the successful first observation.
pub async fn fetch(
    subnet_id: Principal,
    interval_seconds: u64,
) -> Result<SubnetActivityReport, SubnetActivityError> {
    sample(interval_seconds, || metrics::fetch(subnet_id)).await
}

async fn sample<F, Fut>(
    interval_seconds: u64,
    mut read: F,
) -> Result<SubnetActivityReport, SubnetActivityError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<SubnetMetricsReport, SubnetMetricsError>>,
{
    if !(1..=60).contains(&interval_seconds) {
        return Err(SubnetActivityError::InvalidInterval);
    }
    let before = read().await.map_err(SubnetActivityError::First)?;
    tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
    let after = match read().await {
        Ok(after) => after,
        Err(source) => {
            return Err(SubnetActivityError::Second {
                before: Box::new(before),
                source,
            });
        }
    };
    let message_change = match after
        .update_transactions_total
        .cmp(&before.update_transactions_total)
    {
        std::cmp::Ordering::Greater => MessageChange::Increased {
            by: after.update_transactions_total - before.update_transactions_total,
        },
        std::cmp::Ordering::Equal => MessageChange::Unchanged,
        std::cmp::Ordering::Less => MessageChange::Decreased,
    };
    Ok(SubnetActivityReport {
        schema: 1,
        interval_seconds,
        before,
        after,
        message_change,
    })
}

#[cfg(test)]
mod tests;
