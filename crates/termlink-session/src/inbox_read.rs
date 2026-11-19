//! T-3700 (arc-011 P-12, AL-30, operator ruling E 2026-11-19): the bytes an inbox reader signs.
//!
//! A sidecar names itself on every read and claim of its inbox and signs these bytes with its key-card
//! key; the hub checks them (`termlink_hub::inbox_consumers`). The domain prefix keeps a read proof from
//! ever verifying as a post signature; the hub id keeps it from being replayed on another hub.

/// Domain prefix of the read-proof bytes.
pub const READ_DOMAIN: &str = "termlink-inbox-read/v1";

/// The bytes signed for one read (`method` = `channel.subscribe` with the cursor, or `channel.claim` with the
/// offset).
pub fn read_sign_bytes(hub_id: &str, topic: &str, method: &str, position: u64, ts_ms: i64, agent: &str, instance: &str) -> Vec<u8> {
    format!("{READ_DOMAIN}\n{hub_id}\n{topic}\n{method}\n{position}\n{ts_ms}\n{agent}\n{instance}").into_bytes()
}
