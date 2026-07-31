//! Module: ic::live
//!
//! Responsibility: live HTTP transport and wire conversion for the official IC Dashboard API.
//! Does not own: report assembly, custom-source validation, command parsing, or rendering.
//! Boundary: performs one read-only REST lookup and retains request provenance.

use crate::http_endpoint::parse_http_endpoint;
use crate::ic::{
    IcCanisterCollectionSource, IcCanisterCountSourceData, IcCanisterFilters,
    IcCanisterPageController, IcCanisterPageRow, IcCanisterPageSourceData, IcCanisterSource,
    IcCanisterSourceData, IcCanisterUpgrade, IcHostError, IcMetricKind, IcMetricObservation,
    IcMetricQuery, IcMetricSeries, IcMetricSource, IcMetricSourceData, IcSourceRequest,
};
use crate::runtime::block_on_current_thread;
use reqwest::Client;
use serde::{Deserialize as SerdeDeserialize, de::DeserializeOwned};
use std::time::Duration;
use url::Url;

const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

///
/// LiveIcSource
///
/// Live official IC Dashboard source used by report builders outside tests.
///

pub struct LiveIcSource;

impl IcMetricSource for LiveIcSource {
    fn fetch_metric(
        &self,
        request: &IcSourceRequest,
        query: &IcMetricQuery,
    ) -> Result<IcMetricSourceData, IcHostError> {
        super::source::validate_metric_query(query)?;
        let url = metric_url(&request.endpoint, query)?;
        let url_text = url.to_string();
        let wire: serde_json::Value = fetch_live(url)?;
        let series =
            decode_metric_series(wire, query.metric).map_err(|reason| IcHostError::JsonDecode {
                url: url_text,
                reason,
            })?;
        Ok(IcMetricSourceData {
            source: request.clone(),
            query: query.clone(),
            series,
        })
    }
}

impl IcCanisterSource for LiveIcSource {
    fn fetch_canister(
        &self,
        request: &IcSourceRequest,
        canister_id: &str,
    ) -> Result<IcCanisterSourceData, IcHostError> {
        let canister_id = super::source::canonical_canister_id(canister_id)?;
        let url = canister_url(&request.endpoint, &canister_id)?;
        let wire = fetch_live(url)?;
        Ok(DashboardCanister::into_source_data(wire, request))
    }
}

impl IcCanisterCollectionSource for LiveIcSource {
    fn fetch_canister_count(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
    ) -> Result<IcCanisterCountSourceData, IcHostError> {
        let filters = super::source::normalized_filters(filters)?;
        let url = canister_collection_url(&request.endpoint, &filters, CollectionOperation::Count)?;
        let wire: DashboardCanisterCount = fetch_live(url)?;
        Ok(IcCanisterCountSourceData {
            source: request.clone(),
            filters,
            total: wire.total,
        })
    }

    fn fetch_canister_page(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
        limit: u16,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<IcCanisterPageSourceData, IcHostError> {
        super::source::validate_page_limit(limit)?;
        if after.is_some() && before.is_some() {
            return Err(IcHostError::InvalidRequest {
                field: "pagination",
                reason: "after and before are mutually exclusive".to_string(),
            });
        }
        let filters = super::source::normalized_filters(filters)?;
        let after = super::source::canonical_page_cursor("after", after)?;
        let before = super::source::canonical_page_cursor("before", before)?;
        let url = canister_collection_url(
            &request.endpoint,
            &filters,
            CollectionOperation::Page {
                limit,
                after: after.as_deref(),
                before: before.as_deref(),
            },
        )?;
        let wire = fetch_live(url)?;
        Ok(DashboardCanisterPage::into_source_data(
            wire,
            request,
            &filters,
            limit,
            after.as_deref(),
            before.as_deref(),
        ))
    }
}

fn fetch_live<T>(url: Url) -> Result<T, IcHostError>
where
    T: DeserializeOwned + Send,
{
    block_on_current_thread(fetch_json(http_client()?, url))?
}

fn http_client() -> Result<Client, IcHostError> {
    Client::builder()
        .user_agent(concat!("ic-query/", env!("CARGO_PKG_VERSION")))
        .timeout(HTTP_TIMEOUT)
        .build()
        .map_err(|error| IcHostError::HttpClientBuild {
            reason: error.to_string(),
        })
}

async fn fetch_json<T>(client: Client, url: Url) -> Result<T, IcHostError>
where
    T: DeserializeOwned,
{
    let url_text = url.to_string();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| IcHostError::HttpRequest {
            url: url_text.clone(),
            reason: error.to_string(),
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(IcHostError::HttpStatus {
            url: url_text,
            status: status.as_u16(),
        });
    }
    response
        .json::<T>()
        .await
        .map_err(|error| IcHostError::JsonDecode {
            url: url_text,
            reason: error.to_string(),
        })
}

fn canister_url(endpoint: &str, canister_id: &str) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["canisters", canister_id])?;
    Ok(url)
}

