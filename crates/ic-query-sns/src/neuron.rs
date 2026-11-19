//! Selected SNS neuron records and bounded collection, without ownership inference.

use crate::{
    Failure, FailureKind, Observation, Outcome,
    mainnet::{self, Source},
};
use ic_platform_protocol::sns;
use ic_platform_types::Principal;
use serde::Serialize;
use std::collections::BTreeSet;
use tokio::time::Instant;

/// The published, deliberately partial SNS neuron projection.
pub use sns::Neuron;

/// Fixed maximum number of returned neurons per query.
pub const PAGE_SIZE: u32 = 100;

/// Maximum admitted pages per collection.
pub const MAX_PAGES: u8 = 20;

/// One explicitly selected neuron and its independent outcome.
#[derive(Clone, Debug, Serialize)]
pub struct NeuronReport {
    /// Query report schema.
    pub schema: u8,
    /// Fixed admitted mainnet endpoint.
    pub endpoint: &'static str,
    /// Explicit SNS Governance target, not inferred from a ledger.
    pub governance: Principal,
    /// Exact requested 32-byte neuron identifier.
    pub neuron_id: [u8; 32],
    /// Selected fields or exact failure.
    pub neuron: Observation<Neuron>,
}

/// Read one exact neuron using the published SNS projection.
/// Does not return omitted followee/reward/disbursement fields or a full export.
/// Requires Tokio IO/time and applies a 20-second whole-call deadline.
/// # Errors
/// Returns client construction errors; query, decode and ID mismatch failures
/// remain in the report's observation.
pub async fn fetch(governance: Principal, neuron_id: [u8; 32]) -> Result<NeuronReport, Failure> {
    Ok(get(&mainnet::Mainnet::new()?, governance, neuron_id).await)
}

pub(crate) async fn get(
    source: &impl Source,
    governance: Principal,
    neuron_id: [u8; 32],
) -> NeuronReport {
    let args = sns::GetNeuronArgs {
        neuron_id: Some(sns::NeuronId {
            id: neuron_id.to_vec(),
        }),
    };
    let neuron = mainnet::observe(
        source,
        governance,
        "get_neuron",
        args.encode(mainnet::ARGUMENT_BYTES)
            .map_err(mainnet::encode),
        Instant::now() + mainnet::OVERALL,
    )
    .await
    .project(|bytes| {
        let reply = sns::decode_get_neuron(&bytes, mainnet::LIMITS).map_err(mainnet::decode)?;

        match reply.result {
            None => Err(Failure::new(
                FailureKind::MissingResult,
                "Governance omitted result",
            )),
            Some(sns::GetNeuronResult::Error(error)) => Err(Failure {
                kind: FailureKind::Governance,
                message: error.error_message,
                governance_error_type: Some(error.error_type),
            }),
            Some(sns::GetNeuronResult::Neuron(neuron)) => {
                if id(&neuron)? != neuron_id {
                    return Err(mainnet::admission(
                        "returned neuron ID differs from requested ID",
                    ));
                }

                Ok(neuron)
            }
        }
    });

    NeuronReport {
        schema: 1,
        endpoint: mainnet::ENDPOINT,
        governance,
        neuron_id,
        neuron,
    }
}

/// Caller-selected bounded listing for one Governance canister.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ListRequest {
    /// Explicit SNS Governance source.
    pub governance: Principal,
    /// Match source permissions, without calling this principal an owner.
    pub of_principal: Option<Principal>,
    /// Optional exclusive cursor for a bounded continuation; unfiltered only.
    pub start_after: Option<[u8; 32]>,
    /// From 1 through MAX_PAGES. Filtered reads require exactly one page because
    /// the upstream principal-filtered method ignores its cursor.
    pub max_pages: u8,
}

/// Why collection stopped; exhaustion is not an atomic snapshot guarantee.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// A short page exhausted this traversal/filter observation.
    Exhausted,
    /// The page budget was reached; more records may exist.
    PageLimit,
    /// A full principal-filtered page cannot be continued using the upstream cursor.
    FilterLimit,
    /// A read or admission failed; earlier admitted pages remain available.
    Failed,
}

/// One query's cursor and independent selected records or failure.
#[derive(Clone, Debug, Serialize)]
pub struct Page {
    /// Exclusive cursor for this query; absent when beginning without a cursor.
    pub start_after: Option<[u8; 32]>,
    /// Independent timestamps, authentication and response outcome.
    pub observation: Observation<Vec<Neuron>>,
}

/// Bounded, non-atomic neuron evidence with retained partial coverage.
#[derive(Clone, Debug, Serialize)]
pub struct NeuronsReport {
    /// Query report schema.
    pub schema: u8,
    /// Fixed admitted mainnet endpoint.
    pub endpoint: &'static str,
    /// Explicit target, filter and budget.
    pub request: ListRequest,
    /// Requested records per page.
    pub page_size: u32,
    /// All attempted pages, including a final failure when one occurred.
    pub pages: Vec<Page>,
    /// Number of admitted records; not total SNS population.
    pub retained_neurons: usize,
    /// Explicit termination/coverage reason.
    pub stop_reason: StopReason,
    /// True only when a short page ended traversal without a failure.
    pub traversal_exhausted: bool,
    /// Separate live reads have no common state version.
    pub atomic_snapshot: bool,
}

