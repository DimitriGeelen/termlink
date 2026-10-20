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
//!     age <= 2*interval => LIVE, <= 5*interval => STALE, else OFFLINE

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
    /// T-2297 (V2b): the HUB-ATTESTED observed TCP source address
    /// (`metadata.observed_addr`), stamped server-side from the connection's
    /// `peer_addr`. Cannot be forged by the client. `None` for Unix-socket posts
    /// and pre-T-2297 hubs. The resolver PREFERS this over the self-reported
    /// `addr` when present (defeats stale/spoofed self-reports).
    pub observed_addr: Option<String>,
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
    let observed_addr = md_str("observed_addr"); // T-2297: hub-attested, prefer over addr
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
        observed_addr,
        host,
        listen_topics,
        role,
    })
}

/// T-2385 (comms loud-contract, Slice 1): structured reachability preflight for
/// an `agent contact` recipient, computed from the recipient's authoritative
/// `agent-presence` heartbeat. Turns the silent "offset-N to a peer nobody will
/// read" failure into a per-link report the caller can surface loud.
///
/// T-3227: moved here from the CLI so the CLI `agent contact` and the MCP
/// `termlink_agent_contact` share one classifier (and one diagnosis wording).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachabilityReport {
    /// Recipient has a LIVE agent-presence heartbeat (not STALE/OFFLINE/absent).
    pub recipient_live: bool,
    /// Recipient looks agent-backed (heartbeating with a role and/or a bound
    /// PTY) rather than a bare `termlink register --shell` (which emits no
    /// presence at all → the absent arm).
    pub recipient_agent_backed: bool,
    /// Recipient's be-reachable heartbeat carries a bound PTY (`pty_session`), so
    /// its push-waker can inject a doorbell. `false` → the DM lands durably but
    /// the peer is NOT push-woken (it queues until they run `/check-arc`).
    pub waker_running: bool,
    /// `LIVE` / `STALE` / `OFFLINE` / `ABSENT`.
    pub presence_status: &'static str,
    /// One-line human diagnosis of the first broken link; `None` when all green.
    pub diagnosis: Option<String>,
}

impl ReachabilityReport {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "recipient_live": self.recipient_live,
            "recipient_agent_backed": self.recipient_agent_backed,
            "waker_running": self.waker_running,
            "presence_status": self.presence_status,
            "diagnosis": self.diagnosis,
        })
    }
    /// True when any delivery link the recipient controls is broken — the gate
    /// for the loud human WARNING (and, with `--require-reachable`, the hard
    /// fail on `!recipient_live`).
    pub fn any_link_broken(&self) -> bool {
        !self.recipient_live || !self.waker_running
    }
}

/// T-2385: pure classifier — recipient's presence heartbeat → per-link report.
/// `None` means no heartbeat matched the recipient (not `/be-reachable`, or on a
/// different hub). Kept pure (no I/O) so it is unit-testable in isolation.
pub fn classify_reachability(presence: Option<&PresenceMatch>) -> ReachabilityReport {
    match presence {
        None => ReachabilityReport {
            recipient_live: false,
            recipient_agent_backed: false,
            waker_running: false,
            presence_status: "ABSENT",
            diagnosis: Some(
                "no agent-presence heartbeat for recipient — they are not \
                 /be-reachable (or heartbeating on a different hub). The DM will \
                 land durably but nobody is listening to be woken. Confirm with \
                 `/peers --all`."
                    .to_string(),
            ),
        },
        Some(m) => {
            let live = m.status == PresenceStatus::Live;
            let waker = m.pty_session.is_some();
            // A real agent heartbeats with a role and/or a bound PTY. A bare
            // `register --shell` emits no presence (→ the None arm); a heartbeat
            // that exists but carries neither a role nor a pty is treated as
            // not-clearly-agent-backed.
            let agent_backed = m.role.is_some() || m.pty_session.is_some();
            let diagnosis = if !live {
                Some(format!(
                    "recipient presence is {} (not LIVE) — last heartbeat {}s ago; \
                     they are likely gone. The DM will land but may never be read.",
                    m.status.as_str(),
                    m.age_secs
                ))
            } else if !waker {
                Some(
                    "recipient is LIVE but has no push-waker (no bound PTY on its \
                     be-reachable heartbeat) — the DM will queue until they run \
                     `/check-arc`; it will NOT push-wake them."
                        .to_string(),
                )
            } else {
                None
            };
            ReachabilityReport {
                recipient_live: live,
                recipient_agent_backed: agent_backed,
                waker_running: waker,
                presence_status: m.status.as_str(),
                diagnosis,
            }
        }
    }
}

