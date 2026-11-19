//! Exact ICRC balance evidence independent of SNS membership and display metadata.

use crate::{
    Failure, Observation,
    mainnet::{self, Source},
};
use ic_platform_protocol::ledger;
use ic_platform_types::{Account, Principal};
use serde::Serialize;
use tokio::time::Instant;

/// Three independent ledger observations for one explicitly selected account.
#[derive(Clone, Debug, Serialize)]
pub struct BalanceReport {
    /// Query report schema.
    pub schema: u8,
    /// Fixed admitted mainnet endpoint.
    pub endpoint: &'static str,
    /// Explicit source ledger; no SNS identity inference.
    pub ledger: Principal,
    /// Exact requested owner and optional subaccount presence.
    pub account: Account,
    /// Exact base-unit balance as a decimal string, without precision loss.
    pub balance: Observation<String>,
    /// Independent display scale; absence is not an assumed scale.
    pub decimals: Observation<u8>,
    /// Independent raw display label, escaped only by human presentation.
    pub symbol: Observation<String>,
    /// Whether all three observations succeeded.
    pub complete: bool,
    /// Separate queries never claim a single state snapshot.
    pub atomic_snapshot: bool,
}

/// Read one account balance, then independent decimals and symbol observations.
/// Each call has a 20-second deadline within 60 seconds overall; no retries.
/// Raw values are preserved even when display metadata fails. Requires Tokio IO/time.
/// # Errors
/// Returns an error only if the mainnet client cannot be constructed; read failures
/// are retained in the report and never become zero balances.
pub async fn fetch(ledger: Principal, account: Account) -> Result<BalanceReport, Failure> {
    Ok(collect(&mainnet::Mainnet::new()?, ledger, account).await)
}

pub(crate) async fn collect(
    source: &impl Source,
    target: Principal,
    account: Account,
) -> BalanceReport {
    let deadline = Instant::now() + mainnet::OVERALL;
    let balance = mainnet::observe(
        source,
        target,
        "icrc1_balance_of",
        ledger::encode_balance_account(&account, mainnet::ARGUMENT_BYTES).map_err(mainnet::encode),
        deadline,
    )
    .await
    .project(|bytes| {
        let amount = ledger::decode_amount(&bytes, mainnet::LIMITS).map_err(mainnet::decode)?;
        // Bound decimal expansion before allocating its string; never truncate.
        if amount.0.bits() > 4096 {
            return Err(mainnet::admission("balance exceeds 4096-bit report bound"));
        }

        Ok(amount.0.to_string())
    });
    let decimals = mainnet::observe(
        source,
        target,
        "icrc1_decimals",
        ledger::encode_metadata_args(mainnet::ARGUMENT_BYTES).map_err(mainnet::encode),
        deadline,
    )
    .await
    .project(|bytes| ledger::decode_decimals(&bytes, mainnet::LIMITS).map_err(mainnet::decode));
    let symbol = mainnet::observe(
        source,
        target,
        "icrc1_symbol",
        ledger::encode_metadata_args(mainnet::ARGUMENT_BYTES).map_err(mainnet::encode),
        deadline,
    )
    .await
    .project(|bytes| {
        let symbol = ledger::decode_symbol(&bytes, mainnet::LIMITS).map_err(mainnet::decode)?;
        if symbol.len() > 256 {
            return Err(mainnet::admission("symbol exceeds 256 UTF-8 bytes"));
        }

        Ok(symbol)
    });

    BalanceReport {
        schema: 1,
        endpoint: mainnet::ENDPOINT,
        ledger: target,
        account,
        complete: balance.is_read() && decimals.is_read() && symbol.is_read(),
        balance,
        decimals,
        symbol,
        atomic_snapshot: false,
    }
}
