//! Shared pure resolver for `agent-presence` heartbeats (T-2275).
//!
//! Resolves an `agent_id` to its `{identity_fingerprint, pty_session, status}`
//! from a slice of `agent-presence` heartbeat envelopes. This is the semantic
//! core of cross-hub "contact a peer by name" — used by both the CLI
//! (`cmd_agent_contact`) and the MCP handler (`termlink_agent_contact`).
//!
//! The TRANSPORT (walking `hubs.toml` + subscribing `agent-presence` per hub)
//! lives per-crate, mirroring the existing `connect_remote_hub` /
//! `connect_remote_hub_mcp` duplication. ONLY this pure parse+classify logic is
//! shared, so the two callers cannot drift on the heartbeat contract.
//!
//! Parse contract mirrors `scripts/agent-listeners.sh` exactly:
//!   - filter `msg_type == "heartbeat"` and non-empty `metadata.agent_id`
//!   - newest-by-`ts` wins per agent_id (re-heartbeat within the slice)
//!   - `identity_fingerprint` = envelope top-level `sender_id` (T-2270)
//!   - `pty_session` = `metadata.pty_session`
//!   - status from age vs `metadata.interval_secs` (default 30):
//!       age <= 2*interval => LIVE, <= 5*interval => STALE, else OFFLINE

use serde_json::Value;

/// Liveness classification for a presence heartbeat, matching
/// `agent-listeners.sh`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceStatus {
    Live,
    Stale,
    Offline,
}

impl PresenceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PresenceStatus::Live => "LIVE",
            PresenceStatus::Stale => "STALE",
            PresenceStatus::Offline => "OFFLINE",
        }
    }
}

/// A resolved presence match for a given `agent_id` within one heartbeat slice.
#[derive(Debug, Clone)]
pub struct PresenceMatch {
    pub agent_id: String,
    /// Envelope `sender_id` — the peer's identity fingerprint (T-2270). `None`
    /// only if the matched heartbeat carried no `sender_id` (pre-T-2270 peer).
    pub identity_fingerprint: Option<String>,
    pub pty_session: Option<String>,
    pub status: PresenceStatus,
    pub age_secs: i64,
    pub last_ts_ms: i64,
    /// T-2293 (V2 discovery registry): the agent's self-reported reachable hub
    /// address (`metadata.addr`, e.g. `192.168.10.107:9100`) — where a peer
    /// posts to reach this agent. `None` for pre-T-2293 heartbeats; the resolver
    /// then falls back to the hub it read the heartbeat from.
    pub addr: Option<String>,
    /// T-2293: self-reported `metadata.host` (hostname — who, not where-to-post).
    pub host: Option<String>,
    /// T-2293: the agent's read topics (`metadata.listen_topics`, csv-split).
    /// Empty when absent. Part of the `{host:port, hub, topics-read, liveness}`
    /// registry record (AC1).
    pub listen_topics: Vec<String>,
    /// T-2293: self-reported `metadata.role`.
    pub role: Option<String>,
}

fn msg_ts_ms(m: &Value) -> i64 {
    m.get("ts_unix_ms")
        .and_then(|v| v.as_i64())
        .or_else(|| m.get("ts").and_then(|v| v.as_i64()))
        .unwrap_or(0)
}

