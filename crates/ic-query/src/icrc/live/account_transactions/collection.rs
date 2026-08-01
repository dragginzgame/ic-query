//! Module: icrc::live::account_transactions::collection
//!
//! Responsibility: validate and assemble one complete paged account history.
//! Does not own: index discovery, wire decoding, live transport, caching, or reports.
//! Boundary: accepts normalized index pages and emits only complete canonical collections.

use super::cursor::{
    compare_canonical_decimal, nat_text, normalize_transaction_cursor, parse_transaction_cursor,
};
use crate::{
    QueryProgress, QueryProgressEvent, QueryProgressState,
    icrc::model::{
        IcrcAccountTransactionCollectionData, IcrcAccountTransactionError,
        IcrcAccountTransactionRow,
    },
};

///
/// AccountTransactionsPage
///
/// Protocol-neutral account-history page produced by an index wire decoder.
///

pub(super) struct AccountTransactionsPage {
    pub(super) balance: String,
    pub(super) oldest_transaction_id: Option<String>,
    pub(super) next_start: Option<String>,
    pub(super) transactions: Vec<IcrcAccountTransactionRow>,
}

///
/// AccountTransactionCollectionState
///
/// In-progress canonical account-history collection and its failure evidence.
///

pub(super) struct AccountTransactionCollectionState {
    index_canister_id: String,
    balance: Option<String>,
    oldest_transaction_id: Option<String>,
    oldest_transaction_id_initialized: bool,
    transactions: Vec<IcrcAccountTransactionRow>,
    page_count: u32,
    next_cursor: Option<String>,
}

impl AccountTransactionCollectionState {
    pub(super) const fn new(index_canister_id: String) -> Self {
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

    pub(super) const fn page_count(&self) -> u32 {
        self.page_count
    }

    pub(super) fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    pub(super) fn ingest(
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

    pub(super) fn incomplete(&self, reason: impl Into<String>) -> IcrcAccountTransactionError {
        IcrcAccountTransactionError::IncompleteCollection {
            index_canister_id: Some(self.index_canister_id.clone()),
            pages_fetched: self.page_count,
            rows_fetched: self.transactions.len(),
            last_cursor: self.next_cursor.clone(),
            reason: reason.into(),
        }
    }

    pub(super) fn page_error(
        &self,
        source: IcrcAccountTransactionError,
    ) -> IcrcAccountTransactionError {
        IcrcAccountTransactionError::CollectionPage {
            index_canister_id: Some(self.index_canister_id.clone()),
            pages_fetched: self.page_count,
            rows_fetched: self.transactions.len(),
            last_cursor: self.next_cursor.clone(),
            source: Box::new(source),
        }
    }

    pub(super) fn report_progress(
        &self,
        progress: &mut dyn QueryProgress,
        state: QueryProgressState,
    ) {
        progress.report(QueryProgressEvent::PagedRefresh {
            text: format!(
                "refreshing ICRC account transactions: pages={} rows={}",
                self.page_count,
                self.transactions.len()
            ),
            state,
        });
    }

    pub(super) fn into_complete(
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

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use serde_json::json;

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
