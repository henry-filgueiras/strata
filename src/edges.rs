//! Reference markers and typed edge validation, per decision 10
//! (`dec-reference-syntax`).
//!
//! A reference marker is wikilink-style, one grammar with two strictness
//! levels: the canonical bound form `[[stable-id|label]]` and the unbound
//! sugar form `[[kind:N]]` (label optional). Typed edges are front-matter
//! fields whose values are bound markers as quoted YAML strings; the
//! vocabulary is a closed allowlist ([`EDGE_KINDS`]) grown only by
//! decision, and keys outside it are inert data.
//!
//! Validation implements the decision's three tiers: corruption
//! (unparseable values, dangling bound targets, forbidden target kinds),
//! repairable (sugar values, lifecycle-contradicting edges), and advisory
//! (absence — deliberately not checked here; that promotion is idea 13's
//! question).
//!
//! The verification universe is every front-matter `id` in the archaeology
//! tree, managed or not, so edges may target decisions and tasks before
//! those collections are managed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::doctor::Severity;
use crate::read::{Status, Summary};

/// One parsed reference marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker<'a> {
    /// Canonical form: stable id plus mandatory frozen label.
    Bound { id: &'a str, label: &'a str },
    /// Sugar form: a `kind:N` reference, label optional. Legal but weak;
    /// repaired by an explicit bind.
    Sugar {
        reference: &'a str,
        label: Option<&'a str>,
    },
}

/// Parse one reference marker, or `None` when the text is not a marker.
///
/// Grammar per decision 10: `[[` target (`|` label)? `]]` on one line;
/// targets contain no whitespace, `|`, `#` (reserved for future
/// fragments), or `]]`; a target containing `:` is sugar, anything else is
/// a stable id; bound markers require a nonempty label, sugar may omit it.
pub fn parse_marker(text: &str) -> Option<Marker<'_>> {
    let inner = text.strip_prefix("[[")?.strip_suffix("]]")?;
    if inner.contains(']') || inner.contains('\n') {
        return None;
    }
    let (target, label) = match inner.split_once('|') {
        Some((target, label)) => (target, Some(label)),
        None => (inner, None),
    };
    if target.is_empty()
        || target.contains('#')
        || target.contains('|')
        || target.chars().any(char::is_whitespace)
    {
        return None;
    }
    if label.is_some_and(str::is_empty) {
        return None;
    }
    if target.contains(':') {
        Some(Marker::Sugar {
            reference: target,
            label,
        })
    } else {
        Some(Marker::Bound {
            id: target,
            label: label?,
        })
    }
}

/// One typed edge kind from the decided vocabulary.
#[derive(Debug)]
pub struct EdgeKind {
    /// Front-matter key.
    pub key: &'static str,
    /// The artifact kind this edge belongs on.
    pub source_kind: &'static str,
    /// The lifecycle state whose settlement the edge records; the edge on
    /// a source in any other state is a repairable contradiction.
    pub settled_status: Status,
    /// Artifact kinds a bound target may resolve to. Ideas are absent
    /// from every list: no typed edge may target one.
    pub target_kinds: &'static [&'static str],
}

/// The decided vocabulary: terminal-provenance edges only, each landed
/// with its first consumer.
pub const EDGE_KINDS: &[EdgeKind] = &[
    EdgeKind {
        key: "resolved-by",
        source_kind: "dragon",
        settled_status: Status::Closed,
        target_kinds: &["decision", "task"],
    },
    EdgeKind {
        key: "adopted-by",
        source_kind: "idea",
        settled_status: Status::Adopted,
        target_kinds: &["decision", "task"],
    },
];

/// One validation outcome for a typed edge, before doctor attaches the
/// artifact path.
#[derive(Debug)]
pub struct EdgeIssue {
    pub severity: Severity,
    pub problem: &'static str,
    pub detail: String,
}

fn issue(severity: Severity, problem: &'static str, detail: String) -> EdgeIssue {
    EdgeIssue {
        severity,
        problem,
        detail,
    }
}

/// Validate every typed edge on one parsed artifact against the
/// repository's id universe (id -> kind).
pub(crate) fn check_artifact(
    summary: &Summary,
    content: &str,
    universe: &BTreeMap<String, String>,
) -> Vec<EdgeIssue> {
    let mut issues = Vec::new();
    let Some((front_matter, _)) = crate::read::split_front_matter(content) else {
        return issues;
    };
    let Ok(mapping) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(front_matter) else {
        return issues;
    };

    for kind in EDGE_KINDS {
        let Some(value) = mapping.get(kind.key) else {
            continue;
        };
        if summary.kind != kind.source_kind {
            issues.push(issue(
                Severity::Error,
                "invalid-edge",
                format!(
                    "`{}` typed edges belong on {}s, but this artifact is a {}",
                    kind.key, kind.source_kind, summary.kind
                ),
            ));
            continue;
        }
        let markers: Vec<&serde_yaml_ng::Value> = match value {
            serde_yaml_ng::Value::Sequence(sequence) => sequence.iter().collect(),
            other => vec![other],
        };
        for marker_value in markers {
            let Some(text) = marker_value.as_str() else {
                issues.push(issue(
                    Severity::Error,
                    "invalid-edge",
                    format!(
                        "`{}` must be a reference marker as a quoted YAML \
                         string (or a sequence of them); unquoted `[[...]]` \
                         parses as a YAML list, not a marker",
                        kind.key
                    ),
                ));
                continue;
            };
            issues.extend(check_marker(kind, summary, text, universe));
        }
    }
    issues
}

