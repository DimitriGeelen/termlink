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

    /// T-1358: enumerate every cursor scoped to one identity. Returns
    /// `(topic, cursor)` rows. The store key is `<topic>::<fingerprint>` so
    /// we suffix-match on `::<fingerprint>` and strip it.
    pub fn list_for_fingerprint(fingerprint: &str) -> Result<Vec<(String, u64)>> {
        let map = load()?;
        let suffix = format!("::{fingerprint}");
        let mut out: Vec<(String, u64)> = map
            .into_iter()
            .filter_map(|(k, v)| {
                k.strip_suffix(&suffix).map(|t| (t.to_string(), v))
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(out)
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
                None, None, None, false, None, None, false, hub, json_output,
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

/// T-1341: per-sender membership row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MemberRow {
    pub sender_id: String,
    pub posts: u64,
    pub first_ts: Option<i64>,
    pub last_ts: Option<i64>,
}

impl MemberRow {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "sender_id": self.sender_id,
            "posts": self.posts,
            "first_ts": self.first_ts,
            "last_ts": self.last_ts,
        })
    }
}

/// T-1341: pure helper — group envelopes by sender_id and accumulate
/// post-count + first/last ts. Returns a vec sorted by last_ts desc
/// (stable: earlier sender_id wins ties). Skips meta envelopes unless
/// `include_meta` is true. Empty sender_ids are skipped (defensive).
pub(crate) fn summarize_members(msgs: &[Value], include_meta: bool) -> Vec<MemberRow> {
    use std::collections::BTreeMap;
    let mut acc: BTreeMap<String, MemberRow> = BTreeMap::new();
    for m in msgs {
        let mt = m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if !include_meta && UNREAD_META_TYPES.contains(&mt) {
            continue;
        }
        let sender = match m.get("sender_id").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        let ts_opt = m
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| m.get("ts").and_then(|v| v.as_i64()));
        let entry = acc.entry(sender.clone()).or_insert_with(|| MemberRow {
            sender_id: sender,
            posts: 0,
            first_ts: None,
            last_ts: None,
        });
        entry.posts += 1;
        if let Some(ts) = ts_opt {
            entry.first_ts = Some(entry.first_ts.map_or(ts, |a| a.min(ts)));
            entry.last_ts = Some(entry.last_ts.map_or(ts, |a| a.max(ts)));
        }
    }
    let mut rows: Vec<MemberRow> = acc.into_values().collect();
    // BTreeMap → values are already sorted by sender_id (stable for ties).
    // Sort by last_ts desc; None last_ts sorts last.
    rows.sort_by(|a, b| match (a.last_ts, b.last_ts) {
        (Some(av), Some(bv)) => bv.cmp(&av), // larger b → b first → desc
        (Some(_), None) => std::cmp::Ordering::Less, // a has ts, b doesn't → a first
        (None, Some(_)) => std::cmp::Ordering::Greater, // a no ts → a last
        (None, None) => std::cmp::Ordering::Equal,
    });
    rows
}

/// T-1380: pure helper — same as `summarize_members` but pre-filters
/// envelopes by `ts <= as_of_ms`. Envelopes missing a timestamp are
/// treated as ts=0 (always included when as_of >= 0). When `as_of_ms`
/// is None, behaviour is identical to `summarize_members`.
pub(crate) fn summarize_members_as_of(
    msgs: &[Value],
    include_meta: bool,
    as_of_ms: Option<i64>,
) -> Vec<MemberRow> {
    let Some(cutoff) = as_of_ms else {
        return summarize_members(msgs, include_meta);
    };
    let filtered: Vec<Value> = msgs
        .iter()
        .filter(|env| {
            let ts = env
                .get("ts_unix_ms")
                .and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ts <= cutoff
        })
        .cloned()
        .collect();
    summarize_members(&filtered, include_meta)
}

/// T-1341: `channel members <topic>` — per-sender activity summary.
pub(crate) async fn cmd_channel_members(
    topic: &str,
    include_meta: bool,
    as_of_ms: Option<i64>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = summarize_members_as_of(&envelopes, include_meta, as_of_ms);

    if json_output {
        let arr: Vec<Value> = rows.iter().map(MemberRow::to_json).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "topic": topic,
                "include_meta": include_meta,
                "as_of_ms": as_of_ms,
                "members": arr,
            }))?
        );
        return Ok(());
    }
    if rows.is_empty() {
        match as_of_ms {
            Some(ts) => println!("No members on '{topic}' as of ts={ts}."),
            None => println!("No members on '{topic}'."),
        }
        return Ok(());
    }
    if let Some(ts) = as_of_ms {
        println!("Members of '{topic}' as of ts={ts}:");
    }
    for r in &rows {
        let first = r.first_ts.map_or("—".to_string(), |v| v.to_string());
        let last = r.last_ts.map_or("—".to_string(), |v| v.to_string());
        println!("{}  posts={}  first={}  last={}", r.sender_id, r.posts, first, last);
    }
    Ok(())
}

/// T-1340: pure helper — given an offset→envelope index and a leaf offset,
/// walk `metadata.in_reply_to` upward and return the chain in root→leaf
/// order. Caps recursion at 1024 (cycle defense). Returns an empty vec
/// when the leaf isn't found in the index. The leaf itself is included
/// as the last element. Edges with non-numeric in_reply_to are treated
/// as "no parent" (terminate the walk).
pub(crate) fn build_ancestors(
    by_off: &std::collections::HashMap<u64, Value>,
    leaf: u64,
) -> Vec<u64> {
    const MAX_DEPTH: usize = 1024;
    let mut chain: Vec<u64> = Vec::new();
    let mut visited: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut current = leaf;
    if !by_off.contains_key(&current) {
        return chain;
    }
    for _ in 0..MAX_DEPTH {
        if !visited.insert(current) {
            // Cycle — stop without emitting current again.
            break;
        }
        chain.push(current);
        let Some(env) = by_off.get(&current) else { break };
        let parent = env
            .get("metadata")
            .and_then(|md| md.get("in_reply_to"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok());
        let Some(p) = parent else { break };
        if !by_off.contains_key(&p) {
            break;
        }
        current = p;
    }
    chain.reverse(); // emit root → leaf
    chain
}

/// T-1340: `channel ancestors <topic> <offset>` — root→leaf reply chain.
pub(crate) async fn cmd_channel_ancestors(
    topic: &str,
    offset: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;

    use std::collections::HashMap;
    let mut by_off: HashMap<u64, Value> = HashMap::with_capacity(envelopes.len());
    for env in &envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env.clone());
        }
    }
    if !by_off.contains_key(&offset) {
        anyhow::bail!("Topic '{topic}' has no envelope at offset {offset}");
    }
    let chain = build_ancestors(&by_off, offset);

    if json_output {
        let entries: Vec<Value> = chain
            .iter()
            .filter_map(|off| {
                let m = by_off.get(off)?;
                let payload = decode_payload_lossy(m);
                let ts = m
                    .get("ts_unix_ms")
                    .and_then(|v| v.as_i64())
                    .or_else(|| m.get("ts").and_then(|v| v.as_i64()));
                Some(json!({
                    "offset": off,
                    "sender_id": m.get("sender_id").and_then(|v| v.as_str()).unwrap_or("?"),
                    "msg_type": m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("?"),
                    "ts": ts,
                    "payload": payload,
                }))
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "topic": topic,
                "leaf": offset,
                "ancestors": entries,
            }))?
        );
        return Ok(());
    }

    if chain.is_empty() {
        println!("No ancestors for offset {offset} on topic '{topic}'.");
        return Ok(());
    }
    for (depth, off) in chain.iter().enumerate() {
        let Some(m) = by_off.get(off) else { continue };
        let sender = m.get("sender_id").and_then(|v| v.as_str()).unwrap_or("?");
        let msg_type = m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("?");
        let payload = decode_payload_lossy(m);
        let indent = "  ".repeat(depth);
        println!("{indent}[{off}] {sender} {msg_type}: {payload}");
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

/// T-1351: emit a typing indicator. Posts a `msg_type=typing` envelope
/// carrying `metadata.expires_at_ms=now+ttl_ms`. Append-only — old typing
/// envelopes coexist; the list path filters by expiry.
pub(crate) async fn cmd_channel_typing_emit(
    topic: &str,
    ttl_ms: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let expires_at = now_ms + (ttl_ms as i64);
    let metadata = vec![format!("expires_at_ms={expires_at}")];
    cmd_channel_post(
        topic,
        "typing",
        Some(""),
        None,
        None,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1351: structured row for one currently-active typer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TyperRow {
    pub sender_id: String,
    /// Wall-clock ms when this typing indicator expires. Filtered to only
    /// rows where `expires_at_ms > now_ms`.
    pub expires_at_ms: i64,
    /// Envelope timestamp (when the typing indicator was emitted). Drives
    /// sort order in `compute_active_typers`.
    pub ts: i64,
}

impl TyperRow {
    fn to_json(&self) -> Value {
        json!({
            "sender_id": self.sender_id,
            "expires_at_ms": self.expires_at_ms,
            "ts": self.ts,
        })
    }
}

/// T-1351: pure helper — derive the active typer list from a topic walk.
///
/// For each `msg_type=typing` envelope, keep only the LATEST per sender
/// (latest in offset order — most recent typing intent wins). After
/// reduction, drop entries whose `expires_at_ms <= now_ms`. Returns rows
/// sorted by `ts` descending (most recently active first); ties break on
/// sender_id ascending for determinism.
pub(crate) fn compute_active_typers(envelopes: &[Value], now_ms: i64) -> Vec<TyperRow> {
    use std::collections::HashMap;
    let mut latest: HashMap<String, TyperRow> = HashMap::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("typing") {
            continue;
        }
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let expires_at_ms = env
            .get("metadata")
            .and_then(|md| md.get("expires_at_ms"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        // Latest-per-sender: replace if this envelope's offset is greater
        // (envelopes arrive in offset order, so a simple insert/replace
        // works without checking offset explicitly).
        latest.insert(
            sender.clone(),
            TyperRow {
                sender_id: sender,
                expires_at_ms,
                ts,
            },
        );
    }
    let mut rows: Vec<TyperRow> = latest
        .into_values()
        .filter(|r| r.expires_at_ms > now_ms)
        .collect();
    rows.sort_by(|a, b| {
        b.ts.cmp(&a.ts)
            .then_with(|| a.sender_id.cmp(&b.sender_id))
    });
    rows
}

/// T-1351: list active typers on a topic.
pub(crate) async fn cmd_channel_typing_list(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let rows = compute_active_typers(&envelopes, now_ms);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(TyperRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No active typers on topic '{topic}'.");
        return Ok(());
    }
    for r in &rows {
        let remaining = r.expires_at_ms - now_ms;
        println!(
            "{sender}: typing (expires in {remaining}ms)",
            sender = r.sender_id,
        );
    }
    Ok(())
}

/// T-1348: pure helper — assemble the metadata K=V strings for a forwarded
/// envelope. Returns `["forwarded_from=<src>:<off>", "forwarded_sender=<id>"]`
/// in stable order (forwarded_from first). Used by `cmd_channel_forward`.
pub(crate) fn build_forward_metadata(
    src_topic: &str,
    offset: u64,
    original_sender: &str,
) -> Vec<String> {
    vec![
        format!("forwarded_from={src_topic}:{offset}"),
        format!("forwarded_sender={original_sender}"),
    ]
}

/// T-1348: copy an envelope from one topic to another, preserving payload
/// and msg_type. The new envelope on dst is signed by the current identity
/// (so it's NOT a faithful relay — the forwarder is the sender on record);
/// metadata records the source for trace-back.
pub(crate) async fn cmd_channel_forward(
    src_topic: &str,
    offset: u64,
    dst_topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    // Walk the source topic to find the envelope at offset. Walking is
    // consistent with how channel quote / channel ancestors do their
    // lookups — saves us from inventing a single-offset RPC convention.
    let envelopes = walk_topic_full(&sock, src_topic).await?;
    let src_env = envelopes
        .iter()
        .find(|e| e.get("offset").and_then(|v| v.as_u64()) == Some(offset))
        .ok_or_else(|| anyhow!("Source topic '{src_topic}' has no envelope at offset {offset}"))?;
    let original_sender = src_env
        .get("sender_id")
        .and_then(|v| v.as_str())
        .unwrap_or("?")
        .to_string();
    let original_msg_type = src_env
        .get("msg_type")
        .and_then(|v| v.as_str())
        .unwrap_or("post")
        .to_string();
    let payload_b64 = src_env
        .get("payload_b64")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    use base64::Engine;
    let payload_bytes = base64::engine::general_purpose::STANDARD
        .decode(payload_b64)
        .unwrap_or_default();
    let payload_str = String::from_utf8_lossy(&payload_bytes).into_owned();
    let metadata = build_forward_metadata(src_topic, offset, &original_sender);
    cmd_channel_post(
        dst_topic,
        &original_msg_type,
        Some(&payload_str),
        None,
        None,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1345: pure helper — emit a pin/unpin envelope. Wraps `cmd_channel_post`
/// with `msg_type=pin`, an empty payload, and metadata
/// `pin_target=<offset>` + `action=pin|unpin`. Latest action per target wins
/// when computing the current pin set (see `compute_pinned_set`).
pub(crate) async fn cmd_channel_pin(
    topic: &str,
    offset: u64,
    unpin: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let action = if unpin { "unpin" } else { "pin" };
    let metadata = vec![
        format!("pin_target={offset}"),
        format!("action={action}"),
    ];
    cmd_channel_post(
        topic,
        "pin",
        Some(""),
        None,
        None,
        None, // reply_to unused — pin_target carries the reference
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1345: structured row for one currently-pinned target. `pinned_ts` is
/// the ts of the most-recent pin envelope (used for sort order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PinRow {
    pub target: u64,
    pub pinned_by: String,
    pub pinned_ts: i64,
    /// Payload preview from the *original* envelope at `target`. None if the
    /// target is no longer in the topic (rare — would mean a pin that
    /// references an offset out of range). Empty string when the original
    /// payload is non-utf8 / undecodable.
    pub payload: Option<String>,
}

impl PinRow {
    fn to_json(&self) -> Value {
        json!({
            "target": self.target,
            "pinned_by": self.pinned_by,
            "pinned_ts": self.pinned_ts,
            "payload": self.payload,
        })
    }
}

/// T-1345: compute the current pin set from a topic walk.
///
/// Iterates `envelopes` in input order, applying each `msg_type=pin` envelope
/// per its `metadata.action`:
///   - `action=pin`     → record (or update) PinRow for the target
///   - `action=unpin`   → remove the entry for the target
///
/// After the scan, the original envelope at each pinned target is looked up
/// to fill `payload`. Returns rows sorted by `pinned_ts` descending (most
/// recently pinned first); ties break on target ascending for determinism.
///
/// Pure helper — no I/O, no allocation outside the result vector. Designed
/// to be unit-testable and reusable from any topic-walking command.
pub(crate) fn compute_pinned_set(envelopes: &[Value]) -> Vec<PinRow> {
    use std::collections::HashMap;
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    let mut active: HashMap<u64, PinRow> = HashMap::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("pin") {
            continue;
        }
        let md = match env.get("metadata") {
            Some(m) => m,
            None => continue,
        };
        let target = match md
            .get("pin_target")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(t) => t,
            None => continue,
        };
        let action = md.get("action").and_then(|v| v.as_str()).unwrap_or("pin");
        if action == "unpin" {
            active.remove(&target);
            continue;
        }
        // Default + explicit "pin" both pin.
        let pinned_by = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let pinned_ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        active.insert(
            target,
            PinRow {
                target,
                pinned_by,
                pinned_ts,
                payload: None,
            },
        );
    }
    // Fill payload from original envelope (if still in topic).
    for row in active.values_mut() {
        if let Some(orig) = by_off.get(&row.target) {
            row.payload = Some(decode_payload_lossy(orig));
        }
    }
    let mut rows: Vec<PinRow> = active.into_values().collect();
    rows.sort_by(|a, b| {
        b.pinned_ts
            .cmp(&a.pinned_ts)
            .then_with(|| a.target.cmp(&b.target))
    });
    rows
}

/// T-1345: render the current pin set for a topic. Walks the topic, computes
/// the pin set, and renders human or JSON.
pub(crate) async fn cmd_channel_pinned(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_pinned_set(&envelopes);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(PinRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No pinned messages on topic '{topic}'.");
        return Ok(());
    }
    for r in &rows {
        let payload = r.payload.as_deref().unwrap_or("(target missing)");
        println!(
            "[{target}] pinned_by={by} ts={ts}: {payload}",
            target = r.target,
            by = r.pinned_by,
            ts = r.pinned_ts,
        );
    }
    Ok(())
}

/// T-1354: pure helper — emit a star/unstar envelope. Wraps `cmd_channel_post`
/// with `msg_type=star`, an empty payload, and metadata
/// `star_target=<offset>` + `star=true|false`. Latest action per
/// (sender_id, target) wins when computing the current star set (see
/// `compute_starred_set`).
pub(crate) async fn cmd_channel_star(
    topic: &str,
    offset: u64,
    unstar: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let star_value = if unstar { "false" } else { "true" };
    let metadata = vec![
        format!("star_target={offset}"),
        format!("star={star_value}"),
    ];
    cmd_channel_post(
        topic,
        "star",
        Some(""),
        None,
        None,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1354: structured row for one currently-starred (sender_id, target) pair.
/// `starred_ts` is the ts of the most-recent star envelope (used for sort
/// order). `payload` is filled from the original envelope at `target`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StarRow {
    pub target: u64,
    pub starred_by: String,
    pub starred_ts: i64,
    pub payload: Option<String>,
}

impl StarRow {
    fn to_json(&self) -> Value {
        json!({
            "target": self.target,
            "starred_by": self.starred_by,
            "starred_ts": self.starred_ts,
            "payload": self.payload,
        })
    }
}

/// T-1354: compute the current star set from a topic walk.
///
/// Iterates `envelopes` in input order, applying each `msg_type=star` envelope
/// per its `metadata.star` flag, keyed by `(sender_id, star_target)`:
///   - `star=true`  → record/update StarRow for that (user, target)
///   - `star=false` → remove the entry for that (user, target)
///
/// When `caller` is `Some(fp)`, only stars by that fingerprint are returned.
/// When `caller` is `None`, all users' stars are returned (used by --all).
///
/// After the scan, the original envelope at each starred target is looked up
/// to fill `payload`. Returns rows sorted by `starred_ts` descending; ties
/// break on (target, starred_by) ascending for determinism.
///
/// Pure helper — no I/O. Designed for unit tests.
pub(crate) fn compute_starred_set(
    envelopes: &[Value],
    caller: Option<&str>,
) -> Vec<StarRow> {
    use std::collections::HashMap;
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    let mut active: HashMap<(String, u64), StarRow> = HashMap::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("star") {
            continue;
        }
        let md = match env.get("metadata") {
            Some(m) => m,
            None => continue,
        };
        let target = match md
            .get("star_target")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(t) => t,
            None => continue,
        };
        let star_flag = md.get("star").and_then(|v| v.as_str()).unwrap_or("true");
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        if let Some(fp) = caller
            && sender != fp
        {
            continue;
        }
        let key = (sender.clone(), target);
        if star_flag == "false" {
            active.remove(&key);
            continue;
        }
        let starred_ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        active.insert(
            key,
            StarRow {
                target,
                starred_by: sender,
                starred_ts,
                payload: None,
            },
        );
    }
    for row in active.values_mut() {
        if let Some(orig) = by_off.get(&row.target) {
            row.payload = Some(decode_payload_lossy(orig));
        }
    }
    let mut rows: Vec<StarRow> = active.into_values().collect();
    rows.sort_by(|a, b| {
        b.starred_ts
            .cmp(&a.starred_ts)
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.starred_by.cmp(&b.starred_by))
    });
    rows
}

/// T-1354: render the current star set for a topic. Defaults to the calling
/// user's stars; pass `all=true` to include every user.
pub(crate) async fn cmd_channel_starred(
    topic: &str,
    all: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let me_owned: Option<String> = if all {
        None
    } else {
        let id = load_identity_or_create()
            .context("Loading identity for star list scope")?;
        Some(id.fingerprint().to_string())
    };
    let rows = compute_starred_set(&envelopes, me_owned.as_deref());
    if json_output {
        let arr: Vec<Value> = rows.iter().map(StarRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        let scope = if all { "anyone" } else { "you" };
        println!("No starred messages on topic '{topic}' (scope: {scope}).");
        return Ok(());
    }
    for r in &rows {
        let payload = r.payload.as_deref().unwrap_or("(target missing)");
        println!(
            "[{target}] starred_by={by} ts={ts}: {payload}",
            target = r.target,
            by = r.starred_by,
            ts = r.starred_ts,
        );
    }
    Ok(())
}

/// T-1355: emit a poll_start envelope. Payload is the question, options are
/// joined with `|` into `metadata.poll_options`. The returned offset is the
/// poll id used by `vote`/`end`/`results`.
pub(crate) async fn cmd_channel_poll_start(
    topic: &str,
    question: &str,
    options: &[String],
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    if options.len() < 2 {
        return Err(anyhow!(
            "poll requires at least 2 options (got {})",
            options.len()
        ));
    }
    if options.iter().any(|o| o.contains('|')) {
        return Err(anyhow!(
            "option labels cannot contain '|' (used as the metadata delimiter)"
        ));
    }
    let metadata = vec![format!("poll_options={}", options.join("|"))];
    cmd_channel_post(
        topic,
        "poll_start",
        Some(question),
        None,
        None,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1355: emit a poll_vote envelope. Latest vote per (poll_id, sender) wins.
pub(crate) async fn cmd_channel_poll_vote(
    topic: &str,
    poll_id: u64,
    choice: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let metadata = vec![
        format!("poll_id={poll_id}"),
        format!("poll_choice={choice}"),
    ];
    cmd_channel_post(
        topic,
        "poll_vote",
        Some(""),
        None,
        None,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1355: emit a poll_end envelope. Aggregator drops votes whose ts is
/// after this envelope's ts.
pub(crate) async fn cmd_channel_poll_end(
    topic: &str,
    poll_id: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let metadata = vec![format!("poll_id={poll_id}")];
    cmd_channel_post(
        topic,
        "poll_end",
        Some(""),
        None,
        None,
        None,
        &metadata,
        hub,
        json_output,
    )
    .await
}

/// T-1355: per-option tally row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PollOptionRow {
    pub label: String,
    pub count: u64,
    pub voters: Vec<String>,
}

/// T-1355: aggregated poll state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PollState {
    pub poll_id: u64,
    pub question: String,
    pub options: Vec<PollOptionRow>,
    pub closed: bool,
    pub total_votes: u64,
}

/// T-1355: pure helper — derive a poll's current state from a topic walk.
///
/// Locates the `poll_start` envelope at `poll_id`. Returns `None` if absent
/// or wrong msg_type. Walks all `poll_vote` envelopes for that poll_id in
/// offset order — latest vote per sender wins; an out-of-range choice index
/// drops that voter. If a `poll_end` envelope exists for this poll_id, votes
/// whose `ts` is strictly greater than the end ts are ignored, and `closed`
/// is true.
pub(crate) fn compute_poll_state(envelopes: &[Value], poll_id: u64) -> Option<PollState> {
    let start = envelopes.iter().find(|e| {
        e.get("offset").and_then(|v| v.as_u64()) == Some(poll_id)
            && e.get("msg_type").and_then(|v| v.as_str()) == Some("poll_start")
    })?;
    let question = decode_payload_lossy(start);
    let opts_str = start
        .get("metadata")
        .and_then(|m| m.get("poll_options"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let labels: Vec<String> = if opts_str.is_empty() {
        Vec::new()
    } else {
        opts_str.split('|').map(|s| s.to_string()).collect()
    };
    if labels.len() < 2 {
        // Malformed start — treat as no poll for purposes of compute.
        return None;
    }

    let pid = poll_id.to_string();
    // Find poll_end if any.
    let end_ts: Option<i64> = envelopes
        .iter()
        .filter(|e| {
            e.get("msg_type").and_then(|v| v.as_str()) == Some("poll_end")
                && e.get("metadata")
                    .and_then(|m| m.get("poll_id"))
                    .and_then(|v| v.as_str())
                    == Some(pid.as_str())
        })
        .filter_map(|e| {
            e.get("ts_unix_ms")
                .and_then(|v| v.as_i64())
                .or_else(|| e.get("ts").and_then(|v| v.as_i64()))
        })
        .min();
    let closed = end_ts.is_some();

    use std::collections::HashMap;
    // sender -> (choice_index, ts) — latest wins (offset order).
    let mut latest: HashMap<String, (u64, i64)> = HashMap::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("poll_vote") {
            continue;
        }
        let md = match env.get("metadata") {
            Some(m) => m,
            None => continue,
        };
        if md.get("poll_id").and_then(|v| v.as_str()) != Some(pid.as_str()) {
            continue;
        }
        let choice = match md
            .get("poll_choice")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(c) => c,
            None => continue,
        };
        if (choice as usize) >= labels.len() {
            continue;
        }
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        if let Some(ets) = end_ts
            && ts > ets
        {
            continue;
        }
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        latest.insert(sender, (choice, ts));
    }

    let mut option_rows: Vec<PollOptionRow> = labels
        .iter()
        .map(|l| PollOptionRow {
            label: l.clone(),
            count: 0,
            voters: Vec::new(),
        })
        .collect();
    let mut total: u64 = 0;
    let mut by_choice: Vec<Vec<String>> = vec![Vec::new(); labels.len()];
    for (sender, (choice, _ts)) in &latest {
        by_choice[*choice as usize].push(sender.clone());
        total += 1;
    }
    for (i, voters) in by_choice.into_iter().enumerate() {
        let mut v = voters;
        v.sort();
        option_rows[i].count = v.len() as u64;
        option_rows[i].voters = v;
    }
    Some(PollState {
        poll_id,
        question,
        options: option_rows,
        closed,
        total_votes: total,
    })
}

/// T-1355: render poll results. Walks the topic once, computes state, prints.
pub(crate) async fn cmd_channel_poll_results(
    topic: &str,
    poll_id: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let state = compute_poll_state(&envelopes, poll_id).ok_or_else(|| {
        anyhow!(
            "Topic '{topic}' has no poll_start at offset {poll_id} (or it is malformed)"
        )
    })?;
    if json_output {
        let opts: Vec<Value> = state
            .options
            .iter()
            .map(|o| {
                json!({
                    "label": o.label,
                    "count": o.count,
                    "voters": o.voters,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "poll_id": state.poll_id,
                "question": state.question,
                "options": opts,
                "closed": state.closed,
                "total_votes": state.total_votes,
            }))?
        );
        return Ok(());
    }
    let status = if state.closed { "CLOSED" } else { "OPEN" };
    println!(
        "Poll #{} [{}]: {}",
        state.poll_id, status, state.question
    );
    for (i, opt) in state.options.iter().enumerate() {
        println!("  [{i}] {} — {} vote(s)", opt.label, opt.count);
        for v in &opt.voters {
            println!("       · {v}");
        }
    }
    println!("Total votes: {}", state.total_votes);
    Ok(())
}

/// T-1356: aggregated activity digest for a topic, scoped to a time window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DigestSummary {
    pub since_ms: i64,
    pub posts: u64,
    pub distinct_senders: u64,
    pub top_senders: Vec<(String, u64)>,
    pub top_reactions: Vec<(String, u64)>,
    pub pins_added: u64,
    pub pins_removed: u64,
    pub forwards_in: u64,
    pub recent_chats: Vec<DigestChat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DigestChat {
    pub offset: u64,
    pub sender_id: String,
    pub ts: i64,
    pub payload: String,
}

/// T-1356: pure helper — compute a digest from a topic walk + lower bound.
///
/// Filters envelopes to those whose `ts_unix_ms` (or legacy `ts`) is `>=
/// since_ms`. Envelopes without a ts are dropped (defensive).
///
/// Sections:
/// - posts: count of `msg_type=post|chat|note` (the "content" types)
/// - distinct_senders: unique sender_id across all (any-msg-type) envelopes
/// - top_senders: top 3 by content-post count (descending; tie-break sender_id asc)
/// - top_reactions: top 3 reactions by payload (decoded as the emoji)
/// - pins_added / pins_removed: count of pin/unpin events
/// - forwards_in: count of envelopes with `metadata.forwarded_from`
/// - recent_chats: last 3 content posts in offset-asc order, payloads decoded
pub(crate) fn compute_digest(envelopes: &[Value], since_ms: i64) -> DigestSummary {
    use std::collections::HashMap;
    let in_window = |env: &Value| -> Option<i64> {
        env.get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .filter(|t| *t >= since_ms)
    };
    let is_content = |env: &Value| -> bool {
        matches!(
            env.get("msg_type").and_then(|v| v.as_str()),
            Some("post") | Some("chat") | Some("note")
        )
    };

    let mut posts: u64 = 0;
    let mut sender_counts: HashMap<String, u64> = HashMap::new();
    let mut all_senders: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut reaction_counts: HashMap<String, u64> = HashMap::new();
    let mut pins_added: u64 = 0;
    let mut pins_removed: u64 = 0;
    let mut forwards_in: u64 = 0;
    let mut content_envs: Vec<&Value> = Vec::new();

    for env in envelopes {
        if in_window(env).is_none() {
            continue;
        }
        if let Some(s) = env.get("sender_id").and_then(|v| v.as_str()) {
            all_senders.insert(s.to_string());
        }
        if env
            .get("metadata")
            .and_then(|m| m.get("forwarded_from"))
            .is_some()
        {
            forwards_in += 1;
        }
        match env.get("msg_type").and_then(|v| v.as_str()) {
            Some("pin") => {
                let action = env
                    .get("metadata")
                    .and_then(|m| m.get("action"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("pin");
                if action == "unpin" {
                    pins_removed += 1;
                } else {
                    pins_added += 1;
                }
            }
            Some("reaction") => {
                let payload = decode_payload_lossy(env);
                if !payload.is_empty() {
                    *reaction_counts.entry(payload).or_insert(0) += 1;
                }
            }
            _ => {}
        }
        if is_content(env) {
            posts += 1;
            content_envs.push(env);
            if let Some(s) = env.get("sender_id").and_then(|v| v.as_str()) {
                *sender_counts.entry(s.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut top_senders: Vec<(String, u64)> = sender_counts.into_iter().collect();
    top_senders.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_senders.truncate(3);

    let mut top_reactions: Vec<(String, u64)> = reaction_counts.into_iter().collect();
    top_reactions.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_reactions.truncate(3);

    // recent_chats: last 3 by offset.
    content_envs.sort_by_key(|e| e.get("offset").and_then(|v| v.as_u64()).unwrap_or(0));
    let recent_chats: Vec<DigestChat> = content_envs
        .iter()
        .rev()
        .take(3)
        .rev()
        .map(|e| DigestChat {
            offset: e.get("offset").and_then(|v| v.as_u64()).unwrap_or(0),
            sender_id: e
                .get("sender_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            ts: e
                .get("ts_unix_ms")
                .and_then(|v| v.as_i64())
                .or_else(|| e.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0),
            payload: decode_payload_lossy(e),
        })
        .collect();

    DigestSummary {
        since_ms,
        posts,
        distinct_senders: all_senders.len() as u64,
        top_senders,
        top_reactions,
        pins_added,
        pins_removed,
        forwards_in,
        recent_chats,
    }
}

/// T-1356: render the digest. Walks the topic, computes, prints.
pub(crate) async fn cmd_channel_digest(
    topic: &str,
    since_mins: Option<i64>,
    since: Option<i64>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let since_ms = match (since_mins, since) {
        (Some(_), Some(_)) => {
            return Err(anyhow!(
                "--since-mins and --since are mutually exclusive"
            ));
        }
        (Some(n), None) => now_ms - n * 60_000,
        (None, Some(ms)) => ms,
        (None, None) => now_ms - 60 * 60_000, // default: last 60 minutes
    };
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let d = compute_digest(&envelopes, since_ms);

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "since_ms": d.since_ms,
                "posts": d.posts,
                "distinct_senders": d.distinct_senders,
                "top_senders": d.top_senders.iter().map(|(s,c)| json!({"sender_id": s, "count": c})).collect::<Vec<_>>(),
                "top_reactions": d.top_reactions.iter().map(|(r,c)| json!({"reaction": r, "count": c})).collect::<Vec<_>>(),
                "pins_added": d.pins_added,
                "pins_removed": d.pins_removed,
                "forwards_in": d.forwards_in,
                "recent_chats": d.recent_chats.iter().map(|c| json!({
                    "offset": c.offset,
                    "sender_id": c.sender_id,
                    "ts": c.ts,
                    "payload": c.payload,
                })).collect::<Vec<_>>(),
            }))?
        );
        return Ok(());
    }

    println!("Digest for '{topic}' since ts={since}", since = d.since_ms);
    println!(
        "  Posts: {} | Distinct senders: {} | Forwards in: {}",
        d.posts, d.distinct_senders, d.forwards_in
    );
    println!(
        "  Pins: +{} added, -{} removed",
        d.pins_added, d.pins_removed
    );
    if !d.top_senders.is_empty() {
        println!("  Top senders:");
        for (s, c) in &d.top_senders {
            println!("    · {s} — {c}");
        }
    }
    if !d.top_reactions.is_empty() {
        println!("  Top reactions:");
        for (r, c) in &d.top_reactions {
            println!("    · {r} ×{c}");
        }
    }
    if !d.recent_chats.is_empty() {
        println!("  Last {} chat(s):", d.recent_chats.len());
        for c in &d.recent_chats {
            println!(
                "    [{off}] {sender}: {payload}",
                off = c.offset,
                sender = c.sender_id,
                payload = c.payload
            );
        }
    }
    Ok(())
}

/// T-1359: one row in the per-topic emoji-stats output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmojiStatRow {
    pub emoji: String,
    pub count: u64,
    /// (sender_id, per-sender count) sorted by count desc, sender asc.
    pub reactors: Vec<(String, u64)>,
}

impl EmojiStatRow {
    fn to_json(&self) -> Value {
        let reactors: Vec<Value> = self
            .reactors
            .iter()
            .map(|(s, c)| json!({"sender_id": s, "count": c}))
            .collect();
        json!({
            "emoji": self.emoji,
            "count": self.count,
            "distinct_reactors": self.reactors.len(),
            "reactors": reactors,
        })
    }
}

/// T-1359: pure helper — compute per-emoji stats from a topic walk.
///
/// Filters envelopes to `msg_type=reaction` whose offset is NOT in
/// `redacted_offsets(envelopes)`. The reaction's payload is the emoji.
/// Result is sorted by total count desc, ties break on emoji ascending.
/// Pure — no I/O.
pub(crate) fn compute_emoji_stats(envelopes: &[Value]) -> Vec<EmojiStatRow> {
    use std::collections::HashMap;
    let redacted = redacted_offsets(envelopes);
    // emoji -> sender -> count
    let mut by_emoji: HashMap<String, HashMap<String, u64>> = HashMap::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("reaction") {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        let emoji = decode_payload_lossy(env);
        if emoji.is_empty() {
            continue;
        }
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        *by_emoji
            .entry(emoji)
            .or_default()
            .entry(sender)
            .or_insert(0) += 1;
    }
    let mut rows: Vec<EmojiStatRow> = by_emoji
        .into_iter()
        .map(|(emoji, reactors_map)| {
            let count = reactors_map.values().sum();
            let mut reactors: Vec<(String, u64)> = reactors_map.into_iter().collect();
            reactors.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            EmojiStatRow {
                emoji,
                count,
                reactors,
            }
        })
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.emoji.cmp(&b.emoji)));
    rows
}

/// T-1359: render the per-topic emoji breakdown.
pub(crate) async fn cmd_channel_emoji_stats(
    topic: &str,
    by_sender: bool,
    top: Option<usize>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let mut rows = compute_emoji_stats(&envelopes);
    if let Some(n) = top {
        rows.truncate(n);
    }

    if json_output {
        let arr: Vec<Value> = rows.iter().map(EmojiStatRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No reactions on topic '{topic}'.");
        return Ok(());
    }
    println!("Emoji stats for '{topic}':");
    for r in &rows {
        println!(
            "  {emoji} ×{count} ({n} reactor(s))",
            emoji = r.emoji,
            count = r.count,
            n = r.reactors.len()
        );
        if by_sender {
            for (s, c) in &r.reactors {
                println!("    · {s} ×{c}");
            }
        }
    }
    Ok(())
}

/// T-1363: one rendered line in a snippet block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SnippetLine {
    pub offset: u64,
    pub sender: String,
    pub payload: String,
    pub is_target: bool,
}

impl SnippetLine {
    fn to_json(&self) -> Value {
        json!({
            "offset": self.offset,
            "sender": self.sender,
            "payload": self.payload,
            "is_target": self.is_target,
        })
    }
}

/// T-1363: pure helper — pick the snippet window from a topic walk.
///
/// Filters envelopes to content msg_types (`post`/`chat`/`note`) and skips
/// meta types (reaction/edit/redaction/receipt/topic_metadata) so the
/// snippet stays focused.
///
/// Locates the target offset, includes up to `lines` content envelopes on
/// each side. Returns `None` when the target is not in `envelopes` or is
/// itself a meta type. Pure — no I/O.
pub(crate) fn compute_snippet(
    envelopes: &[Value],
    target_offset: u64,
    lines: u64,
) -> Option<Vec<SnippetLine>> {
    let is_content = |env: &Value| -> bool {
        matches!(
            env.get("msg_type").and_then(|v| v.as_str()),
            Some("post") | Some("chat") | Some("note")
        )
    };
    let mut content_envs: Vec<&Value> = envelopes.iter().filter(|e| is_content(e)).collect();
    content_envs.sort_by_key(|e| e.get("offset").and_then(|v| v.as_u64()).unwrap_or(0));

    let target_idx = content_envs
        .iter()
        .position(|e| e.get("offset").and_then(|v| v.as_u64()) == Some(target_offset))?;
    let lines_usize = lines as usize;
    let lo = target_idx.saturating_sub(lines_usize);
    let hi = (target_idx + lines_usize + 1).min(content_envs.len());

    let snippet: Vec<SnippetLine> = content_envs[lo..hi]
        .iter()
        .map(|e| {
            let off = e.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
            SnippetLine {
                offset: off,
                sender: e
                    .get("sender_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string(),
                payload: decode_payload_lossy(e),
                is_target: off == target_offset,
            }
        })
        .collect();
    Some(snippet)
}

/// T-1363: render the snippet for a target envelope.
pub(crate) async fn cmd_channel_snippet(
    topic: &str,
    offset: u64,
    lines: u64,
    header: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let snippet = compute_snippet(&envelopes, offset, lines).ok_or_else(|| {
        anyhow!(
            "Topic '{topic}' has no content envelope at offset {offset} (or it's a meta type)"
        )
    })?;
    if json_output {
        let arr: Vec<Value> = snippet.iter().map(SnippetLine::to_json).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "topic": topic,
                "target_offset": offset,
                "lines": arr,
            }))?
        );
        return Ok(());
    }
    if header {
        println!("From `{topic}` @ offset {offset}:");
    }
    println!("```");
    for line in &snippet {
        let prefix = if line.is_target { ">>" } else { "  " };
        println!(
            "{prefix} [{off}] {sender}: {payload}",
            off = line.offset,
            sender = line.sender,
            payload = line.payload,
        );
    }
    println!("```");
    Ok(())
}

/// T-1362: one reaction row produced by `compute_reactions_of`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReactionsOfRow {
    pub reaction_offset: u64,
    pub parent_offset: u64,
    pub emoji: String,
    pub parent_payload: Option<String>,
    pub ts: i64,
}

impl ReactionsOfRow {
    fn to_json(&self) -> Value {
        json!({
            "reaction_offset": self.reaction_offset,
            "parent_offset": self.parent_offset,
            "emoji": self.emoji,
            "parent_payload": self.parent_payload,
            "ts": self.ts,
        })
    }
}

/// T-1362: pure helper — list every active (non-redacted) reaction posted
/// by `sender` on this topic, with the parent payload preview filled in.
///
/// Filtering:
/// - `msg_type == "reaction"` AND `sender_id == sender`
/// - reaction's offset NOT in `redacted_offsets`
/// - reaction must carry `metadata.in_reply_to` parseable as u64
///
/// Sort: by reaction offset descending (most recent first).
/// Pure — no I/O.
pub(crate) fn compute_reactions_of(envelopes: &[Value], sender: &str) -> Vec<ReactionsOfRow> {
    use std::collections::HashMap;
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    let redacted = redacted_offsets(envelopes);
    let mut rows: Vec<ReactionsOfRow> = Vec::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("reaction") {
            continue;
        }
        if env.get("sender_id").and_then(|v| v.as_str()) != Some(sender) {
            continue;
        }
        let r_off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&r_off) {
            continue;
        }
        let parent_offset = match parent_offset_of(env) {
            Some(p) => p,
            None => continue,
        };
        let emoji = decode_payload_lossy(env);
        if emoji.is_empty() {
            continue;
        }
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let parent_payload = by_off.get(&parent_offset).map(|p| decode_payload_lossy(p));
        rows.push(ReactionsOfRow {
            reaction_offset: r_off,
            parent_offset,
            emoji,
            parent_payload,
            ts,
        });
    }
    rows.sort_by(|a, b| b.reaction_offset.cmp(&a.reaction_offset));
    rows
}

