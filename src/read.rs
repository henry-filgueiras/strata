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
//! artifact: front matter shape, required fields, and filename agreement. A
//! malformed managed file is a typed error naming the
//! path — never silently skipped — but this is not `doctor`: scanning stops
//! at the first problem instead of producing a repository-wide report.
//!
//! # Markdown expectations
//!
//! The artifact title is the single ATX level-one heading (`# Title`) after
//! the front matter. Setext headings (`Title` underlined with `=`) are not
//! recognized. Headings inside fenced code blocks are ignored.
//!
//! # Filesystem boundary
//!
//! A repository working tree is untrusted input (thread 4, task 22): Git
//! round-trips symlinks and arbitrarily large files. Every canonical
//! position is therefore classified without following symlinks — a symlink
//! or other non-regular entry where an artifact belongs is refused, never
//! read — and every content read goes through the bounded
//! [`read_artifact_bytes`] seam. This is per-file rejection, not
//! containment: no canonicalization, and no claim of race-free confinement
//! against concurrent replacement.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::artifact::MAX_SEQUENCE;
use crate::error::Error;
use crate::repo::{DRAGONS_DIR, IDEAS_DIR, SPRINTS_DIR};

/// Per-file byte cap on every managed content read (thread 4, task 22).
///
/// 1 MiB is roughly forty times the largest artifact in this repository
/// (~27 KB): far above any plausible narrative artifact, small enough that
/// one hostile or accidental oversized file cannot exhaust process memory
/// through a single read. This is a per-file safety bound only — a strict
/// scan may still retain up to N × cap across N valid artifacts; the
/// aggregate-retention seam is deliberately deferred (thread 8, idea 18).
pub const MAX_ARTIFACT_BYTES: u64 = 1024 * 1024;

/// Read at most [`MAX_ARTIFACT_BYTES`] from `path`, returning `None` when
/// the file holds more than the cap.
///
/// The bounded `take` is the enforcement, not any metadata inspection: at
/// most `MAX_ARTIFACT_BYTES + 1` bytes are ever pulled, so the bound holds
/// even when a file grows after being classified, and an oversized payload
/// is never fully allocated.
pub(crate) fn read_capped(path: &Path) -> io::Result<Option<Vec<u8>>> {
    use std::io::Read;
    let file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(MAX_ARTIFACT_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_ARTIFACT_BYTES {
        return Ok(None);
    }
    Ok(Some(bytes))
}

/// Read one managed file's content through the task 22 boundary: the path
/// must be a regular file — classified with `symlink_metadata`, so a
/// symlink is refused rather than followed — and the content must fit
/// [`MAX_ARTIFACT_BYTES`] and be valid UTF-8.
///
/// Callers classify entries during their walks; this re-check is the
/// backstop that keeps every byte-reading site behind one seam.
pub(crate) fn read_artifact_bytes(path: &Path) -> Result<String, Error> {
    let meta = fs::symlink_metadata(path).map_err(|source| Error::Filesystem {
        operation: "inspect".into(),
        path: path.to_path_buf(),
        source,
    })?;
    if !meta.is_file() {
        return Err(Error::ArtifactConflict {
            path: path.to_path_buf(),
            reason: format!(
                "a {} occupies a managed artifact position; artifacts must \
                 be regular files, and Strata never follows symbolic links \
                 inside a repository",
                crate::repo::file_kind(&meta)
            ),
        });
    }
    let bytes = read_capped(path)
        .map_err(|source| Error::Filesystem {
            operation: "read".into(),
            path: path.to_path_buf(),
            source,
        })?
        .ok_or_else(|| Error::MalformedArtifact {
            path: path.to_path_buf(),
            reason: format!(
                "file exceeds the {MAX_ARTIFACT_BYTES}-byte per-file read \
                 limit; Strata refuses to load oversized artifacts into \
                 memory"
            ),
        })?;
    String::from_utf8(bytes).map_err(|_| Error::MalformedArtifact {
        path: path.to_path_buf(),
        reason: "contents are not valid UTF-8".into(),
    })
}

/// Lifecycle state of a managed artifact, carried only in front matter per
/// decision 11: placement is flat, so the directory says nothing about
/// state. One vocabulary spans every collection; which states a given
/// collection admits is [`Collection`] data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    Closed,
    Parked,
    Adopted,
    Rejected,
    Active,
    Pending,
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
            Status::Active => "active",
            Status::Pending => "pending",
        }
    }
}

