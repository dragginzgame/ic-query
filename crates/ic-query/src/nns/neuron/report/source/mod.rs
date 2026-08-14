//! Module: nns::neuron::report::source
//!
//! Responsibility: build NNS neuron reports from a portable async source.
//! Does not own: CLI parsing, cache IO, transport internals, or text rendering.
//! Boundary: native, canister, and custom sources converge before report assembly.

#[cfg(all(feature = "canister", target_arch = "wasm32"))]
mod canister;
#[cfg(feature = "nns-host")]
mod host;

use super::{
    NNS_NEURON_INFO_REPORT_SCHEMA_VERSION, NNS_NEURON_LIST_REPORT_SCHEMA_VERSION,
    NNS_NEURON_MAX_PAGE_SIZE, NnsNeuronError,
    classification::{NnsNeuronState, NnsNeuronType, NnsNeuronVisibility, NnsNeuronVote},
    model::{
        NnsNeuronInfoReport, NnsNeuronInfoRequest, NnsNeuronListReport, NnsNeuronListRequest,
        NnsNeuronRow,
    },
};
#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32"),
    test
))]
use super::{
    model::{NnsKnownNeuronData, NnsNeuronBallotRow},
    wire::{GovernanceResult, NeuronInfoWire},
};
use crate::nns::{
    MAINNET_GOVERNANCE_CANISTER_ID,
    governance::{
        NnsGovernanceReportContext, NnsGovernanceRequest, NnsGovernanceSourceData,
        NnsGovernanceSourceProvenance, validate_governance_request, validate_source_provenance,
    },
};
#[cfg(feature = "nns-host")]
use crate::{nns::LiveNnsSource, runtime::block_on_current_thread};
use std::{future::Future, pin::Pin};

#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32"),
    test
))]
const GOVERNANCE_ERROR_TYPE_NOT_FOUND: i32 = 4;

///
/// NnsNeuronPage
///
/// One canonically ascending page from the public Governance neuron index.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNeuronPage {
    /// Public neuron rows returned by Governance.
    pub neurons: Vec<NnsNeuronRow>,
    /// Exclusive lower bound for a possible next page.
    pub next_start_neuron_id: Option<u64>,
}

/// Build one bounded neuron-index page through the native replica adapter.
#[cfg(feature = "nns-host")]
pub fn build_nns_neuron_list_report(
    request: &NnsNeuronListRequest,
) -> Result<NnsNeuronListReport, super::NnsNeuronHostError> {
    Ok(block_on_current_thread(
        build_nns_neuron_list_report_with_source(request, &LiveNnsSource),
    )??)
}

/// Build one exact neuron detail through the native replica adapter.
#[cfg(feature = "nns-host")]
pub fn build_nns_neuron_info_report(
    request: &NnsNeuronInfoRequest,
) -> Result<NnsNeuronInfoReport, super::NnsNeuronHostError> {
    Ok(block_on_current_thread(
        build_nns_neuron_info_report_with_source(request, &LiveNnsSource),
    )??)
}

