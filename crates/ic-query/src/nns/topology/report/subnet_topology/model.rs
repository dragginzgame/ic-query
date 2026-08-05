#[cfg(feature = "nns-topology-host")]
use crate::cache_file::JsonCacheReport;
use crate::subnet_catalog::SubnetKind;
use candid::Principal;
use serde::{Deserialize, Serialize};
#[cfg(feature = "nns-topology-host")]
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// NnsSubnetTopologyReport
///
/// Canonical Subnet and node-provider topology observed at one exact Registry version.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsSubnetTopologyReport {
    /// Serialized report schema version.
    pub schema_version: u32,
    /// Network whose Registry supplied the snapshot.
    pub network: String,
    /// Registry canister queried for the snapshot.
    pub registry_canister_id: String,
    /// Exact Registry version shared by every joined record.
    pub registry_version: u64,
    /// UTC timestamp at which collection began.
    pub fetched_at: String,
    /// Replica endpoint used to query the Registry.
    pub source_endpoint: String,
    /// Collector identity recorded for provenance.
    pub fetched_by: String,
    /// Number of Subnet rows in `subnets`.
    pub subnet_count: usize,
    /// Total number of Subnet member nodes.
    pub node_count: u64,
    /// Canonically ordered Subnet topology rows.
    pub subnets: Vec<NnsSubnetTopologyRow>,
}

#[cfg(feature = "nns-topology-host")]
impl JsonCacheReport for NnsSubnetTopologyReport {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

///
/// NnsSubnetTopologyRow
///
/// One Subnet with raw Registry kind and provider membership counts.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsSubnetTopologyRow {
    /// Canonical textual Subnet principal.
    pub subnet_principal: String,
    /// Raw Subnet classification stored in the Registry.
    pub subnet_kind: SubnetKind,
    /// Number of nodes assigned to this Subnet.
    pub node_count: u32,
    /// Canonically ordered node-provider membership counts.
    pub node_providers: Vec<NnsSubnetNodeProviderRow>,
}

///
/// NnsSubnetNodeProviderRow
///
/// Registry-derived node membership count for one provider on one Subnet.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsSubnetNodeProviderRow {
    /// Canonical textual node-provider principal.
    pub node_provider_principal: String,
    /// Number of this provider's nodes assigned to the Subnet.
    pub node_count: u32,
}

///
/// NnsSubnetTopologyFreshness
///
/// Caller-relative freshness facts derived from a cached report timestamp.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsSubnetTopologyFreshness {
    /// Whether the report is stale under the caller's policy.
    pub stale: bool,
    /// Stable machine-readable explanation of the freshness result.
    pub reason: String,
    /// Maximum acceptable report age supplied by the caller.
    pub stale_after_seconds: u64,
    /// Parsed `fetched_at` timestamp, when valid.
    pub fetched_at_unix_secs: Option<u64>,
    /// Report age, when `fetched_at` is valid and not in the future.
    pub age_seconds: Option<u64>,
}

///
/// NnsSubnetTopologyValidationError
///
/// Canonical-shape and relation-count failures in a Subnet topology report.
///

#[derive(Clone, Debug, Eq, PartialEq, ThisError)]
pub enum NnsSubnetTopologyValidationError {
    /// The report schema cannot be read by this library version.
    #[error("unsupported Subnet topology schema version {found}; expected {expected}")]
    UnsupportedSchemaVersion {
        /// Schema version found in the report.
        found: u32,
        /// Schema version supported by this library.
        expected: u32,
    },

    /// The report contains no Subnet rows.
    #[error("Subnet topology report contains no Subnets")]
    EmptySubnets,

    /// A Registry or topology principal is syntactically invalid.
    #[error("invalid principal in {field}: {value}: {reason}")]
    InvalidPrincipal {
        /// Name of the invalid principal field.
        field: &'static str,
        /// Invalid textual principal value.
        value: String,
        /// Principal parser error.
        reason: String,
    },

    /// A valid principal does not use its canonical textual representation.
    #[error("non-canonical principal in {field}: {value}; expected {canonical}")]
    NonCanonicalPrincipal {
        /// Name of the non-canonical principal field.
        field: &'static str,
        /// Non-canonical textual principal.
        value: String,
        /// Canonical textual representation.
        canonical: String,
    },

    /// More than one row names the same Subnet.
    #[error("duplicate Subnet row for {subnet_principal}")]
    DuplicateSubnet {
        /// Duplicated Subnet principal.
        subnet_principal: String,
    },

