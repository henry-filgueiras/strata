//! Integration tests for the bootstrap command surface and error contract.

use std::process::Output;

fn strata(args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_strata"))
        .args(args)
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
fn help_lists_bootstrap_commands() {
    let out = strata(&["--help"]);
    assert!(out.status.success(), "--help must exit 0");
    let help = stdout(&out);
    for command in [
        "init", "new", "list", "show", "doctor", "close", "reopen", "fortune",
    ] {
        assert!(
            help.contains(command),
            "help output missing `{command}`:\n{help}"
        );
    }
}

#[test]
fn version_flag_reports_version() {
    let out = strata(&["--version"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_subcommand_is_a_usage_error() {
    let out = strata(&[]);
    assert_eq!(out.status.code(), Some(2), "usage errors must exit 2");
    assert!(
        stderr(&out).contains("Usage"),
        "stderr should show usage:\n{}",
        stderr(&out)
    );
}

#[test]
fn unknown_subcommand_is_a_usage_error() {
    let out = strata(&["daemonize"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn unknown_collection_is_rejected_with_guidance() {
    let out = strata(&["new", "widget", "A title"]);
    assert_eq!(out.status.code(), Some(2), "invalid invocation must exit 2");
    let err = stderr(&out);
    assert!(
        err.contains("widget"),
        "error should name the input:\n{err}"
    );
    assert!(
        err.contains("dragon"),
        "error should list valid collections:\n{err}"
    );
}

#[test]
fn malformed_artifact_reference_is_rejected() {
    let out = strata(&["show", "dragon:seven"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("positive integer"),
        "error should state the expected form:\n{}",
        stderr(&out)
    );
}

#[test]
fn unknown_collection_in_reference_is_rejected() {
    let out = strata(&["show", "widget:1"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("widget"),
        "error should name the input:\n{}",
        stderr(&out)
    );
}

#[test]
fn zero_sequence_reference_is_rejected() {
    let out = strata(&["show", "dragon:0"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("start at 1"), "{}", stderr(&out));
}