/// T-1362: render the reactions-of view.
pub(crate) async fn cmd_channel_reactions_of(
    topic: &str,
    sender: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let scope = match sender {
        Some(s) => s.to_string(),
        None => {
            let id = load_identity_or_create()
                .context("Loading identity for reactions-of scope")?;
            id.fingerprint().to_string()
        }
    };
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_reactions_of(&envelopes, &scope);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(ReactionsOfRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No reactions by {scope} on topic '{topic}'.");
        return Ok(());
    }
    println!("Reactions by {scope} on '{topic}':");
    for r in &rows {
        let preview = r
            .parent_payload
            .as_deref()
            .unwrap_or("(parent missing)");
        println!(
            "  {emoji} → offset {parent} ({preview})",
            emoji = r.emoji,
            parent = r.parent_offset,
        );
    }
    Ok(())
}

/// T-1368: aggregated per-topic statistics. Distinct from the lightweight
/// `TopicStats` (T-1335) used by `channel list` — this one is the full
/// dashboard shape for `channel topic-stats`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FullTopicStats {
    pub total: usize,
    pub distinct_senders: usize,
    pub by_msg_type: Vec<(String, usize)>,
    pub top_senders: Vec<(String, usize)>,
    pub distinct_emojis: usize,
    pub top_emojis: Vec<(String, usize)>,
    pub thread_roots: usize,
    pub active_pins: usize,
    pub forwards_in: usize,
    pub edits: usize,
    pub redactions: usize,
    pub first_ts_ms: Option<i64>,
    pub last_ts_ms: Option<i64>,
}

impl FullTopicStats {
    fn to_json(&self) -> Value {
        json!({
            "total": self.total,
            "distinct_senders": self.distinct_senders,
            "by_msg_type": self.by_msg_type.iter().map(|(t, c)| json!({"msg_type": t, "count": c})).collect::<Vec<_>>(),
            "top_senders": self.top_senders.iter().map(|(s, c)| json!({"sender_id": s, "count": c})).collect::<Vec<_>>(),
            "distinct_emojis": self.distinct_emojis,
            "top_emojis": self.top_emojis.iter().map(|(e, c)| json!({"emoji": e, "count": c})).collect::<Vec<_>>(),
            "thread_roots": self.thread_roots,
            "active_pins": self.active_pins,
            "forwards_in": self.forwards_in,
            "edits": self.edits,
            "redactions": self.redactions,
            "first_ts_ms": self.first_ts_ms,
            "last_ts_ms": self.last_ts_ms,
        })
    }
}

/// T-1368: pure helper — aggregate per-topic statistics.
///
/// Counters exclude redacted envelopes (their offset appears in
/// `redacted_offsets`). The redaction envelopes themselves are counted
/// separately under `redactions`.
///
/// Top-N lists are sorted by count desc with name asc tiebreak; truncated
/// to 5 rows. Time span uses `ts_unix_ms` (falling back to `ts`).
pub(crate) fn compute_full_topic_stats(envelopes: &[Value]) -> FullTopicStats {
    use std::collections::{HashMap, HashSet};
    let redacted = redacted_offsets(envelopes);
    let mut total: usize = 0;
    let mut by_type: HashMap<String, usize> = HashMap::new();
    let mut by_sender: HashMap<String, usize> = HashMap::new();
    let mut emoji_count: HashMap<String, usize> = HashMap::new();
    let mut thread_roots_set: HashSet<u64> = HashSet::new();
    let mut active_pins: HashSet<u64> = HashSet::new();
    let mut forwards_in: usize = 0;
    let mut edits: usize = 0;
    let mut redactions: usize = 0;
    let mut first_ts: Option<i64> = None;
    let mut last_ts: Option<i64> = None;

    // First pass: count redactions specially (they're counted regardless of
    // whether the redaction envelope itself is redacted).
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) == Some("redaction") {
            redactions += 1;
        }
    }

    // Two-pass pin state: pin = active, unpin = removes. Last-write-wins.
    let mut pin_state: HashMap<u64, bool> = HashMap::new();

    for env in envelopes {
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        total += 1;
        let mt = env
            .get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        *by_type.entry(mt.clone()).or_insert(0) += 1;
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        *by_sender.entry(sender).or_insert(0) += 1;
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()));
        if let Some(t) = ts {
            first_ts = Some(first_ts.map_or(t, |f| f.min(t)));
            last_ts = Some(last_ts.map_or(t, |l| l.max(t)));
        }
        // Per-type counters
        match mt.as_str() {
            "reaction" => {
                let emoji = decode_payload_lossy(env);
                if !emoji.is_empty() {
                    *emoji_count.entry(emoji).or_insert(0) += 1;
                }
            }
            "edit" => {
                edits += 1;
            }
            "pin" => {
                if let Some(md) = env.get("metadata")
                    && let Some(target) = md
                        .get("pin_target")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                {
                    let action = md.get("action").and_then(|v| v.as_str()).unwrap_or("pin");
                    pin_state.insert(target, action != "unpin");
                }
            }
            _ => {}
        }
        // Thread root: any envelope referenced by another envelope's in_reply_to
        if let Some(parent) = parent_offset_of(env) {
            thread_roots_set.insert(parent);
        }
        // Forwards-in: detected via metadata
        if extract_forward(env).is_some() {
            forwards_in += 1;
        }
    }

    for (target, active) in &pin_state {
        if *active {
            active_pins.insert(*target);
        }
    }

    let distinct_senders = by_sender.len();
    let distinct_emojis = emoji_count.len();

    let mut by_msg_type: Vec<(String, usize)> = by_type.into_iter().collect();
    by_msg_type.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut top_senders: Vec<(String, usize)> = by_sender.into_iter().collect();
    top_senders.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_senders.truncate(5);

    let mut top_emojis: Vec<(String, usize)> = emoji_count.into_iter().collect();
    top_emojis.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_emojis.truncate(5);

    FullTopicStats {
        total,
        distinct_senders,
        by_msg_type,
        top_senders,
        distinct_emojis,
        top_emojis,
        thread_roots: thread_roots_set.len(),
        active_pins: active_pins.len(),
        forwards_in,
        edits,
        redactions,
        first_ts_ms: first_ts,
        last_ts_ms: last_ts,
    }
}

/// T-1368: render the topic-stats dashboard.
pub(crate) async fn cmd_channel_topic_stats(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let stats = compute_full_topic_stats(&envelopes);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&stats.to_json())?);
        return Ok(());
    }
    println!("Topic-stats for '{topic}':");
    println!("  total envelopes:     {}", stats.total);
    println!("  distinct senders:    {}", stats.distinct_senders);
    println!("  thread roots:        {}", stats.thread_roots);
    println!("  active pins:         {}", stats.active_pins);
    println!("  forwards in:         {}", stats.forwards_in);
    println!("  edits:               {}", stats.edits);
    println!("  redactions:          {}", stats.redactions);
    println!("  distinct emojis:     {}", stats.distinct_emojis);
    if let (Some(f), Some(l)) = (stats.first_ts_ms, stats.last_ts_ms) {
        println!("  time span (ms):      {f} → {l}  ({} ms)", l - f);
    }
    if !stats.by_msg_type.is_empty() {
        println!("  by msg_type:");
        for (t, c) in &stats.by_msg_type {
            println!("    {t}: {c}");
        }
    }
    if !stats.top_senders.is_empty() {
        println!("  top senders:");
        for (s, c) in &stats.top_senders {
            println!("    {s}: {c}");
        }
    }
    if !stats.top_emojis.is_empty() {
        println!("  top emojis:");
        for (e, c) in &stats.top_emojis {
            println!("    {e}: {c}");
        }
    }
    Ok(())
}

/// T-1367: one row in the forwards-of view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ForwardOfRow {
    pub forward_offset: u64,
    pub origin_topic: String,
    pub origin_offset: u64,
    pub origin_sender: String,
    pub payload: String,
    pub ts: i64,
}

impl ForwardOfRow {
    fn to_json(&self) -> Value {
        json!({
            "forward_offset": self.forward_offset,
            "origin_topic": self.origin_topic,
            "origin_offset": self.origin_offset,
            "origin_sender": self.origin_sender,
            "payload": self.payload,
            "ts": self.ts,
        })
    }
}

/// T-1367: pure helper — list every active forward envelope by `sender`.
///
/// A forward envelope is identified by `extract_forward` succeeding
/// (`metadata.forwarded_from` parseable as `"<origin-topic>:<origin-offset>"`
/// AND `metadata.forwarded_sender` present). Forwarded envelopes preserve
/// the *original* msg_type (e.g. "chat"), so msg_type isn't the discriminator —
/// the metadata pair is.
///
/// Filters:
/// - `sender_id == sender` (the forwarder, not the original poster)
/// - offset NOT in `redacted_offsets`
/// - `extract_forward` succeeds (well-formed metadata)
///
/// Sort: forward_offset descending (most recent first). Pure — no I/O.
pub(crate) fn compute_forwards_of(envelopes: &[Value], sender: &str) -> Vec<ForwardOfRow> {
    let redacted = redacted_offsets(envelopes);
    let mut rows: Vec<ForwardOfRow> = Vec::new();
    for env in envelopes {
        if env.get("sender_id").and_then(|v| v.as_str()) != Some(sender) {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        let (origin_topic, origin_offset, origin_sender) = match extract_forward(env) {
            Some(t) => t,
            None => continue,
        };
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        rows.push(ForwardOfRow {
            forward_offset: off,
            origin_topic,
            origin_offset,
            origin_sender,
            payload: decode_payload_lossy(env),
            ts,
        });
    }
    rows.sort_by(|a, b| b.forward_offset.cmp(&a.forward_offset));
    rows
}

/// T-1367: render the forwards-of view.
pub(crate) async fn cmd_channel_forwards_of(
    topic: &str,
    sender: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let scope = match sender {
        Some(s) => s.to_string(),
        None => {
            let id = load_identity_or_create()
                .context("Loading identity for forwards-of scope")?;
            id.fingerprint().to_string()
        }
    };
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_forwards_of(&envelopes, &scope);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(ForwardOfRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No forwards by {scope} on topic '{topic}'.");
        return Ok(());
    }
    println!("Forwards by {scope} on '{topic}':");
    for r in &rows {
        let preview = if r.payload.len() > 60 {
            format!("{}…", &r.payload[..60])
        } else {
            r.payload.clone()
        };
        println!(
            "  [forward {fo}] from {ot}:{oo} (orig sender {os}): {preview}",
            fo = r.forward_offset,
            ot = r.origin_topic,
            oo = r.origin_offset,
            os = r.origin_sender,
        );
    }
    Ok(())
}

/// T-1370: one row in the replies-of view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepliesOfRow {
    pub reply_offset: u64,
    pub parent_offset: u64,
    pub parent_sender: String,
    pub parent_payload: String,
    pub reply_payload: String,
    pub ts_ms: i64,
}

impl RepliesOfRow {
    fn to_json(&self) -> Value {
        json!({
            "reply_offset": self.reply_offset,
            "parent_offset": self.parent_offset,
            "parent_sender": self.parent_sender,
            "parent_payload": self.parent_payload,
            "reply_payload": self.reply_payload,
            "ts_ms": self.ts_ms,
        })
    }
}

