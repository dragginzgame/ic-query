//! Owned query error identity; upstream causes and diagnostics remain intact.

use ic_platform::agent::AgentError;
#[cfg(any(test, feature = "icrc-host", feature = "subnet-catalog-host"))]
use ic_platform::agent::TransportError;

///
/// QueryCallError
///
/// Failure of an ordinary query. It grants no authority to retry or resubmit.
///

#[derive(Debug)]
pub struct QueryCallError {
    source: Box<AgentError>,
}

impl std::fmt::Display for QueryCallError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.source.as_ref(), formatter)
    }
}

impl std::error::Error for QueryCallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl From<AgentError> for QueryCallError {
    fn from(source: AgentError) -> Self {
        Self {
            source: Box::new(source),
        }
    }
}

///
/// QueryFailure
///
/// Structural facts borrowed from a query failure, without application policy.
/// Other contains explicitly reviewed upstream cases, not an unknown wildcard.
///

#[cfg(any(test, feature = "icrc-host", feature = "subnet-catalog-host"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryFailure<'a> {
    /// HTTP response status; body and content type remain in the source.
    Http { status: u16 },
    /// Transport flags may overlap and do not prove whether execution occurred.
    Transport { connect: bool, timeout: bool },
    /// Structured IC rejection, retaining its authentication distinction.
    Rejected {
        certified: bool,
        code: u64,
        error_code: Option<&'a str>,
    },
    /// Agent timed out waiting for a response; execution outcome is not inferred.
    ResponseTimeout,
    /// A reviewed upstream failure with no policy projection needed by callers.
    Other,
}

impl QueryCallError {
    /// Inspect facts while preserving the complete original cause.
    #[cfg(any(test, feature = "icrc-host", feature = "subnet-catalog-host"))]
    pub fn failure(&self) -> QueryFailure<'_> {
        // Deliberately exhaustive: new upstream variants require an explicit review.
        match self.source.as_ref() {
            AgentError::HttpError(payload) => QueryFailure::Http {
                status: payload.status,
            },
            AgentError::TransportError(transport) => match transport {
                TransportError::Reqwest(error) => QueryFailure::Transport {
                    connect: error.is_connect(),
                    timeout: error.is_timeout(),
                },
                TransportError::Generic(_) => QueryFailure::Transport {
                    connect: false,
                    timeout: false,
                },
            },
            AgentError::CertifiedReject { reject, .. } => QueryFailure::Rejected {
                certified: true,
                code: reject.reject_code as u64,
                error_code: reject.error_code.as_deref(),
            },
            AgentError::UncertifiedReject { reject, .. } => QueryFailure::Rejected {
                certified: false,
                code: reject.reject_code as u64,
                error_code: reject.error_code.as_deref(),
            },
            AgentError::TimeoutWaitingForResponse() => QueryFailure::ResponseTimeout,
            AgentError::InvalidReplicaUrl(_)
            | AgentError::SigningError(_)
            | AgentError::InvalidCborData(_)
            | AgentError::CannotCalculateRequestId(_)
            | AgentError::CandidError(_)
            | AgentError::UrlParseError(_)
            | AgentError::InvalidMethodError(_)
            | AgentError::PrincipalError(_)
            | AgentError::InvalidReplicaStatus
            | AgentError::RequestStatusDoneNoReply(_)
            | AgentError::MessageError(_)
            | AgentError::Leb128ReadError(_)
            | AgentError::Utf8ReadError(_)
            | AgentError::LookupPathAbsent(_)
            | AgentError::LookupPathUnknown(_)
            | AgentError::LookupPathError(_)
            | AgentError::InvalidRequestStatus(..)
            | AgentError::CertificateVerificationFailed()
            | AgentError::QuerySignatureVerificationFailed
            | AgentError::CertificateNotAuthorized()
            | AgentError::CertificateOutdated(_)
            | AgentError::CertificateHasTooManyDelegations
            | AgentError::MissingSignature
            | AgentError::MalformedSignature
            | AgentError::MalformedPublicKey
            | AgentError::TooManySignatures { .. }
            | AgentError::DerKeyLengthMismatch { .. }
            | AgentError::DerPrefixMismatch { .. }
            | AgentError::NoRootKeyInStatus(_)
            | AgentError::WalletCallFailed(_)
            | AgentError::WalletError(_)
            | AgentError::WalletUpgradeRequired(_)
            | AgentError::ResponseSizeExceededLimit()
            | AgentError::CallDataMismatch { .. }
            | AgentError::InvalidRejectCode(_)
            | AgentError::RouteProviderError(_)
            | AgentError::InvalidHttpResponse(_) => QueryFailure::Other,
        }
    }
}
