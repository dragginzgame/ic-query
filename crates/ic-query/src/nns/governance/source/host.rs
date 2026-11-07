//! Native Governance admission and replica-query transport.

use super::GovernanceQuery;
use crate::{
    nns::{
        LiveNnsSource, MAINNET_GOVERNANCE_CANISTER_ID,
        governance::{
            MAX_NNS_GOVERNANCE_RESPONSE_BYTES, NnsGovernanceError, NnsGovernanceRequest,
            NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection,
            validation::validate_governance_request,
        },
    },
    report::CollectionSourceRequest,
};
use ic_platform::candid::contract::{ArgumentEncoder, CandidType, Principal, serde::Deserialize};

impl GovernanceQuery for LiveNnsSource {
    type Request = CollectionSourceRequest;

    fn prepare(
        &self,
        request: &NnsGovernanceRequest,
    ) -> Result<(Self::Request, NnsGovernanceSourceProvenance), NnsGovernanceError> {
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
        let request = CollectionSourceRequest::new(
            &request.network,
            endpoint,
            &request.fetched_at,
            fetched_by,
        );
        let provenance = NnsGovernanceSourceProvenance::ReplicaQuery {
            endpoint: endpoint.clone(),
            fetched_by: fetched_by.clone(),
        };
        Ok((request, provenance))
    }

    async fn query<Args, Response>(
        &self,
        request: &Self::Request,
        method: &'static str,
        request_message: &'static str,
        response_message: &'static str,
        args: Args,
    ) -> Result<Response, NnsGovernanceError>
    where
        Args: ArgumentEncoder + Send,
        Response: for<'de> Deserialize<'de> + CandidType,
    {
        let arg = crate::candid_call::encode_request(args, request_message)?;
        let agent = crate::agent::endpoint_agent(
            &self.agent,
            &request.endpoint,
            MAX_NNS_GOVERNANCE_RESPONSE_BYTES,
            |reason| NnsGovernanceError::AgentBuild {
                endpoint: request.endpoint.clone(),
                reason,
            },
        )?;
        let canister = Principal::from_text(MAINNET_GOVERNANCE_CANISTER_ID).map_err(|error| {
            NnsGovernanceError::CandidDecode {
                message: "governance_canister_id",
                reason: error.to_string(),
            }
        })?;
        crate::candid_call::query_encoded(agent, &canister, method, response_message, arg).await
    }
}
