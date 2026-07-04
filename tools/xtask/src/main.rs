//! Repo boundary tooling — leak methodology Layer 3b/3c
//! (`docs/corpus-leak-prevention.md` §2).
//!
//! `scan-content` is the CONTENT-keyed gate: no file in a commit may carry the
//! project's corpus-row JSON shape, a canonical poison token, or an encoded
//! (base64/hex) smuggling of either — keyed on bytes, not filenames, so
//! `.json`→`.md` renames, double extensions, and `git add -f` all still fail.
//! It runs over the STAGED blobs from the pre-commit hook and over the whole
//! tracked tree from the required CI `boundary` job, so an unprovisioned clone
//! is still gated server-side.
//!
//! `.boundary-allow.txt` names the sanctioned exceptions (the synthetic canned
//! fixture, the poison-set definition itself, methodology docs that discuss
//! the tokens). It is FROZEN in CI: a pull request that adds a net-new
//! allowlist line fails the `boundary` job (`allowlist-freeze`), so "edit the
//! allowlist in the same PR" is not a bypass — growth lands only via an
//! owner-approved change to the base branch.
//!
//! Honest limits (stated per the methodology, §2 3b): a content scanner over
//! an infinite identity space is BEST-EFFORT defense-in-depth — the hard gate
//! against tree leaks is Layer 1 stopping identity from being *produced* in
//! serializable form. Gzip-compressed literals are not decoded (no compression
//! dependency in the boundary tool); the runtime-decode ban on test files
//! covers the decode-at-runtime half of that hole. Generic PII shapes (emails,
//! MACs) are left to the gitleaks ruleset, which has mature redaction and
//! allowlisting; this tool scans the CLOSED canonical sets.

use std::collections::BTreeSet;
use std::process::{Command, ExitCode};

/// Relative path of the allowlist file at the repo root.
const ALLOWLIST_PATH: &str = ".boundary-allow.txt";

/// Quoted-JSON-key co-occurrence groups that mark a serialized corpus row. A
/// file fails when EVERY member of any one group appears. Quoted `"key":` form
/// deliberately: Rust field identifiers in engine source do not match, so the
/// backstop stays quiet on the schema code and bites on pasted row JSON.
const ROW_SHAPE_GROUPS: &[(&str, &[&str])] = &[
    (
        "corpus-row signature shape",
        &["\"signature\":", "\"fingerprint\":", "\"symptoms\":"],
    ),
    (
        "corpus-row attestation shape",
        &["\"attestation\":", "\"authority_id\":", "\"signature\":"],
    ),
    (
        "corpus-row integrity shape",
        &["\"integrity\":", "\"prev\":", "\"hash\":", "\"run_id\":"],
    ),
];

/// Decode-at-runtime markers forbidden in test files: a test that DECODES a
/// blob at runtime can smuggle an encoded corpus row past every static text
/// scan, so the capability itself is banned where fixtures live.
const RUNTIME_DECODE_MARKERS: &[&str] = &["base64", "hex::decode", "from_hex(", "unhex("];

