//! Artifact creation for managed collections.
//!
//! Strata owns the mechanics callers must not hand-roll: display sequence
//! allocation, deterministic slugging, stable identity assignment, and safe
//! writes. The filesystem stays canonical — a created artifact is an
//! ordinary Markdown file with YAML-style front matter.
//!
//! # Identity
//!
//! The `id` front matter field is an opaque stable string. Artifacts created
//! here use a per-collection prefix (`drg_`, `ide_`) followed by an
//! uppercase ULID. Pre-existing hand-seeded identifiers (for example
//! `drg-bootstrap-branch-collisions` or `idea-strata-fortune`) remain valid:
//! nothing in Strata may require every `id` to be a ULID, and there is no
//! second identity field.
//!
//! # Concurrency boundary
//!
//! Creation is scan-then-write without locking. Two simultaneous Strata
//! processes — or two Git branches — can allocate the same next display
//! sequence; bootstrap deliberately does not make allocation linearizable
//! and introduces no lock service. What IS guaranteed:
//!
//! - no existing file is ever overwritten: content is staged in a temporary
//!   file and persisted with an atomic no-clobber rename;
//! - a failed creation leaves no partial destination artifact (an abandoned
//!   temporary is a dot-file, which artifact scans ignore);
//! - duplicate display sequences produced by concurrent allocation remain on
//!   disk as distinct files, detectable later by `strata doctor`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::read::{Collection, DRAGON, IDEA, SPRINT, Status, TASK};
use crate::repo::{SPRINT_FILE, SPRINTS_DIR};

/// Prefix for generated dragon identities.
pub const DRAGON_ID_PREFIX: &str = "drg_";

/// Prefix for generated idea identities.
pub const IDEA_ID_PREFIX: &str = "ide_";

/// Prefix for generated sprint identities.
pub const SPRINT_ID_PREFIX: &str = "spr_";

/// Prefix for generated task identities.
pub const TASK_ID_PREFIX: &str = "tsk_";

/// Largest display sequence representable by four-digit filename prefixes.
pub const MAX_SEQUENCE: u32 = 9999;

/// A successfully created artifact.
#[derive(Debug)]
pub struct NewArtifact {
    /// Canonical singular kind name, e.g. `dragon`.
    pub kind: &'static str,
    /// Stable opaque identity: collection prefix + uppercase ULID.
    pub id: String,
    /// Collection-scoped display sequence.
    pub sequence: u32,
    /// Destination path relative to the repository root.
    pub relative_path: PathBuf,
}

impl NewArtifact {
    /// Human reference for the created artifact, e.g. `dragon:2`.
    pub fn reference(&self) -> String {
        format!("{}:{}", self.kind, self.sequence)
    }
}

/// Create a new open dragon in the repository at `root`.
pub fn create_dragon(root: &Path, title: &str) -> Result<NewArtifact, Error> {
    const SECTIONS: &[&str] = &["Context", "Question", "Constraints", "Resolution criteria"];
    create(root, &DRAGON, DRAGON_ID_PREFIX, SECTIONS, title)
}

/// Create a new parked idea in the repository at `root`.
pub fn create_idea(root: &Path, title: &str) -> Result<NewArtifact, Error> {
    const SECTIONS: &[&str] = &["Problem", "Sketch", "Evidence"];
    create(root, &IDEA, IDEA_ID_PREFIX, SECTIONS, title)
}

