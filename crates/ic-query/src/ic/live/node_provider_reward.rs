//! Module: ic::live::node_provider_reward
//!
//! Responsibility: live Dashboard node-provider reward URL and wire handling.
//! Does not own: report validation, CLI parsing, provenance projection, or rendering.
//! Boundary: performs one page, one exact record, or one bounded history request.

use super::{LiveIcSource, append_path_segments, dashboard_base_url, fetch_live};
use crate::ic::{
    IcHostError, IcNodeProviderRewardHistoryObservation, IcNodeProviderRewardHistoryQuery,
    IcNodeProviderRewardHistorySourceData, IcNodeProviderRewardInfoSourceData,
    IcNodeProviderRewardListQuery, IcNodeProviderRewardListSourceData, IcNodeProviderRewardRow,
    IcNodeProviderRewardSource, IcNodeProviderRewardXdrConversionRate, IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use serde_json::Number;
use std::collections::BTreeMap;
use url::Url;

const MAX_EXACT_JSON_FLOAT_INTEGER: u64 = (1_u64 << 53) - 1;

impl IcNodeProviderRewardSource for LiveIcSource {
    fn fetch_node_provider_reward_list(
        &self,
        request: &IcSourceRequest,
        query: &IcNodeProviderRewardListQuery,
    ) -> Result<IcNodeProviderRewardListSourceData, IcHostError> {
        source::validate_node_provider_reward_list_query(query)?;
        let url = node_provider_reward_list_url(&request.endpoint, query)?;
        let wire: DashboardNodeProviderRewardList = fetch_live(url)?;
        wire.into_source_data(request, query)
    }

    fn fetch_node_provider_reward_info(
        &self,
        request: &IcSourceRequest,
        reward_id: u64,
    ) -> Result<IcNodeProviderRewardInfoSourceData, IcHostError> {
        let url = node_provider_reward_info_url(&request.endpoint, reward_id)?;
        let wire: DashboardNodeProviderReward = fetch_live(url)?;
        Ok(IcNodeProviderRewardInfoSourceData {
            source: request.clone(),
            reward: wire.into_public()?,
        })
    }

    fn fetch_node_provider_reward_history(
        &self,
        request: &IcSourceRequest,
        query: &IcNodeProviderRewardHistoryQuery,
    ) -> Result<IcNodeProviderRewardHistorySourceData, IcHostError> {
        source::validate_node_provider_reward_history_query(query)?;
        let url = node_provider_reward_history_url(&request.endpoint, query)?;
        let wire: DashboardNodeProviderRewardHistory = fetch_live(url)?;
        Ok(IcNodeProviderRewardHistorySourceData {
            source: request.clone(),
            query: query.clone(),
            observations: wire
                .reward_node_providers
                .into_iter()
                .map(|[amount, timestamp]| {
                    Ok(IcNodeProviderRewardHistoryObservation {
                        timestamp_unix_secs: number_to_u64("reward history timestamp", timestamp)?,
                        amount_e8s: number_to_u64("reward history amount_e8s", amount)?,
                    })
                })
                .collect::<Result<_, IcHostError>>()?,
        })
    }
}

fn node_provider_reward_list_url(
    endpoint: &str,
    query: &IcNodeProviderRewardListQuery,
) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["reward-node-providers"])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("format", "json");
        pairs.append_pair("limit", &query.limit.to_string());
        pairs.append_pair("offset", &query.offset.to_string());
        if let Some(max_reward_index) = query.max_reward_index {
            pairs.append_pair(
                "max_reward_node_provider_index",
                &max_reward_index.to_string(),
            );
        }
    }
    Ok(url)
}

fn node_provider_reward_info_url(endpoint: &str, reward_id: u64) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(
        endpoint,
        &mut url,
        &["reward-node-providers", &reward_id.to_string()],
    )?;
    Ok(url)
}

fn node_provider_reward_history_url(
    endpoint: &str,
    query: &IcNodeProviderRewardHistoryQuery,
) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["timeseries", "reward-node-providers"])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("format", "json");
        pairs.append_pair("start", &query.start_unix_secs.to_string());
        pairs.append_pair("end", &query.end_unix_secs.to_string());
        pairs.append_pair("step", &query.step_secs.to_string());
    }
    Ok(url)
}

