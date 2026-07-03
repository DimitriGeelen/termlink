//! `termlink webhook` — CLI config-authoring verbs for the arc-004 webhook
//! fan-out feature (T-2336, Slice 5).
//!
//! Slices S1–S4 shipped the hub-side runtime (primitive, event-wiring,
//! retry/backoff/dead-letter, governor_status telemetry). The one authoring gap
//! left was that the ONLY way to configure targets was to hand-write the
//! `TERMLINK_WEBHOOK_CONFIG` JSON file. These verbs close it:
//!
//!   - `webhook add`  — merge a target into the config JSON (auto-adds the URL's
//!                      host to the SSRF allowlist), atomic temp+rename write.
//!   - `webhook list` — render configured targets + allowed_hosts, signing keys
//!                      REDACTED in human output (verbatim in `--json`).
//!   - `webhook test` — dispatch a signed sample payload by reusing
//!                      `webhook::dispatch`, so the SSRF host-allowlist guard and
//!                      HMAC signing run identically to production.
//!
//! Reuses `termlink_hub::webhook::{WebhookConfig, WebhookTarget, dispatch,
//! sign_payload, url_host, build_test_client}` — the config types and the
//! dispatch path are the exact ones the hub uses, so the CLI can never drift from
//! production behaviour.

use anyhow::{Context, Result};
use std::io::Read;
use std::path::{Path, PathBuf};

use termlink_hub::webhook::{self, WebhookConfig, WebhookTarget};

/// Resolve the config file path: `--config` flag wins, else the
/// `TERMLINK_WEBHOOK_CONFIG` env var. There is deliberately NO silent default
/// path — a webhook config is security-sensitive (signing keys + SSRF allowlist),
/// so an unspecified location is an error, not a guess.
fn resolve_config_path(config_flag: Option<&str>) -> Result<PathBuf> {
    if let Some(p) = config_flag {
        return Ok(PathBuf::from(p));
    }
    match std::env::var("TERMLINK_WEBHOOK_CONFIG") {
        Ok(p) if !p.is_empty() => Ok(PathBuf::from(p)),
        _ => anyhow::bail!(
            "no webhook config location: pass --config <PATH> or set TERMLINK_WEBHOOK_CONFIG"
        ),
    }
}

/// Read the config at `path`. A missing file is NOT an error — it resolves to an
/// empty (disabled) config, so `webhook add` can bootstrap a fresh file.
fn read_config(path: &Path) -> Result<WebhookConfig> {
    match std::fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw)
            .with_context(|| format!("parse webhook config at {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(WebhookConfig::default()),
        Err(e) => Err(e).with_context(|| format!("read webhook config at {}", path.display())),
    }
}

/// Atomically persist `cfg` to `path` (write temp + rename), pretty-printed.
fn write_config_atomic(path: &Path, cfg: &WebhookConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(cfg).context("serialize webhook config")?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create config dir {}", parent.display()))?;
        }
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json.as_bytes())
        .with_context(|| format!("write temp config {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename temp config into {}", path.display()))?;
    Ok(())
}

/// Generate a random 32-byte signing key as hex. Reads `/dev/urandom` directly —
/// dependency-free and cryptographically adequate on Linux (the deploy target).
fn generate_signing_key() -> Result<String> {
    let mut bytes = [0u8; 32];
    let mut f = std::fs::File::open("/dev/urandom")
        .context("open /dev/urandom to generate a signing key")?;
    f.read_exact(&mut bytes).context("read /dev/urandom")?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

// ── Pure helpers (unit-tested) ────────────────────────────────────────────────

/// Validate a target URL and return its host. Errors when the URL does not parse,
/// has no host, or is not http/https (mirrors the hub's dispatch-time guard so a
/// URL that would be refused at dispatch is refused at authoring time instead).
fn validate_url(url: &str) -> Result<String> {
    // Scheme check via prefix (the CLI does not depend on reqwest directly; host
    // parsing is delegated to the hub crate's `webhook::url_host`, the single
    // source of truth also used at dispatch time).
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        anyhow::bail!("target url must be http/https: '{}'", url);
    }
    webhook::url_host(url)
        .ok_or_else(|| anyhow::anyhow!("target url '{}' has no host component", url))
}

/// Ensure `host` is present exactly once in the SSRF allowlist. Returns true when
/// it was newly added (false when already present) — deny-by-default means an
/// added target whose host is absent would never dispatch, so `add` wires it in.
fn ensure_host_allowed(cfg: &mut WebhookConfig, host: &str) -> bool {
    if cfg.allowed_hosts.iter().any(|h| h == host) {
        return false;
    }
    cfg.allowed_hosts.push(host.to_string());
    true
}

/// Append a target to the config. Pure state transition (no host-allowlist side
/// effect — callers pair this with [`ensure_host_allowed`]).
fn merge_target(cfg: &mut WebhookConfig, target: WebhookTarget) {
    cfg.targets.push(target);
}

/// Redact a signing key for human display — never print the secret to a terminal
/// or log. Shows only a length hint so the operator can tell a key IS set.
fn redact_key(key: &str) -> String {
    if key.is_empty() {
        "<unset>".to_string()
    } else {
        format!("<redacted, {} chars>", key.chars().count())
    }
}

