//! Module: nns::neuron::report::collection
//!
//! Responsibility: advance caller-persisted complete public-neuron walks one page at a time.
//! Does not own: stable memory, filesystem caches, scheduling, retries, or publication.
//! Boundary: validates resumable state before and after exactly one bounded source call.

#[cfg(feature = "nns-host")]
use super::NnsNeuronHostError;
use super::{
    NnsNeuronError,
    model::{NnsNeuronListReport, NnsNeuronListRequest},
    source::{NnsNeuronSource, build_nns_neuron_list_report_with_source, validate_page_size},
};
use crate::nns::{
    MAINNET_GOVERNANCE_CANISTER_ID,
    governance::{
        NnsGovernanceCollectionStatus, NnsGovernanceRequest, NnsGovernanceSourceProvenance,
        NnsGovernanceSourceSelection, validate_governance_request,
        validate_governance_time_interval, validate_source_provenance,
    },
};
#[cfg(feature = "nns-host")]
use crate::{nns::LiveNnsSource, runtime::block_on_current_thread};
use serde::{Deserialize, Serialize};

/// Version of the persistable resumable NNS neuron collection state.
pub const NNS_NEURON_COLLECTION_STATE_SCHEMA_VERSION: u32 = 1;

///
/// NnsNeuronCollectionState
///
/// Serializable continuation state for an explicitly bounded public-neuron walk.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNeuronCollectionState {
    schema_version: u32,
    network: String,
    governance_canister_id: String,
    requested_source: NnsGovernanceSourceSelection,
    source: Option<NnsGovernanceSourceProvenance>,
    page_size: u32,
    max_pages: u32,
    pages_fetched: u32,
    neurons_fetched: u64,
    next_start_neuron_id: Option<u64>,
    started_at: String,
    updated_at: String,
    status: NnsGovernanceCollectionStatus,
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
            schema_version: NNS_NEURON_COLLECTION_STATE_SCHEMA_VERSION,
            network: request.network.clone(),
            governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
            requested_source: request.source.clone(),
            source: None,
            page_size,
            max_pages,
            pages_fetched: 0,
            neurons_fetched: 0,
            next_start_neuron_id: None,
            started_at: request.fetched_at.clone(),
            updated_at: request.fetched_at.clone(),
            status: NnsGovernanceCollectionStatus::Ready,
        })
    }

    /// Return the state schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Return the fixed network identity.
    #[must_use]
    pub fn network(&self) -> &str {
        &self.network
    }

    /// Return the fixed Governance canister identity.
    #[must_use]
    pub fn governance_canister_id(&self) -> &str {
        &self.governance_canister_id
    }

    /// Return the source selection fixed when collection started.
    #[must_use]
    pub const fn requested_source(&self) -> &NnsGovernanceSourceSelection {
        &self.requested_source
    }

    /// Return the concrete source provenance after the first admitted page.
    #[must_use]
    pub const fn source(&self) -> Option<&NnsGovernanceSourceProvenance> {
        self.source.as_ref()
    }

    /// Return the maximum rows requested from each Governance call.
    #[must_use]
    pub const fn page_size(&self) -> u32 {
        self.page_size
    }

    /// Return the cumulative source-call ceiling.
    #[must_use]
    pub const fn max_pages(&self) -> u32 {
        self.max_pages
    }

    /// Return the number of successfully admitted pages.
    #[must_use]
    pub const fn pages_fetched(&self) -> u32 {
        self.pages_fetched
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
        &self.started_at
    }

    /// Return the caller-supplied time attached to the latest admitted page.
    #[must_use]
    pub fn updated_at(&self) -> &str {
        &self.updated_at
    }

    /// Return the collection lifecycle status.
    #[must_use]
    pub const fn status(&self) -> NnsGovernanceCollectionStatus {
        self.status
    }

    /// Return whether Governance API exhaustion was observed.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self.status, NnsGovernanceCollectionStatus::Complete)
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
pub fn advance_nns_neuron_collection(
    request: &NnsGovernanceRequest,
    state: &NnsNeuronCollectionState,
) -> Result<NnsNeuronCollectionStep, NnsNeuronHostError> {
    Ok(block_on_current_thread(
        advance_nns_neuron_collection_with_source(request, state, &LiveNnsSource),
    )??)
}

