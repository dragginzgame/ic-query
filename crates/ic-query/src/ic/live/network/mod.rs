//! Module: ic::live::network
//!
//! Responsibility: live Dashboard network-resource URL and wire handling.
//! Does not own: shared HTTP transport, report projection, canisters, or metric series.
//! Boundary: fetches bounded daily statistics or one finite boundary-node resource.

use super::{LiveIcSource, dashboard_resource_url};
use crate::ic::{
    IcBoundaryNodeDataCenterRow, IcBoundaryNodeDataCentersSourceData, IcDailyStatsQuery,
    IcDailyStatsRow, IcDailyStatsSourceData, IcHostError, IcNetworkSource, IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcNetworkSource for LiveIcSource {
    fn fetch_boundary_node_data_centers<'a>(
        &'a self,
        request: &'a IcSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcBoundaryNodeDataCentersSourceData, IcHostError>>
    {
        Box::pin(async move {
            let url = boundary_node_data_centers_url(&request.endpoint)?;
            let wire: DashboardBoundaryNodeDataCenters = self.fetch_live(url).await?;
            Ok(IcBoundaryNodeDataCentersSourceData {
                source: request.clone(),
                rows: wire.data,
            })
        })
    }

    fn fetch_daily_stats<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcDailyStatsQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcDailyStatsSourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_daily_stats_query(query)?;
            let url = daily_stats_url(&request.endpoint, query)?;
            let wire: DashboardDailyStats = self.fetch_live(url).await?;
            Ok(IcDailyStatsSourceData {
                source: request.clone(),
                query: query.clone(),
                rows: wire.daily_stats,
            })
        })
    }
}

fn boundary_node_data_centers_url(endpoint: &str) -> Result<Url, IcHostError> {
    dashboard_resource_url(endpoint, &["boundary-node-data-centers"])
}

fn daily_stats_url(endpoint: &str, query: &IcDailyStatsQuery) -> Result<Url, IcHostError> {
    let mut url = dashboard_resource_url(endpoint, &["daily-stats"])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("format", "json");
        super::append_time_window(&mut pairs, query.start_unix_secs, query.end_unix_secs);
    }
    Ok(url)
}

#[derive(SerdeDeserialize)]
struct DashboardBoundaryNodeDataCenters {
    data: Vec<IcBoundaryNodeDataCenterRow>,
}

#[derive(SerdeDeserialize)]
struct DashboardDailyStats {
    daily_stats: Vec<IcDailyStatsRow>,
}

#[cfg(test)]
mod tests;
