use common::Risk;
use serde::{Deserialize, Serialize};

/// The consent a caller has granted for actions of a given risk.
///
/// Consent is least-privilege by default: [`Consent::ReadOnlyOnly`] permits
/// only inspection. The [`Dispatcher`](crate::Dispatcher) refuses to run any
/// tool whose risk exceeds the granted consent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Consent {
    /// Only read-only actions are permitted.
    #[default]
    ReadOnlyOnly,
    /// Reversible changes are permitted (a restore point is taken first).
    AllowReversible,
    /// All actions, including destructive ones, are permitted.
    AllowDestructive,
}

impl Consent {
    /// Whether an action of `risk` is permitted under this consent.
    pub fn permits(self, risk: Risk) -> bool {
        match self {
            Consent::ReadOnlyOnly => risk == Risk::ReadOnly,
            Consent::AllowReversible => risk <= Risk::Reversible,
            Consent::AllowDestructive => true,
        }
    }
}
