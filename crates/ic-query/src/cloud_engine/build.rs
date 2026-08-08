//! Module: cloud_engine::build
//!
//! Responsibility: validate source contracts and assemble CloudEngine reports.
//! Does not own: live transport, CLI parsing, caching, or rendering.
//! Boundary: rejects non-mainnet and invalid targets before invoking any source capability.

use super::{
    CLOUD_ENGINE_AUTHORITY, CLOUD_ENGINE_REPORT_SCHEMA_VERSION, CloudEngineHostError,
    CloudEngineOperatorReport, CloudEngineOperatorSourceData, CloudEnginePriceRow,
    CloudEnginePricesReport, CloudEnginePricesSourceData, CloudEngineReportContext,
    CloudEngineSource, CloudEngineSourceRequest, LiveCloudEngineSource,
    MAINNET_CLOUD_ENGINE_CANISTER_ID, MAX_CLOUD_ENGINE_CYCLE_DECIMAL_DIGITS,
    MAX_CLOUD_ENGINE_DOMAINS, MAX_CLOUD_ENGINE_PRICE_ROWS, enforce_mainnet_network,
};
use candid::Principal;
use std::cmp::Ordering;

const MAX_DATA_CENTER_ID_BYTES: usize = 128;
const MAX_DOMAIN_BYTES: usize = 253;

/// Build one live CloudEngine operator report for a Subnet principal.
pub fn build_cloud_engine_operator_report(
    request: &CloudEngineSourceRequest,
    subnet_id: &str,
) -> Result<CloudEngineOperatorReport, CloudEngineHostError> {
    build_cloud_engine_operator_report_with_source(request, subnet_id, &LiveCloudEngineSource)
}

/// Build one CloudEngine operator report from a custom source.
pub fn build_cloud_engine_operator_report_with_source(
    request: &CloudEngineSourceRequest,
    subnet_id: &str,
    source: &dyn CloudEngineSource,
) -> Result<CloudEngineOperatorReport, CloudEngineHostError> {
    enforce_mainnet_network(&request.network)?;
    let subnet_id = canonical_principal("subnet_id", subnet_id)?;
    let mut source = source.fetch_operator(request, &subnet_id)?;
    validate_source_request(request, &source.source)?;
    validate_principal_match("subnet_id", &subnet_id, &source.subnet_id)?;
    validate_operator_source(&mut source)?;

    Ok(CloudEngineOperatorReport {
        context: report_context(request, source.query_call_count),
        subnet_id,
        operator_binding_present: source.operator_canister_id.is_some(),
        operator_canister_id: source.operator_canister_id,
        engine_owner: source.engine_owner,
        platform_admin: source.platform_admin,
        caffeine_enabled: source.caffeine_enabled,
        claimed_domain_count: source.claimed_domains.as_ref().map(Vec::len),
        claimed_domains: source.claimed_domains,
    })
}

/// Build one live bounded CloudEngine marketplace report.
pub fn build_cloud_engine_prices_report(
    request: &CloudEngineSourceRequest,
) -> Result<CloudEnginePricesReport, CloudEngineHostError> {
    build_cloud_engine_prices_report_with_source(request, &LiveCloudEngineSource)
}

/// Build one bounded CloudEngine marketplace report from a custom source.
pub fn build_cloud_engine_prices_report_with_source(
    request: &CloudEngineSourceRequest,
    source: &dyn CloudEngineSource,
) -> Result<CloudEnginePricesReport, CloudEngineHostError> {
    enforce_mainnet_network(&request.network)?;
    let mut source = source.fetch_prices(request)?;
    validate_source_request(request, &source.source)?;
    validate_prices_source(&mut source)?;

    Ok(CloudEnginePricesReport {
        context: report_context(request, source.query_call_count),
        network_fee: source.network_fee,
        price_count: source.prices.len(),
        prices: source.prices,
    })
}

