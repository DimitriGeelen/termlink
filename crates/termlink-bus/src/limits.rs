//! T-3310 (arc-012 step 5, operator ruling D3 = C): ceilings checked on post.
//!
//! The hub's background sweeper (T-2427) only runs where it is switched on, and
//! `Bus::post` used to apply no retention at all, so a burst into a bounded
//! topic grew until a sweep that two of three hubs never run. Each post now
//! checks the topic against a ceiling and trims the oldest records when it is
//! passed. Ceilings use a 2x margin: they fire at twice the limit and trim back
//! to the limit, so a trim happens about once per N posts, not on every post.
//!
//! | retention          | count                         | age                        | live bytes |
//! |--------------------|-------------------------------|----------------------------|------------|
//! | messages(N)        | at 2N, back to N              | -                          | at max, back to max/2 |
//! | days(d)            | at `max_records`, back to half| oldest past 2d, back to d  | at max, back to max/2 |
//! | latest             | at 2, back to 1               | -                          | -          |
//! | latest_per_cv_key  | compact past `max_records`    | -                          | -          |
//! | forever            | WARN at `max_records`         | -                          | WARN at max (never deleted) |
//!
//! All numbers live in [`Ceilings`] and are configurable (hub config file).
//! Trimming is index-only like every other trim; reclaiming the dead bytes on
//! disk is T-3322.

use crate::Retention;

/// D3 defaults (ruled 2026-10-02; agent proposals accepted unchanged).
pub const DEFAULT_MAX_RECORDS: u64 = 10_000;
pub const DEFAULT_MAX_LIVE_BYTES: u64 = 64 * 1024 * 1024;

/// The configurable ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ceilings {
    /// Count ceiling for day-bounded and per-key topics; forever topics warn here.
    pub max_records: u64,
    /// Live-byte ceiling for bounded topics; forever topics warn here.
    pub max_live_bytes: u64,
    /// Check ceilings on post at all (default true). Off leaves bounding to
    /// the background sweeper only, the pre-T-3310 behaviour.
    pub on_post: bool,
}

impl Default for Ceilings {
    fn default() -> Self {
        Self { max_records: DEFAULT_MAX_RECORDS, max_live_bytes: DEFAULT_MAX_LIVE_BYTES, on_post: true }
    }
}

/// What one topic currently holds (index view).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TopicStats {
    pub records: u64,
    pub live_bytes: u64,
    pub oldest_ts_ms: Option<i64>,
}

