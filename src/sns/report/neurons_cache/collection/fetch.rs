use super::super::{
    attempt::{SnsNeuronsAttemptParts, attempt_from_parts, write_sns_neurons_attempt},
    model::CompleteSnsNeurons,
};
use super::progress::sns_neurons_progress_text;
use crate::{
    progress::ProgressLine,
    sns::report::{
        SnsHostError, SnsNeuronsRefreshRequest, hex_bytes,
        source::{MainnetSns, SnsFetchRequest, SnsNeuronId, SnsNeuronsSource},
    },
};
use std::{collections::HashSet, path::Path};

pub(in crate::sns::report::neurons_cache) fn fetch_complete_sns_neurons(
    request: &SnsNeuronsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    source: &dyn SnsNeuronsSource,
    attempt_path: &Path,
) -> Result<CompleteSnsNeurons, SnsHostError> {
    let mut neurons = Vec::new();
    let mut seen = HashSet::new();
    let mut page_count = 0_u32;
    let mut start_page_at: Option<SnsNeuronId> = None;
    let mut progress = ProgressLine::stderr();
    progress.update(&sns_neurons_progress_text(sns, page_count, neurons.len()));

    loop {
        if request
            .max_pages
            .is_some_and(|max_pages| page_count >= max_pages)
        {
            progress.finish(&format!(
                "{} stopped before completion",
                sns_neurons_progress_text(sns, page_count, neurons.len())
            ));
            return Err(SnsHostError::IncompleteRefresh {
                pages_fetched: page_count,
                rows_fetched: neurons.len(),
                reason: "max pages reached before API exhaustion".to_string(),
            });
        }
        let page = match source.fetch_sns_neuron_page(
            fetch_request,
            sns,
            request.page_size,
            start_page_at.as_ref(),
            None,
        ) {
            Ok(page) => page,
            Err(err) => {
                progress.finish(&format!(
                    "{} failed",
                    sns_neurons_progress_text(sns, page_count, neurons.len())
                ));
                return Err(err);
            }
        };
        page_count = page_count.saturating_add(1);
        let page_len = page.neurons.len();
        let next_cursor = page.last_cursor;
        let next_cursor_text = next_cursor.as_ref().map(|cursor| hex_bytes(&cursor.id));
        let mut new_rows = 0_usize;
        for neuron in page.neurons {
            if seen.insert(neuron.neuron_id.clone()) {
                new_rows = new_rows.saturating_add(1);
                neurons.push(neuron);
            }
        }
        write_sns_neurons_attempt(
            attempt_path,
            &attempt_from_parts(SnsNeuronsAttemptParts {
                request,
                fetch_request,
                sns,
                status: "running",
                pages_fetched: page_count,
                rows_fetched: neurons.len(),
                last_cursor: next_cursor_text.clone(),
                last_error: None,
            }),
        )?;
        progress.update(&sns_neurons_progress_text(sns, page_count, neurons.len()));

        start_page_at.clone_from(&next_cursor);
        if page_len < usize::try_from(request.page_size).unwrap_or(usize::MAX)
            || next_cursor.is_none()
            || new_rows == 0
        {
            break;
        }
    }

    progress.finish(&format!(
        "{} complete",
        sns_neurons_progress_text(sns, page_count, neurons.len())
    ));
    Ok(CompleteSnsNeurons {
        neurons,
        page_count,
        last_cursor: start_page_at.as_ref().map(|cursor| hex_bytes(&cursor.id)),
    })
}
