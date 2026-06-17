//! Module: sns::report::live::convert::proposals::timestamp
//!
//! Responsibility: normalize SNS governance proposal timestamps.
//! Does not own: timestamp formatting policy outside proposal conversion.
//! Boundary: treats zero proposal timestamps as absent optional values.

use crate::subnet_catalog::format_utc_timestamp_secs;

/// Convert zero-valued SNS proposal timestamps to `None`.
pub(super) const fn nonzero_timestamp(timestamp_seconds: u64) -> Option<u64> {
    if timestamp_seconds > 0 {
        Some(timestamp_seconds)
    } else {
        None
    }
}

/// Format a nonzero SNS proposal timestamp as UTC text.
pub(super) fn optional_timestamp_text(timestamp_seconds: u64) -> Option<String> {
    nonzero_timestamp(timestamp_seconds).map(format_utc_timestamp_secs)
}
