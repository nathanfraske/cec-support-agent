//! Canonical identity-poison set + structural de-id assertions for the
//! leak-prevention test harness.
//!
//! This is the SINGLE source of truth for the identity tokens every de-id test
//! plants, replacing the per-module token arrays that diverged — and that
//! historically omitted the `action`/`plan.id` fields entirely, so the
//! "adversarial de-id" test passed precisely because it avoided the two fields
//! `de_identify_plan` copied through verbatim. See `docs/corpus-leak-prevention.md`.

/// The canonical identity tokens planted through the pipeline by every de-id
/// test. No token (any casing) may survive into a corpus row, the `--json`
/// envelope, stdout, or any other sink. A superset of the formerly-local arrays,
/// plus transform-bypass shapes (`rig_nathan_desk`, an in-house binary name).
pub const POISON: &[&str] = &[
    "desktop-nathan01",
    "nathan",
    "nathan@example.com",
    "192.168.1.20",
    "00:1a:2b:3c:4d:5e",
    "sn12345678",
    "c:\\users",
    "acmecorp",
    "rig_nathan_desk",
];

/// Assert a sink's bytes carry no poison token (case-insensitive). Names the
/// sink for a useful failure. This is the substring guard — necessary but NOT
/// sufficient against a transforming pipeline; pair it with [`is_grammar_member`]
/// on each emitted symptom (de-id transforms, it does not merely delete).
pub fn assert_no_poison(text: &str, sink: &str) {
    let low = text.to_lowercase();
    for tok in POISON {
        assert!(
            !low.contains(&tok.to_lowercase()),
            "poison token {tok:?} leaked into {sink}: {text}"
        );
    }
}

/// Whether `symptom` is admissible under the de-id grammar (round-trip stable
/// through the extractor). Used to assert that every symptom appearing in a sink
/// is something the extractor itself would produce — a STRUCTURAL check, since
/// de-id is a transformation, not a deletion (a substring scan misses
/// `RIG_NATHAN_DESK` -> `rig_nathan_desk`, which is byte-distinct from the
/// planted token yet still an identity leak).
pub fn is_grammar_member(symptom: &str) -> bool {
    deid::symptom(symptom).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poison_is_lowercase_and_nonempty() {
        assert!(!POISON.is_empty());
        for p in POISON {
            assert_eq!(
                *p,
                p.to_lowercase(),
                "store POISON lowercased for case-insensitive scans"
            );
        }
    }

    #[test]
    fn grammar_member_rejects_poison_accepts_vocabulary() {
        assert!(is_grammar_member("explorer.exe"));
        assert!(is_grammar_member("0x1234"));
        for p in POISON {
            assert!(
                !is_grammar_member(p),
                "poison {p:?} must not be a grammar member"
            );
        }
    }
}
