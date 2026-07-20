//! Integration tests for `strata list` and `strata show` through the
//! compiled binary.
//!
//! Every invocation pins its working directory to a fresh temporary
//! directory so discovery can never walk up into a real repository. JSON
//! output shapes asserted here are compatibility surfaces: field names,
//! field order, and ordering are pinned deliberately.

use std::fs;
use std::path::Path;
use std::process::Output;

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

fn dragon_markdown(id: &str, sequence: u32, status: &str, title: &str) -> String {
    format!(
        "---\nid: {id}\nsequence: {sequence}\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# {title}\n\n## Context\n"
    )
}

fn write_artifact(root: &Path, dir: &str, name: &str, content: &str) {
    fs::write(root.join(dir).join(name), content).unwrap();
}

/// Two hand-seeded artifacts with fully known bytes: one open with a
/// legacy non-ULID ID, one closed with a generated-style ULID ID.
fn seed_known_pair(root: &Path) -> (String, String) {
    let open = dragon_markdown("drg-legacy-seeded", 2, "open", "Legacy seeded risk");
    let closed = dragon_markdown(
        "drg_01K0P6W5PK8T19H7M2V8W6YQ4C",
        1,
        "closed",
        "Resolved risk",
    );
    write_artifact(root, OPEN_DIR, "0002-legacy-seeded-risk.md", &open);
    write_artifact(root, CLOSED_DIR, "0001-resolved-risk.md", &closed);
    (open, closed)
}

