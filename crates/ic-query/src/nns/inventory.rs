//! Module: nns::inventory
//!
//! Responsibility: share Registry-inventory lifecycle and row-resolution mechanics.
//! Does not own: family DTOs, source implementations, cache identity, projection, or public errors.
//! Boundary: one control flow serves node, provider, operator, and data-center reports.

use super::{
    NnsInventoryCacheRequest, NnsInventoryListRequest, NnsInventoryRefreshRequest, NnsSourceRequest,
};
use crate::{
    HostCacheError,
    cache_file::{
        CacheRefreshReason, RefreshCacheWriteResult, load_or_refresh_cache_with_error_policy,
    },
    nns::leaf::write_nns_leaf_json_refresh_cache,
    subnet_catalog::{
        MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, canonical_principal_text,
        format_utc_timestamp_secs, parse_utc_timestamp_secs,
    },
};
use serde::Serialize;
use std::path::PathBuf;

///
/// NnsInventoryHostError
///
/// Internal conversion from a family host error to the shared read-through policy.
///

pub(in crate::nns) trait NnsInventoryHostError: Sized {
    /// Classify recoverable cache content failures, preserving filesystem and source errors.
    fn cache_refresh_reason(self, expected_path: PathBuf) -> Result<CacheRefreshReason, Self>;
    /// Build a family error for semantically invalid cached inventory evidence.
    fn invalid_cache(path: PathBuf, reason: String) -> Self;
    /// Build a family error for a custom source that violates the inventory contract.
    fn invalid_source_data(reason: String) -> Self;
}

///
/// NnsInventoryReport
///
/// Shared provenance and count contract for one complete NNS inventory report.
///

pub(in crate::nns) trait NnsInventoryReport {
    const SCHEMA_VERSION: u32;
    const ITEM_NAME: &'static str;

    fn schema_version(&self) -> u32;
    fn network(&self) -> &str;
    fn registry_canister_id(&self) -> &str;
    fn fetched_at(&self) -> &str;
    fn source_endpoint(&self) -> &str;
    fn declared_row_count(&self) -> usize;
    fn row_count(&self) -> usize;

    /// Validate cache-stable provenance, count, and family-specific evidence.
    fn validate(&self) -> Result<(), String> {
        if self.schema_version() != Self::SCHEMA_VERSION {
            return Err(format!(
                "schema_version is {}, expected {}",
                self.schema_version(),
                Self::SCHEMA_VERSION
            ));
        }
        if self.network() != MAINNET_NETWORK {
            return Err(format!(
                "network is {}, expected {MAINNET_NETWORK}",
                self.network()
            ));
        }
        if self.registry_canister_id() != MAINNET_REGISTRY_CANISTER_ID {
            return Err(format!(
                "registry_canister_id is {}, expected {MAINNET_REGISTRY_CANISTER_ID}",
                self.registry_canister_id()
            ));
        }
        if parse_utc_timestamp_secs(self.fetched_at()).is_none() {
            return Err(format!("fetched_at is invalid: {:?}", self.fetched_at()));
        }
        if self.source_endpoint().trim().is_empty() {
            return Err("source_endpoint must not be empty".to_string());
        }
        if self.declared_row_count() != self.row_count() {
            return Err(format!(
                "{}_count is {}, expected {} from rows",
                Self::ITEM_NAME,
                self.declared_row_count(),
                self.row_count()
            ));
        }
        self.validate_family()
    }

    /// Validate evidence owned only by one inventory family.
    fn validate_family(&self) -> Result<(), String> {
        Ok(())
    }

    /// Validate source provenance against the exact request used for refresh.
    fn validate_source(&self, request: &NnsSourceRequest) -> Result<(), String> {
        self.validate()?;
        if self.network() != request.network {
            return Err(format!(
                "network is {}, requested {}",
                self.network(),
                request.network
            ));
        }
        if self.fetched_at() != request.fetched_at {
            return Err(format!(
                "fetched_at is {:?}, requested {:?}",
                self.fetched_at(),
                request.fetched_at
            ));
        }
        if self.source_endpoint() != request.endpoint {
            return Err(format!(
                "source_endpoint is {:?}, requested {:?}",
                self.source_endpoint(),
                request.endpoint
            ));
        }
        Ok(())
    }
}

