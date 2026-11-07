//! Module: nns::proposals::report::collection
//!
//! Responsibility: advance caller-persisted complete proposal walks one page at a time.
//! Does not own: stable memory, filesystem caches, scheduling, retries, or publication.
//! Boundary: validates resumable input and derives the next state from one admitted source page.

#[cfg(feature = "nns-host")]
use super::NnsProposalHostError;
use super::{
    NnsProposalError,
    model::{NnsProposalListReport, NnsProposalListRequest},
    source::{
        NnsProposalSource, build_nns_proposal_list_report_with_source, validate_proposal_page_size,
    },
};
#[cfg(feature = "nns-host")]
use crate::nns::LiveNnsSource;
use crate::nns::governance::{
    GovernanceCollectionState, NnsGovernanceRequest, governance_collection_accessors,
    validate_governance_continuation, validate_governance_request,
};
use serde::{Deserialize, Serialize};

/// Version of the persistable resumable NNS proposal collection state.
pub const NNS_PROPOSAL_COLLECTION_STATE_SCHEMA_VERSION: u32 = 1;

///
/// NnsProposalCollectionState
///
/// Serializable continuation state for an explicitly bounded proposal walk.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsProposalCollectionState {
    #[serde(flatten)]
    collection: GovernanceCollectionState,
    proposals_fetched: u64,
    next_before_proposal_id: Option<u64>,
}

impl NnsProposalCollectionState {
    /// Start an empty proposal walk with explicit per-page and cumulative call ceilings.
    pub fn new(
        request: &NnsGovernanceRequest,
        page_size: u32,
        max_pages: u32,
    ) -> Result<Self, NnsProposalError> {
        validate_governance_request(request)?;
        validate_proposal_page_size(page_size)?;
        if max_pages == 0 {
            return Err(NnsProposalError::InvalidCollectionMaxPages);
        }
        Ok(Self {
            collection: GovernanceCollectionState::new(request, page_size, max_pages),
            proposals_fetched: 0,
            next_before_proposal_id: None,
        })
    }

    #[cfg(feature = "nns-host")]
    pub(in crate::nns) const fn from_complete_collection(
        collection: GovernanceCollectionState,
        rows: u64,
    ) -> Self {
        Self {
            collection,
            proposals_fetched: rows,
            next_before_proposal_id: None,
        }
    }

    #[cfg(feature = "nns-host")]
    pub(in crate::nns) fn collection_evidence(
        &self,
    ) -> crate::nns::governance::collection::NnsGovernanceCollectionEvidence {
        crate::nns::governance::collection::NnsGovernanceCollectionEvidence::from_state(
            &self.collection,
        )
    }

    governance_collection_accessors!(next_before_proposal_id);

    /// Return the number of successfully admitted proposal rows.
    #[must_use]
    pub const fn proposals_fetched(&self) -> u64 {
        self.proposals_fetched
    }

    /// Return the exclusive cursor for the next page, when one remains.
    #[must_use]
    pub const fn next_before_proposal_id(&self) -> Option<u64> {
        self.next_before_proposal_id
    }
}

///
/// NnsProposalCollectionStep
///
/// One admitted bounded page and the continuation state that follows it.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalCollectionStep {
    /// Unfiltered page returned by the shared proposal report builder.
    pub page: NnsProposalListReport,
    /// Validated state to persist only after retaining the page.
    pub state: NnsProposalCollectionState,
}

/// Advance a resumable proposal walk through the native replica adapter.
#[cfg(feature = "nns-host")]
pub async fn advance_nns_proposal_collection(
    request: &NnsGovernanceRequest,
    state: &NnsProposalCollectionState,
) -> Result<NnsProposalCollectionStep, NnsProposalHostError> {
    Ok(
        advance_nns_proposal_collection_with_source(request, state, &LiveNnsSource::default())
            .await?,
    )
}

/// Advance a resumable proposal walk by exactly one caller-runtime source call.
pub async fn advance_nns_proposal_collection_with_source(
    request: &NnsGovernanceRequest,
    state: &NnsProposalCollectionState,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalCollectionStep, NnsProposalError> {
    validate_governance_request(request)?;
    validate_collection_state(state)?;
    validate_governance_continuation!(request, state, NnsProposalError)?;

    let mut page_request = NnsProposalListRequest::new(request.clone(), state.collection.page_size);
    page_request.before_proposal_id = state.next_before_proposal_id;
    let page = build_nns_proposal_list_report_with_source(&page_request, source).await?;
    let (collection, proposals_fetched) = state.collection.after_page(
        request,
        page.context.source.clone(),
        state.proposals_fetched,
        page.proposals.len(),
        |expected, actual| NnsProposalError::CollectionSourceChanged { expected, actual },
        NnsProposalError::CollectionAccountingOverflow,
    )?;
    let page_count = u32::try_from(page.proposals.len())
        .map_err(|_| NnsProposalError::CollectionAccountingOverflow)?;
    let next_before_proposal_id = (page_count == state.collection.page_size)
        .then(|| {
            page.proposals
                .iter()
                .filter_map(|proposal| proposal.proposal_id)
                .min()
                .filter(|proposal_id| *proposal_id > 1)
        })
        .flatten();
    let next_state = NnsProposalCollectionState {
        collection,
        proposals_fetched,
        next_before_proposal_id,
    };
    Ok(NnsProposalCollectionStep {
        page,
        state: next_state,
    })
}

pub(super) fn validate_collection_state(
    state: &NnsProposalCollectionState,
) -> Result<(), NnsProposalError> {
    let invalid = |reason| NnsProposalError::InvalidCollectionState { reason };
    state
        .collection
        .validate(NNS_PROPOSAL_COLLECTION_STATE_SCHEMA_VERSION, invalid)?;
    validate_proposal_page_size(state.collection.page_size)?;

    let proposals_fetched = state.proposals_fetched;
    let maximum_rows =
        state
            .collection
            .validate_row_bounds("proposals_fetched", proposals_fetched, invalid)?;

    if state
        .next_before_proposal_id
        .is_some_and(|cursor| cursor <= 1)
    {
        return Err(invalid(
            "next_before_proposal_id must be greater than one".to_string(),
        ));
    }

    state.collection.validate_lifecycle(
        proposals_fetched,
        maximum_rows,
        state.next_before_proposal_id.is_some(),
        true,
        invalid,
    )
}

#[cfg(test)]
mod tests;
