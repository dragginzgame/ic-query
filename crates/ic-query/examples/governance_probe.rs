//! Deployable smoke probe for the four direct Governance reports.
//! The runner records whether execution occurred on a local NNS or mainnet.

#[cfg(target_arch = "wasm32")]
mod probe {
    use ic_query::nns::governance::{
        CanisterNnsSource, NnsGovernanceRequest, build_nns_governance_economics_report_with_source,
        build_nns_governance_maturity_modulation_report_with_source,
        build_nns_governance_metrics_report_with_source,
        build_nns_governance_reward_event_report_with_source,
    };
    use serde::Serialize;
    use serde_json::{Value, json};

    fn response<T: Serialize>(result: Result<T, impl ToString>) -> Value {
        match result {
            Ok(report) => json!({"status": "ok", "report": report}),
            Err(error) => json!({"status": "error", "error": error.to_string()}),
        }
    }

    #[ic_cdk::update]
    async fn report(kind: String) -> String {
        let request = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs(
            "ic",
            ic_cdk::api::time() / 1_000_000_000,
        );
        let source = CanisterNnsSource;
        let result = match kind.as_str() {
            "economics" => {
                response(build_nns_governance_economics_report_with_source(&request, &source).await)
            }
            "metrics" => {
                response(build_nns_governance_metrics_report_with_source(&request, &source).await)
            }
            "reward_event" => response(
                build_nns_governance_reward_event_report_with_source(&request, &source).await,
            ),
            "maturity_modulation" => response(
                build_nns_governance_maturity_modulation_report_with_source(&request, &source)
                    .await,
            ),
            _ => json!({"status": "error", "error": "unknown report kind"}),
        };
        result.to_string()
    }
}
