use super::{
    SubnetCatalogHostError, SubnetCatalogSourceFailure, SubnetCatalogSubject,
    failure::subject_from_catalog_error,
};
use crate::{
    hex::hex_bytes,
    http_endpoint::parse_http_endpoint,
    ic_registry::{
        fetch_mainnet_subnet_catalog_async, fetch_mainnet_subnet_catalog_detailed_async,
    },
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
    subnet_catalog::{
        CatalogAssurance, CatalogValidationContext, MAINNET_REGISTRY_CANISTER_ID,
        MAX_SUBNET_CATALOG_AGREEMENT_ENDPOINTS, MIN_SUBNET_CATALOG_AGREEMENT_ENDPOINTS,
        RawSubnetCatalog, ValidatedSubnetCatalog, catalog_agreement_digest,
    },
};
use std::{collections::BTreeSet, future::Future, pin::Pin};

///
/// SubnetCatalogSourceFuture
///
/// Boxed caller-runtime future returned by a Subnet Catalog source.
///

pub type SubnetCatalogSourceFuture<'a> =
    Pin<Box<dyn Future<Output = Result<RawSubnetCatalog, SubnetCatalogHostError>> + Send + 'a>>;

///
/// SubnetCatalogDetailedSourceFuture
///
/// Boxed caller-runtime future retaining typed source failure provenance.
///

pub type SubnetCatalogDetailedSourceFuture<'a> =
    Pin<Box<dyn Future<Output = Result<RawSubnetCatalog, SubnetCatalogSourceFailure>> + Send + 'a>>;

///
/// CatalogSourceSelection
///
/// Explicit bounded Registry endpoint selection for one live catalog collection.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogSourceSelection {
    /// Collect ordinary version-consistent evidence from one endpoint.
    UncertifiedQuery {
        /// Exact caller-selected endpoint.
        endpoint: String,
    },
    /// Require an identical Registry version and canonical payload across endpoints.
    MultiEndpointAgreement {
        /// Caller-selected endpoints; each must have a distinct hostname.
        endpoints: Vec<String>,
    },
}

impl CatalogSourceSelection {
    /// Build a single-endpoint uncertified query selection.
    #[must_use]
    pub fn uncertified_query(endpoint: impl Into<String>) -> Self {
        Self::UncertifiedQuery {
            endpoint: endpoint.into(),
        }
    }

    /// Build a bounded multi-endpoint agreement selection.
    #[must_use]
    pub const fn multi_endpoint_agreement(endpoints: Vec<String>) -> Self {
        Self::MultiEndpointAgreement { endpoints }
    }

    /// Return the assurance this source selection attempts to establish.
    #[must_use]
    pub const fn assurance(&self) -> CatalogAssurance {
        match self {
            Self::UncertifiedQuery { .. } => CatalogAssurance::UncertifiedQuery,
            Self::MultiEndpointAgreement { .. } => CatalogAssurance::MultiEndpointAgreement,
        }
    }

    #[expect(
        clippy::result_large_err,
        reason = "source selection failures retain the original host error and endpoint"
    )]
    pub(super) fn validated_endpoints(&self) -> Result<Vec<String>, SubnetCatalogSourceFailure> {
        let (endpoints, agreement) = match self {
            Self::UncertifiedQuery { endpoint } => (vec![endpoint.clone()], false),
            Self::MultiEndpointAgreement { endpoints } => (endpoints.clone(), true),
        };
        if agreement
            && !(MIN_SUBNET_CATALOG_AGREEMENT_ENDPOINTS..=MAX_SUBNET_CATALOG_AGREEMENT_ENDPOINTS)
                .contains(&endpoints.len())
        {
            return Err(SubnetCatalogSourceFailure::from_source(
                SubnetCatalogHostError::InvalidSourceSelection {
                    reason: format!(
                        "multi-endpoint agreement requires {MIN_SUBNET_CATALOG_AGREEMENT_ENDPOINTS}..={MAX_SUBNET_CATALOG_AGREEMENT_ENDPOINTS} endpoints"
                    ),
                },
            ));
        }
        let mut hostnames = BTreeSet::new();
        for endpoint in &endpoints {
            let parsed = parse_http_endpoint(endpoint).map_err(|reason| {
                SubnetCatalogSourceFailure::new(
                    None,
                    Some(SubnetCatalogSubject::Endpoint(endpoint.clone())),
                    SubnetCatalogHostError::InvalidSourceSelection {
                        reason: format!("invalid endpoint {endpoint:?}: {reason}"),
                    },
                )
            })?;
            let hostname = parsed
                .host_str()
                .expect("validated HTTP endpoint has a hostname")
                .to_ascii_lowercase();
            if !hostnames.insert(hostname) {
                return Err(SubnetCatalogSourceFailure::new(
                    None,
                    Some(SubnetCatalogSubject::Endpoint(endpoint.clone())),
                    SubnetCatalogHostError::InvalidSourceSelection {
                        reason: "source endpoints must use distinct hostnames".to_string(),
                    },
                ));
            }
        }
        let mut endpoints = endpoints;
        endpoints.sort();
        Ok(endpoints)
    }
}