#[test]
fn list_from_repository_root_prints_reference_status_title_and_path() {
    let tmp = init_repo();
    seed_known_pair(tmp.path());

    let out = strata_in(tmp.path(), &["list", "dragons"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "one line per artifact:\n{text}");
    for needle in [
        "dragon:1",
        "closed",
        "Resolved risk",
        "archaeology/dragons/closed/0001-resolved-risk.md",
    ] {
        assert!(lines[0].contains(needle), "missing `{needle}`:\n{text}");
    }
    for needle in [
        "dragon:2",
        "open",
        "Legacy seeded risk",
        "archaeology/dragons/open/0002-legacy-seeded-risk.md",
    ] {
        assert!(lines[1].contains(needle), "missing `{needle}`:\n{text}");
    }
}

#[test]
fn list_from_nested_directory_finds_the_repository() {
    let tmp = init_repo();
    seed_known_pair(tmp.path());
    let nested = tmp.path().join("src/deeply/nested");
    fs::create_dir_all(&nested).unwrap();

    let out = strata_in(&nested, &["list", "dragons"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("dragon:2"), "{}", stdout(&out));
}

#[test]
fn list_accepts_singular_and_plural_collection_names() {
    let tmp = init_repo();
    for name in ["dragon", "dragons"] {
        let out = strata_in(tmp.path(), &["list", name]);
        assert!(out.status.success(), "`list {name}`:\n{}", stderr(&out));
    }
}

#[test]
fn list_orders_by_sequence_then_path_across_open_and_closed() {
    let tmp = init_repo();
    write_artifact(
        tmp.path(),
        OPEN_DIR,
        "0003-third.md",
        &dragon_markdown("id-3", 3, "open", "Third"),
    );
    write_artifact(
        tmp.path(),
        CLOSED_DIR,
        "0001-first.md",
        &dragon_markdown("id-1", 1, "closed", "First"),
    );
    write_artifact(
        tmp.path(),
        OPEN_DIR,
        "0002-second.md",
        &dragon_markdown("id-2", 2, "open", "Second"),
    );

    let out = strata_in(tmp.path(), &["list", "dragons"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    let positions: Vec<usize> = ["dragon:1", "dragon:2", "dragon:3"]
        .iter()
        .map(|reference| text.find(reference).expect(reference))
        .collect();
    assert!(
        positions[0] < positions[1] && positions[1] < positions[2],
        "sequences must ascend:\n{text}"
    );
}

#[test]
fn list_json_pins_field_names_order_and_sorting() {
    let tmp = init_repo();
    seed_known_pair(tmp.path());

    let out = strata_in(tmp.path(), &["list", "dragons", "--json"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stderr(&out), "", "stderr must stay clean on success");
    let expected = concat!(
        "[",
        "{\"id\":\"drg_01K0P6W5PK8T19H7M2V8W6YQ4C\",\"sequence\":1,",
        "\"kind\":\"dragon\",\"status\":\"closed\",\"title\":\"Resolved risk\",",
        "\"created\":\"2026-07-20\",",
        "\"path\":\"archaeology/dragons/closed/0001-resolved-risk.md\"},",
        "{\"id\":\"drg-legacy-seeded\",\"sequence\":2,",
        "\"kind\":\"dragon\",\"status\":\"open\",\"title\":\"Legacy seeded risk\",",
        "\"created\":\"2026-07-20\",",
        "\"path\":\"archaeology/dragons/open/0002-legacy-seeded-risk.md\"}",
        "]\n"
    );
    assert_eq!(stdout(&out), expected);
}

#[test]
fn empty_collection_prints_a_clear_message_and_an_empty_json_array() {
    let tmp = init_repo();

    let human = strata_in(tmp.path(), &["list", "dragons"]);
    assert!(human.status.success(), "{}", stderr(&human));
    assert!(
        stdout(&human).contains("no dragons found"),
        "{}",
        stdout(&human)
    );

    let json = strata_in(tmp.path(), &["list", "dragons", "--json"]);
    assert!(json.status.success(), "{}", stderr(&json));
    assert_eq!(stdout(&json), "[]\n");
}

#[test]
fn show_by_sequence_prints_canonical_contents_byte_for_byte() {
    let tmp = init_repo();
    let (open, _) = seed_known_pair(tmp.path());

    let out = strata_in(tmp.path(), &["show", "dragon:2"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        out.stdout,
        open.as_bytes(),
        "human show must reproduce the file exactly"
    );
}

#[test]
fn show_by_generated_ulid_style_id() {
    let tmp = init_repo();
    let out = strata_in(tmp.path(), &["new", "dragon", "Freshly created"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let content =
        fs::read_to_string(tmp.path().join(OPEN_DIR).join("0001-freshly-created.md")).unwrap();
    let id = content
        .lines()
        .find_map(|line| line.strip_prefix("id: "))
        .expect("created artifact carries an id");

    let shown = strata_in(tmp.path(), &["show", id]);

    assert!(shown.status.success(), "{}", stderr(&shown));
    assert_eq!(stdout(&shown), content);
}

#[test]
fn show_by_legacy_hand_seeded_id() {
    let tmp = init_repo();
    let (open, _) = seed_known_pair(tmp.path());

    let out = strata_in(tmp.path(), &["show", "drg-legacy-seeded"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), open);
}

#[test]
fn show_json_includes_summary_fields_and_exact_content() {
    let tmp = init_repo();
    let (open, _) = seed_known_pair(tmp.path());

    let out = strata_in(tmp.path(), &["show", "dragon:2", "--json"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stderr(&out), "", "stderr must stay clean on success");
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let object = value.as_object().expect("show --json emits an object");
    assert_eq!(object["id"], "drg-legacy-seeded");
    assert_eq!(object["sequence"], 2);
    assert_eq!(object["kind"], "dragon");
    assert_eq!(object["status"], "open");
    assert_eq!(object["title"], "Legacy seeded risk");
    assert_eq!(object["created"], "2026-07-20");
    assert_eq!(
        object["path"],
        "archaeology/dragons/open/0002-legacy-seeded-risk.md"
    );
    assert_eq!(
        object["content"], open,
        "content must be the canonical bytes exactly"
    );
    assert_eq!(object.len(), 8, "no undocumented fields: {object:?}");
}

#[test]
fn show_unknown_sequence_is_artifact_not_found() {
    let tmp = init_repo();
    seed_known_pair(tmp.path());

    let out = strata_in(tmp.path(), &["show", "dragon:41"]);

    assert_eq!(out.status.code(), Some(7), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[artifact-not-found]: "), "{err}");
    assert!(err.contains("dragon:41"), "must echo the reference:\n{err}");
    assert_eq!(stdout(&out), "", "stdout must stay clean on failure");
}

#[test]
fn show_unknown_id_is_artifact_not_found() {
    let tmp = init_repo();
    seed_known_pair(tmp.path());

    let out = strata_in(tmp.path(), &["show", "drg_00000000000000000000000000"]);

    assert_eq!(out.status.code(), Some(7), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[artifact-not-found]: "),
        "{}",
        stderr(&out)
    );
}

#[test]
fn duplicate_sequence_makes_a_human_reference_ambiguous() {
    let tmp = init_repo();
    write_artifact(
        tmp.path(),
        OPEN_DIR,
        "0001-branch-a.md",
        &dragon_markdown("id-a", 1, "open", "Branch A"),
    );
    write_artifact(
        tmp.path(),
        CLOSED_DIR,
        "0001-branch-b.md",
        &dragon_markdown("id-b", 1, "closed", "Branch B"),
    );

    let out = strata_in(tmp.path(), &["show", "dragon:1"]);

    assert_eq!(out.status.code(), Some(8), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[ambiguous-reference]: "), "{err}");
    for candidate in ["0001-branch-a.md", "0001-branch-b.md"] {
        assert!(err.contains(candidate), "must list `{candidate}`:\n{err}");
    }
    assert_eq!(stdout(&out), "", "stdout must stay clean on failure");
}

#[test]
fn duplicate_stable_id_is_ambiguous() {
    let tmp = init_repo();
    write_artifact(
        tmp.path(),
        OPEN_DIR,
        "0001-copy-a.md",
        &dragon_markdown("id-same", 1, "open", "Copy A"),
    );
    write_artifact(
        tmp.path(),
        OPEN_DIR,
        "0002-copy-b.md",
        &dragon_markdown("id-same", 2, "open", "Copy B"),
    );

    let out = strata_in(tmp.path(), &["show", "id-same"]);

    assert_eq!(out.status.code(), Some(8), "{}", stderr(&out));
    assert!(
        stderr(&out).starts_with("error[ambiguous-reference]: "),
        "{}",
        stderr(&out)
    );
}

#[test]
fn list_still_succeeds_with_duplicate_sequences() {
    let tmp = init_repo();
    write_artifact(
        tmp.path(),
        OPEN_DIR,
        "0001-branch-a.md",
        &dragon_markdown("id-a", 1, "open", "Branch A"),
    );
    write_artifact(
        tmp.path(),
        CLOSED_DIR,
        "0001-branch-b.md",
        &dragon_markdown("id-b", 1, "closed", "Branch B"),
    );

    let out = strata_in(tmp.path(), &["list", "dragons"]);

    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("Branch A") && text.contains("Branch B"),
        "{text}"
    );
}

#[test]
fn malformed_artifact_fails_list_naming_the_path() {
    let tmp = init_repo();
    write_artifact(
        tmp.path(),
        OPEN_DIR,
        "0001-broken.md",
        "---\nid: x\nsequence: 1\n---\n\n# Missing required fields\n",
    );

    let out = strata_in(tmp.path(), &["list", "dragons"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[malformed-artifact]: "), "{err}");
    assert!(err.contains("0001-broken.md"), "must name the file:\n{err}");
    assert_eq!(stdout(&out), "", "stdout must stay clean on failure");
}

#[test]
fn malformed_artifact_fails_show_even_for_other_references() {
    let tmp = init_repo();
    seed_known_pair(tmp.path());
    write_artifact(
        tmp.path(),
        OPEN_DIR,
        "0003-broken.md",
        &dragon_markdown("id-broken", 3, "closed", "Wrong status for open dir"),
    );

    let out = strata_in(tmp.path(), &["show", "dragon:2"]);

    assert_eq!(out.status.code(), Some(5), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("0003-broken.md"),
        "must name the malformed file:\n{}",
        stderr(&out)
    );
}

#[test]
fn dot_prefixed_entries_are_ignored_by_list() {
    let tmp = init_repo();
    write_artifact(tmp.path(), OPEN_DIR, ".gitkeep", "");
    write_artifact(tmp.path(), OPEN_DIR, ".strata.artifact.tmpABC", "junk");

    let out = strata_in(tmp.path(), &["list", "dragons", "--json"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "[]\n");
}

#[test]
fn list_and_show_without_a_repository_are_typed_errors() {
    let tmp = tempfile::tempdir().unwrap();

    for args in [
        &["list", "dragons"] as &[&str],
        &["show", "dragon:1"],
        &["show", "drg-some-id"],
    ] {
        let out = strata_in(tmp.path(), args);
        assert_eq!(out.status.code(), Some(3), "{args:?}:\n{}", stderr(&out));
        assert!(
            stderr(&out).starts_with("error[missing-repository]: "),
            "{args:?}:\n{}",
            stderr(&out)
        );
    }
}
