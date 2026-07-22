//! Integration tests for `strata close` and `strata reopen` through the
//! compiled binary, pinning the failure-class contract of decision 8 as
//! amended by decision 11: a transition is one in-place safe write, and
//! the file never moves.

use std::fs;
use std::path::Path;
use std::process::Output;

const DRAGONS_DIR: &str = "archaeology/dragons";

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

/// A dragon whose bytes exercise everything a transition must preserve:
/// an extra front-matter field, trailing whitespace, the literal string
/// `status: open` in prose, and a fenced block containing one.
fn rich_dragon(status: &str) -> String {
    format!(
        "---\nid: drg-rich\nsequence: 1\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\nseverity: high\n---\n\n# Rich dragon\n\nProse mentioning status: open stays put.  \n\n```yaml\nstatus: open\n```\n"
    )
}

fn assert_doctor_healthy(root: &Path) {
    let out = strata_in(root, &["doctor"]);
    assert!(
        out.status.success(),
        "doctor must be healthy:\n{}\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn close_rewrites_only_the_status_and_never_moves_the_file() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md"),
        rich_dragon("open"),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["close", "dragon:1"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let line = stdout(&out);
    for needle in [
        "dragon:1",
        "open",
        "closed",
        "archaeology/dragons/0001-rich-dragon.md",
    ] {
        assert!(
            line.contains(needle),
            "output must name `{needle}`:\n{line}"
        );
    }
    assert_eq!(
        fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap(),
        rich_dragon("closed"),
        "every byte except the status value must be preserved, in place"
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn reopen_round_trips_to_the_original_bytes() {
    let tmp = init_repo();
    let original = rich_dragon("open");
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md"),
        &original,
    )
    .unwrap();

    assert!(
        strata_in(tmp.path(), &["close", "dragon:1"])
            .status
            .success()
    );
    let out = strata_in(tmp.path(), &["reopen", "dragon:1"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("reopened dragon:1"),
        "{}",
        stdout(&out)
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap(),
        original,
        "a close/reopen round trip must be byte-identical"
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn close_resolves_stable_ids() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md"),
        rich_dragon("open"),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["close", "drg-rich"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap(),
        rich_dragon("closed")
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn closing_an_already_closed_artifact_is_an_invalid_invocation() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md"),
        rich_dragon("closed"),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["close", "dragon:1"]);

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[invalid-invocation]:"), "{err}");
    assert!(err.contains("already closed"), "{err}");
    assert_eq!(
        fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap(),
        rich_dragon("closed")
    );
}

#[test]
fn unknown_reference_is_artifact_not_found() {
    let tmp = init_repo();
    let out = strata_in(tmp.path(), &["close", "dragon:41"]);
    assert_eq!(out.status.code(), Some(7));
    assert!(
        stderr(&out).starts_with("error[artifact-not-found]:"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn duplicate_sequence_is_an_ambiguous_reference() {
    // The branch-merge shape of dragon 1: two branches allocated the same
    // sequence, and a merge put both files in the collection directory.
    let tmp = init_repo();
    for (status, name) in [("open", "0001-a.md"), ("closed", "0001-b.md")] {
        fs::write(
            tmp.path().join(DRAGONS_DIR).join(name),
            format!(
                "---\nid: id-{name}\nsequence: 1\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# T\n"
            ),
        )
        .unwrap();
    }

    let out = strata_in(tmp.path(), &["close", "dragon:1"]);

    assert_eq!(out.status.code(), Some(8));
    assert!(
        stderr(&out).starts_with("error[ambiguous-reference]:"),
        "{}",
        stderr(&out)
    );
}

#[cfg(unix)]
#[test]
fn failed_write_leaves_the_artifact_untouched() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = init_repo();
    let original = rich_dragon("open");
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md"),
        &original,
    )
    .unwrap();
    fs::set_permissions(
        tmp.path().join(DRAGONS_DIR),
        fs::Permissions::from_mode(0o555),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["close", "dragon:1"]);

    fs::set_permissions(
        tmp.path().join(DRAGONS_DIR),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    assert_eq!(out.status.code(), Some(6), "{}", stderr(&out));
    assert_eq!(
        fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap(),
        original,
        "a failed status rewrite must leave the artifact unchanged"
    );
    assert_doctor_healthy(tmp.path());
}
