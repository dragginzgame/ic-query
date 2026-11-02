//! Shared identity, bounds and lifecycle metadata for portable Governance walks.

use super::{
    NnsGovernanceCollectionStatus, NnsGovernanceError, NnsGovernanceRequest,
    NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection,
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
    pub(in crate::nns) status: NnsGovernanceCollectionStatus,
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
            status: NnsGovernanceCollectionStatus::Ready,
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

    pub(in crate::nns) fn after_page(
        &self,
        request: &NnsGovernanceRequest,
        source: NnsGovernanceSourceProvenance,
        pages_fetched: u32,
        has_next: bool,
    ) -> Self {
        Self {
            source: Some(source),
            pages_fetched,
            updated_at: request.fetched_at.clone(),
            status: super::collection_status_after_page(has_next, pages_fetched, self.max_pages),
            ..self.clone()
        }
    }
}
