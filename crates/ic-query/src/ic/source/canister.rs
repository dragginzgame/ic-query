//! Module: ic::source::canister
//!
//! Responsibility: Dashboard canister source contracts, request normalization, and projection.
//! Does not own: HTTP transport, shared provenance, metrics, or network resources.
//! Boundary: validates canister source data before constructing canonical reports.

use super::{
    invalid_request, invalid_source, invalid_source_value, report_provenance, validate_provenance,
};
use crate::ic::{
    IcCanisterCountReport, IcCanisterCountSourceData, IcCanisterFilters, IcCanisterPageReport,
    IcCanisterPageRow, IcCanisterPageSourceData, IcCanisterReport, IcCanisterSourceData,
    IcCanisterUpgrade, IcHostError, IcSourceRequest, MAX_IC_CANISTER_PAGE_LIMIT,
};
use candid::Principal;
use std::collections::HashSet;

///
/// IcCanisterSource
///
/// Source contract for fetching one canister from an IC Dashboard-compatible API.
///

pub trait IcCanisterSource {
    /// Fetch one canister with explicit endpoint and collection provenance.
    fn fetch_canister(
        &self,
        request: &IcSourceRequest,
        canister_id: &str,
    ) -> Result<IcCanisterSourceData, IcHostError>;
}

///
/// IcCanisterCollectionSource
///
/// Source contract for bounded IC Dashboard canister discovery.
///

pub trait IcCanisterCollectionSource {
    /// Fetch one filtered canister count with explicit collection provenance.
    fn fetch_canister_count(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
    ) -> Result<IcCanisterCountSourceData, IcHostError>;

    /// Fetch at most `limit` rows without automatically following a cursor.
    fn fetch_canister_page(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
        limit: u16,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<IcCanisterPageSourceData, IcHostError>;
}

pub(in crate::ic) fn canonical_canister_id(value: &str) -> Result<String, IcHostError> {
    canonical_request_principal("canister_id", value)
}

pub(in crate::ic) fn normalized_filters(
    filters: &IcCanisterFilters,
) -> Result<IcCanisterFilters, IcHostError> {
    let mut filters = filters.clone();
    filters.subnet_id = filters
        .subnet_id
        .as_deref()
        .map(|value| canonical_request_principal("filters.subnet_id", value))
        .transpose()?;
    filters.controller_id = filters
        .controller_id
        .as_deref()
        .map(|value| canonical_request_principal("filters.controller_id", value))
        .transpose()?;
    normalize_string_filters("filters.languages", &mut filters.languages)?;
    normalize_string_filters("filters.canister_types", &mut filters.canister_types)?;

    if let Some(query) = filters.query.as_deref() {
        let length = query.chars().count();
        if !(2..=100).contains(&length) {
            return invalid_request("filters.query", "must contain between 2 and 100 characters");
        }
    }
    Ok(filters)
}

pub(in crate::ic) fn canonical_page_cursor(
    field: &'static str,
    cursor: Option<&str>,
) -> Result<Option<String>, IcHostError> {
    cursor
        .map(|value| canonical_request_principal(field, value))
        .transpose()
}

pub(in crate::ic) fn validate_page_limit(limit: u16) -> Result<(), IcHostError> {
    if (1..=MAX_IC_CANISTER_PAGE_LIMIT).contains(&limit) {
        return Ok(());
    }
    invalid_request(
        "limit",
        format!("must be between 1 and {MAX_IC_CANISTER_PAGE_LIMIT}"),
    )
}

pub(in crate::ic) fn report_from_source(
    request: &IcSourceRequest,
    requested_canister_id: &str,
    mut source: IcCanisterSourceData,
) -> Result<IcCanisterReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_principal_match("canister_id", requested_canister_id, &source.canister_id)?;
    validate_canonical_principal("subnet_id", &source.subnet_id)?;

    let mut seen_controllers = HashSet::with_capacity(source.controllers.len());
    for controller in &source.controllers {
        validate_canonical_principal("controller", controller)?;
        if !seen_controllers.insert(controller.clone()) {
            return invalid_source(format!("duplicate controller principal {controller}"));
        }
    }
    source.controllers.sort_unstable();

