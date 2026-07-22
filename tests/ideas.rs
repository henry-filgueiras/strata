//! Integration tests for the `idea` collection through the compiled
//! binary: creation, listing, show, the `adopt`/`reject` transitions, and
//! the lifecycle rules that differ from dragons (terminal states are
//! permanent; transition verbs are collection-scoped).

use std::fs;
use std::path::Path;
use std::process::Output;

const PARKED_DIR: &str = "archaeology/ideas/parked";
const ADOPTED_DIR: &str = "archaeology/ideas/adopted";
const REJECTED_DIR: &str = "archaeology/ideas/rejected";
const DRAGONS_OPEN_DIR: &str = "archaeology/dragons/open";

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

/// A hand-seeded idea in the style of this repository's pre-CLI corpus:
/// slug id, extra prose, trailing whitespace to preserve.
fn rich_idea(status: &str) -> String {
    format!(
        "---\nid: idea-hand-seeded\nsequence: 1\nkind: idea\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# Hand-seeded idea\n\nProse mentioning status: parked stays put.  \n\n## Evidence\n\nSome evidence.\n"
    )
}

fn seed_idea(root: &Path, dir: &str, name: &str, content: &str) {
    fs::create_dir_all(root.join(dir)).unwrap();
    fs::write(root.join(dir).join(name), content).unwrap();
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
fn new_idea_creates_a_parked_artifact_with_generated_identity() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["new", "idea", "Chore ledgers"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("idea:1"), "{}", stdout(&out));
    let path = tmp.path().join(PARKED_DIR).join("0001-chore-ledgers.md");
    let content = fs::read_to_string(&path).unwrap();
    for needle in [
        "kind: idea",
        "status: parked",
        "# Chore ledgers",
        "## Problem",
    ] {
        assert!(content.contains(needle), "missing `{needle}`:\n{content}");
    }
    let id = content
        .lines()
        .find_map(|line| line.strip_prefix("id: "))
        .expect("created idea carries an id");
    assert!(
        id.starts_with("ide_"),
        "generated idea ids use `ide_`: {id}"
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn idea_and_dragon_sequences_are_independent() {
    let tmp = init_repo();
    assert!(
        strata_in(tmp.path(), &["new", "dragon", "A risk"])
            .status
            .success()
    );

    let out = strata_in(tmp.path(), &["new", "idea", "An idea"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("idea:1"),
        "idea sequences must not continue the dragon collection:\n{}",
        stdout(&out)
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn list_ideas_spans_every_lifecycle_directory() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        PARKED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("parked"),
    );
    seed_idea(
        tmp.path(),
        ADOPTED_DIR,
        "0002-adopted-idea.md",
        "---\nid: idea-adopted\nsequence: 2\nkind: idea\nstatus: adopted\ncreated: 2026-07-21\n---\n\n# Adopted idea\n",
    );

    let out = strata_in(tmp.path(), &["list", "ideas"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    for needle in ["idea:1", "parked", "Hand-seeded idea", "idea:2", "adopted"] {
        assert!(text.contains(needle), "missing `{needle}`:\n{text}");
    }
}

#[test]
fn list_ideas_json_pins_field_names_and_values() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        PARKED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("parked"),
    );

    let out = strata_in(tmp.path(), &["list", "ideas", "--json"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let expected = concat!(
        "[",
        "{\"id\":\"idea-hand-seeded\",\"sequence\":1,",
        "\"kind\":\"idea\",\"status\":\"parked\",\"title\":\"Hand-seeded idea\",",
        "\"created\":\"2026-07-20\",",
        "\"path\":\"archaeology/ideas/parked/0001-hand-seeded-idea.md\"}",
        "]\n"
    );
    assert_eq!(stdout(&out), expected);
}

#[test]
fn empty_idea_collection_prints_a_clear_message() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["list", "ideas"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("no ideas found"), "{}", stdout(&out));
    assert!(stdout(&out).contains("strata new idea"), "{}", stdout(&out));
}

#[test]
fn show_resolves_idea_references_and_hand_seeded_ids() {
    let tmp = init_repo();
    let content = rich_idea("parked");
    seed_idea(tmp.path(), PARKED_DIR, "0001-hand-seeded-idea.md", &content);

    let by_reference = strata_in(tmp.path(), &["show", "idea:1"]);
    assert!(by_reference.status.success(), "{}", stderr(&by_reference));
    assert_eq!(stdout(&by_reference), content);

    let by_id = strata_in(tmp.path(), &["show", "idea-hand-seeded"]);
    assert!(by_id.status.success(), "{}", stderr(&by_id));
    assert_eq!(stdout(&by_id), content);
}

#[test]
fn dragon_and_idea_with_the_same_sequence_resolve_independently() {
    let tmp = init_repo();
    let dragon = "---\nid: drg-one\nsequence: 1\nkind: dragon\nstatus: open\ncreated: 2026-07-20\n---\n\n# Dragon one\n";
    fs::write(
        tmp.path().join(DRAGONS_OPEN_DIR).join("0001-dragon-one.md"),
        dragon,
    )
    .unwrap();
    seed_idea(
        tmp.path(),
        PARKED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("parked"),
    );

    let dragon_shown = strata_in(tmp.path(), &["show", "dragon:1"]);
    assert!(dragon_shown.status.success(), "{}", stderr(&dragon_shown));
    assert_eq!(stdout(&dragon_shown), dragon);

    let idea_shown = strata_in(tmp.path(), &["show", "idea:1"]);
    assert!(idea_shown.status.success(), "{}", stderr(&idea_shown));
    assert_eq!(stdout(&idea_shown), rich_idea("parked"));
}

#[test]
fn adopt_moves_the_idea_and_rewrites_only_the_status() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        PARKED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("parked"),
    );

    let out = strata_in(tmp.path(), &["adopt", "idea:1"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let line = stdout(&out);
    for needle in [
        "adopted idea:1",
        "parked -> adopted",
        "archaeology/ideas/adopted/0001-hand-seeded-idea.md",
    ] {
        assert!(
            line.contains(needle),
            "output must name `{needle}`:\n{line}"
        );
    }
    assert!(
        !tmp.path()
            .join(PARKED_DIR)
            .join("0001-hand-seeded-idea.md")
            .exists()
    );
    assert_eq!(
        fs::read_to_string(
            tmp.path()
                .join(ADOPTED_DIR)
                .join("0001-hand-seeded-idea.md")
        )
        .unwrap(),
        rich_idea("adopted"),
        "every byte except the status value must be preserved"
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn reject_materializes_the_rejected_directory_on_first_use() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        PARKED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("parked"),
    );
    assert!(!tmp.path().join(REJECTED_DIR).exists());

    let out = strata_in(tmp.path(), &["reject", "idea-hand-seeded"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        fs::read_to_string(
            tmp.path()
                .join(REJECTED_DIR)
                .join("0001-hand-seeded-idea.md")
        )
        .unwrap(),
        rich_idea("rejected")
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn terminal_idea_states_are_permanent() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        ADOPTED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("adopted"),
    );

    let out = strata_in(tmp.path(), &["reject", "idea:1"]);

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[invalid-invocation]:"), "{err}");
    assert!(
        err.contains("parked -> adopted, parked -> rejected"),
        "the error must name the idea lifecycle:\n{err}"
    );
    assert!(
        tmp.path()
            .join(ADOPTED_DIR)
            .join("0001-hand-seeded-idea.md")
            .is_file(),
        "the artifact must be untouched"
    );
}

#[test]
fn adopting_an_already_adopted_idea_is_an_invalid_invocation() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        ADOPTED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("adopted"),
    );

    let out = strata_in(tmp.path(), &["adopt", "idea:1"]);

    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stderr(&out).contains("already adopted"), "{}", stderr(&out));
}

#[test]
fn transition_verbs_are_collection_scoped() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        PARKED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("parked"),
    );
    let dragon = "---\nid: drg-one\nsequence: 1\nkind: dragon\nstatus: open\ncreated: 2026-07-20\n---\n\n# Dragon one\n";
    fs::write(
        tmp.path().join(DRAGONS_OPEN_DIR).join("0001-dragon-one.md"),
        dragon,
    )
    .unwrap();

    let close_idea = strata_in(tmp.path(), &["close", "idea:1"]);
    assert_eq!(close_idea.status.code(), Some(2), "{}", stderr(&close_idea));
    assert!(
        stderr(&close_idea).contains("strata adopt"),
        "the refusal must point at the idea verbs:\n{}",
        stderr(&close_idea)
    );

    let adopt_dragon = strata_in(tmp.path(), &["adopt", "dragon:1"]);
    assert_eq!(
        adopt_dragon.status.code(),
        Some(2),
        "{}",
        stderr(&adopt_dragon)
    );
    assert!(
        stderr(&adopt_dragon).contains("strata close"),
        "the refusal must point at the dragon verbs:\n{}",
        stderr(&adopt_dragon)
    );

    // Nothing moved.
    assert!(
        tmp.path()
            .join(PARKED_DIR)
            .join("0001-hand-seeded-idea.md")
            .is_file()
    );
    assert!(
        tmp.path()
            .join(DRAGONS_OPEN_DIR)
            .join("0001-dragon-one.md")
            .is_file()
    );
    assert_doctor_healthy(tmp.path());
}

#[test]
fn mismatched_idea_refuses_transition_and_directs_to_doctor() {
    let tmp = init_repo();
    // Status says adopted, placement says parked: the crash-window shape.
    seed_idea(
        tmp.path(),
        PARKED_DIR,
        "0001-hand-seeded-idea.md",
        &rich_idea("adopted"),
    );

    let out = strata_in(tmp.path(), &["adopt", "idea:1"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("lifecycle mismatch"),
        "{}",
        stderr(&out)
    );

    let doctor = strata_in(tmp.path(), &["doctor"]);
    assert_eq!(doctor.status.code(), Some(9));
    assert!(
        stdout(&doctor).contains("lifecycle mismatch"),
        "{}",
        stdout(&doctor)
    );
}

#[test]
fn unknown_idea_reference_is_artifact_not_found() {
    let tmp = init_repo();
    let out = strata_in(tmp.path(), &["adopt", "idea:41"]);
    assert_eq!(out.status.code(), Some(7), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[artifact-not-found]:"),
        "{}",
        stderr(&out)
    );
}
