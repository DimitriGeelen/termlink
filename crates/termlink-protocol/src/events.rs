//! Typed event schemas for the TermLink delegation protocol.
//!
//! All event payloads include a `schema_version` field for forward compatibility.
//! Events without `schema_version` are treated as v1.0 (backward compatibility).

use serde::{Deserialize, Serialize};

/// Current schema version for event payloads.
pub const SCHEMA_VERSION: &str = "1.0";

/// Error codes for task failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Process crashed or was killed.
    Crash,
    /// Execution exceeded time limit.
    Timeout,
    /// Input validation failed.
    Validation,
    /// Dependency or prerequisite not met.
    Dependency,
    /// Explicitly rejected by the target session.
    Rejected,
    /// Cause could not be determined.
    Unknown,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Crash => write!(f, "CRASH"),
            Self::Timeout => write!(f, "TIMEOUT"),
            Self::Validation => write!(f, "VALIDATION"),
            Self::Dependency => write!(f, "DEPENDENCY"),
            Self::Rejected => write!(f, "REJECTED"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Payload for `task.delegate` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDelegate {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub task_id: String,
    pub command: String,
    #[serde(default)]
    pub args: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// Payload for `task.accepted` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAccepted {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Payload for `task.progress` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub task_id: String,
    /// Progress percentage (0-100). None if indeterminate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Payload for `task.completed` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompleted {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub task_id: String,
    #[serde(default)]
    pub result: serde_json::Value,
}

/// Payload for `task.failed` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailed {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub task_id: String,
    #[serde(default = "default_error_code")]
    pub error_code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Whether the caller should retry.
    #[serde(default)]
    pub retryable: bool,
}

/// Payload for `task.cancelled` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCancelled {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_by: Option<String>,
}

// --- Agent Message Protocol ---
// General-purpose agent-to-agent request/response over events.

/// Topic constants for agent messaging.
pub mod agent_topic {
    pub const REQUEST: &str = "agent.request";
    pub const RESPONSE: &str = "agent.response";
    pub const STATUS: &str = "agent.status";
}

/// Response status for agent requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Ok,
    Error,
}

/// Payload for `agent.request` events.
///
/// Sent by an agent to request an action from another agent.
/// The `request_id` is used to correlate responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Unique request identifier (ULID recommended).
    pub request_id: String,
    /// Session ID or name of the sender.
    pub from: String,
    /// Session ID or name of the target.
    pub to: String,
    /// Action to perform (e.g., "query.status", "file.get", "task.run").
    pub action: String,
    /// Action-specific parameters.
    #[serde(default)]
    pub params: serde_json::Value,
    /// Optional timeout in seconds. Target should abandon after this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// Payload for `agent.response` events.
///
/// Sent by the target agent in response to an `agent.request`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Matches the `request_id` from the corresponding `AgentRequest`.
    pub request_id: String,
    /// Session ID or name of the responder.
    pub from: String,
    /// Whether the request succeeded or failed.
    pub status: ResponseStatus,
    /// Result payload on success.
    #[serde(default)]
    pub result: serde_json::Value,
    /// Error description on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Payload for `agent.status` events.
///
/// Sent by the target agent to report progress on an in-flight request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Matches the `request_id` from the corresponding `AgentRequest`.
    pub request_id: String,
    /// Session ID or name of the agent reporting status.
    pub from: String,
    /// Current phase (e.g., "accepted", "running", "finalizing").
    pub phase: String,
    /// Human-readable status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Progress percentage (0-100). None if indeterminate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent: Option<u8>,
}

// --- Negotiation Protocol (T-240) ---
// 4-phase format negotiation over agent events.
// Built on agent.request/response/status — uses the `action` field to distinguish phases.
// Orchestrator brokers introduction (phase 1), then agent and specialist talk directly (phases 2-4).

/// Topic constants for negotiation protocol.
///
/// Negotiation messages are carried as `agent.request`/`agent.response` events
/// with these action values. The `request_id` ties all phases of one negotiation together.
pub mod negotiate_topic {
    /// Orchestrator → Agent: introduce specialist and format schema.
    pub const OFFER: &str = "negotiate.offer";
    /// Agent → Specialist: submit a draft for validation.
    pub const ATTEMPT: &str = "negotiate.attempt";
    /// Specialist → Agent: accept or correct the draft.
    pub const CORRECTION: &str = "negotiate.correction";
    /// Specialist → Agent: negotiation complete, final schema confirmed.
    pub const ACCEPT: &str = "negotiate.accept";
}