    validate_optional_module_hash("module_hash", &source.module_hash)?;
    if source.dashboard_updated_at.is_empty() {
        return invalid_source("dashboard_updated_at must not be empty");
    }

    if let Some(upgrades) = source.upgrades.as_mut() {
        validate_upgrades(upgrades)?;
        upgrades.sort_unstable_by(|left, right| {
            right
                .executed_timestamp_seconds
                .cmp(&left.executed_timestamp_seconds)
                .then_with(|| right.proposal_id.cmp(&left.proposal_id))
                .then_with(|| left.module_hash.cmp(&right.module_hash))
        });
    }

    Ok(IcCanisterReport {
        provenance: report_provenance(source.source),
        canister_id: source.canister_id,
        dashboard_id: source.dashboard_id,
        canister_type: source.canister_type,
        name: source.name,
        subnet_id: source.subnet_id,
        controllers: source.controllers,
        language: source.language,
        module_hash: source.module_hash,
        dashboard_updated_at: source.dashboard_updated_at,
        upgrade_count: source.upgrades.as_ref().map(Vec::len),
        upgrades: source.upgrades,
    })
}

pub(in crate::ic) fn count_report_from_source(
    request: &IcSourceRequest,
    filters: &IcCanisterFilters,
    source: IcCanisterCountSourceData,
) -> Result<IcCanisterCountReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_filter_match(filters, &source.filters)?;

    Ok(IcCanisterCountReport {
        provenance: report_provenance(source.source),
        filters: source.filters,
        total: source.total,
    })
}

pub(in crate::ic) fn page_report_from_source(
    request: &IcSourceRequest,
    filters: &IcCanisterFilters,
    limit: u16,
    after: Option<&str>,
    before: Option<&str>,
    mut source: IcCanisterPageSourceData,
) -> Result<IcCanisterPageReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_filter_match(filters, &source.filters)?;
    if source.requested_limit != limit {
        return invalid_source(format!(
            "requested_limit is {}, expected requested value {limit}",
            source.requested_limit
        ));
    }
    validate_optional_match("after", after, source.after.as_deref())?;
    validate_optional_match("before", before, source.before.as_deref())?;
    if source.rows.len() > usize::from(limit) {
        return invalid_source(format!(
            "source returned {} rows for requested limit {limit}",
            source.rows.len()
        ));
    }

    validate_page_rows(&mut source.rows)?;
    validate_source_cursor("previous_cursor", source.previous_cursor.as_deref())?;
    validate_source_cursor("next_cursor", source.next_cursor.as_deref())?;
    validate_page_boundary_cursor(
        "previous_cursor",
        source.previous_cursor.as_deref(),
        source.rows.first(),
    )?;
    validate_page_boundary_cursor(
        "next_cursor",
        source.next_cursor.as_deref(),
        source.rows.last(),
    )?;

    Ok(IcCanisterPageReport {
        provenance: report_provenance(source.source),
        filters: source.filters,
        requested_limit: source.requested_limit,
        returned_count: source.rows.len(),
        after: source.after,
        before: source.before,
        previous_cursor: source.previous_cursor,
        next_cursor: source.next_cursor,
        rows: source.rows,
    })
}

fn canonical_request_principal(field: &'static str, value: &str) -> Result<String, IcHostError> {
    Principal::from_text(value)
        .map(|principal| principal.to_text())
        .map_err(|error| IcHostError::InvalidPrincipal {
            field,
            reason: error.to_string(),
        })
}

fn normalize_string_filters(field: &'static str, values: &mut [String]) -> Result<(), IcHostError> {
    if values.iter().any(String::is_empty) {
        return invalid_request(field, "values must not be empty");
    }
    values.sort_unstable();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return invalid_request(field, "values must be unique");
    }
    Ok(())
}

fn validate_filter_match(
    expected: &IcCanisterFilters,
    actual: &IcCanisterFilters,
) -> Result<(), IcHostError> {
    if actual == expected {
        return Ok(());
    }
    invalid_source(format!(
        "filters are {actual:?}, expected requested filters {expected:?}"
    ))
}

fn validate_optional_match(
    field: &'static str,
    expected: Option<&str>,
    actual: Option<&str>,
) -> Result<(), IcHostError> {
    if actual == expected {
        return Ok(());
    }
    invalid_source(format!(
        "{field} is {actual:?}, expected requested value {expected:?}"
    ))
}

