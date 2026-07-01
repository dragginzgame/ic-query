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
pub struct MainnetSnsToken {
    pub token_name: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub transfer_fee: String,
    pub total_supply: String,
    pub minting_account_owner: Option<String>,
    pub minting_account_subaccount_hex: Option<String>,
    pub ledger_index_canister_id: Option<String>,
    pub ledger_index_error: Option<String>,
    pub supported_standards: Vec<SnsTokenStandardRow>,
    pub metadata: Vec<SnsTokenMetadataRow>,
}
