//! T-1220 wedge a (T-1225): inbox.* receiver migration helper. Channel-only
//! post T-1166 cut (T-1415 AC3 cleanup).
//!
//! Single async entry point that callers (CLI local, CLI remote, MCP) use to
//! read / aggregate / trim inbox topics via `channel.subscribe` /
//! `channel.list` / `channel.trim`. Reassembles per-transfer summaries from
//! the file.init/chunk/complete event stream mirrored by T-1163
//! (`channel::mirror_inbox_deposit`).
//!
//! Naming: the `_with_fallback` suffix is historical (T-1225 era when a legacy
//! `inbox.list`/`inbox.status`/`inbox.clear` RPC was the fallback). The hub
//! source no longer exposes those legacy methods (T-1166 / T-1415 retirement),
//! and the fleet is uniformly on a build that speaks `channel.*`. A hub that
//! responds with `MethodNotFound` to `channel.subscribe` / `channel.list` /
//! `channel.trim` is treated as a hard error rather than triggering a
//! (non-existent) fallback path. Rename to drop `_with_fallback` is a separate
//! cleanup slice.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use termlink_protocol::jsonrpc::RpcResponse;
use termlink_protocol::{control, TransportAddr};

use crate::client::{Client, ClientError};
use crate::hub_capabilities::HubCapabilitiesCache;

const TOPIC_PREFIX: &str = "inbox:";
const RPC_METHOD_NOT_FOUND: i64 = -32601;

/// Summary of one pending transfer in a target's inbox. Mirrors the
/// `inbox::PendingTransfer` fields that current renderers actually consume so
/// callers swap `inbox.list` → `list_via_channel` without touching display
/// code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InboxEntry {
    pub transfer_id: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub chunks_received: u32,
    #[serde(default)]
    pub total_chunks: u32,
    #[serde(default)]
    pub complete: bool,
}

/// Per-call mutable state — cursor map + warn-once tracker + legacy-only flag.
/// Callers that want process-wide sharing wrap a single instance in their own
/// `Mutex`; T-1225 keeps construction explicit so test setups can stage cursors
/// without globals.
///
/// T-1415: `inbox_channel.rs` no longer consults `legacy_only_peers`
/// (channel-only since T-1166 cut). The field + flag_legacy_only / is_legacy_only
/// methods are retained because `artifact.rs` still gates `file.*` event-emit
/// fallback on them; retiring that path is a separate slice.
#[derive(Debug, Default)]
pub struct FallbackCtx {
    cursors: HashMap<String, u64>,
    warned: HashSet<(String, &'static str)>,
    legacy_only_peers: HashSet<String>,
    /// T-3308: retention gaps seen while resuming from a cursor this ctx saved
    /// itself — records the hub deleted before this reader got them.
    retention_gaps: Vec<RetentionGap>,
}

/// T-3308: one retention gap observed by the inbox reader. Only recorded when
/// the reader resumed from its OWN saved cursor: a fresh read from 0 on a swept
/// topic is not a loss (T-3307's rule — the hub reports, the client judges).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetentionGap {
    pub topic: String,
    pub requested_cursor: u64,
    pub oldest_offset: u64,
    pub skipped: u64,
}

impl FallbackCtx {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a warn-once event. Returns `true` the first time `(host_port, kind)`
    /// is seen, `false` thereafter — caller emits the tracing line on `true`.
    pub fn warn_once(&mut self, host_port: &str, kind: &'static str) -> bool {
        self.warned.insert((host_port.to_string(), kind))
    }

    /// Mark a peer as legacy-only so future calls skip the channel.* dispatch.
    /// T-1415: no longer consulted by `inbox_channel.rs`; remains in use by
    /// `artifact.rs` for the `file.*` event-emit fallback path.
    pub fn flag_legacy_only(&mut self, host_port: &str) {
        self.legacy_only_peers.insert(host_port.to_string());
    }

    pub fn is_legacy_only(&self, host_port: &str) -> bool {
        self.legacy_only_peers.contains(host_port)
    }

    /// T-3308: drain the retention gaps recorded since the last call, so a
    /// caller can tell the operator that records were lost before delivery.
    pub fn take_retention_gaps(&mut self) -> Vec<RetentionGap> {
        std::mem::take(&mut self.retention_gaps)
    }

    #[cfg(test)]
    pub fn set_cursor(&mut self, topic: impl Into<String>, cursor: u64) {
        self.cursors.insert(topic.into(), cursor);
    }

    #[cfg(test)]
    pub fn cursor(&self, topic: &str) -> Option<u64> {
        self.cursors.get(topic).copied()
    }
}

