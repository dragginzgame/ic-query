//! Shared identity, bounds and lifecycle metadata for portable Governance walks.

use super::{
    NnsGovernanceCollectionStatus, NnsGovernanceError, NnsGovernanceRequest,
    NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection, advance_collection_counts,
    validate_governance_collection_identity, validate_governance_collection_page_bounds,
    validate_source_provenance,
};
use crate::nns::MAINNET_GOVERNANCE_CANISTER_ID;
use serde::{Deserialize, Serialize};

///
/// GovernanceCollectionState
///
/// Shared continuation metadata; each family owns its cursor and row accounting.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct GovernanceCollectionState {
    pub(in crate::nns) schema_version: u32,
    pub(in crate::nns) network: String,
    pub(in crate::nns) governance_canister_id: String,
    pub(in crate::nns) requested_source: NnsGovernanceSourceSelection,
    pub(in crate::nns) source: Option<NnsGovernanceSourceProvenance>,
    pub(in crate::nns) page_size: u32,
    pub(in crate::nns) max_pages: u32,
    pub(in crate::nns) pages_fetched: u32,
    pub(in crate::nns) started_at: String,
    pub(in crate::nns) updated_at: String,
}

impl GovernanceCollectionState {
    /// Initialize metadata after the family admits its request and page limits.
    pub(in crate::nns) fn new(
        request: &NnsGovernanceRequest,
        page_size: u32,
        max_pages: u32,
    ) -> Self {
        Self {
            schema_version: 1,
            network: request.network.clone(),
            governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
            requested_source: request.source.clone(),
            source: None,
            page_size,
            max_pages,
            pages_fetched: 0,
            started_at: request.fetched_at.clone(),
            updated_at: request.fetched_at.clone(),
        }
    }

    pub(in crate::nns) fn validate<Error>(
        &self,
        expected_schema: u32,
        invalid: impl Fn(String) -> Error + Copy,
    ) -> Result<(), Error>
    where
        Error: From<NnsGovernanceError>,
    {
        if self.schema_version != expected_schema {
            return Err(invalid(format!(
                "schema_version is {}, expected {expected_schema}",
                self.schema_version
            )));
        }
        validate_governance_collection_identity(
            &self.network,
            &self.governance_canister_id,
            &self.requested_source,
            (&self.started_at, &self.updated_at),
            invalid,
        )?;
        validate_governance_collection_page_bounds(self.max_pages, self.pages_fetched, invalid)?;
        if let Some(source) = &self.source {
            validate_source_provenance(&self.requested_source, source)?;
        }
        Ok(())
    }

    pub(in crate::nns) const fn status(&self, has_cursor: bool) -> NnsGovernanceCollectionStatus {
        if self.pages_fetched == 0 {
            NnsGovernanceCollectionStatus::Ready
        } else {
            super::collection_status_after_page(has_cursor, self.pages_fetched, self.max_pages)
        }
    }

    pub(in crate::nns) fn validate_row_bounds<Error>(
        &self,
        field: &str,
        rows: u64,
        invalid: impl FnOnce(String) -> Error,
    ) -> Result<u64, Error> {
        // Widen before multiplication: both operands originate as u32.
        let maximum = u64::from(self.pages_fetched) * u64::from(self.page_size);
        let minimum = u64::from(self.pages_fetched.saturating_sub(1)) * u64::from(self.page_size);
        if rows < minimum || rows > maximum {
            return Err(invalid(format!(
                "{field} {rows} is outside {minimum}..={maximum} for {} pages of size {}",
                self.pages_fetched, self.page_size
            )));
        }
        Ok(maximum)
    }

