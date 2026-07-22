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

/// Root-relative directory holding open dragon artifacts.
pub const DRAGONS_OPEN_DIR: &str = "archaeology/dragons/open";

/// Root-relative directory holding closed dragon artifacts.
pub const DRAGONS_CLOSED_DIR: &str = "archaeology/dragons/closed";

/// Root-relative directory holding parked idea artifacts.
pub const IDEAS_PARKED_DIR: &str = "archaeology/ideas/parked";

/// Root-relative directory holding adopted idea artifacts.
pub const IDEAS_ADOPTED_DIR: &str = "archaeology/ideas/adopted";

/// Root-relative directory holding rejected idea artifacts.
pub const IDEAS_REJECTED_DIR: &str = "archaeology/ideas/rejected";

/// Directories every bootstrap repository must contain. Idea lifecycle
/// directories are deliberately absent: they are created on first use, the
/// convention adopted after dragon 2 showed Git drops empty directories.
pub const REQUIRED_DIRS: [&str; 2] = [DRAGONS_OPEN_DIR, DRAGONS_CLOSED_DIR];

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
            let content = fs::read_to_string(&config_path).map_err(|source| Error::Filesystem {
                operation: "read".into(),
                path: config_path.clone(),
                source,
            })?;
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

    for dir in REQUIRED_DIRS {
        ensure_dir(root, dir, &mut report.created)?;
    }

    if config_missing {
        write_config(root, &config_path)?;
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
                let content =
                    fs::read_to_string(&config_path).map_err(|source| Error::Filesystem {
                        operation: "read".into(),
                        path: config_path.clone(),
                        source,
                    })?;
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

/// Write the config atomically: an exclusive temporary file in `root`,
/// persisted with a no-clobber rename. A failure at any point leaves no
/// `.strata.toml` behind; a config that appears concurrently is never
/// replaced.
fn write_config(root: &Path, config_path: &Path) -> Result<(), Error> {
    let mut tmp = tempfile::Builder::new()
        .prefix(".strata.toml.tmp")
        .tempfile_in(root)
        .map_err(|source| Error::Filesystem {
            operation: "create temporary config".into(),
            path: root.to_path_buf(),
            source,
        })?;
    tmp.write_all(CONFIG_TEMPLATE.as_bytes())
        .map_err(|source| Error::Filesystem {
            operation: "write temporary config".into(),
            path: tmp.path().to_path_buf(),
            source,
        })?;
    tmp.persist_noclobber(config_path).map_err(|err| {
        if err.error.kind() == io::ErrorKind::AlreadyExists {
            Error::ArtifactConflict {
                path: config_path.to_path_buf(),
                reason: "a config file appeared while initializing".into(),
            }
        } else {
            Error::Filesystem {
                operation: "persist config".into(),
                path: config_path.to_path_buf(),
                source: err.error,
            }
        }
    })?;
    Ok(())
}

fn file_kind(meta: &fs::Metadata) -> &'static str {
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
    fn existing_required_directories_are_accepted() {
        let tmp = temp_root();
        let root = tmp.path();
        fs::create_dir_all(root.join("archaeology/dragons/open")).unwrap();

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
        let nested = tmp.path().join("archaeology/dragons/open");

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