/// Fetch one live mainnet catalog without creating a Tokio runtime or helper thread.
pub async fn fetch_subnet_catalog_async(
    request: &NnsSourceRequest,
) -> Result<RawSubnetCatalog, SubnetCatalogHostError> {
    let fetch_request = mainnet_registry_fetch_request(request, |network| {
        SubnetCatalogHostError::UnsupportedNetwork { network }
    })?;
    Ok(fetch_mainnet_subnet_catalog_async(&fetch_request).await?)
}

///
/// SubnetCatalogSource
///
/// Source contract for fetching complete subnet catalog snapshots.
///

pub trait SubnetCatalogSource: Send + Sync {
    /// Fetch one complete single-endpoint snapshot on the caller's async runtime.
    fn fetch_catalog<'a>(&'a self, request: &'a NnsSourceRequest) -> SubnetCatalogSourceFuture<'a>;

    /// Fetch a snapshot while retaining typed failure provenance when the source can provide it.
    fn fetch_catalog_detailed<'a>(
        &'a self,
        request: &'a NnsSourceRequest,
    ) -> SubnetCatalogDetailedSourceFuture<'a> {
        let future = self.fetch_catalog(request);
        Box::pin(async move {
            future
                .await
                .map_err(SubnetCatalogSourceFailure::from_source)
        })
    }
}

impl SubnetCatalogSource for LiveNnsSource {
    fn fetch_catalog<'a>(&'a self, request: &'a NnsSourceRequest) -> SubnetCatalogSourceFuture<'a> {
        Box::pin(fetch_subnet_catalog_async(request))
    }

    fn fetch_catalog_detailed<'a>(
        &'a self,
        request: &'a NnsSourceRequest,
    ) -> SubnetCatalogDetailedSourceFuture<'a> {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                SubnetCatalogHostError::UnsupportedNetwork { network }
            })
            .map_err(SubnetCatalogSourceFailure::from_source)?;
            fetch_mainnet_subnet_catalog_detailed_async(&fetch_request)
                .await
                .map_err(|failure| {
                    SubnetCatalogSourceFailure::new(
                        failure.registry_version,
                        failure.subject,
                        SubnetCatalogHostError::RegistryRefresh(failure.source),
                    )
                })
        })
    }
}

pub(super) async fn collect_subnet_catalog_detailed(
    network: &str,
    endpoints: Vec<String>,
    fetched_at: &str,
    fetched_by: &str,
    now_unix_secs: u64,
    max_future_skew_seconds: u64,
    source: &dyn SubnetCatalogSource,
) -> Result<RawSubnetCatalog, SubnetCatalogSourceFailure> {
    let validation = CatalogValidationContext::new(
        network,
        MAINNET_REGISTRY_CANISTER_ID,
        now_unix_secs,
        max_future_skew_seconds,
    );
    let mut snapshots = Vec::with_capacity(endpoints.len());
    for endpoint in &endpoints {
        let request = NnsSourceRequest::new(network, endpoint, fetched_at, fetched_by);
        let raw = source
            .fetch_catalog_detailed(&request)
            .await
            .map_err(|error| endpoint_error(error, endpoint, endpoints.len()))?;
        let registry_version = raw.provenance.registry_version;
        let catalog = ValidatedSubnetCatalog::try_from_raw(raw, &validation)
            .map_err(|source| {
                let subject = subject_from_catalog_error(&source);
                SubnetCatalogSourceFailure::new(
                    Some(registry_version),
                    subject,
                    SubnetCatalogHostError::from(source),
                )
            })
            .map_err(|error| endpoint_error(error, endpoint, endpoints.len()))?;
        if catalog.provenance().assurance != CatalogAssurance::UncertifiedQuery
            || catalog.provenance().source_endpoints.len() != 1
            || catalog.provenance().source_endpoints[0] != *endpoint
        {
            return Err(SubnetCatalogSourceFailure::new(
                Some(catalog.provenance().registry_version),
                Some(SubnetCatalogSubject::Endpoint(endpoint.clone())),
                SubnetCatalogHostError::SourceEvidenceMismatch {
                    requested: endpoint.clone(),
                    actual_assurance: catalog.provenance().assurance,
                    actual_endpoints: catalog.provenance().source_endpoints.clone(),
                },
            ));
        }
        snapshots.push(catalog.into_raw());
    }

    if snapshots.len() == 1 {
        return Ok(snapshots.pop().expect("single endpoint snapshot"));
    }

    finish_agreement(snapshots, endpoints, &validation)
}

