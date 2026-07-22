//! Read-only discovery, parsing, and projection of managed artifacts.
//!
//! This module rediscovers canonical artifacts from the filesystem and
//! exposes one typed read model — [`Summary`] — shared by the human and
//! `--json` projections, so both interfaces describe the same facts.
//! Nothing here mutates a repository.
//!
//! # Validation boundary
//!
//! Reading validates exactly what is needed to trust a managed file as an
//! artifact: front matter shape, required fields, filename agreement, and
//! lifecycle placement. A malformed managed file is a typed error naming the
//! path — never silently skipped — but this is not `doctor`: scanning stops
//! at the first problem instead of producing a repository-wide report.
//!
//! # Markdown expectations
//!
//! The artifact title is the single ATX level-one heading (`# Title`) after
//! the front matter. Setext headings (`Title` underlined with `=`) are not
//! recognized. Headings inside fenced code blocks are ignored.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::artifact::MAX_SEQUENCE;
use crate::error::Error;
use crate::repo::{
    DRAGONS_CLOSED_DIR, DRAGONS_OPEN_DIR, IDEAS_ADOPTED_DIR, IDEAS_PARKED_DIR, IDEAS_REJECTED_DIR,
};

/// Lifecycle state of a managed artifact, agreeing with its placement:
/// files under `dragons/open` are `open`, files under `ideas/parked` are
/// `parked`, and so on. One vocabulary spans every collection; which states
/// a given collection admits is [`Collection`] data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    Closed,
    Parked,
    Adopted,
    Rejected,
}

impl Status {
    /// The canonical front-matter spelling.
    pub fn name(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Closed => "closed",
            Status::Parked => "parked",
            Status::Adopted => "adopted",
            Status::Rejected => "rejected",
        }
    }
}

/// One managed collection, described as plain data: its kind name, its
/// lifecycle states with their directories, and its legal transitions.
///
/// This is a value, not a framework: the interpreting machinery (scan,
/// parse, transition, validate) is ordinary code taking one of the two
/// statics below. Creation templates and command vocabulary stay hardcoded
/// per collection; see idea 10 for the extraction discipline.
#[derive(Debug)]
pub struct Collection {
    /// Canonical singular kind name, as written in front matter.
    pub kind: &'static str,
    /// Lifecycle states in scan order: `(status, root-relative directory)`.
    /// The first entry is the home state new artifacts are created in.
    pub states: &'static [(Status, &'static str)],
    /// Legal lifecycle transitions as `(from, to)` pairs.
    pub transitions: &'static [(Status, Status)],
}

/// The dragon collection: unresolved technical risks, `open <-> closed`.
pub static DRAGON: Collection = Collection {
    kind: "dragon",
    states: &[
        (Status::Open, DRAGONS_OPEN_DIR),
        (Status::Closed, DRAGONS_CLOSED_DIR),
    ],
    transitions: &[
        (Status::Open, Status::Closed),
        (Status::Closed, Status::Open),
    ],
};

/// The idea collection: uncommitted proposals, `parked -> adopted | rejected`.
/// Terminal states are permanent; there is no reopen analog.
pub static IDEA: Collection = Collection {
    kind: "idea",
    states: &[
        (Status::Parked, IDEAS_PARKED_DIR),
        (Status::Adopted, IDEAS_ADOPTED_DIR),
        (Status::Rejected, IDEAS_REJECTED_DIR),
    ],
    transitions: &[
        (Status::Parked, Status::Adopted),
        (Status::Parked, Status::Rejected),
    ],
};