/// Build one bounded NNS neuron-index page from a caller-owned async source.
pub async fn build_nns_neuron_list_report_with_source(
    request: &NnsNeuronListRequest,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronListReport, NnsNeuronError> {
    validate_governance_request(&request.governance)?;
    validate_page_size(request.limit)?;
    let data = source
        .fetch_neuron_page(
            &request.governance,
            request.exclusive_start_neuron_id,
            request.limit,
        )
        .await?;
    validate_source_provenance(&request.governance.source, &data.provenance)?;
    validate_neuron_page(
        &data.value,
        request.exclusive_start_neuron_id,
        request.limit,
    )?;
    Ok(list_report_from_rows(
        request,
        NnsNeuronReportProvenance::live(report_context(&request.governance, data.provenance)),
        data.value.neurons,
        data.value.next_start_neuron_id,
        None,
    ))
}

/// Build one exact NNS neuron detail from a caller-owned async source.
pub async fn build_nns_neuron_info_report_with_source(
    request: &NnsNeuronInfoRequest,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronInfoReport, NnsNeuronError> {
    validate_governance_request(&request.governance)?;
    let data = source
        .fetch_neuron(&request.governance, request.neuron_id)
        .await?;
    validate_source_provenance(&request.governance.source, &data.provenance)?;
    if data.value.neuron_id != request.neuron_id {
        return Err(NnsNeuronError::InvalidResponse {
            reason: format!(
                "source returned neuron {}, expected {}",
                data.value.neuron_id, request.neuron_id
            ),
        });
    }
    validate_neuron_rows(std::slice::from_ref(&data.value))?;
    Ok(info_report_from_row(
        request,
        NnsNeuronReportProvenance::live(report_context(&request.governance, data.provenance)),
        data.value,
    ))
}

///
/// NnsNeuronSourceFuture
///
/// Boxed caller-runtime future returned by a neuron source.
///

pub type NnsNeuronSourceFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<NnsGovernanceSourceData<T>, NnsNeuronError>> + Send + 'a>>;

///
/// NnsNeuronSource
///
/// Portable async capability for bounded neuron list and exact detail calls.
///

pub trait NnsNeuronSource: Send + Sync {
    /// Fetch at most one bounded page from the public neuron index.
    fn fetch_neuron_page<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage>;

    /// Fetch one exact public limited neuron view.
    fn fetch_neuron<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow>;
}

///
/// NnsNeuronReportProvenance
///
/// Shared live or cached provenance used while assembling neuron reports.
///

#[derive(Clone)]
pub(super) struct NnsNeuronReportProvenance {
    pub(super) context: NnsGovernanceReportContext,
    pub(super) cache_path: Option<String>,
    pub(super) from_cache: bool,
}

impl NnsNeuronReportProvenance {
    const fn live(context: NnsGovernanceReportContext) -> Self {
        Self {
            context,
            cache_path: None,
            from_cache: false,
        }
    }
}

pub(super) fn list_report_from_rows(
    request: &NnsNeuronListRequest,
    provenance: NnsNeuronReportProvenance,
    neurons: Vec<NnsNeuronRow>,
    next_start_neuron_id: Option<u64>,
    total_neuron_count: Option<usize>,
) -> NnsNeuronListReport {
    NnsNeuronListReport {
        context: NnsGovernanceReportContext {
            schema_version: NNS_NEURON_LIST_REPORT_SCHEMA_VERSION,
            ..provenance.context
        },
        cache_path: provenance.cache_path,
        from_cache: provenance.from_cache,
        requested_limit: request.limit,
        exclusive_start_neuron_id: request.exclusive_start_neuron_id,
        next_start_neuron_id,
        total_neuron_count,
        point_in_time_guaranteed: false,
        returned_neuron_count: neurons.len(),
        verbose: request.verbose,
        neurons,
    }
}

pub(super) fn info_report_from_row(
    request: &NnsNeuronInfoRequest,
    provenance: NnsNeuronReportProvenance,
    neuron: NnsNeuronRow,
) -> NnsNeuronInfoReport {
    NnsNeuronInfoReport {
        context: NnsGovernanceReportContext {
            schema_version: NNS_NEURON_INFO_REPORT_SCHEMA_VERSION,
            ..provenance.context
        },
        cache_path: provenance.cache_path,
        from_cache: provenance.from_cache,
        verbose: request.verbose,
        neuron,
    }
}

pub(super) fn validate_page_size(page_size: u32) -> Result<(), NnsNeuronError> {
    if (1..=NNS_NEURON_MAX_PAGE_SIZE).contains(&page_size) {
        Ok(())
    } else {
        Err(NnsNeuronError::InvalidPageSize {
            page_size,
            max_page_size: NNS_NEURON_MAX_PAGE_SIZE,
        })
    }
}

pub(super) fn validate_neuron_rows(rows: &[NnsNeuronRow]) -> Result<(), NnsNeuronError> {
    for row in rows {
        if row.state_text != NnsNeuronState::from_code(row.state) {
            return Err(NnsNeuronError::InvalidResponse {
                reason: format!(
                    "neuron {} state classification {} does not match raw code {}",
                    row.neuron_id, row.state_text, row.state
                ),
            });
        }
        if row.visibility_text != NnsNeuronVisibility::from_code(row.visibility) {
            return Err(NnsNeuronError::InvalidResponse {
                reason: format!(
                    "neuron {} visibility classification {} does not match raw code {:?}",
                    row.neuron_id, row.visibility_text, row.visibility
                ),
            });
        }
        if row.neuron_type_text != NnsNeuronType::from_code(row.neuron_type) {
            return Err(NnsNeuronError::InvalidResponse {
                reason: format!(
                    "neuron {} type classification {} does not match raw code {:?}",
                    row.neuron_id, row.neuron_type_text, row.neuron_type
                ),
            });
        }
        if let Some(ballot) = row
            .recent_ballots
            .iter()
            .find(|ballot| ballot.vote_text != NnsNeuronVote::from_code(ballot.vote))
        {
            return Err(NnsNeuronError::InvalidResponse {
                reason: format!(
                    "neuron {} ballot vote classification {} does not match raw code {}",
                    row.neuron_id, ballot.vote_text, ballot.vote
                ),
            });
        }
    }
    if rows
        .windows(2)
        .any(|pair| pair[0].neuron_id >= pair[1].neuron_id)
    {
        return Err(NnsNeuronError::InvalidResponse {
            reason: "neuron ids are not strictly ascending and unique".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_neuron_page(
    page: &NnsNeuronPage,
    exclusive_start_neuron_id: Option<u64>,
    page_size: u32,
) -> Result<(), NnsNeuronError> {
    if page.neurons.len() > page_size as usize {
        return Err(NnsNeuronError::InvalidResponse {
            reason: format!(
                "source returned {} rows for page size {page_size}",
                page.neurons.len()
            ),
        });
    }
    validate_neuron_rows(&page.neurons)?;
    if let (Some(start), Some(first)) = (
        exclusive_start_neuron_id,
        page.neurons.first().map(|row| row.neuron_id),
    ) && first <= start
    {
        return Err(NnsNeuronError::InvalidResponse {
            reason: format!("first neuron id {first} is not greater than cursor {start}"),
        });
    }
    let expected_next = (page.neurons.len() == page_size as usize)
        .then(|| page.neurons.last().map(|row| row.neuron_id))
        .flatten();
    if page.next_start_neuron_id != expected_next {
        return Err(NnsNeuronError::InvalidResponse {
            reason: format!(
                "next cursor {:?} does not match expected {:?}",
                page.next_start_neuron_id, expected_next
            ),
        });
    }
    Ok(())
}

fn report_context(
    request: &NnsGovernanceRequest,
    source: NnsGovernanceSourceProvenance,
) -> NnsGovernanceReportContext {
    NnsGovernanceReportContext {
        schema_version: 1,
        network: request.network.clone(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        source,
    }
}

#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32"),
    test
))]
pub(in crate::nns::neuron::report) fn governance_result<Response>(
    result: impl GovernanceResult<Response>,
) -> Result<Response, NnsNeuronError> {
    result
        .into_result()
        .map_err(|error| NnsNeuronError::GovernanceResponse {
            error_type: error.error_type,
            message: error.error_message,
        })
}

#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32"),
    test
))]
pub(in crate::nns::neuron::report) fn map_neuron_info_error(
    error: NnsNeuronError,
    neuron_id: u64,
) -> NnsNeuronError {
    match error {
        NnsNeuronError::GovernanceResponse { error_type, .. }
            if error_type == GOVERNANCE_ERROR_TYPE_NOT_FOUND =>
        {
            NnsNeuronError::NeuronNotFound { neuron_id }
        }
        error => error,
    }
}

#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32"),
    test
))]
pub(in crate::nns::neuron::report) fn neuron_row_from_wire(
    wire: NeuronInfoWire,
) -> Result<NnsNeuronRow, NnsNeuronError> {
    let neuron_id = wire.id.ok_or(NnsNeuronError::MissingNeuronId)?.id;
    Ok(NnsNeuronRow {
        neuron_id,
        state: wire.state,
        state_text: NnsNeuronState::from_code(wire.state),
        visibility: wire.visibility,
        visibility_text: NnsNeuronVisibility::from_code(wire.visibility),
        neuron_type: wire.neuron_type,
        neuron_type_text: NnsNeuronType::from_code(wire.neuron_type),
        stake_e8s: wire.stake_e8s,
        staked_maturity_e8s_equivalent: wire.staked_maturity_e8s_equivalent,
        dissolve_delay_seconds: wire.dissolve_delay_seconds,
        age_seconds: wire.age_seconds,
        created_timestamp_seconds: wire.created_timestamp_seconds,
        retrieved_at_timestamp_seconds: wire.retrieved_at_timestamp_seconds,
        voting_power: wire.voting_power,
        deciding_voting_power: wire.deciding_voting_power,
        potential_voting_power: wire.potential_voting_power,
        voting_power_refreshed_timestamp_seconds: wire.voting_power_refreshed_timestamp_seconds,
        joined_community_fund_timestamp_seconds: wire.joined_community_fund_timestamp_seconds,
        eight_year_gang_bonus_base_e8s: wire.eight_year_gang_bonus_base_e8s,
        known_neuron_data: wire.known_neuron_data.map(|known| NnsKnownNeuronData {
            name: known.name,
            description: known.description,
            links: known.links.unwrap_or_default(),
        }),
        recent_ballots: wire
            .recent_ballots
            .into_iter()
            .map(|ballot| NnsNeuronBallotRow {
                proposal_id: ballot.proposal_id.map(|proposal| proposal.id),
                vote: ballot.vote,
                vote_text: NnsNeuronVote::from_code(ballot.vote),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::{NnsNeuronError, map_neuron_info_error};

    #[test]
    fn neuron_info_maps_only_the_native_not_found_error() {
        let not_found = map_neuron_info_error(
            NnsNeuronError::GovernanceResponse {
                error_type: 4,
                message: "wording is not part of the contract".to_string(),
            },
            42,
        );
        assert!(matches!(
            not_found,
            NnsNeuronError::NeuronNotFound { neuron_id: 42 }
        ));

        let unrelated = map_neuron_info_error(
            NnsNeuronError::GovernanceResponse {
                error_type: 12,
                message: "not found text must not override the code".to_string(),
            },
            42,
        );
        assert!(matches!(
            unrelated,
            NnsNeuronError::GovernanceResponse { error_type: 12, .. }
        ));
    }
}
