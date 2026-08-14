//! Module: nns::governance::source::host
//!
//! Responsibility: adapt replica query calls to the portable Governance source boundary.
//! Does not own: report assembly, process IO, or caller runtime policy.
//! Boundary: only this module converts the Governance request into the shared host request.

use super::{NnsGovernanceSource, NnsGovernanceSourceData, NnsGovernanceSourceFuture};
use crate::nns::{
    LiveNnsSource, NnsSourceRequest,
    governance::{
        NnsGovernanceEconomics, NnsGovernanceError, NnsGovernanceMaturityModulation,
        NnsGovernanceMetrics, NnsGovernanceRequest, NnsGovernanceRewardEvent,
        NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection,
        source::metrics_result,
        validation::validate_governance_request,
        wire::{GetMaturityModulationRequest, GetMaturityModulationResponse, GetMetricsResult},
    },
    governance_query::{query_nns_governance, query_nns_governance_no_args},
};

impl NnsGovernanceSource for LiveNnsSource {
    fn fetch_economics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceEconomics> {
        Box::pin(async move {
            let (request, provenance) = host_request(request)?;
            let economics = query_nns_governance_no_args(
                &request,
                "get_network_economics_parameters",
                "NetworkEconomics",
            )
            .await
            .map_err(NnsGovernanceError::from)?;
            Ok(NnsGovernanceSourceData::new(economics, provenance))
        })
    }

    fn fetch_metrics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceMetrics> {
        Box::pin(async move {
            let (request, provenance) = host_request(request)?;
            let result: GetMetricsResult =
                query_nns_governance_no_args(&request, "get_metrics", "GetMetricsResult")
                    .await
                    .map_err(NnsGovernanceError::from)?;
            Ok(NnsGovernanceSourceData::new(
                NnsGovernanceMetrics::from(metrics_result(result)?),
                provenance,
            ))
        })
    }

    fn fetch_reward_event<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceRewardEvent> {
        Box::pin(async move {
            let (request, provenance) = host_request(request)?;
            let reward_event =
                query_nns_governance_no_args(&request, "get_latest_reward_event", "RewardEvent")
                    .await
                    .map_err(NnsGovernanceError::from)?;
            Ok(NnsGovernanceSourceData::new(reward_event, provenance))
        })
    }

    fn fetch_maturity_modulation<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, Option<NnsGovernanceMaturityModulation>> {
        Box::pin(async move {
            let (request, provenance) = host_request(request)?;
            let response: GetMaturityModulationResponse = query_nns_governance(
                &request,
                "get_maturity_modulation",
                "GetMaturityModulationRequest",
                "GetMaturityModulationResponse",
                &GetMaturityModulationRequest {},
            )
            .await
            .map_err(NnsGovernanceError::from)?;
            Ok(NnsGovernanceSourceData::new(
                response.maturity_modulation,
                provenance,
            ))
        })
    }
}

fn host_request(
    request: &NnsGovernanceRequest,
) -> Result<(NnsSourceRequest, NnsGovernanceSourceProvenance), NnsGovernanceError> {
    validate_governance_request(request)?;
    let NnsGovernanceSourceSelection::ReplicaQuery {
        endpoint,
        fetched_by,
    } = &request.source
    else {
        return Err(NnsGovernanceError::InvalidSourceSelection {
            reason: "the native live adapter requires a replica_query source".to_string(),
        });
    };
    let request =
        NnsSourceRequest::new(&request.network, endpoint, &request.fetched_at, fetched_by);
    let provenance = NnsGovernanceSourceProvenance::ReplicaQuery {
        endpoint: endpoint.clone(),
        fetched_by: fetched_by.clone(),
    };
    Ok((request, provenance))
}
