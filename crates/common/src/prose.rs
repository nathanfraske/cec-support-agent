/// A leaf of human- or request-derived **free text**.
///
/// This is the type-system's finest-grained de-identification boundary
/// (methodology 1b): the leak-bearing fields are all `String` today
/// (`Plan.title`, `PlanStep.description`, `Candidate.rationale`,
/// `DiagnosticEvent.message`, `StepResult.summary`), and `String: Serialize +
/// Display` forever — so removing `Serialize` from the *struct* does nothing for
/// `json!({"why": c.rationale})` or a `format!("{}", plan.title)`. `Prose` fixes
/// that at the leaf:
///
/// - **no `Serialize`/`Deserialize`** — it cannot be written to a corpus row,
///   the `--json`/API envelope, or any serde sink;
/// - **no `Display`** — `format!("{}", prose)` / `write!(w, "{prose}")` do not
///   compile;
/// - **redacting `Debug`** — `format!("{outcome:?}")` / `dbg!(candidate)` can
///   never spill request text, so a struct that holds `Prose` keeps a derived
///   `Debug` and is automatically sealed (methodology 1d).
///
/// The inner string is reachable ONLY through the explicit [`Prose::as_str`] /
/// [`Prose::into_inner`] accessors — the two names the egress lint denylists.
/// A sanctioned human-facing sink (consent rendering, the local human trace,
/// the in-process plan-signing canonicalization) calls them deliberately; a
/// serialize/print sink cannot reach the text without naming one.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct Prose(String);

impl Prose {
    /// Wrap free text as prose. Construction is not the leak — reaching a sink
    /// is — so this is unrestricted; the barrier is on the way *out*.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// Borrow the underlying text. An explicit, denylisted egress accessor:
    /// call it only for a sanctioned human-facing sink.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume into the owned text. An explicit, denylisted egress accessor.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Whether the prose is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for Prose {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl From<&str> for Prose {
    fn from(text: &str) -> Self {
        Self(text.to_string())
    }
}

impl std::fmt::Debug for Prose {
    /// Redacts the content. The byte length is kept (it is not identity) so a
    /// `{:?}` of a containing struct is still structurally useful, but the raw
    /// text never appears — this is what makes `format!("{outcome:?}")` safe.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prose(<redacted {} bytes>)", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_the_content() {
        let secret = "DESKTOP-NATHAN01 nathan 192.168.1.20";
        let prose = Prose::new(secret);
        let shown = format!("{prose:?}");
        assert!(!shown.contains("NATHAN"), "Debug leaked prose: {shown}");
        assert!(!shown.contains("192.168"), "Debug leaked prose: {shown}");
        assert!(shown.contains("redacted"));
    }

    #[test]
    fn accessors_round_trip_the_text() {
        let p = Prose::new("hello");
        assert_eq!(p.as_str(), "hello");
        assert_eq!(p.into_inner(), "hello");
        assert_eq!(Prose::from("x").as_str(), "x");
        assert_eq!(Prose::from(String::from("y")).as_str(), "y");
    }
}
