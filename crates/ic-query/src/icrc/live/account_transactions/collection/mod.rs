//! Module: icrc::live::account_transactions::collection
//!
//! Responsibility: validate and assemble one complete paged account history.
//! Does not own: index discovery, wire decoding, live transport, caching, or reports.
//! Boundary: accepts normalized index pages and emits only complete canonical collections.

use super::cursor::validate_account_transaction_page;
use crate::{
    icrc::model::{
        IcrcAccountTransactionCollectionData, IcrcAccountTransactionError,
        IcrcAccountTransactionRow,
    },
    snapshot_cache::PagedCollectionState,
};

///
/// AccountTransactionsPage
///
/// Protocol-neutral account-history page produced by an index wire decoder.
///

pub(super) struct AccountTransactionsPage {
    pub(super) balance: String,
    pub(super) oldest_transaction_id: Option<String>,
    pub(super) transactions: Vec<IcrcAccountTransactionRow>,
}

impl AccountTransactionsPage {
    pub(super) fn next_start(&self) -> Option<&str> {
        self.transactions.last().map(|row| row.id.as_str())
    }
}

///
/// AccountTransactionCollectionState
///
/// In-progress canonical account-history collection and its failure evidence.
///

pub(super) struct AccountTransactionCollectionState {
    index_canister_id: String,
    first_page: Option<FirstPageEvidence>,
    pages: PagedCollectionState<IcrcAccountTransactionRow, String>,
}

struct FirstPageEvidence {
    balance: String,
    oldest_transaction_id: Option<String>,
}

impl AccountTransactionCollectionState {
    pub(super) const fn new(index_canister_id: String) -> Self {
        Self {
            index_canister_id,
            first_page: None,
            pages: PagedCollectionState::new(),
        }
    }

    pub(super) const fn page_count(&self) -> u32 {
        self.pages.page_count()
    }

    pub(super) fn next_cursor(&self) -> Option<&str> {
        self.pages.next_cursor().map(String::as_str)
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
        validate_account_transaction_page(
            &page.transactions,
            self.next_cursor(),
            page.oldest_transaction_id.as_deref(),
            page.next_start(),
        )
        .map_err(|error| self.incomplete(error.to_string()))?;
        if self
            .first_page
            .as_ref()
            .is_some_and(|first| first.oldest_transaction_id != page.oldest_transaction_id)
        {
            return Err(self.incomplete("index oldest transaction id changed during collection"));
        }
        if page_len > 0 && page.oldest_transaction_id.is_none() {
            return Err(
                self.incomplete("index returned transactions without an oldest transaction id")
            );
        }

        let next_start = page.next_start().map(str::to_owned);
        self.first_page.get_or_insert(FirstPageEvidence {
            balance: page.balance,
            oldest_transaction_id: page.oldest_transaction_id,
        });
        self.pages.ingest(page.transactions, next_start);
        Ok(self.next_cursor().is_none()
            || self.next_cursor()
                == self
                    .first_page
                    .as_ref()
                    .and_then(|first| first.oldest_transaction_id.as_deref()))
    }

    pub(super) fn incomplete(&self, reason: impl Into<String>) -> IcrcAccountTransactionError {
        IcrcAccountTransactionError::IncompleteCollection {
            index_canister_id: Some(self.index_canister_id.clone()),
            pages_fetched: self.pages.page_count(),
            rows_fetched: self.pages.row_count(),
            last_cursor: self.pages.next_cursor().cloned(),
            reason: reason.into(),
        }
    }

    pub(super) fn page_error(
        &self,
        source: IcrcAccountTransactionError,
    ) -> IcrcAccountTransactionError {
        IcrcAccountTransactionError::CollectionPage {
            index_canister_id: Some(self.index_canister_id.clone()),
            pages_fetched: self.pages.page_count(),
            rows_fetched: self.pages.row_count(),
            last_cursor: self.pages.next_cursor().cloned(),
            source: Box::new(source),
        }
    }

    pub(super) const fn progress(&self) -> crate::PagedQueryProgress {
        crate::PagedQueryProgress {
            collection: crate::QueryProgressCollection::IcrcAccountTransactions,
            target: None,
            pages_fetched: self.pages.page_count(),
            rows_fetched: self.pages.row_count(),
        }
    }

    pub(super) fn into_complete(
        mut self,
        token_symbol: String,
        decimals: u8,
    ) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError> {
        let Some(first_page) = self.first_page.take() else {
            return Err(self.incomplete("index collection has no accepted page"));
        };
        let pages = self.pages.into_complete(Clone::clone);
        Ok(IcrcAccountTransactionCollectionData {
            index_canister_id: self.index_canister_id,
            balance: first_page.balance,
            token_symbol,
            decimals,
            transactions: pages.rows,
            page_count: pages.page_count,
            last_cursor: pages.last_cursor,
        })
    }
}

#[cfg(test)]
mod tests;