///
/// NnsInventoryListInput
///
/// Internal view of the collection inputs shared by Registry inventory list requests.
///

pub(in crate::nns) trait NnsInventoryListInput {
    /// Cache identity for the complete inventory snapshot.
    fn cache(&self) -> &NnsInventoryCacheRequest;
    /// Explicit live endpoint used only when the cache needs read-through refresh.
    fn source_endpoint(&self) -> &str;
    /// Caller-provided timestamp used by a read-through refresh.
    fn now_unix_secs(&self) -> u64;
}

impl NnsInventoryListInput for NnsInventoryListRequest {
    fn cache(&self) -> &NnsInventoryCacheRequest {
        &self.cache
    }

    fn source_endpoint(&self) -> &str {
        &self.source_endpoint
    }

    fn now_unix_secs(&self) -> u64 {
        self.now_unix_secs
    }
}

/// Load one Registry inventory cache, refreshing missing or invalid local content.
pub(in crate::nns) fn load_or_refresh_nns_inventory_report<Report, Error>(
    request: &impl NnsInventoryListInput,
    expected_cache_path: PathBuf,
    lock_stale_after_seconds: u64,
    mut load: impl FnMut(&NnsInventoryCacheRequest) -> Result<Report, Error>,
    refresh: impl FnOnce(&NnsInventoryRefreshRequest) -> Result<(), Error>,
) -> Result<Report, Error>
where
    Error: NnsInventoryHostError,
{
    load_or_refresh_cache_with_error_policy(
        || load(request.cache()),
        |error| error.cache_refresh_reason(expected_cache_path),
        |_| {
            let refresh_request = NnsInventoryRefreshRequest::new(
                request.cache().clone(),
                request.source_endpoint(),
                request.now_unix_secs(),
                lock_stale_after_seconds,
            );
            refresh(&refresh_request)
        },
    )
}

/// Build one shared NNS source request after enforcing the family network contract.
pub(in crate::nns) fn fetch_nns_inventory_source_report<Report, Error>(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    enforce_network: impl FnOnce(&str) -> Result<(), Error>,
    fetch: impl FnOnce(&NnsSourceRequest) -> Result<Report, Error>,
) -> Result<Report, Error> {
    enforce_network(network)?;
    let request = NnsSourceRequest::new(
        network,
        source_endpoint,
        format_utc_timestamp_secs(now_unix_secs),
        "ic-query",
    );
    fetch(&request)
}

/// Fetch and atomically publish one network-validated Registry inventory cache.
pub(in crate::nns) fn refresh_nns_inventory_cache<Report, Error>(
    request: &NnsInventoryRefreshRequest,
    component_dir: &'static str,
    cache_file: &str,
    fetch: impl FnOnce(&str, &str, u64) -> Result<Report, Error>,
) -> Result<(Report, RefreshCacheWriteResult), Error>
where
    Report: NnsInventoryReport + Serialize,
    Error: From<HostCacheError> + NnsInventoryHostError,
{
    // Family fetches validate the network before invoking live or fixture sources.
    let report = fetch(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
    )?;
    let source_request = NnsSourceRequest::new(
        &request.cache.network,
        &request.source_endpoint,
        format_utc_timestamp_secs(request.now_unix_secs),
        "ic-query",
    );
    report
        .validate_source(&source_request)
        .map_err(Error::invalid_source_data)?;
    let write_result =
        write_nns_leaf_json_refresh_cache(request, component_dir, cache_file, &report)?;
    Ok((report, write_result))
}

///
/// NnsInventoryInputKind
///
/// Identifier normalization used by one Registry inventory family.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::nns) enum NnsInventoryInputKind {
    /// Canonical IC principal with principal-prefix fallback.
    Principal,
    /// Trimmed, lowercase textual identifier.
    Text,
}