/// Create a new active sprint in the repository at `root`.
///
/// A sprint artifact is `sprint.md` inside a fresh containment directory
/// `NNNN-slug/`; the directory name carries the display sequence. At most
/// one sprint may be active, so creation is refused while one is —
/// naming it, since the caller's next move is usually to close it.
///
/// Deliberate duplication of [`create`] (idea 10 discipline): the
/// containment-directory layout diverges from flat files at almost every
/// step — sequence source, destination materialization, template shape —
/// so the shared machinery reduces to slugging, identity, and the safe
/// write.
pub fn create_sprint(root: &Path, title: &str) -> Result<NewArtifact, Error> {
    const SECTIONS: &[&str] = &["Goal", "Rationale", "Success criteria", "Non-goals"];
    let title = title.trim();
    let slug = slugify(title).ok_or_else(|| Error::InvalidInvocation {
        message: format!(
            "cannot create a sprint titled `{title}`: the title must contain \
             at least one ASCII letter or digit to derive a directory slug"
        ),
    })?;

    let sprints = crate::read::scan_sprints(root)?;
    if let Some(active) = sprints
        .iter()
        .find(|sprint| sprint.summary.status == Status::Active)
    {
        return Err(Error::InvalidInvocation {
            message: format!(
                "sprint `{}` ({}) is still active; at most one sprint may be \
                 active — close it with `strata close {}` first",
                active.summary.reference(),
                active.summary.title,
                active.summary.reference()
            ),
        });
    }

    let max = sprints
        .iter()
        .map(|sprint| sprint.summary.sequence)
        .max()
        .unwrap_or(0);
    if max >= MAX_SEQUENCE {
        return Err(Error::ArtifactConflict {
            path: root.join(SPRINTS_DIR),
            reason: format!(
                "the sprint collection has exhausted the four-digit display \
                 sequence space (last sequence is {MAX_SEQUENCE})"
            ),
        });
    }
    let sequence = max + 1;
    let id = format!("{SPRINT_ID_PREFIX}{}", ulid::Ulid::new());
    let created = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();
    let content = render_artifact(&Template {
        id: &id,
        sequence,
        kind: SPRINT.kind,
        status: Status::Active.name(),
        extra_fields: &[],
        created: &created,
        title,
        sections: SECTIONS,
    });

    let dir_rel = format!("{SPRINTS_DIR}/{sequence:04}-{slug}");
    crate::repo::ensure_dir(root, &dir_rel, &mut Vec::new())?;
    write_new(&root.join(&dir_rel), SPRINT_FILE, &content)?;

    Ok(NewArtifact {
        kind: SPRINT.kind,
        id,
        sequence,
        relative_path: Path::new(&dir_rel).join(SPRINT_FILE),
    })
}

/// Create a new artifact in its collection's home lifecycle state.
///
/// Allocates `max(existing sequence) + 1` across the collection's
/// directory, derives a deterministic kebab-case slug from `title`,
/// assigns a fresh prefixed ULID identity, and writes the Markdown template
/// through a temporary file with an atomic no-clobber persist. Neither
/// `.strata.toml` nor any existing artifact is modified.
fn create(
    root: &Path,
    collection: &Collection,
    id_prefix: &str,
    sections: &[&str],
    title: &str,
) -> Result<NewArtifact, Error> {
    let title = title.trim();
    let kind = collection.kind;
    let slug = slugify(title).ok_or_else(|| Error::InvalidInvocation {
        message: format!(
            "cannot create a {kind} titled `{title}`: the title must contain \
             at least one ASCII letter or digit to derive a filename slug"
        ),
    })?;

    let sequence = next_sequence(root, collection)?;
    let id = format!("{id_prefix}{}", ulid::Ulid::new());
    let created = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();
    let home_status = collection.states[0];
    let content = render_artifact(&Template {
        id: &id,
        sequence,
        kind,
        status: home_status.name(),
        extra_fields: &[],
        created: &created,
        title,
        sections,
    });

    // Git does not round-trip empty directories, so a cloned repository may
    // lack the destination; materialize it with the same conflict checks
    // `init` uses.
    crate::repo::ensure_dir(root, collection.dir, &mut Vec::new())?;
    let filename = format!("{sequence:04}-{slug}.md");
    write_new(&root.join(collection.dir), &filename, &content)?;

    Ok(NewArtifact {
        kind,
        id,
        sequence,
        relative_path: Path::new(collection.dir).join(filename),
    })
}

