//! Repository layout, configuration, and initialization.
//!
//! A Strata repository root is marked by a `.strata.toml` config file and
//! contains the bootstrap directory layout. The filesystem is canonical:
//! nothing here creates hidden state, and an initialized repository remains
//! an ordinary directory tree.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::error::Error;

/// Repository marker and configuration file, at the repository root.
pub const CONFIG_FILE: &str = ".strata.toml";

/// Config contents written by `strata init`.
pub const CONFIG_TEMPLATE: &str = "version = 1\n";

/// The config schema version this build supports.
pub const SUPPORTED_VERSION: i64 = 1;

/// Root-relative path of the Git-attributes policy `strata init`
/// materializes. It lives inside `archaeology/` (decision 14 as
/// amended): the policy governs archaeology Markdown without annexing
/// the host repository's root Markdown, and a root `.gitattributes`
/// belongs to the host repository — init never creates, inspects,
/// merges, rejects, replaces, or deletes one.
pub const GITATTRIBUTES_FILE: &str = "archaeology/.gitattributes";

/// The line-ending policy written by `strata init` (decision 14 as
/// amended): LF-only for Markdown beneath `archaeology/`, enforced by
/// Git at checkout where Git is present. The artifact parser enforces
/// the same format without Git.
pub const GITATTRIBUTES_TEMPLATE: &str = "*.md text eol=lf\n";

/// Root-relative directory holding every dragon artifact, regardless of
/// lifecycle state (decision 11: placement is flat).
pub const DRAGONS_DIR: &str = "archaeology/dragons";

/// Root-relative directory holding every idea artifact, regardless of
/// lifecycle state (decision 11: placement is flat).
pub const IDEAS_DIR: &str = "archaeology/ideas";

/// Root-relative directory holding one containment directory per sprint.
/// Containment is not lifecycle (decision 11): a sprint's directory never
/// changes over its life, and its tasks file inside it.
pub const SPRINTS_DIR: &str = "archaeology/sprints";

/// The fixed filename of a sprint's own artifact inside its containment
/// directory.
pub const SPRINT_FILE: &str = "sprint.md";

/// Directories every bootstrap repository must contain. The ideas
/// directory is deliberately absent: it is created on first use, the
/// convention adopted after dragon 2 showed Git drops empty directories.
pub const REQUIRED_DIRS: [&str; 1] = [DRAGONS_DIR];

/// What an [`init`] invocation changed.
#[derive(Debug, Default)]
pub struct InitReport {
    /// Root-relative paths created by this invocation, in creation order.
    pub created: Vec<PathBuf>,
}

impl InitReport {
    /// True when the repository was already fully initialized.
    pub fn already_initialized(&self) -> bool {
        self.created.is_empty()
    }
}