// ── Verb handlers ─────────────────────────────────────────────────────────────

/// `termlink webhook add` — merge a target into the config.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_webhook_add(
    url: &str,
    signing_key: Option<&str>,
    topics: Vec<String>,
    allowed_hosts: Vec<String>,
    config: Option<&str>,
    json: bool,
) -> Result<()> {
    let host = match validate_url(url) {
        Ok(h) => h,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "url": url, "error": e.to_string()}));
            }
            return Err(e);
        }
    };
    let path = resolve_config_path(config)?;
    let mut cfg = read_config(&path)?;

    let key = match signing_key {
        Some(k) => k.to_string(),
        None => generate_signing_key()?,
    };
    let key_generated = signing_key.is_none();

    let host_added = ensure_host_allowed(&mut cfg, &host);
    // Any extra operator-supplied allowlist hosts (e.g. a redirect target).
    let mut extra_hosts_added = Vec::new();
    for h in &allowed_hosts {
        if ensure_host_allowed(&mut cfg, h) {
            extra_hosts_added.push(h.clone());
        }
    }

    merge_target(
        &mut cfg,
        WebhookTarget {
            url: url.to_string(),
            signing_key: key.clone(),
            topics: topics.clone(),
        },
    );

    write_config_atomic(&path, &cfg)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "config": path.display().to_string(),
                "url": url,
                "host": host,
                "host_added_to_allowlist": host_added,
                "extra_hosts_added": extra_hosts_added,
                "topics": topics,
                "signing_key_generated": key_generated,
                "target_count": cfg.targets.len(),
            })
        );
    } else {
        println!("✓ webhook target added → {}", path.display());
        println!("  url:     {url}");
        println!("  host:    {host}{}", if host_added { " (added to allowlist)" } else { "" });
        if !topics.is_empty() {
            println!("  topics:  {}", topics.join(", "));
        } else {
            println!("  topics:  (none — this target never fires; add --topic '*' or a topic name)");
        }
        if key_generated {
            println!("  signing_key: {} (generated — share with the consumer to verify signatures)", key);
        } else {
            println!("  signing_key: {}", redact_key(&key));
        }
        println!("  targets now configured: {}", cfg.targets.len());
    }
    Ok(())
}

/// `termlink webhook list` — render configured targets + allowlist.
pub(crate) fn cmd_webhook_list(config: Option<&str>, json: bool) -> Result<()> {
    let path = resolve_config_path(config)?;
    let cfg = read_config(&path)?;

    if json {
        // Verbatim config (machine surface — the operator explicitly asked for it,
        // and it already lives in plaintext on disk).
        println!("{}", serde_json::to_string_pretty(&cfg)?);
        return Ok(());
    }

    println!("webhook config: {}", path.display());
    if !cfg.is_enabled() {
        println!("  webhook fan-out disabled (0 targets)");
        return Ok(());
    }
    println!("  allowed_hosts: {}", if cfg.allowed_hosts.is_empty() {
        "(none — every target is SSRF-refused!)".to_string()
    } else {
        cfg.allowed_hosts.join(", ")
    });
    println!("  targets ({}):", cfg.targets.len());
    for (i, t) in cfg.targets.iter().enumerate() {
        let host = webhook::url_host(&t.url).unwrap_or_else(|| "?".to_string());
        let allowed = cfg.allowed_hosts.iter().any(|h| h == &host);
        println!("    [{i}] {}", t.url);
        println!(
            "        host={host}{}  topics=[{}]  signing_key={}",
            if allowed { "" } else { " ⚠ NOT-ALLOWLISTED" },
            t.topics.join(", "),
            redact_key(&t.signing_key),
        );
    }
    Ok(())
}

