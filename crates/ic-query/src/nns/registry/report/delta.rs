//! Module: nns::registry::report::delta
//!
//! Responsibility: validate and expose one certified Registry delta batch.
//! Does not own: historical replay, caching, CLI dispatch, or catalog assurance.
//! Boundary: custom and live sources must satisfy the same bounded report contract.

use super::{
    build::validate_certification,
    error::NnsRegistryHostError,
    model::{
        NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION, NnsCertifiedRegistryDeltaBatchReport,
        NnsCertifiedRegistryDeltaBatchRequest, NnsCertifiedRegistryDeltaLimits,
        NnsCertifiedRegistryDeltaVersion, NnsCertifiedRegistryMutationKind,
    },
    source::{NnsCertifiedRegistryDeltaSource, nns_certified_registry_delta_limits},
};
use crate::{
    hex::is_canonical_lowercase_hex,
    nns::LiveNnsSource,
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs},
};
use std::collections::BTreeSet;

impl_nns_mainnet_network_enforcer!(NnsRegistryHostError);

/// Fetch and validate one live certified Registry delta batch.
pub async fn fetch_nns_certified_registry_delta_batch_async(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
) -> Result<NnsCertifiedRegistryDeltaBatchReport, NnsRegistryHostError> {
    fetch_nns_certified_registry_delta_batch_with_source_async(request, &LiveNnsSource).await
}

/// Fetch and validate one certified Registry delta batch from an explicit source.
pub async fn fetch_nns_certified_registry_delta_batch_with_source_async(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
) -> Result<NnsCertifiedRegistryDeltaBatchReport, NnsRegistryHostError> {
    enforce_mainnet_network(&request.network)?;
    let report = source.fetch_certified_registry_delta_batch(request).await?;
    validate_nns_certified_registry_delta_batch(request, &report)?;
    Ok(report)
}

/// Validate a source-produced delta report without performing network or filesystem IO.
pub fn validate_nns_certified_registry_delta_batch(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    report: &NnsCertifiedRegistryDeltaBatchReport,
) -> Result<(), NnsRegistryHostError> {
    enforce_mainnet_network(&request.network)?;
    validate_equal(
        "schema_version",
        NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION,
        report.schema_version,
    )?;
    validate_text("network", MAINNET_NETWORK, &report.network)?;
    validate_text(
        "registry_canister_id",
        MAINNET_REGISTRY_CANISTER_ID,
        &report.registry_canister_id,
    )?;
    validate_equal(
        "requested_version",
        request.requested_version,
        report.requested_version,
    )?;
    validate_text(
        "fetched_at",
        &format_utc_timestamp_secs(request.now_unix_secs),
        &report.fetched_at,
    )?;
    validate_text(
        "source_endpoint",
        &request.source_endpoint,
        &report.source_endpoint,
    )?;
    validate_text("fetched_by", "ic-query", &report.fetched_by)?;
    if report.limits != nns_certified_registry_delta_limits() {
        return Err(invalid_source_data(
            "limits do not match the fixed certified delta validator ceilings",
        ));
    }
    if report.query_call_count != 1 {
        return Err(invalid_source_data(format!(
            "query_call_count must be 1, got {}",
            report.query_call_count
        )));
    }
    if report.response_bytes == 0 || report.response_bytes > report.limits.max_response_bytes {
        return Err(invalid_source_data(format!(
            "response_bytes must be within 1..={}, got {}",
            report.limits.max_response_bytes, report.response_bytes
        )));
    }
    let minimum_evidence_bytes = report
        .certification
        .certificate_bytes
        .checked_add(report.certification.hash_tree_bytes)
        .ok_or_else(|| invalid_source_data("certified evidence byte counts overflow usize"))?;
    if report.response_bytes < minimum_evidence_bytes {
        return Err(invalid_source_data(
            "response_bytes is smaller than its certificate and hash-tree evidence",
        ));
    }
    validate_certification(request.now_unix_secs, &report.certification)?;
    validate_delta_contents(report)
}

