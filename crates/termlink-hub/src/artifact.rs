//! Hub-side handlers for the T-1248 / T-1164a `artifact.*` RPC surface.
//!
//! Two methods, both Tier-A:
//! - `artifact.put` — streaming chunked upload into the content-addressed
//!   blob store at `<bus-root>/artifacts/`. Driven by the client with
//!   `{ staging_id, offset, chunk_b64, is_final, expected_sha256? }`.
//! - `artifact.get` — streaming chunked download from the same store, keyed
//!   by sha256 hex. Caller iterates with `{ sha256, offset, max_bytes }` until
//!   the response sets `eof: true`.
//!
//! The store itself lives in `termlink-bus::ArtifactStore`; this module is
//! pure JSON-RPC glue.

use base64::Engine;
use serde_json::{json, Value};

#[cfg(test)]
use termlink_bus::ArtifactStore;
use termlink_bus::{BusError, StreamingPutOutcome};
use termlink_protocol::jsonrpc::{ErrorResponse, Response, RpcResponse};

use crate::channel::artifact_store;

/// Maximum slice we'll return in one `artifact.get` chunk. Keeps responses
/// well under MAX_PAYLOAD_SIZE even after base64 expansion.
const DEFAULT_MAX_CHUNK_BYTES: usize = 256 * 1024;

pub async fn handle_artifact_put(id: Value, params: &Value) -> RpcResponse {
    let store = match artifact_store() {
        Some(s) => s,
        None => {
            return ErrorResponse::internal_error(id, "artifact store not initialized").into();
        }
    };

    let staging_id = match param_str(params, "staging_id") {
        Some(s) if !s.is_empty() => s,
        _ => return ErrorResponse::new(id, -32602, "missing or empty 'staging_id'").into(),
    };
    let offset = match params.get("offset").and_then(|v| v.as_u64()) {
        Some(n) => n,
        None => return ErrorResponse::new(id, -32602, "missing 'offset' (u64)").into(),
    };
    let chunk_b64 = match param_str(params, "chunk_b64") {
        Some(s) => s,
        None => return ErrorResponse::new(id, -32602, "missing 'chunk_b64'").into(),
    };
    let is_final = params
        .get("is_final")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let expected_sha256 = param_str(params, "expected_sha256");

    let chunk = match base64::engine::general_purpose::STANDARD.decode(chunk_b64) {
        Ok(b) => b,
        Err(e) => {
            return ErrorResponse::new(id, -32602, &format!("invalid base64 chunk: {e}")).into();
        }
    };

    let outcome = match store.put_streaming(staging_id, offset, &chunk, is_final, expected_sha256)
    {
        Ok(o) => o,
        Err(e) => return artifact_error(id, &e),
    };

    let body = match outcome {
        StreamingPutOutcome::InProgress { bytes_received } => json!({
            "ok": true,
            "in_progress": true,
            "bytes_received": bytes_received,
        }),
        StreamingPutOutcome::Complete {
            sha256,
            total_bytes,
        } => json!({
            "ok": true,
            "in_progress": false,
            "sha256": sha256,
            "total_bytes": total_bytes,
        }),
    };
    Response::success(id, body).into()
}

pub async fn handle_artifact_get(id: Value, params: &Value) -> RpcResponse {
    let store = match artifact_store() {
        Some(s) => s,
        None => {
            return ErrorResponse::internal_error(id, "artifact store not initialized").into();
        }
    };

    let sha256 = match param_str(params, "sha256") {
        Some(s) if !s.is_empty() => s,
        _ => return ErrorResponse::new(id, -32602, "missing or empty 'sha256'").into(),
    };
    let offset = params
        .get("offset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let max_bytes = params
        .get("max_bytes")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_MAX_CHUNK_BYTES)
        .min(DEFAULT_MAX_CHUNK_BYTES);

    let bytes = match store.get(sha256) {
        Ok(b) => b,
        Err(BusError::UnknownArtifact(_)) => {
            return ErrorResponse::new(id, -32004, &format!("artifact {sha256} not found")).into();
        }
        Err(e) => return artifact_error(id, &e),
    };

    let total = bytes.len();
    if offset > total {
        return ErrorResponse::new(
            id,
            -32602,
            &format!("offset {offset} exceeds total {total}"),
        )
        .into();
    }
    let end = (offset + max_bytes).min(total);
    let slice = &bytes[offset..end];
    let chunk_b64 = base64::engine::general_purpose::STANDARD.encode(slice);
    let eof = end == total;

    Response::success(
        id,
        json!({
            "chunk_b64": chunk_b64,
            "bytes_returned": slice.len(),
            "eof": eof,
            "total_bytes": total,
        }),
    )
    .into()
}

