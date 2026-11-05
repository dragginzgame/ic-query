//! Topology combines one Registry graph, independent Governance metadata and Subnet authority.

use super::{
    NnsTopologyHostError, NnsTopologyRefreshRequest,
    request::{subnet_catalog_list_request, subnet_catalog_refresh_request},
};
use crate::{
    nns::{LiveNnsSource, NnsInventoryListRequest, node_provider::NnsNodeProviderSource},
    report::SourceFuture,
    subnet_catalog::{SubnetCatalogListReport, SubnetCatalogRefreshReport},
};

///
/// NnsTopologySource
///
/// Raw inventory capabilities plus the independently governed Subnet Catalog read policy.
///

pub trait NnsTopologySource: NnsNodeProviderSource {
    /// Load the Subnet Catalog under its explicit cache and assurance policy.
    fn fetch_subnet_catalog_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> SourceFuture<'a, Result<SubnetCatalogListReport, NnsTopologyHostError>>;
}

///
/// NnsTopologyRefreshSource
///
/// Raw inventory capabilities plus the independently governed Subnet Catalog refresh policy.
///

pub trait NnsTopologyRefreshSource: NnsNodeProviderSource {
    /// Refresh the Subnet Catalog under the selected publication policy.
    fn refresh_subnet_catalog_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> SourceFuture<'a, Result<SubnetCatalogRefreshReport, NnsTopologyHostError>>;
}

impl NnsTopologySource for LiveNnsSource {
    fn fetch_subnet_catalog_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> SourceFuture<'a, Result<SubnetCatalogListReport, NnsTopologyHostError>> {
        Box::pin(async move {
            Ok(crate::subnet_catalog::build_subnet_catalog_list_report(
                &subnet_catalog_list_request(request),
            )
            .await?)
        })
    }
}
impl NnsTopologyRefreshSource for LiveNnsSource {
    fn refresh_subnet_catalog_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> SourceFuture<'a, Result<SubnetCatalogRefreshReport, NnsTopologyHostError>> {
        Box::pin(async move {
            Ok(
                crate::subnet_catalog::refresh_subnet_catalog(&subnet_catalog_refresh_request(
                    request,
                ))
                .await?,
            )
        })
    }
}
