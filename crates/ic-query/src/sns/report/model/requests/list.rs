//! Module: sns::report::model::requests::list
//!
//! Responsibility: request DTO for deployed SNS list reports.
//! Does not own: command option parsing, source lookup, or text rendering.
//! Boundary: carries validated command inputs into the list report builder.

use crate::sns::report::SnsListSort;

///
/// SnsListRequest
///
/// Request accepted by the deployed SNS list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsListRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub verbose: bool,
    pub sort: SnsListSort,
}

impl SnsListRequest {
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            verbose: false,
            sort: SnsListSort::default(),
        }
    }

    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    #[must_use]
    pub const fn with_sort(mut self, sort: SnsListSort) -> Self {
        self.sort = sort;
        self
    }
}
