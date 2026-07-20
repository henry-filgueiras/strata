//! Artifact creation for the bootstrap `dragon` collection.
//!
//! Strata owns the mechanics callers must not hand-roll: display sequence
//! allocation, deterministic slugging, stable identity assignment, and safe
//! writes. The filesystem stays canonical — a created dragon is an ordinary
//! Markdown file with YAML-style front matter.
//!
//! # Identity
//!
//! The `id` front matter field is an opaque stable string. Artifacts created
//! here use a `drg_` prefix followed by an uppercase ULID. Pre-existing
//! hand-seeded identifiers (for example `drg-bootstrap-branch-collisions`)
//! remain valid: nothing in Strata may require every `id` to be a ULID, and
//! there is no second identity field.
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
use crate::repo::{DRAGONS_CLOSED_DIR, DRAGONS_OPEN_DIR};

/// Prefix for generated dragon identities.
pub const DRAGON_ID_PREFIX: &str = "drg_";

/// Largest display sequence representable by four-digit filename prefixes.
pub const MAX_SEQUENCE: u32 = 9999;

/// A successfully created dragon artifact.
#[derive(Debug)]
pub struct NewDragon {
    /// Stable opaque identity, `drg_` + uppercase ULID.
    pub id: String,
    /// Collection-scoped display sequence.
    pub sequence: u32,
    /// Destination path relative to the repository root.
    pub relative_path: PathBuf,
}

impl NewDragon {
    /// Human reference for the created artifact, e.g. `dragon:2`.
    pub fn reference(&self) -> String {
        format!("dragon:{}", self.sequence)
    }
}

/// Create a new open dragon in the repository at `root`.
///
/// Allocates `max(existing sequence) + 1` across the open and closed dragon
/// directories, derives a deterministic kebab-case slug from `title`,
/// assigns a fresh prefixed ULID identity, and writes the Markdown template
/// through a temporary file with an atomic no-clobber persist. Neither
/// `.strata.toml` nor any existing artifact is modified.
pub fn create_dragon(root: &Path, title: &str) -> Result<NewDragon, Error> {
    let title = title.trim();
    let slug = slugify(title).ok_or_else(|| Error::InvalidInvocation {
        message: format!(
            "cannot create a dragon titled `{title}`: the title must contain \
             at least one ASCII letter or digit to derive a filename slug"
        ),
    })?;

    let sequence = next_sequence(root)?;
    let id = format!("{DRAGON_ID_PREFIX}{}", ulid::Ulid::new());
    let created = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();
    let content = render_dragon(&id, sequence, &created, title);

    let filename = format!("{sequence:04}-{slug}.md");
    write_new(&root.join(DRAGONS_OPEN_DIR), &filename, &content)?;

    Ok(NewDragon {
        id,
        sequence,
        relative_path: Path::new(DRAGONS_OPEN_DIR).join(filename),
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

/// Allocate the next display sequence by scanning both managed dragon
/// directories, refusing to exceed the four-digit space.
fn next_sequence(root: &Path) -> Result<u32, Error> {
    let mut max = 0u32;
    for dir in [DRAGONS_OPEN_DIR, DRAGONS_CLOSED_DIR] {
        max = max.max(max_sequence_in(&root.join(dir))?);
    }
    if max >= MAX_SEQUENCE {
        return Err(Error::ArtifactConflict {
            path: root.join(DRAGONS_OPEN_DIR),
            reason: format!(
                "the dragon collection has exhausted the four-digit display \
                 sequence space (last sequence is {MAX_SEQUENCE})"
            ),
        });
    }
    Ok(max + 1)
}

/// Largest display sequence among the artifacts in one managed directory.
///
/// Every non-hidden entry must be a valid dragon filename; malformed names
/// are a typed error naming the path, never silently skipped. Entries
/// starting with `.` (editor and VCS metadata, abandoned temporaries) are
/// not artifacts and are ignored.
fn max_sequence_in(dir: &Path) -> Result<u32, Error> {
    let entries = fs::read_dir(dir).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            Error::MalformedArtifact {
                path: dir.to_path_buf(),
                reason: "required dragon directory is missing; \
                         run `strata init` to restore the repository layout"
                    .into(),
            }
        } else {
            Error::Filesystem {
                operation: "read directory".into(),
                path: dir.to_path_buf(),
                source,
            }
        }
    })?;

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
            malformed(
                "dragon filenames must be `NNNN-slug.md` with a four-digit \
                 display sequence"
                    .into(),
            )
        })?;
        max = max.max(sequence);
    }
    Ok(max)
}

