use super::{
    error::NnsRegistryHostError,
    model::{
        NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION, NnsCertifiedRegistryDeltaBatchReport,
        NnsCertifiedRegistryDeltaBatchRequest, NnsCertifiedRegistryDeltaLimits,
        NnsCertifiedRegistryDeltaVersion, NnsCertifiedRegistryMutation,
        NnsCertifiedRegistryMutationKind, NnsCertifiedRegistryPrecondition,
        NnsRegistryCertification,
    },
};
use crate::{
    agent::MAX_IC_AGENT_RESPONSE_BODY_BYTES,
    ic_registry::{
        CertifiedRegistryDeltaBatch, MAX_CERTIFIED_DELTA_INLINE_VALUE_BYTES,
        MAX_CERTIFIED_DELTA_KEY_BYTES, MAX_CERTIFIED_DELTA_MUTATIONS,
        MAX_CERTIFIED_DELTA_PRECONDITIONS, MAX_CERTIFIED_DELTA_VERSIONS, MainnetRegistryVersion,
        fetch_mainnet_certified_registry_delta_batch_async, fetch_mainnet_registry_version,
    },
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs},
};
use std::{future::Future, pin::Pin};

///
/// NnsRegistryVersionData
///
/// Source-layer NNS registry version result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsRegistryVersionData {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    /// Authenticated evidence for the certified latest version.
    pub certification: NnsRegistryCertification,
}

impl From<MainnetRegistryVersion> for NnsRegistryVersionData {
    fn from(version: MainnetRegistryVersion) -> Self {
        Self {
            network: version.network,
            registry_canister_id: version.registry_canister_id,
            registry_version: version.registry_version,
            fetched_at: version.fetched_at,
            fetched_by: version.fetched_by,
            source_endpoint: version.source_endpoint,
            certification: NnsRegistryCertification {
                certificate_verified: version.certification.certificate_verified,
                certificate_time_nanos: version.certification.certificate_time_nanos,
                certificate_time: version.certification.certificate_time,
                root_key_digest: version.certification.root_key_digest,
                certificate_hex: version.certification.certificate_hex,
                certificate_bytes: version.certification.certificate_bytes,
                hash_tree_hex: version.certification.hash_tree_hex,
                hash_tree_bytes: version.certification.hash_tree_bytes,
            },
        }
    }
}

///
/// NnsRegistrySource
///
/// Source contract for fetching NNS registry version data.
///

pub trait NnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError>;
}

impl NnsRegistrySource for LiveNnsSource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
        let fetch_request = mainnet_registry_fetch_request(request, |network| {
            NnsRegistryHostError::UnsupportedNetwork { network }
        })?;
        Ok(fetch_mainnet_registry_version(&fetch_request)?.into())
    }
}

///
/// NnsCertifiedRegistryDeltaSourceFuture
///
/// Caller-runtime future returned by a certified Registry delta source.
///

pub type NnsCertifiedRegistryDeltaSourceFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<NnsCertifiedRegistryDeltaBatchReport, NnsRegistryHostError>>
            + Send
            + 'a,
    >,
>;

///
/// NnsCertifiedRegistryDeltaSource
///
/// Async source contract for one authenticated, bounded Registry delta batch.
/// Custom implementations are responsible for authenticating their raw certificate evidence.
///

pub trait NnsCertifiedRegistryDeltaSource: Sync {
    /// Fetch one authenticated batch after the explicitly requested Registry version.
    fn fetch_certified_registry_delta_batch<'a>(
        &'a self,
        request: &'a NnsCertifiedRegistryDeltaBatchRequest,
    ) -> NnsCertifiedRegistryDeltaSourceFuture<'a>;
}

