use std::sync::OnceLock;
use std::time::Duration;

use serde_json::json;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};
use termlink_protocol::TransportAddr;

use termlink_session::client;
use termlink_session::manager;

use crate::aggregator::{EventAggregator, SessionTarget};
use crate::remote_store::RemoteStore;
use crate::topic_lint::{self, LintOutcome};

/// Per-target timeout for broadcast/collect operations.
const PER_TARGET_TIMEOUT: Duration = Duration::from_secs(5);

// T-1166 / T-1415: legacy primitives (event.broadcast, inbox.*) were retired
// 2026-05-11 (operator-authorized cut). Source cleanup landed 2026-05-31 after
// a 9-day clean bake window — last legacy emission was 2026-05-22T11:46Z from
// .122's framework-bridge fallback (T-1814 fix). The retired methods now have
// no router handlers, no cfg-feature gate, no const flip; they fall through
// to forward_to_target like any other unknown method name.

/// Global remote session store (initialized once by the hub server).
static REMOTE_STORE: OnceLock<RemoteStore> = OnceLock::new();

/// Global event aggregator (T-966).
static AGGREGATOR: OnceLock<EventAggregator> = OnceLock::new();

/// Initialize the global remote store. Called once by the hub server.
pub fn init_remote_store() -> RemoteStore {
    let store = RemoteStore::new();
    let _ = REMOTE_STORE.set(store.clone());
    store
}

/// Initialize the global event aggregator. Called once by the hub server.
pub fn init_aggregator() {
    let _ = AGGREGATOR.set(EventAggregator::new(4096));
}

/// Get the global event aggregator.
pub(crate) fn aggregator() -> Option<&'static EventAggregator> {
    AGGREGATOR.get()
}

/// Get the global remote store (returns None if not initialized).
pub(crate) fn remote_store() -> Option<&'static RemoteStore> {
    REMOTE_STORE.get()
}

/// Route a JSON-RPC request to the appropriate handler.
///
/// Hub-local methods (session.discover) are handled directly.
/// All other methods are forwarded to the target session specified in params.target.
pub async fn route(req: &Request, peer_addr: Option<&str>) -> Option<RpcResponse> {
    if req.is_notification() {
        tracing::debug!(method = %req.method, "Hub received notification (ignoring)");
        return None;
    }

    let id = req.id.clone().unwrap_or(serde_json::Value::Null);

    let response = match req.method.as_str() {
        control::method::SESSION_DISCOVER => handle_discover(id, &req.params).await,
        control::method::SESSION_WHOAMI => handle_whoami(id, &req.params).await,
        // T-1166 / T-1415: event.broadcast + inbox.* arms deleted 2026-05-31.
        // These method names now fall through to forward_to_target like any
        // other unknown method.
        control::method::EVENT_COLLECT => handle_event_collect(id, &req.params).await,
        control::method::EVENT_SUBSCRIBE if is_hub_level(&req.params) => {
            handle_hub_subscribe(id, &req.params).await
        }
        control::method::EVENT_EMIT_TO => handle_event_emit_to(id, &req.params).await,
        control::method::ORCHESTRATOR_ROUTE => handle_orchestrator_route(id, &req.params).await,
        control::method::ORCHESTRATOR_BYPASS_STATUS => handle_bypass_status(id),
        control::method::ORCHESTRATOR_BYPASS_INVALIDATE => handle_bypass_invalidate(id, &req.params),
        "session.register_remote" => handle_register_remote(id, &req.params),
        "session.heartbeat" => handle_heartbeat(id, &req.params),
        "session.deregister_remote" => handle_deregister_remote(id, &req.params),
        control::method::CHANNEL_CREATE => {
            crate::channel::handle_channel_create(id, &req.params).await
        }
        control::method::CHANNEL_SET_RETENTION => {
            crate::channel::handle_channel_set_retention(id, &req.params).await
        }
        control::method::CHANNEL_SWEEP => {
            crate::channel::handle_channel_sweep(id, &req.params).await
        }
        control::method::CHANNEL_POST => {
            // T-2297: forward the hub-observed TCP source address so the handler
            // can stamp an attested `observed_addr` on the stored envelope.
            crate::channel::handle_channel_post(id, &req.params, peer_addr).await
        }
        control::method::CHANNEL_SUBSCRIBE => {
            crate::channel::handle_channel_subscribe(id, &req.params).await
        }
        control::method::CHANNEL_LIST => {
            crate::channel::handle_channel_list(id, &req.params).await
        }
        control::method::CHANNEL_TRIM => {
            crate::channel::handle_channel_trim(id, &req.params).await
        }
        control::method::CHANNEL_DELETE => {
            crate::channel::handle_channel_delete(id, &req.params).await
        }
        control::method::CHANNEL_RECEIPTS => {
            crate::channel::handle_channel_receipts(id, &req.params).await
        }
        control::method::CHANNEL_CLAIM => {
            crate::channel::handle_channel_claim(id, &req.params).await
        }
        control::method::CHANNEL_RELEASE => {
            crate::channel::handle_channel_release(id, &req.params).await
        }
        control::method::CHANNEL_FORCE_RELEASE => {
            crate::channel::handle_channel_force_release(id, &req.params).await
        }
        control::method::CHANNEL_TRANSFER_CLAIM => {
            crate::channel::handle_channel_transfer_claim(id, &req.params).await
        }
        control::method::CHANNEL_RENEW => {
            crate::channel::handle_channel_renew(id, &req.params).await
        }
        control::method::CHANNEL_CLAIMS => {
            crate::channel::handle_channel_claims(id, &req.params).await
        }
        control::method::CHANNEL_CLAIMS_SUMMARY => {
            crate::channel::handle_channel_claims_summary(id, &req.params).await
        }
        control::method::CHANNEL_CV_KEYS => {
            crate::channel::handle_channel_cv_keys(id, &req.params).await
        }
        control::method::AGENT_FIND_IDLE => {
            crate::channel::handle_agent_find_idle(id, &req.params).await
        }
        control::method::DIALOG_PRESENCE => {
            crate::channel::handle_dialog_presence(id, &req.params).await
        }
        control::method::ARTIFACT_PUT => {
            crate::artifact::handle_artifact_put(id, &req.params).await
        }
        control::method::ARTIFACT_GET => {
            crate::artifact::handle_artifact_get(id, &req.params).await
        }
        "hub.version" => handle_hub_version(id),
        "hub.legacy_usage" => handle_hub_legacy_usage(id, &req.params),
        "hub.bus_state" => handle_hub_bus_state(id),
        "hub.governor_status" => handle_hub_governor_status(id),
        control::method::HUB_CAPABILITIES => handle_hub_capabilities(id),
        _ => forward_to_target(req, id).await,
    };

    Some(response)
}

/// Handle `session.discover` — list/filter registered sessions.
///
/// Optional params: { tags?: [string], roles?: [string], capabilities?: [string], name?: string }
/// All filters use AND logic. Omitted filters match everything.
async fn handle_discover(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    match manager::list_sessions(false) {
        Ok(sessions) => {
            let tag_filter: Vec<String> = params
                .get("tags")
                .and_then(|t| t.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let role_filter: Vec<String> = params
                .get("roles")
                .and_then(|t| t.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let cap_filter: Vec<String> = params
                .get("capabilities")
                .and_then(|t| t.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let name_filter = params.get("name").and_then(|n| n.as_str());

            let mut entries: Vec<serde_json::Value> = sessions
                .iter()
                .filter(|s| {
                    tag_filter.iter().all(|t| s.tags.contains(t))
                        && role_filter.iter().all(|r| s.roles.contains(r))
                        && cap_filter.iter().all(|c| s.capabilities.contains(c))
                        && name_filter.is_none_or(|n| {
                            s.display_name.to_lowercase().contains(&n.to_lowercase())
                        })
                })
                .map(|s| {
                    let mut entry = json!({
                        "id": s.id.as_str(),
                        "display_name": s.display_name,
                        "state": s.state,
                        "capabilities": s.capabilities,
                        "roles": s.roles,
                        "tags": s.tags,
                        "pid": s.pid,
                    });
                    // T-1441: surface identity_fingerprint so `remote list`
                    // can show it as the value `--target-fp` wants. Omit
                    // (not null) when absent for pre-T-1436 sessions.
                    if let Some(fp) = s.metadata.identity_fingerprint.as_deref() {
                        entry["identity_fingerprint"] = json!(fp);
                    }
                    entry
                })
                .collect();

            // Include remote (TCP) sessions from the in-memory store
            if let Some(store) = remote_store() {
                let remote_entries: Vec<serde_json::Value> = store
                    .list_live()
                    .iter()
                    .filter(|e| {
                        tag_filter.iter().all(|t| e.tags.contains(t))
                            && role_filter.iter().all(|r| e.roles.contains(r))
                            && cap_filter.iter().all(|c| e.capabilities.contains(c))
                            && name_filter.is_none_or(|n| {
                                e.display_name.to_lowercase().contains(&n.to_lowercase())
                            })
                    })
                    .map(|e| e.to_json())
                    .collect();
                entries.extend(remote_entries);
            }

            Response::success(id, json!({ "sessions": entries })).into()
        }
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("Discovery failed: {e}")).into()
        }
    }
}

/// Handle `session.whoami` — return the caller's session identity card.
///
/// T-1299 / T-1297. Disambiguator chain:
///   1. `session_id` hint (from `$TERMLINK_SESSION_ID`) — exact match.
///   2. `display_name` hint — exact match (rejects on collision).
///   3. Neither — return all live candidates so caller can pick.
///
/// Source-PID tree-walk disambiguation is deferred to a follow-up; the current
/// hub does not thread peer credentials through the JSON-RPC dispatch layer.
fn whoami_card(reg: &termlink_session::Registration) -> serde_json::Value {
    json!({
        "id": reg.id.as_str(),
        "display_name": reg.display_name,
        "state": reg.state,
        "capabilities": reg.capabilities,
        "roles": reg.roles,
        "tags": reg.tags,
        "pid": reg.pid,
        "cwd": reg.metadata.cwd,
    })
}

async fn handle_whoami(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let session_id = params.get("session_id").and_then(|v| v.as_str());
    let display_name = params.get("display_name").and_then(|v| v.as_str());

    if let Some(query) = session_id.or(display_name) {
        return match manager::find_session(query) {
            Ok(reg) => Response::success(
                id,
                json!({ "ok": true, "session": whoami_card(&reg) }),
            ).into(),
            Err(e) => Response::success(
                id,
                json!({
                    "ok": false,
                    "found": false,
                    "hint": format!("No session matched '{query}': {e}. Set TERMLINK_SESSION_ID to your session id, or call session.whoami without a hint to list candidates."),
                }),
            ).into(),
        };
    }

    // No hint — return all live sessions as candidates.
    match manager::list_sessions(false) {
        Ok(sessions) => {
            let candidates: Vec<serde_json::Value> = sessions.iter().map(whoami_card).collect();
            Response::success(
                id,
                json!({
                    "ok": true,
                    "ambiguous": true,
                    "candidates": candidates,
                    "hint": "Multiple candidates — set TERMLINK_SESSION_ID=<id> to your session id and retry, or pass session.whoami { session_id: '...' }.",
                }),
            ).into()
        }
        Err(e) => ErrorResponse::internal_error(id, &format!("whoami list failed: {e}")).into(),
    }
}


/// Handle `event.emit_to` — push an event directly to a target session's event bus.
///
/// Params: { target: string, topic: string, payload?: value, from?: string }
/// The hub resolves the target session, enriches the payload with sender info,
/// and forwards an `event.emit` RPC to the target's socket. This is a unicast
/// push — the sender does not need to know the target's socket path.
async fn handle_event_emit_to(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    let target = match params.get("target").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return ErrorResponse::new(
                id,
                -32602,
                "Missing 'target' in params",
            )
            .into();
        }
    };

    let topic = match params.get("topic").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return ErrorResponse::new(
                id,
                -32602,
                "Missing 'topic' in params",
            )
            .into();
        }
    };
    if let Err(e) = validate_topic_name(topic) {
        return ErrorResponse::new(id, -32602, &e).into();
    }

    let payload = params
        .get("payload")
        .cloned()
        .unwrap_or(json!({}));

    let from = params.get("from").and_then(|f| f.as_str());

    // T-1300: Soft-lint at emit. Same semantics as event.broadcast — never
    // blocks; warnings dual-write to `routing:lint`.
    run_topic_lint("event.emit_to", topic, from).await;

    // Resolve target session (local first, then remote)
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(_) => {
            // Check remote store
            if let Some(store) = remote_store()
                && let Some(_remote) = store.get(target) {
                    return ErrorResponse::new(
                        id,
                        control::error_code::CAPABILITY_NOT_SUPPORTED,
                        "emit_to for remote (TCP) sessions is not yet supported",
                    )
                    .into();
                }

            // T-988: For file events, spool to inbox instead of erroring
            if let Ok(true) = crate::inbox::deposit(target, topic, &payload, from) {
                // T-1163: dual-write into channel:inbox:<target> so subscribers can
                // migrate to the channel.* surface without waiting for legacy inbox.*
                // callers. Best-effort; never blocks the deposit response.
                crate::channel::mirror_inbox_deposit(target, topic, &payload, from).await;
                return Response::success(id, json!({
                    "ok": true,
                    "spooled": true,
                    "target": target,
                    "message": format!("Target '{}' offline — file event spooled to inbox", target),
                }))
                .into();
            }

            return ErrorResponse::new(
                id,
                control::error_code::SESSION_NOT_FOUND,
                &format!("Target session '{}' not found", target),
            )
            .into();
        }
    };

    // Enrich payload with sender info for traceability
    let enriched_payload = if let Some(sender) = from {
        let mut p = payload.clone();
        if let Some(obj) = p.as_object_mut() {
            obj.insert("_from".to_string(), json!(sender));
        } else {
            p = json!({ "_data": payload, "_from": sender });
        }
        p
    } else {
        payload
    };

    let emit_params = json!({
        "topic": topic,
        "payload": enriched_payload,
    });

    let addr = reg.addr.to_transport_addr();
    let result = tokio::time::timeout(
        PER_TARGET_TIMEOUT,
        client::rpc_call_addr(&addr, control::method::EVENT_EMIT, emit_params),
    )
    .await;

    match result {
        Ok(Ok(resp)) => {
            match client::unwrap_result(resp) {
                Ok(mut result) => {
                    // Add target info to response
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("target".to_string(), json!(target));
                        if let Some(sender) = from {
                            obj.insert("from".to_string(), json!(sender));
                        }
                    }
                    Response::success(id, result).into()
                }
                Err(e) => {
                    ErrorResponse::internal_error(
                        id,
                        &format!("Target session rejected emit: {e}"),
                    )
                    .into()
                }
            }
        }
        Ok(Err(e)) => {
            ErrorResponse::internal_error(
                id,
                &format!("Failed to connect to target session '{}': {e}", target),
            )
            .into()
        }
        Err(_) => {
            ErrorResponse::internal_error(
                id,
                &format!("Timeout emitting to target session '{}'", target),
            )
            .into()
        }
    }
}

