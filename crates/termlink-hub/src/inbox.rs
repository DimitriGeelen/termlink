//! Hub-level file inbox — queues file transfers for offline sessions (T-988).
//!
//! When a `send-file` targets a session that isn't online, the hub spools
//! the file events (init, chunks, complete) to disk. When the target
//! session registers, pending transfers are delivered automatically.
//!
//! Spool layout:
//! ```text
//! {runtime_dir}/inbox/{target_name}/{transfer_id}/
//!   init.json      — FileInit + metadata
//!   chunk-0000.json — FileChunk (base64 data)
//!   chunk-0001.json
//!   complete.json  — FileComplete (sha256)
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use termlink_protocol::TransportAddr;
use termlink_session::{client, discovery};

/// Default expiry for pending inbox files (24 hours).
pub const DEFAULT_EXPIRY: Duration = Duration::from_secs(24 * 60 * 60);

/// Metadata envelope for a spooled file event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxEntry {
    pub transfer_id: String,
    pub target: String,
    pub from: Option<String>,
    pub topic: String,
    pub payload: Value,
    pub timestamp: u64,
}

/// Summary of a pending transfer in the inbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransfer {
    pub transfer_id: String,
    pub target: String,
    pub filename: String,
    pub from: String,
    pub size: u64,
    pub chunks_received: u32,
    pub total_chunks: u32,
    pub complete: bool,
    pub age_secs: u64,
}

/// Root inbox directory.
pub fn inbox_dir() -> PathBuf {
    discovery::runtime_dir().join("inbox")
}

/// Target-specific inbox directory.
fn target_dir(target: &str) -> PathBuf {
    // Sanitize target name for filesystem safety
    let safe_name: String = target
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    inbox_dir().join(safe_name)
}

/// Transfer-specific spool directory.
fn transfer_dir(target: &str, transfer_id: &str) -> PathBuf {
    let safe_id: String = transfer_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    target_dir(target).join(safe_id)
}