impl NnsCertifiedRegistryDeltaSource for LiveNnsSource {
    fn fetch_certified_registry_delta_batch<'a>(
        &'a self,
        request: &'a NnsCertifiedRegistryDeltaBatchRequest,
    ) -> NnsCertifiedRegistryDeltaSourceFuture<'a> {
        Box::pin(async move {
            let source_request = NnsSourceRequest::from_unix_secs(
                &request.network,
                &request.source_endpoint,
                request.now_unix_secs,
                "ic-query",
            );
            let fetch_request = mainnet_registry_fetch_request(&source_request, |network| {
                NnsRegistryHostError::UnsupportedNetwork { network }
            })?;
            let batch = fetch_mainnet_certified_registry_delta_batch_async(
                &fetch_request,
                request.requested_version,
            )
            .await?;
            report_from_live_batch(request, batch)
        })
    }
}

fn report_from_live_batch(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    batch: CertifiedRegistryDeltaBatch,
) -> Result<NnsCertifiedRegistryDeltaBatchReport, NnsRegistryHostError> {
    let first_version = batch.versions.first().map(|row| row.version);
    let last_version = batch.versions.last().map(|row| row.version);
    let versions = batch
        .versions
        .into_iter()
        .map(|version| {
            let mutations = version
                .mutations
                .into_iter()
                .map(|mutation| {
                    let mutation_kind =
                        NnsCertifiedRegistryMutationKind::from_raw_type(mutation.mutation_type)
                            .ok_or_else(|| NnsRegistryHostError::InvalidSourceData {
                                reason: format!(
                                    "live delta contains unsupported mutation type {}",
                                    mutation.mutation_type
                                ),
                            })?;
                    Ok(NnsCertifiedRegistryMutation {
                        mutation_type: mutation.mutation_type,
                        mutation_kind,
                        key_hex: mutation.key_hex,
                        value_hex: mutation.value_hex,
                    })
                })
                .collect::<Result<Vec<_>, NnsRegistryHostError>>()?;
            Ok(NnsCertifiedRegistryDeltaVersion {
                version: version.version,
                timestamp_nanoseconds: version.timestamp_nanoseconds,
                mutations,
                preconditions: version
                    .preconditions
                    .into_iter()
                    .map(|precondition| NnsCertifiedRegistryPrecondition {
                        key_hex: precondition.key_hex,
                        expected_version: precondition.expected_version,
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, NnsRegistryHostError>>()?;
    let certificate_time = format_utc_timestamp_secs(batch.certificate_time_nanos / 1_000_000_000);

    Ok(NnsCertifiedRegistryDeltaBatchReport {
        schema_version: NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        requested_version: batch.requested_version,
        certified_latest_version: batch.certified_latest_version,
        first_version,
        last_version,
        version_count: versions.len(),
        mutation_count: batch.mutation_count,
        precondition_count: batch.precondition_count,
        inline_value_bytes: batch.inline_value_bytes,
        more_available: batch.more_available,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        query_call_count: 1,
        response_bytes: batch.response_bytes,
        limits: nns_certified_registry_delta_limits(),
        versions,
        certification: NnsRegistryCertification {
            certificate_verified: true,
            certificate_time_nanos: batch.certificate_time_nanos,
            certificate_time,
            root_key_digest: batch.root_key_digest,
            certificate_hex: batch.certificate_hex,
            certificate_bytes: batch.certificate_bytes,
            hash_tree_hex: batch.hash_tree_hex,
            hash_tree_bytes: batch.hash_tree_bytes,
        },
    })
}

/// Return the fixed resource ceilings enforced for every certified delta batch.
#[must_use]
pub const fn nns_certified_registry_delta_limits() -> NnsCertifiedRegistryDeltaLimits {
    NnsCertifiedRegistryDeltaLimits {
        max_versions: MAX_CERTIFIED_DELTA_VERSIONS,
        max_mutations: MAX_CERTIFIED_DELTA_MUTATIONS,
        max_preconditions: MAX_CERTIFIED_DELTA_PRECONDITIONS,
        max_key_bytes: MAX_CERTIFIED_DELTA_KEY_BYTES,
        max_inline_value_bytes: MAX_CERTIFIED_DELTA_INLINE_VALUE_BYTES,
        max_response_bytes: MAX_IC_AGENT_RESPONSE_BODY_BYTES,
    }
}