/// Check if a request is hub-level (no `target` param, or `aggregate: true`).
fn is_hub_level(params: &serde_json::Value) -> bool {
    params.get("target").is_none()
        || params.get("aggregate").and_then(|a| a.as_bool()).unwrap_or(false)
}

/// Handle hub-level `event.subscribe` — return aggregated events from all sessions (T-966).
///
/// Params: { timeout_ms?: u64, topic?: string }
/// No `target` param = hub-level aggregation. With `target` param = forwarded to session.
async fn handle_hub_subscribe(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    let agg = match aggregator() {
        Some(a) => a,
        None => {
            return ErrorResponse::internal_error(id, "Event aggregator not initialized").into();
        }
    };

    let timeout_ms = params
        .get("timeout_ms")
        .and_then(|t| t.as_u64())
        .unwrap_or(5000);
    let topic_filter = params.get("topic").and_then(|t| t.as_str());

    let events = agg
        .collect(Duration::from_millis(timeout_ms), topic_filter)
        .await;

    let json_events: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            json!({
                "session": e.session_id,
                "session_name": e.session_name,
                "seq": e.seq,
                "topic": e.topic,
                "payload": e.payload,
                "timestamp": e.timestamp,
            })
        })
        .collect();

    Response::success(
        id,
        json!({
            "events": json_events,
            "count": json_events.len(),
            "sessions": agg.session_count().await,
        }),
    )
    .into()
}

/// Handle `event.collect` — poll events from multiple sessions (fan-in).
///
/// Params: { targets?: [string], since?: {session_id: seq}, topic?: string }
/// If targets is omitted, collects from all live sessions.
async fn handle_event_collect(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    // Resolve target sessions
    let registrations = if let Some(targets) = params.get("targets").and_then(|t| t.as_array()) {
        let mut regs = Vec::new();
        for t in targets {
            if let Some(name) = t.as_str()
                && let Ok(r) = manager::find_session(name) {
                    regs.push(r);
                }
        }
        regs
    } else {
        match manager::list_sessions(false) {
            Ok(sessions) => sessions
                .iter()
                .filter_map(|s| manager::find_session(s.id.as_str()).ok())
                .collect(),
            Err(e) => {
                return ErrorResponse::internal_error(
                    id,
                    &format!("Failed to list sessions: {e}"),
                )
                .into();
            }
        }
    };

    let since_map = params
        .get("since")
        .and_then(|s| s.as_object())
        .cloned()
        .unwrap_or_default();

    // Global since_default: used as fallback when no per-session cursor exists.
    // Enables --since flag at CLI level to replay history from a sequence number.
    let since_default = params.get("since_default").and_then(|s| s.as_u64());

    let topic_filter = params.get("topic").and_then(|t| t.as_str());

    // Optional timeout_ms: when set, use event.subscribe (server-side blocking)
    // instead of event.poll (instant snapshot). This eliminates polling latency
    // for callers that would otherwise sleep between collect calls.
    let subscribe_timeout_ms = params
        .get("timeout_ms")
        .and_then(|t| t.as_u64());

    // Dispatch polls concurrently with per-target timeout
    let mut join_set = tokio::task::JoinSet::new();
    let num_targets = registrations.len().max(1) as u64;

    for reg in registrations {
        let sid = reg.id.to_string();
        let display_name = reg.display_name.clone();
        let addr = reg.addr.to_transport_addr();
        let since_map = since_map.clone();
        let topic_filter = topic_filter.map(String::from);

        join_set.spawn(async move {
            // Choose RPC method based on timeout_ms parameter
            let (method, rpc_params) = if let Some(timeout_ms) = subscribe_timeout_ms {
                let per_session_timeout = timeout_ms / num_targets;
                let effective_timeout = per_session_timeout.max(100); // at least 100ms
                let mut p = json!({"timeout_ms": effective_timeout});
                if let Some(seq_val) = since_map.get(&sid) {
                    p["since"] = seq_val.clone();
                } else if let Some(default_seq) = since_default {
                    p["since"] = json!(default_seq);
                }
                if let Some(t) = &topic_filter {
                    p["topic"] = json!(t);
                }
                (control::method::EVENT_SUBSCRIBE, p)
            } else {
                let mut p = json!({});
                if let Some(seq_val) = since_map.get(&sid) {
                    p["since"] = seq_val.clone();
                } else if let Some(default_seq) = since_default {
                    p["since"] = json!(default_seq);
                }
                if let Some(t) = &topic_filter {
                    p["topic"] = json!(t);
                }
                (control::method::EVENT_POLL, p)
            };

            let result = tokio::time::timeout(
                PER_TARGET_TIMEOUT,
                client::rpc_call_addr(&addr, method, rpc_params),
            )
            .await;

            match result {
                Ok(Ok(resp)) => {
                    if let Ok(result) = client::unwrap_result(resp) {
                        let mut events = Vec::new();
                        if let Some(ev_array) = result["events"].as_array() {
                            for event in ev_array {
                                let mut enriched = event.clone();
                                enriched["session"] = json!(&sid);
                                enriched["session_name"] = json!(&display_name);
                                events.push(enriched);
                            }
                        }
                        let next_seq = result.get("next_seq").cloned();
                        Some((sid, events, next_seq))
                    } else {
                        None
                    }
                }
                Ok(Err(e)) => {
                    tracing::debug!(session = %sid, error = %e, "Collect: failed to reach session");
                    None
                }
                Err(_) => {
                    tracing::debug!(session = %sid, "Collect: timeout reaching session");
                    None
                }
            }
        });
    }

    let mut all_events: Vec<serde_json::Value> = Vec::new();
    let mut cursors = json!({});

    while let Some(result) = join_set.join_next().await {
        if let Ok(Some((sid, events, next_seq))) = result {
            all_events.extend(events);
            if let Some(next) = next_seq {
                cursors[sid] = next;
            }
        }
    }

    // Sort by timestamp, then seq
    all_events.sort_by(|a, b| {
        let ta = a["timestamp"].as_u64().unwrap_or(0);
        let tb = b["timestamp"].as_u64().unwrap_or(0);
        ta.cmp(&tb)
            .then_with(|| {
                let sa = a["seq"].as_u64().unwrap_or(0);
                let sb = b["seq"].as_u64().unwrap_or(0);
                sa.cmp(&sb)
            })
    });

    Response::success(
        id,
        json!({
            "events": all_events,
            "count": all_events.len(),
            "cursors": cursors,
        }),
    )
    .into()
}