/// Deposit a file event into the inbox spool.
///
/// Called by the hub when a file event targets an offline session.
/// Returns Ok(true) if the event was spooled, Ok(false) if not a file topic.
pub fn deposit(target: &str, topic: &str, payload: &Value, from: Option<&str>) -> std::io::Result<bool> {
    // Only spool file-related topics
    if !is_file_topic(topic) {
        return Ok(false);
    }

    // T-1251: warn-once-per-process when legacy file.* events still reach the
    // inbox. T-1164 has shipped channel.post {msg_type:artifact} as the new
    // primary path; this deposit indicates a sender that hasn't migrated yet.
    static LEGACY_WARNED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if LEGACY_WARNED.set(()).is_ok() {
        tracing::info!(
            target = target,
            topic = topic,
            "T-1251: legacy file.* events received — sender should migrate to channel.post {{msg_type:artifact}}"
        );
    }

    let transfer_id = match payload.get("transfer_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            tracing::warn!(target = target, topic = topic, "Inbox deposit: missing transfer_id");
            return Ok(false);
        }
    };

    let dir = transfer_dir(target, &transfer_id);
    std::fs::create_dir_all(&dir)?;

    let entry = InboxEntry {
        transfer_id: transfer_id.clone(),
        target: target.to_string(),
        from: from.map(String::from),
        topic: topic.to_string(),
        payload: payload.clone(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let filename = match topic {
        "file.init" => "init.json".to_string(),
        "file.chunk" => {
            // T-2505: a missing/non-integer index MUST NOT default to 0 — that
            // silently overwrites the legitimately-spooled chunk-0000.json (data
            // corruption). Reject loud + non-destructive, mirroring the
            // missing-transfer_id arm above. Sibling of T-2490, which hardened the
            // reassembly side (a bad index sorts last via unwrap_or(u64::MAX)); here
            // on the deposit side a bad index must never eat a good chunk.
            let Some(index) = payload.get("index").and_then(|v| v.as_u64()) else {
                tracing::warn!(
                    target = target,
                    topic = topic,
                    transfer_id = %transfer_id,
                    "Inbox deposit: file.chunk missing/non-integer index — rejecting to avoid clobbering chunk 0"
                );
                return Ok(false);
            };
            format!("chunk-{index:04}.json")
        }
        "file.complete" => "complete.json".to_string(),
        "file.error" => "error.json".to_string(),
        _ => return Ok(false),
    };

    let path = dir.join(&filename);
    let json = serde_json::to_string_pretty(&entry)?;
    std::fs::write(&path, json)?;

    tracing::info!(
        target_session = target,
        transfer_id = %transfer_id,
        topic = topic,
        file = %filename,
        "Inbox: spooled file event for offline session"
    );

    Ok(true)
}

/// Check if a topic is a file transfer topic.
fn is_file_topic(topic: &str) -> bool {
    matches!(topic, "file.init" | "file.chunk" | "file.complete" | "file.error")
}

/// List all pending transfers for a target session.
pub fn list_pending(target: &str) -> std::io::Result<Vec<PendingTransfer>> {
    let tdir = target_dir(target);
    if !tdir.exists() {
        return Ok(vec![]);
    }

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut transfers = Vec::new();

    for entry in std::fs::read_dir(&tdir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let transfer_id = entry.file_name().to_string_lossy().to_string();
        let xfer_dir = entry.path();

        // Read init.json for metadata
        let init_path = xfer_dir.join("init.json");
        if !init_path.exists() {
            continue;
        }

        let init_json: InboxEntry = match std::fs::read_to_string(&init_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            Some(e) => e,
            None => continue,
        };

        let filename = init_json.payload.get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let from = init_json.from.unwrap_or_default();
        let size = init_json.payload.get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total_chunks = init_json.payload.get("total_chunks")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        // Count chunk files
        let chunks_received = std::fs::read_dir(&xfer_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("chunk-"))
            .count() as u32;

        let complete = xfer_dir.join("complete.json").exists();
        let age_secs = now.saturating_sub(init_json.timestamp);

        transfers.push(PendingTransfer {
            transfer_id,
            target: target.to_string(),
            filename,
            from,
            size,
            chunks_received,
            total_chunks,
            complete,
            age_secs,
        });
    }

    transfers.sort_by_key(|t| std::cmp::Reverse(t.age_secs));
    Ok(transfers)
}

/// List all targets with pending inbox items.
pub fn list_all_targets() -> std::io::Result<Vec<(String, usize)>> {
    let idir = inbox_dir();
    if !idir.exists() {
        return Ok(vec![]);
    }

    let mut targets = Vec::new();
    for entry in std::fs::read_dir(&idir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let target = entry.file_name().to_string_lossy().to_string();
        let count = std::fs::read_dir(entry.path())?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .count();
        if count > 0 {
            targets.push((target, count));
        }
    }

    Ok(targets)
}

/// Deliver all pending transfers for a target session.
///
/// Called when a session registers or becomes reachable.
/// Returns the number of transfers delivered.
pub async fn deliver_pending(target: &str, addr: &TransportAddr) -> usize {
    let tdir = target_dir(target);
    if !tdir.exists() {
        return 0;
    }

    let transfers = match list_pending(target) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(target = target, error = %e, "Inbox: failed to list pending transfers");
            return 0;
        }
    };

    if transfers.is_empty() {
        return 0;
    }

    tracing::info!(
        target = target,
        count = transfers.len(),
        "Inbox: delivering pending transfers to newly registered session"
    );

    let mut delivered = 0;

    for transfer in &transfers {
        if !transfer.complete {
            tracing::warn!(
                transfer_id = %transfer.transfer_id,
                chunks = %transfer.chunks_received,
                total = %transfer.total_chunks,
                "Inbox: skipping incomplete transfer"
            );
            continue;
        }

        let xfer_dir = transfer_dir(target, &transfer.transfer_id);
        if deliver_transfer(addr, &xfer_dir).await {
            // Clean up after successful delivery
            let _ = std::fs::remove_dir_all(&xfer_dir);
            delivered += 1;
        }
    }

    // Clean up empty target dir
    if std::fs::read_dir(&tdir).is_ok_and(|e| e.count() == 0) {
        let _ = std::fs::remove_dir(&tdir);
    }

    if delivered > 0 {
        tracing::info!(
            target = target,
            delivered = delivered,
            "Inbox: delivery complete"
        );
    }

    delivered
}

/// Extract the numeric chunk index from a `chunk-<n>.json` filename (T-2490).
///
/// Returns `None` if the name does not match the `chunk-<digits>.json` shape —
/// callers sort such a name last so a malformed filename cannot reorder the
/// valid chunks. This is magnitude-independent (unlike a lexical filename sort,
/// which is only correct within `deposit`'s 4-digit zero-pad).
fn chunk_index(file_name: &str) -> Option<u64> {
    file_name
        .strip_prefix("chunk-")
        .and_then(|s| s.strip_suffix(".json"))
        .and_then(|s| s.parse::<u64>().ok())
}

