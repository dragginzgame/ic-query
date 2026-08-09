//! Module: ic::source::icrc_index
//!
//! Responsibility: official ICRC account and holder index source contracts and projection.
//! Does not own: native ledger queries, HTTP transport, command parsing, or rendering.
//! Boundary: validates one exact account or one response-bounded cursor page.

use super::{
    canonical_request_principal, invalid_request, invalid_source, report_provenance,
    validate_principal_match, validate_provenance,
};
use crate::ic::{
    IcHostError, IcIcrcAccountInfoReport, IcIcrcAccountInfoSourceData, IcIcrcAccountListQuery,
    IcIcrcAccountListReport, IcIcrcAccountListSourceData, IcIcrcAccountRow, IcIcrcHolderListQuery,
    IcIcrcHolderListReport, IcIcrcHolderListSourceData, IcIcrcHolderRow, IcIcrcHolderSourceRow,
    IcSourceRequest, MAX_ICRC_INDEX_CURSOR_CHARS, MAX_ICRC_INDEX_PAGE_ROWS,
};
use std::collections::BTreeSet;

///
/// IcIcrcIndexSource
///
/// Source contract for bounded official Dashboard ICRC account and holder index queries.
///

pub trait IcIcrcIndexSource {
    /// Fetch one bounded account page without automatic cursor follow-up.
    fn fetch_account_list(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcAccountListQuery,
    ) -> Result<IcIcrcAccountListSourceData, IcHostError>;

    /// Fetch one exact account record.
    fn fetch_account_info(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        account_id: &str,
    ) -> Result<IcIcrcAccountInfoSourceData, IcHostError>;

    /// Fetch one bounded holder page without automatic cursor follow-up.
    fn fetch_holder_list(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcHolderListQuery,
    ) -> Result<IcIcrcHolderListSourceData, IcHostError>;
}

pub(in crate::ic) fn normalized_account_list_query(
    query: &IcIcrcAccountListQuery,
) -> Result<IcIcrcAccountListQuery, IcHostError> {
    validate_page_query(query.after.as_deref(), query.before.as_deref(), query.limit)?;
    let mut normalized = query.clone();
    normalized.owner = query
        .owner
        .as_deref()
        .map(|owner| canonical_request_principal("query.owner", owner))
        .transpose()?;
    Ok(normalized)
}

pub(in crate::ic) fn validate_holder_list_query(
    query: &IcIcrcHolderListQuery,
) -> Result<(), IcHostError> {
    validate_page_query(query.after.as_deref(), query.before.as_deref(), query.limit)
}

pub(in crate::ic) fn validate_account_id(account_id: &str) -> Result<(), IcHostError> {
    validate_opaque_value("account_id", account_id)
}