/// Create a new pending task in the active sprint at `root`.
///
/// Tasks require an active sprint: the new file lands in its containment
/// directory, stamps the sprint's stable id into the `sprint:` field, and
/// takes the next display sequence globally across every sprint.
pub fn create_task(root: &Path, title: &str) -> Result<NewArtifact, Error> {
    const SECTIONS: &[&str] = &["Objective", "Acceptance criteria"];
    let title = title.trim();
    let slug = slugify(title).ok_or_else(|| Error::InvalidInvocation {
        message: format!(
            "cannot create a task titled `{title}`: the title must contain \
             at least one ASCII letter or digit to derive a filename slug"
        ),
    })?;

    let sprints = crate::read::scan_sprints(root)?;
    let Some(active) = sprints
        .iter()
        .find(|sprint| sprint.summary.status == Status::Active)
    else {
        return Err(Error::InvalidInvocation {
            message: "tasks belong to a sprint, and no sprint is active; \
                      open one with `strata new sprint \"<goal>\"` first"
                .into(),
        });
    };

    let tasks = crate::read::scan_tasks(root)?;
    let max = tasks
        .iter()
        .map(|task| task.summary.sequence)
        .max()
        .unwrap_or(0);
    if max >= MAX_SEQUENCE {
        return Err(Error::ArtifactConflict {
            path: root.join(SPRINTS_DIR),
            reason: format!(
                "the task collection has exhausted the four-digit display \
                 sequence space (last sequence is {MAX_SEQUENCE})"
            ),
        });
    }
    let sequence = max + 1;
    let id = format!("{TASK_ID_PREFIX}{}", ulid::Ulid::new());
    let created = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();
    let content = render_artifact(&Template {
        id: &id,
        sequence,
        kind: TASK.kind,
        status: Status::Pending.name(),
        extra_fields: &[("sprint", &active.summary.id)],
        created: &created,
        title,
        sections: SECTIONS,
    });

    let sprint_dir = active
        .summary
        .path
        .rsplit_once('/')
        .expect("sprint paths always contain a directory")
        .0
        .to_string();
    let filename = format!("{sequence:04}-{slug}.md");
    write_new(&root.join(&sprint_dir), &filename, &content)?;

    Ok(NewArtifact {
        kind: TASK.kind,
        id,
        sequence,
        relative_path: Path::new(&sprint_dir).join(filename),
    })
}

/// Derive a deterministic slug from a title.
///
/// Lowercase ASCII alphanumerics, words separated by single hyphens; runs of
/// any other character — including all non-ASCII characters, which are not
/// transliterated — collapse into one separator, and leading/trailing
/// separators are stripped. Returns `None` when nothing sluggable remains.
pub fn slugify(title: &str) -> Option<String> {
    let mut slug = String::new();
    let mut pending_separator = false;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_separator {
                slug.push('-');
            }
            slug.push(c.to_ascii_lowercase());
            pending_separator = false;
        } else if !slug.is_empty() {
            pending_separator = true;
        }
    }
    if slug.is_empty() { None } else { Some(slug) }
}

/// Allocate the next display sequence by scanning the collection's
/// directory, refusing to exceed the four-digit space.
fn next_sequence(root: &Path, collection: &Collection) -> Result<u32, Error> {
    let max = max_sequence_in(&root.join(collection.dir), collection.kind)?;
    if max >= MAX_SEQUENCE {
        return Err(Error::ArtifactConflict {
            path: root.join(collection.dir),
            reason: format!(
                "the {} collection has exhausted the four-digit display \
                 sequence space (last sequence is {MAX_SEQUENCE})",
                collection.kind
            ),
        });
    }
    Ok(max + 1)
}

