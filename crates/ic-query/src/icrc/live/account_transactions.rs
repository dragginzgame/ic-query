//! Module: icrc::live::account_transactions
//!
//! Responsibility: resolve one ICRC index and query page or complete account history.
//! Does not own: public report assembly, CLI parsing, caching, or text rendering.
//! Boundary: validates ledger/index identity once per operation and projects responses losslessly.

use super::fetch::{
    account_from_parts, live_query_context, query_index_principal, query_token_display_fields,
};
use crate::{
    QueryProgress, QueryProgressEvent, QueryProgressState,
    hex::hex_bytes,
    icrc::{
        ledger::{
            GetIndexPrincipalResult, IcrcAccount, index_principal_error_text, principal_from_text,
            query_ledger, query_ledger_arg_bytes,
        },
        model::{
            IcrcAccountRow, IcrcAccountTransactionCollectionData, IcrcAccountTransactionError,
            IcrcAccountTransactionPageData, IcrcAccountTransactionPageRequest,
            IcrcAccountTransactionRefreshRequest, IcrcAccountTransactionRow, IcrcError,
        },
    },
};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_agent::Agent;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::{cmp::Ordering, str::FromStr};

const INDEX_LEDGER_ID_METHOD: &str = "ledger_id";
const INDEX_ACCOUNT_TRANSACTIONS_METHOD: &str = "get_account_transactions";

pub(super) async fn fetch_account_transaction_page_async(
    request: &IcrcAccountTransactionPageRequest,
) -> Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError> {
    let context = resolve_account_transaction_context(
        &request.source_endpoint,
        &request.ledger_canister_id,
        request.index_canister_id.as_deref(),
        &request.account_owner,
        request.subaccount_hex.as_deref(),
    )
    .await?;
    let page =
        query_account_transaction_page(&context, request.start.as_deref(), request.limit).await?;

    Ok(IcrcAccountTransactionPageData {
        index_canister_id: context.index_canister.to_text(),
        balance: page.balance,
        oldest_transaction_id: page.oldest_transaction_id,
        next_start: page.next_start,
        token_symbol: context.token_symbol,
        decimals: context.decimals,
        transactions: page.transactions,
    })
}

pub(super) async fn fetch_complete_account_transactions_async(
    request: &IcrcAccountTransactionRefreshRequest,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError> {
    let context = resolve_account_transaction_context(
        &request.cache.source_endpoint,
        &request.cache.ledger_canister_id,
        request.index_canister_id.as_deref(),
        &request.cache.account_owner,
        request.cache.subaccount_hex.as_deref(),
    )
    .await?;
    let mut state = AccountTransactionCollectionState::new(context.index_canister.to_text());

    report_collection_progress(progress, &state, QueryProgressState::Running);
    loop {
        if request
            .max_pages
            .is_some_and(|max_pages| state.page_count >= max_pages)
        {
            let error = state.incomplete("max pages reached before index exhaustion");
            report_collection_progress(progress, &state, QueryProgressState::Stopped);
            return Err(error);
        }

        let page = match query_account_transaction_page(
            &context,
            state.next_cursor.as_deref(),
            request.page_size,
        )
        .await
        {
            Ok(page) => page,
            Err(source) => {
                let error = state.page_error(source);
                report_collection_progress(progress, &state, QueryProgressState::Failed);
                return Err(error);
            }
        };
        let exhausted = match state.ingest(page, request.page_size) {
            Ok(exhausted) => exhausted,
            Err(error) => {
                report_collection_progress(progress, &state, QueryProgressState::Failed);
                return Err(error);
            }
        };
        report_collection_progress(progress, &state, QueryProgressState::Running);
        if exhausted {
            break;
        }
    }
    report_collection_progress(progress, &state, QueryProgressState::Complete);

    state.into_complete(context.token_symbol, context.decimals)
}

struct AccountTransactionQueryContext {
    agent: Agent,
    index_canister: Principal,
    account: IcrcAccount,
    token_symbol: String,
    decimals: u8,
}

async fn resolve_account_transaction_context(
    source_endpoint: &str,
    ledger_canister_id: &str,
    index_canister_id: Option<&str>,
    account_owner: &str,
    subaccount_hex: Option<&str>,
) -> Result<AccountTransactionQueryContext, IcrcAccountTransactionError> {
    let (agent, ledger_canister) = live_query_context(source_endpoint, ledger_canister_id)?;
    let index_canister = match index_canister_id {
        Some(index_canister_id) => {
            principal_from_text::<IcrcError>(index_canister_id, "index_canister_id")?
        }
        None => resolve_index_canister(&agent, &ledger_canister).await?,
    };
    let (actual_ledger, token_display) = futures::try_join!(
        query_ledger::<Principal, IcrcError>(&agent, &index_canister, INDEX_LEDGER_ID_METHOD),
        query_token_display_fields(&agent, &ledger_canister),
    )?;
    if actual_ledger != ledger_canister {
        return Err(IcrcAccountTransactionError::IndexLedgerMismatch {
            index_canister_id: index_canister.to_text(),
            expected_ledger_canister_id: ledger_canister.to_text(),
            actual_ledger_canister_id: actual_ledger.to_text(),
        });
    }
    let account = account_from_parts(account_owner, subaccount_hex, "account_owner")?;
    let (token_symbol, decimals) = token_display;

    Ok(AccountTransactionQueryContext {
        agent,
        index_canister,
        account,
        token_symbol,
        decimals,
    })
}

async fn query_account_transaction_page(
    context: &AccountTransactionQueryContext,
    start: Option<&str>,
    limit: u32,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    let args = IcrcIndexAccountTransactionsArgs {
        account: context.account.clone(),
        start: start.map(parse_transaction_cursor).transpose()?,
        max_results: Nat::from(limit),
    };
    let result = query_ledger_arg_bytes::<IcrcIndexAccountTransactionsArgs, IcrcError>(
        &context.agent,
        &context.index_canister,
        INDEX_ACCOUNT_TRANSACTIONS_METHOD,
        &args,
    )
    .await?;
    decode_account_transactions(&result, &context.index_canister)
}

pub(in crate::icrc) fn normalize_transaction_cursor(
    value: &str,
) -> Result<String, IcrcAccountTransactionError> {
    parse_transaction_cursor(value).map(|cursor| nat_text(&cursor))
}

fn parse_transaction_cursor(value: &str) -> Result<Nat, IcrcAccountTransactionError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(IcrcAccountTransactionError::InvalidCursor {
            value: value.to_string(),
            reason: "expected unsigned decimal text".to_string(),
        });
    }
    Nat::from_str(value).map_err(|error| IcrcAccountTransactionError::InvalidCursor {
        value: value.to_string(),
        reason: error.to_string(),
    })
}

