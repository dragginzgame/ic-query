use crate::{
    cache_file::{CacheFileError, HostCacheError},
    ic_registry::RegistryFetchError,
    network::enforce_mainnet_network_with,
    subnet_catalog::CatalogError,
};
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// SubnetCatalogErrorCode
///
/// Stable machine-readable code for a Subnet Catalog host failure.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogErrorCode {
    /// The requested network is unsupported.
    UnsupportedNetwork,
    /// The cache-only operation found no catalog.
    MissingCatalog,
    /// Caller supplied an incompatible cache read policy.
    InvalidReadPolicy,
    /// A shared cache or lock operation failed.
    CacheOperation,
    /// Live Registry collection failed.
    RegistryRefresh,
    /// Raw catalog content failed deterministic validation.
    CatalogValidation,
}

impl SubnetCatalogErrorCode {
    /// Return the stable snake-case code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedNetwork => "unsupported_network",
            Self::MissingCatalog => "missing_catalog",
            Self::InvalidReadPolicy => "invalid_read_policy",
            Self::CacheOperation => "cache_operation",
            Self::RegistryRefresh => "registry_refresh",
            Self::CatalogValidation => "catalog_validation",
        }
    }
}

///
/// SubnetCatalogErrorCategory
///
/// Stable operational category for a Subnet Catalog host failure.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogErrorCategory {
    /// Invalid caller input or policy.
    Input,
    /// Missing local evidence.
    Missing,
    /// Filesystem or refresh-lock failure.
    CacheIo,
    /// Managed cache path confinement or permission failure.
    Confinement,
    /// Network or remote Registry source failure.
    Network,
    /// Evidence identity or assurance failure.
    Authority,
    /// Deterministic raw catalog validation failure.
    Validation,
}

impl SubnetCatalogErrorCategory {
    /// Return the stable snake-case category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Missing => "missing",
            Self::CacheIo => "cache_io",
            Self::Confinement => "confinement",
            Self::Network => "network",
            Self::Authority => "authority",
            Self::Validation => "validation",
        }
    }
}

///
/// SubnetCatalogRetryability
///
/// Whether retrying without changing local inputs can reasonably succeed.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogRetryability {
    /// A later retry may succeed without changing the request.
    Retryable,
    /// The caller must change policy, input, or local evidence first.
    NotRetryable,
}

///
/// SubnetCatalogRemediation
///
/// Structured remediation that a caller may render in its own vocabulary.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogRemediation {
    /// Select the canonical mainnet `ic` network.
    UseMainnet,
    /// Explicitly refresh the Subnet Catalog.
    RefreshCatalog,
}

///
/// SubnetCatalogHostError
///
/// Errors returned by host-backed subnet catalog loading and refresh operations.
///

#[derive(Debug, ThisError)]
pub enum SubnetCatalogHostError {
    #[error("unsupported Subnet Catalog network {network:?}; expected mainnet identity \"ic\"")]
    UnsupportedNetwork { network: String },

    #[error("subnet catalog cache is missing at {}", path.display())]
    MissingCatalog { path: PathBuf },

    /// The requested cache operation received an incompatible read policy.
    #[error("invalid subnet catalog read policy: {reason}")]
    InvalidReadPolicy {
        /// Deterministic policy failure.
        reason: String,
    },

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error("live NNS registry refresh failed: {0}")]
    RegistryRefresh(#[from] RegistryFetchError),

    #[error(transparent)]
    Catalog(#[from] CatalogError),
}

impl SubnetCatalogHostError {
    /// Return the stable machine-readable error code.
    #[must_use]
    pub const fn code(&self) -> SubnetCatalogErrorCode {
        match self {
            Self::UnsupportedNetwork { .. } => SubnetCatalogErrorCode::UnsupportedNetwork,
            Self::MissingCatalog { .. } => SubnetCatalogErrorCode::MissingCatalog,
            Self::InvalidReadPolicy { .. } => SubnetCatalogErrorCode::InvalidReadPolicy,
            Self::Cache(_) => SubnetCatalogErrorCode::CacheOperation,
            Self::RegistryRefresh(_) => SubnetCatalogErrorCode::RegistryRefresh,
            Self::Catalog(_) => SubnetCatalogErrorCode::CatalogValidation,
        }
    }

    /// Return the stable operational error category.
    #[must_use]
    pub const fn category(&self) -> SubnetCatalogErrorCategory {
        match self {
            Self::UnsupportedNetwork { .. } | Self::InvalidReadPolicy { .. } => {
                SubnetCatalogErrorCategory::Input
            }
            Self::MissingCatalog { .. } => SubnetCatalogErrorCategory::Missing,
            Self::Cache(HostCacheError::Operation {
                source:
                    CacheFileError::UnsupportedConfinementPlatform { .. }
                    | CacheFileError::Confinement { .. }
                    | CacheFileError::UnsafeManagedPermissions { .. },
                ..
            }) => SubnetCatalogErrorCategory::Confinement,
            Self::Cache(_) => SubnetCatalogErrorCategory::CacheIo,
            Self::RegistryRefresh(_) => SubnetCatalogErrorCategory::Network,
            Self::Catalog(
                CatalogError::NetworkMismatch { .. }
                | CatalogError::RegistryCanisterMismatch { .. }
                | CatalogError::UnsupportedAssurance { .. }
                | CatalogError::ClassificationPolicyVersionMismatch { .. }
                | CatalogError::ClassificationPolicyDigestMismatch { .. }
                | CatalogError::ResolverPolicyMismatch { .. }
                | CatalogError::CatalogDigestMismatch { .. },
            ) => SubnetCatalogErrorCategory::Authority,
            Self::Catalog(_) => SubnetCatalogErrorCategory::Validation,
        }
    }

    /// Return whether an unchanged retry may reasonably succeed.
    #[must_use]
    pub const fn retryability(&self) -> SubnetCatalogRetryability {
        match self {
            Self::RegistryRefresh(_) => SubnetCatalogRetryability::Retryable,
            _ => SubnetCatalogRetryability::NotRetryable,
        }
    }

    /// Return structured remediation when one action is unambiguous.
    #[must_use]
    pub const fn remediation(&self) -> Option<SubnetCatalogRemediation> {
        match self {
            Self::UnsupportedNetwork { .. } => Some(SubnetCatalogRemediation::UseMainnet),
            Self::MissingCatalog { .. } => Some(SubnetCatalogRemediation::RefreshCatalog),
            _ => None,
        }
    }
}

pub(super) fn enforce_mainnet_network(network: &str) -> Result<(), SubnetCatalogHostError> {
    enforce_mainnet_network_with(network, |network| {
        SubnetCatalogHostError::UnsupportedNetwork { network }
    })
}

pub(super) fn subnet_cache_error(err: CacheFileError) -> SubnetCatalogHostError {
    HostCacheError::operation("subnet catalog", err).into()
}