/// Handle `session.register_remote` — register a TCP session in the hub's memory.
///
/// Params: { display_name, host, port, pid?, roles?, tags?, capabilities? }
fn handle_register_remote(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let store = match remote_store() {
        Some(s) => s,
        None => {
            return ErrorResponse::internal_error(id, "Remote store not initialized").into();
        }
    };

    let host = match params.get("host").and_then(|h| h.as_str()) {
        Some(h) => h.to_string(),
        None => return ErrorResponse::new(id, -32602, "Missing 'host' in params").into(),
    };
    let port = match params.get("port").and_then(|p| p.as_u64()) {
        Some(p) => p as u16,
        None => return ErrorResponse::new(id, -32602, "Missing 'port' in params").into(),
    };
    let display_name = params
        .get("display_name")
        .and_then(|n| n.as_str())
        .unwrap_or("remote")
        .to_string();
    let pid = params.get("pid").and_then(|p| p.as_u64()).map(|p| p as u32);
    let roles = extract_string_array(params, "roles");
    let tags = extract_string_array(params, "tags");
    let capabilities = extract_string_array(params, "capabilities");
    // T-1131: sessions may declare their wire protocol version; default to 1
    // when absent so pre-T-1131 clients keep registering normally.
    let protocol_version = params
        .get("protocol_version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u8)
        .unwrap_or(1);

    let display_name_clone = display_name.clone();
    let host_clone = host.clone();
    let session_id = store.register(crate::remote_store::RemoteSessionInfo {
        display_name,
        host,
        port,
        pid,
        roles,
        tags,
        capabilities,
        protocol_version,
    });
    tracing::info!(id = %session_id, protocol_version, "Remote session registered");

    // T-966: Subscribe aggregator to this session's event bus
    if let Some(agg) = aggregator() {
        let target = SessionTarget {
            id: session_id.clone(),
            display_name: display_name_clone,
            addr: TransportAddr::tcp(host_clone, port),
        };
        tokio::spawn(async move {
            agg.add_session(target).await;
        });
    }

    Response::success(id, json!({ "id": session_id })).into()
}

/// Handle `hub.version` — return the hub's binary version and wire protocol version.
///
/// Tier-A (opaque). No params, no auth beyond what the connection already has.
/// T-1132 (from T-1071 GO) — fleet doctor calls this to surface version diversity
/// across the fleet before a Tier-B RPC fails on a skewed hub.
fn handle_hub_version(id: serde_json::Value) -> RpcResponse {
    Response::success(
        id,
        json!({
            "hub_version": env!("CARGO_PKG_VERSION"),
            "protocol_version": termlink_protocol::DATA_PLANE_VERSION,
            "control_plane_version": termlink_protocol::CONTROL_PLANE_VERSION,
        }),
    )
    .into()
}

/// Handle `hub.legacy_usage` — return T-1166 cut-readiness telemetry.
///
/// Tier-A (opaque). Optional params `{"window_seconds": <u64>}` (default 7d).
/// Reads the local rpc-audit.jsonl, filters by window, returns counts +
/// last-seen + per-caller breakdown for every legacy method.
///
/// T-1432: fleet doctor walks each reachable hub and aggregates these into
/// a fleet-wide cut verdict (CUT-READY iff every hub reports total_legacy=0
/// for the bake window).
fn handle_hub_legacy_usage(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let window_seconds = params
        .get("window_seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(7 * 86400); // default 7d
    let summary = crate::rpc_audit::summarize_legacy_usage(window_seconds);
    Response::success(id, summary).into()
}

/// Handle `hub.bus_state` — return G-050 audit telemetry: runtime_dir
/// path + bus/meta.db presence/size/mtime + a heuristic volatility flag
/// (true iff runtime_dir starts with `/tmp/`).
///
/// Tier-A. No params, scope=Observe. T-1446 (G-050 audit-sweep follow-up
/// to T-1444 NO-GO). Fleet doctor walks each reachable hub via
/// `--topic-durability` and aggregates a fleet-wide verdict.
fn handle_hub_bus_state(id: serde_json::Value) -> RpcResponse {
    let runtime_dir = termlink_session::discovery::runtime_dir();
    let meta_db = runtime_dir.join("bus").join("meta.db");
    let runtime_dir_str = runtime_dir.to_string_lossy().to_string();

    let (audit_present, size_bytes, mtime_unix) = match std::fs::metadata(&meta_db) {
        Ok(m) => {
            let mtime = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            (true, m.len(), mtime)
        }
        Err(_) => (false, 0u64, 0u64),
    };

    // Heuristic: any path under `/tmp/` is presumed volatile. This catches
    // the legacy `/tmp/termlink-0` default plus any operator override that
    // accidentally lands on /tmp. False positives possible (a deliberately
    // /tmp-on-disk setup would be flagged, but that's vanishingly rare for
    // production hubs and the operator can dismiss the warning).
    let runtime_dir_volatile = runtime_dir_str.starts_with("/tmp/");

    Response::success(
        id,
        json!({
            "runtime_dir": runtime_dir_str,
            "runtime_dir_volatile": runtime_dir_volatile,
            "audit_present": audit_present,
            "meta_db_size_bytes": size_bytes,
            "meta_db_mtime_unix": mtime_unix,
        }),
    )
    .into()
}

/// Handle `hub.governor_status` — return T-2048 governor telemetry.
///
/// Tier-A. No params, scope=Observe. Reads from the process-global
/// `ConnGovernor` + `RateGovernor` installed at hub start (or lazily
/// initialised with defaults if `governor::init` was somehow skipped).
///
/// Response shape:
/// ```json
/// {
///   "connections_active": u32,
///   "connections_max": u32,
///   "capacity_hits_total": u64,
///   "rate_buckets_active": usize,
///   "rate_buckets_evicted_total": u64,
///   "rate_hits_total": u64,
///   "max_rate_per_sec": u32,
///   "dedupe_entries_active": u64,
///   "dedupe_hits_total": u64,
///   "dedupe_ttl_ms": i64,
///   "cv_index_entries_active": u64,
///   "cv_index_topics_active": u64,
///   "cv_index_overflow_total": u64,
///   "cv_index_cap_per_topic": u64,
///   "webhook_enabled": bool,
///   "webhook_target_count": u64,
///   "webhook_retry_depth": u64,
///   "webhook_enqueued_total": u64,
///   "webhook_retry_success_total": u64,
///   "webhook_dropped_full_total": u64,
///   "webhook_dead_letter_total": u64
/// }
/// ```
///
/// Pair with `hub.bus_state` (G-050) for full hub-health rollup. T-1991
/// "found in production not predicted" failure mode becomes a
/// "fleet doctor surfaces it" event as soon as a wrapper consumes
/// this RPC. T-2049 adds the three `dedupe_*` fields so operators can
/// see how many client_msg_id duplicates the hub has absorbed (a
/// non-zero `dedupe_hits_total` is the smoking gun for "hub blip
/// caused a spoke retry — and we caught it before subscribers saw
/// the double-apply"). T-2110 adds the four `cv_index_*` fields so
/// operators can monitor substrate primitive #9 health — a non-zero
/// `cv_index_overflow_total` means some topic has saturated its
/// per-topic cap and new cv-tagged posts are being silently
/// un-indexed (likely poster mis-emitting cv_key). T-2139 adds
/// `rate_buckets_evicted_total` so operators can confirm the T-2137
/// rate-bucket eviction loop is firing (non-zero = active eviction;
/// stuck at zero with growing `rate_buckets_active` = unwired loop or
/// pre-T-2137 binary). T-2335 adds the seven `webhook_*` fields so
/// operators can observe the arc-004 outbound webhook fan-out subsystem:
/// `webhook_enabled` false = the opt-in subsystem is inert (no
/// `TERMLINK_WEBHOOK_CONFIG`); a non-zero `webhook_dead_letter_total`
/// means some external endpoint failed past `WEBHOOK_MAX_ATTEMPTS`; a
/// growing `webhook_retry_depth` means a target is currently unreachable
/// and posts are backing up in the in-memory retry queue.
fn handle_hub_governor_status(id: serde_json::Value) -> RpcResponse {
    let conn = crate::governor::conn_governor();
    let rate = crate::governor::rate_governor();
    let dedupe = crate::dedupe::post_dedupe();
    Response::success(
        id,
        json!({
            "connections_active": conn.current(),
            "connections_max": conn.max(),
            "capacity_hits_total": conn.capacity_hits_total(),
            "rate_buckets_active": rate.buckets_active(),
            "rate_buckets_evicted_total": rate.evictions_total(),
            "rate_hits_total": rate.rate_hits_total(),
            "max_rate_per_sec": rate.rate_per_sec(),
            "dedupe_entries_active": dedupe.entries_active(),
            "dedupe_hits_total": dedupe.hits_total(),
            "dedupe_ttl_ms": dedupe.ttl_ms(),
            "cv_index_entries_active": crate::cv_index::entries_active(),
            "cv_index_topics_active": crate::cv_index::topics_active(),
            "cv_index_overflow_total": crate::cv_index::overflow_total(),
            "cv_index_cap_per_topic": crate::cv_index::cap_per_topic() as u64,
            // T-2335: webhook fan-out (arc-004) observability. `webhook_enabled`
            // false ⇒ TERMLINK_WEBHOOK_CONFIG unset/empty (subsystem inert,
            // Directive-4 no-hard-dependency). Non-zero `webhook_dead_letter_total`
            // is the smoking gun that some external endpoint has been failing past
            // WEBHOOK_MAX_ATTEMPTS; a growing `webhook_retry_depth` means a target
            // is currently unreachable and posts are backing up in the retry queue.
            "webhook_enabled": crate::webhook::webhooks().is_some(),
            "webhook_target_count": crate::webhook::target_count() as u64,
            "webhook_retry_depth": crate::webhook::retry_queue().depth() as u64,
            "webhook_enqueued_total": crate::webhook::retry_queue().enqueued_total(),
            "webhook_retry_success_total": crate::webhook::retry_queue().retry_success_total(),
            "webhook_dropped_full_total": crate::webhook::retry_queue().dropped_full_total(),
            "webhook_dead_letter_total": crate::webhook::retry_queue().dead_letter_total(),
        }),
    )
    .into()
}

/// Handle `hub.capabilities` — return the list of JSON-RPC methods this hub
/// serves directly (T-1215 / T-1214 GO Option B). Enables federating clients
/// to detect stranger-lineage peers and avoid probing each method individually.
///
/// Response shape:
/// ```json
/// { "methods": ["channel.list", ..., "session.discover"], "hub_version": "...", "protocol_version": "..." }
/// ```
/// `methods` is sorted. Only methods recognized by `route()`'s explicit match
/// arms are listed — forwarded session methods are intentionally excluded.
fn handle_hub_capabilities(id: serde_json::Value) -> RpcResponse {
    // Kept in sync with the match arms in `route()`. Excludes the `_ =>
    // forward_to_target` catchall and hub.auth (which is handled at the TLS
    // frame layer, not by this router).
    //
    // T-1166 / T-1415: `event.broadcast`, `inbox.list`, `inbox.status`,
    // `inbox.clear` were retired (cut landed 2026-05-31). Their advertisement
    // here was removed 2026-06-05 — capability consumers no longer get told a
    // method exists that the hub returns -32601 for.
    let mut methods: Vec<&'static str> = vec![
        control::method::SESSION_DISCOVER,
        control::method::SESSION_WHOAMI,
        control::method::EVENT_COLLECT,
        control::method::EVENT_SUBSCRIBE,
        control::method::EVENT_EMIT_TO,
        control::method::ORCHESTRATOR_ROUTE,
        control::method::ORCHESTRATOR_BYPASS_STATUS,
        control::method::ORCHESTRATOR_BYPASS_INVALIDATE,
        "session.register_remote",
        "session.heartbeat",
        "session.deregister_remote",
        control::method::CHANNEL_CREATE,
        control::method::CHANNEL_SET_RETENTION,
        control::method::CHANNEL_SWEEP,
        control::method::CHANNEL_POST,
        control::method::CHANNEL_SUBSCRIBE,
        control::method::CHANNEL_LIST,
        control::method::CHANNEL_TRIM,
        control::method::CHANNEL_DELETE,
        control::method::CHANNEL_RECEIPTS,
        control::method::CHANNEL_CLAIM,
        control::method::CHANNEL_RELEASE,
        control::method::CHANNEL_FORCE_RELEASE,
        control::method::CHANNEL_TRANSFER_CLAIM,
        control::method::CHANNEL_RENEW,
        control::method::CHANNEL_CLAIMS,
        control::method::CHANNEL_CLAIMS_SUMMARY,
        control::method::CHANNEL_CV_KEYS,
        control::method::AGENT_FIND_IDLE,
        control::method::DIALOG_PRESENCE,
        control::method::ARTIFACT_PUT,
        control::method::ARTIFACT_GET,
        "hub.version",
        "hub.legacy_usage",
        "hub.bus_state",
        "hub.governor_status",
        control::method::HUB_CAPABILITIES,
    ];
    methods.sort_unstable();

    // T-1405 / T-1415: legacy_primitives is now hardcoded false (cut landed
    // 2026-05-31). Field retained for downstream consumers that probe
    // post-cut vs pre-cut hubs (returns false on every post-cut hub).
    // See docs/migrations/T-1166-retire-legacy-primitives.md.
    let features = json!({
        "legacy_primitives": false,
    });

    Response::success(
        id,
        json!({
            "methods": methods,
            "hub_version": env!("CARGO_PKG_VERSION"),
            "protocol_version": termlink_protocol::DATA_PLANE_VERSION,
            "control_plane_version": termlink_protocol::CONTROL_PLANE_VERSION,
            "features": features,
        }),
    )
    .into()
}

/// Handle `session.heartbeat` — refresh TTL for a remote session.
///
/// Params: { id }
fn handle_heartbeat(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let store = match remote_store() {
        Some(s) => s,
        None => {
            return ErrorResponse::internal_error(id, "Remote store not initialized").into();
        }
    };

    let session_id = match params.get("id").and_then(|i| i.as_str()) {
        Some(i) => i,
        None => return ErrorResponse::new(id, -32602, "Missing 'id' in params").into(),
    };

    if store.heartbeat(session_id) {
        Response::success(id, json!({ "ok": true })).into()
    } else {
        ErrorResponse::new(
            id,
            control::error_code::SESSION_NOT_FOUND,
            &format!("Remote session '{}' not found", session_id),
        )
        .into()
    }
}

/// Handle `session.deregister_remote` — remove a remote session.
///
/// Params: { id }
fn handle_deregister_remote(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let store = match remote_store() {
        Some(s) => s,
        None => {
            return ErrorResponse::internal_error(id, "Remote store not initialized").into();
        }
    };

    let session_id = match params.get("id").and_then(|i| i.as_str()) {
        Some(i) => i,
        None => return ErrorResponse::new(id, -32602, "Missing 'id' in params").into(),
    };

    if store.deregister(session_id) {
        tracing::info!(id = %session_id, "Remote session deregistered");
        // T-966: Remove aggregator subscription
        if let Some(agg) = aggregator() {
            let sid = session_id.to_string();
            tokio::spawn(async move {
                agg.remove_session(&sid).await;
            });
        }
        Response::success(id, json!({ "ok": true })).into()
    } else {
        ErrorResponse::new(
            id,
            control::error_code::SESSION_NOT_FOUND,
            &format!("Remote session '{}' not found", session_id),
        )
        .into()
    }
}

/// Extract a string array from a JSON value by key, defaulting to empty vec.
fn extract_string_array(params: &serde_json::Value, key: &str) -> Vec<String> {
    params
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Handle `orchestrator.route` — discover a specialist, forward a method, relay the response.
///
/// Combines session.discover + forward into a single atomic call:
///   1. Find sessions matching the selector (tags/roles/capabilities/name)
///   2. Forward the specified method+params to the first matching session
///   3. If the first fails, try the next candidate (failover)
///   4. Return the specialist's response plus routing metadata
///
/// When `task_type` is provided, sessions with a matching `task-type:<type>` tag are
/// sorted before other candidates (preferred but not required). This enables task-aware
/// routing without breaking existing method-based routing.
///
/// Params: {
///   selector: { tags?: [...], roles?: [...], capabilities?: [...], name?: string },
///   method: string,       // RPC method to call on the specialist
///   params: object,       // params to pass to the specialist
///   timeout_secs?: number // per-target timeout (default: 5)
///   task_type?: string    // task workflow type (build/test/audit/review) — prefers
///                         // specialists with matching "task-type:<type>" tag
/// }
///
/// Response: {
///   routed_to: { id, display_name },
///   candidates: number,
///   result: <specialist's response payload>
/// }
async fn handle_orchestrator_route(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    // Extract required method
    let method = match params.get("method").and_then(|m| m.as_str()) {
        Some(m) => m.to_string(),
        None => {
            return ErrorResponse::new(id, -32602, "Missing 'method' in params").into();
        }
    };

    // Check if this is a mutating command (skip bypass for read-write operations)
    let mutating = params
        .get("mutating")
        .and_then(|m| m.as_bool())
        .unwrap_or(false);

    // Optional task workflow type (build/test/audit/review) — prefers matching specialists
    let task_type = params.get("task_type").and_then(|t| t.as_str()).map(String::from);

    // Build the cache/bypass key: "method" or "method::task_type" when task_type is present.
    // This ensures task-type-specific routes are cached separately from generic ones.
    let routing_key = match &task_type {
        Some(tt) => format!("{method}::{tt}"),
        None => method.clone(),
    };

    // Check bypass registry before routing to a specialist (skip for mutating commands)
    if !mutating {
        let registry = crate::bypass::BypassRegistry::load();
        if let Some(entry) = registry.check(&routing_key) {
            tracing::info!(
                method = %method,
                routing_key = %routing_key,
                run_count = entry.run_count,
                "orchestrator.route: bypass registry hit — command is Tier 3"
            );
            return Response::success(
                id,
                json!({
                    "bypassed": true,
                    "command": method,
                    "tier": entry.tier,
                    "run_count": entry.run_count,
                    "task_type": task_type,
                    "note": "routing shortcut, not execution authorization",
                }),
            )
            .into();
        }
    }

    // Layer 2: Check route cache (between bypass and full discovery)
    if !mutating {
        let route_cache = crate::route_cache::RouteCache::load();
        match route_cache.lookup(&routing_key) {
            crate::route_cache::CacheLookup::Hit(entry) => {
                tracing::info!(
                    method = %method,
                    specialist = %entry.specialist,
                    confidence = entry.effective_confidence(),
                    hit_count = entry.hit_count,
                    "orchestrator.route: route cache hit"
                );
                // Use cached route as selector hint — filter by specialist name
                let cached_selector = json!({
                    "name": entry.specialist,
                });
                // Fall through to discovery with the cached selector
                // (we override selector below to prefer the cached specialist)
                let forward_params_inner = params.get("params").cloned().unwrap_or(json!({}));
                let timeout_secs_inner = params
                    .get("timeout_secs")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(5);
                let timeout_inner = Duration::from_secs(timeout_secs_inner);

                let sessions = match manager::list_sessions(false) {
                    Ok(s) => s,
                    Err(e) => {
                        return ErrorResponse::internal_error(
                            id,
                            &format!("Failed to list sessions: {e}"),
                        )
                        .into();
                    }
                };

                let name_filter = cached_selector.get("name").and_then(|n| n.as_str());
                let candidates: Vec<_> = sessions
                    .into_iter()
                    .filter(|s| {
                        name_filter.is_none_or(|n| {
                            s.display_name.to_lowercase().contains(&n.to_lowercase())
                        })
                    })
                    .collect();

                if let Some(reg) = candidates.first() {
                    let addr = reg.addr.to_transport_addr();
                    let session_id = reg.id.as_str().to_string();
                    let result = tokio::time::timeout(timeout_inner, async {
                        let mut c = client::Client::connect_addr_raw(&addr).await?;
                        c.call(&method, id.clone(), forward_params_inner.clone()).await
                    })
                    .await;

                    match result {
                        Ok(Ok(RpcResponse::Success(resp))) => {
                            // Record cache hit (keyed by routing_key for task-type awareness)
                            let cache_path = crate::route_cache::cache_path();
                            let rk_clone = routing_key.clone();
                            if let Ok(mut cache) = std::fs::read_to_string(&cache_path)
                                .ok()
                                .and_then(|d| serde_json::from_str::<crate::route_cache::RouteCache>(&d).ok())
                                .ok_or(())
                                .or_else(|_| Ok::<_, ()>(crate::route_cache::RouteCache::default()))
                            {
                                cache.record_hit(&rk_clone);
                                let _ = cache.save_to(&cache_path);
                            }

                            // Also record in bypass registry
                            if !mutating {
                                let reg_path = crate::bypass::registry_path();
                                let rk_clone2 = routing_key.clone();
                                let _ = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                                    let _ = r.record_orchestrated_run(&rk_clone2, crate::bypass::RunOutcome::Success);
                                });
                            }

                            return Response::success(
                                id,
                                json!({
                                    "routed_to": {
                                        "id": session_id,
                                        "display_name": reg.display_name,
                                    },
                                    "cached_route": true,
                                    "candidates": 1,
                                    "result": resp.result,
                                }),
                            )
                            .into();
                        }
                        _ => {
                            // Cache route failed — invalidate and fall through to full discovery
                            tracing::warn!(
                                method = %method,
                                routing_key = %routing_key,
                                "orchestrator.route: cached route failed, falling through to full discovery"
                            );
                            let cache_path = crate::route_cache::cache_path();
                            let rk_clone = routing_key.clone();
                            if let Ok(mut cache) = std::fs::read_to_string(&cache_path)
                                .ok()
                                .and_then(|d| serde_json::from_str::<crate::route_cache::RouteCache>(&d).ok())
                                .ok_or(())
                                .or_else(|_| Ok::<_, ()>(crate::route_cache::RouteCache::default()))
                            {
                                cache.invalidate(&rk_clone);
                                let _ = cache.save_to(&cache_path);
                            }
                        }
                    }
                }
                // Cached specialist not found or failed — fall through to normal discovery
            }
            crate::route_cache::CacheLookup::Stale(entry) => {
                tracing::debug!(
                    method = %method,
                    specialist = %entry.specialist,
                    confidence = entry.effective_confidence(),
                    "orchestrator.route: route cache stale, proceeding to full discovery"
                );
                // Fall through to normal discovery (stale hint logged but not used)
            }
            crate::route_cache::CacheLookup::Miss => {
                // No cache entry — normal discovery
            }
        }
    }

    let forward_params = params.get("params").cloned().unwrap_or(json!({}));
    let selector = params.get("selector").cloned().unwrap_or(json!({}));
    let timeout_secs = params
        .get("timeout_secs")
        .and_then(|t| t.as_u64())
        .unwrap_or(5);
    let timeout = Duration::from_secs(timeout_secs);

    // Discover candidates using same filter logic as session.discover
    let sessions = match manager::list_sessions(false) {
        Ok(s) => s,
        Err(e) => {
            return ErrorResponse::internal_error(
                id,
                &format!("Failed to list sessions: {e}"),
            )
            .into();
        }
    };

    let tag_filter = extract_string_array(&selector, "tags");
    let role_filter = extract_string_array(&selector, "roles");
    let cap_filter = extract_string_array(&selector, "capabilities");
    let name_filter = selector.get("name").and_then(|n| n.as_str());

    let mut candidates: Vec<_> = sessions
        .into_iter()
        .filter(|s| {
            tag_filter.iter().all(|t| s.tags.contains(t))
                && role_filter.iter().all(|r| s.roles.contains(r))
                && cap_filter.iter().all(|c| s.capabilities.contains(c))
                && name_filter.is_none_or(|n| {
                    s.display_name.to_lowercase().contains(&n.to_lowercase())
                })
        })
        .collect();

    // Task-type preference: sort candidates so sessions with a matching
    // "task-type:<type>" tag appear before others. This is a stable sort —
    // within each group the original order (creation time) is preserved.
    if let Some(ref tt) = task_type {
        let task_type_tag = format!("task-type:{tt}");
        candidates.sort_by_key(|s| if s.tags.contains(&task_type_tag) { 0u8 } else { 1u8 });
    }

    // Also check remote sessions
    if let Some(store) = remote_store() {
        let remote_matches: Vec<_> = store
            .list_live()
            .into_iter()
            .filter(|e| {
                tag_filter.iter().all(|t| e.tags.contains(t))
                    && role_filter.iter().all(|r| e.roles.contains(r))
                    && cap_filter.iter().all(|c| e.capabilities.contains(c))
                    && name_filter.is_none_or(|n| {
                        e.display_name.to_lowercase().contains(&n.to_lowercase())
                    })
            })
            .collect();

        // Convert remote entries to a forwarding attempt below
        for entry in remote_matches {
            // Try remote candidates after local ones
            let addr = TransportAddr::Tcp {
                host: entry.host.clone(),
                port: entry.port,
            };
            let result = tokio::time::timeout(timeout, async {
                let mut c = client::Client::connect_addr_raw(&addr).await?;
                c.call(&method, id.clone(), forward_params.clone()).await
            })
            .await;

            if let Ok(Ok(RpcResponse::Success(resp))) = result {
                return Response::success(
                    id,
                    json!({
                        "routed_to": { "id": entry.id, "display_name": entry.display_name },
                        "candidates": candidates.len() + 1,
                        "result": resp.result,
                    }),
                )
                .into();
            }
        }
    }

    let total_candidates = candidates.len();

    if candidates.is_empty() {
        return ErrorResponse::new(
            id,
            control::error_code::SESSION_NOT_FOUND,
            "No sessions match the selector",
        )
        .into();
    }

    // Try candidates in order (failover), skipping circuit-opened sessions
    let cb = crate::circuit_breaker::global();
    let mut last_error = String::new();
    let mut skipped_count = 0usize;
    let mut tried_count = 0usize;
    for reg in candidates.drain(..) {
        let session_id = reg.id.as_str().to_string();

        // Skip sessions with open circuits (avoids cascading timeout delays)
        if cb.should_skip(&session_id) {
            skipped_count += 1;
            tracing::debug!(
                session = %session_id,
                "orchestrator.route: circuit open — skipping candidate"
            );
            last_error = format!("{}: circuit open (skipped)", reg.display_name);
            continue;
        }

        tried_count += 1;
        let addr = reg.addr.to_transport_addr();
        let result = tokio::time::timeout(timeout, async {
            let mut c = client::Client::connect_addr_raw(&addr).await?;
            c.call(&method, id.clone(), forward_params.clone()).await
        })
        .await;

        match result {
            Ok(Ok(RpcResponse::Success(resp))) => {
                cb.record_success(&session_id);
                // Record successful orchestrated run (skip for mutating commands)
                if !mutating {
                    let reg_path = crate::bypass::registry_path();
                    let rk_clone = routing_key.clone();
                    let _ =
                        crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                            if r.record_orchestrated_run(&rk_clone, crate::bypass::RunOutcome::Success) {
                                tracing::info!(
                                    routing_key = %rk_clone,
                                    "orchestrator.route: command promoted to bypass registry"
                                );
                            }
                        });

                    // Record route in cache (Layer 2) for future lookups
                    let cache_path = crate::route_cache::cache_path();
                    let rk_for_cache = routing_key.clone();
                    let specialist_name = reg.display_name.clone();
                    let mut route_cache = crate::route_cache::RouteCache::load_from(&cache_path);
                    route_cache.record_route(
                        &rk_for_cache,
                        &specialist_name,
                        crate::route_cache::RequestSchema::default(),
                    );
                    let _ = route_cache.save_to(&cache_path);
                    tracing::debug!(
                        routing_key = %rk_for_cache,
                        specialist = %specialist_name,
                        "orchestrator.route: recorded route in cache"
                    );
                }
                return Response::success(
                    id,
                    json!({
                        "routed_to": {
                            "id": reg.id.as_str(),
                            "display_name": reg.display_name,
                        },
                        "candidates": total_candidates,
                        "result": resp.result,
                    }),
                )
                .into();
            }
            Ok(Ok(RpcResponse::Error(e))) => {
                // RPC error = command failure (the specialist responded with an error)
                // Command failures don't open the circuit (session is alive, just rejected the call)
                if !mutating {
                    let reg_path = crate::bypass::registry_path();
                    let rk_clone = routing_key.clone();
                    let _ = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                        r.record_orchestrated_run(&rk_clone, crate::bypass::RunOutcome::CommandFailure);
                    });
                }
                last_error = format!("{}: {}", reg.display_name, e.error.message);
                tracing::debug!(
                    target = reg.display_name,
                    error = %e.error.message,
                    "orchestrator.route: candidate returned error, trying next"
                );
            }
            Ok(Err(e)) => {
                cb.record_failure(&session_id);
                // Connection error = infra failure (specialist never received the call)
                if !mutating {
                    let reg_path = crate::bypass::registry_path();
                    let rk_clone = routing_key.clone();
                    let _ = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                        r.record_orchestrated_run(&rk_clone, crate::bypass::RunOutcome::InfraFailure);
                    });
                }
                last_error = format!("{}: {}", reg.display_name, e);
                tracing::debug!(
                    target = reg.display_name,
                    error = %e,
                    "orchestrator.route: candidate connection failed, trying next"
                );
            }
            Err(_) => {
                cb.record_failure(&session_id);
                // Timeout = infra failure (specialist didn't respond in time)
                if !mutating {
                    let reg_path = crate::bypass::registry_path();
                    let rk_clone = routing_key.clone();
                    let _ = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                        r.record_orchestrated_run(&rk_clone, crate::bypass::RunOutcome::InfraFailure);
                    });
                }
                last_error = format!("{}: timeout", reg.display_name);
                tracing::debug!(
                    target = reg.display_name,
                    "orchestrator.route: candidate timed out, trying next"
                );
            }
        }
    }

    ErrorResponse::new(
        id,
        control::error_code::SESSION_NOT_FOUND,
        &format!(
            "All {} candidate(s) failed ({} tried, {} circuit-open skipped). Last: {}",
            total_candidates, tried_count, skipped_count, last_error
        ),
    )
    .into()
}

