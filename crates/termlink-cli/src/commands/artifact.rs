//! CLI wrappers around the session crate's artifact upload and download helpers, so a shell script can move artifact bytes.
//! T-3134 (arc-011 S2): thin CLI wrappers over the existing
//! `send_artifact_via_client` / `download_artifact_via_client` helpers
//! (`crates/termlink-session/src/artifact.rs`, landed in T-1164b/T-1248/T-1249).
//!
//! Scope Fence (T-3076): no change to the `artifact.put`/`artifact.get` wire
//! protocol, the hub-side router, or the capability-fallback logic — this file
//! only gives a shell script a way to move the bytes those already-shipped
//! functions know how to move. Local hub only, mirroring `file send`'s own
//! `resolve_hub_paths()` pattern; no remote/fleet routing.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};

use termlink_session::artifact::{
    download_artifact_via_client, send_artifact_via_client, ArtifactManifest, SendOutcome,
    SendPath,
};
use termlink_session::client;
use termlink_session::hub_capabilities::HubCapabilitiesCache;
use termlink_session::inbox_channel::FallbackCtx;
use termlink_protocol::TransportAddr;

use crate::util::generate_request_id;

fn timeout_message(operation: &str, timeout: Duration, detail: &str) -> String {
    format!(
        "{operation} timed out after {timeout:?} ({detail}) — \
         is the local hub up? check `termlink hub status`, then retry"
    )
}

/// Do the positional `<sha256>` and `--expected-sha256` arguments agree?
///
/// IW-3 (T-3076) made `--expected-sha256` MANDATORY on `get`, unlike `file
/// receive`'s optional flag — the sha256 IS the fetch key, so an explicit
/// mismatch between what the caller asked to fetch and what they said they
/// expected is a caller-side bug worth catching before any network call, not
/// after. Case-insensitive, matching `reconcile_expected_sha256`'s convention
/// in `commands/file.rs`. Pure and unit-testable — no live hub needed.
pub(crate) fn sha256_args_agree(requested: &str, expected: &str) -> bool {
    requested.trim().eq_ignore_ascii_case(expected.trim())
}

/// T-3661: which inbox topic (without the `inbox:` prefix) an `artifact put --to <to>`
/// notice goes to. Since the sidecar scheme, an agent's inbox is hub-qualified,
/// `inbox:<hub_id>/<agent>` (`termlink_sidecar::send::inbox_topic`); posting to the bare
/// `inbox:<agent>` failed with -32013 unknown topic AFTER the bytes were stored (055 report).
///
/// Order: a `to` that already names a hub (`<hub_id>/<agent>`) is used as given; else the
/// hub-qualified inbox when it exists on this hub; else a legacy bare `inbox:<to>` when
/// that exists. Neither existing is an error naming both, raised BEFORE any upload.
pub(crate) fn pick_inbox_target(
    to: &str,
    hub_id: Option<&str>,
    existing_topics: &[String],
) -> Result<String, String> {
    if to.contains('/') {
        return Ok(to.to_string());
    }
    let has = |t: &str| existing_topics.iter().any(|e| e == t);
    if let Some(h) = hub_id {
        let qualified = format!("{h}/{to}");
        if has(&format!("inbox:{qualified}")) {
            return Ok(qualified);
        }
    }
    if has(&format!("inbox:{to}")) {
        return Ok(to.to_string());
    }
    let looked = match hub_id {
        Some(h) => format!("inbox:{h}/{to} or inbox:{to}"),
        None => format!("inbox:{to} (the hub states no hub_id)"),
    };
    Err(format!(
        "no inbox for '{to}' on this hub: looked for {looked}. Is the agent's sidecar \
         running on this hub? (cross-hub delivery is not supported here, T-3614)"
    ))
}

/// T-3661: ask the local hub for its id and its `inbox:` topics, then pick the target.
async fn resolve_inbox_target(client: &mut client::Client, to: &str) -> Result<String, String> {
    use termlink_protocol::jsonrpc::RpcResponse;
    let hub_id = match client.call("hub.version", serde_json::json!("artifact-hub-id"), serde_json::json!({})).await {
        Ok(RpcResponse::Success(ok)) => ok.result.get("hub_id").and_then(|v| v.as_str()).map(str::to_owned),
        _ => None,
    };
    let topics: Vec<String> = match client
        .call(
            termlink_protocol::control::method::CHANNEL_LIST,
            serde_json::json!("artifact-inboxes"),
            serde_json::json!({"prefix": "inbox:"}),
        )
        .await
    {
        Ok(RpcResponse::Success(ok)) => ok.result["topics"]
            .as_array()
            .map(|a| a.iter().filter_map(|t| t["name"].as_str().map(str::to_owned)).collect())
            .unwrap_or_default(),
        Ok(RpcResponse::Error(e)) => {
            return Err(format!("channel.list error {}: {} (could not look up the inbox)", e.error.code, e.error.message));
        }
        Err(e) => return Err(format!("channel.list: {e} (could not look up the inbox)")),
    };
    pick_inbox_target(to, hub_id.as_deref(), &topics)
}