fn artifact_error(id: Value, e: &BusError) -> RpcResponse {
    let code = match e {
        BusError::ArtifactOffsetMismatch { .. } | BusError::ArtifactHashMismatch { .. } => -32602,
        BusError::UnknownArtifact(_) => -32004,
        _ => -32603,
    };
    ErrorResponse::new(id, code, &e.to_string()).into()
}

fn param_str<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(|v| v.as_str())
}

/// Test helper that does NOT depend on the global store — feeds an arbitrary
/// store into the same handler logic. Used by the unit tests in this module
/// and by integration tests in `termlink-session` (T-1248 verification).
#[cfg(test)]
pub(crate) async fn handle_artifact_put_with(
    store: &ArtifactStore,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let staging_id = match param_str(params, "staging_id") {
        Some(s) if !s.is_empty() => s,
        _ => return ErrorResponse::new(id, -32602, "missing or empty 'staging_id'").into(),
    };
    let offset = match params.get("offset").and_then(|v| v.as_u64()) {
        Some(n) => n,
        None => return ErrorResponse::new(id, -32602, "missing 'offset'").into(),
    };
    let chunk_b64 = match param_str(params, "chunk_b64") {
        Some(s) => s,
        None => return ErrorResponse::new(id, -32602, "missing 'chunk_b64'").into(),
    };
    let is_final = params
        .get("is_final")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let expected_sha256 = param_str(params, "expected_sha256");
    let chunk = match base64::engine::general_purpose::STANDARD.decode(chunk_b64) {
        Ok(b) => b,
        Err(e) => return ErrorResponse::new(id, -32602, &format!("bad b64: {e}")).into(),
    };
    let outcome = match store.put_streaming(staging_id, offset, &chunk, is_final, expected_sha256)
    {
        Ok(o) => o,
        Err(e) => return artifact_error(id, &e),
    };
    let body = match outcome {
        StreamingPutOutcome::InProgress { bytes_received } => json!({
            "ok": true,
            "in_progress": true,
            "bytes_received": bytes_received,
        }),
        StreamingPutOutcome::Complete {
            sha256,
            total_bytes,
        } => json!({
            "ok": true,
            "in_progress": false,
            "sha256": sha256,
            "total_bytes": total_bytes,
        }),
    };
    Response::success(id, body).into()
}

