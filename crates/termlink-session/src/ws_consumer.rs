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
    #[error("WS consumer supports TCP hubs only (WS-over-Unix is a follow-on)")]
    UnsupportedTransport,
    #[error("TCP/TLS connect failed: {0}")]
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

/// Connect to a TCP hub over WebSocket (TLS already terminated), authenticate,
/// subscribe to `topics`, and forward each pushed `hub.event`'s `params` into
/// `tx` until the socket closes or errors.
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
    let (host, port) = match addr {
        TransportAddr::Tcp { host, port } => (host.clone(), *port),
        TransportAddr::Unix { .. } => return Err(WsConsumerError::UnsupportedTransport),
    };

    // TLS-terminate first, then upgrade over the plaintext-to-us TLS stream.
    let tls = crate::client::Client::connect_tls_stream(addr)
        .await
        .map_err(WsConsumerError::Connect)?;
    // TLS is already done — use client_async (NOT client_async_tls). The URL
    // scheme is cosmetic here; the host authority is what tungstenite needs.
    let url = format!("wss://{host}:{port}/");
    let (ws, _resp) = tokio_tungstenite::client_async(url, tls)
        .await
        .map_err(WsConsumerError::Handshake)?;
    let (mut sink, mut source) = ws.split();

    // --- authenticate ---
    sink.send(Message::Text(build_ws_auth_request(token_raw).into()))
        .await
        .map_err(WsConsumerError::Stream)?;
    let auth_ack = next_text_frame(&mut source)
        .await?
        .ok_or(WsConsumerError::ClosedEarly("auth"))?;
    if !auth_ack_ok(&auth_ack) {
        return Err(WsConsumerError::AuthRejected);
    }

    // --- subscribe ---
    sink.send(Message::Text(build_ws_subscribe_request(topics).into()))
        .await
        .map_err(WsConsumerError::Stream)?;
    let _sub_ack = next_text_frame(&mut source)
        .await?
        .ok_or(WsConsumerError::ClosedEarly("subscribe"))?;

    // --- stream pushes ---
    while let Some(msg) = source.next().await {
        let msg = msg.map_err(WsConsumerError::Stream)?;
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
async fn next_text_frame<S>(source: &mut S) -> Result<Option<serde_json::Value>, WsConsumerError>
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    while let Some(msg) = source.next().await {
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
    fn ws_auth_ack_ok_detection() {
        let ok = serde_json::json!({"result":{"authenticated":true}});
        assert!(auth_ack_ok(&ok));
        let no = serde_json::json!({"result":{"authenticated":false}});
        assert!(!auth_ack_ok(&no));
        let err = serde_json::json!({"error":{"code":-32001,"message":"nope"}});
        assert!(!auth_ack_ok(&err));
    }
}
