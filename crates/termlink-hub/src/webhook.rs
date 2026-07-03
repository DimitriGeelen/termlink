//! Outbound webhook fan-out — Slice 1 (T-2332, descends from the T-2331 GO).
//!
//! **SEND PRIMITIVE ONLY.** Security-first, opt-in. This slice delivers the
//! signed + allowlisted outbound POST and nothing else:
//!   - HMAC-SHA256 signed payloads (`X-Termlink-Signature: sha256=<hex>`)
//!   - deny-by-default host allowlist (SSRF guard)
//!
//! Slice 2 (T-2333) wires the primitive to real hub events: a per-target topic
//! filter ([`WebhookTarget::topics`] / [`WebhookConfig::targets_for`]), a
//! process-global runtime loaded at hub startup from `TERMLINK_WEBHOOK_CONFIG`
//! ([`init`] / [`webhooks`]), and a fire-and-forget [`fan_out`] invoked from the
//! `channel.post` `Ok(offset)` arm.
//!
//! Explicitly OUT of scope here (later slices):
//!   - retry / backoff / dead-letter (Slice 3, will reuse the T-2051 queue pattern)
//!   - CLI config verbs + observability counters (Slice 4)
//!
//! Portability (Directive 4): outbound HTTP must never become a hard dependency of
//! the substrate. Zero configured targets ⇒ [`WebhookConfig::is_enabled`] is false
//! and nothing dispatches — no behaviour change for a hub with no webhooks.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::OnceLock;
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;

/// A single external HTTP endpoint the hub may POST to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTarget {
    /// Absolute `http`/`https` URL to POST to. Its host must appear in
    /// [`WebhookConfig::allowed_hosts`] or dispatch is refused.
    pub url: String,
    /// Opaque secret used to HMAC-SHA256-sign the payload body. Distinct from the
    /// peer-auth `hub.secret` — a compromised webhook key must not grant hub auth.
    pub signing_key: String,
    /// Topics that trigger this target. A post on `topic` fires this target iff
    /// `topics` contains that exact topic OR the `"*"` wildcard. An empty list
    /// never fires — opt-in by construction (mirrors the deny-by-default host
    /// allowlist). Slice 2 (T-2333).
    #[serde(default)]
    pub topics: Vec<String>,
}

impl WebhookTarget {
    /// True iff a post on `topic` should fan out to this target: exact membership
    /// in [`WebhookTarget::topics`] or the `"*"` wildcard. Empty ⇒ never.
    pub fn matches_topic(&self, topic: &str) -> bool {
        self.topics.iter().any(|t| t == "*" || t == topic)
    }
}

/// Hub-level webhook configuration. Deny-by-default: a target only dispatches if
/// its URL host is an exact member of `allowed_hosts`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Exact-match allowlist of permitted URL hosts (SSRF guard). A host absent
    /// here is refused. Empty ⇒ nothing can dispatch.
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    /// Configured targets. Empty ⇒ the feature is disabled (opt-in).
    #[serde(default)]
    pub targets: Vec<WebhookTarget>,
}

impl WebhookConfig {
    /// True only when at least one target is configured. A hub with no targets is
    /// unaffected by the webhook subsystem (opt-in / no hard dependency).
    pub fn is_enabled(&self) -> bool {
        !self.targets.is_empty()
    }

    /// Targets that should fan out for a post on `topic` (Slice 2, T-2333).
    /// Selection only — the host-allowlist SSRF guard still runs per target
    /// inside [`dispatch`], so a topic match never bypasses the allowlist.
    pub fn targets_for(&self, topic: &str) -> Vec<&WebhookTarget> {
        self.targets
            .iter()
            .filter(|t| t.matches_topic(topic))
            .collect()
    }
}

/// Process-global webhook runtime: the parsed config plus a shared, bounded-timeout
/// HTTP client. Installed once at hub startup by [`init`]. Absent ⇒ the subsystem
/// is disabled and [`fan_out`] is a no-op (opt-in / no hard dependency).
pub struct WebhookRuntime {
    cfg: WebhookConfig,
    client: reqwest::Client,
}

static WEBHOOKS: OnceLock<Option<WebhookRuntime>> = OnceLock::new();