impl Collection {
    /// The lifecycle directory holding artifacts in `status`.
    ///
    /// # Panics
    ///
    /// Panics if `status` is not a state of this collection; callers only
    /// obtain statuses by parsing this collection's artifacts.
    pub fn dir_of(&self, status: Status) -> &'static str {
        self.states
            .iter()
            .find(|(s, _)| *s == status)
            .map(|(_, dir)| *dir)
            .unwrap_or_else(|| panic!("{} has no `{status}` state", self.kind))
    }

    /// Parse a front-matter status string against this collection's states.
    pub fn parse_status(&self, name: &str) -> Option<Status> {
        self.states
            .iter()
            .map(|(status, _)| *status)
            .find(|status| status.name() == name)
    }

    /// Whether `from -> to` is a legal lifecycle transition.
    pub fn allows(&self, from: Status, to: Status) -> bool {
        self.transitions.contains(&(from, to))
    }

    /// Human list of valid status names, for error messages.
    fn status_names(&self) -> String {
        let names: Vec<&str> = self.states.iter().map(|(s, _)| s.name()).collect();
        names.join("` or `")
    }

    /// Human list of legal transitions, for error messages.
    pub fn transition_names(&self) -> String {
        let arrows: Vec<String> = self
            .transitions
            .iter()
            .map(|(from, to)| format!("{from} -> {to}"))
            .collect();
        arrows.join(", ")
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The typed read model for one artifact, shared by the human and JSON
/// projections. Serialized field names and order are a compatibility
/// surface pinned by tests.
///
/// `path` is always repository-relative with `/` separators; absolute
/// paths never appear in projections.
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    /// Stable opaque identity. Generated IDs are prefixed ULIDs, but any
    /// non-empty string is valid; readers must not assume ULID structure.
    pub id: String,
    /// Collection-scoped display sequence, as in `dragon:7`.
    pub sequence: u32,
    /// Artifact kind: the collection's singular name, e.g. `dragon`.
    pub kind: String,
    /// Lifecycle state, agreeing with repository placement.
    pub status: Status,
    /// Title from the artifact's level-one Markdown heading.
    pub title: String,
    /// Opaque creation stamp from front matter.
    pub created: String,
    /// Repository-relative path with `/` separators.
    pub path: String,
}

impl Summary {
    /// Human reference for this artifact, e.g. `dragon:7`.
    pub fn reference(&self) -> String {
        format!("{}:{}", self.kind, self.sequence)
    }
}

/// A parsed canonical artifact: the shared summary plus the exact file
/// contents.
#[derive(Debug, Clone)]
pub struct Artifact {
    pub summary: Summary,
    /// Canonical Markdown contents, byte-for-byte.
    pub content: String,
}

/// The `show --json` projection: the summary fields plus the canonical
/// contents. Field names and order are pinned by tests.
#[derive(Debug, Serialize)]
pub struct ShowRecord<'a> {
    #[serde(flatten)]
    pub summary: &'a Summary,
    pub content: &'a str,
}

impl Artifact {
    /// The `show --json` projection of this artifact.
    pub fn show_record(&self) -> ShowRecord<'_> {
        ShowRecord {
            summary: &self.summary,
            content: &self.content,
        }
    }
}

/// One way to name a single artifact during resolution.
#[derive(Debug, Clone, Copy)]
pub enum Selector<'a> {
    /// A human reference by display sequence, e.g. `dragon:7`. Sequences are
    /// collection-scoped, so callers pass only that collection's artifacts.
    Sequence(u32),
    /// A stable opaque identity, compared verbatim.
    Id(&'a str),
}

/// Required artifact front matter. Unknown fields are tolerated so future
/// metadata never breaks older readers.
#[derive(Debug, Deserialize)]
struct FrontMatter {
    id: String,
    sequence: u32,
    kind: String,
    status: String,
    created: String,
}

/// Parse every artifact of one collection in the repository at `root`,
/// sorted deterministically by display sequence ascending, then
/// repository-relative path ascending.
///
/// Every non-hidden entry in a managed directory must be a valid artifact
/// of the collection; the first malformed file is a typed error naming its
/// path. Dot-prefixed entries are not artifacts and are ignored.
pub fn scan(root: &Path, collection: &Collection) -> Result<Vec<Artifact>, Error> {
    let mut artifacts = Vec::new();
    for (status, dir_rel) in collection.states {
        let dir = root.join(dir_rel);
        for name in managed_entries(&dir)? {
            artifacts.push(parse_artifact(
                &dir.join(&name),
                dir_rel,
                &name,
                collection,
                *status,
            )?);
        }
    }
    artifacts.sort_by(|a, b| {
        (a.summary.sequence, &a.summary.path).cmp(&(b.summary.sequence, &b.summary.path))
    });
    Ok(artifacts)
}

