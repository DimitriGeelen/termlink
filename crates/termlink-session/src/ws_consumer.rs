//! Client-side WebSocket push consumer (T-2309, arc-004 push-transport S3b).
//!
//! The hub side (S1–S4) accepts a WebSocket after TLS termination, authenticates
//! it, applies a per-connection `hub.ws_subscribe` topic filter, and pushes
//! `hub.event` JSON-RPC notification frames the instant a matching event is
//! produced. This module is the client half: it connects, authenticates,
//! subscribes, and forwards each pushed event's `params` into an mpsc channel
//! until the socket closes or errors.
//!
//! Keeping tungstenite here (not in the CLI crate) means the CLI depends only on
//! `tokio::sync::mpsc` to consume the stream, and on WS end it degrades to the
//! existing poll loop — the durable substrate stays authoritative (arc IW-5).

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use termlink_protocol::TransportAddr;

/// Errors from the WS consumer. The CLI treats any of these as "WS unavailable →
/// degrade to poll", so they are informational rather than fatal.
#[derive(Debug, thiserror::Error)]
pub enum WsConsumerError {
    #[error("hub connect failed: {0}")]
    Connect(std::io::Error),
    #[error("WebSocket handshake failed: {0}")]
    Handshake(tokio_tungstenite::tungstenite::Error),
    #[error("WebSocket stream error: {0}")]
    Stream(tokio_tungstenite::tungstenite::Error),
    #[error("hub closed the socket before the {0} ack")]
    ClosedEarly(&'static str),
    #[error("authentication rejected by hub")]
    AuthRejected,
    #[error("malformed frame: {0}")]
    Malformed(serde_json::Error),
    #[error("hub sent no frame within the read timeout (silent/half-open link)")]
    ReadTimeout,
}

/// T-2446 (WS#4): client-side read timeout. `run_ws_session` used to await
/// `source.next()` with no bound, so a half-open hub link (no FIN/RST) hung the
/// consumer forever and the reconnect loop never fired. This bounds every read;
/// a live hub (post-T-2442) pings ~every 30s and each ping frame resets the
/// window, so healthy quiet push sessions are unaffected. Env-tunable, clamped.
fn client_read_timeout() -> std::time::Duration {
    let ms = std::env::var("TERMLINK_WS_CLIENT_READ_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(90_000)
        .clamp(1_000, 3_600_000);
    std::time::Duration::from_millis(ms)
}

/// Await the next stream item bounded by `read_timeout`. Returns
/// `Err(ReadTimeout)` when the hub sends nothing within the window — the caller
/// then surfaces an error and the reconnect loop degrades to poll instead of
/// hanging. `Ok(None)` still means a clean stream end.
async fn next_frame_bounded<S>(
    source: &mut S,
    read_timeout: std::time::Duration,
) -> Result<Option<Result<Message, tokio_tungstenite::tungstenite::Error>>, WsConsumerError>
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    match tokio::time::timeout(read_timeout, source.next()).await {
        Ok(item) => Ok(item),
        Err(_) => Err(WsConsumerError::ReadTimeout),
    }
}

/// Build the `hub.auth` request frame (pure — unit-tested).
pub fn build_ws_auth_request(token_raw: &str) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "hub.auth",
        "id": "ws-auth",
        "params": { "token": token_raw },
    })
    .to_string()
}

/// Build the `hub.ws_subscribe` request frame (pure — unit-tested).
pub fn build_ws_subscribe_request(topics: &[String]) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "hub.ws_subscribe",
        "id": "ws-sub",
        "params": { "topics": topics },
    })
    .to_string()
}

/// Extract the `params` of a `hub.event` notification frame, or `None` for any
/// other frame (acks, unrelated notifications). Pure — unit-tested.
pub fn hub_event_params(frame: &serde_json::Value) -> Option<serde_json::Value> {
    if frame.get("method").and_then(|m| m.as_str()) == Some("hub.event") {
        Some(frame.get("params").cloned().unwrap_or(serde_json::Value::Null))
    } else {
        None
    }
}

/// Whether an auth-ack frame reports success (`result.authenticated == true`).
/// Pure — unit-tested.
pub fn auth_ack_ok(frame: &serde_json::Value) -> bool {
    frame
        .get("result")
        .and_then(|r| r.get("authenticated"))
        .and_then(|a| a.as_bool())
        .unwrap_or(false)
}

/// Whether the WS session must authenticate before subscribing (T-2313). TCP
/// hubs require a `hub.auth` token; Unix hubs are peer-cred-trusted (the hub
/// pre-grants `Execute` scope, which already satisfies `hub.ws_subscribe`) and
/// therefore skip auth entirely. `stream_ws_events` uses this to decide the
/// `token` passed to `run_ws_session`. Pure — unit-tested.
pub fn ws_auth_required(addr: &TransportAddr) -> bool {
    matches!(addr, TransportAddr::Tcp { .. })
}

