//! On-screen EULA acceptance — a liability boundary distinct from the risk
//! consent gate.
//!
//! Some remediations install third-party software that carries an end-user
//! license agreement. Accepting that license is the USER's act, not the shop's:
//! if the engine (or a technician) clicked through the EULA on the user's
//! behalf, the shop would be accepting the license terms and the liability for
//! them. So the engine never accepts a EULA — it REFUSES to run a EULA-bearing
//! install unless the user accepted that specific license on screen, and the
//! acceptance is what this type records.
//!
//! This is orthogonal to [`crate::Consent`]: consent is about how risky a
//! change is (read-only / reversible / destructive); a EULA requirement is a
//! legal precondition on a specific action, and a low-risk install can still
//! carry one. Both gates must pass for a EULA-bearing step to run.
//!
//! The set is populated on the TARGET, where the executor presents each
//! required EULA and captures the user's on-screen acceptance before execution;
//! see `docs/eula-acceptance-playbook.md`. The engine-side guarantee is only
//! this: no acceptance recorded ⇒ the installer never runs.

use std::collections::BTreeSet;

/// The set of EULAs the user accepted on screen for this execution. Built on
/// the target after on-screen acceptance; consulted by [`crate::execute_plan`],
/// which refuses a EULA-bearing step whose id is absent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EulaAcceptances(BTreeSet<String>);

impl EulaAcceptances {
    /// No EULAs accepted. The correct value for a run with no install steps —
    /// and the fail-closed default, since it makes every EULA-bearing step
    /// refuse until an acceptance is explicitly recorded.
    pub fn none() -> Self {
        Self(BTreeSet::new())
    }

    /// Record the user's on-screen acceptance of `eula` (a tool's EULA id).
    /// Builder-style so a target can accumulate acceptances before executing.
    pub fn accept(mut self, eula: impl Into<String>) -> Self {
        self.0.insert(eula.into());
        self
    }

    /// Whether the user accepted `eula` on screen.
    pub fn accepted(&self, eula: &str) -> bool {
        self.0.contains(eula)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_accepts_nothing_and_accept_records() {
        let empty = EulaAcceptances::none();
        assert!(
            !empty.accepted("signalrgb"),
            "the default refuses everything"
        );
        let accepted = EulaAcceptances::none().accept("signalrgb");
        assert!(accepted.accepted("signalrgb"));
        assert!(!accepted.accepted("thermalright"), "only what was accepted");
    }
}