pub(crate) async fn cmd_artifact_put(
    path: &str,
    to: &str,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let file_path = Path::new(path);
    let file_data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("Failed to read file '{path}': {e}");
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": to, "error": msg}));
            }
            anyhow::bail!(msg);
        }
    };
    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());
    let size = file_data.len() as u64;

    let (_, hub_socket) = super::infrastructure::resolve_hub_paths();
    if !hub_socket.exists() {
        let msg = "No local hub socket found — `termlink artifact put` is local-hub-only \
                    (see `termlink hub status`); no hub to upload to"
            .to_string();
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "target": to, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    let identity = super::channel::load_identity_or_create()?;
    let addr = TransportAddr::unix(&hub_socket);
    let mut client = client::Client::connect_addr(&addr)
        .await
        .with_context(|| format!("connect hub at {}", hub_socket.display()))?;
    let inbox_target = match resolve_inbox_target(&mut client, to).await {
        Ok(t) => t,
        Err(msg) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": to, "error": msg}));
            }
            anyhow::bail!(msg);
        }
    };
    let host_port = format!("local:{}", hub_socket.display());
    let cache = HubCapabilitiesCache::new();
    let mut ctx = FallbackCtx::new();
    let manifest = ArtifactManifest {
        filename: filename.clone(),
        size,
        // Must equal the signing identity's fingerprint, not an arbitrary label:
        // `send_artifact_via_client` copies `manifest.from` verbatim into the
        // signed `channel.post` envelope's `sender_id`, and the hub rejects a
        // sender_id that doesn't match the pubkey-derived fingerprint (T-1427,
        // CHANNEL_IDENTITY_MISMATCH -32014). Discovered live while proving this
        // AC — see T-3140 for the same latent bug in the three pre-existing
        // callers (file.rs/remote.rs/tools.rs all use a `cli-<pid>`-style label).
        from: identity.fingerprint().to_string(),
        transfer_id: Some(generate_request_id().replace("req-", "xfer-")),
        content_type: None,
    };

    let timeout = Duration::from_secs(timeout_secs);
    let outcome = tokio::time::timeout(
        timeout,
        send_artifact_via_client(
            &mut client,
            &host_port,
            &inbox_target,
            &file_data,
            &manifest,
            &identity,
            &cache,
            &mut ctx,
        ),
    )
    .await
    .map_err(|_| anyhow::anyhow!("{}", timeout_message("artifact.put", timeout, to)))?
    .with_context(|| "send_artifact_via_client failed")?;

    match outcome {
        SendOutcome::LegacyOnly => {
            let msg = format!(
                "peer '{to}' hub does not advertise artifact.put/channel.post — \
                 fall back to `termlink file send {to} {path}` (deprecated)"
            );
            if json {
                super::json_error_exit(
                    serde_json::json!({"ok": false, "target": to, "error": msg, "legacy_only": true}),
                );
            }
            anyhow::bail!(msg);
        }
        SendOutcome::Sent { sha256, channel_offset, total_bytes, path: used_path } => {
            let via = match used_path {
                SendPath::Inline => "channel.inline",
                SendPath::Chunked => "channel.artifact",
            };
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "filename": filename,
                        "size": total_bytes,
                        "sha256": sha256,
                        "target": to,
                        "topic": format!("inbox:{inbox_target}"),
                        "channel_offset": channel_offset,
                        "via": via,
                    })
                );
            } else {
                println!(
                    "Uploaded {filename} ({total_bytes} bytes) to inbox:{inbox_target} via {via}"
                );
                println!("  sha256:         {sha256}");
                println!("  channel_offset: {channel_offset}");
                println!(
                    "  fetch with:     termlink artifact get {sha256} --expected-sha256 {sha256} -o <path>"
                );
            }
            Ok(())
        }
    }
}