/// Handle `orchestrator.bypass_status` — query the bypass registry contents.
fn handle_bypass_status(id: serde_json::Value) -> RpcResponse {
    let registry = crate::bypass::BypassRegistry::load();
    let entries: Vec<_> = registry
        .entries
        .values()
        .map(|e| {
            json!({
                "command": e.command,
                "tier": e.tier,
                "run_count": e.run_count,
                "fail_count": e.fail_count,
                "promoted_at": e.promoted_at,
                "last_run": e.last_run,
            })
        })
        .collect();
    let candidates: Vec<_> = registry
        .candidates
        .iter()
        .map(|(cmd, stats)| {
            json!({
                "command": cmd,
                "success_count": stats.success_count,
                "fail_count": stats.fail_count,
                "remaining": crate::bypass::PROMOTION_THRESHOLD.saturating_sub(stats.success_count),
            })
        })
        .collect();
    Response::success(
        id,
        json!({
            "bypassed_commands": entries,
            "promotion_candidates": candidates,
        }),
    )
    .into()
}

/// Handle `orchestrator.bypass_invalidate` — remove bypass entries by pattern or all.
///
/// Params:
///   - `pattern` (string, optional): substring pattern to match (case-insensitive).
///     If omitted, clears the entire registry.
///   - `all` (bool, optional): if true, clears everything (same as omitting pattern).
fn handle_bypass_invalidate(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let all = params
        .get("all")
        .and_then(|a| a.as_bool())
        .unwrap_or(false);
    let pattern = params.get("pattern").and_then(|p| p.as_str());

    let reg_path = crate::bypass::registry_path();
    let result = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
        let removed = if all || pattern.is_none() {
            r.invalidate_all()
        } else if let Some(pat) = pattern {
            r.invalidate(pat)
        } else {
            unreachable!()
        };
        tracing::info!(
            pattern = pattern.unwrap_or("*"),
            removed,
            "orchestrator.bypass_invalidate: cleared bypass entries"
        );
    });

    match result {
        Ok(registry) => Response::success(
            id,
            json!({
                "invalidated": true,
                "remaining_entries": registry.entries.len(),
                "remaining_candidates": registry.candidates.len(),
            }),
        )
        .into(),
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("Failed to update registry: {e}")).into()
        }
    }
}

