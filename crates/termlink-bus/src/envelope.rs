use serde::{Deserialize, Serialize};

/// A single bus message. `payload` is opaque bytes — the codec is caller's
/// choice (per T-1155 §"Open questions deferred"). Identity/signature fields
/// will be added in T-1159.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub topic: String,
    pub sender_id: String,
    pub msg_type: String,
    pub payload: Vec<u8>,
    pub artifact_ref: Option<String>,
    pub ts_unix_ms: i64,
}
