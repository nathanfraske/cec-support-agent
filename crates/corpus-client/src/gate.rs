use thiserror::Error;

use crate::schema::Contribution;

/// Error returned when a contribution fails the sign-off gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("refused: outcome is not verifier- or human-confirmed (sign-off gate)")]
pub struct GateError;

/// The sign-off gate (invariant 6). Returns `Ok(())` only if the contribution
/// is confirmed by a verifier or a human; otherwise [`GateError`].
///
/// Every [`CorpusStore::submit`](crate::CorpusStore::submit) implementation
/// MUST call this before persisting or transmitting a contribution.
pub fn ensure_signed_off(contribution: &Contribution) -> Result<(), GateError> {
    if contribution.sign_off.is_confirmed() {
        Ok(())
    } else {
        Err(GateError)
    }
}
