//! Integration tests for `strata fortune` through the compiled binary.
//!
//! Selection is random by design, so these tests pin the output shape, the
//! empty states, and membership in the open set — never a specific pick.
//! The staleness bias itself is pinned structurally by the unit tests on
//! the pure weight function.

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

fn dragon(id: &str, sequence: u32, status: &str, title: &str, body: &str) -> String {
    format!(
        "---\nid: {id}\nsequence: {sequence}\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# {title}\n\n## Context\n\n{body}\n"
    )
}

#[test]
fn empty_repository_prints_a_friendly_message_and_exits_zero() {
    let tmp = init_repo();

    let out = strata_in(tmp.path(), &["fortune"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("no open dragons"), "{}", stdout(&out));
}

#[test]
fn marker_only_repository_prints_the_friendly_message() {
    let tmp = init_repo();
    fs::remove_dir_all(tmp.path().join("archaeology")).unwrap();

    let out = strata_in(tmp.path(), &["fortune"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("no open dragons"), "{}", stdout(&out));
}

#[test]
fn closed_dragons_alone_are_never_recalled() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-slain.md"),
        dragon("id-1", 1, "closed", "Slain risk", "Long resolved."),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["fortune"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("no open dragons"), "{}", stdout(&out));
}

#[test]
fn output_names_reference_title_age_path_and_excerpt() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-lone-risk.md"),
        dragon(
            "id-1",
            1,
            "open",
            "Lone risk",
            "The first prose line of the risk.",
        ),
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["fortune"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    for needle in [
        "dragon:1",
        "Lone risk",
        "open",
        "archaeology/dragons/0001-lone-risk.md",
        "  The first prose line of the risk.",
    ] {
        assert!(text.contains(needle), "missing `{needle}`:\n{text}");
    }
}

#[test]
fn unparseable_created_stamp_degrades_to_age_unknown() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-undated.md"),
        "---\nid: id-1\nsequence: 1\nkind: dragon\nstatus: open\ncreated: sometime\n---\n\n# Undated risk\n",
    )
    .unwrap();

    let out = strata_in(tmp.path(), &["fortune"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("age unknown"), "{}", stdout(&out));
}

#[test]
fn every_recall_names_an_open_dragon_and_never_a_closed_one() {
    let tmp = init_repo();
    let open_titles = ["First open risk", "Second open risk", "Third open risk"];
    for (i, title) in open_titles.iter().enumerate() {
        let sequence = i as u32 + 1;
        fs::write(
            tmp.path()
                .join(DRAGONS_DIR)
                .join(format!("000{sequence}-open-{sequence}.md")),
            dragon(
                &format!("id-open-{sequence}"),
                sequence,
                "open",
                title,
                "Prose.",
            ),
        )
        .unwrap();
    }
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0004-closed.md"),
        dragon("id-closed", 4, "closed", "Closed risk", "Prose."),
    )
    .unwrap();

    for _ in 0..12 {
        let out = strata_in(tmp.path(), &["fortune"]);
        assert!(out.status.success(), "{}", stderr(&out));
        let text = stdout(&out);
        assert!(
            open_titles.iter().any(|title| text.contains(title)),
            "recall must name a member of the open set:\n{text}"
        );
        assert!(
            !text.contains("Closed risk"),
            "closed dragons must never be recalled:\n{text}"
        );
    }
}

fn idea(id: &str, sequence: u32, status: &str, title: &str, body: &str) -> String {
    format!(
        "---\nid: {id}\nsequence: {sequence}\nkind: idea\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# {title}\n\n## Problem\n\n{body}\n"
    )
}

fn seed_idea(root: &Path, dir: &str, name: &str, content: &str) {
    fs::create_dir_all(root.join(dir)).unwrap();
    fs::write(root.join(dir).join(name), content).unwrap();
}

#[test]
fn a_lone_parked_idea_is_recalled_with_the_full_output_shape() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        "archaeology/ideas",
        "0001-lone-idea.md",
        &idea(
            "id-1",
            1,
            "parked",
            "Lone idea",
            "The proposal's first line.",
        ),
    );

    let out = strata_in(tmp.path(), &["fortune"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    for needle in [
        "idea:1",
        "Lone idea",
        "archaeology/ideas/0001-lone-idea.md",
        "  The proposal's first line.",
    ] {
        assert!(text.contains(needle), "missing `{needle}`:\n{text}");
    }
}

#[test]
fn recalls_span_both_collections_and_never_terminal_states() {
    let tmp = init_repo();
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0001-open.md"),
        dragon("id-open", 1, "open", "Open risk", "Prose."),
    )
    .unwrap();
    fs::write(
        tmp.path().join(DRAGONS_DIR).join("0002-closed.md"),
        dragon("id-closed", 2, "closed", "Closed risk", "Prose."),
    )
    .unwrap();
    seed_idea(
        tmp.path(),
        "archaeology/ideas",
        "0001-parked.md",
        &idea("id-parked", 1, "parked", "Parked idea", "Prose."),
    );
    seed_idea(
        tmp.path(),
        "archaeology/ideas",
        "0002-adopted.md",
        &idea("id-adopted", 2, "adopted", "Adopted idea", "Prose."),
    );

    let mut seen_dragon = false;
    let mut seen_idea = false;
    for _ in 0..40 {
        let out = strata_in(tmp.path(), &["fortune"]);
        assert!(out.status.success(), "{}", stderr(&out));
        let text = stdout(&out);
        assert!(
            !text.contains("Closed risk") && !text.contains("Adopted idea"),
            "terminal states must never surface:\n{text}"
        );
        seen_dragon |= text.contains("Open risk");
        seen_idea |= text.contains("Parked idea");
        if seen_dragon && seen_idea {
            break;
        }
    }
    // Equal ages give each candidate probability 1/2 per draw; forty draws
    // failing to surface both is ~2^-39 — vanishing, not flaky.
    assert!(
        seen_dragon && seen_idea,
        "both collections must be reachable (dragon: {seen_dragon}, idea: {seen_idea})"
    );
}

#[test]
fn the_empty_state_names_both_collections() {
    let tmp = init_repo();
    seed_idea(
        tmp.path(),
        "archaeology/ideas",
        "0001-adopted.md",
        &idea("id-adopted", 1, "adopted", "Adopted idea", "Prose."),
    );

    let out = strata_in(tmp.path(), &["fortune"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("no open dragons or parked ideas"), "{text}");
    assert!(text.contains("strata new idea"), "{text}");
}

#[test]
fn fortune_never_mutates_the_repository() {
    let tmp = init_repo();
    let content = dragon("id-1", 1, "open", "Stable risk", "Prose.");
    let path = tmp.path().join(DRAGONS_DIR).join("0001-stable-risk.md");
    fs::write(&path, &content).unwrap();

    for _ in 0..3 {
        assert!(strata_in(tmp.path(), &["fortune"]).status.success());
    }

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        content,
        "fortune is read-only"
    );
    let doctor = strata_in(tmp.path(), &["doctor"]);
    assert!(doctor.status.success(), "{}", stdout(&doctor));
}
