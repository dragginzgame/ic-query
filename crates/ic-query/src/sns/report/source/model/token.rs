//! Module: sns::report::source::model::token
//!
//! Responsibility: source-layer SNS token metadata model.
//! Does not own: ledger transport, amount formatting, or report rendering.
//! Boundary: carries raw token metadata from sources to report builders.

use crate::icrc::IcrcTokenData;

///
/// MainnetSnsToken
///
/// Source-layer SNS ledger token metadata result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsToken {
    /// Native ledger token metadata shared with ICRC source capabilities.
    pub token: IcrcTokenData,
    /// Ledger-discovered index, independently retained from SNS discovery.
    pub ledger_index_canister_id: Option<String>,
    /// Discovery failure retained without discarding successfully fetched metadata.
    pub ledger_index_error: Option<String>,
}
