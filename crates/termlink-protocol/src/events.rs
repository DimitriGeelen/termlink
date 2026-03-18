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
}