/// Forward a request to the target session specified in params.target.
async fn forward_to_target(req: &Request, id: serde_json::Value) -> RpcResponse {
    // Extract target from params
    let target = match req.params.get("target").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return ErrorResponse::new(
                id,
                termlink_protocol::control::error_code::SESSION_NOT_FOUND,
                "Missing 'target' in params",
            )
            .into();
        }
    };

    // Resolve target: try local FS first, then remote store
    let addr = if let Ok(reg) = manager::find_session(target) {
        reg.addr.to_transport_addr()
    } else if let Some(entry) = remote_store().and_then(|s| {
        // Try by ID first, then by display name
        s.get(target).or_else(|| {
            s.list_live()
                .into_iter()
                .find(|e| e.display_name == target || e.id == target)
        })
    }) {
        TransportAddr::Tcp {
            host: entry.host.clone(),
            port: entry.port,
        }
    } else {
        return ErrorResponse::new(
            id,
            control::error_code::SESSION_NOT_FOUND,
            &format!("Target '{}' not found (local or remote)", target),
        )
        .into();
    };

    // Forward the request, preserving the original request id
    let forward_result = async {
        let mut c = client::Client::connect_addr_raw(&addr).await?;
        c.call(&req.method, id.clone(), req.params.clone()).await
    };
    match forward_result.await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::warn!(
                target = target,
                error = %e,
                "Failed to forward request to session"
            );
            ErrorResponse::new(
                id,
                control::error_code::SESSION_NOT_FOUND,
                &format!("Failed to reach target: {e}"),
            )
            .into()
        }
    }
}

/// Resolve a target string to a transport address.
///
/// Public so the CLI can use direct routing without the hub.
pub fn resolve_target(target: &str) -> Result<TransportAddr, String> {
    // Try local FS first
    if let Ok(r) = manager::find_session(target) {
        return Ok(r.addr.to_transport_addr());
    }
    // Try remote store
    if let Some(entry) = remote_store().and_then(|s| {
        s.get(target).or_else(|| {
            s.list_live()
                .into_iter()
                .find(|e| e.display_name == target || e.id == target)
        })
    }) {
        return Ok(TransportAddr::Tcp {
            host: entry.host,
            port: entry.port,
        });
    }
    Err(format!("Session '{}' not found (local or remote)", target))
}

/// Resolve a target string to a socket path (convenience for Unix-only callers).
pub fn resolve_target_path(target: &str) -> Result<std::path::PathBuf, String> {
    manager::find_session(target)
        .map(|r| r.socket_path().to_path_buf())
        .map_err(|e| e.to_string())
}




/// T-1298: validate topic names at hub emit boundaries. The accepted character
/// set is `[a-z0-9._:-]` (no whitespace, no uppercase, no XML/punctuation),
/// length-capped at 256 bytes. Returns a descriptive error string on failure
/// so the caller can echo it via JSON-RPC `-32602` invalid-params.
///
/// Discovered need: T-1297 Spike 1 found a topic literally named
/// `learning.shared</topic>\n<parameter name="from">email-archive` — XML
/// prompt-interpolation leaked into the topic string and the hub accepted it.
fn validate_topic_name(topic: &str) -> Result<(), String> {
    const MAX_LEN: usize = 256;
    if topic.is_empty() {
        return Err("Empty topic name".to_string());
    }
    if topic.len() > MAX_LEN {
        return Err(format!(
            "Topic name too long ({} bytes, max {MAX_LEN})",
            topic.len()
        ));
    }
    for (i, c) in topic.char_indices() {
        let ok = matches!(c, 'a'..='z' | '0'..='9' | '.' | ':' | '-');
        if !ok {
            // Truncate offending substring to 64 chars in the error
            let preview: String = topic.chars().take(64).collect();
            return Err(format!(
                "Invalid topic name '{preview}': illegal char {c:?} at byte {i} (allowed: a-z 0-9 . : -)"
            ));
        }
    }
    Ok(())
}