#[derive(SerdeDeserialize)]
struct DashboardNodeProviderRewardList {
    data: Vec<DashboardNodeProviderReward>,
    max_reward_node_provider_index: u64,
    total_reward_node_providers: u64,
}

impl DashboardNodeProviderRewardList {
    fn into_source_data(
        self,
        request: &IcSourceRequest,
        query: &IcNodeProviderRewardListQuery,
    ) -> Result<IcNodeProviderRewardListSourceData, IcHostError> {
        Ok(IcNodeProviderRewardListSourceData {
            source: request.clone(),
            query: query.clone(),
            resolved_max_reward_index: self.max_reward_node_provider_index,
            total_reward_records: self.total_reward_node_providers,
            rows: self
                .data
                .into_iter()
                .map(DashboardNodeProviderReward::into_public)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(SerdeDeserialize)]
struct DashboardNodeProviderReward {
    amount_e8s: u64,
    #[serde(default)]
    details: BTreeMap<String, serde_json::Value>,
    maximum_node_provider_rewards_e8s: Option<u64>,
    minimum_xdr_permyriad_per_icp: Option<u64>,
    node_provider: String,
    proposal_id: Option<Number>,
    registry_version: Option<Number>,
    reward_mode: String,
    id: u64,
    timestamp_seconds: u64,
    updated_at: String,
    #[serde(default)]
    xdr_conversion_rate: DashboardNodeProviderRewardXdrConversionRate,
}

impl DashboardNodeProviderReward {
    fn into_public(self) -> Result<IcNodeProviderRewardRow, IcHostError> {
        Ok(IcNodeProviderRewardRow {
            reward_id: self.id,
            amount_e8s: self.amount_e8s,
            details: self.details,
            maximum_node_provider_rewards_e8s: self.maximum_node_provider_rewards_e8s,
            minimum_xdr_permyriad_per_icp: self.minimum_xdr_permyriad_per_icp,
            node_provider_id: self.node_provider,
            proposal_id: self
                .proposal_id
                .map(|value| number_to_u64("proposal_id", value))
                .transpose()?,
            registry_version: self
                .registry_version
                .map(|value| number_to_u64("registry_version", value))
                .transpose()?,
            reward_mode: self.reward_mode,
            reward_timestamp_unix_secs: self.timestamp_seconds,
            dashboard_updated_at: self.updated_at,
            xdr_conversion_rate: self.xdr_conversion_rate.into_public()?,
        })
    }
}

#[derive(Default, SerdeDeserialize)]
struct DashboardNodeProviderRewardXdrConversionRate {
    timestamp_seconds: Option<Number>,
    xdr_permyriad_per_icp: Option<Number>,
}

impl DashboardNodeProviderRewardXdrConversionRate {
    fn into_public(self) -> Result<IcNodeProviderRewardXdrConversionRate, IcHostError> {
        Ok(IcNodeProviderRewardXdrConversionRate {
            timestamp_unix_secs: self
                .timestamp_seconds
                .map(|value| number_to_u64("xdr timestamp_seconds", value))
                .transpose()?,
            xdr_permyriad_per_icp: self
                .xdr_permyriad_per_icp
                .map(|value| number_to_u64("xdr_permyriad_per_icp", value))
                .transpose()?,
        })
    }
}

#[derive(SerdeDeserialize)]
struct DashboardNodeProviderRewardHistory {
    reward_node_providers: Vec<[Number; 2]>,
}

fn number_to_u64(field: &str, value: Number) -> Result<u64, IcHostError> {
    if let Some(value) = value.as_u64() {
        return Ok(value);
    }
    let raw = value.to_string();
    let Some((integer, fraction)) = raw.split_once('.') else {
        return Err(IcHostError::InvalidSourceData {
            reason: format!("{field} is not an unsigned integer: {raw}"),
        });
    };
    if integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.is_empty()
        || !fraction.bytes().all(|byte| byte == b'0')
    {
        return Err(IcHostError::InvalidSourceData {
            reason: format!("{field} is not an unsigned integer: {raw}"),
        });
    }
    let parsed = integer
        .parse()
        .map_err(|_| IcHostError::InvalidSourceData {
            reason: format!("{field} is outside the u64 range: {raw}"),
        })?;
    if parsed > MAX_EXACT_JSON_FLOAT_INTEGER {
        return Err(IcHostError::InvalidSourceData {
            reason: format!(
                "{field} uses a decimal JSON number above the exact-integer range: {raw}"
            ),
        });
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reward_urls_preserve_explicit_page_and_history_bounds() {
        let list = node_provider_reward_list_url(
            "https://ic-api.internetcomputer.org/api/v3/",
            &IcNodeProviderRewardListQuery::new(25, 50, Some(6_470)),
        )
        .expect("list URL");
        assert_eq!(
            list.as_str(),
            "https://ic-api.internetcomputer.org/api/v3/reward-node-providers?format=json&limit=25&offset=50&max_reward_node_provider_index=6470"
        );

        let history = node_provider_reward_history_url(
            "https://ic-api.internetcomputer.org/api/v3",
            &IcNodeProviderRewardHistoryQuery::new(1_783_900_000, 1_784_300_000, 86_400),
        )
        .expect("history URL");
        assert_eq!(history.path(), "/api/v3/timeseries/reward-node-providers");
        assert_eq!(
            history.query(),
            Some("format=json&start=1783900000&end=1784300000&step=86400")
        );
        assert_eq!(
            node_provider_reward_info_url("https://example.com/api/v3", 7_562)
                .expect("info URL")
                .path(),
            "/api/v3/reward-node-providers/7562"
        );
    }

    #[test]
    fn wire_preserves_current_and_historical_shapes() {
        let current: DashboardNodeProviderReward = serde_json::from_str(
            r#"{
                "amount_e8s": 1583574085000,
                "details": {"to_account":"0000000000000000000000000000000000000000000000000000000000000000"},
                "id": 7562,
                "maximum_node_provider_rewards_e8s": 10000000000000,
                "minimum_xdr_permyriad_per_icp": 20000,
                "node_provider": "rrkah-fqaaa-aaaaa-aaaaq-cai",
                "proposal_id": null,
                "registry_version": null,
                "reward_mode": "RewardToAccount",
                "timestamp_seconds": 1784081341,
                "updated_at": "2026-07-15T04:30:01.558435",
                "xdr_conversion_rate": {"timestamp_seconds":1784073600,"xdr_permyriad_per_icp":16379}
            }"#,
        )
        .expect("current wire");
        let current = current.into_public().expect("current public row");
        assert_eq!(current.reward_id, 7_562);
        assert_eq!(
            current.xdr_conversion_rate.xdr_permyriad_per_icp,
            Some(16_379)
        );

        let historical: DashboardNodeProviderReward = serde_json::from_str(
            r#"{
                "amount_e8s": 1,
                "details": {},
                "id": 707,
                "maximum_node_provider_rewards_e8s": null,
                "minimum_xdr_permyriad_per_icp": null,
                "node_provider": "rrkah-fqaaa-aaaaa-aaaaq-cai",
                "proposal_id": 10270.0,
                "registry_version": null,
                "reward_mode": "RewardToAccount",
                "timestamp_seconds": 1620000000,
                "updated_at": "2021-05-03T00:00:00",
                "xdr_conversion_rate": {}
            }"#,
        )
        .expect("historical wire");
        let historical = historical.into_public().expect("historical public row");
        assert_eq!(historical.proposal_id, Some(10_270));
        assert_eq!(historical.xdr_conversion_rate.timestamp_unix_secs, None);
    }

    #[test]
    fn integer_decoder_rejects_fractional_values() {
        assert_eq!(
            number_to_u64("value", serde_json::from_str("10270.0").expect("number"))
                .expect("integral float"),
            10_270
        );
        assert!(number_to_u64("value", serde_json::from_str("1.5").expect("number")).is_err());
        assert!(
            number_to_u64(
                "value",
                serde_json::from_str("9007199254740993.0").expect("number")
            )
            .is_err()
        );
    }
}