/// Initialize a Strata repository at `root`.
///
/// Mutation-safety contract:
///
/// - An existing `.strata.toml` is never modified, truncated, or replaced.
///   It is accepted only as a regular file (not a symlink or directory)
///   containing a supported config; anything else is a typed error.
/// - Required directories are created when missing and accepted when
///   present; any non-directory object occupying a required path component
///   is a conflict.
/// - A missing `archaeology/.gitattributes` gains the decision 14
///   line-ending policy, written with the same atomic no-clobber
///   discipline as the config. An existing regular file there is
///   preserved byte-for-byte and never parsed: Strata cannot safely
///   infer or merge arbitrary Git-attribute policies, so the parser's
///   LF diagnosis remains the backstop. A non-regular object at that
///   managed path is a conflict. A root `.gitattributes` is outside the
///   init surface entirely — never created, inspected, or touched, even
///   when its contents disagree with Strata's policy. No Git executable
///   or `.git` directory is required.
/// - The config is written last, via an exclusive temporary file and an
///   atomic no-clobber persist: after a failed run `.strata.toml` either
///   does not exist or is complete — never partial or truncated.
/// - Directory creation is NOT transactional. A failed run may leave newly
///   created empty directories behind. That is the documented atomicity
///   boundary: empty directories are harmless and a re-run converges.
///   (Crash durability — fsync — is out of scope for bootstrap; the
///   guarantee covers process-level failures.)
pub fn init(root: &Path) -> Result<InitReport, Error> {
    let mut report = InitReport::default();

    let config_path = root.join(CONFIG_FILE);
    let config_missing = match fs::symlink_metadata(&config_path) {
        Ok(meta) if meta.is_file() => {
            let content = read_config(&config_path)?;
            validate_config(&content).map_err(|reason| Error::MalformedArtifact {
                path: config_path.clone(),
                reason,
            })?;
            false
        }
        Ok(meta) => {
            return Err(Error::ArtifactConflict {
                path: config_path,
                reason: format!(
                    "a {} occupies the config path, which must be a regular file",
                    file_kind(&meta)
                ),
            });
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => true,
        Err(source) => {
            return Err(Error::Filesystem {
                operation: "inspect".into(),
                path: config_path,
                source,
            });
        }
    };

    let attributes_path = root.join(GITATTRIBUTES_FILE);
    let attributes_missing = match fs::symlink_metadata(&attributes_path) {
        // An existing policy is preserved byte-for-byte, never parsed or
        // merged.
        Ok(meta) if meta.is_file() => false,
        Ok(meta) => {
            return Err(Error::ArtifactConflict {
                path: attributes_path,
                reason: format!(
                    "a {} occupies the `{GITATTRIBUTES_FILE}` policy path, \
                     which must be a regular file",
                    file_kind(&meta)
                ),
            });
        }
        // NotADirectory means a parent component is not a directory; the
        // nested path itself is then simply absent here, and the required
        // directory walk below reports the real conflict at the component.
        Err(err)
            if err.kind() == io::ErrorKind::NotFound
                || err.kind() == io::ErrorKind::NotADirectory =>
        {
            true
        }
        Err(source) => {
            return Err(Error::Filesystem {
                operation: "inspect".into(),
                path: attributes_path,
                source,
            });
        }
    };

    for dir in REQUIRED_DIRS {
        ensure_dir(root, dir, &mut report.created)?;
    }

    if attributes_missing {
        write_template(root, &attributes_path, GITATTRIBUTES_TEMPLATE)?;
        report.created.push(PathBuf::from(GITATTRIBUTES_FILE));
    }

    if config_missing {
        write_template(root, &config_path, CONFIG_TEMPLATE)?;
        report.created.push(PathBuf::from(CONFIG_FILE));
    }

    Ok(report)
}

/// Locate the repository root by walking upward from `start`.
///
/// Each directory from `start` to the filesystem root is checked for a
/// `.strata.toml` marker; the first directory containing a valid one is the
/// repository root. `start` should be an absolute path (such as the current
/// working directory) so the upward walk covers every ancestor.
///
/// A marker that exists but cannot be trusted stops the search with a typed
/// error instead of being silently walked past: continuing upward could
/// resolve to an unrelated outer repository and write artifacts to the wrong
/// tree. Unparseable or unsupported contents are `malformed-artifact`; a
/// non-regular file (directory, symlink) at the marker path is
/// `artifact-conflict`, matching how [`init`] classifies the same states.
pub fn discover(start: &Path) -> Result<PathBuf, Error> {
    let mut next = Some(start);
    while let Some(dir) = next {
        let config_path = dir.join(CONFIG_FILE);
        match fs::symlink_metadata(&config_path) {
            Ok(meta) if meta.is_file() => {
                let content = read_config(&config_path)?;
                validate_config(&content).map_err(|reason| Error::MalformedArtifact {
                    path: config_path,
                    reason,
                })?;
                return Ok(dir.to_path_buf());
            }
            Ok(meta) => {
                return Err(Error::ArtifactConflict {
                    path: config_path,
                    reason: format!(
                        "a {} occupies the repository marker path, which must be a regular file",
                        file_kind(&meta)
                    ),
                });
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(Error::Filesystem {
                    operation: "inspect".into(),
                    path: config_path,
                    source,
                });
            }
        }
        next = dir.parent();
    }
    Err(Error::MissingRepository {
        searched_from: start.to_path_buf(),
    })
}

/// Validate the contents of a `.strata.toml` config.
///
/// The config is ordinary TOML configuration, not a splice-mutated
/// artifact, so it sits outside decision 14's LF-only artifact-byte
/// contract (as amended): any line endings the TOML parser accepts —
/// including CRLF — are valid, and Strata never normalizes or rewrites
/// the file.
///
/// Accepts any TOML table whose `version` equals the supported integer.
/// Unknown keys are tolerated: keys added later within the same schema
/// version must not make older configs unreadable.
pub fn validate_config(content: &str) -> Result<(), String> {
    let table: toml::Table = content
        .parse()
        .map_err(|err| format!("not valid TOML: {err}"))?;
    match table.get("version") {
        None => Err("missing required `version` key".into()),
        Some(toml::Value::Integer(version)) if *version == SUPPORTED_VERSION => Ok(()),
        Some(toml::Value::Integer(version)) => Err(format!(
            "unsupported config version {version}; this build supports version {SUPPORTED_VERSION}"
        )),
        Some(other) => Err(format!(
            "`version` must be an integer, found {}",
            other.type_str()
        )),
    }
}

/// Walk `rel` component by component under `root`, creating missing
/// directories and refusing any component occupied by a non-directory.
///
/// Components are checked without following symlinks: a symlink where a
/// required directory belongs is a conflict, not a directory. Besides
/// `init`, writers use this to materialize managed directories on demand:
/// Git does not round-trip empty directories, so a cloned repository may
/// carry the marker without the layout.
pub(crate) fn ensure_dir(root: &Path, rel: &str, created: &mut Vec<PathBuf>) -> Result<(), Error> {
    let mut rel_path = PathBuf::new();
    for component in Path::new(rel).components() {
        rel_path.push(component);
        let path = root.join(&rel_path);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() => {}
            Ok(meta) => {
                return Err(Error::ArtifactConflict {
                    path,
                    reason: format!(
                        "a {} occupies the path of a required directory",
                        file_kind(&meta)
                    ),
                });
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&path).map_err(|source| Error::Filesystem {
                    operation: "create directory".into(),
                    path: path.clone(),
                    source,
                })?;
                created.push(rel_path.clone());
            }
            Err(source) => {
                return Err(Error::Filesystem {
                    operation: "inspect".into(),
                    path,
                    source,
                });
            }
        }
    }
    Ok(())
}

