//! Module: sns::report::proposals_cache::storage
//!
//! Responsibility: group proposal cache loading, lookup, and storage identity.
//! Does not own: refresh orchestration, report status assembly, or text rendering.
//! Boundary: re-exports storage helpers used by proposal cache reports.

use crate::sns::report::{
    SnsHostError,
    cache_storage::SnsCacheStorageFamily,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_SCHEMA_VERSION, model::SNS_PROPOSALS_CACHE_FIELDS,
        paths::SnsProposalsCacheCollection,
    },
};
use std::path::PathBuf;

mod load;
mod lookup;

pub(super) use load::load_sns_proposals_cache_at;
pub(super) use lookup::load_sns_proposals_cache_for_input_with_path;

impl SnsCacheStorageFamily for SnsProposalsCacheCollection {
    const CACHE_SCHEMA_VERSION: u32 = SNS_PROPOSALS_CACHE_SCHEMA_VERSION;
    const CACHE_FIELDS: &'static [&'static str] = SNS_PROPOSALS_CACHE_FIELDS;

    fn missing_cache_error(path: PathBuf) -> SnsHostError {
        SnsHostError::MissingProposalsCache { path }
    }
}