fn validate_page_rows(rows: &mut [IcCanisterPageRow]) -> Result<(), IcHostError> {
    let mut seen_canisters = HashSet::with_capacity(rows.len());
    let mut seen_dashboard_ids = HashSet::with_capacity(rows.len());
    let mut previous_canister_id: Option<&str> = None;

    for row in rows {
        validate_canonical_principal("row.canister_id", &row.canister_id)?;
        validate_canonical_principal("row.subnet_id", &row.subnet_id)?;
        if !seen_canisters.insert(row.canister_id.clone()) {
            return invalid_source(format!("duplicate canister_id {}", row.canister_id));
        }
        if !seen_dashboard_ids.insert(row.dashboard_id) {
            return invalid_source(format!("duplicate dashboard_id {}", row.dashboard_id));
        }
        if previous_canister_id.is_some_and(|previous| previous >= row.canister_id.as_str()) {
            return invalid_source("rows must be strictly ordered by canister_id");
        }
        previous_canister_id = Some(&row.canister_id);

        let mut seen_controllers = HashSet::with_capacity(row.controllers.len());
        for controller in &row.controllers {
            validate_canonical_principal("row.controller", &controller.principal_id)?;
            if !seen_controllers.insert(controller.principal_id.clone()) {
                return invalid_source(format!(
                    "duplicate controller principal {} for canister {}",
                    controller.principal_id, row.canister_id
                ));
            }
        }
        row.controllers.sort_unstable_by(|left, right| {
            left.principal_id
                .cmp(&right.principal_id)
                .then_with(|| left.raw_metadata.cmp(&right.raw_metadata))
        });
        validate_optional_module_hash("row.module_hash", &row.module_hash)?;
        if row.dashboard_updated_at.is_empty() {
            return invalid_source(format!(
                "dashboard_updated_at must not be empty for canister {}",
                row.canister_id
            ));
        }
    }
    Ok(())
}

fn validate_source_cursor(field: &'static str, cursor: Option<&str>) -> Result<(), IcHostError> {
    if let Some(cursor) = cursor {
        validate_canonical_principal(field, cursor)?;
    }
    Ok(())
}

fn validate_page_boundary_cursor(
    field: &'static str,
    cursor: Option<&str>,
    boundary: Option<&IcCanisterPageRow>,
) -> Result<(), IcHostError> {
    if let (Some(cursor), Some(boundary)) = (cursor, boundary)
        && cursor != boundary.canister_id
    {
        return invalid_source(format!(
            "{field} is {cursor:?}, expected page boundary {:?}",
            boundary.canister_id
        ));
    }
    Ok(())
}

fn validate_principal_match(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), IcHostError> {
    validate_canonical_principal(field, actual)?;
    if actual == expected {
        return Ok(());
    }
    invalid_source(format!(
        "{field} is {actual:?}, expected requested principal {expected:?}"
    ))
}

fn validate_canonical_principal(field: &'static str, value: &str) -> Result<(), IcHostError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_source_value(format!("{field} {value:?}: {error}")))?;
    let canonical = principal.to_text();
    if canonical != value {
        return invalid_source(format!(
            "{field} {value:?} is not canonical principal text; expected {canonical:?}"
        ));
    }
    Ok(())
}

fn validate_upgrades(upgrades: &[IcCanisterUpgrade]) -> Result<(), IcHostError> {
    let mut proposal_ids = HashSet::with_capacity(upgrades.len());
    for upgrade in upgrades {
        validate_module_hash("upgrade.module_hash", &upgrade.module_hash)?;
        if !proposal_ids.insert(upgrade.proposal_id) {
            return invalid_source(format!(
                "duplicate upgrade proposal_id {}",
                upgrade.proposal_id
            ));
        }
    }
    Ok(())
}

fn validate_optional_module_hash(field: &'static str, value: &str) -> Result<(), IcHostError> {
    if value.is_empty() {
        return Ok(());
    }
    validate_module_hash(field, value)
}

fn validate_module_hash(field: &'static str, value: &str) -> Result<(), IcHostError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    invalid_source(format!(
        "{field} must be 64-character lowercase hexadecimal text"
    ))
}
