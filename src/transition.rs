//! Lifecycle transitions for managed collections.
//!
//! A transition moves an artifact between lifecycle directories and rewrites
//! exactly its front-matter `status` value; every other byte is preserved.
//! The mutation contract is decision 8 (`dec-mutation-failure-classes`),
//! executed as two atomic steps with the content rewrite as the commit
//! point:
//!
//! 1. stage the full payload with the new `status` in a temporary beside
//!    the source and atomically replace the source;
//! 2. atomically rename the source into the target lifecycle directory.
//!
//! Placement follows status: an interrupted transition reads as "committed
//! but not yet filed", `doctor` diagnoses the resulting status/placement
//! mismatch precisely, and the front matter is authoritative for repair.
//! When step 2 fails, the command rolls step 1 back; only a doubly-failed
//! rollback leaves the mismatch behind, reported as the dedicated
//! `transition-interrupted` error naming that state.
//!
//! # Concurrency boundary
//!
//! The destination no-clobber guarantee is check-then-rename, matching
//! creation's scan-then-write posture: bootstrap does not linearize
//! concurrent Strata processes, and a file appearing in the destination
//! between the check and the rename is outside the contract. Within one
//! process the artifact exists at exactly one path at every instant.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::error::Error;
use crate::read::{self, Artifact, Collection, Selector, Status};

/// A successfully performed lifecycle transition.
#[derive(Debug)]
pub struct Transition {
    /// Human reference of the artifact, e.g. `dragon:2`.
    pub reference: String,
    /// Lifecycle state before the transition.
    pub from: Status,
    /// Lifecycle state after the transition.
    pub to: Status,
    /// Repository-relative destination path with `/` separators.
    pub to_path: String,
}

/// Transition one artifact of `collection` to `to`, resolving `target` like
/// `strata show`.
///
/// Resolution reuses the strict read pipeline: an artifact whose status and
/// placement already disagree fails the scan as `malformed-artifact` naming
/// the lifecycle mismatch and directing the user to `doctor` — a transition
/// never silently repairs, and a re-run after an interrupted transition
/// refuses the same way. A transition the collection's lifecycle does not
/// define (such as un-adopting an idea) is an invalid invocation.
pub fn transition(
    root: &Path,
    collection: &Collection,
    target: Selector<'_>,
    display: &str,
    to: Status,
) -> Result<Transition, Error> {
    let artifacts = read::scan(root, collection)?;
    let artifact = read::resolve(&artifacts, target, display)?;
    perform(root, collection, artifact, to, &mut RealFs)
}

/// The two mutating primitives a transition performs, separated so tests can
/// inject failures at each returned-error boundary — including the rollback
/// path, which no external fault can reach deterministically.
pub(crate) trait TransitionFs {
    /// Atomically replace `dest` with `content` via an exclusive temporary
    /// file beside it. Clobbering is intended: `dest` is the artifact being
    /// rewritten in place.
    fn replace(&mut self, dest: &Path, content: &str) -> io::Result<()>;
    /// Atomically rename `src` to `dst`.
    fn rename(&mut self, src: &Path, dst: &Path) -> io::Result<()>;
}

pub(crate) struct RealFs;

impl TransitionFs for RealFs {
    fn replace(&mut self, dest: &Path, content: &str) -> io::Result<()> {
        let dir = dest.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "artifact path has no parent")
        })?;
        let mut tmp = tempfile::Builder::new()
            .prefix(".strata.artifact.tmp")
            .tempfile_in(dir)?;
        tmp.write_all(content.as_bytes())?;
        tmp.persist(dest).map_err(|err| err.error)?;
        Ok(())
    }

    fn rename(&mut self, src: &Path, dst: &Path) -> io::Result<()> {
        fs::rename(src, dst)
    }
}

