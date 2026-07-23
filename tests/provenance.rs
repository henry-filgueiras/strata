//! Integration tests for provenance riding the transition commands:
//! `close --resolved-by` and `adopt --adopted-by` write the typed edge
//! and the transition in one invocation (task 21, decision 10 vocabulary).

use std::fs;
use std::path::Path;
use std::process::Output;

const DRAGONS_DIR: &str = "archaeology/dragons";
const IDEAS_DIR: &str = "archaeology/ideas";
const DECISIONS_DIR: &str = "archaeology/decisions";

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

fn seed_dragon(root: &Path) {
    fs::write(
        root.join(DRAGONS_DIR).join("0001-risk.md"),
        "---\nid: drg-risk\nsequence: 1\nkind: dragon\nstatus: open\ncreated: 2026-07-20\n---\n\n# A risk\n",
    )
    .unwrap();
}

/// A decision is an unmanaged collection; the provenance target universe
/// must still reach it.
fn seed_decision(root: &Path) {
    fs::create_dir_all(root.join(DECISIONS_DIR)).unwrap();
    fs::write(
        root.join(DECISIONS_DIR).join("0001-settle-it.md"),
        "---\nid: dec-settle-it\nsequence: 1\nkind: decision\nstatus: accepted\ncreated: 2026-07-20\n---\n\n# Settle the risk\n",
    )
    .unwrap();
}

fn seed_idea(root: &Path) {
    fs::create_dir_all(root.join(IDEAS_DIR)).unwrap();
    fs::write(
        root.join(IDEAS_DIR).join("0001-proposal.md"),
        "---\nid: idea-proposal\nsequence: 1\nkind: idea\nstatus: parked\ncreated: 2026-07-20\n---\n\n# A proposal\n",
    )
    .unwrap();
}

#[test]
fn close_resolved_by_writes_the_bound_edge_and_transition_together() {
    let tmp = init_repo();
    seed_dragon(tmp.path());
    seed_decision(tmp.path());

    // A sequence-form target is resolved to its stable id and the
    // target's title is frozen as the label.
    let out = strata_in(
        tmp.path(),
        &["close", "dragon:1", "--resolved-by", "decision:1"],
    );

    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-risk.md")).unwrap();
    assert!(content.contains("status: closed"), "{content}");
    assert!(
        content.contains("resolved-by: \"[[dec-settle-it|Settle the risk]]\""),
        "{content}"
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn adopt_adopted_by_accepts_a_stable_id_target() {
    let tmp = init_repo();
    seed_idea(tmp.path());
    seed_decision(tmp.path());

    let out = strata_in(
        tmp.path(),
        &["adopt", "idea:1", "--adopted-by", "dec-settle-it"],
    );

    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(tmp.path().join(IDEAS_DIR).join("0001-proposal.md")).unwrap();
    assert!(content.contains("status: adopted"), "{content}");
    assert!(
        content.contains("adopted-by: \"[[dec-settle-it|Settle the risk]]\""),
        "{content}"
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn an_unresolvable_target_fails_the_whole_invocation() {
    let tmp = init_repo();
    seed_dragon(tmp.path());
    let original = fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-risk.md")).unwrap();

    let out = strata_in(
        tmp.path(),
        &["close", "dragon:1", "--resolved-by", "decision:41"],
    );

    assert_eq!(out.status.code(), Some(7), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[artifact-not-found]:"),
        "{}",
        stderr(&out)
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-risk.md")).unwrap(),
        original,
        "no transition without its edge"
    );
}

#[test]
fn an_idea_target_is_refused_by_the_vocabulary() {
    let tmp = init_repo();
    seed_dragon(tmp.path());
    seed_idea(tmp.path());

    let out = strata_in(
        tmp.path(),
        &["close", "dragon:1", "--resolved-by", "idea-proposal"],
    );

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("idea"), "{err}");
    assert!(err.contains("decision, task"), "{err}");
    let content = fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-risk.md")).unwrap();
    assert!(content.contains("status: open"), "nothing may change");
}

#[test]
fn an_ambiguous_stable_id_target_is_refused_naming_every_claimant() {
    // Task 23: a second admitted claimant of the target id — here an
    // unmanaged log — makes the stable-id arm refuse with the same
    // `ambiguous-reference` contract as the `kind:N` arm, before any
    // mutation.
    let tmp = init_repo();
    seed_dragon(tmp.path());
    seed_decision(tmp.path());
    fs::create_dir_all(tmp.path().join("archaeology/logs")).unwrap();
    fs::write(
        tmp.path().join("archaeology/logs/0001-imposter.md"),
        "---\nid: dec-settle-it\nkind: log\n---\n\n# Imposter\n",
    )
    .unwrap();
    let original = fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-risk.md")).unwrap();

    let out = strata_in(
        tmp.path(),
        &["close", "dragon:1", "--resolved-by", "dec-settle-it"],
    );

    assert_eq!(out.status.code(), Some(8), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[ambiguous-reference]:"), "{err}");
    for path in [
        "archaeology/decisions/0001-settle-it.md",
        "archaeology/logs/0001-imposter.md",
    ] {
        assert!(err.contains(path), "missing `{path}`: {err}");
    }
    assert_eq!(
        fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-risk.md")).unwrap(),
        original,
        "refusal must precede any mutation"
    );
}

#[test]
fn bare_transitions_behave_exactly_as_before() {
    let tmp = init_repo();
    seed_dragon(tmp.path());

    let out = strata_in(tmp.path(), &["close", "dragon:1"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-risk.md")).unwrap();
    assert!(content.contains("status: closed"), "{content}");
    assert!(!content.contains("resolved-by"), "{content}");
}

#[test]
fn resolved_by_belongs_to_no_other_collection() {
    let tmp = init_repo();
    assert!(
        strata_in(tmp.path(), &["new", "sprint", "Current"])
            .status
            .success()
    );

    let out = strata_in(
        tmp.path(),
        &["close", "sprint:1", "--resolved-by", "decision:1"],
    );

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("no such edge for sprints"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn an_existing_edge_is_never_silently_rewritten() {
    let tmp = init_repo();
    seed_decision(tmp.path());
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-risk.md"),
        "---\nid: drg-risk\nsequence: 1\nkind: dragon\nstatus: open\ncreated: 2026-07-20\nresolved-by: \"[[dec-settle-it|Settle the risk]]\"\n---\n\n# A risk\n",
    )
    .unwrap();

    let out = strata_in(
        tmp.path(),
        &["close", "dragon:1", "--resolved-by", "decision:1"],
    );

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    assert!(stderr(&out).contains("already carries"), "{}", stderr(&out));
    let content = fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-risk.md")).unwrap();
    assert!(
        content.contains("status: open"),
        "no edge without its transition"
    );
}
