//! T-2031 (arc-parallel-substrate Slice 3) — client-side ergonomic surface for
//! the `channel.claim` / `channel.release` / `channel.renew` JSON-RPC verbs
//! shipped in Slices 1 + 2 (T-2029, T-2030).
//!
//! Two layers:
//!
//! 1. **Low-level wrappers** ([`channel_claim`], [`channel_release`],
//!    [`channel_renew`]) — single-shot async functions that issue one RPC and
//!    map hub errors into a typed [`ClaimError`]. Intentionally NOT
//!    offline-queued: a delayed claim is meaningless (the offset may already
//!    be released to another worker by the time the queued post drains).
//!    Transport failure surfaces as [`ClaimError::Transport`] — callers decide
//!    whether to retry, fall back, or fail the slice.
//!
//! 2. **High-level RAII** ([`LeasedClaim`]) — wraps a successful claim with an
//!    auto-renew background task and Drop-fires-nack semantics. Drop is the
//!    sharp edge: when a worker panics or is killed before `ack()`/`nack()`,
//!    Drop aborts the renew task and fires fire-and-forget
//!    `release(ack=false)` so the slot reopens fast instead of waiting for
//!    `claimed_until` to lapse + the next claimant's lazy-evict pass.
//!
//! ADR: `docs/architecture/parallel-execution-substrate.md` §4.2 (lease-with-
//! renewal + lazy expiry) and §6 manifest (Slice 3).

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::task::JoinHandle;

use termlink_protocol::control::{error_code, method};
use termlink_protocol::jsonrpc::RpcResponse;
use termlink_protocol::transport::TransportAddr;

use crate::client::{rpc_call_addr, ClientError};

/// Successful response shape for `channel.claim` and `channel.renew`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimSummary {
    pub claim_id: String,
    pub topic: String,
    pub offset: u64,
    pub claimer: String,
    pub claimed_at: i64,
    pub claimed_until: i64,
}

/// Successful response shape for `channel.release`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSummary {
    pub claim_id: String,
    pub topic: String,
    pub offset: u64,
    pub ack: bool,
}

/// Typed view of the four claim-specific JSON-RPC error codes plus the two
/// catch-all classes (transport/protocol). The variants mirror the codes
/// defined in `termlink_protocol::control::error_code`:
///
/// | Code   | Variant       | Cause                                                |
/// |--------|---------------|------------------------------------------------------|
/// | -32015 | `Conflict`    | offset already claimed by another worker             |
/// | -32016 | `NotFound`    | claim_id never existed, was released, or lazy-evicted|
/// | -32017 | `NotOwned`    | caller is not the original claimer                   |
/// | -32018 | `Expired`     | claimed_until has lapsed (renew refused)             |
#[derive(Debug, thiserror::Error)]
pub enum ClaimError {
    #[error("offset {offset} of topic {topic:?} is already claimed by another worker")]
    Conflict { topic: String, offset: u64 },

    #[error("claim {claim_id:?} not found (never existed, released, or expired)")]
    NotFound { claim_id: String },

    #[error("claim {claim_id:?} is held by another claimer")]
    NotOwned { claim_id: String },

    #[error("claim {claim_id:?} has expired — renew refused")]
    Expired { claim_id: String },

    #[error("hub error: code={code} message={message}")]
    Hub { code: i64, message: String },

