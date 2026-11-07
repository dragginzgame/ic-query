//! Typed Governance operations shared by native and replicated transports.

use super::{NnsGovernanceSource, NnsGovernanceSourceData, NnsGovernanceSourceFuture};
use crate::nns::governance::{
    NnsGovernanceEconomics, NnsGovernanceError, NnsGovernanceMaturityModulation,
    NnsGovernanceMetrics, NnsGovernanceRequest, NnsGovernanceRewardEvent,
    NnsGovernanceSourceProvenance,
    wire::{
        GetMaturityModulationRequest, GetMaturityModulationResponse, GetMetricsResult,
        NetworkEconomics, RewardEvent,
    },
};
use ic_platform::candid::contract::{ArgumentEncoder, CandidType, serde::Deserialize};
use std::future::Future;

///
/// GovernanceQuery
///
/// Private transport seam retaining adapter-specific request admission and calls.
///

pub(in crate::nns) trait GovernanceQuery: Send + Sync {
    type Request: Send + Sync;

    fn prepare(
        &self,
        request: &NnsGovernanceRequest,
    ) -> Result<(Self::Request, NnsGovernanceSourceProvenance), NnsGovernanceError>;

    fn query<Args, Response>(
        &self,
        request: &Self::Request,
        method: &'static str,
        request_message: &'static str,
        response_message: &'static str,
        args: Args,
    ) -> impl Future<Output = Result<Response, NnsGovernanceError>> + Send
    where
        Args: ArgumentEncoder + Send,
        Response: for<'de> Deserialize<'de> + CandidType;

    fn fetch<Args, Response>(
        &self,
        request: &NnsGovernanceRequest,
        method: &'static str,
        request_message: &'static str,
        response_message: &'static str,
        args: Args,
    ) -> impl Future<Output = Result<NnsGovernanceSourceData<Response>, NnsGovernanceError>> + Send
    where
        Args: ArgumentEncoder + Send,
        Response: for<'de> Deserialize<'de> + CandidType,
    {
        async move {
            let (request, provenance) = self.prepare(request)?;
            let value = self
                .query(&request, method, request_message, response_message, args)
                .await?;
            Ok(NnsGovernanceSourceData::new(value, provenance))
        }
    }
}

impl<T: GovernanceQuery> NnsGovernanceSource for T {
    fn fetch_economics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceEconomics> {
        Box::pin(async move {
            let data: NnsGovernanceSourceData<NetworkEconomics> = self
                .fetch(
                    request,
                    "get_network_economics_parameters",
                    "()",
                    "NetworkEconomics",
                    (),
                )
                .await?;
            Ok(NnsGovernanceSourceData::new(
                data.value.into(),
                data.provenance,
            ))
        })
    }

    fn fetch_metrics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceMetrics> {
        Box::pin(async move {
            let data: NnsGovernanceSourceData<GetMetricsResult> = self
                .fetch(request, "get_metrics", "()", "GetMetricsResult", ())
                .await?;
            Ok(NnsGovernanceSourceData::new(
                super::metrics_result(data.value)?.into(),
                data.provenance,
            ))
        })
    }

    fn fetch_reward_event<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceRewardEvent> {
        Box::pin(async move {
            let data: NnsGovernanceSourceData<RewardEvent> = self
                .fetch(request, "get_latest_reward_event", "()", "RewardEvent", ())
                .await?;
            Ok(NnsGovernanceSourceData::new(
                data.value.into(),
                data.provenance,
            ))
        })
    }

    fn fetch_maturity_modulation<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, Option<NnsGovernanceMaturityModulation>> {
        Box::pin(async move {
            let data: NnsGovernanceSourceData<GetMaturityModulationResponse> = self
                .fetch(
                    request,
                    "get_maturity_modulation",
                    "GetMaturityModulationRequest",
                    "GetMaturityModulationResponse",
                    (GetMaturityModulationRequest {},),
                )
                .await?;
            Ok(NnsGovernanceSourceData::new(
                data.value.maturity_modulation.map(Into::into),
                data.provenance,
            ))
        })
    }
}
