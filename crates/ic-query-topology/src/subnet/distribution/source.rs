//! Fixed-mainnet source and bounded relationship admission.

use ic_platform::native::ReadClient;
use ic_platform_protocol::registry as protobuf;
use ic_platform_types::Principal;

use super::{
    DataCenterReference, DataCenterReport, DistributionError, OperatorReport, ProviderReference,
    RecordEvidence,
};
use crate::mainnet;

pub(super) struct Observation {
    pub bytes: Vec<u8>,
    pub observed_at: u64,
}

pub(super) trait Source {
    async fn latest(&self) -> Result<Observation, DistributionError>;
    async fn value(&self, key: &str, version: u64) -> Result<Observation, DistributionError>;
}

///
/// MainnetSource
/// Reads the mainnet Registry version and records for the distribution report.
///

pub(super) struct MainnetSource(pub ReadClient);

impl MainnetSource {
    async fn query(
        &self,
        method: &str,
        argument: Vec<u8>,
    ) -> Result<Observation, DistributionError> {
        let registry = mainnet::REGISTRY
            .parse()
            .map_err(|error| DistributionError::SourceIdentity(format!("{error}")))?;
        let bytes = mainnet::query(&self.0, registry, method, &argument).await?;

        Ok(Observation {
            bytes,
            observed_at: mainnet::observed_at()?,
        })
    }
}

impl Source for MainnetSource {
    async fn latest(&self) -> Result<Observation, DistributionError> {
        self.query("get_latest_version", Vec::new()).await
    }

    async fn value(&self, key: &str, version: u64) -> Result<Observation, DistributionError> {
        // Includes the 256-byte data-center identity, prefix and full-width version.
        let argument = protobuf::encode_get_value_request(key.as_bytes(), Some(version), 512)?;

        self.query(mainnet::REGISTRY_METHOD, argument).await
    }
}

pub(super) fn envelope(
    bytes: &[u8],
    version: u64,
) -> Result<protobuf::GetValueResponse<'_>, DistributionError> {
    let response = protobuf::decode_get_value_response(bytes, mainnet::REGISTRY_LIMITS)?;
    if let Some(error) = &response.error {
        return Err(DistributionError::RegistryRejected {
            code: error.code,
            reason: error.reason.to_owned(),
            key: error.key.to_vec(),
        });
    }
    if response.version > version {
        return Err(DistributionError::VersionSkew {
            requested: version,
            returned: response.version,
        });
    }
    if response.value.is_none() {
        return Err(DistributionError::MissingContent);
    }

    Ok(response)
}

fn evidence(response: &protobuf::GetValueResponse<'_>, observed_at: u64) -> RecordEvidence {
    RecordEvidence {
        source_method: mainnet::REGISTRY_METHOD,
        record_version: response.version,
        record_timestamp_nanoseconds: response.timestamp_nanoseconds,
        observed_at_unix_seconds: observed_at,
    }
}

pub(super) fn data_center_identity(raw: &str) -> Option<String> {
    if raw.is_empty() || raw.len() > 256 || raw.trim() != raw {
        return None;
    }
    let identity = raw.to_lowercase();

    (identity.len() <= 256).then_some(identity)
}

pub(super) fn operator(
    id: &Principal,
    response: &protobuf::GetValueResponse<'_>,
    observed_at: u64,
) -> Result<OperatorReport, DistributionError> {
    let record = protobuf::decode_node_operator_record(
        response.value.ok_or(DistributionError::MissingContent)?,
        mainnet::REGISTRY_LIMITS,
    )?;
    if !record.node_operator_principal_id.is_empty()
        && Principal::try_from_slice(record.node_operator_principal_id)
            .ok()
            .as_ref()
            != Some(id)
    {
        return Err(DistributionError::IdentityMismatch);
    }
    let raw_provider = record.node_provider_principal_id;
    let provider = ProviderReference {
        raw: (raw_provider.len() <= 29).then(|| raw_provider.to_vec()),
        byte_length: raw_provider.len(),
        identity: if raw_provider.is_empty() {
            None
        } else {
            Principal::try_from_slice(raw_provider).ok()
        },
    };
    let data_center = DataCenterReference {
        raw: (record.dc_id.len() <= 256).then(|| record.dc_id.to_owned()),
        byte_length: record.dc_id.len(),
        identity: data_center_identity(record.dc_id),
    };

    Ok(OperatorReport {
        registered_operator_bytes: record.node_operator_principal_id.to_vec(),
        provider,
        data_center,
        evidence: evidence(response, observed_at),
    })
}

pub(super) fn data_center(
    id: &str,
    response: &protobuf::GetValueResponse<'_>,
    observed_at: u64,
) -> Result<DataCenterReport, DistributionError> {
    let record = protobuf::decode_data_center_record(
        response.value.ok_or(DistributionError::MissingContent)?,
        mainnet::REGISTRY_LIMITS,
    )?;
    if record.id.len() > 256 || record.region.len() > 256 || record.owner.len() > 256 {
        return Err(DistributionError::TextLimit);
    }
    if data_center_identity(record.id).as_deref() != Some(id) {
        return Err(DistributionError::IdentityMismatch);
    }

    Ok(DataCenterReport {
        id: record.id.to_owned(),
        region: record.region.to_owned(),
        owner: record.owner.to_owned(),
        evidence: evidence(response, observed_at),
    })
}