/// Connect to a hub over WebSocket, subscribe to `topics`, and forward each
/// pushed `hub.event`'s `params` into `tx` until the socket closes or errors.
///
/// Transport-adaptive (T-2313, arc-004 WS-over-Unix):
/// - **TCP hub:** TLS-terminate first, then upgrade over the TLS stream, and
///   authenticate with `token_raw` (a `hub.auth` handshake).
/// - **Unix hub:** upgrade directly over the raw Unix socket (no TLS — the UDS is
///   peer-cred-trusted) and **skip auth** — Unix connections are pre-granted
///   `Execute` scope by the hub, which already satisfies `hub.ws_subscribe`.
///   `token_raw` is ignored for Unix.
///
/// Returns `Ok(())` on a clean close and `Err(_)` on any failure — the caller
/// (CLI) degrades to the poll loop in either case. Returns early (Ok) if the
/// receiver is dropped.
pub async fn stream_ws_events(
    addr: &TransportAddr,
    token_raw: &str,
    topics: &[String],
    tx: mpsc::Sender<serde_json::Value>,
) -> Result<(), WsConsumerError> {
    // Single source of truth for the auth decision — None for Unix (peer-cred
    // trust), Some(token) for TCP. Keeps the two transport arms consistent.
    let token = if ws_auth_required(addr) {
        Some(token_raw)
    } else {
        None
    };
    match addr {
        TransportAddr::Tcp { host, port } => {
            // TLS-terminate first, then upgrade over the plaintext-to-us TLS stream.
            let tls = crate::client::Client::connect_tls_stream(addr)
                .await
                .map_err(WsConsumerError::Connect)?;
            // TLS is already done — use client_async (NOT client_async_tls). The
            // URL scheme is cosmetic here; the host authority is what tungstenite
            // needs.
            let url = format!("wss://{host}:{port}/");
            let (ws, _resp) = tokio_tungstenite::client_async(url, tls)
                .await
                .map_err(WsConsumerError::Handshake)?;
            // TCP hubs require auth: `token` is Some(minted token).
            run_ws_session(ws, token, topics, tx).await
        }
        TransportAddr::Unix { path } => {
            let stream = tokio::net::UnixStream::connect(path)
                .await
                .map_err(WsConsumerError::Connect)?;
            // Raw WS over the Unix socket — no TLS. The URL host is cosmetic.
            let (ws, _resp) = tokio_tungstenite::client_async("ws://localhost/", stream)
                .await
                .map_err(WsConsumerError::Handshake)?;
            // Unix is peer-cred-trusted and pre-granted Execute scope → `token`
            // is None → run_ws_session skips the auth handshake.
            run_ws_session(ws, token, topics, tx).await
        }
    }
}

/// Shared post-connect session: (optionally) authenticate, subscribe, then stream
/// pushed `hub.event` params into `tx`. Generic over the underlying transport so
/// the TLS-wrapped TCP stream and the raw Unix stream share one loop.
///
/// `token`: `Some(raw)` → send a `hub.auth` handshake and require an ok ack;
/// `None` → skip auth entirely (Unix peer-cred trust).
async fn run_ws_session<S>(
    ws: tokio_tungstenite::WebSocketStream<S>,
    token: Option<&str>,
    topics: &[String],
    tx: mpsc::Sender<serde_json::Value>,
) -> Result<(), WsConsumerError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (mut sink, mut source) = ws.split();
    let read_timeout = client_read_timeout();

    // --- authenticate (TCP only) ---
    if let Some(token_raw) = token {
        sink.send(Message::Text(build_ws_auth_request(token_raw).into()))
            .await
            .map_err(WsConsumerError::Stream)?;
        let auth_ack = next_text_frame(&mut source, read_timeout)
            .await?
            .ok_or(WsConsumerError::ClosedEarly("auth"))?;
        if !auth_ack_ok(&auth_ack) {
            return Err(WsConsumerError::AuthRejected);
        }
    }

    // --- subscribe ---
    sink.send(Message::Text(build_ws_subscribe_request(topics).into()))
        .await
        .map_err(WsConsumerError::Stream)?;
    let _sub_ack = next_text_frame(&mut source, read_timeout)
        .await?
        .ok_or(WsConsumerError::ClosedEarly("subscribe"))?;

    // --- stream pushes ---
    // T-2446: bounded read — a silent/half-open hub yields Err(ReadTimeout) so
    // the reconnect loop degrades to poll instead of hanging forever.
    loop {
        let msg = match next_frame_bounded(&mut source, read_timeout).await? {
            Some(m) => m.map_err(WsConsumerError::Stream)?,
            None => break,
        };
        match msg {
            Message::Text(t) => {
                let frame: serde_json::Value =
                    serde_json::from_str(t.as_str()).map_err(WsConsumerError::Malformed)?;
                if let Some(params) = hub_event_params(&frame) {
                    // Receiver gone → consumer is done; return cleanly.
                    if tx.send(params).await.is_err() {
                        return Ok(());
                    }
                }
            }
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Close(_) => break,
            _ => {}
        }
    }
    Ok(())
}