fn report_context(
    request: &CloudEngineSourceRequest,
    query_call_count: usize,
) -> CloudEngineReportContext {
    CloudEngineReportContext {
        schema_version: CLOUD_ENGINE_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        authority: CLOUD_ENGINE_AUTHORITY.to_string(),
        engine_canister_id: MAINNET_CLOUD_ENGINE_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        source_endpoint: request.endpoint.clone(),
        fetched_by: request.fetched_by.clone(),
        certified: false,
        point_in_time_guaranteed: false,
        query_call_count,
    }
}

fn validate_source_request(
    expected: &CloudEngineSourceRequest,
    actual: &CloudEngineSourceRequest,
) -> Result<(), CloudEngineHostError> {
    for (field, expected, actual) in [
        (
            "network",
            expected.network.as_str(),
            actual.network.as_str(),
        ),
        (
            "source_endpoint",
            expected.endpoint.as_str(),
            actual.endpoint.as_str(),
        ),
        (
            "fetched_at",
            expected.fetched_at.as_str(),
            actual.fetched_at.as_str(),
        ),
        (
            "fetched_by",
            expected.fetched_by.as_str(),
            actual.fetched_by.as_str(),
        ),
    ] {
        if actual != expected {
            return invalid_source(format!(
                "{field} is {actual:?}, expected requested value {expected:?}"
            ));
        }
    }
    Ok(())
}

fn validate_operator_source(
    source: &mut CloudEngineOperatorSourceData,
) -> Result<(), CloudEngineHostError> {
    match source.operator_canister_id.as_deref() {
        None => {
            if source.query_call_count != 1 {
                return invalid_source(format!(
                    "a missing operator binding requires one query call, got {}",
                    source.query_call_count
                ));
            }
            if source.engine_owner.is_some()
                || source.platform_admin.is_some()
                || source.caffeine_enabled.is_some()
                || source.claimed_domains.is_some()
            {
                return invalid_source(
                    "a missing operator binding cannot carry operator detail fields",
                );
            }
        }
        Some(operator) => {
            validate_canonical_principal("operator_canister_id", operator)?;
            if source.query_call_count != 5 {
                return invalid_source(format!(
                    "a present operator binding requires five query calls, got {}",
                    source.query_call_count
                ));
            }
            if let Some(owner) = source.engine_owner.as_deref() {
                validate_canonical_principal("engine_owner", owner)?;
            }
            if let Some(admin) = source.platform_admin.as_deref() {
                validate_canonical_principal("platform_admin", admin)?;
            }
            if let Some(domains) = source.claimed_domains.as_mut() {
                validate_domains(domains)?;
            }
        }
    }
    Ok(())
}

fn validate_domains(domains: &mut [String]) -> Result<(), CloudEngineHostError> {
    if domains.len() > MAX_CLOUD_ENGINE_DOMAINS {
        return invalid_source(format!(
            "operator returned {} domains, maximum is {MAX_CLOUD_ENGINE_DOMAINS}",
            domains.len()
        ));
    }
    for domain in domains.iter() {
        if domain.is_empty()
            || domain.len() > MAX_DOMAIN_BYTES
            || domain.trim() != domain
            || domain
                .chars()
                .any(|character| character.is_control() || character.is_whitespace())
        {
            return invalid_source(format!("invalid claimed domain {domain:?}"));
        }
    }
    domains.sort_unstable();
    if domains.windows(2).any(|pair| pair[0] == pair[1]) {
        return invalid_source("claimed domains must be unique");
    }
    Ok(())
}

fn validate_prices_source(
    source: &mut CloudEnginePricesSourceData,
) -> Result<(), CloudEngineHostError> {
    if source.query_call_count != 2 {
        return invalid_source(format!(
            "marketplace collection requires exactly two query calls, got {}",
            source.query_call_count
        ));
    }
    if !source.network_fee.is_finite() || source.network_fee.is_sign_negative() {
        return invalid_source("network_fee must be a finite non-negative float");
    }
    if source.prices.len() > MAX_CLOUD_ENGINE_PRICE_ROWS {
        return invalid_source(format!(
            "marketplace returned {} rows, maximum is {MAX_CLOUD_ENGINE_PRICE_ROWS}",
            source.prices.len()
        ));
    }
    for row in &source.prices {
        validate_price_row(row)?;
    }
    source
        .prices
        .sort_unstable_by(|left, right| left.key.cmp(&right.key));
    if source
        .prices
        .windows(2)
        .any(|pair| pair[0].key == pair[1].key)
    {
        return invalid_source("marketplace keys must be unique");
    }
    Ok(())
}

