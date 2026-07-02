//! Compile-fail guards pinning the Layer-1 leak-prevention type barrier.
//!
//! Each case is a documented leak-shaped edit that MUST fail to compile, so a
//! regression (re-adding `Serialize`/`Display`, or making a field public) turns
//! a would-be silent leak into a red build:
//!
//! - `candidate_not_serializable` — `serde_json::to_string(&candidate)` (1a).
//! - `contribution_struct_literal` — a struct-literal `Contribution { .. }` that
//!   bypasses `Contribution::new`/the de-id mint (1f).
//! - `prose_no_display` — `format!("{}", prose)` with no `Display` (1b).
//!
//! The `.stderr` fixtures are pinned to the workspace toolchain (1.96.1, per
//! `rust-toolchain.toml` + `ci.yml`); regenerate with `TRYBUILD=overwrite` on a
//! deliberate toolchain bump.

#[test]
fn leak_barrier_edits_do_not_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
