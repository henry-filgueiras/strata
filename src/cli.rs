//! The Strata command-line surface.
//!
//! Commands express user intent (create, list, inspect, validate); the
//! mechanics of numbering, slugging, identity, and safe writes belong to the
//! operations behind them, not to callers.

use std::fmt;
use std::str::FromStr;

use clap::{Parser, Subcommand};

/// Git-friendly project archaeology and repository-local memory.
#[derive(Debug, Parser)]
#[command(name = "strata", version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// The bootstrap command surface.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a Strata repository in the current directory
    Init,
    /// Create an artifact; Strata assigns its sequence, slug, and identity
    New {
        /// Collection for the new artifact (`dragon`, `idea`, `sprint`, or
        /// `task`; tasks are created in an active sprint, chosen with
        /// `--sprint` when several are active)
        collection: Collection,
        /// Human-readable title for the artifact
        title: String,
        /// Tasks only: the owning sprint, as `sprint:N` or a sprint's
        /// stable id; required when more than one sprint is active
        #[arg(long)]
        sprint: Option<String>,
        /// Emit a deterministic JSON object describing the created
        /// artifact instead of the human-readable line
        #[arg(long)]
        json: bool,
    },
    /// List the artifacts in a collection
    List {
        /// Collection to list (`dragons`, `ideas`, `sprints`, or `tasks`)
        collection: Collection,
        /// Emit a deterministic JSON array instead of human-readable lines
        #[arg(long)]
        json: bool,
        /// Tasks only: list only the tasks owned by active sprints
        #[arg(long)]
        active: bool,
    },
    /// Show one artifact
    Show {
        /// `collection:sequence` reference (e.g. `dragon:7`) or a stable
        /// artifact `id`
        reference: ArtifactTarget,
        /// Emit a deterministic JSON object instead of the raw artifact
        #[arg(long)]
        json: bool,
    },
    /// Validate repository invariants and report corruption
    Doctor {
        /// Emit findings as a deterministic JSON array instead of
        /// human-readable lines
        #[arg(long)]
        json: bool,
    },
    /// Close an open dragon, an active sprint, or a pending task
    Close {
        /// `dragon:sequence`, `sprint:sequence`, or `task:sequence`
        /// reference, or a stable artifact `id`
        reference: ArtifactTarget,
        /// Dragons only: record what resolved the dragon (`decision:N`,
        /// `task:N`, or a stable id) as a `resolved-by` edge in the same
        /// write
        #[arg(long)]
        resolved_by: Option<String>,
    },
    /// Reopen a closed dragon
    Reopen {
        /// `dragon:sequence` reference (e.g. `dragon:7`) or a stable
        /// artifact `id`
        reference: ArtifactTarget,
    },
    /// Adopt a parked idea into its terminal adopted state
    Adopt {
        /// `idea:sequence` reference (e.g. `idea:12`) or a stable
        /// artifact `id`
        reference: ArtifactTarget,
        /// Record what adopted the idea (`decision:N`, `task:N`, or a
        /// stable id) as an `adopted-by` edge in the same write
        #[arg(long)]
        adopted_by: Option<String>,
    },
    /// Reject a parked idea into its terminal rejected state
    Reject {
        /// `idea:sequence` reference (e.g. `idea:12`) or a stable
        /// artifact `id`
        reference: ArtifactTarget,
    },
    /// Surface one open dragon or parked idea at random, favoring stale
    /// artifacts
    Fortune,
}

/// Artifact collections known to this implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    Dragon,
    Idea,
    Sprint,
    Task,
}

impl Collection {
    /// Canonical singular name, as used in artifact references.
    pub fn name(self) -> &'static str {
        match self {
            Collection::Dragon => "dragon",
            Collection::Idea => "idea",
            Collection::Sprint => "sprint",
            Collection::Task => "task",
        }
    }
}

impl FromStr for Collection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dragon" | "dragons" => Ok(Collection::Dragon),
            "idea" | "ideas" => Ok(Collection::Idea),
            "sprint" | "sprints" => Ok(Collection::Sprint),
            "task" | "tasks" => Ok(Collection::Task),
            other => Err(format!(
                "unknown collection `{other}`; collections are: dragon, idea, \
                 sprint, task"
            )),
        }
    }
}

impl fmt::Display for Collection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A human-facing artifact reference such as `dragon:7`.
///
/// References use the collection-scoped display sequence, not the stable
/// identity; sequences start at 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactRef {
    pub collection: Collection,
    pub sequence: u32,
}

impl FromStr for ArtifactRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (collection, sequence) = s.split_once(':').ok_or_else(|| {
            format!(
                "invalid artifact reference `{s}`; expected `collection:sequence`, e.g. `dragon:7`"
            )
        })?;
        let collection = collection.parse::<Collection>()?;
        let sequence = sequence.parse::<u32>().map_err(|_| {
            format!(
                "invalid sequence `{sequence}` in artifact reference `{s}`; \
                 expected a positive integer, e.g. `dragon:7`"
            )
        })?;
        if sequence == 0 {
            return Err(format!(
                "invalid sequence `0` in artifact reference `{s}`; sequences start at 1"
            ));
        }
        Ok(ArtifactRef {
            collection,
            sequence,
        })
    }
}

impl fmt::Display for ArtifactRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.collection, self.sequence)
    }
}