/// `termlink webhook test` — dispatch a signed sample payload to a target,
/// reusing the production `webhook::dispatch` path (SSRF guard + HMAC signing).
pub(crate) async fn cmd_webhook_test(
    url: &str,
    signing_key: Option<&str>,
    allowed_hosts: Vec<String>,
    topic: Option<&str>,
    config: Option<&str>,
    json: bool,
) -> Result<()> {
    let host = match validate_url(url) {
        Ok(h) => h,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "url": url, "error": e.to_string()}));
            }
            return Err(e);
        }
    };

    // Resolve the signing key + allowlist: an existing config target matching the
    // URL supplies both by default; --signing-key / --allowed-host override/augment.
    // The config is best-effort here (unlike `add`/`list`, `test` does not require
    // one — an operator can smoke-test an ad-hoc URL with an explicit key).
    let cfg_from_file = resolve_config_path(config)
        .ok()
        .and_then(|p| read_config(&p).ok());
    let matching = cfg_from_file
        .as_ref()
        .and_then(|c| c.targets.iter().find(|t| t.url == url));

    let key = signing_key
        .map(|k| k.to_string())
        .or_else(|| matching.map(|t| t.signing_key.clone()))
        .unwrap_or_else(|| "test-key".to_string());

    // Build the effective allowlist: config's + operator extras. The URL's own
    // host is NOT auto-added — `test` mirrors production deny-by-default, so a host
    // absent from the allowlist refuses LOUDLY (that IS the useful signal: "this
    // target would be SSRF-refused in production"). Permit an ad-hoc host with
    // --allowed-host.
    let mut effective = WebhookConfig::default();
    if let Some(c) = &cfg_from_file {
        effective.allowed_hosts = c.allowed_hosts.clone();
    }
    for h in &allowed_hosts {
        ensure_host_allowed(&mut effective, h);
    }
    let host_allowlisted = effective.allowed_hosts.iter().any(|h| h == &host);

    let sample_topic = topic.unwrap_or("webhook-test");
    let body = serde_json::json!({
        "topic": sample_topic,
        "kind": "webhook-test",
        "note": "termlink webhook test — sample payload (T-2336)",
    });
    let body_bytes = serde_json::to_vec(&body)?;

    let target = WebhookTarget {
        url: url.to_string(),
        signing_key: key,
        topics: vec![sample_topic.to_string()],
    };

    let client = webhook::build_test_client()?;
    let result = webhook::dispatch(&client, &effective, &target, &body_bytes).await;

    match result {
        Ok(status) => {
            if json {
                println!("{}", serde_json::json!({"ok": true, "url": url, "http_status": status, "topic": sample_topic}));
            } else {
                println!("✓ dispatched to {url}");
                println!("  http_status: {status}");
                println!("  topic:       {sample_topic}");
                println!("  signature:   HMAC-SHA256 sent as X-Termlink-Signature");
            }
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false, "url": url, "error": msg,
                    "host": host, "host_allowlisted": host_allowlisted,
                }));
            }
            // SSRF refusal is loud, not swallowed.
            if !host_allowlisted {
                anyhow::bail!(
                    "webhook test failed: {}\n  hint: host '{}' is not in the allowlist — this target would be SSRF-refused in production.\n        add it with: termlink webhook add --url {} --allowed-host {}\n        or permit it for this test only: termlink webhook test --url {} --allowed-host {}",
                    msg, host, url, host, url, host
                );
            }
            anyhow::bail!("webhook test failed: {}", msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_target(url: &str) -> WebhookTarget {
        WebhookTarget { url: url.to_string(), signing_key: "k".into(), topics: vec!["*".into()] }
    }

    #[test]
    fn validate_url_accepts_https_and_returns_host() {
        assert_eq!(validate_url("https://hooks.example.com/x").unwrap(), "hooks.example.com");
        assert_eq!(validate_url("http://10.0.0.5:8080/y").unwrap(), "10.0.0.5");
    }

    #[test]
    fn validate_url_rejects_non_http_and_hostless() {
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("not a url").is_err());
        assert!(validate_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn add_to_empty_config_adds_target_and_host() {
        let mut cfg = WebhookConfig::default();
        assert!(!cfg.is_enabled());
        let host = validate_url("https://hooks.example.com/hook").unwrap();
        let added = ensure_host_allowed(&mut cfg, &host);
        merge_target(&mut cfg, mk_target("https://hooks.example.com/hook"));
        assert!(added);
        assert!(cfg.is_enabled());
        assert_eq!(cfg.targets.len(), 1);
        assert_eq!(cfg.allowed_hosts, vec!["hooks.example.com"]);
    }

    #[test]
    fn ensure_host_allowed_is_idempotent_no_dup() {
        let mut cfg = WebhookConfig::default();
        assert!(ensure_host_allowed(&mut cfg, "a.example.com"));
        assert!(!ensure_host_allowed(&mut cfg, "a.example.com"));
        assert!(!ensure_host_allowed(&mut cfg, "a.example.com"));
        assert_eq!(cfg.allowed_hosts, vec!["a.example.com"]);
    }

    #[test]
    fn redact_key_hides_secret_but_signals_presence() {
        assert_eq!(redact_key(""), "<unset>");
        let r = redact_key("supersecretkey");
        assert!(!r.contains("supersecretkey"));
        assert!(r.contains("14")); // length hint
    }

    #[test]
    fn read_config_missing_file_is_empty_not_error() {
        let p = Path::new("/nonexistent/definitely/not/here/webhook.json");
        let cfg = read_config(p).unwrap();
        assert!(!cfg.is_enabled());
        assert_eq!(cfg.targets.len(), 0);
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = std::env::temp_dir().join(format!("tl-webhook-test-{}", std::process::id()));
        let path = dir.join("webhook.json");
        let mut cfg = WebhookConfig::default();
        ensure_host_allowed(&mut cfg, "hooks.example.com");
        merge_target(&mut cfg, mk_target("https://hooks.example.com/z"));
        write_config_atomic(&path, &cfg).unwrap();
        let back = read_config(&path).unwrap();
        assert_eq!(back.targets.len(), 1);
        assert_eq!(back.allowed_hosts, vec!["hooks.example.com"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_config_path_prefers_flag() {
        let p = resolve_config_path(Some("/tmp/explicit.json")).unwrap();
        assert_eq!(p, PathBuf::from("/tmp/explicit.json"));
    }
}
