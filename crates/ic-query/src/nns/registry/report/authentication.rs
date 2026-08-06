//! Module: nns::registry::report::authentication
//!
//! Responsibility: reauthenticate one retained certified Registry delta report locally.
//! Does not own: source calls, replay, persistence, catalog projection, or assurance policy.
//! Boundary: the sealed capability is returned only after raw mainnet evidence is reverified.

use super::{
    NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
    NnsCertifiedRegistryMutationKind, NnsCertifiedRegistryValueEncoding, NnsRegistryHostError,
    validate_nns_certified_registry_delta_batch,
};
use crate::{
    agent::build_historical_certificate_agent,
    hex::{decode_lowercase_hex, hex_bytes},
    ic_registry::{
        AuthenticatedRegistryDeltaWitness, CertifiedRegistryValueEncoding, RegistryFetchError,
        authenticate_certified_registry_delta_witness,
    },
    subnet_catalog::MAINNET_REGISTRY_CANISTER_ID,
};
use candid::Principal;
use std::fmt::Debug;

///
/// NnsAuthenticatedRegistryDeltaBatch
///
/// Borrowed schema-3 report whose retained mainnet certificate and witness were reauthenticated.
///

#[derive(Debug)]
pub struct NnsAuthenticatedRegistryDeltaBatch<'a> {
    report: &'a NnsCertifiedRegistryDeltaBatchReport,
}

impl<'a> NnsAuthenticatedRegistryDeltaBatch<'a> {
    /// Return the structurally validated report qualified by this authentication capability.
    #[must_use]
    pub const fn report(&self) -> &'a NnsCertifiedRegistryDeltaBatchReport {
        self.report
    }

    #[cfg(test)]
    pub(crate) const fn from_validated_fixture(
        report: &'a NnsCertifiedRegistryDeltaBatchReport,
    ) -> Self {
        Self { report }
    }
}

/// Reauthenticate one retained certified Registry delta report without a network call.
///
/// This operation rebuilds the mainnet-root-key verifier locally, authenticates the raw
/// certificate and mixed-tree commitment, decodes the committed delta contents, and compares
/// them with every report version, mutation, precondition, and chunk reference.
pub fn reauthenticate_nns_certified_registry_delta_batch<'a>(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    report: &'a NnsCertifiedRegistryDeltaBatchReport,
) -> Result<NnsAuthenticatedRegistryDeltaBatch<'a>, NnsRegistryHostError> {
    validate_nns_certified_registry_delta_batch(request, report)?;
    let agent = build_historical_certificate_agent(&request.source_endpoint, |reason| {
        RegistryFetchError::AgentBuild {
            endpoint: request.source_endpoint.clone(),
            reason,
        }
    })
    .map_err(authentication_error)?;
    let registry_canister =
        Principal::from_text(MAINNET_REGISTRY_CANISTER_ID).map_err(|error| {
            authentication_error(RegistryFetchError::InvalidPrincipal {
                field: "registry_canister_id",
                reason: error.to_string(),
            })
        })?;
    let certificate = decode_evidence("certificate_hex", &report.certification.certificate_hex)?;
    let hash_tree = decode_evidence("hash_tree_hex", &report.certification.hash_tree_hex)?;
    let witness = authenticate_certified_registry_delta_witness(
        &agent,
        &registry_canister,
        report.requested_version,
        certificate,
        &hash_tree,
    )
    .map_err(authentication_error)?;
    validate_authenticated_witness(report, &witness)?;
    Ok(NnsAuthenticatedRegistryDeltaBatch { report })
}

fn decode_evidence(field: &'static str, value: &str) -> Result<Vec<u8>, NnsRegistryHostError> {
    decode_lowercase_hex(value).ok_or_else(|| NnsRegistryHostError::InvalidSourceData {
        reason: format!("{field} could not be decoded as lowercase hexadecimal"),
    })
}

fn validate_authenticated_witness(
    report: &NnsCertifiedRegistryDeltaBatchReport,
    witness: &AuthenticatedRegistryDeltaWitness,
) -> Result<(), NnsRegistryHostError> {
    validate_authenticated_metadata(report, witness)?;
    validate_authenticated_versions(report, witness)
}

fn validate_authenticated_metadata(
    report: &NnsCertifiedRegistryDeltaBatchReport,
    witness: &AuthenticatedRegistryDeltaWitness,
) -> Result<(), NnsRegistryHostError> {
    authenticated_equal(
        "certified_latest_version",
        &witness.certified_latest_version,
        &report.certified_latest_version,
    )?;
    authenticated_equal(
        "mutation_count",
        &witness.mutation_count,
        &report.mutation_count,
    )?;
    authenticated_equal(
        "precondition_count",
        &witness.precondition_count,
        &report.precondition_count,
    )?;
    authenticated_equal(
        "inline_value_bytes",
        &witness.inline_value_bytes,
        &report.inline_value_bytes,
    )?;
    authenticated_equal(
        "chunk_reference_count",
        &witness.chunk_reference_count,
        &report.chunk_reference_count,
    )?;
    authenticated_equal(
        "more_available",
        &witness.more_available,
        &report.more_available,
    )?;
    authenticated_equal(
        "certificate_time_nanos",
        &witness.certificate_time_nanos,
        &report.certification.certificate_time_nanos,
    )?;
    authenticated_equal(
        "root_key_digest",
        &witness.root_key_digest,
        &report.certification.root_key_digest,
    )?;
    authenticated_equal(
        "certificate_hex",
        &witness.certificate_hex,
        &report.certification.certificate_hex,
    )?;
    authenticated_equal(
        "certificate_bytes",
        &witness.certificate_bytes,
        &report.certification.certificate_bytes,
    )?;
    authenticated_equal(
        "hash_tree_hex",
        &witness.hash_tree_hex,
        &report.certification.hash_tree_hex,
    )?;
    authenticated_equal(
        "hash_tree_bytes",
        &witness.hash_tree_bytes,
        &report.certification.hash_tree_bytes,
    )?;
    authenticated_equal(
        "version_count",
        &witness.versions.len(),
        &report.versions.len(),
    )?;
    Ok(())
}