/// Maximum number of correction rounds before negotiation fails.
pub const NEGOTIATE_MAX_ROUNDS: u8 = 5;

/// Phase 1: Orchestrator introduces the specialist and expected format.
///
/// Sent as an `agent.request` with `action: "negotiate.offer"` in `params`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiateOffer {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Session ID of the specialist to negotiate with.
    pub specialist_id: String,
    /// Display name of the specialist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specialist_name: Option<String>,
    /// JSON Schema describing the expected format.
    pub format_schema: serde_json::Value,
    /// Example of a valid payload (for the agent to reference).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    /// Semantic constraints that can't be expressed in JSON Schema alone.
    #[serde(default)]
    pub constraints: Vec<String>,
    /// Format identifier (e.g., "specialist/report-v2").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_id: Option<String>,
}

/// Phase 2: Agent submits a draft to the specialist for validation.
///
/// Sent as an `agent.request` with `action: "negotiate.attempt"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiateAttempt {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// The draft payload for validation.
    pub draft: serde_json::Value,
    /// Questions the agent has about the format (optional).
    #[serde(default)]
    pub questions: Vec<String>,
    /// Which round this is (1-based, for tracking convergence).
    #[serde(default = "default_round")]
    pub round: u8,
}

/// A single correction item from the specialist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionFix {
    /// JSON path to the problematic field (e.g., "findings[0].ref").
    pub field: String,
    /// What the specialist expected.
    pub expected: String,
    /// What the agent provided.
    pub got: String,
    /// Hint for how to fix it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Phase 3: Specialist corrects the agent's draft.
///
/// Sent as an `agent.response` with `action: "negotiate.correction"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiateCorrection {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Whether the draft was accepted (true = done, false = revise and resubmit).
    pub accepted: bool,
    /// Specific fixes needed (empty if accepted).
    #[serde(default)]
    pub fixes: Vec<CorrectionFix>,
    /// Updated schema if the specialist revised its expectations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_schema: Option<serde_json::Value>,
    /// Error message if negotiation failed (max rounds exceeded, impasse).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Phase 4: Specialist accepts the draft — negotiation complete.
///
/// Sent as an `agent.response` with `action: "negotiate.accept"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiateAccept {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// The final agreed schema (may differ from the original offer).
    pub final_schema: serde_json::Value,
    /// Optional template for future interactions (cacheable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<serde_json::Value>,
    /// Format identifier for cache key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_id: Option<String>,
}

/// Negotiation state machine — tracks progress through the 4-phase protocol.
#[derive(Debug, Clone)]
pub struct NegotiationState {
    /// Shared request_id across all phases.
    pub request_id: String,
    /// Current phase of negotiation.
    pub phase: NegotiatePhase,
    /// Number of correction rounds completed.
    pub round: u8,
    /// Maximum allowed rounds.
    pub max_rounds: u8,
    /// Specialist session ID.
    pub specialist_id: String,
    /// Current schema being negotiated.
    pub current_schema: serde_json::Value,
    /// Accumulated corrections from all rounds.
    pub corrections: Vec<CorrectionFix>,
}

/// Phases of the negotiation protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiatePhase {
    /// Offer received, ready to attempt.
    OfferReceived,
    /// Draft submitted, waiting for correction.
    AttemptSent,
    /// Correction received, preparing next attempt.
    CorrectionReceived,
    /// Negotiation complete — accepted.
    Accepted,
    /// Negotiation failed — max rounds or impasse.
    Failed,
}

