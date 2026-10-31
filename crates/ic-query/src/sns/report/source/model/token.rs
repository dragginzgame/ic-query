//! Module: sns::report::source::model::token
//!
//! Responsibility: source-layer SNS token metadata model.
//! Does not own: ledger transport, amount formatting, or report rendering.
//! Boundary: carries raw token metadata from sources to report builders.

use crate::icrc::{IcrcIndexData, IcrcTokenData};

///
/// MainnetSnsToken
///
/// Source-layer SNS ledger token metadata result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsToken {
    /// Native ledger token metadata shared with ICRC source capabilities.
    pub token: IcrcTokenData,
    /// Ledger discovery outcome, including nonfatal transport failures.
    pub ledger_index: IcrcIndexData,
}
