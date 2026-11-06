//! Module: nns::data_center
//!
//! Responsibility: assemble data-center CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts data-center arguments to the typed library API.

use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;
use ic_query::nns::data_center::build_nns_data_center_info_report_with_source_and_progress;
use ic_query::nns::data_center::build_nns_data_center_list_report_with_source_and_progress;
use ic_query_text::nns::data_center::nns_data_center_info_report_text;
use ic_query_text::nns::data_center::nns_data_center_list_report_text;

mod spec;

pub(in crate::nns) fn command() -> clap::Command {
    leaf::command(
        &spec::DATA_CENTER_SPEC,
        ic_query::nns::data_center::DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    )
}

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    leaf::run_cached_leaf(
        matches,
        network,
        async |request, progress| {
            build_nns_data_center_list_report_with_source_and_progress(
                request,
                &ic_query::nns::LiveNnsSource::default(),
                progress,
            )
            .await
        },
        async |request, progress| {
            build_nns_data_center_info_report_with_source_and_progress(
                request,
                &ic_query::nns::LiveNnsSource::default(),
                progress,
            )
            .await
        },
        nns_data_center_list_report_text,
        nns_data_center_info_report_text,
    )
    .await
}

#[cfg(test)]
pub(in crate::nns) use spec::DATA_CENTER_SPEC;