/// One managed collection, described as plain data: its kind name, its
/// directory, its lifecycle states, and its legal transitions.
///
/// This is a value, not a framework: the interpreting machinery (scan,
/// parse, transition, validate) is ordinary code taking one of the two
/// statics below. Creation templates and command vocabulary stay hardcoded
/// per collection; see idea 10 for the extraction discipline.
#[derive(Debug)]
pub struct Collection {
    /// Canonical singular kind name, as written in front matter.
    pub kind: &'static str,
    /// The one root-relative directory holding every artifact of the
    /// collection, regardless of lifecycle state (decision 11). Sprints
    /// and tasks share the sprints directory; their layouts diverge from
    /// flat files in ways this descriptor cannot express (see the
    /// dedicated scanners), which is recorded evidence for idea 10.
    pub dir: &'static str,
    /// Admitted lifecycle states; the first is the home state new
    /// artifacts are created in.
    pub states: &'static [Status],
    /// Legal lifecycle transitions as `(from, to)` pairs.
    pub transitions: &'static [(Status, Status)],
    /// Whether reaching `closed` stamps a `closed:` date line into the
    /// front matter as part of the transition write.
    pub stamp_closed: bool,
}

/// The dragon collection: unresolved technical risks, `open <-> closed`.
pub static DRAGON: Collection = Collection {
    kind: "dragon",
    dir: DRAGONS_DIR,
    states: &[Status::Open, Status::Closed],
    transitions: &[
        (Status::Open, Status::Closed),
        (Status::Closed, Status::Open),
    ],
    stamp_closed: false,
};

/// The idea collection: uncommitted proposals, `parked -> adopted | rejected`.
/// Terminal states are permanent; there is no reopen analog.
pub static IDEA: Collection = Collection {
    kind: "idea",
    dir: IDEAS_DIR,
    states: &[Status::Parked, Status::Adopted, Status::Rejected],
    transitions: &[
        (Status::Parked, Status::Adopted),
        (Status::Parked, Status::Rejected),
    ],
    stamp_closed: false,
};

/// The sprint collection: units of scoped work, `active -> closed`.
/// A sprint artifact is `sprint.md` inside its own containment directory
/// `NNNN-slug/`; the directory name carries the display sequence.
pub static SPRINT: Collection = Collection {
    kind: "sprint",
    dir: SPRINTS_DIR,
    states: &[Status::Active, Status::Closed],
    transitions: &[(Status::Active, Status::Closed)],
    stamp_closed: true,
};

/// The task collection: work items, `pending -> closed`. Task files live
/// inside their owning sprint's containment directory; sequences are
/// global across sprints.
pub static TASK: Collection = Collection {
    kind: "task",
    dir: SPRINTS_DIR,
    states: &[Status::Pending, Status::Closed],
    transitions: &[(Status::Pending, Status::Closed)],
    stamp_closed: true,
};

impl Collection {
    /// Parse a front-matter status string against this collection's states.
    pub fn parse_status(&self, name: &str) -> Option<Status> {
        self.states
            .iter()
            .copied()
            .find(|status| status.name() == name)
    }

    /// Whether `from -> to` is a legal lifecycle transition.
    pub fn allows(&self, from: Status, to: Status) -> bool {
        self.transitions.contains(&(from, to))
    }

    /// Human list of valid status names, for error messages.
    fn status_names(&self) -> String {
        let names: Vec<&str> = self.states.iter().map(|s| s.name()).collect();
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
    /// Lifecycle state, from front matter (the sole authority per
    /// decision 11).
    pub status: Status,
    /// Title from the artifact's level-one Markdown heading.
    pub title: String,
    /// Opaque creation stamp from front matter.
    pub created: String,
    /// For tasks only: the stable id of the owning sprint, from the
    /// `sprint:` front-matter field. Absent for every other kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprint: Option<String>,
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
    /// Required on tasks (the owning sprint's stable id); inert elsewhere.
    sprint: Option<String>,
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
    let dir = root.join(collection.dir);
    for (name, file_type) in managed_entries(&dir)? {
        let path = dir.join(&name);
        if file_type.is_symlink() || !(file_type.is_file() || file_type.is_dir()) {
            return Err(non_regular_entry(&path, &file_type));
        }
        if file_type.is_dir() {
            return Err(Error::ArtifactConflict {
                path,
                reason: "a directory sits inside a managed collection \
                         directory; placement is flat (decision 11), so \
                         artifacts file directly in the collection directory"
                    .into(),
            });
        }
        artifacts.push(parse_artifact(&path, collection.dir, &name, collection)?);
    }
    artifacts.sort_by(|a, b| {
        (a.summary.sequence, &a.summary.path).cmp(&(b.summary.sequence, &b.summary.path))
    });
    Ok(artifacts)
}

/// Scan `collection` through the scanner its layout requires: flat files
/// for dragons and ideas, containment directories for sprints and tasks.
pub fn scan_collection(root: &Path, collection: &Collection) -> Result<Vec<Artifact>, Error> {
    match collection.kind {
        "sprint" => scan_sprints(root),
        "task" => scan_tasks(root),
        _ => scan(root, collection),
    }
}

/// Parse every sprint in the repository at `root`, sorted by display
/// sequence ascending, then path.
///
/// A sprint artifact is `sprint.md` inside a containment directory named
/// `NNNN-slug` under the sprints directory; the directory name carries the
/// display sequence. A non-directory entry in the sprints directory, a
/// malformed containment name, or a missing `sprint.md` is a typed error.
pub fn scan_sprints(root: &Path) -> Result<Vec<Artifact>, Error> {
    let mut artifacts = Vec::new();
    for name in sprint_dir_names(root)? {
        let (sequence, dir_rel) = (parse_dir_sequence(root, &name)?, name);
        let path = root
            .join(SPRINTS_DIR)
            .join(&dir_rel)
            .join(crate::repo::SPRINT_FILE);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_file() => {}
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(non_regular_entry(&path, &meta.file_type()));
            }
            Ok(_) | Err(_) => {
                return Err(Error::MalformedArtifact {
                    path: root.join(SPRINTS_DIR).join(&dir_rel),
                    reason: format!(
                        "sprint containment directories must hold a `{}` artifact",
                        crate::repo::SPRINT_FILE
                    ),
                });
            }
        }
        artifacts.push(parse_artifact_at(
            &path,
            &format!("{SPRINTS_DIR}/{dir_rel}/{}", crate::repo::SPRINT_FILE),
            sequence,
            "the containment directory name",
            &SPRINT,
        )?);
    }
    artifacts.sort_by(|a, b| {
        (a.summary.sequence, &a.summary.path).cmp(&(b.summary.sequence, &b.summary.path))
    });
    Ok(artifacts)
}

