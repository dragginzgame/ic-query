//! Registered feature and resource declarations, without resolving replica defaults.
use crate::{ReadError, mainnet};
use ic_platform::nns_registry_types::protobuf;
use ic_platform_protocol::registry_settings as wire;
use ic_platform_types::Principal;
use serde::Serialize;
use thiserror::Error;

/// Raw external settings DTOs; optional presence is retained without interpretation.
pub use wire::{ResourceLimits, SubnetFeatures, SubnetSettings};

const SETTINGS_LIMITS: wire::Limits = wire::Limits {
    bytes: mainnet::REGISTRY_LIMITS.bytes,
    fields: mainnet::REGISTRY_LIMITS.fields,
};

/// One selected subnet's registered settings and original source evidence.
#[derive(Debug, Serialize)]
pub struct SubnetSettingsReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// Requested identity, bound by the Registry key rather than embedded fields.
    pub subnet_id: Principal,
    /// Raw settings; absence, empty messages and zero/default selections differ.
    pub settings: SubnetSettings,
    /// Evidence for this query only, not an effective-settings or capacity claim.
    pub evidence: SubnetSettingsEvidence,
}

/// Source and assurance for one Registry record.
#[derive(Debug, Serialize)]
pub struct SubnetSettingsEvidence {
    /// Admitted mainnet endpoint.
    pub endpoint: &'static str,
    /// Queried mainnet Registry canister.
    pub source_canister: &'static str,
    /// Registry query method.
    pub source_method: &'static str,
    /// Exact key constructed from the canonical selected subnet ID.
    pub source_key: String,
    /// Local completion time in raw Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Record mutation version; not the latest global Registry version.
    pub record_version: u64,
    /// Raw record mutation timestamp, in Unix nanoseconds.
    pub record_timestamp_nanoseconds: u64,
    /// Agent verified the query response signature using authenticated node keys.
    pub query_signature_verified: bool,
    /// False: signatures do not consensus-certify the Registry value.
    pub result_certified: bool,
    /// False: a configured SEV flag supplies no verified attestation.
    pub tee_attestation_verified: bool,
}

