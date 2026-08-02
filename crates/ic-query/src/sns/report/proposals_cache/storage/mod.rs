//! Module: sns::report::proposals_cache::storage
//!
//! Responsibility: group proposal cache loading, lookup, and storage identity.
//! Does not own: refresh orchestration, report status assembly, or text rendering.
//! Boundary: re-exports storage helpers used by proposal cache reports.

use crate::sns::report::{
    SnsHostError,
    cache_storage::SnsCacheStorageFamily,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        model::{SNS_PROPOSALS_CACHE_FIELDS, SnsProposalsCacheRows},
        paths::SnsProposalsCacheCollection,
    },
};
use std::{collections::HashSet, path::PathBuf};

mod lookup;

pub(super) use lookup::load_sns_proposals_cache_for_input_with_path;

impl SnsCacheStorageFamily for SnsProposalsCacheCollection {
    type Data = SnsProposalsCacheRows;

    const CACHE_SCHEMA_VERSION: u32 = SNS_PROPOSALS_CACHE_SCHEMA_VERSION;
    const CACHE_FIELDS: &'static [&'static str] = SNS_PROPOSALS_CACHE_FIELDS;
    const CACHE_ITEM_NAME: &'static str = "proposal";

    fn missing_cache_error(path: PathBuf) -> SnsHostError {
        SnsHostError::MissingProposalsCache { path }
    }

    fn row_count(data: &Self::Data) -> usize {
        data.proposals.len()
    }

    fn validate_rows(data: &Self::Data) -> Result<(), String> {
        let mut proposal_ids = HashSet::new();
        if let Some(duplicate) = data
            .proposals
            .iter()
            .map(|proposal| proposal.proposal_id)
            .find(|proposal_id| !proposal_ids.insert(*proposal_id))
        {
            return Err(format!("duplicate proposal id {duplicate}"));
        }
        Ok(())
    }
}