struct AccountTransactionCollectionState {
    index_canister_id: String,
    balance: Option<String>,
    oldest_transaction_id: Option<String>,
    oldest_transaction_id_initialized: bool,
    transactions: Vec<IcrcAccountTransactionRow>,
    page_count: u32,
    next_cursor: Option<String>,
}

impl AccountTransactionCollectionState {
    const fn new(index_canister_id: String) -> Self {
        Self {
            index_canister_id,
            balance: None,
            oldest_transaction_id: None,
            oldest_transaction_id_initialized: false,
            transactions: Vec::new(),
            page_count: 0,
            next_cursor: None,
        }
    }

    fn ingest(
        &mut self,
        page: AccountTransactionsPage,
        page_size: u32,
    ) -> Result<bool, IcrcAccountTransactionError> {
        let page_len = page.transactions.len();
        if page_len > usize::try_from(page_size).unwrap_or(usize::MAX) {
            return Err(self.incomplete(format!(
                "index returned {page_len} transactions for page size {page_size}"
            )));
        }
        if !self.oldest_transaction_id_initialized {
            self.balance = Some(page.balance);
            self.oldest_transaction_id = page.oldest_transaction_id.clone();
            self.oldest_transaction_id_initialized = true;
        } else if self.oldest_transaction_id != page.oldest_transaction_id {
            return Err(self.incomplete("index oldest transaction id changed during collection"));
        }
        if page_len > 0 && self.oldest_transaction_id.is_none() {
            return Err(
                self.incomplete("index returned transactions without an oldest transaction id")
            );
        }

        for transaction in page.transactions {
            let normalized = normalize_transaction_cursor(&transaction.id)
                .map_err(|error| self.incomplete(error.to_string()))?;
            if normalized != transaction.id {
                return Err(self.incomplete("index returned a non-canonical transaction id"));
            }
            self.transactions.push(transaction);
        }
        self.page_count = self.page_count.saturating_add(1);

        if let Some(next_cursor) = page.next_start.as_deref() {
            let next = parse_transaction_cursor(next_cursor)
                .map_err(|error| self.incomplete(error.to_string()))?;
            if nat_text(&next) != next_cursor {
                return Err(self.incomplete("index returned a non-canonical transaction cursor"));
            }
            if let Some(previous_cursor) = self.next_cursor.as_deref()
                && next
                    >= parse_transaction_cursor(previous_cursor)
                        .map_err(|error| self.incomplete(error.to_string()))?
            {
                return Err(self.incomplete("index cursor did not move toward older transactions"));
            }
        }
        self.next_cursor = page.next_start;

        let exhausted =
            self.next_cursor.is_none() || self.next_cursor == self.oldest_transaction_id;
        if !exhausted && page_len == 0 {
            return Err(
                self.incomplete("index returned no transactions while advertising another cursor")
            );
        }
        Ok(exhausted)
    }

    fn incomplete(&self, reason: impl Into<String>) -> IcrcAccountTransactionError {
        IcrcAccountTransactionError::IncompleteCollection {
            index_canister_id: Some(self.index_canister_id.clone()),
            pages_fetched: self.page_count,
            rows_fetched: self.transactions.len(),
            last_cursor: self.next_cursor.clone(),
            reason: reason.into(),
        }
    }

    fn page_error(&self, source: IcrcAccountTransactionError) -> IcrcAccountTransactionError {
        IcrcAccountTransactionError::CollectionPage {
            index_canister_id: Some(self.index_canister_id.clone()),
            pages_fetched: self.page_count,
            rows_fetched: self.transactions.len(),
            last_cursor: self.next_cursor.clone(),
            source: Box::new(source),
        }
    }

    fn into_complete(
        mut self,
        token_symbol: String,
        decimals: u8,
    ) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError> {
        self.transactions
            .sort_unstable_by(|left, right| compare_canonical_decimal(&right.id, &left.id));
        if self
            .transactions
            .windows(2)
            .any(|rows| rows[0].id == rows[1].id)
        {
            return Err(self.incomplete("index returned a duplicate transaction id"));
        }
        Ok(IcrcAccountTransactionCollectionData {
            index_canister_id: self.index_canister_id,
            balance: self.balance.unwrap_or_else(|| "0".to_string()),
            token_symbol,
            decimals,
            transactions: self.transactions,
            page_count: self.page_count,
            last_cursor: self.next_cursor,
        })
    }
}

