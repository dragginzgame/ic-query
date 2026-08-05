use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::{
    certification::{CertifiedDataError, validate_certificate_time},
    ic_registry::{
        MainnetRegistryCertification, MainnetRegistryFetchRequest, MainnetRegistryVersion,
        RegistryFetchError,
        model::CertifiedRegistryDeltaBatch,
        transport::{get_certified_changes_since, get_certified_latest_version},
    },
    subnet_catalog::{
        MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs,
        parse_utc_timestamp_secs,
    },
};

pub(in crate::ic_registry) async fn fetch_mainnet_registry_version_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetRegistryVersion, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let certified = get_certified_latest_version(&agent, &registry_canister).await?;
    validate_live_certificate_time(request, certified.certificate_time_nanos)?;
    Ok(MainnetRegistryVersion {
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: certified.registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        certification: MainnetRegistryCertification {
            certificate_verified: true,
            certificate_time_nanos: certified.certificate_time_nanos,
            certificate_time: format_utc_timestamp_secs(
                certified.certificate_time_nanos / 1_000_000_000,
            ),
            root_key_digest: certified.root_key_digest,
            certificate_hex: certified.certificate_hex,
            certificate_bytes: certified.certificate_bytes,
            hash_tree_hex: certified.hash_tree_hex,
            hash_tree_bytes: certified.hash_tree_bytes,
        },
    })
}

pub(in crate::ic_registry) async fn fetch_mainnet_certified_registry_delta_batch_async(
    request: &MainnetRegistryFetchRequest,
    requested_version: u64,
) -> Result<CertifiedRegistryDeltaBatch, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let batch = get_certified_changes_since(&agent, &registry_canister, requested_version).await?;
    validate_live_certificate_time(request, batch.certificate_time_nanos)?;
    Ok(batch)
}

fn validate_live_certificate_time(
    request: &MainnetRegistryFetchRequest,
    certificate_time_nanos: u64,
) -> Result<(), RegistryFetchError> {
    let fetched_at = parse_utc_timestamp_secs(&request.fetched_at).ok_or_else(|| {
        RegistryFetchError::InvalidCertifiedRegistry {
            reason: format!(
                "collection time {:?} is not a supported UTC timestamp",
                request.fetched_at
            ),
        }
    })?;
    validate_certificate_time(fetched_at, certificate_time_nanos).map_err(|error| match error {
        CertifiedDataError::Authentication { reason } | CertifiedDataError::Invalid { reason } => {
            RegistryFetchError::InvalidCertifiedRegistry { reason }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certificate_time_must_be_close_to_the_caller_observation_time() {
        let request = MainnetRegistryFetchRequest {
            endpoint: "https://icp-api.io".to_string(),
            fetched_at: "2026-08-05T12:00:00Z".to_string(),
            fetched_by: "test".to_string(),
        };
        let fetched_at = parse_utc_timestamp_secs(&request.fetched_at).expect("timestamp");

        for certificate_time in [
            fetched_at - crate::certification::MAX_CERTIFICATE_TIME_SKEW_SECONDS,
            fetched_at,
            fetched_at + crate::certification::MAX_CERTIFICATE_TIME_SKEW_SECONDS,
        ] {
            validate_live_certificate_time(&request, certificate_time * 1_000_000_000)
                .expect("accepted skew boundary");
        }

        let error = validate_live_certificate_time(
            &request,
            (fetched_at + crate::certification::MAX_CERTIFICATE_TIME_SKEW_SECONDS + 1)
                * 1_000_000_000,
        )
        .expect_err("future certificate outside skew");
        assert!(matches!(
            error,
            RegistryFetchError::InvalidCertifiedRegistry { reason }
                if reason.contains("outside the accepted")
        ));
    }
}
