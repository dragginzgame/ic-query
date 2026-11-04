//! Module: nns::neuron::report::collection
//!
//! Responsibility: advance caller-persisted complete public-neuron walks one page at a time.
//! Does not own: stable memory, filesystem caches, scheduling, retries, or publication.
//! Boundary: validates resumable input and derives the next state from one admitted source page.

#[cfg(feature = "nns-host")]
use super::NnsNeuronHostError;
use super::{
    NnsNeuronError,
    model::{NnsNeuronListReport, NnsNeuronListRequest},
    source::{NnsNeuronSource, build_nns_neuron_list_report_with_source, validate_page_size},
};
#[cfg(feature = "nns-host")]
use crate::nns::LiveNnsSource;
use crate::nns::governance::{
    GovernanceCollectionState, NnsGovernanceCollectionStatus, NnsGovernanceRequest,
    NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection, validate_governance_continuation,
    validate_governance_request,
};
use serde::{Deserialize, Serialize};

/// Version of the persistable resumable NNS neuron collection state.
pub const NNS_NEURON_COLLECTION_STATE_SCHEMA_VERSION: u32 = 1;

///
/// NnsNeuronCollectionState
///
/// Serializable continuation state for an explicitly bounded public-neuron walk.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsNeuronCollectionState {
    #[serde(flatten)]
    collection: GovernanceCollectionState,
    neurons_fetched: u64,
    next_start_neuron_id: Option<u64>,
}

impl NnsNeuronCollectionState {
    /// Start an empty neuron walk with explicit per-page and cumulative call ceilings.
    pub fn new(
        request: &NnsGovernanceRequest,
        page_size: u32,
        max_pages: u32,
    ) -> Result<Self, NnsNeuronError> {
        validate_governance_request(request)?;
        validate_page_size(page_size)?;
        if max_pages == 0 {
            return Err(NnsNeuronError::InvalidCollectionMaxPages);
        }
        Ok(Self {
            collection: GovernanceCollectionState::new(request, page_size, max_pages),
            neurons_fetched: 0,
            next_start_neuron_id: None,
        })
    }

    #[cfg(feature = "nns-host")]
    pub(in crate::nns) const fn from_complete_collection(
        collection: GovernanceCollectionState,
        rows: u64,
    ) -> Self {
        Self {
            collection,
            neurons_fetched: rows,
            next_start_neuron_id: None,
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
    pub const fn requested_source(&self) -> &NnsGovernanceSourceSelection {
        &self.collection.requested_source
    }

    /// Return the concrete source provenance after the first admitted page.
    #[must_use]
    pub const fn source(&self) -> Option<&NnsGovernanceSourceProvenance> {
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

    /// Return the number of successfully admitted neuron rows.
    #[must_use]
    pub const fn neurons_fetched(&self) -> u64 {
        self.neurons_fetched
    }

    /// Return the exclusive lower neuron-id bound for the next page.
    #[must_use]
    pub const fn next_start_neuron_id(&self) -> Option<u64> {
        self.next_start_neuron_id
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
    pub const fn status(&self) -> NnsGovernanceCollectionStatus {
        self.collection.status(self.next_start_neuron_id.is_some())
    }

    /// Return whether Governance API exhaustion was observed.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self.status(), NnsGovernanceCollectionStatus::Complete)
    }
}

///
/// NnsNeuronCollectionStep
///
/// One admitted bounded page and the continuation state that follows it.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNeuronCollectionStep {
    /// Page returned by the shared public-neuron report builder.
    pub page: NnsNeuronListReport,
    /// Validated state to persist only after retaining the page.
    pub state: NnsNeuronCollectionState,
}

/// Advance a resumable public-neuron walk through the native replica adapter.
#[cfg(feature = "nns-host")]
pub async fn advance_nns_neuron_collection(
    request: &NnsGovernanceRequest,
    state: &NnsNeuronCollectionState,
) -> Result<NnsNeuronCollectionStep, NnsNeuronHostError> {
    Ok(
        advance_nns_neuron_collection_with_source(request, state, &LiveNnsSource::default())
            .await?,
    )
}

/// Advance a resumable public-neuron walk by exactly one caller-runtime source call.
pub async fn advance_nns_neuron_collection_with_source(
    request: &NnsGovernanceRequest,
    state: &NnsNeuronCollectionState,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronCollectionStep, NnsNeuronError> {
    validate_governance_request(request)?;
    validate_collection_state(state)?;
    validate_governance_continuation!(request, state, NnsNeuronError)?;

    let mut page_request = NnsNeuronListRequest::new(request.clone(), state.collection.page_size);
    page_request.exclusive_start_neuron_id = state.next_start_neuron_id;
    let page = build_nns_neuron_list_report_with_source(&page_request, source).await?;
    if let Some(expected) = &state.collection.source
        && *expected != page.context.source
    {
        return Err(NnsNeuronError::CollectionSourceChanged {
            expected: expected.clone(),
            actual: page.context.source,
        });
    }

    let (pages_fetched, neurons_fetched) = crate::nns::governance::advance_collection_counts(
        state.collection.pages_fetched,
        state.neurons_fetched,
        u64::try_from(page.neurons.len())
            .map_err(|_| NnsNeuronError::CollectionAccountingOverflow)?,
    )
    .ok_or(NnsNeuronError::CollectionAccountingOverflow)?;
    let next_start_neuron_id = page.next_start_neuron_id;
    let next_state = NnsNeuronCollectionState {
        collection: state.collection.after_page(
            request,
            page.context.source.clone(),
            pages_fetched,
        ),
        neurons_fetched,
        next_start_neuron_id,
    };
    Ok(NnsNeuronCollectionStep {
        page,
        state: next_state,
    })
}

pub(super) fn validate_collection_state(
    state: &NnsNeuronCollectionState,
) -> Result<(), NnsNeuronError> {
    let invalid = |reason| NnsNeuronError::InvalidCollectionState { reason };
    state
        .collection
        .validate(NNS_NEURON_COLLECTION_STATE_SCHEMA_VERSION, invalid)?;
    validate_page_size(state.collection.page_size)?;

    let neurons_fetched = state.neurons_fetched;
    let maximum_rows = crate::nns::governance::validate_governance_collection_row_bounds(
        "neurons_fetched",
        neurons_fetched,
        state.collection.pages_fetched,
        state.collection.page_size,
        invalid,
    )?;

    let valid_lifecycle = crate::nns::governance::collection_lifecycle_matches(
        state.status(),
        state.collection.pages_fetched,
        state.collection.max_pages,
        state.neurons_fetched,
        maximum_rows,
        state.next_start_neuron_id.is_some(),
        state.collection.source.is_some(),
    ) && (state.status() != NnsGovernanceCollectionStatus::Complete
        || neurons_fetched < maximum_rows);
    if !valid_lifecycle {
        return Err(invalid(format!(
            "status {} disagrees with cursor, provenance, or counters",
            state.status()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
