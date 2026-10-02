//! T-3309 (arc-012, T-3304 IW-4 C'): per-topic activity, kept OUTSIDE the
//! message log so retention can never delete it.
//!
//! What is recorded:
//! - writes: last writer, last write time, total writes. A `topic_metadata`
//!   envelope (describe / ensure touch) is NOT activity (consultation catch:
//!   an ensure-topic must not make a dead topic look alive).
//! - fetches: last fetch (any `channel.subscribe` call), last DATA fetch (one
//!   that returned at least one record), totals, and the same per named reader.
//! - `unread_since_ms`: the time of the oldest write not yet followed by a data
//!   fetch. Set by a write when unset, cleared by a data fetch. This is what
//!   catches a topic that is written to constantly and read by nobody (the
//!   ring20 probe class) — "no read AND no write" never fires on it.
//!
//! Flags are advisory (the operator ruled flag, never delete):
//! - `idle`: no write and no fetch for N days, once the hub has been watching
//!   the topic for at least N days.
//! - `unread`: a write has waited N days without any data fetch.
//!
//! Scope, stated plainly: a "fetch" is a `channel.subscribe` call. Topics read
//! only through other RPCs (cv_keys, find_idle, search) are not seen as read.

use serde::Serialize;

/// Default flag threshold (operator ruling, IW-4: 30 days).
pub const DEFAULT_FLAG_DAYS: u64 = 30;

/// Activity for one topic. All times are unix milliseconds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TopicActivity {
    /// When this hub started tracking the topic (first post or fetch seen).
    pub tracking_since_ms: i64,
    pub last_write_ms: Option<i64>,
    pub last_writer: Option<String>,
    pub writes_total: u64,
    pub last_fetch_ms: Option<i64>,
    pub last_data_fetch_ms: Option<i64>,
    pub fetches_total: u64,
    pub unread_since_ms: Option<i64>,
}

/// Fetch activity of one named reader on one topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReaderActivity {
    pub reader: String,
    pub last_fetch_ms: i64,
    pub last_data_fetch_ms: Option<i64>,
}

/// The advisory flags for `act` at `now_ms`, with an N-day threshold.
/// Pure, so the boundary cases are unit-tested without a clock.
pub fn activity_flags(act: &TopicActivity, now_ms: i64, days: u64) -> Vec<&'static str> {
    let window = (days as i64).saturating_mul(86_400_000);
    let mut flags = Vec::new();
    let watched_long_enough = now_ms - act.tracking_since_ms >= window;
    let last_seen = [act.last_write_ms, act.last_fetch_ms, Some(act.tracking_since_ms)]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(act.tracking_since_ms);
    if watched_long_enough && now_ms - last_seen >= window {
        flags.push("idle");
    }
    if let Some(since) = act.unread_since_ms
        && now_ms - since >= window
    {
        flags.push("unread");
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400_000;

    fn act() -> TopicActivity {
        TopicActivity { tracking_since_ms: 0, ..Default::default() }
    }

    #[test]
    fn no_flag_before_the_hub_has_watched_n_days() {
        // A topic first seen today is not idle just because nothing happened yet.
        assert!(activity_flags(&act(), 29 * DAY, 30).is_empty());
        assert_eq!(activity_flags(&act(), 30 * DAY, 30), vec!["idle"]);
    }

    #[test]
    fn idle_needs_both_no_write_and_no_fetch() {
        let mut a = act();
        a.last_write_ms = Some(40 * DAY);
        assert!(activity_flags(&a, 60 * DAY, 30).is_empty(), "written 20 days ago");
        a.last_write_ms = Some(DAY);
        a.last_fetch_ms = Some(50 * DAY);
        assert!(activity_flags(&a, 60 * DAY, 30).is_empty(), "fetched 10 days ago");
        a.last_fetch_ms = Some(DAY);
        assert_eq!(activity_flags(&a, 60 * DAY, 30), vec!["idle"]);
    }

    #[test]
    fn unread_fires_on_a_busy_topic_nobody_reads() {
        // The ring20 probe shape: written every minute, never fetched with data.
        let mut a = act();
        a.last_write_ms = Some(60 * DAY);
        a.writes_total = 80_000;
        a.unread_since_ms = Some(10 * DAY);
        assert_eq!(activity_flags(&a, 60 * DAY, 30), vec!["unread"], "busy, so not idle");
        a.unread_since_ms = Some(40 * DAY);
        assert!(activity_flags(&a, 60 * DAY, 30).is_empty(), "waited only 20 days");
    }

    #[test]
    fn both_flags_can_hold() {
        let mut a = act();
        a.last_write_ms = Some(DAY);
        a.unread_since_ms = Some(DAY);
        assert_eq!(activity_flags(&a, 60 * DAY, 30), vec!["idle", "unread"]);
    }
}
