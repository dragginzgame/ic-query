//! Match public declarations to observed candidates without selecting a tariff.

use super::{available, prices};
use ic_platform_types::Principal;
use serde::Serialize;
use std::{future::Future, time::Duration};
use tokio::time::Instant;

/// Candidate observations and all matching public price declarations.
#[derive(Debug, Serialize)]
pub struct CandidatesReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// Total monotonic time across the two bounded collections.
    pub elapsed: Duration,
    /// Availability exhausted and both price reads succeeded, not price coverage.
    pub collection_complete: bool,
    /// Always false: pages, prices and fee are independent observations.
    pub atomic_snapshot: bool,
    /// Original pages, stop reason, limits and per-page source evidence.
    pub availability: available::AvailableNodesReport,
    /// Original price evidence or the failure that prevented matching.
    pub pricing: CandidatePricing,
}

/// Matching outcome; a missing declaration differs from an unread price list.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CandidatePricing {
    /// Matching is possible even when the separate fee observation failed.
    Read {
        /// Exact declarations and independently observed fee, without conversion.
        report: prices::CloudEnginePricesReport,
        /// One result for every admitted candidate, in acquisition order.
        matches: Vec<CandidateMatches>,
    },
    /// Candidate evidence survives a price-read or price-admission failure.
    Failed {
        /// Original typed cause; matching coverage is unknown, not zero.
        error: prices::PriceError,
    },
}

/// All declared scopes matching one candidate; no precedence is assigned.
#[derive(Debug, Serialize)]
pub struct CandidateMatches {
    /// Candidate identity, also present in the retained availability pages.
    pub node_id: Principal,
    /// Zero-based indices into `pricing.report.prices.entries`, in that order.
    /// Empty means no public declaration matched, not a zero price. Even one
    /// match is not an effective tariff or quote; owner discounts are unobserved.
    pub price_indices: Vec<usize>,
}

/// Collect new-engine candidates, then match public price declaration scopes.
///
/// Uses the existing availability (60 seconds, 1–100 pages) and price (30 seconds)
/// collectors sequentially, without retries or extra queries. Class and present
/// provider/DC scopes must match exactly; absent scopes are unrestricted. No DC
/// normalization, price precedence, discount, total or capacity claim is added.
/// Cancellation drops the pending collection and starts no later collection.
///
/// # Errors
/// Invalid page limits or an initial availability failure return its typed error.
/// Partial availability is retained and matched. Any later pricing failure stays
/// in the returned report, with `collection_complete = false`.
pub async fn fetch(page_limit: u16) -> Result<CandidatesReport, available::AvailableError> {
    collect(available::fetch_with_page_limit(page_limit), prices::fetch).await
}

async fn collect<A, P, F>(
    availability: A,
    pricing: F,
) -> Result<CandidatesReport, available::AvailableError>
where
    A: Future<Output = Result<available::AvailableNodesReport, available::AvailableError>>,
    P: Future<Output = Result<prices::CloudEnginePricesReport, prices::PriceError>>,
    F: FnOnce() -> P,
{
    let start = Instant::now();
    let availability = availability.await?;
    let pricing = match pricing().await {
        Ok(report) => {
            let matches = availability
                .pages
                .iter()
                .flat_map(|page| &page.nodes)
                .map(|node| CandidateMatches {
                    node_id: node.node_id,
                    price_indices: matching_indices(node, &report.prices.entries),
                })
                .collect();
            CandidatePricing::Read { report, matches }
        }
        Err(error) => CandidatePricing::Failed { error },
    };
    let collection_complete = availability.pagination_exhausted
        && matches!(&pricing, CandidatePricing::Read { report, .. } if report.complete);
    Ok(CandidatesReport {
        schema: 1,
        elapsed: start.elapsed(),
        collection_complete,
        atomic_snapshot: false,
        availability,
        pricing,
    })
}

fn matching_indices(
    node: &available::AvailableNode,
    prices: &[prices::MarketplacePrice],
) -> Vec<usize> {
    prices
        .iter()
        .enumerate()
        .filter_map(|(index, price)| {
            (price.node_type == node.node_type
                && price
                    .provider
                    .is_none_or(|provider| provider == node.provider_id)
                && price
                    .data_center
                    .as_ref()
                    .is_none_or(|dc| Some(dc) == node.data_center.as_ref()))
            .then_some(index)
        })
        .collect()
}

#[cfg(test)]
mod tests;
