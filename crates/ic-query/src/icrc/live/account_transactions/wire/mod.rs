//! Module: icrc::live::account_transactions::wire
//!
//! Responsibility: query the account-history index method and select its supported response codec.
//! Does not own: index discovery, collection state, public reports, caching, or text rendering.
//! Boundary: shares only the request envelope; generic ICRC and deployed ICP responses remain separate.

mod generic;
mod icp;

use super::collection::AccountTransactionsPage;
use crate::{
    hex::hex_bytes,
    icrc::{
        ledger::{IcrcAccount, query_ledger_arg_bytes},
        model::{IcrcAccountRow, IcrcAccountTransactionError, IcrcError},
    },
};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_agent::Agent;
use serde_json::{Map as JsonMap, Value as JsonValue};

const INDEX_ACCOUNT_TRANSACTIONS_METHOD: &str = "get_account_transactions";

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexAccountTransactionsArgs {
    account: IcrcAccount,
    start: Option<Nat>,
    max_results: Nat,
}

#[derive(Default)]
struct TransactionSummary {
    amount_base_units: Option<String>,
    fee_base_units: Option<String>,
    from: Option<IcrcAccountRow>,
    to: Option<IcrcAccountRow>,
    spender: Option<IcrcAccountRow>,
    memo_hex: Option<String>,
    created_at_time_unix_nanos: Option<String>,
    expires_at_unix_nanos: Option<String>,
    expected_allowance_base_units: Option<String>,
}

pub(super) async fn query_account_transaction_page(
    agent: &Agent,
    index_canister: &Principal,
    account: &IcrcAccount,
    start: Option<Nat>,
    limit: u32,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    let args = IcrcIndexAccountTransactionsArgs {
        account: account.clone(),
        start,
        max_results: Nat::from(limit),
    };
    let bytes = query_ledger_arg_bytes::<IcrcIndexAccountTransactionsArgs, IcrcError>(
        agent,
        index_canister,
        INDEX_ACCOUNT_TRANSACTIONS_METHOD,
        &args,
    )
    .await?;
    decode_account_transactions(&bytes, index_canister)
}

fn decode_account_transactions(
    bytes: &[u8],
    index_canister: &Principal,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    match generic::decode(bytes, index_canister) {
        Ok(result) => result,
        Err(generic_error) => match icp::decode(bytes, index_canister) {
            Ok(result) => result,
            Err(icp_error) => Err(IcrcError::CandidDecode {
                message: INDEX_ACCOUNT_TRANSACTIONS_METHOD,
                reason: format!("generic ICRC index: {generic_error}; ICP index: {icp_error}"),
            }
            .into()),
        },
    }
}

fn optional_blob_json(value: Option<&[u8]>) -> JsonValue {
    optional_json(value.map(|value| JsonValue::String(hex_bytes(value))))
}

fn optional_json(value: Option<JsonValue>) -> JsonValue {
    value.unwrap_or(JsonValue::Null)
}

fn object<const N: usize>(entries: [(&str, JsonValue); N]) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<JsonMap<_, _>>(),
    )
}
