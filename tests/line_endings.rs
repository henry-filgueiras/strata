//! Integration tests for the LF-only canonical format (decision 14,
//! task 26) through the compiled binary.
//!
//! CRLF and bare-CR content is refused with a diagnosis naming line
//! endings as the actual cause — never "missing front matter" — and the
//! refused file is left byte-identical. LF content keeps parsing,
//! transitioning, and preserving unrelated bytes.

use std::fs;
use std::path::Path;
use std::process::Output;

const DRAGONS_DIR: &str = "archaeology/dragons";
const CONFIG_FILE: &str = ".strata.toml";

fn strata_in(dir: &Path, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_strata"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run strata binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn init_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let out = strata_in(tmp.path(), &["init"]);
    assert!(out.status.success(), "init failed:\n{}", stderr(&out));
    tmp
}

fn dragon_markdown(id: &str, sequence: u32, title: &str) -> String {
    format!(
        "---\nid: {id}\nsequence: {sequence}\nkind: dragon\nstatus: open\ncreated: 2026-07-20\n---\n\n# {title}\n\n## Context\n"
    )
}

fn seed_crlf_dragon(root: &Path, sequence: u32, name: &str) -> String {
    let crlf = dragon_markdown(
        &format!("drg-crlf-{sequence}"),
        sequence,
        "Windows checkout",
    )
    .replace('\n', "\r\n");
    fs::write(root.join(DRAGONS_DIR).join(name), &crlf).unwrap();
    crlf
}

fn expect_line_ending_refusal(out: &Output, path_needle: &str) {
    assert_eq!(out.status.code(), Some(5), "{}", stderr(out));
    let err = stderr(out);
    assert!(err.starts_with("error[malformed-artifact]: "), "{err}");
    assert!(err.contains(path_needle), "must name the path:\n{err}");
    assert!(err.contains("CRLF"), "must name the cause:\n{err}");
    assert!(err.contains("LF-only"), "must name the policy:\n{err}");
    assert!(
        err.contains("archaeology/.gitattributes"),
        "the repair names the archaeology policy:\n{err}"
    );
    assert!(
        !err.contains("missing front matter"),
        "must not decay into the front-matter diagnosis:\n{err}"
    );
}

#[test]
fn show_refuses_a_crlf_artifact_by_sequence_naming_line_endings() {
    let tmp = init_repo();
    let crlf = seed_crlf_dragon(tmp.path(), 1, "0001-windows-checkout.md");

    let out = strata_in(tmp.path(), &["show", "dragon:1"]);

    expect_line_ending_refusal(&out, "0001-windows-checkout.md");
    assert_eq!(
        fs::read_to_string(
            tmp.path()
                .join(DRAGONS_DIR)
                .join("0001-windows-checkout.md")
        )
        .unwrap(),
        crlf,
        "the refused file must remain byte-identical"
    );
}

#[test]
fn list_refuses_a_crlf_artifact_naming_line_endings() {
    let tmp = init_repo();
    seed_crlf_dragon(tmp.path(), 1, "0001-windows-checkout.md");

    let out = strata_in(tmp.path(), &["list", "dragons"]);

    expect_line_ending_refusal(&out, "0001-windows-checkout.md");
}

#[test]
fn transition_refuses_a_crlf_artifact_and_changes_nothing() {
    let tmp = init_repo();
    let crlf = seed_crlf_dragon(tmp.path(), 1, "0001-windows-checkout.md");

    let out = strata_in(tmp.path(), &["close", "dragon:1"]);

    expect_line_ending_refusal(&out, "0001-windows-checkout.md");
    assert_eq!(
        fs::read_to_string(
            tmp.path()
                .join(DRAGONS_DIR)
                .join("0001-windows-checkout.md")
        )
        .unwrap(),
        crlf,
        "a refused transition must leave the file byte-identical"
    );
}

#[test]
fn bare_carriage_return_is_refused_with_its_own_truthful_diagnosis() {
    let tmp = init_repo();
    let content = dragon_markdown("drg-cr", 1, "Classic Mac") + "stray\rreturn\n";
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-classic-mac.md"),
        &content,
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["show", "dragon:1"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("bare carriage return"), "{err}");
    assert!(err.contains("LF-only"), "{err}");
    assert!(!err.contains("missing front matter"), "{err}");
}

#[test]
fn doctor_reports_every_crlf_artifact_path_with_the_line_ending_cause() {
    let tmp = init_repo();
    seed_crlf_dragon(tmp.path(), 1, "0001-first-crlf.md");
    seed_crlf_dragon(tmp.path(), 2, "0002-second-crlf.md");
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0003-healthy.md"),
        dragon_markdown("drg-ok", 3, "Healthy"),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["doctor"]);

    assert_eq!(out.status.code(), Some(9), "{}", stderr(&out));
    let report = stdout(&out);
    for needle in [
        "malformed-artifact",
        "archaeology/dragons/0001-first-crlf.md",
        "archaeology/dragons/0002-second-crlf.md",
        "CRLF",
    ] {
        assert!(report.contains(needle), "missing `{needle}`:\n{report}");
    }
    assert!(
        !report.contains("0003-healthy.md"),
        "the LF artifact is not a finding:\n{report}"
    );
}

#[test]
fn crlf_config_is_valid_and_discovery_succeeds_through_it() {
    // The config is ordinary TOML outside the artifact-byte contract
    // (decision 14 as amended): CRLF is whatever the TOML parser says
    // it is — valid.
    let tmp = tempfile::tempdir().unwrap();
    let crlf = "version = 1\r\n";
    fs::write(tmp.path().join(CONFIG_FILE), crlf).unwrap();
    fs::create_dir_all(tmp.path().join(DRAGONS_DIR)).unwrap();

    let out = strata_in(tmp.path(), &["list", "dragons"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        fs::read_to_string(tmp.path().join(CONFIG_FILE)).unwrap(),
        crlf,
        "the config is never normalized or rewritten"
    );
}

#[test]
fn invalid_crlf_toml_keeps_the_ordinary_truthful_toml_diagnosis() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(CONFIG_FILE), "version = [broken\r\n").unwrap();

    let out = strata_in(tmp.path(), &["list", "dragons"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[malformed-artifact]: "), "{err}");
    assert!(
        err.contains("not valid TOML"),
        "the diagnosis is about TOML, not line endings:\n{err}"
    );
}

#[test]
fn lf_artifacts_still_parse_transition_and_preserve_unrelated_bytes() {
    let tmp = init_repo();
    let content = dragon_markdown("drg-lf", 1, "Healthy risk") + "\ntrailing  detail\n";
    let path = tmp.path().join(DRAGONS_DIR).join("0001-healthy-risk.md");
    fs::write(&path, &content).unwrap();

    let shown = strata_in(tmp.path(), &["show", "dragon:1"]);
    assert!(shown.status.success(), "{}", stderr(&shown));
    assert_eq!(stdout(&shown), content, "show is byte-exact");

    let closed = strata_in(tmp.path(), &["close", "dragon:1"]);
    assert!(closed.status.success(), "{}", stderr(&closed));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        content.replace("status: open", "status: closed"),
        "the transition rewrites only the status line"
    );
}
