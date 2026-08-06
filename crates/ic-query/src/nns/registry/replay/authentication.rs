//! Module: nns::registry::replay::authentication
//!
//! Responsibility: preserve the built-in live source's authenticated replay capability.
//! Does not own: certificate verification, replay, projection, persistence, or assurance policy.
//! Boundary: only the built-in bootstrap path can construct the authenticated wrapper.

use super::{NnsRegistryReplayError, NnsRegistryReplaySession};

///
/// NnsAuthenticatedRegistryReplaySession
///
/// Complete replay session collected through ic-query's mainnet-root-key verifier.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsAuthenticatedRegistryReplaySession {
    session: NnsRegistryReplaySession,
}

impl NnsAuthenticatedRegistryReplaySession {
    pub(super) fn from_built_in(
        session: NnsRegistryReplaySession,
    ) -> Result<Self, NnsRegistryReplayError> {
        if !session.is_complete()
            || session.root_key_digest().is_none()
            || session.evidence_chain_digest().is_none()
            || session.complete_state_digest().is_none()
        {
            return Err(
                NnsRegistryReplayError::AuthenticationRequiresCompleteSession {
                    selected_version: session.selected_version(),
                    through_version: session.state().through_version(),
                },
            );
        }
        Ok(Self { session })
    }

    /// Return the complete exact-target replay session authenticated by the built-in source.
    #[must_use]
    pub const fn replay_session(&self) -> &NnsRegistryReplaySession {
        &self.session
    }

    /// Consume the authenticated wrapper and deliberately discard its type-level capability.
    #[must_use]
    pub fn into_replay_session(self) -> NnsRegistryReplaySession {
        self.session
    }
}
