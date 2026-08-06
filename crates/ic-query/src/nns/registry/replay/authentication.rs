//! Module: nns::registry::replay::authentication
//!
//! Responsibility: preserve authenticated replay across live and retained-evidence paths.
//! Does not own: certificate verification, replay mechanics, persistence, or assurance policy.
//! Boundary: only complete sessions composed entirely from built-in verification can be sealed.

use super::{
    NnsRegistryReplayError, NnsRegistryReplayProgress, NnsRegistryReplaySession,
    NnsRegistryReplaySessionLimits,
};
use crate::nns::registry::NnsAuthenticatedRegistryDeltaBatch;

///
/// NnsAuthenticatedRegistryReplayBuilder
///
/// Bounded in-memory replay that admits only locally reauthenticated retained batches.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsAuthenticatedRegistryReplayBuilder {
    session: NnsRegistryReplaySession,
}

impl NnsAuthenticatedRegistryReplayBuilder {
    /// Create a version-zero authenticated replay builder with explicit cumulative limits.
    #[must_use]
    pub const fn new(limits: NnsRegistryReplaySessionLimits) -> Self {
        Self {
            session: NnsRegistryReplaySession::new(limits),
        }
    }

    /// Atomically admit one exact report already qualified by local reauthentication.
    pub fn apply_batch(
        &mut self,
        batch: &NnsAuthenticatedRegistryDeltaBatch<'_>,
    ) -> Result<NnsRegistryReplayProgress, NnsRegistryReplayError> {
        self.session.apply_prevalidated_batch(batch.report())
    }

    /// Return authenticated replay progress without exposing an ordinary mutable session.
    #[must_use]
    pub const fn replay_session(&self) -> &NnsRegistryReplaySession {
        &self.session
    }

    /// Seal a complete exact-target replay session, rejecting incomplete retained evidence.
    pub fn into_authenticated_replay_session(
        self,
    ) -> Result<NnsAuthenticatedRegistryReplaySession, NnsRegistryReplayError> {
        NnsAuthenticatedRegistryReplaySession::from_verified_complete(self.session)
    }
}

///
/// NnsAuthenticatedRegistryReplaySession
///
/// Complete replay session whose every batch passed ic-query's mainnet-root-key verifier.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsAuthenticatedRegistryReplaySession {
    session: NnsRegistryReplaySession,
}

impl NnsAuthenticatedRegistryReplaySession {
    pub(super) fn from_verified_complete(
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