/// Perform the transition of one resolved artifact.
pub(crate) fn perform(
    root: &Path,
    collection: &Collection,
    artifact: &Artifact,
    to: Status,
    fs_ops: &mut dyn TransitionFs,
) -> Result<Transition, Error> {
    let from = artifact.summary.status;
    let reference = artifact.summary.reference();
    let src_rel = &artifact.summary.path;
    if from == to {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`{reference}` is already {from} (at `{src_rel}`); no transition needed"
            ),
        });
    }
    if !collection.allows(from, to) {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`{reference}` is {from}, and `{from} -> {to}` is not a {} \
                 transition; the {} lifecycle is: {}",
                collection.kind,
                collection.kind,
                collection.transition_names()
            ),
        });
    }

    let filename = src_rel
        .rsplit('/')
        .next()
        .expect("summary paths always contain a filename");
    let dst_dir_rel = collection.dir_of(to);
    let dst_rel = format!("{dst_dir_rel}/{filename}");
    let src = root.join(src_rel);
    let dst = root.join(&dst_rel);

    let rewritten =
        rewrite_status(&artifact.content, from, to).map_err(|reason| Error::MalformedArtifact {
            path: src.clone(),
            reason,
        })?;

    match fs::symlink_metadata(&dst) {
        Ok(_) => {
            return Err(Error::ArtifactConflict {
                path: dst,
                reason: "an artifact already occupies the destination path".into(),
            });
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(Error::Filesystem {
                operation: "inspect transition destination".into(),
                path: dst,
                source,
            });
        }
    }

    // Git does not round-trip empty directories, so a cloned repository may
    // lack the destination; materialize it before mutating anything.
    crate::repo::ensure_dir(root, dst_dir_rel, &mut Vec::new())?;

    // Step 1, the commit point: the artifact's own front matter now records
    // the transition.
    fs_ops
        .replace(&src, &rewritten)
        .map_err(|source| Error::Filesystem {
            operation: "rewrite front-matter status".into(),
            path: src.clone(),
            source,
        })?;

    // Step 2, mechanical filing: placement follows status.
    if let Err(rename_error) = fs_ops.rename(&src, &dst) {
        return Err(match fs_ops.replace(&src, &artifact.content) {
            Ok(()) => Error::Filesystem {
                operation: format!(
                    "file the transition into `{dst_dir_rel}` (the status \
                     rewrite was rolled back; `{reference}` is unchanged)"
                ),
                path: src,
                source: rename_error,
            },
            Err(rollback_error) => Error::TransitionInterrupted {
                path: src,
                status: to.name(),
                placement: collection.dir_of(from),
                destination: dst_rel,
                rename_error,
                rollback_error,
            },
        });
    }

    Ok(Transition {
        reference,
        from,
        to,
        to_path: dst_rel,
    })
}

/// Rewrite exactly the front-matter `status` value from `from` to `to`,
/// preserving every other byte.
///
/// Only the front-matter block is touched; `status:` lines in the body are
/// content. The value must be written in the plain unquoted form the
/// templates produce — exactly one top-level `status:` line whose trimmed
/// value equals `from`. Anything else (quoted values, trailing comments,
/// duplicated keys) is refused rather than guessed at, since a wrong splice
/// would corrupt canonical bytes.
fn rewrite_status(content: &str, from: Status, to: Status) -> Result<String, String> {
    let (fm_start, fm_end) = front_matter_region(content)
        .ok_or_else(|| "missing front matter: cannot locate the `status` value".to_string())?;
    let front_matter = &content[fm_start..fm_end];

    let mut value_range = None;
    let mut offset = 0;
    for line in front_matter.split_inclusive('\n') {
        let line_text = line.strip_suffix('\n').unwrap_or(line);
        if let Some(value) = line_text.strip_prefix("status:") {
            let trimmed = value.trim();
            if trimmed != from.name() {
                return Err(format!(
                    "the front-matter `status` value is not written as plain \
                     `{}`, so Strata cannot rewrite it precisely; edit the \
                     file by hand",
                    from.name()
                ));
            }
            if value_range.is_some() {
                return Err(
                    "multiple front-matter `status` lines; repair the file by hand".to_string(),
                );
            }
            let leading = value.len() - value.trim_start().len();
            let start = fm_start + offset + "status:".len() + leading;
            value_range = Some(start..start + trimmed.len());
        }
        offset += line.len();
    }

    let range = value_range.ok_or_else(|| {
        format!(
            "no plain front-matter `status: {}` line found to rewrite; edit \
             the file by hand",
            from.name()
        )
    })?;
    let mut rewritten = String::with_capacity(content.len() + to.name().len());
    rewritten.push_str(&content[..range.start]);
    rewritten.push_str(to.name());
    rewritten.push_str(&content[range.end..]);
    Ok(rewritten)
}

