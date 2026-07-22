//! Integration tests for the `task` collection through the compiled
//! binary: creation in the active sprint, discovery across sprints,
//! listing with the active filter, and closure.

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

/// Seed a closed sprint carrying one closed task, the shape history leaves
/// behind.
fn seed_closed_sprint_with_task(root: &Path, sequence: u32, task_sequence: u32) {
    let dir = root
        .join(SPRINTS_DIR)
        .join(format!("{sequence:04}-history"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("sprint.md"),
        format!(
            "---\nid: spr-history-{sequence}\nsequence: {sequence}\nkind: sprint\nstatus: closed\ncreated: 2026-07-20\n---\n\n# History {sequence}\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join(format!("{task_sequence:04}-done.md")),
        format!(
            "---\nid: tsk-done-{task_sequence}\nsequence: {task_sequence}\nkind: task\nstatus: closed\nsprint: spr-history-{sequence}\ncreated: 2026-07-20\n---\n\n# Done work {task_sequence}\n"
        ),
    )
    .unwrap();
}

#[test]
fn task_lifecycle_end_to_end() {
    let tmp = init_repo();
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Current work"])
            .status
            .success()
    );

    let created = strata_in(tmp.path(), &["new", "task", "Ship the feature"]);
    assert!(created.status.success(), "{}", stderr(&created));
    assert!(
        stdout(&created).contains("created task:1"),
        "{}",
        stdout(&created)
    );
    let task_path = tmp
        .path()
        .join(SPRINTS_DIR)
        .join("0001-current-work/0001-ship-the-feature.md");
    let content = fs::read_to_string(&task_path).unwrap();
    for needle in [
        "kind: task",
        "status: pending",
        "sprint: spr_",
        "# Ship the feature",
        "## Objective",
        "## Acceptance criteria",
    ] {
        assert!(content.contains(needle), "missing `{needle}`:\n{content}");
    }
    assert_doctor_healthy(tmp.path());

    let show = strata_in(tmp.path(), &["show", "task:1"]);
    assert!(show.status.success(), "{}", stderr(&show));
    assert!(stdout(&show).contains("# Ship the feature"));

    let closed = strata_in(tmp.path(), &["close", "task:1"]);
    assert!(closed.status.success(), "{}", stderr(&closed));
    assert!(
        stdout(&closed).contains("closed task:1 (pending -> closed)"),
        "{}",
        stdout(&closed)
    );
    let content = fs::read_to_string(&task_path).unwrap();
    assert!(content.contains("status: closed"), "{content}");
    assert!(
        content.contains("\nclosed: "),
        "must stamp closed:\n{content}"
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn task_creation_requires_an_active_sprint() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["new", "task", "Orphan work"]);

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("no sprint is active"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn task_sequences_are_global_across_sprints() {
    let tmp = init_repo();
    seed_closed_sprint_with_task(tmp.path(), 1, 7);
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Current"])
            .status
            .success()
    );

    let out = strata_in(tmp.path(), &["new", "task", "Next work"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("created task:8"),
        "sequences continue globally across sprints: {}",
        stdout(&out)
    );
    assert!(
        tmp.path()
            .join(SPRINTS_DIR)
            .join("0002-current/0008-next-work.md")
            .is_file(),
        "the task must land in the active sprint's directory"
    );
}

#[test]
fn list_tasks_spans_sprints_and_active_filters_to_the_active_sprint() {
    let tmp = init_repo();
    seed_closed_sprint_with_task(tmp.path(), 1, 1);
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Current"])
            .status
            .success()
    );
    assert!(
        strata_in(tmp.path(), &["new", "task", "Fresh work"])
            .status
            .success()
    );

    let all = strata_in(tmp.path(), &["list", "tasks"]);
    assert!(all.status.success(), "{}", stderr(&all));
    let text = stdout(&all);
    assert!(text.contains("task:1") && text.contains("task:2"), "{text}");

    let active = strata_in(tmp.path(), &["list", "tasks", "--active"]);
    assert!(active.status.success(), "{}", stderr(&active));
    let text = stdout(&active);
    assert!(text.contains("task:2"), "{text}");
    assert!(
        !text.contains("task:1"),
        "closed sprints' tasks must be filtered out:\n{text}"
    );

    let json = strata_in(tmp.path(), &["list", "tasks", "--json"]);
    let listed: serde_json::Value = serde_json::from_str(stdout(&json).trim()).unwrap();
    let tasks = listed.as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["kind"], "task");
    assert_eq!(tasks[0]["sprint"], "spr-history-1");

    let misuse = strata_in(tmp.path(), &["list", "dragons", "--active"]);
    assert_eq!(misuse.status.code(), Some(2), "{}", stderr(&misuse));
}

#[test]
fn misfiled_tasks_are_doctor_errors() {
    let tmp = init_repo();
    seed_closed_sprint_with_task(tmp.path(), 1, 1);
    // A task claiming a sprint that does not exist.
    fs::write(
        tmp.path()
            .join(SPRINTS_DIR)
            .join("0001-history/0002-stray.md"),
        "---\nid: tsk-stray\nsequence: 2\nkind: task\nstatus: closed\nsprint: spr-nowhere\ncreated: 2026-07-20\n---\n\n# Stray\n",
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["doctor"]);

    assert_eq!(out.status.code(), Some(9), "{}", stderr(&out));
    let report = stdout(&out);
    assert!(report.contains("misfiled-task"), "{report}");
    assert!(report.contains("spr-nowhere"), "{report}");
}

#[test]
fn hand_seeded_task_identities_close_by_id_without_rewrites() {
    let tmp = init_repo();
    let dir = tmp.path().join(SPRINTS_DIR).join("0001-current");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("sprint.md"),
        "---\nid: spr-current\nsequence: 1\nkind: sprint\nstatus: active\ncreated: 2026-07-20\n---\n\n# Current\n",
    )
    .unwrap();
    fs::write(
        dir.join("0001-legacy.md"),
        "---\nid: tsk-legacy\nsequence: 1\nkind: task\nstatus: pending\nsprint: spr-current\ncreated: 2026-07-20\n---\n\n# Legacy task\n",
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["close", "tsk-legacy"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(dir.join("0001-legacy.md")).unwrap();
    assert!(
        content.contains("id: tsk-legacy"),
        "identities are never rewritten:\n{content}"
    );
    assert!(content.contains("status: closed"), "{content}");
    assert_doctor_healthy(tmp.path());
}
