//! Integration tests for the `task` collection through the compiled
//! binary: creation in an active sprint (inferred or `--sprint`-selected
//! per decision 15), discovery across sprints, listing with the active
//! filter, and closure.

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

// --- decision 15: concurrent active sprints and explicit placement ---

/// Two active sprints created through the CLI; returns the stable id of
/// the second, harvested from `new sprint --json`.
fn seed_two_active_sprints(root: &Path) -> String {
    assert!(
        strata_in(root, &["new", "sprint", "Alpha"])
            .status
            .success()
    );
    let out = strata_in(root, &["new", "sprint", "Beta", "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let payload: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    payload["id"].as_str().unwrap().to_string()
}

#[test]
fn bare_task_creation_with_multiple_active_sprints_refuses_naming_all() {
    let tmp = init_repo();
    seed_two_active_sprints(tmp.path());

    let out = strata_in(tmp.path(), &["new", "task", "Homeless work"]);

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[invalid-invocation]:"), "{err}");
    for needle in ["sprint:1", "Alpha", "sprint:2", "Beta", "--sprint"] {
        assert!(err.contains(needle), "missing `{needle}`:\n{err}");
    }
    for dir in ["0001-alpha", "0002-beta"] {
        let entries: Vec<_> = fs::read_dir(tmp.path().join(SPRINTS_DIR).join(dir))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(entries, vec!["sprint.md"], "refusal must write nothing");
    }
}

#[test]
fn explicit_sequence_selection_places_the_task_in_the_chosen_sprint() {
    let tmp = init_repo();
    seed_two_active_sprints(tmp.path());

    let out = strata_in(
        tmp.path(),
        &["new", "task", "Alpha work", "--sprint", "sprint:1"],
    );

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        tmp.path()
            .join(SPRINTS_DIR)
            .join("0001-alpha/0001-alpha-work.md")
            .is_file(),
        "the task lands only in the chosen sprint"
    );
    assert!(
        !tmp.path()
            .join(SPRINTS_DIR)
            .join("0002-beta/0001-alpha-work.md")
            .exists()
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn explicit_stable_id_selection_places_the_task_in_the_chosen_sprint() {
    let tmp = init_repo();
    let beta_id = seed_two_active_sprints(tmp.path());

    let out = strata_in(
        tmp.path(),
        &["new", "task", "Beta work", "--sprint", &beta_id],
    );

    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(
        tmp.path()
            .join(SPRINTS_DIR)
            .join("0002-beta/0001-beta-work.md"),
    )
    .unwrap();
    assert!(
        content.contains(&format!("sprint: {beta_id}")),
        "the task carries the chosen sprint's id:\n{content}"
    );
}

#[test]
fn a_closed_selected_sprint_is_refused_before_writing() {
    let tmp = init_repo();
    seed_closed_sprint_with_task(tmp.path(), 1, 1);
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Current"])
            .status
            .success()
    );

    let out = strata_in(
        tmp.path(),
        &["new", "task", "Late work", "--sprint", "sprint:1"],
    );

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("sprint:1") && err.contains("closed"), "{err}");
    let entries: Vec<_> = fs::read_dir(tmp.path().join(SPRINTS_DIR).join("0001-history"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(entries.len(), 2, "the refusal must write nothing");
}

#[test]
fn non_sprint_selectors_and_misplaced_sprint_flags_are_refused() {
    let tmp = init_repo();
    seed_two_active_sprints(tmp.path());

    // A sequence reference into a non-sprint collection cannot name the
    // owning sprint.
    let out = strata_in(
        tmp.path(),
        &["new", "task", "Confused", "--sprint", "dragon:1"],
    );
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stderr(&out).contains("dragon"), "{}", stderr(&out));

    // `--sprint` belongs to task creation only.
    for kind in ["dragon", "idea", "sprint"] {
        let out = strata_in(
            tmp.path(),
            &["new", kind, "Misflagged", "--sprint", "sprint:1"],
        );
        assert_eq!(out.status.code(), Some(2), "{kind}: {}", stderr(&out));
        assert!(
            stderr(&out).starts_with("error[invalid-invocation]:"),
            "{}",
            stderr(&out)
        );
        assert!(stderr(&out).contains(kind), "{}", stderr(&out));
    }
}

#[test]
fn task_sequences_are_global_across_concurrent_sprints() {
    let tmp = init_repo();
    seed_two_active_sprints(tmp.path());

    let first = strata_in(
        tmp.path(),
        &["new", "task", "In alpha", "--sprint", "sprint:1"],
    );
    assert!(first.status.success(), "{}", stderr(&first));
    assert!(
        stdout(&first).contains("created task:1"),
        "{}",
        stdout(&first)
    );

    let second = strata_in(
        tmp.path(),
        &["new", "task", "In beta", "--sprint", "sprint:2"],
    );
    assert!(second.status.success(), "{}", stderr(&second));
    assert!(
        stdout(&second).contains("created task:2"),
        "sequences span concurrent sprints: {}",
        stdout(&second)
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn list_tasks_active_is_the_union_across_all_active_sprints() {
    let tmp = init_repo();
    seed_closed_sprint_with_task(tmp.path(), 1, 1);
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Alpha"])
            .status
            .success()
    );
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Beta"])
            .status
            .success()
    );
    for (title, sprint) in [("Alpha work", "sprint:2"), ("Beta work", "sprint:3")] {
        assert!(
            strata_in(tmp.path(), &["new", "task", title, "--sprint", sprint])
                .status
                .success()
        );
    }

    let human = strata_in(tmp.path(), &["list", "tasks", "--active"]);
    assert!(human.status.success(), "{}", stderr(&human));
    let text = stdout(&human);
    assert!(
        text.contains("task:2") && text.contains("task:3"),
        "the union spans every active sprint:\n{text}"
    );
    assert!(
        !text.contains("task:1"),
        "closed sprints' tasks are excluded:\n{text}"
    );

    let json = strata_in(tmp.path(), &["list", "tasks", "--active", "--json"]);
    assert!(json.status.success(), "{}", stderr(&json));
    let listed: serde_json::Value = serde_json::from_str(stdout(&json).trim()).unwrap();
    let tasks = listed.as_array().unwrap();
    let sequences: Vec<u64> = tasks
        .iter()
        .map(|t| t["sequence"].as_u64().unwrap())
        .collect();
    assert_eq!(
        sequences,
        vec![2, 3],
        "same filtered set, deterministic global order"
    );
}
