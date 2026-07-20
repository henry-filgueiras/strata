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
        /// Collection for the new artifact (bootstrap supports `dragon`)
        collection: Collection,
        /// Human-readable title for the artifact
        title: String,
    },
    /// List the artifacts in a collection
    List {
        /// Collection to list (`dragon` or `dragons`)
        collection: Collection,
    },
    /// Show one artifact
    Show {
        /// Artifact reference in `collection:sequence` form, e.g. `dragon:7`
        reference: ArtifactRef,
    },
    /// Validate repository invariants and report corruption
    Doctor,
}

/// Artifact collections known to the bootstrap implementation.
///
/// Bootstrap hardcodes a single collection while the workflow is proven.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    Dragon,
}

impl Collection {
    /// Canonical singular name, as used in artifact references.
    pub fn name(self) -> &'static str {
        match self {
            Collection::Dragon => "dragon",
        }
    }
}

impl FromStr for Collection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dragon" | "dragons" => Ok(Collection::Dragon),
            other => Err(format!(
                "unknown collection `{other}`; bootstrap collections are: dragon"
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
    fn cli_definition_is_internally_consistent() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
