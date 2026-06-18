//! Module: sns::report::live
//!
//! Responsibility: group live SNS source, fetch, query, conversion, and wire modules.
//! Does not own: command parsing, report assembly, cache IO, or rendering.
//! Boundary: exposes live-source adapters and test-only wire helpers to SNS reports.

mod client;
mod convert;
mod fetch;
mod query;
mod types;

pub(super) use super::SnsHostError;
pub(super) use client::LiveSnsSource;
#[cfg(test)]
pub(super) use convert::metadata_row;
#[cfg(test)]
pub(super) use types::IcrcMetadataValue;
