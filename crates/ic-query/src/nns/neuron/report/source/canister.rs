//! Module: nns::neuron::report::source::canister
//!
//! Responsibility: call bounded NNS neuron APIs from replicated canister execution.
//! Does not own: scheduling, retries, persistence, report assembly, or view policy.
//! Boundary: maps shared canister transport responses into neuron source data.

use super::{
    NnsNeuronPage, NnsNeuronSource, NnsNeuronSourceFuture, governance_result,
    map_neuron_info_error, neuron_row_from_wire,
};
use crate::nns::{
    governance::{
        CanisterNnsSource, NnsGovernanceRequest, NnsGovernanceSourceData, call_with_arg,
        canister_provenance,
    },
    neuron::report::{
        model::NnsNeuronRow,
        wire::{
            GetNeuronIndexRequestWire, GetNeuronIndexResultWire, GetNeuronInfoResultWire,
            NeuronIdWire,
        },
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
                &GetNeuronIndexRequestWire {
                    exclusive_start_neuron_id: exclusive_start_neuron_id
                        .map(|id| NeuronIdWire { id }),
                    page_size: Some(page_size),
                },
            )
            .await?;
            let neurons = governance_result(result)?
                .neurons
                .into_iter()
                .map(neuron_row_from_wire)
                .collect::<Result<Vec<_>, _>>()?;
            let next_start_neuron_id = (neurons.len() == page_size as usize)
                .then(|| neurons.last().map(|row| row.neuron_id))
                .flatten();
            Ok(NnsGovernanceSourceData::new(
                NnsNeuronPage {
                    neurons,
                    next_start_neuron_id,
                },
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
                neuron_row_from_wire(wire)?,
                provenance,
            ))
        })
    }
}
