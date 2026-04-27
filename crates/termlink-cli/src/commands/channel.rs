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
    mentions: &[String],
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
            // T-1325: pack mentions into metadata if provided
            let metadata: Vec<String> = if mentions.is_empty() {
                Vec::new()
            } else {
                vec![format!("mentions={}", mentions.join(","))]
            };
            cmd_channel_post(
                &topic,
                "chat",
                Some(msg),
                None,
                None, // sender_id defaults to identity fingerprint
                reply_to,
                &metadata,
                hub,
                json_output,
            )
            .await
        }
        None => {
            // Default read mode: --resume + --reactions (the rich
            // conversation view the agent typically wants).
            cmd_channel_subscribe(
                &topic, 0, true, false, 100, false, None, None, true, false, true, true,
                None, hub, json_output,
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
    unread: bool,
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

    if !unread {
        // T-1320 legacy path — peer column only.
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
        return Ok(());
    }

    // T-1338: inbox view. For each DM, fetch the caller's last receipt
    // (via channel.receipts RPC) and walk the rest of the topic to count
    // content envelopes. Then sort unread-first.
    let mut rows: Vec<DmInboxRow> = Vec::with_capacity(dms.len());
    for (topic, peer) in &dms {
        let row = compute_dm_inbox_row(&sock, topic, peer, &my_id).await?;
        rows.push(row);
    }
    sort_dm_inbox(&mut rows);

    if json_output {
        let arr: Vec<Value> = rows.iter().map(DmInboxRow::to_json).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"my_id": my_id, "dms": arr}))?
        );
        return Ok(());
    }
    if rows.is_empty() {
        let prefix: String = my_id.chars().take(12).collect();
        eprintln!("No DM topics found for identity {prefix}");
        return Ok(());
    }
    for r in &rows {
        let first = match r.first_unread {
            Some(o) => format!("first={o}"),
            None => "first=—".to_string(),
        };
        println!(
            "{}  (peer={})  unread={}  {}",
            r.topic, r.peer, r.unread, first
        );
    }
    Ok(())
}

/// T-1338: per-row of the DM inbox view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DmInboxRow {
    pub topic: String,
    pub peer: String,
    pub unread: u64,
    pub first_unread: Option<u64>,
}

impl DmInboxRow {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "topic": self.topic,
            "peer": self.peer,
            "unread": self.unread,
            "first_unread": self.first_unread,
        })
    }
}

/// T-1338: stable sort that floats unread DMs to the top. Within each
/// (unread > 0) group and within the (unread == 0) group, original order
/// is preserved (Rust's sort_by is stable).
pub(crate) fn sort_dm_inbox(rows: &mut [DmInboxRow]) {
    rows.sort_by(|a, b| {
        let a_has = if a.unread > 0 { 0u8 } else { 1 };
        let b_has = if b.unread > 0 { 0u8 } else { 1 };
        a_has.cmp(&b_has)
    });
}

/// T-1338: walk one DM topic and produce the inbox row for it. Reuses the
/// T-1329 hub-side aggregation when available, falling back to up_to=0
/// if the receipts call fails (then ALL content counts as unread, which
/// is the correct conservative answer).
async fn compute_dm_inbox_row(
    sock: &std::path::Path,
    topic: &str,
    peer: &str,
    my_id: &str,
) -> Result<DmInboxRow> {
    let mut up_to: u64 = 0;
    let server_resp = client::rpc_call(
        sock,
        method::CHANNEL_RECEIPTS,
        json!({"topic": topic}),
    )
    .await
    .context("Hub rpc_call (channel.receipts) failed")?;
    if let termlink_protocol::jsonrpc::RpcResponse::Success(r) = server_resp {
        for entry in r.result["receipts"].as_array().cloned().unwrap_or_default() {
            if entry.get("sender_id").and_then(|v| v.as_str()) == Some(my_id) {
                up_to = entry.get("up_to").and_then(|v| v.as_u64()).unwrap_or(0);
                break;
            }
        }
    }

    let envelopes = walk_topic_full(sock, topic).await?;
    let (count, first) = count_unread(&envelopes, up_to);
    Ok(DmInboxRow {
        topic: topic.to_string(),
        peer: peer.to_string(),
        unread: count,
        first_unread: first,
    })
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

/// T-1337: pure helper — given a slice of envelopes (any order) and a
/// timestamp anchor in milliseconds, return the highest offset whose
/// `ts_unix_ms` (or hub-aliased `ts`) is `>= since`. None when nothing
/// satisfies. Used by `channel ack --since` to anchor receipts to a
/// recent slice of activity.
pub(crate) fn latest_offset_since(msgs: &[Value], since_ms: i64) -> Option<u64> {
    let mut best: Option<u64> = None;
    for m in msgs {
        let ts_opt = m
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| m.get("ts").and_then(|v| v.as_i64()));
        let Some(ts) = ts_opt else { continue };
        if ts < since_ms {
            continue;
        }
        let off = m.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
        match best {
            Some(b) if b >= off => {}
            _ => best = Some(off),
        }
    }
    best
}

/// T-1337: pure helper — return the maximum `ts` (preferring `ts_unix_ms`)
/// across the slice, or None when no envelope carries a timestamp. Used
/// to enrich the "no activity since X" error hint with the topic's actual
/// latest activity.
pub(crate) fn max_ts(msgs: &[Value]) -> Option<i64> {
    let mut best: Option<i64> = None;
    for m in msgs {
        let ts_opt = m
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| m.get("ts").and_then(|v| v.as_i64()));
        if let Some(ts) = ts_opt {
            best = Some(best.map_or(ts, |b| b.max(ts)));
        }
    }
    best
}