/// Bounded per-request timeout. External endpoints must never let a hung POST
/// pin a spawned dispatch task indefinitely.
const WEBHOOK_TIMEOUT_SECS: u64 = 10;

/// Install the process-global webhook runtime from `TERMLINK_WEBHOOK_CONFIG`
/// (a path to a JSON [`WebhookConfig`]). Idempotent (`OnceLock` semantics).
///
/// Failure is always soft — a missing env var, unreadable file, unparseable
/// JSON, or a config with no targets all resolve to a DISABLED subsystem with
/// NO panic. Outbound HTTP is opt-in and must never be a hard dependency of the
/// substrate (Directive 4). Called from hub startup alongside
/// [`crate::dedupe::init`] / [`crate::cv_index::init`].
pub fn init() {
    let runtime = load_runtime_from_env();
    let enabled = runtime.is_some();
    let _ = WEBHOOKS.set(runtime);
    if enabled {
        tracing::info!("Hub webhook fan-out active (T-2333 — TERMLINK_WEBHOOK_CONFIG configured)");
    } else {
        tracing::debug!("Hub webhook fan-out disabled (no TERMLINK_WEBHOOK_CONFIG)");
    }
}

/// Ensure the process-wide rustls crypto provider is installed (aws-lc-rs — the
/// same backend the hub's TLS stack uses). `reqwest` is built with the
/// `rustls-tls-webpki-roots-no-provider` feature so it does NOT pull in a second
/// `ring` provider (which would make rustls's process-default ambiguous and panic
/// the `tls::` tests). The trade-off is that reqwest then needs the process-default
/// provider installed explicitly before it builds any client, or it panics
/// "No provider set". Idempotent — a no-op if another component already installed
/// one (its `Err` is intentionally ignored).
fn ensure_crypto_provider() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

/// Build the runtime from env, or `None` when disabled for ANY reason.
fn load_runtime_from_env() -> Option<WebhookRuntime> {
    let path = std::env::var("TERMLINK_WEBHOOK_CONFIG").ok()?;
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(path = %path, error = %e, "webhook config unreadable — subsystem disabled");
            return None;
        }
    };
    let cfg: WebhookConfig = match serde_json::from_str(&raw) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(path = %path, error = %e, "webhook config unparseable — subsystem disabled");
            return None;
        }
    };
    if !cfg.is_enabled() {
        return None;
    }
    // reqwest (no-provider feature) needs the process-default crypto provider
    // installed before it can build a TLS-capable client.
    ensure_crypto_provider();
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(WEBHOOK_TIMEOUT_SECS))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "webhook HTTP client build failed — subsystem disabled");
            return None;
        }
    };
    Some(WebhookRuntime { cfg, client })
}

/// Access the global runtime, or `None` when the subsystem is disabled (either
/// [`init`] was never called, or it resolved to disabled).
pub fn webhooks() -> Option<&'static WebhookRuntime> {
    WEBHOOKS.get().and_then(|o| o.as_ref())
}

/// Fan a hub event out to every configured target subscribed to `topic`.
///
/// Fire-and-forget: each matching target's [`dispatch`] runs in its own spawned
/// task, so a slow or unreachable endpoint never blocks the `channel.post`
/// response. No-op (returns immediately) when the subsystem is disabled or no
/// target matches the topic. Dispatch outcomes are logged via `tracing`.
pub fn fan_out(topic: &str, body: Vec<u8>) {
    let Some(rt) = webhooks() else { return };
    let targets = rt.cfg.targets_for(topic);
    if targets.is_empty() {
        return;
    }
    for target in targets {
        let client = rt.client.clone();
        let cfg = rt.cfg.clone();
        let target = target.clone();
        let body = body.clone();
        let topic = topic.to_string();
        tokio::spawn(async move {
            match dispatch(&client, &cfg, &target, &body).await {
                Ok(status) => tracing::debug!(
                    topic = %topic, url = %target.url, status,
                    "webhook dispatched"
                ),
                Err(e) => tracing::warn!(
                    topic = %topic, url = %target.url, error = %e,
                    "webhook dispatch failed"
                ),
            }
        });
    }
}