impl std::fmt::Display for NegotiatePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OfferReceived => write!(f, "offer-received"),
            Self::AttemptSent => write!(f, "attempt-sent"),
            Self::CorrectionReceived => write!(f, "correction-received"),
            Self::Accepted => write!(f, "accepted"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl NegotiationState {
    /// Create a new negotiation from an offer.
    pub fn from_offer(request_id: &str, offer: &NegotiateOffer) -> Self {
        Self {
            request_id: request_id.to_string(),
            phase: NegotiatePhase::OfferReceived,
            round: 0,
            max_rounds: NEGOTIATE_MAX_ROUNDS,
            specialist_id: offer.specialist_id.clone(),
            current_schema: offer.format_schema.clone(),
            corrections: Vec::new(),
        }
    }

    /// Record that an attempt was sent.
    pub fn record_attempt(&mut self) -> Result<(), NegotiateError> {
        if self.phase == NegotiatePhase::Accepted || self.phase == NegotiatePhase::Failed {
            return Err(NegotiateError::AlreadyTerminated);
        }
        if self.round >= self.max_rounds {
            self.phase = NegotiatePhase::Failed;
            return Err(NegotiateError::MaxRoundsExceeded);
        }
        self.round += 1;
        self.phase = NegotiatePhase::AttemptSent;
        Ok(())
    }

    /// Process a correction from the specialist.
    pub fn record_correction(&mut self, correction: &NegotiateCorrection) -> Result<(), NegotiateError> {
        if self.phase != NegotiatePhase::AttemptSent {
            return Err(NegotiateError::UnexpectedPhase {
                expected: NegotiatePhase::AttemptSent,
                got: self.phase,
            });
        }

        self.corrections.extend(correction.fixes.clone());

        if correction.accepted {
            self.phase = NegotiatePhase::Accepted;
        } else {
            self.phase = NegotiatePhase::CorrectionReceived;
            if let Some(ref revised) = correction.revised_schema {
                self.current_schema = revised.clone();
            }
        }

        Ok(())
    }

    /// Process an accept from the specialist.
    pub fn record_accept(&mut self, accept: &NegotiateAccept) {
        self.phase = NegotiatePhase::Accepted;
        self.current_schema = accept.final_schema.clone();
    }

    /// Check if negotiation is still in progress.
    pub fn is_active(&self) -> bool {
        !matches!(self.phase, NegotiatePhase::Accepted | NegotiatePhase::Failed)
    }

    /// Check if negotiation succeeded.
    pub fn is_accepted(&self) -> bool {
        self.phase == NegotiatePhase::Accepted
    }
}

/// Errors during negotiation state transitions.
#[derive(Debug, Clone)]
pub enum NegotiateError {
    /// Negotiation already completed (accepted or failed).
    AlreadyTerminated,
    /// Exceeded the maximum number of correction rounds.
    MaxRoundsExceeded,
    /// Received a message in an unexpected phase.
    UnexpectedPhase {
        expected: NegotiatePhase,
        got: NegotiatePhase,
    },
}

impl std::fmt::Display for NegotiateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyTerminated => write!(f, "negotiation already terminated"),
            Self::MaxRoundsExceeded => write!(f, "exceeded max negotiation rounds ({NEGOTIATE_MAX_ROUNDS})"),
            Self::UnexpectedPhase { expected, got } => {
                write!(f, "unexpected phase: expected {expected}, got {got}")
            }
        }
    }
}

fn default_round() -> u8 {
    1
}

// --- File Transfer Protocol ---
// Chunked file transfer over events with base64 encoding and SHA-256 integrity.

/// Topic constants for file transfer.
pub mod file_topic {
    pub const INIT: &str = "file.init";
    pub const CHUNK: &str = "file.chunk";
    pub const COMPLETE: &str = "file.complete";
    pub const ERROR: &str = "file.error";
}

/// Payload for `file.init` events — announces a new file transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInit {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Unique transfer identifier for correlating chunks.
    pub transfer_id: String,
    /// Original filename (basename only, no path).
    pub filename: String,
    /// Total file size in bytes.
    pub size: u64,
    /// Number of chunks that will follow.
    pub total_chunks: u32,
    /// Session name of the sender.
    pub from: String,
}

/// Payload for `file.chunk` events — one chunk of file data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Matches `transfer_id` from `FileInit`.
    pub transfer_id: String,
    /// Zero-based chunk index.
    pub index: u32,
    /// Base64-encoded chunk data.
    pub data: String,
}

/// Payload for `file.complete` events — signals transfer finished.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileComplete {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Matches `transfer_id` from `FileInit`.
    pub transfer_id: String,
    /// Hex-encoded SHA-256 of the complete file.
    pub sha256: String,
}

/// Payload for `file.error` events — signals transfer failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileError {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Matches `transfer_id` from `FileInit`.
    pub transfer_id: String,
    /// Human-readable error message.
    pub message: String,
}