///
/// NnsInventoryResolveError
///
/// Family-independent failure from exact-or-unique-prefix row resolution.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) enum NnsInventoryResolveError {
    /// No row matched the original input.
    NotFound {
        /// Original caller input retained for the family error.
        input: String,
    },
    /// More than one row matched the normalized prefix.
    Ambiguous {
        /// Normalized lowercase prefix.
        prefix: String,
        /// Canonically ordered identifiers that matched.
        matches: Vec<String>,
    },
}

///
/// NnsInventoryRow
///
/// Internal identifier access required by Registry inventory resolution.
///

pub(in crate::nns) trait NnsInventoryRow: Clone {
    /// Canonical identifier used for exact and prefix lookup.
    fn inventory_id(&self) -> &str;
}

/// Resolve a Registry inventory row by exact normalized identifier or unique prefix.
pub(in crate::nns) fn resolve_nns_inventory_row<Row: NnsInventoryRow>(
    rows: &[Row],
    input: &str,
    input_kind: NnsInventoryInputKind,
    exact_source: &'static str,
    prefix_source: &'static str,
) -> Result<(Row, String), NnsInventoryResolveError> {
    let exact = match input_kind {
        NnsInventoryInputKind::Principal => canonical_principal_text(input).ok(),
        NnsInventoryInputKind::Text => normalized_text(input),
    };
    if let Some(exact) = exact
        && let Some(row) = rows.iter().find(|row| row.inventory_id() == exact)
    {
        return Ok((row.clone(), exact_source.to_string()));
    }

    let prefix = input.trim().to_ascii_lowercase();
    if prefix.is_empty() {
        return Err(NnsInventoryResolveError::NotFound {
            input: input.to_string(),
        });
    }
    let matches = rows
        .iter()
        .filter(|row| row.inventory_id().starts_with(&prefix))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [row] => Ok(((*row).clone(), prefix_source.to_string())),
        [] => Err(NnsInventoryResolveError::NotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsInventoryResolveError::Ambiguous {
            prefix,
            matches: matches
                .into_iter()
                .map(|row| row.inventory_id().to_string())
                .collect(),
        }),
    }
}

fn normalized_text(input: &str) -> Option<String> {
    let normalized = input.trim().to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Row(&'static str);

    impl NnsInventoryRow for Row {
        fn inventory_id(&self) -> &str {
            self.0
        }
    }

    #[test]
    fn resolves_normalized_text_exactly_and_principals_by_prefix() {
        let text_rows = [Row("dc1")];
        assert_eq!(
            resolve_nns_inventory_row(
                &text_rows,
                " DC1 ",
                NnsInventoryInputKind::Text,
                "id",
                "prefix",
            ),
            Ok((Row("dc1"), "id".to_string()))
        );

        let principal_rows = [Row("ryjl3-tyaaa-aaaaa-aaaba-cai")];
        assert_eq!(
            resolve_nns_inventory_row(
                &principal_rows,
                "ryjl",
                NnsInventoryInputKind::Principal,
                "principal",
                "principal_prefix",
            ),
            Ok((
                Row("ryjl3-tyaaa-aaaaa-aaaba-cai"),
                "principal_prefix".to_string(),
            ))
        );
    }

    #[test]
    fn preserves_not_found_input_and_ambiguous_matches() {
        let rows = [Row("dc1"), Row("dc2")];
        assert_eq!(
            resolve_nns_inventory_row(&rows, " ", NnsInventoryInputKind::Text, "id", "prefix",),
            Err(NnsInventoryResolveError::NotFound {
                input: " ".to_string(),
            })
        );
        assert_eq!(
            resolve_nns_inventory_row(&rows, "DC", NnsInventoryInputKind::Text, "id", "prefix",),
            Err(NnsInventoryResolveError::Ambiguous {
                prefix: "dc".to_string(),
                matches: vec!["dc1".to_string(), "dc2".to_string()],
            })
        );
    }
}