/// T-1315/T-1337: post a `msg_type=receipt` envelope. Body is `up_to=<N>`
/// (text for human readability when subscribed without aggregation); the
/// authoritative routing field is `metadata.up_to=<N>`. Resolution of
/// `up_to`:
///   - explicit `--up-to N`: trusted as-is
///   - `--since MS` (T-1337): walks the topic, picks the highest offset
///     whose envelope has `ts >= MS`. Errors with hint when nothing
///     matches (includes the topic's actual latest ts when present).
///   - neither: auto-resolves to the topic's current latest offset
pub(crate) async fn cmd_channel_ack(
    topic: &str,
    up_to: Option<u64>,
    since_ms: Option<i64>,
    sender_id: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let up_to_resolved = match (up_to, since_ms) {
        (Some(n), _) => n,
        (None, Some(since)) => {
            // T-1337: walk the topic and pick the highest offset whose ts
            // satisfies the anchor.
            let sock = hub_socket(hub)?;
            let envelopes = walk_topic_full(&sock, topic).await?;
            match latest_offset_since(&envelopes, since) {
                Some(n) => n,
                None => {
                    let hint = match max_ts(&envelopes) {
                        Some(ts) => format!(
                            " — topic's latest envelope is at ts={ts} (since={since}, gap={} ms)",
                            since.saturating_sub(ts)
                        ),
                        None => String::new(),
                    };
                    anyhow::bail!(
                        "No envelope on '{topic}' has ts >= {since}{hint}",
                    )
                }
            }
        }
        (None, None) => {
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

/// T-1315 (read-side aggregator) → T-1329 (server-side aggregator).
/// Prefers `channel.receipts` RPC (hub aggregates in one walk); falls back
/// to the legacy client-side walker when the hub returns `MethodNotFound`
/// (-32601). Output text/JSON is identical between the two paths.
pub(crate) async fn cmd_channel_receipts(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    use std::collections::HashMap;
    struct Receipt {
        up_to: u64,
        ts: i64,
    }
    let mut latest: HashMap<String, Receipt> = HashMap::new();

    // T-1329 fast path: hub-side aggregation. One RPC, no pagination.
    let server_resp = client::rpc_call(
        &sock,
        method::CHANNEL_RECEIPTS,
        json!({"topic": topic}),
    )
    .await
    .context("Hub rpc_call (channel.receipts) failed")?;
    let mut fall_back_to_walker = false;
    match server_resp {
        termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
            for entry in r.result["receipts"].as_array().cloned().unwrap_or_default() {
                let sender = match entry.get("sender_id").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                let up_to = entry.get("up_to").and_then(|v| v.as_u64()).unwrap_or(0);
                let ts = entry.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
                latest.insert(sender, Receipt { up_to, ts });
            }
        }
        termlink_protocol::jsonrpc::RpcResponse::Error(e) if e.error.code == -32601 => {
            // Old hub — fall back to the legacy client walker below.
            fall_back_to_walker = true;
        }
        termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
            return Err(anyhow!(
                "Hub returned error for channel.receipts: JSON-RPC error {}: {}",
                e.error.code,
                e.error.message
            ));
        }
    }

    if fall_back_to_walker {
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
    remove: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    if !remove {
        return cmd_channel_post(
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
        .await;
    }
    // T-1330: removal path. Walk the topic, find the latest reaction this
    // identity posted with matching parent + payload, and emit a redaction
    // targeting that offset. Identity-aware: an explicit --sender-id wins;
    // otherwise we resolve the local identity fingerprint, mirroring
    // cmd_channel_post.
    let me: String = match sender_id {
        Some(s) => s.to_string(),
        None => {
            let id = load_identity_or_create()
                .context("Loading identity for reaction removal")?;
            id.fingerprint().to_string()
        }
    };
    let parent_str = parent_offset.to_string();
    let sock = hub_socket(hub)?;
    let mut cursor: u64 = 0;
    let limit: u64 = 1000;
    let mut found: Option<u64> = None;
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
        if let Some(off) = find_my_reaction_offset(&msgs, &me, &parent_str, reaction) {
            // Latest-wins: keep updating; later pages may yield a higher offset.
            found = Some(off);
        }
        cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
        if (n as u64) < limit {
            break;
        }
    }
    let target = found.ok_or_else(|| {
        anyhow!(
            "No reaction by '{me}' on parent {parent_offset} matching '{reaction}' \
             found on topic '{topic}'"
        )
    })?;
    cmd_channel_redact(topic, target, Some("reaction-remove"), hub, json_output).await
}

/// T-1331: pure helper — return references to envelopes with
/// `ts >= since`. Bound is inclusive; envelopes lacking a `ts` field
/// (shouldn't happen post-T-1287) are excluded.
pub(crate) fn filter_msgs_since(msgs: &[Value], since: i64) -> Vec<&Value> {
    msgs.iter()
        .filter(|m| {
            m.get("ts")
                .and_then(|v| v.as_i64())
                .map(|t| t >= since)
                .unwrap_or(false)
        })
        .collect()
}

/// T-1330: pure helper — scan a page of envelopes and return the highest
/// offset of a reaction envelope that matches (sender, parent, payload).
/// Returns None when nothing matches. Caller paginates and keeps the
/// highest across all pages.
pub(crate) fn find_my_reaction_offset(
    msgs: &[Value],
    sender: &str,
    parent: &str,
    payload: &str,
) -> Option<u64> {
    let mut best: Option<u64> = None;
    for m in msgs {
        if m.get("msg_type").and_then(|v| v.as_str()) != Some("reaction") {
            continue;
        }
        if m.get("sender_id").and_then(|v| v.as_str()) != Some(sender) {
            continue;
        }
        let parent_val = m
            .get("metadata")
            .and_then(|md| md.get("in_reply_to"))
            .and_then(|v| v.as_str());
        if parent_val != Some(parent) {
            continue;
        }
        let payload_b64 = m.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(payload_b64)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default();
        if decoded != payload {
            continue;
        }
        let offset = m.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
        match best {
            Some(b) if b >= offset => {}
            _ => best = Some(offset),
        }
    }
    best
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

/// T-1321: an edit envelope (`msg_type=edit` carrying `metadata.replaces=<offset>`).
/// `parent` is the original offset being replaced; `text` is the new payload.
struct Edit<'a> {
    parent: u64,
    text: String,
    sender: &'a str,
    ts_ms: u64,
}

fn extract_edit(m: &Value) -> Option<Edit<'_>> {
    if m.get("msg_type").and_then(|v| v.as_str()) != Some("edit") {
        return None;
    }
    let parent = m
        .get("metadata")
        .and_then(|md| md.get("replaces"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())?;
    let sender = m.get("sender_id").and_then(|v| v.as_str()).unwrap_or("?");
    let ts_ms = m.get("ts_ms").and_then(|v| v.as_u64()).unwrap_or(0);
    let payload_b64 = m.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("");
    let text = base64::engine::general_purpose::STANDARD
        .decode(payload_b64)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_default();
    Some(Edit {
        parent,
        text,
        sender,
        ts_ms,
    })
}

/// T-1321: pure helper — given a sequence of (parent_offset, ts_ms, text) edit
/// records, return a map `parent → latest_text` where latest is decided by
/// max ts_ms (ties broken by later position in sequence). The streaming
/// subscribe loop inlines this logic to merge across batches, but the pure
/// version here pins the algorithm under test.
#[cfg(test)]
fn collapse_edits_map(edits: &[(u64, u64, String)]) -> std::collections::HashMap<u64, String> {
    let mut latest: std::collections::HashMap<u64, (u64, String)> = Default::default();
    for (parent, ts, text) in edits {
        latest
            .entry(*parent)
            .and_modify(|(prev_ts, prev_text)| {
                if *ts >= *prev_ts {
                    *prev_ts = *ts;
                    *prev_text = text.clone();
                }
            })
            .or_insert_with(|| (*ts, text.clone()));
    }
    latest.into_iter().map(|(k, (_, t))| (k, t)).collect()
}

/// T-1328: pure helper — pre-order DFS over a parent→children map starting
/// at `root`, returning (offset, depth) pairs. Children are visited in
/// ascending offset order for deterministic output. Stops at `root`'s
/// subtree; unrelated branches in the map are ignored.
pub(crate) fn build_thread(
    parents: &std::collections::HashMap<u64, Vec<u64>>,
    root: u64,
) -> Vec<(u64, usize)> {
    let mut out: Vec<(u64, usize)> = Vec::new();
    fn visit(
        parents: &std::collections::HashMap<u64, Vec<u64>>,
        node: u64,
        depth: usize,
        out: &mut Vec<(u64, usize)>,
    ) {
        out.push((node, depth));
        if let Some(children) = parents.get(&node) {
            let mut sorted: Vec<u64> = children.clone();
            sorted.sort_unstable();
            for child in sorted {
                visit(parents, child, depth + 1, out);
            }
        }
    }
    visit(parents, root, 0, &mut out);
    out
}

/// T-1328: walk a topic, build parent→children map from `metadata.in_reply_to`,
/// DFS-render the subtree rooted at `root`. One-shot read (no --follow).
pub(crate) async fn cmd_channel_thread(
    topic: &str,
    root: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let mut all_msgs: Vec<Value> = Vec::new();
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
        all_msgs.extend(msgs);
        cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
        if (n as u64) < limit {
            break;
        }
    }
    if !all_msgs.iter().any(|m| m["offset"].as_u64() == Some(root)) {
        anyhow::bail!("Topic '{topic}' has no message at offset {root}");
    }
    // Index msgs by offset, build parent→children map from metadata.in_reply_to.
    use std::collections::HashMap;
    let mut by_off: HashMap<u64, Value> = HashMap::with_capacity(all_msgs.len());
    let mut parents: HashMap<u64, Vec<u64>> = HashMap::new();
    for m in &all_msgs {
        let Some(off) = m["offset"].as_u64() else { continue };
        by_off.insert(off, m.clone());
        if let Some(parent_str) = m
            .get("metadata")
            .and_then(|md| md.get("in_reply_to"))
            .and_then(|v| v.as_str())
            && let Ok(parent) = parent_str.parse::<u64>()
        {
            parents.entry(parent).or_default().push(off);
        }
    }
    let order = build_thread(&parents, root);

    if json_output {
        // Flat list with depth for JSON consumers; preserve order.
        let entries: Vec<Value> = order
            .iter()
            .filter_map(|(off, depth)| {
                let m = by_off.get(off)?;
                let payload_b64 = m["payload_b64"].as_str().unwrap_or("");
                let payload = base64::engine::general_purpose::STANDARD
                    .decode(payload_b64)
                    .ok()
                    .and_then(|b| String::from_utf8(b).ok())
                    .unwrap_or_default();
                Some(json!({
                    "offset": off,
                    "depth": depth,
                    "sender_id": m["sender_id"].as_str().unwrap_or("?"),
                    "msg_type": m["msg_type"].as_str().unwrap_or("?"),
                    "payload": payload,
                }))
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "topic": topic,
                "root": root,
                "thread": entries,
            }))?
        );
        return Ok(());
    }

    for (off, depth) in &order {
        let Some(m) = by_off.get(off) else { continue };
        let sender = m["sender_id"].as_str().unwrap_or("?");
        let msg_type = m["msg_type"].as_str().unwrap_or("?");
        let payload_b64 = m["payload_b64"].as_str().unwrap_or("");
        let payload = base64::engine::general_purpose::STANDARD
            .decode(payload_b64)
            .unwrap_or_default();
        let payload_str = String::from_utf8_lossy(&payload);
        let indent = "  ".repeat(*depth);
        println!("{indent}[{off}] {sender} {msg_type}: {payload_str}");
    }
    Ok(())
}

/// T-1325 / T-1333: pure helper — does the comma-separated `mentions` CSV
/// contain the target id?
/// - Strict (comma split + whitespace trim, no substring match).
/// - Empty CSV and empty target both return false.
/// - **Wildcard (T-1333):** `target == "*"` matches any non-empty mention csv
///   (Matrix `@room` analogue — "did this post mention ANYONE?"). A csv that
///   itself contains `*` (e.g. `metadata.mentions=*` or `alice,*`) matches
///   any non-empty target — the post tagged everyone, so any specific
///   subscriber's filter should fire.
pub(crate) fn mentions_match(csv: &str, target: &str) -> bool {
    let target = target.trim();
    if target.is_empty() {
        return false;
    }
    let parts: Vec<&str> = csv.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return false;
    }
    if target == "*" {
        // "Anyone tagged at all?" — any non-empty csv satisfies.
        return true;
    }
    if parts.contains(&"*") {
        // Post mentioned everyone — every specific subscriber matches.
        return true;
    }
    parts.contains(&target)
}

