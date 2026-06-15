use super::super::{
    attempt::{
        SnsNeuronsAttemptParts, attempt_from_parts, failed_attempt_from_latest_progress,
        write_sns_neurons_attempt,
    },
    collection::fetch_complete_sns_neurons,
    errors::sns_cache_file_error,
    paths::SnsNeuronsCachePaths,
};
use super::{context::SnsNeuronsRefreshContext, publish::publish_complete_sns_neurons_cache};
use crate::{
    cache_file::{RefreshLockRequest, create_parent_directory, with_refresh_lock},
    sns::report::{
        SnsHostError, SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest,
        live::LiveSnsSource,
        lookup::{enforce_mainnet_network, lookup_request_from_parts, resolve_sns_lookup},
        source::SnsNeuronsSource,
    },
};

const SNS_NEURONS_REFRESH_LOCK_STALE_AFTER_SECONDS: u64 = 30 * 60;

pub fn refresh_sns_neurons_cache(
    request: &SnsNeuronsRefreshRequest,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source(request, &LiveSnsSource)
}

pub(in crate::sns::report) fn refresh_sns_neurons_cache_with_source(
    request: &SnsNeuronsRefreshRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let paths = SnsNeuronsCachePaths::for_root(
        &request.icp_root,
        &request.network,
        &lookup.sns.root_canister_id,
    );
    create_parent_directory(&paths.cache_path).map_err(sns_cache_file_error)?;
    let replaced_existing_cache = paths.cache_path.is_file();
    let context_paths = paths.clone();
    let fetch_request = lookup.fetch_request;
    let list = lookup.list;
    let id = lookup.id;
    let sns = lookup.sns;
    with_refresh_lock(
        RefreshLockRequest {
            lock_path: &paths.lock_path,
            target_path: &paths.cache_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: SNS_NEURONS_REFRESH_LOCK_STALE_AFTER_SECONDS,
        },
        sns_cache_file_error,
        || {
            refresh_sns_neurons_cache_locked(
                SnsNeuronsRefreshContext {
                    request,
                    fetch_request: &fetch_request,
                    list,
                    id,
                    sns,
                    paths: context_paths,
                    replaced_existing_cache,
                },
                source,
            )
        },
    )
}

fn refresh_sns_neurons_cache_locked(
    context: SnsNeuronsRefreshContext<'_>,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    write_sns_neurons_attempt(
        &context.paths.attempt_path,
        &attempt_from_parts(SnsNeuronsAttemptParts {
            request: context.request,
            fetch_request: context.fetch_request,
            sns: &context.sns,
            status: "running",
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
            last_error: None,
        }),
    )?;
    match fetch_complete_sns_neurons(
        context.request,
        context.fetch_request,
        &context.sns,
        source,
        &context.paths.attempt_path,
    ) {
        Ok(complete) => publish_complete_sns_neurons_cache(context, complete),
        Err(err) => {
            let _ = write_sns_neurons_attempt(
                &context.paths.attempt_path,
                &failed_attempt_from_latest_progress(
                    &context.paths.attempt_path,
                    context.request,
                    context.fetch_request,
                    &context.sns,
                    &err,
                ),
            );
            Err(err)
        }
    }
}
