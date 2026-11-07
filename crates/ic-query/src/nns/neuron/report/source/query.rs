//! Typed Governance operations shared by the native and canister adapters.

use super::{
    NnsNeuronPage, NnsNeuronSource, NnsNeuronSourceFuture, neuron_page_from_wire,
    neuron_page_request, neuron_row_from_response,
};
use crate::nns::{
    governance::{GovernanceQuery, NnsGovernanceRequest, NnsGovernanceSourceData},
    neuron::report::{
        model::NnsNeuronRow,
        wire::{GetNeuronIndexResultWire, GetNeuronInfoResultWire},
    },
};

impl<T: GovernanceQuery> NnsNeuronSource for T {
    fn fetch_neuron_page<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        Box::pin(async move {
            let result: NnsGovernanceSourceData<GetNeuronIndexResultWire> = self
                .fetch(
                    request,
                    "get_neuron_index",
                    "GetNeuronIndexRequest",
                    "GetNeuronIndexResult",
                    (neuron_page_request(exclusive_start_neuron_id, page_size),),
                )
                .await?;
            Ok(NnsGovernanceSourceData::new(
                neuron_page_from_wire(result.value, page_size)?,
                result.provenance,
            ))
        })
    }

    fn fetch_neuron<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
        Box::pin(async move {
            let result: NnsGovernanceSourceData<GetNeuronInfoResultWire> = self
                .fetch(
                    request,
                    "get_neuron_info",
                    "nat64",
                    "NeuronInfoResult",
                    (neuron_id,),
                )
                .await?;
            Ok(NnsGovernanceSourceData::new(
                neuron_row_from_response(result.value, neuron_id)?,
                result.provenance,
            ))
        })
    }
}