    #[error("transport: {0}")]
    Transport(#[from] ClientError),

    #[error("protocol: {0}")]
    Protocol(String),
}

/// Issue a single `channel.claim` RPC. Direct call — never queued.
pub async fn channel_claim(
    addr: &TransportAddr,
    topic: &str,
    offset: u64,
    claimer: &str,
    ttl_ms: u32,
) -> Result<ClaimSummary, ClaimError> {
    let params = json!({
        "topic": topic,
        "offset": offset,
        "claimer": claimer,
        "ttl_ms": ttl_ms,
    });
    let resp = rpc_call_addr(addr, method::CHANNEL_CLAIM, params).await?;
    parse_claim_response(resp)
}

/// Issue a single `channel.renew` RPC. Direct call — never queued.
pub async fn channel_renew(
    addr: &TransportAddr,
    claim_id: &str,
    claimer: &str,
    additional_ttl_ms: u32,
) -> Result<ClaimSummary, ClaimError> {
    let params = json!({
        "claim_id": claim_id,
        "claimer": claimer,
        "additional_ttl_ms": additional_ttl_ms,
    });
    let resp = rpc_call_addr(addr, method::CHANNEL_RENEW, params).await?;
    parse_claim_response(resp)
}

/// Issue a single `channel.release` RPC. Direct call — never queued.
pub async fn channel_release(
    addr: &TransportAddr,
    claim_id: &str,
    claimer: &str,
    ack: bool,
) -> Result<ReleaseSummary, ClaimError> {
    let params = json!({
        "claim_id": claim_id,
        "claimer": claimer,
        "ack": ack,
    });
    let resp = rpc_call_addr(addr, method::CHANNEL_RELEASE, params).await?;
    parse_release_response(resp)
}

fn parse_claim_response(resp: RpcResponse) -> Result<ClaimSummary, ClaimError> {
    match resp {
        RpcResponse::Success(ok) => {
            let r = &ok.result;
            let claim_id = field_str(r, "claim_id")?;
            let topic = field_str(r, "topic")?;
            let offset = field_u64(r, "offset")?;
            let claimer = field_str(r, "claimer")?;
            let claimed_at = field_i64(r, "claimed_at")?;
            let claimed_until = field_i64(r, "claimed_until")?;
            Ok(ClaimSummary {
                claim_id,
                topic,
                offset,
                claimer,
                claimed_at,
                claimed_until,
            })
        }
        RpcResponse::Error(e) => Err(map_hub_error(e.error.code, e.error.message, e.error.data)),
    }
}

fn parse_release_response(resp: RpcResponse) -> Result<ReleaseSummary, ClaimError> {
    match resp {
        RpcResponse::Success(ok) => {
            let r = &ok.result;
            let claim_id = field_str(r, "claim_id")?;
            let topic = field_str(r, "topic")?;
            let offset = field_u64(r, "offset")?;
            let ack = r
                .get("ack")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| ClaimError::Protocol("missing ack".into()))?;
            Ok(ReleaseSummary {
                claim_id,
                topic,
                offset,
                ack,
            })
        }
        RpcResponse::Error(e) => Err(map_hub_error(e.error.code, e.error.message, e.error.data)),
    }
}

fn map_hub_error(code: i64, message: String, data: Option<Value>) -> ClaimError {
    let data_str = |k: &str| -> Option<String> {
        data.as_ref()
            .and_then(|d| d.get(k))
            .and_then(|v| v.as_str().map(String::from))
    };
    let data_u64 = |k: &str| -> Option<u64> {
        data.as_ref().and_then(|d| d.get(k)).and_then(|v| v.as_u64())
    };
    match code {
        c if c == error_code::CLAIM_CONFLICT => ClaimError::Conflict {
            topic: data_str("topic").unwrap_or_default(),
            offset: data_u64("offset").unwrap_or(0),
        },
        c if c == error_code::CLAIM_NOT_FOUND => ClaimError::NotFound {
            claim_id: data_str("claim_id").unwrap_or_default(),
        },
        c if c == error_code::CLAIM_NOT_OWNED => ClaimError::NotOwned {
            claim_id: data_str("claim_id").unwrap_or_default(),
        },
        c if c == error_code::CLAIM_EXPIRED => ClaimError::Expired {
            claim_id: data_str("claim_id").unwrap_or_default(),
        },
        _ => ClaimError::Hub { code, message },
    }
}

fn field_str(v: &Value, k: &str) -> Result<String, ClaimError> {
    v.get(k)
        .and_then(|x| x.as_str())
        .map(String::from)
        .ok_or_else(|| ClaimError::Protocol(format!("missing '{k}' string field")))
}

fn field_i64(v: &Value, k: &str) -> Result<i64, ClaimError> {
    v.get(k)
        .and_then(|x| x.as_i64())
        .ok_or_else(|| ClaimError::Protocol(format!("missing '{k}' i64 field")))
}

fn field_u64(v: &Value, k: &str) -> Result<u64, ClaimError> {
    v.get(k)
        .and_then(|x| x.as_u64())
        .ok_or_else(|| ClaimError::Protocol(format!("missing '{k}' u64 field")))
}

