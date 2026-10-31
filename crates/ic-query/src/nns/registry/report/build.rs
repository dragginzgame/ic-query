use super::{
    error::NnsRegistryHostError,
    model::{
        NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION, NnsRegistryVersionReport,
        NnsRegistryVersionRequest,
    },
    source::{NnsRegistrySource, NnsRegistryVersionData},
};
use crate::report::CollectionSourceRequest;
use crate::{
    certification::validate_certificate_time,
    hex::{HexEvidenceError, is_lowercase_hex_32_bytes, validate_nonempty_hex_evidence},
    nns::LiveNnsSource,
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};
use ic_host_text::timestamp::format_utc_timestamp_secs;

impl_nns_mainnet_network_enforcer!(NnsRegistryHostError);

pub async fn build_nns_registry_version_report(
    request: &NnsRegistryVersionRequest,
) -> Result<NnsRegistryVersionReport, NnsRegistryHostError> {
    build_nns_registry_version_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_registry_version_report_with_source(
    request: &NnsRegistryVersionRequest,
    source: &dyn NnsRegistrySource,
) -> Result<NnsRegistryVersionReport, NnsRegistryHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let fetch_request = CollectionSourceRequest::new(
        MAINNET_NETWORK,
        &request.source_endpoint,
        fetched_at,
        "ic-query",
    );
    let version = source.fetch_registry_version(&fetch_request).await?;
    validate_source_result(&fetch_request, &version)?;
    validate_certification(request.now_unix_secs, &version.certification)?;
    Ok(registry_version_report_from_version(version))
}

fn validate_source_result(
    request: &CollectionSourceRequest,
    version: &NnsRegistryVersionData,
) -> Result<(), NnsRegistryHostError> {
    for (field, expected, actual) in [
        (
            "network",
            request.network.as_str(),
            version.network.as_str(),
        ),
        (
            "registry_canister_id",
            MAINNET_REGISTRY_CANISTER_ID,
            version.registry_canister_id.as_str(),
        ),
        (
            "fetched_at",
            request.fetched_at.as_str(),
            version.fetched_at.as_str(),
        ),
        (
            "source_endpoint",
            request.endpoint.as_str(),
            version.source_endpoint.as_str(),
        ),
        (
            "fetched_by",
            request.fetched_by.as_str(),
            version.fetched_by.as_str(),
        ),
    ] {
        if expected != actual {
            return Err(NnsRegistryHostError::SourceMismatch {
                field,
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
    }
    Ok(())
}

pub(super) fn validate_certification(
    now_unix_secs: u64,
    certification: &super::model::NnsRegistryCertification,
) -> Result<(), NnsRegistryHostError> {
    if !certification.certificate_verified {
        return Err(invalid_source_data(
            "certificate_verified must be true for NnsRegistrySource results",
        ));
    }
    validate_evidence_hex(
        "certificate_hex",
        &certification.certificate_hex,
        certification.certificate_bytes,
    )?;
    validate_evidence_hex(
        "hash_tree_hex",
        &certification.hash_tree_hex,
        certification.hash_tree_bytes,
    )?;
    if !is_lowercase_hex_32_bytes(&certification.root_key_digest) {
        return Err(invalid_source_data(
            "root_key_digest must be exactly 32 bytes of lowercase hexadecimal",
        ));
    }

    let certificate_time_secs = certification.certificate_time_nanos / 1_000_000_000;
    if certification.certificate_time != format_utc_timestamp_secs(certificate_time_secs) {
        return Err(invalid_source_data(
            "certificate_time does not match certificate_time_nanos",
        ));
    }
    validate_certificate_time(now_unix_secs, certification.certificate_time_nanos).map_err(
        |error| {
            invalid_source_data(match error {
                crate::certification::CertifiedDataError::Authentication { reason }
                | crate::certification::CertifiedDataError::Invalid { reason } => reason,
            })
        },
    )?;
    Ok(())
}

fn validate_evidence_hex(
    field: &str,
    value: &str,
    byte_count: usize,
) -> Result<(), NnsRegistryHostError> {
    validate_nonempty_hex_evidence(value, byte_count).map_err(|error| {
        invalid_source_data(match error {
            HexEvidenceError::Empty => format!("{field} must not be empty"),
            HexEvidenceError::ByteCountOverflow => {
                format!("{field} byte count exceeds the supported range")
            }
            HexEvidenceError::LengthMismatch => {
                format!("{field} length does not match its byte count")
            }
            HexEvidenceError::NonLowercaseHex => {
                format!("{field} must be canonical lowercase hexadecimal")
            }
        })
    })
}

fn invalid_source_data(reason: impl Into<String>) -> NnsRegistryHostError {
    NnsRegistryHostError::InvalidSourceData {
        reason: reason.into(),
    }
}

fn registry_version_report_from_version(
    version: NnsRegistryVersionData,
) -> NnsRegistryVersionReport {
    NnsRegistryVersionReport {
        schema_version: NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION,
        network: version.network,
        registry_canister_id: version.registry_canister_id,
        registry_version: version.registry_version,
        fetched_at: version.fetched_at,
        source_endpoint: version.source_endpoint,
        fetched_by: version.fetched_by,
        certification: version.certification,
    }
}
