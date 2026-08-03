//! Module: nns::neuron::report::source
//!
//! Responsibility: expose the public NNS neuron source capability and assemble validated reports.
//! Does not own: Governance wire transport, cache publication, CLI parsing, or Dashboard analytics.
//! Boundary: keeps source-independent pagination, provenance, and report projection together.

mod live;

use super::{
    NNS_NEURON_FETCHED_BY, NNS_NEURON_INFO_REPORT_SCHEMA_VERSION,
    NNS_NEURON_LIST_REPORT_SCHEMA_VERSION, NNS_NEURON_MAX_PAGE_SIZE, NnsNeuronHostError,
    enforce_mainnet_network,
    model::{
        NnsNeuronInfoReport, NnsNeuronInfoRequest, NnsNeuronListReport, NnsNeuronListRequest,
        NnsNeuronRow,
    },
};
use crate::{
    ic_registry::MAINNET_GOVERNANCE_CANISTER_ID,
    nns::{LiveNnsSource, NnsSourceRequest},
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};

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

///
/// NnsNeuronSource
///
/// Source capability for public NNS Governance neuron views.
///

pub trait NnsNeuronSource {
    /// Fetch one page from the comprehensive public neuron index.
    fn fetch_neuron_page(
        &self,
        request: &NnsSourceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> Result<NnsNeuronPage, NnsNeuronHostError>;

    /// Fetch one public limited neuron view.
    fn fetch_neuron(
        &self,
        request: &NnsSourceRequest,
        neuron_id: u64,
    ) -> Result<NnsNeuronRow, NnsNeuronHostError>;
}

/// Build one live public NNS neuron-index page.
pub fn build_nns_neuron_list_report(
    request: &NnsNeuronListRequest,
) -> Result<NnsNeuronListReport, NnsNeuronHostError> {
    build_nns_neuron_list_report_with_source(request, &LiveNnsSource)
}

/// Build one public live NNS neuron detail report.
pub fn build_nns_neuron_info_report(
    request: &NnsNeuronInfoRequest,
) -> Result<NnsNeuronInfoReport, NnsNeuronHostError> {
    build_nns_neuron_info_report_with_source(request, &LiveNnsSource)
}

/// Build one public NNS neuron-index page from a custom source.
pub fn build_nns_neuron_list_report_with_source(
    request: &NnsNeuronListRequest,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronListReport, NnsNeuronHostError> {
    validate_page_size(request.limit)?;
    enforce_mainnet_network(&request.network)?;
    let provenance = live_provenance(request.now_unix_secs, &request.source_endpoint);
    let fetch_request = provenance.source_request();
    let page = source.fetch_neuron_page(
        &fetch_request,
        request.exclusive_start_neuron_id,
        request.limit,
    )?;
    validate_neuron_page(&page, request.exclusive_start_neuron_id, request.limit)?;
    Ok(list_report_from_rows(
        request,
        provenance,
        page.neurons,
        page.next_start_neuron_id,
        None,
    ))
}

/// Build one public NNS neuron detail report from a custom source.
pub fn build_nns_neuron_info_report_with_source(
    request: &NnsNeuronInfoRequest,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronInfoReport, NnsNeuronHostError> {
    enforce_mainnet_network(&request.network)?;
    let provenance = live_provenance(request.now_unix_secs, &request.source_endpoint);
    let neuron = source.fetch_neuron(&provenance.source_request(), request.neuron_id)?;
    if neuron.neuron_id != request.neuron_id {
        return Err(NnsNeuronHostError::InvalidPage {
            reason: format!(
                "source returned neuron {}, expected {}",
                neuron.neuron_id, request.neuron_id
            ),
        });
    }
    Ok(info_report_from_row(request, provenance, neuron))
}

///
/// NnsNeuronReportProvenance
///
/// Shared live or cached provenance used while assembling neuron reports.
///

#[derive(Clone)]
pub(super) struct NnsNeuronReportProvenance {
    pub(super) fetched_at: String,
    pub(super) source_endpoint: String,
    pub(super) fetched_by: String,
    pub(super) cache_path: Option<String>,
    pub(super) from_cache: bool,
}

impl NnsNeuronReportProvenance {
    fn source_request(&self) -> NnsSourceRequest {
        NnsSourceRequest::new(
            MAINNET_NETWORK,
            &self.source_endpoint,
            &self.fetched_at,
            &self.fetched_by,
        )
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
        schema_version: NNS_NEURON_LIST_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at: provenance.fetched_at,
        source_endpoint: provenance.source_endpoint,
        fetched_by: provenance.fetched_by,
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
        schema_version: NNS_NEURON_INFO_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at: provenance.fetched_at,
        source_endpoint: provenance.source_endpoint,
        fetched_by: provenance.fetched_by,
        cache_path: provenance.cache_path,
        from_cache: provenance.from_cache,
        verbose: request.verbose,
        neuron,
    }
}

pub(super) fn validate_page_size(page_size: u32) -> Result<(), NnsNeuronHostError> {
    if (1..=NNS_NEURON_MAX_PAGE_SIZE).contains(&page_size) {
        Ok(())
    } else {
        Err(NnsNeuronHostError::InvalidPageSize {
            page_size,
            max_page_size: NNS_NEURON_MAX_PAGE_SIZE,
        })
    }
}

pub(super) fn validate_neuron_rows(rows: &[NnsNeuronRow]) -> Result<(), NnsNeuronHostError> {
    if rows
        .windows(2)
        .any(|pair| pair[0].neuron_id >= pair[1].neuron_id)
    {
        return Err(NnsNeuronHostError::InvalidPage {
            reason: "neuron ids are not strictly ascending and unique".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_neuron_page(
    page: &NnsNeuronPage,
    exclusive_start_neuron_id: Option<u64>,
    page_size: u32,
) -> Result<(), NnsNeuronHostError> {
    if page.neurons.len() > page_size as usize {
        return Err(NnsNeuronHostError::InvalidPage {
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
        return Err(NnsNeuronHostError::InvalidPage {
            reason: format!("first neuron id {first} is not greater than cursor {start}"),
        });
    }
    let expected_next = (page.neurons.len() == page_size as usize)
        .then(|| page.neurons.last().map(|row| row.neuron_id))
        .flatten();
    if page.next_start_neuron_id != expected_next {
        return Err(NnsNeuronHostError::InvalidPage {
            reason: format!(
                "next cursor {:?} does not match expected {:?}",
                page.next_start_neuron_id, expected_next
            ),
        });
    }
    Ok(())
}

fn live_provenance(now_unix_secs: u64, source_endpoint: &str) -> NnsNeuronReportProvenance {
    NnsNeuronReportProvenance {
        fetched_at: format_utc_timestamp_secs(now_unix_secs),
        source_endpoint: source_endpoint.to_string(),
        fetched_by: NNS_NEURON_FETCHED_BY.to_string(),
        cache_path: None,
        from_cache: false,
    }
}