/// T-2385 / T-3227: pick the recipient's heartbeat out of an `agent-presence`
/// slice. Keyed on the target NAME when known (the authoritative agent-presence
/// key); else on the resolved `peer_fp` (find a heartbeat whose `sender_id`
/// matches, read its `metadata.agent_id`, then resolve normally so status/age
/// bands are consistent). Pure — the transport that fetched `msgs` is per-crate.
pub fn select_recipient_presence(
    msgs: &[Value],
    target_name: Option<&str>,
    peer_fp: &str,
    now_ms: i64,
) -> Option<PresenceMatch> {
    if let Some(name) = target_name {
        return resolve_agent_presence(msgs, name, now_ms);
    }
    let agent_id = msgs.iter().rev().find_map(|m| {
        let sid = m.get("sender_id").and_then(|v| v.as_str())?;
        if sid != peer_fp {
            return None;
        }
        m.get("metadata")
            .and_then(|md| md.get("agent_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    })?;
    resolve_agent_presence(msgs, &agent_id, now_ms)
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

    /// T-2385: the reachability classifier across all four recipient states.
    #[test]
    fn classify_reachability_all_states() {
        fn pm(
            status: PresenceStatus,
            pty: Option<&str>,
            role: Option<&str>,
        ) -> PresenceMatch {
            PresenceMatch {
                agent_id: "peer".to_string(),
                identity_fingerprint: Some("abc1230000000000".to_string()),
                pty_session: pty.map(|s| s.to_string()),
                status,
                age_secs: 5,
                last_ts_ms: 0,
                addr: None,
                observed_addr: None,
                host: None,
                listen_topics: vec![],
                role: role.map(|s| s.to_string()),
            }
        }

        // LIVE + bound PTY → all green, no diagnosis, nothing broken.
        let r = classify_reachability(Some(&pm(
            PresenceStatus::Live,
            Some("tmux:0"),
            Some("claude-code"),
        )));
        assert!(r.recipient_live && r.waker_running && r.recipient_agent_backed);
        assert_eq!(r.presence_status, "LIVE");
        assert!(r.diagnosis.is_none());
        assert!(!r.any_link_broken());

        // LIVE but no PTY → waker_running=false, diagnosis names the no-waker breakpoint.
        let r =
            classify_reachability(Some(&pm(PresenceStatus::Live, None, Some("claude-code"))));
        assert!(r.recipient_live && !r.waker_running);
        assert!(r.any_link_broken());
        assert!(r.diagnosis.as_deref().unwrap().contains("no push-waker"));

        // STALE → recipient_live=false, loud "not LIVE" diagnosis.
        let r = classify_reachability(Some(&pm(
            PresenceStatus::Stale,
            Some("tmux:0"),
            Some("claude-code"),
        )));
        assert!(!r.recipient_live);
        assert_eq!(r.presence_status, "STALE");
        assert!(r.any_link_broken());
        assert!(r.diagnosis.as_deref().unwrap().contains("not LIVE"));

        // Absent heartbeat → recipient_live=false, "no agent-presence heartbeat".
        let r = classify_reachability(None);
        assert!(!r.recipient_live && !r.waker_running && !r.recipient_agent_backed);
        assert_eq!(r.presence_status, "ABSENT");
        assert!(r
            .diagnosis
            .as_deref()
            .unwrap()
            .contains("no agent-presence heartbeat"));
    }

    /// T-3227: the recipient selector, by name and by fp (the --target-fp path).
    #[test]
    fn select_recipient_presence_by_name_and_fp() {
        let now = 1_000_000_000_000i64;
        let msgs = vec![
            hb("alice", "aaaa1111", "pty-1", now - 5_000, 30),
            hb("bob", "bbbb2222", "", now - 5_000, 30),
        ];
        let m = select_recipient_presence(&msgs, Some("alice"), "ignored", now).unwrap();
        assert_eq!(m.agent_id, "alice");
        let m = select_recipient_presence(&msgs, None, "bbbb2222", now).unwrap();
        assert_eq!(m.agent_id, "bob");
        assert!(m.pty_session.is_none());
        assert!(select_recipient_presence(&msgs, None, "cccc3333", now).is_none());
        assert!(select_recipient_presence(&msgs, Some("carol"), "aaaa1111", now).is_none());
    }
}