fn compare_canonical_decimal(left: &str, right: &str) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn report_collection_progress(
    progress: &mut dyn QueryProgress,
    state: &AccountTransactionCollectionState,
    status: QueryProgressState,
) {
    progress.report(QueryProgressEvent::PagedRefresh {
        text: format!(
            "refreshing ICRC account transactions: pages={} rows={}",
            state.page_count,
            state.transactions.len()
        ),
        state: status,
    });
}

async fn resolve_index_canister(
    agent: &ic_agent::Agent,
    ledger_canister: &Principal,
) -> Result<Principal, IcrcAccountTransactionError> {
    let result = query_index_principal(agent, ledger_canister)
        .await
        .map_err(|source| IcrcAccountTransactionError::IndexDiscovery {
            ledger_canister_id: ledger_canister.to_text(),
            source,
        })?;
    match result {
        GetIndexPrincipalResult::Ok(index_canister) => Ok(index_canister),
        GetIndexPrincipalResult::Err(error) => Err(IcrcAccountTransactionError::IndexUnavailable {
            ledger_canister_id: ledger_canister.to_text(),
            reason: index_principal_error_text(error),
        }),
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexAccountTransactionsArgs {
    account: IcrcAccount,
    start: Option<Nat>,
    max_results: Nat,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum IcrcIndexTransactionsResult {
    Ok(IcrcIndexTransactions),
    Err(IcrcIndexTransactionsError),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransactions {
    balance: Nat,
    transactions: Vec<IcrcIndexTransactionWithId>,
    oldest_tx_id: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransactionsError {
    message: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransactionWithId {
    id: Nat,
    transaction: IcrcIndexTransaction,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransaction {
    burn: Option<IcrcIndexBurn>,
    kind: String,
    mint: Option<IcrcIndexMint>,
    approve: Option<IcrcIndexApprove>,
    fee_collector: Option<IcrcIndexFeeCollector>,
    authorized_mint: Option<IcrcIndexAuthorizedMint>,
    authorized_burn: Option<IcrcIndexAuthorizedBurn>,
    timestamp: u64,
    transfer: Option<IcrcIndexTransfer>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexApprove {
    fee: Option<Nat>,
    from: IcrcAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    expected_allowance: Option<Nat>,
    expires_at: Option<u64>,
    spender: IcrcAccount,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexBurn {
    from: IcrcAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    spender: Option<IcrcAccount>,
    fee: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexMint {
    to: IcrcAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    fee: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransfer {
    to: IcrcAccount,
    fee: Option<Nat>,
    from: IcrcAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    spender: Option<IcrcAccount>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexFeeCollector {
    caller: Option<Principal>,
    fee_collector: Option<IcrcAccount>,
    ts: Option<u64>,
    mthd: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexAuthorizedMint {
    to: IcrcAccount,
    amount: Nat,
    created_at_time: Option<u64>,
    caller: Option<Principal>,
    mthd: Option<String>,
    reason: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexAuthorizedBurn {
    from: IcrcAccount,
    amount: Nat,
    created_at_time: Option<u64>,
    caller: Option<Principal>,
    mthd: Option<String>,
    reason: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum IcpIndexTransactionsResult {
    Ok(IcpIndexTransactions),
    Err(IcrcIndexTransactionsError),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTransactions {
    balance: u64,
    transactions: Vec<IcpIndexTransactionWithId>,
    oldest_tx_id: Option<u64>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTransactionWithId {
    id: u64,
    transaction: IcpIndexTransaction,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTransaction {
    memo: u64,
    icrc1_memo: Option<Vec<u8>>,
    operation: IcpIndexOperation,
    created_at_time: Option<IcpIndexTimestamp>,
    timestamp: Option<IcpIndexTimestamp>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum IcpIndexOperation {
    Approve {
        fee: IcpIndexTokens,
        from: String,
        allowance: IcpIndexTokens,
        expires_at: Option<IcpIndexTimestamp>,
        spender: String,
        expected_allowance: Option<IcpIndexTokens>,
    },
    Burn {
        from: String,
        amount: IcpIndexTokens,
        spender: Option<String>,
    },
    Mint {
        to: String,
        amount: IcpIndexTokens,
    },
    Transfer {
        to: String,
        fee: IcpIndexTokens,
        from: String,
        amount: IcpIndexTokens,
        spender: Option<String>,
    },
}

#[derive(CandidType, Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTimestamp {
    timestamp_nanos: u64,
}

#[derive(CandidType, Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTokens {
    e8s: u64,
}

struct AccountTransactionsPage {
    balance: String,
    oldest_transaction_id: Option<String>,
    next_start: Option<String>,
    transactions: Vec<IcrcAccountTransactionRow>,
}

fn decode_account_transactions(
    bytes: &[u8],
    index_canister: &Principal,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    match candid::decode_one::<IcrcIndexTransactionsResult>(bytes) {
        Ok(result) => generic_account_transactions_page(result, index_canister),
        Err(generic_error) => match candid::decode_one::<IcpIndexTransactionsResult>(bytes) {
            Ok(result) => icp_account_transactions_page(result, index_canister),
            Err(icp_error) => Err(IcrcError::CandidDecode {
                message: INDEX_ACCOUNT_TRANSACTIONS_METHOD,
                reason: format!("generic ICRC index: {generic_error}; ICP index: {icp_error}"),
            }
            .into()),
        },
    }
}

fn generic_account_transactions_page(
    result: IcrcIndexTransactionsResult,
    index_canister: &Principal,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    let transactions = match result {
        IcrcIndexTransactionsResult::Ok(transactions) => transactions,
        IcrcIndexTransactionsResult::Err(error) => {
            return Err(IcrcAccountTransactionError::IndexQuery {
                index_canister_id: index_canister.to_text(),
                message: error.message,
            });
        }
    };
    Ok(AccountTransactionsPage {
        balance: nat_text(&transactions.balance),
        oldest_transaction_id: transactions.oldest_tx_id.as_ref().map(nat_text),
        next_start: transactions
            .transactions
            .last()
            .map(|transaction| nat_text(&transaction.id)),
        transactions: transactions
            .transactions
            .into_iter()
            .map(account_transaction_row)
            .collect(),
    })
}

fn icp_account_transactions_page(
    result: IcpIndexTransactionsResult,
    index_canister: &Principal,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    let transactions = match result {
        IcpIndexTransactionsResult::Ok(transactions) => transactions,
        IcpIndexTransactionsResult::Err(error) => {
            return Err(IcrcAccountTransactionError::IndexQuery {
                index_canister_id: index_canister.to_text(),
                message: error.message,
            });
        }
    };
    Ok(AccountTransactionsPage {
        balance: transactions.balance.to_string(),
        oldest_transaction_id: transactions.oldest_tx_id.map(|id| id.to_string()),
        next_start: transactions
            .transactions
            .last()
            .map(|transaction| transaction.id.to_string()),
        transactions: transactions
            .transactions
            .into_iter()
            .map(icp_account_transaction_row)
            .collect(),
    })
}

fn account_transaction_row(transaction: IcrcIndexTransactionWithId) -> IcrcAccountTransactionRow {
    let summary = transaction_summary(&transaction.transaction);
    IcrcAccountTransactionRow {
        id: nat_text(&transaction.id),
        kind: transaction.transaction.kind.clone(),
        timestamp_unix_nanos: Some(transaction.transaction.timestamp.to_string()),
        amount_base_units: summary.amount_base_units,
        fee_base_units: summary.fee_base_units,
        from: summary.from,
        to: summary.to,
        spender: summary.spender,
        memo_hex: summary.memo_hex,
        created_at_time_unix_nanos: summary.created_at_time_unix_nanos,
        expires_at_unix_nanos: summary.expires_at_unix_nanos,
        expected_allowance_base_units: summary.expected_allowance_base_units,
        raw_transaction: transaction_json(&transaction.transaction),
    }
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

fn transaction_summary(transaction: &IcrcIndexTransaction) -> TransactionSummary {
    if let Some(transfer) = transaction.transfer.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&transfer.amount)),
            fee_base_units: transfer.fee.as_ref().map(nat_text),
            from: Some(account_row(&transfer.from)),
            to: Some(account_row(&transfer.to)),
            spender: transfer.spender.as_ref().map(account_row),
            memo_hex: transfer.memo.as_deref().map(hex_bytes),
            created_at_time_unix_nanos: transfer.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    if let Some(mint) = transaction.mint.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&mint.amount)),
            fee_base_units: mint.fee.as_ref().map(nat_text),
            to: Some(account_row(&mint.to)),
            memo_hex: mint.memo.as_deref().map(hex_bytes),
            created_at_time_unix_nanos: mint.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    if let Some(burn) = transaction.burn.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&burn.amount)),
            fee_base_units: burn.fee.as_ref().map(nat_text),
            from: Some(account_row(&burn.from)),
            spender: burn.spender.as_ref().map(account_row),
            memo_hex: burn.memo.as_deref().map(hex_bytes),
            created_at_time_unix_nanos: burn.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    if let Some(approve) = transaction.approve.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&approve.amount)),
            fee_base_units: approve.fee.as_ref().map(nat_text),
            from: Some(account_row(&approve.from)),
            spender: Some(account_row(&approve.spender)),
            memo_hex: approve.memo.as_deref().map(hex_bytes),
            created_at_time_unix_nanos: approve.created_at_time.map(|time| time.to_string()),
            expires_at_unix_nanos: approve.expires_at.map(|time| time.to_string()),
            expected_allowance_base_units: approve.expected_allowance.as_ref().map(nat_text),
            ..TransactionSummary::default()
        };
    }
    if let Some(mint) = transaction.authorized_mint.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&mint.amount)),
            to: Some(account_row(&mint.to)),
            created_at_time_unix_nanos: mint.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    if let Some(burn) = transaction.authorized_burn.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&burn.amount)),
            from: Some(account_row(&burn.from)),
            created_at_time_unix_nanos: burn.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    TransactionSummary::default()
}

fn account_row(account: &IcrcAccount) -> IcrcAccountRow {
    IcrcAccountRow {
        owner: Some(account.owner.to_text()),
        subaccount_hex: account.subaccount.as_deref().map(hex_bytes),
        account_identifier: None,
    }
}

fn icp_account_transaction_row(
    transaction: IcpIndexTransactionWithId,
) -> IcrcAccountTransactionRow {
    let (kind, mut summary, operation) = icp_operation_parts(&transaction.transaction.operation);
    summary.memo_hex = transaction.transaction.icrc1_memo.as_deref().map(hex_bytes);
    summary.created_at_time_unix_nanos = transaction
        .transaction
        .created_at_time
        .map(|timestamp| timestamp.timestamp_nanos.to_string());
    IcrcAccountTransactionRow {
        id: transaction.id.to_string(),
        kind: kind.to_string(),
        timestamp_unix_nanos: transaction
            .transaction
            .timestamp
            .map(|timestamp| timestamp.timestamp_nanos.to_string()),
        amount_base_units: summary.amount_base_units,
        fee_base_units: summary.fee_base_units,
        from: summary.from,
        to: summary.to,
        spender: summary.spender,
        memo_hex: summary.memo_hex,
        created_at_time_unix_nanos: summary.created_at_time_unix_nanos,
        expires_at_unix_nanos: summary.expires_at_unix_nanos,
        expected_allowance_base_units: summary.expected_allowance_base_units,
        raw_transaction: object([
            (
                "memo",
                JsonValue::String(transaction.transaction.memo.to_string()),
            ),
            (
                "icrc1_memo_hex",
                optional_blob_json(transaction.transaction.icrc1_memo.as_deref()),
            ),
            ("operation", operation),
            (
                "created_at_time_unix_nanos",
                optional_timestamp_json(transaction.transaction.created_at_time),
            ),
            (
                "timestamp_unix_nanos",
                optional_timestamp_json(transaction.transaction.timestamp),
            ),
        ]),
    }
}

fn icp_operation_parts(
    operation: &IcpIndexOperation,
) -> (&'static str, TransactionSummary, JsonValue) {
    match operation {
        IcpIndexOperation::Approve {
            fee,
            from,
            allowance,
            expires_at,
            spender,
            expected_allowance,
        } => icp_approve_parts(
            *fee,
            from,
            *allowance,
            *expires_at,
            spender,
            *expected_allowance,
        ),
        IcpIndexOperation::Burn {
            from,
            amount,
            spender,
        } => icp_burn_parts(from, *amount, spender.as_deref()),
        IcpIndexOperation::Mint { to, amount } => icp_mint_parts(to, *amount),
        IcpIndexOperation::Transfer {
            to,
            fee,
            from,
            amount,
            spender,
        } => icp_transfer_parts(to, *fee, from, *amount, spender.as_deref()),
    }
}

fn icp_approve_parts(
    fee: IcpIndexTokens,
    from: &str,
    allowance: IcpIndexTokens,
    expires_at: Option<IcpIndexTimestamp>,
    spender: &str,
    expected_allowance: Option<IcpIndexTokens>,
) -> (&'static str, TransactionSummary, JsonValue) {
    (
        "approve",
        TransactionSummary {
            amount_base_units: Some(allowance.e8s.to_string()),
            fee_base_units: Some(fee.e8s.to_string()),
            from: Some(account_identifier_row(from)),
            spender: Some(account_identifier_row(spender)),
            expires_at_unix_nanos: expires_at
                .map(|timestamp| timestamp.timestamp_nanos.to_string()),
            expected_allowance_base_units: expected_allowance.map(|tokens| tokens.e8s.to_string()),
            ..TransactionSummary::default()
        },
        object([
            ("kind", JsonValue::String("approve".to_string())),
            ("fee_base_units", tokens_json(fee)),
            (
                "from_account_identifier",
                JsonValue::String(from.to_string()),
            ),
            ("allowance_base_units", tokens_json(allowance)),
            ("expires_at_unix_nanos", optional_timestamp_json(expires_at)),
            (
                "spender_account_identifier",
                JsonValue::String(spender.to_string()),
            ),
            (
                "expected_allowance_base_units",
                optional_tokens_json(expected_allowance),
            ),
        ]),
    )
}

fn icp_burn_parts(
    from: &str,
    amount: IcpIndexTokens,
    spender: Option<&str>,
) -> (&'static str, TransactionSummary, JsonValue) {
    (
        "burn",
        TransactionSummary {
            amount_base_units: Some(amount.e8s.to_string()),
            from: Some(account_identifier_row(from)),
            spender: spender.map(account_identifier_row),
            ..TransactionSummary::default()
        },
        object([
            ("kind", JsonValue::String("burn".to_string())),
            (
                "from_account_identifier",
                JsonValue::String(from.to_string()),
            ),
            ("amount_base_units", tokens_json(amount)),
            (
                "spender_account_identifier",
                optional_json(spender.map(|spender| JsonValue::String(spender.to_string()))),
            ),
        ]),
    )
}

fn icp_mint_parts(
    to: &str,
    amount: IcpIndexTokens,
) -> (&'static str, TransactionSummary, JsonValue) {
    (
        "mint",
        TransactionSummary {
            amount_base_units: Some(amount.e8s.to_string()),
            to: Some(account_identifier_row(to)),
            ..TransactionSummary::default()
        },
        object([
            ("kind", JsonValue::String("mint".to_string())),
            ("to_account_identifier", JsonValue::String(to.to_string())),
            ("amount_base_units", tokens_json(amount)),
        ]),
    )
}

fn icp_transfer_parts(
    to: &str,
    fee: IcpIndexTokens,
    from: &str,
    amount: IcpIndexTokens,
    spender: Option<&str>,
) -> (&'static str, TransactionSummary, JsonValue) {
    (
        "transfer",
        TransactionSummary {
            amount_base_units: Some(amount.e8s.to_string()),
            fee_base_units: Some(fee.e8s.to_string()),
            from: Some(account_identifier_row(from)),
            to: Some(account_identifier_row(to)),
            spender: spender.map(account_identifier_row),
            ..TransactionSummary::default()
        },
        object([
            ("kind", JsonValue::String("transfer".to_string())),
            ("to_account_identifier", JsonValue::String(to.to_string())),
            ("fee_base_units", tokens_json(fee)),
            (
                "from_account_identifier",
                JsonValue::String(from.to_string()),
            ),
            ("amount_base_units", tokens_json(amount)),
            (
                "spender_account_identifier",
                optional_json(spender.map(|spender| JsonValue::String(spender.to_string()))),
            ),
        ]),
    )
}

fn account_identifier_row(account_identifier: &str) -> IcrcAccountRow {
    IcrcAccountRow {
        owner: None,
        subaccount_hex: None,
        account_identifier: Some(account_identifier.to_string()),
    }
}

fn tokens_json(tokens: IcpIndexTokens) -> JsonValue {
    JsonValue::String(tokens.e8s.to_string())
}

fn optional_tokens_json(tokens: Option<IcpIndexTokens>) -> JsonValue {
    optional_json(tokens.map(tokens_json))
}

fn optional_timestamp_json(timestamp: Option<IcpIndexTimestamp>) -> JsonValue {
    optional_json(
        timestamp.map(|timestamp| JsonValue::String(timestamp.timestamp_nanos.to_string())),
    )
}

fn transaction_json(transaction: &IcrcIndexTransaction) -> JsonValue {
    object([
        ("kind", JsonValue::String(transaction.kind.clone())),
        (
            "timestamp_unix_nanos",
            JsonValue::String(transaction.timestamp.to_string()),
        ),
        (
            "burn",
            optional_json(transaction.burn.as_ref().map(burn_json)),
        ),
        (
            "mint",
            optional_json(transaction.mint.as_ref().map(mint_json)),
        ),
        (
            "approve",
            optional_json(transaction.approve.as_ref().map(approve_json)),
        ),
        (
            "fee_collector",
            optional_json(transaction.fee_collector.as_ref().map(fee_collector_json)),
        ),
        (
            "authorized_mint",
            optional_json(
                transaction
                    .authorized_mint
                    .as_ref()
                    .map(authorized_mint_json),
            ),
        ),
        (
            "authorized_burn",
            optional_json(
                transaction
                    .authorized_burn
                    .as_ref()
                    .map(authorized_burn_json),
            ),
        ),
        (
            "transfer",
            optional_json(transaction.transfer.as_ref().map(transfer_json)),
        ),
    ])
}

fn approve_json(approve: &IcrcIndexApprove) -> JsonValue {
    object([
        ("fee_base_units", optional_nat_json(approve.fee.as_ref())),
        ("from", account_json(&approve.from)),
        ("memo_hex", optional_blob_json(approve.memo.as_deref())),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(approve.created_at_time),
        ),
        ("amount_base_units", nat_json(&approve.amount)),
        (
            "expected_allowance_base_units",
            optional_nat_json(approve.expected_allowance.as_ref()),
        ),
        (
            "expires_at_unix_nanos",
            optional_u64_json(approve.expires_at),
        ),
        ("spender", account_json(&approve.spender)),
    ])
}

fn burn_json(burn: &IcrcIndexBurn) -> JsonValue {
    object([
        ("from", account_json(&burn.from)),
        ("memo_hex", optional_blob_json(burn.memo.as_deref())),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(burn.created_at_time),
        ),
        ("amount_base_units", nat_json(&burn.amount)),
        (
            "spender",
            optional_json(burn.spender.as_ref().map(account_json)),
        ),
        ("fee_base_units", optional_nat_json(burn.fee.as_ref())),
    ])
}

fn mint_json(mint: &IcrcIndexMint) -> JsonValue {
    object([
        ("to", account_json(&mint.to)),
        ("memo_hex", optional_blob_json(mint.memo.as_deref())),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(mint.created_at_time),
        ),
        ("amount_base_units", nat_json(&mint.amount)),
        ("fee_base_units", optional_nat_json(mint.fee.as_ref())),
    ])
}

fn transfer_json(transfer: &IcrcIndexTransfer) -> JsonValue {
    object([
        ("to", account_json(&transfer.to)),
        ("fee_base_units", optional_nat_json(transfer.fee.as_ref())),
        ("from", account_json(&transfer.from)),
        ("memo_hex", optional_blob_json(transfer.memo.as_deref())),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(transfer.created_at_time),
        ),
        ("amount_base_units", nat_json(&transfer.amount)),
        (
            "spender",
            optional_json(transfer.spender.as_ref().map(account_json)),
        ),
    ])
}

