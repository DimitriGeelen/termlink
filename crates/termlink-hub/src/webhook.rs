//! Outbound webhook fan-out — Slice 1 (T-2332, descends from the T-2331 GO).
//!
//! **SEND PRIMITIVE ONLY.** Security-first, opt-in. This slice delivers the
//! signed + allowlisted outbound POST and nothing else:
//!   - HMAC-SHA256 signed payloads (`X-Termlink-Signature: sha256=<hex>`)
//!   - deny-by-default host allowlist (SSRF guard)
//!
//! Explicitly OUT of scope here (later slices):
//!   - event → webhook dispatch wiring (Slice 2)
//!   - retry / backoff / dead-letter (Slice 3, will reuse the T-2051 queue pattern)
//!   - CLI config verbs + observability counters (Slice 4)
//!
//! Portability (Directive 4): outbound HTTP must never become a hard dependency of
//! the substrate. Zero configured targets ⇒ [`WebhookConfig::is_enabled`] is false
//! and nothing dispatches — no behaviour change for a hub with no webhooks.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

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
        let client = reqwest::Client::new();
        let cfg = WebhookConfig::default();
        let target = WebhookTarget {
            url: "http://169.254.169.254/latest/meta-data".to_string(),
            signing_key: "k".to_string(),
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
    }
}
