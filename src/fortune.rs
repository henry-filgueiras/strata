//! Ambient recall: pick one open dragon or parked idea to resurface,
//! favoring stale artifacts.
//!
//! Fortune is read-only advice — no mutation, no Git, no new state. The
//! staleness bias is pinned structurally, not statistically: [`weight`] is a
//! pure function from candidate metadata to weights (monotonic in age,
//! nonzero for every candidate), and the single random draw enters
//! [`pick`] as a parameter, so tests never sample distributions.
//!
//! Deliberately excluded until a real automation consumer exists (see task
//! 8's amendment): `--seed`, `--json`, and any selection-metadata surface.
//! The weighting is an implementation detail, not a compatibility contract.

use jiff::civil::Date;

/// Selection weight for one open dragon, from its age in days.
///
/// `age + 1`, clamping negative (future-dated) ages to zero: monotonic in
/// age, and at least 1 so every open dragon — including one whose `created`
/// stamp is unparseable (`None`) — has nonzero probability.
pub fn weight(age_days: Option<i64>) -> u64 {
    let age = age_days.map_or(0, |days| days.max(0));
    age as u64 + 1
}

/// Age in whole days of an artifact `created` on the given ISO date, or
/// `None` when the stamp is not a parseable `YYYY-MM-DD` date (the stamp is
/// opaque to the read model; fortune degrades rather than refuses).
pub fn age_days(created: &str, today: Date) -> Option<i64> {
    let date: Date = created.parse().ok()?;
    let span = date.until(today).ok()?;
    Some(i64::from(span.get_days()))
}

/// Human age line: `opened today`, `open 1 day`, `open N days`, or
/// `age unknown` for an unparseable stamp.
pub fn age_text(age_days: Option<i64>) -> String {
    match age_days {
        None => "age unknown".into(),
        Some(days) if days <= 0 => "opened today".into(),
        Some(1) => "open 1 day".into(),
        Some(days) => format!("open {days} days"),
    }
}

/// Weighted draw: map `roll` onto the cumulative weight line and return the
/// selected index. `weights` must be nonempty with a positive total —
/// guaranteed by [`weight`] — and `roll` is the caller's randomness, taken
/// as a parameter so selection itself stays deterministic and testable.
pub fn pick(weights: &[u64], roll: u128) -> usize {
    let total: u128 = weights.iter().map(|w| u128::from(*w)).sum();
    debug_assert!(total > 0, "weights must be nonempty and positive");
    let mut point = roll % total;
    for (index, w) in weights.iter().enumerate() {
        let w = u128::from(*w);
        if point < w {
            return index;
        }
        point -= w;
    }
    weights.len() - 1
}

/// The first `max_lines` prose lines of an artifact body: front matter,
/// headings, blank lines, and fenced code are skipped, so the excerpt reads
/// as the dragon's own words.
pub fn excerpt(content: &str, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_fence = false;
    let mut past_title = false;
    for line in content.lines() {
        let line = line.trim_end();
        if line.starts_with("```") || line.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if !past_title {
            // Everything up to and including the single `# Title` heading is
            // front matter or title, not excerpt material.
            past_title = line.starts_with("# ");
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        lines.push(line.to_string());
        if lines.len() == max_lines {
            break;
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_is_monotonic_in_age_and_everywhere_nonzero() {
        let ages: Vec<Option<i64>> = std::iter::once(None).chain((-2..1000).map(Some)).collect();
        for age in &ages {
            assert!(weight(*age) > 0, "weight must be nonzero for {age:?}");
        }
        for pair in (0..1000i64).collect::<Vec<_>>().windows(2) {
            assert!(
                weight(Some(pair[0])) <= weight(Some(pair[1])),
                "weight must be monotonic in age"
            );
        }
    }

    #[test]
    fn unknown_and_future_ages_weigh_like_today() {
        assert_eq!(weight(None), weight(Some(0)));
        assert_eq!(weight(Some(-5)), weight(Some(0)));
    }

    #[test]
    fn age_days_parses_iso_dates_and_degrades_on_anything_else() {
        let today: Date = "2026-07-22".parse().unwrap();
        assert_eq!(age_days("2026-07-20", today), Some(2));
        assert_eq!(age_days("2026-07-22", today), Some(0));
        assert_eq!(age_days("2026-07-25", today), Some(-3));
        assert_eq!(age_days("yesterday-ish", today), None);
        assert_eq!(age_days("", today), None);
    }

    #[test]
    fn age_text_covers_every_shape() {
        assert_eq!(age_text(None), "age unknown");
        assert_eq!(age_text(Some(0)), "opened today");
        assert_eq!(age_text(Some(1)), "open 1 day");
        assert_eq!(age_text(Some(41)), "open 41 days");
    }

    #[test]
    fn pick_walks_the_cumulative_weight_line() {
        let weights = [1, 3, 2];
        let expected = [0, 1, 1, 1, 2, 2];
        for (roll, want) in expected.iter().enumerate() {
            assert_eq!(pick(&weights, roll as u128), *want, "roll {roll}");
        }
        // The roll wraps modulo the total, so any entropy value selects.
        assert_eq!(pick(&weights, 6), 0);
        assert_eq!(pick(&weights, u128::MAX), pick(&weights, u128::MAX % 6));
    }

    #[test]
    fn every_positive_weight_is_reachable() {
        let weights = [5, 1, 4];
        let total: u128 = 10;
        let mut seen = [false; 3];
        for roll in 0..total {
            seen[pick(&weights, roll)] = true;
        }
        assert_eq!(seen, [true, true, true]);
    }

    #[test]
    fn excerpt_takes_prose_and_skips_structure() {
        let content = "---\nid: x\nstatus: open\n---\n\n# Title\n\n## Context\n\nFirst prose line.  \n\n```sh\n# not prose\ncode\n```\n\nSecond prose line.\nThird prose line.\nFourth prose line.\n";
        assert_eq!(
            excerpt(content, 3),
            vec![
                "First prose line.".to_string(),
                "Second prose line.".to_string(),
                "Third prose line.".to_string(),
            ]
        );
    }

    #[test]
    fn excerpt_of_a_template_only_body_is_empty() {
        let content = "---\nid: x\n---\n\n# Title\n\n## Context\n\n## Question\n";
        assert!(excerpt(content, 3).is_empty());
    }
}
