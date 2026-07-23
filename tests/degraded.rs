//! Integration tests for degraded-corpus operability (decision 13,
//! task 27) through the compiled binary.
//!
//! Flat creation succeeds beside a malformed sibling with honest
//! reachability reporting: the stable `warning[degraded-repository]:`
//! stderr line qualifies the success, exit status stays 0, and strict
//! reads keep refusing while naming both the requested target and the
//! blocking sibling. Removing only the blocker restores full access.

use std::fs;
use std::path::Path;
use std::process::Output;

const DRAGONS_DIR: &str = "archaeology/dragons";
const IDEAS_DIR: &str = "archaeology/ideas";
const SPRINTS_DIR: &str = "archaeology/sprints";
const WARNING: &str = "warning[degraded-repository]:";

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

/// A malformed sibling occupying dragon sequence 1: a conforming
/// filename whose content has no front matter.
fn seed_malformed_sibling(root: &Path) -> &'static str {
    fs::write(
        root.join(DRAGONS_DIR).join("0001-junk.md"),
        "no front matter here\n",
    )
    .unwrap();
    "0001-junk.md"
}

fn created_id(root: &Path, rel_path: &str) -> String {
    fs::read_to_string(root.join(rel_path))
        .unwrap()
        .lines()
        .find_map(|l| l.strip_prefix("id: ").map(str::to_string))
        .expect("created artifact carries an id")
}

#[test]
fn creation_beside_a_malformed_sibling_allocates_past_it_with_the_warning() {
    let tmp = init_repo();
    let sibling = seed_malformed_sibling(tmp.path());

    let out = strata_in(tmp.path(), &["new", "dragon", "Degraded but created"]);

    // The malformed sibling reserves sequence 1; creation allocates past
    // it and reports success on stdout, qualified on stderr.
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("created dragon:2"), "{text}");
    let warning = stderr(&out);
    assert!(warning.starts_with(WARNING), "{warning}");
    for needle in [
        "dragon:2",
        "archaeology/dragons/0002-degraded-but-created.md",
        sibling,
        "exit status remains success",
        "repairing the blocker restores normal access",
    ] {
        assert!(warning.contains(needle), "missing `{needle}`:\n{warning}");
    }
    assert!(
        !warning.contains("error["),
        "a successful creation must not carry an error token:\n{warning}"
    );

    // The created artifact itself is individually valid.
    let content = fs::read_to_string(
        tmp.path()
            .join(DRAGONS_DIR)
            .join("0002-degraded-but-created.md"),
    )
    .unwrap();
    assert!(content.contains("# Degraded but created"), "{content}");
}

#[test]
fn new_idea_beside_a_malformed_sibling_is_also_created_with_the_warning() {
    let tmp = init_repo();
    fs::create_dir_all(tmp.path().join(IDEAS_DIR)).unwrap();
    fs::write(tmp.path().join(IDEAS_DIR).join("0003-junk.md"), "junk\n").unwrap();

    let out = strata_in(tmp.path(), &["new", "idea", "Still parked"]);

    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).contains("created idea:4"), "{}", stdout(&out));
    let warning = stderr(&out);
    assert!(warning.starts_with(WARNING), "{warning}");
    assert!(warning.contains("0003-junk.md"), "{warning}");
}

#[test]
fn degraded_json_creation_keeps_stdout_parseable_and_the_warning_on_stderr() {
    let tmp = init_repo();
    seed_malformed_sibling(tmp.path());

    let out = strata_in(tmp.path(), &["new", "dragon", "Machine caller", "--json"]);

    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let payload: serde_json::Value = serde_json::from_str(stdout(&out).trim())
        .expect("degraded --json stdout must remain one valid JSON object");
    assert_eq!(payload["kind"], "dragon");
    assert_eq!(payload["sequence"], 2);
    assert_eq!(payload["reference"], "dragon:2");
    assert_eq!(
        payload["path"],
        "archaeology/dragons/0002-machine-caller.md"
    );
    assert!(
        payload["id"].as_str().unwrap().starts_with("drg_"),
        "{payload}"
    );
    assert!(stderr(&out).starts_with(WARNING), "{}", stderr(&out));
}

