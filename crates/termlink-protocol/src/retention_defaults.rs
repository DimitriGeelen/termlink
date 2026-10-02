//! T-3310 (arc-012 step 5, operator rulings D1/D2): the ONE table of default
//! retentions for a new topic, shared by the hub (`channel.create` with no
//! retention), the CLI (`channel create`, `--ensure-topic`) and the MCP create
//! tool. Before this, each surface carried its own default and every client
//! asked for `forever` explicitly, so "forever by omission" lived in the
//! clients where no hub-side default could reach it.
//!
//! The table (D2, ruled 2026-10-02):
//! - the four operator-durable topics          -> forever (owner "operator")
//! - `state:*`                                  -> latest
//! - `inbox:*`, `dm:*` (mail: bounded by count, never by age, so unread mail
//!   is only lost by being outnumbered)          -> messages 1000
//! - `agent-presence`, `agent-chat-arc`, `agent-listeners-*`, `agent-conv-*` -> messages 1000
//! - test-debris names (T-2426)                 -> days 7
//! - everything else, including `sidecar:*`     -> days 14
//!
//! The values are defaults: an explicit retention always wins, and the hub
//! config file (T-3310 D3) can override the numbers.

/// The topics the operator ruled intentionally permanent (T-2057 audit §5,
/// T-3304 IW-2). They keep `forever` without naming an owner.
pub const OPERATOR_DURABLE_TOPICS: [&str; 4] = [
    "channel:learnings",
    "policy-decisions",
    "framework:pickup",
    "broadcast:global",
];

/// Owner recorded for an operator-durable topic.
pub const OPERATOR_DURABLE_OWNER: &str = "operator";
/// Reason recorded for an operator-durable topic.
pub const OPERATOR_DURABLE_REASON: &str = "operator-durable topic (T-2057, T-3304 IW-2)";

/// Default count bound for mail and high-rate topics.
pub const DEFAULT_MESSAGES: u64 = 1000;
/// Default age bound for ordinary topics (D2).
pub const DEFAULT_DAYS: u32 = 14;
/// Default age bound for test-debris topics (T-2426).
pub const DEBRIS_DAYS: u32 = 7;

/// A default retention, independent of the bus crate's `Retention` type so
/// every surface can use it. `value` is meaningful for `Days`/`Messages` only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultRetention {
    Forever,
    Latest,
    Messages(u64),
    Days(u32),
}

impl DefaultRetention {
    /// The wire form `channel.create` accepts: `{"kind": .., "value": ..}`.
    pub fn to_json(self) -> serde_json::Value {
        match self {
            DefaultRetention::Forever => serde_json::json!({"kind": "forever"}),
            DefaultRetention::Latest => serde_json::json!({"kind": "latest"}),
            DefaultRetention::Messages(n) => serde_json::json!({"kind": "messages", "value": n}),
            DefaultRetention::Days(d) => serde_json::json!({"kind": "days", "value": d}),
        }
    }
}

pub fn is_operator_durable(name: &str) -> bool {
    OPERATOR_DURABLE_TOPICS.contains(&name)
}

/// Mail topics: bounded by count, never by age (D2).
pub fn is_mail_pattern(name: &str) -> bool {
    name.starts_with("inbox:") || name.starts_with("dm:")
}

/// Per-agent / chat streams that already default to a count bound (T-2058).
pub fn is_high_rate_pattern(name: &str) -> bool {
    matches!(name, "agent-presence" | "agent-chat-arc")
        || name.starts_with("agent-listeners-")
        || name.starts_with("agent-conv-")
        || name.starts_with("dm:")
}

/// Single-value state topics (T-2145).
pub fn is_single_value_state_pattern(name: &str) -> bool {
    name.starts_with("state:")
}

/// Test-debris namespaces (T-2426).
pub fn is_debris_pattern(name: &str) -> bool {
    let task_prefixed = |p: &str| {
        name.len() > p.len() && name.starts_with(p) && name.as_bytes()[p.len()].is_ascii_digit()
    };
    task_prefixed("t-")
        || task_prefixed("T-")
        || name.starts_with("xhub-")
        || name.starts_with("stress-")
        || name.starts_with("scratch:")
        || name.starts_with("smoke:")
        || name.starts_with("smoke-")
}

/// The default retention for a new topic named `name` (D2 table, in order).
pub fn default_retention(name: &str) -> DefaultRetention {
    if is_operator_durable(name) {
        DefaultRetention::Forever
    } else if is_single_value_state_pattern(name) {
        DefaultRetention::Latest
    } else if is_mail_pattern(name) || is_high_rate_pattern(name) {
        DefaultRetention::Messages(DEFAULT_MESSAGES)
    } else if is_debris_pattern(name) {
        DefaultRetention::Days(DEBRIS_DAYS)
    } else {
        DefaultRetention::Days(DEFAULT_DAYS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t3310_default_table_matches_the_d2_ruling() {
        for t in OPERATOR_DURABLE_TOPICS {
            assert_eq!(default_retention(t), DefaultRetention::Forever, "{t}");
        }
        assert_eq!(default_retention("state:leader"), DefaultRetention::Latest);
        assert_eq!(default_retention("inbox:abc/010-termlink"), DefaultRetention::Messages(1000));
        assert_eq!(default_retention("dm:a:b"), DefaultRetention::Messages(1000));
        assert_eq!(default_retention("agent-presence"), DefaultRetention::Messages(1000));
        assert_eq!(default_retention("agent-conv-x"), DefaultRetention::Messages(1000));
        assert_eq!(default_retention("T-3310-scratch"), DefaultRetention::Days(7));
        assert_eq!(default_retention("smoke:x"), DefaultRetention::Days(7));
        // Everything else, sidecar included, is 14 days: no name falls through to forever.
        assert_eq!(default_retention("sidecar:010-termlink"), DefaultRetention::Days(14));
        assert_eq!(default_retention("aef-install-findings"), DefaultRetention::Days(14));
        assert_eq!(default_retention("health:ring20-fedprobe"), DefaultRetention::Days(14));
    }

    #[test]
    fn t3310_no_default_is_forever_outside_the_durable_four() {
        for n in ["x", "inbox:", "dm:", "sidecar:a", "agent-chat-arc", "Tx", "t-", "policy"] {
            assert_ne!(default_retention(n), DefaultRetention::Forever, "{n}");
        }
    }

    #[test]
    fn t3310_to_json_is_the_channel_create_wire_form() {
        assert_eq!(DefaultRetention::Days(14).to_json(), serde_json::json!({"kind":"days","value":14}));
        assert_eq!(DefaultRetention::Messages(1000).to_json(), serde_json::json!({"kind":"messages","value":1000}));
        assert_eq!(DefaultRetention::Forever.to_json(), serde_json::json!({"kind":"forever"}));
    }
}
