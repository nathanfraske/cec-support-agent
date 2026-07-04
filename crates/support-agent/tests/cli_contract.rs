//! Process-level contract tests for the `cec-support-agent` CLI.
//!
//! These pin the wire contract AllMyStuff codes against (see
//! `docs/integration-rfc-for-chris.md`): under `--json`, stdout is exactly one
//! `cec-diagnose/v1` line and carries no request prose; cold start is unchanged.
//! They run the real binary, so they catch stdout writes from helper functions
//! that a unit test of `diagnose_envelope` cannot (the D2 regression: a free
//! function reached from `run()` that used a bare `println!`).

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cec-support-agent"))
}

/// A request laced with identity that MUST NOT appear on stdout under `--json`.
const PLANTED: &str =
    "DESKTOP-NATHAN01 user nathan at 192.168.1.20 serial SN12345678: explorer.exe crashes on login 0x1234";

fn nonempty_lines(s: &str) -> Vec<&str> {
    s.lines().filter(|l| !l.trim().is_empty()).collect()
}

#[test]
fn json_stdout_is_exactly_one_de_identified_envelope_line() {
    // Both the diagnose-only path AND the --sign-off path (which runs record_outcome,
    // the D2 free-function-println! culprit) must keep stdout to one envelope line.
    for extra in [Vec::new(), vec!["--sign-off", "human"]] {
        let out = bin()
            .args([
                "diagnose",
                "--offline",
                "--no-questions",
                "--json",
                "--describe",
                PLANTED,
            ])
            .args(&extra)
            .output()
            .expect("run cec-support-agent");
        assert!(out.status.success(), "non-zero exit (extra={extra:?})");

        let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
        let lines = nonempty_lines(&stdout);
        assert_eq!(
            lines.len(),
            1,
            "stdout must be exactly one line under --json (extra={extra:?}); got:\n{stdout}"
        );
        let line = lines[0];
        assert!(
            line.starts_with('{') && line.contains("\"schema_version\":\"cec-diagnose/v1\""),
            "stdout line is not a cec-diagnose/v1 envelope (extra={extra:?}): {line}"
        );

        // De-id: no planted identity anywhere on stdout (envelope is the only line).
        let low = stdout.to_lowercase();
        for tok in ["desktop-nathan01", "nathan", "192.168.1.20", "sn12345678"] {
            assert!(
                !low.contains(tok),
                "identity {tok:?} leaked onto stdout (extra={extra:?}): {stdout}"
            );
        }
    }
}

#[test]
fn cold_start_emits_the_human_trace_on_stdout_and_no_envelope() {
    // No --json: behavior is the historical human trace on stdout, and the machine
    // envelope is NOT emitted. Guards the "cold start byte-identical" invariant.
    let out = bin()
        .args([
            "diagnose",
            "--offline",
            "--no-questions",
            "--describe",
            "explorer.exe crashes on login 0x1234",
        ])
        .output()
        .expect("run cec-support-agent");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.starts_with("cec-support-agent: diagnose"),
        "cold-start human trace changed: {stdout}"
    );
    assert!(
        !stdout.contains("schema_version"),
        "cold start must not emit the cec-diagnose/v1 envelope"
    );
}

#[test]
fn json_config_class_reflects_inventory_keys_from_stdin() {
    // The `--inventory-keys -` (stdin) branch: keys piped in change the config_class
    // away from the coarse default, and no raw key token leaks onto stdout.
    use std::io::Write as _;
    use std::process::Stdio;

    let mut child = bin()
        .args([
            "diagnose",
            "--offline",
            "--no-questions",
            "--json",
            "--inventory-keys",
            "-",
            "--describe",
            "explorer.exe crashes on login 0x1234",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"os:windows 11 23h2\nhost:DESKTOP-NATHAN01\ngpu:rtx 4070 ti\n")
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let lines = nonempty_lines(&stdout);
    assert_eq!(lines.len(), 1, "stdout must be one line: {stdout}");
    assert!(lines[0].contains("\"schema_version\":\"cec-diagnose/v1\""));
    // multi-token + hostname keys are hashed into config_class, never echoed.
    assert!(
        !stdout.to_lowercase().contains("desktop-nathan01"),
        "inventory key leaked onto stdout: {stdout}"
    );
}

/// Run a diagnose and return the envelope's `fault.fingerprint` and
/// `config_class` values, with `CEC_FINGERPRINT_SALT` controlled explicitly.
fn envelope_keys(salt: Option<&str>) -> (String, String) {
    let mut cmd = bin();
    cmd.args([
        "diagnose",
        "--offline",
        "--no-questions",
        "--json",
        "--describe",
        "explorer.exe crashes on login 0x1234",
    ])
    .env_remove("CEC_FINGERPRINT_SALT");
    if let Some(salt) = salt {
        cmd.env("CEC_FINGERPRINT_SALT", salt);
    }
    let out = cmd.output().expect("run cec-support-agent");
    assert!(out.status.success(), "non-zero exit (salt={salt:?})");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let lines = nonempty_lines(&stdout);
    assert_eq!(lines.len(), 1, "stdout must be one envelope line: {stdout}");
    let envelope: serde_json::Value = serde_json::from_str(lines[0]).expect("valid JSON envelope");
    (
        envelope["fault"]["fingerprint"]
            .as_str()
            .expect("fault.fingerprint is a string")
            .to_string(),
        envelope["config_class"]
            .as_str()
            .expect("config_class is a string")
            .to_string(),
    )
}

#[test]
fn a_configured_fingerprint_salt_moves_the_retrieval_keys() {
    // leak-C7 e2e: the same request under a per-deployment salt produces a
    // fingerprint and config class UNLINKABLE to the cold-start ones — and the
    // salt value itself never appears on the wire.
    let (cold_fp, cold_class) = envelope_keys(None);
    let salt = "e2e-deployment-salt-0123456789abcdef";
    let (salted_fp, salted_class) = envelope_keys(Some(salt));
    assert_ne!(salted_fp, cold_fp, "salt must move the fault fingerprint");
    assert_ne!(salted_class, cold_class, "salt must move the config class");
    assert_eq!(salted_fp.len(), 64, "v2 fingerprints are HMAC-SHA256 hex");
    for value in [&salted_fp, &salted_class] {
        assert!(
            !value.contains(salt),
            "the salt value must never surface in a retrieval key"
        );
    }
}

#[test]
fn a_short_fingerprint_salt_refuses_startup() {
    // Fail closed: a set-but-weak salt is a startup error with a fixed message
    // that never echoes the value — never a silent cold-start fallback.
    let out = bin()
        .args([
            "diagnose",
            "--offline",
            "--no-questions",
            "--json",
            "--describe",
            "explorer.exe crashes on login 0x1234",
        ])
        .env("CEC_FINGERPRINT_SALT", "zq9weak")
        .output()
        .expect("run cec-support-agent");
    assert!(
        !out.status.success(),
        "a too-short CEC_FINGERPRINT_SALT must refuse startup"
    );
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("CEC_FINGERPRINT_SALT"),
        "the refusal must name the misconfigured variable: {stderr}"
    );
    assert!(
        !stderr.contains("zq9weak"),
        "the refusal must never echo the salt value: {stderr}"
    );
}