/// T-1325: extract mentions CSV from `metadata.mentions` if present.
fn extract_mentions(m: &Value) -> Option<String> {
    m.get("metadata")
        .and_then(|md| md.get("mentions"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// T-1325: render `[@alice,bob]` style marker truncated to first 3 ids.
fn render_mention_marker(csv: &str) -> String {
    let ids: Vec<&str> = csv.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if ids.is_empty() {
        return String::new();
    }
    let shown: Vec<&str> = ids.iter().take(3).copied().collect();
    let suffix = if ids.len() > 3 {
        format!("+{}", ids.len() - 3)
    } else {
        String::new()
    };
    format!(" @{}{suffix}", shown.join(","))
}

/// T-1324: pure helper — count `chat`-style posts per sender, ignoring
/// metadata envelopes (reaction/edit/redaction/topic_metadata/receipt).
/// Returns (sender_id, post_count) sorted by count descending, then by
/// sender_id ascending for stable ties.
pub(crate) fn summarize_senders(msgs: &[Value]) -> Vec<(String, u64)> {
    use std::collections::HashMap;
    const META: &[&str] = &["reaction", "edit", "redaction", "topic_metadata", "receipt"];
    let mut counts: HashMap<String, u64> = HashMap::new();
    for m in msgs {
        let mt = m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if META.contains(&mt) {
            continue;
        }
        let sender = m
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        *counts.entry(sender).or_insert(0) += 1;
    }
    let mut entries: Vec<(String, u64)> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries
}

/// T-1324: synthesized topic view — description (latest), retention, post
/// count, top senders, latest receipt per sender. One-pass walk over the
/// topic; reuses helpers from T-1315/T-1323.
pub(crate) async fn cmd_channel_info(
    topic: &str,
    since: Option<i64>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    // Pull retention + count from channel.list with the topic name as exact prefix.
    let list_resp = client::rpc_call(&sock, method::CHANNEL_LIST, json!({"prefix": topic}))
        .await
        .context("Hub rpc_call (channel.list) failed")?;
    let list_result = client::unwrap_result(list_resp)
        .map_err(|e| anyhow!("Hub returned error for channel.list: {e}"))?;
    let topics = list_result["topics"].as_array().cloned().unwrap_or_default();
    let entry = topics
        .into_iter()
        .find(|t| t.get("name").and_then(|v| v.as_str()) == Some(topic))
        .ok_or_else(|| anyhow!("Topic '{topic}' not found"))?;
    let count = entry.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    let retention_kind = entry["retention"]["kind"]
        .as_str()
        .unwrap_or("?")
        .to_string();
    let retention_value = entry["retention"]
        .get("value")
        .and_then(|v| v.as_u64());

    // Single full walk to compute description / senders / receipts.
    let mut all_msgs: Vec<Value> = Vec::with_capacity(count as usize);
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
        all_msgs.extend(msgs);
        cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
        if (n as u64) < limit {
            break;
        }
    }
    // T-1331: bound the slice when --since is set. Description / senders /
    // receipts are computed over the slice; total `count` (above) stays
    // unbounded so the operator can see "12 of 23 in last hour".
    let bounded: Vec<Value> = match since {
        Some(s) => filter_msgs_since(&all_msgs, s).into_iter().cloned().collect(),
        None => Vec::new(),
    };
    let view: &[Value] = match since {
        Some(_) => &bounded,
        None => &all_msgs,
    };
    let posts_since = since.map(|_| view.len() as u64);
    let description = latest_description(view).map(|(_, d)| d);
    let senders = summarize_senders(view);

    // Latest receipt per sender (mirror cmd_channel_receipts logic).
    use std::collections::HashMap;
    struct Rcpt {
        up_to: u64,
        ts: i64,
    }
    let mut receipts: HashMap<String, Rcpt> = HashMap::new();
    for m in view {
        if m.get("msg_type").and_then(|v| v.as_str()) != Some("receipt") {
            continue;
        }
        let Some(sender) = m
            .get("sender_id")
            .and_then(|v| v.as_str())
            .map(String::from)
        else {
            continue;
        };
        let Some(up_to) = m
            .get("metadata")
            .and_then(|md| md.get("up_to"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
        else {
            continue;
        };
        let ts = m.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);
        match receipts.get(&sender) {
            Some(prev) if prev.ts > ts => {}
            Some(prev) if prev.ts == ts && prev.up_to >= up_to => {}
            _ => {
                receipts.insert(sender, Rcpt { up_to, ts });
            }
        }
    }

    if json_output {
        let senders_json: Vec<Value> = senders
            .iter()
            .map(|(s, n)| json!({"sender_id": s, "posts": n}))
            .collect();
        let receipts_json: Vec<Value> = {
            let mut entries: Vec<(&String, &Rcpt)> = receipts.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            entries
                .iter()
                .map(|(s, r)| json!({"sender_id": s, "up_to": r.up_to, "ts_unix_ms": r.ts}))
                .collect()
        };
        let mut obj = json!({
            "topic": topic,
            "retention": {
                "kind": retention_kind,
                "value": retention_value,
            },
            "count": count,
            "description": description,
            "senders": senders_json,
            "receipts": receipts_json,
        });
        if let (Some(s), Some(ps), Some(map)) = (since, posts_since, obj.as_object_mut()) {
            map.insert("since".to_string(), json!(s));
            map.insert("posts_since".to_string(), json!(ps));
        }
        println!("{}", serde_json::to_string_pretty(&obj)?);
        return Ok(());
    }

    println!("Topic: {topic}");
    match retention_value {
        Some(v) => println!("Retention: {retention_kind}:{v}"),
        None => println!("Retention: {retention_kind}"),
    }
    match (since, posts_since) {
        (Some(s), Some(ps)) => println!("Posts: {count} ({ps} since {s})"),
        _ => println!("Posts: {count}"),
    }
    println!(
        "Description: {}",
        description.as_deref().unwrap_or("(none)")
    );
    println!("Senders: {}", senders.len());
    for (s, n) in senders.iter().take(5) {
        println!("  {s}  ({n} posts)");
    }
    if !receipts.is_empty() {
        println!("Receipts: {}", receipts.len());
        let mut entries: Vec<(&String, &Rcpt)> = receipts.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (s, r) in entries {
            println!("  {s}  up to {}  (ts={})", r.up_to, r.ts);
        }
    }
    Ok(())
}

/// T-1332: msg_types that DON'T count toward "unread" — purely meta envelopes
/// like reactions, edits, redactions, receipts and topic-metadata. The aim is
/// to mirror what a human would mentally count: "new content I haven't seen."
/// T-1334 also uses this set to find the latest content message for `reply`.
const UNREAD_META_TYPES: &[&str] =
    &["receipt", "reaction", "redaction", "edit", "topic_metadata"];

/// T-1334: pure helper — return the highest offset whose `msg_type` is NOT
/// in `UNREAD_META_TYPES`. Returns None when the slice is empty or contains
/// only meta envelopes. Used by `channel reply` to auto-thread to the
/// topic's most recent content message.
pub(crate) fn latest_content_offset(msgs: &[Value]) -> Option<u64> {
    let mut best: Option<u64> = None;
    for m in msgs {
        let mt = m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if UNREAD_META_TYPES.contains(&mt) {
            continue;
        }
        let off = m.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
        match best {
            Some(b) if b >= off => {}
            _ => best = Some(off),
        }
    }
    best
}

/// T-1334: `channel reply <topic> <text>` — walks the topic, picks the
/// highest-offset content envelope, and posts a reply with
/// `metadata.in_reply_to=<that-offset>`. Errors when the topic has no
/// content to reply to.
pub(crate) async fn cmd_channel_reply(
    topic: &str,
    payload: &str,
    mentions: &[String],
    sender_id: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let mut latest: Option<u64> = None;
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
        if let Some(off) = latest_content_offset(&msgs) {
            // Per-page max; loop's outer cmp keeps the running highest.
            latest = Some(latest.map_or(off, |prev| prev.max(off)));
        }
        cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
        if (n as u64) < limit {
            break;
        }
    }
    let parent = latest.ok_or_else(|| {
        anyhow!("No content message found on topic '{topic}' to reply to")
    })?;
    let metadata: Vec<String> = if mentions.is_empty() {
        Vec::new()
    } else {
        vec![format!("mentions={}", mentions.join(","))]
    };
    cmd_channel_post(
        topic,
        "chat",
        Some(payload),
        None,
        sender_id,
        Some(parent),
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1332: pure helper — given a slice of envelopes (sorted by ascending
/// offset) and the caller's last-acked `up_to`, return (count_unread,
/// first_unread_offset). "Unread" = offset > up_to AND msg_type not in
/// `UNREAD_META_TYPES`.
pub(crate) fn count_unread(msgs: &[Value], up_to: u64) -> (u64, Option<u64>) {
    let mut count: u64 = 0;
    let mut first: Option<u64> = None;
    for m in msgs {
        let off = m.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
        if off <= up_to {
            continue;
        }
        let mt = m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if UNREAD_META_TYPES.contains(&mt) {
            continue;
        }
        if first.is_none() {
            first = Some(off);
        }
        count += 1;
    }
    (count, first)
}

/// T-1332: `channel unread <topic> [--sender <id>]` — what's new for me?
pub(crate) async fn cmd_channel_unread(
    topic: &str,
    sender: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sender_id: String = match sender {
        Some(s) => s.to_string(),
        None => {
            let id = load_identity_or_create()
                .context("Loading identity for unread count")?;
            id.fingerprint().to_string()
        }
    };
    let sock = hub_socket(hub)?;

    // T-1329: prefer hub-side aggregation; fall back gracefully if old hub.
    let mut up_to: u64 = 0;
    let server_resp = client::rpc_call(
        &sock,
        method::CHANNEL_RECEIPTS,
        json!({"topic": topic}),
    )
    .await
    .context("Hub rpc_call (channel.receipts) failed")?;
    if let termlink_protocol::jsonrpc::RpcResponse::Success(r) = server_resp {
        for entry in r.result["receipts"].as_array().cloned().unwrap_or_default() {
            if entry.get("sender_id").and_then(|v| v.as_str()) == Some(sender_id.as_str()) {
                up_to = entry.get("up_to").and_then(|v| v.as_u64()).unwrap_or(0);
                break;
            }
        }
    }
    // (If the hub returned MethodNotFound or any error, we silently treat
    //  the sender as having no receipt — equivalent to up_to=0. The unread
    //  count then defaults to "everything", which is the correct
    //  conservative answer when receipts are unavailable.)

    // Walk topic from up_to+1 onwards, count content envelopes.
    let mut total_count: u64 = 0;
    let mut total_first: Option<u64> = None;
    let mut last_offset: u64 = 0;
    let start_cursor: u64 = up_to.saturating_add(1);
    let mut cursor: u64 = start_cursor;
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
        if let Some(m) = msgs.last() {
            last_offset = m.get("offset").and_then(|v| v.as_u64()).unwrap_or(last_offset);
        }
        // Same comparator the helper uses, but operating on this batch.
        let (c, f) = count_unread(&msgs, up_to);
        total_count += c;
        if total_first.is_none() {
            total_first = f;
        }
        cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
        if (n as u64) < limit {
            break;
        }
    }

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "topic": topic,
                "sender_id": sender_id,
                "up_to": up_to,
                "unread_count": total_count,
                "first_unread": total_first,
                "last_offset": last_offset,
            }))?
        );
        return Ok(());
    }
    if total_count == 0 {
        println!("Topic '{topic}': up to date for {sender_id} (last receipt up_to={up_to})");
    } else {
        let first = total_first.unwrap_or(up_to + 1);
        println!(
            "Topic '{topic}': {total_count} unread for {sender_id} \
             (first new offset {first}, last {last_offset}, last receipt up_to={up_to})"
        );
    }
    Ok(())
}

