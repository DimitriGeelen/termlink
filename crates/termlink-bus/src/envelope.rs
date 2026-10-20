//! The bus `Envelope` type: a single message with an opaque payload, as appended to a topic log.
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
    /// T-3486: the poster's ed25519 signature, stored only when the hub verified
    /// it at `channel.post` (see `EnvelopeSignature`). `None` for hub-originated
    /// envelopes and for records written before this field existed. Readers can
    /// recompute `canonical_sign_bytes` from the fields above and verify without
    /// trusting the hub. `metadata` is NOT covered by the signature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<EnvelopeSignature>,
}

/// T-3486: the detached signature a `channel.post` carried, kept as the hex the
/// wire used so this crate needs no ed25519 dependency. The signed bytes are
/// `canonical_sign_bytes(topic, msg_type, payload, artifact_ref, ts_unix_ms)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeSignature {
    pub sender_pubkey_hex: String,
    pub signature_hex: String,
}

#[cfg(test)]
mod t3486_signature_tests {
    use super::*;

    /// A record exactly as written before T-3486 added `signature` — no such key.
    const PRE_T3486_RECORD: &str = r#"{"topic":"t","sender_id":"s","msg_type":"note","payload":[104,105],"artifact_ref":null,"ts_unix_ms":5}"#;

    #[test]
    fn pre_change_record_still_decodes_with_no_signature() {
        let env: Envelope = serde_json::from_str(PRE_T3486_RECORD).expect("old record must decode");
        assert_eq!(env.topic, "t");
        assert_eq!(env.payload, b"hi");
        assert!(env.signature.is_none(), "absent field reads as unsigned");
    }

    #[test]
    fn unsigned_envelope_encodes_without_the_signature_key() {
        let env = Envelope {
            topic: "t".into(),
            sender_id: "s".into(),
            msg_type: "note".into(),
            payload: b"hi".to_vec(),
            artifact_ref: None,
            ts_unix_ms: 5,
            metadata: BTreeMap::new(),
            signature: None,
        };
        let json = serde_json::to_string(&env).unwrap();
        assert!(!json.contains("signature"), "None must not change the on-disk bytes: {json}");
        assert_eq!(json, PRE_T3486_RECORD, "unsigned output is byte-identical to the pre-change record");
    }

    #[test]
    fn signed_envelope_round_trips() {
        let env = Envelope {
            topic: "t".into(),
            sender_id: "s".into(),
            msg_type: "note".into(),
            payload: b"hi".to_vec(),
            artifact_ref: None,
            ts_unix_ms: 5,
            metadata: BTreeMap::new(),
            signature: Some(EnvelopeSignature {
                sender_pubkey_hex: "aa".repeat(32),
                signature_hex: "bb".repeat(64),
            }),
        };
        let back: Envelope = serde_json::from_str(&serde_json::to_string(&env).unwrap()).unwrap();
        assert_eq!(back.signature, env.signature);
    }
}
