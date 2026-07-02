// Leak-prevention 1a: a raw `Candidate` has no `Serialize`, so serializing one
// is a hard `E0277` — not a silent leak of its free-text `rationale` (which a
// model often echoes the request into). The de-identified action vocabulary
// reaches a sink via `diagnose_envelope`; the raw candidate never does.
fn main() {
    let plan = common::Plan::new("p", "title");
    let candidate = common::Candidate::new(plan, "why", common::CandidateSource::ColdModel);
    let _ = serde_json::to_string(&candidate);
}
