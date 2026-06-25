//! Module: sns::report::assemble::token
//!
//! Responsibility: assemble SNS token reports.
//! Does not own: ledger metadata fetching, supply lookup, amount formatting, or rendering.
//! Boundary: combines resolved SNS identity and source token data into report output.

use crate::sns::report::{
    MainnetSns, MainnetSnsList, MainnetSnsToken, SNS_TOKEN_REPORT_SCHEMA_VERSION, SnsTokenReport,
};

pub(in crate::sns::report) fn sns_token_report_from_parts(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    token: MainnetSnsToken,
) -> SnsTokenReport {
    SnsTokenReport {
        schema_version: SNS_TOKEN_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        sns_index_canister_id: sns.index_canister_id,
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        ledger_index_canister_id: token.ledger_index_canister_id,
        ledger_index_error: token.ledger_index_error,
        supported_standards: token.supported_standards,
        metadata: token.metadata,
    }
}
