//! Module: ic::live::icrc_index
//!
//! Responsibility: live official ICRC account and holder index URL and wire conversion.
//! Does not own: native ledger calls, report projection, bounds policy, or rendering.
//! Boundary: fetches one exact account or one response-bounded cursor page.

use super::{LiveIcSource, ledger_resource_url};
use crate::ic::{
    IcHostError, IcIcrcAccountInfoSourceData, IcIcrcAccountListQuery, IcIcrcAccountListSourceData,
    IcIcrcAccountSourceRow, IcIcrcHolderListQuery, IcIcrcHolderListSourceData,
    IcIcrcHolderSourceRow, IcIcrcIndexSource, IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcIcrcIndexSource for LiveIcSource {
    fn fetch_account_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        ledger_canister_id: &'a str,
        query: &'a IcIcrcAccountListQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcIcrcAccountListSourceData, IcHostError>> {
        Box::pin(async move {
            let query = source::normalized_account_list_query(query)?;
            let ledger_canister_id =
                source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
            let wire: CursorPage<IcIcrcAccountSourceRow> = self
                .fetch_live(account_list_url(
                    &request.endpoint,
                    &ledger_canister_id,
                    &query,
                )?)
                .await?;
            Ok(IcIcrcAccountListSourceData {
                source: request.clone(),
                ledger_canister_id,
                query,
                previous_cursor: wire.previous_cursor,
                next_cursor: wire.next_cursor,
                rows: wire.data,
            })
        })
    }

    fn fetch_account_info<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        ledger_canister_id: &'a str,
        account_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<IcIcrcAccountInfoSourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_account_id(account_id)?;
            let ledger_canister_id =
                source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
            let wire: IcIcrcAccountSourceRow = self
                .fetch_live(account_info_url(
                    &request.endpoint,
                    &ledger_canister_id,
                    account_id,
                )?)
                .await?;
            Ok(IcIcrcAccountInfoSourceData {
                source: request.clone(),
                account: wire,
            })
        })
    }

    fn fetch_holder_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        ledger_canister_id: &'a str,
        query: &'a IcIcrcHolderListQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcIcrcHolderListSourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_holder_list_query(query)?;
            let ledger_canister_id =
                source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
            let wire: CursorPage<IcIcrcHolderSourceRow> = self
                .fetch_live(holder_list_url(
                    &request.endpoint,
                    &ledger_canister_id,
                    query,
                )?)
                .await?;
            Ok(IcIcrcHolderListSourceData {
                source: request.clone(),
                ledger_canister_id,
                query: query.clone(),
                previous_cursor: wire.previous_cursor,
                next_cursor: wire.next_cursor,
                rows: wire.data,
            })
        })
    }
}

fn account_list_url(
    endpoint: &str,
    ledger_canister_id: &str,
    query: &IcIcrcAccountListQuery,
) -> Result<Url, IcHostError> {
    let mut url = ledger_resource_url(endpoint, ledger_canister_id, &["accounts"])?;
    {
        let mut pairs = url.query_pairs_mut();
        if let Some(owner) = &query.owner {
            pairs.append_pair("owner", owner);
        }
    }
    append_page_query(
        &mut url,
        query.after.as_deref(),
        query.before.as_deref(),
        query.limit,
        query.sort_by.as_api_value(),
    );
    Ok(url)
}

fn account_info_url(
    endpoint: &str,
    ledger_canister_id: &str,
    account_id: &str,
) -> Result<Url, IcHostError> {
    ledger_resource_url(endpoint, ledger_canister_id, &["accounts", account_id])
}

fn holder_list_url(
    endpoint: &str,
    ledger_canister_id: &str,
    query: &IcIcrcHolderListQuery,
) -> Result<Url, IcHostError> {
    let mut url = ledger_resource_url(endpoint, ledger_canister_id, &["holders"])?;
    append_page_query(
        &mut url,
        query.after.as_deref(),
        query.before.as_deref(),
        query.limit,
        query.sort_by.as_api_value(),
    );
    Ok(url)
}

fn append_page_query(
    url: &mut Url,
    after: Option<&str>,
    before: Option<&str>,
    limit: u16,
    sort_by: &str,
) {
    let mut pairs = url.query_pairs_mut();
    for (key, cursor) in [("after", after), ("before", before)] {
        for component in cursor.into_iter().flat_map(|cursor| cursor.split(',')) {
            pairs.append_pair(key, component);
        }
    }
    pairs.append_pair("limit", &limit.to_string());
    pairs.append_pair("sort_by", sort_by);
}

#[derive(SerdeDeserialize)]
struct CursorPage<Row> {
    data: Vec<Row>,
    next_cursor: Option<String>,
    previous_cursor: Option<String>,
}

#[cfg(test)]
mod tests;
