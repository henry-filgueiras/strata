//! Typed errors for Strata operations.
//!
//! Automated callers must never parse prose. Two stable machine contracts
//! exist:
//!
//! - the process exit code, per [`Error::exit_code`];
//! - the leading `error[<code>]:` token on the first stderr line, where
//!   `<code>` is the stable identifier from [`Error::code`].
//!
//! Message text after the colon is human-oriented and may change freely.
//! Invalid invocations rejected by `clap` itself (unknown flags, missing
//! arguments, unparseable values) also exit with code 2, so the invalid
//! invocation category is uniform whether the rejection happens during
//! parsing or afterwards.

use std::io;
use std::path::PathBuf;

/// An error from a Strata operation, categorized for machines and explained
/// for humans.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The invocation parsed but is semantically invalid for this repository
    /// or operation.
    #[error("{message}")]
    InvalidInvocation { message: String },

    /// No Strata repository exists at or above the working directory.
    #[error(
        "no Strata repository found at or above `{searched_from}`; \
         run `strata init` in the intended repository root"
    )]
    MissingRepository { searched_from: PathBuf },

    /// Completing the operation would overwrite or collide with an existing
    /// artifact.
    #[error(
        "artifact conflict at `{path}`: {reason}; \
         Strata never overwrites existing artifacts — \
         resolve the conflicting file, then retry"
    )]
    ArtifactConflict { path: PathBuf, reason: String },

    /// An artifact exists but cannot be parsed as a valid Strata artifact.
    #[error(
        "malformed artifact `{path}`: {reason}; \
         repair the file by hand or run `strata doctor` for a full report"
    )]
    MalformedArtifact { path: PathBuf, reason: String },

    /// An underlying filesystem operation failed.
    #[error("filesystem {operation} failed for `{path}`: {source}")]
    Filesystem {
        operation: String,
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// No managed artifact matches the requested reference.
    #[error(
        "no artifact matches `{reference}`; \
         run `strata list dragons` to see the artifacts Strata manages"
    )]
    ArtifactNotFound { reference: String },

    /// A reference that must name exactly one artifact matches several.
    #[error(
        "reference `{reference}` is ambiguous: it matches {}; \
         refer to the artifact by the stable `id` in its front matter, \
         or repair the duplicated metadata by hand",
        .candidates.join(", ")
    )]
    AmbiguousReference {
        reference: String,
        /// Repository-relative paths of every matching artifact.
        candidates: Vec<String>,
    },

    /// `strata doctor` completed its scan and the repository has validation
    /// findings. The findings themselves are the stdout payload; this error
    /// is the machine-readable summary on stderr.
    #[error(
        "repository validation found {problems} problem(s); \
         the report on stdout names each affected path — \
         repair the files by hand, then re-run `strata doctor`"
    )]
    UnhealthyRepository { problems: usize },
}

