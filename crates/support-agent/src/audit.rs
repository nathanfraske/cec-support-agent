// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Execution audit log — the de-identified record of WHICH plan produced WHICH
//! outcome and WHEN. The on-machine twin of the corpus query-log (the gap named
//! as cartography V7 / MH-1): a remote execution surface must be attributable,
//! and this is the seam that makes it so.
//!
//! De-identified BY CONSTRUCTION. A record carries only the MINTED plan id (never
//! the raw, pre-mint id a model generator may have produced), the opaque run id,
//! a unix timestamp, the de-identified outcome-label token, and — at rung-2 — a
//! hashed caller key. It NEVER carries the request text, tool output, a step
//! summary, or any raw identifier: the same egress discipline as the wire
//! envelope. The field set is closed, so a future field cannot silently ride
//! along.
//!
//! The sink is a deliberately minimal seam, not a storage engine. The default
//! [`NullSink`] records nothing; a deployment wires a persistent,
//! access-controlled sink here, and a caller-identity layer fills in `caller_key`
//! at rung-2 (today the loopback socket's trust boundary is the OS user, so there
//! is no caller identity to hash). See `docs/test-validation-fleet-design.md`
//! §2.2 invariant 3.

use std::time::{SystemTime, UNIX_EPOCH};

/// One execution audit record. Every field is de-identified vocabulary or an
/// opaque token — there is deliberately no field for free text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecord {
    /// The MINTED (de-identified) plan id that produced the outcome — the same
    /// id the corpus row carries, read from the stored contribution, never the
    /// raw pre-mint id.
    pub plan_id: String,
    /// The opaque per-run id (OS entropy). Ties the record to the outcome's
    /// provenance without revealing identity; empty when a run had no provenance
    /// pin.
    pub run_id: String,
    /// Unix seconds when the outcome was recorded.
    pub at_unix_secs: u64,
    /// The de-identified outcome-label token (e.g. `resolved_confirmed`,
    /// `withdrawn`) — the SAME token the `cec-execute/v1` wire envelope emits.
    pub outcome: String,
    /// A hashed caller key, once a caller-identity layer exists (rung-2). `None`
    /// today: the loopback socket's trust boundary is the OS user, so there is no
    /// caller identity to hash yet.
    pub caller_key: Option<String>,
}

impl ExecutionRecord {
    /// Build a record from already-de-identified pieces, stamping the current
    /// wall-clock second. `caller_key` is `None` until a caller-identity layer
    /// exists. The caller passes the MINTED plan id (e.g.
    /// `contribution.outcome().plan().id()`), never a raw plan id.
    pub fn new(
        plan_id: impl Into<String>,
        run_id: impl Into<String>,
        outcome: impl Into<String>,
    ) -> Self {
        let at_unix_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or(0);
        Self {
            plan_id: plan_id.into(),
            run_id: run_id.into(),
            at_unix_secs,
            outcome: outcome.into(),
            caller_key: None,
        }
    }

    /// A single de-identified log line: a compact JSON object of the closed field
    /// set only. No prose, tool output, or raw identifier can appear — the type
    /// has no field for one. The de-id serialization a deployment's persistent
    /// sink writes; exercised by the tests today (the default `NullSink` does not
    /// serialize).
    #[allow(dead_code)] // used by a deployment's persistent sink + the tests
    pub fn to_line(&self) -> String {
        serde_json::json!({
            "plan_id": self.plan_id,
            "run_id": self.run_id,
            "at": self.at_unix_secs,
            "outcome": self.outcome,
            "caller": self.caller_key,
        })
        .to_string()
    }
}

/// Where execution audit records go. The default is a no-op; a deployment wires a
/// persistent, access-controlled sink. Kept minimal on purpose — this is the
/// seam, not the storage.
pub trait AuditSink: Send + Sync {
    /// Record one execution outcome. Implementations MUST NOT enrich the record
    /// with anything beyond [`ExecutionRecord`]'s de-identified fields.
    fn record(&self, record: &ExecutionRecord);
}

/// The default sink: records nothing. The bootstrap has no persistent,
/// access-controlled audit store; a deployment wires one here.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullSink;

impl AuditSink for NullSink {
    fn record(&self, _record: &ExecutionRecord) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_audit_line_is_a_closed_de_identified_field_set() {
        let record = ExecutionRecord::new("driver_regression", "a1b2c3", "resolved_confirmed");
        assert!(record.caller_key.is_none(), "no caller identity at rung-1");

        let value: serde_json::Value = serde_json::from_str(&record.to_line()).expect("valid json");
        let object = value.as_object().expect("an object");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        // Exactly these keys, forever — no prose/summary/describe field can ride
        // along, because the type has none.
        assert_eq!(keys, ["at", "caller", "outcome", "plan_id", "run_id"]);
        assert_eq!(object["caller"], serde_json::Value::Null);
        assert_eq!(object["plan_id"], "driver_regression");
        assert_eq!(object["outcome"], "resolved_confirmed");
    }

    #[test]
    fn null_sink_is_a_no_op() {
        // The default sink accepts a record and does nothing — it must never
        // panic or block.
        NullSink.record(&ExecutionRecord::new("p", "r", "withdrawn"));
    }
}
