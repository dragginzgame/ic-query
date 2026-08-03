//! Module: ic::model::requests::canisters
//!
//! Responsibility: official Dashboard canister detail, count, and page request contracts.
//! Does not own: network metrics, network resources, transport, or reports.
//! Boundary: keeps shared filters and explicit bounded cursor-page intent together.

use serde::Serialize;

///
/// IcCanisterRequest
///
/// Request accepted by the official Dashboard canister report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterRequest {
    /// Dashboard API base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Canister principal to inspect.
    pub canister_id: String,
}

impl IcCanisterRequest {
    /// Construct a live Dashboard canister request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            canister_id: canister_id.into(),
        }
    }
}

///
/// IcCanisterFilters
///
/// Official Dashboard filters shared by canister count and page requests.
///

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct IcCanisterFilters {
    /// Select canisters according to whether the Dashboard records a name.
    pub has_name: Option<bool>,
    /// Select canisters assigned to this Subnet principal.
    pub subnet_id: Option<String>,
    /// Select canisters controlled by this principal.
    pub controller_id: Option<String>,
    /// Raw Dashboard language labels to include.
    pub languages: Vec<String>,
    /// Raw Dashboard canister classifications to include.
    pub canister_types: Vec<String>,
    /// Raw Dashboard text search, between two and one hundred characters.
    pub query: Option<String>,
}

///
/// IcCanisterCountRequest
///
/// Request for one bounded official Dashboard canister-count lookup.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterCountRequest {
    /// Dashboard API v4 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Filters applied by the Dashboard.
    pub filters: IcCanisterFilters,
}

impl IcCanisterCountRequest {
    /// Construct a live Dashboard canister-count request without filters.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            filters: IcCanisterFilters::default(),
        }
    }

    /// Set the Dashboard filters used by this request.
    #[must_use]
    pub fn with_filters(mut self, filters: IcCanisterFilters) -> Self {
        self.filters = filters;
        self
    }
}

///
/// IcCanisterPageRequest
///
/// Request for one bounded official Dashboard canister page.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterPageRequest {
    /// Dashboard API v4 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Filters applied by the Dashboard.
    pub filters: IcCanisterFilters,
    /// Maximum rows requested from the API.
    pub limit: u16,
    /// Exclusive forward cursor returned by an earlier page.
    pub after: Option<String>,
    /// Exclusive backward cursor returned by an earlier page.
    pub before: Option<String>,
}

impl IcCanisterPageRequest {
    /// Construct a live Dashboard page request with the default bounded limit.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            filters: IcCanisterFilters::default(),
            limit: crate::ic::DEFAULT_IC_CANISTER_PAGE_LIMIT,
            after: None,
            before: None,
        }
    }

    /// Set the Dashboard filters used by this request.
    #[must_use]
    pub fn with_filters(mut self, filters: IcCanisterFilters) -> Self {
        self.filters = filters;
        self
    }

    /// Set the maximum number of returned rows.
    #[must_use]
    pub const fn with_limit(mut self, limit: u16) -> Self {
        self.limit = limit;
        self
    }

    /// Set an exclusive forward cursor.
    #[must_use]
    pub fn with_after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    /// Set an exclusive backward cursor.
    #[must_use]
    pub fn with_before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }
}
