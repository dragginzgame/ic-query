//! Daily marketplace declarations and an independently observed network markup.

mod source;
#[cfg(test)]
mod tests;
mod types;

use std::{collections::BTreeSet, time::Duration};

use ic_platform_protocol::{
    candid as decoder,
    cloud_engine::{self as wire, NodeType},
};
use tokio::time::Instant;

use crate::mainnet;
use source::{Observation, Source};
pub use types::*;

use super::CANISTER;

const PRICES_METHOD: &str = "listMarketplacePrices";
const FEE_METHOD: &str = "getNetworkFee";
const OVERALL: Duration = Duration::from_secs(30);
const MAX_ROWS: usize = 1000;
const MAX_KEY_BYTES: usize = 512;
const MAX_DC_BYTES: usize = 256;
const MAX_DIGITS: usize = 256;
const LIMITS: decoder::Limits = decoder::Limits {
    bytes: 512 * 1024,
    header: 64 * 1024,
    types: 256,
    work: 2_000_000,
    allocation: 4 * 1024 * 1024,
    vector: 1024,
    skipping: 128 * 1024,
};

/// Read public daily price declarations, then the separately observed network fee.
///
/// Uses anonymous verified mainnet queries, 1 MiB HTTP responses and 20-second
/// leaf deadlines, with 30 seconds overall. No mutation, quote or retry is added.
/// At most 1,000 unique price identities are admitted. Requires Tokio IO/time.
/// Partial success retains admitted prices if fee collection fails. Cancellation
/// drops the pending operation and never starts another read.
///
/// # Errors
/// Returns a typed error if the first price read or admission fails. Later errors
/// are retained in the report's `network_fee` outcome, with `complete = false`.
pub async fn fetch() -> Result<CloudEnginePricesReport, PriceError> {
    let client = mainnet::client()?;
    collect(&source::MainnetSource(client)).await
}

async fn collect(source: &impl Source) -> Result<CloudEnginePricesReport, PriceError> {
    let start = Instant::now();
    let deadline = start + OVERALL;
    let started_at_unix_seconds = mainnet::observed_at()?;
    let prices = tokio::time::timeout_at(deadline, async {
        project_prices(&source.query(PRICES_METHOD).await?)
    })
    .await
    .map_err(|_| PriceError::OverallDeadline)??;
    // Do not start a second request if initial decoding/admission used the budget.
    let network_fee = if Instant::now() >= deadline {
        FeeOutcome::OverallDeadline
    } else {
        match tokio::time::timeout_at(deadline, async {
            project_fee(&source.query(FEE_METHOD).await?)
        })
        .await
        {
            Err(_) => FeeOutcome::OverallDeadline,
            Ok(Err(error)) => FeeOutcome::Failed { error },
            Ok(Ok((markup, evidence))) => FeeOutcome::Read { markup, evidence },
        }
    };
    Ok(CloudEnginePricesReport {
        schema: 1,
        started_at_unix_seconds,
        elapsed: start.elapsed(),
        complete: matches!(network_fee, FeeOutcome::Read { .. }),
        atomic_snapshot: false,
        prices,
        network_fee,
    })
}

fn evidence(method: &'static str, observed_at_unix_seconds: u64) -> PriceEvidence {
    PriceEvidence {
        endpoint: mainnet::ENDPOINT,
        source_canister: CANISTER,
        source_method: method,
        observed_at_unix_seconds,
        query_signature_verified: true,
        result_certified: false,
    }
}

fn project_prices(observation: &Observation) -> Result<PriceObservation, PriceError> {
    let rows = wire::decode_list_marketplace_prices_response(&observation.bytes, LIMITS)?;
    bound("rows", rows.len(), MAX_ROWS)?;
    let mut identities = BTreeSet::new();
    let mut entries = Vec::with_capacity(rows.len());
    // Admit numeric magnitude before any decimal expansion.
    let maximum = decoder::Nat::from(10u64).0.pow(MAX_DIGITS as u32);
    for row in rows {
        bound("key_bytes", row.key.len(), MAX_KEY_BYTES)?;
        if let Some(dc) = &row.entry.dc {
            bound("dc_bytes", dc.len(), MAX_DC_BYTES)?;
        }
        let mut expected = row
            .entry
            .provider
            .map_or_else(String::new, |id| format!("{id},"));
        let node_type = row.entry.node_type;
        expected.push_str(class_key(&node_type));
        if let Some(dc) = &row.entry.dc {
            expected.push(',');
            expected.push_str(dc);
        }
        if row.key != expected {
            return Err(AdmissionError::KeyMismatch.into());
        }
        if !identities.insert(row.key.clone()) {
            return Err(AdmissionError::DuplicateKey.into());
        }
        for amount in [&row.entry.price.net_cycles, &row.entry.price.gross_cycles] {
            if amount.0 >= maximum {
                return Err(AdmissionError::CycleDigits.into());
            }
        }
        if row.entry.price.gross_cycles < row.entry.price.net_cycles {
            return Err(AdmissionError::GrossBelowNet.into());
        }
        entries.push(MarketplacePrice {
            key: row.key,
            node_type,
            provider: row.entry.provider,
            data_center: row.entry.dc,
            net_cycles_per_node_per_day: row.entry.price.net_cycles.0.to_str_radix(10),
            gross_cycles_per_node_per_day: row.entry.price.gross_cycles.0.to_str_radix(10),
            updated_at_unix_nanoseconds: row.entry.updated_at,
        });
    }
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(PriceObservation {
        entries,
        evidence: evidence(PRICES_METHOD, observation.observed_at),
    })
}

fn class_key(class: &NodeType) -> &'static str {
    match class {
        NodeType::Type4 => "type4",
        NodeType::Type4_1 => "type4.1",
        NodeType::Type4_2 => "type4.2",
        NodeType::Type4_3 => "type4.3",
        NodeType::Type4_4 => "type4.4",
        NodeType::Type4_5 => "type4.5",
    }
}

fn bound(field: &'static str, actual: usize, maximum: usize) -> Result<(), AdmissionError> {
    if actual > maximum {
        Err(AdmissionError::Limit {
            field,
            actual,
            maximum,
        })
    } else {
        Ok(())
    }
}

fn project_fee(observation: &Observation) -> Result<(f64, PriceEvidence), PriceError> {
    let markup = wire::decode_get_network_fee_response(&observation.bytes, LIMITS)?;
    if !markup.is_finite() || markup < 0.0 {
        return Err(AdmissionError::InvalidFee.into());
    }
    Ok((markup, evidence(FEE_METHOD, observation.observed_at)))
}