/// T-1323: emit a `msg_type=topic_metadata` envelope carrying a topic
/// description (`metadata.description=<text>`). Append-only — repeat calls
/// add new records; the reader picks the latest by ts_ms.
pub(crate) async fn cmd_channel_describe(
    topic: &str,
    description: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let metadata = vec![format!("description={description}")];
    cmd_channel_post(
        topic,
        "topic_metadata",
        Some(description),
        None,
        None,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1323: pure helper — given a slice of envelope JSON values, return the
/// most recent (ts_ms, description) from `msg_type=topic_metadata` records.
/// Returns `None` if there are no such records. Consumed by `channel info`
/// (T-1324) to surface the description in the synthesized topic view.
pub(crate) fn latest_description(msgs: &[Value]) -> Option<(u64, String)> {
    msgs.iter()
        .filter(|m| m.get("msg_type").and_then(|v| v.as_str()) == Some("topic_metadata"))
        .filter_map(|m| {
            let ts = m.get("ts_ms").and_then(|v| v.as_u64()).unwrap_or(0);
            let desc = m
                .get("metadata")
                .and_then(|md| md.get("description"))
                .and_then(|v| v.as_str())
                .map(String::from)?;
            Some((ts, desc))
        })
        .max_by_key(|(ts, _)| *ts)
}

/// T-1322: a redaction envelope (`msg_type=redaction` carrying
/// `metadata.redacts=<offset>` and optional `reason`).
struct Redaction<'a> {
    target: u64,
    sender: &'a str,
    reason: Option<String>,
}

fn extract_redaction(m: &Value) -> Option<Redaction<'_>> {
    if m.get("msg_type").and_then(|v| v.as_str()) != Some("redaction") {
        return None;
    }
    let target = m
        .get("metadata")
        .and_then(|md| md.get("redacts"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())?;
    let sender = m.get("sender_id").and_then(|v| v.as_str()).unwrap_or("?");
    let reason = m
        .get("metadata")
        .and_then(|md| md.get("reason"))
        .and_then(|v| v.as_str())
        .map(String::from);
    Some(Redaction {
        target,
        sender,
        reason,
    })
}

/// T-1322: pure helper — given a slice of envelope JSON values, return the
/// set of offsets targeted by `msg_type=redaction` records (the parents
/// being retracted). Used by `--hide-redacted` to suppress them.
fn redacted_offsets(msgs: &[Value]) -> std::collections::HashSet<u64> {
    msgs.iter()
        .filter_map(extract_redaction)
        .map(|r| r.target)
        .collect()
}

/// T-1322: emit a `msg_type=redaction` envelope retracting a previous post.
/// Append-only: hub keeps the original; readers may opt to hide it via
/// `subscribe --hide-redacted`.
pub(crate) async fn cmd_channel_redact(
    topic: &str,
    redacts: u64,
    reason: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let mut metadata = vec![format!("redacts={redacts}")];
    if let Some(r) = reason {
        metadata.push(format!("reason={r}"));
    }
    cmd_channel_post(
        topic,
        "redaction",
        Some(""), // empty payload — the redaction is metadata-only
        None,
        None,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1321: emit a `msg_type=edit` envelope with `metadata.replaces=<offset>`.
/// Append-only: hub keeps the original; reader-side decides whether to render
/// collapsed view. Old peers see two records (original + edit).
pub(crate) async fn cmd_channel_edit(
    topic: &str,
    replaces: u64,
    payload: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let metadata = vec![format!("replaces={replaces}")];
    cmd_channel_post(
        topic,
        "edit",
        Some(payload),
        None,
        None, // sender defaults to identity fingerprint
        None, // reply_to not used (replaces carries the reference)
        &metadata,
        hub,
        json_output,
    )
    .await
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
    collapse_edits: bool,
    hide_redacted: bool,
    filter_mentions: Option<&str>,
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
    // T-1321: when collapse_edits is on, accumulate all edits by parent offset
    // across batches (key = parent_offset, value = (latest_ts_ms, latest_text)).
    let mut edits_by_parent: std::collections::HashMap<u64, (u64, String)> =
        Default::default();
    // T-1322: when hide_redacted is on, accumulate redaction targets across
    // batches so a parent that arrived in batch N can be hidden when its
    // redaction arrives in batch N+1.
    let mut redacted: std::collections::HashSet<u64> = Default::default();
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
        // T-1330: ALWAYS collect redaction targets up-front so the reaction
        // aggregator (and other passes) can skip envelopes whose redaction
        // is in this or a prior batch. Matches Matrix m.annotation removal
        // semantics — a redacted reaction is gone from the aggregate
        // regardless of `--hide-redacted`.
        let batch_redacted = redacted_offsets(&msgs);
        // T-1314: when aggregating, do a first pass to bucket new reactions
        // into the persistent map, then a second pass to print non-reaction
        // lines with their accumulated reaction summary inline (looking up
        // reactions by THIS line's own offset). Reactions accumulated from
        // earlier batches still attach to a parent re-rendered in this batch.
        if aggregate_reactions && !json_output {
            for m in &msgs {
                if let Some(r) = extract_reaction(m) {
                    let off = m.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
                    if redacted.contains(&off) || batch_redacted.contains(&off) {
                        continue; // T-1330: redacted reactions are suppressed
                    }
                    reactions_by_parent
                        .entry(r.parent.to_string())
                        .or_default()
                        .push((r.payload, r.sender.to_string()));
                }
            }
        }
        // T-1321: bucket edits in their own pass-1 pass so the original-message
        // render in pass 2 can substitute the latest version.
        if collapse_edits && !json_output {
            for m in &msgs {
                if let Some(e) = extract_edit(m) {
                    let _ = e.sender; // (held only for symmetry; not rendered yet)
                    edits_by_parent
                        .entry(e.parent)
                        .and_modify(|(prev_ts, prev_text)| {
                            if e.ts_ms >= *prev_ts {
                                *prev_ts = e.ts_ms;
                                prev_text.clone_from(&e.text);
                            }
                        })
                        .or_insert((e.ts_ms, e.text));
                }
            }
        }
        // T-1322: pass-1 collect redaction targets so pass-2 can suppress them
        // when hide_redacted is on. T-1330: ALWAYS carry forward the batch's
        // redacted offsets in the persistent set so a redaction in a later
        // page suppresses a reaction whose envelope was in an earlier page
        // (the next subscribe iteration will rebuild reactions_by_parent
        // with the updated redacted set).
        redacted.extend(&batch_redacted);
        for m in &msgs {
            if json_output {
                println!("{}", serde_json::to_string(m)?);
                continue;
            }
            if aggregate_reactions && extract_reaction(m).is_some() {
                continue; // already bucketed in pass 1
            }
            // T-1321: in collapsed mode, suppress edit envelopes — the parent
            // line will already show the latest version.
            if collapse_edits && extract_edit(m).is_some() {
                continue;
            }
            // T-1322: redaction handling
            //   - hide_redacted=true → suppress redaction envelopes AND their
            //     target parents (if seen in this batch or any prior one).
            //   - hide_redacted=false → render redactions explicitly so the
            //     operator can audit what was retracted (default).
            if hide_redacted && extract_redaction(m).is_some() {
                continue;
            }
            if let Some(r) = extract_redaction(m) {
                // T-1326 (e2e fix): a redaction envelope itself never carries
                // mentions metadata, so when --filter-mentions is on we must
                // suppress the explicit-render branch too. Otherwise the
                // filtered view leaks redaction lines that don't match.
                if let Some(target) = filter_mentions {
                    let csv = extract_mentions(m).unwrap_or_default();
                    if !mentions_match(&csv, target) {
                        continue;
                    }
                }
                let off = m["offset"].as_u64().unwrap_or(0);
                let reason = r
                    .reason
                    .as_deref()
                    .map(|s| format!(" (reason: {s})"))
                    .unwrap_or_default();
                println!(
                    "[{off} redact] {sender} → offset {target}{reason}",
                    sender = r.sender,
                    target = r.target,
                );
                continue;
            }
            let offset = m["offset"].as_u64().unwrap_or(0);
            // T-1322: skip parents that have been redacted (only in hide mode).
            if hide_redacted && redacted.contains(&offset) {
                continue;
            }
            let sender = m["sender_id"].as_str().unwrap_or("?");
            let msg_type = m["msg_type"].as_str().unwrap_or("?");
            let payload_b64 = m["payload_b64"].as_str().unwrap_or("");
            let payload = base64::engine::general_purpose::STANDARD
                .decode(payload_b64)
                .unwrap_or_default();
            let mut payload_str = String::from_utf8_lossy(&payload).into_owned();
            // T-1321: substitute latest edit text if this offset has been edited.
            let mut edited_marker = "";
            if collapse_edits
                && let Some((_ts, latest)) = edits_by_parent.get(&offset)
            {
                payload_str = latest.clone();
                edited_marker = " (edited)";
            }
            // T-1313: visual threading marker — replies prefixed with ↳<parent>
            let reply_marker = m
                .get("metadata")
                .and_then(|md| md.get("in_reply_to"))
                .and_then(|v| v.as_str())
                .map(|p| format!(" ↳{p}"))
                .unwrap_or_default();
            // T-1325: mention marker (`@alice,bob` truncated to first 3) and
            // optional `--filter-mentions <id>` client-side filter.
            let mentions_csv = extract_mentions(m);
            let mention_marker = mentions_csv
                .as_deref()
                .map(render_mention_marker)
                .unwrap_or_default();
            if let Some(target) = filter_mentions {
                let csv = mentions_csv.as_deref().unwrap_or("");
                if !mentions_match(csv, target) {
                    continue;
                }
            }
            // T-1314: reaction envelopes get a compact non-aggregated render
            // (msg_type prefix dropped; the `react` tag in the bracket is the cue).
            if msg_type == "reaction" {
                println!("[{offset}{reply_marker}{mention_marker} react] {sender} {payload_str}");
            } else {
                println!("[{offset}{reply_marker}{mention_marker}] {sender} {msg_type}: {payload_str}{edited_marker}");
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
    stats: bool,
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
    if !stats {
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
        return Ok(());
    }

    // T-1335: --stats. For each topic, walk it once and accumulate the
    // breakdown. Empty topic list short-circuits.
    let topics_raw = result["topics"].as_array().cloned().unwrap_or_default();
    let mut rows: Vec<TopicStats> = Vec::with_capacity(topics_raw.len());
    for t in &topics_raw {
        let name = t["name"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let msgs = walk_topic_full(&sock, &name).await?;
        rows.push(compute_topic_stats(&name, &msgs));
    }
    if json_output {
        let arr: Vec<Value> = rows.iter().map(TopicStats::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
    } else if rows.is_empty() {
        println!("No channels.");
    } else {
        for r in &rows {
            println!("{}", r.render_human());
        }
    }
    Ok(())
}

/// T-1335: walk a single topic to completion via `channel.subscribe` paging.
/// Returns all envelopes as JSON values in offset-ascending order. Bounded by
/// hub-page limit (1000); large topics make multiple round-trips.
async fn walk_topic_full(sock: &std::path::Path, topic: &str) -> Result<Vec<Value>> {
    let mut all: Vec<Value> = Vec::new();
    let mut cursor: u64 = 0;
    let limit: u64 = 1000;
    loop {
        let resp = client::rpc_call(
            sock,
            method::CHANNEL_SUBSCRIBE,
            json!({"topic": topic, "cursor": cursor, "limit": limit}),
        )
        .await
        .context("Hub rpc_call (channel.subscribe) failed")?;
        let result = client::unwrap_result(resp).map_err(|e| {
            anyhow!("Hub returned error for channel.subscribe('{topic}'): {e}")
        })?;
        let msgs = result["messages"].as_array().cloned().unwrap_or_default();
        let n = msgs.len();
        all.extend(msgs);
        cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
        if (n as u64) < limit {
            break;
        }
    }
    Ok(all)
}

/// T-1335: per-topic statistics row. `meta` counts envelopes whose msg_type is
/// in `UNREAD_META_TYPES`; everything else is `content`. Senders are distinct.
/// Timestamps are min/max across the topic; None when the topic is empty or
/// no envelope carries `ts_unix_ms`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TopicStats {
    pub topic: String,
    pub content: u64,
    pub meta: u64,
    pub senders: u64,
    pub first_ts: Option<i64>,
    pub last_ts: Option<i64>,
}

impl TopicStats {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "topic": self.topic,
            "content": self.content,
            "meta": self.meta,
            "senders": self.senders,
            "first_ts": self.first_ts,
            "last_ts": self.last_ts,
        })
    }

    pub(crate) fn render_human(&self) -> String {
        let range = match (self.first_ts, self.last_ts) {
            (Some(a), Some(b)) => format!("{a}..{b}"),
            _ => "—".to_string(),
        };
        format!(
            "{}  content={}  meta={}  senders={}  ts={}",
            self.topic, self.content, self.meta, self.senders, range
        )
    }
}

/// T-1335: pure helper — given a topic name and its envelopes (any order),
/// compute the breakdown without touching the network. Senders are deduped
/// case-sensitively. Envelopes missing `ts_unix_ms` are skipped from the
/// timestamp range but still counted toward content/meta and sender set.
pub(crate) fn compute_topic_stats(topic: &str, msgs: &[Value]) -> TopicStats {
    use std::collections::BTreeSet;
    let mut content: u64 = 0;
    let mut meta: u64 = 0;
    let mut senders: BTreeSet<String> = BTreeSet::new();
    let mut first_ts: Option<i64> = None;
    let mut last_ts: Option<i64> = None;
    for m in msgs {
        let mt = m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if UNREAD_META_TYPES.contains(&mt) {
            meta += 1;
        } else {
            content += 1;
        }
        if let Some(s) = m.get("sender_id").and_then(|v| v.as_str())
            && !s.is_empty()
        {
            senders.insert(s.to_string());
        }
        // Hub serializes the envelope timestamp as `ts`; CLI-side aggregates
        // sometimes call it `ts_unix_ms`. Accept either, prefer `ts_unix_ms`.
        let ts_opt = m
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| m.get("ts").and_then(|v| v.as_i64()));
        if let Some(ts) = ts_opt {
            first_ts = Some(first_ts.map_or(ts, |a| a.min(ts)));
            last_ts = Some(last_ts.map_or(ts, |a| a.max(ts)));
        }
    }
    TopicStats {
        topic: topic.to_string(),
        content,
        meta,
        senders: senders.len() as u64,
        first_ts,
        last_ts,
    }
}

/// T-1336: pure helper — does `text` match `pattern` under the given mode?
/// `regex=true` compiles `pattern` as a Rust regex (with `(?i)` prefix when
/// `case_sensitive=false`). `regex=false` does a substring check (folding
/// both sides to lowercase when `case_sensitive=false`). Returns `Err` only
/// when regex compilation fails — substring mode is infallible.
pub(crate) fn payload_matches(
    text: &str,
    pattern: &str,
    regex: bool,
    case_sensitive: bool,
) -> Result<bool> {
    if regex {
        let effective = if case_sensitive {
            pattern.to_string()
        } else {
            format!("(?i){pattern}")
        };
        let re = ::regex::Regex::new(&effective)
            .map_err(|e| anyhow!("invalid regex pattern '{pattern}': {e}"))?;
        Ok(re.is_match(text))
    } else if case_sensitive {
        Ok(text.contains(pattern))
    } else {
        Ok(text.to_lowercase().contains(&pattern.to_lowercase()))
    }
}

/// T-1336: decode an envelope's base64 payload to a UTF-8 string (lossy on
/// invalid sequences). Returns empty string when `payload_b64` is missing
/// or decode fails — search mode treats both as "no content to match".
fn decode_payload_lossy(env: &Value) -> String {
    let b64 = env.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("");
    if b64.is_empty() {
        return String::new();
    }
    match base64::engine::general_purpose::STANDARD.decode(b64) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => String::new(),
    }
}

/// T-1336: `channel search <topic> <pattern>` — read-only client-side grep.
/// Walks the topic via channel.subscribe, filters envelopes by msg_type
/// (skips meta unless `all`), decodes payload, applies the matcher, and
/// prints/returns matches. Validates the regex BEFORE walking the topic
/// so a typo fails fast.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_channel_search(
    topic: &str,
    pattern: &str,
    regex: bool,
    case_sensitive: bool,
    all: bool,
    limit: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    // Fail-fast: validate the regex once up-front.
    if regex {
        let effective = if case_sensitive {
            pattern.to_string()
        } else {
            format!("(?i){pattern}")
        };
        ::regex::Regex::new(&effective)
            .map_err(|e| anyhow!("invalid regex pattern '{pattern}': {e}"))?;
    }

    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;

    let mut hits: Vec<Value> = Vec::new();
    for env in &envelopes {
        let mt = env
            .get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !all && UNREAD_META_TYPES.contains(&mt) {
            continue;
        }
        let payload = decode_payload_lossy(env);
        if payload.is_empty() {
            continue;
        }
        if !payload_matches(&payload, pattern, regex, case_sensitive)? {
            continue;
        }
        let offset = env.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()));
        hits.push(json!({
            "offset": offset,
            "sender_id": sender,
            "ts": ts,
            "msg_type": mt,
            "payload": payload,
        }));
        if limit > 0 && hits.len() as u64 >= limit {
            break;
        }
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&Value::Array(hits))?);
    } else if hits.is_empty() {
        println!("No matches.");
    } else {
        for h in &hits {
            let off = h["offset"].as_u64().unwrap_or(0);
            let sender = h["sender_id"].as_str().unwrap_or("?");
            let mt = h["msg_type"].as_str().unwrap_or("?");
            let payload = h["payload"].as_str().unwrap_or("");
            println!("[{off}] {sender} ({mt}): {payload}");
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

    #[test]
    fn collapse_edits_picks_latest() {
        // (parent_offset, ts_ms, text)
        let edits = vec![
            (5, 1000, "v1".to_string()),
            (5, 2000, "v2".to_string()),
            (5, 1500, "v1.5".to_string()), // older than v2 → loses
            (7, 500, "other-only".to_string()),
        ];
        let map = collapse_edits_map(&edits);
        assert_eq!(map.get(&5).map(String::as_str), Some("v2"));
        assert_eq!(map.get(&7).map(String::as_str), Some("other-only"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn latest_description_picks_most_recent() {
        let msgs = vec![
            json!({"msg_type": "chat", "ts_ms": 500, "metadata": {}}),
            json!({
                "msg_type": "topic_metadata", "ts_ms": 1000,
                "metadata": {"description": "v1"}
            }),
            json!({
                "msg_type": "topic_metadata", "ts_ms": 2000,
                "metadata": {"description": "v2"}
            }),
            json!({
                "msg_type": "topic_metadata", "ts_ms": 1500,
                "metadata": {"description": "v1.5 (older than v2)"}
            }),
        ];
        let got = latest_description(&msgs);
        assert_eq!(got, Some((2000, "v2".to_string())));
    }

    #[test]
    fn latest_description_returns_none_for_empty_or_no_topic_metadata() {
        assert_eq!(latest_description(&[]), None);
        let only_chat = vec![
            json!({"msg_type": "chat", "ts_ms": 1, "metadata": {}}),
            json!({"msg_type": "reaction", "ts_ms": 2, "metadata": {"in_reply_to": "0"}}),
        ];
        assert_eq!(latest_description(&only_chat), None);
        // topic_metadata missing the description field is ignored
        let malformed = vec![
            json!({"msg_type": "topic_metadata", "ts_ms": 1, "metadata": {}}),
        ];
        assert_eq!(latest_description(&malformed), None);
    }

    #[test]
    fn build_thread_orders_dfs_with_depth() {
        // Tree: 0 → 1, 0 → 2, 1 → 3
        // Pre-order DFS from 0: 0, 1, 3, 2 with depths 0, 1, 2, 1
        let mut parents: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
        parents.insert(0, vec![1, 2]);
        parents.insert(1, vec![3]);
        let got = build_thread(&parents, 0);
        assert_eq!(got, vec![(0, 0), (1, 1), (3, 2), (2, 1)]);
    }

    #[test]
    fn build_thread_handles_disconnected_subtree() {
        // Two separate trees: {0→1} and {5→6}; rooting at 0 should not include 5/6
        let mut parents: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
        parents.insert(0, vec![1]);
        parents.insert(5, vec![6]);
        let got = build_thread(&parents, 0);
        assert_eq!(got, vec![(0, 0), (1, 1)]);
    }

    #[test]
    fn build_thread_returns_just_root_when_no_children() {
        let parents: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
        assert_eq!(build_thread(&parents, 42), vec![(42, 0)]);
    }

    #[test]
    fn mentions_match_csv_lookups() {
        // Hit
        assert!(mentions_match("alice,bob,carol", "bob"));
        // Miss
        assert!(!mentions_match("alice,bob", "carol"));
        // Whitespace tolerated on both sides
        assert!(mentions_match("alice, bob , carol", "bob"));
        assert!(mentions_match("alice,bob", "  bob  "));
        // Empty CSV / empty target
        assert!(!mentions_match("", "bob"));
        assert!(!mentions_match("alice,bob", ""));
        assert!(!mentions_match("alice,bob", "   "));
        // Substring is NOT a match (strict comma split)
        assert!(!mentions_match("alicia,bobby", "alice"));
        assert!(!mentions_match("alicebob", "alice"));
    }

    #[test]
    fn mentions_match_wildcard_target_matches_any_non_empty() {
        // T-1333: target=* means "did this post mention ANYONE?"
        assert!(mentions_match("alice", "*"));
        assert!(mentions_match("alice,bob", "*"));
        // Empty csv → still false (no one was tagged).
        assert!(!mentions_match("", "*"));
        assert!(!mentions_match("   ", "*"));
    }

    #[test]
    fn mentions_match_wildcard_in_csv_matches_any_target() {
        // T-1333: csv=* means "@room" (everyone). Any specific target hits.
        assert!(mentions_match("*", "alice"));
        assert!(mentions_match("alice,*", "carol"));
        // Whitespace tolerated.
        assert!(mentions_match(" * ", "bob"));
    }

    #[test]
    fn summarize_senders_counts_only_content_msgs() {
        let msgs = vec![
            json!({"sender_id": "alice", "msg_type": "chat"}),
            json!({"sender_id": "alice", "msg_type": "chat"}),
            json!({"sender_id": "bob", "msg_type": "chat"}),
            // metadata envelopes — should be excluded
            json!({"sender_id": "alice", "msg_type": "reaction", "metadata": {"in_reply_to": "0"}}),
            json!({"sender_id": "alice", "msg_type": "edit", "metadata": {"replaces": "0"}}),
            json!({"sender_id": "alice", "msg_type": "redaction", "metadata": {"redacts": "0"}}),
            json!({"sender_id": "alice", "msg_type": "receipt", "metadata": {"up_to": "0"}}),
            json!({"sender_id": "alice", "msg_type": "topic_metadata"}),
        ];
        let got = summarize_senders(&msgs);
        // alice: 2 content posts, bob: 1; sorted by count desc.
        assert_eq!(
            got,
            vec![("alice".to_string(), 2), ("bob".to_string(), 1)]
        );
    }

    #[test]
    fn redacted_offsets_collects_targets() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat", "payload_b64": "", "metadata": {}}),
            json!({
                "offset": 1, "msg_type": "redaction", "sender_id": "alice",
                "payload_b64": "", "metadata": {"redacts": "0", "reason": "typo"}
            }),
            json!({"offset": 2, "msg_type": "chat", "payload_b64": "", "metadata": {}}),
            json!({
                "offset": 3, "msg_type": "redaction", "sender_id": "bob",
                "payload_b64": "", "metadata": {"redacts": "2"}
            }),
            // malformed redaction (missing redacts) — should be skipped
            json!({"offset": 4, "msg_type": "redaction", "metadata": {}}),
        ];
        let r = redacted_offsets(&msgs);
        assert!(r.contains(&0));
        assert!(r.contains(&2));
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn collapse_edits_handles_tied_timestamp_by_position() {
        let edits = vec![
            (1, 1000, "first".to_string()),
            (1, 1000, "second".to_string()), // ts tie → later position wins
        ];
        let map = collapse_edits_map(&edits);
        assert_eq!(map.get(&1).map(String::as_str), Some("second"));
    }

    /// T-1330: helper that finds the latest matching reaction-by-me on a parent.
    fn react(offset: u64, sender: &str, parent: &str, payload: &str) -> serde_json::Value {
        let p_b64 = base64::engine::general_purpose::STANDARD.encode(payload.as_bytes());
        json!({
            "offset": offset,
            "msg_type": "reaction",
            "sender_id": sender,
            "payload_b64": p_b64,
            "metadata": {"in_reply_to": parent},
        })
    }

    #[test]
    fn find_my_reaction_offset_picks_latest_match() {
        let msgs = vec![
            react(2, "alice", "0", "👍"),
            react(5, "alice", "0", "👍"),
            react(7, "alice", "0", "👍"),
        ];
        assert_eq!(
            find_my_reaction_offset(&msgs, "alice", "0", "👍"),
            Some(7)
        );
    }

    #[test]
    fn find_my_reaction_offset_returns_none_on_empty() {
        let msgs: Vec<serde_json::Value> = vec![];
        assert_eq!(find_my_reaction_offset(&msgs, "alice", "0", "👍"), None);
    }

    #[test]
    fn find_my_reaction_offset_filters_by_sender() {
        let msgs = vec![react(2, "bob", "0", "👍")];
        assert_eq!(find_my_reaction_offset(&msgs, "alice", "0", "👍"), None);
    }

    #[test]
    fn find_my_reaction_offset_filters_by_parent() {
        let msgs = vec![react(2, "alice", "1", "👍")];
        assert_eq!(find_my_reaction_offset(&msgs, "alice", "0", "👍"), None);
    }

    #[test]
    fn find_my_reaction_offset_filters_by_payload() {
        let msgs = vec![react(2, "alice", "0", "👀")];
        assert_eq!(find_my_reaction_offset(&msgs, "alice", "0", "👍"), None);
    }

    #[test]
    fn latest_content_offset_empty_returns_none() {
        let msgs: Vec<Value> = vec![];
        assert_eq!(latest_content_offset(&msgs), None);
    }

    #[test]
    fn latest_content_offset_only_meta_returns_none() {
        let msgs = vec![
            json!({"offset": 1, "msg_type": "reaction"}),
            json!({"offset": 2, "msg_type": "edit"}),
            json!({"offset": 3, "msg_type": "topic_metadata"}),
        ];
        assert_eq!(latest_content_offset(&msgs), None);
    }

    #[test]
    fn latest_content_offset_picks_highest_content() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat"}),
            json!({"offset": 1, "msg_type": "reaction"}),
            json!({"offset": 2, "msg_type": "chat"}),
            json!({"offset": 3, "msg_type": "edit"}),
            json!({"offset": 4, "msg_type": "chat"}),
            json!({"offset": 5, "msg_type": "receipt"}),
        ];
        assert_eq!(latest_content_offset(&msgs), Some(4));
    }

    #[test]
    fn count_unread_empty_returns_zero() {
        let msgs: Vec<Value> = vec![];
        let (c, f) = count_unread(&msgs, 0);
        assert_eq!(c, 0);
        assert_eq!(f, None);
    }

    #[test]
    fn count_unread_skips_at_or_below_bound() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat"}),
            json!({"offset": 1, "msg_type": "chat"}),
            json!({"offset": 2, "msg_type": "chat"}),
        ];
        let (c, f) = count_unread(&msgs, 1);
        assert_eq!(c, 1);
        assert_eq!(f, Some(2));
    }

    #[test]
    fn count_unread_excludes_meta_envelopes() {
        let msgs = vec![
            json!({"offset": 1, "msg_type": "chat"}),
            json!({"offset": 2, "msg_type": "reaction"}),
            json!({"offset": 3, "msg_type": "edit"}),
            json!({"offset": 4, "msg_type": "redaction"}),
            json!({"offset": 5, "msg_type": "topic_metadata"}),
            json!({"offset": 6, "msg_type": "receipt"}),
            json!({"offset": 7, "msg_type": "chat"}),
        ];
        let (c, f) = count_unread(&msgs, 0);
        assert_eq!(c, 2, "only offsets 1 and 7 are content");
        assert_eq!(f, Some(1));
    }

    #[test]
    fn count_unread_first_is_first_content_above_bound() {
        let msgs = vec![
            json!({"offset": 5, "msg_type": "reaction"}), // skipped (meta)
            json!({"offset": 6, "msg_type": "chat"}),
            json!({"offset": 7, "msg_type": "chat"}),
        ];
        let (c, f) = count_unread(&msgs, 4);
        assert_eq!(c, 2);
        assert_eq!(f, Some(6));
    }

    #[test]
    fn filter_msgs_since_inclusive_bound() {
        let msgs = vec![
            json!({"ts": 99, "msg_type": "chat"}),
            json!({"ts": 100, "msg_type": "chat"}),
            json!({"ts": 101, "msg_type": "chat"}),
        ];
        let out = filter_msgs_since(&msgs, 100);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0]["ts"], 100);
        assert_eq!(out[1]["ts"], 101);
    }

    #[test]
    fn filter_msgs_since_empty_input() {
        let msgs: Vec<Value> = vec![];
        assert!(filter_msgs_since(&msgs, 0).is_empty());
    }

    #[test]
    fn filter_msgs_since_all_before_returns_empty() {
        let msgs = vec![
            json!({"ts": 50, "msg_type": "chat"}),
            json!({"ts": 99, "msg_type": "chat"}),
        ];
        assert!(filter_msgs_since(&msgs, 100).is_empty());
    }

    #[test]
    fn filter_msgs_since_all_after_returns_all() {
        let msgs = vec![
            json!({"ts": 200, "msg_type": "chat"}),
            json!({"ts": 300, "msg_type": "chat"}),
        ];
        let out = filter_msgs_since(&msgs, 100);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn filter_msgs_since_drops_envelopes_without_ts() {
        let msgs = vec![
            json!({"msg_type": "chat"}), // no ts
            json!({"ts": 100, "msg_type": "chat"}),
        ];
        let out = filter_msgs_since(&msgs, 0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["ts"], 100);
    }

    #[test]
    fn find_my_reaction_offset_ignores_non_reaction_envelopes() {
        let msgs = vec![
            json!({
                "offset": 2, "msg_type": "chat", "sender_id": "alice",
                "payload_b64": "", "metadata": {"in_reply_to": "0"}
            }),
        ];
        assert_eq!(find_my_reaction_offset(&msgs, "alice", "0", "👍"), None);
    }

    // ---- T-1335: compute_topic_stats / TopicStats -----------------------

    #[test]
    fn compute_topic_stats_empty_topic_yields_zeros() {
        let s = compute_topic_stats("dm:a:b", &[]);
        assert_eq!(s.content, 0);
        assert_eq!(s.meta, 0);
        assert_eq!(s.senders, 0);
        assert_eq!(s.first_ts, None);
        assert_eq!(s.last_ts, None);
        assert_eq!(s.topic, "dm:a:b");
    }

    #[test]
    fn compute_topic_stats_classifies_content_vs_meta() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat",      "sender_id": "alice", "ts_unix_ms": 100}),
            json!({"offset": 1, "msg_type": "reaction",  "sender_id": "bob",   "ts_unix_ms": 200}),
            json!({"offset": 2, "msg_type": "edit",      "sender_id": "alice", "ts_unix_ms": 300}),
            json!({"offset": 3, "msg_type": "redaction", "sender_id": "bob",   "ts_unix_ms": 400}),
            json!({"offset": 4, "msg_type": "receipt",   "sender_id": "alice", "ts_unix_ms": 500}),
            json!({"offset": 5, "msg_type": "topic_metadata", "sender_id": "alice", "ts_unix_ms": 600}),
            json!({"offset": 6, "msg_type": "note",      "sender_id": "carol", "ts_unix_ms": 700}),
        ];
        let s = compute_topic_stats("t", &msgs);
        // chat + note = 2 content; reaction/edit/redaction/receipt/topic_metadata = 5 meta
        assert_eq!(s.content, 2);
        assert_eq!(s.meta, 5);
        assert_eq!(s.senders, 3); // alice, bob, carol
        assert_eq!(s.first_ts, Some(100));
        assert_eq!(s.last_ts, Some(700));
    }

    #[test]
    fn compute_topic_stats_accepts_hub_ts_field_alias() {
        // Hub serializes timestamp as `ts` (not `ts_unix_ms`). Stats helper
        // must accept both — regression for live-hub smoke during T-1335.
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat", "sender_id": "alice", "ts": 100}),
            json!({"offset": 1, "msg_type": "chat", "sender_id": "bob",   "ts": 300}),
        ];
        let s = compute_topic_stats("t", &msgs);
        assert_eq!(s.first_ts, Some(100));
        assert_eq!(s.last_ts, Some(300));
    }

    #[test]
    fn compute_topic_stats_prefers_ts_unix_ms_over_ts() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat", "sender_id": "alice",
                   "ts": 1, "ts_unix_ms": 100}),
        ];
        let s = compute_topic_stats("t", &msgs);
        assert_eq!(s.first_ts, Some(100));
    }

    #[test]
    fn compute_topic_stats_skips_missing_ts_but_still_counts() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat", "sender_id": "alice"}),  // no ts
            json!({"offset": 1, "msg_type": "chat", "sender_id": "bob",   "ts_unix_ms": 50}),
        ];
        let s = compute_topic_stats("t", &msgs);
        assert_eq!(s.content, 2);
        assert_eq!(s.senders, 2);
        assert_eq!(s.first_ts, Some(50));
        assert_eq!(s.last_ts, Some(50));
    }

    #[test]
    fn compute_topic_stats_dedupes_senders() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat", "sender_id": "alice", "ts_unix_ms": 1}),
            json!({"offset": 1, "msg_type": "chat", "sender_id": "alice", "ts_unix_ms": 2}),
            json!({"offset": 2, "msg_type": "chat", "sender_id": "alice", "ts_unix_ms": 3}),
        ];
        let s = compute_topic_stats("t", &msgs);
        assert_eq!(s.senders, 1);
        assert_eq!(s.content, 3);
    }

    #[test]
    fn compute_topic_stats_unknown_msg_type_counts_as_content() {
        // Strict allow-list semantics: anything not in UNREAD_META_TYPES
        // is content. Future msg types ("status", "presence", whatever)
        // will land in content by default — that's the desired behavior
        // because operators want to see them.
        let msgs = vec![
            json!({"offset": 0, "msg_type": "presence", "sender_id": "alice", "ts_unix_ms": 1}),
            json!({"offset": 1, "msg_type": "",         "sender_id": "alice", "ts_unix_ms": 2}),
        ];
        let s = compute_topic_stats("t", &msgs);
        assert_eq!(s.content, 2);
        assert_eq!(s.meta, 0);
    }

    #[test]
    fn topic_stats_render_human_includes_all_fields() {
        let s = TopicStats {
            topic: "team:eng".to_string(),
            content: 5,
            meta: 12,
            senders: 3,
            first_ts: Some(100),
            last_ts: Some(900),
        };
        let line = s.render_human();
        assert!(line.contains("team:eng"));
        assert!(line.contains("content=5"));
        assert!(line.contains("meta=12"));
        assert!(line.contains("senders=3"));
        assert!(line.contains("100..900"));
    }

    #[test]
    fn topic_stats_render_human_dashes_when_no_ts() {
        let s = TopicStats {
            topic: "t".to_string(),
            content: 0, meta: 0, senders: 0,
            first_ts: None, last_ts: None,
        };
        let line = s.render_human();
        assert!(line.contains("ts=—"), "got: {line}");
    }

    #[test]
    fn topic_stats_to_json_round_trips_fields() {
        let s = TopicStats {
            topic: "t".to_string(),
            content: 7, meta: 3, senders: 2,
            first_ts: Some(10), last_ts: Some(20),
        };
        let v = s.to_json();
        assert_eq!(v["topic"], "t");
        assert_eq!(v["content"], 7);
        assert_eq!(v["meta"], 3);
        assert_eq!(v["senders"], 2);
        assert_eq!(v["first_ts"], 10);
        assert_eq!(v["last_ts"], 20);
    }

    // ---- T-1336: payload_matches ---------------------------------------

    #[test]
    fn payload_matches_substring_default_case_insensitive() {
        // Default mode (regex=false, case_sensitive=false)
        assert!(payload_matches("Hello World", "hello", false, false).unwrap());
        assert!(payload_matches("HELLO", "hello", false, false).unwrap());
        assert!(!payload_matches("foo bar", "baz", false, false).unwrap());
    }

    #[test]
    fn payload_matches_substring_case_sensitive() {
        assert!(payload_matches("Hello", "Hello", false, true).unwrap());
        assert!(!payload_matches("Hello", "hello", false, true).unwrap());
    }

    #[test]
    fn payload_matches_regex_basic() {
        assert!(payload_matches("error: 404", r"error:\s+\d+", true, true).unwrap());
        assert!(!payload_matches("just text", r"error:\s+\d+", true, true).unwrap());
    }

    #[test]
    fn payload_matches_regex_case_insensitive() {
        // case_sensitive=false should auto-prefix `(?i)` for regex mode
        assert!(payload_matches("ERROR 500", r"error", true, false).unwrap());
    }

    #[test]
    fn payload_matches_invalid_regex_errors() {
        assert!(payload_matches("anything", r"(?P<unclosed", true, true).is_err());
    }

    #[test]
    fn payload_matches_empty_pattern_substring_always_true() {
        // Empty substring matches every string — Rust's str::contains semantics.
        // This is acceptable for `channel search` because empty pattern is
        // a UX bug on the caller's end; CLI shouldn't try to second-guess.
        assert!(payload_matches("foo", "", false, false).unwrap());
        assert!(payload_matches("", "", false, false).unwrap());
    }

    #[test]
    fn decode_payload_lossy_handles_missing_field() {
        let env = json!({"offset": 0, "msg_type": "chat"});
        assert_eq!(decode_payload_lossy(&env), "");
    }

    #[test]
    fn decode_payload_lossy_decodes_valid_b64() {
        // "hello" → aGVsbG8=
        let env = json!({"offset": 0, "msg_type": "chat", "payload_b64": "aGVsbG8="});
        assert_eq!(decode_payload_lossy(&env), "hello");
    }

    #[test]
    fn decode_payload_lossy_returns_empty_on_invalid_b64() {
        let env = json!({"offset": 0, "msg_type": "chat", "payload_b64": "not-base64-!!!"});
        assert_eq!(decode_payload_lossy(&env), "");
    }

    // ---- T-1337: latest_offset_since / max_ts --------------------------

    #[test]
    fn latest_offset_since_picks_highest_above_anchor() {
        let msgs = vec![
            json!({"offset": 0, "ts": 100}),
            json!({"offset": 1, "ts": 200}),
            json!({"offset": 2, "ts": 300}),
            json!({"offset": 3, "ts": 400}),
        ];
        assert_eq!(latest_offset_since(&msgs, 200), Some(3));
        assert_eq!(latest_offset_since(&msgs, 350), Some(3));
        assert_eq!(latest_offset_since(&msgs, 500), None);
    }

    #[test]
    fn latest_offset_since_inclusive_at_anchor() {
        let msgs = vec![
            json!({"offset": 0, "ts": 100}),
            json!({"offset": 1, "ts": 200}),
        ];
        // Boundary: exactly equal to anchor must match (>= semantics).
        assert_eq!(latest_offset_since(&msgs, 200), Some(1));
        assert_eq!(latest_offset_since(&msgs, 201), None);
    }

    #[test]
    fn latest_offset_since_skips_envelopes_with_no_ts() {
        let msgs = vec![
            json!({"offset": 0, "ts": 100}),
            json!({"offset": 1}), // no ts
            json!({"offset": 2, "ts": 200}),
        ];
        assert_eq!(latest_offset_since(&msgs, 50), Some(2));
        // At 200, only offset 2 satisfies — offset 1 is unranked.
        assert_eq!(latest_offset_since(&msgs, 200), Some(2));
    }

    #[test]
    fn latest_offset_since_accepts_ts_unix_ms_alias() {
        let msgs = vec![
            json!({"offset": 0, "ts_unix_ms": 100}),
            json!({"offset": 1, "ts_unix_ms": 200}),
        ];
        assert_eq!(latest_offset_since(&msgs, 150), Some(1));
    }

    #[test]
    fn latest_offset_since_empty_returns_none() {
        assert_eq!(latest_offset_since(&[], 100), None);
    }

    // ---- T-1338: sort_dm_inbox -----------------------------------------

    fn row(topic: &str, peer: &str, unread: u64, first: Option<u64>) -> DmInboxRow {
        DmInboxRow {
            topic: topic.to_string(),
            peer: peer.to_string(),
            unread,
            first_unread: first,
        }
    }

    #[test]
    fn sort_dm_inbox_floats_unread_to_top() {
        let mut rows = vec![
            row("dm:a:b", "b", 0, None),
            row("dm:c:d", "d", 3, Some(7)),
            row("dm:e:f", "f", 0, None),
            row("dm:g:h", "h", 1, Some(0)),
        ];
        sort_dm_inbox(&mut rows);
        // Both unread DMs come first (in original relative order),
        // zero-unread DMs come second (in original relative order).
        let topics: Vec<&str> = rows.iter().map(|r| r.topic.as_str()).collect();
        assert_eq!(topics, vec!["dm:c:d", "dm:g:h", "dm:a:b", "dm:e:f"]);
    }

    #[test]
    fn sort_dm_inbox_all_zero_keeps_order() {
        let mut rows = vec![
            row("dm:a:b", "b", 0, None),
            row("dm:c:d", "d", 0, None),
        ];
        sort_dm_inbox(&mut rows);
        assert_eq!(rows[0].topic, "dm:a:b");
        assert_eq!(rows[1].topic, "dm:c:d");
    }

    #[test]
    fn sort_dm_inbox_all_unread_keeps_order() {
        let mut rows = vec![
            row("dm:a:b", "b", 5, Some(1)),
            row("dm:c:d", "d", 2, Some(0)),
        ];
        sort_dm_inbox(&mut rows);
        assert_eq!(rows[0].topic, "dm:a:b");
        assert_eq!(rows[1].topic, "dm:c:d");
    }

    #[test]
    fn dm_inbox_row_to_json_round_trips() {
        let r = row("dm:a:b", "b", 4, Some(5));
        let v = r.to_json();
        assert_eq!(v["topic"], "dm:a:b");
        assert_eq!(v["peer"], "b");
        assert_eq!(v["unread"], 4);
        assert_eq!(v["first_unread"], 5);
    }

    #[test]
    fn max_ts_returns_highest_or_none() {
        let msgs = vec![
            json!({"offset": 0, "ts": 100}),
            json!({"offset": 1, "ts": 50}),
            json!({"offset": 2, "ts": 200}),
        ];
        assert_eq!(max_ts(&msgs), Some(200));
        assert_eq!(max_ts(&[]), None);
        // No-ts envelope only → None
        assert_eq!(max_ts(&[json!({"offset": 0})]), None);
    }
}