fn fee_collector_json(fee_collector: &IcrcIndexFeeCollector) -> JsonValue {
    object([
        (
            "caller",
            optional_json(
                fee_collector
                    .caller
                    .as_ref()
                    .map(|caller| JsonValue::String(caller.to_text())),
            ),
        ),
        (
            "fee_collector",
            optional_json(fee_collector.fee_collector.as_ref().map(account_json)),
        ),
        ("timestamp_unix_nanos", optional_u64_json(fee_collector.ts)),
        (
            "method",
            optional_json(fee_collector.mthd.clone().map(JsonValue::String)),
        ),
    ])
}

fn authorized_mint_json(mint: &IcrcIndexAuthorizedMint) -> JsonValue {
    object([
        ("to", account_json(&mint.to)),
        ("amount_base_units", nat_json(&mint.amount)),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(mint.created_at_time),
        ),
        (
            "caller",
            optional_json(
                mint.caller
                    .as_ref()
                    .map(|caller| JsonValue::String(caller.to_text())),
            ),
        ),
        (
            "method",
            optional_json(mint.mthd.clone().map(JsonValue::String)),
        ),
        (
            "reason",
            optional_json(mint.reason.clone().map(JsonValue::String)),
        ),
    ])
}

fn authorized_burn_json(burn: &IcrcIndexAuthorizedBurn) -> JsonValue {
    object([
        ("from", account_json(&burn.from)),
        ("amount_base_units", nat_json(&burn.amount)),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(burn.created_at_time),
        ),
        (
            "caller",
            optional_json(
                burn.caller
                    .as_ref()
                    .map(|caller| JsonValue::String(caller.to_text())),
            ),
        ),
        (
            "method",
            optional_json(burn.mthd.clone().map(JsonValue::String)),
        ),
        (
            "reason",
            optional_json(burn.reason.clone().map(JsonValue::String)),
        ),
    ])
}