/// Read frames until the next Text frame (skipping ping/pong). Returns `Ok(None)`
/// if the stream closes first.
async fn next_text_frame<S>(
    source: &mut S,
    read_timeout: std::time::Duration,
) -> Result<Option<serde_json::Value>, WsConsumerError>
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    // T-2446: bound the handshake reads too — a hub that upgrades but never acks
    // auth/subscribe would otherwise hang here indefinitely.
    while let Some(msg) = next_frame_bounded(source, read_timeout).await? {
        let msg = msg.map_err(WsConsumerError::Stream)?;
        match msg {
            Message::Text(t) => {
                let v: serde_json::Value =
                    serde_json::from_str(t.as_str()).map_err(WsConsumerError::Malformed)?;
                return Ok(Some(v));
            }
            Message::Ping(_) | Message::Pong(_) => continue,
            Message::Close(_) => return Ok(None),
            _ => continue,
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- T-2446 client read timeout (WS#4) ---------------------------------

    #[tokio::test]
    async fn next_frame_bounded_times_out_on_silent_source() {
        // A source that never yields (a half-open hub link) must produce
        // Err(ReadTimeout) within the window — NOT hang forever. This is the
        // load-bearing behaviour that lets the reconnect loop degrade to poll.
        let mut silent = futures_util::stream::pending::<
            Result<Message, tokio_tungstenite::tungstenite::Error>,
        >();
        let started = std::time::Instant::now();
        let r = next_frame_bounded(&mut silent, std::time::Duration::from_millis(120)).await;
        assert!(
            matches!(r, Err(WsConsumerError::ReadTimeout)),
            "silent source must time out, got {r:?}"
        );
        assert!(
            started.elapsed() < std::time::Duration::from_secs(2),
            "must return promptly at the timeout, not hang"
        );
    }

    #[tokio::test]
    async fn next_frame_bounded_returns_ready_frame_no_false_timeout() {
        // A frame already available must be returned unchanged — the timeout is
        // a backstop, not a throttle. Guards against a false-positive timeout.
        let mut ready = futures_util::stream::iter(vec![Ok(Message::Text("hi".into()))]);
        let r = next_frame_bounded(&mut ready, std::time::Duration::from_secs(5)).await;
        match r {
            Ok(Some(Ok(Message::Text(t)))) => assert_eq!(t.as_str(), "hi"),
            other => panic!("expected the ready Text frame, got {other:?}"),
        }
    }

    #[test]
    fn client_read_timeout_defaults_and_clamps() {
        // Default when unset (env is process-global; this test only reads).
        let d = client_read_timeout();
        assert!(
            d >= std::time::Duration::from_secs(1)
                && d <= std::time::Duration::from_secs(3600),
            "timeout stays within the clamp band"
        );
    }

    #[test]
    fn ws_auth_request_shape() {
        let raw = build_ws_auth_request("tok-abc");
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["method"], "hub.auth");
        assert_eq!(v["params"]["token"], "tok-abc");
        assert_eq!(v["jsonrpc"], "2.0");
    }

    #[test]
    fn ws_subscribe_request_shape() {
        let topics = vec!["inbox.queued".to_string(), "dm:*".to_string()];
        let raw = build_ws_subscribe_request(&topics);
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["method"], "hub.ws_subscribe");
        assert_eq!(v["params"]["topics"][0], "inbox.queued");
        assert_eq!(v["params"]["topics"][1], "dm:*");
    }

    #[test]
    fn ws_hub_event_mapping() {
        let push = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "hub.event",
            "params": { "topic": "inbox.queued", "payload": { "message_offset": 7 } }
        });
        let params = hub_event_params(&push).expect("hub.event yields params");
        assert_eq!(params["topic"], "inbox.queued");
        assert_eq!(params["payload"]["message_offset"], 7);

        // Non-event frames (acks, other notifications) map to None.
        let ack = serde_json::json!({"jsonrpc":"2.0","id":"ws-sub","result":{"subscribed":["x"],"count":1}});
        assert!(hub_event_params(&ack).is_none());
        let other = serde_json::json!({"jsonrpc":"2.0","method":"hub.other","params":{}});
        assert!(hub_event_params(&other).is_none());
    }

    #[test]
    fn ws_auth_required_by_transport() {
        // T-2313: TCP requires a hub.auth handshake; Unix skips it (peer-cred trust).
        assert!(ws_auth_required(&TransportAddr::tcp("127.0.0.1", 9100)));
        assert!(!ws_auth_required(&TransportAddr::unix(
            std::path::PathBuf::from("/tmp/termlink-0/hub.sock")
        )));
    }

    #[test]
    fn ws_auth_ack_ok_detection() {
        let ok = serde_json::json!({"result":{"authenticated":true}});
        assert!(auth_ack_ok(&ok));
        let no = serde_json::json!({"result":{"authenticated":false}});
        assert!(!auth_ack_ok(&no));
        let err = serde_json::json!({"error":{"code":-32001,"message":"nope"}});
        assert!(!auth_ack_ok(&err));
    }
}
