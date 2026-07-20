//! Integration tests for `strata init` through the compiled binary.
//!
//! Every invocation runs inside a fresh temporary directory (which is not a
//! Git repository), proving Git is optional and keeping the real repository
//! untouched.

use std::fs;
use std::path::Path;
use std::process::Output;

const CONFIG_FILE: &str = ".strata.toml";
const REQUIRED_DIRS: [&str; 2] = ["archaeology/dragons/open", "archaeology/dragons/closed"];

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

#[test]
fn init_creates_expected_layout_in_empty_non_git_directory() {
    let tmp = tempfile::tempdir().unwrap();

    let out = strata_in(tmp.path(), &["init"]);

    assert!(out.status.success(), "init failed:\n{}", stderr(&out));
    assert!(
        !tmp.path().join(".git").exists(),
        "test premise: not a Git repository"
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join(CONFIG_FILE)).unwrap(),
        "version = 1\n"
    );
    for dir in REQUIRED_DIRS {
        assert!(tmp.path().join(dir).is_dir(), "missing {dir}");
    }
    assert!(stdout(&out).contains("initialized"), "{}", stdout(&out));
}

#[test]
fn rerun_succeeds_without_changing_existing_files() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(strata_in(tmp.path(), &["init"]).status.success());
    let custom = "# annotated by hand\nversion = 1\n";
    fs::write(tmp.path().join(CONFIG_FILE), custom).unwrap();

    let out = strata_in(tmp.path(), &["init"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("already initialized"),
        "{}",
        stdout(&out)
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join(CONFIG_FILE)).unwrap(),
        custom,
        "rerun must preserve an existing valid config byte-for-byte"
    );
}

#[test]
fn invalid_config_is_a_malformed_artifact_error_and_is_not_overwritten() {
    let tmp = tempfile::tempdir().unwrap();
    let content = "version = 99\n";
    fs::write(tmp.path().join(CONFIG_FILE), content).unwrap();

    let out = strata_in(tmp.path(), &["init"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[malformed-artifact]: "),
        "{}",
        stderr(&out)
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join(CONFIG_FILE)).unwrap(),
        content,
        "invalid config must survive untouched"
    );
}

#[test]
fn config_path_occupied_by_directory_is_an_artifact_conflict() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join(CONFIG_FILE)).unwrap();

    let out = strata_in(tmp.path(), &["init"]);

    assert_eq!(out.status.code(), Some(4), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[artifact-conflict]: "),
        "{}",
        stderr(&out)
    );
}

#[test]
fn required_directory_occupied_by_file_is_an_artifact_conflict() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("archaeology"), "in the way").unwrap();

    let out = strata_in(tmp.path(), &["init"]);

    assert_eq!(out.status.code(), Some(4), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[artifact-conflict]: "), "{err}");
    assert!(
        err.contains("archaeology"),
        "error must name the path:\n{err}"
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join("archaeology")).unwrap(),
        "in the way",
        "conflicting file must survive untouched"
    );
    assert!(
        !tmp.path().join(CONFIG_FILE).exists(),
        "no config may be written by a failed init"
    );
}