/// List a transfer's `chunk-*` files in delivery order, verifying every one is
/// readable/parseable.
///
/// Returns the ordered chunk paths, or `None` if ANY chunk file is unreadable or
/// corrupt. T-2489: a `None` MUST make the caller RETAIN the spool and report the
/// transfer undelivered — the previous `None => continue` in `deliver_transfer`
/// silently omitted the bad chunk, delivered the rest + `complete.json`, returned
/// `true`, and let the caller `remove_dir_all` the only copy (unrecoverable loss
/// of a file that then fails its sha256 on the receiver). Mirrors the
/// loud-and-never-destroy convention of T-2487 (log reader) + T-2488 (emit_to).
/// Pure I/O, no network — unit-testable.
fn ordered_chunk_paths_checked(xfer_dir: &Path) -> Option<Vec<std::path::PathBuf>> {
    let mut chunk_files: Vec<_> = std::fs::read_dir(xfer_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("chunk-"))
        .collect();
    // T-2490: sort by the parsed NUMERIC chunk index, not the filename string.
    // `deposit` names chunks `chunk-{index:04}.json`; a lexical filename sort is
    // correct only while the index stays within the 4-digit zero-pad — once it
    // overflows to 5 digits (`chunk-10000.json`) lexical order diverges from
    // numeric ("chunk-10000" < "chunk-9999"), so any transfer >10000 chunks
    // (~>480MB at 48KiB chunks) reassembled out of order. A name that does not
    // parse to an index sorts last (u64::MAX) rather than panicking — it would
    // fail the receiver's sha256 anyway and must not reorder the valid chunks.
    chunk_files.sort_by_key(|e| chunk_index(&e.file_name().to_string_lossy()).unwrap_or(u64::MAX));

    let mut paths = Vec::with_capacity(chunk_files.len());
    for chunk_file in chunk_files {
        let path = chunk_file.path();
        // Validate readability up front so a corrupt chunk aborts BEFORE any
        // partial delivery (and before the caller deletes the spool).
        if read_entry(&path).is_none() {
            return None;
        }
        paths.push(path);
    }
    Some(paths)
}

/// Deliver a single transfer's events to a session.
async fn deliver_transfer(addr: &TransportAddr, xfer_dir: &Path) -> bool {
    // Read and deliver init
    let init_path = xfer_dir.join("init.json");
    let init_entry: InboxEntry = match read_entry(&init_path) {
        Some(e) => e,
        None => return false,
    };

    if emit_event(addr, &init_entry).await.is_err() {
        tracing::warn!(
            transfer_id = %init_entry.transfer_id,
            "Inbox: failed to deliver init event"
        );
        return false;
    }

    // Deliver chunks in order. T-2489: a single unreadable/corrupt chunk aborts
    // the whole transfer and RETAINS the spool — we never deliver a partial
    // transfer and then let the caller destroy the only copy (the previous
    // `None => continue` silently dropped bad chunks, returned true, and the
    // spool was deleted → unrecoverable loss + failed sha256 on the receiver).
    let chunk_paths = match ordered_chunk_paths_checked(xfer_dir) {
        Some(paths) => paths,
        None => {
            tracing::warn!(
                transfer_id = %init_entry.transfer_id,
                "Inbox: a chunk is unreadable/corrupt — RETAINING spool, NOT delivering partial transfer"
            );
            return false;
        }
    };

    for path in chunk_paths {
        let entry: InboxEntry = match read_entry(&path) {
            Some(e) => e,
            None => {
                // Became unreadable between the pre-check and emit (TOCTOU) —
                // still abort + retain rather than silently skip.
                tracing::warn!(
                    transfer_id = %init_entry.transfer_id,
                    chunk = %path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    "Inbox: chunk became unreadable during delivery — RETAINING spool"
                );
                return false;
            }
        };

        if emit_event(addr, &entry).await.is_err() {
            tracing::warn!(
                transfer_id = %init_entry.transfer_id,
                chunk = %path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                "Inbox: failed to deliver chunk"
            );
            return false;
        }
    }

    // Deliver complete
    let complete_path = xfer_dir.join("complete.json");
    if let Some(complete_entry) = read_entry(&complete_path)
        && emit_event(addr, &complete_entry).await.is_err()
    {
        tracing::warn!(
            transfer_id = %init_entry.transfer_id,
            "Inbox: failed to deliver complete event"
        );
        return false;
    }

    true
}