    pub(in crate::nns) fn validate_lifecycle<Error>(
        &self,
        rows: u64,
        maximum_rows: u64,
        has_cursor: bool,
        complete_page_valid: bool,
        invalid: impl FnOnce(String) -> Error,
    ) -> Result<(), Error> {
        let status = self.status(has_cursor);
        let has_source = self.source.is_some();
        let valid = match status {
            NnsGovernanceCollectionStatus::Ready => {
                self.pages_fetched == 0 && rows == 0 && !has_cursor && !has_source
            }
            NnsGovernanceCollectionStatus::Collecting => {
                self.pages_fetched > 0
                    && self.pages_fetched < self.max_pages
                    && rows == maximum_rows
                    && has_cursor
                    && has_source
            }
            NnsGovernanceCollectionStatus::Complete => {
                // Neurons require a short terminal page; proposals may exhaust a full page.
                self.pages_fetched > 0 && !has_cursor && has_source && complete_page_valid
            }
            NnsGovernanceCollectionStatus::PageLimitReached => {
                self.pages_fetched == self.max_pages
                    && rows == maximum_rows
                    && has_cursor
                    && has_source
            }
        };
        if !valid {
            return Err(invalid(format!(
                "status {status} disagrees with cursor, provenance, or counters"
            )));
        }
        Ok(())
    }

    pub(in crate::nns) fn after_page<Error>(
        &self,
        request: &NnsGovernanceRequest,
        source: NnsGovernanceSourceProvenance,
        rows_fetched: u64,
        page_rows: usize,
        source_changed: impl FnOnce(
            NnsGovernanceSourceProvenance,
            NnsGovernanceSourceProvenance,
        ) -> Error,
        overflow: Error,
    ) -> Result<(Self, u64), Error> {
        if let Some(expected) = &self.source
            && *expected != source
        {
            return Err(source_changed(expected.clone(), source));
        }
        let counts = u64::try_from(page_rows).ok().and_then(|page_rows| {
            advance_collection_counts(self.pages_fetched, rows_fetched, page_rows)
        });
        let (pages_fetched, rows_fetched) = counts.ok_or(overflow)?;
        Ok((
            Self {
                source: Some(source),
                pages_fetched,
                updated_at: request.fetched_at.clone(),
                ..self.clone()
            },
            rows_fetched,
        ))
    }
}

// Preserve both public collection APIs while keeping shared metadata accessors together.
macro_rules! governance_collection_accessors {
    ($cursor:ident) => {
        /// Return the state schema version.
        #[must_use]
        pub const fn schema_version(&self) -> u32 {
            self.collection.schema_version
        }

        /// Return the fixed network identity.
        #[must_use]
        pub fn network(&self) -> &str {
            &self.collection.network
        }

        /// Return the fixed Governance canister identity.
        #[must_use]
        pub fn governance_canister_id(&self) -> &str {
            &self.collection.governance_canister_id
        }

        /// Return the source selection fixed when collection started.
        #[must_use]
        pub const fn requested_source(
            &self,
        ) -> &$crate::nns::governance::NnsGovernanceSourceSelection {
            &self.collection.requested_source
        }

        /// Return the concrete source provenance after the first admitted page.
        #[must_use]
        pub const fn source(
            &self,
        ) -> Option<&$crate::nns::governance::NnsGovernanceSourceProvenance> {
            self.collection.source.as_ref()
        }

        /// Return the maximum rows requested from each Governance call.
        #[must_use]
        pub const fn page_size(&self) -> u32 {
            self.collection.page_size
        }

        /// Return the cumulative source-call ceiling.
        #[must_use]
        pub const fn max_pages(&self) -> u32 {
            self.collection.max_pages
        }

        /// Return the number of successfully admitted pages.
        #[must_use]
        pub const fn pages_fetched(&self) -> u32 {
            self.collection.pages_fetched
        }

        /// Return the caller-supplied time at which the collection state was created.
        #[must_use]
        pub fn started_at(&self) -> &str {
            &self.collection.started_at
        }

        /// Return the caller-supplied time attached to the latest admitted page.
        #[must_use]
        pub fn updated_at(&self) -> &str {
            &self.collection.updated_at
        }

        /// Return the collection lifecycle status.
        #[must_use]
        pub const fn status(&self) -> $crate::nns::governance::NnsGovernanceCollectionStatus {
            self.collection.status(self.$cursor.is_some())
        }

        /// Return whether Governance API exhaustion was observed.
        #[must_use]
        pub const fn is_complete(&self) -> bool {
            matches!(
                self.status(),
                $crate::nns::governance::NnsGovernanceCollectionStatus::Complete
            )
        }
    };
}
pub(in crate::nns) use governance_collection_accessors;