/// The poison tokens this tool scans the TREE for: the canonical
/// `leakguard::POISON` set minus the bare author first name. The author's
/// public identity (LICENSE copyright, repo URLs, doc bylines) is not a corpus
/// leak, and a bare-substring match on it would flag half the repository; the
/// compound identity tokens (hostname, asset tag, email, serial, MAC, IP)
/// carry the actual signal and all stay.
fn poison_tokens() -> Vec<&'static str> {
    leakguard::POISON
        .iter()
        .copied()
        .filter(|t| *t != "nathan")
        .collect()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    match argv.as_slice() {
        ["scan-content", rest @ ..] => scan_content(rest),
        ["allowlist-freeze", "--against", base] => allowlist_freeze(base),
        ["install-hooks"] => install_hooks(),
        _ => {
            eprintln!(
                "usage:\n  xtask scan-content [--staged | --all]\n  \
                 xtask allowlist-freeze --against <ref>\n  xtask install-hooks"
            );
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// scan-content
// ---------------------------------------------------------------------------

struct Finding {
    path: String,
    check: &'static str,
    detail: String,
}

fn scan_content(rest: &[&str]) -> ExitCode {
    let all = match rest {
        [] | ["--staged"] => false,
        ["--all"] => true,
        _ => {
            eprintln!("scan-content takes --staged (default) or --all");
            return ExitCode::FAILURE;
        }
    };
    let allow = match load_allowlist() {
        Ok(allow) => allow,
        Err(error) => {
            eprintln!("boundary: cannot read {ALLOWLIST_PATH}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let files = if all { tracked_files() } else { staged_files() };
    let files = match files {
        Ok(files) => files,
        Err(error) => {
            eprintln!("boundary: git enumeration failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut findings = Vec::new();
    for path in &files {
        // The allowlist file itself and the gitleaks config carry the shapes
        // they guard against by construction; both are covered structurally
        // (freeze + CI) rather than by self-scanning.
        if path == ALLOWLIST_PATH || path == ".gitleaks.toml" {
            continue;
        }
        let bytes = if all {
            std::fs::read(path).unwrap_or_default()
        } else {
            staged_blob(path)
        };
        if bytes.is_empty() || bytes.contains(&0) {
            continue; // absent, or binary — filename patterns + gitleaks cover binaries
        }
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        scan_text(path, &text, &allow, &mut findings, "");
        // Decode-and-rescan: long base64/hex runs are decoded one level and
        // the decoded TEXT re-scanned, so an encoded row/token cannot ride a
        // string literal past the plain-text checks.
        if !allow.permits(path, "decode") {
            for decoded in decodable_runs(&text) {
                scan_text(path, &decoded, &allow, &mut findings, " (decoded literal)");
            }
        }
        // Runtime-decode ban in test files (path-keyed: integration-test dirs
        // and *_test.rs — the places fixtures live).
        let is_test_file = path.contains("/tests/") || path.ends_with("_test.rs");
        if is_test_file && !allow.permits(path, "runtime-decode") {
            for marker in RUNTIME_DECODE_MARKERS {
                if text.contains(marker) {
                    findings.push(Finding {
                        path: path.clone(),
                        check: "runtime-decode",
                        detail: format!(
                            "test file uses decode-at-runtime marker {marker:?} — an encoded \
                             fixture decoded at runtime bypasses the static content gate"
                        ),
                    });
                }
            }
        }
    }
    if findings.is_empty() {
        eprintln!(
            "boundary: scan-content clean ({} file(s), {})",
            files.len(),
            if all { "tracked tree" } else { "staged" }
        );
        return ExitCode::SUCCESS;
    }
    for f in &findings {
        eprintln!("boundary: {}: [{}] {}", f.path, f.check, f.detail);
    }
    eprintln!(
        "boundary: {} finding(s). A sanctioned synthetic literal belongs in \
         {ALLOWLIST_PATH} (line: `<path> <check>`), which is FROZEN in CI — growth \
         needs the owner's approval on the base branch.",
        findings.len()
    );
    ExitCode::FAILURE
}

/// Run the plain-text checks (row shape + poison) over one text body.
fn scan_text(path: &str, text: &str, allow: &Allowlist, findings: &mut Vec<Finding>, origin: &str) {
    if !allow.permits(path, "row-shape") {
        for (name, keys) in ROW_SHAPE_GROUPS {
            if keys.iter().all(|k| text.contains(k)) {
                findings.push(Finding {
                    path: path.to_string(),
                    check: "row-shape",
                    detail: format!("{name}{origin}: all of {keys:?} co-occur"),
                });
            }
        }
    }
    if !allow.permits(path, "poison") {
        let lower = text.to_lowercase();
        for token in poison_tokens() {
            if lower.contains(token) {
                findings.push(Finding {
                    path: path.to_string(),
                    check: "poison",
                    detail: format!("canonical poison token {token:?} present{origin}"),
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Allowlist
// ---------------------------------------------------------------------------

/// Parsed `.boundary-allow.txt`: `<path> <check>` per line, `#` comments.
/// `<check>` is one of row-shape | poison | decode | runtime-decode | all.
struct Allowlist(BTreeSet<(String, String)>);

impl Allowlist {
    fn permits(&self, path: &str, check: &str) -> bool {
        self.0.contains(&(path.to_string(), check.to_string()))
            || self.0.contains(&(path.to_string(), "all".to_string()))
    }
}

fn load_allowlist() -> Result<Allowlist, String> {
    let text = match std::fs::read_to_string(ALLOWLIST_PATH) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.to_string()),
    };
    Ok(Allowlist(parse_allowlist(&text)?))
}

fn parse_allowlist(text: &str) -> Result<BTreeSet<(String, String)>, String> {
    let mut set = BTreeSet::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(path), Some(check), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(format!(
                "{ALLOWLIST_PATH}:{}: expected `<path> <check>`",
                i + 1
            ));
        };
        if !matches!(
            check,
            "row-shape" | "poison" | "decode" | "runtime-decode" | "all"
        ) {
            return Err(format!(
                "{ALLOWLIST_PATH}:{}: unknown check {check:?}",
                i + 1
            ));
        }
        set.insert((path.to_string(), check.to_string()));
    }
    Ok(set)
}

/// CI freeze: every allowlist entry must already exist at `base` — net-new
/// entries fail, so the allowlist grows only via the base branch (owner
/// approval), never inside the same PR that needs the exception. Bootstrap
/// exemption: if the file does not exist at `base` at all, the check passes
/// (the PR that INTRODUCES the mechanism necessarily seeds it).
fn allowlist_freeze(base: &str) -> ExitCode {
    let current = match load_allowlist() {
        Ok(allow) => allow.0,
        Err(error) => {
            eprintln!("boundary: {error}");
            return ExitCode::FAILURE;
        }
    };
    let shown = git(&["show", &format!("{base}:{ALLOWLIST_PATH}")]);
    let base_text = match shown {
        Ok(text) => text,
        Err(_) => {
            eprintln!("boundary: {ALLOWLIST_PATH} absent at {base} — bootstrap, freeze passes");
            return ExitCode::SUCCESS;
        }
    };
    let base_set = match parse_allowlist(&base_text) {
        Ok(set) => set,
        Err(error) => {
            eprintln!("boundary: allowlist at {base} unparseable: {error}");
            return ExitCode::FAILURE;
        }
    };
    let new: Vec<_> = current.difference(&base_set).collect();
    if new.is_empty() {
        eprintln!("boundary: allowlist-freeze clean against {base}");
        return ExitCode::SUCCESS;
    }
    for (path, check) in &new {
        eprintln!("boundary: NET-NEW allowlist entry `{path} {check}` (not present at {base})");
    }
    eprintln!(
        "boundary: allowlist growth must land via the base branch with the owner's \
         review, never inside the PR that needs the exception."
    );
    ExitCode::FAILURE
}

// ---------------------------------------------------------------------------
// install-hooks
// ---------------------------------------------------------------------------

/// Point `core.hooksPath` at the tracked hooks so the pre-commit gate runs on
/// every clone that opts in. Gitleaks inside the hook is warn-and-skip (the
/// required CI `secrets` + `boundary` jobs are the server-side backstop), so
/// installation has no external dependency.
fn install_hooks() -> ExitCode {
    match git(&["config", "core.hooksPath", "scripts/githooks"]) {
        Ok(_) => {
            eprintln!("boundary: core.hooksPath -> scripts/githooks (pre-commit gate active)");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("boundary: git config failed: {error}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// git plumbing + decoding
// ---------------------------------------------------------------------------

fn git(args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn tracked_files() -> Result<Vec<String>, String> {
    Ok(split_z(&git(&["ls-files", "-z"])?))
}

fn staged_files() -> Result<Vec<String>, String> {
    Ok(split_z(&git(&[
        "diff",
        "--cached",
        "--name-only",
        "--diff-filter=ACM",
        "-z",
    ])?))
}

fn split_z(text: &str) -> Vec<String> {
    text.split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// The STAGED content of `path` (`git show :path`), not the working tree — the
/// gate judges what would actually enter the commit.
fn staged_blob(path: &str) -> Vec<u8> {
    Command::new("git")
        .args(["show", &format!(":{path}")])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| o.stdout)
        .unwrap_or_default()
}

/// Minimum length of a base64/hex character run worth decoding: long enough to
/// carry a smuggled token, short enough to catch a single encoded row field.
const MIN_DECODE_RUN: usize = 64;

/// Extract long base64/hex runs and decode each one level; only decodes that
/// yield valid UTF-8 text are returned (binary output — e.g. a real 64-hex
/// SHA-256 fixture value — carries no scannable text and is skipped).
fn decodable_runs(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for run in char_runs(text, |c| {
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
    }) {
        if run.len() < MIN_DECODE_RUN {
            continue;
        }
        if run.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Some(bytes) = hex_decode(&run) {
                if let Ok(text) = String::from_utf8(bytes) {
                    out.push(text);
                }
            }
        }
        if let Some(bytes) = base64_decode(&run) {
            if let Ok(text) = String::from_utf8(bytes) {
                out.push(text);
            }
        }
    }
    out
}

fn char_runs(text: &str, pred: impl Fn(char) -> bool) -> Vec<String> {
    let mut runs = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if pred(c) {
            current.push(c);
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    runs
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

/// Minimal standard-alphabet base64 decoder (std-only: the boundary tool takes
/// no decoding dependency). Rejects any non-alphabet byte; `=` only as final
/// padding.
fn base64_decode(text: &str) -> Option<Vec<u8>> {
    fn val(b: u8) -> Option<u32> {
        match b {
            b'A'..=b'Z' => Some((b - b'A') as u32),
            b'a'..=b'z' => Some((b - b'a' + 26) as u32),
            b'0'..=b'9' => Some((b - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let stripped = text.trim_end_matches('=');
    let pad = text.len() - stripped.len();
    if pad > 2 || text.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    for chunk in stripped.as_bytes().chunks(4) {
        let mut acc: u32 = 0;
        for (i, &b) in chunk.iter().enumerate() {
            acc |= val(b)? << (18 - 6 * i);
        }
        let bytes = chunk.len() * 6 / 8;
        for i in 0..bytes {
            out.push((acc >> (16 - 8 * i)) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_allow() -> Allowlist {
        Allowlist(BTreeSet::new())
    }

    #[test]
    fn a_pasted_corpus_row_shape_is_flagged() {
        let mut findings = Vec::new();
        let row = r#"{"signature": {"fingerprint": "aa", "symptoms": []}}"#;
        scan_text("x.md", row, &empty_allow(), &mut findings, "");
        assert!(
            findings.iter().any(|f| f.check == "row-shape"),
            "quoted-key co-occurrence must flag a pasted row regardless of extension"
        );
    }

    #[test]
    fn engine_source_identifiers_do_not_flag_row_shape() {
        // Rust field idents (no quoted `"key":` form) must stay quiet — the
        // schema source legitimately names every row field.
        let mut findings = Vec::new();
        let src = "struct S { signature: X, fingerprint: String, symptoms: Vec<Y> }";
        scan_text("schema.rs", src, &empty_allow(), &mut findings, "");
        assert!(findings.is_empty());
    }

    #[test]
    fn a_poison_token_is_flagged_case_insensitively() {
        let mut findings = Vec::new();
        scan_text(
            "notes.txt",
            "host was DESKTOP-NATHAN01 ok",
            &empty_allow(),
            &mut findings,
            "",
        );
        assert!(findings.iter().any(|f| f.check == "poison"));
    }

    #[test]
    fn the_bare_author_name_is_not_a_tree_poison_token() {
        let mut findings = Vec::new();
        scan_text(
            "LICENSE",
            "Copyright 2026 Nathan M. Fraske / github.com/nathanfraske",
            &empty_allow(),
            &mut findings,
            "",
        );
        assert!(
            findings.is_empty(),
            "the author's public identity is not a corpus leak"
        );
    }

    #[test]
    fn a_base64_encoded_poison_token_is_caught_by_decode_and_rescan() {
        // base64("the box DESKTOP-NATHAN01 belongs to nathan@example.com !!")
        // — built by the reference encoder below to keep the fixture honest.
        let plain = "the box DESKTOP-NATHAN01 belongs to nathan@example.com !!";
        let encoded = reference_base64(plain.as_bytes());
        assert!(
            encoded.len() >= MIN_DECODE_RUN,
            "fixture long enough to decode"
        );
        let decoded = decodable_runs(&format!("let blob = \"{encoded}\";"));
        assert!(
            decoded.iter().any(|d| d.contains("DESKTOP-NATHAN01")),
            "the encoded literal must decode back to scannable text"
        );
        let mut findings = Vec::new();
        for d in &decoded {
            scan_text(
                "fixture.rs",
                d,
                &empty_allow(),
                &mut findings,
                " (decoded literal)",
            );
        }
        assert!(findings.iter().any(|f| f.check == "poison"));
    }

    #[test]
    fn a_hex_hash_fixture_does_not_false_positive() {
        // A real 64-hex SHA-256 value decodes to binary, not UTF-8 text.
        let hash = "ce05cacfccaf87dcf265af5671f2ca3bcee0f8f789dd51f028252acc8f2f1547";
        assert!(decodable_runs(hash).is_empty());
    }

    #[test]
    fn allowlist_parses_permits_and_rejects_unknown_checks() {
        let set = parse_allowlist("# c\ncrates/a.rs poison\ndocs/b.md all\n").expect("parses");
        let allow = Allowlist(set);
        assert!(allow.permits("crates/a.rs", "poison"));
        assert!(!allow.permits("crates/a.rs", "row-shape"));
        assert!(
            allow.permits("docs/b.md", "row-shape"),
            "`all` covers every check"
        );
        assert!(parse_allowlist("x.rs bogus-check\n").is_err());
    }

    #[test]
    fn base64_decoder_round_trips_the_reference_encoder() {
        for case in [&b"any carnal pleasure."[..], b"a", b"ab", b"abc", b""] {
            let encoded = reference_base64(case);
            assert_eq!(
                base64_decode(&encoded).expect("decodes"),
                case,
                "decoder disagrees with the reference encoder on {case:?}"
            );
        }
        assert!(
            base64_decode("####").is_none(),
            "non-alphabet bytes refused"
        );
    }

    /// Test-only reference encoder, so the decode tests need no fixture blobs.
    fn reference_base64(bytes: &[u8]) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let mut acc = 0u32;
            for (i, &b) in chunk.iter().enumerate() {
                acc |= (b as u32) << (16 - 8 * i);
            }
            for i in 0..4 {
                if i <= chunk.len() {
                    out.push(TABLE[((acc >> (18 - 6 * i)) & 63) as usize] as char);
                } else {
                    out.push('=');
                }
            }
        }
        out
    }
}