/// Write one init template atomically: an exclusive temporary file in
/// `root`, persisted with a no-clobber rename. A failure at any point
/// leaves nothing behind at `path`; a file that appears concurrently is
/// never replaced.
fn write_template(root: &Path, path: &Path, template: &str) -> Result<(), Error> {
    let mut tmp = tempfile::Builder::new()
        .prefix(".strata.init.tmp")
        .tempfile_in(root)
        .map_err(|source| Error::Filesystem {
            operation: "create temporary file".into(),
            path: root.to_path_buf(),
            source,
        })?;
    tmp.write_all(template.as_bytes())
        .map_err(|source| Error::Filesystem {
            operation: "write temporary file".into(),
            path: tmp.path().to_path_buf(),
            source,
        })?;
    tmp.persist_noclobber(path).map_err(|err| {
        if err.error.kind() == io::ErrorKind::AlreadyExists {
            Error::ArtifactConflict {
                path: path.to_path_buf(),
                reason: "a file appeared at this path while initializing".into(),
            }
        } else {
            Error::Filesystem {
                operation: "persist".into(),
                path: path.to_path_buf(),
                source: err.error,
            }
        }
    })?;
    Ok(())
}

/// Read the config marker through the bounded seam: the marker is
/// repository-controlled content like any artifact, so the task 22
/// per-file cap applies to it too. Callers have already classified the
/// path as a regular file with `symlink_metadata`.
fn read_config(path: &Path) -> Result<String, Error> {
    let bytes = crate::read::read_capped(path)
        .map_err(|source| Error::Filesystem {
            operation: "read".into(),
            path: path.to_path_buf(),
            source,
        })?
        .ok_or_else(|| Error::MalformedArtifact {
            path: path.to_path_buf(),
            reason: format!(
                "file exceeds the {}-byte per-file read limit",
                crate::read::MAX_ARTIFACT_BYTES
            ),
        })?;
    String::from_utf8(bytes).map_err(|_| Error::MalformedArtifact {
        path: path.to_path_buf(),
        reason: "contents are not valid UTF-8".into(),
    })
}