fn validate_authenticated_versions(
    report: &NnsCertifiedRegistryDeltaBatchReport,
    witness: &AuthenticatedRegistryDeltaWitness,
) -> Result<(), NnsRegistryHostError> {
    for (version_index, (committed, reported)) in
        witness.versions.iter().zip(&report.versions).enumerate()
    {
        let context = format!("versions[{version_index}]");
        authenticated_equal(
            &format!("{context}.version"),
            &committed.version,
            &reported.version,
        )?;
        authenticated_equal(
            &format!("{context}.timestamp_nanoseconds"),
            &committed.timestamp_nanoseconds,
            &reported.timestamp_nanoseconds,
        )?;
        authenticated_equal(
            &format!("{context}.mutation_count"),
            &committed.mutations.len(),
            &reported.mutations.len(),
        )?;
        authenticated_equal(
            &format!("{context}.precondition_count"),
            &committed.preconditions.len(),
            &reported.preconditions.len(),
        )?;
        validate_authenticated_mutations(&context, committed, reported)?;
        validate_authenticated_preconditions(&context, committed, reported)?;
    }
    Ok(())
}

fn validate_authenticated_mutations(
    version_context: &str,
    committed: &crate::ic_registry::CertifiedRegistryDeltaVersion,
    reported: &super::NnsCertifiedRegistryDeltaVersion,
) -> Result<(), NnsRegistryHostError> {
    for (mutation_index, (committed, reported)) in committed
        .mutations
        .iter()
        .zip(&reported.mutations)
        .enumerate()
    {
        let context = format!("{version_context}.mutations[{mutation_index}]");
        authenticated_equal(
            &format!("{context}.mutation_type"),
            &committed.mutation_type,
            &reported.mutation_type,
        )?;
        authenticated_equal(
            &format!("{context}.key_hex"),
            &committed.key_hex,
            &reported.key_hex,
        )?;
        authenticated_equal(
            &format!("{context}.value_encoding"),
            &public_value_encoding(committed.value_encoding),
            &reported.value_encoding,
        )?;
        authenticated_equal(
            &format!("{context}.mutation_kind"),
            &NnsCertifiedRegistryMutationKind::from_raw_type(committed.mutation_type),
            &Some(reported.mutation_kind),
        )?;
        let chunk_sha256_hexes = committed
            .chunk_sha256s
            .iter()
            .map(|sha256| hex_bytes(sha256))
            .collect::<Vec<_>>();
        authenticated_equal(
            &format!("{context}.chunk_sha256_hexes"),
            &chunk_sha256_hexes,
            &reported.chunk_sha256_hexes,
        )?;
        if committed.value_encoding != CertifiedRegistryValueEncoding::Chunked {
            authenticated_equal(
                &format!("{context}.value_hex"),
                &committed.value_hex,
                &reported.value_hex,
            )?;
        }
    }
    Ok(())
}

fn validate_authenticated_preconditions(
    version_context: &str,
    committed: &crate::ic_registry::CertifiedRegistryDeltaVersion,
    reported: &super::NnsCertifiedRegistryDeltaVersion,
) -> Result<(), NnsRegistryHostError> {
    for (precondition_index, (committed, reported)) in committed
        .preconditions
        .iter()
        .zip(&reported.preconditions)
        .enumerate()
    {
        let context = format!("{version_context}.preconditions[{precondition_index}]");
        authenticated_equal(
            &format!("{context}.key_hex"),
            &committed.key_hex,
            &reported.key_hex,
        )?;
        authenticated_equal(
            &format!("{context}.expected_version"),
            &committed.expected_version,
            &reported.expected_version,
        )?;
    }
    Ok(())
}

const fn public_value_encoding(
    value: CertifiedRegistryValueEncoding,
) -> NnsCertifiedRegistryValueEncoding {
    match value {
        CertifiedRegistryValueEncoding::Absent => NnsCertifiedRegistryValueEncoding::Absent,
        CertifiedRegistryValueEncoding::Inline => NnsCertifiedRegistryValueEncoding::Inline,
        CertifiedRegistryValueEncoding::Chunked => NnsCertifiedRegistryValueEncoding::Chunked,
    }
}

fn authenticated_equal<T: Debug + PartialEq>(
    field: &str,
    authenticated: &T,
    reported: &T,
) -> Result<(), NnsRegistryHostError> {
    if authenticated == reported {
        Ok(())
    } else {
        Err(authentication_error(
            RegistryFetchError::InvalidCertifiedRegistry {
                reason: format!(
                    "authenticated {field} mismatch: expected {authenticated:?}, got {reported:?}"
                ),
            },
        ))
    }
}

const fn authentication_error(source: RegistryFetchError) -> NnsRegistryHostError {
    NnsRegistryHostError::EvidenceAuthentication { source }
}