#[expect(
    clippy::result_large_err,
    reason = "agreement failures retain the original host error and typed subject"
)]
fn finish_agreement(
    mut snapshots: Vec<RawSubnetCatalog>,
    endpoints: Vec<String>,
    validation: &CatalogValidationContext,
) -> Result<RawSubnetCatalog, SubnetCatalogSourceFailure> {
    let mut first = snapshots.remove(0);
    let reference_endpoint = endpoints[0].clone();
    let reference_registry_version = first.provenance.registry_version;
    let reference_agreement_digest =
        hex_bytes(&catalog_agreement_digest(&first).map_err(|source| {
            let subject = subject_from_catalog_error(&source);
            SubnetCatalogSourceFailure::new(
                Some(reference_registry_version),
                subject,
                SubnetCatalogHostError::Catalog(source),
            )
        })?);
    let mut registry_query_call_count = first.provenance.registry_query_call_count;
    for (endpoint, snapshot) in endpoints.iter().skip(1).zip(&snapshots) {
        let registry_version = snapshot.provenance.registry_version;
        let agreement_digest =
            hex_bytes(&catalog_agreement_digest(snapshot).map_err(|source| {
                let subject = subject_from_catalog_error(&source);
                SubnetCatalogSourceFailure::new(
                    Some(registry_version),
                    subject,
                    SubnetCatalogHostError::Catalog(source),
                )
            })?);
        if registry_version != reference_registry_version
            || agreement_digest != reference_agreement_digest
        {
            return Err(SubnetCatalogSourceFailure::new(
                Some(registry_version),
                Some(SubnetCatalogSubject::Endpoint(endpoint.clone())),
                SubnetCatalogHostError::AgreementMismatch {
                    reference_endpoint,
                    reference_registry_version,
                    reference_agreement_digest,
                    endpoint: endpoint.clone(),
                    registry_version,
                    agreement_digest,
                },
            ));
        }
        registry_query_call_count = registry_query_call_count
            .checked_add(snapshot.provenance.registry_query_call_count)
            .ok_or_else(|| {
                SubnetCatalogSourceFailure::new(
                    Some(reference_registry_version),
                    None,
                    SubnetCatalogHostError::RegistryQueryCallCountOverflow,
                )
            })?;
    }
    first
        .promote_to_multi_endpoint_agreement(endpoints, registry_query_call_count)
        .map_err(|source| {
            let subject = subject_from_catalog_error(&source);
            SubnetCatalogSourceFailure::new(
                Some(reference_registry_version),
                subject,
                SubnetCatalogHostError::Catalog(source),
            )
        })?;
    ValidatedSubnetCatalog::try_from_raw(first.clone(), validation).map_err(|source| {
        let subject = subject_from_catalog_error(&source);
        SubnetCatalogSourceFailure::new(
            Some(reference_registry_version),
            subject,
            SubnetCatalogHostError::Catalog(source),
        )
    })?;
    Ok(first)
}

fn endpoint_error(
    error: SubnetCatalogSourceFailure,
    endpoint: &str,
    endpoint_count: usize,
) -> SubnetCatalogSourceFailure {
    if endpoint_count == 1 {
        error
    } else {
        SubnetCatalogSourceFailure::new(
            error.registry_version,
            Some(SubnetCatalogSubject::Endpoint(endpoint.to_string())),
            SubnetCatalogHostError::AgreementEndpoint {
                endpoint: endpoint.to_string(),
                source: Box::new(error.source),
            },
        )
    }
}