/// Dispatch + reassemble. Single entry point for callers using an
/// unauthenticated socket. Opens one `Client::connect_addr` for the dispatch.
///
/// For callers that already hold an authenticated `Client` (CLI remote / MCP
/// remote), use `list_via_channel_with_client` instead.
///
/// T-1415: name is historical (T-1166 cut removed the legacy `inbox.list`
/// fallback). Hub must support `channel.subscribe`; `MethodNotFound` is now
/// a hard error.
pub async fn list_via_channel(
    addr: &TransportAddr,
    target: &str,
    cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<Vec<InboxEntry>> {
    let host_port = host_port_str(addr);
    let mut client = Client::connect_addr(addr)
        .await
        .map_err(|e| io::Error::other(format!("connect {host_port}: {e}")))?;
    list_via_channel_with_client(&mut client, &host_port, target, cache, ctx).await
}

/// T-1231: Variant for callers who already hold an authenticated `Client`
/// (CLI remote `cmd_remote_inbox_*`, MCP remote `termlink_remote_inbox_*`).
/// Caller supplies the `host_port` string for warn-once dedup.
///
/// T-1415: cache parameter retained for ABI stability across the 12 callsites;
/// pre-call cap probe was the only consumer and has been removed alongside
/// the legacy fallback path. The hub MUST advertise / serve
/// `channel.subscribe`; `MethodNotFound` is surfaced as an error.
pub async fn list_via_channel_with_client(
    client: &mut Client,
    host_port: &str,
    target: &str,
    _cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<Vec<InboxEntry>> {
    let topic = format!("{TOPIC_PREFIX}{target}");
    let own_cursor = ctx.cursors.get(&topic).copied();
    let saved_cursor = own_cursor.unwrap_or(0);
    let walked = page_through_subscribe(
        saved_cursor,
        &mut ClientPages { client, topic: &topic },
    )
    .await;
    match walked {
        Ok((messages, next_cursor, first_gap)) => {
            note_retention_gap(ctx, host_port, &topic, own_cursor, first_gap);
            if ctx.warn_once(host_port, "channel.subscribe") {
                tracing::info!(
                    host = %host_port,
                    target = %target,
                    "T-1225: using channel.subscribe"
                );
            }
            ctx.cursors.insert(topic, next_cursor);
            Ok(fold_envelopes(&messages))
        }
        Err(SubscribeError::MethodNotFound) => Err(io::Error::other(format!(
            "hub at {host_port} does not support channel.subscribe — required since T-1166 retired legacy inbox.list (T-1415). Upgrade the remote hub."
        ))),
        Err(SubscribeError::Other(e)) => Err(e),
    }
}

/// Aggregate inbox status, rebuilt by the channel path from a
/// `channel.list(prefix="inbox:")` reply (T-1229b / T-1235).
///
/// **This is a RECORD count, not a transfer count (T-3197).** `channel.list`
/// returns each topic's total record count and carries no `msg_type`
/// breakdown, so the number here includes every envelope on an `inbox:*`
/// topic — `file.init`/`file.chunk` transfer envelopes *and* ordinary messages
/// such as `sidecar.consult` and `note`, which share the same topic.
///
/// The fields were previously named `total_transfers` / `pending` and the CLI
/// printed them as "N pending transfer(s)". That was false and it was
/// dangerous: measured 2026-09-28, this reported 232 "pending transfers"
/// across 22 targets while `inbox list` returned **zero** transfers for every
/// one of them. The 232 were live messages on `retention: forever` topics,
/// including 49 unread inbound consults — and `inbox clear` resolves to
/// `channel.trim`, so the mislabel came within one command of authorising
/// their deletion. A metric that names a benign-sounding unit for something
/// else does not merely mislead; it recruits the operator into destroying what
/// it miscounted (PL-391).
///
/// For an actual pending-transfer count, use `list_via_channel` /
/// `inbox list <target>`, which folds envelopes by `msg_type`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct InboxStatus {
    /// Total records across all `inbox:*` topics. NOT a transfer count.
    /// `serde(alias)` keeps older stored/piped payloads parseable; the
    /// serialized name changes deliberately (PL-244).
    #[serde(alias = "total_transfers")]
    pub total_records: u64,
    pub targets: Vec<InboxStatusTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InboxStatusTarget {
    pub target: String,
    /// Records on this target's topic. NOT a transfer count — see `InboxStatus`.
    #[serde(alias = "pending")]
    pub records: u64,
}

/// Dispatch + aggregate. Single entry point for inbox.status callers using an
/// unauthenticated socket. Opens one `Client::connect_addr` for the dispatch.
///
/// For callers holding an authenticated `Client`, use
/// `status_via_channel_with_client` (remote variants).
///
/// T-1415: name historical; channel-only post T-1166 cut.
pub async fn status_via_channel(
    addr: &TransportAddr,
    cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<InboxStatus> {
    let host_port = host_port_str(addr);
    let mut client = Client::connect_addr(addr)
        .await
        .map_err(|e| io::Error::other(format!("connect {host_port}: {e}")))?;
    status_via_channel_with_client(&mut client, &host_port, cache, ctx).await
}

/// T-1235: Variant for callers who already hold an authenticated `Client`.
/// T-1415: channel-only — cache retained for ABI; `MethodNotFound` is an error.
pub async fn status_via_channel_with_client(
    client: &mut Client,
    host_port: &str,
    _cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<InboxStatus> {
    match call_channel_list_via_client(client, TOPIC_PREFIX).await {
        Ok(value) => {
            if ctx.warn_once(host_port, "channel.list") {
                tracing::info!(host = %host_port, "T-1235: using channel.list");
            }
            Ok(aggregate_status_from_channel_list(&value))
        }
        Err(SubscribeError::MethodNotFound) => Err(io::Error::other(format!(
            "hub at {host_port} does not support channel.list — required since T-1166 retired legacy inbox.status (T-1415). Upgrade the remote hub."
        ))),
        Err(SubscribeError::Other(e)) => Err(e),
    }
}

/// Pure aggregation: sum per-topic RECORD counts from a `channel.list` reply
/// (filtered to `inbox:` prefix) into the InboxStatus shape. Strips the
/// `inbox:` prefix to derive target names. Public so dual-read mergers and
/// tests can drive it without a transport.
///
/// T-3197: the summed `count` is every record on the topic regardless of
/// `msg_type`. Do not reintroduce a "transfers" reading of this number — the
/// information needed to compute one is not present in a `channel.list` reply.
pub fn aggregate_status_from_channel_list(channel_list_result: &Value) -> InboxStatus {
    let topics = channel_list_result
        .get("topics")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut targets: Vec<InboxStatusTarget> = Vec::new();
    let mut total: u64 = 0;
    for t in topics {
        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let target = match name.strip_prefix(TOPIC_PREFIX) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        let records = t.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        total += records;
        targets.push(InboxStatusTarget { target, records });
    }
    InboxStatus {
        total_records: total,
        targets,
    }
}

async fn call_channel_list_via_client(
    client: &mut Client,
    prefix: &str,
) -> Result<Value, SubscribeError> {
    let resp = client
        .call(
            control::method::CHANNEL_LIST,
            json!("inbox-status-list"),
            json!({"prefix": prefix}),
        )
        .await
        .map_err(|e| SubscribeError::Other(map_client_err("channel.list", e)))?;
    match resp {
        RpcResponse::Success(ok) => Ok(ok.result),
        RpcResponse::Error(e) if e.error.code == RPC_METHOD_NOT_FOUND => {
            Err(SubscribeError::MethodNotFound)
        }
        RpcResponse::Error(e) => Err(SubscribeError::Other(io::Error::other(format!(
            "channel.list error {}: {}",
            e.error.code, e.error.message
        )))),
    }
}

/// Result of an inbox clear operation. Same shape as legacy `inbox.clear`
/// reply (`{cleared, target}`) so the migration is a drop-in for renderers
/// that read those two fields. T-1230c.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InboxClearResult {
    pub cleared: u64,
    pub target: String,
}

/// Selector for `clear_via_channel`: clear one target's spool, or all
/// inbox targets in one call. T-1230c.
#[derive(Debug, Clone)]
pub enum ClearScope {
    Target(String),
    All,
}

/// Dispatch + trim. Single entry point for inbox.clear callers using an
/// unauthenticated socket. Opens one `Client::connect_addr` for the dispatch.
///
/// For callers holding an authenticated `Client`, use
/// `clear_via_channel_with_client` (remote variants).
///
/// T-1415: name historical; channel-only post T-1166 cut.
pub async fn clear_via_channel(
    addr: &TransportAddr,
    scope: ClearScope,
    cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<InboxClearResult> {
    let host_port = host_port_str(addr);
    let mut client = Client::connect_addr(addr)
        .await
        .map_err(|e| io::Error::other(format!("connect {host_port}: {e}")))?;
    clear_via_channel_with_client(&mut client, &host_port, scope, cache, ctx).await
}

/// T-1230c: Variant for callers who already hold an authenticated `Client`.
/// T-1415: channel-only — cache retained for ABI; `MethodNotFound` is an error.
pub async fn clear_via_channel_with_client(
    client: &mut Client,
    host_port: &str,
    scope: ClearScope,
    _cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<InboxClearResult> {
    match clear_via_channel_trim(client, &scope).await {
        Ok(result) => {
            if ctx.warn_once(host_port, "channel.trim") {
                tracing::info!(host = %host_port, scope = ?scope, "T-1230c: using channel.trim");
            }
            Ok(result)
        }
        Err(SubscribeError::MethodNotFound) => Err(io::Error::other(format!(
            "hub at {host_port} does not support channel.trim — required since T-1166 retired legacy inbox.clear (T-1415). Upgrade the remote hub."
        ))),
        Err(SubscribeError::Other(e)) => Err(e),
    }
}

/// Channel-side clear: trim one topic, or list+trim every `inbox:*` topic.
async fn clear_via_channel_trim(
    client: &mut Client,
    scope: &ClearScope,
) -> Result<InboxClearResult, SubscribeError> {
    match scope {
        ClearScope::Target(target) => {
            let topic = format!("{TOPIC_PREFIX}{target}");
            let value = call_channel_trim_via_client(client, &topic).await?;
            let deleted = value.get("deleted").and_then(|v| v.as_u64()).unwrap_or(0);
            Ok(InboxClearResult {
                cleared: deleted,
                target: target.clone(),
            })
        }
        ClearScope::All => {
            let list = call_channel_list_via_client(client, TOPIC_PREFIX).await?;
            let topics = topics_with_inbox_prefix(&list);
            let mut total: u64 = 0;
            for topic in topics {
                let value = call_channel_trim_via_client(client, &topic).await?;
                total += value.get("deleted").and_then(|v| v.as_u64()).unwrap_or(0);
            }
            Ok(InboxClearResult {
                cleared: total,
                target: "all".to_string(),
            })
        }
    }
}

/// Pure: filter a `channel.list` reply to the topic names that start with
/// the inbox prefix. Public so dual-read mergers and tests can drive it
/// without a transport.
pub fn topics_with_inbox_prefix(channel_list_result: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let topics = match channel_list_result.get("topics").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return out,
    };
    for t in topics {
        let name = match t.get("name").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };
        if name.starts_with(TOPIC_PREFIX) && name.len() > TOPIC_PREFIX.len() {
            out.push(name.to_string());
        }
    }
    out
}

async fn call_channel_trim_via_client(
    client: &mut Client,
    topic: &str,
) -> Result<Value, SubscribeError> {
    let resp = client
        .call(
            control::method::CHANNEL_TRIM,
            json!("inbox-clear-trim"),
            json!({"topic": topic}),
        )
        .await
        .map_err(|e| SubscribeError::Other(map_client_err("channel.trim", e)))?;
    match resp {
        RpcResponse::Success(ok) => Ok(ok.result),
        RpcResponse::Error(e) if e.error.code == RPC_METHOD_NOT_FOUND => {
            Err(SubscribeError::MethodNotFound)
        }
        RpcResponse::Error(e) => Err(SubscribeError::Other(io::Error::other(format!(
            "channel.trim error {}: {}",
            e.error.code, e.error.message
        )))),
    }
}

/// Internal error type for the channel-subscribe leg so the channel-side
/// callers can react to method-not-found without parsing strings.
enum SubscribeError {
    MethodNotFound,
    Other(io::Error),
}

fn map_client_err(label: &str, e: ClientError) -> io::Error {
    io::Error::other(format!("{label}: {e}"))
}

pub(crate) async fn probe_caps_via_client(
    client: &mut Client,
    host_port: &str,
    cache: &HubCapabilitiesCache,
) -> io::Result<Vec<String>> {
    if let Some(cached) = cache.get(host_port) {
        return Ok(cached);
    }
    let resp = client
        .call(
            control::method::HUB_CAPABILITIES,
            json!("inbox-probe"),
            json!({}),
        )
        .await
        .map_err(|e| map_client_err("hub.capabilities", e))?;
    let methods = extract_methods(&resp)?;
    cache.set(host_port.to_string(), methods.clone());
    Ok(methods)
}

fn extract_methods(resp: &RpcResponse) -> io::Result<Vec<String>> {
    match resp {
        RpcResponse::Success(ok) => Ok(ok
            .result
            .get("methods")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()),
        RpcResponse::Error(e) => Err(io::Error::other(format!(
            "hub.capabilities error: {}",
            e.error.message
        ))),
    }
}

/// T-3308: one `channel.subscribe` page as the inbox reader needs it.
struct SubscribePage {
    messages: Vec<Value>,
    next_cursor: u64,
    /// `"limit"` / `"end"` / `"deadline"`; `None` from a hub predating T-3308.
    end_reason: Option<String>,
    /// The hub's T-3307 `gap`, when the requested cursor was already swept.
    gap: Option<RetentionGap>,
}

/// T-3308: page size the inbox reader asks for.
const INBOX_PAGE_LIMIT: u64 = 1000;
/// T-3308: upper bound on pages per read, so a hub that keeps answering "limit"
/// (or a bug) cannot spin the reader forever. 200 x 1000 records.
const INBOX_MAX_PAGES: usize = 200;

/// T-3308: does the walk continue after this page? Only while the cursor
/// advances, and only when the page says more may follow — `limit` or
/// `deadline` — or, from a hub too old to say, when the page came back full.
fn inbox_page_continues(page: &SubscribePage, cursor: u64) -> bool {
    if page.next_cursor <= cursor {
        return false;
    }
    match page.end_reason.as_deref() {
        Some("limit") | Some("deadline") => true,
        Some(_) => false,
        None => page.messages.len() as u64 >= INBOX_PAGE_LIMIT,
    }
}

/// T-3308: read every page from `cursor`. Before, the reader fetched ONE page of
/// up to 1000 records, so on a bigger `inbox:*` topic a one-shot `inbox list`
/// (which always starts at 0) never saw later transfers. Returns all messages,
/// the final cursor, and the gap reported on the FIRST page (the only one whose
/// requested cursor the caller chose).
trait PageSource {
    async fn fetch(&mut self, cursor: u64) -> Result<SubscribePage, SubscribeError>;
}

struct ClientPages<'a> {
    client: &'a mut Client,
    topic: &'a str,
}

impl PageSource for ClientPages<'_> {
    async fn fetch(&mut self, cursor: u64) -> Result<SubscribePage, SubscribeError> {
        call_channel_subscribe_via_client(self.client, self.topic, cursor).await
    }
}

async fn page_through_subscribe<S: PageSource>(
    mut cursor: u64,
    source: &mut S,
) -> Result<(Vec<Value>, u64, Option<RetentionGap>), SubscribeError> {
    let mut all = Vec::new();
    let mut first_gap = None;
    for page_no in 0..INBOX_MAX_PAGES {
        let page = source.fetch(cursor).await?;
        if page_no == 0 {
            first_gap = page.gap.clone();
        }
        let more = inbox_page_continues(&page, cursor);
        let next = page.next_cursor.max(cursor);
        all.extend(page.messages);
        cursor = next;
        if !more {
            return Ok((all, cursor, first_gap));
        }
    }
    tracing::warn!(
        pages = INBOX_MAX_PAGES,
        cursor,
        "T-3308: inbox read stopped at the page cap; the rest is read on the next call"
    );
    Ok((all, cursor, first_gap))
}

/// T-3308: record a hub-reported gap as a LOSS only when the reader resumed
/// from a cursor it saved itself (`own_cursor`). A first read from 0 on a swept
/// topic is a fresh reader, not a loss (T-3307).
fn note_retention_gap(
    ctx: &mut FallbackCtx,
    host_port: &str,
    topic: &str,
    own_cursor: Option<u64>,
    gap: Option<RetentionGap>,
) {
    let (Some(_), Some(gap)) = (own_cursor, gap) else { return };
    tracing::warn!(
        host = %host_port,
        topic = %topic,
        requested_cursor = gap.requested_cursor,
        oldest_offset = gap.oldest_offset,
        skipped = gap.skipped,
        "T-3308: {} inbox record(s) were removed by retention before this reader got them",
        gap.skipped
    );
    ctx.retention_gaps.push(RetentionGap { topic: topic.to_string(), ..gap });
}

fn gap_from_result(v: &Value) -> Option<RetentionGap> {
    let g = v.get("gap")?;
    Some(RetentionGap {
        topic: String::new(),
        requested_cursor: g.get("requested_cursor")?.as_u64()?,
        oldest_offset: g.get("oldest_offset")?.as_u64()?,
        skipped: g.get("skipped")?.as_u64()?,
    })
}

async fn call_channel_subscribe_via_client(
    client: &mut Client,
    topic: &str,
    cursor: u64,
) -> Result<SubscribePage, SubscribeError> {
    let resp = client
        .call(
            control::method::CHANNEL_SUBSCRIBE,
            json!("inbox-sub"),
            json!({
                "topic": topic,
                "cursor": cursor,
                "limit": INBOX_PAGE_LIMIT,
            }),
        )
        .await
        .map_err(|e| SubscribeError::Other(map_client_err("channel.subscribe", e)))?;

    // T-2573 (SQ-11 c2): a hub walk that hit its deadline returns the records
    // it collected plus a resume point — deliver them instead of failing.
    if let Some(page) = crate::client::deadline_partial_page(&resp) {
        let messages = page["messages"].as_array().cloned().unwrap_or_default();
        let next_cursor = page["next_cursor"].as_u64().unwrap_or(cursor);
        return Ok(SubscribePage {
            messages,
            next_cursor,
            end_reason: Some("deadline".to_string()),
            gap: None,
        });
    }

    match resp {
        RpcResponse::Success(ok) => {
            let messages = ok
                .result
                .get("messages")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let next_cursor = ok
                .result
                .get("next_cursor")
                .and_then(|v| v.as_u64())
                .unwrap_or(cursor);
            Ok(SubscribePage {
                messages,
                next_cursor,
                end_reason: ok
                    .result
                    .get("end_reason")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                gap: gap_from_result(&ok.result),
            })
        }
        RpcResponse::Error(e) if e.error.code == RPC_METHOD_NOT_FOUND => {
            Err(SubscribeError::MethodNotFound)
        }
        RpcResponse::Error(e) => Err(SubscribeError::Other(io::Error::other(format!(
            "channel.subscribe error {}: {}",
            e.error.code, e.error.message
        )))),
    }
}

/// Fold a stream of `inbox:<target>` channel envelopes into per-transfer
/// summaries. Drops transfers that emit `file.error`. Public for direct use by
/// dual-read mergers in a follow-up wedge.
pub fn fold_envelopes(messages: &[Value]) -> Vec<InboxEntry> {
    let mut by_id: BTreeMap<String, InboxEntry> = BTreeMap::new();
    let mut errored: HashSet<String> = HashSet::new();

    for msg in messages {
        let msg_type = msg.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        let payload_b64 = msg
            .get("payload_b64")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let decoded = match B64.decode(payload_b64) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let mirror: Value = match serde_json::from_slice(&decoded) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let inner = mirror.get("payload").cloned().unwrap_or(Value::Null);
        let transfer_id = inner
            .get("transfer_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if transfer_id.is_empty() {
            continue;
        }

        match msg_type {
            "file.init" => {
                let entry = InboxEntry {
                    transfer_id: transfer_id.clone(),
                    filename: inner
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    from: inner
                        .get("from")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    size: inner.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
                    chunks_received: 0,
                    total_chunks: inner
                        .get("total_chunks")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    complete: false,
                };
                by_id.entry(transfer_id).or_insert(entry);
            }
            "file.chunk" => {
                if let Some(e) = by_id.get_mut(&transfer_id) {
                    e.chunks_received = e.chunks_received.saturating_add(1);
                }
            }
            "file.complete" => {
                if let Some(e) = by_id.get_mut(&transfer_id) {
                    e.complete = true;
                    if e.total_chunks > 0 {
                        e.chunks_received = e.total_chunks;
                    }
                }
            }
            "file.error" => {
                errored.insert(transfer_id);
            }
            _ => {}
        }
    }

    by_id
        .into_iter()
        .filter(|(id, _)| !errored.contains(id))
        .map(|(_, v)| v)
        .collect()
}

fn host_port_str(addr: &TransportAddr) -> String {
    match addr {
        TransportAddr::Tcp { host, port } => format!("{host}:{port}"),
        TransportAddr::Unix { path } => format!("unix:{}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T-1310: env-var injection helper. Uses a unique env-var name guard
    /// pattern to avoid contaminating other tests in the binary; we save and
    /// restore the original value rather than relying on test isolation.
    /// Uses `serial_test`-free approach via SAFETY: tests in this block do
    /// not run concurrently with the env var set because we restore on drop.
    struct EnvGuard {
        key: &'static str,
        prev: Option<String>,
    }
    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = std::env::var(key).ok();
            // SAFETY: cargo test runs with multiple threads but tests using
            // this guard hold the env var only briefly within a single test
            // function; collisions with concurrent reads in this module are
            // benign (helper is idempotent in shape).
            unsafe { std::env::set_var(key, value) };
            Self { key, prev }
        }
        fn unset(key: &'static str) -> Self {
            let prev = std::env::var(key).ok();
            unsafe { std::env::remove_var(key) };
            Self { key, prev }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.prev {
                    Some(v) => std::env::set_var(self.key, v),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    // T-1415: params_with_session_from_all_scenarios test deleted — function
    // it covered (params_with_session_from) deleted alongside legacy fallback paths.

    fn synth_msg(msg_type: &str, payload: Value) -> Value {
        let mirror = json!({"from": "session-A", "payload": payload});
        let bytes = serde_json::to_vec(&mirror).unwrap();
        json!({
            "offset": 0,
            "topic": "inbox:test-target",
            "sender_id": "hub:inbox.deposit",
            "msg_type": msg_type,
            "payload_b64": B64.encode(&bytes),
            "artifact_ref": null,
            "ts": 0,
        })
    }

    #[test]
    fn fold_envelopes_assembles_pending_transfer() {
        let msgs = vec![
            synth_msg(
                "file.init",
                json!({
                    "transfer_id": "xfer-1",
                    "filename": "a.bin",
                    "from": "alpha",
                    "size": 1024,
                    "total_chunks": 1
                }),
            ),
            synth_msg("file.chunk", json!({"transfer_id": "xfer-1", "index": 0})),
            synth_msg(
                "file.complete",
                json!({"transfer_id": "xfer-1", "sha256": "deadbeef"}),
            ),
        ];
        let entries = fold_envelopes(&msgs);
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.transfer_id, "xfer-1");
        assert_eq!(e.filename, "a.bin");
        assert_eq!(e.size, 1024);
        assert_eq!(e.total_chunks, 1);
        assert_eq!(e.chunks_received, 1);
        assert!(e.complete);
    }

    #[test]
    fn fold_envelopes_groups_by_transfer_id() {
        let msgs = vec![
            synth_msg(
                "file.init",
                json!({
                    "transfer_id": "xfer-A",
                    "filename": "a",
                    "from": "x",
                    "size": 10,
                    "total_chunks": 2
                }),
            ),
            synth_msg(
                "file.init",
                json!({
                    "transfer_id": "xfer-B",
                    "filename": "b",
                    "from": "x",
                    "size": 20,
                    "total_chunks": 1
                }),
            ),
            synth_msg("file.chunk", json!({"transfer_id": "xfer-A", "index": 0})),
            synth_msg("file.chunk", json!({"transfer_id": "xfer-A", "index": 1})),
            synth_msg("file.chunk", json!({"transfer_id": "xfer-B", "index": 0})),
        ];
        let entries = fold_envelopes(&msgs);
        assert_eq!(entries.len(), 2);
        let a = entries.iter().find(|e| e.transfer_id == "xfer-A").unwrap();
        let b = entries.iter().find(|e| e.transfer_id == "xfer-B").unwrap();
        assert_eq!(a.chunks_received, 2);
        assert_eq!(b.chunks_received, 1);
        assert!(!a.complete);
        assert!(!b.complete);
    }

    #[test]
    fn fold_envelopes_drops_errored_transfer() {
        let msgs = vec![
            synth_msg(
                "file.init",
                json!({
                    "transfer_id": "xfer-bad",
                    "filename": "x",
                    "from": "x",
                    "size": 1,
                    "total_chunks": 1
                }),
            ),
            synth_msg(
                "file.error",
                json!({"transfer_id": "xfer-bad", "message": "boom"}),
            ),
        ];
        assert!(fold_envelopes(&msgs).is_empty());
    }

    #[test]
    fn fold_envelopes_ignores_malformed_messages() {
        let msgs = vec![
            json!({"msg_type": "file.init", "payload_b64": "@@@not-base64@@@"}),
            json!({"msg_type": "file.init", "payload_b64": B64.encode(b"not-json")}),
            synth_msg("file.init", json!({})), // missing transfer_id
        ];
        assert!(fold_envelopes(&msgs).is_empty());
    }

    /// T-3308: a fake hub that pages like the real one.
    struct FakePages {
        total: u64,
        limit: u64,
        /// None = a hub predating T-3308 (no end_reason field).
        new_hub: bool,
        oldest: u64,
        calls: usize,
    }

    impl PageSource for FakePages {
        async fn fetch(&mut self, cursor: u64) -> Result<SubscribePage, SubscribeError> {
            self.calls += 1;
            let start = cursor.max(self.oldest);
            let end = (start + self.limit).min(self.total);
            let messages: Vec<Value> = (start..end).map(|o| json!({"offset": o})).collect();
            let full = messages.len() as u64 == self.limit;
            Ok(SubscribePage {
                next_cursor: if end > start { end } else { cursor },
                end_reason: self.new_hub.then(|| if full { "limit" } else { "end" }.to_string()),
                gap: (cursor < self.oldest).then(|| RetentionGap {
                    topic: String::new(),
                    requested_cursor: cursor,
                    oldest_offset: self.oldest,
                    skipped: self.oldest - cursor,
                }),
                messages,
            })
        }
    }

    #[tokio::test]
    async fn t3308_inbox_reader_reads_past_the_first_page() {
        // Before T-3308 one page (1000) was all a one-shot `inbox list` ever saw.
        let mut hub = FakePages { total: 2500, limit: INBOX_PAGE_LIMIT, new_hub: true, oldest: 0, calls: 0 };
        let (msgs, next, gap) = page_through_subscribe(0, &mut hub).await.ok().unwrap();
        assert_eq!(msgs.len(), 2500);
        assert_eq!(msgs.last().unwrap()["offset"], 2499);
        assert_eq!(next, 2500);
        assert!(gap.is_none());
        assert_eq!(hub.calls, 3);
    }

    #[tokio::test]
    async fn t3308_inbox_reader_pages_an_old_hub_by_page_size() {
        // A hub with no end_reason: a full page means "maybe more".
        let mut hub = FakePages { total: 2000, limit: INBOX_PAGE_LIMIT, new_hub: false, oldest: 0, calls: 0 };
        let (msgs, next, _) = page_through_subscribe(0, &mut hub).await.ok().unwrap();
        assert_eq!(msgs.len(), 2000);
        assert_eq!(next, 2000);
        assert_eq!(hub.calls, 3, "two full pages, then an empty one that stops the walk");
    }

    #[tokio::test]
    async fn t3308_inbox_reader_stops_when_cursor_does_not_advance() {
        struct Stuck;
        impl PageSource for Stuck {
            async fn fetch(&mut self, cursor: u64) -> Result<SubscribePage, SubscribeError> {
                Ok(SubscribePage {
                    messages: vec![],
                    next_cursor: cursor,
                    end_reason: Some("limit".into()),
                    gap: None,
                })
            }
        }
        let (msgs, next, _) = page_through_subscribe(7, &mut Stuck).await.ok().unwrap();
        assert!(msgs.is_empty());
        assert_eq!(next, 7);
    }

    #[tokio::test]
    async fn t3308_inbox_reader_records_gap_only_for_its_own_cursor() {
        let mut ctx = FallbackCtx::new();
        let mut hub = FakePages { total: 50, limit: INBOX_PAGE_LIMIT, new_hub: true, oldest: 30, calls: 0 };
        // Resumed from a cursor this ctx saved: a real loss.
        let (msgs, _, gap) = page_through_subscribe(10, &mut hub).await.ok().unwrap();
        assert_eq!(msgs.len(), 20);
        note_retention_gap(&mut ctx, "h:1", "inbox:t", Some(10), gap);
        let gaps = ctx.take_retention_gaps();
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].topic, "inbox:t");
        assert_eq!(gaps[0].skipped, 20);
        assert!(ctx.take_retention_gaps().is_empty(), "take drains");
        // A fresh reader starting at 0 on the same swept topic: not a loss.
        let (_, _, gap) = page_through_subscribe(0, &mut hub).await.ok().unwrap();
        assert!(gap.is_some(), "the hub still reports the fact");
        note_retention_gap(&mut ctx, "h:1", "inbox:t", None, gap);
        assert!(ctx.take_retention_gaps().is_empty());
    }

    #[test]
    fn fallback_ctx_warn_once_dedupes() {
        let mut ctx = FallbackCtx::new();
        assert!(ctx.warn_once("h:1", "channel.subscribe"));
        assert!(!ctx.warn_once("h:1", "channel.subscribe"));
        assert!(ctx.warn_once("h:2", "channel.subscribe"));
        assert!(ctx.warn_once("h:1", "inbox.list"));
    }

    #[test]
    fn fallback_ctx_legacy_only_flag() {
        let mut ctx = FallbackCtx::new();
        assert!(!ctx.is_legacy_only("h:1"));
        ctx.flag_legacy_only("h:1");
        assert!(ctx.is_legacy_only("h:1"));
        assert!(!ctx.is_legacy_only("h:2"));
    }

    #[test]
    fn fallback_ctx_cursor_roundtrip() {
        let mut ctx = FallbackCtx::new();
        assert!(ctx.cursor("inbox:t").is_none());
        ctx.set_cursor("inbox:t", 42);
        assert_eq!(ctx.cursor("inbox:t"), Some(42));
    }

    // T-1235 / T-1229b — channel.list aggregation tests for inbox.status path.

    #[test]
    fn status_aggregates_two_inbox_topics() {
        let resp = json!({
            "topics": [
                {"name": "inbox:alice", "retention": {"kind": "forever"}, "count": 3},
                {"name": "inbox:bob",   "retention": {"kind": "forever"}, "count": 1},
            ]
        });
        let s = aggregate_status_from_channel_list(&resp);
        assert_eq!(s.total_records, 4);
        assert_eq!(s.targets.len(), 2);
        let alice = s.targets.iter().find(|t| t.target == "alice").unwrap();
        let bob = s.targets.iter().find(|t| t.target == "bob").unwrap();
        assert_eq!(alice.records, 3);
        assert_eq!(bob.records, 1);
    }

    /// T-3197 / PL-391. The load-bearing test: this count includes NON-transfer
    /// envelopes. `channel.list` reports a topic's total record count with no
    /// `msg_type` breakdown, and `inbox:*` topics carry ordinary messages
    /// (`sidecar.consult`, `note`) alongside `file.*` transfer envelopes.
    ///
    /// Measured on the live hub 2026-09-28: this path reported 232 while
    /// `list_via_channel` — which folds by `msg_type` — returned ZERO transfers
    /// for all 22 targets. The old field name `total_transfers` asserted the
    /// opposite, and `inbox clear` resolves to `channel.trim`, so the label came
    /// one command away from authorising deletion of 49 unread inbound consults.
    ///
    /// If someone renames these fields back toward "transfers", this test is the
    /// statement of why they must not.
    #[test]
    fn status_counts_records_including_non_transfer_envelopes() {
        // A topic whose records are entirely ordinary messages — zero transfers.
        let resp = json!({
            "topics": [
                {"name": "inbox:cacc73ea/010-termlink", "retention": {"kind": "forever"}, "count": 49},
            ]
        });
        let s = aggregate_status_from_channel_list(&resp);
        assert_eq!(
            s.total_records, 49,
            "every record counts, regardless of msg_type — this is NOT a transfer count"
        );
        assert_eq!(s.targets[0].records, 49);

        // The folding path is the one that answers "how many transfers?", and on
        // the same envelopes it answers zero: none carry a transfer_id.
        let consults = vec![
            json!({"msg_type": "sidecar.consult", "payload_b64": B64.encode(b"{\"payload\":{}}")}),
            json!({"msg_type": "note", "payload_b64": B64.encode(b"{\"payload\":{}}")}),
        ];
        assert!(
            fold_envelopes(&consults).is_empty(),
            "non-transfer envelopes must contribute no InboxEntry — the two surfaces \
             genuinely measure different things, which is the whole finding"
        );
    }

    /// T-3197: a payload written by an older binary still parses. The serialized
    /// name changed deliberately (PL-244); deserialization stays compatible.
    #[test]
    fn status_deserializes_legacy_field_names_via_alias() {
        let legacy = json!({
            "total_transfers": 7,
            "targets": [{"target": "alice", "pending": 7}]
        });
        let s: InboxStatus = serde_json::from_value(legacy).expect("legacy payload must parse");
        assert_eq!(s.total_records, 7);
        assert_eq!(s.targets[0].records, 7);

        // ...and the serialized form uses the honest names.
        let out = serde_json::to_value(&s).unwrap();
        assert!(out.get("total_records").is_some(), "honest name must be emitted");
        assert!(
            out.get("total_transfers").is_none(),
            "the misleading name must NOT be emitted — a consumer reading it gets a loud \
             null rather than a number that never meant transfers"
        );
    }

    #[test]
    fn status_skips_non_inbox_prefix_topics_defensively() {
        // The channel.list call uses prefix=inbox: so the hub *should* only
        // return inbox: topics, but be defensive against future prefix changes
        // or a hub that returns extras — only inbox:* contributes to the count.
        let resp = json!({
            "topics": [
                {"name": "inbox:carol", "count": 2},
                {"name": "event:noise", "count": 99},
                {"name": "inbox:",      "count": 1},
            ]
        });
        let s = aggregate_status_from_channel_list(&resp);
        assert_eq!(s.total_records, 2, "only inbox:carol contributes");
        assert_eq!(s.targets.len(), 1);
        assert_eq!(s.targets[0].target, "carol");
    }

    #[test]
    fn status_handles_missing_count_field_as_zero() {
        // Hub running an older binary that lacks T-1233 won't include count;
        // helper should degrade gracefully rather than panic.
        let resp = json!({
            "topics": [
                {"name": "inbox:dave"},
                {"name": "inbox:eve", "count": 5},
            ]
        });
        let s = aggregate_status_from_channel_list(&resp);
        assert_eq!(s.total_records, 5);
        let dave = s.targets.iter().find(|t| t.target == "dave").unwrap();
        assert_eq!(dave.records, 0);
    }

    #[test]
    fn status_handles_empty_topics_list() {
        let resp = json!({"topics": []});
        let s = aggregate_status_from_channel_list(&resp);
        assert_eq!(s.total_records, 0);
        assert!(s.targets.is_empty());
    }

    #[test]
    fn status_handles_missing_topics_field() {
        // Defensive: a malformed response with no "topics" key should yield empty
        // status rather than erroring.
        let resp = json!({});
        let s = aggregate_status_from_channel_list(&resp);
        assert_eq!(s.total_records, 0);
        assert!(s.targets.is_empty());
    }

    // === T-1230c: clear_via_channel aggregation tests ===

    #[test]
    fn topics_with_inbox_prefix_filters_and_strips_correctly() {
        let resp = json!({"topics": [
            {"name": "inbox:alice", "retention": {}},
            {"name": "inbox:bob", "retention": {}},
            {"name": "events:other", "retention": {}},
            {"name": "inbox:", "retention": {}},
        ]});
        let topics = topics_with_inbox_prefix(&resp);
        assert_eq!(topics, vec!["inbox:alice", "inbox:bob"]);
    }

    #[test]
    fn topics_with_inbox_prefix_handles_empty() {
        assert!(topics_with_inbox_prefix(&json!({"topics": []})).is_empty());
        assert!(topics_with_inbox_prefix(&json!({})).is_empty());
        assert!(topics_with_inbox_prefix(&json!({"topics": "not-an-array"})).is_empty());
    }

    #[test]
    fn topics_with_inbox_prefix_skips_missing_name_field() {
        let resp = json!({"topics": [
            {"name": "inbox:alice"},
            {"retention": {}},
            {"name": null},
        ]});
        assert_eq!(topics_with_inbox_prefix(&resp), vec!["inbox:alice"]);
    }

    /// T-1229g regression: channel.list returns ALL `inbox:` topics from the
    /// hub bus records — including topics whose target is offline (no live
    /// subscriber). The aggregator must surface those, since fleet-doctor's
    /// G-013 invariant counts pending-for-offline-targets too. Legacy
    /// `inbox.status` had the same property; this test prevents regression.
    #[test]
    fn aggregate_status_includes_offline_targets() {
        // Synthesize a channel.list reply where:
        //  - "alice" is a live target (count 2)
        //  - "bob-offline" target has 5 pending transfers but no subscriber —
        //    its topic exists because the hub mirrored deposits via T-1163.
        //  - "carol" is empty live (count 0).
        let resp = json!({
            "topics": [
                {"name": "inbox:alice", "retention": {}, "count": 2},
                {"name": "inbox:bob-offline", "retention": {}, "count": 5},
                {"name": "inbox:carol", "retention": {}, "count": 0},
            ]
        });
        let s = aggregate_status_from_channel_list(&resp);
        assert_eq!(s.total_records, 7, "offline target's count must be summed");
        let names: Vec<&str> = s.targets.iter().map(|t| t.target.as_str()).collect();
        assert!(
            names.contains(&"bob-offline"),
            "offline target must appear in InboxStatus — fleet-doctor depends on this"
        );
        let bob = s.targets.iter().find(|t| t.target == "bob-offline").unwrap();
        assert_eq!(bob.records, 5);
    }

    #[test]
    fn fallback_ctx_warn_once_keys_distinguish_clear_from_status() {
        let mut ctx = FallbackCtx::new();
        // T-1235 keys
        assert!(ctx.warn_once("h:1", "channel.list"));
        assert!(!ctx.warn_once("h:1", "channel.list"));
        // T-1230c keys must not be deduped against T-1235 keys
        assert!(ctx.warn_once("h:1", "channel.trim"));
        assert!(!ctx.warn_once("h:1", "channel.trim"));
        assert!(ctx.warn_once("h:1", "inbox.clear"));
    }
}