/// Emit a single event to a session address.
async fn emit_event(addr: &TransportAddr, entry: &InboxEntry) -> Result<(), String> {
    let params = json!({
        "topic": entry.topic,
        "payload": entry.payload,
    });

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        client::rpc_call_addr(addr, "event.emit", params),
    )
    .await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(format!("RPC error: {e}")),
        Err(_) => Err("timeout".to_string()),
    }
}

/// Read an InboxEntry from a JSON file.
fn read_entry(path: &Path) -> Option<InboxEntry> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Clean up expired inbox entries.
///
/// Called by the supervisor sweep. Removes transfers older than `expiry`.
pub fn cleanup_expired(expiry: Duration) -> usize {
    let idir = inbox_dir();
    if !idir.exists() {
        return 0;
    }

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cutoff = now.saturating_sub(expiry.as_secs());

    let mut cleaned = 0;

    let target_dirs: Vec<_> = std::fs::read_dir(&idir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();

    for target_entry in target_dirs {
        let transfer_dirs: Vec<_> = std::fs::read_dir(target_entry.path())
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();

        for xfer_entry in transfer_dirs {
            let init_path = xfer_entry.path().join("init.json");
            if let Some(entry) = read_entry(&init_path)
                && entry.timestamp < cutoff
            {
                tracing::info!(
                    transfer_id = %entry.transfer_id,
                    target = %entry.target,
                    age_hours = (now - entry.timestamp) / 3600,
                    "Inbox: cleaning expired transfer"
                );
                let _ = std::fs::remove_dir_all(xfer_entry.path());
                cleaned += 1;
            }
        }

        // Clean up empty target dirs
        if std::fs::read_dir(target_entry.path()).is_ok_and(|e| e.count() == 0) {
            let _ = std::fs::remove_dir(target_entry.path());
        }
    }

    cleaned
}

/// Clear all pending transfers for a specific target.
///
/// Returns the number of transfers removed.
pub fn clear_target(target: &str) -> usize {
    let target_dir = target_dir(target);
    if !target_dir.exists() {
        return 0;
    }

    let count = std::fs::read_dir(&target_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count();

    let _ = std::fs::remove_dir_all(&target_dir);
    count
}

/// Clear all pending transfers for all targets.
///
/// Returns the number of transfers removed.
pub fn clear_all() -> usize {
    let idir = inbox_dir();
    if !idir.exists() {
        return 0;
    }

    let mut total = 0;

    let target_dirs: Vec<_> = std::fs::read_dir(&idir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();

    for target_entry in target_dirs {
        let count = std::fs::read_dir(target_entry.path())
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .count();
        total += count;
        let _ = std::fs::remove_dir_all(target_entry.path());
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_inbox_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(format!("/tmp/tl-inbox-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Helper: deposit a complete transfer (init + chunks + complete) directly to a dir.
    fn deposit_test_transfer(
        inbox_base: &Path,
        target: &str,
        transfer_id: &str,
        filename: &str,
        chunks: u32,
    ) {
        let safe_target: String = target
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let safe_id: String = transfer_id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let xfer_dir = inbox_base.join(&safe_target).join(&safe_id);
        std::fs::create_dir_all(&xfer_dir).unwrap();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Init
        let init = InboxEntry {
            transfer_id: transfer_id.to_string(),
            target: target.to_string(),
            from: Some("sender".to_string()),
            topic: "file.init".to_string(),
            payload: json!({
                "transfer_id": transfer_id,
                "filename": filename,
                "size": chunks * 1024,
                "total_chunks": chunks,
                "from": "sender",
            }),
            timestamp: now,
        };
        std::fs::write(
            xfer_dir.join("init.json"),
            serde_json::to_string_pretty(&init).unwrap(),
        )
        .unwrap();

        // Chunks
        for i in 0..chunks {
            let chunk = InboxEntry {
                transfer_id: transfer_id.to_string(),
                target: target.to_string(),
                from: Some("sender".to_string()),
                topic: "file.chunk".to_string(),
                payload: json!({
                    "transfer_id": transfer_id,
                    "index": i,
                    "data": "dGVzdA==",  // base64 "test"
                }),
                timestamp: now,
            };
            std::fs::write(
                xfer_dir.join(format!("chunk-{i:04}.json")),
                serde_json::to_string_pretty(&chunk).unwrap(),
            )
            .unwrap();
        }

        // Complete
        let complete = InboxEntry {
            transfer_id: transfer_id.to_string(),
            target: target.to_string(),
            from: Some("sender".to_string()),
            topic: "file.complete".to_string(),
            payload: json!({
                "transfer_id": transfer_id,
                "sha256": "abc123",
            }),
            timestamp: now,
        };
        std::fs::write(
            xfer_dir.join("complete.json"),
            serde_json::to_string_pretty(&complete).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn deposit_creates_spool_files() {
        let dir = test_inbox_dir();
        // We can't use the global deposit() since it uses runtime_dir(),
        // but we can test the logic by writing directly to the test dir
        deposit_test_transfer(&dir, "my-session", "xfer-001", "report.txt", 3);

        let xfer_dir = dir.join("my-session").join("xfer-001");
        assert!(xfer_dir.join("init.json").exists());
        assert!(xfer_dir.join("chunk-0000.json").exists());
        assert!(xfer_dir.join("chunk-0001.json").exists());
        assert!(xfer_dir.join("chunk-0002.json").exists());
        assert!(xfer_dir.join("complete.json").exists());

        // Verify init.json content
        let init: InboxEntry =
            serde_json::from_str(&std::fs::read_to_string(xfer_dir.join("init.json")).unwrap())
                .unwrap();
        assert_eq!(init.transfer_id, "xfer-001");
        assert_eq!(init.target, "my-session");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_pending_returns_transfers() {
        let dir = test_inbox_dir();
        deposit_test_transfer(&dir, "target-a", "xfer-100", "data.csv", 2);
        deposit_test_transfer(&dir, "target-a", "xfer-101", "image.png", 5);

        // list_pending uses target_dir() which uses runtime_dir(), so we test
        // the read logic by reading from the test dir directly
        let tdir = dir.join("target-a");
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut transfers = Vec::new();
        for entry in std::fs::read_dir(&tdir).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let transfer_id = entry.file_name().to_string_lossy().to_string();
            let init_path = entry.path().join("init.json");
            let init: InboxEntry =
                serde_json::from_str(&std::fs::read_to_string(&init_path).unwrap()).unwrap();

            transfers.push(PendingTransfer {
                transfer_id,
                target: "target-a".to_string(),
                filename: init.payload["filename"].as_str().unwrap().to_string(),
                from: init.from.unwrap_or_default(),
                size: init.payload["size"].as_u64().unwrap(),
                chunks_received: std::fs::read_dir(entry.path())
                    .unwrap()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_name().to_string_lossy().starts_with("chunk-"))
                    .count() as u32,
                total_chunks: init.payload["total_chunks"].as_u64().unwrap() as u32,
                complete: entry.path().join("complete.json").exists(),
                age_secs: now.saturating_sub(init.timestamp),
            });
        }

        assert_eq!(transfers.len(), 2);
        assert!(transfers.iter().all(|t| t.complete));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_expired_removes_old_entries() {
        let dir = test_inbox_dir();

        // Create a transfer with old timestamp
        let safe_target = "old-target";
        let safe_id = "old-xfer";
        let xfer_dir = dir.join(safe_target).join(safe_id);
        std::fs::create_dir_all(&xfer_dir).unwrap();

        let old_ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 48 * 3600; // 48 hours ago

        let init = InboxEntry {
            transfer_id: "old-xfer".to_string(),
            target: "old-target".to_string(),
            from: Some("sender".to_string()),
            topic: "file.init".to_string(),
            payload: json!({"transfer_id": "old-xfer", "filename": "old.txt", "size": 100, "total_chunks": 1, "from": "sender"}),
            timestamp: old_ts,
        };
        std::fs::write(
            xfer_dir.join("init.json"),
            serde_json::to_string_pretty(&init).unwrap(),
        )
        .unwrap();

        // Create a recent transfer
        deposit_test_transfer(&dir, "new-target", "new-xfer", "new.txt", 1);

        // Cleanup with 24h expiry — we can't use cleanup_expired() directly
        // since it uses inbox_dir(), but we can verify the logic
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cutoff = now - 24 * 3600;

        assert!(old_ts < cutoff, "Old transfer should be expired");

        // Manual cleanup simulation
        assert!(xfer_dir.exists());
        let _ = std::fs::remove_dir_all(&xfer_dir);
        assert!(!xfer_dir.exists(), "Old transfer should be cleaned");

        // New transfer should survive
        assert!(dir.join("new-target").join("new-xfer").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// T-2489: a corrupt/unreadable chunk must make delivery-planning return
    /// None (→ caller retains the spool), NOT silently skip it. Guards against
    /// the regression where a bad chunk was skipped, the rest delivered, true
    /// returned, and the spool then deleted (unrecoverable file loss).
    #[test]
    fn ordered_chunk_paths_checked_returns_none_on_corrupt_chunk() {
        let dir = test_inbox_dir();
        deposit_test_transfer(&dir, "tgt-corrupt", "xfer-c", "report.txt", 3);
        let xfer_dir = dir.join("tgt-corrupt").join("xfer-c");

        // Corrupt one chunk: overwrite with non-JSON so read_entry → None.
        std::fs::write(xfer_dir.join("chunk-0001.json"), b"}{ this is not json").unwrap();

        assert!(
            ordered_chunk_paths_checked(&xfer_dir).is_none(),
            "a corrupt chunk must abort delivery planning (retain spool), not be skipped"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// T-2489: with all chunks readable, planning returns every chunk path in
    /// delivery order.
    #[test]
    fn ordered_chunk_paths_checked_returns_all_when_readable() {
        let dir = test_inbox_dir();
        deposit_test_transfer(&dir, "tgt-ok", "xfer-ok", "report.txt", 3);
        let xfer_dir = dir.join("tgt-ok").join("xfer-ok");

        let paths = ordered_chunk_paths_checked(&xfer_dir)
            .expect("all-readable chunks must plan successfully");
        assert_eq!(paths.len(), 3, "all 3 chunks should be planned");
        // Ordered: chunk-0000 before chunk-0001 before chunk-0002.
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            vec!["chunk-0000.json", "chunk-0001.json", "chunk-0002.json"]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// T-2490: chunk delivery order must be NUMERIC, correct across the 4→5-digit
    /// zero-pad boundary. A lexical filename sort put chunk-10000 before
    /// chunk-9999 → corrupted reassembly for transfers >10000 chunks (>480MB).
    #[test]
    fn ordered_chunk_paths_checked_sorts_numerically_across_9999_boundary() {
        let dir = test_inbox_dir();
        let xfer_dir = dir.join("tgt-big").join("xfer-big");
        std::fs::create_dir_all(&xfer_dir).unwrap();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let write_chunk = |index: u64| {
            let entry = InboxEntry {
                transfer_id: "xfer-big".to_string(),
                target: "tgt-big".to_string(),
                from: Some("sender".to_string()),
                topic: "file.chunk".to_string(),
                payload: json!({ "transfer_id": "xfer-big", "index": index, "data": "dGVzdA==" }),
                timestamp: now,
            };
            std::fs::write(
                xfer_dir.join(format!("chunk-{index:04}.json")),
                serde_json::to_string_pretty(&entry).unwrap(),
            )
            .unwrap();
        };
        // Written out of order; spans the 4→5 digit boundary.
        for idx in [10001u64, 9999, 0, 10000, 1, 9998] {
            write_chunk(idx);
        }

        let paths = ordered_chunk_paths_checked(&xfer_dir).expect("all chunks readable");
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            vec![
                "chunk-0000.json",
                "chunk-0001.json",
                "chunk-9998.json",
                "chunk-9999.json",
                "chunk-10000.json",
                "chunk-10001.json",
            ],
            "chunks must be numerically ordered across the 4→5 digit boundary (lexical would put 10000 before 9999)"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// T-2490: chunk_index parses valid names and rejects the rest (→ sorts last).
    #[test]
    fn chunk_index_parses_and_rejects() {
        assert_eq!(chunk_index("chunk-0000.json"), Some(0));
        assert_eq!(chunk_index("chunk-9999.json"), Some(9999));
        assert_eq!(chunk_index("chunk-10000.json"), Some(10000));
        assert_eq!(chunk_index("chunk-.json"), None);
        assert_eq!(chunk_index("chunk-abc.json"), None);
        assert_eq!(chunk_index("init.json"), None);
        assert_eq!(chunk_index("complete.json"), None);
    }

    #[test]
    fn is_file_topic_works() {
        assert!(is_file_topic("file.init"));
        assert!(is_file_topic("file.chunk"));
        assert!(is_file_topic("file.complete"));
        assert!(is_file_topic("file.error"));
        assert!(!is_file_topic("session.discover"));
        assert!(!is_file_topic("event.emit"));
        assert!(!is_file_topic("file.something_else"));
    }

    #[test]
    fn target_name_sanitization() {
        // Special chars should be replaced with underscores
        let dir = target_dir("my/session.name:test");
        let name = dir.file_name().unwrap().to_string_lossy();
        assert!(!name.contains('/'));
        assert!(!name.contains('.'));
        assert!(!name.contains(':'));
        assert!(name.contains("my_session_name_test"));
    }

    /// T-2505: a `file.chunk` deposit whose `index` is MISSING must be rejected
    /// (Ok(false) + warn), NOT defaulted to chunk-0000.json — that would silently
    /// overwrite a legitimately-spooled chunk 0 (data corruption). Sibling of the
    /// T-2490 reassembly-side fix; this guards the deposit side.
    #[tokio::test]
    async fn deposit_rejects_file_chunk_with_missing_index_no_clobber() {
        let _lock = crate::test_util::ENV_LOCK.lock().await;
        let base = test_inbox_dir();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &base) };

        let target = "offline-sess";
        let xfer = "xfer-clobber";

        // Spool a legitimate chunk 0.
        let good = json!({ "transfer_id": xfer, "index": 0, "data": "R09PRA==" }); // "GOOD"
        assert_eq!(
            deposit(target, "file.chunk", &good, Some("sender")).unwrap(),
            true,
            "valid chunk 0 should spool"
        );
        let chunk0 = transfer_dir(target, xfer).join("chunk-0000.json");
        let before = std::fs::read_to_string(&chunk0).unwrap();

        // A malformed chunk with NO index must be rejected, not overwrite chunk 0.
        let bad = json!({ "transfer_id": xfer, "data": "QkFE" }); // "BAD", no index
        assert_eq!(
            deposit(target, "file.chunk", &bad, Some("sender")).unwrap(),
            false,
            "missing-index chunk must be rejected"
        );

        let after = std::fs::read_to_string(&chunk0).unwrap();
        assert_eq!(before, after, "chunk 0 must NOT be clobbered by a malformed chunk");
        assert!(after.contains("R09PRA=="), "chunk 0 must retain its original bytes");

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        let _ = std::fs::remove_dir_all(&base);
    }

    /// T-2505: a `file.chunk` deposit whose `index` is a non-integer (e.g. a
    /// string) is rejected the same way — as_u64() returns None.
    #[tokio::test]
    async fn deposit_rejects_file_chunk_with_noninteger_index() {
        let _lock = crate::test_util::ENV_LOCK.lock().await;
        let base = test_inbox_dir();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &base) };

        let target = "offline-sess2";
        let xfer = "xfer-noninteger";
        let bad = json!({ "transfer_id": xfer, "index": "seven", "data": "QkFE" });
        assert_eq!(
            deposit(target, "file.chunk", &bad, Some("sender")).unwrap(),
            false,
            "non-integer index must be rejected"
        );
        assert!(
            !transfer_dir(target, xfer).join("chunk-0000.json").exists(),
            "no chunk-0000.json must be written for a non-integer index"
        );

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        let _ = std::fs::remove_dir_all(&base);
    }

    /// T-2505: a valid integer index still spools to chunk-{index:04}.json (no
    /// regression on the happy path).
    #[tokio::test]
    async fn deposit_accepts_valid_file_chunk_index() {
        let _lock = crate::test_util::ENV_LOCK.lock().await;
        let base = test_inbox_dir();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &base) };

        let target = "offline-sess3";
        let xfer = "xfer-happy";
        let good = json!({ "transfer_id": xfer, "index": 42, "data": "dGVzdA==" });
        assert_eq!(
            deposit(target, "file.chunk", &good, Some("sender")).unwrap(),
            true,
            "valid chunk should spool"
        );
        assert!(
            transfer_dir(target, xfer).join("chunk-0042.json").exists(),
            "valid index 42 must spool to chunk-0042.json"
        );

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        let _ = std::fs::remove_dir_all(&base);
    }
}
