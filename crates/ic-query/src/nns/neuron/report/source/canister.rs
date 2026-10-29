//! Module: nns::neuron::report::source::canister
//!
//! Responsibility: call bounded NNS neuron APIs from replicated canister execution.
//! Does not own: scheduling, retries, persistence, report assembly, or view policy.
//! Boundary: maps shared canister transport responses into neuron source data.

use super::{
    NnsNeuronPage, NnsNeuronSource, NnsNeuronSourceFuture, governance_result,
    map_neuron_info_error, neuron_page_from_wire, neuron_page_request, neuron_row_from_wire,
};
use crate::nns::{
    governance::{
        CanisterNnsSource, NnsGovernanceRequest, NnsGovernanceSourceData, call_with_arg,
        canister_provenance,
    },
    neuron::report::{
        model::NnsNeuronRow,
        wire::{GetNeuronIndexResultWire, GetNeuronInfoResultWire},
    },
};

impl NnsNeuronSource for CanisterNnsSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        Box::pin(async move {
            let provenance = canister_provenance(request)?;
            let result: GetNeuronIndexResultWire = call_with_arg(
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
            let provenance = canister_provenance(request)?;
            let result: GetNeuronInfoResultWire =
                call_with_arg("get_neuron_info", "nat64", "NeuronInfoResult", &neuron_id).await?;
            let wire = governance_result(result)
                .map_err(|error| map_neuron_info_error(error, neuron_id))?;
            Ok(NnsGovernanceSourceData::new(
                neuron_row_from_wire(*wire)?,
                provenance,
            ))
        })
    }
}
