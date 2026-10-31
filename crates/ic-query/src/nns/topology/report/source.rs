use crate::nns::NnsInventoryListRequest;
use crate::nns::NnsInventoryRefreshReport;

use super::{
    NnsTopologyHostError, NnsTopologyRefreshRequest,
    request::{
        inventory_refresh_request, node_list_request, subnet_catalog_list_request,
        subnet_catalog_refresh_request,
    },
};
use crate::{
    nns::{
        LiveNnsSource,
        data_center::{
            NnsDataCenterListReport, build_nns_data_center_list_report,
            refresh_nns_data_center_report,
        },
        node::{NnsNodeListReport, build_nns_node_list_report, refresh_nns_node_report},
        node_operator::{
            NnsNodeOperatorListReport, build_nns_node_operator_list_report,
            refresh_nns_node_operator_report,
        },
        node_provider::{
            NnsNodeProviderListReport, build_nns_node_provider_list_report,
            refresh_nns_node_provider_report,
        },
    },
    subnet_catalog::{
        SubnetCatalogListReport, SubnetCatalogRefreshReport, build_subnet_catalog_list_report,
        refresh_subnet_catalog,
    },
};

///
/// NnsTopologySource
///
/// Source contract for fetching complete NNS topology component reports.
///

pub trait NnsTopologySource: Sync {
    /// Load a complete Subnet Catalog input under its selected cache policy.
    fn fetch_subnet_catalog_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<SubnetCatalogListReport, NnsTopologyHostError>>;

    /// Load the complete Registry node inventory.
    fn fetch_node_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeListReport, NnsTopologyHostError>>;

    /// Load the complete Registry node-provider inventory.
    fn fetch_node_provider_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderListReport, NnsTopologyHostError>>;

    /// Load the complete Registry node-operator inventory.
    fn fetch_node_operator_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeOperatorListReport, NnsTopologyHostError>>;

    /// Load the complete Registry data-center inventory.
    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsTopologyHostError>>;
}

///
/// NnsTopologyRefreshSource
///
/// Source contract for refreshing complete NNS topology component reports.
///

pub trait NnsTopologyRefreshSource: Sync {
    /// Collect and validate a replacement Subnet Catalog.
    fn refresh_subnet_catalog_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<SubnetCatalogRefreshReport, NnsTopologyHostError>>;

    /// Collect and validate a replacement node inventory.
    fn refresh_node_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>;

    /// Collect and validate a replacement node-provider inventory.
    fn refresh_node_provider_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>;

    /// Collect and validate a replacement node-operator inventory.
    fn refresh_node_operator_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>;

    /// Collect and validate a replacement data-center inventory.
    fn refresh_data_center_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>;
}

impl NnsTopologySource for LiveNnsSource {
    fn fetch_subnet_catalog_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<SubnetCatalogListReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            Ok(build_subnet_catalog_list_report(&subnet_catalog_list_request(request)).await?)
        })
    }

    fn fetch_node_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeListReport, NnsTopologyHostError>> {
        Box::pin(async move { Ok(build_nns_node_list_report(&node_list_request(request)).await?) })
    }

    fn fetch_node_provider_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderListReport, NnsTopologyHostError>>
    {
        Box::pin(async move { Ok(build_nns_node_provider_list_report(request).await?) })
    }

    fn fetch_node_operator_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeOperatorListReport, NnsTopologyHostError>>
    {
        Box::pin(async move { Ok(build_nns_node_operator_list_report(request).await?) })
    }

    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsTopologyHostError>>
    {
        Box::pin(async move { Ok(build_nns_data_center_list_report(request).await?) })
    }
}

impl NnsTopologyRefreshSource for LiveNnsSource {
    fn refresh_subnet_catalog_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<SubnetCatalogRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            Ok(refresh_subnet_catalog(&subnet_catalog_refresh_request(request)).await?)
        })
    }

    fn refresh_node_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(
            async move { Ok(refresh_nns_node_report(&inventory_refresh_request(request)).await?) },
        )
    }

    fn refresh_node_provider_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            Ok(refresh_nns_node_provider_report(&inventory_refresh_request(request)).await?)
        })
    }

    fn refresh_node_operator_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            Ok(refresh_nns_node_operator_report(&inventory_refresh_request(request)).await?)
        })
    }

    fn refresh_data_center_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            Ok(refresh_nns_data_center_report(&inventory_refresh_request(request)).await?)
        })
    }
}