/// Parse every task in the repository at `root`, across all sprint
/// containment directories, sorted by display sequence ascending, then
/// path. Task sequences are global across sprints.
pub fn scan_tasks(root: &Path) -> Result<Vec<Artifact>, Error> {
    let mut artifacts = Vec::new();
    for dir_name in sprint_dir_names(root)? {
        let dir_rel = format!("{SPRINTS_DIR}/{dir_name}");
        let dir = root.join(&dir_rel);
        for (name, file_type) in managed_entries(&dir)? {
            if name == crate::repo::SPRINT_FILE {
                continue;
            }
            let path = dir.join(&name);
            if file_type.is_symlink() || !(file_type.is_file() || file_type.is_dir()) {
                return Err(non_regular_entry(&path, &file_type));
            }
            if file_type.is_dir() {
                return Err(Error::ArtifactConflict {
                    path,
                    reason: "a directory sits inside a sprint containment \
                             directory; tasks file directly in their sprint's \
                             directory (decision 11)"
                        .into(),
                });
            }
            artifacts.push(parse_artifact(&path, &dir_rel, &name, &TASK)?);
        }
    }
    artifacts.sort_by(|a, b| {
        (a.summary.sequence, &a.summary.path).cmp(&(b.summary.sequence, &b.summary.path))
    });
    Ok(artifacts)
}

/// Non-hidden sprint containment directory names, unordered. Every entry
/// in the sprints directory must be a directory; stray files are a typed
/// error rather than skipped.
fn sprint_dir_names(root: &Path) -> Result<Vec<String>, Error> {
    let dir = root.join(SPRINTS_DIR);
    let mut names = Vec::new();
    for (name, file_type) in managed_entries(&dir)? {
        if file_type.is_symlink() {
            return Err(Error::ArtifactConflict {
                path: dir.join(&name),
                reason: "a symbolic link occupies a sprint containment \
                         position; containment directories must be real \
                         directories, and Strata never follows symbolic \
                         links inside a repository"
                    .into(),
            });
        }
        if !file_type.is_dir() {
            return Err(Error::MalformedArtifact {
                path: dir.join(&name),
                reason: "the sprints directory holds one containment \
                         directory per sprint; a loose file cannot be a \
                         sprint artifact"
                    .into(),
            });
        }
        names.push(name);
    }
    Ok(names)
}