/// Extract the display sequence from a valid dragon filename
/// (`NNNN-slug.md`), or `None` when the name does not conform.
fn parse_sequence(name: &str) -> Option<u32> {
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

/// Render the dragon Markdown payload: front matter plus template sections.
fn render_dragon(id: &str, sequence: u32, created: &str, title: &str) -> String {
    format!(
        "---\n\
         id: {id}\n\
         sequence: {sequence}\n\
         kind: dragon\n\
         status: open\n\
         created: {created}\n\
         ---\n\
         \n\
         # {title}\n\
         \n\
         ## Context\n\
         \n\
         ## Question\n\
         \n\
         ## Constraints\n\
         \n\
         ## Resolution criteria\n"
    )
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

    fn temp_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create temporary directory");
        repo::init(tmp.path()).expect("initialize repository");
        tmp
    }

    fn open_dir_entries(root: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(root.join(DRAGONS_OPEN_DIR))
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
            Path::new(DRAGONS_OPEN_DIR).join("0001-branch-sequence-collisions.md")
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
    fn sequence_allocation_scans_open_and_closed_directories() {
        let tmp = temp_repo();
        fs::write(
            tmp.path().join(DRAGONS_OPEN_DIR).join("0001-old.md"),
            "seeded",
        )
        .unwrap();
        fs::write(
            tmp.path().join(DRAGONS_CLOSED_DIR).join("0005-resolved.md"),
            "seeded",
        )
        .unwrap();

        let dragon = create_dragon(tmp.path(), "Next risk").unwrap();

        assert_eq!(dragon.sequence, 6, "must continue after the closed maximum");
        assert!(
            tmp.path()
                .join(DRAGONS_OPEN_DIR)
                .join("0006-next-risk.md")
                .is_file()
        );
    }

    #[test]
    fn seeded_non_ulid_ids_are_left_untouched() {
        let tmp = temp_repo();
        let seeded_path = tmp.path().join(DRAGONS_OPEN_DIR).join("0001-seeded.md");
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
            let path = tmp.path().join(DRAGONS_OPEN_DIR).join(bad);
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
        fs::write(tmp.path().join(DRAGONS_OPEN_DIR).join(".gitkeep"), "").unwrap();

        let dragon = create_dragon(tmp.path(), "Ignores dotfiles").unwrap();

        assert_eq!(dragon.sequence, 1);
    }

    #[test]
    fn missing_managed_directory_is_a_typed_error() {
        let tmp = temp_repo();
        fs::remove_dir(tmp.path().join(DRAGONS_CLOSED_DIR)).unwrap();

        let err = create_dragon(tmp.path(), "Any title").unwrap_err();

        match err {
            Error::MalformedArtifact { path, reason } => {
                assert!(path.ends_with(DRAGONS_CLOSED_DIR), "{path:?}");
                assert!(reason.contains("strata init"), "{reason}");
            }
            other => panic!("expected malformed artifact, got {other:?}"),
        }
    }

    #[test]
    fn sequence_exhaustion_is_a_typed_error_not_a_five_digit_filename() {
        let tmp = temp_repo();
        fs::write(
            tmp.path().join(DRAGONS_CLOSED_DIR).join("9999-last.md"),
            "seeded",
        )
        .unwrap();

        let err = create_dragon(tmp.path(), "One too many").unwrap_err();

        assert!(matches!(err, Error::ArtifactConflict { .. }), "{err:?}");
        assert_eq!(
            open_dir_entries(tmp.path()),
            Vec::<String>::new(),
            "no artifact may be created on exhaustion"
        );
    }

    #[test]
    fn destination_conflict_refuses_overwrite_and_leaves_no_temporary() {
        let tmp = temp_repo();
        let dir = tmp.path().join(DRAGONS_OPEN_DIR);
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
            open_dir_entries(tmp.path()),
            vec!["0001-taken.md".to_string()],
            "a failed persist must not litter temporaries"
        );
    }

    #[cfg(unix)]
    #[test]
    fn induced_write_failure_leaves_no_partial_artifact() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = temp_repo();
        let dir = tmp.path().join(DRAGONS_OPEN_DIR);
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();

        let result = create_dragon(tmp.path(), "Cannot be written");

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            matches!(result, Err(Error::Filesystem { .. })),
            "{result:?}"
        );
        assert_eq!(
            open_dir_entries(tmp.path()),
            Vec::<String>::new(),
            "a failed creation must leave nothing behind"
        );
    }

    #[test]
    fn invalid_title_creates_nothing() {
        let tmp = temp_repo();

        let err = create_dragon(tmp.path(), "!!!").unwrap_err();

        assert!(matches!(err, Error::InvalidInvocation { .. }), "{err:?}");
        assert_eq!(open_dir_entries(tmp.path()), Vec::<String>::new());
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