/// T-1300: Run topic↔role soft-lint and dual-write a warning envelope on
/// mismatch. The emit path always proceeds; this is best-effort observability.
/// `from` is the optional caller-session id (e.g. `$TERMLINK_SESSION_ID`).
/// When absent, lint is skipped (logged at debug) — the originating client is
/// unidentifiable, so we can't compare its roles to anything.
async fn run_topic_lint(method: &str, topic: &str, from: Option<&str>) {
    let Some(from) = from else {
        tracing::debug!(method, topic, "topic_lint: no `from` — skipping");
        return;
    };
    let (caller_roles, display_name) = match manager::find_session(from) {
        Ok(reg) => (reg.roles, reg.display_name),
        Err(e) => {
            tracing::debug!(
                method, topic, from,
                error = %e,
                "topic_lint: caller session not resolvable — skipping"
            );
            return;
        }
    };
    let rules = topic_lint::current_rules();
    let outcome = topic_lint::lint(topic, &caller_roles, &rules);
    if let LintOutcome::Warn {
        rule_prefix,
        expected_roles,
        actual_roles,
    } = outcome
    {
        // T-1301: Caller's relay_for declaration may cover this topic; if so,
        // suppress the warning entirely (no log, no dual-write).
        let relay_for = topic_lint::current_relay_for(&display_name);
        if topic_lint::relay_suppresses(topic, &relay_for) {
            tracing::debug!(
                method, topic, from, display_name,
                relay_for = ?relay_for,
                "topic_lint: WARN suppressed by caller's relay_for declaration"
            );
            return;
        }
        let payload = topic_lint::warning_payload(
            method,
            topic,
            Some(from),
            &rule_prefix,
            &expected_roles,
            &actual_roles,
        );
        tracing::warn!(
            method, topic, from, display_name,
            rule_prefix = %rule_prefix,
            expected = ?expected_roles,
            actual = ?actual_roles,
            "topic_lint: WARN — caller role does not match topic prefix policy"
        );
        crate::channel::mirror_routing_lint_warning(method, &payload).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::io::AsyncWriteExt;
    use tokio::sync::RwLock;

    use termlink_session::handler::SessionContext;
    use termlink_session::registration::SessionConfig;
    use termlink_session::Registration;
    use termlink_session::server;

    use crate::test_util::ENV_LOCK;
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(format!("/tmp/tl-hub-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    async fn start_test_session(
        sessions_dir: &Path,
        name: &str,
    ) -> (
        tokio::task::JoinHandle<()>,
        Registration,
    ) {
        let config = SessionConfig {
            display_name: Some(name.into()),
            ..Default::default()
        };
        let session = termlink_session::Session::register_in(config, sessions_dir)
            .await
            .unwrap();

        let session_id = session.id().clone();
        let (registration, listener, _) = session.into_parts();
        let reg = registration.clone();
        let json_path = Registration::json_path(sessions_dir, &session_id);
        let ctx = SessionContext::new(registration)
            .with_registration_path(json_path);
        let shared = Arc::new(RwLock::new(ctx));

        let handle = tokio::spawn(async move {
            server::run_accept_loop(listener, shared).await;
        });

        (handle, reg)
    }

    async fn start_test_session_with_tags(
        sessions_dir: &Path,
        name: &str,
        tags: Vec<String>,
    ) -> (
        tokio::task::JoinHandle<()>,
        Registration,
    ) {
        let config = SessionConfig {
            display_name: Some(name.into()),
            tags,
            ..Default::default()
        };
        let session = termlink_session::Session::register_in(config, sessions_dir)
            .await
            .unwrap();

        let session_id = session.id().clone();
        let (registration, listener, _) = session.into_parts();
        let reg = registration.clone();
        let json_path = Registration::json_path(sessions_dir, &session_id);
        let ctx = SessionContext::new(registration)
            .with_registration_path(json_path);
        let shared = Arc::new(RwLock::new(ctx));

        let handle = tokio::spawn(async move {
            server::run_accept_loop(listener, shared).await;
        });

        (handle, reg)
    }

    #[tokio::test]
    async fn discover_returns_sessions() {
        let dir = test_dir();

        let (h1, _r1) = start_test_session(&dir, "session-a").await;
        let (h2, _r2) = start_test_session(&dir, "session-b").await;

        // Discover using list_sessions_in directly
        let sessions = manager::list_sessions_in(&dir, false).unwrap();
        assert_eq!(sessions.len(), 2);

        let names: Vec<&str> = sessions.iter().map(|s| s.display_name.as_str()).collect();
        assert!(names.contains(&"session-a"));
        assert!(names.contains(&"session-b"));

        h1.abort();
        h2.abort();
    }

    /// T-1441: handle_discover surfaces identity_fingerprint per session
    /// so `remote list` can render the FP column. Field is omitted (not
    /// null) for pre-T-1436 sessions; present when a session loaded an
    /// agent identity at registration.
    #[tokio::test]
    async fn discover_includes_identity_fingerprint_when_present() {
        let _lock = ENV_LOCK.lock().await;
        if let Some(s) = super::remote_store() { s.clear(); }
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "fp-probe").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_discover(json!("fp-1"), &json!({"name": "fp-probe"})).await;
        if let RpcResponse::Success(r) = resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert_eq!(sessions.len(), 1);
            // identity_fingerprint reflects what r1.metadata recorded.
            // start_test_session uses Session::register_in → Registration::new
            // → load_identity_fingerprint_best_effort. Field is Some when
            // ~/.termlink/identity.key exists in this environment, None
            // otherwise — assert the returned JSON tracks that.
            let actual_fp = sessions[0]["identity_fingerprint"].as_str();
            let expected_fp = r1.metadata.identity_fingerprint.as_deref();
            assert_eq!(
                actual_fp, expected_fp,
                "discover JSON must surface session metadata identity_fingerprint"
            );
        } else {
            panic!("expected success response from handle_discover");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        h1.abort();
    }

    #[tokio::test]
    async fn forward_to_target_session() {
        let dir = test_dir();

        let (handle, reg) = start_test_session(&dir, "target-sess").await;

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Send a ping directly to the session (simulating hub forwarding)
        let resp = client::rpc_call(
            reg.socket_path(),
            "termlink.ping",
            json!({}),
        )
        .await
        .unwrap();

        let result = client::unwrap_result(resp).unwrap();
        assert_eq!(result["display_name"], "target-sess");

        handle.abort();
    }

    #[tokio::test]
    async fn forward_missing_target_returns_error() {
        let req = Request::new(
            "query.status",
            json!("req-1"),
            json!({"target": "nonexistent-session"}),
        );

        let resp = route(&req, None).await.unwrap();
        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, control::error_code::SESSION_NOT_FOUND);
        } else {
            panic!("Expected error response");
        }
    }



    #[tokio::test]
    async fn collect_aggregates_events() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "coll-a").await;
        let (h2, r2) = start_test_session(&sessions_dir, "coll-b").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Emit events to each session directly
        client::rpc_call(
            r1.socket_path(),
            "event.emit",
            json!({"topic": "build.done", "payload": {"id": 1}}),
        ).await.unwrap();

        client::rpc_call(
            r2.socket_path(),
            "event.emit",
            json!({"topic": "test.pass", "payload": {"id": 2}}),
        ).await.unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_event_collect(json!("cl-1"), &json!({})).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 2);
            let events = r.result["events"].as_array().unwrap();
            let topics: Vec<&str> = events.iter().filter_map(|e| e["topic"].as_str()).collect();
            assert!(topics.contains(&"build.done"));
            assert!(topics.contains(&"test.pass"));

            // Each event should have session metadata
            for event in events {
                assert!(event.get("session").is_some());
                assert!(event.get("session_name").is_some());
            }

            // Cursors should be present
            let cursors = r.result["cursors"].as_object().unwrap();
            assert_eq!(cursors.len(), 2);
        } else {
            panic!("Expected success response");
        }

        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn collect_with_since_cursors() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "cur-a").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Emit two events
        client::rpc_call(
            r1.socket_path(),
            "event.emit",
            json!({"topic": "a", "payload": {}}),
        ).await.unwrap();
        client::rpc_call(
            r1.socket_path(),
            "event.emit",
            json!({"topic": "b", "payload": {}}),
        ).await.unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Collect with since cursor at seq 0 — should get only event at seq 1
        let sid = r1.id.as_str();
        let params = json!({
            "since": { sid: 0 },
        });
        let resp = handle_event_collect(json!("cl-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 1);
            let events = r.result["events"].as_array().unwrap();
            assert_eq!(events[0]["topic"], "b");
        } else {
            panic!("Expected success response");
        }

        h1.abort();
    }


    #[tokio::test]
    async fn whoami_resolves_by_session_id() {
        let _lock = ENV_LOCK.lock().await;
        if let Some(s) = super::remote_store() { s.clear(); }
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        let (h, r) = start_test_session(&sessions_dir, "whoami-id-target").await;
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_whoami(
            json!("w-1"),
            &json!({ "session_id": r.id.as_str() }),
        ).await;

        if let RpcResponse::Success(r2) = resp {
            assert_eq!(r2.result["ok"], json!(true));
            assert_eq!(r2.result["session"]["display_name"], "whoami-id-target");
            assert_eq!(r2.result["session"]["id"], r.id.as_str());
        } else {
            panic!("Expected success");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        h.abort();
    }

    #[tokio::test]
    async fn whoami_resolves_by_display_name() {
        let _lock = ENV_LOCK.lock().await;
        if let Some(s) = super::remote_store() { s.clear(); }
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        let (h, _r) = start_test_session(&sessions_dir, "whoami-name-target").await;
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_whoami(
            json!("w-2"),
            &json!({ "display_name": "whoami-name-target" }),
        ).await;

        if let RpcResponse::Success(r2) = resp {
            assert_eq!(r2.result["ok"], json!(true));
            assert_eq!(r2.result["session"]["display_name"], "whoami-name-target");
        } else {
            panic!("Expected success");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        h.abort();
    }

    #[tokio::test]
    async fn whoami_no_hint_returns_candidate_list() {
        let _lock = ENV_LOCK.lock().await;
        if let Some(s) = super::remote_store() { s.clear(); }
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        let (h1, _r1) = start_test_session(&sessions_dir, "whoami-cand-a").await;
        let (h2, _r2) = start_test_session(&sessions_dir, "whoami-cand-b").await;
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_whoami(json!("w-3"), &json!({})).await;

        if let RpcResponse::Success(r2) = resp {
            assert_eq!(r2.result["ok"], json!(true));
            assert_eq!(r2.result["ambiguous"], json!(true));
            let candidates = r2.result["candidates"].as_array().unwrap();
            assert_eq!(candidates.len(), 2);
            let names: Vec<&str> = candidates.iter()
                .map(|c| c["display_name"].as_str().unwrap())
                .collect();
            assert!(names.contains(&"whoami-cand-a"));
            assert!(names.contains(&"whoami-cand-b"));
            assert!(r2.result["hint"].as_str().unwrap().contains("TERMLINK_SESSION_ID"));
        } else {
            panic!("Expected success");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn whoami_unknown_hint_returns_not_found_with_helpful_hint() {
        let _lock = ENV_LOCK.lock().await;
        if let Some(s) = super::remote_store() { s.clear(); }
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_whoami(
            json!("w-4"),
            &json!({ "session_id": "tl-does-not-exist" }),
        ).await;

        if let RpcResponse::Success(r2) = resp {
            assert_eq!(r2.result["ok"], json!(false));
            assert_eq!(r2.result["found"], json!(false));
            assert!(r2.result["hint"].as_str().unwrap().contains("TERMLINK_SESSION_ID"));
        } else {
            panic!("Expected success (not_found is success-with-payload, not RPC error)");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
    }

    #[tokio::test]
    async fn discover_with_filters() {
        let _lock = ENV_LOCK.lock().await;
        // Clear remote store to avoid leakage from other tests
        if let Some(s) = super::remote_store() { s.clear(); }
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Register sessions with different tags via session.update
        let (h1, r1) = start_test_session(&sessions_dir, "web-prod").await;
        let (h2, r2) = start_test_session(&sessions_dir, "api-staging").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Tag session 1 as "prod"
        client::rpc_call(
            r1.socket_path(),
            "session.update",
            json!({"tags": ["prod", "web"]}),
        ).await.unwrap();

        // Tag session 2 as "staging"
        client::rpc_call(
            r2.socket_path(),
            "session.update",
            json!({"tags": ["staging", "api"]}),
        ).await.unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Discover with tag filter — only prod
        let resp = handle_discover(json!("d-1"), &json!({"tags": ["prod"]})).await;

        if let RpcResponse::Success(r) = resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0]["display_name"], "web-prod");
            assert!(sessions[0]["tags"].as_array().unwrap().contains(&json!("prod")));
        } else {
            panic!("Expected success");
        }

        // Discover with name filter
        let resp = handle_discover(json!("d-2"), &json!({"name": "api"})).await;

        if let RpcResponse::Success(r) = resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0]["display_name"], "api-staging");
        } else {
            panic!("Expected success");
        }

        // Discover with no filters — gets both
        let resp = handle_discover(json!("d-3"), &json!({})).await;

        if let RpcResponse::Success(r) = resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert_eq!(sessions.len(), 2);
        } else {
            panic!("Expected success");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn forward_without_target_param_returns_error() {
        let req = Request::new(
            "query.status",
            json!("req-1"),
            json!({}), // no target
        );

        let resp = route(&req, None).await.unwrap();
        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, control::error_code::SESSION_NOT_FOUND);
            assert!(err.error.message.contains("Missing"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn register_remote_and_discover() {
        let _lock = ENV_LOCK.lock().await;
        // Initialize the remote store for this test (clear any leftovers)
        let _store = super::init_remote_store();
        if let Some(s) = super::remote_store() { s.clear(); }

        // Register a remote session via RPC handler
        let params = json!({
            "display_name": "remote-worker",
            "host": "192.168.1.50",
            "port": 9001,
            "pid": 12345,
            "tags": ["gpu", "worker"],
            "roles": ["compute"],
        });
        let resp = super::handle_register_remote(json!("reg-1"), &params);
        let session_id = if let RpcResponse::Success(r) = &resp {
            r.result["id"].as_str().unwrap().to_string()
        } else {
            panic!("Expected success response from register_remote");
        };
        assert!(session_id.starts_with("tl-tcp-"));

        // Discover should include the remote session
        let resp = super::handle_discover(json!("d-1"), &json!({})).await;
        if let RpcResponse::Success(r) = &resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            let remote = sessions.iter().find(|s| s["id"] == session_id);
            assert!(remote.is_some(), "Remote session should appear in discover");
            let remote = remote.unwrap();
            assert_eq!(remote["display_name"], "remote-worker");
            assert_eq!(remote["addr"]["type"], "tcp");
            assert_eq!(remote["addr"]["host"], "192.168.1.50");
            assert_eq!(remote["addr"]["port"], 9001);
            assert_eq!(remote["remote"], true);
        } else {
            panic!("Expected success response from discover");
        }

        // Discover with tag filter should find it
        let resp = super::handle_discover(json!("d-2"), &json!({"tags": ["gpu"]})).await;
        if let RpcResponse::Success(r) = &resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert!(sessions.iter().any(|s| s["id"] == session_id));
        } else {
            panic!("Expected success");
        }

        // Heartbeat should work
        let resp = super::handle_heartbeat(json!("hb-1"), &json!({"id": session_id}));
        if let RpcResponse::Success(r) = &resp {
            assert_eq!(r.result["ok"], true);
        } else {
            panic!("Expected success from heartbeat");
        }

        // Deregister
        let resp = super::handle_deregister_remote(json!("dr-1"), &json!({"id": session_id}));
        if let RpcResponse::Success(r) = &resp {
            assert_eq!(r.result["ok"], true);
        } else {
            panic!("Expected success from deregister");
        }

        // Should no longer appear in discover
        let resp = super::handle_discover(json!("d-3"), &json!({})).await;
        if let RpcResponse::Success(r) = &resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert!(!sessions.iter().any(|s| s["id"] == session_id));
        } else {
            panic!("Expected success");
        }
    }

    /// Helper: start a hub with Unix + TCP listeners.
    /// Returns (hub_handle, shutdown_tx, hub_socket_path, tcp_port, secret_hex).
    async fn start_hub_with_tcp(
        dir: &Path,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::sync::watch::Sender<bool>,
        PathBuf,
        u16,
        String,
    ) {
        use crate::server::run_accept_loop;
        use tokio::net::{TcpListener, UnixListener};
        use tokio::sync::watch;

        let hub_socket = dir.join("hub.sock");
        let secret = termlink_session::auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        (handle, tx, hub_socket, tcp_port, secret_hex)
    }

    /// Helper: connect to TCP, authenticate, return (lines_reader, writer).
    async fn tcp_connect_and_auth(
        tcp_port: u16,
        secret_hex: &str,
        scope: termlink_session::auth::PermissionScope,
    ) -> (
        tokio::io::Lines<tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>>,
        tokio::net::tcp::OwnedWriteHalf,
    ) {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let secret_vec: Vec<u8> = (0..secret_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&secret_hex[i..i + 2], 16).unwrap())
            .collect();
        let secret_bytes: [u8; 32] = secret_vec.try_into().expect("secret must be 32 bytes");

        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = tokio::io::BufReader::new(reader).lines();

        let token = termlink_session::auth::create_token(&secret_bytes, scope, "", 3600);
        let req = json!({
            "jsonrpc": "2.0",
            "method": "hub.auth",
            "id": "auth",
            "params": { "token": token.raw }
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["result"]["authenticated"], true);

        (lines, writer)
    }


    #[tokio::test]
    async fn tcp_collect_aggregates_events() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "tcp-coll-a").await;
        let (h2, r2) = start_test_session(&sessions_dir, "tcp-coll-b").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Emit events directly to each session
        client::rpc_call(
            r1.socket_path(),
            "event.emit",
            json!({"topic": "build.done", "payload": {"machine": "A"}}),
        )
        .await
        .unwrap();
        client::rpc_call(
            r2.socket_path(),
            "event.emit",
            json!({"topic": "test.pass", "payload": {"machine": "B"}}),
        )
        .await
        .unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, secret_hex) =
            start_hub_with_tcp(&dir).await;

        // Connect via TCP and authenticate
        let (mut lines, mut writer) = tcp_connect_and_auth(
            tcp_port,
            &secret_hex,
            termlink_session::auth::PermissionScope::Execute,
        )
        .await;

        // Collect events via TCP connection
        let req = json!({
            "jsonrpc": "2.0",
            "method": "event.collect",
            "id": "cl-tcp-1",
            "params": {}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "cl-tcp-1");
        assert_eq!(resp["result"]["count"], 2);
        let events = resp["result"]["events"].as_array().unwrap();
        let topics: Vec<&str> = events.iter().filter_map(|e| e["topic"].as_str()).collect();
        assert!(topics.contains(&"build.done"));
        assert!(topics.contains(&"test.pass"));

        // Each event should have session metadata
        for event in events {
            assert!(event.get("session").is_some());
            assert!(event.get("session_name").is_some());
        }

        // Cursors should be present
        let cursors = resp["result"]["cursors"].as_object().unwrap();
        assert_eq!(cursors.len(), 2);

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn tcp_unauthenticated_broadcast_rejected() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, _secret_hex) =
            start_hub_with_tcp(&dir).await;

        // Connect via TCP without authenticating
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};

        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = TokioBufReader::new(reader).lines();

        // Try broadcast — should be rejected
        let req = json!({
            "jsonrpc": "2.0",
            "method": "event.broadcast",
            "id": "bc-noauth",
            "params": {"topic": "test", "payload": {}}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["error"]["code"], -32009, "Broadcast should require auth");

        // Try collect — should also be rejected
        let req = json!({
            "jsonrpc": "2.0",
            "method": "event.collect",
            "id": "cl-noauth",
            "params": {}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["error"]["code"], -32009, "Collect should require auth");
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    #[tokio::test]
    async fn forward_to_remote_session_via_tcp() {
        let _lock = ENV_LOCK.lock().await;
        // Start a real session listening on TCP
        let dir = test_dir();
        // Isolate runtime dir so connect_addr won't find a stale hub.cert.pem
        // from a previous real hub run (T-165 TLS auto-detection).
        // SAFETY: ENV_LOCK ensures single-threaded access to env vars in tests.
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", dir.to_str().unwrap()) };
        let (handle, reg) = start_test_session(&dir, "tcp-target").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Also start a TCP listener that forwards to this session
        // (simulating a remote session reachable via TCP)
        let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let tcp_port = tcp_listener.local_addr().unwrap().port();
        let socket_path = reg.socket_path().to_path_buf();

        // Proxy: accept TCP, forward to Unix session
        let proxy_handle = tokio::spawn(async move {
            loop {
                let (tcp_stream, _) = tcp_listener.accept().await.unwrap();
                let sp = socket_path.clone();
                tokio::spawn(async move {
                    let unix_stream = tokio::net::UnixStream::connect(&sp).await.unwrap();
                    let (mut tcp_r, mut tcp_w) = tokio::io::split(tcp_stream);
                    let (mut unix_r, mut unix_w) = tokio::io::split(unix_stream);
                    tokio::select! {
                        _ = tokio::io::copy(&mut tcp_r, &mut unix_w) => {}
                        _ = tokio::io::copy(&mut unix_r, &mut tcp_w) => {}
                    }
                });
            }
        });

        // Initialize remote store and register the session as remote (clear first)
        let _store = super::init_remote_store();
        let store = super::remote_store().unwrap();
        store.clear();
        let remote_id = store.register(crate::remote_store::RemoteSessionInfo {
            display_name: "tcp-target".into(),
            host: "127.0.0.1".into(),
            port: tcp_port,
            pid: None,
            roles: vec![],
            tags: vec![],
            capabilities: vec![],
            protocol_version: 1,
        });

        // Forward a ping to the remote session via the router
        let req = Request::new(
            "termlink.ping",
            json!("fwd-tcp-1"),
            json!({"target": &remote_id}),
        );
        let resp = super::route(&req, None).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["display_name"], "tcp-target");
            assert_eq!(r.result["state"], "ready");
        } else {
            panic!("Expected success — forward to remote TCP session should work");
        }

        // Also test lookup by display name
        let req = Request::new(
            "termlink.ping",
            json!("fwd-tcp-2"),
            json!({"target": "tcp-target"}),
        );
        let resp = super::route(&req, None).await.unwrap();
        // This might resolve to local or remote — either is fine for this test
        assert!(matches!(resp, RpcResponse::Success(_)));

        proxy_handle.abort();
        handle.abort();
    }

    /// T-923 end-to-end: TCP-bound hub + hub.auth + transparent forwarder
    /// to a local session, proving the cross-host routing path that T-924's
    /// call_session() CLI helper will drive.
    #[tokio::test]
    async fn tcp_forward_to_local_session_after_auth() {
        use tokio::io::AsyncWriteExt;

        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (session_handle, _reg) =
            start_test_session(&sessions_dir, "fwd-tcp-local").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // manager::find_session() reads sessions_dir() which is relative to
        // TERMLINK_RUNTIME_DIR; point it at the test runtime.
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, secret_hex) =
            start_hub_with_tcp(&dir).await;

        let (mut lines, mut writer) = tcp_connect_and_auth(
            tcp_port,
            &secret_hex,
            termlink_session::auth::PermissionScope::Interact,
        )
        .await;

        // Forward termlink.ping through the hub by session display name.
        let req = json!({
            "jsonrpc": "2.0",
            "method": "termlink.ping",
            "id": "fwd-1",
            "params": { "target": "fwd-tcp-local" }
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "fwd-1");
        assert!(
            resp.get("result").is_some(),
            "forwarder should return success, got: {resp}"
        );
        assert_eq!(resp["result"]["display_name"], "fwd-tcp-local");
        assert_eq!(resp["result"]["state"], "ready");

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
        session_handle.abort();
    }

    /// T-923 scope gap check: the hub rejects a forwarded write-scope method
    /// BEFORE reaching forward_to_target when the connection only holds
    /// Observe scope. This proves forwarded calls are not a scope bypass.
    #[tokio::test]
    async fn tcp_forward_rejected_when_scope_insufficient() {
        use tokio::io::AsyncWriteExt;

        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (session_handle, _reg) =
            start_test_session(&sessions_dir, "fwd-scope-target").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, secret_hex) =
            start_hub_with_tcp(&dir).await;

        let (mut lines, mut writer) = tcp_connect_and_auth(
            tcp_port,
            &secret_hex,
            termlink_session::auth::PermissionScope::Observe,
        )
        .await;

        // kv.set requires Interact — connection only has Observe. The hub
        // must deny the call at the scope gate and MUST NOT forward.
        let req = json!({
            "jsonrpc": "2.0",
            "method": "kv.set",
            "id": "scope-1",
            "params": {
                "target": "fwd-scope-target",
                "key": "k",
                "value": "v"
            }
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(
            resp["error"]["code"].as_i64().unwrap_or(0),
            -32010,
            "Observe scope must not reach kv.set forwarder: {resp}"
        );
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("Permission denied"),
            "expected permission-denied message, got: {resp}"
        );

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
        session_handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_discovers_and_forwards() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start a session that will be our "specialist"
        let (handle, _reg) = start_test_session(&sessions_dir, "specialist-a").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route a ping to any session matching the name
        let params = json!({
            "selector": { "name": "specialist" },
            "method": "termlink.ping",
            "params": {},
        });

        let resp = handle_orchestrator_route(json!("orch-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["routed_to"]["display_name"], "specialist-a");
            assert_eq!(r.result["candidates"], 1);
            // The forwarded ping should return the session info
            assert_eq!(r.result["result"]["display_name"], "specialist-a");
        } else {
            panic!("Expected success, got error");
        }

        handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_no_match_returns_error() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "selector": { "name": "nonexistent" },
            "method": "termlink.ping",
            "params": {},
        });

        let resp = handle_orchestrator_route(json!("orch-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Error(e) = resp {
            assert_eq!(e.error.code, control::error_code::SESSION_NOT_FOUND);
            assert!(e.error.message.contains("No sessions match"));
        } else {
            panic!("Expected error for no matching sessions");
        }
    }

    #[tokio::test]
    async fn orchestrator_route_transport_failure_tracked_in_bypass() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start the dead session first (lower created_at, sorted first by list_sessions)
        let (dead_handle, _dead_reg) =
            start_test_session(&sessions_dir, "dead-specialist").await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Start the live session second
        let (live_handle, _live_reg) =
            start_test_session(&sessions_dir, "live-specialist").await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Kill the dead session's listener but leave socket file intact.
        // Socket file + our PID = passes liveness check, but connect will fail.
        dead_handle.abort();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "selector": {},
            "method": "termlink.ping",
            "params": {},
            "timeout_secs": 1,
        });

        let resp = handle_orchestrator_route(json!("orch-transport-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        // Should succeed via the live specialist (failover from dead)
        match &resp {
            RpcResponse::Success(r) => {
                assert_eq!(r.result["routed_to"]["display_name"], "live-specialist");
            }
            RpcResponse::Error(e) => {
                panic!("Expected success via failover, got error: {}", e.error.message);
            }
        }

        // Check bypass registry — infra failures (connection to dead session) should be
        // invisible. Only the success from the live session should be recorded.
        let reg_path = dir.join("bypass-registry.json");
        let bypass_reg = crate::bypass::BypassRegistry::load_from(&reg_path);

        let stats = bypass_reg.candidates.get("termlink.ping");
        assert!(
            stats.is_some(),
            "termlink.ping should be tracked in bypass candidates"
        );
        let stats = stats.unwrap();
        assert_eq!(
            stats.fail_count, 0,
            "Infra failures should NOT count against fail_count, got {}",
            stats.fail_count
        );
        assert_eq!(
            stats.success_count, 1,
            "Should have 1 success from live specialist"
        );

        live_handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_mutating_skips_bypass_tracking() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (handle, _reg) = start_test_session(&sessions_dir, "specialist-mut").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route 6 times with mutating=true (use termlink.ping — test sessions handle it)
        for i in 0..6 {
            let params = json!({
                "selector": { "name": "specialist" },
                "method": "termlink.ping",
                "params": {},
                "mutating": true,
            });
            let resp =
                handle_orchestrator_route(json!(format!("mut-{i}")), &params).await;
            assert!(
                matches!(resp, RpcResponse::Success(_)),
                "Mutating route should succeed"
            );
        }

        // Check bypass registry — should NOT have tracked termlink.ping
        let reg_path = dir.join("bypass-registry.json");
        let bypass_reg = crate::bypass::BypassRegistry::load_from(&reg_path);
        assert!(
            !bypass_reg.candidates.contains_key("termlink.ping"),
            "Mutating command should NOT be tracked in bypass candidates"
        );
        assert!(
            !bypass_reg.entries.contains_key("termlink.ping"),
            "Mutating command should NOT be promoted to bypass"
        );

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_non_mutating_promotes_normally() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (handle, _reg) = start_test_session(&sessions_dir, "specialist-nm").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route 5 times without mutating flag (default = false)
        for i in 0..5 {
            let params = json!({
                "selector": { "name": "specialist" },
                "method": "termlink.ping",
                "params": {},
            });
            let resp =
                handle_orchestrator_route(json!(format!("nm-{i}")), &params).await;
            assert!(matches!(resp, RpcResponse::Success(_)));
        }

        // Should be promoted after 5 successes
        let reg_path = dir.join("bypass-registry.json");
        let bypass_reg = crate::bypass::BypassRegistry::load_from(&reg_path);
        assert!(
            bypass_reg.entries.contains_key("termlink.ping"),
            "Non-mutating command should be promoted to bypass after 5 runs"
        );

        // 6th call should return bypassed=true
        let params = json!({
            "selector": { "name": "specialist" },
            "method": "termlink.ping",
            "params": {},
        });
        let resp = handle_orchestrator_route(json!("nm-bypass"), &params).await;
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["bypassed"], true);
        } else {
            panic!("Expected bypass response");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_task_type_prefers_tagged_specialist() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start a generic specialist (no task-type tag)
        let (h_generic, _) = start_test_session(&sessions_dir, "generic-specialist").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Start a build-specialist (with task-type:build tag)
        let (h_build, _) = start_test_session_with_tags(
            &sessions_dir,
            "build-specialist",
            vec!["task-type:build".into()],
        )
        .await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route with task_type=build — should prefer build-specialist
        let params = json!({
            "selector": {},
            "method": "termlink.ping",
            "params": {},
            "task_type": "build",
        });

        let resp = handle_orchestrator_route(json!("tt-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(
                r.result["routed_to"]["display_name"], "build-specialist",
                "Task-type routing should prefer the tagged specialist"
            );
            assert_eq!(r.result["candidates"], 2);
        } else {
            panic!("Expected success, got error");
        }

        h_generic.abort();
        h_build.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_task_type_falls_back_when_no_match() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start a test-specialist (tagged for test, not audit)
        let (h_test, _) = start_test_session_with_tags(
            &sessions_dir,
            "test-specialist",
            vec!["task-type:test".into()],
        )
        .await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route with task_type=audit — no specialist has that tag, should fall back
        let params = json!({
            "selector": {},
            "method": "termlink.ping",
            "params": {},
            "task_type": "audit",
        });

        let resp = handle_orchestrator_route(json!("tt-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(
                r.result["routed_to"]["display_name"], "test-specialist",
                "Should fall back to available specialist when no task-type match"
            );
            assert_eq!(r.result["candidates"], 1);
        } else {
            panic!("Expected success via fallback, got error");
        }

        h_test.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_no_task_type_backward_compatible() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start a generic and a tagged specialist
        let (h_generic, _) = start_test_session(&sessions_dir, "generic-specialist").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let (h_build, _) = start_test_session_with_tags(
            &sessions_dir,
            "build-specialist",
            vec!["task-type:build".into()],
        )
        .await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route WITHOUT task_type — should succeed with both candidates available
        let params = json!({
            "selector": {},
            "method": "termlink.ping",
            "params": {},
        });

        let resp = handle_orchestrator_route(json!("tt-3"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            // Without task_type, routing succeeds and both candidates are visible
            assert_eq!(r.result["candidates"], 2);
            // Any specialist is fine — the key is routing works without task_type
            let name = r.result["routed_to"]["display_name"].as_str().unwrap();
            assert!(
                name == "generic-specialist" || name == "build-specialist",
                "Expected one of the specialists, got {name}"
            );
        } else {
            panic!("Expected success, got error");
        }

        h_generic.abort();
        h_build.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_missing_method_returns_error() {
        let params = json!({
            "selector": { "name": "anything" },
        });

        let resp = handle_orchestrator_route(json!("orch-3"), &params).await;

        if let RpcResponse::Error(e) = resp {
            assert_eq!(e.error.code, -32602);
            assert!(e.error.message.contains("Missing 'method'"));
        } else {
            panic!("Expected error for missing method");
        }
    }

    // === event.emit_to tests ===

    #[tokio::test]
    async fn emit_to_pushes_event_to_target() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "emit-to-target").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "target": r1.id.as_str(),
            "topic": "task.result",
            "payload": {"status": "done", "output": "42"},
        });

        let resp = handle_event_emit_to(json!("eto-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["topic"], "task.result");
            assert_eq!(r.result["target"], r1.id.as_str());
            assert!(r.result["seq"].as_u64().is_some());
        } else {
            panic!("Expected success response, got: {resp:?}");
        }

        // Verify event landed on target
        let resp = client::rpc_call(r1.socket_path(), "event.poll", json!({})).await.unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], "task.result");
        assert_eq!(events[0]["payload"]["status"], "done");

        h1.abort();
    }

    #[tokio::test]
    async fn emit_to_enriches_with_sender() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "emit-to-sender").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "target": r1.id.as_str(),
            "topic": "negotiate.offer",
            "payload": {"format": "json"},
            "from": "worker-1",
        });

        let resp = handle_event_emit_to(json!("eto-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["from"], "worker-1");
        } else {
            panic!("Expected success response, got: {resp:?}");
        }

        // Verify sender info is in the event payload
        let resp = client::rpc_call(r1.socket_path(), "event.poll", json!({})).await.unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events[0]["payload"]["_from"], "worker-1");
        assert_eq!(events[0]["payload"]["format"], "json");

        h1.abort();
    }

    #[tokio::test]
    async fn emit_to_unknown_target_returns_error() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        std::fs::create_dir_all(dir.join("sessions")).unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "target": "nonexistent-session",
            "topic": "test.ping",
        });

        let resp = handle_event_emit_to(json!("eto-3"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Error(e) = resp {
            assert_eq!(e.error.code, control::error_code::SESSION_NOT_FOUND);
            assert!(e.error.message.contains("nonexistent-session"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn emit_to_missing_params_returns_error() {
        // Missing target
        let params = json!({"topic": "test"});
        let resp = handle_event_emit_to(json!("eto-4a"), &params).await;
        if let RpcResponse::Error(e) = resp {
            assert!(e.error.message.contains("target"));
        } else {
            panic!("Expected error for missing target");
        }

        // Missing topic
        let params = json!({"target": "some-session"});
        let resp = handle_event_emit_to(json!("eto-4b"), &params).await;
        if let RpcResponse::Error(e) = resp {
            assert!(e.error.message.contains("topic"));
        } else {
            panic!("Expected error for missing topic");
        }
    }

    // --- extract_string_array tests ---

    #[test]
    fn extract_string_array_with_strings() {
        let params = json!({"tags": ["alpha", "beta", "gamma"]});
        let result = extract_string_array(&params, "tags");
        assert_eq!(result, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn extract_string_array_missing_key() {
        let params = json!({"other": "value"});
        let result = extract_string_array(&params, "tags");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_string_array_null_value() {
        let params = json!({"tags": null});
        let result = extract_string_array(&params, "tags");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_string_array_non_array() {
        let params = json!({"tags": "single-string"});
        let result = extract_string_array(&params, "tags");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_string_array_mixed_types() {
        // Non-string elements should be filtered out
        let params = json!({"items": ["valid", 42, "also-valid", true, null]});
        let result = extract_string_array(&params, "items");
        assert_eq!(result, vec!["valid", "also-valid"]);
    }

    #[test]
    fn extract_string_array_empty_array() {
        let params = json!({"tags": []});
        let result = extract_string_array(&params, "tags");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_string_array_empty_params() {
        let params = json!({});
        let result = extract_string_array(&params, "anything");
        assert!(result.is_empty());
    }

    // === Inbox RPC Tests (T-1000) ===





    // === remote session lifecycle error-path tests (T-1007) ===

    #[test]
    fn heartbeat_missing_id_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_heartbeat(json!(1), &json!({}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("Missing"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing id"),
        }
    }

    #[test]
    fn heartbeat_nonexistent_session_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_heartbeat(json!(1), &json!({"id": "tl-tcp-nonexistent"}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, control::error_code::SESSION_NOT_FOUND);
            }
            RpcResponse::Success(_) => panic!("Expected error for nonexistent session"),
        }
    }

    #[test]
    fn deregister_remote_missing_id_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_deregister_remote(json!(1), &json!({}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("Missing"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing id"),
        }
    }

    #[test]
    fn deregister_remote_nonexistent_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_deregister_remote(json!(1), &json!({"id": "tl-tcp-ghost"}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, control::error_code::SESSION_NOT_FOUND);
            }
            RpcResponse::Success(_) => panic!("Expected error for nonexistent session"),
        }
    }

    #[test]
    fn register_remote_missing_host_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_register_remote(json!(1), &json!({"port": 9001}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("host"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing host"),
        }
    }

    #[test]
    fn register_remote_missing_port_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_register_remote(json!(1), &json!({"host": "192.168.1.1"}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("port"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing port"),
        }
    }

    #[tokio::test]
    async fn hub_subscribe_returns_events_structure() {
        super::init_aggregator();
        let params = json!({"timeout_ms": 100});
        let resp = super::handle_hub_subscribe(json!(1), &params).await;
        match resp {
            RpcResponse::Success(r) => {
                assert!(r.result["events"].is_array());
                // Aggregator is a process-global singleton; parallel tests may
                // inject during the 100ms window. Verify shape, not exact zero.
                assert!(r.result["count"].is_number());
                assert!(r.result["sessions"].is_number());
            }
            RpcResponse::Error(e) => panic!("Expected success: {}", e.error.message),
        }
    }

    // === inbox.clear RPC tests (T-1005) ===




    // T-1132

    #[test]
    fn hub_version_returns_binary_version_and_protocol_version() {
        let resp = super::handle_hub_version(json!(7));
        match resp {
            RpcResponse::Success(r) => {
                assert_eq!(r.id, json!(7));
                assert_eq!(r.result["hub_version"], env!("CARGO_PKG_VERSION"));
                assert_eq!(
                    r.result["protocol_version"],
                    termlink_protocol::DATA_PLANE_VERSION
                );
                // T-1632: control_plane_version is a separate axis from
                // protocol_version (= DATA_PLANE_VERSION). T-1166 cut bumped
                // CONTROL_PLANE_VERSION 2→3; this assertion locks the wire
                // emit so older clients can distinguish a post-cut hub.
                assert_eq!(
                    r.result["control_plane_version"],
                    termlink_protocol::CONTROL_PLANE_VERSION
                );
            }
            RpcResponse::Error(e) => panic!("Expected success: {}", e.error.message),
        }
    }






    // T-1298: topic-name validation at hub emit boundaries.
    #[test]
    fn validate_topic_name_accepts_real_topics() {
        for t in &[
            "agent.request",
            "build.done",
            "inbox:carol",
            "channel.list",
            "event.broadcast",
            "kv.change",
            "session.exited",
            "deploy.tcp",
            "a",
            "9",
        ] {
            assert!(
                validate_topic_name(t).is_ok(),
                "expected '{t}' to be valid: {:?}",
                validate_topic_name(t)
            );
        }
    }

    #[test]
    fn validate_topic_name_rejects_uppercase() {
        let err = validate_topic_name("Agent.Request").unwrap_err();
        assert!(err.contains("illegal char"), "got: {err}");
        assert!(err.contains("'A'") || err.contains("\"A\""), "should mention offending char, got: {err}");
    }

    #[test]
    fn validate_topic_name_rejects_xml_interpolation() {
        // Real-world example from T-1297 Spike 1.
        let bad = "learning.shared</topic>\n<parameter name=\"from\">email-archive";
        let err = validate_topic_name(bad).unwrap_err();
        assert!(err.contains("Invalid topic name"), "got: {err}");
    }

    #[test]
    fn validate_topic_name_rejects_newline_or_whitespace() {
        assert!(validate_topic_name("foo\nbar").is_err());
        assert!(validate_topic_name("foo bar").is_err());
        assert!(validate_topic_name("foo\tbar").is_err());
    }

    #[test]
    fn validate_topic_name_rejects_too_long() {
        let long = "a".repeat(257);
        let err = validate_topic_name(&long).unwrap_err();
        assert!(err.contains("too long"), "got: {err}");
    }

    #[test]
    fn validate_topic_name_rejects_empty() {
        assert!(validate_topic_name("").is_err());
    }

    /// T-1446: hub.bus_state reports audit_present=true and
    /// runtime_dir_volatile=false for a /var/lib-style runtime_dir
    /// containing a bus/meta.db file.
    #[tokio::test]
    async fn hub_bus_state_reports_durable_for_var_lib_path() {
        let _lock = ENV_LOCK.lock().await;
        // Use a CARGO_MANIFEST_DIR-rooted path so runtime_dir doesn't start
        // with /tmp/ (which would trigger the volatile heuristic and defeat
        // the test).
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-tmp")
            .join(format!("tl-hub-bs-durable-{}", std::process::id()));
        let bus_dir = dir.join("bus");
        std::fs::create_dir_all(&bus_dir).unwrap();
        let meta_db = bus_dir.join("meta.db");
        std::fs::write(&meta_db, b"PLACEHOLDER").unwrap();

        // The handler reads runtime_dir from termlink_session::discovery::runtime_dir().
        // Override via env var for this test.
        let prev = std::env::var("TERMLINK_RUNTIME_DIR").ok();
        // SAFETY: tests serialise on ENV_LOCK so this set_var is exclusive.
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir); }

        let resp = handle_hub_bus_state(json!("test-bs-1"));
        let RpcResponse::Success(r) = resp else {
            panic!("expected success, got {resp:?}");
        };
        assert_eq!(r.result.get("audit_present").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(r.result.get("runtime_dir_volatile").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(r.result.get("meta_db_size_bytes").and_then(|v| v.as_u64()), Some(11));
        assert!(r.result.get("meta_db_mtime_unix").and_then(|v| v.as_u64()).unwrap_or(0) > 0);
        let rd = r.result.get("runtime_dir").and_then(|v| v.as_str()).unwrap();
        assert!(rd.contains("tl-hub-bs-durable"), "runtime_dir was: {rd}");

        // Restore env
        unsafe {
            match prev {
                Some(v) => std::env::set_var("TERMLINK_RUNTIME_DIR", v),
                None => std::env::remove_var("TERMLINK_RUNTIME_DIR"),
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// T-1446: hub.bus_state reports runtime_dir_volatile=true when runtime_dir
    /// starts with /tmp/.
    #[tokio::test]
    async fn hub_bus_state_reports_volatile_for_tmp_path() {
        let _lock = ENV_LOCK.lock().await;
        let dir = PathBuf::from(format!("/tmp/termlink-bs-vol-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let prev = std::env::var("TERMLINK_RUNTIME_DIR").ok();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir); }

        let resp = handle_hub_bus_state(json!("test-bs-2"));
        let RpcResponse::Success(r) = resp else {
            panic!("expected success, got {resp:?}");
        };
        assert_eq!(r.result.get("runtime_dir_volatile").and_then(|v| v.as_bool()), Some(true));
        // No bus/meta.db created in this test → audit_present=false
        assert_eq!(r.result.get("audit_present").and_then(|v| v.as_bool()), Some(false));

        unsafe {
            match prev {
                Some(v) => std::env::set_var("TERMLINK_RUNTIME_DIR", v),
                None => std::env::remove_var("TERMLINK_RUNTIME_DIR"),
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    // T-2110: hub.governor_status exposes the substrate primitive #9
    // (cv_index) counters alongside the existing T-2048 connection/rate
    // and T-2049 dedupe counters. Pure additive — verify all 13 expected
    // fields are present.
    #[tokio::test]
    async fn governor_status_exposes_cv_index_counters() {
        let resp = handle_hub_governor_status(json!("gs-cv"));
        let RpcResponse::Success(r) = resp else {
            panic!("expected success, got {resp:?}");
        };
        // T-2048 fields (6) + T-2139 rate_buckets_evicted_total (1) = 7.
        for field in [
            "connections_active",
            "connections_max",
            "capacity_hits_total",
            "rate_buckets_active",
            "rate_buckets_evicted_total",
            "rate_hits_total",
            "max_rate_per_sec",
        ] {
            assert!(
                r.result.get(field).is_some(),
                "expected T-2048/T-2139 field {field} in hub.governor_status response"
            );
        }
        // T-2049 dedupe fields (3).
        for field in ["dedupe_entries_active", "dedupe_hits_total", "dedupe_ttl_ms"] {
            assert!(
                r.result.get(field).is_some(),
                "expected T-2049 field {field} in hub.governor_status response"
            );
        }
        // T-2110 cv_index fields (4).
        for field in [
            "cv_index_entries_active",
            "cv_index_topics_active",
            "cv_index_overflow_total",
            "cv_index_cap_per_topic",
        ] {
            assert!(
                r.result.get(field).is_some(),
                "expected T-2110 field {field} in hub.governor_status response"
            );
            // All cv_index fields must be u64-representable.
            assert!(
                r.result[field].as_u64().is_some(),
                "expected T-2110 field {field} to be u64"
            );
        }
        // cap_per_topic must be > 0 (clamped to >=1 in CvIndex::new).
        assert!(
            r.result["cv_index_cap_per_topic"].as_u64().unwrap() >= 1,
            "cv_index_cap_per_topic must be >= 1 (CvIndex::new clamps to min 1)"
        );
        // T-2335 webhook fan-out (arc-004) fields (7). `webhook_enabled` is a
        // bool; the rest are u64 counters. In a fresh test process the
        // subsystem is disabled (no TERMLINK_WEBHOOK_CONFIG) so enabled=false
        // and every counter is 0 — but the fields must always be PRESENT so a
        // wrapper can read them without a pre-slice-vs-post-slice probe.
        assert_eq!(
            r.result.get("webhook_enabled").and_then(|v| v.as_bool()),
            Some(false),
            "expected T-2335 webhook_enabled=false in a config-less test process"
        );
        for field in [
            "webhook_target_count",
            "webhook_retry_depth",
            "webhook_enqueued_total",
            "webhook_retry_success_total",
            "webhook_dropped_full_total",
            "webhook_dead_letter_total",
        ] {
            assert!(
                r.result.get(field).and_then(|v| v.as_u64()).is_some(),
                "expected T-2335 field {field} to be a present u64 in hub.governor_status response"
            );
        }
    }
}
