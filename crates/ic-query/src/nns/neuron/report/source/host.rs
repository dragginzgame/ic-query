//! Module: nns::neuron::report::source::host
//!
//! Responsibility: query NNS neurons through the native replica adapter.
//! Does not own: report assembly, cache publication, or runtime blocking.
//! Boundary: maps native Governance responses into the portable source contract.

use super::{
    NnsNeuronPage, NnsNeuronSource, NnsNeuronSourceFuture, governance_result,
    map_neuron_info_error, neuron_page_from_wire, neuron_page_request, neuron_row_from_wire,
};
use crate::nns::{
    LiveNnsSource,
    governance::{NnsGovernanceRequest, NnsGovernanceSourceData, host_request},
    governance_query::query_nns_governance,
    neuron::report::{
        model::NnsNeuronRow,
        wire::{GetNeuronIndexResultWire, GetNeuronInfoResultWire},
    },
};

impl NnsNeuronSource for LiveNnsSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        Box::pin(async move {
            let (request, provenance) = host_request(request)?;
            let result: GetNeuronIndexResultWire = query_nns_governance(
                self,
                &request,
                "get_neuron_index",
                "GetNeuronIndexRequest",
                "GetNeuronIndexResult",
                &neuron_page_request(exclusive_start_neuron_id, page_size),
            )
            .await?;
            Ok(NnsGovernanceSourceData::new(
                neuron_page_from_wire(result, page_size)?,
                provenance,
            ))
        })
    }

    fn fetch_neuron<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
        Box::pin(async move {
            let (request, provenance) = host_request(request)?;
            let result: GetNeuronInfoResultWire = query_nns_governance(
                self,
                &request,
                "get_neuron_info",
                "nat64",
                "NeuronInfoResult",
                &neuron_id,
            )
            .await?;
            let wire = governance_result(result)
                .map_err(|error| map_neuron_info_error(error, neuron_id))?;
            Ok(NnsGovernanceSourceData::new(
                neuron_row_from_wire(wire)?,
                provenance,
            ))
        })
    }
}
