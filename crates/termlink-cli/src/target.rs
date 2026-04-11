//! Shared cross-host routing helpers — T-924.
//!
//! Any CLI command that is naturally cross-host (session-scoped RPCs like
//! ping, status, kv, inject, etc.) should flatten `TargetOpts` into its arg
//! struct and call `call_session` instead of hand-rolling the
//! `client::rpc_call(reg.socket_path(), ...)` vs.
//! `connect_remote_hub + rpc.call` split.
//!
//! The forwarding mechanism itself lives in the hub (`router.rs` fallthrough
//! to `forward_to_target`), verified end-to-end by T-923.
//!
//! Consumers are wired in by T-925..T-935; until then every public item is
//! "unused" from the binary's perspective but exercised by the unit tests
//! below.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use serde_json::Value;

use termlink_session::{client, manager};

use crate::commands::remote::connect_remote_hub;

/// Target routing options shared by every cross-host CLI command.
#[derive(Debug, Clone, Args)]
pub(crate) struct TargetOpts {
    /// Remote hub address, e.g. "host:4112". When set, the command is
    /// forwarded through the hub; when omitted, the command runs against a
    /// local session via its unix socket.
    ///
    /// Rust field is named `hub` so it does not collide with consumer
    /// commands whose own `target` field historically names a session (e.g.
    /// `Ping { target: Option<String> }`). The CLI flag stays `--target`
    /// per the T-921 inception decision.
    #[arg(long = "target", global = true)]
    pub hub: Option<String>,

    /// Path to a hex-encoded HMAC secret file for the remote hub.
    #[arg(long = "secret-file", global = true)]
    pub secret_file: Option<PathBuf>,

    /// Explicit hex-encoded HMAC secret (64 chars / 32 bytes). Prefer
    /// `--secret-file` for shell-history hygiene.
    #[arg(long, global = true)]
    pub secret: Option<String>,

    /// Auth scope to request from the hub: observe | interact | control |
    /// execute. Defaults to the per-method minimum when omitted.
    #[arg(long, global = true)]
    pub scope: Option<String>,

    /// Target session — ID or display name. Required for session-scoped
    /// commands.
    #[arg(long = "session", short = 's')]
    pub session: String,
}

impl TargetOpts {
    /// Resolve the HMAC secret as 32 raw bytes. Precedence:
    ///   1. `--secret` explicit hex
    ///   2. `--secret-file` path
    ///   3. `$HOME/.termlink/secrets/<host>.hex` (implicit, only when
    ///      `--target` is set)
    pub(crate) fn resolve_secret(&self) -> Result<Vec<u8>> {
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        self.resolve_secret_with_home(home.as_deref())
    }

    /// Same as `resolve_secret` but with an injectable HOME directory for
    /// tests.
    fn resolve_secret_with_home(&self, home: Option<&Path>) -> Result<Vec<u8>> {
        let hex = if let Some(h) = &self.secret {
            h.clone()
        } else if let Some(path) = &self.secret_file {
            std::fs::read_to_string(path)
                .with_context(|| format!("Secret file not found: {}", path.display()))?
                .trim()
                .to_string()
        } else if let Some(hub) = &self.hub {
            let (host, _port) = parse_hub_addr(hub)?;
            let home = home.context(
                "Cannot locate $HOME for implicit secret lookup — set HOME or pass --secret-file",
            )?;
            let path = home
                .join(".termlink")
                .join("secrets")
                .join(format!("{host}.hex"));
            std::fs::read_to_string(&path)
                .with_context(|| {
                    format!("Implicit secret not found at {}", path.display())
                })?
                .trim()
                .to_string()
        } else {
            anyhow::bail!(
                "No secret source: provide --secret, --secret-file, or --target with a configured secret at $HOME/.termlink/secrets/<host>.hex"
            );
        };
        parse_hex_secret(&hex)
    }

    /// Resolve the scope for a given method: explicit `--scope` first, then
    /// the per-method minimum from `default_scope_for`. Always validates.
    pub(crate) fn resolve_scope(&self, method: &str) -> Result<&'static str> {
        let s = self.scope.as_deref().unwrap_or_else(|| default_scope_for(method));
        normalize_scope(s)
    }
}

/// Parse a `HOST:PORT` hub address. Returns `(host, port)`.
fn parse_hub_addr(hub: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = hub.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid hub address '{}'. Expected HOST:PORT", hub);
    }
    if parts[0].is_empty() {
        anyhow::bail!("Invalid hub address '{}'. Host is empty", hub);
    }
    let port: u16 = parts[1]
        .parse()
        .with_context(|| format!("Invalid port in '{hub}'"))?;
    Ok((parts[0].to_string(), port))
}