/// Advance a resumable public-neuron walk by exactly one caller-runtime source call.
pub async fn advance_nns_neuron_collection_with_source(
    request: &NnsGovernanceRequest,
    state: &NnsNeuronCollectionState,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronCollectionStep, NnsNeuronError> {
    validate_governance_request(request)?;
    validate_collection_state(state)?;
    validate_continuation_request(request, state)?;
    validate_governance_time_interval(
        "updated_at",
        &state.updated_at,
        "fetched_at",
        &request.fetched_at,
    )?;
    match state.status {
        NnsGovernanceCollectionStatus::Complete => {
            return Err(NnsNeuronError::CollectionComplete {
                pages_fetched: state.pages_fetched,
            });
        }
        NnsGovernanceCollectionStatus::PageLimitReached => {
            return Err(NnsNeuronError::CollectionPageLimitReached {
                pages_fetched: state.pages_fetched,
                max_pages: state.max_pages,
            });
        }
        NnsGovernanceCollectionStatus::Ready | NnsGovernanceCollectionStatus::Collecting => {}
    }

    let mut page_request = NnsNeuronListRequest::new(request.clone(), state.page_size);
    page_request.exclusive_start_neuron_id = state.next_start_neuron_id;
    let page = build_nns_neuron_list_report_with_source(&page_request, source).await?;
    if let Some(expected) = &state.source
        && *expected != page.context.source
    {
        return Err(NnsNeuronError::CollectionSourceChanged {
            expected: expected.clone(),
            actual: page.context.source,
        });
    }

    let pages_fetched = state
        .pages_fetched
        .checked_add(1)
        .ok_or(NnsNeuronError::CollectionAccountingOverflow)?;
    let page_row_count = u64::try_from(page.returned_neuron_count)
        .map_err(|_| NnsNeuronError::CollectionAccountingOverflow)?;
    let neurons_fetched = state
        .neurons_fetched
        .checked_add(page_row_count)
        .ok_or(NnsNeuronError::CollectionAccountingOverflow)?;
    let next_start_neuron_id = page.next_start_neuron_id;
    let status = if next_start_neuron_id.is_none() {
        NnsGovernanceCollectionStatus::Complete
    } else if pages_fetched == state.max_pages {
        NnsGovernanceCollectionStatus::PageLimitReached
    } else {
        NnsGovernanceCollectionStatus::Collecting
    };
    let next_state = NnsNeuronCollectionState {
        source: Some(page.context.source.clone()),
        pages_fetched,
        neurons_fetched,
        next_start_neuron_id,
        updated_at: request.fetched_at.clone(),
        status,
        ..state.clone()
    };
    validate_collection_state(&next_state)?;
    Ok(NnsNeuronCollectionStep {
        page,
        state: next_state,
    })
}

fn validate_continuation_request(
    request: &NnsGovernanceRequest,
    state: &NnsNeuronCollectionState,
) -> Result<(), NnsNeuronError> {
    if request.network != state.network {
        return Err(NnsNeuronError::CollectionRequestMismatch {
            field: "network",
            expected: state.network.clone(),
            actual: request.network.clone(),
        });
    }
    if request.source != state.requested_source {
        return Err(NnsNeuronError::CollectionRequestMismatch {
            field: "requested_source",
            expected: format!("{:?}", state.requested_source),
            actual: format!("{:?}", request.source),
        });
    }
    Ok(())
}

pub(super) fn validate_collection_state(
    state: &NnsNeuronCollectionState,
) -> Result<(), NnsNeuronError> {
    let invalid = |reason| NnsNeuronError::InvalidCollectionState { reason };
    if state.schema_version != NNS_NEURON_COLLECTION_STATE_SCHEMA_VERSION {
        return Err(invalid(format!(
            "schema_version is {}, expected {NNS_NEURON_COLLECTION_STATE_SCHEMA_VERSION}",
            state.schema_version
        )));
    }
    if state.governance_canister_id != MAINNET_GOVERNANCE_CANISTER_ID {
        return Err(invalid(format!(
            "governance_canister_id is {}, expected {MAINNET_GOVERNANCE_CANISTER_ID}",
            state.governance_canister_id
        )));
    }
    let state_request = NnsGovernanceRequest {
        network: state.network.clone(),
        fetched_at: state.started_at.clone(),
        source: state.requested_source.clone(),
    };
    validate_governance_request(&state_request)?;
    validate_governance_time_interval(
        "started_at",
        &state.started_at,
        "updated_at",
        &state.updated_at,
    )?;
    validate_page_size(state.page_size)?;
    if state.max_pages == 0 {
        return Err(invalid("max_pages must be greater than zero".to_string()));
    }
    if state.pages_fetched > state.max_pages {
        return Err(invalid(format!(
            "pages_fetched {} exceeds max_pages {}",
            state.pages_fetched, state.max_pages
        )));
    }
    if let Some(source) = &state.source {
        validate_source_provenance(&state.requested_source, source)?;
    }

    let page_size = u64::from(state.page_size);
    let pages_fetched = u64::from(state.pages_fetched);
    let neurons_fetched = state.neurons_fetched;
    let maximum_rows = pages_fetched
        .checked_mul(page_size)
        .ok_or(NnsNeuronError::CollectionAccountingOverflow)?;
    let minimum_rows = pages_fetched
        .saturating_sub(1)
        .checked_mul(page_size)
        .ok_or(NnsNeuronError::CollectionAccountingOverflow)?;
    if neurons_fetched < minimum_rows || neurons_fetched > maximum_rows {
        return Err(invalid(format!(
            "neurons_fetched {} is outside {}..={} for {} pages of size {}",
            state.neurons_fetched, minimum_rows, maximum_rows, state.pages_fetched, state.page_size
        )));
    }

    let valid_lifecycle = match state.status {
        NnsGovernanceCollectionStatus::Ready => {
            state.pages_fetched == 0
                && state.neurons_fetched == 0
                && state.next_start_neuron_id.is_none()
                && state.source.is_none()
        }
        NnsGovernanceCollectionStatus::Collecting => {
            state.pages_fetched > 0
                && state.pages_fetched < state.max_pages
                && neurons_fetched == maximum_rows
                && state.next_start_neuron_id.is_some()
                && state.source.is_some()
        }
        NnsGovernanceCollectionStatus::Complete => {
            state.pages_fetched > 0
                && neurons_fetched < maximum_rows
                && state.next_start_neuron_id.is_none()
                && state.source.is_some()
        }
        NnsGovernanceCollectionStatus::PageLimitReached => {
            state.pages_fetched == state.max_pages
                && neurons_fetched == maximum_rows
                && state.next_start_neuron_id.is_some()
                && state.source.is_some()
        }
    };
    if !valid_lifecycle {
        return Err(invalid(format!(
            "status {} disagrees with cursor, provenance, or counters",
            state.status
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nns::governance::NnsGovernanceError;

    struct UnexpectedSource;
    impl NnsNeuronSource for UnexpectedSource {
        fn fetch_neuron_page<'a>(
            &'a self,
            _: &'a NnsGovernanceRequest,
            _: Option<u64>,
            _: u32,
        ) -> crate::nns::neuron::NnsNeuronSourceFuture<'a, crate::nns::neuron::NnsNeuronPage>
        {
            panic!("unexpected source call")
        }
        fn fetch_neuron<'a>(
            &'a self,
            _: &'a NnsGovernanceRequest,
            _: u64,
        ) -> crate::nns::neuron::NnsNeuronSourceFuture<'a, crate::nns::neuron::NnsNeuronRow>
        {
            panic!("unexpected source call")
        }
    }

    #[test]
    fn collection_rejects_backward_continuations_and_restored_intervals() {
        let start = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 100);
        let state = NnsNeuronCollectionState::new(&start, 2, 3).unwrap();
        let before = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 99);
        let mut future = std::pin::pin!(advance_nns_neuron_collection_with_source(
            &before,
            &state,
            &UnexpectedSource
        ));
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        let result = std::future::Future::poll(future.as_mut(), &mut context);
        assert!(matches!(
            result,
            std::task::Poll::Ready(Err(NnsNeuronError::Governance(
                NnsGovernanceError::InvalidTimestampOrder { .. }
            )))
        ));
        let mut value = serde_json::to_value(&state).unwrap();
        value["updated_at"] = serde_json::json!(before.fetched_at);
        let restored = serde_json::from_value(value).unwrap();
        assert!(matches!(
            validate_collection_state(&restored),
            Err(NnsNeuronError::Governance(
                NnsGovernanceError::InvalidTimestampOrder { .. }
            ))
        ));
        assert!(
            validate_governance_time_interval(
                "updated_at",
                &state.updated_at,
                "fetched_at",
                &start.fetched_at
            )
            .is_ok()
        );
    }

    #[test]
    fn collection_validates_request_and_restored_timestamps() {
        let mut request = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs(
            "ic",
            1_700_000_000,
        );
        let valid_state = NnsNeuronCollectionState::new(&request, 2, 3).expect("valid state");
        for timestamp in ["not a UTC timestamp", "+2023-11-14T22:13:20Z"] {
            request.fetched_at = timestamp.to_string();
            assert!(matches!(
                NnsNeuronCollectionState::new(&request, 2, 3),
                Err(NnsNeuronError::Governance(
                    NnsGovernanceError::InvalidTimestamp {
                        field: "fetched_at",
                        ..
                    }
                ))
            ));
            for field in ["started_at", "updated_at"] {
                let mut value = serde_json::to_value(&valid_state).expect("serialize state");
                value[field] = serde_json::json!(timestamp);
                let state = serde_json::from_value(value).expect("restore caller state");
                assert!(matches!(
                    validate_collection_state(&state),
                    Err(NnsNeuronError::Governance(
                        NnsGovernanceError::InvalidTimestamp { value, .. }
                    )) if value == timestamp
                ));
            }
        }
    }
}
