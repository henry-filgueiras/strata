//! Integration tests for `strata doctor` through the compiled binary.

use std::fs;
use std::path::Path;
use std::process::Output;

const OPEN_DIR: &str = "archaeology/dragons/open";
const CLOSED_DIR: &str = "archaeology/dragons/closed";

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

fn dragon_markdown(id: &str, sequence: u32, status: &str, title: &str) -> String {
    format!(
        "---\nid: {id}\nsequence: {sequence}\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# {title}\n"
    )
}

#[test]
fn healthy_repository_exits_zero_with_a_summary() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(OPEN_DIR).join("0001-fine.md"),
        dragon_markdown("id-1", 1, "open", "Fine"),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["doctor"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("1 artifact(s) checked, no problems found"),
        "{}",
        stdout(&out)
    );
    assert!(stderr(&out).is_empty(), "{}", stderr(&out));
}

#[test]
fn marker_only_repository_is_healthy() {
    // Git drops empty directories on round-trip; doctor must not diagnose
    // that state as corruption (decision 5).
    let tmp = init_repo();
    fs::remove_dir_all(tmp.path().join("archaeology")).unwrap();

    let out = strata_in(tmp.path(), &["doctor"]);

    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn unhealthy_repository_reports_every_finding_and_exits_nine() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(OPEN_DIR).join("0001-bare.md"),
        "# No front matter\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(OPEN_DIR).join("0002-a.md"),
        dragon_markdown("id-same", 2, "open", "A"),
    )
    .unwrap();
    fs::write(
        tmp.path().join(CLOSED_DIR).join("0002-b.md"),
        dragon_markdown("id-same", 2, "closed", "B"),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["doctor"]);

    assert_eq!(out.status.code(), Some(9), "{}", stderr(&out));
    let report = stdout(&out);
    for needle in [
        "malformed-artifact",
        "duplicate-id",
        "duplicate-sequence",
        "0001-bare.md",
    ] {
        assert!(report.contains(needle), "missing `{needle}`:\n{report}");
    }
    assert!(
        stderr(&out).starts_with("error[unhealthy-repository]: "),
        "{}",
        stderr(&out)
    );
    assert!(stderr(&out).contains("3 problem(s)"), "{}", stderr(&out));
}

#[test]
fn json_findings_stay_parseable_when_validation_fails() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(OPEN_DIR).join("0001-bare.md"),
        "# No front matter\n",
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["doctor", "--json"]);

    assert_eq!(out.status.code(), Some(9), "{}", stderr(&out));
    let findings: serde_json::Value = serde_json::from_str(stdout(&out).trim())
        .expect("doctor --json stdout must be valid JSON on failure");
    let findings = findings.as_array().expect("findings must be an array");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0]["problem"], "malformed-artifact");
    assert_eq!(findings[0]["path"], "archaeology/dragons/open/0001-bare.md");
}

#[test]
fn json_findings_are_an_empty_array_when_healthy() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["doctor", "--json"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out).trim(), "[]");
}

#[test]
fn doctor_outside_a_repository_is_a_missing_repository_error() {
    let tmp = tempfile::tempdir().unwrap();

    let out = strata_in(tmp.path(), &["doctor"]);

    assert_eq!(out.status.code(), Some(3), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[missing-repository]: "),
        "{}",
        stderr(&out)
    );
}