/// Parse a hex-encoded 32-byte secret.
fn parse_hex_secret(hex: &str) -> Result<Vec<u8>> {
    if hex.len() != 64 {
        anyhow::bail!(
            "Secret must be 64 hex characters (32 bytes), got {}",
            hex.len()
        );
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .context("Secret contains invalid hex characters")
}

/// Validate a scope string and return the canonical form the hub expects.
fn normalize_scope(s: &str) -> Result<&'static str> {
    match s {
        "observe" => Ok("observe"),
        "interact" => Ok("interact"),
        "control" => Ok("control"),
        "execute" => Ok("execute"),
        other => anyhow::bail!(
            "Invalid scope '{}'. Use: observe, interact, control, execute",
            other
        ),
    }
}

/// Per-method minimum scope default. Mirrors
/// `termlink_session::auth::method_scope` at a higher semantic level so this
/// helper can feed strings directly into `connect_remote_hub`.
pub(crate) fn default_scope_for(method: &str) -> &'static str {
    match method {
        // Observe — read-only
        "termlink.ping"
        | "query.status"
        | "query.output"
        | "event.poll"
        | "kv.get"
        | "kv.list"
        | "session.discover" => "observe",
        // Interact — writes
        "event.emit"
        | "event.broadcast"
        | "session.update"
        | "kv.set"
        | "kv.delete" => "interact",
        // Control — lifecycle
        "session.spawn" | "session.stop" => "control",
        // Execute — PTY injection
        "command.inject" | "command.run" => "execute",
        // Safe default for unknown methods — requires write privileges
        _ => "interact",
    }
}