/// An artifact named on the command line, as `show`, `close`, and `reopen`
/// accept it.
///
/// A reference containing `:` is a human [`ArtifactRef`] and must parse as
/// one; anything else is a stable opaque identity, passed through verbatim.
/// IDs are opaque strings — nothing here may assume ULID structure — but a
/// stable-id *address* must satisfy the decision 12 addressability
/// contract: non-empty and free of `:`, whitespace, `#`, `|`, and `]`.
/// Ids outside that subset remain valid identities; they are refused here,
/// naming the offending character class, because no command surface can
/// reach them by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactTarget {
    Reference(ArtifactRef),
    Id(String),
}

impl FromStr for ArtifactTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(':') {
            return s.parse::<ArtifactRef>().map(ArtifactTarget::Reference);
        }
        if s.is_empty() {
            return Err("empty artifact reference; expected `collection:sequence` \
                 (e.g. `dragon:7`) or a stable artifact `id`"
                .into());
        }
        if let Err(violation) = crate::edges::addressable(s) {
            return Err(format!(
                "artifact id `{s}` {}, so it is not addressable as a \
                 stable-id reference; repair the artifact's `id` by hand or \
                 refer to the artifact by `collection:sequence`",
                violation.describe()
            ));
        }
        Ok(ArtifactTarget::Id(s.to_string()))
    }
}

impl fmt::Display for ArtifactTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArtifactTarget::Reference(reference) => reference.fmt(f),
            ArtifactTarget::Id(id) => f.write_str(id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_accepts_singular_and_plural() {
        assert_eq!("dragon".parse::<Collection>(), Ok(Collection::Dragon));
        assert_eq!("dragons".parse::<Collection>(), Ok(Collection::Dragon));
    }

    #[test]
    fn collection_rejects_unknown_names_with_guidance() {
        let err = "widget".parse::<Collection>().unwrap_err();
        assert!(err.contains("widget"), "error should name the input: {err}");
        assert!(
            err.contains("dragon"),
            "error should list valid collections: {err}"
        );
    }

    #[test]
    fn artifact_ref_parses_canonical_form() {
        let reference: ArtifactRef = "dragon:7".parse().unwrap();
        assert_eq!(reference.collection, Collection::Dragon);
        assert_eq!(reference.sequence, 7);
    }

    #[test]
    fn artifact_ref_accepts_zero_padded_sequences() {
        let reference: ArtifactRef = "dragon:0007".parse().unwrap();
        assert_eq!(reference.sequence, 7);
    }

    #[test]
    fn artifact_ref_rejects_missing_separator() {
        let err = "dragon7".parse::<ArtifactRef>().unwrap_err();
        assert!(err.contains("collection:sequence"), "{err}");
    }

    #[test]
    fn artifact_ref_rejects_non_numeric_sequence() {
        let err = "dragon:seven".parse::<ArtifactRef>().unwrap_err();
        assert!(err.contains("positive integer"), "{err}");
    }

    #[test]
    fn artifact_ref_rejects_zero_sequence() {
        let err = "dragon:0".parse::<ArtifactRef>().unwrap_err();
        assert!(err.contains("start at 1"), "{err}");
    }

    #[test]
    fn artifact_ref_displays_canonical_singular_form() {
        let reference: ArtifactRef = "dragons:0003".parse().unwrap();
        assert_eq!(reference.to_string(), "dragon:3");
    }

    #[test]
    fn show_target_parses_human_references_and_opaque_ids() {
        assert_eq!(
            "dragon:7".parse::<ArtifactTarget>(),
            Ok(ArtifactTarget::Reference(ArtifactRef {
                collection: Collection::Dragon,
                sequence: 7,
            }))
        );
        for id in [
            "drg_01K0P6W5PK8T19H7M2V8W6YQ4C",
            "drg-bootstrap-branch-collisions",
        ] {
            assert_eq!(
                id.parse::<ArtifactTarget>(),
                Ok(ArtifactTarget::Id(id.into()))
            );
        }
    }

    #[test]
    fn show_target_rejects_invalid_human_references_and_empty_ids() {
        let err = "dragon:seven".parse::<ArtifactTarget>().unwrap_err();
        assert!(err.contains("positive integer"), "{err}");
        let err = "widget:1".parse::<ArtifactTarget>().unwrap_err();
        assert!(err.contains("widget"), "{err}");
        let err = "".parse::<ArtifactTarget>().unwrap_err();
        assert!(err.contains("empty artifact reference"), "{err}");
    }

    #[test]
    fn unaddressable_bare_ids_are_refused_naming_the_class() {
        // Decision 12: a `:`-bearing id is captured by the `kind:N`
        // grammar (so it can never reach the id arm), and the other
        // excluded classes are refused explicitly.
        let err = "drg:odd".parse::<ArtifactTarget>().unwrap_err();
        assert!(err.contains("unknown collection `drg`"), "{err}");
        for (id, needle) in [
            ("dec spacey", "whitespace"),
            ("dec#frag", "`#`"),
            ("dec|pipe", "`|`"),
            ("dec]close", "`]`"),
        ] {
            let err = id.parse::<ArtifactTarget>().unwrap_err();
            assert!(err.contains(needle), "for {id:?}: {err}");
            assert!(err.contains("not addressable"), "for {id:?}: {err}");
        }
    }

    #[test]
    fn cli_definition_is_internally_consistent() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
