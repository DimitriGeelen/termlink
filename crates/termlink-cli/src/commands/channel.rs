//! CLI glue for the T-1160 channel bus.

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use base64::Engine;
use serde_json::{Value, json};

use termlink_protocol::control::{channel::canonical_sign_bytes, method};
use termlink_session::agent_identity::{Identity, identity_path};
use termlink_session::bus_client::{BusClient, PostOutcome};
use termlink_session::client;
use termlink_session::offline_queue::{PendingPost, default_queue_path};

use super::infrastructure::resolve_hub_paths;

fn identity_base_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("TERMLINK_IDENTITY_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").context("HOME is not set; cannot resolve identity dir")?;
    Ok(PathBuf::from(home).join(".termlink"))
}

pub(crate) fn load_identity_or_create() -> Result<Identity> {
    let base = identity_base_dir()?;
    let path = identity_path(&base);
    if !path.exists() {
        // Auto-create on first use — matches 'termlink identity show' UX of surfacing
        // the missing file, but channel.post *needs* a key to proceed.
        Identity::init(&base, false).map_err(|e| anyhow!("Failed to init identity: {e}"))
    } else {
        Identity::load_or_create(&base).map_err(|e| anyhow!("Failed to load identity: {e}"))
    }
}

fn parse_retention(spec: &str) -> Result<Value> {
    if spec == "forever" {
        return Ok(json!({"kind": "forever"}));
    }
    if let Some(n_str) = spec.strip_prefix("days:") {
        let n: u32 = n_str.parse().context("days:N must be a positive integer")?;
        return Ok(json!({"kind": "days", "value": n}));
    }
    if let Some(n_str) = spec.strip_prefix("messages:") {
        let n: u64 = n_str.parse().context("messages:N must be a positive integer")?;
        return Ok(json!({"kind": "messages", "value": n}));
    }
    anyhow::bail!("retention must be 'forever', 'days:N', or 'messages:N' (got: {spec})");
}

fn hub_socket(hub: Option<&str>) -> Result<PathBuf> {
    if let Some(h) = hub {
        return Ok(PathBuf::from(h));
    }
    let (_, sock) = resolve_hub_paths();
    if !sock.exists() {
        anyhow::bail!(
            "Hub is not running (no socket at {}) — start it with 'termlink hub start'",
            sock.display()
        );
    }
    Ok(sock)
}

/// `channel post` tolerates a missing socket (offline-queue fallback), so
/// resolve the path without asserting it exists. T-1174.
fn hub_socket_soft(hub: Option<&str>) -> PathBuf {
    if let Some(h) = hub {
        return PathBuf::from(h);
    }
    let (_, sock) = resolve_hub_paths();
    sock
}