    /// Subnet rows are not sorted by canonical principal.
    #[error(
        "Subnet rows are not canonically ordered: {previous_subnet_principal} sorts after {subnet_principal}"
    )]
    NonCanonicalSubnetOrder {
        /// Principal in the preceding row.
        previous_subnet_principal: String,
        /// Out-of-order Subnet principal.
        subnet_principal: String,
    },

    /// A provider appears more than once within a Subnet.
    #[error(
        "duplicate node-provider row for {node_provider_principal} on Subnet {subnet_principal}"
    )]
    DuplicateNodeProvider {
        /// Subnet containing the duplicate.
        subnet_principal: String,
        /// Duplicated node-provider principal.
        node_provider_principal: String,
    },

    /// Provider rows within a Subnet are not sorted by canonical principal.
    #[error(
        "node-provider rows on Subnet {subnet_principal} are not canonically ordered: {previous_node_provider_principal} sorts after {node_provider_principal}"
    )]
    NonCanonicalNodeProviderOrder {
        /// Subnet containing the out-of-order rows.
        subnet_principal: String,
        /// Principal in the preceding provider row.
        previous_node_provider_principal: String,
        /// Out-of-order node-provider principal.
        node_provider_principal: String,
    },

    /// A provider row declares no nodes.
    #[error("node provider {node_provider_principal} on Subnet {subnet_principal} has zero nodes")]
    ZeroNodeProviderCount {
        /// Subnet containing the zero-count row.
        subnet_principal: String,
        /// Provider whose count is zero.
        node_provider_principal: String,
    },

    /// Provider node counts do not sum to the Subnet node count.
    #[error(
        "provider node counts on Subnet {subnet_principal} sum to {provider_node_count}, but the Subnet declares {subnet_node_count} nodes"
    )]
    SubnetNodeCountMismatch {
        /// Subnet whose counts disagree.
        subnet_principal: String,
        /// Node count declared by the Subnet row.
        subnet_node_count: u32,
        /// Sum of the Subnet's provider node counts.
        provider_node_count: u64,
    },

    /// The declared Subnet count differs from the row count.
    #[error("report declares {declared} Subnets but contains {actual} rows")]
    SubnetCountMismatch {
        /// Count declared in report metadata.
        declared: usize,
        /// Number of Subnet rows present.
        actual: usize,
    },

    /// The declared total node count differs from the Subnet-row total.
    #[error("report declares {declared} nodes but Subnet rows contain {actual}")]
    NodeCountMismatch {
        /// Count declared in report metadata.
        declared: u64,
        /// Sum of node counts in the Subnet rows.
        actual: u64,
    },
}

impl NnsSubnetTopologyReport {
    /// Validate schema, canonical ordering, principal syntax, uniqueness, and counts.
    pub fn validate(&self) -> Result<(), NnsSubnetTopologyValidationError> {
        if self.schema_version != super::NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION {
            return Err(NnsSubnetTopologyValidationError::UnsupportedSchemaVersion {
                found: self.schema_version,
                expected: super::NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
            });
        }
        if self.subnets.is_empty() {
            return Err(NnsSubnetTopologyValidationError::EmptySubnets);
        }
        validate_principal(&self.registry_canister_id, "registry_canister_id")?;

        let mut previous_subnet: Option<&str> = None;
        let mut actual_node_count = 0_u64;
        for subnet in &self.subnets {
            validate_principal(&subnet.subnet_principal, "subnet_principal")?;
            if let Some(previous) = previous_subnet {
                if previous == subnet.subnet_principal {
                    return Err(NnsSubnetTopologyValidationError::DuplicateSubnet {
                        subnet_principal: subnet.subnet_principal.clone(),
                    });
                }
                if previous > subnet.subnet_principal.as_str() {
                    return Err(NnsSubnetTopologyValidationError::NonCanonicalSubnetOrder {
                        previous_subnet_principal: previous.to_string(),
                        subnet_principal: subnet.subnet_principal.clone(),
                    });
                }
            }
            validate_node_providers(subnet)?;
            actual_node_count = actual_node_count.saturating_add(u64::from(subnet.node_count));
            previous_subnet = Some(subnet.subnet_principal.as_str());
        }

        if self.subnet_count != self.subnets.len() {
            return Err(NnsSubnetTopologyValidationError::SubnetCountMismatch {
                declared: self.subnet_count,
                actual: self.subnets.len(),
            });
        }
        if self.node_count != actual_node_count {
            return Err(NnsSubnetTopologyValidationError::NodeCountMismatch {
                declared: self.node_count,
                actual: actual_node_count,
            });
        }
        Ok(())
    }
}

