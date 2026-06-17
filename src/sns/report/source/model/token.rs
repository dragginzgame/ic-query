//! Module: sns::report::source::model::token
//!
//! Responsibility: source-layer SNS token metadata model.
//! Does not own: ledger transport, amount formatting, or report rendering.
//! Boundary: carries raw token metadata from sources to report builders.

use crate::sns::report::{SnsTokenMetadataRow, SnsTokenStandardRow};

///
/// MainnetSnsToken
///
/// Source-layer SNS ledger token metadata result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsToken {
    pub(in crate::sns::report) token_name: String,
    pub(in crate::sns::report) token_symbol: String,
    pub(in crate::sns::report) decimals: u8,
    pub(in crate::sns::report) transfer_fee: String,
    pub(in crate::sns::report) total_supply: String,
    pub(in crate::sns::report) minting_account_owner: Option<String>,
    pub(in crate::sns::report) minting_account_subaccount_hex: Option<String>,
    pub(in crate::sns::report) ledger_index_canister_id: Option<String>,
    pub(in crate::sns::report) ledger_index_error: Option<String>,
    pub(in crate::sns::report) supported_standards: Vec<SnsTokenStandardRow>,
    pub(in crate::sns::report) metadata: Vec<SnsTokenMetadataRow>,
}
