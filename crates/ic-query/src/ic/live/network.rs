//! Module: ic::live::network
//!
//! Responsibility: live Dashboard network-resource URL and wire handling.
//! Does not own: shared HTTP transport, report projection, canisters, or metric series.
//! Boundary: fetches bounded daily statistics or one finite boundary-node resource.

use super::{LiveIcSource, append_path_segments, dashboard_base_url, fetch_live};
use crate::ic::{
    IcBoundaryNodeDataCenterRow, IcBoundaryNodeDataCentersSourceData, IcDailyStatsQuery,
    IcDailyStatsRow, IcDailyStatsSourceData, IcHostError, IcNetworkSource, IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcNetworkSource for LiveIcSource {
    fn fetch_boundary_node_data_centers(
        &self,
        request: &IcSourceRequest,
    ) -> Result<IcBoundaryNodeDataCentersSourceData, IcHostError> {
        let url = boundary_node_data_centers_url(&request.endpoint)?;
        let wire: DashboardBoundaryNodeDataCenters = fetch_live(url)?;
        Ok(IcBoundaryNodeDataCentersSourceData {
            source: request.clone(),
            rows: wire
                .data
                .into_iter()
                .map(DashboardBoundaryNodeDataCenter::into_public)
                .collect(),
        })
    }

    fn fetch_daily_stats(
        &self,
        request: &IcSourceRequest,
        query: &IcDailyStatsQuery,
    ) -> Result<IcDailyStatsSourceData, IcHostError> {
        source::validate_daily_stats_query(query)?;
        let url = daily_stats_url(&request.endpoint, query)?;
        let wire: DashboardDailyStats = fetch_live(url)?;
        Ok(IcDailyStatsSourceData {
            source: request.clone(),
            query: query.clone(),
            rows: wire
                .daily_stats
                .into_iter()
                .map(DashboardDailyStatsRow::into_public)
                .collect(),
        })
    }
}

fn boundary_node_data_centers_url(endpoint: &str) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["boundary-node-data-centers"])?;
    Ok(url)
}

fn daily_stats_url(endpoint: &str, query: &IcDailyStatsQuery) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["daily-stats"])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("format", "json");
        pairs.append_pair("start", &query.start_unix_secs.to_string());
        pairs.append_pair("end", &query.end_unix_secs.to_string());
    }
    Ok(url)
}

#[derive(SerdeDeserialize)]
struct DashboardBoundaryNodeDataCenters {
    data: Vec<DashboardBoundaryNodeDataCenter>,
}

#[derive(SerdeDeserialize)]
struct DashboardBoundaryNodeDataCenter {
    dc_id: String,
    latitude: String,
    longitude: String,
    name: String,
    owner: String,
    region: String,
    total_nodes: String,
}

impl DashboardBoundaryNodeDataCenter {
    fn into_public(self) -> IcBoundaryNodeDataCenterRow {
        IcBoundaryNodeDataCenterRow {
            dc_id: self.dc_id,
            name: self.name,
            owner: self.owner,
            region: self.region,
            latitude: self.latitude,
            longitude: self.longitude,
            total_nodes: self.total_nodes,
        }
    }
}

#[derive(SerdeDeserialize)]
struct DashboardDailyStats {
    daily_stats: Vec<DashboardDailyStatsRow>,
}

#[derive(SerdeDeserialize)]
struct DashboardDailyStatsRow {
    average_query_transactions_per_second: String,
    average_transactions_per_second: String,
    average_update_transactions_per_second: String,
    blocks_per_second_average: String,
    day: String,
    max_query_transactions_per_second: String,
    max_total_transactions_per_second: String,
    max_update_transactions_per_second: String,
    timestamp: u64,
}

impl DashboardDailyStatsRow {
    fn into_public(self) -> IcDailyStatsRow {
        IcDailyStatsRow {
            day: self.day,
            timestamp_unix_secs: self.timestamp,
            average_query_transactions_per_second: self.average_query_transactions_per_second,
            average_update_transactions_per_second: self.average_update_transactions_per_second,
            average_transactions_per_second: self.average_transactions_per_second,
            max_query_transactions_per_second: self.max_query_transactions_per_second,
            max_update_transactions_per_second: self.max_update_transactions_per_second,
            max_total_transactions_per_second: self.max_total_transactions_per_second,
            blocks_per_second_average: self.blocks_per_second_average,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_node_data_centers_url_appends_one_v4_resource() {
        for endpoint in [
            "https://ic-api.internetcomputer.org/api/v4",
            "https://ic-api.internetcomputer.org/api/v4/",
        ] {
            assert_eq!(
                boundary_node_data_centers_url(endpoint)
                    .expect("boundary-node data-centers URL")
                    .as_str(),
                "https://ic-api.internetcomputer.org/api/v4/boundary-node-data-centers"
            );
        }
    }

    #[test]
    fn daily_stats_url_preserves_explicit_bounds() {
        let query = IcDailyStatsQuery::new(1_700_000_000, 1_700_604_800);
        let url = daily_stats_url("https://ic-api.internetcomputer.org/api/v3/", &query)
            .expect("daily-statistics URL");

        assert_eq!(url.path(), "/api/v3/daily-stats");
        assert_eq!(
            url.query(),
            Some("format=json&start=1700000000&end=1700604800")
        );
    }

    #[test]
    fn daily_stats_wire_decoder_preserves_raw_rates_and_additive_fields() {
        let wire: DashboardDailyStats = serde_json::from_str(
            r#"{
                "daily_stats": [{
                    "average_query_transactions_per_second": "3057.0771",
                    "average_transactions_per_second": "4378.980149999999",
                    "average_update_transactions_per_second": "1321.9030499999997",
                    "blocks_per_second_average": "193.50055560014323",
                    "day": "2026-07-31",
                    "max_query_transactions_per_second": "3635.62381",
                    "max_total_transactions_per_second": "5062.08807",
                    "max_update_transactions_per_second": "1688.4959999999999",
                    "timestamp": 1785542399,
                    "future_field": {"value": 1.5}
                }],
                "future_top_level": true
            }"#,
        )
        .expect("current daily-statistics payload");

        let row = wire
            .daily_stats
            .into_iter()
            .next()
            .expect("one row")
            .into_public();
        assert_eq!(row.day, "2026-07-31");
        assert_eq!(row.timestamp_unix_secs, 1_785_542_399);
        assert_eq!(row.average_transactions_per_second, "4378.980149999999");
    }

    #[test]
    fn boundary_node_wire_decoder_preserves_zero_counts_and_additive_fields() {
        let wire: DashboardBoundaryNodeDataCenters = serde_json::from_str(
            r#"{
                "data": [{
                    "dc_id": "fr1",
                    "latitude": "50.1109",
                    "longitude": "8.6821",
                    "name": "Frankfurt",
                    "owner": "Equinix",
                    "region": "North America,US,Frankfurt",
                    "total_nodes": "0",
                    "future_field": true
                }],
                "future_top_level": true
            }"#,
        )
        .expect("current boundary-node payload");

        let row = wire.data.into_iter().next().expect("one row").into_public();
        assert_eq!(row.dc_id, "fr1");
        assert_eq!(row.region, "North America,US,Frankfurt");
        assert_eq!(row.total_nodes, "0");
    }
}