fn validate_price_row(row: &CloudEnginePriceRow) -> Result<(), CloudEngineHostError> {
    if let Some(provider) = row.provider_id.as_deref() {
        validate_canonical_principal("provider_id", provider)?;
    }
    if let Some(data_center) = row.data_center_id.as_deref()
        && (data_center.is_empty()
            || data_center.len() > MAX_DATA_CENTER_ID_BYTES
            || data_center.trim() != data_center
            || data_center.contains(',')
            || data_center
                .chars()
                .any(|character| character.is_control() || character.is_whitespace()))
    {
        return invalid_source(format!("invalid data_center_id {data_center:?}"));
    }

    let expected_key = marketplace_key(row);
    if row.key != expected_key {
        return invalid_source(format!(
            "marketplace key {:?} does not match row identity {:?}",
            row.key, expected_key
        ));
    }
    validate_decimal_cycles("net_cycles_per_month", &row.net_cycles_per_month)?;
    validate_decimal_cycles("gross_cycles_per_month", &row.gross_cycles_per_month)?;
    if decimal_cmp(&row.gross_cycles_per_month, &row.net_cycles_per_month) == Ordering::Less {
        return invalid_source(format!(
            "gross cycles {} are less than net cycles {} for {}",
            row.gross_cycles_per_month, row.net_cycles_per_month, row.key
        ));
    }
    if row.updated_at_unix_nanos <= 0 {
        return invalid_source(format!(
            "updated_at_unix_nanos must be positive for {}",
            row.key
        ));
    }
    Ok(())
}

fn marketplace_key(row: &CloudEnginePriceRow) -> String {
    let mut parts = Vec::with_capacity(3);
    if let Some(provider) = row.provider_id.as_deref() {
        parts.push(provider);
    }
    parts.push(row.node_type.as_str());
    if let Some(data_center) = row.data_center_id.as_deref() {
        parts.push(data_center);
    }
    parts.join(",")
}

fn validate_decimal_cycles(field: &'static str, value: &str) -> Result<(), CloudEngineHostError> {
    if value.is_empty()
        || value.len() > MAX_CLOUD_ENGINE_CYCLE_DECIMAL_DIGITS
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return invalid_source(format!(
            "{field} must be canonical non-negative decimal text of at most {MAX_CLOUD_ENGINE_CYCLE_DECIMAL_DIGITS} digits"
        ));
    }
    Ok(())
}

fn decimal_cmp(left: &str, right: &str) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn canonical_principal(field: &'static str, value: &str) -> Result<String, CloudEngineHostError> {
    Principal::from_text(value)
        .map(|principal| principal.to_text())
        .map_err(|error| CloudEngineHostError::InvalidPrincipal {
            field,
            reason: error.to_string(),
        })
}

fn validate_canonical_principal(
    field: &'static str,
    value: &str,
) -> Result<(), CloudEngineHostError> {
    let canonical = canonical_principal(field, value).map_err(|error| match error {
        CloudEngineHostError::InvalidPrincipal { reason, .. } => {
            CloudEngineHostError::InvalidSourceData {
                reason: format!("invalid {field} {value:?}: {reason}"),
            }
        }
        other => other,
    })?;
    if canonical != value {
        return invalid_source(format!(
            "{field} {value:?} is not canonical principal text; expected {canonical:?}"
        ));
    }
    Ok(())
}

fn validate_principal_match(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), CloudEngineHostError> {
    validate_canonical_principal(field, actual)?;
    if actual != expected {
        return invalid_source(format!(
            "{field} is {actual:?}, expected requested principal {expected:?}"
        ));
    }
    Ok(())
}

fn invalid_source<T>(reason: impl Into<String>) -> Result<T, CloudEngineHostError> {
    Err(CloudEngineHostError::InvalidSourceData {
        reason: reason.into(),
    })
}