impl Error {
    /// Stable machine-readable identifier for this error category.
    pub fn code(&self) -> &'static str {
        match self {
            Error::InvalidInvocation { .. } => "invalid-invocation",
            Error::MissingRepository { .. } => "missing-repository",
            Error::ArtifactConflict { .. } => "artifact-conflict",
            Error::MalformedArtifact { .. } => "malformed-artifact",
            Error::Filesystem { .. } => "filesystem-failure",
            Error::ArtifactNotFound { .. } => "artifact-not-found",
            Error::AmbiguousReference { .. } => "ambiguous-reference",
            Error::UnhealthyRepository { .. } => "unhealthy-repository",
        }
    }

    /// Stable process exit code for this error category.
    ///
    /// - 1: reserved for general failure (previously the transitional
    ///   `unimplemented` category, retired when the last stub gained
    ///   behavior; not reused)
    /// - 2: invalid invocation (matches `clap` usage errors)
    /// - 3: missing repository
    /// - 4: artifact conflict
    /// - 5: malformed artifact
    /// - 6: filesystem failure
    /// - 7: artifact not found
    /// - 8: ambiguous reference
    /// - 9: unhealthy repository (`doctor` found validation problems)
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::InvalidInvocation { .. } => 2,
            Error::MissingRepository { .. } => 3,
            Error::ArtifactConflict { .. } => 4,
            Error::MalformedArtifact { .. } => 5,
            Error::Filesystem { .. } => 6,
            Error::ArtifactNotFound { .. } => 7,
            Error::AmbiguousReference { .. } => 8,
            Error::UnhealthyRepository { .. } => 9,
        }
    }

    /// The stderr rendering: a stable `error[<code>]:` token followed by the
    /// human-oriented message.
    pub fn render(&self) -> String {
        format!("error[{}]: {self}", self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_of_each() -> Vec<Error> {
        vec![
            Error::InvalidInvocation {
                message: "cannot combine those options".into(),
            },
            Error::MissingRepository {
                searched_from: PathBuf::from("/work/project"),
            },
            Error::ArtifactConflict {
                path: PathBuf::from("archaeology/dragons/open/0002-x.md"),
                reason: "a file with sequence 2 already exists".into(),
            },
            Error::MalformedArtifact {
                path: PathBuf::from("archaeology/dragons/open/0001-y.md"),
                reason: "missing `id` in front matter".into(),
            },
            Error::Filesystem {
                operation: "rename".into(),
                path: PathBuf::from("archaeology/dragons/open/0003-z.md"),
                source: io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"),
            },
            Error::ArtifactNotFound {
                reference: "dragon:41".into(),
            },
            Error::AmbiguousReference {
                reference: "dragon:2".into(),
                candidates: vec![
                    "archaeology/dragons/open/0002-a.md".into(),
                    "archaeology/dragons/closed/0002-b.md".into(),
                ],
            },
            Error::UnhealthyRepository { problems: 3 },
        ]
    }

    #[test]
    fn error_codes_are_distinct() {
        let errors = one_of_each();
        let mut codes: Vec<_> = errors.iter().map(Error::code).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(
            codes.len(),
            errors.len(),
            "duplicate error codes: {codes:?}"
        );
    }

    #[test]
    fn exit_codes_match_documented_contract() {
        let expected = [
            ("invalid-invocation", 2),
            ("missing-repository", 3),
            ("artifact-conflict", 4),
            ("malformed-artifact", 5),
            ("filesystem-failure", 6),
            ("artifact-not-found", 7),
            ("ambiguous-reference", 8),
            ("unhealthy-repository", 9),
        ];
        for error in one_of_each() {
            let want = expected
                .iter()
                .find(|(code, _)| *code == error.code())
                .map(|(_, exit)| *exit)
                .expect("every variant has a documented exit code");
            assert_eq!(
                error.exit_code(),
                want,
                "wrong exit code for {}",
                error.code()
            );
        }
    }

    #[test]
    fn render_leads_with_stable_machine_token() {
        for error in one_of_each() {
            let rendered = error.render();
            let prefix = format!("error[{}]: ", error.code());
            assert!(
                rendered.starts_with(&prefix),
                "rendering must lead with `{prefix}`: {rendered}"
            );
        }
    }

    #[test]
    fn messages_name_the_path_involved() {
        for error in one_of_each() {
            let message = error.to_string();
            match &error {
                Error::MissingRepository {
                    searched_from: path,
                }
                | Error::ArtifactConflict { path, .. }
                | Error::MalformedArtifact { path, .. }
                | Error::Filesystem { path, .. } => {
                    assert!(
                        message.contains(&path.display().to_string()),
                        "message must name `{}`: {message}",
                        path.display()
                    );
                }
                Error::AmbiguousReference { candidates, .. } => {
                    for candidate in candidates {
                        assert!(
                            message.contains(candidate),
                            "message must name every candidate `{candidate}`: {message}"
                        );
                    }
                }
                Error::InvalidInvocation { .. }
                | Error::ArtifactNotFound { .. }
                | Error::UnhealthyRepository { .. } => {}
            }
        }
    }

    #[test]
    fn messages_suggest_a_next_step() {
        let missing = Error::MissingRepository {
            searched_from: PathBuf::from("/work/project"),
        };
        assert!(missing.to_string().contains("strata init"));

        let malformed = Error::MalformedArtifact {
            path: PathBuf::from("a.md"),
            reason: "bad front matter".into(),
        };
        assert!(malformed.to_string().contains("strata doctor"));
    }
}