pub(in crate::ic) fn icrc_account_list_report_from_source(
    request: &IcSourceRequest,
    ledger_canister_id: &str,
    query: &IcIcrcAccountListQuery,
    source: IcIcrcAccountListSourceData,
) -> Result<IcIcrcAccountListReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_principal_match(
        "ledger_canister_id",
        ledger_canister_id,
        &source.ledger_canister_id,
    )?;
    if source.query != *query {
        return invalid_source(format!(
            "ICRC account-list query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }
    validate_returned_page(
        source.rows.len(),
        query.limit,
        source.previous_cursor.as_deref(),
        source.next_cursor.as_deref(),
    )?;

    let mut ids = BTreeSet::new();
    let mut rows = Vec::with_capacity(source.rows.len());
    for (index, row) in source.rows.into_iter().enumerate() {
        let projected = project_account_row(index, ledger_canister_id, row)?;
        if let Some(owner) = &query.owner
            && projected.owner != *owner
        {
            return invalid_source(format!(
                "account row {index} owner {:?} does not match requested owner {owner:?}",
                projected.owner
            ));
        }
        if !ids.insert(projected.account_id.clone()) {
            return invalid_source(format!(
                "account page contains duplicate account id {:?}",
                projected.account_id
            ));
        }
        rows.push(projected);
    }

    Ok(IcIcrcAccountListReport {
        provenance: report_provenance(source.source),
        ledger_canister_id: source.ledger_canister_id,
        query: source.query,
        returned_count: rows.len(),
        previous_cursor: source.previous_cursor,
        next_cursor: source.next_cursor,
        rows,
    })
}

pub(in crate::ic) fn icrc_account_info_report_from_source(
    request: &IcSourceRequest,
    ledger_canister_id: &str,
    account_id: &str,
    source: IcIcrcAccountInfoSourceData,
) -> Result<IcIcrcAccountInfoReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    let account = project_account_row(0, ledger_canister_id, source.account)?;
    if account.account_id != account_id {
        return invalid_source(format!(
            "account id is {:?}, expected requested account id {account_id:?}",
            account.account_id
        ));
    }
    Ok(IcIcrcAccountInfoReport {
        provenance: report_provenance(source.source),
        account,
    })
}

pub(in crate::ic) fn icrc_holder_list_report_from_source(
    request: &IcSourceRequest,
    ledger_canister_id: &str,
    query: &IcIcrcHolderListQuery,
    source: IcIcrcHolderListSourceData,
) -> Result<IcIcrcHolderListReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_principal_match(
        "ledger_canister_id",
        ledger_canister_id,
        &source.ledger_canister_id,
    )?;
    if source.query != *query {
        return invalid_source(format!(
            "ICRC holder-list query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }
    validate_returned_page(
        source.rows.len(),
        query.limit,
        source.previous_cursor.as_deref(),
        source.next_cursor.as_deref(),
    )?;

    let mut principals = BTreeSet::new();
    let mut rows = Vec::with_capacity(source.rows.len());
    for (index, row) in source.rows.into_iter().enumerate() {
        let projected = project_holder_row(index, ledger_canister_id, row)?;
        if !principals.insert(projected.principal.clone()) {
            return invalid_source(format!(
                "holder page contains duplicate principal {:?}",
                projected.principal
            ));
        }
        rows.push(projected);
    }

    Ok(IcIcrcHolderListReport {
        provenance: report_provenance(source.source),
        ledger_canister_id: source.ledger_canister_id,
        query: source.query,
        returned_count: rows.len(),
        previous_cursor: source.previous_cursor,
        next_cursor: source.next_cursor,
        rows,
    })
}

fn validate_page_query(
    after: Option<&str>,
    before: Option<&str>,
    limit: u16,
) -> Result<(), IcHostError> {
    if !(1..=MAX_ICRC_INDEX_PAGE_ROWS).contains(&limit) {
        return invalid_request(
            "query.limit",
            format!("must be between 1 and {MAX_ICRC_INDEX_PAGE_ROWS}"),
        );
    }
    if after.is_some() && before.is_some() {
        return invalid_request("query", "after and before are mutually exclusive");
    }
    if let Some(after) = after {
        validate_request_cursor("query.after", after)?;
    }
    if let Some(before) = before {
        validate_request_cursor("query.before", before)?;
    }
    Ok(())
}

fn validate_returned_page(
    returned_count: usize,
    limit: u16,
    previous_cursor: Option<&str>,
    next_cursor: Option<&str>,
) -> Result<(), IcHostError> {
    if returned_count > usize::from(limit) {
        return invalid_source(format!(
            "index page returned {returned_count} rows for a request limited to {limit}"
        ));
    }
    for (field, cursor) in [
        ("previous_cursor", previous_cursor),
        ("next_cursor", next_cursor),
    ] {
        if let Some(cursor) = cursor {
            validate_source_cursor(field, cursor)?;
        }
    }
    Ok(())
}

fn project_account_row(
    index: usize,
    ledger_canister_id: &str,
    row: crate::ic::IcIcrcAccountSourceRow,
) -> Result<IcIcrcAccountRow, IcHostError> {
    validate_source_opaque_value(&format!("account row {index} account_id"), &row.account_id)?;
    let owner = canonical_source_principal(&format!("account row {index} owner"), &row.owner)?;
    validate_principal_match(
        "account row ledger_canister_id",
        ledger_canister_id,
        &row.ledger_canister_id,
    )?;
    validate_unsigned_decimal(
        &format!("account row {index} balance"),
        &row.balance_base_units,
    )?;
    if row.dashboard_updated_at.trim().is_empty() {
        return invalid_source(format!(
            "account row {index} dashboard_updated_at must not be empty"
        ));
    }
    let (created_at_unix_secs, created_at_subsec_nanos) =
        split_unix_nanos(row.created_at_unix_nanos);
    Ok(IcIcrcAccountRow {
        account_id: row.account_id,
        owner,
        subaccount: row.subaccount,
        balance_base_units: row.balance_base_units,
        total_transactions: row.total_transactions,
        created_at_unix_secs,
        created_at_subsec_nanos,
        ledger_canister_id: row.ledger_canister_id,
        latest_transaction_index: row.latest_transaction_index,
        dashboard_updated_at: row.dashboard_updated_at,
        active_fee_collector: row.active_fee_collector,
        fee_collector_block_ranges: row.fee_collector_block_ranges,
    })
}

fn project_holder_row(
    index: usize,
    ledger_canister_id: &str,
    row: IcIcrcHolderSourceRow,
) -> Result<IcIcrcHolderRow, IcHostError> {
    let principal =
        canonical_source_principal(&format!("holder row {index} principal"), &row.principal)?;
    validate_principal_match(
        "holder row ledger_canister_id",
        ledger_canister_id,
        &row.ledger_canister_id,
    )?;
    validate_unsigned_decimal(
        &format!("holder row {index} balance"),
        &row.balance_base_units,
    )?;
    if !row.percentage.is_number() {
        return invalid_source(format!(
            "holder row {index} percentage must be a JSON number"
        ));
    }
    if !matches!(
        &row.value_usd,
        serde_json::Value::Null | serde_json::Value::Number(_) | serde_json::Value::String(_)
    ) {
        return invalid_source(format!(
            "holder row {index} value_usd must be a JSON number, string, or null"
        ));
    }
    if row.dashboard_updated_at.trim().is_empty() {
        return invalid_source(format!(
            "holder row {index} dashboard_updated_at must not be empty"
        ));
    }
    let (created_at_unix_secs, created_at_subsec_nanos) =
        split_unix_nanos(row.created_at_unix_nanos);
    Ok(IcIcrcHolderRow {
        principal,
        balance_base_units: row.balance_base_units,
        total_transactions: row.total_transactions,
        created_at_unix_secs,
        created_at_subsec_nanos,
        ledger_canister_id: row.ledger_canister_id,
        latest_transaction_index: row.latest_transaction_index,
        percentage: row.percentage,
        value_usd: row.value_usd,
        dashboard_updated_at: row.dashboard_updated_at,
    })
}

fn split_unix_nanos(value: u64) -> (u64, u32) {
    const NANOS_PER_SECOND: u64 = 1_000_000_000;
    (
        value / NANOS_PER_SECOND,
        u32::try_from(value % NANOS_PER_SECOND).expect("nanosecond remainder fits u32"),
    )
}

fn validate_opaque_value(field: &'static str, value: &str) -> Result<(), IcHostError> {
    if value.is_empty() || value.trim() != value {
        return invalid_request(
            field,
            "must be non-empty and have no surrounding whitespace",
        );
    }
    if value.chars().count() > MAX_ICRC_INDEX_CURSOR_CHARS {
        return invalid_request(
            field,
            format!("must not exceed {MAX_ICRC_INDEX_CURSOR_CHARS} characters"),
        );
    }
    Ok(())
}

fn validate_source_opaque_value(field: &str, value: &str) -> Result<(), IcHostError> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().count() > MAX_ICRC_INDEX_CURSOR_CHARS
    {
        return invalid_source(format!(
            "{field} must contain 1 to {MAX_ICRC_INDEX_CURSOR_CHARS} characters without surrounding whitespace"
        ));
    }
    Ok(())
}

fn validate_request_cursor(field: &'static str, value: &str) -> Result<(), IcHostError> {
    validate_opaque_value(field, value)?;
    if !valid_cursor_components(value) {
        return invalid_request(
            field,
            "must contain one or two non-empty comma-separated components",
        );
    }
    Ok(())
}

fn validate_source_cursor(field: &str, value: &str) -> Result<(), IcHostError> {
    validate_source_opaque_value(field, value)?;
    if !valid_cursor_components(value) {
        return invalid_source(format!(
            "{field} must contain one or two non-empty comma-separated components"
        ));
    }
    Ok(())
}

fn valid_cursor_components(value: &str) -> bool {
    let mut components = value.split(',');
    let first = components
        .next()
        .is_some_and(|component| !component.is_empty());
    let second = components.next();
    first && second.is_none_or(|component| !component.is_empty()) && components.next().is_none()
}

fn canonical_source_principal(field: &str, value: &str) -> Result<String, IcHostError> {
    candid::Principal::from_text(value)
        .map(|principal| principal.to_text())
        .map_err(|error| IcHostError::InvalidSourceData {
            reason: format!("{field} is not a principal: {error}"),
        })
}

fn validate_unsigned_decimal(field: &str, value: &str) -> Result<(), IcHostError> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value != "0" && value.starts_with('0'))
    {
        return invalid_source(format!(
            "{field} value {value:?} is not canonical unsigned decimal text"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ic::{IcIcrcAccountSort, IcIcrcHolderSort};

    #[test]
    fn page_queries_reject_conflicting_cursors_and_invalid_limits() {
        let account = IcIcrcAccountListQuery::new(20, IcIcrcAccountSort::Id)
            .with_after("next")
            .with_before("previous");
        assert!(normalized_account_list_query(&account).is_err());

        let holder = IcIcrcHolderListQuery::new(0, IcIcrcHolderSort::Principal);
        assert!(validate_holder_list_query(&holder).is_err());
    }

    #[test]
    fn nanosecond_timestamps_normalize_without_losing_subsecond_precision() {
        assert_eq!(
            split_unix_nanos(1_731_833_720_056_622_711),
            (1_731_833_720, 56_622_711)
        );
    }

    #[test]
    fn holder_projection_rejects_structured_or_boolean_value_evidence() {
        let row = IcIcrcHolderSourceRow {
            principal: "aaaaa-aa".to_string(),
            balance_base_units: "1".to_string(),
            total_transactions: 1,
            created_at_unix_nanos: 1_000_000_000,
            ledger_canister_id: "mxzaz-hqaaa-aaaar-qaada-cai".to_string(),
            latest_transaction_index: 1,
            percentage: serde_json::json!(0.1),
            value_usd: serde_json::json!(true),
            dashboard_updated_at: "2026-08-09T10:00:00Z".to_string(),
        };

        assert!(project_holder_row(0, "mxzaz-hqaaa-aaaar-qaada-cai", row).is_err());
    }
}