/// Dispatch an RPC call to a session, routing through the hub when
/// `opts.hub` is set and hitting the session's local unix socket otherwise.
/// Returns the `result` payload on success.
pub(crate) async fn call_session(
    opts: &TargetOpts,
    method: &str,
    params: Value,
) -> Result<Value> {
    use termlink_protocol::jsonrpc::RpcResponse;

    if let Some(hub) = &opts.hub {
        // Cross-host path: forward through the hub.
        let secret_bytes = opts.resolve_secret()?;
        let secret_hex: String =
            secret_bytes.iter().map(|b| format!("{b:02x}")).collect();
        let scope = opts.resolve_scope(method)?;

        let mut client =
            connect_remote_hub(hub, None, Some(&secret_hex), scope).await?;

        // Inject `target` into params so the hub forwarder can resolve it.
        let params = inject_target(params, &opts.session)?;

        match client
            .call(method, serde_json::json!("call"), params)
            .await?
        {
            RpcResponse::Success(r) => Ok(r.result),
            RpcResponse::Error(e) => {
                anyhow::bail!("{} {}", e.error.code, e.error.message)
            }
        }
    } else {
        // Local path: dial the session's unix socket directly.
        let reg = manager::find_session(&opts.session)
            .with_context(|| format!("Session not found: {}", opts.session))?;
        let resp = client::rpc_call(reg.socket_path(), method, params)
            .await
            .with_context(|| format!("RPC {method} failed"))?;
        client::unwrap_result(resp).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

/// Merge `target` into a JSON-RPC params object, accepting object or null.
fn inject_target(params: Value, session: &str) -> Result<Value> {
    match params {
        Value::Object(mut m) => {
            m.insert("target".to_string(), Value::String(session.to_string()));
            Ok(Value::Object(m))
        }
        Value::Null => Ok(serde_json::json!({ "target": session })),
        other => anyhow::bail!(
            "params must be a JSON object or null, got: {other}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn base_opts() -> TargetOpts {
        TargetOpts {
            hub: None,
            secret_file: None,
            secret: None,
            scope: None,
            session: "sess-1".into(),
        }
    }

    // A known-good 32-byte hex secret.
    const HEX_32: &str =
        "0011223344556677889900aabbccddeeff00112233445566778899aabbccddee";

    #[test]
    fn resolve_secret_prefers_explicit_hex() {
        let mut opts = base_opts();
        opts.hub = Some("example.com:4112".into());
        opts.secret = Some(HEX_32.into());
        opts.secret_file = Some(PathBuf::from("/no/such/file")); // would error if read

        let bytes = opts.resolve_secret_with_home(None).unwrap();
        assert_eq!(bytes.len(), 32);
        assert_eq!(bytes[0], 0x00);
        assert_eq!(bytes[1], 0x11);
        assert_eq!(bytes[31], 0xee);
    }

    #[test]
    fn resolve_secret_reads_secret_file_when_no_explicit() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("k.hex");
        std::fs::write(&path, format!("{HEX_32}\n")).unwrap(); // trailing newline handled

        let mut opts = base_opts();
        opts.hub = Some("example.com:4112".into());
        opts.secret_file = Some(path);

        let bytes = opts.resolve_secret_with_home(None).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn resolve_secret_reads_implicit_path_from_home_when_target_set() {
        let dir = tempdir().unwrap();
        let secrets_dir = dir.path().join(".termlink").join("secrets");
        std::fs::create_dir_all(&secrets_dir).unwrap();
        std::fs::write(secrets_dir.join("hostA.hex"), HEX_32).unwrap();

        let mut opts = base_opts();
        opts.hub = Some("hostA:4112".into());

        let bytes = opts.resolve_secret_with_home(Some(dir.path())).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn resolve_secret_errors_when_target_set_but_no_source_available() {
        let dir = tempdir().unwrap(); // empty — no secrets file
        let mut opts = base_opts();
        opts.hub = Some("nowhere:4112".into());

        let err = opts
            .resolve_secret_with_home(Some(dir.path()))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("Implicit secret not found"),
            "expected implicit-not-found error, got: {err}"
        );
    }

    #[test]
    fn resolve_secret_errors_when_no_source_and_no_target() {
        let opts = base_opts(); // nothing set — local-only mode doesn't need a secret
        let err = opts
            .resolve_secret_with_home(None)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("No secret source"),
            "expected no-source error, got: {err}"
        );
    }

    #[test]
    fn resolve_secret_rejects_invalid_hex_length() {
        let mut opts = base_opts();
        opts.secret = Some("deadbeef".into()); // too short

        let err = opts
            .resolve_secret_with_home(None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("64 hex characters"), "got: {err}");
    }

    #[test]
    fn resolve_secret_rejects_invalid_hex_chars() {
        let mut opts = base_opts();
        // 64 chars but with a non-hex character in the middle.
        opts.secret = Some(
            "0011223344556677889900aabbccddeeff00112233445566778899aabbccddZZ"
                .into(),
        );

        let err = opts
            .resolve_secret_with_home(None)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("invalid hex characters"),
            "got: {err}"
        );
    }

    #[test]
    fn parse_hub_addr_rejects_missing_port() {
        let err = parse_hub_addr("example.com").unwrap_err().to_string();
        assert!(err.contains("HOST:PORT"), "got: {err}");
    }

    #[test]
    fn parse_hub_addr_rejects_non_numeric_port() {
        let err = parse_hub_addr("example.com:abc").unwrap_err().to_string();
        assert!(err.contains("Invalid port"), "got: {err}");
    }

    #[test]
    fn parse_hub_addr_rejects_empty_host() {
        let err = parse_hub_addr(":4112").unwrap_err().to_string();
        assert!(err.contains("Host is empty"), "got: {err}");
    }

    #[test]
    fn parse_hub_addr_accepts_host_and_port() {
        let (host, port) = parse_hub_addr("192.168.1.5:4112").unwrap();
        assert_eq!(host, "192.168.1.5");
        assert_eq!(port, 4112);
    }

    #[test]
    fn normalize_scope_rejects_unknown() {
        let err = normalize_scope("admin").unwrap_err().to_string();
        assert!(err.contains("Invalid scope"), "got: {err}");
        assert!(err.contains("execute"), "should list valid scopes: {err}");
    }

    #[test]
    fn normalize_scope_accepts_four_canonical() {
        for s in ["observe", "interact", "control", "execute"] {
            assert_eq!(normalize_scope(s).unwrap(), s);
        }
    }

    #[test]
    fn default_scope_matches_per_method_semantics() {
        assert_eq!(default_scope_for("termlink.ping"), "observe");
        assert_eq!(default_scope_for("query.status"), "observe");
        assert_eq!(default_scope_for("kv.get"), "observe");
        assert_eq!(default_scope_for("kv.set"), "interact");
        assert_eq!(default_scope_for("event.emit"), "interact");
        assert_eq!(default_scope_for("session.spawn"), "control");
        assert_eq!(default_scope_for("command.inject"), "execute");
        // Unknown methods default to interact — safer than observe.
        assert_eq!(default_scope_for("future.method.we.have.not.seen"), "interact");
    }

    #[test]
    fn resolve_scope_uses_explicit_when_set() {
        let mut opts = base_opts();
        opts.scope = Some("control".into());
        assert_eq!(opts.resolve_scope("termlink.ping").unwrap(), "control");
    }

    #[test]
    fn resolve_scope_falls_back_to_per_method_default() {
        let opts = base_opts();
        assert_eq!(opts.resolve_scope("termlink.ping").unwrap(), "observe");
        assert_eq!(opts.resolve_scope("kv.set").unwrap(), "interact");
    }

    #[test]
    fn resolve_scope_rejects_invalid_explicit() {
        let mut opts = base_opts();
        opts.scope = Some("god-mode".into());
        let err = opts.resolve_scope("termlink.ping").unwrap_err().to_string();
        assert!(err.contains("Invalid scope"), "got: {err}");
    }

    #[test]
    fn inject_target_merges_into_object_params() {
        let params = serde_json::json!({ "key": "k", "value": "v" });
        let merged = inject_target(params, "sess-42").unwrap();
        assert_eq!(merged["key"], "k");
        assert_eq!(merged["value"], "v");
        assert_eq!(merged["target"], "sess-42");
    }

    #[test]
    fn inject_target_converts_null_params() {
        let merged = inject_target(Value::Null, "sess-42").unwrap();
        assert_eq!(merged["target"], "sess-42");
        assert_eq!(merged.as_object().unwrap().len(), 1);
    }

    #[test]
    fn inject_target_rejects_non_object_non_null() {
        let err = inject_target(Value::Bool(true), "s")
            .unwrap_err()
            .to_string();
        assert!(err.contains("must be a JSON object or null"), "got: {err}");
    }
}
