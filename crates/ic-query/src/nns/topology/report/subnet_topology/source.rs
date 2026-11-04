use super::{NnsSubnetTopologyHostError, NnsSubnetTopologyReport};
use crate::report::CollectionSourceRequest;
use crate::{
    ic_registry::fetch_mainnet_subnet_topology,
    nns::{LiveNnsSource, source::mainnet_registry_fetch_request},
};
use ic_host_text::timestamp::format_utc_timestamp_secs;

///
/// NnsSubnetTopologySource
///
/// Source contract for one complete, exact-version Subnet topology report.
///

pub trait NnsSubnetTopologySource: Sync {
    /// Fetch and join a complete topology report at one Registry version.
    fn fetch_subnet_topology_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError>>;
}

impl NnsSubnetTopologySource for LiveNnsSource {
    fn fetch_subnet_topology_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError>>
    {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsSubnetTopologyHostError::UnsupportedNetwork { network }
            })?;
            let report = fetch_mainnet_subnet_topology(&fetch_request).await?;
            report.validate()?;
            Ok(report)
        })
    }
}

pub(super) fn source_request(
    network: &str,
    endpoint: &str,
    now_unix_secs: u64,
) -> CollectionSourceRequest {
    CollectionSourceRequest::new(
        network,
        endpoint,
        format_utc_timestamp_secs(now_unix_secs),
        "ic-query",
    )
}
