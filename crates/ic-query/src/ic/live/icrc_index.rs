//! Module: ic::live::icrc_index
//!
//! Responsibility: live official ICRC account and holder index URL and wire conversion.
//! Does not own: native ledger calls, report projection, bounds policy, or rendering.
//! Boundary: fetches one exact account or one response-bounded cursor page.

use super::{LiveIcSource, append_path_segments, dashboard_base_url, fetch_live};
use crate::ic::{
    IcHostError, IcIcrcAccountInfoSourceData, IcIcrcAccountListQuery, IcIcrcAccountListSourceData,
    IcIcrcAccountSourceRow, IcIcrcHolderListQuery, IcIcrcHolderListSourceData,
    IcIcrcHolderSourceRow, IcIcrcIndexSource, IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use serde_json::Value as JsonValue;
use url::Url;

impl IcIcrcIndexSource for LiveIcSource {
    fn fetch_account_list(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcAccountListQuery,
    ) -> Result<IcIcrcAccountListSourceData, IcHostError> {
        let query = source::normalized_account_list_query(query)?;
        let ledger_canister_id =
            source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
        let wire: CursorPage<Account> = fetch_live(account_list_url(
            &request.endpoint,
            &ledger_canister_id,
            &query,
        )?)?;
        Ok(IcIcrcAccountListSourceData {
            source: request.clone(),
            ledger_canister_id,
            query,
            previous_cursor: wire.previous_cursor,
            next_cursor: wire.next_cursor,
            rows: wire.data.into_iter().map(account_source_row).collect(),
        })
    }

    fn fetch_account_info(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        account_id: &str,
    ) -> Result<IcIcrcAccountInfoSourceData, IcHostError> {
        source::validate_account_id(account_id)?;
        let ledger_canister_id =
            source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
        let wire: Account = fetch_live(account_info_url(
            &request.endpoint,
            &ledger_canister_id,
            account_id,
        )?)?;
        Ok(IcIcrcAccountInfoSourceData {
            source: request.clone(),
            account: account_source_row(wire),
        })
    }

    fn fetch_holder_list(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcHolderListQuery,
    ) -> Result<IcIcrcHolderListSourceData, IcHostError> {
        source::validate_holder_list_query(query)?;
        let ledger_canister_id =
            source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
        let wire: CursorPage<Holder> = fetch_live(holder_list_url(
            &request.endpoint,
            &ledger_canister_id,
            query,
        )?)?;
        Ok(IcIcrcHolderListSourceData {
            source: request.clone(),
            ledger_canister_id,
            query: query.clone(),
            previous_cursor: wire.previous_cursor,
            next_cursor: wire.next_cursor,
            rows: wire
                .data
                .into_iter()
                .map(|row| IcIcrcHolderSourceRow {
                    principal: row.principal,
                    balance_base_units: row.balance,
                    total_transactions: row.total_transactions,
                    created_at_unix_nanos: row.created_timestamp,
                    ledger_canister_id: row.ledger_canister_id,
                    latest_transaction_index: row.latest_transaction_index,
                    percentage: row.percentage,
                    value_usd: row.value_usd,
                    dashboard_updated_at: row.updated_at,
                })
                .collect(),
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
        for component in query.after.iter().flat_map(|cursor| cursor.split(',')) {
            pairs.append_pair("after", component);
        }
        for component in query.before.iter().flat_map(|cursor| cursor.split(',')) {
            pairs.append_pair("before", component);
        }
        pairs.append_pair("limit", &query.limit.to_string());
        pairs.append_pair("sort_by", query.sort_by.as_api_value());
    }
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
    {
        let mut pairs = url.query_pairs_mut();
        for component in query.after.iter().flat_map(|cursor| cursor.split(',')) {
            pairs.append_pair("after", component);
        }
        for component in query.before.iter().flat_map(|cursor| cursor.split(',')) {
            pairs.append_pair("before", component);
        }
        pairs.append_pair("limit", &query.limit.to_string());
        pairs.append_pair("sort_by", query.sort_by.as_api_value());
    }
    Ok(url)
}

fn ledger_resource_url(
    endpoint: &str,
    ledger_canister_id: &str,
    resource_path: &[&str],
) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["ledgers", ledger_canister_id])?;
    append_path_segments(endpoint, &mut url, resource_path)?;
    Ok(url)
}

fn account_source_row(row: Account) -> IcIcrcAccountSourceRow {
    IcIcrcAccountSourceRow {
        account_id: row.id,
        owner: row.owner,
        subaccount: row.subaccount,
        balance_base_units: row.balance,
        total_transactions: row.total_transactions,
        created_at_unix_nanos: row.created_timestamp,
        ledger_canister_id: row.ledger_canister_id,
        latest_transaction_index: row.latest_transaction_index,
        dashboard_updated_at: row.updated_at,
        active_fee_collector: row.active_fee_collector,
        fee_collector_block_ranges: row.fee_collector_block_ranges,
    }
}

#[derive(SerdeDeserialize)]
struct CursorPage<Row> {
    data: Vec<Row>,
    next_cursor: Option<String>,
    previous_cursor: Option<String>,
}

#[derive(SerdeDeserialize)]
struct Account {
    id: String,
    owner: String,
    subaccount: String,
    balance: String,
    total_transactions: u64,
    created_timestamp: u64,
    ledger_canister_id: String,
    latest_transaction_index: u64,
    updated_at: String,
    #[serde(default)]
    active_fee_collector: bool,
    #[serde(default)]
    fee_collector_block_ranges: Vec<Vec<JsonValue>>,
}

#[derive(SerdeDeserialize)]
struct Holder {
    principal: String,
    balance: String,
    total_transactions: u64,
    created_timestamp: u64,
    ledger_canister_id: String,
    latest_transaction_index: u64,
    percentage: JsonValue,
    value_usd: JsonValue,
    updated_at: String,
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