/// Failure modes for a single dispatch attempt.
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    /// The target URL did not parse or had no host component.
    #[error("invalid target url: {0}")]
    InvalidUrl(String),
    /// The target host was not on the allowlist (SSRF guard fired). No network
    /// call was made.
    #[error("host not allowlisted (SSRF guard): {0}")]
    HostNotAllowed(String),
    /// The underlying HTTP request failed.
    #[error("http error: {0}")]
    Http(String),
}

/// Compute the `sha256=<hex>` signature header value for a payload `body`.
///
/// HMAC-SHA256 keyed by `signing_key`. External consumers recompute this over the
/// received body and compare to the `X-Termlink-Signature` header to verify the
/// payload originated from the hub.
pub fn sign_payload(signing_key: &str, body: &[u8]) -> String {
    // HMAC accepts a key of any length, so this never fails.
    let mut mac =
        HmacSha256::new_from_slice(signing_key.as_bytes()).expect("HMAC accepts any key length");
    mac.update(body);
    let tag = mac.finalize().into_bytes();
    let mut hex = String::with_capacity(7 + tag.len() * 2);
    hex.push_str("sha256=");
    for b in tag.iter() {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

/// Extract the host from a URL, or `None` on parse failure / missing host.
fn url_host(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()?
        .host_str()
        .map(|h| h.to_string())
}

/// Deny-by-default allowlist check (SSRF guard). A URL is permitted only if its
/// host is an **exact** member of `allowed_hosts` — no suffix/substring matching,
/// so `hooks.example.com.evil.com` never matches `hooks.example.com`.
pub fn host_allowed(url: &str, allowed_hosts: &[String]) -> bool {
    match url_host(url) {
        Some(h) => allowed_hosts.iter().any(|a| a == &h),
        None => false,
    }
}

/// POST `body` to `target`, HMAC-signed, iff the target host is allowlisted.
///
/// The allowlist check happens **before** any network activity, so a
/// non-allowlisted target (e.g. a cloud-metadata SSRF probe) is refused without
/// the hub ever opening a connection. Returns the HTTP status code on success.
pub async fn dispatch(
    client: &reqwest::Client,
    cfg: &WebhookConfig,
    target: &WebhookTarget,
    body: &[u8],
) -> Result<u16, WebhookError> {
    if url_host(&target.url).is_none() {
        return Err(WebhookError::InvalidUrl(target.url.clone()));
    }
    // SSRF guard fires here — before .send() — so no connection is attempted.
    if !host_allowed(&target.url, &cfg.allowed_hosts) {
        return Err(WebhookError::HostNotAllowed(target.url.clone()));
    }
    let sig = sign_payload(&target.signing_key, body);
    let resp = client
        .post(&target.url)
        .header("content-type", "application/json")
        .header("x-termlink-signature", sig)
        .body(body.to_vec())
        .send()
        .await
        .map_err(|e| WebhookError::Http(e.to_string()))?;
    Ok(resp.status().as_u16())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_payload_matches_canonical_vector() {
        // Canonical HMAC-SHA256 test vector (Wikipedia HMAC example):
        //   key="key", msg="The quick brown fox jumps over the lazy dog"
        let sig = sign_payload("key", b"The quick brown fox jumps over the lazy dog");
        assert_eq!(
            sig,
            "sha256=f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn sign_payload_is_deterministic_and_sensitive() {
        assert_eq!(sign_payload("k", b"a"), sign_payload("k", b"a"));
        assert_ne!(sign_payload("k", b"a"), sign_payload("k", b"b")); // body-sensitive
        assert_ne!(sign_payload("k1", b"a"), sign_payload("k2", b"a")); // key-sensitive
    }

    #[test]
    fn host_allowed_denies_by_default() {
        let allow: Vec<String> = vec![];
        assert!(!host_allowed("https://example.com/hook", &allow));
    }

    #[test]
    fn host_allowed_exact_match_only() {
        let allow = vec!["hooks.example.com".to_string()];
        assert!(host_allowed("https://hooks.example.com/x", &allow));
        assert!(!host_allowed("https://evil.example.com/x", &allow));
        // Suffix attack must NOT match — exact host equality only.
        assert!(!host_allowed("https://hooks.example.com.evil.com/x", &allow));
    }

    #[test]
    fn host_allowed_refuses_garbage_url() {
        let allow = vec!["example.com".to_string()];
        assert!(!host_allowed("not a url", &allow));
        assert!(!host_allowed("", &allow));
    }

    #[tokio::test]
    async fn dispatch_refuses_non_allowlisted_without_network() {
        // 169.254.169.254 is the cloud-metadata SSRF target. With an empty
        // allowlist it must be refused BEFORE any connection is attempted.
        // Install the crypto provider first — reqwest's no-provider feature
        // needs it before building a client (mirrors load_runtime_from_env).
        ensure_crypto_provider();
        let client = reqwest::Client::new();
        let cfg = WebhookConfig::default();
        let target = WebhookTarget {
            url: "http://169.254.169.254/latest/meta-data".to_string(),
            signing_key: "k".to_string(),
            topics: vec!["*".to_string()],
        };
        let err = dispatch(&client, &cfg, &target, b"x").await.unwrap_err();
        assert!(
            matches!(err, WebhookError::HostNotAllowed(_)),
            "expected HostNotAllowed, got {err:?}"
        );
    }

    #[test]
    fn config_disabled_when_no_targets() {
        assert!(!WebhookConfig::default().is_enabled());
    }

    #[test]
    fn config_parses_from_json() {
        let json = r#"{
            "allowed_hosts": ["hooks.example.com"],
            "targets": [{"url": "https://hooks.example.com/x", "signing_key": "s"}]
        }"#;
        let cfg: WebhookConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.is_enabled());
        assert_eq!(cfg.allowed_hosts, vec!["hooks.example.com".to_string()]);
        assert_eq!(cfg.targets.len(), 1);
        assert_eq!(cfg.targets[0].url, "https://hooks.example.com/x");
        // topics defaults to empty when the field is absent (Slice 2).
        assert!(cfg.targets[0].topics.is_empty());
    }

    // ── Slice 2 (T-2333) ───────────────────────────────────────────────

    fn target(url: &str, topics: &[&str]) -> WebhookTarget {
        WebhookTarget {
            url: url.to_string(),
            signing_key: "s".to_string(),
            topics: topics.iter().map(|t| t.to_string()).collect(),
        }
    }

    #[test]
    fn matches_topic_exact_wildcard_and_empty() {
        assert!(target("https://h/x", &["work-queue"]).matches_topic("work-queue"));
        assert!(!target("https://h/x", &["work-queue"]).matches_topic("other"));
        // Wildcard matches anything.
        assert!(target("https://h/x", &["*"]).matches_topic("anything"));
        // Empty topics never fires — opt-in by construction.
        assert!(!target("https://h/x", &[]).matches_topic("work-queue"));
        // No prefix/substring matching — exact only.
        assert!(!target("https://h/x", &["work"]).matches_topic("work-queue"));
    }

    #[test]
    fn targets_for_selects_matching_only() {
        let cfg = WebhookConfig {
            allowed_hosts: vec!["h".to_string()],
            targets: vec![
                target("https://h/a", &["work-queue"]),
                target("https://h/b", &["*"]),
                target("https://h/c", &["other"]),
                target("https://h/d", &[]),
            ],
        };
        let hit: Vec<&str> = cfg
            .targets_for("work-queue")
            .iter()
            .map(|t| t.url.as_str())
            .collect();
        // a (exact) + b (wildcard); NOT c (other topic) or d (empty).
        assert_eq!(hit, vec!["https://h/a", "https://h/b"]);
        assert!(cfg.targets_for("nothing-matches").iter().all(|t| t.url == "https://h/b"));
    }

    #[test]
    fn webhooks_none_when_uninitialised_in_this_test_binary() {
        // init() is never called in the unit-test binary, so the accessor is
        // None and fan_out is a no-op — proving the opt-in default. (OnceLock is
        // process-global; asserting None here documents the disabled default
        // without depending on cross-test ordering, since no test calls init.)
        assert!(webhooks().is_none());
        // fan_out on a disabled subsystem returns immediately without spawning.
        fan_out("work-queue", b"{}".to_vec());
    }
}
