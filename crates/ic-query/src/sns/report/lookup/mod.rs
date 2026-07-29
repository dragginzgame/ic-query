//! Module: sns::report::lookup
//!
//! Responsibility: group SNS lookup helpers.
//! Does not own: live transport, report assembly, cache IO, or rendering.
//! Boundary: resolves lookup inputs into mainnet SNS identities for builders.

mod ids;
mod model;
mod request;
mod resolve;

pub(in crate::sns::report) use ids::assign_sns_ids_in_current_order;
pub(in crate::sns::report) use request::{
    lookup_request_from_parts, sns_list_fetch_request, validate_sns_refresh_page_size,
};
pub(in crate::sns::report) use resolve::resolve_sns_lookup;
