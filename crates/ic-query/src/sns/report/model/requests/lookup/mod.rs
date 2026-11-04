//! Module: sns::report::model::requests::lookup
//!
//! Responsibility: shared request DTO for direct SNS lookup reports.
//! Does not own: command option parsing, source resolution, or rendering.
//! Boundary: carries validated lookup inputs into one SNS report builder.

use ic_platform::candid::contract::Principal;
use thiserror::Error as ThisError;

///
/// SnsLookupError
///
/// Caller input is neither a numeric list position nor a root principal.
///

#[derive(Clone, Debug, Eq, PartialEq, ThisError)]
#[error("invalid SNS root principal: {input}")]
pub struct SnsLookupError {
    /// Original rejected selector.
    pub input: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SnsLookupTarget {
    Id(usize),
    Root(Principal),
}

///
/// SnsLookupRequest
///
/// Shared request accepted by direct SNS info, token, params, swap, and upgrade builders.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsLookupRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    input: String,
    target: SnsLookupTarget,
}

impl SnsLookupRequest {
    /// Parse the selector once while retaining the original caller input.
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        input: impl Into<String>,
    ) -> Result<Self, SnsLookupError> {
        let input = input.into();
        let target = match input.parse::<usize>() {
            Ok(id) => SnsLookupTarget::Id(id),
            Err(_) => {
                SnsLookupTarget::Root(Principal::from_text(&input).map_err(|_| SnsLookupError {
                    input: input.clone(),
                })?)
            }
        };
        Ok(Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            input,
            target,
        })
    }

    /// Original caller selector, retained independently of canonical principal text.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Numeric SNS-W list position when the caller selected one.
    #[must_use]
    pub const fn id(&self) -> Option<usize> {
        match self.target {
            SnsLookupTarget::Id(id) => Some(id),
            SnsLookupTarget::Root(_) => None,
        }
    }

    /// Parsed root principal when the caller selected a stable SNS identity.
    #[must_use]
    pub const fn root_canister_id(&self) -> Option<&Principal> {
        match &self.target {
            SnsLookupTarget::Root(root) => Some(root),
            SnsLookupTarget::Id(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
