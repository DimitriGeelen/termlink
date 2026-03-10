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
}
