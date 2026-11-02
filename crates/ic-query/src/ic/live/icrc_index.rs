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
mod tests {
    use super::*;
    use crate::ic::{
        DEFAULT_ICRC_ACCOUNT_INFO_SOURCE_ENDPOINT, DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
        IcIcrcAccountSort, IcIcrcHolderSort,
    };

    const LEDGER: &str = "mxzaz-hqaaa-aaaar-qaada-cai";

    #[test]
    fn index_wire_records_preserve_raw_fields_and_optional_fee_defaults() {
        let value = serde_json::json!({
            "id": "opaque-account", "owner": "aaaaa-aa", "subaccount": "00",
            "balance": "18446744073709551616", "total_transactions": 7,
            "created_timestamp": u64::MAX, "ledger_canister_id": LEDGER,
            "latest_transaction_index": 8, "updated_at": "source-time"
        });
        let account: IcIcrcAccountSourceRow = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(account.account_id, "opaque-account");
        assert_eq!(account.balance_base_units, "18446744073709551616");
        assert_eq!(account.created_at_unix_nanos, u64::MAX);
        assert_eq!(account.dashboard_updated_at, "source-time");
        assert!(!account.active_fee_collector);
        assert_eq!(
            account.fee_collector_block_ranges,
            Vec::<Vec<serde_json::Value>>::new()
        );
        let mut missing = value;
        missing.as_object_mut().unwrap().remove("owner");
        assert!(serde_json::from_value::<IcIcrcAccountSourceRow>(missing).is_err());
        let holder: IcIcrcHolderSourceRow = serde_json::from_value(serde_json::json!({
            "principal": "aaaaa-aa", "balance": "123", "total_transactions": 7,
            "created_timestamp": u64::MAX, "ledger_canister_id": LEDGER,
            "latest_transaction_index": 8, "percentage": "12.5", "value_usd": null,
            "updated_at": "source-time"
        }))
        .unwrap();
        assert_eq!(holder.balance_base_units, "123");
        assert_eq!(holder.created_at_unix_nanos, u64::MAX);
        assert_eq!(holder.percentage, serde_json::json!("12.5"));
        assert!(holder.value_usd.is_null());
    }

    #[test]
    fn account_list_url_preserves_owner_cursor_limit_and_sort() {
        let query = IcIcrcAccountListQuery::new(2, IcIcrcAccountSort::BalanceDescending)
            .with_owner("222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae")
            .with_after("1668734888.0,hkmli-faaaa-aaaar-qb4ba-cai");
        let url = account_list_url(DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT, LEDGER, &query)
            .expect("account-list URL");

        assert_eq!(
            url.path(),
            "/api/v2/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/accounts"
        );
        assert_eq!(
            url.query(),
            Some(
                "owner=222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae&after=1668734888.0&after=hkmli-faaaa-aaaar-qb4ba-cai&limit=2&sort_by=-balance"
            )
        );
    }

    #[test]
    fn holder_list_and_exact_account_use_their_declared_api_versions() {
        let holder = holder_list_url(
            DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
            LEDGER,
            &IcIcrcHolderListQuery::new(20, IcIcrcHolderSort::Principal),
        )
        .expect("holder-list URL");
        let account = account_info_url(
            DEFAULT_ICRC_ACCOUNT_INFO_SOURCE_ENDPOINT,
            LEDGER,
            "222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae",
        )
        .expect("account-info URL");

        assert_eq!(
            holder.path(),
            "/api/v2/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/holders"
        );
        assert_eq!(
            account.path(),
            "/api/v1/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/accounts/222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae"
        );
    }
}
