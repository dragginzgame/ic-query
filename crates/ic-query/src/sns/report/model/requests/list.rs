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
