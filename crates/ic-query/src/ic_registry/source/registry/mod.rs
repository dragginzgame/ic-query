use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::nns::registry::NnsRegistryVersionData;
use crate::{
    certification::{CertifiedDataError, validate_certificate_time},
    ic_registry::{
        MainnetRegistryFetchRequest, RegistryFetchError, transport::get_certified_latest_version,
    },
};
use ic_host_text::timestamp::parse_utc_timestamp_secs;

pub async fn fetch_mainnet_registry_version(
    request: &MainnetRegistryFetchRequest,
) -> Result<NnsRegistryVersionData, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let certified = get_certified_latest_version(&agent, &registry_canister).await?;
    validate_live_certificate_time(request, certified.certification.certificate_time_nanos)?;
    Ok(NnsRegistryVersionData {
        provenance: request.inventory_provenance(certified.registry_version),
        certification: certified.certification,
    })
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
mod tests;
