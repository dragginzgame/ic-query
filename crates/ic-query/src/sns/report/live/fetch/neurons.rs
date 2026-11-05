//! Module: sns::report::live::fetch::neurons
//!
//! Responsibility: fetch SNS governance neurons.
//! Does not own: lookup resolution, cache storage, sorting, or rendering.
//! Boundary: queries bounded and paged neuron data for live reports and refreshes.
use crate::candid_call::query_canister;
use crate::report::CollectionSourceRequest;
use crate::sns::report::{
    SnsHostError, SnsNeuronRow,
    live::{
        convert::sns_neuron_row,
        fetch::governance_canister,
        query::{principal_from_text, sns_agent},
        types::{
            GetNeuronRequest, GetNeuronResponse, GetNeuronResult, ListNeuronsRequest,
            ListNeuronsResponse,
        },
    },
    source::{
        MainnetSnsNeuronPage, SnsNeuronId, sns_neuron_id_from_text, validate_mainnet_sns_neuron,
        validate_mainnet_sns_neuron_page,
    },
};
use crate::sns::report::{SnsNeuronSource, SnsNeuronsSource};

impl SnsNeuronSource for crate::sns::LiveSnsSource {
    fn fetch_sns_neuron<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        governance_canister_id: &'a str,
        neuron_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<SnsNeuronRow, SnsHostError>> {
        Box::pin(async move {
            let parsed_neuron_id = sns_neuron_id_from_text(neuron_id)?;
            let agent = sns_agent(self, request)?;
            let governance_canister = governance_canister(governance_canister_id)?;
            let method = "get_neuron";
            let response: GetNeuronResponse = query_canister::<_, _, SnsHostError>(
                agent,
                &governance_canister,
                method,
                "GetNeuronRequest",
                "GetNeuronResponse",
                &GetNeuronRequest {
                    neuron_id: Some(super::super::types::NeuronId {
                        id: parsed_neuron_id.id,
                    }),
                },
            )
            .await?;
            let neuron = match response
                .result
                .ok_or(SnsHostError::MissingGovernanceResult { method })?
            {
                GetNeuronResult::Neuron(neuron) => sns_neuron_row(*neuron)?,
                GetNeuronResult::Error(error) => {
                    return Err(SnsHostError::GovernanceError {
                        method,
                        error_type: error.error_type,
                        message: error.error_message,
                    });
                }
            };
            validate_mainnet_sns_neuron(&neuron, neuron_id)?;
            Ok(neuron)
        })
    }
}

impl SnsNeuronsSource for crate::sns::LiveSnsSource {
    fn fetch_sns_neurons<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        governance_canister_id: &'a str,
        limit: u32,
        owner_principal_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsNeuronRow>, SnsHostError>> {
        Box::pin(async move {
            let page = query_neuron_page(
                self,
                request,
                governance_canister_id,
                limit,
                None,
                owner_principal_id,
            )
            .await?;
            Ok(page.neurons)
        })
    }

    fn fetch_sns_neuron_page<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        governance_canister_id: &'a str,
        limit: u32,
        start_page_at: Option<&'a SnsNeuronId>,
        owner_principal_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsNeuronPage, SnsHostError>> {
        Box::pin(query_neuron_page(
            self,
            request,
            governance_canister_id,
            limit,
            start_page_at,
            owner_principal_id,
        ))
    }
}

async fn query_neuron_page(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
    governance_canister_id: &str,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeuronPage, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let governance_canister = governance_canister(governance_canister_id)?;
    let owner_principal = owner_principal_id
        .map(|principal| principal_from_text(principal, "owner_principal_id"))
        .transpose()?;
    let response: ListNeuronsResponse = query_canister::<_, _, SnsHostError>(
        agent,
        &governance_canister,
        "list_neurons",
        "ListNeuronsRequest",
        "ListNeuronsResponse",
        &ListNeuronsRequest {
            of_principal: owner_principal,
            limit,
            start_page_at: start_page_at.map(Into::into),
        },
    )
    .await?;
    let last_cursor = response
        .neurons
        .last()
        .and_then(|neuron| neuron.id.clone().map(Into::into));
    let neurons = response
        .neurons
        .into_iter()
        .map(sns_neuron_row)
        .collect::<Result<Vec<_>, _>>()?;
    let page = MainnetSnsNeuronPage {
        neurons,
        last_cursor,
    };
    validate_mainnet_sns_neuron_page(&page, limit, start_page_at)?;
    Ok(page)
}