fn validate_delta_contents(
    report: &NnsCertifiedRegistryDeltaBatchReport,
) -> Result<(), NnsRegistryHostError> {
    if report.requested_version > report.certified_latest_version {
        return Err(invalid_source_data(
            "requested_version exceeds certified_latest_version",
        ));
    }
    if report.versions.len() > report.limits.max_versions {
        return Err(invalid_source_data(format!(
            "version count exceeds the maximum of {}",
            report.limits.max_versions
        )));
    }
    validate_equal("version_count", report.versions.len(), report.version_count)?;
    validate_equal(
        "first_version",
        report.versions.first().map(|row| row.version),
        report.first_version,
    )?;
    validate_equal(
        "last_version",
        report.versions.last().map(|row| row.version),
        report.last_version,
    )?;

    if report.versions.is_empty() && report.requested_version != report.certified_latest_version {
        return Err(invalid_source_data(
            "versions must not be empty before certified_latest_version",
        ));
    }

    let totals = validate_version_sequence(report)?;
    validate_equal("mutation_count", totals.mutations, report.mutation_count)?;
    validate_equal(
        "precondition_count",
        totals.preconditions,
        report.precondition_count,
    )?;
    validate_equal(
        "inline_value_bytes",
        totals.inline_value_bytes,
        report.inline_value_bytes,
    )?;
    let last_version = report.last_version.unwrap_or(report.requested_version);
    validate_equal(
        "more_available",
        last_version < report.certified_latest_version,
        report.more_available,
    )?;
    Ok(())
}

#[derive(Default)]
struct DeltaTotals {
    mutations: usize,
    preconditions: usize,
    inline_value_bytes: usize,
}

fn validate_version_sequence(
    report: &NnsCertifiedRegistryDeltaBatchReport,
) -> Result<DeltaTotals, NnsRegistryHostError> {
    if report.versions.is_empty() {
        return Ok(DeltaTotals::default());
    }
    let mut expected_version = report.requested_version.checked_add(1).ok_or_else(|| {
        invalid_source_data("requested_version cannot advance without overflowing u64")
    })?;
    let mut totals = DeltaTotals::default();
    for version in &report.versions {
        if version.version != expected_version {
            return Err(invalid_source_data(format!(
                "version sequence expected {expected_version}, got {}",
                version.version
            )));
        }
        if version.version > report.certified_latest_version {
            return Err(invalid_source_data(format!(
                "version {} exceeds certified_latest_version {}",
                version.version, report.certified_latest_version
            )));
        }
        let version_totals = validate_version_contents(version, &report.limits)?;
        totals.mutations = checked_total(
            "mutation_count",
            totals.mutations,
            version_totals.mutations,
            report.limits.max_mutations,
        )?;
        totals.preconditions = checked_total(
            "precondition_count",
            totals.preconditions,
            version_totals.preconditions,
            report.limits.max_preconditions,
        )?;
        totals.inline_value_bytes = checked_total(
            "inline_value_bytes",
            totals.inline_value_bytes,
            version_totals.inline_value_bytes,
            report.limits.max_inline_value_bytes,
        )?;
        expected_version = expected_version
            .checked_add(1)
            .ok_or_else(|| invalid_source_data("version sequence overflows u64"))?;
    }
    Ok(totals)
}