pub(crate) async fn cmd_artifact_get(
    sha256: &str,
    expected_sha256: &str,
    output: &str,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    if !sha256_args_agree(sha256, expected_sha256) {
        let msg = format!(
            "SHA-256 argument mismatch: fetching {sha256} but --expected-sha256 says \
             {expected_sha256} — refusing before contacting the hub. Both must name the \
             same artifact (IW-3: --expected-sha256 is a mandatory confirmation, not an \
             independent value)."
        );
        if json {
            super::json_error_exit(serde_json::json!({
                "ok": false,
                "error": msg,
                "sha256_requested": sha256,
                "sha256_expected": expected_sha256,
            }));
        }
        anyhow::bail!(msg);
    }

    let (_, hub_socket) = super::infrastructure::resolve_hub_paths();
    if !hub_socket.exists() {
        let msg = "No local hub socket found — `termlink artifact get` is local-hub-only \
                    (see `termlink hub status`); no hub to download from"
            .to_string();
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    let addr = TransportAddr::unix(&hub_socket);
    let mut client = client::Client::connect_addr(&addr)
        .await
        .with_context(|| format!("connect hub at {}", hub_socket.display()))?;

    let timeout = Duration::from_secs(timeout_secs);
    let bytes = tokio::time::timeout(timeout, download_artifact_via_client(&mut client, sha256))
        .await
        .map_err(|_| anyhow::anyhow!("{}", timeout_message("artifact.get", timeout, sha256)))?;
    let bytes = match bytes {
        Ok(b) => b,
        Err(e) => {
            let msg = format!("download_artifact_via_client {sha256}: {e}");
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "error": msg,
                    "sha256": sha256,
                }));
            }
            anyhow::bail!(msg);
        }
    };

    let size = bytes.len() as u64;
    if let Err(e) = std::fs::write(output, &bytes) {
        let msg = format!("Failed to write '{output}': {e}");
        if json {
            super::json_error_exit(serde_json::json!({
                "ok": false,
                "error": msg,
                "sha256": sha256,
                "size": size,
            }));
        }
        anyhow::bail!(msg);
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "path": output,
                "size": size,
                "sha256": sha256,
                "sha256_verified": true,
            })
        );
    } else {
        println!("Saved {output} ({size} bytes)");
        println!("  sha256: {sha256} (verified against expected)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_args_agree_matches_identical() {
        let h = "a".repeat(64);
        assert!(sha256_args_agree(&h, &h));
    }

    #[test]
    fn sha256_args_agree_case_insensitive() {
        assert!(sha256_args_agree(&"AB".repeat(32), &"ab".repeat(32)));
    }

    #[test]
    fn sha256_args_agree_trims_whitespace() {
        let h = "c".repeat(64);
        let padded = format!(" {h}\n");
        assert!(sha256_args_agree(&padded, &h));
    }

    #[test]
    fn sha256_args_agree_rejects_mismatch() {
        assert!(!sha256_args_agree(&"a".repeat(64), &"b".repeat(64)));
    }

    #[test]
    fn sha256_args_agree_rejects_prefix_only_match() {
        // A truncated/typo'd hash must not silently pass as "close enough".
        let full = "d".repeat(64);
        let truncated = "d".repeat(32);
        assert!(!sha256_args_agree(&full, &truncated));
    }

    // T-3661: the 055 report — the notice went to bare inbox:<agent> while the real inbox
    // was hub-qualified, so put exited 1 after a good upload.
    fn topics(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn pick_inbox_target_prefers_hub_qualified_inbox() {
        let t = topics(&["inbox:cacc73ea32b121dd/055-agentic-fleet-cockpit", "inbox:055-agentic-fleet-cockpit"]);
        assert_eq!(
            pick_inbox_target("055-agentic-fleet-cockpit", Some("cacc73ea32b121dd"), &t).unwrap(),
            "cacc73ea32b121dd/055-agentic-fleet-cockpit"
        );
    }

    #[test]
    fn pick_inbox_target_only_qualified_exists_is_the_055_case() {
        let t = topics(&["inbox:cacc73ea32b121dd/055-agentic-fleet-cockpit"]);
        assert_eq!(
            pick_inbox_target("055-agentic-fleet-cockpit", Some("cacc73ea32b121dd"), &t).unwrap(),
            "cacc73ea32b121dd/055-agentic-fleet-cockpit"
        );
    }

    #[test]
    fn pick_inbox_target_falls_back_to_legacy_bare_inbox() {
        let t = topics(&["inbox:old-agent"]);
        assert_eq!(pick_inbox_target("old-agent", Some("abc"), &t).unwrap(), "old-agent");
    }

    #[test]
    fn pick_inbox_target_qualified_to_is_used_as_given() {
        assert_eq!(pick_inbox_target("abc/agent", Some("zzz"), &[]).unwrap(), "abc/agent");
    }

    #[test]
    fn pick_inbox_target_no_inbox_is_an_error_naming_both() {
        let e = pick_inbox_target("ghost", Some("abc"), &topics(&["inbox:abc/other"])).unwrap_err();
        assert!(e.contains("inbox:abc/ghost") && e.contains("inbox:ghost"), "{e}");
    }
}
