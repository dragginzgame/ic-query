//! Module: icrc::live::account_transactions::wire
//!
//! Responsibility: query the account-history index method and select its supported response codec.
//! Does not own: index discovery, collection state, public reports, caching, or text rendering.
//! Boundary: shares request and report assembly; generic ICRC and deployed ICP response codecs remain separate.

use ic_platform::icrc_ledger_types::Account as LedgerAccount;
mod generic;
mod icp;
mod json;

use super::collection::AccountTransactionsPage;
use crate::candid_call::query_bytes;
use crate::icrc::model::{
    IcrcAccountRow, IcrcAccountTransactionError, IcrcAccountTransactionRow, IcrcError,
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::{CandidType, Nat, Principal, serde::Deserialize};

const INDEX_ACCOUNT_TRANSACTIONS_METHOD: &str = "get_account_transactions";

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexAccountTransactionsArgs {
    account: LedgerAccount,
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

impl TransactionSummary {
    fn into_row(
        self,
        id: String,
        kind: String,
        timestamp_unix_nanos: Option<String>,
        raw_transaction: serde_json::Value,
    ) -> IcrcAccountTransactionRow {
        IcrcAccountTransactionRow {
            id,
            kind,
            timestamp_unix_nanos,
            amount_base_units: self.amount_base_units,
            fee_base_units: self.fee_base_units,
            from: self.from,
            to: self.to,
            spender: self.spender,
            memo_hex: self.memo_hex,
            created_at_time_unix_nanos: self.created_at_time_unix_nanos,
            expires_at_unix_nanos: self.expires_at_unix_nanos,
            expected_allowance_base_units: self.expected_allowance_base_units,
            raw_transaction,
        }
    }
}

pub(super) async fn query_account_transaction_page(
    agent: &Agent,
    index_canister: &Principal,
    account: &LedgerAccount,
    start: Option<Nat>,
    limit: u32,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    let args = IcrcIndexAccountTransactionsArgs {
        account: *account,
        start,
        max_results: Nat::from(limit),
    };
    let bytes = query_bytes::<IcrcIndexAccountTransactionsArgs, IcrcError>(
        agent,
        index_canister,
        INDEX_ACCOUNT_TRANSACTIONS_METHOD,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_error_reply_preserves_target_and_message_in_both_codecs() {
        // External get_account_transactions reply: (variant { Err = record { message = "index unavailable" } }).
        let bytes = b"\x44\x49\x44\x4c\x02\x6c\x01\xc7\xeb\xc4\xd0\x09\x71\x6b\x01\xc5\xfe\xd2\x01\x00\x01\x01\x00\x11\x69\x6e\x64\x65\x78\x20\x75\x6e\x61\x76\x61\x69\x6c\x61\x62\x6c\x65";
        let index = Principal::anonymous();
        for result in [generic::decode(bytes, &index), icp::decode(bytes, &index)] {
            let result = result.expect("decode external index error reply");
            assert!(matches!(
                result,
                Err(IcrcAccountTransactionError::IndexQuery { index_canister_id, message })
                    if index_canister_id == index.to_text() && message == "index unavailable"
            ));
        }
    }
}