fn validate_version_contents(
    version: &NnsCertifiedRegistryDeltaVersion,
    limits: &NnsCertifiedRegistryDeltaLimits,
) -> Result<DeltaTotals, NnsRegistryHostError> {
    if version.mutations.is_empty() {
        return Err(invalid_source_data(format!(
            "version {} contains no mutations",
            version.version
        )));
    }
    let mut mutation_keys = BTreeSet::new();
    let mut inline_value_bytes = 0_usize;
    for mutation in &version.mutations {
        validate_hex_key(&mutation.key_hex, limits.max_key_bytes)?;
        if !mutation_keys.insert(&mutation.key_hex) {
            return Err(invalid_source_data(format!(
                "version {} mutates key {} more than once",
                version.version, mutation.key_hex
            )));
        }
        if mutation.mutation_type != mutation.mutation_kind.raw_type() {
            return Err(invalid_source_data(format!(
                "mutation type {} does not match kind {:?}",
                mutation.mutation_type, mutation.mutation_kind
            )));
        }
        inline_value_bytes = checked_total(
            "inline_value_bytes",
            inline_value_bytes,
            validated_value_bytes(mutation.mutation_kind, mutation.value_hex.as_deref())?,
            limits.max_inline_value_bytes,
        )?;
    }
    let mut precondition_keys = BTreeSet::new();
    for precondition in &version.preconditions {
        validate_hex_key(&precondition.key_hex, limits.max_key_bytes)?;
        if !precondition_keys.insert(&precondition.key_hex) {
            return Err(invalid_source_data(format!(
                "version {} repeats precondition key {}",
                version.version, precondition.key_hex
            )));
        }
    }
    Ok(DeltaTotals {
        mutations: version.mutations.len(),
        preconditions: version.preconditions.len(),
        inline_value_bytes,
    })
}

fn validated_value_bytes(
    kind: NnsCertifiedRegistryMutationKind,
    value: Option<&str>,
) -> Result<usize, NnsRegistryHostError> {
    match (kind, value) {
        (NnsCertifiedRegistryMutationKind::Delete, None) => Ok(0),
        (NnsCertifiedRegistryMutationKind::Delete, Some(_)) => {
            Err(invalid_source_data("delete mutation carries value_hex"))
        }
        (_, Some(value))
            if value.len().is_multiple_of(2) && crate::hex::is_lowercase_hex(value) =>
        {
            Ok(value.len() / 2)
        }
        (_, Some(_)) => Err(invalid_source_data(
            "mutation value_hex must be canonical lowercase hexadecimal",
        )),
        (_, None) => Err(invalid_source_data(
            "non-delete mutation must carry complete inline value_hex",
        )),
    }
}

fn validate_hex_key(value: &str, max_bytes: usize) -> Result<(), NnsRegistryHostError> {
    if !is_canonical_lowercase_hex(value) {
        return Err(invalid_source_data(
            "Registry keys must be nonempty canonical lowercase hexadecimal",
        ));
    }
    if value.len() / 2 > max_bytes {
        return Err(invalid_source_data(format!(
            "Registry key exceeds the maximum of {max_bytes} bytes"
        )));
    }
    Ok(())
}

fn checked_total(
    field: &str,
    current: usize,
    increment: usize,
    maximum: usize,
) -> Result<usize, NnsRegistryHostError> {
    let total = current
        .checked_add(increment)
        .ok_or_else(|| invalid_source_data(format!("{field} overflows usize")))?;
    if total > maximum {
        return Err(invalid_source_data(format!(
            "{field} exceeds the maximum of {maximum}"
        )));
    }
    Ok(total)
}

fn validate_text(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), NnsRegistryHostError> {
    if expected == actual {
        Ok(())
    } else {
        Err(NnsRegistryHostError::SourceMismatch {
            field,
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
    }
}

fn validate_equal<T>(field: &str, expected: T, actual: T) -> Result<(), NnsRegistryHostError>
where
    T: std::fmt::Debug + PartialEq,
{
    if expected == actual {
        Ok(())
    } else {
        Err(invalid_source_data(format!(
            "{field} mismatch: expected {expected:?}, got {actual:?}"
        )))
    }
}

fn invalid_source_data(reason: impl Into<String>) -> NnsRegistryHostError {
    NnsRegistryHostError::InvalidSourceData {
        reason: reason.into(),
    }
}