/// T-1370: pure helper — list every reply envelope by `sender`.
///
/// A "reply" is an envelope where `metadata.in_reply_to` parses as a u64 AND
/// `msg_type != "reaction"`. Reactions also carry `in_reply_to` (T-1314) but
/// are a different aggregate — see `compute_reactions_of` for that view.
///
/// Filters:
/// - `sender_id == sender`
/// - `parent_offset_of(env)` is `Some`
/// - `msg_type != "reaction"`
/// - reply offset NOT in `redacted_offsets`
///
/// `parent_payload` / `parent_sender` are best-effort: empty strings if the
/// parent offset is absent from the topic snapshot or itself redacted.
///
/// Sort: `reply_offset` descending (most recent first). Pure — no I/O.
pub(crate) fn compute_replies_of(envelopes: &[Value], sender: &str) -> Vec<RepliesOfRow> {
    use std::collections::HashMap;
    let redacted = redacted_offsets(envelopes);
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    let mut rows: Vec<RepliesOfRow> = Vec::new();
    for env in envelopes {
        if env.get("sender_id").and_then(|v| v.as_str()) != Some(sender) {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        if env.get("msg_type").and_then(|v| v.as_str()) == Some("reaction") {
            continue;
        }
        let parent = match parent_offset_of(env) {
            Some(p) => p,
            None => continue,
        };
        let (parent_sender, parent_payload) = match by_off.get(&parent) {
            Some(p) if !redacted.contains(&parent) => (
                p.get("sender_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                decode_payload_lossy(p),
            ),
            _ => (String::new(), String::new()),
        };
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        rows.push(RepliesOfRow {
            reply_offset: off,
            parent_offset: parent,
            parent_sender,
            parent_payload,
            reply_payload: decode_payload_lossy(env),
            ts_ms: ts,
        });
    }
    rows.sort_by(|a, b| b.reply_offset.cmp(&a.reply_offset));
    rows
}

/// T-1370: render the replies-of view.
pub(crate) async fn cmd_channel_replies_of(
    topic: &str,
    sender: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let scope = match sender {
        Some(s) => s.to_string(),
        None => {
            let id = load_identity_or_create()
                .context("Loading identity for replies-of scope")?;
            id.fingerprint().to_string()
        }
    };
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_replies_of(&envelopes, &scope);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(RepliesOfRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No replies by {scope} on topic '{topic}'.");
        return Ok(());
    }
    println!("Replies by {scope} on '{topic}':");
    for r in &rows {
        let preview = |s: &str, n: usize| -> String {
            if s.len() > n {
                format!("{}…", &s[..n])
            } else {
                s.to_string()
            }
        };
        let parent_line = if r.parent_payload.is_empty() {
            format!("  ↳ to [{po}] (parent missing or redacted)", po = r.parent_offset)
        } else {
            format!(
                "  ↳ to [{po}] {ps}: {pp}",
                po = r.parent_offset,
                ps = r.parent_sender,
                pp = preview(&r.parent_payload, 60),
            )
        };
        println!("[reply {ro}] {rp}", ro = r.reply_offset, rp = preview(&r.reply_payload, 60));
        println!("{parent_line}");
    }
    Ok(())
}

/// T-1371: one row in the mentions-of view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MentionsOfRow {
    pub mention_offset: u64,
    pub sender_id: String,
    pub payload: String,
    pub mentions_csv: String,
    pub ts_ms: i64,
}

impl MentionsOfRow {
    fn to_json(&self) -> Value {
        json!({
            "mention_offset": self.mention_offset,
            "sender_id": self.sender_id,
            "payload": self.payload,
            "mentions_csv": self.mentions_csv,
            "ts_ms": self.ts_ms,
        })
    }
}

/// T-1371: pure helper — list every envelope on the topic that mentions
/// `user` via `metadata.mentions` CSV, regardless of author.
///
/// Filters:
/// - `mentions_match(metadata.mentions, user)` is true (T-1333 rules: empty
///   target rejected; literal-equality on parts; `target == "*"` matches any
///   non-empty csv; csv containing `*` matches any specific target)
/// - msg_type NOT in `UNREAD_META_TYPES` (skip receipt/reaction/edit/...)
/// - offset NOT in redacted_offsets
///
/// Sort: `mention_offset` descending. Pure — no I/O.
pub(crate) fn compute_mentions_of(envelopes: &[Value], user: &str) -> Vec<MentionsOfRow> {
    let redacted = redacted_offsets(envelopes);
    let mut rows: Vec<MentionsOfRow> = Vec::new();
    for env in envelopes {
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if UNREAD_META_TYPES.contains(&mt) {
            continue;
        }
        let csv = match extract_mentions(env) {
            Some(s) => s,
            None => continue,
        };
        if !mentions_match(&csv, user) {
            continue;
        }
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        rows.push(MentionsOfRow {
            mention_offset: off,
            sender_id: sender,
            payload: decode_payload_lossy(env),
            mentions_csv: csv,
            ts_ms: ts,
        });
    }
    rows.sort_by(|a, b| b.mention_offset.cmp(&a.mention_offset));
    rows
}

/// T-1371: render the mentions-of view.
pub(crate) async fn cmd_channel_mentions_of(
    topic: &str,
    user: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_mentions_of(&envelopes, user);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(MentionsOfRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No mentions of {user} on topic '{topic}'.");
        return Ok(());
    }
    println!("Mentions of {user} on '{topic}':");
    for r in &rows {
        let preview = if r.payload.len() > 60 {
            format!("{}…", &r.payload[..60])
        } else {
            r.payload.clone()
        };
        println!(
            "  [@ {mo}] {sender} (mentions={csv}): {preview}",
            mo = r.mention_offset,
            sender = r.sender_id,
            csv = r.mentions_csv,
        );
    }
    Ok(())
}

/// T-1372: one row in the pin-history audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PinHistoryRow {
    pub event_offset: u64,
    pub action: String, // "pin" or "unpin"
    pub target_offset: u64,
    pub actor_sender: String,
    pub ts_ms: i64,
    pub target_payload: Option<String>,
}

impl PinHistoryRow {
    fn to_json(&self) -> Value {
        json!({
            "event_offset": self.event_offset,
            "action": self.action,
            "target_offset": self.target_offset,
            "actor_sender": self.actor_sender,
            "ts_ms": self.ts_ms,
            "target_payload": self.target_payload,
        })
    }
}

/// T-1372: pure helper — chronological audit log of pin/unpin events.
///
/// Unlike `compute_pinned_set` (T-1345) which collapses to last-write-wins,
/// this preserves every toggle. Useful for forensic queries: "who pinned
/// what when, and was it ever undone?"
///
/// Filters:
/// - `msg_type == "pin"`
/// - `metadata.pin_target` parses as u64 (malformed envelopes silently skipped)
///
/// Action: `metadata.action` literal ("pin" / "unpin"). Default + missing
/// treated as "pin". `target_payload` filled from the topic snapshot when
/// the target offset is present; None otherwise (target may be redacted,
/// outside the snapshot, or itself a meta envelope).
///
/// Sort: `event_offset` ascending (chronological). Pure — no I/O.
pub(crate) fn compute_pin_history(envelopes: &[Value]) -> Vec<PinHistoryRow> {
    use std::collections::HashMap;
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    let mut rows: Vec<PinHistoryRow> = Vec::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("pin") {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        let md = match env.get("metadata") {
            Some(m) => m,
            None => continue,
        };
        let target = match md
            .get("pin_target")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(t) => t,
            None => continue,
        };
        let action = md
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("pin");
        let action = if action == "unpin" { "unpin" } else { "pin" };
        let actor = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let target_payload = by_off.get(&target).map(|e| decode_payload_lossy(e));
        rows.push(PinHistoryRow {
            event_offset: off,
            action: action.to_string(),
            target_offset: target,
            actor_sender: actor,
            ts_ms: ts,
            target_payload,
        });
    }
    rows.sort_by(|a, b| a.event_offset.cmp(&b.event_offset));
    rows
}

/// T-1372: render the pin-history audit log.
pub(crate) async fn cmd_channel_pin_history(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_pin_history(&envelopes);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(PinHistoryRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No pin events on topic '{topic}'.");
        return Ok(());
    }
    println!("Pin history for '{topic}':");
    for r in &rows {
        let preview = match &r.target_payload {
            Some(p) if p.len() > 60 => format!("{}…", &p[..60]),
            Some(p) => p.clone(),
            None => "(target not in snapshot)".to_string(),
        };
        println!(
            "  [{eo}] {action} → [{to}] by {actor}: {preview}",
            eo = r.event_offset,
            action = r.action.to_uppercase(),
            to = r.target_offset,
            actor = r.actor_sender,
        );
    }
    Ok(())
}

/// T-1373: one row in the redactions audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RedactionRow {
    pub event_offset: u64,
    pub target_offset: u64,
    pub redactor_sender: String,
    pub reason: Option<String>,
    pub ts_ms: i64,
    pub target_payload: Option<String>,
}

impl RedactionRow {
    fn to_json(&self) -> Value {
        json!({
            "event_offset": self.event_offset,
            "target_offset": self.target_offset,
            "redactor_sender": self.redactor_sender,
            "reason": self.reason,
            "ts_ms": self.ts_ms,
            "target_payload": self.target_payload,
        })
    }
}

/// T-1373: pure helper — chronological audit of redaction events.
///
/// One row per `msg_type=redaction` envelope whose `metadata.redacts`
/// parses as u64. Reason is best-effort (passed straight through if
/// `metadata.reason` exists). `target_payload` is best-effort from the
/// topic snapshot — None when the target is missing or itself a meta
/// envelope without payload.
///
/// Sort: `event_offset` ascending. Pure — no I/O. Reuses `extract_redaction`
/// (T-1322) so the discriminator logic stays in one place.
pub(crate) fn compute_redactions(envelopes: &[Value]) -> Vec<RedactionRow> {
    use std::collections::HashMap;
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    let mut rows: Vec<RedactionRow> = Vec::new();
    for env in envelopes {
        let r = match extract_redaction(env) {
            Some(r) => r,
            None => continue,
        };
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let target_payload = by_off.get(&r.target).map(|e| decode_payload_lossy(e));
        rows.push(RedactionRow {
            event_offset: off,
            target_offset: r.target,
            redactor_sender: r.sender.to_string(),
            reason: r.reason,
            ts_ms: ts,
            target_payload,
        });
    }
    rows.sort_by(|a, b| a.event_offset.cmp(&b.event_offset));
    rows
}

/// T-1373: render the redaction audit log.
pub(crate) async fn cmd_channel_redactions(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_redactions(&envelopes);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(RedactionRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No redactions on topic '{topic}'.");
        return Ok(());
    }
    println!("Redactions on '{topic}':");
    for r in &rows {
        let preview = match &r.target_payload {
            Some(p) if p.len() > 60 => format!("{}…", &p[..60]),
            Some(p) => p.clone(),
            None => "(target not in snapshot)".to_string(),
        };
        let reason = match &r.reason {
            Some(r) => format!(" reason=\"{r}\""),
            None => String::new(),
        };
        println!(
            "  [{eo}] redacts → [{to}] by {actor}{reason}: {preview}",
            eo = r.event_offset,
            to = r.target_offset,
            actor = r.redactor_sender,
        );
    }
    Ok(())
}

/// T-1374: one row in the reactions-on rollup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReactionsOnRow {
    pub emoji: String,
    pub count: u64,
    pub senders: Vec<String>,
}

impl ReactionsOnRow {
    fn to_json(&self) -> Value {
        json!({
            "emoji": self.emoji,
            "count": self.count,
            "senders": self.senders,
        })
    }
}

/// T-1374: pure helper — per-target reaction rollup.
///
/// Walks the topic, filters `msg_type=reaction` whose
/// `metadata.in_reply_to == target_offset` and that are not redacted,
/// groups by emoji. `count` is the total reactions (so a single sender
/// hitting 👍 twice still counts twice — captures repeated tapping). `senders`
/// is deduplicated (set semantics, sorted asc) so "who reacted" is clean.
///
/// Sort: count desc, emoji asc tiebreak. Pure — no I/O.
pub(crate) fn compute_reactions_on(envelopes: &[Value], target: u64) -> Vec<ReactionsOnRow> {
    use std::collections::{BTreeSet, HashMap};
    let redacted = redacted_offsets(envelopes);
    // emoji -> (count, set of senders)
    let mut by_emoji: HashMap<String, (u64, BTreeSet<String>)> = HashMap::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("reaction") {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        let parent = match parent_offset_of(env) {
            Some(p) => p,
            None => continue,
        };
        if parent != target {
            continue;
        }
        let emoji = decode_payload_lossy(env);
        if emoji.is_empty() {
            continue;
        }
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let entry = by_emoji.entry(emoji).or_insert_with(|| (0, BTreeSet::new()));
        entry.0 += 1;
        entry.1.insert(sender);
    }
    let mut rows: Vec<ReactionsOnRow> = by_emoji
        .into_iter()
        .map(|(emoji, (count, senders))| ReactionsOnRow {
            emoji,
            count,
            senders: senders.into_iter().collect(),
        })
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.emoji.cmp(&b.emoji)));
    rows
}

/// T-1374: render the per-message reaction rollup.
pub(crate) async fn cmd_channel_reactions_on(
    topic: &str,
    target: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_reactions_on(&envelopes, target);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(ReactionsOnRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No reactions on '{topic}':[{target}].");
        return Ok(());
    }
    println!("Reactions on '{topic}':[{target}]:");
    for r in &rows {
        let senders = r.senders.join(", ");
        println!("  {emoji} ×{count} — {senders}", emoji = r.emoji, count = r.count);
    }
    Ok(())
}

/// T-1375: one row in the edit-stats summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditStatsRow {
    pub target_offset: u64,
    pub target_sender: String,
    pub target_payload: String,
    pub edit_count: u64,
    pub latest_editor: String,
    pub latest_ts_ms: i64,
}

impl EditStatsRow {
    fn to_json(&self) -> Value {
        json!({
            "target_offset": self.target_offset,
            "target_sender": self.target_sender,
            "target_payload": self.target_payload,
            "edit_count": self.edit_count,
            "latest_editor": self.latest_editor,
            "latest_ts_ms": self.latest_ts_ms,
        })
    }
}

/// T-1375: pure helper — topic-wide edit count summary.
///
/// One row per target offset that has at least one non-redacted edit. The
/// topic-wide companion to `compute_edits_of` (T-1366, single-target full
/// history). Completes the audit trio (T-1372 pin-history, T-1373
/// redactions, T-1375 edit-stats) — three pure rollups, one per mutation
/// type.
///
/// Filters:
/// - edit envelopes with non-numeric `metadata.replaces` → ignored
/// - edits whose own offset is redacted → not counted
/// - targets that are themselves redacted → row dropped entirely
///
/// `latest_editor` / `latest_ts_ms` reflect the most recent surviving edit
/// (max ts among non-redacted edits; offset asc tiebreak).
///
/// Sort: edit_count desc, target_offset asc tiebreak. Pure — no I/O.
pub(crate) fn compute_edit_stats(envelopes: &[Value]) -> Vec<EditStatsRow> {
    use std::collections::HashMap;
    let redacted = redacted_offsets(envelopes);
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    // target -> (count, latest_editor, latest_ts, latest_offset for tiebreak)
    let mut by_target: HashMap<u64, (u64, String, i64, u64)> = HashMap::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("edit") {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        let target = match env
            .get("metadata")
            .and_then(|md| md.get("replaces"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(t) => t,
            None => continue,
        };
        if redacted.contains(&target) {
            continue;
        }
        let editor = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let entry = by_target
            .entry(target)
            .or_insert((0, String::new(), i64::MIN, 0));
        entry.0 += 1;
        // Latest by ts; offset asc tiebreak when equal.
        if ts > entry.2 || (ts == entry.2 && off > entry.3) {
            entry.1 = editor;
            entry.2 = ts;
            entry.3 = off;
        }
    }
    let mut rows: Vec<EditStatsRow> = by_target
        .into_iter()
        .filter_map(|(target, (count, latest_editor, latest_ts, _))| {
            let target_env = by_off.get(&target)?;
            Some(EditStatsRow {
                target_offset: target,
                target_sender: target_env
                    .get("sender_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                target_payload: decode_payload_lossy(target_env),
                edit_count: count,
                latest_editor,
                latest_ts_ms: latest_ts,
            })
        })
        .collect();
    rows.sort_by(|a, b| {
        b.edit_count
            .cmp(&a.edit_count)
            .then_with(|| a.target_offset.cmp(&b.target_offset))
    });
    rows
}

/// T-1375: render the edit-stats summary.
pub(crate) async fn cmd_channel_edit_stats(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_edit_stats(&envelopes);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(EditStatsRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No edits on topic '{topic}'.");
        return Ok(());
    }
    println!("Edit-stats for '{topic}':");
    for r in &rows {
        let preview = if r.target_payload.len() > 60 {
            format!("{}…", &r.target_payload[..60])
        } else {
            r.target_payload.clone()
        };
        println!(
            "  [{to}] ×{count} edits (last by {le}) — {ts} {sender}: {preview}",
            to = r.target_offset,
            count = r.edit_count,
            le = r.latest_editor,
            ts = r.latest_ts_ms,
            sender = r.target_sender,
        );
    }
    Ok(())
}

/// T-1376: one row in the canonical-state view of a topic — the Matrix-style
/// render where `m.replace` (edits) have been applied and `m.redaction`-
/// targeted offsets are hidden. This is the "what does this topic say right
/// now" view, distinct from raw subscribe (envelope stream) and from the
/// audit-log views (T-1372 pin-history, T-1373 redactions, T-1375 edit-stats).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StateRow {
    pub offset: u64,
    pub sender_id: String,
    pub payload: String,
    pub is_edited: bool,
    pub edit_count: u64,
    pub latest_edit_ts_ms: i64,
    pub ts_ms: i64,
    pub is_redacted: bool,
}

impl StateRow {
    fn to_json(&self) -> Value {
        json!({
            "offset": self.offset,
            "sender_id": self.sender_id,
            "payload": self.payload,
            "is_edited": self.is_edited,
            "edit_count": self.edit_count,
            "latest_edit_ts_ms": self.latest_edit_ts_ms,
            "ts_ms": self.ts_ms,
            "is_redacted": self.is_redacted,
        })
    }
}

/// T-1376: pure helper — build the canonical state of a topic.
///
/// One row per content message in the topic, in offset-asc order.
/// Filters:
/// - meta envelopes (`UNREAD_META_TYPES`: receipt/reaction/redaction/edit/topic_metadata)
///   are skipped — they are NOT content rows
/// - if `include_redacted` is false, rows whose offset is in the redaction
///   target set are dropped entirely
/// - if `include_redacted` is true, redacted rows surface with payload set
///   to `"[REDACTED]"` and `is_redacted=true`
///
/// Edit collapse: when a content row has at least one non-redacted edit
/// targeting it, payload becomes the latest edit's text (max ts_ms; offset
/// asc tiebreak), `is_edited=true`, `edit_count` reflects the number of
/// surviving (non-redacted) edits, and `latest_edit_ts_ms` is the ts of
/// that latest edit. When no edits, `is_edited=false`, `edit_count=0`,
/// `latest_edit_ts_ms=0`.
///
/// `ts_ms` is always the original content row's timestamp (not the latest
/// edit's). Use `latest_edit_ts_ms` to know when the current text was
/// written.
pub(crate) fn compute_state(envelopes: &[Value], include_redacted: bool) -> Vec<StateRow> {
    use std::collections::HashMap;
    let redacted = redacted_offsets(envelopes);
    // Build per-target latest edit map (only non-redacted edits count).
    // target -> (latest_ts, latest_offset, latest_text, count)
    let mut edits: HashMap<u64, (i64, u64, String, u64)> = HashMap::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("edit") {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        let target = match env
            .get("metadata")
            .and_then(|md| md.get("replaces"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(t) => t,
            None => continue,
        };
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let text = decode_payload_lossy(env);
        let entry = edits
            .entry(target)
            .or_insert((i64::MIN, 0, String::new(), 0));
        entry.3 += 1;
        if ts > entry.0 || (ts == entry.0 && off > entry.1) {
            entry.0 = ts;
            entry.1 = off;
            entry.2 = text;
        }
    }
    let mut rows: Vec<StateRow> = Vec::new();
    for env in envelopes {
        let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if UNREAD_META_TYPES.contains(&mt) {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        let is_red = redacted.contains(&off);
        if is_red && !include_redacted {
            continue;
        }
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let original_payload = decode_payload_lossy(env);
        let (payload, is_edited, edit_count, latest_edit_ts) = if is_red {
            ("[REDACTED]".to_string(), false, 0u64, 0i64)
        } else if let Some((latest_ts, _, text, count)) = edits.get(&off) {
            (text.clone(), true, *count, *latest_ts)
        } else {
            (original_payload, false, 0u64, 0i64)
        };
        rows.push(StateRow {
            offset: off,
            sender_id: sender,
            payload,
            is_edited,
            edit_count,
            latest_edit_ts_ms: latest_edit_ts,
            ts_ms: ts,
            is_redacted: is_red,
        });
    }
    rows.sort_by_key(|r| r.offset);
    rows
}

/// T-1376: render the canonical state view.
pub(crate) async fn cmd_channel_state(
    topic: &str,
    include_redacted: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_state(&envelopes, include_redacted);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(StateRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No content messages on topic '{topic}'.");
        return Ok(());
    }
    println!("Canonical state of '{topic}':");
    for r in &rows {
        let marker = if r.is_redacted {
            " [redacted]"
        } else if r.is_edited {
            " *"
        } else {
            ""
        };
        let edit_suffix = if r.edit_count > 0 {
            format!(" (×{} edits)", r.edit_count)
        } else {
            String::new()
        };
        println!(
            "  [{off}]{marker} {sender}: {payload}{edits}",
            off = r.offset,
            sender = r.sender_id,
            payload = r.payload,
            edits = edit_suffix,
        );
    }
    Ok(())
}

/// T-1379: one row in the per-target reply rollup. Per-target companion
/// to T-1370 `replies-of` (per-sender).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuoteStatsRow {
    pub target_offset: u64,
    pub target_sender: String,
    pub target_payload: String,
    pub reply_count: u64,
    pub distinct_repliers: Vec<String>,
    pub latest_reply_ts_ms: i64,
}

impl QuoteStatsRow {
    fn to_json(&self) -> Value {
        json!({
            "target_offset": self.target_offset,
            "target_sender": self.target_sender,
            "target_payload": self.target_payload,
            "reply_count": self.reply_count,
            "distinct_repliers": self.distinct_repliers,
            "latest_reply_ts_ms": self.latest_reply_ts_ms,
        })
    }
}

/// T-1379: pure helper — per-target reply rollup.
///
/// One row per target offset that has at least one surviving reply.
/// Filters:
/// - replies are envelopes with parseable `metadata.in_reply_to` AND
///   `msg_type != "reaction"` (reactions are not replies)
/// - reply offsets that are themselves redacted are excluded
/// - target offsets that are themselves redacted drop their row entirely
///
/// `distinct_repliers` is sorted-asc (BTreeSet → Vec).
/// Sort: reply_count desc, target_offset asc tiebreak.
pub(crate) fn compute_quote_stats(envelopes: &[Value]) -> Vec<QuoteStatsRow> {
    use std::collections::{BTreeSet, HashMap};
    let redacted = redacted_offsets(envelopes);
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    // target -> (count, BTreeSet<sender>, latest_ts)
    let mut by_target: HashMap<u64, (u64, BTreeSet<String>, i64)> = HashMap::new();
    for env in envelopes {
        let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if mt == "reaction" {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        let parent = match parent_offset_of(env) {
            Some(p) => p,
            None => continue,
        };
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let entry = by_target
            .entry(parent)
            .or_insert((0, BTreeSet::new(), i64::MIN));
        entry.0 += 1;
        entry.1.insert(sender);
        if ts > entry.2 {
            entry.2 = ts;
        }
    }
    let mut rows: Vec<QuoteStatsRow> = by_target
        .into_iter()
        .filter_map(|(target, (count, repliers, latest_ts))| {
            if redacted.contains(&target) {
                return None;
            }
            let target_env = by_off.get(&target)?;
            Some(QuoteStatsRow {
                target_offset: target,
                target_sender: target_env
                    .get("sender_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                target_payload: decode_payload_lossy(target_env),
                reply_count: count,
                distinct_repliers: repliers.into_iter().collect(),
                latest_reply_ts_ms: latest_ts,
            })
        })
        .collect();
    rows.sort_by(|a, b| {
        b.reply_count
            .cmp(&a.reply_count)
            .then_with(|| a.target_offset.cmp(&b.target_offset))
    });
    rows
}

/// T-1379: render the per-target reply rollup.
pub(crate) async fn cmd_channel_quote_stats(
    topic: &str,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_quote_stats(&envelopes);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(QuoteStatsRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No replies on '{topic}'.");
        return Ok(());
    }
    println!("Quote-stats for '{topic}':");
    for r in &rows {
        let preview = if r.target_payload.len() > 60 {
            format!("{}…", &r.target_payload[..60])
        } else {
            r.target_payload.clone()
        };
        let repliers = r.distinct_repliers.join(", ");
        println!(
            "  [{to}] ×{count} replies from {repliers} (last ts={ts}) — {sender}: {preview}",
            to = r.target_offset,
            count = r.reply_count,
            ts = r.latest_reply_ts_ms,
            sender = r.target_sender,
        );
    }
    Ok(())
}

/// T-1378: point-in-time canonical view — Matrix backfill semantics.
/// Reuses `StateRow` shape (T-1376) but applies collapse logic only to
/// envelopes whose ts is `<= as_of_ms`. Edits and redactions later than
/// the cutoff are NOT applied — they hadn't happened yet.
///
/// Filter pipeline:
/// 1. drop envelopes with ts > as_of_ms (didn't exist yet)
/// 2. delegate to `compute_state` on the filtered slice
///
/// `as_of_ms` is in the same scale as `ts_unix_ms` / `ts` envelope fields.
/// Envelopes missing a timestamp are treated as ts=0 (always included
/// when as_of >= 0; never excluded by the upper bound).
pub(crate) fn compute_snapshot(
    envelopes: &[Value],
    as_of_ms: i64,
    include_redacted: bool,
) -> Vec<StateRow> {
    let filtered: Vec<Value> = envelopes
        .iter()
        .filter(|env| {
            let ts = env
                .get("ts_unix_ms")
                .and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ts <= as_of_ms
        })
        .cloned()
        .collect();
    compute_state(&filtered, include_redacted)
}

/// T-1378: render the point-in-time snapshot.
pub(crate) async fn cmd_channel_snapshot(
    topic: &str,
    as_of_ms: i64,
    include_redacted: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_snapshot(&envelopes, as_of_ms, include_redacted);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(StateRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No content messages on '{topic}' as of ts={as_of_ms}.");
        return Ok(());
    }
    println!("Snapshot of '{topic}' as of ts={as_of_ms}:");
    for r in &rows {
        let marker = if r.is_redacted {
            " [redacted]"
        } else if r.is_edited {
            " *"
        } else {
            ""
        };
        let edit_suffix = if r.edit_count > 0 {
            format!(" (×{} edits)", r.edit_count)
        } else {
            String::new()
        };
        println!(
            "  [{off}]{marker} {sender}: {payload}{edits}",
            off = r.offset,
            sender = r.sender_id,
            payload = r.payload,
            edits = edit_suffix,
        );
    }
    Ok(())
}

/// T-1377: one row in the chronological receipt audit log. Each row is one
/// `msg_type=receipt` envelope; distinct from `cmd_channel_receipts`
/// (T-1315 LWW snapshot) and `cmd_channel_ack_status` (T-1361 dashboard
/// with lag).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AckHistoryRow {
    pub receipt_offset: u64,
    pub sender_id: String,
    pub up_to: u64,
    pub ts_ms: i64,
}

impl AckHistoryRow {
    fn to_json(&self) -> Value {
        json!({
            "receipt_offset": self.receipt_offset,
            "sender_id": self.sender_id,
            "up_to": self.up_to,
            "ts_ms": self.ts_ms,
        })
    }
}

/// T-1377: pure helper — chronological receipt audit log.
///
/// One row per `msg_type=receipt` envelope with parseable `metadata.up_to`.
/// When `user_filter` is `Some(uid)`, only rows with `sender_id == uid`
/// survive.
///
/// Sort: ts_ms asc, receipt_offset asc tiebreak.
///
/// Filters: receipts with non-numeric or missing `metadata.up_to` are
/// silently dropped (malformed shape — not actionable as ack-state).
pub(crate) fn compute_ack_history(
    envelopes: &[Value],
    user_filter: Option<&str>,
) -> Vec<AckHistoryRow> {
    let mut rows: Vec<AckHistoryRow> = Vec::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("receipt") {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        if let Some(uid) = user_filter
            && sender != uid
        {
            continue;
        }
        let up_to = match env
            .get("metadata")
            .and_then(|md| md.get("up_to"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(u) => u,
            None => continue,
        };
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        rows.push(AckHistoryRow {
            receipt_offset: off,
            sender_id: sender,
            up_to,
            ts_ms: ts,
        });
    }
    rows.sort_by(|a, b| {
        a.ts_ms
            .cmp(&b.ts_ms)
            .then_with(|| a.receipt_offset.cmp(&b.receipt_offset))
    });
    rows
}

/// T-1377: render the chronological receipt audit log.
pub(crate) async fn cmd_channel_ack_history(
    topic: &str,
    user: Option<&str>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let rows = compute_ack_history(&envelopes, user);
    if json_output {
        let arr: Vec<Value> = rows.iter().map(AckHistoryRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        match user {
            Some(u) => println!("No receipts on '{topic}' from {u}."),
            None => println!("No receipts on '{topic}'."),
        }
        return Ok(());
    }
    match user {
        Some(u) => println!("Ack-history of '{topic}' (user={u}):"),
        None => println!("Ack-history of '{topic}':"),
    }
    for r in &rows {
        println!(
            "  [{off}] ts={ts} {sender} → up_to={up}",
            off = r.receipt_offset,
            ts = r.ts_ms,
            sender = r.sender_id,
            up = r.up_to,
        );
    }
    Ok(())
}

/// T-1366: one row in the edits-of report (either the original or an edit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditRow {
    pub offset: u64,
    pub sender_id: String,
    pub ts_ms: i64,
    pub payload: String,
}

impl EditRow {
    fn to_json(&self) -> Value {
        json!({
            "offset": self.offset,
            "sender_id": self.sender_id,
            "ts_ms": self.ts_ms,
            "payload": self.payload,
        })
    }
}

/// T-1366: edits-of report for one target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditsOfReport {
    pub original: EditRow,
    pub edits: Vec<EditRow>,
}

/// T-1366: pure helper — build the edit history for `target` in `envelopes`.
///
/// Returns `None` when:
/// - the target offset is not present, OR
/// - the target itself is in the redacted-offsets set
///
/// Otherwise returns a report whose `original` is the target's row, and
/// `edits` is the chronological list of `msg_type=edit` envelopes whose
/// `metadata.replaces == target`. Sort: ts_ms asc, edit-offset asc tiebreak.
/// Filters:
/// - non-numeric `metadata.replaces` → ignored
/// - redacted edit offsets → dropped
/// - edits referencing other targets → not in this report
pub(crate) fn compute_edits_of(envelopes: &[Value], target: u64) -> Option<EditsOfReport> {
    let redacted = redacted_offsets(envelopes);
    if redacted.contains(&target) {
        return None;
    }
    let target_env = envelopes
        .iter()
        .find(|e| e.get("offset").and_then(|v| v.as_u64()) == Some(target))?;
    let original = EditRow {
        offset: target,
        sender_id: target_env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
        ts_ms: target_env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| target_env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0),
        payload: decode_payload_lossy(target_env),
    };
    let mut edits: Vec<EditRow> = Vec::new();
    for env in envelopes {
        if env.get("msg_type").and_then(|v| v.as_str()) != Some("edit") {
            continue;
        }
        let off = match env.get("offset").and_then(|v| v.as_u64()) {
            Some(o) => o,
            None => continue,
        };
        if redacted.contains(&off) {
            continue;
        }
        let replaces = env
            .get("metadata")
            .and_then(|md| md.get("replaces"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok());
        if replaces != Some(target) {
            continue;
        }
        edits.push(EditRow {
            offset: off,
            sender_id: env
                .get("sender_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            ts_ms: env
                .get("ts_unix_ms")
                .and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0),
            payload: decode_payload_lossy(env),
        });
    }
    edits.sort_by(|a, b| a.ts_ms.cmp(&b.ts_ms).then_with(|| a.offset.cmp(&b.offset)));
    Some(EditsOfReport { original, edits })
}

/// T-1366: render the edits-of view.
pub(crate) async fn cmd_channel_edits_of(
    topic: &str,
    offset: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let report = match compute_edits_of(&envelopes, offset) {
        Some(r) => r,
        None => anyhow::bail!(
            "Target offset {offset} not found or redacted on topic '{topic}'"
        ),
    };
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "original": report.original.to_json(),
                "edits": report.edits.iter().map(EditRow::to_json).collect::<Vec<_>>(),
            }))?
        );
        return Ok(());
    }
    println!(
        "Edits of offset {} on '{}' ({} edit{}):",
        report.original.offset,
        topic,
        report.edits.len(),
        if report.edits.len() == 1 { "" } else { "s" }
    );
    println!(
        "  [original {} ts={} {}] {}",
        report.original.offset, report.original.ts_ms, report.original.sender_id, report.original.payload
    );
    for e in &report.edits {
        println!(
            "  [edit {} ts={} {}] {}",
            e.offset, e.ts_ms, e.sender_id, e.payload
        );
    }
    Ok(())
}