/// Resolve `target` to exactly one artifact.
///
/// Zero matches is `artifact-not-found`; more than one is
/// `ambiguous-reference` naming every candidate — Strata never silently
/// picks among duplicates. `display` is the reference as the user wrote it,
/// used in error messages.
pub fn resolve<'a>(
    artifacts: &'a [Artifact],
    target: Selector<'_>,
    display: &str,
) -> Result<&'a Artifact, Error> {
    let mut matches = artifacts.iter().filter(|artifact| match target {
        Selector::Sequence(sequence) => artifact.summary.sequence == sequence,
        Selector::Id(id) => artifact.summary.id == id,
    });
    let Some(first) = matches.next() else {
        return Err(Error::ArtifactNotFound {
            reference: display.to_string(),
        });
    };
    let rest: Vec<&Artifact> = matches.collect();
    if rest.is_empty() {
        return Ok(first);
    }
    let candidates = std::iter::once(first)
        .chain(rest)
        .map(|artifact| artifact.summary.path.clone())
        .collect();
    Err(Error::AmbiguousReference {
        reference: display.to_string(),
        candidates,
    })
}

/// Non-hidden entry names of one managed directory, in unspecified order.
///
/// The repository is defined by its marker alone: Git does not round-trip
/// empty directories, so a missing managed directory is an empty
/// collection, not damage. A non-directory object occupying the managed
/// path is a real conflict. Non-UTF-8 names cannot be artifacts and are
/// malformed rather than skipped.
fn managed_entries(dir: &Path) -> Result<Vec<String>, Error> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) if source.kind() == io::ErrorKind::NotADirectory => {
            return Err(Error::ArtifactConflict {
                path: dir.to_path_buf(),
                reason: "a non-directory object occupies a managed directory \
                         path; move it aside"
                    .into(),
            });
        }
        Err(source) => {
            return Err(Error::Filesystem {
                operation: "read directory".into(),
                path: dir.to_path_buf(),
                source,
            });
        }
    };

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::Filesystem {
            operation: "read directory entry".into(),
            path: dir.to_path_buf(),
            source,
        })?;
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            return Err(Error::MalformedArtifact {
                path: dir.join(&name),
                reason: "filename is not valid UTF-8".into(),
            });
        };
        if name_str.starts_with('.') {
            continue;
        }
        names.push(name_str.to_string());
    }
    Ok(names)
}

/// Parse one managed artifact file into the read model, validating filename
/// agreement, required front matter, lifecycle placement, and the title
/// heading. `doctor` reuses this per-file pipeline so validation semantics
/// cannot drift between scanning and diagnosis.
pub(crate) fn parse_artifact(
    path: &Path,
    dir_rel: &str,
    file_name: &str,
    collection: &Collection,
    placement: Status,
) -> Result<Artifact, Error> {
    let kind = collection.kind;
    let malformed = |reason: String| Error::MalformedArtifact {
        path: path.to_path_buf(),
        reason,
    };

    let filename_sequence = crate::artifact::parse_sequence(file_name).ok_or_else(|| {
        malformed(format!(
            "{kind} filenames must be `NNNN-slug.md` with a four-digit \
             display sequence"
        ))
    })?;

    let content = fs::read_to_string(path).map_err(|source| {
        if source.kind() == io::ErrorKind::InvalidData {
            malformed("contents are not valid UTF-8".into())
        } else {
            Error::Filesystem {
                operation: "read".into(),
                path: path.to_path_buf(),
                source,
            }
        }
    })?;

    let (front_matter, body) = split_front_matter(&content).ok_or_else(|| {
        malformed(
            "missing front matter: artifacts must open with a `---` line \
             and close the metadata block with another"
                .into(),
        )
    })?;

    let meta: FrontMatter = serde_yaml_ng::from_str(front_matter)
        .map_err(|err| malformed(format!("invalid front matter: {err}")))?;

    if meta.id.is_empty() {
        return Err(malformed(
            "front-matter `id` must be a non-empty string".into(),
        ));
    }
    if meta.kind != kind {
        return Err(malformed(format!(
            "front-matter `kind` is `{}`, but artifacts in `{dir_rel}` must \
             be `{kind}`",
            meta.kind
        )));
    }
    let status = collection.parse_status(&meta.status).ok_or_else(|| {
        malformed(format!(
            "front-matter `status` is `{}`; {kind}s are `{}`",
            meta.status,
            collection.status_names()
        ))
    })?;
    if status != placement {
        return Err(malformed(format!(
            "lifecycle mismatch: the file sits in `{dir_rel}` but declares \
             `status: {status}`; placement and status must agree"
        )));
    }
    if !(1..=MAX_SEQUENCE).contains(&meta.sequence) {
        return Err(malformed(format!(
            "front-matter `sequence` is {}, outside the valid range 1..={MAX_SEQUENCE}",
            meta.sequence
        )));
    }
    if meta.sequence != filename_sequence {
        return Err(malformed(format!(
            "sequence mismatch: the filename says {filename_sequence} but \
             front matter says {}; they must agree",
            meta.sequence
        )));
    }
    if meta.created.is_empty() {
        return Err(malformed(
            "front-matter `created` must be a non-empty string".into(),
        ));
    }

    let title = extract_title(body).map_err(malformed)?;

    Ok(Artifact {
        summary: Summary {
            id: meta.id,
            sequence: meta.sequence,
            kind: meta.kind,
            status,
            title,
            created: meta.created,
            path: format!("{dir_rel}/{file_name}"),
        },
        content,
    })
}