/// RAII handle wrapping a successful claim with auto-renew + drop-fires-nack.
///
/// The renew background task wakes at `ttl_ms / 2` cadence and issues
/// `channel.renew` with `additional_ttl_ms = ttl_ms`. The latest
/// `claimed_until` is published into a shared atomic so callers can observe
/// the moving deadline.
///
/// Lifecycle:
/// - [`LeasedClaim::ack`] → release(ack=true), aborts renew task, advances cursor.
/// - [`LeasedClaim::nack`] → release(ack=false), aborts renew task, frees slot.
/// - Drop without ack/nack → aborts renew task, spawns fire-and-forget
///   release(ack=false). If no tokio runtime is current, the release is
///   skipped (Drop never panics) and the slot will be lazy-evicted by the
///   next claimant after `claimed_until`.
pub struct LeasedClaim {
    addr: TransportAddr,
    claim_id: String,
    topic: String,
    offset: u64,
    claimer: String,
    claimed_at: i64,
    claimed_until: Arc<AtomicI64>,
    renew_handle: Option<JoinHandle<()>>,
    consumed: bool,
}

impl LeasedClaim {
    /// Acquire a claim and start the auto-renew background task.
    pub async fn acquire(
        addr: TransportAddr,
        topic: &str,
        offset: u64,
        claimer: &str,
        ttl_ms: u32,
    ) -> Result<Self, ClaimError> {
        let summary = channel_claim(&addr, topic, offset, claimer, ttl_ms).await?;
        let claimed_until = Arc::new(AtomicI64::new(summary.claimed_until));
        let renew_handle = spawn_renew_task(
            addr.clone(),
            summary.claim_id.clone(),
            claimer.to_string(),
            ttl_ms,
            claimed_until.clone(),
        );
        Ok(Self {
            addr,
            claim_id: summary.claim_id,
            topic: summary.topic,
            offset: summary.offset,
            claimer: summary.claimer,
            claimed_at: summary.claimed_at,
            claimed_until,
            renew_handle: Some(renew_handle),
            consumed: false,
        })
    }