/// What a post must do after appending.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeilingAction {
    None,
    /// Keep only the newest `keep` records.
    KeepLast { keep: u64, reason: &'static str },
    /// Drop records older than `cutoff_ms`.
    KeepAfter { cutoff_ms: i64, reason: &'static str },
    /// Drop the oldest records until at most `target` live bytes remain.
    KeepBytes { target: u64, reason: &'static str },
    /// Per-key compaction (latest_per_cv_key).
    CompactPerKey,
    /// Forever topic past a ceiling: warn, never delete.
    WarnForever { reason: &'static str },
}

const DAY_MS: i64 = 86_400_000;

/// Pure decision for one topic. Count and age are checked before bytes; the
/// first ceiling passed decides (the next post re-checks the rest).
pub fn ceiling_action(r: Retention, s: TopicStats, now_ms: i64, c: Ceilings) -> CeilingAction {
    let over_bytes = s.live_bytes > c.max_live_bytes;
    match r {
        Retention::Messages(n) => {
            if s.records > n.saturating_mul(2) {
                CeilingAction::KeepLast { keep: n, reason: "count past 2x messages limit" }
            } else if over_bytes {
                CeilingAction::KeepBytes { target: c.max_live_bytes / 2, reason: "live bytes past ceiling" }
            } else {
                CeilingAction::None
            }
        }
        Retention::Days(d) => {
            let window = i64::from(d).saturating_mul(DAY_MS);
            if s.records > c.max_records {
                CeilingAction::KeepLast { keep: c.max_records / 2, reason: "count past ceiling" }
            } else if let Some(oldest) = s.oldest_ts_ms
                && now_ms - oldest > window.saturating_mul(2)
            {
                CeilingAction::KeepAfter { cutoff_ms: now_ms - window, reason: "oldest past 2x days window" }
            } else if over_bytes {
                CeilingAction::KeepBytes { target: c.max_live_bytes / 2, reason: "live bytes past ceiling" }
            } else {
                CeilingAction::None
            }
        }
        Retention::Latest => {
            if s.records > 2 {
                CeilingAction::KeepLast { keep: 1, reason: "latest topic past 2 records" }
            } else {
                CeilingAction::None
            }
        }
        Retention::LatestPerCvKey => {
            if s.records > c.max_records {
                CeilingAction::CompactPerKey
            } else {
                CeilingAction::None
            }
        }
        Retention::Forever => {
            if s.records > c.max_records {
                CeilingAction::WarnForever { reason: "forever topic past record ceiling" }
            } else if over_bytes {
                CeilingAction::WarnForever { reason: "forever topic past byte ceiling" }
            } else {
                CeilingAction::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 100 * DAY_MS;
    fn st(records: u64, live_bytes: u64, oldest_days_ago: Option<i64>) -> TopicStats {
        TopicStats { records, live_bytes, oldest_ts_ms: oldest_days_ago.map(|d| NOW - d * DAY_MS) }
    }
    fn c() -> Ceilings {
        Ceilings { max_records: 10_000, max_live_bytes: 1000, on_post: true }
    }

    #[test]
    fn messages_trim_fires_past_2n_and_keeps_n() {
        let r = Retention::Messages(1000);
        assert_eq!(ceiling_action(r, st(2000, 0, None), NOW, c()), CeilingAction::None);
        assert_eq!(
            ceiling_action(r, st(2001, 0, None), NOW, c()),
            CeilingAction::KeepLast { keep: 1000, reason: "count past 2x messages limit" }
        );
    }

    #[test]
    fn days_topic_has_a_count_ceiling_too() {
        // The operator's flood case: a 14-day topic must not grow without bound inside its window.
        let r = Retention::Days(14);
        assert_eq!(ceiling_action(r, st(10_000, 0, Some(1)), NOW, c()), CeilingAction::None);
        assert_eq!(
            ceiling_action(r, st(10_001, 0, Some(1)), NOW, c()),
            CeilingAction::KeepLast { keep: 5000, reason: "count past ceiling" }
        );
    }

    #[test]
    fn days_age_trim_fires_past_2x_window_and_cuts_to_1x() {
        let r = Retention::Days(14);
        assert_eq!(ceiling_action(r, st(10, 0, Some(28)), NOW, c()), CeilingAction::None);
        assert_eq!(
            ceiling_action(r, st(10, 0, Some(29)), NOW, c()),
            CeilingAction::KeepAfter { cutoff_ms: NOW - 14 * DAY_MS, reason: "oldest past 2x days window" }
        );
    }

    #[test]
    fn byte_ceiling_trims_bounded_topics_to_half() {
        for r in [Retention::Messages(1000), Retention::Days(14)] {
            assert_eq!(ceiling_action(r, st(5, 1000, Some(0)), NOW, c()), CeilingAction::None);
            assert_eq!(
                ceiling_action(r, st(5, 1001, Some(0)), NOW, c()),
                CeilingAction::KeepBytes { target: 500, reason: "live bytes past ceiling" }
            );
        }
    }

    #[test]
    fn forever_only_ever_warns() {
        let r = Retention::Forever;
        assert_eq!(ceiling_action(r, st(10_000, 1000, None), NOW, c()), CeilingAction::None);
        assert!(matches!(ceiling_action(r, st(10_001, 0, None), NOW, c()), CeilingAction::WarnForever { .. }));
        assert!(matches!(ceiling_action(r, st(1, 1001, None), NOW, c()), CeilingAction::WarnForever { .. }));
    }

    #[test]
    fn latest_and_per_key_compact() {
        assert_eq!(ceiling_action(Retention::Latest, st(2, 0, None), NOW, c()), CeilingAction::None);
        assert!(matches!(ceiling_action(Retention::Latest, st(3, 0, None), NOW, c()), CeilingAction::KeepLast { keep: 1, .. }));
        assert_eq!(ceiling_action(Retention::LatestPerCvKey, st(10_001, 0, None), NOW, c()), CeilingAction::CompactPerKey);
    }
}