/// Largest display sequence among the artifacts in one managed directory.
///
/// A missing managed directory is an empty collection (Git does not
/// round-trip empty directories); a non-directory object occupying the
/// managed path is a conflict. Every non-hidden entry must be a valid
/// dragon filename; malformed names are a typed error naming the path,
/// never silently skipped. Entries starting with `.` (editor and VCS
/// metadata, abandoned temporaries) are not artifacts and are ignored.
fn max_sequence_in(dir: &Path, kind: &str) -> Result<u32, Error> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(0),
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

    let mut max = 0u32;
    for entry in entries {
        let entry = entry.map_err(|source| Error::Filesystem {
            operation: "read directory entry".into(),
            path: dir.to_path_buf(),
            source,
        })?;
        let name = entry.file_name();
        let malformed = |reason: String| Error::MalformedArtifact {
            path: dir.join(&name),
            reason,
        };
        let Some(name_str) = name.to_str() else {
            return Err(malformed("filename is not valid UTF-8".into()));
        };
        if name_str.starts_with('.') {
            continue;
        }
        let sequence = parse_sequence(name_str).ok_or_else(|| {
            malformed(format!(
                "{kind} filenames must be `NNNN-slug.md` with a four-digit \
                 display sequence"
            ))
        })?;
        max = max.max(sequence);
    }
    Ok(max)
}

/// Extract the display sequence from a valid artifact filename
/// (`NNNN-slug.md`), or `None` when the name does not conform.
pub(crate) fn parse_sequence(name: &str) -> Option<u32> {
    let digits = name.get(..4)?;
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let slug = name[4..].strip_prefix('-')?.strip_suffix(".md")?;
    if slug.is_empty() {
        return None;
    }
    digits.parse().ok()
}

/// Extract the display sequence from a valid sprint containment directory
/// name (`NNNN-slug`), or `None` when the name does not conform.
pub(crate) fn parse_dir_sequence(name: &str) -> Option<u32> {
    let digits = name.get(..4)?;
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let slug = name[4..].strip_prefix('-')?;
    if slug.is_empty() || slug.ends_with(".md") {
        return None;
    }
    digits.parse().ok()
}

/// One artifact payload to render: front matter plus template sections.
/// `extra_fields` land in the front matter after `status`, before
/// `created`.
struct Template<'a> {
    id: &'a str,
    sequence: u32,
    kind: &'a str,
    status: &'a str,
    extra_fields: &'a [(&'a str, &'a str)],
    created: &'a str,
    title: &'a str,
    sections: &'a [&'a str],
}

/// Render an artifact Markdown payload.
fn render_artifact(template: &Template<'_>) -> String {
    let Template {
        id,
        sequence,
        kind,
        status,
        extra_fields,
        created,
        title,
        sections,
    } = template;
    let mut content = format!(
        "---\n\
         id: {id}\n\
         sequence: {sequence}\n\
         kind: {kind}\n\
         status: {status}\n"
    );
    for (key, value) in *extra_fields {
        content.push_str(key);
        content.push_str(": ");
        content.push_str(value);
        content.push('\n');
    }
    content.push_str(&format!(
        "created: {created}\n\
         ---\n\
         \n\
         # {title}\n"
    ));
    for section in *sections {
        content.push_str("\n## ");
        content.push_str(section);
        content.push('\n');
    }
    content
}