/// T-1365: one row in the threads index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreadIndexRow {
    pub root_offset: u64,
    pub reply_count: usize,
    pub participants: usize,
    pub last_ts_ms: i64,
    pub root_payload: Option<String>,
}

impl ThreadIndexRow {
    fn to_json(&self) -> Value {
        json!({
            "root_offset": self.root_offset,
            "reply_count": self.reply_count,
            "participants": self.participants,
            "last_ts_ms": self.last_ts_ms,
            "root_payload": self.root_payload,
        })
    }
}

/// T-1365: pure helper — index every thread in a topic.
///
/// A "thread root" is any envelope that another envelope refers to via
/// `metadata.in_reply_to`. The index includes one row per root with:
/// - `reply_count` — non-redacted descendants (transitive)
/// - `participants` — distinct sender_ids in the thread including root sender
/// - `last_ts_ms` — max ts across the thread (root + descendants)
/// - `root_payload` — payload preview of the root envelope (None if redacted/missing)
///
/// Filtering rules:
/// - root that is redacted → row dropped entirely
/// - replies that are redacted → don't count toward reply_count or participants
/// - thread with zero non-redacted replies → row dropped
/// - non-numeric `in_reply_to` → reply ignored
///
/// Sort: by `last_ts_ms` descending (most recently active first); offset asc tiebreak.
/// Pure — no I/O.
pub(crate) fn compute_threads_index(envelopes: &[Value]) -> Vec<ThreadIndexRow> {
    use std::collections::{HashMap, HashSet};
    let redacted = redacted_offsets(envelopes);
    let mut by_off: HashMap<u64, &Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    // parent → list of (reply_offset, reply_sender, reply_ts) for non-redacted replies
    let mut children: HashMap<u64, Vec<(u64, String, i64)>> = HashMap::new();
    for env in envelopes {
        let Some(off) = env.get("offset").and_then(|v| v.as_u64()) else { continue };
        if redacted.contains(&off) {
            continue;
        }
        let Some(parent) = parent_offset_of(env) else { continue };
        let sender = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ts = env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        children.entry(parent).or_default().push((off, sender, ts));
    }
    let mut rows: Vec<ThreadIndexRow> = Vec::new();
    for (root_off, _) in children.iter().filter(|(off, _)| !redacted.contains(off)) {
        let root_env = match by_off.get(root_off) {
            Some(e) => *e,
            None => continue,
        };
        // BFS gather all descendants (transitive)
        let mut stack: Vec<u64> = vec![*root_off];
        let mut seen: HashSet<u64> = HashSet::new();
        seen.insert(*root_off);
        let mut reply_count: usize = 0;
        let mut participants: HashSet<String> = HashSet::new();
        let root_sender = root_env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if !root_sender.is_empty() {
            participants.insert(root_sender);
        }
        let mut last_ts: i64 = root_env
            .get("ts_unix_ms")
            .and_then(|v| v.as_i64())
            .or_else(|| root_env.get("ts").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        while let Some(parent) = stack.pop() {
            if let Some(kids) = children.get(&parent) {
                for (k_off, k_sender, k_ts) in kids {
                    if !seen.insert(*k_off) {
                        continue;
                    }
                    reply_count += 1;
                    if !k_sender.is_empty() {
                        participants.insert(k_sender.clone());
                    }
                    if *k_ts > last_ts {
                        last_ts = *k_ts;
                    }
                    stack.push(*k_off);
                }
            }
        }
        if reply_count == 0 {
            continue;
        }
        rows.push(ThreadIndexRow {
            root_offset: *root_off,
            reply_count,
            participants: participants.len(),
            last_ts_ms: last_ts,
            root_payload: Some(decode_payload_lossy(root_env)),
        });
    }
    rows.sort_by(|a, b| {
        b.last_ts_ms
            .cmp(&a.last_ts_ms)
            .then_with(|| a.root_offset.cmp(&b.root_offset))
    });
    rows
}

/// T-1365: render the threads index.
pub(crate) async fn cmd_channel_threads(
    topic: &str,
    top: Option<usize>,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    let mut rows = compute_threads_index(&envelopes);
    if let Some(n) = top {
        rows.truncate(n);
    }
    if json_output {
        let arr: Vec<Value> = rows.iter().map(ThreadIndexRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No threads on topic '{topic}'.");
        return Ok(());
    }
    println!(
        "Threads on '{topic}' ({n} root{s}):",
        n = rows.len(),
        s = if rows.len() == 1 { "" } else { "s" }
    );
    for r in &rows {
        let preview = r.root_payload.as_deref().unwrap_or("(no payload)");
        let preview = if preview.len() > 60 {
            format!("{}…", &preview[..60])
        } else {
            preview.to_string()
        };
        println!(
            "  [{root}] replies={rc} participants={p} last_ts={ts}: {preview}",
            root = r.root_offset,
            rc = r.reply_count,
            p = r.participants,
            ts = r.last_ts_ms,
        );
    }
    Ok(())
}

/// T-1361: one row in the read-receipt dashboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AckStatusRow {
    pub sender_id: String,
    /// `None` when the sender posted content but never emitted a receipt.
    pub up_to: Option<u64>,
    pub latest: u64,
    pub lag: u64,
    pub receipt_ts: i64,
}

impl AckStatusRow {
    fn to_json(&self) -> Value {
        json!({
            "sender_id": self.sender_id,
            "up_to": self.up_to,
            "latest": self.latest,
            "lag": self.lag,
            "ts": self.receipt_ts,
        })
    }
}

/// T-1361: pure helper — compute the per-sender ack-status rows.
///
/// Inputs:
/// - `envelopes`: full topic walk (used to extract member set + latest offset)
/// - `receipts`: latest receipt per sender, as `(sender_id -> (up_to, ts))`
///
/// Rows:
/// - Senders with a receipt: `up_to = Some(U)`, `lag = max(0, latest - U)`
/// - Senders who posted content but no receipt: `up_to = None`, `lag = latest + 1`
///
/// Sorted by lag descending; ties break on sender_id ascending. Pure — no I/O.
pub(crate) fn compute_ack_status(
    envelopes: &[Value],
    receipts: &std::collections::HashMap<String, (u64, i64)>,
    latest_offset: u64,
) -> Vec<AckStatusRow> {
    use std::collections::HashSet;
    // Members = anyone who posted any non-meta envelope. Use a permissive
    // definition (anyone with sender_id) so the dashboard surfaces lurkers
    // who reacted but never wrote.
    let mut members: HashSet<String> = HashSet::new();
    for env in envelopes {
        if let Some(s) = env.get("sender_id").and_then(|v| v.as_str()) {
            members.insert(s.to_string());
        }
    }
    // Always include receipt-only senders too.
    for sender in receipts.keys() {
        members.insert(sender.clone());
    }
    let mut rows: Vec<AckStatusRow> = members
        .into_iter()
        .map(|sender_id| match receipts.get(&sender_id) {
            Some((up_to, ts)) => {
                let lag = latest_offset.saturating_sub(*up_to);
                AckStatusRow {
                    sender_id,
                    up_to: Some(*up_to),
                    latest: latest_offset,
                    lag,
                    receipt_ts: *ts,
                }
            }
            None => AckStatusRow {
                sender_id,
                up_to: None,
                latest: latest_offset,
                lag: latest_offset + 1,
                receipt_ts: 0,
            },
        })
        .collect();
    rows.sort_by(|a, b| {
        b.lag
            .cmp(&a.lag)
            .then_with(|| a.sender_id.cmp(&b.sender_id))
    });
    rows
}

/// T-1361: render the ack-status dashboard.
pub(crate) async fn cmd_channel_ack_status(
    topic: &str,
    pending_only: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    if envelopes.is_empty() {
        println!("Topic '{topic}' is empty.");
        return Ok(());
    }
    let latest_offset = envelopes
        .iter()
        .filter_map(|e| e.get("offset").and_then(|v| v.as_u64()))
        .max()
        .unwrap_or(0);

    // Latest-receipt per sender via channel.receipts RPC (with envelope-walk
    // fallback for old hubs).
    use std::collections::HashMap;
    let mut receipts: HashMap<String, (u64, i64)> = HashMap::new();
    let server_resp = client::rpc_call(
        &sock,
        method::CHANNEL_RECEIPTS,
        json!({"topic": topic}),
    )
    .await
    .context("Hub rpc_call (channel.receipts) failed")?;
    let mut fallback = false;
    match server_resp {
        termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
            for entry in r.result["receipts"].as_array().cloned().unwrap_or_default() {
                let sender = match entry.get("sender_id").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                let up_to = entry.get("up_to").and_then(|v| v.as_u64()).unwrap_or(0);
                let ts = entry
                    .get("ts_unix_ms")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                receipts.insert(sender, (up_to, ts));
            }
        }
        termlink_protocol::jsonrpc::RpcResponse::Error(e) if e.error.code == -32601 => {
            fallback = true;
        }
        termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
            return Err(anyhow!(
                "Hub returned error for channel.receipts: {} {}",
                e.error.code,
                e.error.message
            ));
        }
    }
    if fallback {
        // Walk the topic for receipt envelopes.
        for env in &envelopes {
            if env.get("msg_type").and_then(|v| v.as_str()) != Some("receipt") {
                continue;
            }
            let sender = match env.get("sender_id").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let up_to = match env
                .get("metadata")
                .and_then(|md| md.get("up_to"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
            {
                Some(v) => v,
                None => continue,
            };
            let ts = env
                .get("ts_unix_ms")
                .and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            match receipts.get(&sender) {
                Some((_, prev_ts)) if *prev_ts > ts => {}
                _ => {
                    receipts.insert(sender, (up_to, ts));
                }
            }
        }
    }

    let mut rows = compute_ack_status(&envelopes, &receipts, latest_offset);
    if pending_only {
        rows.retain(|r| r.lag > 0);
    }

    if json_output {
        let arr: Vec<Value> = rows.iter().map(AckStatusRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        if pending_only {
            println!("All members are caught up on '{topic}'.");
        } else {
            println!("No members on '{topic}'.");
        }
        return Ok(());
    }
    println!("Ack status on '{topic}' (latest offset = {latest_offset}):");
    for r in &rows {
        let ack = match r.up_to {
            Some(u) => u.to_string(),
            None => "-".to_string(),
        };
        println!(
            "  {sender}  ack={ack}  lag={lag}  ts={ts}",
            sender = r.sender_id,
            lag = r.lag,
            ts = r.receipt_ts,
        );
    }
    Ok(())
}

/// T-1358: per-topic unread row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnreadRow {
    pub topic: String,
    pub cursor: u64,
    pub latest: u64,
    pub unread: u64,
}

impl UnreadRow {
    fn to_json(&self) -> Value {
        json!({
            "topic": self.topic,
            "cursor": self.cursor,
            "latest": self.latest,
            "unread": self.unread,
        })
    }
}

/// T-1358: pure helper — given a list of `(topic, cursor)` from the local
/// cursor store and a `topic_counts` map (from channel.list), produce
/// rows for topics where new envelopes have arrived since the cursor.
///
/// Rules:
/// - Topic missing from `topic_counts`: silently dropped (topic was deleted
///   on the hub or doesn't exist there)
/// - `count == 0`: latest is undefined; row dropped
/// - `cursor + 1 >= count`: caller is at-or-ahead; row dropped (no unread)
/// - Otherwise: `latest = count - 1`, `unread = count - 1 - cursor`
///
/// Result is sorted by descending `unread` (highest first); ties break on
/// topic ascending for determinism. Pure — no I/O.
pub(crate) fn compute_unread_rows(
    cursors: &[(String, u64)],
    topic_counts: &std::collections::HashMap<String, u64>,
) -> Vec<UnreadRow> {
    let mut rows: Vec<UnreadRow> = Vec::new();
    for (topic, cursor) in cursors {
        let count = match topic_counts.get(topic) {
            Some(c) => *c,
            None => continue,
        };
        if count == 0 {
            continue;
        }
        let latest = count - 1;
        if *cursor >= latest {
            continue;
        }
        let unread = latest - cursor;
        rows.push(UnreadRow {
            topic: topic.clone(),
            cursor: *cursor,
            latest,
            unread,
        });
    }
    rows.sort_by(|a, b| b.unread.cmp(&a.unread).then_with(|| a.topic.cmp(&b.topic)));
    rows
}

/// T-1358: render the cross-topic unread inbox.
pub(crate) async fn cmd_channel_inbox(
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let identity = load_identity_or_create()
        .context("Loading identity for unread scope")?;
    let fp = identity.fingerprint().to_string();
    let cursors = cursor_store::list_for_fingerprint(&fp)?;

    if cursors.is_empty() {
        if json_output {
            println!("[]");
        } else {
            println!("No cursors recorded yet — use `subscribe --resume` to start tracking topics.");
        }
        return Ok(());
    }

    let sock = hub_socket(hub)?;
    let resp = client::rpc_call(&sock, method::CHANNEL_LIST, json!({}))
        .await
        .context("Hub rpc_call (channel.list) failed")?;
    let result = client::unwrap_result(resp)
        .map_err(|e| anyhow!("Hub returned error for channel.list: {e}"))?;
    let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    if let Some(arr) = result["topics"].as_array() {
        for entry in arr {
            let name = match entry.get("name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let count = entry.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            counts.insert(name, count);
        }
    }
    let rows = compute_unread_rows(&cursors, &counts);

    if json_output {
        let arr: Vec<Value> = rows.iter().map(UnreadRow::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&Value::Array(arr))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("No unread topics.");
        return Ok(());
    }
    println!("{} topic(s) with unread content:", rows.len());
    for r in &rows {
        println!(
            "  {topic} — {unread} unread (latest={latest}, cursor={cursor})",
            topic = r.topic,
            unread = r.unread,
            latest = r.latest,
            cursor = r.cursor,
        );
    }
    Ok(())
}

/// T-1344: pure helper — extract `metadata.in_reply_to` from an envelope and
/// parse it as a u64. Returns `None` when the field is absent or non-numeric.
/// Reactions and reply posts both carry this key (T-1313 / T-1314 contracts).
pub(crate) fn parent_offset_of(env: &Value) -> Option<u64> {
    env.get("metadata")
        .and_then(|md| md.get("in_reply_to"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())
}

/// T-1344: render an envelope inline with its parent quoted on a preceding
/// line. Walks the topic once, locates the envelope at `offset`, and looks
/// up the parent via `metadata.in_reply_to`. Errors when the offset itself
/// is missing; renders alone with a "no parent" note when the env is not a
/// reply or the parent reference cannot be resolved.
pub(crate) async fn cmd_channel_quote(
    topic: &str,
    offset: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let sock = hub_socket(hub)?;
    let envelopes = walk_topic_full(&sock, topic).await?;
    use std::collections::HashMap;
    let mut by_off: HashMap<u64, Value> = HashMap::with_capacity(envelopes.len());
    for env in envelopes {
        if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
            by_off.insert(off, env);
        }
    }
    let child = by_off
        .get(&offset)
        .ok_or_else(|| anyhow!("Topic '{topic}' has no envelope at offset {offset}"))?
        .clone();
    let parent = parent_offset_of(&child).and_then(|p| by_off.get(&p).cloned());

    if json_output {
        let render = |m: &Value| -> Value {
            let off = m.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
            let sender = m.get("sender_id").and_then(|v| v.as_str()).unwrap_or("?");
            let msg_type = m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("?");
            let ts = m
                .get("ts_unix_ms")
                .and_then(|v| v.as_i64())
                .or_else(|| m.get("ts").and_then(|v| v.as_i64()));
            json!({
                "offset": off,
                "sender_id": sender,
                "msg_type": msg_type,
                "ts": ts,
                "payload": decode_payload_lossy(m),
            })
        };
        let parent_json = parent.as_ref().map(render).unwrap_or(Value::Null);
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "topic": topic,
                "child": render(&child),
                "parent": parent_json,
            }))?
        );
        return Ok(());
    }

    let render_line = |m: &Value, prefix: &str| {
        let off = m.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
        let sender = m.get("sender_id").and_then(|v| v.as_str()).unwrap_or("?");
        let msg_type = m.get("msg_type").and_then(|v| v.as_str()).unwrap_or("?");
        let payload = decode_payload_lossy(m);
        println!("{prefix}[{off}] {sender} {msg_type}: {payload}");
    };
    match parent {
        Some(p) => {
            render_line(&p, "> ");
            render_line(&child, "");
        }
        None => {
            // Two cases:
            //   1. envelope has no in_reply_to → not a reply, render alone
            //   2. has in_reply_to but parent missing from topic → render with note
            match parent_offset_of(&child) {
                Some(p) => println!("> [{p} ?] (parent not in topic)"),
                None => println!("(no parent — not a reply)"),
            }
            render_line(&child, "");
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
/// T-1343: pure helper — should an envelope be emitted given the optional
/// `--since <ms>` filter? Returns true when no filter is set, when the
/// envelope carries a ts >= since, or when the filter is set but the
/// envelope has no usable ts (we keep ts-less envelopes; defensive — they
/// might be meta lines like edit/redaction markers without ts).
pub(crate) fn should_emit_for_since(env: &Value, since: Option<i64>) -> bool {
    let Some(threshold) = since else { return true };
    let ts_opt = env
        .get("ts_unix_ms")
        .and_then(|v| v.as_i64())
        .or_else(|| env.get("ts").and_then(|v| v.as_i64()));
    match ts_opt {
        Some(ts) => ts >= threshold,
        None => true,
    }
}

/// T-1352: pure helper — closing pair to `should_emit_for_since`. Returns
/// true when no filter is set, when the envelope carries a ts <= until,
/// or when the filter is set but the envelope has no usable ts (defensive
/// keep — same rationale as --since). Together they define an inclusive
/// `[since, until]` window when both are passed.
pub(crate) fn should_emit_for_until(env: &Value, until: Option<i64>) -> bool {
    let Some(threshold) = until else { return true };
    let ts_opt = env
        .get("ts_unix_ms")
        .and_then(|v| v.as_i64())
        .or_else(|| env.get("ts").and_then(|v| v.as_i64()));
    match ts_opt {
        Some(ts) => ts <= threshold,
        None => true,
    }
}

/// T-1349: pure helper — extract forward-provenance metadata from an envelope.
/// Returns `Some((src_topic, offset, orig_sender))` when both
/// `metadata.forwarded_from` (formatted `<topic>:<offset>`) and
/// `metadata.forwarded_sender` are present and parsable. Defensive: if
/// `forwarded_sender` is absent, returns `None` (we want both fields to
/// trust the provenance). Topics may contain colons (e.g. `dm:a:b`) so we
/// split on the LAST colon to get offset.
pub(crate) fn extract_forward(env: &Value) -> Option<(String, u64, String)> {
    let md = env.get("metadata")?;
    let from = md.get("forwarded_from").and_then(|v| v.as_str())?;
    let sender = md
        .get("forwarded_sender")
        .and_then(|v| v.as_str())?
        .to_string();
    let (topic, off_str) = from.rsplit_once(':')?;
    let off = off_str.parse::<u64>().ok()?;
    Some((topic.to_string(), off, sender))
}

/// T-1347: pure helper — does `sender` match the comma-separated allowlist?
/// Strict equality (comma-split + trim). Empty list returns `false` (no
/// allowed senders means nothing matches). Empty sender returns `false`.
/// Case-sensitive — sender_ids are fingerprint hashes where case matters.
pub(crate) fn sender_in_csv(sender: &str, csv: &str) -> bool {
    if sender.is_empty() {
        return false;
    }
    let parts: Vec<&str> = csv
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return false;
    }
    parts.contains(&sender)
}

/// T-1346: pure helper — return the last `n` items from `items` (or all
/// when `n >= items.len()`, or empty when `n == 0`). When `tail` is `None`,
/// returns a clone of all items unchanged. Used by `cmd_channel_subscribe`
/// to slice rendered envelope outputs to the last N before printing.
pub(crate) fn tail_slice<T: Clone>(items: &[T], tail: Option<usize>) -> Vec<T> {
    match tail {
        None => items.to_vec(),
        Some(0) => Vec::new(),
        Some(n) if n >= items.len() => items.to_vec(),
        Some(n) => items[items.len() - n..].to_vec(),
    }
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
    since: Option<i64>,
    until: Option<i64>,
    show_parent: bool,
    tail: Option<usize>,
    senders_filter: Option<&str>,
    show_forwards: bool,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    use std::fmt::Write as _;
    let tail_mode = tail.is_some();
    // T-1346: when --tail is set, accumulate per-envelope rendered output
    // here; after the polling loop completes (only reachable when !follow,
    // which conflicts_with --tail), emit the last N. Each entry is the
    // complete output for one envelope (1+ lines, with trailing newlines).
    let mut env_outputs: Vec<String> = Vec::new();
    let sock = hub_socket(hub)?;
    // T-1344: when --show-parent is on, seed an offset-keyed cache by walking
    // the topic once before the streaming loop. Live envelopes seen during
    // --follow are added to the cache as they arrive (see emission loop).
    // Cache miss for a known parent reference renders a "[parent ?]" stub
    // rather than blocking — better degraded UX than a hard error.
    let mut parent_cache: std::collections::HashMap<u64, Value> =
        std::collections::HashMap::new();
    if show_parent {
        let seed = walk_topic_full(&sock, topic).await.unwrap_or_default();
        for env in seed {
            if let Some(off) = env.get("offset").and_then(|v| v.as_u64()) {
                parent_cache.insert(off, env);
            }
        }
    }
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
        // T-1346: per-envelope buffered emit. `flush!()` sends the buffered
        // output for one envelope to either stdout (immediate) or env_outputs
        // (when --tail is set). Empty buffer is a no-op so we can flush
        // unconditionally before every `continue`.
        for m in &msgs {
            let mut env_out = String::new();
            macro_rules! flush {
                () => {
                    if !env_out.is_empty() {
                        if tail_mode {
                            env_outputs.push(std::mem::take(&mut env_out));
                        } else {
                            print!("{}", env_out);
                            env_out.clear();
                        }
                    }
                };
            }
            // T-1343 / T-1352: render-time `[since, until]` window. Pure
            // drop — pagination and aggregation passes already ran. Affects
            // both JSON-lines and human output identically.
            if !should_emit_for_since(m, since) {
                continue;
            }
            if !should_emit_for_until(m, until) {
                continue;
            }
            // T-1347: render-time --senders <csv> filter. Same shape as
            // --since: applied to both JSON and human output, after all
            // reaction/edit/redaction aggregation passes have already run
            // on the full set.
            if let Some(csv) = senders_filter {
                let s = m.get("sender_id").and_then(|v| v.as_str()).unwrap_or("");
                if !sender_in_csv(s, csv) {
                    continue;
                }
            }
            // T-1344: keep the parent cache fresh as new envelopes stream in
            // (so a future reply to this offset finds it without a re-walk).
            if show_parent
                && let Some(off) = m.get("offset").and_then(|v| v.as_u64())
            {
                parent_cache.entry(off).or_insert_with(|| m.clone());
            }
            if json_output {
                if show_parent {
                    let parent_off = parent_offset_of(m);
                    let parent_val = parent_off
                        .and_then(|off| parent_cache.get(&off))
                        .cloned()
                        .map(|v| v as Value)
                        .unwrap_or(Value::Null);
                    let mut wrapper = m.clone();
                    if let Some(obj) = wrapper.as_object_mut() {
                        obj.insert("parent".to_string(), parent_val);
                    }
                    let _ = writeln!(env_out, "{}", serde_json::to_string(&wrapper)?);
                } else {
                    let _ = writeln!(env_out, "{}", serde_json::to_string(m)?);
                }
                flush!();
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
                let _ = writeln!(
                    env_out,
                    "[{off} redact] {sender} → offset {target}{reason}",
                    sender = r.sender,
                    target = r.target,
                );
                flush!();
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
            // T-1344: human render — emit a `> [parent] sender msg_type: payload`
            // quote line BEFORE the main line when --show-parent and this env is
            // a reply. Placement AFTER all filter checks so we never emit a
            // dangling quote for an envelope that is then suppressed (e.g. a
            // reaction under --reactions, or a non-matching mention filter).
            if show_parent
                && let Some(parent_off) = parent_offset_of(m)
            {
                match parent_cache.get(&parent_off) {
                    Some(p) => {
                        let psender = p.get("sender_id").and_then(|v| v.as_str()).unwrap_or("?");
                        let pmsg = p.get("msg_type").and_then(|v| v.as_str()).unwrap_or("?");
                        let pp = decode_payload_lossy(p);
                        let _ = writeln!(env_out, "> [{parent_off}] {psender} {pmsg}: {pp}");
                    }
                    None => {
                        let _ = writeln!(env_out, "> [{parent_off} ?] (parent not in cache)");
                    }
                }
            }
            // T-1349: forward provenance prefix — emit `[fwd from <src>:<off>
            // by <orig_sender>]` BEFORE the main render line when
            // --show-forwards and this env carries forwarded_from metadata.
            // Placed alongside show_parent so both are visible together when
            // forwarding a reply.
            if show_forwards
                && let Some((src, off, orig)) = extract_forward(m)
            {
                let _ = writeln!(env_out, "[fwd from {src}:{off} by {orig}]");
            }
            // T-1314: reaction envelopes get a compact non-aggregated render
            // (msg_type prefix dropped; the `react` tag in the bracket is the cue).
            if msg_type == "reaction" {
                let _ = writeln!(
                    env_out,
                    "[{offset}{reply_marker}{mention_marker} react] {sender} {payload_str}",
                );
            } else {
                let _ = writeln!(
                    env_out,
                    "[{offset}{reply_marker}{mention_marker}] {sender} {msg_type}: {payload_str}{edited_marker}",
                );
                if aggregate_reactions {
                    let summary = reactions_summary(&reactions_by_parent, offset, by_sender);
                    if !summary.is_empty() {
                        let _ = writeln!(env_out, "    └─ reactions: {summary}");
                    }
                    printed_parents.insert(offset);
                }
            }
            flush!();
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
            // T-1346: when --tail is set, emit only the last N collected
            // envelope outputs. (--tail conflicts_with --follow at the
            // clap level, so we only ever reach the slicing path in
            // single-shot mode.)
            if tail_mode {
                let kept = tail_slice(&env_outputs, tail);
                for chunk in kept {
                    print!("{}", chunk);
                }
            }
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

/// T-1339: `channel mentions [--for <id>]` — cross-topic @-mentions inbox.
/// Resolves the target id (defaults to caller's identity), enumerates
/// every topic via channel.list (optionally filtered by `prefix`), walks
/// each, and accumulates content envelopes whose mentions CSV matches
/// the target via `mentions_match` (T-1325/T-1333 wildcard semantics —
/// `*` in CSV = @room). Skips meta envelopes (UNREAD_META_TYPES). Read-
/// only. Output: human form groups by topic; `--json` emits a flat
/// array suitable for piping to jq.
pub(crate) async fn cmd_channel_mentions(
    target: Option<&str>,
    prefix: Option<&str>,
    limit: u64,
    hub: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let resolved_target: String = match target {
        Some(s) => s.to_string(),
        None => {
            let id = load_identity_or_create()
                .context("Loading identity for mentions target")?;
            id.fingerprint().to_string()
        }
    };

    let sock = hub_socket(hub)?;
    let params = match prefix {
        Some(p) => json!({"prefix": p}),
        None => json!({}),
    };
    let resp = client::rpc_call(&sock, method::CHANNEL_LIST, params)
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

    let mut hits: Vec<Value> = Vec::new();
    'topic_loop: for topic in &topics {
        let envelopes = walk_topic_full(&sock, topic).await?;
        for env in &envelopes {
            let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
            if UNREAD_META_TYPES.contains(&mt) {
                continue;
            }
            let csv = match extract_mentions(env) {
                Some(s) => s,
                None => continue,
            };
            if !mentions_match(&csv, &resolved_target) {
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
            let payload = decode_payload_lossy(env);
            hits.push(json!({
                "topic": topic,
                "offset": offset,
                "sender_id": sender,
                "ts": ts,
                "msg_type": mt,
                "payload": payload,
                "mentions": csv,
            }));
            if limit > 0 && hits.len() as u64 >= limit {
                break 'topic_loop;
            }
        }
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&Value::Array(hits))?);
        return Ok(());
    }
    if hits.is_empty() {
        println!("No mentions for '{resolved_target}'.");
        return Ok(());
    }
    // Group by topic for the human view.
    let mut last_topic: Option<&str> = None;
    for h in &hits {
        let topic = h["topic"].as_str().unwrap_or("?");
        if last_topic != Some(topic) {
            if last_topic.is_some() {
                println!();
            }
            println!("== {topic} ==");
            last_topic = Some(topic);
        }
        let off = h["offset"].as_u64().unwrap_or(0);
        let sender = h["sender_id"].as_str().unwrap_or("?");
        let payload = h["payload"].as_str().unwrap_or("");
        println!("  [{off}] {sender}: {payload}");
    }
    Ok(())
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

    // ---- T-1343: should_emit_for_since --------------------------------

    #[test]
    fn should_emit_for_since_passes_when_no_filter() {
        let env = json!({"offset": 0, "ts": 100});
        assert!(should_emit_for_since(&env, None));
    }

    #[test]
    fn should_emit_for_since_emits_at_or_above_threshold() {
        let env = json!({"offset": 0, "ts": 200});
        assert!(should_emit_for_since(&env, Some(100)));
        assert!(should_emit_for_since(&env, Some(200))); // >= boundary
        assert!(!should_emit_for_since(&env, Some(201)));
    }

    #[test]
    fn should_emit_for_since_accepts_ts_unix_ms() {
        let env = json!({"offset": 0, "ts_unix_ms": 200});
        assert!(should_emit_for_since(&env, Some(150)));
        assert!(!should_emit_for_since(&env, Some(250)));
    }

    #[test]
    fn should_emit_for_since_keeps_envelope_with_no_ts() {
        // Defensive: ts-less envelopes (e.g. legacy meta) are kept rather
        // than silently dropped. Operator can use other filters.
        let env = json!({"offset": 0, "msg_type": "edit"});
        assert!(should_emit_for_since(&env, Some(100)));
    }

    // ---- T-1341: summarize_members ------------------------------------

    #[test]
    fn summarize_members_groups_by_sender_with_first_last_ts() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat", "sender_id": "alice", "ts": 100}),
            json!({"offset": 1, "msg_type": "chat", "sender_id": "bob",   "ts": 200}),
            json!({"offset": 2, "msg_type": "chat", "sender_id": "alice", "ts": 300}),
            json!({"offset": 3, "msg_type": "chat", "sender_id": "bob",   "ts": 400}),
            json!({"offset": 4, "msg_type": "chat", "sender_id": "carol", "ts": 250}),
        ];
        let rows = summarize_members(&msgs, false);
        // last_ts desc: bob 400, alice 300, carol 250
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].sender_id, "bob");
        assert_eq!(rows[0].posts, 2);
        assert_eq!(rows[0].first_ts, Some(200));
        assert_eq!(rows[0].last_ts, Some(400));
        assert_eq!(rows[1].sender_id, "alice");
        assert_eq!(rows[1].posts, 2);
        assert_eq!(rows[1].first_ts, Some(100));
        assert_eq!(rows[1].last_ts, Some(300));
        assert_eq!(rows[2].sender_id, "carol");
        assert_eq!(rows[2].posts, 1);
    }

    #[test]
    fn summarize_members_skips_meta_by_default() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat",     "sender_id": "alice", "ts": 100}),
            json!({"offset": 1, "msg_type": "reaction", "sender_id": "bob",   "ts": 200}),
            json!({"offset": 2, "msg_type": "receipt",  "sender_id": "alice", "ts": 300}),
        ];
        let rows = summarize_members(&msgs, false);
        // Only alice's chat counts; bob's reaction + alice's receipt skipped.
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sender_id, "alice");
        assert_eq!(rows[0].posts, 1);
        assert_eq!(rows[0].last_ts, Some(100));
    }

    #[test]
    fn summarize_members_include_meta_counts_everything() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat",     "sender_id": "alice", "ts": 100}),
            json!({"offset": 1, "msg_type": "reaction", "sender_id": "bob",   "ts": 200}),
            json!({"offset": 2, "msg_type": "receipt",  "sender_id": "alice", "ts": 300}),
        ];
        let rows = summarize_members(&msgs, true);
        assert_eq!(rows.len(), 2);
        // bob's last_ts 200; alice's last_ts 300 → alice first.
        assert_eq!(rows[0].sender_id, "alice");
        assert_eq!(rows[0].posts, 2);
        assert_eq!(rows[1].sender_id, "bob");
        assert_eq!(rows[1].posts, 1);
    }

    #[test]
    fn summarize_members_skips_empty_sender_id() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat", "sender_id": "", "ts": 100}),
            json!({"offset": 1, "msg_type": "chat", "ts": 200}),
            json!({"offset": 2, "msg_type": "chat", "sender_id": "alice", "ts": 300}),
        ];
        let rows = summarize_members(&msgs, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sender_id, "alice");
    }

    #[test]
    fn summarize_members_handles_no_ts() {
        let msgs = vec![
            json!({"offset": 0, "msg_type": "chat", "sender_id": "alice"}),  // no ts
            json!({"offset": 1, "msg_type": "chat", "sender_id": "bob", "ts": 100}),
        ];
        let rows = summarize_members(&msgs, false);
        assert_eq!(rows.len(), 2);
        // bob has last_ts=100, alice has none → bob first by sort order
        assert_eq!(rows[0].sender_id, "bob");
        assert_eq!(rows[1].sender_id, "alice");
        assert_eq!(rows[0].first_ts, Some(100));
        assert_eq!(rows[1].first_ts, None);
    }

    #[test]
    fn member_row_to_json_round_trips() {
        let r = MemberRow {
            sender_id: "alice".to_string(),
            posts: 3,
            first_ts: Some(100),
            last_ts: Some(300),
        };
        let v = r.to_json();
        assert_eq!(v["sender_id"], "alice");
        assert_eq!(v["posts"], 3);
        assert_eq!(v["first_ts"], 100);
        assert_eq!(v["last_ts"], 300);
    }

    // ---- T-1340: build_ancestors --------------------------------------

    fn idx(records: &[(u64, Option<&str>)]) -> std::collections::HashMap<u64, Value> {
        let mut m = std::collections::HashMap::new();
        for (off, parent) in records {
            let env = match parent {
                Some(p) => json!({
                    "offset": off,
                    "metadata": {"in_reply_to": p},
                }),
                None => json!({"offset": off}),
            };
            m.insert(*off, env);
        }
        m
    }

    #[test]
    fn build_ancestors_linear_chain_root_to_leaf() {
        // 0 ← 1 ← 2 ← 3
        let by_off = idx(&[
            (0, None),
            (1, Some("0")),
            (2, Some("1")),
            (3, Some("2")),
        ]);
        assert_eq!(build_ancestors(&by_off, 3), vec![0, 1, 2, 3]);
    }

    #[test]
    fn build_ancestors_root_only_returns_just_self() {
        let by_off = idx(&[(0, None)]);
        assert_eq!(build_ancestors(&by_off, 0), vec![0]);
    }

    #[test]
    fn build_ancestors_missing_leaf_returns_empty() {
        let by_off = idx(&[(0, None), (1, Some("0"))]);
        assert_eq!(build_ancestors(&by_off, 99), Vec::<u64>::new());
    }

    #[test]
    fn build_ancestors_terminates_when_parent_missing() {
        // 5 → 7 (parent), but 7 is not in the index. Walk yields just [5].
        let by_off = idx(&[(5, Some("7"))]);
        assert_eq!(build_ancestors(&by_off, 5), vec![5]);
    }

    #[test]
    fn build_ancestors_breaks_cycle() {
        // 0 ← 1 ← 0 (cycle). Walk must terminate after seeing 0 twice.
        let by_off = idx(&[
            (0, Some("1")), // 0 → 1
            (1, Some("0")), // 1 → 0 (creates cycle)
        ]);
        let chain = build_ancestors(&by_off, 0);
        // Cycle is detected; chain has both nodes exactly once.
        assert_eq!(chain.len(), 2);
        let unique: std::collections::HashSet<&u64> = chain.iter().collect();
        assert_eq!(unique.len(), 2);
    }

    #[test]
    fn build_ancestors_skips_non_numeric_parent() {
        // metadata.in_reply_to = "not-a-number" → terminate at offset 5.
        let mut by_off = std::collections::HashMap::new();
        by_off.insert(5, json!({
            "offset": 5,
            "metadata": {"in_reply_to": "not-a-number"},
        }));
        assert_eq!(build_ancestors(&by_off, 5), vec![5]);
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

    // T-1344: parent_offset_of helper
    #[test]
    fn parent_offset_of_returns_none_for_orphan() {
        let env = json!({"offset": 0, "msg_type": "post", "metadata": {}});
        assert_eq!(parent_offset_of(&env), None);
    }

    #[test]
    fn parent_offset_of_parses_numeric_string() {
        let env = json!({"offset": 5, "metadata": {"in_reply_to": "3"}});
        assert_eq!(parent_offset_of(&env), Some(3));
    }

    #[test]
    fn parent_offset_of_returns_none_for_non_numeric() {
        let env = json!({"offset": 5, "metadata": {"in_reply_to": "not-a-number"}});
        assert_eq!(parent_offset_of(&env), None);
    }

    #[test]
    fn parent_offset_of_returns_none_for_missing_metadata() {
        let env = json!({"offset": 5, "msg_type": "post"});
        assert_eq!(parent_offset_of(&env), None);
    }

    #[test]
    fn parent_offset_of_handles_reaction_envelope() {
        // Reactions carry in_reply_to → the parent they react to.
        let env = json!({
            "offset": 7,
            "msg_type": "reaction",
            "metadata": {"in_reply_to": "2"},
        });
        assert_eq!(parent_offset_of(&env), Some(2));
    }

    // T-1345: compute_pinned_set
    fn pin_env(off: u64, target: u64, action: &str, by: &str, ts: i64) -> Value {
        json!({
            "offset": off,
            "msg_type": "pin",
            "sender_id": by,
            "ts": ts,
            "payload_b64": "",
            "metadata": {
                "pin_target": target.to_string(),
                "action": action,
            },
        })
    }

    fn content_env(off: u64, payload: &str) -> Value {
        use base64::Engine;
        let p = base64::engine::general_purpose::STANDARD.encode(payload);
        json!({
            "offset": off,
            "msg_type": "post",
            "sender_id": "alice",
            "payload_b64": p,
        })
    }

    #[test]
    fn compute_pinned_set_empty_topic_is_empty() {
        assert_eq!(compute_pinned_set(&[]), Vec::<PinRow>::new());
    }

    #[test]
    fn compute_pinned_set_single_pin_appears() {
        let envs = vec![
            content_env(0, "important note"),
            pin_env(1, 0, "pin", "alice", 100),
        ];
        let rows = compute_pinned_set(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target, 0);
        assert_eq!(rows[0].pinned_by, "alice");
        assert_eq!(rows[0].pinned_ts, 100);
        assert_eq!(rows[0].payload.as_deref(), Some("important note"));
    }

    #[test]
    fn compute_pinned_set_unpin_removes_target() {
        let envs = vec![
            content_env(0, "x"),
            pin_env(1, 0, "pin", "alice", 100),
            pin_env(2, 0, "unpin", "bob", 200),
        ];
        assert!(compute_pinned_set(&envs).is_empty());
    }

    #[test]
    fn compute_pinned_set_repin_after_unpin_restores() {
        // pin → unpin → pin: result is one active row with the LATEST ts.
        let envs = vec![
            content_env(0, "x"),
            pin_env(1, 0, "pin", "alice", 100),
            pin_env(2, 0, "unpin", "bob", 200),
            pin_env(3, 0, "pin", "carol", 300),
        ];
        let rows = compute_pinned_set(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pinned_by, "carol");
        assert_eq!(rows[0].pinned_ts, 300);
    }

    #[test]
    fn compute_pinned_set_multiple_targets_sorted_desc() {
        let envs = vec![
            content_env(0, "first"),
            content_env(1, "second"),
            content_env(2, "third"),
            pin_env(3, 0, "pin", "alice", 100),
            pin_env(4, 1, "pin", "bob", 300),
            pin_env(5, 2, "pin", "carol", 200),
        ];
        let rows = compute_pinned_set(&envs);
        assert_eq!(rows.len(), 3);
        // Sorted by pinned_ts desc: 300 (target=1), 200 (target=2), 100 (target=0)
        assert_eq!(rows[0].target, 1);
        assert_eq!(rows[1].target, 2);
        assert_eq!(rows[2].target, 0);
    }

    #[test]
    fn compute_pinned_set_target_missing_keeps_row_with_no_payload() {
        // Pin references an offset not in the topic — degraded but visible.
        let envs = vec![pin_env(0, 99, "pin", "alice", 100)];
        let rows = compute_pinned_set(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target, 99);
        assert_eq!(rows[0].payload, None);
    }

    #[test]
    fn compute_pinned_set_skips_non_pin_envelopes() {
        // Reaction / chat / edit envelopes must NOT contribute to pin set.
        let envs = vec![
            content_env(0, "x"),
            json!({"offset": 1, "msg_type": "reaction", "metadata": {"in_reply_to": "0"}}),
            json!({"offset": 2, "msg_type": "edit", "metadata": {"replaces": "0"}}),
        ];
        assert!(compute_pinned_set(&envs).is_empty());
    }

    #[test]
    fn compute_pinned_set_skips_non_numeric_target() {
        let envs = vec![
            content_env(0, "x"),
            json!({
                "offset": 1, "msg_type": "pin", "sender_id": "alice",
                "metadata": {"pin_target": "not-a-number", "action": "pin"},
            }),
        ];
        assert!(compute_pinned_set(&envs).is_empty());
    }

    // T-1354: compute_starred_set
    fn star_env(off: u64, target: u64, star: bool, by: &str, ts: i64) -> Value {
        json!({
            "offset": off,
            "msg_type": "star",
            "sender_id": by,
            "ts": ts,
            "payload_b64": "",
            "metadata": {
                "star_target": target.to_string(),
                "star": if star { "true" } else { "false" },
            },
        })
    }

    #[test]
    fn compute_starred_set_empty_topic_is_empty() {
        assert_eq!(compute_starred_set(&[], None), Vec::<StarRow>::new());
    }

    #[test]
    fn compute_starred_set_single_star_appears() {
        let envs = vec![
            content_env(0, "hello"),
            star_env(1, 0, true, "alice", 100),
        ];
        let rows = compute_starred_set(&envs, None);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target, 0);
        assert_eq!(rows[0].starred_by, "alice");
        assert_eq!(rows[0].starred_ts, 100);
        assert_eq!(rows[0].payload.as_deref(), Some("hello"));
    }

    #[test]
    fn compute_starred_set_unstar_removes_target() {
        let envs = vec![
            content_env(0, "hi"),
            star_env(1, 0, true, "alice", 100),
            star_env(2, 0, false, "alice", 200),
        ];
        assert!(compute_starred_set(&envs, None).is_empty());
    }

    #[test]
    fn compute_starred_set_unstar_without_prior_is_noop() {
        let envs = vec![
            content_env(0, "hi"),
            star_env(1, 0, false, "alice", 100),
        ];
        assert!(compute_starred_set(&envs, None).is_empty());
    }

    #[test]
    fn compute_starred_set_per_user_keys() {
        // alice and bob both star offset 0 — both rows survive (different users).
        let envs = vec![
            content_env(0, "shared"),
            star_env(1, 0, true, "alice", 100),
            star_env(2, 0, true, "bob", 200),
        ];
        let rows = compute_starred_set(&envs, None);
        assert_eq!(rows.len(), 2);
        // Newest first.
        assert_eq!(rows[0].starred_by, "bob");
        assert_eq!(rows[1].starred_by, "alice");
    }

    #[test]
    fn compute_starred_set_caller_filter() {
        let envs = vec![
            content_env(0, "shared"),
            star_env(1, 0, true, "alice", 100),
            star_env(2, 0, true, "bob", 200),
        ];
        let rows = compute_starred_set(&envs, Some("alice"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].starred_by, "alice");
    }

    #[test]
    fn compute_starred_set_one_user_unstar_does_not_affect_other() {
        let envs = vec![
            content_env(0, "shared"),
            star_env(1, 0, true, "alice", 100),
            star_env(2, 0, true, "bob", 150),
            star_env(3, 0, false, "alice", 200), // alice unstars
        ];
        let rows = compute_starred_set(&envs, None);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].starred_by, "bob");
    }

    #[test]
    fn compute_starred_set_skips_non_star_envelopes() {
        let envs = vec![
            content_env(0, "x"),
            pin_env(1, 0, "pin", "alice", 100),
        ];
        assert!(compute_starred_set(&envs, None).is_empty());
    }

    #[test]
    fn compute_starred_set_skips_non_numeric_target() {
        let envs = vec![
            content_env(0, "x"),
            json!({
                "offset": 1, "msg_type": "star", "sender_id": "alice", "ts": 100,
                "metadata": {"star_target": "garbage", "star": "true"},
            }),
        ];
        assert!(compute_starred_set(&envs, None).is_empty());
    }

    // T-1355: compute_poll_state
    fn poll_start_env(off: u64, q: &str, opts: &[&str], by: &str, ts: i64) -> Value {
        use base64::Engine;
        let p = base64::engine::general_purpose::STANDARD.encode(q);
        json!({
            "offset": off,
            "msg_type": "poll_start",
            "sender_id": by,
            "ts": ts,
            "payload_b64": p,
            "metadata": {
                "poll_options": opts.join("|"),
            },
        })
    }
    fn poll_vote_env(off: u64, poll_id: u64, choice: u64, by: &str, ts: i64) -> Value {
        json!({
            "offset": off,
            "msg_type": "poll_vote",
            "sender_id": by,
            "ts": ts,
            "payload_b64": "",
            "metadata": {
                "poll_id": poll_id.to_string(),
                "poll_choice": choice.to_string(),
            },
        })
    }
    fn poll_end_env(off: u64, poll_id: u64, by: &str, ts: i64) -> Value {
        json!({
            "offset": off,
            "msg_type": "poll_end",
            "sender_id": by,
            "ts": ts,
            "payload_b64": "",
            "metadata": {
                "poll_id": poll_id.to_string(),
            },
        })
    }

    #[test]
    fn compute_poll_state_no_start_returns_none() {
        assert!(compute_poll_state(&[], 0).is_none());
    }

    #[test]
    fn compute_poll_state_start_only_no_votes() {
        let envs = vec![poll_start_env(0, "Lunch?", &["Pizza", "Salad"], "alice", 100)];
        let s = compute_poll_state(&envs, 0).unwrap();
        assert_eq!(s.question, "Lunch?");
        assert_eq!(s.options.len(), 2);
        assert_eq!(s.options[0].label, "Pizza");
        assert_eq!(s.options[0].count, 0);
        assert_eq!(s.total_votes, 0);
        assert!(!s.closed);
    }

    #[test]
    fn compute_poll_state_one_vote() {
        let envs = vec![
            poll_start_env(0, "Q", &["A", "B"], "alice", 100),
            poll_vote_env(1, 0, 1, "bob", 200),
        ];
        let s = compute_poll_state(&envs, 0).unwrap();
        assert_eq!(s.options[1].count, 1);
        assert_eq!(s.options[1].voters, vec!["bob"]);
        assert_eq!(s.total_votes, 1);
    }

    #[test]
    fn compute_poll_state_vote_replacement() {
        // Bob votes A then changes mind to B; only B counts.
        let envs = vec![
            poll_start_env(0, "Q", &["A", "B"], "alice", 100),
            poll_vote_env(1, 0, 0, "bob", 200),
            poll_vote_env(2, 0, 1, "bob", 300),
        ];
        let s = compute_poll_state(&envs, 0).unwrap();
        assert_eq!(s.options[0].count, 0);
        assert_eq!(s.options[1].count, 1);
        assert_eq!(s.total_votes, 1);
    }

    #[test]
    fn compute_poll_state_closed_drops_late_votes() {
        let envs = vec![
            poll_start_env(0, "Q", &["A", "B"], "alice", 100),
            poll_vote_env(1, 0, 0, "bob", 200),
            poll_end_env(2, 0, "alice", 250),
            // late vote — must be ignored.
            poll_vote_env(3, 0, 1, "carol", 300),
        ];
        let s = compute_poll_state(&envs, 0).unwrap();
        assert!(s.closed);
        assert_eq!(s.options[0].count, 1);
        assert_eq!(s.options[1].count, 0);
        assert_eq!(s.total_votes, 1);
    }

    #[test]
    fn compute_poll_state_multiple_voters() {
        let envs = vec![
            poll_start_env(0, "Q", &["A", "B", "C"], "alice", 100),
            poll_vote_env(1, 0, 0, "bob", 200),
            poll_vote_env(2, 0, 0, "carol", 250),
            poll_vote_env(3, 0, 2, "dave", 300),
        ];
        let s = compute_poll_state(&envs, 0).unwrap();
        assert_eq!(s.options[0].count, 2);
        assert_eq!(s.options[0].voters, vec!["bob", "carol"]);
        assert_eq!(s.options[2].count, 1);
        assert_eq!(s.total_votes, 3);
    }

    #[test]
    fn compute_poll_state_out_of_range_choice_dropped() {
        let envs = vec![
            poll_start_env(0, "Q", &["A", "B"], "alice", 100),
            poll_vote_env(1, 0, 5, "bob", 200), // out of range
        ];
        let s = compute_poll_state(&envs, 0).unwrap();
        assert_eq!(s.total_votes, 0);
    }

    #[test]
    fn compute_poll_state_filters_by_poll_id() {
        // Two polls in the same topic; voting on one must not affect the other.
        let envs = vec![
            poll_start_env(0, "P0", &["A", "B"], "alice", 100),
            poll_start_env(1, "P1", &["X", "Y"], "alice", 110),
            poll_vote_env(2, 0, 1, "bob", 200),
            poll_vote_env(3, 1, 0, "carol", 250),
        ];
        let s0 = compute_poll_state(&envs, 0).unwrap();
        let s1 = compute_poll_state(&envs, 1).unwrap();
        assert_eq!(s0.options[1].count, 1);
        assert_eq!(s0.options[0].count, 0);
        assert_eq!(s1.options[0].count, 1);
        assert_eq!(s1.options[1].count, 0);
    }

    #[test]
    fn compute_poll_state_malformed_start_too_few_options_returns_none() {
        // Only one option — invalid.
        let envs = vec![poll_start_env(0, "Q", &["only"], "alice", 100)];
        assert!(compute_poll_state(&envs, 0).is_none());
    }

    // T-1356: compute_digest
    fn chat_env(off: u64, sender: &str, ts: i64, payload: &str) -> Value {
        use base64::Engine;
        let p = base64::engine::general_purpose::STANDARD.encode(payload);
        json!({
            "offset": off,
            "msg_type": "chat",
            "sender_id": sender,
            "ts": ts,
            "payload_b64": p,
        })
    }
    fn reaction_env(off: u64, sender: &str, ts: i64, emoji: &str) -> Value {
        use base64::Engine;
        let p = base64::engine::general_purpose::STANDARD.encode(emoji);
        json!({
            "offset": off,
            "msg_type": "reaction",
            "sender_id": sender,
            "ts": ts,
            "payload_b64": p,
            "metadata": { "in_reply_to": "0" },
        })
    }
    fn forward_env(off: u64, sender: &str, ts: i64, payload: &str) -> Value {
        use base64::Engine;
        let p = base64::engine::general_purpose::STANDARD.encode(payload);
        json!({
            "offset": off,
            "msg_type": "chat",
            "sender_id": sender,
            "ts": ts,
            "payload_b64": p,
            "metadata": { "forwarded_from": "src:0:alice" },
        })
    }

    #[test]
    fn compute_digest_empty_topic() {
        let d = compute_digest(&[], 0);
        assert_eq!(d.posts, 0);
        assert_eq!(d.distinct_senders, 0);
        assert!(d.top_senders.is_empty());
        assert!(d.recent_chats.is_empty());
    }

    #[test]
    fn compute_digest_filters_by_since() {
        let envs = vec![
            chat_env(0, "alice", 50, "old"),
            chat_env(1, "alice", 200, "new"),
        ];
        let d = compute_digest(&envs, 100);
        assert_eq!(d.posts, 1);
        assert_eq!(d.recent_chats.len(), 1);
        assert_eq!(d.recent_chats[0].payload, "new");
    }

    #[test]
    fn compute_digest_top_senders_sorted_desc() {
        let envs = vec![
            chat_env(0, "alice", 100, "a1"),
            chat_env(1, "alice", 110, "a2"),
            chat_env(2, "alice", 120, "a3"),
            chat_env(3, "bob", 130, "b1"),
            chat_env(4, "bob", 140, "b2"),
            chat_env(5, "carol", 150, "c1"),
        ];
        let d = compute_digest(&envs, 0);
        assert_eq!(d.posts, 6);
        assert_eq!(d.distinct_senders, 3);
        assert_eq!(d.top_senders.len(), 3);
        assert_eq!(d.top_senders[0], ("alice".to_string(), 3));
        assert_eq!(d.top_senders[1], ("bob".to_string(), 2));
        assert_eq!(d.top_senders[2], ("carol".to_string(), 1));
    }

    #[test]
    fn compute_digest_top_senders_truncated_to_three() {
        let envs = vec![
            chat_env(0, "a", 100, "x"),
            chat_env(1, "b", 100, "x"),
            chat_env(2, "c", 100, "x"),
            chat_env(3, "d", 100, "x"),
            chat_env(4, "e", 100, "x"),
        ];
        let d = compute_digest(&envs, 0);
        assert_eq!(d.top_senders.len(), 3);
    }

    #[test]
    fn compute_digest_top_reactions() {
        let envs = vec![
            reaction_env(0, "alice", 100, "👍"),
            reaction_env(1, "bob", 100, "👍"),
            reaction_env(2, "carol", 100, "❤"),
        ];
        let d = compute_digest(&envs, 0);
        assert_eq!(d.top_reactions.len(), 2);
        assert_eq!(d.top_reactions[0], ("👍".to_string(), 2));
        assert_eq!(d.top_reactions[1], ("❤".to_string(), 1));
    }

    #[test]
    fn compute_digest_pins_counted() {
        let envs = vec![
            pin_env(0, 5, "pin", "alice", 100),
            pin_env(1, 7, "pin", "alice", 110),
            pin_env(2, 5, "unpin", "alice", 120),
        ];
        let d = compute_digest(&envs, 0);
        assert_eq!(d.pins_added, 2);
        assert_eq!(d.pins_removed, 1);
    }

    #[test]
    fn compute_digest_forwards_counted() {
        let envs = vec![forward_env(0, "alice", 100, "fwd"), chat_env(1, "alice", 100, "native")];
        let d = compute_digest(&envs, 0);
        assert_eq!(d.forwards_in, 1);
        assert_eq!(d.posts, 2);
    }

    // T-1363: compute_snippet
    #[test]
    fn compute_snippet_target_in_middle() {
        let envs = vec![
            chat_env(0, "alice", 100, "first"),
            chat_env(1, "alice", 110, "second"),
            chat_env(2, "alice", 120, "target"),
            chat_env(3, "alice", 130, "fourth"),
            chat_env(4, "alice", 140, "fifth"),
        ];
        let s = compute_snippet(&envs, 2, 1).unwrap();
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].offset, 1);
        assert_eq!(s[1].offset, 2);
        assert!(s[1].is_target);
        assert_eq!(s[2].offset, 3);
    }

    #[test]
    fn compute_snippet_target_at_start() {
        let envs = vec![
            chat_env(0, "alice", 100, "first"),
            chat_env(1, "alice", 110, "second"),
            chat_env(2, "alice", 120, "third"),
        ];
        let s = compute_snippet(&envs, 0, 2).unwrap();
        assert_eq!(s.len(), 3); // 0 + 2 ahead
        assert!(s[0].is_target);
    }

    #[test]
    fn compute_snippet_target_at_end() {
        let envs = vec![
            chat_env(0, "alice", 100, "first"),
            chat_env(1, "alice", 110, "second"),
            chat_env(2, "alice", 120, "third"),
        ];
        let s = compute_snippet(&envs, 2, 2).unwrap();
        assert_eq!(s.len(), 3); // 2 behind + target
        assert!(s[2].is_target);
    }

    #[test]
    fn compute_snippet_lines_zero() {
        let envs = vec![
            chat_env(0, "alice", 100, "first"),
            chat_env(1, "alice", 110, "target"),
            chat_env(2, "alice", 120, "third"),
        ];
        let s = compute_snippet(&envs, 1, 0).unwrap();
        assert_eq!(s.len(), 1);
        assert!(s[0].is_target);
    }

    #[test]
    fn compute_snippet_lines_larger_than_topic() {
        let envs = vec![
            chat_env(0, "alice", 100, "a"),
            chat_env(1, "alice", 110, "target"),
        ];
        let s = compute_snippet(&envs, 1, 100).unwrap();
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn compute_snippet_target_missing_returns_none() {
        let envs = vec![chat_env(0, "alice", 100, "x")];
        assert!(compute_snippet(&envs, 99, 2).is_none());
    }

    #[test]
    fn compute_snippet_skips_meta_envelopes() {
        // Reactions and redactions should NOT count as snippet lines.
        let envs = vec![
            chat_env(0, "alice", 100, "first"),
            reaction_env(1, "alice", 110, "👍"),
            chat_env(2, "alice", 120, "target"),
            redaction_env(3, 0, "alice"),
            chat_env(4, "alice", 140, "fourth"),
        ];
        let s = compute_snippet(&envs, 2, 5).unwrap();
        // Only content envelopes: offsets 0, 2, 4.
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].offset, 0);
        assert_eq!(s[1].offset, 2);
        assert_eq!(s[2].offset, 4);
    }

    // T-1362: compute_reactions_of
    #[test]
    fn compute_reactions_of_empty_topic() {
        assert!(compute_reactions_of(&[], "alice").is_empty());
    }

    #[test]
    fn compute_reactions_of_filters_by_sender() {
        let envs = vec![
            content_env(0, "post"),
            reaction_env(1, "alice", 100, "👍"),
            reaction_env(2, "bob", 100, "❤"),
        ];
        let rows = compute_reactions_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].emoji, "👍");
    }

    #[test]
    fn compute_reactions_of_excludes_redacted() {
        let envs = vec![
            content_env(0, "post"),
            reaction_env(1, "alice", 100, "👍"),
            reaction_env(2, "alice", 110, "❤"),
            redaction_env(3, 1, "alice"),
        ];
        let rows = compute_reactions_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reaction_offset, 2);
    }

    #[test]
    fn compute_reactions_of_sorted_offset_desc() {
        let envs = vec![
            content_env(0, "post"),
            reaction_env(1, "alice", 100, "👍"),
            reaction_env(2, "alice", 200, "❤"),
            reaction_env(3, "alice", 300, "🚀"),
        ];
        let rows = compute_reactions_of(&envs, "alice");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].emoji, "🚀");
        assert_eq!(rows[1].emoji, "❤");
        assert_eq!(rows[2].emoji, "👍");
    }

    #[test]
    fn compute_reactions_of_fills_parent_preview() {
        let envs = vec![
            content_env(0, "first message"),
            reaction_env(1, "alice", 100, "👍"),
        ];
        let rows = compute_reactions_of(&envs, "alice");
        assert_eq!(rows[0].parent_payload.as_deref(), Some("first message"));
    }

    #[test]
    fn compute_reactions_of_skips_when_missing_in_reply_to() {
        // Reaction without metadata.in_reply_to is skipped.
        let envs = vec![
            content_env(0, "post"),
            json!({
                "offset": 1, "msg_type": "reaction", "sender_id": "alice", "ts": 100,
                "payload_b64": "8J+RjQ==",  // base64 of 👍
            }),
        ];
        assert!(compute_reactions_of(&envs, "alice").is_empty());
    }

    #[test]
    fn compute_reactions_of_skips_empty_payload() {
        let envs = vec![
            content_env(0, "post"),
            json!({
                "offset": 1, "msg_type": "reaction", "sender_id": "alice", "ts": 100,
                "payload_b64": "",
                "metadata": {"in_reply_to": "0"},
            }),
        ];
        assert!(compute_reactions_of(&envs, "alice").is_empty());
    }

    // T-1361: compute_ack_status
    fn ack_receipts(items: &[(&str, u64, i64)]) -> std::collections::HashMap<String, (u64, i64)> {
        items.iter().map(|(s, u, t)| (s.to_string(), (*u, *t))).collect()
    }

    #[test]
    fn compute_ack_status_empty_topic_no_receipts() {
        let receipts = ack_receipts(&[]);
        let rows = compute_ack_status(&[], &receipts, 0);
        assert!(rows.is_empty());
    }

    #[test]
    fn compute_ack_status_single_member_caught_up() {
        let envs = vec![chat_env(0, "alice", 100, "msg")];
        let receipts = ack_receipts(&[("alice", 0, 200)]);
        let rows = compute_ack_status(&envs, &receipts, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sender_id, "alice");
        assert_eq!(rows[0].lag, 0);
    }

    #[test]
    fn compute_ack_status_member_without_receipt_is_full_unread() {
        let envs = vec![chat_env(0, "alice", 100, "msg"), chat_env(1, "bob", 110, "msg")];
        // Only alice has a receipt; bob has none.
        let receipts = ack_receipts(&[("alice", 1, 200)]);
        let rows = compute_ack_status(&envs, &receipts, 1);
        let bob_row = rows.iter().find(|r| r.sender_id == "bob").unwrap();
        assert!(bob_row.up_to.is_none());
        assert_eq!(bob_row.lag, 2); // latest+1 = 1+1 = 2
    }

    #[test]
    fn compute_ack_status_mixed_lag() {
        let envs = vec![
            chat_env(0, "alice", 100, "msg"),
            chat_env(1, "bob", 110, "msg"),
            chat_env(2, "alice", 120, "msg"),
            chat_env(3, "carol", 130, "msg"),
        ];
        // alice acked up to 1 (lag=2), bob acked up to 0 (lag=3), carol no receipt (lag=4).
        let receipts = ack_receipts(&[
            ("alice", 1, 200),
            ("bob", 0, 200),
        ]);
        let rows = compute_ack_status(&envs, &receipts, 3);
        assert_eq!(rows.len(), 3);
        // Sorted by lag desc: carol(4), bob(3), alice(2).
        assert_eq!(rows[0].sender_id, "carol");
        assert_eq!(rows[0].lag, 4);
        assert_eq!(rows[1].sender_id, "bob");
        assert_eq!(rows[1].lag, 3);
        assert_eq!(rows[2].sender_id, "alice");
        assert_eq!(rows[2].lag, 2);
    }

    #[test]
    fn compute_ack_status_tie_break_on_sender() {
        let envs = vec![chat_env(0, "zebra", 100, "x"), chat_env(1, "apple", 110, "y")];
        // Both at full lag, no receipts.
        let receipts = ack_receipts(&[]);
        let rows = compute_ack_status(&envs, &receipts, 1);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].sender_id, "apple");
        assert_eq!(rows[1].sender_id, "zebra");
    }

    #[test]
    fn compute_ack_status_includes_receipt_only_senders() {
        // Sender posted a receipt but never wrote content — they still appear.
        let envs = vec![chat_env(0, "alice", 100, "msg")];
        let receipts = ack_receipts(&[
            ("alice", 0, 200),
            ("bob", 0, 250), // receipt only
        ]);
        let rows = compute_ack_status(&envs, &receipts, 0);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn compute_ack_status_ack_ahead_of_latest_clamped_to_zero() {
        // Pathological: receipt up_to > latest_offset. Should saturate to lag=0.
        let envs = vec![chat_env(0, "alice", 100, "msg")];
        let receipts = ack_receipts(&[("alice", 99, 200)]);
        let rows = compute_ack_status(&envs, &receipts, 0);
        assert_eq!(rows[0].lag, 0);
    }

    // T-1359: compute_emoji_stats
    fn redaction_env(off: u64, target: u64, by: &str) -> Value {
        json!({
            "offset": off,
            "msg_type": "redaction",
            "sender_id": by,
            "ts": 100,
            "payload_b64": "",
            "metadata": {"redacts": target.to_string()},
        })
    }

    #[test]
    fn compute_emoji_stats_empty() {
        assert!(compute_emoji_stats(&[]).is_empty());
    }

    #[test]
    fn compute_emoji_stats_single_emoji() {
        let envs = vec![
            content_env(0, "msg"),
            reaction_env(1, "alice", 100, "👍"),
        ];
        let rows = compute_emoji_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].emoji, "👍");
        assert_eq!(rows[0].count, 1);
        assert_eq!(rows[0].reactors.len(), 1);
    }

    #[test]
    fn compute_emoji_stats_multiple_emojis_sorted_desc() {
        let envs = vec![
            content_env(0, "msg"),
            reaction_env(1, "alice", 100, "👍"),
            reaction_env(2, "bob", 100, "👍"),
            reaction_env(3, "carol", 100, "👍"),
            reaction_env(4, "alice", 100, "❤"),
            reaction_env(5, "bob", 100, "🚀"),
        ];
        let rows = compute_emoji_stats(&envs);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].emoji, "👍");
        assert_eq!(rows[0].count, 3);
        // ❤ and 🚀 both have count 1; tie-break on emoji asc.
        assert_eq!(rows[1].count, 1);
        assert_eq!(rows[2].count, 1);
    }

    #[test]
    fn compute_emoji_stats_redacted_excluded() {
        let envs = vec![
            content_env(0, "msg"),
            reaction_env(1, "alice", 100, "👍"),
            reaction_env(2, "bob", 100, "👍"),
            redaction_env(3, 1, "alice"),
        ];
        let rows = compute_emoji_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].count, 1); // alice's was redacted; only bob's left
        assert_eq!(rows[0].reactors[0].0, "bob");
    }

    #[test]
    fn compute_emoji_stats_per_sender_count() {
        // alice reacts twice with 👍, bob once. reactors row should be sorted
        // alice (2) then bob (1).
        let envs = vec![
            content_env(0, "msg"),
            reaction_env(1, "alice", 100, "👍"),
            reaction_env(2, "alice", 100, "👍"),
            reaction_env(3, "bob", 100, "👍"),
        ];
        let rows = compute_emoji_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].count, 3);
        assert_eq!(rows[0].reactors.len(), 2);
        assert_eq!(rows[0].reactors[0], ("alice".to_string(), 2));
        assert_eq!(rows[0].reactors[1], ("bob".to_string(), 1));
    }

    #[test]
    fn compute_emoji_stats_skips_non_reaction_envelopes() {
        let envs = vec![
            content_env(0, "msg"),
            content_env(1, "another"),
        ];
        assert!(compute_emoji_stats(&envs).is_empty());
    }

    #[test]
    fn compute_emoji_stats_skips_empty_payload() {
        let envs = vec![
            json!({
                "offset": 0, "msg_type": "reaction", "sender_id": "alice",
                "ts": 100, "payload_b64": "",
            }),
        ];
        assert!(compute_emoji_stats(&envs).is_empty());
    }

    // T-1358: compute_unread_rows
    fn counts_map(items: &[(&str, u64)]) -> std::collections::HashMap<String, u64> {
        items.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn compute_unread_rows_empty_cursors() {
        let counts = counts_map(&[("foo", 5)]);
        assert!(compute_unread_rows(&[], &counts).is_empty());
    }

    #[test]
    fn compute_unread_rows_caller_caught_up() {
        // count=5 → latest=4. cursor=4 → caught up.
        let cursors = vec![("foo".to_string(), 4)];
        let counts = counts_map(&[("foo", 5)]);
        assert!(compute_unread_rows(&cursors, &counts).is_empty());
    }

    #[test]
    fn compute_unread_rows_simple_unread() {
        // count=10 → latest=9. cursor=5 → unread = 9-5 = 4.
        let cursors = vec![("foo".to_string(), 5)];
        let counts = counts_map(&[("foo", 10)]);
        let rows = compute_unread_rows(&cursors, &counts);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].unread, 4);
        assert_eq!(rows[0].latest, 9);
        assert_eq!(rows[0].cursor, 5);
    }

    #[test]
    fn compute_unread_rows_topic_missing_dropped() {
        let cursors = vec![("foo".to_string(), 1), ("bar".to_string(), 0)];
        let counts = counts_map(&[("foo", 5)]);
        let rows = compute_unread_rows(&cursors, &counts);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].topic, "foo");
    }

    #[test]
    fn compute_unread_rows_zero_count_dropped() {
        let cursors = vec![("foo".to_string(), 0)];
        let counts = counts_map(&[("foo", 0)]);
        assert!(compute_unread_rows(&cursors, &counts).is_empty());
    }

    #[test]
    fn compute_unread_rows_cursor_ahead_of_latest_dropped() {
        // cursor=10, count=5 → latest=4, cursor >= latest. drop.
        let cursors = vec![("foo".to_string(), 10)];
        let counts = counts_map(&[("foo", 5)]);
        assert!(compute_unread_rows(&cursors, &counts).is_empty());
    }

    #[test]
    fn compute_unread_rows_sorted_by_unread_desc() {
        let cursors = vec![
            ("a".to_string(), 0), // unread=4
            ("b".to_string(), 0), // unread=9
            ("c".to_string(), 0), // unread=1
        ];
        let counts = counts_map(&[("a", 5), ("b", 10), ("c", 2)]);
        let rows = compute_unread_rows(&cursors, &counts);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].topic, "b");
        assert_eq!(rows[1].topic, "a");
        assert_eq!(rows[2].topic, "c");
    }

    #[test]
    fn compute_unread_rows_tie_break_on_topic() {
        // Two topics with same unread count — tie break alphabetical.
        let cursors = vec![
            ("zebra".to_string(), 0),
            ("apple".to_string(), 0),
        ];
        let counts = counts_map(&[("zebra", 5), ("apple", 5)]);
        let rows = compute_unread_rows(&cursors, &counts);
        assert_eq!(rows[0].topic, "apple");
        assert_eq!(rows[1].topic, "zebra");
    }

    #[test]
    fn compute_digest_recent_chats_last_three_in_order() {
        let envs = vec![
            chat_env(0, "alice", 100, "first"),
            chat_env(1, "alice", 110, "second"),
            chat_env(2, "alice", 120, "third"),
            chat_env(3, "alice", 130, "fourth"),
            chat_env(4, "alice", 140, "fifth"),
        ];
        let d = compute_digest(&envs, 0);
        assert_eq!(d.recent_chats.len(), 3);
        assert_eq!(d.recent_chats[0].payload, "third");
        assert_eq!(d.recent_chats[1].payload, "fourth");
        assert_eq!(d.recent_chats[2].payload, "fifth");
    }

    // T-1352: should_emit_for_until
    #[test]
    fn should_emit_for_until_no_filter_keeps_all() {
        let env = json!({"ts": 5000});
        assert!(should_emit_for_until(&env, None));
    }

    #[test]
    fn should_emit_for_until_keeps_at_boundary() {
        // ts == until → kept (inclusive upper bound).
        let env = json!({"ts": 1000});
        assert!(should_emit_for_until(&env, Some(1000)));
    }

    #[test]
    fn should_emit_for_until_keeps_before() {
        let env = json!({"ts": 500});
        assert!(should_emit_for_until(&env, Some(1000)));
    }

    #[test]
    fn should_emit_for_until_drops_after() {
        let env = json!({"ts": 1500});
        assert!(!should_emit_for_until(&env, Some(1000)));
    }

    #[test]
    fn should_emit_for_until_keeps_ts_less_envelope() {
        // Defensive: same precedent as --since — keep envelopes without ts.
        let env = json!({"offset": 0, "msg_type": "post"});
        assert!(should_emit_for_until(&env, Some(1000)));
    }

    #[test]
    fn should_emit_for_until_uses_ts_unix_ms_when_present() {
        // Mirror should_emit_for_since: prefer ts_unix_ms over ts when both.
        let env = json!({"ts_unix_ms": 500, "ts": 5000});
        assert!(should_emit_for_until(&env, Some(1000)));
    }

    // T-1351: compute_active_typers
    fn typing_env(off: u64, sender: &str, ts: i64, expires_at: i64) -> Value {
        json!({
            "offset": off,
            "msg_type": "typing",
            "sender_id": sender,
            "ts": ts,
            "metadata": {"expires_at_ms": expires_at.to_string()},
        })
    }

    #[test]
    fn compute_active_typers_empty_input() {
        assert_eq!(compute_active_typers(&[], 1000), Vec::<TyperRow>::new());
    }

    #[test]
    fn compute_active_typers_single_active() {
        let envs = vec![typing_env(0, "alice", 100, 5000)];
        let rows = compute_active_typers(&envs, 1000);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sender_id, "alice");
        assert_eq!(rows[0].expires_at_ms, 5000);
        assert_eq!(rows[0].ts, 100);
    }

    #[test]
    fn compute_active_typers_single_expired_dropped() {
        // expires_at_ms (500) is in the past relative to now (1000)
        let envs = vec![typing_env(0, "alice", 100, 500)];
        let rows = compute_active_typers(&envs, 1000);
        assert!(rows.is_empty());
    }

    #[test]
    fn compute_active_typers_now_equals_expiry_dropped() {
        // Boundary: expires_at_ms == now_ms must be considered expired.
        // Reasoning: if "expires_at" is the moment the indicator stops being
        // valid, then at that moment it is no longer active.
        let envs = vec![typing_env(0, "alice", 100, 1000)];
        let rows = compute_active_typers(&envs, 1000);
        assert!(rows.is_empty());
    }

    #[test]
    fn compute_active_typers_multiple_some_expired() {
        let envs = vec![
            typing_env(0, "alice", 100, 5000),
            typing_env(1, "bob", 200, 500),    // expired
            typing_env(2, "carol", 300, 6000),
        ];
        let rows = compute_active_typers(&envs, 1000);
        assert_eq!(rows.len(), 2);
        // Sort by ts desc → carol (300), alice (100)
        assert_eq!(rows[0].sender_id, "carol");
        assert_eq!(rows[1].sender_id, "alice");
    }

    #[test]
    fn compute_active_typers_latest_per_sender_wins() {
        // alice has 2 typing envelopes; the LATEST (offset-wise) wins. Older
        // active envelope must NOT mask a newer expired one.
        let envs = vec![
            typing_env(0, "alice", 100, 5000), // active (would survive if alone)
            typing_env(1, "alice", 200, 500),  // expired (replaces the active one)
        ];
        let rows = compute_active_typers(&envs, 1000);
        assert!(rows.is_empty(), "newer expired envelope must replace older active");
    }

    #[test]
    fn compute_active_typers_skips_non_typing() {
        let envs = vec![
            json!({"offset": 0, "msg_type": "post", "sender_id": "alice"}),
            json!({"offset": 1, "msg_type": "reaction", "sender_id": "bob"}),
        ];
        assert!(compute_active_typers(&envs, 1000).is_empty());
    }

    #[test]
    fn compute_active_typers_sorted_by_ts_desc_with_sender_tie_break() {
        let envs = vec![
            typing_env(0, "bob", 500, 5000),
            typing_env(1, "alice", 500, 5000), // same ts → tie break on sender asc
            typing_env(2, "carol", 1000, 5000),
        ];
        let rows = compute_active_typers(&envs, 100);
        assert_eq!(rows[0].sender_id, "carol"); // ts=1000
        assert_eq!(rows[1].sender_id, "alice"); // ts=500, sender < bob
        assert_eq!(rows[2].sender_id, "bob");   // ts=500
    }

    // T-1349: extract_forward
    #[test]
    fn extract_forward_returns_none_for_normal_envelope() {
        let env = json!({"offset": 0, "msg_type": "post", "metadata": {}});
        assert_eq!(extract_forward(&env), None);
    }

    #[test]
    fn extract_forward_returns_none_when_metadata_absent() {
        let env = json!({"offset": 0, "msg_type": "post"});
        assert_eq!(extract_forward(&env), None);
    }

    #[test]
    fn extract_forward_parses_well_formed_metadata() {
        let env = json!({
            "metadata": {
                "forwarded_from": "topic:42",
                "forwarded_sender": "alice",
            }
        });
        assert_eq!(
            extract_forward(&env),
            Some(("topic".to_string(), 42, "alice".to_string()))
        );
    }

    #[test]
    fn extract_forward_handles_topic_with_colons() {
        // dm:a:b is a valid topic name; we split on LAST colon for offset.
        let env = json!({
            "metadata": {
                "forwarded_from": "dm:alice:bob:7",
                "forwarded_sender": "carol",
            }
        });
        assert_eq!(
            extract_forward(&env),
            Some(("dm:alice:bob".to_string(), 7, "carol".to_string()))
        );
    }

    #[test]
    fn extract_forward_returns_none_when_offset_non_numeric() {
        let env = json!({
            "metadata": {
                "forwarded_from": "topic:not-a-number",
                "forwarded_sender": "alice",
            }
        });
        assert_eq!(extract_forward(&env), None);
    }

    #[test]
    fn extract_forward_returns_none_when_sender_missing() {
        // Defensive: both fields required; partial provenance should NOT be
        // surfaced — could mask a malformed forward emit.
        let env = json!({
            "metadata": {
                "forwarded_from": "topic:42",
            }
        });
        assert_eq!(extract_forward(&env), None);
    }

    #[test]
    fn extract_forward_returns_none_when_from_lacks_colon() {
        // Malformed metadata.forwarded_from — no offset separator.
        let env = json!({
            "metadata": {
                "forwarded_from": "topic-no-colon",
                "forwarded_sender": "alice",
            }
        });
        assert_eq!(extract_forward(&env), None);
    }

    // T-1348: build_forward_metadata
    #[test]
    fn build_forward_metadata_emits_two_kv_pairs_in_stable_order() {
        let md = build_forward_metadata("room:dev", 42, "alice-fingerprint");
        assert_eq!(md.len(), 2);
        assert_eq!(md[0], "forwarded_from=room:dev:42");
        assert_eq!(md[1], "forwarded_sender=alice-fingerprint");
    }

    #[test]
    fn build_forward_metadata_handles_empty_sender() {
        // Defensive: a missing sender becomes empty string. The K=V pair is
        // still emitted (with empty value) so callers can detect the case.
        let md = build_forward_metadata("topic", 0, "");
        assert_eq!(md[1], "forwarded_sender=");
    }

    #[test]
    fn build_forward_metadata_handles_topic_with_colons() {
        // Topic names like "dm:a:b" contain colons. Forward metadata must
        // still include them verbatim so receivers can split-on-LAST-colon
        // to extract offset.
        let md = build_forward_metadata("dm:alice:bob", 7, "carol");
        assert_eq!(md[0], "forwarded_from=dm:alice:bob:7");
    }

    // T-1347: sender_in_csv
    #[test]
    fn sender_in_csv_empty_csv_returns_false() {
        assert!(!sender_in_csv("alice", ""));
        assert!(!sender_in_csv("alice", "  ,  ,  "));
    }

    #[test]
    fn sender_in_csv_empty_sender_returns_false() {
        assert!(!sender_in_csv("", "alice,bob"));
    }

    #[test]
    fn sender_in_csv_single_id_match() {
        assert!(sender_in_csv("alice", "alice"));
    }

    #[test]
    fn sender_in_csv_multi_id_match() {
        assert!(sender_in_csv("bob", "alice,bob,carol"));
        assert!(sender_in_csv("alice", "alice,bob"));
        assert!(sender_in_csv("carol", "alice,bob,carol"));
    }

    #[test]
    fn sender_in_csv_no_match() {
        assert!(!sender_in_csv("dave", "alice,bob,carol"));
    }

    #[test]
    fn sender_in_csv_strips_whitespace() {
        assert!(sender_in_csv("alice", "  alice  ,  bob  "));
        assert!(sender_in_csv("bob", " alice , bob "));
    }

    #[test]
    fn sender_in_csv_case_sensitive() {
        // sender_ids are fingerprint hashes; case must matter.
        assert!(!sender_in_csv("Alice", "alice"));
        assert!(!sender_in_csv("alice", "ALICE"));
    }

    // T-1346: tail_slice
    #[test]
    fn tail_slice_none_returns_full_clone() {
        let v = vec![1, 2, 3];
        assert_eq!(tail_slice(&v, None), vec![1, 2, 3]);
    }

    #[test]
    fn tail_slice_zero_returns_empty() {
        let v = vec![1, 2, 3];
        assert_eq!(tail_slice(&v, Some(0)), Vec::<i32>::new());
    }

    #[test]
    fn tail_slice_n_greater_than_len_returns_all() {
        let v = vec![1, 2, 3];
        assert_eq!(tail_slice(&v, Some(99)), vec![1, 2, 3]);
    }

    #[test]
    fn tail_slice_n_less_than_len_returns_last_n() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(tail_slice(&v, Some(3)), vec![3, 4, 5]);
    }

    #[test]
    fn tail_slice_n_equal_to_len_returns_all() {
        let v = vec!["a", "b"];
        assert_eq!(tail_slice(&v, Some(2)), vec!["a", "b"]);
    }

    #[test]
    fn tail_slice_empty_input_yields_empty_for_any_n() {
        let v: Vec<i32> = Vec::new();
        assert_eq!(tail_slice(&v, None), Vec::<i32>::new());
        assert_eq!(tail_slice(&v, Some(0)), Vec::<i32>::new());
        assert_eq!(tail_slice(&v, Some(5)), Vec::<i32>::new());
    }

    #[test]
    fn tail_slice_preserves_order() {
        let v = vec![10, 20, 30, 40, 50, 60];
        // Last 4 should be [30, 40, 50, 60] — oldest first.
        assert_eq!(tail_slice(&v, Some(4)), vec![30, 40, 50, 60]);
    }

    #[test]
    fn compute_pinned_set_uses_ts_unix_ms_when_present() {
        // Hub may serialize as `ts` or `ts_unix_ms`; helper must accept either.
        let envs = vec![
            content_env(0, "x"),
            json!({
                "offset": 1, "msg_type": "pin", "sender_id": "alice",
                "ts_unix_ms": 555,
                "metadata": {"pin_target": "0", "action": "pin"},
            }),
        ];
        let rows = compute_pinned_set(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pinned_ts, 555);
    }

    // T-1365: compute_threads_index
    fn mk_post(off: u64, sender: &str, ts: i64, payload: &str) -> Value {
        use base64::Engine;
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "post",
            "ts_unix_ms": ts,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "metadata": {},
        })
    }
    fn mk_reply(off: u64, sender: &str, ts: i64, parent: u64, payload: &str) -> Value {
        use base64::Engine;
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "post",
            "ts_unix_ms": ts,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "metadata": {"in_reply_to": parent.to_string()},
        })
    }
    fn mk_redact(off: u64, sender: &str, ts: i64, target: u64) -> Value {
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "redaction",
            "ts_unix_ms": ts,
            "payload_b64": "",
            "metadata": {"redacts": target.to_string()},
        })
    }

    #[test]
    fn threads_index_no_replies_empty() {
        let envs = vec![
            mk_post(0, "alice", 1000, "hello"),
            mk_post(1, "bob", 1100, "world"),
        ];
        let rows = compute_threads_index(&envs);
        assert!(rows.is_empty());
    }

    #[test]
    fn threads_index_single_thread() {
        let envs = vec![
            mk_post(0, "alice", 1000, "root"),
            mk_reply(1, "bob", 1100, 0, "reply1"),
            mk_reply(2, "carol", 1200, 0, "reply2"),
        ];
        let rows = compute_threads_index(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].root_offset, 0);
        assert_eq!(rows[0].reply_count, 2);
        assert_eq!(rows[0].participants, 3); // alice + bob + carol
        assert_eq!(rows[0].last_ts_ms, 1200);
        assert_eq!(rows[0].root_payload.as_deref(), Some("root"));
    }

    #[test]
    fn threads_index_multiple_threads_sorted_desc_by_last_ts() {
        let envs = vec![
            mk_post(0, "alice", 1000, "thread A root"),
            mk_reply(1, "bob", 5000, 0, "reply A1"),
            mk_post(2, "carol", 1500, "thread B root"),
            mk_reply(3, "dave", 9000, 2, "reply B1"),
        ];
        let rows = compute_threads_index(&envs);
        assert_eq!(rows.len(), 2);
        // B (last_ts=9000) before A (last_ts=5000)
        assert_eq!(rows[0].root_offset, 2);
        assert_eq!(rows[0].last_ts_ms, 9000);
        assert_eq!(rows[1].root_offset, 0);
        assert_eq!(rows[1].last_ts_ms, 5000);
    }

    #[test]
    fn threads_index_redacted_root_drops_row() {
        let envs = vec![
            mk_post(0, "alice", 1000, "to-be-redacted"),
            mk_reply(1, "bob", 1100, 0, "orphan reply"),
            mk_redact(2, "alice", 1200, 0),
        ];
        let rows = compute_threads_index(&envs);
        assert!(rows.is_empty(), "redacted root must drop the row");
    }

    #[test]
    fn threads_index_redacted_reply_doesnt_count() {
        let envs = vec![
            mk_post(0, "alice", 1000, "root"),
            mk_reply(1, "bob", 1100, 0, "real reply"),
            mk_reply(2, "mallory", 1150, 0, "spam"),
            mk_redact(3, "mallory", 1200, 2),
        ];
        let rows = compute_threads_index(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reply_count, 1);
        // participants = alice + bob (mallory's reply was redacted)
        assert_eq!(rows[0].participants, 2);
    }

    #[test]
    fn threads_index_deep_nesting_counts_transitively() {
        // 0 → 1 → 2 → 3
        let envs = vec![
            mk_post(0, "a", 100, "root"),
            mk_reply(1, "b", 200, 0, "r1"),
            mk_reply(2, "c", 300, 1, "r2"),
            mk_reply(3, "d", 400, 2, "r3"),
        ];
        let rows = compute_threads_index(&envs);
        // Root at offset 0 has 3 transitive descendants. Offsets 1 and 2 are
        // also "roots" because they have replies → they each become a row too.
        // So we expect 3 rows: roots 0, 1, 2.
        assert_eq!(rows.len(), 3);
        let row_for_0 = rows.iter().find(|r| r.root_offset == 0).unwrap();
        assert_eq!(row_for_0.reply_count, 3);
        assert_eq!(row_for_0.participants, 4);
        assert_eq!(row_for_0.last_ts_ms, 400);
        let row_for_1 = rows.iter().find(|r| r.root_offset == 1).unwrap();
        assert_eq!(row_for_1.reply_count, 2);
        let row_for_2 = rows.iter().find(|r| r.root_offset == 2).unwrap();
        assert_eq!(row_for_2.reply_count, 1);
    }

    // T-1368: compute_topic_stats
    fn mk_react(off: u64, sender: &str, ts: i64, parent: u64, emoji: &str) -> Value {
        use base64::Engine;
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "reaction",
            "ts_unix_ms": ts,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(emoji),
            "metadata": {"in_reply_to": parent.to_string()},
        })
    }
    fn mk_pin(off: u64, sender: &str, ts: i64, target: u64, unpin: bool) -> Value {
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "pin",
            "ts_unix_ms": ts,
            "payload_b64": "",
            "metadata": {
                "pin_target": target.to_string(),
                "action": if unpin { "unpin" } else { "pin" },
            },
        })
    }

    #[test]
    fn topic_stats_empty_topic_zero() {
        let envs: Vec<Value> = vec![];
        let s = compute_full_topic_stats(&envs);
        assert_eq!(s.total, 0);
        assert_eq!(s.distinct_senders, 0);
        assert!(s.first_ts_ms.is_none());
        assert!(s.last_ts_ms.is_none());
    }

    #[test]
    fn topic_stats_single_post() {
        let envs = vec![mk_post(0, "alice", 100, "hi")];
        let s = compute_full_topic_stats(&envs);
        assert_eq!(s.total, 1);
        assert_eq!(s.distinct_senders, 1);
        assert_eq!(s.by_msg_type, vec![("post".to_string(), 1)]);
        assert_eq!(s.first_ts_ms, Some(100));
        assert_eq!(s.last_ts_ms, Some(100));
    }

    #[test]
    fn topic_stats_mixed_msg_types() {
        let envs = vec![
            mk_post(0, "alice", 100, "root"),
            mk_reply(1, "bob", 200, 0, "reply"),
            mk_react(2, "carol", 300, 0, "👍"),
            mk_react(3, "dave", 400, 0, "👍"),
            mk_react(4, "eve", 500, 0, "❤"),
        ];
        let s = compute_full_topic_stats(&envs);
        assert_eq!(s.total, 5);
        assert_eq!(s.distinct_senders, 5);
        assert_eq!(s.thread_roots, 1); // offset 0 is the root
        assert_eq!(s.distinct_emojis, 2); // 👍 and ❤
        assert_eq!(s.top_emojis[0].0, "👍");
        assert_eq!(s.top_emojis[0].1, 2);
        assert_eq!(s.first_ts_ms, Some(100));
        assert_eq!(s.last_ts_ms, Some(500));
    }

    #[test]
    fn topic_stats_redacted_excluded_from_counters() {
        let envs = vec![
            mk_post(0, "alice", 100, "kept"),
            mk_post(1, "bob", 200, "to-redact"),
            mk_redact(2, "alice", 300, 1),
        ];
        let s = compute_full_topic_stats(&envs);
        // total includes the redaction envelope itself but NOT the redacted post
        assert_eq!(s.total, 2); // post 0 + redaction 2; post 1 dropped
        assert_eq!(s.distinct_senders, 1); // only alice (bob's post was redacted)
        assert_eq!(s.redactions, 1);
    }

    #[test]
    fn topic_stats_active_pins_lww() {
        let envs = vec![
            mk_post(0, "alice", 100, "p0"),
            mk_post(1, "alice", 110, "p1"),
            mk_pin(2, "alice", 200, 0, false), // pin 0
            mk_pin(3, "alice", 300, 1, false), // pin 1
            mk_pin(4, "alice", 400, 0, true),  // unpin 0
        ];
        let s = compute_full_topic_stats(&envs);
        assert_eq!(s.active_pins, 1); // only offset 1 still pinned
    }

    #[test]
    fn topic_stats_top_senders_sorted_desc_with_tiebreak() {
        let envs = vec![
            mk_post(0, "zelda", 100, "x"),
            mk_post(1, "amy", 110, "x"),
            mk_post(2, "amy", 120, "x"),
            mk_post(3, "bob", 130, "x"),
            mk_post(4, "bob", 140, "x"),
        ];
        let s = compute_full_topic_stats(&envs);
        // amy=2, bob=2 → tiebreak by name asc → amy first; zelda=1
        assert_eq!(s.top_senders[0].0, "amy");
        assert_eq!(s.top_senders[0].1, 2);
        assert_eq!(s.top_senders[1].0, "bob");
        assert_eq!(s.top_senders[1].1, 2);
        assert_eq!(s.top_senders[2].0, "zelda");
        assert_eq!(s.top_senders[2].1, 1);
    }

    #[test]
    fn topic_stats_forwards_in_via_metadata() {
        let envs = vec![
            mk_post(0, "alice", 100, "p"),
            mk_forward(1, "alice", 200, "other-topic", 5, "bob", "fwd"),
        ];
        let s = compute_full_topic_stats(&envs);
        assert_eq!(s.forwards_in, 1);
    }

    // T-1367: compute_forwards_of
    fn mk_forward(
        off: u64,
        sender: &str,
        ts: i64,
        origin_topic: &str,
        origin_offset: u64,
        origin_sender: &str,
        payload: &str,
    ) -> Value {
        use base64::Engine;
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "forward",
            "ts_unix_ms": ts,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "metadata": {
                "forwarded_from": format!("{origin_topic}:{origin_offset}"),
                "forwarded_sender": origin_sender,
            },
        })
    }

    #[test]
    fn forwards_of_no_forwards_empty() {
        let envs = vec![mk_post(0, "alice", 100, "post")];
        let rows = compute_forwards_of(&envs, "alice");
        assert!(rows.is_empty());
    }

    #[test]
    fn forwards_of_single_forward() {
        let envs = vec![
            mk_post(0, "alice", 100, "stuff"),
            mk_forward(1, "alice", 200, "other", 5, "bob", "fwd-payload"),
        ];
        let rows = compute_forwards_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].forward_offset, 1);
        assert_eq!(rows[0].origin_topic, "other");
        assert_eq!(rows[0].origin_offset, 5);
        assert_eq!(rows[0].origin_sender, "bob");
        assert_eq!(rows[0].payload, "fwd-payload");
        assert_eq!(rows[0].ts, 200);
    }

    #[test]
    fn forwards_of_multiple_sorted_desc() {
        let envs = vec![
            mk_forward(1, "alice", 100, "a", 1, "bob", "first"),
            mk_forward(3, "alice", 300, "b", 2, "carol", "third"),
            mk_forward(2, "alice", 200, "c", 3, "dave", "second"),
        ];
        let rows = compute_forwards_of(&envs, "alice");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].forward_offset, 3);
        assert_eq!(rows[1].forward_offset, 2);
        assert_eq!(rows[2].forward_offset, 1);
    }

    #[test]
    fn forwards_of_other_sender_excluded() {
        let envs = vec![
            mk_forward(1, "alice", 100, "x", 1, "bob", "alice-fwd"),
            mk_forward(2, "carol", 200, "y", 2, "bob", "carol-fwd"),
        ];
        let rows_a = compute_forwards_of(&envs, "alice");
        let rows_c = compute_forwards_of(&envs, "carol");
        assert_eq!(rows_a.len(), 1);
        assert_eq!(rows_a[0].payload, "alice-fwd");
        assert_eq!(rows_c.len(), 1);
        assert_eq!(rows_c[0].payload, "carol-fwd");
    }

    #[test]
    fn forwards_of_redacted_dropped() {
        let envs = vec![
            mk_forward(1, "alice", 100, "x", 1, "bob", "kept"),
            mk_forward(2, "alice", 200, "y", 2, "bob", "to-redact"),
            mk_redact(3, "alice", 300, 2),
        ];
        let rows = compute_forwards_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].forward_offset, 1);
    }

    #[test]
    fn forwards_of_malformed_metadata_ignored() {
        use base64::Engine;
        let envs = vec![
            // forwarded_from missing colon → extract_forward returns None
            json!({
                "offset": 1,
                "sender_id": "alice",
                "msg_type": "forward",
                "ts_unix_ms": 100,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode("garbage"),
                "metadata": {
                    "forwarded_from": "no-colon-here",
                    "forwarded_sender": "bob",
                },
            }),
        ];
        let rows = compute_forwards_of(&envs, "alice");
        assert!(rows.is_empty());
    }

    // T-1366: compute_edits_of
    fn mk_edit(off: u64, sender: &str, ts: i64, replaces: u64, payload: &str) -> Value {
        use base64::Engine;
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "edit",
            "ts_unix_ms": ts,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "metadata": {"replaces": replaces.to_string()},
        })
    }

    #[test]
    fn edits_of_target_with_no_edits_returns_only_original() {
        let envs = vec![mk_post(5, "alice", 100, "hello")];
        let r = compute_edits_of(&envs, 5).expect("target present");
        assert_eq!(r.original.offset, 5);
        assert_eq!(r.original.payload, "hello");
        assert!(r.edits.is_empty());
    }

    #[test]
    fn edits_of_multiple_edits_chronological() {
        let envs = vec![
            mk_post(5, "alice", 100, "v0"),
            mk_edit(6, "alice", 200, 5, "v1"),
            mk_edit(7, "alice", 300, 5, "v2"),
            // Out-of-order ts (older but later offset) — should sort by ts asc
            mk_edit(8, "alice", 250, 5, "v1.5"),
        ];
        let r = compute_edits_of(&envs, 5).unwrap();
        assert_eq!(r.edits.len(), 3);
        assert_eq!(r.edits[0].payload, "v1");      // ts 200
        assert_eq!(r.edits[1].payload, "v1.5");    // ts 250
        assert_eq!(r.edits[2].payload, "v2");      // ts 300
    }

    #[test]
    fn edits_of_redacted_edit_dropped() {
        let envs = vec![
            mk_post(5, "alice", 100, "v0"),
            mk_edit(6, "alice", 200, 5, "v1"),
            mk_edit(7, "alice", 300, 5, "v2"),
            mk_redact(8, "alice", 350, 7), // redact v2
        ];
        let r = compute_edits_of(&envs, 5).unwrap();
        assert_eq!(r.edits.len(), 1);
        assert_eq!(r.edits[0].payload, "v1");
    }

    #[test]
    fn edits_of_redacted_target_returns_none() {
        let envs = vec![
            mk_post(5, "alice", 100, "v0"),
            mk_edit(6, "alice", 200, 5, "v1"),
            mk_redact(7, "alice", 300, 5),
        ];
        assert!(compute_edits_of(&envs, 5).is_none());
    }

    #[test]
    fn edits_of_non_numeric_replaces_ignored() {
        use base64::Engine;
        let envs = vec![
            mk_post(5, "alice", 100, "v0"),
            json!({
                "offset": 6,
                "sender_id": "alice",
                "msg_type": "edit",
                "ts_unix_ms": 200,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode("garbage"),
                "metadata": {"replaces": "not-a-number"},
            }),
        ];
        let r = compute_edits_of(&envs, 5).unwrap();
        assert!(r.edits.is_empty());
    }

    #[test]
    fn edits_of_other_targets_not_in_report() {
        let envs = vec![
            mk_post(5, "alice", 100, "five"),
            mk_post(7, "bob", 110, "seven"),
            mk_edit(8, "alice", 200, 5, "v1-of-five"),
            mk_edit(9, "bob", 210, 7, "v1-of-seven"),
        ];
        let r = compute_edits_of(&envs, 5).unwrap();
        assert_eq!(r.edits.len(), 1);
        assert_eq!(r.edits[0].payload, "v1-of-five");
    }

    #[test]
    fn edits_of_missing_target_returns_none() {
        let envs = vec![mk_post(5, "alice", 100, "v0")];
        assert!(compute_edits_of(&envs, 99).is_none());
    }

    #[test]
    fn threads_index_non_numeric_in_reply_to_ignored() {
        use base64::Engine;
        let envs = vec![
            mk_post(0, "a", 100, "root"),
            json!({
                "offset": 1,
                "sender_id": "b",
                "msg_type": "post",
                "ts_unix_ms": 200,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode("garbage parent"),
                "metadata": {"in_reply_to": "not-a-number"},
            }),
        ];
        let rows = compute_threads_index(&envs);
        assert!(rows.is_empty());
    }

    // T-1370: compute_replies_of
    #[test]
    fn replies_of_happy_path_desc() {
        let envs = vec![
            mk_post(0, "bob", 100, "parent-zero"),
            mk_reply(1, "alice", 200, 0, "reply-1"),
            mk_post(2, "bob", 250, "parent-two"),
            mk_reply(3, "alice", 300, 2, "reply-3"),
            mk_reply(4, "alice", 400, 0, "reply-4"),
        ];
        let rows = compute_replies_of(&envs, "alice");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].reply_offset, 4);
        assert_eq!(rows[1].reply_offset, 3);
        assert_eq!(rows[2].reply_offset, 1);
        assert_eq!(rows[0].parent_offset, 0);
        assert_eq!(rows[0].parent_sender, "bob");
        assert_eq!(rows[0].parent_payload, "parent-zero");
        assert_eq!(rows[2].reply_payload, "reply-1");
    }

    #[test]
    fn replies_of_excludes_other_sender_and_non_replies() {
        let envs = vec![
            mk_post(0, "carol", 100, "p"),
            mk_reply(1, "alice", 200, 0, "alice-reply"),
            mk_reply(2, "bob", 300, 0, "bob-reply"),
            mk_post(3, "alice", 400, "alice-not-reply"),
        ];
        let rows = compute_replies_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reply_offset, 1);
        assert_eq!(rows[0].reply_payload, "alice-reply");
    }

    #[test]
    fn replies_of_redacted_dropped() {
        let envs = vec![
            mk_post(0, "bob", 100, "parent"),
            mk_reply(1, "alice", 200, 0, "kept"),
            mk_reply(2, "alice", 300, 0, "to-redact"),
            mk_redact(3, "alice", 400, 2),
        ];
        let rows = compute_replies_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reply_offset, 1);
    }

    #[test]
    fn replies_of_reactions_excluded() {
        use base64::Engine;
        let envs = vec![
            mk_post(0, "bob", 100, "parent"),
            mk_reply(1, "alice", 200, 0, "real-reply"),
            json!({
                "offset": 2,
                "sender_id": "alice",
                "msg_type": "reaction",
                "ts_unix_ms": 250,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode("👍"),
                "metadata": {"in_reply_to": "0"},
            }),
        ];
        let rows = compute_replies_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reply_offset, 1);
        assert_eq!(rows[0].reply_payload, "real-reply");
    }

    #[test]
    fn replies_of_missing_parent_renders_empty_parent_fields() {
        let envs = vec![
            mk_reply(5, "alice", 200, 99, "orphan-reply"),
        ];
        let rows = compute_replies_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].parent_offset, 99);
        assert_eq!(rows[0].parent_sender, "");
        assert_eq!(rows[0].parent_payload, "");
    }

    // T-1371: compute_mentions_of
    fn mk_mention(off: u64, sender: &str, ts: i64, payload: &str, mentions_csv: &str) -> Value {
        use base64::Engine;
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "post",
            "ts_unix_ms": ts,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "metadata": {"mentions": mentions_csv},
        })
    }

    #[test]
    fn mentions_of_happy_path_desc() {
        let envs = vec![
            mk_post(0, "carol", 50, "no-mention"),
            mk_mention(1, "bob", 100, "hey alice", "alice"),
            mk_mention(2, "carol", 200, "hi alice and dave", "alice,dave"),
            mk_mention(3, "bob", 300, "alice again", "alice"),
        ];
        let rows = compute_mentions_of(&envs, "alice");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].mention_offset, 3);
        assert_eq!(rows[1].mention_offset, 2);
        assert_eq!(rows[2].mention_offset, 1);
        assert_eq!(rows[0].sender_id, "bob");
        assert_eq!(rows[1].mentions_csv, "alice,dave");
    }

    #[test]
    fn mentions_of_wildcard_csv_matches_any_specific_user() {
        let envs = vec![
            mk_mention(0, "bob", 100, "@room ping", "*"),
            mk_mention(1, "carol", 200, "alice only", "alice"),
        ];
        let rows = compute_mentions_of(&envs, "alice");
        assert_eq!(rows.len(), 2);
        let rows_dave = compute_mentions_of(&envs, "dave");
        assert_eq!(rows_dave.len(), 1, "dave only matches the @room post");
        assert_eq!(rows_dave[0].mentions_csv, "*");
    }

    #[test]
    fn mentions_of_redacted_dropped() {
        let envs = vec![
            mk_mention(0, "bob", 100, "kept", "alice"),
            mk_mention(1, "bob", 200, "to-redact", "alice"),
            mk_redact(2, "bob", 300, 1),
        ];
        let rows = compute_mentions_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].mention_offset, 0);
    }

    #[test]
    fn mentions_of_non_matching_excluded() {
        let envs = vec![
            mk_post(0, "bob", 100, "no-mention-at-all"),
            mk_mention(1, "bob", 200, "hi carol", "carol"),
            mk_mention(2, "bob", 300, "alice", "alice"),
        ];
        let rows = compute_mentions_of(&envs, "alice");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].mention_offset, 2);
    }

    #[test]
    fn mentions_of_meta_msg_types_excluded() {
        use base64::Engine;
        let envs = vec![
            json!({
                "offset": 0,
                "sender_id": "bob",
                "msg_type": "reaction",
                "ts_unix_ms": 100,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode("👍"),
                "metadata": {"mentions": "alice", "in_reply_to": "5"},
            }),
            mk_mention(1, "bob", 200, "real ping", "alice"),
        ];
        let rows = compute_mentions_of(&envs, "alice");
        assert_eq!(rows.len(), 1, "reaction must not count as mention");
        assert_eq!(rows[0].mention_offset, 1);
    }

    // T-1372: compute_pin_history
    fn mk_pin_event(off: u64, sender: &str, ts: i64, target: u64, action: Option<&str>) -> Value {
        let md = match action {
            Some(a) => json!({"pin_target": target.to_string(), "action": a}),
            None => json!({"pin_target": target.to_string()}),
        };
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "pin",
            "ts_unix_ms": ts,
            "payload_b64": "",
            "metadata": md,
        })
    }

    #[test]
    fn pin_history_pin_then_unpin_two_rows_asc() {
        let envs = vec![
            mk_post(0, "alice", 100, "the-target"),
            mk_pin_event(1, "alice", 200, 0, None),
            mk_pin_event(2, "bob", 300, 0, Some("unpin")),
        ];
        let rows = compute_pin_history(&envs);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].event_offset, 1);
        assert_eq!(rows[0].action, "pin");
        assert_eq!(rows[0].target_offset, 0);
        assert_eq!(rows[0].actor_sender, "alice");
        assert_eq!(rows[0].target_payload.as_deref(), Some("the-target"));
        assert_eq!(rows[1].event_offset, 2);
        assert_eq!(rows[1].action, "unpin");
        assert_eq!(rows[1].actor_sender, "bob");
    }

    #[test]
    fn pin_history_multiple_toggles_all_preserved() {
        // Audit, not LWW — every toggle stays.
        let envs = vec![
            mk_post(0, "a", 100, "x"),
            mk_pin_event(1, "a", 200, 0, None),
            mk_pin_event(2, "a", 300, 0, Some("unpin")),
            mk_pin_event(3, "b", 400, 0, Some("pin")),
            mk_pin_event(4, "b", 500, 0, Some("unpin")),
        ];
        let rows = compute_pin_history(&envs);
        assert_eq!(rows.len(), 4);
        let actions: Vec<&str> = rows.iter().map(|r| r.action.as_str()).collect();
        assert_eq!(actions, vec!["pin", "unpin", "pin", "unpin"]);
    }

    #[test]
    fn pin_history_malformed_pin_target_skipped() {
        use base64::Engine;
        let envs = vec![
            mk_post(0, "a", 100, "x"),
            mk_pin_event(1, "a", 200, 0, None),
            json!({
                "offset": 2,
                "sender_id": "a",
                "msg_type": "pin",
                "ts_unix_ms": 300,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode(""),
                "metadata": {"pin_target": "not-a-number"},
            }),
            // pin envelope with no metadata at all
            json!({
                "offset": 3,
                "sender_id": "a",
                "msg_type": "pin",
                "ts_unix_ms": 400,
                "payload_b64": "",
            }),
        ];
        let rows = compute_pin_history(&envs);
        assert_eq!(rows.len(), 1, "only the well-formed pin event survives");
        assert_eq!(rows[0].event_offset, 1);
    }

    #[test]
    fn pin_history_default_action_is_pin() {
        let envs = vec![
            mk_post(0, "a", 100, "x"),
            mk_pin_event(1, "a", 200, 0, None), // metadata.action absent
        ];
        let rows = compute_pin_history(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].action, "pin");
    }

    #[test]
    fn pin_history_target_payload_none_when_absent() {
        // Pin pointing at offset 99 which isn't in the snapshot.
        let envs = vec![mk_pin_event(0, "a", 100, 99, None)];
        let rows = compute_pin_history(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target_offset, 99);
        assert!(rows[0].target_payload.is_none());
    }

    // T-1373: compute_redactions
    fn mk_redact_with_reason(off: u64, sender: &str, ts: i64, target: u64, reason: &str) -> Value {
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "redaction",
            "ts_unix_ms": ts,
            "payload_b64": "",
            "metadata": {"redacts": target.to_string(), "reason": reason},
        })
    }

    #[test]
    fn redactions_chronological_asc() {
        let envs = vec![
            mk_post(0, "a", 100, "first"),
            mk_post(1, "b", 200, "second"),
            mk_redact(2, "a", 300, 1),
            mk_redact(3, "b", 400, 0),
        ];
        let rows = compute_redactions(&envs);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].event_offset, 2);
        assert_eq!(rows[0].target_offset, 1);
        assert_eq!(rows[0].redactor_sender, "a");
        assert_eq!(rows[0].target_payload.as_deref(), Some("second"));
        assert_eq!(rows[1].event_offset, 3);
        assert_eq!(rows[1].target_offset, 0);
        assert_eq!(rows[1].target_payload.as_deref(), Some("first"));
    }

    #[test]
    fn redactions_with_reason() {
        let envs = vec![
            mk_post(0, "a", 100, "oops"),
            mk_redact_with_reason(1, "a", 200, 0, "wrong channel"),
        ];
        let rows = compute_redactions(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reason.as_deref(), Some("wrong channel"));
    }

    #[test]
    fn redactions_target_payload_none_when_absent() {
        // Redact offset 99 which isn't in the snapshot.
        let envs = vec![mk_redact(0, "a", 100, 99)];
        let rows = compute_redactions(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target_offset, 99);
        assert!(rows[0].target_payload.is_none());
        assert!(rows[0].reason.is_none());
    }

    // T-1374: compute_reactions_on
    fn mk_reaction(off: u64, sender: &str, ts: i64, parent: u64, emoji: &str) -> Value {
        use base64::Engine;
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "reaction",
            "ts_unix_ms": ts,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(emoji),
            "metadata": {"in_reply_to": parent.to_string()},
        })
    }

    #[test]
    fn reactions_on_two_emojis_count_desc() {
        let envs = vec![
            mk_post(0, "alice", 100, "target"),
            mk_reaction(1, "alice", 200, 0, "👍"),
            mk_reaction(2, "bob", 300, 0, "👍"),
            mk_reaction(3, "alice", 400, 0, "🎉"),
        ];
        let rows = compute_reactions_on(&envs, 0);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].emoji, "👍");
        assert_eq!(rows[0].count, 2);
        assert_eq!(rows[0].senders, vec!["alice".to_string(), "bob".to_string()]);
        assert_eq!(rows[1].emoji, "🎉");
        assert_eq!(rows[1].count, 1);
        assert_eq!(rows[1].senders, vec!["alice".to_string()]);
    }

    #[test]
    fn reactions_on_same_sender_dedup_in_senders() {
        let envs = vec![
            mk_post(0, "alice", 100, "target"),
            mk_reaction(1, "alice", 200, 0, "👍"),
            mk_reaction(2, "alice", 300, 0, "👍"), // alice taps again
        ];
        let rows = compute_reactions_on(&envs, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].count, 2, "count captures repeats");
        assert_eq!(rows[0].senders, vec!["alice".to_string()], "senders dedup");
    }

    #[test]
    fn reactions_on_redacted_excluded() {
        let envs = vec![
            mk_post(0, "alice", 100, "target"),
            mk_reaction(1, "alice", 200, 0, "👍"),
            mk_reaction(2, "bob", 300, 0, "👍"),
            mk_redact(3, "alice", 400, 2),
        ];
        let rows = compute_reactions_on(&envs, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].count, 1);
        assert_eq!(rows[0].senders, vec!["alice".to_string()]);
    }

    #[test]
    fn reactions_on_other_target_excluded() {
        let envs = vec![
            mk_post(0, "alice", 100, "target-zero"),
            mk_post(1, "alice", 150, "target-one"),
            mk_reaction(2, "alice", 200, 0, "👍"),
            mk_reaction(3, "bob", 300, 1, "🎉"),
        ];
        let rows = compute_reactions_on(&envs, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].emoji, "👍");
        let rows1 = compute_reactions_on(&envs, 1);
        assert_eq!(rows1.len(), 1);
        assert_eq!(rows1[0].emoji, "🎉");
    }

    // T-1375: compute_edit_stats
    fn mk_edit_event(off: u64, sender: &str, ts: i64, target: u64, new_text: &str) -> Value {
        use base64::Engine;
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "edit",
            "ts_unix_ms": ts,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(new_text),
            "metadata": {"replaces": target.to_string()},
        })
    }

    #[test]
    fn edit_stats_single_target_three_edits() {
        let envs = vec![
            mk_post(0, "alice", 100, "v0"),
            mk_edit_event(1, "alice", 200, 0, "v1"),
            mk_edit_event(2, "bob", 300, 0, "v2"),
            mk_edit_event(3, "alice", 400, 0, "v3"),
        ];
        let rows = compute_edit_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target_offset, 0);
        assert_eq!(rows[0].edit_count, 3);
        assert_eq!(rows[0].latest_editor, "alice", "alice's offset 3 edit at ts 400 wins");
        assert_eq!(rows[0].latest_ts_ms, 400);
        assert_eq!(rows[0].target_payload, "v0", "target_payload is the ORIGINAL");
    }

    #[test]
    fn edit_stats_two_targets_sorted_desc() {
        let envs = vec![
            mk_post(0, "alice", 100, "tgt-zero"),
            mk_post(1, "alice", 150, "tgt-one"),
            mk_edit_event(2, "alice", 200, 0, "z1"),
            mk_edit_event(3, "alice", 300, 0, "z2"),
            mk_edit_event(4, "bob", 400, 1, "o1"),
        ];
        let rows = compute_edit_stats(&envs);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].target_offset, 0);
        assert_eq!(rows[0].edit_count, 2);
        assert_eq!(rows[1].target_offset, 1);
        assert_eq!(rows[1].edit_count, 1);
    }

    #[test]
    fn edit_stats_redacted_edit_excluded() {
        let envs = vec![
            mk_post(0, "alice", 100, "v0"),
            mk_edit_event(1, "alice", 200, 0, "v1"),
            mk_edit_event(2, "alice", 300, 0, "to-redact"),
            mk_redact(3, "alice", 400, 2),
        ];
        let rows = compute_edit_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].edit_count, 1, "redacted edit doesn't count");
        assert_eq!(rows[0].latest_editor, "alice");
    }

    #[test]
    fn edit_stats_redacted_target_drops_row() {
        let envs = vec![
            mk_post(0, "alice", 100, "v0"),
            mk_post(1, "bob", 150, "still-here"),
            mk_edit_event(2, "alice", 200, 0, "v1"),
            mk_edit_event(3, "alice", 300, 1, "b1"),
            mk_redact(4, "alice", 400, 0), // target 0 redacted
        ];
        let rows = compute_edit_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target_offset, 1);
    }

    #[test]
    fn edit_stats_malformed_replaces_skipped() {
        use base64::Engine;
        let envs = vec![
            mk_post(0, "a", 100, "v0"),
            json!({
                "offset": 1,
                "sender_id": "a",
                "msg_type": "edit",
                "ts_unix_ms": 200,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode("v1"),
                "metadata": {"replaces": "not-a-number"},
            }),
            mk_edit_event(2, "a", 300, 0, "v2"),
        ];
        let rows = compute_edit_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].edit_count, 1);
    }

    #[test]
    fn redactions_malformed_redacts_skipped() {
        use base64::Engine;
        let envs = vec![
            mk_post(0, "a", 100, "x"),
            // bogus redacts (non-numeric)
            json!({
                "offset": 1,
                "sender_id": "a",
                "msg_type": "redaction",
                "ts_unix_ms": 200,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode(""),
                "metadata": {"redacts": "not-a-number"},
            }),
            mk_redact(2, "a", 300, 0),
        ];
        let rows = compute_redactions(&envs);
        assert_eq!(rows.len(), 1, "only the well-formed redaction survives");
        assert_eq!(rows[0].event_offset, 2);
    }

    // T-1376: compute_state — canonical Matrix-style render

    #[test]
    fn state_empty_topic_yields_no_rows() {
        let envs: Vec<Value> = vec![];
        let rows = compute_state(&envs, false);
        assert!(rows.is_empty());
    }

    #[test]
    fn state_single_post_unedited() {
        let envs = vec![mk_post(0, "alice", 100, "hello")];
        let rows = compute_state(&envs, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].offset, 0);
        assert_eq!(rows[0].sender_id, "alice");
        assert_eq!(rows[0].payload, "hello");
        assert!(!rows[0].is_edited);
        assert_eq!(rows[0].edit_count, 0);
        assert!(!rows[0].is_redacted);
        assert_eq!(rows[0].ts_ms, 100);
    }

    #[test]
    fn state_single_edit_collapses_to_latest_text() {
        let envs = vec![
            mk_post(0, "alice", 100, "v0"),
            mk_edit_event(1, "alice", 200, 0, "v1"),
        ];
        let rows = compute_state(&envs, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].offset, 0);
        assert_eq!(rows[0].payload, "v1", "latest edit text wins");
        assert!(rows[0].is_edited);
        assert_eq!(rows[0].edit_count, 1);
        assert_eq!(rows[0].latest_edit_ts_ms, 200);
        assert_eq!(rows[0].ts_ms, 100, "ts_ms is the original post's ts");
    }

    #[test]
    fn state_two_edits_latest_ts_wins() {
        let envs = vec![
            mk_post(0, "alice", 100, "v0"),
            mk_edit_event(1, "alice", 200, 0, "v1"),
            mk_edit_event(2, "bob", 300, 0, "v2"),
        ];
        let rows = compute_state(&envs, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].payload, "v2");
        assert_eq!(rows[0].edit_count, 2);
        assert_eq!(rows[0].latest_edit_ts_ms, 300);
    }

    #[test]
    fn state_redacted_dropped_by_default() {
        let envs = vec![
            mk_post(0, "alice", 100, "secret"),
            mk_post(1, "alice", 150, "kept"),
            mk_redact(2, "alice", 200, 0),
        ];
        let rows = compute_state(&envs, false);
        assert_eq!(rows.len(), 1, "redacted offset 0 is dropped");
        assert_eq!(rows[0].offset, 1);
        assert_eq!(rows[0].payload, "kept");
    }

    #[test]
    fn state_redacted_shown_when_include_redacted_true() {
        let envs = vec![
            mk_post(0, "alice", 100, "secret"),
            mk_post(1, "alice", 150, "kept"),
            mk_redact(2, "alice", 200, 0),
        ];
        let rows = compute_state(&envs, true);
        assert_eq!(rows.len(), 2);
        let redacted_row = rows.iter().find(|r| r.offset == 0).unwrap();
        assert!(redacted_row.is_redacted);
        assert_eq!(redacted_row.payload, "[REDACTED]");
        assert!(!redacted_row.is_edited);
    }

    #[test]
    fn state_meta_envelopes_skipped() {
        let envs = vec![
            mk_post(0, "alice", 100, "real"),
            mk_reaction(1, "bob", 150, 0, "👍"),
            mk_redact(2, "alice", 200, 9999), // redaction targeting unknown
        ];
        let rows = compute_state(&envs, false);
        assert_eq!(rows.len(), 1, "reaction + redaction envelopes are not content rows");
        assert_eq!(rows[0].offset, 0);
    }

    #[test]
    fn state_redacted_edit_does_not_apply() {
        // Edit at offset 2 is itself redacted -> shouldn't update payload.
        let envs = vec![
            mk_post(0, "alice", 100, "v0"),
            mk_edit_event(1, "alice", 200, 0, "v1"),
            mk_edit_event(2, "alice", 300, 0, "v-bogus"),
            mk_redact(3, "alice", 400, 2),
        ];
        let rows = compute_state(&envs, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].payload, "v1", "the surviving edit wins, not the redacted one");
        assert_eq!(rows[0].edit_count, 1);
    }

    #[test]
    fn state_offset_asc_order() {
        let envs = vec![
            mk_post(0, "alice", 100, "first"),
            mk_post(1, "bob", 200, "second"),
            mk_post(2, "carol", 300, "third"),
        ];
        let rows = compute_state(&envs, false);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].offset, 0);
        assert_eq!(rows[1].offset, 1);
        assert_eq!(rows[2].offset, 2);
    }

    // T-1377: compute_ack_history — chronological receipt audit log

    fn mk_receipt(off: u64, sender: &str, ts: i64, up_to: u64) -> Value {
        json!({
            "offset": off,
            "sender_id": sender,
            "msg_type": "receipt",
            "ts_unix_ms": ts,
            "payload_b64": "",
            "metadata": {"up_to": up_to.to_string()},
        })
    }

    #[test]
    fn ack_history_empty_topic_yields_no_rows() {
        let envs: Vec<Value> = vec![];
        let rows = compute_ack_history(&envs, None);
        assert!(rows.is_empty());
    }

    #[test]
    fn ack_history_includes_only_receipts() {
        let envs = vec![
            mk_post(0, "alice", 100, "msg"),
            mk_receipt(1, "bob", 200, 0),
        ];
        let rows = compute_ack_history(&envs, None);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sender_id, "bob");
        assert_eq!(rows[0].up_to, 0);
        assert_eq!(rows[0].receipt_offset, 1);
    }

    #[test]
    fn ack_history_sorts_ts_asc() {
        let envs = vec![
            mk_post(0, "alice", 100, "m"),
            mk_receipt(1, "bob", 300, 0),
            mk_receipt(2, "carol", 200, 0),
            mk_receipt(3, "dave", 400, 0),
        ];
        let rows = compute_ack_history(&envs, None);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].sender_id, "carol", "ts=200 first");
        assert_eq!(rows[1].sender_id, "bob");
        assert_eq!(rows[2].sender_id, "dave");
    }

    #[test]
    fn ack_history_user_filter() {
        let envs = vec![
            mk_post(0, "alice", 100, "m"),
            mk_receipt(1, "bob", 200, 0),
            mk_receipt(2, "carol", 300, 0),
            mk_receipt(3, "bob", 400, 0),
        ];
        let rows = compute_ack_history(&envs, Some("bob"));
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.sender_id == "bob"));
        assert_eq!(rows[0].ts_ms, 200);
        assert_eq!(rows[1].ts_ms, 400);
    }

    #[test]
    fn ack_history_malformed_up_to_skipped() {
        let envs = vec![
            mk_post(0, "alice", 100, "m"),
            json!({
                "offset": 1,
                "sender_id": "bob",
                "msg_type": "receipt",
                "ts_unix_ms": 200,
                "payload_b64": "",
                "metadata": {"up_to": "not-a-number"},
            }),
            mk_receipt(2, "bob", 300, 1),
        ];
        let rows = compute_ack_history(&envs, None);
        assert_eq!(rows.len(), 1, "only well-formed receipt survives");
        assert_eq!(rows[0].receipt_offset, 2);
    }

    #[test]
    fn ack_history_offset_tiebreak_when_ts_equal() {
        let envs = vec![
            mk_receipt(5, "alice", 100, 0),
            mk_receipt(2, "bob", 100, 0),
            mk_receipt(8, "carol", 100, 0),
        ];
        let rows = compute_ack_history(&envs, None);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].receipt_offset, 2);
        assert_eq!(rows[1].receipt_offset, 5);
        assert_eq!(rows[2].receipt_offset, 8);
    }

    // T-1378: compute_snapshot — point-in-time canonical view

    #[test]
    fn snapshot_empty_topic() {
        let envs: Vec<Value> = vec![];
        let rows = compute_snapshot(&envs, 1000, false);
        assert!(rows.is_empty());
    }

    #[test]
    fn snapshot_before_first_message_is_empty() {
        let envs = vec![
            mk_post(0, "alice", 500, "hello"),
        ];
        let rows = compute_snapshot(&envs, 100, false);
        assert!(rows.is_empty(), "as_of=100 < first ts=500 → no content");
    }

    #[test]
    fn snapshot_at_message_ts_includes_it() {
        let envs = vec![
            mk_post(0, "alice", 500, "hello"),
            mk_post(1, "bob", 1000, "world"),
        ];
        let rows = compute_snapshot(&envs, 500, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].offset, 0, "ts=500 inclusive");
    }

    #[test]
    fn snapshot_edit_after_cutoff_not_applied() {
        // post at 100, edit at 500 → snapshot at 300 should show ORIGINAL
        let envs = vec![
            mk_post(0, "alice", 100, "v0-original"),
            mk_edit_event(1, "alice", 500, 0, "v1-later"),
        ];
        let rows = compute_snapshot(&envs, 300, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].payload, "v0-original", "edit at ts=500 hadn't happened by as_of=300");
        assert!(!rows[0].is_edited);
        assert_eq!(rows[0].edit_count, 0);
    }

    #[test]
    fn snapshot_edit_before_cutoff_is_applied() {
        // post at 100, edit at 200 → snapshot at 300 sees edit applied
        let envs = vec![
            mk_post(0, "alice", 100, "v0"),
            mk_edit_event(1, "alice", 200, 0, "v1"),
        ];
        let rows = compute_snapshot(&envs, 300, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].payload, "v1", "edit at ts=200 already happened by as_of=300");
        assert!(rows[0].is_edited);
        assert_eq!(rows[0].edit_count, 1);
    }

    #[test]
    fn snapshot_redaction_after_cutoff_not_applied() {
        // post at 100, redact at 500 → snapshot at 300 still shows original
        let envs = vec![
            mk_post(0, "alice", 100, "still-here"),
            mk_redact(1, "alice", 500, 0),
        ];
        let rows = compute_snapshot(&envs, 300, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].payload, "still-here", "redaction at ts=500 hadn't happened");
        assert!(!rows[0].is_redacted);
    }

    #[test]
    fn snapshot_redaction_before_cutoff_is_applied() {
        let envs = vec![
            mk_post(0, "alice", 100, "doomed"),
            mk_post(1, "alice", 150, "kept"),
            mk_redact(2, "alice", 200, 0),
        ];
        let rows = compute_snapshot(&envs, 300, false);
        assert_eq!(rows.len(), 1, "redacted offset 0 dropped at as_of=300");
        assert_eq!(rows[0].offset, 1);
    }

    // T-1379: compute_quote_stats — per-target reply rollup

    #[test]
    fn quote_stats_empty_yields_no_rows() {
        let envs: Vec<Value> = vec![];
        let rows = compute_quote_stats(&envs);
        assert!(rows.is_empty());
    }

    #[test]
    fn quote_stats_single_reply() {
        let envs = vec![
            mk_post(0, "alice", 100, "tgt"),
            mk_reply(1, "bob", 200, 0, "lgtm"),
        ];
        let rows = compute_quote_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target_offset, 0);
        assert_eq!(rows[0].reply_count, 1);
        assert_eq!(rows[0].distinct_repliers, vec!["bob"]);
        assert_eq!(rows[0].latest_reply_ts_ms, 200);
        assert_eq!(rows[0].target_payload, "tgt");
    }

    #[test]
    fn quote_stats_multi_reply_distinct_senders() {
        let envs = vec![
            mk_post(0, "alice", 100, "popular"),
            mk_reply(1, "bob", 200, 0, "r1"),
            mk_reply(2, "carol", 300, 0, "r2"),
            mk_reply(3, "bob", 400, 0, "r3"),
        ];
        let rows = compute_quote_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reply_count, 3);
        assert_eq!(rows[0].distinct_repliers, vec!["bob", "carol"], "bob deduped + sorted");
        assert_eq!(rows[0].latest_reply_ts_ms, 400);
    }

    #[test]
    fn quote_stats_two_targets_sorted_by_count() {
        let envs = vec![
            mk_post(0, "alice", 100, "first"),
            mk_post(1, "alice", 110, "second"),
            mk_reply(2, "bob", 200, 0, "r1"),
            mk_reply(3, "carol", 210, 1, "r2"),
            mk_reply(4, "bob", 220, 1, "r3"),
            mk_reply(5, "dave", 230, 1, "r4"),
        ];
        let rows = compute_quote_stats(&envs);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].target_offset, 1, "target 1 has 3 replies, comes first");
        assert_eq!(rows[0].reply_count, 3);
        assert_eq!(rows[1].target_offset, 0);
        assert_eq!(rows[1].reply_count, 1);
    }

    #[test]
    fn quote_stats_reactions_not_counted() {
        let envs = vec![
            mk_post(0, "alice", 100, "tgt"),
            mk_reaction(1, "bob", 150, 0, "👍"),
            mk_reply(2, "carol", 200, 0, "real reply"),
        ];
        let rows = compute_quote_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reply_count, 1, "reaction does not count");
        assert_eq!(rows[0].distinct_repliers, vec!["carol"]);
    }

    #[test]
    fn quote_stats_redacted_reply_excluded() {
        let envs = vec![
            mk_post(0, "alice", 100, "tgt"),
            mk_reply(1, "bob", 200, 0, "real"),
            mk_reply(2, "carol", 300, 0, "to-be-redacted"),
            mk_redact(3, "carol", 400, 2),
        ];
        let rows = compute_quote_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reply_count, 1, "redacted reply doesn't count");
    }

    #[test]
    fn quote_stats_redacted_target_drops_row() {
        let envs = vec![
            mk_post(0, "alice", 100, "doomed"),
            mk_post(1, "alice", 110, "kept"),
            mk_reply(2, "bob", 200, 0, "ignored"),
            mk_reply(3, "bob", 210, 1, "ok"),
            mk_redact(4, "alice", 300, 0),
        ];
        let rows = compute_quote_stats(&envs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target_offset, 1);
    }

    // T-1380: summarize_members_as_of — retro membership query

    #[test]
    fn members_as_of_none_matches_existing() {
        let envs = vec![
            mk_post(0, "alice", 100, "p"),
            mk_post(1, "bob", 200, "p"),
        ];
        let baseline = summarize_members(&envs, false);
        let with_none = summarize_members_as_of(&envs, false, None);
        assert_eq!(baseline.len(), with_none.len());
        for (a, b) in baseline.iter().zip(with_none.iter()) {
            assert_eq!(a.sender_id, b.sender_id);
            assert_eq!(a.posts, b.posts);
            assert_eq!(a.first_ts, b.first_ts);
            assert_eq!(a.last_ts, b.last_ts);
        }
    }

    #[test]
    fn members_as_of_before_history_is_empty() {
        let envs = vec![
            mk_post(0, "alice", 500, "p"),
            mk_post(1, "bob", 600, "p"),
        ];
        let rows = summarize_members_as_of(&envs, false, Some(100));
        assert!(rows.is_empty());
    }

    #[test]
    fn members_as_of_mid_history_partial() {
        let envs = vec![
            mk_post(0, "alice", 100, "p"),
            mk_post(1, "bob", 200, "p"),
            mk_post(2, "carol", 300, "p"),
        ];
        let rows = summarize_members_as_of(&envs, false, Some(250));
        assert_eq!(rows.len(), 2, "alice + bob; carol's post at ts=300 not yet");
        let senders: Vec<&str> = rows.iter().map(|r| r.sender_id.as_str()).collect();
        assert!(senders.contains(&"alice"));
        assert!(senders.contains(&"bob"));
        assert!(!senders.contains(&"carol"));
    }

    #[test]
    fn members_as_of_excludes_sender_only_after_cutoff() {
        let envs = vec![
            mk_post(0, "alice", 100, "early"),
            mk_post(1, "alice", 500, "late"),
            mk_post(2, "bob", 600, "even-later"),
        ];
        // as_of=200: alice has 1 post (ts=100), bob has none yet
        let rows = summarize_members_as_of(&envs, false, Some(200));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sender_id, "alice");
        assert_eq!(rows[0].posts, 1);
    }

    #[test]
    fn members_as_of_inclusive_at_cutoff() {
        let envs = vec![
            mk_post(0, "alice", 100, "p"),
            mk_post(1, "bob", 200, "p"),
        ];
        let rows = summarize_members_as_of(&envs, false, Some(200));
        assert_eq!(rows.len(), 2, "ts=200 inclusive");
    }

    #[test]
    fn snapshot_partial_history_walks_correctly() {
        // 3 posts, 1 edit, 1 redact at staggered times
        let envs = vec![
            mk_post(0, "alice", 100, "p0"),
            mk_post(1, "alice", 200, "p1"),
            mk_post(2, "alice", 300, "p2"),
            mk_edit_event(3, "alice", 400, 0, "p0-edited"),
            mk_redact(4, "alice", 500, 1),
        ];
        // as_of=250: only p0 and p1 visible, no edit/redact yet
        let rows = compute_snapshot(&envs, 250, false);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].payload, "p0");
        assert!(!rows[0].is_edited);
        assert_eq!(rows[1].payload, "p1");
        // as_of=450: p0 edited, p1 still here (redact at 500 hasn't happened), p2 here
        let rows = compute_snapshot(&envs, 450, false);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].payload, "p0-edited");
        assert!(rows[0].is_edited);
        // as_of=600: everything happened
        let rows = compute_snapshot(&envs, 600, false);
        assert_eq!(rows.len(), 2, "p1 redacted at 500");
        assert_eq!(rows[0].offset, 0);
        assert_eq!(rows[0].payload, "p0-edited");
        assert_eq!(rows[1].offset, 2);
    }
}
