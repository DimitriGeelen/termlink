//! Governance event payload for the data plane Governance frame type (0x8).
//!
//! Governance frames are informational (audit trail, metrics) — not enforcement.
//! A [`GovernanceEvent`] is serialized as JSON into the frame payload.

use serde::{Deserialize, Serialize};

/// A governance event emitted when a pattern matches output text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceEvent {
    /// Name of the pattern that matched (from config).
    pub pattern_name: String,
    /// The text that matched the pattern.
    pub match_text: String,
    /// Unix timestamp (seconds) when the match was detected.
    pub timestamp: u64,
    /// Channel ID of the output frame that triggered the match.
    pub channel_id: u32,
}

impl GovernanceEvent {
    /// Serialize this event as a JSON payload for a Governance frame.
    pub fn to_payload(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("GovernanceEvent is always serializable")
    }

    /// Deserialize a GovernanceEvent from a frame payload.
    pub fn from_payload(payload: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_governance_event() {
        let event = GovernanceEvent {
            pattern_name: "secret_leak".into(),
            match_text: "AWS_SECRET_ACCESS_KEY=AKIA...".into(),
            timestamp: 1712345678,
            channel_id: 1,
        };
        let payload = event.to_payload();
        let decoded = GovernanceEvent::from_payload(&payload).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn governance_event_json_format() {
        let event = GovernanceEvent {
            pattern_name: "error".into(),
            match_text: "FATAL ERROR".into(),
            timestamp: 1000,
            channel_id: 0,
        };
        let json: serde_json::Value = serde_json::from_slice(&event.to_payload()).unwrap();
        assert_eq!(json["pattern_name"], "error");
        assert_eq!(json["match_text"], "FATAL ERROR");
        assert_eq!(json["timestamp"], 1000);
        assert_eq!(json["channel_id"], 0);
    }

    #[test]
    fn from_payload_invalid_json() {
        assert!(GovernanceEvent::from_payload(b"not json").is_err());
    }
}