fn validate_node_providers(
    subnet: &NnsSubnetTopologyRow,
) -> Result<(), NnsSubnetTopologyValidationError> {
    let mut previous_provider: Option<&str> = None;
    let mut provider_node_count = 0_u64;
    for provider in &subnet.node_providers {
        validate_principal(&provider.node_provider_principal, "node_provider_principal")?;
        if provider.node_count == 0 {
            return Err(NnsSubnetTopologyValidationError::ZeroNodeProviderCount {
                subnet_principal: subnet.subnet_principal.clone(),
                node_provider_principal: provider.node_provider_principal.clone(),
            });
        }
        if let Some(previous) = previous_provider {
            if previous == provider.node_provider_principal {
                return Err(NnsSubnetTopologyValidationError::DuplicateNodeProvider {
                    subnet_principal: subnet.subnet_principal.clone(),
                    node_provider_principal: provider.node_provider_principal.clone(),
                });
            }
            if previous > provider.node_provider_principal.as_str() {
                return Err(
                    NnsSubnetTopologyValidationError::NonCanonicalNodeProviderOrder {
                        subnet_principal: subnet.subnet_principal.clone(),
                        previous_node_provider_principal: previous.to_string(),
                        node_provider_principal: provider.node_provider_principal.clone(),
                    },
                );
            }
        }
        provider_node_count = provider_node_count.saturating_add(u64::from(provider.node_count));
        previous_provider = Some(provider.node_provider_principal.as_str());
    }
    if provider_node_count != u64::from(subnet.node_count) {
        return Err(NnsSubnetTopologyValidationError::SubnetNodeCountMismatch {
            subnet_principal: subnet.subnet_principal.clone(),
            subnet_node_count: subnet.node_count,
            provider_node_count,
        });
    }
    Ok(())
}

fn validate_principal(
    value: &str,
    field: &'static str,
) -> Result<(), NnsSubnetTopologyValidationError> {
    let principal = Principal::from_text(value).map_err(|err| {
        NnsSubnetTopologyValidationError::InvalidPrincipal {
            field,
            value: value.to_string(),
            reason: err.to_string(),
        }
    })?;
    let canonical = principal.to_text();
    if canonical != value {
        return Err(NnsSubnetTopologyValidationError::NonCanonicalPrincipal {
            field,
            value: value.to_string(),
            canonical,
        });
    }
    Ok(())
}

///
/// NnsSubnetTopologyCacheRequest
///
/// Cache-root and network identity for a joined Subnet topology cache.
///

#[cfg(feature = "nns-topology-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsSubnetTopologyCacheRequest {
    /// Root directory containing the shared cache.
    pub cache_root: PathBuf,
    /// Network cache namespace.
    pub network: String,
}

#[cfg(feature = "nns-topology-host")]
impl NnsSubnetTopologyCacheRequest {
    /// Create a cache request for a cache root and network.
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
        }
    }
}

///
/// NnsSubnetTopologyRefreshRequest
///
/// Inputs for one explicit live refresh and atomic cache publication.
///

#[cfg(feature = "nns-topology-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsSubnetTopologyRefreshRequest {
    /// Cache identity and destination.
    pub cache: NnsSubnetTopologyCacheRequest,
    /// Replica endpoint used for the live Registry query.
    pub source_endpoint: String,
    /// Current Unix timestamp used for provenance and lock policy.
    pub now_unix_secs: u64,
    /// Age after which an existing refresh lock is reported as stale.
    pub lock_stale_after_seconds: u64,
}

#[cfg(feature = "nns-topology-host")]
impl NnsSubnetTopologyRefreshRequest {
    /// Create an explicit refresh request.
    #[must_use]
    pub fn new(
        cache: NnsSubnetTopologyCacheRequest,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            cache,
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            lock_stale_after_seconds,
        }
    }
}

///
/// CachedNnsSubnetTopologyReport
///
/// Validated Subnet topology report paired with its shared cache path.
///

#[cfg(feature = "nns-topology-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedNnsSubnetTopologyReport {
    /// Canonical path from which the report was loaded or to which it was published.
    pub path: PathBuf,
    /// Validated joined topology report.
    pub report: NnsSubnetTopologyReport,
}