pub(crate) fn file_kind(meta: &fs::Metadata) -> &'static str {
    let file_type = meta.file_type();
    if file_type.is_dir() {
        "directory"
    } else if file_type.is_symlink() {
        "symbolic link"
    } else if file_type.is_file() {
        "regular file"
    } else {
        "non-regular file"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root() -> tempfile::TempDir {
        tempfile::tempdir().expect("create temporary directory")
    }

    #[test]
    fn init_creates_config_and_required_directories() {
        let tmp = temp_root();
        let root = tmp.path();

        let report = init(root).unwrap();

        assert_eq!(
            fs::read_to_string(root.join(CONFIG_FILE)).unwrap(),
            CONFIG_TEMPLATE
        );
        for dir in REQUIRED_DIRS {
            assert!(root.join(dir).is_dir(), "missing required directory {dir}");
        }
        assert!(!report.already_initialized());
        assert!(report.created.contains(&PathBuf::from(CONFIG_FILE)));
    }

    #[test]
    fn rerun_is_nondestructive_and_reports_nothing_created() {
        let tmp = temp_root();
        let root = tmp.path();

        init(root).unwrap();
        let report = init(root).unwrap();

        assert!(
            report.already_initialized(),
            "rerun created {:?}",
            report.created
        );
        assert_eq!(
            fs::read_to_string(root.join(CONFIG_FILE)).unwrap(),
            CONFIG_TEMPLATE
        );
    }

    #[test]
    fn existing_valid_config_is_preserved_byte_for_byte() {
        let tmp = temp_root();
        let root = tmp.path();
        let custom = "# hand-written\nversion = 1\nfuture_key = true\n";
        fs::write(root.join(CONFIG_FILE), custom).unwrap();

        let report = init(root).unwrap();

        assert_eq!(fs::read_to_string(root.join(CONFIG_FILE)).unwrap(), custom);
        assert!(!report.created.contains(&PathBuf::from(CONFIG_FILE)));
    }

    #[test]
    fn init_materializes_the_nested_line_ending_policy_only() {
        let tmp = temp_root();
        let root = tmp.path();

        let report = init(root).unwrap();

        assert_eq!(
            fs::read_to_string(root.join(GITATTRIBUTES_FILE)).unwrap(),
            GITATTRIBUTES_TEMPLATE
        );
        assert!(report.created.contains(&PathBuf::from(GITATTRIBUTES_FILE)));
        assert!(
            !root.join(".gitattributes").exists(),
            "a root .gitattributes belongs to the host repository and is \
             never created"
        );
    }

    #[test]
    fn existing_gitattributes_is_preserved_byte_for_byte_and_never_parsed() {
        let tmp = temp_root();
        let root = tmp.path();
        // Not even valid attribute syntax: preservation must not depend on
        // Strata understanding the policy.
        let custom = "* text=auto\n<<not attribute syntax>>\n";
        fs::create_dir_all(root.join("archaeology")).unwrap();
        fs::write(root.join(GITATTRIBUTES_FILE), custom).unwrap();

        let report = init(root).unwrap();

        assert_eq!(
            fs::read_to_string(root.join(GITATTRIBUTES_FILE)).unwrap(),
            custom
        );
        assert!(!report.created.contains(&PathBuf::from(GITATTRIBUTES_FILE)));
    }

    #[test]
    fn root_gitattributes_is_ignored_and_untouched_even_when_it_disagrees() {
        let tmp = temp_root();
        let root = tmp.path();
        // A host policy that contradicts Strata's: still not init's to
        // inspect, merge, reject, replace, or delete.
        let host_policy = "*.md text eol=crlf\n";
        fs::write(root.join(".gitattributes"), host_policy).unwrap();

        let report = init(root).unwrap();

        assert_eq!(
            fs::read_to_string(root.join(".gitattributes")).unwrap(),
            host_policy,
            "the host repository's root policy must survive byte-identical"
        );
        assert!(
            !report.created.contains(&PathBuf::from(".gitattributes")),
            "the root path is outside the init surface"
        );
        assert_eq!(
            fs::read_to_string(root.join(GITATTRIBUTES_FILE)).unwrap(),
            GITATTRIBUTES_TEMPLATE,
            "the nested policy is still materialized"
        );
    }

    #[test]
    fn gitattributes_path_occupied_by_directory_is_a_conflict() {
        let tmp = temp_root();
        let root = tmp.path();
        fs::create_dir_all(root.join(GITATTRIBUTES_FILE)).unwrap();

        let err = init(root).unwrap_err();

        assert!(matches!(err, Error::ArtifactConflict { .. }), "{err:?}");
        assert!(
            root.join(GITATTRIBUTES_FILE).is_dir(),
            "conflicting object must survive"
        );
        assert!(
            !root.join(CONFIG_FILE).exists(),
            "config must not be written when initialization fails"
        );
    }

    #[test]
    fn initialized_repository_missing_gitattributes_gains_it_on_rerun() {
        let tmp = temp_root();
        let root = tmp.path();
        init(root).unwrap();
        fs::remove_file(root.join(GITATTRIBUTES_FILE)).unwrap();

        let report = init(root).unwrap();

        assert_eq!(
            fs::read_to_string(root.join(GITATTRIBUTES_FILE)).unwrap(),
            GITATTRIBUTES_TEMPLATE
        );
        assert_eq!(report.created, vec![PathBuf::from(GITATTRIBUTES_FILE)]);
    }

    #[test]
    fn existing_required_directories_are_accepted() {
        let tmp = temp_root();
        let root = tmp.path();
        fs::create_dir_all(root.join("archaeology/dragons")).unwrap();

        init(root).unwrap();

        for dir in REQUIRED_DIRS {
            assert!(root.join(dir).is_dir());
        }
    }

    #[test]
    fn unsupported_config_version_is_malformed_and_untouched() {
        let tmp = temp_root();
        let root = tmp.path();
        let content = "version = 2\n";
        fs::write(root.join(CONFIG_FILE), content).unwrap();

        let err = init(root).unwrap_err();

        assert!(matches!(err, Error::MalformedArtifact { .. }), "{err:?}");
        assert_eq!(fs::read_to_string(root.join(CONFIG_FILE)).unwrap(), content);
    }

    #[test]
    fn unparseable_config_is_malformed_and_untouched() {
        let tmp = temp_root();
        let root = tmp.path();
        let content = "version = [not toml";
        fs::write(root.join(CONFIG_FILE), content).unwrap();

        let err = init(root).unwrap_err();

        assert!(matches!(err, Error::MalformedArtifact { .. }), "{err:?}");
        assert_eq!(fs::read_to_string(root.join(CONFIG_FILE)).unwrap(), content);
    }

    #[test]
    fn config_path_occupied_by_directory_is_a_conflict() {
        let tmp = temp_root();
        let root = tmp.path();
        fs::create_dir(root.join(CONFIG_FILE)).unwrap();

        let err = init(root).unwrap_err();

        assert!(matches!(err, Error::ArtifactConflict { .. }), "{err:?}");
        assert!(
            root.join(CONFIG_FILE).is_dir(),
            "conflicting object must survive"
        );
    }

    #[cfg(unix)]
    #[test]
    fn config_path_occupied_by_symlink_is_a_conflict() {
        let tmp = temp_root();
        let root = tmp.path();
        fs::write(root.join("real.toml"), CONFIG_TEMPLATE).unwrap();
        std::os::unix::fs::symlink(root.join("real.toml"), root.join(CONFIG_FILE)).unwrap();

        let err = init(root).unwrap_err();

        assert!(matches!(err, Error::ArtifactConflict { .. }), "{err:?}");
    }

    #[test]
    fn required_directory_path_occupied_by_file_is_a_conflict() {
        let tmp = temp_root();
        let root = tmp.path();
        fs::create_dir(root.join("archaeology")).unwrap();
        fs::write(root.join("archaeology/dragons"), "not a directory").unwrap();

        let err = init(root).unwrap_err();

        match &err {
            Error::ArtifactConflict { path, .. } => {
                assert!(path.ends_with("archaeology/dragons"), "{err:?}");
            }
            other => panic!("expected conflict, got {other:?}"),
        }
        assert_eq!(
            fs::read_to_string(root.join("archaeology/dragons")).unwrap(),
            "not a directory",
            "conflicting file must survive untouched"
        );
        assert!(
            !root.join(CONFIG_FILE).exists(),
            "config must not be written when initialization fails"
        );
    }

    #[cfg(unix)]
    #[test]
    fn failed_config_write_leaves_no_partial_file() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = temp_root();
        let root = tmp.path();
        // Pre-create the directories so only the config write remains, then
        // make the root read-only to naturally fail that write.
        for dir in REQUIRED_DIRS {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        fs::set_permissions(root, fs::Permissions::from_mode(0o555)).unwrap();

        let result = init(root);

        fs::set_permissions(root, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            matches!(result, Err(Error::Filesystem { .. })),
            "{result:?}"
        );
        assert!(
            !root.join(CONFIG_FILE).exists(),
            "a failed init must leave no partial config"
        );
    }

    #[test]
    fn discover_finds_root_from_the_root_itself() {
        let tmp = temp_root();
        init(tmp.path()).unwrap();

        let root = discover(tmp.path()).unwrap();

        assert_eq!(root, tmp.path());
    }

    #[test]
    fn discover_finds_root_from_a_nested_directory() {
        let tmp = temp_root();
        init(tmp.path()).unwrap();
        let nested = tmp.path().join("archaeology/dragons");

        let root = discover(&nested).unwrap();

        assert_eq!(root, tmp.path());
    }

    #[test]
    fn discover_reports_missing_repository_with_the_search_origin() {
        let tmp = temp_root();
        let nested = tmp.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();

        let err = discover(&nested).unwrap_err();

        match err {
            Error::MissingRepository { searched_from } => assert_eq!(searched_from, nested),
            other => panic!("expected missing repository, got {other:?}"),
        }
    }

    #[test]
    fn discover_rejects_malformed_marker_instead_of_walking_past_it() {
        let tmp = temp_root();
        // A valid repository above a directory carrying a broken marker: the
        // walk must stop at the broken marker, not resolve the outer root.
        init(tmp.path()).unwrap();
        let inner = tmp.path().join("vendored");
        fs::create_dir(&inner).unwrap();
        fs::write(inner.join(CONFIG_FILE), "version = [broken").unwrap();
        let nested = inner.join("deep");
        fs::create_dir(&nested).unwrap();

        let err = discover(&nested).unwrap_err();

        match err {
            Error::MalformedArtifact { path, .. } => assert_eq!(path, inner.join(CONFIG_FILE)),
            other => panic!("expected malformed artifact, got {other:?}"),
        }
    }

    #[test]
    fn discover_rejects_directory_at_marker_path() {
        let tmp = temp_root();
        fs::create_dir(tmp.path().join(CONFIG_FILE)).unwrap();

        let err = discover(tmp.path()).unwrap_err();

        assert!(matches!(err, Error::ArtifactConflict { .. }), "{err:?}");
    }

    #[test]
    fn validate_config_accepts_supported_version_with_unknown_keys() {
        assert_eq!(validate_config("version = 1"), Ok(()));
        assert_eq!(validate_config("version = 1\nextra = \"ok\"\n"), Ok(()));
    }

    #[test]
    fn validate_config_accepts_crlf_line_endings() {
        // The config is ordinary TOML outside the artifact-byte contract
        // (decision 14 as amended): whatever the TOML parser accepts is
        // valid.
        assert_eq!(validate_config("version = 1\r\n"), Ok(()));
        assert_eq!(
            validate_config("# annotated\r\nversion = 1\r\nextra = \"ok\"\r\n"),
            Ok(())
        );
    }

    #[test]
    fn init_preserves_an_existing_valid_crlf_config_byte_for_byte() {
        let tmp = temp_root();
        let root = tmp.path();
        let crlf = "# hand-written\r\nversion = 1\r\n";
        fs::write(root.join(CONFIG_FILE), crlf).unwrap();

        let report = init(root).unwrap();

        assert_eq!(fs::read_to_string(root.join(CONFIG_FILE)).unwrap(), crlf);
        assert!(!report.created.contains(&PathBuf::from(CONFIG_FILE)));
    }

    #[test]
    fn validate_config_rejects_missing_wrong_type_and_unsupported_version() {
        assert!(validate_config("").unwrap_err().contains("missing"));
        assert!(
            validate_config("version = \"1\"")
                .unwrap_err()
                .contains("integer")
        );
        assert!(
            validate_config("version = 2")
                .unwrap_err()
                .contains("unsupported config version 2")
        );
    }
}
