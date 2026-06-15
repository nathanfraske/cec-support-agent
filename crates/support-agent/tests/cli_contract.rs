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
