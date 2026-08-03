//! Module: cache::completeness
//!
//! Responsibility: define and validate complete paged-collection evidence.
//! Does not own: family cache envelopes, row validation, or filesystem IO.
//! Boundary: preserves the raw persisted status while centralizing the one
//! complete-collection contract shared by NNS, SNS, and ICRC caches.

use serde::{Deserialize as SerdeDeserialize, Serialize};

const API_EXHAUSTED_STATUS: &str = "api_exhausted";

///
/// CacheCollectionCompleteness
///
/// Persisted evidence that a paged cache collection exhausted its source API.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct CacheCollectionCompleteness {
    /// Raw persisted completeness status; complete collections use `api_exhausted`.
    pub status: String,
    /// Maximum rows requested from the source per page.
    pub page_size: u32,
    /// Number of source pages collected.
    pub page_count: u32,
    /// Number of unique rows retained in the complete cache.
    pub row_count: usize,
    /// Whether every page is guaranteed to describe one source instant.
    pub point_in_time_guaranteed: bool,
}

impl CacheCollectionCompleteness {
    /// Construct evidence for an API-exhausted paged collection.
    #[must_use]
    pub fn api_exhausted(
        page_size: u32,
        page_count: u32,
        row_count: usize,
        point_in_time_guaranteed: bool,
    ) -> Self {
        Self {
            status: API_EXHAUSTED_STATUS.to_string(),
            page_size,
            page_count,
            row_count,
            point_in_time_guaranteed,
        }
    }

    /// Return whether the raw status claims source API exhaustion.
    #[must_use]
    pub fn is_api_exhausted(&self) -> bool {
        self.status == API_EXHAUSTED_STATUS
    }
}

/// Validate shared API-exhaustion, page, and row-count evidence.
pub fn validate_cache_collection_completeness(
    completeness: &CacheCollectionCompleteness,
    actual_row_count: usize,
) -> Result<(), String> {
    if !completeness.is_api_exhausted() {
        return Err(format!(
            "completeness status is {}, expected {API_EXHAUSTED_STATUS}",
            completeness.status
        ));
    }
    if completeness.page_size == 0 {
        return Err("completeness page_size must be greater than zero".to_string());
    }
    if completeness.page_count == 0 {
        return Err("completeness page_count must be greater than zero".to_string());
    }
    if completeness.row_count != actual_row_count {
        return Err(format!(
            "completeness row_count is {}, actual row count is {actual_row_count}",
            completeness.row_count
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completeness_requires_api_exhaustion_and_consistent_page_evidence() {
        for page_size in 0..=2 {
            for page_count in 0..=2 {
                for declared_row_count in 0..=2 {
                    for actual_row_count in 0..=2 {
                        let completeness = CacheCollectionCompleteness::api_exhausted(
                            page_size,
                            page_count,
                            declared_row_count,
                            false,
                        );
                        let expected_valid = page_size > 0
                            && page_count > 0
                            && declared_row_count == actual_row_count;

                        assert_eq!(
                            validate_cache_collection_completeness(&completeness, actual_row_count)
                                .is_ok(),
                            expected_valid,
                            "page_size={page_size}, page_count={page_count}, declared_row_count={declared_row_count}, actual_row_count={actual_row_count}"
                        );
                    }
                }
            }
        }

        let mut unknown = CacheCollectionCompleteness::api_exhausted(1, 1, 0, false);
        unknown.status = "unknown".to_string();
        assert_eq!(
            validate_cache_collection_completeness(&unknown, 0),
            Err("completeness status is unknown, expected api_exhausted".to_string())
        );
        assert_eq!(
            serde_json::to_value(CacheCollectionCompleteness::api_exhausted(1, 1, 0, false))
                .expect("serialize completeness")["status"],
            "api_exhausted"
        );
    }
}