#[cfg(test)]
pub(crate) async fn handle_artifact_get_with(
    store: &ArtifactStore,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let sha256 = match param_str(params, "sha256") {
        Some(s) if !s.is_empty() => s,
        _ => return ErrorResponse::new(id, -32602, "missing 'sha256'").into(),
    };
    let offset = params
        .get("offset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let max_bytes = params
        .get("max_bytes")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_MAX_CHUNK_BYTES)
        .min(DEFAULT_MAX_CHUNK_BYTES);
    let bytes = match store.get(sha256) {
        Ok(b) => b,
        Err(BusError::UnknownArtifact(_)) => {
            return ErrorResponse::new(id, -32004, "not found").into();
        }
        Err(e) => return artifact_error(id, &e),
    };
    let total = bytes.len();
    if offset > total {
        return ErrorResponse::new(id, -32602, "offset > total").into();
    }
    let end = (offset + max_bytes).min(total);
    let slice = &bytes[offset..end];
    Response::success(
        id,
        json!({
            "chunk_b64": base64::engine::general_purpose::STANDARD.encode(slice),
            "bytes_returned": slice.len(),
            "eof": end == total,
            "total_bytes": total,
        }),
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn b64(b: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(b)
    }

    fn body_of(rsp: &RpcResponse) -> Value {
        // Both Response and ErrorResponse serialize to the standard JSON-RPC
        // envelope. Test extraction goes through the wire format so we know
        // the client sees the same fields.
        serde_json::to_value(rsp).expect("rpc response serializes")
    }

    #[tokio::test]
    async fn put_complete_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let payload = b"hub artifact e2e";
        let params = json!({
            "staging_id": "t1",
            "offset": 0,
            "chunk_b64": b64(payload),
            "is_final": true,
        });
        let rsp = handle_artifact_put_with(&store, json!(1), &params).await;
        let v = body_of(&rsp);
        assert_eq!(v["result"]["ok"], true);
        assert_eq!(v["result"]["in_progress"], false);
        let sha = v["result"]["sha256"].as_str().unwrap().to_string();

        let get_params = json!({"sha256": sha, "offset": 0, "max_bytes": 1024});
        let rsp = handle_artifact_get_with(&store, json!(2), &get_params).await;
        let v = body_of(&rsp);
        assert_eq!(v["result"]["eof"], true);
        let chunk = base64::engine::general_purpose::STANDARD
            .decode(v["result"]["chunk_b64"].as_str().unwrap())
            .unwrap();
        assert_eq!(chunk, payload);
    }

    #[tokio::test]
    async fn put_chunked_5mb_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        // ~5MB of varied bytes (not all-zero) so sha256 is meaningful.
        let mut payload = Vec::with_capacity(5 * 1024 * 1024);
        for i in 0..(5 * 1024 * 1024 / 4) {
            payload.extend_from_slice(&(i as u32).to_be_bytes());
        }
        let chunk_size = 256 * 1024;
        let mut offset = 0u64;
        let mut iter = payload.chunks(chunk_size).peekable();
        let mut final_sha: Option<String> = None;
        while let Some(chunk) = iter.next() {
            let is_final = iter.peek().is_none();
            let params = json!({
                "staging_id": "big",
                "offset": offset,
                "chunk_b64": b64(chunk),
                "is_final": is_final,
            });
            let rsp = handle_artifact_put_with(&store, json!(offset), &params).await;
            let v = body_of(&rsp);
            assert_eq!(v["result"]["ok"], true);
            if is_final {
                final_sha = v["result"]["sha256"].as_str().map(String::from);
            }
            offset += chunk.len() as u64;
        }
        let sha = final_sha.expect("final chunk returned a sha256");

        // Pull it back chunked.
        let mut got = Vec::with_capacity(payload.len());
        let mut goff = 0u64;
        loop {
            let params = json!({"sha256": sha, "offset": goff, "max_bytes": 256 * 1024});
            let rsp = handle_artifact_get_with(&store, json!(goff), &params).await;
            let v = body_of(&rsp);
            let chunk = base64::engine::general_purpose::STANDARD
                .decode(v["result"]["chunk_b64"].as_str().unwrap())
                .unwrap();
            got.extend_from_slice(&chunk);
            if v["result"]["eof"].as_bool().unwrap() {
                break;
            }
            goff += chunk.len() as u64;
        }
        assert_eq!(got.len(), payload.len());
        assert_eq!(got, payload);
    }

    #[tokio::test]
    async fn get_unknown_returns_error() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let params = json!({"sha256": "0".repeat(64)});
        let rsp = handle_artifact_get_with(&store, json!(1), &params).await;
        let v = body_of(&rsp);
        assert!(v.get("error").is_some(), "expected error envelope");
    }

    #[tokio::test]
    async fn put_hash_mismatch_rejected() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let bogus = "0".repeat(64);
        let params = json!({
            "staging_id": "rejid",
            "offset": 0,
            "chunk_b64": b64(b"genuine"),
            "is_final": true,
            "expected_sha256": bogus,
        });
        let rsp = handle_artifact_put_with(&store, json!(1), &params).await;
        let v = body_of(&rsp);
        assert!(v.get("error").is_some());
    }
}
