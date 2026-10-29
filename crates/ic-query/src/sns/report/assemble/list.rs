//! Module: sns::report::assemble::list
//!
//! Responsibility: apply deployed SNS list views and assemble list and info reports.
//! Does not own: SNS-W fetching, sort algorithms, lookup resolution, or rendering.
//! Boundary: live and cached lists share filtering, ordering, counts, and DTO projection.

use crate::report::ReportProvenance;
use crate::sns::report::{
    JoinedMainnetSnsInventory, MAINNET_SNS_WASM_CANISTER_ID, MainnetSns,
    SNS_INFO_REPORT_SCHEMA_VERSION, SNS_LIST_REPORT_SCHEMA_VERSION, SnsInfoReport, SnsListReport,
    SnsListRequest, SnsListRow, SnsSourceRequest,
    view::{filter_mainnet_sns_instances, sort_mainnet_sns_instances},
};

pub(in crate::sns::report) fn sns_list_report_from_list(
    mut list: JoinedMainnetSnsInventory,
    request: &SnsListRequest,
    provenance: ReportProvenance,
) -> SnsListReport {
    let catalog_sns_count = list.sns_instances.len();
    filter_mainnet_sns_instances(&mut list.sns_instances, request.all_lifecycles);
    sort_mainnet_sns_instances(&mut list.sns_instances, request.sort);
    let JoinedMainnetSnsInventory {
        network,
        sns_wasm_canister_id,
        fetched_at,
        fetched_by,
        source_endpoint,
        sns_instances,
    } = list;
    let metadata_error_count = sns_instances
        .iter()
        .filter(|sns| sns.metadata_error.is_some())
        .count();
    let lifecycle_error_count = sns_instances
        .iter()
        .filter(|sns| sns.lifecycle_error.is_some())
        .count();
    let sns_instances = sns_instances
        .into_iter()
        .map(|sns| SnsListRow {
            id: sns.id,
            name: sns.name,
            canisters: sns.canisters,
            metadata_error: sns.metadata_error,
            lifecycle: sns.lifecycle,

            lifecycle_error: sns.lifecycle_error,
        })
        .collect::<Vec<_>>();
    SnsListReport {
        schema_version: SNS_LIST_REPORT_SCHEMA_VERSION,
        network,
        sns_wasm_canister_id,
        fetched_at,
        source_endpoint,
        fetched_by,
        data_source: provenance.data_source,
        cache_path: provenance.cache_path,
        cache_complete: provenance.cache_complete,
        all_lifecycles: request.all_lifecycles,
        verbose: request.verbose,
        sort: request.sort.as_str().to_string(),
        catalog_sns_count,
        excluded_sns_count: catalog_sns_count.saturating_sub(sns_instances.len()),
        sns_count: sns_instances.len(),
        metadata_error_count,
        lifecycle_error_count,
        sns_instances,
    }
}

pub(in crate::sns::report) fn sns_info_report_from_parts(
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
) -> SnsInfoReport {
    SnsInfoReport {
        schema_version: SNS_INFO_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        description: sns.description,
        url: sns.url,
        canisters: sns.canisters,
        metadata_error: sns.metadata_error,
    }
}