fn metric_url(endpoint: &str, query: &IcMetricQuery) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &[query.metric.as_str()])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("format", "json");
        pairs.append_pair("start", &query.start_unix_secs.to_string());
        pairs.append_pair("end", &query.end_unix_secs.to_string());
        pairs.append_pair("step", &query.step_secs.to_string());
    }
    Ok(url)
}

fn decode_metric_series(
    value: serde_json::Value,
    metric: IcMetricKind,
) -> Result<Vec<IcMetricSeries>, String> {
    let serde_json::Value::Object(mut object) = value else {
        return Err("expected a JSON object".to_string());
    };
    metric
        .series_names()
        .iter()
        .map(|name| {
            let raw = object
                .remove(*name)
                .ok_or_else(|| format!("missing required series {name:?}"))?;
            let observations: Vec<(u64, String)> = serde_json::from_value(raw)
                .map_err(|error| format!("invalid series {name:?}: {error}"))?;
            Ok(IcMetricSeries {
                name: (*name).to_string(),
                observations: observations
                    .into_iter()
                    .map(|(timestamp_unix_secs, value)| IcMetricObservation {
                        timestamp_unix_secs,
                        value,
                    })
                    .collect(),
            })
        })
        .collect()
}

enum CollectionOperation<'a> {
    Count,
    Page {
        limit: u16,
        after: Option<&'a str>,
        before: Option<&'a str>,
    },
}

fn canister_collection_url(
    endpoint: &str,
    filters: &IcCanisterFilters,
    operation: CollectionOperation<'_>,
) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    let path = match operation {
        CollectionOperation::Count => &["canisters", "count"][..],
        CollectionOperation::Page { .. } => &["canisters"][..],
    };
    append_path_segments(endpoint, &mut url, path)?;

    {
        let mut query = url.query_pairs_mut();
        if let Some(has_name) = filters.has_name {
            query.append_pair("has_name", if has_name { "true" } else { "false" });
        }
        if let Some(subnet_id) = filters.subnet_id.as_deref() {
            query.append_pair("subnet_id", subnet_id);
        }
        if let Some(controller_id) = filters.controller_id.as_deref() {
            query.append_pair("controller_id", controller_id);
        }
        for language in &filters.languages {
            query.append_pair("language", language);
        }
        for canister_type in &filters.canister_types {
            query.append_pair("canister_type", canister_type);
        }
        if let Some(search) = filters.query.as_deref() {
            query.append_pair("query", search);
        }
        if let CollectionOperation::Page {
            limit,
            after,
            before,
        } = operation
        {
            query.append_pair("sort_by", "canister_id");
            query.append_pair("limit", &limit.to_string());
            if let Some(after) = after {
                query.append_pair("after", after);
            }
            if let Some(before) = before {
                query.append_pair("before", before);
            }
        }
    }
    Ok(url)
}

fn dashboard_base_url(endpoint: &str) -> Result<Url, IcHostError> {
    let url = parse_http_endpoint(endpoint).map_err(|reason| invalid_endpoint(endpoint, reason))?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err(invalid_endpoint(
            endpoint,
            "base endpoint must not include a query or fragment",
        ));
    }

    Ok(url)
}

fn append_path_segments(endpoint: &str, url: &mut Url, path: &[&str]) -> Result<(), IcHostError> {
    let mut segments = url
        .path_segments_mut()
        .map_err(|()| invalid_endpoint(endpoint, "base endpoint cannot accept path segments"))?;
    segments.pop_if_empty();
    segments.extend(path.iter().copied());
    Ok(())
}

fn invalid_endpoint(endpoint: &str, reason: impl Into<String>) -> IcHostError {
    IcHostError::InvalidEndpoint {
        endpoint: endpoint.to_string(),
        reason: reason.into(),
    }
}

#[derive(SerdeDeserialize)]
struct DashboardCanister {
    canister_id: String,
    canister_type: Option<String>,
    controllers: Vec<String>,
    id: u64,
    language: String,
    module_hash: String,
    name: String,
    subnet_id: String,
    updated_at: String,
    upgrades: Option<Vec<DashboardCanisterUpgrade>>,
}