/// Split `---`-delimited front matter from the Markdown body.
///
/// The file must begin with a `---` line; the metadata block ends at the
/// next line consisting of `---`. Returns `(front_matter, body)`, or `None`
/// when either delimiter is missing.
fn split_front_matter(content: &str) -> Option<(&str, &str)> {
    let rest = content.strip_prefix("---\n")?;
    if let Some(end) = rest.find("\n---\n") {
        Some((&rest[..end + 1], &rest[end + 5..]))
    } else if let Some(front) = rest.strip_suffix("\n---") {
        Some((front, ""))
    } else {
        None
    }
}

/// Extract the single ATX level-one heading from the body.
///
/// Exactly one `# Title` line must exist outside fenced code blocks; a
/// missing, empty, or duplicated title is an error described for the
/// `malformed-artifact` reason.
fn extract_title(body: &str) -> Result<String, String> {
    let mut title: Option<&str> = None;
    let mut in_fence = false;
    for line in body.lines() {
        let line = line.trim_end();
        if line.starts_with("```") || line.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if line == "#"
            || line
                .strip_prefix("# ")
                .is_some_and(|rest| rest.trim().is_empty())
        {
            return Err("the level-one heading has no title text".into());
        }
        if let Some(text) = line.strip_prefix("# ") {
            if title.is_some() {
                return Err("multiple level-one `#` headings: an artifact has exactly \
                     one title"
                    .into());
            }
            title = Some(text.trim());
        }
    }
    title
        .map(str::to_string)
        .ok_or_else(|| "missing the level-one `# Title` heading that names the artifact".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo;

    fn temp_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create temporary directory");
        repo::init(tmp.path()).expect("initialize repository");
        tmp
    }

    fn dragon_markdown(id: &str, sequence: u32, status: &str, title: &str) -> String {
        format!(
            "---\nid: {id}\nsequence: {sequence}\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# {title}\n\n## Context\n"
        )
    }

    fn write_dragon(root: &Path, dir: &str, name: &str, content: &str) {
        fs::write(root.join(dir).join(name), content).unwrap();
    }

    fn expect_malformed(err: Error, name: &str, reason_needle: &str) {
        match err {
            Error::MalformedArtifact { path, reason } => {
                assert!(
                    path.ends_with(name),
                    "expected path ending {name}: {path:?}"
                );
                assert!(
                    reason.contains(reason_needle),
                    "reason should mention `{reason_needle}`: {reason}"
                );
            }
            other => panic!("expected malformed artifact, got {other:?}"),
        }
    }

    #[test]
    fn scan_parses_generated_and_legacy_artifacts() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0002-ulid-style.md",
            &dragon_markdown("drg_01K0P6W5PK8T19H7M2V8W6YQ4C", 2, "open", "ULID style"),
        );
        // Hand-written legacy artifact: the parser must not assume IDs are
        // ULIDs, and must tolerate unknown front-matter fields.
        write_dragon(
            tmp.path(),
            DRAGONS_CLOSED_DIR,
            "0001-legacy.md",
            "---\nid: drg-bootstrap-branch-collisions\nsequence: 1\nkind: dragon\nstatus: closed\ncreated: 2026-07-20\nseverity: high\n---\n\n# Legacy dragon\n",
        );

        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[0].summary.id, "drg-bootstrap-branch-collisions");
        assert_eq!(artifacts[0].summary.status, Status::Closed);
        assert_eq!(artifacts[0].summary.title, "Legacy dragon");
        assert_eq!(
            artifacts[0].summary.path,
            format!("{DRAGONS_CLOSED_DIR}/0001-legacy.md")
        );
        assert_eq!(artifacts[1].summary.id, "drg_01K0P6W5PK8T19H7M2V8W6YQ4C");
        assert_eq!(artifacts[1].summary.reference(), "dragon:2");
    }

    #[test]
    fn scan_sorts_by_sequence_then_path_across_directories() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0003-third.md",
            &dragon_markdown("id-3", 3, "open", "Third"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_CLOSED_DIR,
            "0001-first.md",
            &dragon_markdown("id-1", 1, "closed", "First"),
        );
        // Duplicate sequence across directories: closed/ sorts before open/
        // because the tiebreak is the repository-relative path.
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-duplicate.md",
            &dragon_markdown("id-1-dup", 1, "open", "Duplicate"),
        );

        let paths: Vec<String> = scan(tmp.path(), &DRAGON)
            .unwrap()
            .into_iter()
            .map(|a| a.summary.path)
            .collect();

        assert_eq!(
            paths,
            vec![
                format!("{DRAGONS_CLOSED_DIR}/0001-first.md"),
                format!("{DRAGONS_OPEN_DIR}/0001-duplicate.md"),
                format!("{DRAGONS_OPEN_DIR}/0003-third.md"),
            ]
        );
    }

    #[test]
    fn scan_ideas_spans_all_three_lifecycle_directories() {
        let tmp = temp_repo();
        for (dir, sequence, status, title) in [
            (crate::repo::IDEAS_PARKED_DIR, 1u32, "parked", "Parked"),
            (crate::repo::IDEAS_ADOPTED_DIR, 2, "adopted", "Adopted"),
            (crate::repo::IDEAS_REJECTED_DIR, 3, "rejected", "Rejected"),
        ] {
            fs::create_dir_all(tmp.path().join(dir)).unwrap();
            fs::write(
                tmp.path().join(dir).join(format!("{sequence:04}-i.md")),
                format!(
                    "---\nid: idea-{sequence}\nsequence: {sequence}\nkind: idea\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# {title}\n"
                ),
            )
            .unwrap();
        }

        let ideas = scan(tmp.path(), &IDEA).unwrap();

        assert_eq!(ideas.len(), 3);
        assert_eq!(ideas[0].summary.reference(), "idea:1");
        assert_eq!(ideas[0].summary.status, Status::Parked);
        assert_eq!(ideas[2].summary.status, Status::Rejected);
    }

    #[test]
    fn idea_status_vocabulary_excludes_dragon_statuses() {
        let tmp = temp_repo();
        fs::create_dir_all(tmp.path().join(crate::repo::IDEAS_PARKED_DIR)).unwrap();
        fs::write(
            tmp.path()
                .join(crate::repo::IDEAS_PARKED_DIR)
                .join("0001-open-idea.md"),
            "---\nid: idea-x\nsequence: 1\nkind: idea\nstatus: open\ncreated: 2026-07-20\n---\n\n# T\n",
        )
        .unwrap();

        let err = scan(tmp.path(), &IDEA).unwrap_err();

        expect_malformed(err, "0001-open-idea.md", "parked");
    }

    #[test]
    fn collection_lifecycle_data_answers_transitions_and_directories() {
        assert!(DRAGON.allows(Status::Open, Status::Closed));
        assert!(DRAGON.allows(Status::Closed, Status::Open));
        assert!(IDEA.allows(Status::Parked, Status::Adopted));
        assert!(IDEA.allows(Status::Parked, Status::Rejected));
        for (from, to) in [
            (Status::Adopted, Status::Parked),
            (Status::Rejected, Status::Parked),
            (Status::Adopted, Status::Rejected),
        ] {
            assert!(!IDEA.allows(from, to), "{from} -> {to} must be illegal");
        }
        assert_eq!(IDEA.dir_of(Status::Parked), crate::repo::IDEAS_PARKED_DIR);
        assert_eq!(IDEA.parse_status("adopted"), Some(Status::Adopted));
        assert_eq!(IDEA.parse_status("open"), None);
    }

    #[test]
    fn scan_of_empty_repository_returns_no_artifacts() {
        let tmp = temp_repo();
        assert!(scan(tmp.path(), &DRAGON).unwrap().is_empty());
    }

    #[test]
    fn dot_entries_are_ignored_during_scan() {
        let tmp = temp_repo();
        write_dragon(tmp.path(), DRAGONS_OPEN_DIR, ".gitkeep", "");
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            ".strata.artifact.tmpXYZ",
            "junk",
        );

        assert!(scan(tmp.path(), &DRAGON).unwrap().is_empty());
    }

    #[test]
    fn content_is_preserved_byte_for_byte() {
        let tmp = temp_repo();
        let content = dragon_markdown("id-1", 1, "open", "Exact bytes")
            + "\ntrailing detail with  double spaces\n";
        write_dragon(tmp.path(), DRAGONS_OPEN_DIR, "0001-exact.md", &content);

        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        assert_eq!(artifacts[0].content, content);
    }

    #[test]
    fn malformed_filename_is_a_typed_error_naming_the_path() {
        let tmp = temp_repo();
        write_dragon(tmp.path(), DRAGONS_OPEN_DIR, "notes.txt", "not an artifact");

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "notes.txt", "NNNN-slug.md");
    }

    #[test]
    fn missing_front_matter_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-bare.md",
            "# Just a title\n",
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-bare.md", "front matter");
    }

    #[test]
    fn unterminated_front_matter_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-open-ended.md",
            "---\nid: x\nsequence: 1\n\n# Title\n",
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-open-ended.md", "front matter");
    }

    #[test]
    fn unparseable_front_matter_mapping_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-broken.md",
            "---\nid: [unclosed\n---\n\n# Title\n",
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-broken.md", "invalid front matter");
    }

    #[test]
    fn missing_required_field_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-incomplete.md",
            "---\nid: x\nsequence: 1\nkind: dragon\nstatus: open\n---\n\n# Title\n",
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-incomplete.md", "created");
    }

    #[test]
    fn wrong_field_type_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-typed.md",
            "---\nid: x\nsequence: seven\nkind: dragon\nstatus: open\ncreated: 2026-07-20\n---\n\n# Title\n",
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-typed.md", "invalid front matter");
    }

    #[test]
    fn wrong_kind_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-decision.md",
            "---\nid: x\nsequence: 1\nkind: decision\nstatus: open\ncreated: 2026-07-20\n---\n\n# Title\n",
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-decision.md", "kind");
    }

    #[test]
    fn invalid_status_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-status.md",
            &dragon_markdown("x", 1, "resolved", "Title"),
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-status.md", "status");
    }

    #[test]
    fn status_and_placement_must_agree() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-misplaced.md",
            &dragon_markdown("x", 1, "closed", "Misplaced"),
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-misplaced.md", "lifecycle mismatch");
    }

    #[test]
    fn filename_and_front_matter_sequence_must_agree() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0002-shifted.md",
            &dragon_markdown("x", 3, "open", "Shifted"),
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0002-shifted.md", "sequence mismatch");
    }

    #[test]
    fn out_of_range_sequence_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0000-zero.md",
            &dragon_markdown("x", 0, "open", "Zero"),
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0000-zero.md", "range");
    }

    #[test]
    fn empty_id_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-anon.md",
            "---\nid: \"\"\nsequence: 1\nkind: dragon\nstatus: open\ncreated: 2026-07-20\n---\n\n# Title\n",
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-anon.md", "id");
    }

    #[test]
    fn missing_title_heading_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-untitled.md",
            "---\nid: x\nsequence: 1\nkind: dragon\nstatus: open\ncreated: 2026-07-20\n---\n\n## Only a subsection\n",
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-untitled.md", "level-one");
    }

    #[test]
    fn duplicate_title_headings_are_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-twice.md",
            &(dragon_markdown("x", 1, "open", "First title") + "\n# Second title\n"),
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-twice.md", "one title");
    }

    #[test]
    fn heading_lines_inside_code_fences_are_not_titles() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-fenced.md",
            &(dragon_markdown("x", 1, "open", "Fenced")
                + "\n```sh\n# a shell comment, not a heading\n```\n"),
        );

        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        assert_eq!(artifacts[0].summary.title, "Fenced");
    }

    #[test]
    fn resolve_by_sequence_and_by_id() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-one.md",
            &dragon_markdown("drg-legacy-one", 1, "open", "One"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0002-two.md",
            &dragon_markdown("drg_01K0P6W5PK8T19H7M2V8W6YQ4C", 2, "open", "Two"),
        );
        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        let by_sequence = resolve(&artifacts, Selector::Sequence(2), "dragon:2").unwrap();
        assert_eq!(by_sequence.summary.title, "Two");

        let by_id = resolve(&artifacts, Selector::Id("drg-legacy-one"), "drg-legacy-one").unwrap();
        assert_eq!(by_id.summary.title, "One");
    }

    #[test]
    fn resolve_reports_not_found_with_the_reference() {
        let err = resolve(&[], Selector::Sequence(4), "dragon:4").unwrap_err();
        match err {
            Error::ArtifactNotFound { reference } => assert_eq!(reference, "dragon:4"),
            other => panic!("expected artifact-not-found, got {other:?}"),
        }
    }

    #[test]
    fn resolve_refuses_duplicate_sequences() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-a.md",
            &dragon_markdown("id-a", 1, "open", "A"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_CLOSED_DIR,
            "0001-b.md",
            &dragon_markdown("id-b", 1, "closed", "B"),
        );
        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        let err = resolve(&artifacts, Selector::Sequence(1), "dragon:1").unwrap_err();

        match err {
            Error::AmbiguousReference {
                reference,
                candidates,
            } => {
                assert_eq!(reference, "dragon:1");
                assert_eq!(candidates.len(), 2);
                assert!(candidates.iter().any(|p| p.ends_with("0001-a.md")));
                assert!(candidates.iter().any(|p| p.ends_with("0001-b.md")));
            }
            other => panic!("expected ambiguous-reference, got {other:?}"),
        }
    }

    #[test]
    fn resolve_refuses_duplicate_ids() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-a.md",
            &dragon_markdown("id-same", 1, "open", "A"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0002-b.md",
            &dragon_markdown("id-same", 2, "open", "B"),
        );
        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        let err = resolve(&artifacts, Selector::Id("id-same"), "id-same").unwrap_err();

        assert!(matches!(err, Error::AmbiguousReference { .. }), "{err:?}");
    }

    #[test]
    fn missing_managed_directory_scans_as_an_empty_collection() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-alone.md",
            &dragon_markdown("id-1", 1, "open", "Alone"),
        );
        fs::remove_dir(tmp.path().join(DRAGONS_CLOSED_DIR)).unwrap();

        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].summary.title, "Alone");
    }

    #[test]
    fn marker_only_repository_scans_as_empty() {
        // Simulate `git clone` of a freshly initialized repository: Git
        // preserves the marker but drops every empty directory.
        let tmp = temp_repo();
        fs::remove_dir_all(tmp.path().join("archaeology")).unwrap();

        assert!(scan(tmp.path(), &DRAGON).unwrap().is_empty());
    }

    #[test]
    fn non_directory_at_managed_path_is_a_conflict() {
        let tmp = temp_repo();
        fs::remove_dir(tmp.path().join(DRAGONS_CLOSED_DIR)).unwrap();
        fs::write(tmp.path().join(DRAGONS_CLOSED_DIR), "not a directory").unwrap();

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        match err {
            Error::ArtifactConflict { path, .. } => {
                assert!(path.ends_with(DRAGONS_CLOSED_DIR), "{path:?}");
            }
            other => panic!("expected artifact conflict, got {other:?}"),
        }
    }

    #[test]
    fn summary_json_field_names_and_order_are_stable() {
        let summary = Summary {
            id: "drg-x".into(),
            sequence: 7,
            kind: "dragon".into(),
            status: Status::Open,
            title: "A title".into(),
            created: "2026-07-20".into(),
            path: "archaeology/dragons/open/0007-a-title.md".into(),
        };

        assert_eq!(
            serde_json::to_string(&summary).unwrap(),
            "{\"id\":\"drg-x\",\"sequence\":7,\"kind\":\"dragon\",\"status\":\"open\",\
             \"title\":\"A title\",\"created\":\"2026-07-20\",\
             \"path\":\"archaeology/dragons/open/0007-a-title.md\"}"
        );
    }

    #[test]
    fn split_front_matter_requires_both_delimiters() {
        assert_eq!(
            split_front_matter("---\na: 1\n---\nbody\n"),
            Some(("a: 1\n", "body\n"))
        );
        assert_eq!(split_front_matter("---\na: 1\n---"), Some(("a: 1", "")));
        assert_eq!(split_front_matter("a: 1\n---\nbody\n"), None);
        assert_eq!(split_front_matter("---\na: 1\n"), None);
        assert_eq!(split_front_matter(""), None);
    }
}