/// Validate one marker string carried by a typed edge.
fn check_marker(
    kind: &EdgeKind,
    summary: &Summary,
    text: &str,
    universe: &BTreeMap<String, String>,
) -> Option<EdgeIssue> {
    match parse_marker(text) {
        None => Some(issue(
            Severity::Error,
            "invalid-edge",
            format!(
                "`{}` value `{text}` is not a reference marker; expected \
                 `[[stable-id|label]]`",
                kind.key
            ),
        )),
        Some(Marker::Sugar { reference, .. }) => Some(issue(
            Severity::Advice,
            "unbound-edge",
            format!(
                "`{}` holds the unbound sugar reference `{reference}`; bind \
                 it to a stable id so the edge is verifiable",
                kind.key
            ),
        )),
        Some(Marker::Bound { id, .. }) => match universe.get(id) {
            None => Some(issue(
                Severity::Error,
                "dangling-edge",
                format!(
                    "`{}` targets `{id}`, but no artifact in this repository \
                     carries that id",
                    kind.key
                ),
            )),
            Some(target_kind) if !kind.target_kinds.contains(&target_kind.as_str()) => Some(issue(
                Severity::Error,
                "invalid-edge",
                format!(
                    "`{}` targets `{id}`, a {target_kind}; legal targets \
                         are: {}",
                    kind.key,
                    kind.target_kinds.join(", ")
                ),
            )),
            Some(_) if summary.status != kind.settled_status => Some(issue(
                Severity::Advice,
                "stale-edge",
                format!(
                    "`{}` claims settlement, but this {} is {}, not {}; \
                     investigate or remove the edge",
                    kind.key, summary.kind, summary.status, kind.settled_status
                ),
            )),
            Some(_) => None,
        },
    }
}

/// Best-effort harvest of every front-matter `id` (mapped to its `kind`)
/// in the archaeology tree, managed or not.
///
/// Files without parseable front matter, non-Markdown files, and dot
/// entries are skipped silently: the universe answers "does this id
/// exist", not "is this file valid". Traversal is sorted so duplicate ids
/// resolve to the same first-seen kind deterministically; duplicate ids
/// among managed artifacts are separately real findings.
pub(crate) fn harvest_ids(root: &Path) -> BTreeMap<String, String> {
    let mut universe = BTreeMap::new();
    let mut stack = vec![root.join("archaeology")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
        paths.sort();
        for path in paths {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !name.ends_with(".md") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Some((front_matter, _)) = crate::read::split_front_matter(&content) else {
                continue;
            };
            let Ok(mapping) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(front_matter) else {
                continue;
            };
            if let (Some(id), Some(kind)) = (
                mapping.get("id").and_then(|v| v.as_str()),
                mapping.get("kind").and_then(|v| v.as_str()),
            ) {
                universe
                    .entry(id.to_string())
                    .or_insert_with(|| kind.to_string());
            }
        }
    }
    universe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_marker_accepts_bound_and_sugar_forms() {
        assert_eq!(
            parse_marker("[[dec-reference-syntax|reference syntax]]"),
            Some(Marker::Bound {
                id: "dec-reference-syntax",
                label: "reference syntax",
            })
        );
        assert_eq!(
            parse_marker("[[dragon:3]]"),
            Some(Marker::Sugar {
                reference: "dragon:3",
                label: None,
            })
        );
        assert_eq!(
            parse_marker("[[idea:12|relevance surfacing]]"),
            Some(Marker::Sugar {
                reference: "idea:12",
                label: Some("relevance surfacing"),
            })
        );
        // Labels may contain anything but `]]` and newlines.
        assert_eq!(
            parse_marker("[[drg_01KY|open <-> closed, and: more]]"),
            Some(Marker::Bound {
                id: "drg_01KY",
                label: "open <-> closed, and: more",
            })
        );
    }

    #[test]
    fn parse_marker_rejects_non_markers() {
        for bad in [
            "",
            "plain prose",
            "[[]]",
            "[[|label]]",
            "[[id|]]",
            "[[dec-x]]",             // bound requires a label
            "[[two words|label]]",   // whitespace in target
            "[[dec-x#fragment|l]]",  // `#` reserved for fragments
            "[[dec-x|a]]b]]",        // stray closing delimiter
            "[dec-x|label]",         // single brackets
            "[[dec-x|line\nbreak]]", // markers are one line
        ] {
            assert_eq!(parse_marker(bad), None, "must reject {bad:?}");
        }
    }

    #[test]
    fn vocabulary_never_admits_idea_targets() {
        for kind in EDGE_KINDS {
            assert!(
                !kind.target_kinds.contains(&"idea"),
                "`{}` must not target ideas",
                kind.key
            );
        }
    }
}