impl DashboardCanister {
    fn into_source_data(self, request: &IcSourceRequest) -> IcCanisterSourceData {
        IcCanisterSourceData {
            source: request.clone(),
            canister_id: self.canister_id,
            dashboard_id: self.id,
            canister_type: self.canister_type,
            name: self.name,
            subnet_id: self.subnet_id,
            controllers: self.controllers,
            language: self.language,
            module_hash: self.module_hash,
            dashboard_updated_at: self.updated_at,
            upgrades: self.upgrades.map(|upgrades| {
                upgrades
                    .into_iter()
                    .map(DashboardCanisterUpgrade::into_public)
                    .collect()
            }),
        }
    }
}

#[derive(SerdeDeserialize)]
struct DashboardCanisterCount {
    total: u64,
}

#[derive(SerdeDeserialize)]
struct DashboardCanisterPage {
    data: Vec<DashboardCanisterPageRow>,
    next_cursor: Option<String>,
    previous_cursor: Option<String>,
}

impl DashboardCanisterPage {
    fn into_source_data(
        self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
        requested_limit: u16,
        after: Option<&str>,
        before: Option<&str>,
    ) -> IcCanisterPageSourceData {
        IcCanisterPageSourceData {
            source: request.clone(),
            filters: filters.clone(),
            requested_limit,
            after: after.map(str::to_string),
            before: before.map(str::to_string),
            previous_cursor: self.previous_cursor,
            next_cursor: self.next_cursor,
            rows: self
                .data
                .into_iter()
                .map(DashboardCanisterPageRow::into_public)
                .collect(),
        }
    }
}

#[derive(SerdeDeserialize)]
struct DashboardCanisterPageRow {
    canister_id: String,
    canister_type: Option<String>,
    controllers: Vec<(String, Option<String>)>,
    id: u64,
    language: String,
    module_hash: String,
    name: String,
    subnet_id: String,
    updated_at: String,
}

impl DashboardCanisterPageRow {
    fn into_public(self) -> IcCanisterPageRow {
        IcCanisterPageRow {
            canister_id: self.canister_id,
            dashboard_id: self.id,
            canister_type: self.canister_type,
            name: self.name,
            subnet_id: self.subnet_id,
            controllers: self
                .controllers
                .into_iter()
                .map(|(principal_id, raw_metadata)| IcCanisterPageController {
                    principal_id,
                    raw_metadata,
                })
                .collect(),
            language: self.language,
            module_hash: self.module_hash,
            dashboard_updated_at: self.updated_at,
        }
    }
}

#[derive(SerdeDeserialize)]
struct DashboardCanisterUpgrade {
    executed_timestamp_seconds: u64,
    module_hash: String,
    proposal_id: u64,
}

