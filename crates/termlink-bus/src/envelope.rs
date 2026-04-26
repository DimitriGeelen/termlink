use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A single bus message. `payload` is opaque bytes — the codec is caller's
/// choice (per T-1155 §"Open questions deferred"). Identity/signature fields
/// will be added in T-1159.
///
/// `metadata` (T-1287 / T-243) is a routing/filtering hint map — well-known
/// keys include `conversation_id` (groups messages into a multi-turn dialog)
/// and `event_type` (one of `turn`, `typing`, `receipt`, `presence`, `member`
/// per the T-1288 catalog). Metadata is NOT included in canonical signed
/// bytes — trusted-mesh threat model treats it as routing only. Future task
/// can promote to signed if threat model expands.
///
/// `BTreeMap` (over `HashMap`) for deterministic iteration order across runs
/// and platforms. `#[serde(default)]` keeps deserialization backwards-compat
/// with envelopes written before the field existed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub topic: String,
    pub sender_id: String,
    pub msg_type: String,
    pub payload: Vec<u8>,
    pub artifact_ref: Option<String>,
    pub ts_unix_ms: i64,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}