/// Resolve the newest `agent-presence` heartbeat for `agent_id` in `msgs`.
///
/// Returns `None` if no heartbeat envelope carries `metadata.agent_id ==
/// agent_id`. Newest-by-ts wins when an agent re-heartbeats within the slice.
/// Status is classified from the matched envelope's own `metadata.interval_secs`
/// (default 30) per the agent-listeners.sh 2x/5x bands. `now_ms` is the caller's
/// wall clock in unix-ms; pass the same clock the heartbeats' `ts` use.
pub fn resolve_agent_presence(
    msgs: &[Value],
    agent_id: &str,
    now_ms: i64,
) -> Option<PresenceMatch> {
    let mut best: Option<&Value> = None;
    let mut best_ts = i64::MIN;
    for m in msgs {
        if m.get("msg_type").and_then(|v| v.as_str()) != Some("heartbeat") {
            continue;
        }
        let aid = m
            .get("metadata")
            .and_then(|md| md.get("agent_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if aid.is_empty() || aid != agent_id {
            continue;
        }
        let ts = msg_ts_ms(m);
        if ts >= best_ts {
            best_ts = ts;
            best = Some(m);
        }
    }
    let m = best?;
    let interval = m
        .get("metadata")
        .and_then(|md| md.get("interval_secs"))
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        })
        .filter(|i| *i > 0)
        .unwrap_or(30);
    let age_secs = (now_ms / 1000) - (best_ts / 1000);
    let status = if age_secs <= 2 * interval {
        PresenceStatus::Live
    } else if age_secs <= 5 * interval {
        PresenceStatus::Stale
    } else {
        PresenceStatus::Offline
    };
    let identity_fingerprint = m
        .get("sender_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let pty_session = m
        .get("metadata")
        .and_then(|md| md.get("pty_session"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    // T-2293: pull a metadata string field, treating empty as absent.
    let md_str = |key: &str| {
        m.get("metadata")
            .and_then(|md| md.get(key))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
    };
    let addr = md_str("addr");
    let host = md_str("host");
    let role = md_str("role");
    let listen_topics = md_str("listen_topics")
        .map(|csv| {
            csv.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(PresenceMatch {
        agent_id: agent_id.to_string(),
        identity_fingerprint,
        pty_session,
        status,
        age_secs,
        last_ts_ms: best_ts,
        addr,
        host,
        listen_topics,
        role,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hb(agent_id: &str, sender: &str, pty: &str, ts_ms: i64, interval: i64) -> Value {
        json!({
            "msg_type": "heartbeat",
            "sender_id": sender,
            "ts_unix_ms": ts_ms,
            "metadata": {
                "agent_id": agent_id,
                "pty_session": pty,
                "interval_secs": interval.to_string(),
            }
        })
    }

    #[test]
    fn live_match_resolves_fp_and_pty() {
        let now = 1_000_000_000_000i64;
        let msgs = vec![hb("alice", "deadbeef", "pty-1", now - 10_000, 30)];
        let m = resolve_agent_presence(&msgs, "alice", now).expect("match");
        assert_eq!(m.status, PresenceStatus::Live);
        assert_eq!(m.identity_fingerprint.as_deref(), Some("deadbeef"));
        assert_eq!(m.pty_session.as_deref(), Some("pty-1"));
        assert!(m.age_secs >= 9 && m.age_secs <= 11);
    }

    // T-2293 (V2): the resolver surfaces addr/host/listen_topics/role from
    // heartbeat metadata; absent fields default to None / empty (back-compat).
    #[test]
    fn surfaces_registry_fields_when_present() {
        let now = 1_000_000_000_000i64;
        let msg = json!({
            "msg_type": "heartbeat",
            "sender_id": "deadbeef",
            "ts_unix_ms": now - 10_000,
            "metadata": {
                "agent_id": "alice",
                "role": "claude-code",
                "host": "ring20-107",
                "addr": "192.168.10.107:9100",
                "listen_topics": "dm:alice:*, agent-chat-arc ,",
                "interval_secs": "30",
            }
        });
        let m = resolve_agent_presence(&[msg], "alice", now).expect("match");
        assert_eq!(m.addr.as_deref(), Some("192.168.10.107:9100"));
        assert_eq!(m.host.as_deref(), Some("ring20-107"));
        assert_eq!(m.role.as_deref(), Some("claude-code"));
        // csv split, trimmed, empties dropped.
        assert_eq!(m.listen_topics, vec!["dm:alice:*", "agent-chat-arc"]);
    }

    #[test]
    fn registry_fields_absent_default_empty() {
        let now = 1_000_000_000_000i64;
        let msgs = vec![hb("alice", "deadbeef", "pty-1", now - 10_000, 30)];
        let m = resolve_agent_presence(&msgs, "alice", now).expect("match");
        assert!(m.addr.is_none());
        assert!(m.host.is_none());
        assert!(m.role.is_none());
        assert!(m.listen_topics.is_empty());
    }

    #[test]
    fn stale_band_classified() {
        // age 100s with interval 30 => > 2*30 (60) but <= 5*30 (150) => STALE.
        let now = 1_000_000_000_000i64;
        let msgs = vec![hb("bob", "cafe", "pty-2", now - 100_000, 30)];
        let m = resolve_agent_presence(&msgs, "bob", now).expect("match");
        assert_eq!(m.status, PresenceStatus::Stale);
    }

    #[test]
    fn offline_band_classified() {
        // age 200s with interval 30 => > 5*30 (150) => OFFLINE.
        let now = 1_000_000_000_000i64;
        let msgs = vec![hb("carol", "f00d", "pty-3", now - 200_000, 30)];
        let m = resolve_agent_presence(&msgs, "carol", now).expect("match");
        assert_eq!(m.status, PresenceStatus::Offline);
    }

    #[test]
    fn no_match_returns_none() {
        let now = 1_000_000_000_000i64;
        let msgs = vec![hb("alice", "deadbeef", "pty-1", now - 5_000, 30)];
        assert!(resolve_agent_presence(&msgs, "nobody", now).is_none());
    }

    #[test]
    fn newest_wins_on_duplicate_agent_id() {
        let now = 1_000_000_000_000i64;
        let msgs = vec![
            hb("alice", "old-fp", "pty-old", now - 90_000, 30),
            hb("alice", "new-fp", "pty-new", now - 5_000, 30),
        ];
        let m = resolve_agent_presence(&msgs, "alice", now).expect("match");
        assert_eq!(m.identity_fingerprint.as_deref(), Some("new-fp"));
        assert_eq!(m.pty_session.as_deref(), Some("pty-new"));
        assert_eq!(m.status, PresenceStatus::Live);
    }

    #[test]
    fn non_heartbeat_envelopes_ignored() {
        let now = 1_000_000_000_000i64;
        let mut chat = hb("alice", "deadbeef", "pty-1", now - 5_000, 30);
        chat["msg_type"] = json!("turn");
        let msgs = vec![chat];
        assert!(resolve_agent_presence(&msgs, "alice", now).is_none());
    }

    #[test]
    fn default_interval_when_absent() {
        // No interval_secs => default 30. age 50s => <= 60 => LIVE.
        let now = 1_000_000_000_000i64;
        let msg = json!({
            "msg_type": "heartbeat",
            "sender_id": "abc",
            "ts_unix_ms": now - 50_000,
            "metadata": {"agent_id": "dave"}
        });
        let m = resolve_agent_presence(&[msg], "dave", now).expect("match");
        assert_eq!(m.status, PresenceStatus::Live);
        assert!(m.pty_session.is_none());
    }
}
