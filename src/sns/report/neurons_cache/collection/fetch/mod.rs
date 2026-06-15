mod attempt;
mod state;

use super::super::model::CompleteSnsNeurons;
use super::progress::sns_neurons_progress_text;
use crate::{
    progress::ProgressLine,
    sns::report::{
        SnsHostError, SnsNeuronsRefreshRequest,
        source::{MainnetSns, SnsFetchRequest, SnsNeuronsSource},
    },
};
use state::SnsNeuronsCollectionState;
use std::path::Path;

pub(in crate::sns::report::neurons_cache) fn fetch_complete_sns_neurons(
    request: &SnsNeuronsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    source: &dyn SnsNeuronsSource,
    attempt_path: &Path,
) -> Result<CompleteSnsNeurons, SnsHostError> {
    let mut state = SnsNeuronsCollectionState::new();
    let mut progress = ProgressLine::stderr();
    progress.update(&progress_text(sns, &state));

    loop {
        if max_pages_reached(request, &state) {
            progress.finish(&format!(
                "{} stopped before completion",
                progress_text(sns, &state)
            ));
            return Err(incomplete_refresh_error(&state));
        }

        let page = match source.fetch_sns_neuron_page(
            fetch_request,
            sns,
            request.page_size,
            state.start_page_at(),
            None,
        ) {
            Ok(page) => page,
            Err(err) => {
                progress.finish(&format!("{} failed", progress_text(sns, &state)));
                return Err(err);
            }
        };
        let page = state.ingest_page(page);
        attempt::write_running_attempt(attempt_path, request, fetch_request, sns, &state, &page)?;
        progress.update(&progress_text(sns, &state));

        if page.exhausts_collection(request.page_size, state.has_next_cursor()) {
            break;
        }
    }

    progress.finish(&format!("{} complete", progress_text(sns, &state)));
    Ok(state.into_complete())
}

fn progress_text(sns: &MainnetSns, state: &SnsNeuronsCollectionState) -> String {
    sns_neurons_progress_text(sns, state.page_count(), state.row_count())
}

fn max_pages_reached(
    request: &SnsNeuronsRefreshRequest,
    state: &SnsNeuronsCollectionState,
) -> bool {
    request
        .max_pages
        .is_some_and(|max_pages| state.page_count() >= max_pages)
}

fn incomplete_refresh_error(state: &SnsNeuronsCollectionState) -> SnsHostError {
    SnsHostError::IncompleteRefresh {
        pages_fetched: state.page_count(),
        rows_fetched: state.row_count(),
        reason: "max pages reached before API exhaustion".to_string(),
    }
}