/// Parse the display sequence from a sprint containment directory name
/// (`NNNN-slug`).
fn parse_dir_sequence(root: &Path, name: &str) -> Result<u32, Error> {
    crate::artifact::parse_dir_sequence(name).ok_or_else(|| Error::MalformedArtifact {
        path: root.join(SPRINTS_DIR).join(name),
        reason: "sprint containment directories must be named `NNNN-slug` \
                 with a four-digit display sequence"
            .into(),
    })
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

/// Refusal for a symlink or other non-regular entry at a canonical
/// artifact position: such entries are never read and never followed
/// (thread 4, task 22).
fn non_regular_entry(path: &Path, file_type: &fs::FileType) -> Error {
    let what = if file_type.is_symlink() {
        "symbolic link"
    } else {
        "non-regular file"
    };
    Error::ArtifactConflict {
        path: path.to_path_buf(),
        reason: format!(
            "a {what} occupies a managed artifact position; artifacts must \
             be regular files, and Strata never follows symbolic links \
             inside a repository"
        ),
    }
}

/// Non-hidden entries of one managed directory as `(name, file type)`, in
/// unspecified order. File types come from the directory entries and never
/// follow symlinks.
///
/// The repository is defined by its marker alone: Git does not round-trip
/// empty directories, so a missing managed directory is an empty
/// collection, not damage. The directory itself is classified with
/// `symlink_metadata`: a symlink or other non-directory object occupying
/// the managed path is a real conflict, never traversed. Non-UTF-8 names
/// cannot be artifacts and are malformed rather than skipped.
fn managed_entries(dir: &Path) -> Result<Vec<(String, fs::FileType)>, Error> {
    match fs::symlink_metadata(dir) {
        Ok(meta) if meta.is_dir() => {}
        Ok(meta) => {
            return Err(Error::ArtifactConflict {
                path: dir.to_path_buf(),
                reason: format!(
                    "a {} occupies a managed directory path; move it aside",
                    crate::repo::file_kind(&meta)
                ),
            });
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(Error::Filesystem {
                operation: "inspect".into(),
                path: dir.to_path_buf(),
                source,
            });
        }
    }
    let entries = fs::read_dir(dir).map_err(|source| Error::Filesystem {
        operation: "read directory".into(),
        path: dir.to_path_buf(),
        source,
    })?;

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
        let file_type = entry.file_type().map_err(|source| Error::Filesystem {
            operation: "inspect directory entry".into(),
            path: dir.join(&name),
            source,
        })?;
        names.push((name_str.to_string(), file_type));
    }
    Ok(names)
}

/// Parse one managed artifact file into the read model, validating filename
/// agreement, required front matter, and the title heading. `doctor`
/// reuses this per-file pipeline so validation semantics cannot drift
/// between scanning and diagnosis.
pub(crate) fn parse_artifact(
    path: &Path,
    dir_rel: &str,
    file_name: &str,
    collection: &Collection,
) -> Result<Artifact, Error> {
    let kind = collection.kind;
    let filename_sequence =
        crate::artifact::parse_sequence(file_name).ok_or_else(|| Error::MalformedArtifact {
            path: path.to_path_buf(),
            reason: format!(
                "{kind} filenames must be `NNNN-slug.md` with a four-digit \
                 display sequence"
            ),
        })?;
    parse_artifact_at(
        path,
        &format!("{dir_rel}/{file_name}"),
        filename_sequence,
        "the filename",
        collection,
    )
}

/// Parse one managed artifact file whose display sequence is carried by
/// `sequence_carrier` (a filename for flat files, a containment directory
/// name for sprints), validating it against the front matter.
pub(crate) fn parse_artifact_at(
    path: &Path,
    path_rel: &str,
    expected_sequence: u32,
    sequence_carrier: &str,
    collection: &Collection,
) -> Result<Artifact, Error> {
    let kind = collection.kind;
    let malformed = |reason: String| Error::MalformedArtifact {
        path: path.to_path_buf(),
        reason,
    };

    let content = read_artifact_bytes(path)?;

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
            "front-matter `kind` is `{}`, but this file must be a `{kind}`",
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
    if !(1..=MAX_SEQUENCE).contains(&meta.sequence) {
        return Err(malformed(format!(
            "front-matter `sequence` is {}, outside the valid range 1..={MAX_SEQUENCE}",
            meta.sequence
        )));
    }
    if meta.sequence != expected_sequence {
        return Err(malformed(format!(
            "sequence mismatch: {sequence_carrier} says {expected_sequence} \
             but front matter says {}; they must agree",
            meta.sequence
        )));
    }
    if meta.created.is_empty() {
        return Err(malformed(
            "front-matter `created` must be a non-empty string".into(),
        ));
    }
    let sprint = if kind == "task" {
        match &meta.sprint {
            Some(id) if !id.is_empty() => Some(id.clone()),
            _ => {
                return Err(malformed(
                    "tasks must carry a `sprint:` front-matter field naming \
                     the owning sprint's stable id"
                        .into(),
                ));
            }
        }
    } else {
        None
    };

    let title = extract_title(body).map_err(malformed)?;

    Ok(Artifact {
        summary: Summary {
            id: meta.id,
            sequence: meta.sequence,
            kind: meta.kind,
            status,
            title,
            created: meta.created,
            sprint,
            path: path_rel.to_string(),
        },
        content,
    })
}

