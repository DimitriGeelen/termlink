//! Typed helpers for the T-2838 assignment / result-manifest payload contract.
//!
//! T-2838 decomposition item 3. The schemas were drafted and round-tripped in
//! spike S3 (see `docs/reports/T-2838-delivery-to-turn-contract.md`); this
//! module codifies them so callers stop hand-rolling the JSON.
//!
//! # This is a PAYLOAD contract, not a wire type
//!
//! S3's finding was that no protocol change is needed. `msg_type` is a
//! free-form string, `--reply-to` already sets `metadata.in_reply_to`
//! (T-1313), and `artifact_ref` is carried inside canonical signed bytes. The
//! schema therefore lives entirely in the payload, versioned by its own
//! `schema` field, and a reader that does not recognise it skips by `msg_type`.
//! That is why this module lives in `termlink-session` next to `ack_retry`
//! rather than in `termlink-protocol` — nothing here touches the wire.
//!
//! # Why not reuse `artifact::ArtifactManifest`
//!
//! That type describes a byte TRANSFER (sha256 + total_bytes + `SendPath`) for
//! the inline/chunked file-send path. [`ArtifactRef`] describes a byte
//! REFERENCE: it names bytes that already exist on a shared filesystem and are
//! never put on the bus. Charter non-goal #2 — manifests reference artifacts
//! and never archive them. Conflating the two would make it possible to
//! accidentally ship content through a manifest, which is the thing the
//! non-goal forbids.
//!
//! # Unknown values are preserved, never coerced
//!
//! [`ResultStatus`] keeps an unrecognised status as [`ResultStatus::Other`]
//! rather than mapping it to a plausible-looking default. A default that looks
//! valid is the dangerous kind: the same class as reporting `ready:false` for a
//! registration nobody checked (T-2550), or `delivered` for bytes nobody read
//! (T-2838 item 4). If a producer says something this version does not model,
//! the honest encoding is to carry the string through.

use serde::{Deserialize, Serialize};

/// Schema tag for [`Assignment`]. Emitted and REQUIRED on parse.
pub const SCHEMA_ASSIGNMENT: &str = "termlink.assignment.v0";
/// Schema tag for [`ResultManifest`]. Emitted and REQUIRED on parse.
pub const SCHEMA_RESULT_MANIFEST: &str = "termlink.result_manifest.v0";

/// Why a payload could not be read as one of these envelopes.
///
/// `SchemaMismatch` is deliberately distinct from `Malformed`: "this is a
/// well-formed envelope of a different kind" and "this is not parseable" are
/// different facts, and a caller routing by `msg_type` needs to tell them apart
/// rather than treating every non-match as corruption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Valid JSON, but `schema` is absent or names a different contract.
    SchemaMismatch { expected: &'static str, found: Option<String> },
    /// Not valid JSON for this shape.
    Malformed(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::SchemaMismatch { expected, found } => match found {
                Some(s) => write!(f, "expected schema {expected}, found {s}"),
                None => write!(f, "expected schema {expected}, payload has no `schema` field"),
            },
            ParseError::Malformed(e) => write!(f, "malformed payload: {e}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// A reference to bytes that already exist. NEVER carries content.
///
/// `bytes` is the producer's byte count and `sha256` the producer's digest; a
/// consumer on the same filesystem re-reads `path` and can verify it got
/// exactly what the producer meant. The manifest stays O(number of artifacts),
/// not O(bytes) — S3 measured a 584-byte manifest describing 28,835 bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
}

/// What an assignment asks for, in capability terms rather than by naming a
/// specific agent — the issuer generally cannot know which agent will take it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AgentProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub min_capabilities: Vec<String>,
}

/// `termlink.assignment.v0` — posted with `--msg-type assignment`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    pub schema: String,
    pub assignment_id: String,
    pub issued_by: String,
    #[serde(default)]
    pub agent_profile: AgentProfile,
    pub task: String,
    /// 0 means "no deadline". Kept as a plain integer rather than an
    /// `Option` because the drafted envelope in S3 uses 0 and round-tripped
    /// that way; changing the encoding now would break the recorded example.
    #[serde(default)]
    pub deadline_unix_ms: i64,
    #[serde(default)]
    pub artifacts_in: Vec<ArtifactRef>,
}