/// Byte range of the front-matter block body, mirroring the read pipeline's
/// delimiter rules: the file opens with `---\n` and the block ends at the
/// next `---` line.
fn front_matter_region(content: &str) -> Option<(usize, usize)> {
    let rest = content.strip_prefix("---\n")?;
    if let Some(end) = rest.find("\n---\n") {
        Some((4, 4 + end + 1))
    } else if rest.ends_with("\n---") {
        Some((4, content.len() - 3))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo;
    use crate::repo::{DRAGONS_CLOSED_DIR, DRAGONS_OPEN_DIR};

    fn temp_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create temporary directory");
        repo::init(tmp.path()).expect("initialize repository");
        tmp
    }

    fn rich_dragon(status: &str) -> String {
        format!(
            "---\nid: drg-rich\nsequence: 1\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\nseverity: high\n---\n\n# Rich dragon\n\nBody mentions status: open in prose.  \n\n```yaml\nstatus: open\n```\n"
        )
    }

    fn seed(root: &Path, dir: &str, name: &str, content: &str) {
        fs::write(root.join(dir).join(name), content).unwrap();
    }

    fn scan_one(root: &Path) -> Artifact {
        let artifacts = read::scan(root, &read::DRAGON).unwrap();
        assert_eq!(artifacts.len(), 1);
        artifacts.into_iter().next().unwrap()
    }

    /// Injectable failures: `fail_rename` fails every rename; replaces
    /// succeed until `fail_replace_after` have run, then fail.
    struct FailingFs {
        fail_rename: bool,
        fail_replace_after: usize,
        replaces: usize,
    }

    impl TransitionFs for FailingFs {
        fn replace(&mut self, dest: &Path, content: &str) -> io::Result<()> {
            self.replaces += 1;
            if self.replaces > self.fail_replace_after {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected replace failure",
                ));
            }
            RealFs.replace(dest, content)
        }

        fn rename(&mut self, src: &Path, dst: &Path) -> io::Result<()> {
            if self.fail_rename {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected rename failure",
                ));
            }
            RealFs.rename(src, dst)
        }
    }

    #[test]
    fn rewrite_status_changes_exactly_the_status_value() {
        let original = rich_dragon("open");
        let rewritten = rewrite_status(&original, Status::Open, Status::Closed).unwrap();

        assert_eq!(rewritten, rich_dragon("closed"));
        assert!(
            rewritten.contains("Body mentions status: open in prose."),
            "body bytes must be untouched"
        );
        assert!(
            rewritten.contains("```yaml\nstatus: open\n```"),
            "fenced content must be untouched"
        );
    }

    #[test]
    fn rewrite_status_preserves_unusual_spacing_around_the_value() {
        let original = "---\nid: x\nsequence: 1\nkind: dragon\nstatus:   open  \ncreated: 2026-07-20\n---\n\n# T\n";
        let rewritten = rewrite_status(original, Status::Open, Status::Closed).unwrap();
        assert_eq!(
            rewritten,
            "---\nid: x\nsequence: 1\nkind: dragon\nstatus:   closed  \ncreated: 2026-07-20\n---\n\n# T\n"
        );
    }

    #[test]
    fn rewrite_status_refuses_quoted_or_commented_values() {
        for status_line in ["status: \"open\"", "status: 'open'", "status: open # note"] {
            let content = format!("---\nid: x\n{status_line}\n---\n\n# T\n");
            let err = rewrite_status(&content, Status::Open, Status::Closed).unwrap_err();
            assert!(err.contains("by hand"), "for {status_line:?}: {err}");
        }
    }

    #[test]
    fn rewrite_status_refuses_when_no_status_line_exists() {
        let err =
            rewrite_status("---\nid: x\n---\n\n# T\n", Status::Open, Status::Closed).unwrap_err();
        assert!(err.contains("no plain front-matter"), "{err}");
    }

    #[test]
    fn undefined_transitions_are_invalid_invocations_naming_the_lifecycle() {
        let tmp = temp_repo();
        fs::create_dir_all(tmp.path().join(crate::repo::IDEAS_ADOPTED_DIR)).unwrap();
        seed(
            tmp.path(),
            crate::repo::IDEAS_ADOPTED_DIR,
            "0001-settled.md",
            "---\nid: idea-settled\nsequence: 1\nkind: idea\nstatus: adopted\ncreated: 2026-07-20\n---\n\n# Settled\n",
        );

        let err = transition(
            tmp.path(),
            &read::IDEA,
            Selector::Sequence(1),
            "idea:1",
            Status::Rejected,
        )
        .unwrap_err();

        assert!(matches!(err, Error::InvalidInvocation { .. }), "{err:?}");
        let message = err.to_string();
        assert!(
            message.contains("parked -> adopted, parked -> rejected"),
            "the refusal must name the legal lifecycle: {message}"
        );
        assert!(
            tmp.path()
                .join(crate::repo::IDEAS_ADOPTED_DIR)
                .join("0001-settled.md")
                .is_file(),
            "nothing may move"
        );
    }

    #[test]
    fn failed_rename_rolls_back_to_the_original_bytes() {
        let tmp = temp_repo();
        let original = rich_dragon("open");
        seed(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-rich-dragon.md",
            &original,
        );
        let artifact = scan_one(tmp.path());
        let mut fs_ops = FailingFs {
            fail_rename: true,
            fail_replace_after: usize::MAX,
            replaces: 0,
        };

        let err = perform(
            tmp.path(),
            &read::DRAGON,
            &artifact,
            Status::Closed,
            &mut fs_ops,
        )
        .unwrap_err();

        assert!(matches!(err, Error::Filesystem { .. }), "{err:?}");
        assert!(
            err.to_string().contains("rolled back"),
            "the error must state the rollback: {err}"
        );
        let src = tmp
            .path()
            .join(DRAGONS_OPEN_DIR)
            .join("0001-rich-dragon.md");
        assert_eq!(
            fs::read_to_string(&src).unwrap(),
            original,
            "rollback must restore the original bytes"
        );
        assert!(
            !tmp.path()
                .join(DRAGONS_CLOSED_DIR)
                .join("0001-rich-dragon.md")
                .exists(),
            "nothing may reach the destination"
        );
        assert!(crate::doctor::check(tmp.path()).unwrap().healthy());
    }

    #[test]
    fn doubly_failed_rollback_reports_the_interrupted_state_doctor_diagnoses() {
        let tmp = temp_repo();
        seed(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-rich-dragon.md",
            &rich_dragon("open"),
        );
        let artifact = scan_one(tmp.path());
        let mut fs_ops = FailingFs {
            fail_rename: true,
            fail_replace_after: 1,
            replaces: 0,
        };

        let err = perform(
            tmp.path(),
            &read::DRAGON,
            &artifact,
            Status::Closed,
            &mut fs_ops,
        )
        .unwrap_err();

        let rendered = err.render();
        assert!(
            matches!(err, Error::TransitionInterrupted { .. }),
            "{err:?}"
        );
        assert!(
            rendered.contains("status: closed")
                && rendered.contains(DRAGONS_OPEN_DIR)
                && rendered.contains("doctor"),
            "the error must name the mismatch state and point at doctor: {rendered}"
        );

        // The leaked state is exactly the crash-window intermediate:
        // committed status, stale placement — one valid artifact, one path.
        let src = tmp
            .path()
            .join(DRAGONS_OPEN_DIR)
            .join("0001-rich-dragon.md");
        assert_eq!(fs::read_to_string(&src).unwrap(), rich_dragon("closed"));

        let report = crate::doctor::check(tmp.path()).unwrap();
        assert_eq!(report.findings.len(), 1, "{:?}", report.findings);
        assert_eq!(report.findings[0].problem, "malformed-artifact");
        assert!(
            report.findings[0].detail.contains("lifecycle mismatch"),
            "{}",
            report.findings[0].detail
        );

        // A re-run refuses the mismatched artifact instead of repairing it.
        let rerun = transition(
            tmp.path(),
            &read::DRAGON,
            Selector::Sequence(1),
            "dragon:1",
            Status::Closed,
        )
        .unwrap_err();
        assert!(
            matches!(rerun, Error::MalformedArtifact { .. }),
            "{rerun:?}"
        );
        assert!(rerun.to_string().contains("lifecycle mismatch"), "{rerun}");
    }
}