/// Write `content` to `dir/filename` through an exclusive temporary file in
/// `dir` and an atomic no-clobber persist. A failure at any point leaves no
/// destination file; an existing destination is never replaced.
fn write_new(dir: &Path, filename: &str, content: &str) -> Result<(), Error> {
    let destination = dir.join(filename);
    let mut tmp = tempfile::Builder::new()
        .prefix(".strata.artifact.tmp")
        .tempfile_in(dir)
        .map_err(|source| Error::Filesystem {
            operation: "create temporary artifact".into(),
            path: dir.to_path_buf(),
            source,
        })?;
    tmp.write_all(content.as_bytes())
        .map_err(|source| Error::Filesystem {
            operation: "write temporary artifact".into(),
            path: tmp.path().to_path_buf(),
            source,
        })?;
    tmp.persist_noclobber(&destination).map_err(|err| {
        if err.error.kind() == io::ErrorKind::AlreadyExists {
            Error::ArtifactConflict {
                path: destination.clone(),
                reason: "an artifact already occupies the destination path".into(),
            }
        } else {
            Error::Filesystem {
                operation: "persist artifact".into(),
                path: destination.clone(),
                source: err.error,
            }
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo;
    use crate::repo::{DRAGONS_DIR, IDEAS_DIR};

    fn temp_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create temporary directory");
        repo::init(tmp.path()).expect("initialize repository");
        tmp
    }

    fn dragons_dir_entries(root: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(root.join(DRAGONS_DIR))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn slugify_is_deterministic_kebab_case() {
        let cases = [
            ("Branch sequence collisions", "branch-sequence-collisions"),
            ("Hello,   World!!", "hello-world"),
            ("--Already--Separated--", "already-separated"),
            ("MiXeD CaSe 123", "mixed-case-123"),
            ("v2.0 rollout", "v2-0-rollout"),
        ];
        for (title, expected) in cases {
            assert_eq!(slugify(title).as_deref(), Some(expected), "for {title:?}");
        }
    }

    #[test]
    fn slugify_treats_non_ascii_as_separators_without_transliterating() {
        assert_eq!(slugify("Café risk").as_deref(), Some("caf-risk"));
        assert_eq!(slugify("naïve merge").as_deref(), Some("na-ve-merge"));
        assert_eq!(slugify("日本語"), None);
    }

    #[test]
    fn slugify_rejects_titles_with_nothing_sluggable() {
        for title in ["", "   ", "!!!", "---", "…"] {
            assert_eq!(slugify(title), None, "for {title:?}");
        }
    }

    #[test]
    fn create_writes_zero_padded_filename_with_front_matter_and_headings() {
        let tmp = temp_repo();

        let dragon = create_dragon(tmp.path(), "Branch sequence collisions").unwrap();

        assert_eq!(dragon.sequence, 1);
        assert_eq!(dragon.reference(), "dragon:1");
        assert_eq!(
            dragon.relative_path,
            Path::new(DRAGONS_DIR).join("0001-branch-sequence-collisions.md")
        );
        let content = fs::read_to_string(tmp.path().join(&dragon.relative_path)).unwrap();
        assert!(content.starts_with("---\n"), "{content}");
        for line in [
            format!("id: {}", dragon.id),
            "sequence: 1".into(),
            "kind: dragon".into(),
            "status: open".into(),
        ] {
            assert!(
                content.contains(&format!("\n{line}\n")),
                "missing `{line}` in:\n{content}"
            );
        }
        for heading in [
            "# Branch sequence collisions",
            "## Context",
            "## Question",
            "## Constraints",
            "## Resolution criteria",
        ] {
            assert!(
                content.contains(heading),
                "missing `{heading}` in:\n{content}"
            );
        }
    }

    #[test]
    fn create_idea_writes_a_parked_artifact_with_idea_template() {
        let tmp = temp_repo();

        let idea = create_idea(tmp.path(), "Declarative specs").unwrap();

        assert_eq!(idea.sequence, 1);
        assert_eq!(idea.reference(), "idea:1");
        assert_eq!(
            idea.relative_path,
            Path::new(IDEAS_DIR).join("0001-declarative-specs.md")
        );
        assert!(idea.id.starts_with(IDEA_ID_PREFIX), "{}", idea.id);
        let content = fs::read_to_string(tmp.path().join(&idea.relative_path)).unwrap();
        for line in ["kind: idea", "status: parked", "sequence: 1"] {
            assert!(
                content.contains(&format!("\n{line}\n")),
                "missing `{line}` in:\n{content}"
            );
        }
        for heading in [
            "# Declarative specs",
            "## Problem",
            "## Sketch",
            "## Evidence",
        ] {
            assert!(
                content.contains(heading),
                "missing `{heading}` in:\n{content}"
            );
        }
    }

    #[test]
    fn idea_sequences_are_collection_scoped_and_ignore_dragons() {
        let tmp = temp_repo();
        // A dragon with a high sequence must not influence idea allocation.
        fs::write(
            tmp.path().join(DRAGONS_DIR).join("0009-dragon.md"),
            "seeded",
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join(IDEAS_DIR)).unwrap();
        fs::write(
            tmp.path().join(IDEAS_DIR).join("0004-rejected.md"),
            "seeded",
        )
        .unwrap();

        let idea = create_idea(tmp.path(), "Next idea").unwrap();

        assert_eq!(idea.sequence, 5, "must continue after the terminal maximum");
    }

    #[test]
    fn create_idea_materializes_the_ideas_directory_on_first_use() {
        let tmp = temp_repo();
        assert!(
            !tmp.path().join(IDEAS_DIR).exists(),
            "init must not pre-create the ideas directory"
        );

        let idea = create_idea(tmp.path(), "First idea").unwrap();

        assert!(tmp.path().join(&idea.relative_path).is_file());
    }

    #[test]
    fn create_stamps_an_iso_date() {
        let tmp = temp_repo();

        let dragon = create_dragon(tmp.path(), "Dated risk").unwrap();

        let content = fs::read_to_string(tmp.path().join(&dragon.relative_path)).unwrap();
        let created = content
            .lines()
            .find_map(|l| l.strip_prefix("created: "))
            .expect("front matter must include `created`");
        let bytes = created.as_bytes();
        assert_eq!(bytes.len(), 10, "created must be YYYY-MM-DD: {created}");
        assert!(
            bytes.iter().enumerate().all(|(i, b)| match i {
                4 | 7 => *b == b'-',
                _ => b.is_ascii_digit(),
            }),
            "created must be YYYY-MM-DD: {created}"
        );
    }

    #[test]
    fn generated_ids_are_prefixed_uppercase_ulids_and_unique() {
        let tmp = temp_repo();

        let first = create_dragon(tmp.path(), "First risk").unwrap();
        let second = create_dragon(tmp.path(), "Second risk").unwrap();

        assert_ne!(first.id, second.id);
        for id in [&first.id, &second.id] {
            let ulid = id
                .strip_prefix(DRAGON_ID_PREFIX)
                .unwrap_or_else(|| panic!("id must start with `{DRAGON_ID_PREFIX}`: {id}"));
            assert_eq!(ulid.len(), 26, "{id}");
            assert!(
                ulid.bytes().all(
                    |b| b.is_ascii_digit() || (b.is_ascii_uppercase() && !b"ILOU".contains(&b))
                ),
                "id must be an uppercase Crockford base32 ULID: {id}"
            );
        }
    }

    #[test]
    fn sequence_allocation_spans_every_lifecycle_state() {
        let tmp = temp_repo();
        fs::write(tmp.path().join(DRAGONS_DIR).join("0001-old.md"), "seeded").unwrap();
        fs::write(
            tmp.path().join(DRAGONS_DIR).join("0005-resolved.md"),
            "seeded",
        )
        .unwrap();

        let dragon = create_dragon(tmp.path(), "Next risk").unwrap();

        assert_eq!(dragon.sequence, 6, "must continue after the maximum");
        assert!(
            tmp.path()
                .join(DRAGONS_DIR)
                .join("0006-next-risk.md")
                .is_file()
        );
    }

    #[test]
    fn seeded_non_ulid_ids_are_left_untouched() {
        let tmp = temp_repo();
        let seeded_path = tmp.path().join(DRAGONS_DIR).join("0001-seeded.md");
        let seeded = "---\nid: drg-bootstrap-branch-collisions\nsequence: 1\n---\n";
        fs::write(&seeded_path, seeded).unwrap();

        let dragon = create_dragon(tmp.path(), "Fresh risk").unwrap();

        assert_eq!(dragon.sequence, 2);
        assert_eq!(
            fs::read_to_string(&seeded_path).unwrap(),
            seeded,
            "existing artifacts must remain byte-identical"
        );
    }

    #[test]
    fn malformed_filenames_are_reported_not_skipped() {
        let tmp = temp_repo();
        for bad in [
            "notes.txt",
            "12-short.md",
            "0002.md",
            "0002-.md",
            "abcd-x.md",
        ] {
            let path = tmp.path().join(DRAGONS_DIR).join(bad);
            fs::write(&path, "junk").unwrap();

            let err = create_dragon(tmp.path(), "Any title").unwrap_err();

            match err {
                Error::MalformedArtifact { path: reported, .. } => {
                    assert_eq!(reported, path, "error must name the offending file")
                }
                other => panic!("expected malformed artifact for {bad:?}, got {other:?}"),
            }
            fs::remove_file(&path).unwrap();
        }
    }

    #[test]
    fn dot_entries_are_not_artifacts_and_are_ignored() {
        let tmp = temp_repo();
        fs::write(tmp.path().join(DRAGONS_DIR).join(".gitkeep"), "").unwrap();

        let dragon = create_dragon(tmp.path(), "Ignores dotfiles").unwrap();

        assert_eq!(dragon.sequence, 1);
    }

    #[test]
    fn create_materializes_missing_managed_directories() {
        // Simulate `git clone` of a freshly initialized repository: Git
        // preserves the marker but drops every empty directory.
        let tmp = temp_repo();
        fs::remove_dir_all(tmp.path().join("archaeology")).unwrap();

        let dragon = create_dragon(tmp.path(), "Post-clone risk").unwrap();

        assert_eq!(dragon.sequence, 1);
        assert!(tmp.path().join(&dragon.relative_path).is_file());
    }

    #[test]
    fn non_directory_at_managed_path_is_a_conflict() {
        let tmp = temp_repo();
        fs::remove_dir(tmp.path().join(DRAGONS_DIR)).unwrap();
        fs::write(tmp.path().join(DRAGONS_DIR), "not a directory").unwrap();

        let err = create_dragon(tmp.path(), "Any title").unwrap_err();

        assert!(matches!(err, Error::ArtifactConflict { .. }), "{err:?}");
    }

    #[test]
    fn sequence_exhaustion_is_a_typed_error_not_a_five_digit_filename() {
        let tmp = temp_repo();
        fs::write(tmp.path().join(DRAGONS_DIR).join("9999-last.md"), "seeded").unwrap();

        let err = create_dragon(tmp.path(), "One too many").unwrap_err();

        assert!(matches!(err, Error::ArtifactConflict { .. }), "{err:?}");
        assert_eq!(
            dragons_dir_entries(tmp.path()),
            vec!["9999-last.md".to_string()],
            "no artifact may be created on exhaustion"
        );
    }

    #[test]
    fn destination_conflict_refuses_overwrite_and_leaves_no_temporary() {
        let tmp = temp_repo();
        let dir = tmp.path().join(DRAGONS_DIR);
        fs::write(dir.join("0001-taken.md"), "original").unwrap();

        // Drive the write layer directly: it is the guard that holds even
        // when a file appears between the sequence scan and the persist.
        let err = write_new(&dir, "0001-taken.md", "new content").unwrap_err();

        assert!(matches!(err, Error::ArtifactConflict { .. }), "{err:?}");
        assert_eq!(
            fs::read_to_string(dir.join("0001-taken.md")).unwrap(),
            "original",
            "the existing artifact must survive byte-identical"
        );
        assert_eq!(
            dragons_dir_entries(tmp.path()),
            vec!["0001-taken.md".to_string()],
            "a failed persist must not litter temporaries"
        );
    }

    #[cfg(unix)]
    #[test]
    fn induced_write_failure_leaves_no_partial_artifact() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = temp_repo();
        let dir = tmp.path().join(DRAGONS_DIR);
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();

        let result = create_dragon(tmp.path(), "Cannot be written");

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            matches!(result, Err(Error::Filesystem { .. })),
            "{result:?}"
        );
        assert_eq!(
            dragons_dir_entries(tmp.path()),
            Vec::<String>::new(),
            "a failed creation must leave nothing behind"
        );
    }

    #[test]
    fn invalid_title_creates_nothing() {
        let tmp = temp_repo();

        let err = create_dragon(tmp.path(), "!!!").unwrap_err();

        assert!(matches!(err, Error::InvalidInvocation { .. }), "{err:?}");
        assert_eq!(dragons_dir_entries(tmp.path()), Vec::<String>::new());
    }

    #[test]
    fn create_sprint_writes_template_in_a_fresh_containment_directory() {
        let tmp = temp_repo();

        let sprint = create_sprint(tmp.path(), "Placement and sprints").unwrap();

        assert_eq!(sprint.sequence, 1);
        assert_eq!(sprint.reference(), "sprint:1");
        assert!(sprint.id.starts_with(SPRINT_ID_PREFIX), "{}", sprint.id);
        assert_eq!(
            sprint.relative_path,
            Path::new(SPRINTS_DIR).join("0001-placement-and-sprints/sprint.md")
        );
        let content = fs::read_to_string(tmp.path().join(&sprint.relative_path)).unwrap();
        for needle in [
            "kind: sprint",
            "status: active",
            "# Placement and sprints",
            "## Goal",
            "## Rationale",
            "## Success criteria",
            "## Non-goals",
        ] {
            assert!(content.contains(needle), "missing `{needle}`:\n{content}");
        }
    }

    #[test]
    fn create_sprint_is_refused_while_one_is_active() {
        let tmp = temp_repo();
        create_sprint(tmp.path(), "First").unwrap();

        let err = create_sprint(tmp.path(), "Second").unwrap_err();

        assert!(matches!(err, Error::InvalidInvocation { .. }), "{err:?}");
        let message = err.to_string();
        assert!(
            message.contains("sprint:1") && message.contains("strata close"),
            "the refusal must name the active sprint and the way out: {message}"
        );
    }

    #[test]
    fn sprint_sequences_continue_after_closed_sprints() {
        let tmp = temp_repo();
        let dir = tmp.path().join(SPRINTS_DIR).join("0004-history");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(SPRINT_FILE),
            "---\nid: spr-history\nsequence: 4\nkind: sprint\nstatus: closed\ncreated: 2026-07-20\n---\n\n# History\n",
        )
        .unwrap();

        let sprint = create_sprint(tmp.path(), "Next").unwrap();

        assert_eq!(sprint.sequence, 5);
    }

    #[test]
    fn parse_dir_sequence_accepts_only_containment_directory_names() {
        assert_eq!(parse_dir_sequence("0001-bootstrap"), Some(1));
        assert_eq!(parse_dir_sequence("9999-x"), Some(9999));
        for bad in ["0001-x.md", "001-x", "0001-", "abcd-x", "0001"] {
            assert_eq!(parse_dir_sequence(bad), None, "for {bad:?}");
        }
    }

    #[test]
    fn parse_sequence_accepts_only_valid_dragon_filenames() {
        assert_eq!(parse_sequence("0001-x.md"), Some(1));
        assert_eq!(parse_sequence("9999-some-long-slug.md"), Some(9999));
        for bad in [
            "001-x.md",
            "00001-x.md",
            "0001x.md",
            "0001-.md",
            "0001-x.txt",
            "0001-x.md.bak",
        ] {
            assert_eq!(parse_sequence(bad), None, "for {bad:?}");
        }
    }
}
