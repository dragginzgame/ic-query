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
    source::validate_sns_proposal_rows,
};
use std::path::PathBuf;

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
        validate_sns_proposal_rows(&data.proposals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HostCacheError, cache_file::write_managed_json_pretty_atomically,
        sns::report::cache_storage::read_sns_cache_header, test_support::temp_dir,
    };
    use std::{fs, io::Write};

    #[test]
    fn sns_header_lookup_skips_row_decoding_but_checks_complete_json_syntax() {
        let root = temp_dir("ic-query-sns-header-lookup");
        let path = root.join("sns/ic/root/proposals/full.json");
        let cache = serde_json::json!({
            "schema_version": 1,
            "network": "ic",
            "domain": "sns",
            "entity": "root",
            "collection": "proposals",
            "scope": "full",
            "id": 7,
            "proposals": [0, "x".repeat(64 * 1024)],
        });
        write_managed_json_pretty_atomically(
            &root,
            &path,
            &cache,
            |_, error| error.to_string(),
            |error| error.to_string(),
        )
        .unwrap();
        let header = read_sns_cache_header::<SnsProposalsCacheCollection>(&root, &path, "ic")
            .expect("lookup does not deserialize proposal rows");
        assert_eq!(header.id, 7);
        assert_eq!(header.domain, "sns");
        assert_eq!(header.entity, "root");
        assert_eq!(header.collection, "proposals");
        assert_eq!(header.scope, "full");

        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"invalid trailing JSON")
            .unwrap();
        assert!(matches!(
            read_sns_cache_header::<SnsProposalsCacheCollection>(&root, &path, "ic"),
            Err(SnsHostError::Cache(HostCacheError::ParseCache { .. }))
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