fn hex_of(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

pub(crate) async fn cmd_channel_create(
    name: &str,
    retention: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let retention_val = parse_retention(retention)?;
    let sock = hub_socket(hub)?;
    let resp = client::rpc_call(
        &sock,
        method::CHANNEL_CREATE,
        json!({"name": name, "retention": retention_val}),
    )
    .await
    .context("Hub rpc_call failed")?;
    let result = client::unwrap_result(resp)
        .map_err(|e| anyhow!("Hub returned error for channel.create: {e}"))?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Created topic '{}' (retention: {})", name, retention);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_channel_post(
    topic: &str,
    msg_type: &str,
    payload: Option<&str>,
    artifact_ref: Option<&str>,
    sender_id: Option<&str>,
    reply_to: Option<u64>,
    metadata_kvs: &[String],
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let payload_bytes = match payload {
        Some(p) => p.as_bytes().to_vec(),
        None => {
            let mut buf = Vec::new();
            use std::io::Read;
            std::io::stdin().read_to_end(&mut buf).context("read stdin")?;
            buf
        }
    };
    let identity = load_identity_or_create()?;
    let ts_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let signed = canonical_sign_bytes(topic, msg_type, &payload_bytes, artifact_ref, ts_unix_ms);
    let sig = identity.sign(&signed);
    let resolved_sender = sender_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| identity.fingerprint().to_string());
    // T-1313: assemble metadata. Order: --metadata K=V parsed first, then
    // --reply-to overlays in_reply_to so the dedicated flag wins. Empty map
    // when neither flag is given keeps wire shape unchanged for legacy callers.
    let mut metadata: std::collections::BTreeMap<String, String> = Default::default();
    for kv in metadata_kvs {
        let (k, v) = kv
            .split_once('=')
            .ok_or_else(|| anyhow!("--metadata expects KEY=VALUE, got: {kv}"))?;
        if k.is_empty() {
            anyhow::bail!("--metadata key must be non-empty (got: {kv})");
        }
        metadata.insert(k.to_string(), v.to_string());
    }
    if let Some(off) = reply_to {
        metadata.insert("in_reply_to".to_string(), off.to_string());
    }
    let pending = PendingPost {
        topic: topic.to_string(),
        msg_type: msg_type.to_string(),
        payload: payload_bytes,
        artifact_ref: artifact_ref.map(|s| s.to_string()),
        ts_unix_ms,
        sender_id: resolved_sender,
        sender_pubkey_hex: identity.public_key_hex().to_string(),
        signature_hex: hex_of(&sig.to_bytes()),
        metadata,
    };
    let sock = hub_socket_soft(hub);
    let queue_path = default_queue_path();
    let (client, _flush_task) = BusClient::connect(sock, &queue_path)
        .context("open bus client / offline queue")?;
    // Opportunistic drain: the CLI is one-shot, so the background flush task
    // never gets a 5 s tick. Drain any backlog *before* posting so queued items
    // keep FIFO order relative to this call. Best-effort; transport failure
    // leaves the queue intact for the next invocation. T-1174.
    if client.queue_size() > 0 {
        let report = client.flush().await;
        if report.sent > 0 && !json_output {
            eprintln!(
                "Drained {} queued post(s) from previous offline period",
                report.sent
            );
        }
    }
    let outcome = client
        .post(pending)
        .await
        .map_err(|e| anyhow!("channel.post failed (and offline queue also failed): {e}"))?;
    match outcome {
        PostOutcome::Delivered { offset } => {
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "delivered": {"offset": offset, "ts": ts_unix_ms}
                    }))?
                );
            } else {
                println!("Posted to {topic} — offset={offset}, ts={ts_unix_ms}");
            }
        }
        PostOutcome::Queued { queue_id } => {
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "queued": {
                            "queue_id": queue_id,
                            "queue_path": queue_path.display().to_string(),
                        }
                    }))?
                );
            } else {
                println!(
                    "Queued to {topic} — queue_id={queue_id} (hub unreachable; will flush on next reconnect)"
                );
            }
        }
    }
    Ok(())
}

/// T-1318: per-(topic, identity_fingerprint) persistent cursor store.
/// JSON map at `~/.termlink/cursors.json` — `{"<topic>::<fingerprint>": <offset>}`.
/// Atomic write via tmp + rename. Missing file = no entries.
mod cursor_store {
    use anyhow::{Context, Result};
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    fn store_path() -> Result<PathBuf> {
        if let Ok(dir) = std::env::var("TERMLINK_IDENTITY_DIR") {
            return Ok(PathBuf::from(dir).join("cursors.json"));
        }
        let home = std::env::var("HOME")
            .context("HOME is not set; cannot resolve cursor store path")?;
        Ok(PathBuf::from(home).join(".termlink").join("cursors.json"))
    }

    fn key(topic: &str, fingerprint: &str) -> String {
        format!("{topic}::{fingerprint}")
    }

