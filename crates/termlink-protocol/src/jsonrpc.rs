use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 successful response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: serde_json::Value,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub error: RpcError,
}

/// Standard JSON-RPC 2.0 error codes.
pub mod standard_error {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
}

impl Request {
    /// Create a new JSON-RPC 2.0 request.
    pub fn new(method: &str, id: serde_json::Value, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: method.into(),
            id: Some(id),
            params,
        }
    }

    /// Create a notification (no id, no response expected).
    pub fn notification(method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: method.into(),
            id: None,
            params,
        }
    }

    /// Whether this is a notification (no response expected).
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

impl Response {
    /// Create a successful response.
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result,
        }
    }
}

impl ErrorResponse {
    /// Create an error response.
    pub fn new(id: serde_json::Value, code: i64, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            error: RpcError {
                code,
                message: message.into(),
                data: None,
            },
        }
    }

    /// Create an error response with additional data.
    pub fn with_data(
        id: serde_json::Value,
        code: i64,
        message: &str,
        data: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            error: RpcError {
                code,
                message: message.into(),
                data: Some(data),
            },
        }
    }

    /// Method not found error.
    pub fn method_not_found(id: serde_json::Value, method: &str) -> Self {
        Self::new(
            id,
            standard_error::METHOD_NOT_FOUND,
            &format!("Method not found: {method}"),
        )
    }

    /// Parse error (malformed JSON).
    pub fn parse_error() -> Self {
        Self::new(
            serde_json::Value::Null,
            standard_error::PARSE_ERROR,
            "Parse error",
        )
    }

    /// Internal error.
    pub fn internal_error(id: serde_json::Value, message: &str) -> Self {
        Self::new(id, standard_error::INTERNAL_ERROR, message)
    }
}

/// Either a success or error response — used for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcResponse {
    Success(Response),
    Error(ErrorResponse),
}

impl From<Response> for RpcResponse {
    fn from(r: Response) -> Self {
        Self::Success(r)
    }
}

impl From<ErrorResponse> for RpcResponse {
    fn from(e: ErrorResponse) -> Self {
        Self::Error(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_serialization() {
        let req = Request::new(
            "query.status",
            json!("req-1"),
            json!({"target": "tl-abc12345"}),
        );
        let json = serde_json::to_string(&req).unwrap();
        let parsed: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.method, "query.status");
        assert_eq!(parsed.id, Some(json!("req-1")));
    }

    #[test]
    fn notification_has_no_id() {
        let notif = Request::notification("event.state_change", json!({"state": "ready"}));
        assert!(notif.is_notification());
        let json = serde_json::to_string(&notif).unwrap();
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn success_response() {
        let resp = Response::success(json!("req-1"), json!({"status": "ok"}));
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, json!("req-1"));
    }

    #[test]
    fn error_response() {
        let err = ErrorResponse::method_not_found(json!("req-1"), "unknown.method");
        let json = serde_json::to_string(&err).unwrap();
        let parsed: ErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.error.code, standard_error::METHOD_NOT_FOUND);
        assert!(parsed.error.message.contains("unknown.method"));
    }

    #[test]
    fn rpc_response_untagged() {
        let success: RpcResponse = Response::success(json!(1), json!("ok")).into();
        let json = serde_json::to_string(&success).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));

        let error: RpcResponse =
            ErrorResponse::parse_error().into();
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"error\""));
        assert!(!json.contains("\"result\""));
    }

    #[test]
    fn parse_error_has_null_id() {
        let err = ErrorResponse::parse_error();
        assert_eq!(err.id, serde_json::Value::Null);
    }
}