/// Failed reads return no settings report, never fabricated empty settings.
#[derive(Debug, Error)]
pub enum SubnetSettingsError {
    /// Mainnet transport, verification, deadline or local clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The fixed Registry identity could not cross the Agent boundary.
    #[error("invalid Registry identity: {0}")]
    SourceIdentity(String),
    /// Existing bounded Registry request/envelope codec refusal.
    #[error(transparent)]
    Envelope(#[from] protobuf::Error),
    /// Selected nested settings were malformed or exceeded shared limits.
    #[error("Registry settings decoding failed: {0:?}")]
    Settings(wire::Error),
    /// Registry rejection, preserving raw fields without displaying source text.
    #[error("Registry rejected subnet settings (code {code})")]
    RegistryRejected {
        /// Raw source error code.
        code: i32,
        /// Raw reason, requiring escaping before terminal display.
        reason: String,
        /// Raw key bytes from the rejection.
        key: Vec<u8>,
    },
    /// No inline value; a present empty protobuf record is separately valid.
    #[error("Registry returned no subnet settings content")]
    MissingContent,
}

/// Fetch registered settings for one mainnet subnet, with no joins or mutations.
///
/// One `get_value` query under the existing 20-second deadline and 1 MiB HTTP
/// ceiling. Envelope admission keeps 64 KiB/4096-field/128 KiB allocation bounds.
/// The allocation-free selected settings decoder shares a 4096-field budget
/// across nested messages and admits at most 64 KiB. No retries, persistence,
/// effective-default resolution, capacity inference or attestation verification.
/// Requires Tokio IO/time; cancellation drops the pending read.
///
/// # Errors
/// Preserves source/transport/envelope failures and refuses missing inline content
/// or malformed/excessive settings. Zero values and absent nested messages succeed.
pub async fn fetch(subnet_id: Principal) -> Result<SubnetSettingsReport, SubnetSettingsError> {
    let key = source_key(subnet_id);
    let argument = protobuf::encode_get_value_request(key.as_bytes(), None, 256)?;
    let agent = mainnet::agent()?;
    let registry = mainnet::REGISTRY
        .parse()
        .map_err(|error| SubnetSettingsError::SourceIdentity(format!("{error}")))?;
    let bytes = mainnet::read(
        agent
            .query(&registry, mainnet::REGISTRY_METHOD)
            .with_arg(argument)
            .call(),
    )
    .await?;
    project(subnet_id, &bytes, mainnet::observed_at()?)
}

fn source_key(id: Principal) -> String {
    format!("subnet_record_{id}")
}

fn project(
    subnet_id: Principal,
    bytes: &[u8],
    observed_at: u64,
) -> Result<SubnetSettingsReport, SubnetSettingsError> {
    let response = protobuf::decode_get_value_response(bytes, mainnet::REGISTRY_LIMITS)?;
    if let Some(error) = response.error {
        return Err(SubnetSettingsError::RegistryRejected {
            code: error.code,
            reason: error.reason.to_owned(),
            key: error.key.to_vec(),
        });
    }
    let settings = wire::decode_subnet_settings(
        response.value.ok_or(SubnetSettingsError::MissingContent)?,
        SETTINGS_LIMITS,
    )
    .map_err(SubnetSettingsError::Settings)?;
    Ok(SubnetSettingsReport {
        schema: 1,
        subnet_id,
        settings,
        evidence: SubnetSettingsEvidence {
            endpoint: mainnet::ENDPOINT,
            source_canister: mainnet::REGISTRY,
            source_method: mainnet::REGISTRY_METHOD,
            source_key: source_key(subnet_id),
            observed_at_unix_seconds: observed_at,
            record_version: response.version,
            record_timestamp_nanoseconds: response.timestamp_nanoseconds,
            query_signature_verified: true,
            result_certified: false,
            tee_attestation_verified: false,
        },
    })
}

#[cfg(test)]
mod tests;

/// Independent settings acquisition, preserving failures separately from absence.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SettingsOutcome {
    /// Successful raw declarations and source evidence.
    Read {
        /// Original report, retained once.
        report: SubnetSettingsReport,
    },
    /// Unavailable declarations, not empty or default settings.
    Failed {
        /// Original typed failure; JSON uses categories and source error code.
        #[serde(serialize_with = "settings_error")]
        error: SubnetSettingsError,
    },
}
impl SettingsOutcome {
    /// Borrow the acquired report, if available.
    #[must_use]
    pub fn report(&self) -> Option<&SubnetSettingsReport> {
        match self {
            Self::Read { report } => Some(report),
            Self::Failed { .. } => None,
        }
    }
}
impl From<Result<SubnetSettingsReport, SubnetSettingsError>> for SettingsOutcome {
    fn from(value: Result<SubnetSettingsReport, SubnetSettingsError>) -> Self {
        match value {
            Ok(report) => Self::Read { report },
            Err(error) => Self::Failed { error },
        }
    }
}
fn settings_error<S: serde::Serializer>(
    error: &SubnetSettingsError,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeStruct;
    let kind = match error {
        SubnetSettingsError::Read(ReadError::Transport(_)) => "transport",
        SubnetSettingsError::Read(ReadError::Deadline) => "read_deadline",
        SubnetSettingsError::Read(ReadError::Clock(_)) => "clock",
        SubnetSettingsError::SourceIdentity(_) => "source_identity",
        SubnetSettingsError::Envelope(_) => "envelope",
        SubnetSettingsError::Settings(_) => "settings_decode",
        SubnetSettingsError::RegistryRejected { .. } => "registry_rejected",
        SubnetSettingsError::MissingContent => "missing_content",
    };
    let code = match error {
        SubnetSettingsError::RegistryRejected { code, .. } => Some(*code),
        _ => None,
    };
    let mut value = serializer.serialize_struct("SettingsError", 2)?;
    value.serialize_field("kind", kind)?;
    value.serialize_field("registry_code", &code)?;
    value.end()
}
