macro_rules! nns_leaf_refresh_report {
    (
        $report_type:ident,
        $schema_version:expr,
        $request:ident,
        $report:ident,
        $write_result:ident,
        $count_field:ident
        $(, $governance_canister_id:expr)?
        $(,)?
    ) => {
        $report_type {
            schema_version: $schema_version,
            network: $report.network.clone(),
            cache_path: $write_result.cache_path,
            refresh_lock_path: $write_result.refresh_lock_path,
            output_path: $write_result.output_path,
            $(governance_canister_id: $governance_canister_id,)?
            registry_canister_id: $report.registry_canister_id.clone(),
            registry_version: $report.registry_version,
            fetched_at: $report.fetched_at.clone(),
            source_endpoint: $report.source_endpoint.clone(),
            fetched_by: $report.fetched_by.clone(),
            dry_run: $request.dry_run,
            wrote_cache: $write_result.wrote_cache,
            replaced_existing_cache: $write_result.replaced_existing_cache,
            $count_field: $report.$count_field,
        }
    };
}

macro_rules! nns_leaf_refresh_report_text {
    (
        $report:ident,
        $governance_canister_id:expr,
        $count_label:literal,
        $count_field:ident
        $(,)?
    ) => {
        $crate::nns::render::nns_leaf_refresh_report_text($crate::nns::render::NnsLeafRefreshText {
            network: &$report.network,
            cache_path: &$report.cache_path,
            refresh_lock_path: &$report.refresh_lock_path,
            governance_canister_id: $governance_canister_id,
            registry_canister_id: &$report.registry_canister_id,
            registry_version: $report.registry_version,
            fetched_at: &$report.fetched_at,
            source_endpoint: &$report.source_endpoint,
            fetched_by: &$report.fetched_by,
            dry_run: $report.dry_run,
            wrote_cache: $report.wrote_cache,
            replaced_existing_cache: $report.replaced_existing_cache,
            count_label: $count_label,
            count: $report.$count_field,
        })
    };
}