    pub fn claim_id(&self) -> &str {
        &self.claim_id
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn claimer(&self) -> &str {
        &self.claimer
    }

    pub fn claimed_at(&self) -> i64 {
        self.claimed_at
    }

    /// Last-known `claimed_until` — moves forward as the renew task succeeds.
    pub fn claimed_until(&self) -> i64 {
        self.claimed_until.load(Ordering::Relaxed)
    }

    /// Consume the claim with `ack=true` (advance claimer's cursor past the offset).
    pub async fn ack(mut self) -> Result<ReleaseSummary, ClaimError> {
        self.consumed = true;
        if let Some(h) = self.renew_handle.take() {
            h.abort();
        }
        channel_release(&self.addr, &self.claim_id, &self.claimer, true).await
    }

    /// Consume the claim with `ack=false` (free the slot without advancing cursor).
    pub async fn nack(mut self) -> Result<ReleaseSummary, ClaimError> {
        self.consumed = true;
        if let Some(h) = self.renew_handle.take() {
            h.abort();
        }
        channel_release(&self.addr, &self.claim_id, &self.claimer, false).await
    }
}

impl Drop for LeasedClaim {
    fn drop(&mut self) {
        if let Some(h) = self.renew_handle.take() {
            h.abort();
        }
        if self.consumed {
            return;
        }
        // Best-effort fire-and-forget nack. If we're outside a tokio runtime
        // (e.g. a sync test or post-runtime shutdown) we silently skip — the
        // slot will be lazy-evicted on next claim attempt for this offset.
        if tokio::runtime::Handle::try_current().is_ok() {
            let addr = self.addr.clone();
            let claim_id = self.claim_id.clone();
            let claimer = self.claimer.clone();
            tokio::spawn(async move {
                let _ = channel_release(&addr, &claim_id, &claimer, false).await;
            });
        }
    }
}

fn spawn_renew_task(
    addr: TransportAddr,
    claim_id: String,
    claimer: String,
    ttl_ms: u32,
    claimed_until: Arc<AtomicI64>,
) -> JoinHandle<()> {
    let cadence = Duration::from_millis((ttl_ms as u64 / 2).max(50));
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(cadence).await;
            match channel_renew(&addr, &claim_id, &claimer, ttl_ms).await {
                Ok(summary) => {
                    claimed_until.store(summary.claimed_until, Ordering::Relaxed);
                }
                Err(ClaimError::NotFound { .. })
                | Err(ClaimError::Expired { .. })
                | Err(ClaimError::NotOwned { .. }) => {
                    // Lease is gone — no point continuing to renew.
                    tracing::debug!(
                        claim_id = %claim_id,
                        "renew loop: claim is gone, exiting"
                    );
                    break;
                }
                Err(e) => {
                    // Transport / hub blip — keep trying on the next tick.
                    tracing::debug!(
                        claim_id = %claim_id,
                        error = %e,
                        "renew loop: transient renew failure, will retry"
                    );
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use termlink_protocol::jsonrpc::{ErrorResponse, Response, RpcError};

    fn success_claim() -> RpcResponse {
        RpcResponse::Success(Response {
            jsonrpc: "2.0".into(),
            id: json!(1),
            result: json!({
                "ok": true,
                "claim_id": "clm-100-foo-0",
                "topic": "T",
                "offset": 0_u64,
                "claimer": "w1",
                "claimed_at": 1000_i64,
                "claimed_until": 31_000_i64,
            }),
        })
    }

    fn err_resp(code: i64, message: &str, data: Option<Value>) -> RpcResponse {
        RpcResponse::Error(ErrorResponse {
            jsonrpc: "2.0".into(),
            id: json!(1),
            error: RpcError {
                code,
                message: message.into(),
                data,
            },
        })
    }

    #[test]
    fn parses_successful_claim() {
        let s = parse_claim_response(success_claim()).expect("parses");
        assert_eq!(s.claim_id, "clm-100-foo-0");
        assert_eq!(s.topic, "T");
        assert_eq!(s.offset, 0);
        assert_eq!(s.claimer, "w1");
        assert_eq!(s.claimed_at, 1000);
        assert_eq!(s.claimed_until, 31_000);
    }

    #[test]
    fn parses_successful_release() {
        let resp = RpcResponse::Success(Response {
            jsonrpc: "2.0".into(),
            id: json!(1),
            result: json!({
                "ok": true,
                "claim_id": "clm-100-foo-0",
                "topic": "T",
                "offset": 0_u64,
                "ack": true,
            }),
        });
        let r = parse_release_response(resp).expect("parses");
        assert_eq!(r.claim_id, "clm-100-foo-0");
        assert!(r.ack);
    }

    #[test]
    fn maps_minus_32015_to_conflict() {
        let resp = err_resp(
            error_code::CLAIM_CONFLICT,
            "taken",
            Some(json!({"topic": "T", "offset": 5_u64})),
        );
        match parse_claim_response(resp) {
            Err(ClaimError::Conflict { topic, offset }) => {
                assert_eq!(topic, "T");
                assert_eq!(offset, 5);
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    #[test]
    fn maps_minus_32016_to_not_found() {
        let resp = err_resp(
            error_code::CLAIM_NOT_FOUND,
            "gone",
            Some(json!({"claim_id": "clm-X"})),
        );
        match parse_claim_response(resp) {
            Err(ClaimError::NotFound { claim_id }) => assert_eq!(claim_id, "clm-X"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn maps_minus_32017_to_not_owned() {
        let resp = err_resp(
            error_code::CLAIM_NOT_OWNED,
            "not yours",
            Some(json!({"claim_id": "clm-Y"})),
        );
        match parse_release_response(resp) {
            Err(ClaimError::NotOwned { claim_id }) => assert_eq!(claim_id, "clm-Y"),
            other => panic!("expected NotOwned, got {other:?}"),
        }
    }

    #[test]
    fn maps_minus_32018_to_expired() {
        let resp = err_resp(
            error_code::CLAIM_EXPIRED,
            "expired",
            Some(json!({"claim_id": "clm-Z"})),
        );
        match parse_claim_response(resp) {
            Err(ClaimError::Expired { claim_id }) => assert_eq!(claim_id, "clm-Z"),
            other => panic!("expected Expired, got {other:?}"),
        }
    }

    #[test]
    fn maps_unknown_code_to_hub() {
        let resp = err_resp(-32099, "weird", None);
        match parse_claim_response(resp) {
            Err(ClaimError::Hub { code, .. }) => assert_eq!(code, -32099),
            other => panic!("expected Hub, got {other:?}"),
        }
    }

    #[test]
    fn malformed_success_surfaces_protocol_error() {
        let resp = RpcResponse::Success(Response {
            jsonrpc: "2.0".into(),
            id: json!(1),
            result: json!({"ok": true}),
        });
        match parse_claim_response(resp) {
            Err(ClaimError::Protocol(_)) => {}
            other => panic!("expected Protocol, got {other:?}"),
        }
    }

    #[test]
    fn release_missing_ack_surfaces_protocol_error() {
        let resp = RpcResponse::Success(Response {
            jsonrpc: "2.0".into(),
            id: json!(1),
            result: json!({
                "ok": true,
                "claim_id": "x",
                "topic": "T",
                "offset": 0_u64,
            }),
        });
        match parse_release_response(resp) {
            Err(ClaimError::Protocol(_)) => {}
            other => panic!("expected Protocol, got {other:?}"),
        }
    }
}