    fn load() -> Result<BTreeMap<String, u64>> {
        let path = store_path()?;
        if !path.exists() {
            return Ok(BTreeMap::new());
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read cursors from {}", path.display()))?;
        if raw.trim().is_empty() {
            return Ok(BTreeMap::new());
        }
        let parsed: Value = serde_json::from_str(&raw)
            .with_context(|| format!("parse cursors at {}", path.display()))?;
        let mut out = BTreeMap::new();
        if let Some(obj) = parsed.as_object() {
            for (k, v) in obj {
                if let Some(n) = v.as_u64() {
                    out.insert(k.clone(), n);
                }
            }
        }
        Ok(out)
    }

    fn save(map: &BTreeMap<String, u64>) -> Result<()> {
        let path = store_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create parent dir for {}", path.display()))?;
        }
        let json = serde_json::to_string_pretty(map)?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json)
            .with_context(|| format!("write cursors tmp at {}", tmp.display()))?;
        fs::rename(&tmp, &path)
            .with_context(|| format!("rename cursors tmp → {}", path.display()))?;
        Ok(())
    }

    pub fn get(topic: &str, fingerprint: &str) -> Result<Option<u64>> {
        Ok(load()?.get(&key(topic, fingerprint)).copied())
    }

    pub fn put(topic: &str, fingerprint: &str, cursor: u64) -> Result<()> {
        let mut map = load()?;
        map.insert(key(topic, fingerprint), cursor);
        save(&map)
    }

    pub fn remove(topic: &str, fingerprint: &str) -> Result<()> {
        let mut map = load()?;
        if map.remove(&key(topic, fingerprint)).is_some() {
            save(&map)?;
        }
        Ok(())
    }
}

/// T-1319: derive the canonical DM topic name from `(my_id, peer_id)`.
/// Sorted alphabetically and joined as `dm:<a>:<b>` so both ends agree.
fn dm_topic(my_id: &str, peer: &str) -> String {
    let (a, b) = if my_id <= peer {
        (my_id, peer)
    } else {
        (peer, my_id)
    };
    format!("dm:{a}:{b}")
}

/// T-1319: ensure a topic exists. Idempotent — if create returns
/// "already exists" we treat it as success. Used by `channel dm` so the
/// caller doesn't have to think about whether the topic was set up.
async fn ensure_topic(sock: &std::path::Path, name: &str) -> Result<()> {
    let resp = client::rpc_call(
        sock,
        method::CHANNEL_CREATE,
        json!({"name": name, "retention": {"kind": "forever"}}),
    )
    .await
    .context("Hub rpc_call (channel.create) failed")?;
    match client::unwrap_result(resp) {
        Ok(_) => Ok(()),
        // T-1160 channel.create is idempotent on (name, retention) so
        // re-creating an existing topic shouldn't error. If the hub does
        // return an error here it's a real problem worth surfacing.
        Err(e) => Err(anyhow!("channel.create failed: {e}")),
    }
}

/// T-1319: DM shorthand. Resolves canonical `dm:<a>:<b>` topic from caller
/// identity + peer; in read mode opens with `--resume --reactions`; in
/// `--send` mode posts to the topic; `--topic-only` short-circuits.
pub(crate) async fn cmd_channel_dm(
    peer: &str,
    send: Option<&str>,
    reply_to: Option<u64>,
    topic_only: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let identity = load_identity_or_create()?;
    let my_id = identity.fingerprint().to_string();
    let topic = dm_topic(&my_id, peer);
    if topic_only {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"topic": topic, "my_id": my_id, "peer": peer}))?
            );
        } else {
            println!("{topic}");
        }
        return Ok(());
    }
    // Auto-create the topic on either path (idempotent forever-retention).
    let sock = hub_socket(hub)?;
    ensure_topic(&sock, &topic).await?;
    match send {
        Some(msg) => {
            cmd_channel_post(
                &topic,
                "chat",
                Some(msg),
                None,
                None, // sender_id defaults to identity fingerprint
                reply_to,
                &[],
                hub,
                json_output,
            )
            .await
        }
        None => {
            // Default read mode: --resume + --reactions (the rich
            // conversation view the agent typically wants).
            cmd_channel_subscribe(
                &topic, 0, true, false, 100, false, None, None, true, false, hub, json_output,
            )
            .await
        }
    }
}