fn default_schema_version() -> String {
    SCHEMA_VERSION.to_string()
}

fn default_error_code() -> ErrorCode {
    ErrorCode::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_progress_roundtrip() {
        let progress = TaskProgress {
            schema_version: SCHEMA_VERSION.to_string(),
            task_id: "T-001".to_string(),
            percent: Some(42),
            message: Some("Building crate...".to_string()),
        };
        let json = serde_json::to_string(&progress).unwrap();
        let parsed: TaskProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.percent, Some(42));
        assert_eq!(parsed.schema_version, "1.0");
    }

    #[test]
    fn task_cancelled_roundtrip() {
        let cancelled = TaskCancelled {
            schema_version: SCHEMA_VERSION.to_string(),
            task_id: "T-002".to_string(),
            reason: Some("User interrupt".to_string()),
            cancelled_by: Some("orchestrator".to_string()),
        };
        let json = serde_json::to_string(&cancelled).unwrap();
        let parsed: TaskCancelled = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.reason.as_deref(), Some("User interrupt"));
        assert_eq!(parsed.cancelled_by.as_deref(), Some("orchestrator"));
    }

    #[test]
    fn task_failed_with_error_code() {
        let failed = TaskFailed {
            schema_version: SCHEMA_VERSION.to_string(),
            task_id: "T-003".to_string(),
            error_code: ErrorCode::Timeout,
            message: Some("Exceeded 30s limit".to_string()),
            retryable: true,
        };
        let json = serde_json::to_string(&failed).unwrap();
        assert!(json.contains("\"TIMEOUT\""));
        let parsed: TaskFailed = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.error_code, ErrorCode::Timeout);
        assert!(parsed.retryable);
    }

    #[test]
    fn backward_compat_missing_schema_version() {
        // V1 events won't have schema_version — should default to "1.0"
        let json = r#"{"task_id": "T-004", "result": {}}"#;
        let completed: TaskCompleted = serde_json::from_str(json).unwrap();
        assert_eq!(completed.schema_version, "1.0");
    }

    #[test]
    fn backward_compat_missing_error_code() {
        // V1 task.failed won't have error_code — should default to UNKNOWN
        let json = r#"{"task_id": "T-005", "message": "crashed"}"#;
        let failed: TaskFailed = serde_json::from_str(json).unwrap();
        assert_eq!(failed.error_code, ErrorCode::Unknown);
        assert_eq!(failed.schema_version, "1.0");
    }

    #[test]
    fn error_code_serialization() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::Crash).unwrap(),
            "\"CRASH\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::Validation).unwrap(),
            "\"VALIDATION\""
        );
    }

    #[test]
    fn task_delegate_full() {
        let delegate = TaskDelegate {
            schema_version: SCHEMA_VERSION.to_string(),
            task_id: "T-006".to_string(),
            command: "build".to_string(),
            args: serde_json::json!({"crate": "termlink-session"}),
            timeout_secs: Some(300),
        };
        let json = serde_json::to_string_pretty(&delegate).unwrap();
        assert!(json.contains("schema_version"));
        assert!(json.contains("1.0"));
    }

    // --- Agent Message Protocol tests ---

    #[test]
    fn agent_request_roundtrip() {
        let req = AgentRequest {
            schema_version: SCHEMA_VERSION.to_string(),
            request_id: "01HXYZ123456".to_string(),
            from: "agent-orchestrator".to_string(),
            to: "agent-worker-1".to_string(),
            action: "task.run".to_string(),
            params: serde_json::json!({"command": "cargo test", "cwd": "/project"}),
            timeout_secs: Some(120),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: AgentRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.request_id, "01HXYZ123456");
        assert_eq!(parsed.from, "agent-orchestrator");
        assert_eq!(parsed.to, "agent-worker-1");
        assert_eq!(parsed.action, "task.run");
        assert_eq!(parsed.timeout_secs, Some(120));
    }

    #[test]
    fn agent_response_ok_roundtrip() {
        let resp = AgentResponse {
            schema_version: SCHEMA_VERSION.to_string(),
            request_id: "01HXYZ123456".to_string(),
            from: "agent-worker-1".to_string(),
            status: ResponseStatus::Ok,
            result: serde_json::json!({"exit_code": 0, "output": "all tests passed"}),
            error_message: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, ResponseStatus::Ok);
        assert_eq!(parsed.result["exit_code"], 0);
        assert!(parsed.error_message.is_none());
    }

    #[test]
    fn agent_response_error_roundtrip() {
        let resp = AgentResponse {
            schema_version: SCHEMA_VERSION.to_string(),
            request_id: "01HXYZ123456".to_string(),
            from: "agent-worker-1".to_string(),
            status: ResponseStatus::Error,
            result: serde_json::json!({}),
            error_message: Some("command not found".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, ResponseStatus::Error);
        assert_eq!(parsed.error_message.as_deref(), Some("command not found"));
    }

    #[test]
    fn agent_status_roundtrip() {
        let status = AgentStatus {
            schema_version: SCHEMA_VERSION.to_string(),
            request_id: "01HXYZ123456".to_string(),
            from: "agent-worker-1".to_string(),
            phase: "running".to_string(),
            message: Some("Compiling crate 3/5...".to_string()),
            percent: Some(60),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: AgentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.phase, "running");
        assert_eq!(parsed.percent, Some(60));
        assert_eq!(parsed.message.as_deref(), Some("Compiling crate 3/5..."));
    }

    #[test]
    fn agent_request_minimal() {
        // Minimal request without optional fields
        let json = r#"{"request_id":"r1","from":"a","to":"b","action":"ping"}"#;
        let parsed: AgentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.schema_version, "1.0");
        assert_eq!(parsed.action, "ping");
        assert!(parsed.timeout_secs.is_none());
        assert_eq!(parsed.params, serde_json::json!(null));
    }

    #[test]
    fn agent_topic_constants() {
        assert_eq!(agent_topic::REQUEST, "agent.request");
        assert_eq!(agent_topic::RESPONSE, "agent.response");
        assert_eq!(agent_topic::STATUS, "agent.status");
    }

    #[test]
    fn response_status_serialization() {
        assert_eq!(serde_json::to_string(&ResponseStatus::Ok).unwrap(), "\"ok\"");
        assert_eq!(serde_json::to_string(&ResponseStatus::Error).unwrap(), "\"error\"");
    }

    // --- File Transfer Protocol tests ---

    #[test]
    fn file_init_roundtrip() {
        let init = FileInit {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: "xfer-001".to_string(),
            filename: "test.bin".to_string(),
            size: 102400,
            total_chunks: 3,
            from: "sender-session".to_string(),
        };
        let json = serde_json::to_string(&init).unwrap();
        let parsed: FileInit = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.transfer_id, "xfer-001");
        assert_eq!(parsed.filename, "test.bin");
        assert_eq!(parsed.size, 102400);
        assert_eq!(parsed.total_chunks, 3);
    }

    #[test]
    fn file_chunk_roundtrip() {
        let chunk = FileChunk {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: "xfer-001".to_string(),
            index: 0,
            data: "SGVsbG8gV29ybGQ=".to_string(),
        };
        let json = serde_json::to_string(&chunk).unwrap();
        let parsed: FileChunk = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.index, 0);
        assert_eq!(parsed.data, "SGVsbG8gV29ybGQ=");
    }

    #[test]
    fn file_complete_roundtrip() {
        let complete = FileComplete {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: "xfer-001".to_string(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        };
        let json = serde_json::to_string(&complete).unwrap();
        let parsed: FileComplete = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.sha256, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn file_error_roundtrip() {
        let error = FileError {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: "xfer-001".to_string(),
            message: "Disk full".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        let parsed: FileError = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message, "Disk full");
    }

    #[test]
    fn file_topic_constants() {
        assert_eq!(file_topic::INIT, "file.init");
        assert_eq!(file_topic::CHUNK, "file.chunk");
        assert_eq!(file_topic::COMPLETE, "file.complete");
        assert_eq!(file_topic::ERROR, "file.error");
    }

    // --- Negotiation Protocol tests ---

    #[test]
    fn negotiate_topic_constants() {
        assert_eq!(negotiate_topic::OFFER, "negotiate.offer");
        assert_eq!(negotiate_topic::ATTEMPT, "negotiate.attempt");
        assert_eq!(negotiate_topic::CORRECTION, "negotiate.correction");
        assert_eq!(negotiate_topic::ACCEPT, "negotiate.accept");
    }

    #[test]
    fn negotiate_offer_roundtrip() {
        let offer = NegotiateOffer {
            schema_version: SCHEMA_VERSION.to_string(),
            specialist_id: "git-specialist-01".to_string(),
            specialist_name: Some("git-specialist".to_string()),
            format_schema: serde_json::json!({
                "type": "object",
                "required": ["title", "findings"],
                "properties": {
                    "title": {"type": "string"},
                    "findings": {"type": "array"}
                }
            }),
            example: Some(serde_json::json!({"title": "Report", "findings": []})),
            constraints: vec!["findings must reference file:line".to_string()],
            format_id: Some("specialist/report-v2".to_string()),
        };
        let json = serde_json::to_string(&offer).unwrap();
        let parsed: NegotiateOffer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.specialist_id, "git-specialist-01");
        assert_eq!(parsed.constraints.len(), 1);
        assert_eq!(parsed.format_id.as_deref(), Some("specialist/report-v2"));
    }

    #[test]
    fn negotiate_attempt_roundtrip() {
        let attempt = NegotiateAttempt {
            schema_version: SCHEMA_VERSION.to_string(),
            draft: serde_json::json!({"title": "My Report", "findings": [{"ref": "main.rs:42"}]}),
            questions: vec!["Is severity required?".to_string()],
            round: 1,
        };
        let json = serde_json::to_string(&attempt).unwrap();
        let parsed: NegotiateAttempt = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.round, 1);
        assert_eq!(parsed.questions.len(), 1);
    }

    #[test]
    fn negotiate_correction_reject_roundtrip() {
        let correction = NegotiateCorrection {
            schema_version: SCHEMA_VERSION.to_string(),
            accepted: false,
            fixes: vec![
                CorrectionFix {
                    field: "findings[0].ref".to_string(),
                    expected: "file:line format".to_string(),
                    got: "line 42".to_string(),
                    hint: Some("prefix with filename".to_string()),
                },
            ],
            revised_schema: None,
            error_message: None,
        };
        let json = serde_json::to_string(&correction).unwrap();
        let parsed: NegotiateCorrection = serde_json::from_str(&json).unwrap();
        assert!(!parsed.accepted);
        assert_eq!(parsed.fixes.len(), 1);
        assert_eq!(parsed.fixes[0].field, "findings[0].ref");
    }

    #[test]
    fn negotiate_correction_accept_roundtrip() {
        let correction = NegotiateCorrection {
            schema_version: SCHEMA_VERSION.to_string(),
            accepted: true,
            fixes: vec![],
            revised_schema: None,
            error_message: None,
        };
        let json = serde_json::to_string(&correction).unwrap();
        let parsed: NegotiateCorrection = serde_json::from_str(&json).unwrap();
        assert!(parsed.accepted);
        assert!(parsed.fixes.is_empty());
    }

    #[test]
    fn negotiate_accept_roundtrip() {
        let accept = NegotiateAccept {
            schema_version: SCHEMA_VERSION.to_string(),
            final_schema: serde_json::json!({"type": "object", "required": ["title"]}),
            template: Some(serde_json::json!({"title": "{{title}}", "findings": []})),
            format_id: Some("specialist/report-v2".to_string()),
        };
        let json = serde_json::to_string(&accept).unwrap();
        let parsed: NegotiateAccept = serde_json::from_str(&json).unwrap();
        assert!(parsed.template.is_some());
        assert_eq!(parsed.format_id.as_deref(), Some("specialist/report-v2"));
    }

    #[test]
    fn negotiation_state_happy_path() {
        let offer = NegotiateOffer {
            schema_version: SCHEMA_VERSION.to_string(),
            specialist_id: "specialist-01".to_string(),
            specialist_name: None,
            format_schema: serde_json::json!({"type": "object"}),
            example: None,
            constraints: vec![],
            format_id: None,
        };
        let mut state = NegotiationState::from_offer("req-001", &offer);
        assert_eq!(state.phase, NegotiatePhase::OfferReceived);
        assert!(state.is_active());

        // Round 1: attempt → correction (rejected)
        state.record_attempt().unwrap();
        assert_eq!(state.phase, NegotiatePhase::AttemptSent);
        assert_eq!(state.round, 1);

        let correction = NegotiateCorrection {
            schema_version: SCHEMA_VERSION.to_string(),
            accepted: false,
            fixes: vec![CorrectionFix {
                field: "title".to_string(),
                expected: "string".to_string(),
                got: "null".to_string(),
                hint: None,
            }],
            revised_schema: None,
            error_message: None,
        };
        state.record_correction(&correction).unwrap();
        assert_eq!(state.phase, NegotiatePhase::CorrectionReceived);
        assert_eq!(state.corrections.len(), 1);

        // Round 2: attempt → correction (accepted)
        state.record_attempt().unwrap();
        assert_eq!(state.round, 2);

        let accept_correction = NegotiateCorrection {
            schema_version: SCHEMA_VERSION.to_string(),
            accepted: true,
            fixes: vec![],
            revised_schema: None,
            error_message: None,
        };
        state.record_correction(&accept_correction).unwrap();
        assert_eq!(state.phase, NegotiatePhase::Accepted);
        assert!(!state.is_active());
        assert!(state.is_accepted());
    }

    #[test]
    fn negotiation_state_max_rounds() {
        let offer = NegotiateOffer {
            schema_version: SCHEMA_VERSION.to_string(),
            specialist_id: "specialist-01".to_string(),
            specialist_name: None,
            format_schema: serde_json::json!({}),
            example: None,
            constraints: vec![],
            format_id: None,
        };
        let mut state = NegotiationState::from_offer("req-002", &offer);
        state.max_rounds = 2; // Override for test

        // Round 1
        state.record_attempt().unwrap();
        let reject = NegotiateCorrection {
            schema_version: SCHEMA_VERSION.to_string(),
            accepted: false,
            fixes: vec![],
            revised_schema: None,
            error_message: None,
        };
        state.record_correction(&reject).unwrap();

        // Round 2
        state.record_attempt().unwrap();
        state.record_correction(&reject).unwrap();

        // Round 3 — should fail
        let result = state.record_attempt();
        assert!(result.is_err());
        assert_eq!(state.phase, NegotiatePhase::Failed);
        assert!(!state.is_active());
        assert!(!state.is_accepted());
    }

    #[test]
    fn negotiation_state_revised_schema() {
        let offer = NegotiateOffer {
            schema_version: SCHEMA_VERSION.to_string(),
            specialist_id: "specialist-01".to_string(),
            specialist_name: None,
            format_schema: serde_json::json!({"version": 1}),
            example: None,
            constraints: vec![],
            format_id: None,
        };
        let mut state = NegotiationState::from_offer("req-003", &offer);

        state.record_attempt().unwrap();
        let correction = NegotiateCorrection {
            schema_version: SCHEMA_VERSION.to_string(),
            accepted: false,
            fixes: vec![],
            revised_schema: Some(serde_json::json!({"version": 2})),
            error_message: None,
        };
        state.record_correction(&correction).unwrap();
        assert_eq!(state.current_schema, serde_json::json!({"version": 2}));
    }

    #[test]
    fn negotiation_state_accept_shortcut() {
        let offer = NegotiateOffer {
            schema_version: SCHEMA_VERSION.to_string(),
            specialist_id: "specialist-01".to_string(),
            specialist_name: None,
            format_schema: serde_json::json!({}),
            example: None,
            constraints: vec![],
            format_id: None,
        };
        let mut state = NegotiationState::from_offer("req-004", &offer);

        let accept = NegotiateAccept {
            schema_version: SCHEMA_VERSION.to_string(),
            final_schema: serde_json::json!({"final": true}),
            template: None,
            format_id: None,
        };
        state.record_accept(&accept);
        assert!(state.is_accepted());
        assert_eq!(state.current_schema, serde_json::json!({"final": true}));
    }

    #[test]
    fn negotiate_phase_display() {
        assert_eq!(NegotiatePhase::OfferReceived.to_string(), "offer-received");
        assert_eq!(NegotiatePhase::AttemptSent.to_string(), "attempt-sent");
        assert_eq!(NegotiatePhase::CorrectionReceived.to_string(), "correction-received");
        assert_eq!(NegotiatePhase::Accepted.to_string(), "accepted");
        assert_eq!(NegotiatePhase::Failed.to_string(), "failed");
    }
}
