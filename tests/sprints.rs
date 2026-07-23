//! Integration tests for the `sprint` collection through the compiled
//! binary: creation (including concurrent active sprints, decision 15),
//! listing, show, and closure with its pending-task guard.

use std::fs;
use std::path::Path;
use std::process::Output;

const SPRINTS_DIR: &str = "archaeology/sprints";

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
fn sprint_lifecycle_end_to_end() {
    let tmp = init_repo();

    let created = strata_in(tmp.path(), &["new", "sprint", "Prove the loop"]);
    assert!(created.status.success(), "{}", stderr(&created));
    assert!(
        stdout(&created).contains("created sprint:1"),
        "{}",
        stdout(&created)
    );
    assert!(
        tmp.path()
            .join(SPRINTS_DIR)
            .join("0001-prove-the-loop/sprint.md")
            .is_file()
    );

    let list = strata_in(tmp.path(), &["list", "sprints"]);
    assert!(list.status.success(), "{}", stderr(&list));
    let line = stdout(&list);
    for needle in ["sprint:1", "active", "Prove the loop"] {
        assert!(line.contains(needle), "missing `{needle}`:\n{line}");
    }

    let show = strata_in(tmp.path(), &["show", "sprint:1"]);
    assert!(show.status.success(), "{}", stderr(&show));
    assert!(
        stdout(&show).contains("# Prove the loop"),
        "{}",
        stdout(&show)
    );

    assert_doctor_healthy(tmp.path());

    let closed = strata_in(tmp.path(), &["close", "sprint:1"]);
    assert!(closed.status.success(), "{}", stderr(&closed));
    assert!(
        stdout(&closed).contains("closed sprint:1 (active -> closed)"),
        "{}",
        stdout(&closed)
    );
    let content = fs::read_to_string(
        tmp.path()
            .join(SPRINTS_DIR)
            .join("0001-prove-the-loop/sprint.md"),
    )
    .unwrap();
    assert!(content.contains("status: closed"), "{content}");
    assert!(
        content.contains("\nclosed: "),
        "must stamp closed:\n{content}"
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn list_sprints_json_carries_the_summary_fields() {
    let tmp = init_repo();
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Prove the loop"])
            .status
            .success()
    );

    let out = strata_in(tmp.path(), &["list", "sprints", "--json"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let listed: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    let sprint = &listed.as_array().expect("an array")[0];
    assert_eq!(sprint["kind"], "sprint");
    assert_eq!(sprint["sequence"], 1);
    assert_eq!(sprint["status"], "active");
    assert_eq!(
        sprint["path"],
        "archaeology/sprints/0001-prove-the-loop/sprint.md"
    );
}

#[test]
fn concurrent_active_sprints_are_created_normally_and_doctor_green() {
    // Decision 15 supersedes the former
    // `a_second_active_sprint_is_refused_naming_the_first` regression:
    // active-sprint cardinality is not repository validity.
    let tmp = init_repo();
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "First"])
            .status
            .success()
    );

    let out = strata_in(tmp.path(), &["new", "sprint", "Second"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("created sprint:2"),
        "{}",
        stdout(&out)
    );
    assert!(
        tmp.path()
            .join(SPRINTS_DIR)
            .join("0002-second/sprint.md")
            .is_file()
    );

    let doctor = strata_in(tmp.path(), &["doctor"]);
    assert!(
        doctor.status.success(),
        "concurrent active sprints are doctor-green:\n{}",
        stdout(&doctor)
    );
}

#[test]
fn closing_a_sprint_with_pending_tasks_is_refused_naming_them() {
    let tmp = init_repo();
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Busy"])
            .status
            .success()
    );
    let sprint_md =
        fs::read_to_string(tmp.path().join(SPRINTS_DIR).join("0001-busy/sprint.md")).unwrap();
    let sprint_id = sprint_md
        .lines()
        .find_map(|line| line.strip_prefix("id: "))
        .expect("sprint front matter has an id");
    fs::write(
        tmp.path().join(SPRINTS_DIR).join("0001-busy/0001-work.md"),
        format!(
            "---\nid: tsk-work\nsequence: 1\nkind: task\nstatus: pending\nsprint: {sprint_id}\ncreated: 2026-07-20\n---\n\n# Unfinished work\n"
        ),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["close", "sprint:1"]);

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("task:1"), "{err}");
    assert!(err.contains("Unfinished work"), "{err}");
    let content =
        fs::read_to_string(tmp.path().join(SPRINTS_DIR).join("0001-busy/sprint.md")).unwrap();
    assert!(content.contains("status: active"), "nothing may change");
}

#[test]
fn hand_seeded_sprint_identities_resolve_and_close_by_id() {
    let tmp = init_repo();
    let dir = tmp.path().join(SPRINTS_DIR).join("0001-legacy");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("sprint.md"),
        "---\nid: spr-legacy\nsequence: 1\nkind: sprint\nstatus: active\ncreated: 2026-07-20\n---\n\n# Legacy sprint\n",
    )
    .unwrap();

    let show = strata_in(tmp.path(), &["show", "spr-legacy"]);
    assert!(show.status.success(), "{}", stderr(&show));
    assert!(stdout(&show).contains("# Legacy sprint"));

    let out = strata_in(tmp.path(), &["close", "spr-legacy"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(dir.join("sprint.md")).unwrap();
    assert!(
        content.contains("id: spr-legacy"),
        "identities are never rewritten:\n{content}"
    );
    assert!(content.contains("status: closed"), "{content}");
    assert_doctor_healthy(tmp.path());
}

#[test]
fn sprint_transition_verbs_are_collection_scoped() {
    let tmp = init_repo();
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Current"])
            .status
            .success()
    );

    let out = strata_in(tmp.path(), &["reopen", "sprint:1"]);

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("sprints close"),
        "the refusal must name the verbs that apply: {}",
        stderr(&out)
    );
}