fn account_json(account: &IcrcAccount) -> JsonValue {
    object([
        ("owner", JsonValue::String(account.owner.to_text())),
        (
            "subaccount_hex",
            optional_json(
                account
                    .subaccount
                    .as_deref()
                    .map(hex_bytes)
                    .map(JsonValue::String),
            ),
        ),
    ])
}

fn nat_json(value: &Nat) -> JsonValue {
    JsonValue::String(nat_text(value))
}

fn optional_nat_json(value: Option<&Nat>) -> JsonValue {
    optional_json(value.map(nat_json))
}

fn nat_text(value: &Nat) -> String {
    value.0.to_str_radix(10)
}

fn optional_u64_json(value: Option<u64>) -> JsonValue {
    optional_json(value.map(|value| JsonValue::String(value.to_string())))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nat_text_is_plain_decimal_for_reusable_pagination_cursors() {
        assert_eq!(nat_text(&Nat::from(779_513_u64)), "779513");
    }

    #[test]
    fn account_transaction_cursor_accepts_nat_beyond_u64_and_canonicalizes_zeroes() {
        assert_eq!(
            normalize_transaction_cursor("18446744073709551616")
                .expect("arbitrary candid Nat cursor"),
            "18446744073709551616"
        );
        assert_eq!(
            normalize_transaction_cursor("00042").expect("decimal cursor"),
            "42"
        );
    }

    #[test]
    fn collection_state_requires_stable_exhausting_unique_pages() {
        let mut state =
            AccountTransactionCollectionState::new(Principal::management_canister().to_text());

        assert!(
            !state
                .ingest(page(&["10", "9"], Some("8"), Some("9")), 3)
                .expect("short non-exhausting first page")
        );
        assert!(
            state
                .ingest(page(&["8"], Some("8"), Some("8")), 2)
                .expect("exhausting page")
        );
        let complete = state
            .into_complete("TEST".to_string(), 8)
            .expect("unique complete collection");

        assert_eq!(complete.page_count, 2);
        assert_eq!(
            complete
                .transactions
                .iter()
                .map(|transaction| transaction.id.as_str())
                .collect::<Vec<_>>(),
            vec!["10", "9", "8"]
        );
    }

    #[test]
    fn collection_state_rejects_duplicate_rows_and_changed_oldest_id() {
        let mut duplicate =
            AccountTransactionCollectionState::new(Principal::management_canister().to_text());
        duplicate
            .ingest(page(&["10", "9"], Some("8"), Some("9")), 2)
            .expect("first page");
        duplicate
            .ingest(page(&["9", "8"], Some("8"), Some("8")), 2)
            .expect("duplicate is detected after canonical sorting");
        let duplicate_error = duplicate
            .into_complete("TEST".to_string(), 8)
            .expect_err("duplicate transaction id");
        assert!(matches!(
            duplicate_error,
            IcrcAccountTransactionError::IncompleteCollection {
                reason,
                ..
            } if reason.contains("duplicate")
        ));

        let mut changed_oldest =
            AccountTransactionCollectionState::new(Principal::management_canister().to_text());
        changed_oldest
            .ingest(page(&["10", "9"], Some("1"), Some("9")), 2)
            .expect("first page");
        let changed_error = changed_oldest
            .ingest(page(&["8"], Some("2"), Some("8")), 2)
            .expect_err("changed oldest transaction id");
        assert!(matches!(
            changed_error,
            IcrcAccountTransactionError::IncompleteCollection {
                reason,
                ..
            } if reason.contains("oldest transaction id changed")
        ));
    }

    #[test]
    fn collection_page_error_retains_the_resolved_index() {
        let index_canister_id = Principal::management_canister().to_text();
        let state = AccountTransactionCollectionState::new(index_canister_id.clone());

        let error = state.page_error(IcrcAccountTransactionError::InvalidCursor {
            value: "bad".to_string(),
            reason: "fixture".to_string(),
        });

        assert!(matches!(
            error,
            IcrcAccountTransactionError::CollectionPage {
                index_canister_id: Some(actual),
                pages_fetched: 0,
                rows_fetched: 0,
                last_cursor: None,
                ..
            } if actual == index_canister_id
        ));
    }

    #[test]
    fn current_index_wire_shape_round_trips_and_projects_approve_losslessly() {
        let owner = IcrcAccount {
            owner: Principal::anonymous(),
            subaccount: Some(vec![0xaa; 32]),
        };
        let spender = IcrcAccount {
            owner: Principal::management_canister(),
            subaccount: None,
        };
        let response = IcrcIndexTransactionsResult::Ok(IcrcIndexTransactions {
            balance: Nat::from(500_u64),
            transactions: vec![IcrcIndexTransactionWithId {
                id: Nat::from(99_u64),
                transaction: IcrcIndexTransaction {
                    burn: None,
                    kind: "approve".to_string(),
                    mint: None,
                    approve: Some(IcrcIndexApprove {
                        fee: Some(Nat::from(10_u64)),
                        from: owner,
                        memo: Some(vec![0x01, 0x02]),
                        created_at_time: Some(1_700_000_000_000_000_000),
                        amount: Nat::from(250_u64),
                        expected_allowance: Some(Nat::from(200_u64)),
                        expires_at: Some(1_800_000_000_000_000_000),
                        spender: spender.clone(),
                    }),
                    fee_collector: Some(IcrcIndexFeeCollector {
                        caller: Some(Principal::anonymous()),
                        fee_collector: Some(spender),
                        ts: Some(1_700_000_000_000_000_001),
                        mthd: Some("icrc2_approve".to_string()),
                    }),
                    authorized_mint: None,
                    authorized_burn: None,
                    timestamp: 1_700_000_000_000_000_002,
                    transfer: None,
                },
            }],
            oldest_tx_id: Some(Nat::from(7_u64)),
        });

        let bytes = candid::encode_one(&response).expect("encode index response");
        let decoded: IcrcIndexTransactionsResult =
            candid::decode_one(&bytes).expect("decode index response");
        assert_eq!(decoded, response);

        let IcrcIndexTransactionsResult::Ok(decoded) = decoded else {
            panic!("expected successful index response");
        };
        let row = account_transaction_row(
            decoded
                .transactions
                .into_iter()
                .next()
                .expect("transaction row"),
        );

        assert_eq!(row.id, "99");
        assert_eq!(row.amount_base_units.as_deref(), Some("250"));
        assert_eq!(row.fee_base_units.as_deref(), Some("10"));
        assert_eq!(row.memo_hex.as_deref(), Some("0102"));
        assert_eq!(row.expected_allowance_base_units.as_deref(), Some("200"));
        assert_eq!(
            row.raw_transaction["approve"]["from"]["subaccount_hex"],
            json!("aa".repeat(32))
        );
        assert_eq!(
            row.raw_transaction["fee_collector"]["method"],
            json!("icrc2_approve")
        );
        assert_eq!(row.raw_transaction["transfer"], JsonValue::Null);
    }

    #[test]
    fn deployed_icp_index_wire_shape_decodes_and_preserves_account_identifiers() {
        let response = IcpIndexTransactionsResult::Ok(IcpIndexTransactions {
            balance: 500,
            transactions: vec![IcpIndexTransactionWithId {
                id: 99,
                transaction: IcpIndexTransaction {
                    memo: 7,
                    icrc1_memo: Some(vec![0x01, 0x02]),
                    operation: IcpIndexOperation::Transfer {
                        to: "to-account".to_string(),
                        fee: IcpIndexTokens { e8s: 10 },
                        from: "from-account".to_string(),
                        amount: IcpIndexTokens { e8s: 250 },
                        spender: Some("spender-account".to_string()),
                    },
                    created_at_time: Some(IcpIndexTimestamp {
                        timestamp_nanos: 1_700_000_000_000_000_000,
                    }),
                    timestamp: Some(IcpIndexTimestamp {
                        timestamp_nanos: 1_700_000_000_000_000_002,
                    }),
                },
            }],
            oldest_tx_id: Some(7),
        });
        let bytes = candid::encode_one(&response).expect("encode ICP index response");

        let page = decode_account_transactions(&bytes, &Principal::management_canister())
            .expect("decode ICP account transactions");
        let row = &page.transactions[0];

        assert_eq!(page.balance, "500");
        assert_eq!(page.oldest_transaction_id.as_deref(), Some("7"));
        assert_eq!(page.next_start.as_deref(), Some("99"));
        assert_eq!(row.kind, "transfer");
        assert_eq!(row.amount_base_units.as_deref(), Some("250"));
        assert_eq!(row.fee_base_units.as_deref(), Some("10"));
        assert_eq!(
            row.from
                .as_ref()
                .and_then(|account| account.account_identifier.as_deref()),
            Some("from-account")
        );
        assert_eq!(row.memo_hex.as_deref(), Some("0102"));
        assert_eq!(row.raw_transaction["memo"], json!("7"));
        assert_eq!(
            row.raw_transaction["operation"]["spender_account_identifier"],
            json!("spender-account")
        );
    }

    fn page(
        transaction_ids: &[&str],
        oldest_transaction_id: Option<&str>,
        next_start: Option<&str>,
    ) -> AccountTransactionsPage {
        AccountTransactionsPage {
            balance: "100".to_string(),
            oldest_transaction_id: oldest_transaction_id.map(str::to_string),
            next_start: next_start.map(str::to_string),
            transactions: transaction_ids
                .iter()
                .map(|id| IcrcAccountTransactionRow {
                    id: (*id).to_string(),
                    kind: "transfer".to_string(),
                    timestamp_unix_nanos: None,
                    amount_base_units: None,
                    fee_base_units: None,
                    from: None,
                    to: None,
                    spender: None,
                    memo_hex: None,
                    created_at_time_unix_nanos: None,
                    expires_at_unix_nanos: None,
                    expected_allowance_base_units: None,
                    raw_transaction: json!({"kind": "transfer"}),
                })
                .collect(),
        }
    }
}
