//! Module: ic::live::canister
//!
//! Responsibility: live Dashboard canister lookup and bounded collection wire handling.
//! Does not own: shared HTTP transport, report projection, metrics, or network resources.
//! Boundary: builds official canister URLs and preserves raw wire values in source data.

use super::{LiveIcSource, dashboard_resource_url};
use crate::ic::{
    IcCanisterCollectionSource, IcCanisterCountSourceData, IcCanisterFilters,
    IcCanisterPageController, IcCanisterPageRow, IcCanisterPageSourceData, IcCanisterSource,
    IcCanisterSourceData, IcCanisterUpgrade, IcHostError, IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcCanisterSource for LiveIcSource {
    fn fetch_canister<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        canister_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<IcCanisterSourceData, IcHostError>> {
        Box::pin(async move {
            let canister_id = source::canonical_canister_id(canister_id)?;
            let url = canister_url(&request.endpoint, &canister_id)?;
            let wire = self.fetch_live(url).await?;
            Ok(DashboardCanister::into_source_data(wire, request))
        })
    }
}

impl IcCanisterCollectionSource for LiveIcSource {
    fn fetch_canister_count<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        filters: &'a IcCanisterFilters,
    ) -> crate::report::SourceFuture<'a, Result<IcCanisterCountSourceData, IcHostError>> {
        Box::pin(async move {
            let filters = source::normalized_filters(filters)?;
            let url =
                canister_collection_url(&request.endpoint, &filters, CollectionOperation::Count)?;
            let wire: DashboardCanisterCount = self.fetch_live(url).await?;
            Ok(IcCanisterCountSourceData {
                source: request.clone(),
                filters,
                total: wire.total,
            })
        })
    }

    fn fetch_canister_page<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        filters: &'a IcCanisterFilters,
        limit: u16,
        after: Option<&'a str>,
        before: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<IcCanisterPageSourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_page_limit(limit)?;
            source::validate_page_cursor_exclusivity(after, before)?;
            let filters = source::normalized_filters(filters)?;
            let (after, before) = source::canonical_page_cursors(after, before)?;
            let url = canister_collection_url(
                &request.endpoint,
                &filters,
                CollectionOperation::Page {
                    limit,
                    after: after.as_deref(),
                    before: before.as_deref(),
                },
            )?;
            let wire = self.fetch_live(url).await?;
            Ok(DashboardCanisterPage::into_source_data(
                wire,
                request,
                &filters,
                limit,
                after.as_deref(),
                before.as_deref(),
            ))
        })
    }
}

fn canister_url(endpoint: &str, canister_id: &str) -> Result<Url, IcHostError> {
    dashboard_resource_url(endpoint, &["canisters", canister_id])
}

enum CollectionOperation<'a> {
    Count,
    Page {
        limit: u16,
        after: Option<&'a str>,
        before: Option<&'a str>,
    },
}

fn canister_collection_url(
    endpoint: &str,
    filters: &IcCanisterFilters,
    operation: CollectionOperation<'_>,
) -> Result<Url, IcHostError> {
    let path = match operation {
        CollectionOperation::Count => &["canisters", "count"][..],
        CollectionOperation::Page { .. } => &["canisters"][..],
    };
    let mut url = dashboard_resource_url(endpoint, path)?;

    {
        let mut query = url.query_pairs_mut();
        if let Some(has_name) = filters.has_name {
            query.append_pair("has_name", if has_name { "true" } else { "false" });
        }
        if let Some(subnet_id) = filters.subnet_id.as_deref() {
            query.append_pair("subnet_id", subnet_id);
        }
        if let Some(controller_id) = filters.controller_id.as_deref() {
            query.append_pair("controller_id", controller_id);
        }
        for language in &filters.languages {
            query.append_pair("language", language);
        }
        for canister_type in &filters.canister_types {
            query.append_pair("canister_type", canister_type);
        }
        if let Some(search) = filters.query.as_deref() {
            query.append_pair("query", search);
        }
        if let CollectionOperation::Page {
            limit,
            after,
            before,
        } = operation
        {
            query.append_pair("sort_by", "canister_id");
            query.append_pair("limit", &limit.to_string());
            if let Some(after) = after {
                query.append_pair("after", after);
            }
            if let Some(before) = before {
                query.append_pair("before", before);
            }
        }
    }
    Ok(url)
}

#[derive(SerdeDeserialize)]
struct DashboardCanister {
    canister_id: String,
    canister_type: Option<String>,
    controllers: Vec<String>,
    id: u64,
    language: String,
    module_hash: String,
    name: String,
    subnet_id: String,
    updated_at: String,
    upgrades: Option<Vec<IcCanisterUpgrade>>,
}

impl DashboardCanister {
    fn into_source_data(self, request: &IcSourceRequest) -> IcCanisterSourceData {
        IcCanisterSourceData {
            source: request.clone(),
            data: crate::ic::IcCanisterData {
                canister_id: self.canister_id,
                dashboard_id: self.id,
                canister_type: self.canister_type,
                name: self.name,
                subnet_id: self.subnet_id,
                controllers: self.controllers,
                language: self.language,
                module_hash: self.module_hash,
                dashboard_updated_at: self.updated_at,
                upgrades: self.upgrades,
            },
        }
    }
}

#[derive(SerdeDeserialize)]
struct DashboardCanisterCount {
    total: u64,
}

#[derive(SerdeDeserialize)]
struct DashboardCanisterPage {
    data: Vec<DashboardCanisterPageRow>,
    next_cursor: Option<String>,
    previous_cursor: Option<String>,
}

impl DashboardCanisterPage {
    fn into_source_data(
        self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
        requested_limit: u16,
        after: Option<&str>,
        before: Option<&str>,
    ) -> IcCanisterPageSourceData {
        IcCanisterPageSourceData {
            source: request.clone(),
            filters: filters.clone(),
            requested_limit,
            after: after.map(str::to_string),
            before: before.map(str::to_string),
            previous_cursor: self.previous_cursor,
            next_cursor: self.next_cursor,
            rows: self
                .data
                .into_iter()
                .map(DashboardCanisterPageRow::into_public)
                .collect(),
        }
    }
}

#[derive(SerdeDeserialize)]
struct DashboardCanisterPageRow {
    canister_id: String,
    canister_type: Option<String>,
    controllers: Vec<(String, Option<String>)>,
    id: u64,
    language: String,
    module_hash: String,
    name: String,
    subnet_id: String,
    updated_at: String,
}

impl DashboardCanisterPageRow {
    fn into_public(self) -> IcCanisterPageRow {
        IcCanisterPageRow {
            canister_id: self.canister_id,
            dashboard_id: self.id,
            canister_type: self.canister_type,
            name: self.name,
            subnet_id: self.subnet_id,
            controllers: self
                .controllers
                .into_iter()
                .map(|(principal_id, raw_metadata)| IcCanisterPageController {
                    principal_id,
                    raw_metadata,
                })
                .collect(),
            language: self.language,
            module_hash: self.module_hash,
            dashboard_updated_at: self.updated_at,
        }
    }
}

#[cfg(test)]
mod tests;
