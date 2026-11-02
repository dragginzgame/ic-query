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
    GovernanceCollectionState, NnsGovernanceCollectionStatus, NnsGovernanceRequest,
    NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection, validate_governance_continuation,
    validate_governance_request,
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
        self.collection
            .status(self.next_before_proposal_id.is_some())
    }

    /// Return whether Governance API exhaustion was observed.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self.status(), NnsGovernanceCollectionStatus::Complete)
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
    if let Some(expected) = &state.collection.source
        && *expected != page.context.source
    {
        return Err(NnsProposalError::CollectionSourceChanged {
            expected: expected.clone(),
            actual: page.context.source,
        });
    }

    let page_count = u32::try_from(page.proposals.len())
        .map_err(|_| NnsProposalError::CollectionAccountingOverflow)?;
    let (pages_fetched, proposals_fetched) = crate::nns::governance::advance_collection_counts(
        state.collection.pages_fetched,
        state.proposals_fetched,
        u64::from(page_count),
    )
    .ok_or(NnsProposalError::CollectionAccountingOverflow)?;
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
        collection: state.collection.after_page(
            request,
            page.context.source.clone(),
            pages_fetched,
        ),
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
    let maximum_rows = crate::nns::governance::validate_governance_collection_row_bounds(
        "proposals_fetched",
        proposals_fetched,
        state.collection.pages_fetched,
        state.collection.page_size,
        invalid,
    )?;

    if state
        .next_before_proposal_id
        .is_some_and(|cursor| cursor <= 1)
    {
        return Err(invalid(
            "next_before_proposal_id must be greater than one".to_string(),
        ));
    }

    let valid_lifecycle = crate::nns::governance::collection_lifecycle_matches(
        state.status(),
        state.collection.pages_fetched,
        state.collection.max_pages,
        state.proposals_fetched,
        maximum_rows,
        state.next_before_proposal_id.is_some(),
        state.collection.source.is_some(),
    );
    if !valid_lifecycle {
        return Err(invalid(format!(
            "status {} disagrees with cursor, provenance, or counters",
            state.status()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nns::governance::{NnsGovernanceError, validate_governance_time_interval};

    struct UnexpectedSource;
    impl NnsProposalSource for UnexpectedSource {
        fn fetch_proposals<'a>(
            &'a self,
            _: &'a NnsGovernanceRequest,
            _: u32,
            _: Option<u64>,
            _: crate::nns::proposals::NnsProposalStatusFilter,
            _: crate::nns::proposals::NnsProposalRewardStatusFilter,
        ) -> crate::nns::proposals::NnsProposalSourceFuture<
            'a,
            Vec<crate::nns::proposals::NnsProposalRow>,
        > {
            panic!("unexpected source call")
        }
        fn fetch_proposal<'a>(
            &'a self,
            _: &'a NnsGovernanceRequest,
            _: u64,
        ) -> crate::nns::proposals::NnsProposalSourceFuture<'a, crate::nns::proposals::NnsProposalRow>
        {
            panic!("unexpected source call")
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn collection_rejects_backward_continuations_and_restored_intervals() {
        let start = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 100);
        let state = NnsProposalCollectionState::new(&start, 2, 3).unwrap();
        let before = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 99);
        let mut future = std::pin::pin!(advance_nns_proposal_collection_with_source(
            &before,
            &state,
            &UnexpectedSource
        ));
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        let result = std::future::Future::poll(future.as_mut(), &mut context);
        assert!(matches!(
            result,
            std::task::Poll::Ready(Err(NnsProposalError::Governance(
                NnsGovernanceError::InvalidTimestampOrder { .. }
            )))
        ));
        let mut value = serde_json::to_value(&state).unwrap();
        value["updated_at"] = serde_json::json!(before.fetched_at);
        let restored = serde_json::from_value(value).unwrap();
        assert!(matches!(
            validate_collection_state(&restored),
            Err(NnsProposalError::Governance(
                NnsGovernanceError::InvalidTimestampOrder { .. }
            ))
        ));
        assert!(
            validate_governance_time_interval(
                "updated_at",
                &state.collection.updated_at,
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
        let valid_state = NnsProposalCollectionState::new(&request, 2, 3).expect("valid state");
        for (field, value) in [
            ("governance_canister_id", serde_json::json!("aaaaa-aa")),
            ("max_pages", serde_json::json!(0)),
            ("pages_fetched", serde_json::json!(4)),
        ] {
            let mut restored = serde_json::to_value(&valid_state).unwrap();
            restored[field] = value;
            let restored = serde_json::from_value(restored).unwrap();
            assert!(
                matches!(
                    validate_collection_state(&restored),
                    Err(NnsProposalError::InvalidCollectionState { .. })
                ),
                "invalid restored {field}"
            );
        }
        for timestamp in ["not a UTC timestamp", "+2023-11-14T22:13:20Z"] {
            request.fetched_at = timestamp.to_string();
            assert!(matches!(
                NnsProposalCollectionState::new(&request, 2, 3),
                Err(NnsProposalError::Governance(
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
                    Err(NnsProposalError::Governance(
                        NnsGovernanceError::InvalidTimestamp { value, .. }
                    )) if value == timestamp
                ));
            }
        }
    }
}
