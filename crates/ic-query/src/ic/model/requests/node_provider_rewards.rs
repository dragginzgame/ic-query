//! Module: ic::model::requests::node_provider_rewards
//!
//! Responsibility: bounded official Dashboard node-provider reward request contracts.
//! Does not own: transport, returned reward records, validation, or rendering.
//! Boundary: captures one explicit page, one exact reward id, or one bounded history window.

use serde::Serialize;

///
/// IcNodeProviderRewardListQuery
///
/// One explicitly bounded official Dashboard node-provider reward page query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeProviderRewardListQuery {
    /// Maximum reward rows requested from the API.
    pub limit: u16,
    /// Zero-based reward-row offset.
    pub offset: u64,
    /// Optional reward-index ceiling returned by an earlier page.
    pub max_reward_index: Option<u64>,
}

impl IcNodeProviderRewardListQuery {
    /// Construct one bounded node-provider reward page query.
    #[must_use]
    pub const fn new(limit: u16, offset: u64, max_reward_index: Option<u64>) -> Self {
        Self {
            limit,
            offset,
            max_reward_index,
        }
    }
}

///
/// IcNodeProviderRewardListRequest
///
/// Request accepted by the bounded official Dashboard node-provider reward list builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeProviderRewardListRequest {
    /// Dashboard API v3 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Explicit page bounds.
    pub query: IcNodeProviderRewardListQuery,
}

impl IcNodeProviderRewardListRequest {
    /// Construct one bounded live Dashboard node-provider reward list request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        query: IcNodeProviderRewardListQuery,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            query,
        }
    }
}

///
/// IcNodeProviderRewardInfoRequest
///
/// Request accepted by the exact official Dashboard node-provider reward builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeProviderRewardInfoRequest {
    /// Dashboard API v3 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Exact Dashboard node-provider reward id.
    pub reward_id: u64,
}

impl IcNodeProviderRewardInfoRequest {
    /// Construct one exact live Dashboard node-provider reward request.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64, reward_id: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            reward_id,
        }
    }
}

///
/// IcNodeProviderRewardHistoryQuery
///
/// One explicitly bounded official Dashboard node-provider reward history query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeProviderRewardHistoryQuery {
    /// Inclusive history start as Unix seconds.
    pub start_unix_secs: u64,
    /// Inclusive history end as Unix seconds.
    pub end_unix_secs: u64,
    /// Requested observation interval in seconds.
    pub step_secs: u32,
}

impl IcNodeProviderRewardHistoryQuery {
    /// Construct one explicit node-provider reward history window.
    #[must_use]
    pub const fn new(start_unix_secs: u64, end_unix_secs: u64, step_secs: u32) -> Self {
        Self {
            start_unix_secs,
            end_unix_secs,
            step_secs,
        }
    }
}

///
/// IcNodeProviderRewardHistoryRequest
///
/// Request accepted by the bounded official Dashboard node-provider reward history builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeProviderRewardHistoryRequest {
    /// Dashboard API v3 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Explicit history bounds.
    pub query: IcNodeProviderRewardHistoryQuery,
}

impl IcNodeProviderRewardHistoryRequest {
    /// Construct one bounded live Dashboard node-provider reward history request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        query: IcNodeProviderRewardHistoryQuery,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            query,
        }
    }
}
