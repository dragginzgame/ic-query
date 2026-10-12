//! Module: icrc::live::fetch
//!
//! Responsibility: expose cohesive live ICRC fetch owners and shared query context.
//! Does not own: source traits, synchronous runtime adaptation, report construction, or output.
//! Boundary: keeps account, history, and capability host calls behind one internal facade.

mod account;
mod capabilities;
mod history;

pub(super) use account::{
    account_from_parts, fetch_allowance_async, fetch_balance_async, fetch_token_async,
    query_token_display_fields,
};
pub(super) use capabilities::fetch_capabilities_async;
pub(super) use history::{
    fetch_archives_async, fetch_block_types_async, fetch_index_async, fetch_tip_certificate_async,
    fetch_transactions_async, query_index_principal, validate_transactions_data,
};

use crate::icrc::{ledger::principal_from_text, model::IcrcError};
use candid::Principal;
use ic_agent::Agent;

pub(super) fn live_query_context(
    source: &crate::icrc::LiveIcrcSource,
    source_endpoint: &str,
    ledger_canister_id: &str,
) -> Result<(Agent, Principal), IcrcError> {
    Ok((
        source
            .agent
            .get(
                source_endpoint,
                crate::agent::MAX_IC_AGENT_RESPONSE_BODY_BYTES,
                |reason| IcrcError::AgentBuild {
                    endpoint: source_endpoint.to_owned(),
                    reason,
                },
            )?
            .clone(),
        principal_from_text::<IcrcError>(ledger_canister_id, "ledger_canister_id")?,
    ))
}
