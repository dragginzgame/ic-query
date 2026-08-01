//! Module: icrc::live::account_transactions
//!
//! Responsibility: resolve one ICRC index and query a page or complete account history.
//! Does not own: wire-specific response conversion, collection validation, caching, or rendering.
//! Boundary: validates ledger/index identity once and delegates each protocol concern explicitly.

mod collection;
mod cursor;
mod wire;

use self::collection::{AccountTransactionCollectionState, AccountTransactionsPage};
use self::cursor::parse_transaction_cursor;
pub(in crate::icrc) use self::cursor::{
    normalize_transaction_cursor, validate_canonical_account_transactions,
};
use super::fetch::{
    account_from_parts, live_query_context, query_index_principal, query_token_display_fields,
};
use crate::{
    QueryProgress, QueryProgressState,
    icrc::{
        ledger::{
            GetIndexPrincipalResult, IcrcAccount, index_principal_error_text, principal_from_text,
            query_ledger,
        },
        model::{
            IcrcAccountTransactionCollectionData, IcrcAccountTransactionError,
            IcrcAccountTransactionPageData, IcrcAccountTransactionPageRequest,
            IcrcAccountTransactionRefreshRequest, IcrcError,
        },
    },
};
use candid::Principal;
use ic_agent::Agent;

const INDEX_LEDGER_ID_METHOD: &str = "ledger_id";

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

    state.report_progress(progress, QueryProgressState::Running);
    loop {
        if request
            .max_pages
            .is_some_and(|max_pages| state.page_count() >= max_pages)
        {
            let error = state.incomplete("max pages reached before index exhaustion");
            state.report_progress(progress, QueryProgressState::Stopped);
            return Err(error);
        }

        let page =
            match query_account_transaction_page(&context, state.next_cursor(), request.page_size)
                .await
            {
                Ok(page) => page,
                Err(source) => {
                    let error = state.page_error(source);
                    state.report_progress(progress, QueryProgressState::Failed);
                    return Err(error);
                }
            };
        let exhausted = match state.ingest(page, request.page_size) {
            Ok(exhausted) => exhausted,
            Err(error) => {
                state.report_progress(progress, QueryProgressState::Failed);
                return Err(error);
            }
        };
        state.report_progress(progress, QueryProgressState::Running);
        if exhausted {
            break;
        }
    }
    state.report_progress(progress, QueryProgressState::Complete);

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
    wire::query_account_transaction_page(
        &context.agent,
        &context.index_canister,
        &context.account,
        start.map(parse_transaction_cursor).transpose()?,
        limit,
    )
    .await
}

async fn resolve_index_canister(
    agent: &Agent,
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
