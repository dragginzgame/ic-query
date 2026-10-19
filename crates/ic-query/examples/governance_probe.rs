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

    const METHOD: u64 = 1;
    const REQUEST: ic_wire::Limits = ic_wire::Limits {
        bytes: 128,
        allocation: 2048,
        nodes: 4,
        depth: 4,
    };
    const RESPONSE: ic_wire::Limits = ic_wire::Limits {
        bytes: 2 * 1024 * 1024,
        allocation: 24 * 1024 * 1024,
        nodes: 4,
        depth: 4,
    };

    #[expect(
        clippy::unnecessary_wraps,
        reason = "Platform guards require Result; oversized input must trap"
    )]
    fn admit() -> Result<(), String> {
        if ic_platform::ic0::msg_arg_data_size() > REQUEST.bytes {
            ic_platform::cdk::api::trap("probe request exceeds its byte allowance");
        }
        Ok(())
    }

    fn decode(bytes: Vec<u8>) -> String {
        ic_wire::native::decode_request(METHOD, &bytes, REQUEST)
            .unwrap_or_else(|error| ic_platform::cdk::api::trap(format!("probe request: {error}")))
    }

    fn encode(value: String) -> Vec<u8> {
        ic_wire::native::encode_response(METHOD, &value, RESPONSE)
            .unwrap_or_else(|error| ic_platform::cdk::api::trap(format!("probe response: {error}")))
    }

    #[ic_platform::cdk::update(guard = "admit", decode_with = "decode", encode_with = "encode")]
    async fn report(kind: String) -> String {
        let request = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs(
            "ic",
            ic_platform::ic0::time() / 1_000_000_000,
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
