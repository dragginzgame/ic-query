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
        NnsCertifiedRegistryDeltaVersion, NnsCertifiedRegistryMutation,
        NnsCertifiedRegistryMutationKind, NnsCertifiedRegistryValueEncoding,
    },
    source::{NnsCertifiedRegistryDeltaSource, nns_certified_registry_delta_limits},
};
use crate::{
    hex::{decode_lowercase_hex, is_canonical_lowercase_hex, is_lowercase_hex},
    nns::LiveNnsSource,
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs},
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

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
    let expected_query_call_count = report
        .chunk_query_call_count
        .checked_add(1)
        .ok_or_else(|| invalid_source_data("query_call_count overflows u64"))?;
    if report.query_call_count != expected_query_call_count {
        return Err(invalid_source_data(format!(
            "query_call_count must equal one certified query plus chunk_query_call_count; expected {expected_query_call_count}, got {}",
            report.query_call_count,
        )));
    }
    if report.certified_response_bytes == 0
        || report.certified_response_bytes > report.limits.max_response_body_bytes
    {
        return Err(invalid_source_data(format!(
            "certified_response_bytes must be within 1..={}, got {}",
            report.limits.max_response_body_bytes, report.certified_response_bytes
        )));
    }
    if report.chunk_response_bytes > report.limits.max_chunk_response_bytes {
        return Err(invalid_source_data(format!(
            "chunk_response_bytes exceeds the maximum of {}",
            report.limits.max_chunk_response_bytes
        )));
    }
    if (report.chunk_query_call_count == 0) != (report.chunk_response_bytes == 0) {
        return Err(invalid_source_data(
            "chunk query and response-byte accounting disagree",
        ));
    }
    let expected_response_bytes = report
        .certified_response_bytes
        .checked_add(report.chunk_response_bytes)
        .ok_or_else(|| invalid_source_data("response_bytes overflows usize"))?;
    validate_equal(
        "response_bytes",
        expected_response_bytes,
        report.response_bytes,
    )?;
    let minimum_evidence_bytes = report
        .certification
        .certificate_bytes
        .checked_add(report.certification.hash_tree_bytes)
        .ok_or_else(|| invalid_source_data("certified evidence byte counts overflow usize"))?;
    if report.certified_response_bytes < minimum_evidence_bytes {
        return Err(invalid_source_data(
            "certified_response_bytes is smaller than its certificate and hash-tree evidence",
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
    validate_equal(
        "chunk_value_bytes",
        totals.chunk_value_bytes,
        report.chunk_value_bytes,
    )?;
    validate_equal("value_bytes", totals.value_bytes, report.value_bytes)?;
    validate_equal(
        "chunk_reference_count",
        totals.chunk_reference_count,
        report.chunk_reference_count,
    )?;
    let chunk_evidence = validate_chunk_evidence(report, &totals.chunk_sha256_hexes)?;
    validate_chunked_values(report, &chunk_evidence)?;
    let unique_chunk_query_count = u64::try_from(chunk_evidence.len())
        .map_err(|_| invalid_source_data("unique chunk query count exceeds u64"))?;
    validate_equal(
        "chunk_query_call_count",
        unique_chunk_query_count,
        report.chunk_query_call_count,
    )?;
    let last_version = report.last_version.unwrap_or(report.requested_version);
    validate_equal(
        "more_available",
        last_version < report.certified_latest_version,
        report.more_available,
    )?;
    Ok(())
}

fn validate_chunk_evidence<'a>(
    report: &'a NnsCertifiedRegistryDeltaBatchReport,
    referenced_sha256_hexes: &BTreeSet<String>,
) -> Result<BTreeMap<&'a str, &'a str>, NnsRegistryHostError> {
    let mut chunks = BTreeMap::new();
    let mut evidence_bytes = 0_usize;
    let mut previous_sha256 = None;
    for row in &report.chunk_evidence {
        if row.sha256_hex.len() != 64 || !is_canonical_lowercase_hex(&row.sha256_hex) {
            return Err(invalid_source_data(
                "chunk evidence SHA-256 digests must be exactly 64 lowercase hexadecimal characters",
            ));
        }
        if previous_sha256.is_some_and(|previous| previous >= row.sha256_hex.as_str()) {
            return Err(invalid_source_data(
                "chunk evidence must be unique and strictly ordered by SHA-256 digest",
            ));
        }
        if !row.content_hex.len().is_multiple_of(2) || !is_lowercase_hex(&row.content_hex) {
            return Err(invalid_source_data(
                "chunk evidence content must be lowercase hexadecimal with an even length",
            ));
        }
        let content = decode_lowercase_hex(&row.content_hex).ok_or_else(|| {
            invalid_source_data(
                "chunk evidence content could not be decoded as lowercase hexadecimal",
            )
        })?;
        if content.len() > report.limits.max_chunk_bytes {
            return Err(invalid_source_data(format!(
                "chunk evidence content exceeds the maximum of {} bytes",
                report.limits.max_chunk_bytes
            )));
        }
        let actual_sha256 = crate::hex::hex_bytes(&Sha256::digest(&content));
        if actual_sha256 != row.sha256_hex {
            return Err(invalid_source_data(format!(
                "chunk evidence content does not match SHA-256 digest {}",
                row.sha256_hex
            )));
        }
        evidence_bytes = checked_total(
            "chunk_evidence_bytes",
            evidence_bytes,
            content.len(),
            report.limits.max_value_bytes,
        )?;
        chunks.insert(row.sha256_hex.as_str(), row.content_hex.as_str());
        previous_sha256 = Some(row.sha256_hex.as_str());
    }
    validate_equal(
        "chunk_evidence_bytes",
        evidence_bytes,
        report.chunk_evidence_bytes,
    )?;
    if chunks.len() != referenced_sha256_hexes.len()
        || referenced_sha256_hexes
            .iter()
            .any(|sha256| !chunks.contains_key(sha256.as_str()))
    {
        return Err(invalid_source_data(
            "chunk evidence must contain exactly the unique digests referenced by mutations",
        ));
    }
    Ok(chunks)
}

fn validate_chunked_values(
    report: &NnsCertifiedRegistryDeltaBatchReport,
    chunks: &BTreeMap<&str, &str>,
) -> Result<(), NnsRegistryHostError> {
    for mutation in report
        .versions
        .iter()
        .flat_map(|version| version.mutations.iter())
        .filter(|mutation| mutation.value_encoding == NnsCertifiedRegistryValueEncoding::Chunked)
    {
        let value_hex = mutation.value_hex.as_deref().ok_or_else(|| {
            invalid_source_data("chunked mutation is missing reconstructed value content")
        })?;
        let mut offset = 0_usize;
        for sha256 in &mutation.chunk_sha256_hexes {
            let content_hex = chunks.get(sha256.as_str()).ok_or_else(|| {
                invalid_source_data(format!(
                    "chunked mutation references missing chunk evidence {sha256}"
                ))
            })?;
            let end = offset.checked_add(content_hex.len()).ok_or_else(|| {
                invalid_source_data("chunked value reconstruction overflows usize")
            })?;
            if value_hex.get(offset..end) != Some(*content_hex) {
                return Err(invalid_source_data(
                    "chunked mutation value does not match its ordered chunk evidence",
                ));
            }
            offset = end;
        }
        if offset != value_hex.len() {
            return Err(invalid_source_data(
                "chunked mutation value does not match its ordered chunk evidence",
            ));
        }
    }
    Ok(())
}

#[derive(Default)]
struct DeltaTotals {
    mutations: usize,
    preconditions: usize,
    inline_value_bytes: usize,
    chunk_value_bytes: usize,
    value_bytes: usize,
    chunk_reference_count: usize,
    chunk_sha256_hexes: BTreeSet<String>,
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
        totals.chunk_value_bytes = checked_total(
            "chunk_value_bytes",
            totals.chunk_value_bytes,
            version_totals.chunk_value_bytes,
            report.limits.max_value_bytes,
        )?;
        totals.value_bytes = checked_total(
            "value_bytes",
            totals.value_bytes,
            version_totals.value_bytes,
            report.limits.max_value_bytes,
        )?;
        totals.chunk_reference_count = checked_total(
            "chunk_reference_count",
            totals.chunk_reference_count,
            version_totals.chunk_reference_count,
            report.limits.max_chunk_references,
        )?;
        totals
            .chunk_sha256_hexes
            .extend(version_totals.chunk_sha256_hexes);
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
    if let Some(pair) = version
        .mutations
        .windows(2)
        .find(|pair| pair[0].key_hex > pair[1].key_hex)
    {
        return Err(invalid_source_data(format!(
            "version {} mutation key {} follows {} out of canonical order",
            version.version, pair[1].key_hex, pair[0].key_hex
        )));
    }
    let mut totals = DeltaTotals::default();
    for mutation in &version.mutations {
        validate_hex_key(&mutation.key_hex, limits.max_key_bytes)?;
        if mutation.mutation_type != mutation.mutation_kind.raw_type() {
            return Err(invalid_source_data(format!(
                "mutation type {} does not match kind {:?}",
                mutation.mutation_type, mutation.mutation_kind
            )));
        }
        let value = validate_mutation_value(mutation, limits)?;
        totals.inline_value_bytes = checked_total(
            "inline_value_bytes",
            totals.inline_value_bytes,
            value.inline,
            limits.max_inline_value_bytes,
        )?;
        totals.chunk_value_bytes = checked_total(
            "chunk_value_bytes",
            totals.chunk_value_bytes,
            value.chunked,
            limits.max_value_bytes,
        )?;
        totals.value_bytes = checked_total(
            "value_bytes",
            totals.value_bytes,
            value.total,
            limits.max_value_bytes,
        )?;
        totals.chunk_reference_count = checked_total(
            "chunk_reference_count",
            totals.chunk_reference_count,
            mutation.chunk_sha256_hexes.len(),
            limits.max_chunk_references,
        )?;
        totals
            .chunk_sha256_hexes
            .extend(mutation.chunk_sha256_hexes.iter().cloned());
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
    totals.mutations = version.mutations.len();
    totals.preconditions = version.preconditions.len();
    Ok(totals)
}

#[derive(Clone, Copy)]
struct MutationValueBytes {
    inline: usize,
    chunked: usize,
    total: usize,
}

fn validate_mutation_value(
    mutation: &NnsCertifiedRegistryMutation,
    limits: &NnsCertifiedRegistryDeltaLimits,
) -> Result<MutationValueBytes, NnsRegistryHostError> {
    let value_bytes = match mutation.value_hex.as_deref() {
        Some(value) if value.len().is_multiple_of(2) && crate::hex::is_lowercase_hex(value) => {
            value.len() / 2
        }
        Some(_) => {
            return Err(invalid_source_data(
                "mutation value_hex must be lowercase hexadecimal with an even length",
            ));
        }
        None => 0,
    };
    match mutation.value_encoding {
        NnsCertifiedRegistryValueEncoding::Absent => {
            if mutation.mutation_kind != NnsCertifiedRegistryMutationKind::Delete
                || mutation.value_hex.is_some()
                || !mutation.chunk_sha256_hexes.is_empty()
            {
                return Err(invalid_source_data(
                    "absent value encoding requires a delete with no value or chunk hashes",
                ));
            }
            Ok(MutationValueBytes {
                inline: 0,
                chunked: 0,
                total: 0,
            })
        }
        NnsCertifiedRegistryValueEncoding::Inline => {
            if mutation.value_hex.is_none() || !mutation.chunk_sha256_hexes.is_empty() {
                return Err(invalid_source_data(
                    "inline value encoding requires value content and no chunk hashes",
                ));
            }
            Ok(MutationValueBytes {
                inline: value_bytes,
                chunked: 0,
                total: value_bytes,
            })
        }
        NnsCertifiedRegistryValueEncoding::Chunked => {
            if mutation.value_hex.is_none() || mutation.chunk_sha256_hexes.is_empty() {
                return Err(invalid_source_data(
                    "chunked value encoding requires reconstructed value content and chunk hashes",
                ));
            }
            if value_bytes > limits.max_reconstructed_value_bytes {
                return Err(invalid_source_data(format!(
                    "reconstructed value exceeds the maximum of {} bytes",
                    limits.max_reconstructed_value_bytes
                )));
            }
            for hash in &mutation.chunk_sha256_hexes {
                if hash.len() != 64 || !is_canonical_lowercase_hex(hash) {
                    return Err(invalid_source_data(
                        "chunk SHA-256 digests must be exactly 64 lowercase hexadecimal characters",
                    ));
                }
            }
            Ok(MutationValueBytes {
                inline: 0,
                chunked: value_bytes,
                total: value_bytes,
            })
        }
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
