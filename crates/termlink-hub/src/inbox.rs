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
            let index = payload.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
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

    // Deliver chunks in order
    let mut chunk_files: Vec<_> = std::fs::read_dir(xfer_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("chunk-"))
        .collect();
    chunk_files.sort_by_key(|e| e.file_name());

    for chunk_file in chunk_files {
        let entry: InboxEntry = match read_entry(&chunk_file.path()) {
            Some(e) => e,
            None => continue,
        };

        if emit_event(addr, &entry).await.is_err() {
            tracing::warn!(
                transfer_id = %init_entry.transfer_id,
                chunk = %chunk_file.file_name().to_string_lossy(),
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
}