/// Split `---`-delimited front matter from the Markdown body.
///
/// The file must begin with a `---` line; the metadata block ends at the
/// next line consisting of `---`. Returns `(front_matter, body)`, or `None`
/// when either delimiter is missing.
pub(crate) fn split_front_matter(content: &str) -> Option<(&str, &str)> {
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
pub(crate) fn extract_title(body: &str) -> Result<String, String> {
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
            DRAGONS_DIR,
            "0002-ulid-style.md",
            &dragon_markdown("drg_01K0P6W5PK8T19H7M2V8W6YQ4C", 2, "open", "ULID style"),
        );
        // Hand-written legacy artifact: the parser must not assume IDs are
        // ULIDs, and must tolerate unknown front-matter fields.
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
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
            format!("{DRAGONS_DIR}/0001-legacy.md")
        );
        assert_eq!(artifacts[1].summary.id, "drg_01K0P6W5PK8T19H7M2V8W6YQ4C");
        assert_eq!(artifacts[1].summary.reference(), "dragon:2");
    }

    #[test]
    fn scan_sorts_by_sequence_then_path() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0003-third.md",
            &dragon_markdown("id-3", 3, "open", "Third"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-first.md",
            &dragon_markdown("id-1", 1, "closed", "First"),
        );
        // Duplicate sequence (a branch collision): the tiebreak is the
        // repository-relative path.
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
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
                format!("{DRAGONS_DIR}/0001-duplicate.md"),
                format!("{DRAGONS_DIR}/0001-first.md"),
                format!("{DRAGONS_DIR}/0003-third.md"),
            ]
        );
    }

    #[test]
    fn scan_ideas_spans_all_three_lifecycle_states() {
        let tmp = temp_repo();
        for (sequence, status, title) in [
            (1u32, "parked", "Parked"),
            (2, "adopted", "Adopted"),
            (3, "rejected", "Rejected"),
        ] {
            fs::create_dir_all(tmp.path().join(IDEAS_DIR)).unwrap();
            fs::write(
                tmp.path().join(IDEAS_DIR).join(format!("{sequence:04}-i.md")),
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
        fs::create_dir_all(tmp.path().join(IDEAS_DIR)).unwrap();
        fs::write(
            tmp.path().join(IDEAS_DIR).join("0001-open-idea.md"),
            "---\nid: idea-x\nsequence: 1\nkind: idea\nstatus: open\ncreated: 2026-07-20\n---\n\n# T\n",
        )
        .unwrap();

        let err = scan(tmp.path(), &IDEA).unwrap_err();

        expect_malformed(err, "0001-open-idea.md", "parked");
    }

    #[test]
    fn collection_lifecycle_data_answers_transitions_and_statuses() {
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
        assert_eq!(IDEA.parse_status("adopted"), Some(Status::Adopted));
        assert_eq!(IDEA.parse_status("open"), None);
    }

    fn seed_sprint(root: &Path, dir_name: &str, sequence: u32, status: &str) {
        let dir = root.join(SPRINTS_DIR).join(dir_name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(crate::repo::SPRINT_FILE),
            format!(
                "---\nid: spr-{sequence}\nsequence: {sequence}\nkind: sprint\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# Sprint {sequence}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn scan_sprints_parses_containment_directories() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0001-first", 1, "closed");
        seed_sprint(tmp.path(), "0002-second", 2, "active");
        // A task file inside a sprint directory is not a sprint.
        fs::write(
            tmp.path().join(SPRINTS_DIR).join("0001-first").join("0001-task.md"),
            "---\nid: tsk-1\nsequence: 1\nkind: task\nstatus: closed\nsprint: spr-1\ncreated: 2026-07-20\n---\n\n# T\n",
        )
        .unwrap();

        let sprints = scan_sprints(tmp.path()).unwrap();

        assert_eq!(sprints.len(), 2);
        assert_eq!(sprints[0].summary.reference(), "sprint:1");
        assert_eq!(sprints[1].summary.status, Status::Active);
        assert_eq!(
            sprints[1].summary.path,
            format!("{SPRINTS_DIR}/0002-second/sprint.md")
        );
    }

    #[test]
    fn sprint_directory_and_front_matter_sequence_must_agree() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0002-shifted", 1, "closed");

        let err = scan_sprints(tmp.path()).unwrap_err();

        expect_malformed(err, "sprint.md", "sequence mismatch");
    }

    #[test]
    fn loose_file_in_the_sprints_directory_is_malformed() {
        let tmp = temp_repo();
        fs::create_dir_all(tmp.path().join(SPRINTS_DIR)).unwrap();
        fs::write(tmp.path().join(SPRINTS_DIR).join("notes.md"), "junk").unwrap();

        let err = scan_sprints(tmp.path()).unwrap_err();

        expect_malformed(err, "notes.md", "containment");
    }

    #[test]
    fn sprint_directory_without_sprint_file_is_malformed() {
        let tmp = temp_repo();
        fs::create_dir_all(tmp.path().join(SPRINTS_DIR).join("0001-empty")).unwrap();

        let err = scan_sprints(tmp.path()).unwrap_err();

        expect_malformed(err, "0001-empty", "sprint.md");
    }

    #[test]
    fn tasks_scan_across_sprints_and_require_the_sprint_field() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0001-first", 1, "closed");
        seed_sprint(tmp.path(), "0002-second", 2, "active");
        for (dir, sequence, status, sprint) in [
            ("0001-first", 1u32, "closed", "spr-1"),
            ("0002-second", 2, "pending", "spr-2"),
        ] {
            fs::write(
                tmp.path()
                    .join(SPRINTS_DIR)
                    .join(dir)
                    .join(format!("{sequence:04}-work.md")),
                format!(
                    "---\nid: tsk-{sequence}\nsequence: {sequence}\nkind: task\nstatus: {status}\nsprint: {sprint}\ncreated: 2026-07-20\n---\n\n# Work {sequence}\n"
                ),
            )
            .unwrap();
        }

        let tasks = scan_tasks(tmp.path()).unwrap();

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].summary.sprint.as_deref(), Some("spr-1"));
        assert_eq!(tasks[1].summary.status, Status::Pending);

        // A task without a sprint field is malformed.
        fs::write(
            tmp.path()
                .join(SPRINTS_DIR)
                .join("0002-second")
                .join("0003-orphan.md"),
            "---\nid: tsk-3\nsequence: 3\nkind: task\nstatus: pending\ncreated: 2026-07-20\n---\n\n# Orphan\n",
        )
        .unwrap();
        let err = scan_tasks(tmp.path()).unwrap_err();
        expect_malformed(err, "0003-orphan.md", "sprint");
    }

    #[test]
    fn scan_of_empty_repository_returns_no_artifacts() {
        let tmp = temp_repo();
        assert!(scan(tmp.path(), &DRAGON).unwrap().is_empty());
    }

    #[test]
    fn dot_entries_are_ignored_during_scan() {
        let tmp = temp_repo();
        write_dragon(tmp.path(), DRAGONS_DIR, ".gitkeep", "");
        write_dragon(tmp.path(), DRAGONS_DIR, ".strata.artifact.tmpXYZ", "junk");

        assert!(scan(tmp.path(), &DRAGON).unwrap().is_empty());
    }

    #[test]
    fn content_is_preserved_byte_for_byte() {
        let tmp = temp_repo();
        let content = dragon_markdown("id-1", 1, "open", "Exact bytes")
            + "\ntrailing detail with  double spaces\n";
        write_dragon(tmp.path(), DRAGONS_DIR, "0001-exact.md", &content);

        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        assert_eq!(artifacts[0].content, content);
    }

    #[test]
    fn malformed_filename_is_a_typed_error_naming_the_path() {
        let tmp = temp_repo();
        write_dragon(tmp.path(), DRAGONS_DIR, "notes.txt", "not an artifact");

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "notes.txt", "NNNN-slug.md");
    }

    #[test]
    fn missing_front_matter_is_malformed() {
        let tmp = temp_repo();
        write_dragon(tmp.path(), DRAGONS_DIR, "0001-bare.md", "# Just a title\n");

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-bare.md", "front matter");
    }

    #[test]
    fn unterminated_front_matter_is_malformed() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
            "0001-status.md",
            &dragon_markdown("x", 1, "resolved", "Title"),
        );

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-status.md", "status");
    }

    #[test]
    fn any_admitted_status_is_legal_anywhere_in_the_collection_directory() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-settled.md",
            &dragon_markdown("x", 1, "closed", "Settled"),
        );

        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        assert_eq!(artifacts[0].summary.status, Status::Closed);
    }

    #[test]
    fn directory_inside_a_collection_directory_is_a_conflict() {
        // A leftover lifecycle subdirectory from before decision 11.
        let tmp = temp_repo();
        fs::create_dir(tmp.path().join(DRAGONS_DIR).join("open")).unwrap();

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        match err {
            Error::ArtifactConflict { path, reason } => {
                assert!(path.ends_with("open"), "{path:?}");
                assert!(reason.contains("flat"), "{reason}");
            }
            other => panic!("expected conflict, got {other:?}"),
        }
    }

    #[test]
    fn filename_and_front_matter_sequence_must_agree() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
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
            DRAGONS_DIR,
            "0001-one.md",
            &dragon_markdown("drg-legacy-one", 1, "open", "One"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
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
            DRAGONS_DIR,
            "0001-a.md",
            &dragon_markdown("id-a", 1, "open", "A"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
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
            DRAGONS_DIR,
            "0001-a.md",
            &dragon_markdown("id-same", 1, "open", "A"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0002-b.md",
            &dragon_markdown("id-same", 2, "open", "B"),
        );
        let artifacts = scan(tmp.path(), &DRAGON).unwrap();

        let err = resolve(&artifacts, Selector::Id("id-same"), "id-same").unwrap_err();

        assert!(matches!(err, Error::AmbiguousReference { .. }), "{err:?}");
    }

    #[test]
    fn missing_managed_directory_scans_as_an_empty_collection() {
        // The ideas directory is created on first use; its absence is an
        // empty collection, not damage, even while dragons exist.
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-alone.md",
            &dragon_markdown("id-1", 1, "open", "Alone"),
        );

        assert!(scan(tmp.path(), &IDEA).unwrap().is_empty());
        assert_eq!(scan(tmp.path(), &DRAGON).unwrap().len(), 1);
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
        fs::remove_dir(tmp.path().join(DRAGONS_DIR)).unwrap();
        fs::write(tmp.path().join(DRAGONS_DIR), "not a directory").unwrap();

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        match err {
            Error::ArtifactConflict { path, .. } => {
                assert!(path.ends_with(DRAGONS_DIR), "{path:?}");
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
            sprint: None,
            path: "archaeology/dragons/0007-a-title.md".into(),
        };

        assert_eq!(
            serde_json::to_string(&summary).unwrap(),
            "{\"id\":\"drg-x\",\"sequence\":7,\"kind\":\"dragon\",\"status\":\"open\",\
             \"title\":\"A title\",\"created\":\"2026-07-20\",\
             \"path\":\"archaeology/dragons/0007-a-title.md\"}"
        );
    }

    #[test]
    fn task_summaries_serialize_the_owning_sprint() {
        let summary = Summary {
            id: "tsk-x".into(),
            sequence: 17,
            kind: "task".into(),
            status: Status::Pending,
            title: "A task".into(),
            created: "2026-07-22".into(),
            sprint: Some("spr-x".into()),
            path: "archaeology/sprints/0005-x/0017-a-task.md".into(),
        };

        assert_eq!(
            serde_json::to_string(&summary).unwrap(),
            "{\"id\":\"tsk-x\",\"sequence\":17,\"kind\":\"task\",\"status\":\"pending\",\
             \"title\":\"A task\",\"created\":\"2026-07-22\",\"sprint\":\"spr-x\",\
             \"path\":\"archaeology/sprints/0005-x/0017-a-task.md\"}"
        );
    }

    // --- task 22: the bounded-read seam ---

    #[test]
    fn read_accepts_a_file_exactly_at_the_byte_limit() {
        let tmp = temp_repo();
        let path = tmp.path().join(DRAGONS_DIR).join("0001-cap.md");
        fs::write(&path, vec![b'a'; MAX_ARTIFACT_BYTES as usize]).unwrap();

        let content = read_artifact_bytes(&path).unwrap();

        assert_eq!(content.len() as u64, MAX_ARTIFACT_BYTES);
    }

    #[test]
    fn read_refuses_one_byte_over_the_limit_naming_the_cap() {
        let tmp = temp_repo();
        let path = tmp.path().join(DRAGONS_DIR).join("0001-over.md");
        fs::write(&path, vec![b'a'; MAX_ARTIFACT_BYTES as usize + 1]).unwrap();

        let err = read_artifact_bytes(&path).unwrap_err();

        expect_malformed(err, "0001-over.md", &MAX_ARTIFACT_BYTES.to_string());
    }

    #[test]
    fn bounded_invalid_utf8_keeps_the_typed_distinction() {
        let tmp = temp_repo();
        let path = tmp.path().join(DRAGONS_DIR).join("0001-latin1.md");
        fs::write(&path, [b'-', b'-', b'-', b'\n', 0xff, 0xfe]).unwrap();

        let err = read_artifact_bytes(&path).unwrap_err();

        expect_malformed(err, "0001-latin1.md", "not valid UTF-8");
    }

    #[test]
    fn oversized_regular_artifact_fails_the_scan_with_the_cap() {
        // No symlink involved: an oversized committed artifact is the
        // load-bearing case from thread 4's claim 2.
        let tmp = temp_repo();
        let mut content = dragon_markdown("drg-big", 1, "open", "Big");
        content.push_str(&"x".repeat(MAX_ARTIFACT_BYTES as usize));
        write_dragon(tmp.path(), DRAGONS_DIR, "0001-big.md", &content);

        let err = scan(tmp.path(), &DRAGON).unwrap_err();

        expect_malformed(err, "0001-big.md", &MAX_ARTIFACT_BYTES.to_string());
    }

    // --- task 22: symlinks are classified, refused, and never followed ---

    #[cfg(unix)]
    mod symlink_boundary {
        use super::*;
        use std::os::unix::fs::symlink;

        fn expect_symlink_conflict(err: Error, name: &str) {
            match err {
                Error::ArtifactConflict { path, reason } => {
                    assert!(
                        path.ends_with(name),
                        "expected path ending {name}: {path:?}"
                    );
                    assert!(reason.contains("symbolic link"), "{reason}");
                }
                other => panic!("expected artifact conflict, got {other:?}"),
            }
        }

        #[test]
        fn file_symlink_at_a_flat_artifact_position_is_refused_unread() {
            let tmp = temp_repo();
            let outside = tempfile::tempdir().unwrap();
            let target = outside.path().join("outside.md");
            fs::write(
                &target,
                dragon_markdown("drg-outside", 2, "open", "Outside"),
            )
            .unwrap();
            symlink(&target, tmp.path().join(DRAGONS_DIR).join("0002-evil.md")).unwrap();

            let err = scan(tmp.path(), &DRAGON).unwrap_err();

            // The scan refuses before any read, so the outside content
            // can never reach a projection.
            expect_symlink_conflict(err, "0002-evil.md");
        }

        #[test]
        fn directory_symlink_at_a_flat_artifact_position_remains_a_conflict() {
            let tmp = temp_repo();
            symlink(
                tmp.path().join("archaeology"),
                tmp.path().join(DRAGONS_DIR).join("loop"),
            )
            .unwrap();

            let err = scan(tmp.path(), &DRAGON).unwrap_err();

            expect_symlink_conflict(err, "loop");
        }

        #[test]
        fn symlinked_managed_directory_is_a_conflict_never_traversed() {
            let tmp = temp_repo();
            let outside = tempfile::tempdir().unwrap();
            fs::write(
                outside.path().join("0001-planted.md"),
                dragon_markdown("drg-planted", 1, "open", "Planted"),
            )
            .unwrap();
            fs::remove_dir(tmp.path().join(DRAGONS_DIR)).unwrap();
            symlink(outside.path(), tmp.path().join(DRAGONS_DIR)).unwrap();

            let err = scan(tmp.path(), &DRAGON).unwrap_err();

            expect_symlink_conflict(err, DRAGONS_DIR);
        }

        #[test]
        fn symlinked_sprint_containment_directory_is_refused() {
            let tmp = temp_repo();
            let outside = tempfile::tempdir().unwrap();
            fs::write(
                outside.path().join(crate::repo::SPRINT_FILE),
                "---\nid: spr-evil\nsequence: 1\nkind: sprint\nstatus: active\ncreated: 2026-07-20\n---\n\n# Evil\n",
            )
            .unwrap();
            fs::create_dir_all(tmp.path().join(SPRINTS_DIR)).unwrap();
            symlink(
                outside.path(),
                tmp.path().join(SPRINTS_DIR).join("0001-evil"),
            )
            .unwrap();

            let err = scan_sprints(tmp.path()).unwrap_err();

            expect_symlink_conflict(err, "0001-evil");
        }

        #[test]
        fn symlinked_sprint_file_is_refused() {
            let tmp = temp_repo();
            let outside = tempfile::tempdir().unwrap();
            let target = outside.path().join("sprint.md");
            fs::write(
                &target,
                "---\nid: spr-evil\nsequence: 1\nkind: sprint\nstatus: active\ncreated: 2026-07-20\n---\n\n# Evil\n",
            )
            .unwrap();
            let dir = tmp.path().join(SPRINTS_DIR).join("0001-hollow");
            fs::create_dir_all(&dir).unwrap();
            symlink(&target, dir.join(crate::repo::SPRINT_FILE)).unwrap();

            let err = scan_sprints(tmp.path()).unwrap_err();

            expect_symlink_conflict(err, "sprint.md");
        }

        #[test]
        fn symlinked_task_file_is_refused() {
            let tmp = temp_repo();
            seed_sprint(tmp.path(), "0001-first", 1, "active");
            let outside = tempfile::tempdir().unwrap();
            let target = outside.path().join("task.md");
            fs::write(
                &target,
                "---\nid: tsk-evil\nsequence: 1\nkind: task\nstatus: pending\nsprint: spr-1\ncreated: 2026-07-20\n---\n\n# Evil\n",
            )
            .unwrap();
            symlink(
                &target,
                tmp.path()
                    .join(SPRINTS_DIR)
                    .join("0001-first")
                    .join("0001-evil.md"),
            )
            .unwrap();

            let err = scan_tasks(tmp.path()).unwrap_err();

            expect_symlink_conflict(err, "0001-evil.md");
        }
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