/// T-1320: pure filter — given a list of topic names and the caller's
/// identity fingerprint, return only DM topics involving the caller, paired
/// with the *other* fingerprint. A DM topic is `dm:<a>:<b>` where `a` and
/// `b` are sorted; the caller is whichever side equals `my_id`.
fn dm_list_filter(topics: &[String], my_id: &str) -> Vec<(String, String)> {
    topics
        .iter()
        .filter_map(|name| {
            let rest = name.strip_prefix("dm:")?;
            let (a, b) = rest.split_once(':')?;
            if a == my_id {
                Some((name.clone(), b.to_string()))
            } else if b == my_id {
                Some((name.clone(), a.to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// T-1320: discover DM topics for the caller's identity. Queries
/// `channel.list` (no prefix), filters to `dm:<a>:<b>` where one side is
/// the caller, prints `<topic>  (peer=<other-fp>)`. Empty result prints a
/// hint to stderr and exits 0.
pub(crate) async fn cmd_channel_dm_list(
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let identity = load_identity_or_create()?;
    let my_id = identity.fingerprint().to_string();
    let sock = hub_socket(hub)?;
    let resp = client::rpc_call(&sock, method::CHANNEL_LIST, json!({}))
        .await
        .context("Hub rpc_call (channel.list) failed")?;
    let result = client::unwrap_result(resp)
        .map_err(|e| anyhow!("Hub returned error for channel.list: {e}"))?;
    let topics: Vec<String> = result["topics"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let dms = dm_list_filter(&topics, &my_id);
    if json_output {
        let rows: Vec<_> = dms
            .iter()
            .map(|(t, p)| json!({"topic": t, "peer": p}))
            .collect();
        println!("{}", serde_json::to_string_pretty(&json!({"my_id": my_id, "dms": rows}))?);
        return Ok(());
    }
    if dms.is_empty() {
        let prefix: String = my_id.chars().take(12).collect();
        eprintln!("No DM topics found for identity {prefix}");
        return Ok(());
    }
    for (topic, peer) in &dms {
        println!("{topic}  (peer={peer})");
    }
    Ok(())
}

/// T-1315: resolve the topic's current latest offset by querying
/// `channel.list` with the topic's exact name as prefix and reading `count`.
/// Returns `Ok(None)` for an empty topic. Used by `channel ack` when the
/// caller doesn't supply `--up-to`.
async fn resolve_latest_offset(sock: &std::path::Path, topic: &str) -> Result<Option<u64>> {
    let resp = client::rpc_call(
        sock,
        method::CHANNEL_LIST,
        json!({"prefix": topic}),
    )
    .await
    .context("Hub rpc_call (channel.list) failed")?;
    let result = client::unwrap_result(resp)
        .map_err(|e| anyhow!("Hub returned error for channel.list: {e}"))?;
    let topics = result["topics"].as_array().cloned().unwrap_or_default();
    let entry = topics
        .into_iter()
        .find(|t| t.get("name").and_then(|v| v.as_str()) == Some(topic))
        .ok_or_else(|| anyhow!("Topic '{topic}' not found"))?;
    let count = entry.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    Ok(if count == 0 { None } else { Some(count - 1) })
}

/// T-1315: post a `msg_type=receipt` envelope. Body is `up_to=<N>` (text
/// for human readability when subscribed without aggregation); the
/// authoritative routing field is `metadata.up_to=<N>`.
pub(crate) async fn cmd_channel_ack(
    topic: &str,
    up_to: Option<u64>,
    sender_id: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    // Auto-resolve when --up-to omitted. Empty topic → can't ack; surface
    // a friendly error rather than posting up_to=0 which would be a lie.
    let up_to_resolved = match up_to {
        Some(n) => n,
        None => {
            let sock = hub_socket(hub)?;
            match resolve_latest_offset(&sock, topic).await? {
                Some(n) => n,
                None => anyhow::bail!("Topic '{topic}' is empty — nothing to ack"),
            }
        }
    };
    let payload = format!("up_to={up_to_resolved}");
    let metadata = vec![format!("up_to={up_to_resolved}")];
    cmd_channel_post(
        topic,
        "receipt",
        Some(&payload),
        None,
        sender_id,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1315: read-side aggregator. Subscribes from offset 0 (one-shot), filters
/// to `msg_type=receipt`, keeps the most-recent receipt per sender, prints
/// sorted. Cap at 1000 messages per page; for very long-lived topics this may
/// need pagination — kept simple for v1 since most "active conversation"
/// topics have low message counts.
pub(crate) async fn cmd_channel_receipts(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    // Walk the entire topic via repeated subscribe calls. Stops when a page
    // returns fewer messages than the limit (signals end of stream). Keeps
    // only the latest receipt per sender; previous receipts are overwritten.
    use std::collections::HashMap;
    struct Receipt {
        up_to: u64,
        ts: i64,
    }
    let mut latest: HashMap<String, Receipt> = HashMap::new();
    let mut cursor: u64 = 0;
    let limit: u64 = 1000;
    loop {
        let resp = client::rpc_call(
            &sock,
            method::CHANNEL_SUBSCRIBE,
            json!({"topic": topic, "cursor": cursor, "limit": limit}),
        )
        .await
        .context("Hub rpc_call (channel.subscribe) failed")?;
        let result = client::unwrap_result(resp)
            .map_err(|e| anyhow!("Hub returned error for channel.subscribe: {e}"))?;
        let msgs = result["messages"].as_array().cloned().unwrap_or_default();
        let n = msgs.len();
        for m in &msgs {
            if m.get("msg_type").and_then(|v| v.as_str()) != Some("receipt") {
                continue;
            }
            let sender = match m.get("sender_id").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let up_to = m
                .get("metadata")
                .and_then(|md| md.get("up_to"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok());
            let Some(up_to) = up_to else { continue };
            let ts = m.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);
            // Latest-wins by ts; ties broken by higher up_to.
            match latest.get(&sender) {
                Some(prev) if prev.ts > ts => {}
                Some(prev) if prev.ts == ts && prev.up_to >= up_to => {}
                _ => {
                    latest.insert(sender, Receipt { up_to, ts });
                }
            }
        }
        cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
        if (n as u64) < limit {
            break;
        }
    }
    let mut entries: Vec<(String, &Receipt)> =
        latest.iter().map(|(k, v)| (k.clone(), v)).collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    if json_output {
        let arr: Vec<Value> = entries
            .iter()
            .map(|(s, r)| json!({"sender_id": s, "up_to": r.up_to, "ts_unix_ms": r.ts}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"topic": topic, "receipts": arr}))?
        );
    } else if entries.is_empty() {
        println!("No receipts on '{topic}'.");
    } else {
        println!("Receipts on '{topic}':");
        for (s, r) in entries {
            println!("  {s}  up to {}  (ts={})", r.up_to, r.ts);
        }
    }
    Ok(())
}

/// T-1314: post a `msg_type=reaction` envelope pointing at a parent offset.
/// Thin wrapper over `cmd_channel_post` — same path, fixed msg_type, reply_to
/// set to the parent. Payload is the reaction string (typically an emoji or
/// short tag like "ack", "wip", "done").
pub(crate) async fn cmd_channel_react(
    topic: &str,
    parent_offset: u64,
    reaction: &str,
    sender_id: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    cmd_channel_post(
        topic,
        "reaction",
        Some(reaction),
        None,
        sender_id,
        Some(parent_offset),
        &[],
        hub,
        json_output,
    )
    .await
}

/// T-1314 / T-1317: payload-decoded view of a reaction envelope. Per-reactor
/// identity is captured so `--by-sender` can render `👍 by alice, bob` while
/// the default count-form ignores it.
struct Reaction<'a> {
    parent: &'a str,
    sender: &'a str,
    payload: String,
}

fn extract_reaction(m: &Value) -> Option<Reaction<'_>> {
    if m.get("msg_type").and_then(|v| v.as_str()) != Some("reaction") {
        return None;
    }
    let parent = m
        .get("metadata")
        .and_then(|md| md.get("in_reply_to"))
        .and_then(|v| v.as_str())?;
    let sender = m.get("sender_id").and_then(|v| v.as_str()).unwrap_or("?");
    let payload_b64 = m.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("");
    let payload = base64::engine::general_purpose::STANDARD
        .decode(payload_b64)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_default();
    Some(Reaction {
        parent,
        sender,
        payload,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_channel_subscribe(
    topic: &str,
    cursor: u64,
    resume: bool,
    reset: bool,
    limit: u64,
    follow: bool,
    conversation_id: Option<&str>,
    in_reply_to: Option<u64>,
    aggregate_reactions: bool,
    by_sender: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    // T-1318: load identity for cursor key (per-topic, per-identity store).
    // We need the fingerprint regardless of whether --resume/--reset are used,
    // because a successful subscribe writes the latest cursor back ONLY when
    // resume=true (avoid surprise side-effects for callers not opting in).
    let identity_fingerprint = if resume || reset {
        Some(load_identity_or_create()?.fingerprint().to_string())
    } else {
        None
    };
    if reset
        && let Some(ref fp) = identity_fingerprint
    {
        cursor_store::remove(topic, fp)
            .context("clear persisted cursor")?;
    }
    let mut cursor = if resume {
        match identity_fingerprint
            .as_ref()
            .and_then(|fp| cursor_store::get(topic, fp).ok().flatten())
        {
            Some(stored) => stored,
            None => cursor, // no entry → fall through to --cursor value
        }
    } else {
        cursor
    };
    // T-1314 / T-1317: when aggregate_reactions is on, reactions accumulate
    // here (parent_offset → [(emoji, sender_id)]) and surface as a trailing
    // summary on the parent line. Sender is preserved for `--by-sender`.
    let mut reactions_by_parent: std::collections::HashMap<String, Vec<(String, String)>> =
        Default::default();
    let mut printed_parents: std::collections::HashSet<u64> = Default::default();
    loop {
        let mut params = json!({"topic": topic, "cursor": cursor, "limit": limit});
        if let Some(cid) = conversation_id
            && let Some(obj) = params.as_object_mut()
        {
            obj.insert("conversation_id".to_string(), json!(cid));
        }
        if let Some(off) = in_reply_to
            && let Some(obj) = params.as_object_mut()
        {
            // T-1313: hub filter is by string equality on metadata.in_reply_to
            // (consistent with conversation_id filter shape — both are strings).
            obj.insert("in_reply_to".to_string(), json!(off.to_string()));
        }
        let resp = client::rpc_call(&sock, method::CHANNEL_SUBSCRIBE, params)
            .await
            .context("Hub rpc_call failed")?;
        let result = client::unwrap_result(resp)
            .map_err(|e| anyhow!("Hub returned error for channel.subscribe: {e}"))?;
        let msgs = result["messages"].as_array().cloned().unwrap_or_default();
        // T-1314: when aggregating, do a first pass to bucket new reactions
        // into the persistent map, then a second pass to print non-reaction
        // lines with their accumulated reaction summary inline (looking up
        // reactions by THIS line's own offset). Reactions accumulated from
        // earlier batches still attach to a parent re-rendered in this batch.
        if aggregate_reactions && !json_output {
            for m in &msgs {
                if let Some(r) = extract_reaction(m) {
                    reactions_by_parent
                        .entry(r.parent.to_string())
                        .or_default()
                        .push((r.payload, r.sender.to_string()));
                }
            }
        }
        for m in &msgs {
            if json_output {
                println!("{}", serde_json::to_string(m)?);
                continue;
            }
            if aggregate_reactions && extract_reaction(m).is_some() {
                continue; // already bucketed in pass 1
            }
            let offset = m["offset"].as_u64().unwrap_or(0);
            let sender = m["sender_id"].as_str().unwrap_or("?");
            let msg_type = m["msg_type"].as_str().unwrap_or("?");
            let payload_b64 = m["payload_b64"].as_str().unwrap_or("");
            let payload = base64::engine::general_purpose::STANDARD
                .decode(payload_b64)
                .unwrap_or_default();
            let payload_str = String::from_utf8_lossy(&payload);
            // T-1313: visual threading marker — replies prefixed with ↳<parent>
            let reply_marker = m
                .get("metadata")
                .and_then(|md| md.get("in_reply_to"))
                .and_then(|v| v.as_str())
                .map(|p| format!(" ↳{p}"))
                .unwrap_or_default();
            // T-1314: reaction envelopes get a compact non-aggregated render
            // (msg_type prefix dropped; the `react` tag in the bracket is the cue).
            if msg_type == "reaction" {
                println!("[{offset}{reply_marker} react] {sender} {payload_str}");
            } else {
                println!("[{offset}{reply_marker}] {sender} {msg_type}: {payload_str}");
                if aggregate_reactions {
                    let summary = reactions_summary(&reactions_by_parent, offset, by_sender);
                    if !summary.is_empty() {
                        println!("    └─ reactions: {summary}");
                    }
                    printed_parents.insert(offset);
                }
            }
        }
        // Drop reaction entries whose parent has now been printed so the map
        // doesn't grow unbounded on long --follow runs. Reactions for parents
        // we haven't yet seen stay queued — they'll attach when/if the parent
        // arrives in a later batch (e.g. backfill from a cursor jump).
        if aggregate_reactions && !json_output {
            reactions_by_parent.retain(|k, _| {
                k.parse::<u64>()
                    .map(|p| !printed_parents.contains(&p))
                    .unwrap_or(true)
            });
        }
        let next = result["next_cursor"].as_u64().unwrap_or(cursor);
        // T-1318: persist next_cursor whenever --resume was set so the next
        // invocation picks up where this one stopped. Best-effort: if the
        // store write fails, log and continue — losing a cursor entry just
        // means the next --resume re-reads from --cursor (default 0), which
        // is safe degradation.
        if resume
            && let Some(ref fp) = identity_fingerprint
            && let Err(e) = cursor_store::put(topic, fp, next)
        {
            eprintln!("warning: failed to persist cursor: {e}");
        }
        if !follow {
            return Ok(());
        }
        cursor = next;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

/// T-1314 / T-1317: collapse a list of `(emoji, sender)` reactions into a
/// summary string. Default form is count-grouped (`👍 ×3, 👀 ×1`); with
/// `by_sender=true` it switches to identity form (`👍 by alice, bob, carol`).
/// Both forms preserve first-seen order of emojis for deterministic output.
fn reactions_summary(
    by_parent: &std::collections::HashMap<String, Vec<(String, String)>>,
    parent: u64,
    by_sender: bool,
) -> String {
    let Some(list) = by_parent.get(&parent.to_string()) else {
        return String::new();
    };
    let mut order: Vec<String> = Vec::new();
    let mut by_emoji: std::collections::HashMap<String, Vec<String>> = Default::default();
    for (emoji, sender) in list {
        if !by_emoji.contains_key(emoji) {
            order.push(emoji.clone());
        }
        by_emoji.entry(emoji.clone()).or_default().push(sender.clone());
    }
    order
        .into_iter()
        .map(|k| {
            let senders = &by_emoji[&k];
            if by_sender {
                // De-dup senders within this emoji bucket so a sender who
                // accidentally double-reacted with the same emoji shows once.
                let mut seen = std::collections::HashSet::new();
                let unique: Vec<String> = senders
                    .iter()
                    .filter(|s| seen.insert(s.as_str().to_string()))
                    .cloned()
                    .collect();
                format!("{k} by {}", unique.join(", "))
            } else if senders.len() == 1 {
                k
            } else {
                format!("{k} ×{}", senders.len())
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// T-1172: Read-only view of the local offline queue (T-1161).
///
/// No hub contact — opens the SQLite file at `queue_path` (or default
/// `~/.termlink/outbound.sqlite`) and reports pending count + head
/// metadata. Safe to run while a live `BusClient` owns the queue
/// because rusqlite handles the WAL-mode concurrency.
pub(crate) fn cmd_channel_queue_status(queue_path: Option<&str>, json_output: bool) -> Result<()> {
    use termlink_session::offline_queue::{default_queue_path, OfflineQueue};

    let path = match queue_path {
        Some(p) => PathBuf::from(p),
        None => default_queue_path(),
    };

    if !path.exists() {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "queue_path": path.display().to_string(),
                    "exists": false,
                    "pending": 0,
                }))?
            );
        } else {
            println!("pending: 0 (queue file not created yet: {})", path.display());
        }
        return Ok(());
    }

    let queue = OfflineQueue::open(&path)
        .with_context(|| format!("Failed to open offline queue at {}", path.display()))?;
    let size = queue.size().context("Failed to read queue size")?;
    let head = queue.peek_oldest().context("Failed to peek queue head")?;

    if json_output {
        let head_json = head.as_ref().map(|(id, post)| {
            json!({
                "queue_id": id.0,
                "topic": post.topic,
                "msg_type": post.msg_type,
                "ts_unix_ms": post.ts_unix_ms,
                "sender_id": post.sender_id,
                "artifact_ref": post.artifact_ref,
            })
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "queue_path": path.display().to_string(),
                "exists": true,
                "cap": queue.cap(),
                "pending": size,
                "oldest": head_json,
            }))?
        );
    } else {
        println!("queue:    {}", path.display());
        println!("cap:      {} (env TERMLINK_OUTBOUND_CAP overrides)", queue.cap());
        println!("pending:  {size}");
        if let Some((id, post)) = head {
            println!(
                "oldest:   id={} topic={} msg_type={} ts_ms={} sender={}",
                id.0, post.topic, post.msg_type, post.ts_unix_ms, post.sender_id
            );
        }
    }
    Ok(())
}

pub(crate) async fn cmd_channel_list(
    prefix: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let params = match prefix {
        Some(p) => json!({"prefix": p}),
        None => json!({}),
    };
    let resp = client::rpc_call(&sock, method::CHANNEL_LIST, params)
        .await
        .context("Hub rpc_call failed")?;
    let result = client::unwrap_result(resp)
        .map_err(|e| anyhow!("Hub returned error for channel.list: {e}"))?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let topics = result["topics"].as_array().cloned().unwrap_or_default();
        if topics.is_empty() {
            println!("No channels.");
        } else {
            for t in &topics {
                let name = t["name"].as_str().unwrap_or("?");
                let kind = t["retention"]["kind"].as_str().unwrap_or("?");
                let value = t["retention"].get("value");
                match value {
                    Some(v) => println!("  {name}  [{kind}:{v}]"),
                    None => println!("  {name}  [{kind}]"),
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dm_list_filters_to_caller_identity() {
        let me = "abc123";
        let topics = vec![
            "dm:abc123:def456".to_string(),    // me on left
            "dm:000aaa:abc123".to_string(),    // me on right
            "dm:def456:000aaa".to_string(),    // not me
            "team:engineering".to_string(),    // not a DM
            "dm:abc123:abc123".to_string(),    // self-DM (degenerate but valid)
            "dm:malformed".to_string(),        // missing second colon — skip
        ];
        let result = dm_list_filter(&topics, me);
        // Expect: 3 hits (abc123:def456 → peer=def456,
        //                  000aaa:abc123 → peer=000aaa,
        //                  abc123:abc123 → peer=abc123 [self-DM])
        let topic_names: Vec<&str> = result.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(
            topic_names,
            vec![
                "dm:abc123:def456",
                "dm:000aaa:abc123",
                "dm:abc123:abc123",
            ],
        );
        let peers: Vec<&str> = result.iter().map(|(_, p)| p.as_str()).collect();
        assert_eq!(peers, vec!["def456", "000aaa", "abc123"]);
    }

    #[test]
    fn dm_list_filter_returns_empty_when_no_match() {
        let topics = vec!["dm:x:y".to_string(), "team:foo".to_string()];
        assert!(dm_list_filter(&topics, "z").is_empty());
    }
}
