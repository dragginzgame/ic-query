//! Module: subnet_catalog::time
//!
//! Responsibility: derive catalog staleness metadata from canonical timestamps.
//!
//! Does not own: timestamp codecs, cache refresh policy, host IO, or report rendering.
//!
//! Boundary: Host owns UTC parsing; callers supply the observation time and age limit.

use super::{CatalogStaleStatus, RawSubnetCatalog};
use crate::freshness::freshness_facts;
use ic_host_text::timestamp::parse_utc_timestamp_secs;

/// Computes stale/fresh metadata for a catalog relative to a caller-provided time.
#[must_use]
pub fn catalog_stale_status(
    catalog: &RawSubnetCatalog,
    now_unix_secs: u64,
    stale_after_seconds: u64,
) -> CatalogStaleStatus {
    let freshness = freshness_facts(
        parse_utc_timestamp_secs(&catalog.provenance.fetched_at),
        now_unix_secs,
        stale_after_seconds,
    );
    CatalogStaleStatus {
        catalog_stale: freshness.stale,
        stale_reason: freshness.reason.to_string(),
        stale_after_seconds: freshness.stale_after_seconds,
        fetched_at_unix_secs: freshness.fetched_at_unix_secs,
        age_seconds: freshness.age_seconds,
    }
}
