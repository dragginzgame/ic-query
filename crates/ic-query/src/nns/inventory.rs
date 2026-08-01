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
    cache_file::{RefreshCacheWriteResult, load_or_refresh_missing_cache},
    nns::leaf::write_nns_leaf_json_refresh_cache,
    subnet_catalog::{canonical_principal_text, format_utc_timestamp_secs},
};
use serde::Serialize;
use std::path::PathBuf;

///
/// NnsInventoryHostError
///
/// Internal conversion from a family host error to the shared missing-cache policy.
///

pub(in crate::nns) trait NnsInventoryHostError: Sized {
    /// Return the missing path that permits read-through refresh, or preserve any other error.
    fn missing_cache_path(self) -> Result<PathBuf, Self>;
}

///
/// NnsInventoryListInput
///
/// Internal view of the collection inputs shared by Registry inventory list requests.
///

pub(in crate::nns) trait NnsInventoryListInput {
    /// Cache identity for the complete inventory snapshot.
    fn cache(&self) -> &NnsInventoryCacheRequest;
    /// Explicit live endpoint used only when the cache is missing.
    fn source_endpoint(&self) -> &str;
    /// Caller-provided timestamp used by a missing-cache refresh.
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

/// Load one Registry inventory cache, refreshing only when it is missing.
pub(in crate::nns) fn load_or_refresh_nns_inventory_report<Report, Error>(
    request: &impl NnsInventoryListInput,
    lock_stale_after_seconds: u64,
    mut load: impl FnMut(&NnsInventoryCacheRequest) -> Result<Report, Error>,
    refresh: impl FnOnce(&NnsInventoryRefreshRequest) -> Result<(), Error>,
) -> Result<Report, Error>
where
    Error: NnsInventoryHostError,
{
    load_or_refresh_missing_cache(
        || load(request.cache()),
        NnsInventoryHostError::missing_cache_path,
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
    Report: Serialize,
    Error: From<HostCacheError>,
{
    // Family fetches validate the network before invoking live or fixture sources.
    let report = fetch(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
    )?;
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