impl DashboardCanisterUpgrade {
    fn into_public(self) -> IcCanisterUpgrade {
        IcCanisterUpgrade {
            executed_timestamp_seconds: self.executed_timestamp_seconds,
            module_hash: self.module_hash,
            proposal_id: self.proposal_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canister_url_appends_path_to_endpoints_with_or_without_trailing_slash() {
        for endpoint in [
            "https://ic-api.internetcomputer.org/api/v3",
            "https://ic-api.internetcomputer.org/api/v3/",
        ] {
            assert_eq!(
                canister_url(endpoint, "ryjl3-tyaaa-aaaaa-aaaba-cai")
                    .expect("canister URL")
                    .as_str(),
                "https://ic-api.internetcomputer.org/api/v3/canisters/ryjl3-tyaaa-aaaaa-aaaba-cai"
            );
        }
    }

    #[test]
    fn canister_url_rejects_query_and_fragment_components() {
        for endpoint in [
            "https://ic-api.internetcomputer.org/api/v3?limit=1",
            "https://ic-api.internetcomputer.org/api/v3#canisters",
        ] {
            assert!(matches!(
                canister_url(endpoint, "ryjl3-tyaaa-aaaaa-aaaba-cai"),
                Err(IcHostError::InvalidEndpoint { .. })
            ));
        }
    }

    #[test]
    fn metric_url_preserves_explicit_bound_and_step() {
        let query = IcMetricQuery::new(
            IcMetricKind::InstructionRate,
            1_700_000_000,
            1_700_003_600,
            300,
        );
        let url = metric_url("https://metrics-api.internetcomputer.org/api/v1/", &query)
            .expect("metric URL");

        assert_eq!(url.path(), "/api/v1/instruction-rate");
        assert_eq!(
            url.query(),
            Some("format=json&start=1700000000&end=1700003600&step=300")
        );
    }

    #[test]
    fn metric_decoder_preserves_raw_values_and_ignores_additive_fields() {
        let value = serde_json::json!({
            "total_nodes": [[1_700_000_000_u64, "559"]],
            "up_nodes": [[1_700_000_000_u64, "558"]],
            "future_field": true
        });

        let series =
            decode_metric_series(value, IcMetricKind::IcNodeCount).expect("current metric payload");

        assert_eq!(series.len(), 2);
        assert_eq!(series[0].name, "total_nodes");
        assert_eq!(series[0].observations[0].value, "559");
        assert_eq!(series[1].name, "up_nodes");
    }

    #[test]
    fn collection_urls_preserve_official_filters_and_one_bounded_page() {
        let filters = IcCanisterFilters {
            has_name: Some(true),
            subnet_id: Some(
                "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe".to_string(),
            ),
            controller_id: Some("r7inp-6aaaa-aaaaa-aaabq-cai".to_string()),
            languages: vec!["motoko".to_string(), "rust".to_string()],
            canister_types: vec!["ledger".to_string()],
            query: Some("ICP Ledger".to_string()),
        };
        let endpoint = "https://ic-api.internetcomputer.org/api/v4";

        let count = canister_collection_url(endpoint, &filters, CollectionOperation::Count)
            .expect("count URL");
        let page = canister_collection_url(
            endpoint,
            &filters,
            CollectionOperation::Page {
                limit: 25,
                after: Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
                before: None,
            },
        )
        .expect("page URL");

        assert_eq!(count.path(), "/api/v4/canisters/count");
        assert_eq!(
            count
                .query_pairs()
                .filter(|(key, _)| key == "language")
                .map(|(_, value)| value.into_owned())
                .collect::<Vec<_>>(),
            ["motoko", "rust"]
        );
        assert_eq!(page.path(), "/api/v4/canisters");
        assert!(
            page.query_pairs()
                .any(|(key, value)| { key == "sort_by" && value == "canister_id" })
        );
        assert!(
            page.query_pairs()
                .any(|(key, value)| key == "limit" && value == "25")
        );
        assert!(
            page.query_pairs()
                .any(|(key, value)| { key == "after" && value == "ryjl3-tyaaa-aaaaa-aaaba-cai" })
        );
        assert!(!page.query_pairs().any(|(key, _)| key == "before"));
    }

    #[test]
    fn collection_url_rejects_query_and_fragment_components() {
        for endpoint in [
            "https://ic-api.internetcomputer.org/api/v4?limit=1",
            "https://ic-api.internetcomputer.org/api/v4#canisters",
        ] {
            assert!(matches!(
                canister_collection_url(
                    endpoint,
                    &IcCanisterFilters::default(),
                    CollectionOperation::Count,
                ),
                Err(IcHostError::InvalidEndpoint { .. })
            ));
        }
    }

    #[test]
    fn wire_decoder_preserves_null_upgrade_history_and_ignores_additive_fields() {
        let wire: DashboardCanister = serde_json::from_str(
            r#"{
                "canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
                "canister_type": null,
                "controllers": [],
                "id": 1,
                "language": "",
                "module_hash": "",
                "name": "",
                "subnet_id": "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe",
                "updated_at": "2026-07-30T17:47:41.745647",
                "upgrades": null,
                "future_field": true
            }"#,
        )
        .expect("current Dashboard payload");

        assert_eq!(wire.canister_type, None);
        assert!(wire.upgrades.is_none());
    }

    #[test]
    fn page_wire_decoder_preserves_controller_tuple_metadata() {
        let wire: DashboardCanisterPage = serde_json::from_str(
            r#"{
                "data": [{
                    "canister_id": "2223e-iaaaa-aaaac-awyra-cai",
                    "canister_type": null,
                    "controllers": [
                        ["lyt4m-myaaa-aaaac-aadkq-cai", ""],
                        ["r7inp-6aaaa-aaaaa-aaabq-cai", null]
                    ],
                    "id": 918419,
                    "language": "",
                    "module_hash": "",
                    "name": "",
                    "subnet_id": "4zbus-z2bmt-ilreg-xakz4-6tyre-hsqj4-slb4g-zjwqo-snjcc-iqphi-3qe",
                    "updated_at": "2026-07-31T05:13:38.882316",
                    "upgrades": [],
                    "future_field": true
                }],
                "next_cursor": "2223e-iaaaa-aaaac-awyra-cai",
                "previous_cursor": null
            }"#,
        )
        .expect("current Dashboard page payload");
        let source = wire.into_source_data(
            &IcSourceRequest::new("https://example.com/api/v4", "now", "test"),
            &IcCanisterFilters::default(),
            1,
            None,
            None,
        );

        assert_eq!(
            source.rows[0].controllers[0].raw_metadata.as_deref(),
            Some("")
        );
        assert_eq!(source.rows[0].controllers[1].raw_metadata, None);
    }
}