/// Collect up to 20 pages of 100 records within a shared 60-second deadline.
/// Each read is bounded to 20 seconds; cancellation drops pending work.
/// Filtered reads require one page and never pretend that a full page is complete.
/// # Errors
/// Returns invalid selector/budget or client construction errors before reads.
/// Later read/admission failures remain in the report alongside prior pages.
pub async fn list(request: ListRequest) -> Result<NeuronsReport, Failure> {
    validate_request(request)?;

    collect(&mainnet::Mainnet::new()?, request).await
}

fn validate_request(request: ListRequest) -> Result<(), Failure> {
    if !(1..=MAX_PAGES).contains(&request.max_pages) {
        return Err(mainnet::admission("max_pages must be 1 through 20"));
    }
    if request.of_principal.is_some() && (request.max_pages != 1 || request.start_after.is_some()) {
        return Err(mainnet::admission(
            "principal-filtered SNS listings support only one page and no cursor",
        ));
    }

    Ok(())
}

pub(crate) async fn collect(
    source: &impl Source,
    request: ListRequest,
) -> Result<NeuronsReport, Failure> {
    collect_pages(request, |cursor, deadline| async move {
        let args = sns::ListNeuronsArgs {
            of_principal: request.of_principal,
            limit: PAGE_SIZE,
            start_page_at: cursor.map(|id: [u8; 32]| sns::NeuronId { id: id.to_vec() }),
        };

        mainnet::observe(
            source,
            request.governance,
            "list_neurons",
            args.encode(mainnet::ARGUMENT_BYTES)
                .map_err(mainnet::encode),
            deadline,
        )
        .await
        .project(|bytes| {
            sns::decode_list_neurons(&bytes, mainnet::LIMITS)
                .map(|r| r.neurons)
                .map_err(mainnet::decode)
        })
    })
    .await
}

pub(crate) async fn collect_pages<F, Fut>(
    request: ListRequest,
    mut read: F,
) -> Result<NeuronsReport, Failure>
where
    F: FnMut(Option<[u8; 32]>, Instant) -> Fut,
    Fut: std::future::Future<Output = Observation<Vec<Neuron>>>,
{
    validate_request(request)?;
    let deadline = Instant::now() + mainnet::OVERALL;
    let mut report = NeuronsReport {
        schema: 1,
        endpoint: mainnet::ENDPOINT,
        request,
        page_size: PAGE_SIZE,
        pages: Vec::new(),
        retained_neurons: 0,
        stop_reason: StopReason::PageLimit,
        traversal_exhausted: false,
        atomic_snapshot: false,
    };
    let mut cursor = request.start_after;
    for _ in 0..request.max_pages {
        let start_after = cursor;
        let observation = read(cursor, deadline).await.project(|neurons| {
            validate_page(&neurons, cursor, request.of_principal)?;

            Ok(neurons)
        });
        let count = match &observation.outcome {
            Outcome::Failed { .. } => {
                report.stop_reason = StopReason::Failed;

                None
            }
            Outcome::Read { value } => {
                cursor = value.last().map(id).transpose()?;
                report.retained_neurons += value.len();

                Some(value.len())
            }
        };
        report.pages.push(Page {
            start_after,
            observation,
        });

        match count {
            None => break,
            Some(n) if n < PAGE_SIZE as usize => {
                report.stop_reason = StopReason::Exhausted;
                report.traversal_exhausted = true;
                break;
            }
            Some(_) if request.of_principal.is_some() => {
                report.stop_reason = StopReason::FilterLimit;
                break;
            }
            Some(_) => {}
        }
    }

    Ok(report)
}

fn id(neuron: &Neuron) -> Result<[u8; 32], Failure> {
    neuron
        .id
        .as_ref()
        .and_then(|id| id.id.as_slice().try_into().ok())
        .ok_or_else(|| mainnet::admission("neuron ID is absent or not exactly 32 bytes"))
}

pub(crate) fn validate_page(
    neurons: &[Neuron],
    cursor: Option<[u8; 32]>,
    principal: Option<Principal>,
) -> Result<(), Failure> {
    if neurons.len() > PAGE_SIZE as usize {
        return Err(mainnet::admission("page exceeds requested neuron limit"));
    }
    let mut seen = BTreeSet::new();
    let mut previous = cursor;
    for neuron in neurons {
        let current = id(neuron)?;
        if !seen.insert(current) {
            return Err(mainnet::admission("duplicate neuron ID in page"));
        }
        if let Some(principal) = principal {
            if !neuron
                .permissions
                .iter()
                .any(|p| p.principal == Some(principal))
            {
                return Err(mainnet::admission(
                    "filtered neuron lacks requested permission principal",
                ));
            }
        } else if previous.is_some_and(|p| current <= p) {
            return Err(mainnet::admission(
                "neuron page is unordered or does not advance exclusive cursor",
            ));
        }
        previous = Some(current);
    }

    Ok(())
}
