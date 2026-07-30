//! Module: ic::build
//!
//! Responsibility: build IC Dashboard canister reports through source capabilities.
//! Does not own: HTTP transport, source result validation, command parsing, or rendering.
//! Boundary: validates request identity before any live source call.

use crate::{
    ic::{
        IcCanisterReport, IcCanisterRequest, IcCanisterSource, IcHostError, IcSourceRequest,
        LiveIcSource,
        source::{canonical_canister_id, report_from_source},
    },
    subnet_catalog::format_utc_timestamp_secs,
};

/// Build one live canister report from the official IC Dashboard API.
pub fn build_ic_canister_report(
    request: &IcCanisterRequest,
) -> Result<IcCanisterReport, IcHostError> {
    build_ic_canister_report_with_source(request, &LiveIcSource)
}

/// Build one canister report through a custom Dashboard source capability.
pub fn build_ic_canister_report_with_source(
    request: &IcCanisterRequest,
    source: &dyn IcCanisterSource,
) -> Result<IcCanisterReport, IcHostError> {
    let canister_id = canonical_canister_id(&request.canister_id)?;
    let source_request = IcSourceRequest::new(
        &request.source_endpoint,
        format_utc_timestamp_secs(request.now_unix_secs),
        "ic-query",
    );
    let source_data = source.fetch_canister(&source_request, &canister_id)?;
    report_from_source(&source_request, &canister_id, source_data)
}