#[test]
fn healthy_json_creation_uses_the_same_schema_with_no_warning() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["new", "dragon", "Healthy caller", "--json"]);

    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    let payload: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(payload["reference"], "dragon:1");
    // Field names and order are a pinned compatibility surface.
    assert!(
        text.starts_with("{\"kind\":\"dragon\",\"id\":\"drg_"),
        "{text}"
    );
    assert!(
        text.trim_end()
            .ends_with("\"path\":\"archaeology/dragons/0001-healthy-caller.md\"}"),
        "{text}"
    );
    assert_eq!(stderr(&out), "", "healthy creation emits no warning");
}

#[test]
fn healthy_human_creation_emits_no_warning() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["new", "dragon", "Plainly reachable"]);

    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("created dragon:1"),
        "{}",
        stdout(&out)
    );
    assert_eq!(stderr(&out), "");
}

#[test]
fn json_creation_covers_sprint_and_task_kinds() {
    let tmp = init_repo();

    let sprint = strata_in(tmp.path(), &["new", "sprint", "Machine sprint", "--json"]);
    assert_eq!(sprint.status.code(), Some(0), "{}", stderr(&sprint));
    let payload: serde_json::Value = serde_json::from_str(stdout(&sprint).trim()).unwrap();
    assert_eq!(payload["kind"], "sprint");
    assert_eq!(payload["reference"], "sprint:1");
    assert_eq!(
        payload["path"],
        "archaeology/sprints/0001-machine-sprint/sprint.md"
    );

    let task = strata_in(tmp.path(), &["new", "task", "Machine task", "--json"]);
    assert_eq!(task.status.code(), Some(0), "{}", stderr(&task));
    let payload: serde_json::Value = serde_json::from_str(stdout(&task).trim()).unwrap();
    assert_eq!(payload["kind"], "task");
    assert_eq!(payload["reference"], "task:1");
    assert!(
        payload["id"].as_str().unwrap().starts_with("tsk_"),
        "{payload}"
    );
}

#[test]
fn creation_adds_no_doctor_finding_beyond_the_preexisting_sibling() {
    let tmp = init_repo();
    seed_malformed_sibling(tmp.path());
    let before = strata_in(tmp.path(), &["doctor", "--json"]);
    assert_eq!(before.status.code(), Some(9), "{}", stderr(&before));

    let created = strata_in(tmp.path(), &["new", "dragon", "No new findings"]);
    assert_eq!(created.status.code(), Some(0), "{}", stderr(&created));

    let after = strata_in(tmp.path(), &["doctor", "--json"]);
    assert_eq!(after.status.code(), Some(9), "{}", stderr(&after));
    assert_eq!(
        stdout(&before),
        stdout(&after),
        "creation must not introduce a new doctor finding"
    );
}