impl Assignment {
    /// Build an assignment with the schema tag already set.
    pub fn new(assignment_id: impl Into<String>, issued_by: impl Into<String>, task: impl Into<String>) -> Self {
        Self {
            schema: SCHEMA_ASSIGNMENT.to_string(),
            assignment_id: assignment_id.into(),
            issued_by: issued_by.into(),
            agent_profile: AgentProfile::default(),
            task: task.into(),
            deadline_unix_ms: 0,
            artifacts_in: Vec::new(),
        }
    }

    /// Parse, REQUIRING the schema tag. A payload of a different kind is
    /// rejected as [`ParseError::SchemaMismatch`], never silently accepted.
    pub fn parse(payload: &[u8]) -> Result<Self, ParseError> {
        let v: serde_json::Value =
            serde_json::from_slice(payload).map_err(|e| ParseError::Malformed(e.to_string()))?;
        check_schema(&v, SCHEMA_ASSIGNMENT)?;
        serde_json::from_value(v).map_err(|e| ParseError::Malformed(e.to_string()))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Assignment serialises")
    }
}

/// Outcome of an assignment. An unrecognised value is PRESERVED, not coerced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultStatus {
    Ok,
    Failed,
    Partial,
    /// A status this version does not model. Carried through verbatim so a
    /// newer producer is never misreported as one of the known values.
    Other(String),
}

impl ResultStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ResultStatus::Ok => "ok",
            ResultStatus::Failed => "failed",
            ResultStatus::Partial => "partial",
            ResultStatus::Other(s) => s,
        }
    }

    /// True only for [`ResultStatus::Ok`]. `Other` is NOT success — an
    /// unmodelled status is unknown, and unknown must not read as ok.
    pub fn is_ok(&self) -> bool {
        matches!(self, ResultStatus::Ok)
    }
}

impl From<&str> for ResultStatus {
    fn from(s: &str) -> Self {
        match s {
            "ok" => ResultStatus::Ok,
            "failed" => ResultStatus::Failed,
            "partial" => ResultStatus::Partial,
            other => ResultStatus::Other(other.to_string()),
        }
    }
}

impl Serialize for ResultStatus {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ResultStatus {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(ResultStatus::from(s.as_str()))
    }
}

/// `termlink.result_manifest.v0` — posted with
/// `--msg-type result_manifest --reply-to <assignment offset>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultManifest {
    pub schema: String,
    pub assignment_id: String,
    /// The bus offset of the assignment this answers. Mirrors
    /// `metadata.in_reply_to`, which `--reply-to` sets on the envelope; carried
    /// in the payload too so the manifest is self-describing when read outside
    /// its envelope.
    pub in_reply_to: u64,
    pub status: ResultStatus,
    #[serde(default)]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default)]
    pub artifacts_out: Vec<ArtifactRef>,
}

impl ResultManifest {
    pub fn new(assignment_id: impl Into<String>, in_reply_to: u64, status: ResultStatus) -> Self {
        Self {
            schema: SCHEMA_RESULT_MANIFEST.to_string(),
            assignment_id: assignment_id.into(),
            in_reply_to,
            status,
            summary: String::new(),
            host: None,
            repo: None,
            artifacts_out: Vec::new(),
        }
    }

    pub fn parse(payload: &[u8]) -> Result<Self, ParseError> {
        let v: serde_json::Value =
            serde_json::from_slice(payload).map_err(|e| ParseError::Malformed(e.to_string()))?;
        check_schema(&v, SCHEMA_RESULT_MANIFEST)?;
        serde_json::from_value(v).map_err(|e| ParseError::Malformed(e.to_string()))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ResultManifest serialises")
    }
}

