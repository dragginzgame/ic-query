use super::{
    error::NnsRegistryHostError,
    model::{
        NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION, NnsRegistryVersionReport,
        NnsRegistryVersionRequest,
    },
    source::{NnsRegistrySource, NnsRegistryVersionData},
};
use crate::{
    certification::validate_certificate_time,
    hex::is_lowercase_hex,
    nns::{LiveNnsSource, NnsSourceRequest},
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs},
};

impl_nns_mainnet_network_enforcer!(NnsRegistryHostError);

pub fn build_nns_registry_version_report(
    request: &NnsRegistryVersionRequest,
) -> Result<NnsRegistryVersionReport, NnsRegistryHostError> {
    build_nns_registry_version_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_registry_version_report_with_source(
    request: &NnsRegistryVersionRequest,
    source: &dyn NnsRegistrySource,
) -> Result<NnsRegistryVersionReport, NnsRegistryHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let fetch_request = NnsSourceRequest::new(
        MAINNET_NETWORK,
        &request.source_endpoint,
        fetched_at,
        "ic-query",
    );
    let version = source.fetch_registry_version(&fetch_request)?;
    validate_source_result(&fetch_request, &version)?;
    validate_certification(request.now_unix_secs, &version.certification)?;
    Ok(registry_version_report_from_version(version))
}

fn validate_source_result(
    request: &NnsSourceRequest,
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
    if certification.root_key_digest.len() != 64
        || !is_lowercase_hex(&certification.root_key_digest)
    {
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
    if byte_count == 0 {
        return Err(invalid_source_data(format!("{field} must not be empty")));
    }
    let expected_length = byte_count.checked_mul(2).ok_or_else(|| {
        invalid_source_data(format!("{field} byte count exceeds the supported range"))
    })?;
    if value.len() != expected_length {
        return Err(invalid_source_data(format!(
            "{field} length does not match its byte count"
        )));
    }
    if !is_lowercase_hex(value) {
        return Err(invalid_source_data(format!(
            "{field} must be canonical lowercase hexadecimal"
        )));
    }
    Ok(())
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