#[test]
fn show_by_sequence_and_id_refuses_naming_target_and_blocker() {
    let tmp = init_repo();
    let sibling = seed_malformed_sibling(tmp.path());
    let out = strata_in(tmp.path(), &["new", "dragon", "Blocked target"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let id = created_id(tmp.path(), "archaeology/dragons/0002-blocked-target.md");

    for (target, spelled) in [("dragon:2", "dragon:2"), (id.as_str(), id.as_str())] {
        let shown = strata_in(tmp.path(), &["show", target]);
        assert_eq!(shown.status.code(), Some(5), "{}", stderr(&shown));
        let err = stderr(&shown);
        assert!(
            err.starts_with("error[malformed-artifact]: "),
            "the original typed category is preserved:\n{err}"
        );
        assert!(err.contains(sibling), "must name the blocker:\n{err}");
        assert!(
            err.contains(&format!("`{spelled}`")),
            "must name the requested target:\n{err}"
        );
    }
}

#[test]
fn list_remains_strict_and_names_the_blocker() {
    let tmp = init_repo();
    let sibling = seed_malformed_sibling(tmp.path());
    strata_in(tmp.path(), &["new", "dragon", "Unlistable"]);

    let out = strata_in(tmp.path(), &["list", "dragons"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    assert!(stderr(&out).contains(sibling), "{}", stderr(&out));
}

#[test]
fn admitted_transition_refuses_naming_target_and_blocker_leaving_bytes_unchanged() {
    let tmp = init_repo();
    let sibling = seed_malformed_sibling(tmp.path());
    strata_in(tmp.path(), &["new", "dragon", "Untransitionable"]);
    let created_path = tmp
        .path()
        .join(DRAGONS_DIR)
        .join("0002-untransitionable.md");
    let created_bytes = fs::read_to_string(&created_path).unwrap();

    let out = strata_in(tmp.path(), &["close", "dragon:2"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[malformed-artifact]: "), "{err}");
    assert!(err.contains(sibling), "must name the blocker:\n{err}");
    assert!(err.contains("`dragon:2`"), "must name the target:\n{err}");
    assert_eq!(
        fs::read_to_string(&created_path).unwrap(),
        created_bytes,
        "a refused transition leaves every file unchanged"
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join(DRAGONS_DIR).join(sibling)).unwrap(),
        "no front matter here\n",
        "the blocking sibling is evidence, never touched"
    );
}

#[test]
fn removing_only_the_blocker_restores_full_access_without_touching_the_artifact() {
    let tmp = init_repo();
    let sibling = seed_malformed_sibling(tmp.path());
    strata_in(tmp.path(), &["new", "dragon", "Patient artifact"]);
    let created_path = tmp
        .path()
        .join(DRAGONS_DIR)
        .join("0002-patient-artifact.md");
    let created_bytes = fs::read_to_string(&created_path).unwrap();
    let id = created_id(tmp.path(), "archaeology/dragons/0002-patient-artifact.md");

    fs::remove_file(tmp.path().join(DRAGONS_DIR).join(sibling)).unwrap();

    let listed = strata_in(tmp.path(), &["list", "dragons"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
    assert!(stdout(&listed).contains("dragon:2"), "{}", stdout(&listed));
    for target in ["dragon:2", id.as_str()] {
        let shown = strata_in(tmp.path(), &["show", target]);
        assert!(shown.status.success(), "{target}: {}", stderr(&shown));
        assert_eq!(stdout(&shown), created_bytes, "show is byte-exact");
    }
    let closed = strata_in(tmp.path(), &["close", "dragon:2"]);
    assert!(closed.status.success(), "{}", stderr(&closed));
    assert_eq!(
        fs::read_to_string(&created_path).unwrap(),
        created_bytes.replace("status: open", "status: closed"),
        "recovery required no repair to the created artifact itself"
    );
}

#[test]
fn malformed_duplicate_sequence_and_id_claimants_are_never_bypassed() {
    let tmp = init_repo();
    let created = strata_in(tmp.path(), &["new", "dragon", "Contested"]);
    assert_eq!(created.status.code(), Some(0), "{}", stderr(&created));
    let id = created_id(tmp.path(), "archaeology/dragons/0001-contested.md");

    // A malformed file claiming the same sequence: resolution must refuse
    // rather than silently pick the valid claimant.
    let claimant = tmp.path().join(DRAGONS_DIR).join("0001-impostor.md");
    fs::write(&claimant, "no front matter, claims sequence 1\n").unwrap();
    let by_sequence = strata_in(tmp.path(), &["show", "dragon:1"]);
    assert_eq!(
        by_sequence.status.code(),
        Some(5),
        "{}",
        stderr(&by_sequence)
    );
    assert!(
        stderr(&by_sequence).contains("0001-impostor.md"),
        "the malformed claimant is evidence, not noise:\n{}",
        stderr(&by_sequence)
    );

    // A malformed file claiming the same stable id: same refusal by id.
    fs::write(
        &claimant,
        format!("---\nid: {id}\nsequence: 1\nkind: dragon\n---\nbroken\n"),
    )
    .unwrap();
    let by_id = strata_in(tmp.path(), &["show", &id]);
    assert_eq!(by_id.status.code(), Some(5), "{}", stderr(&by_id));
    assert!(
        stderr(&by_id).contains("0001-impostor.md"),
        "{}",
        stderr(&by_id)
    );
}

#[test]
fn sprint_and_task_creation_keep_the_strict_containment_boundary() {
    let tmp = init_repo();
    fs::create_dir_all(tmp.path().join(SPRINTS_DIR)).unwrap();
    fs::write(tmp.path().join(SPRINTS_DIR).join("notes.md"), "loose\n").unwrap();

    let sprint = strata_in(tmp.path(), &["new", "sprint", "Refused sprint"]);
    assert_eq!(sprint.status.code(), Some(5), "{}", stderr(&sprint));
    assert!(
        !tmp.path()
            .join(SPRINTS_DIR)
            .join("0001-refused-sprint")
            .exists(),
        "a refused sprint creation writes nothing"
    );

    let task = strata_in(tmp.path(), &["new", "task", "Refused task"]);
    assert_eq!(task.status.code(), Some(5), "{}", stderr(&task));
    for out in [&sprint, &task] {
        assert!(
            !stderr(out).contains(WARNING),
            "containment creation never degrades:\n{}",
            stderr(out)
        );
    }
}