fn check_schema(v: &serde_json::Value, expected: &'static str) -> Result<(), ParseError> {
    match v.get("schema").and_then(|s| s.as_str()) {
        Some(s) if s == expected => Ok(()),
        Some(s) => Err(ParseError::SchemaMismatch { expected, found: Some(s.to_string()) }),
        None => Err(ParseError::SchemaMismatch { expected, found: None }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ref() -> ArtifactRef {
        ArtifactRef {
            path: "/opt/termlink/docs/reports/x.md".to_string(),
            sha256: "49307347f678df34".to_string(),
            bytes: 14762,
            media_type: Some("text/markdown".to_string()),
        }
    }

    #[test]
    fn assignment_round_trips() {
        let mut a = Assignment::new("AS-0001", "orchestrator", "summarise S4 findings");
        a.agent_profile = AgentProfile {
            role: Some("analyst".to_string()),
            min_capabilities: vec!["read".to_string(), "write".to_string()],
        };
        let back = Assignment::parse(a.to_json().as_bytes()).expect("parses");
        assert_eq!(back, a);
        assert_eq!(back.schema, SCHEMA_ASSIGNMENT);
    }

    #[test]
    fn result_manifest_round_trips_with_artifact_refs() {
        let mut m = ResultManifest::new("AS-0001", 3, ResultStatus::Ok);
        m.summary = "done".to_string();
        m.artifacts_out = vec![sample_ref()];
        let back = ResultManifest::parse(m.to_json().as_bytes()).expect("parses");
        assert_eq!(back, m);
        assert_eq!(back.artifacts_out[0].bytes, 14762);
    }

    #[test]
    fn parse_rejects_a_different_schema_rather_than_accepting_it() {
        // The whole point of the schema field. A result_manifest handed to
        // Assignment::parse is a well-formed envelope of the WRONG KIND, and
        // that is a different fact from "corrupt" — the error says which.
        let m = ResultManifest::new("AS-0001", 3, ResultStatus::Ok);
        match Assignment::parse(m.to_json().as_bytes()) {
            Err(ParseError::SchemaMismatch { expected, found }) => {
                assert_eq!(expected, SCHEMA_ASSIGNMENT);
                assert_eq!(found.as_deref(), Some(SCHEMA_RESULT_MANIFEST));
            }
            other => panic!("expected SchemaMismatch, got {other:?}"),
        }
    }

    #[test]
    fn parse_rejects_a_payload_with_no_schema_field() {
        let err = Assignment::parse(br#"{"assignment_id":"AS-1","issued_by":"x","task":"t"}"#)
            .expect_err("must not accept an untagged payload");
        assert_eq!(
            err,
            ParseError::SchemaMismatch { expected: SCHEMA_ASSIGNMENT, found: None }
        );
    }

    #[test]
    fn unknown_status_is_preserved_and_is_not_ok() {
        // The arc's recurring rule: never coerce unknown into a plausible
        // default. A newer producer's status must survive the round trip AND
        // must not read as success.
        let m = ResultManifest::new("AS-9", 1, ResultStatus::from("cancelled-by-operator"));
        let back = ResultManifest::parse(m.to_json().as_bytes()).expect("parses");
        assert_eq!(back.status, ResultStatus::Other("cancelled-by-operator".to_string()));
        assert!(!back.status.is_ok(), "an unmodelled status must never read as ok");
        assert_eq!(back.status.as_str(), "cancelled-by-operator");
    }

    #[test]
    fn only_ok_is_ok() {
        assert!(ResultStatus::Ok.is_ok());
        assert!(!ResultStatus::Failed.is_ok());
        assert!(!ResultStatus::Partial.is_ok());
        assert!(!ResultStatus::Other("ok-ish".to_string()).is_ok());
    }

    #[test]
    fn unknown_fields_are_ignored_for_forward_compatibility() {
        // A v1 producer adding a field must not break a v0 consumer. This is
        // deliberate: `deny_unknown_fields` would turn every additive schema
        // change into a hard failure at every existing reader.
        let json = format!(
            r#"{{"schema":"{SCHEMA_ASSIGNMENT}","assignment_id":"AS-2","issued_by":"o",
                 "task":"t","future_field":{{"nested":true}}}}"#
        );
        let a = Assignment::parse(json.as_bytes()).expect("v0 reader tolerates v1 fields");
        assert_eq!(a.assignment_id, "AS-2");
    }

    #[test]
    fn malformed_is_distinct_from_schema_mismatch() {
        match Assignment::parse(b"not json at all") {
            Err(ParseError::Malformed(_)) => {}
            other => panic!("expected Malformed, got {other:?}"),
        }
    }

    #[test]
    fn artifact_ref_media_type_is_optional_and_omitted_when_absent() {
        let r = ArtifactRef { media_type: None, ..sample_ref() };
        let json = serde_json::to_string(&r).unwrap();
        assert!(!json.contains("media_type"), "absent optional must not serialise as null");
    }
}
