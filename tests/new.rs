//! Integration tests for `strata new` through the compiled binary.
//!
//! Every invocation pins its working directory to a fresh temporary
//! directory so discovery can never walk up into a real repository.

use std::fs;
use std::path::Path;
use std::process::Output;

const CONFIG_FILE: &str = ".strata.toml";
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

#[test]
fn new_dragon_from_repository_root_reports_reference_and_path() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["new", "dragon", "Branch sequence collisions"]);

    assert!(out.status.success(), "new failed:\n{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("dragon:1"), "missing reference:\n{text}");
    let rel = format!("{OPEN_DIR}/0001-branch-sequence-collisions.md");
    assert!(text.contains(&rel), "missing relative path:\n{text}");

    let content = fs::read_to_string(tmp.path().join(&rel)).unwrap();
    assert!(content.starts_with("---\n"), "{content}");
    for needle in [
        "\nid: drg_",
        "\nsequence: 1\n",
        "\nkind: dragon\n",
        "\nstatus: open\n",
        "\ncreated: ",
        "# Branch sequence collisions",
        "## Context",
        "## Question",
        "## Constraints",
        "## Resolution criteria",
    ] {
        assert!(
            content.contains(needle),
            "missing `{needle}` in:\n{content}"
        );
    }
}

#[test]
fn new_dragon_from_nested_directory_writes_at_the_repository_root() {
    let tmp = init_repo();
    let nested = tmp.path().join("src/deeply/nested");
    fs::create_dir_all(&nested).unwrap();

    let out = strata_in(&nested, &["new", "dragon", "Found from below"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        tmp.path()
            .join(OPEN_DIR)
            .join("0001-found-from-below.md")
            .is_file(),
        "artifact must land under the discovered root"
    );
}

#[test]
fn sequences_advance_across_invocations_and_closed_artifacts() {
    let tmp = init_repo();
    fs::write(tmp.path().join(CLOSED_DIR).join("0004-resolved.md"), "seed").unwrap();

    let first = strata_in(tmp.path(), &["new", "dragon", "First"]);
    let second = strata_in(tmp.path(), &["new", "dragon", "Second"]);

    assert!(first.status.success(), "{}", stderr(&first));
    assert!(second.status.success(), "{}", stderr(&second));
    assert!(stdout(&first).contains("dragon:5"), "{}", stdout(&first));
    assert!(stdout(&second).contains("dragon:6"), "{}", stdout(&second));
    assert!(tmp.path().join(OPEN_DIR).join("0005-first.md").is_file());
    assert!(tmp.path().join(OPEN_DIR).join("0006-second.md").is_file());
}

#[test]
fn missing_repository_is_a_typed_error() {
    let tmp = tempfile::tempdir().unwrap();

    let out = strata_in(tmp.path(), &["new", "dragon", "No repository here"]);

    assert_eq!(out.status.code(), Some(3), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[missing-repository]: "),
        "{}",
        stderr(&out)
    );
}

#[test]
fn malformed_marker_during_discovery_is_rejected_not_walked_past() {
    let tmp = init_repo();
    let inner = tmp.path().join("vendored");
    fs::create_dir(&inner).unwrap();
    fs::write(inner.join(CONFIG_FILE), "version = \"broken\"").unwrap();

    let out = strata_in(&inner, &["new", "dragon", "Should not resolve upward"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[malformed-artifact]: "),
        "{}",
        stderr(&out)
    );
    let open: Vec<_> = fs::read_dir(tmp.path().join(OPEN_DIR)).unwrap().collect();
    assert!(open.is_empty(), "the outer repository must stay untouched");
}

#[test]
fn unsluggable_title_is_an_invalid_invocation() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["new", "dragon", "!!!"]);

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[invalid-invocation]: "),
        "{}",
        stderr(&out)
    );
}

#[test]
fn malformed_artifact_filename_blocks_creation_with_a_named_path() {
    let tmp = init_repo();
    fs::write(tmp.path().join(OPEN_DIR).join("scratch.txt"), "junk").unwrap();

    let out = strata_in(tmp.path(), &["new", "dragon", "Blocked"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[malformed-artifact]: "), "{err}");
    assert!(err.contains("scratch.txt"), "must name the file:\n{err}");
}
