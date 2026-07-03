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
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

/// Number of configured webhook targets (0 when the subsystem is disabled).
/// Surfaced via `hub.governor_status` (T-2335) so operators can confirm the
/// hub actually loaded the targets they configured in `TERMLINK_WEBHOOK_CONFIG`
/// — a `webhook_enabled=true` with `webhook_target_count=0` is impossible by
/// construction ([`WebhookConfig::is_enabled`] requires ≥1 target), so the pair
/// disambiguates "config path unset" from "config loaded but empty".
pub fn target_count() -> usize {
    webhooks().map(|rt| rt.cfg.targets.len()).unwrap_or(0)
}

/// Fan a hub event out to every configured target subscribed to `topic`.
///
/// Fire-and-forget: each matching target's first [`dispatch`] runs inline in its
/// own spawned task, so a slow or unreachable endpoint never blocks the
/// `channel.post` response. No-op (returns immediately) when the subsystem is
/// disabled or no target matches the topic. A *retryable* failure (5xx / transport
/// error) is enqueued into the retry queue (Slice 3, T-2334); a *permanent* failure
/// (4xx / config error) is dropped and logged, never retried.
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
        // prior_attempts = 0: this is the first (inline) attempt.
        tokio::spawn(dispatch_once_and_handle(client, cfg, target, topic, body, 0));
    }
}

// ── Slice 3 (T-2334): retry / backoff / dead-letter ────────────────────────
//
// An in-memory bounded retry queue with per-entry exponential backoff + jitter.
// Reuses the *shape* of the T-2051 offline-queue flush loop (an `attempts`
// counter + a jittered periodic drain + poison→dead-letter after N attempts)
// WITHOUT its SQLite store, keeping a `Mutex<Connection>` off the hot
// `channel.post` path. Deliberate tradeoff (PL-111): in-flight retries do NOT
// survive a hub restart — acceptable for best-effort/opt-in outbound webhooks.

/// Max delivery attempts before an entry is dead-lettered (T-2051 POISON_THRESHOLD
/// analog). Attempt 1 is the inline `fan_out` try; attempts 2..=MAX are retries.
const WEBHOOK_MAX_ATTEMPTS: u32 = 5;
/// Default retry-queue capacity (env `TERMLINK_WEBHOOK_RETRY_CAP` overrides).
const DEFAULT_RETRY_CAP: usize = 1000;
/// How many dead-letter records to retain for observability (bounded ring).
const DEAD_LETTER_RING_CAP: usize = 100;
/// Exponential-backoff base (attempt 1 waits ~this long before retry).
const BACKOFF_BASE_MS: u64 = 1000;
/// Exponential-backoff ceiling — a single retry never waits longer than this.
const BACKOFF_CAP_MS: u64 = 60_000;

/// Classification of a single dispatch result, deciding retry policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// 2xx — delivered.
    Success,
    /// 4xx or a config-level error (bad URL / non-allowlisted host). Retrying
    /// cannot fix it, so the entry is dropped, not retried.
    PermanentDrop,
    /// 5xx (or 1xx/3xx) or a transport error — worth retrying with backoff.
    Retryable,
}

/// Pure classification of a [`dispatch`] result into a [`DispatchOutcome`].
/// 2xx ⇒ Success; 4xx ⇒ PermanentDrop; anything else Ok ⇒ Retryable;
/// config errors (`InvalidUrl`/`HostNotAllowed`) ⇒ PermanentDrop; transport
/// errors (`Http`) ⇒ Retryable.
pub fn classify_outcome(result: &Result<u16, WebhookError>) -> DispatchOutcome {
    match result {
        Ok(s) if (200..300).contains(s) => DispatchOutcome::Success,
        Ok(s) if (400..500).contains(s) => DispatchOutcome::PermanentDrop,
        Ok(_) => DispatchOutcome::Retryable,
        Err(WebhookError::InvalidUrl(_)) | Err(WebhookError::HostNotAllowed(_)) => {
            DispatchOutcome::PermanentDrop
        }
        Err(WebhookError::Http(_)) => DispatchOutcome::Retryable,
    }
}

/// Deterministic exponential-backoff base for `attempts` (no jitter): monotonic
/// non-decreasing, capped at [`BACKOFF_CAP_MS`]. `attempts` is clamped before the
/// shift so a large value can never overflow the `1 << n`.
pub fn backoff_base_ms(attempts: u32) -> u64 {
    let shift = attempts.min(20);
    BACKOFF_BASE_MS.saturating_mul(1u64 << shift).min(BACKOFF_CAP_MS)
}

/// Apply ±25% jitter to a backoff `base`, decorrelating retries across targets
/// (T-2055 thundering-herd guard) using a cheap wall-clock-nanos entropy source —
/// no `rand` crate dependency. Result stays within `[base - base/4, base + base/4]`.
pub fn jitter_ms(base: u64) -> u64 {
    if base == 0 {
        return 0;
    }
    let span = base / 2; // full jitter window = 50% of base, centred on base
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let delta = nanos % (span + 1); // [0, span]
    base.saturating_sub(base / 4).saturating_add(delta)
}

/// Full backoff delay for `attempts`: jittered exponential base.
pub fn backoff_delay_ms(attempts: u32) -> u64 {
    jitter_ms(backoff_base_ms(attempts))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// One queued retry: a target + payload + how many attempts have been made + the
/// absolute wall-clock time at which the next attempt is due.
#[derive(Debug, Clone)]
struct RetryEntry {
    target: WebhookTarget,
    topic: String,
    body: Vec<u8>,
    attempts: u32,
    next_attempt_ms: u64,
}

/// A terminally-failed delivery retained for observability (Slice 4 surface).
#[derive(Debug, Clone)]
pub struct DeadLetter {
    pub url: String,
    pub topic: String,
    pub attempts: u32,
    pub reason: String,
    pub ts_ms: u64,
}

/// In-memory bounded retry queue with counters. Process-global via
/// [`retry_queue`]. All state is behind mutexes; the hot path only touches it on
/// a *failed* first attempt, so the common (success / no-webhook) path is untouched.
pub struct RetryQueue {
    inner: Mutex<VecDeque<RetryEntry>>,
    dead_letters: Mutex<VecDeque<DeadLetter>>,
    cap: usize,
    enqueued_total: AtomicU64,
    retry_success_total: AtomicU64,
    dropped_full_total: AtomicU64,
    dead_letter_total: AtomicU64,
}

impl RetryQueue {
    fn new(cap: usize) -> Self {
        RetryQueue {
            inner: Mutex::new(VecDeque::new()),
            dead_letters: Mutex::new(VecDeque::new()),
            cap,
            enqueued_total: AtomicU64::new(0),
            retry_success_total: AtomicU64::new(0),
            dropped_full_total: AtomicU64::new(0),
            dead_letter_total: AtomicU64::new(0),
        }
    }

    /// Enqueue a retry entry. Returns `Err(())` (and bumps `dropped_full_total`)
    /// when the queue is at capacity — a loud, counted drop, never a silent one.
    fn enqueue(&self, entry: RetryEntry) -> Result<(), ()> {
        let mut q = self.inner.lock().unwrap();
        if q.len() >= self.cap {
            self.dropped_full_total.fetch_add(1, Ordering::Relaxed);
            return Err(());
        }
        q.push_back(entry);
        self.enqueued_total.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Remove and return every entry whose `next_attempt_ms <= now_ms`, leaving
    /// not-yet-due entries in place (order preserved).
    fn drain_due(&self, now_ms: u64) -> Vec<RetryEntry> {
        let mut q = self.inner.lock().unwrap();
        let mut due = Vec::new();
        let mut kept = VecDeque::with_capacity(q.len());
        while let Some(e) = q.pop_front() {
            if e.next_attempt_ms <= now_ms {
                due.push(e);
            } else {
                kept.push_back(e);
            }
        }
        *q = kept;
        due
    }

    fn record_retry_success(&self) {
        self.retry_success_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Terminally fail an entry: retain a bounded dead-letter record + bump the
    /// counter. Drops the oldest record when the ring is full.
    fn dead_letter(&self, entry: &RetryEntry, reason: &str) {
        self.dead_letter_total.fetch_add(1, Ordering::Relaxed);
        let mut dl = self.dead_letters.lock().unwrap();
        if dl.len() >= DEAD_LETTER_RING_CAP {
            dl.pop_front();
        }
        dl.push_back(DeadLetter {
            url: entry.target.url.clone(),
            topic: entry.topic.clone(),
            attempts: entry.attempts,
            reason: reason.to_string(),
            ts_ms: now_ms(),
        });
    }

    /// Current number of entries awaiting retry.
    pub fn depth(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
    pub fn enqueued_total(&self) -> u64 {
        self.enqueued_total.load(Ordering::Relaxed)
    }
    pub fn retry_success_total(&self) -> u64 {
        self.retry_success_total.load(Ordering::Relaxed)
    }
    pub fn dropped_full_total(&self) -> u64 {
        self.dropped_full_total.load(Ordering::Relaxed)
    }
    pub fn dead_letter_total(&self) -> u64 {
        self.dead_letter_total.load(Ordering::Relaxed)
    }
    /// Snapshot of retained dead-letter records (most-recent last).
    pub fn dead_letters(&self) -> Vec<DeadLetter> {
        self.dead_letters.lock().unwrap().iter().cloned().collect()
    }
}

static RETRY_QUEUE: OnceLock<RetryQueue> = OnceLock::new();

fn parse_retry_cap() -> usize {
    std::env::var("TERMLINK_WEBHOOK_RETRY_CAP")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(DEFAULT_RETRY_CAP)
}

/// Access the process-global retry queue (lazy-init with the env-configured cap).
pub fn retry_queue() -> &'static RetryQueue {
    RETRY_QUEUE.get_or_init(|| RetryQueue::new(parse_retry_cap()))
}

/// Schedule a retry for a target that just failed with a [`DispatchOutcome::Retryable`]
/// result after `attempts` total attempts. Dead-letters instead when `attempts`
/// has reached [`WEBHOOK_MAX_ATTEMPTS`], or when the queue is full.
fn schedule_retry(target: WebhookTarget, topic: String, body: Vec<u8>, attempts: u32) {
    let q = retry_queue();
    if attempts >= WEBHOOK_MAX_ATTEMPTS {
        let entry = RetryEntry {
            target,
            topic,
            body,
            attempts,
            next_attempt_ms: 0,
        };
        q.dead_letter(&entry, "max attempts exhausted");
        tracing::warn!(
            url = %entry.target.url, topic = %entry.topic, attempts,
            "webhook dead-lettered — max attempts exhausted"
        );
        return;
    }
    let next_attempt_ms = now_ms() + backoff_delay_ms(attempts);
    let entry = RetryEntry {
        target,
        topic,
        body,
        attempts,
        next_attempt_ms,
    };
    if q.enqueue(entry.clone()).is_err() {
        tracing::warn!(
            url = %entry.target.url, topic = %entry.topic,
            "webhook retry dropped — retry queue full (TERMLINK_WEBHOOK_RETRY_CAP)"
        );
    }
}

/// Dispatch once and route the outcome: record success, drop a permanent failure,
/// or schedule a retry (bumping the attempt count). Shared by the inline
/// [`fan_out`] first attempt (`prior_attempts = 0`) and the drain-loop retries.
async fn dispatch_once_and_handle(
    client: reqwest::Client,
    cfg: WebhookConfig,
    target: WebhookTarget,
    topic: String,
    body: Vec<u8>,
    prior_attempts: u32,
) {
    let result = dispatch(&client, &cfg, &target, &body).await;
    match classify_outcome(&result) {
        DispatchOutcome::Success => {
            if prior_attempts > 0 {
                retry_queue().record_retry_success();
            }
            tracing::debug!(topic = %topic, url = %target.url, "webhook delivered");
        }
        DispatchOutcome::PermanentDrop => {
            tracing::warn!(
                topic = %topic, url = %target.url, ?result,
                "webhook permanently failed (4xx / config) — dropped, no retry"
            );
        }
        DispatchOutcome::Retryable => {
            schedule_retry(target, topic, body, prior_attempts + 1);
        }
    }
}

/// Spawn the background retry-drain loop (mirror of
/// [`crate::governor::spawn_rate_evict_loop`]). Every tick it drains due entries
/// and re-dispatches each in its own task. Idles cheaply when the subsystem is
/// disabled. Interval tunable via `TERMLINK_WEBHOOK_RETRY_INTERVAL_MS`
/// (clamped 250..=60000, default 2000). Must be called from within a Tokio runtime.
pub fn spawn_retry_loop() {
    let interval_ms = std::env::var("TERMLINK_WEBHOOK_RETRY_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(2000)
        .clamp(250, 60_000);
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms));
        loop {
            ticker.tick().await;
            let Some(rt) = webhooks() else { continue };
            let due = retry_queue().drain_due(now_ms());
            for entry in due {
                let client = rt.client.clone();
                let cfg = rt.cfg.clone();
                tokio::spawn(dispatch_once_and_handle(
                    client,
                    cfg,
                    entry.target,
                    entry.topic,
                    entry.body,
                    entry.attempts,
                ));
            }
        }
    });
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

    // ── Slice 3 (T-2334): retry / backoff / dead-letter ────────────────

    #[test]
    fn classify_outcome_maps_status_and_errors() {
        assert_eq!(classify_outcome(&Ok(200)), DispatchOutcome::Success);
        assert_eq!(classify_outcome(&Ok(204)), DispatchOutcome::Success);
        // 4xx is permanent — retrying won't fix a bad/unauthorized request.
        assert_eq!(classify_outcome(&Ok(400)), DispatchOutcome::PermanentDrop);
        assert_eq!(classify_outcome(&Ok(404)), DispatchOutcome::PermanentDrop);
        // 5xx is retryable.
        assert_eq!(classify_outcome(&Ok(500)), DispatchOutcome::Retryable);
        assert_eq!(classify_outcome(&Ok(503)), DispatchOutcome::Retryable);
        // Config errors are permanent; transport errors are retryable.
        assert_eq!(
            classify_outcome(&Err(WebhookError::HostNotAllowed("x".into()))),
            DispatchOutcome::PermanentDrop
        );
        assert_eq!(
            classify_outcome(&Err(WebhookError::InvalidUrl("x".into()))),
            DispatchOutcome::PermanentDrop
        );
        assert_eq!(
            classify_outcome(&Err(WebhookError::Http("timeout".into()))),
            DispatchOutcome::Retryable
        );
    }

    #[test]
    fn backoff_base_is_monotonic_and_capped() {
        assert_eq!(backoff_base_ms(0), 1000);
        assert_eq!(backoff_base_ms(1), 2000);
        assert_eq!(backoff_base_ms(2), 4000);
        assert_eq!(backoff_base_ms(3), 8000);
        // Monotonic non-decreasing all the way to (and stuck at) the cap.
        let mut prev = 0;
        for n in 0..40u32 {
            let cur = backoff_base_ms(n);
            assert!(cur >= prev, "backoff must be non-decreasing at n={n}");
            assert!(cur <= BACKOFF_CAP_MS, "backoff must never exceed the cap");
            prev = cur;
        }
        // Large attempt counts saturate at the cap (no shift overflow / panic).
        assert_eq!(backoff_base_ms(1000), BACKOFF_CAP_MS);
    }

    #[test]
    fn jitter_stays_within_25_percent_bounds() {
        let base = 8000u64;
        for _ in 0..1000 {
            let j = jitter_ms(base);
            // delta ∈ [0, base/2]; result = (base - base/4) + delta ⇒ [base-25%, base+25%].
            assert!(
                j >= base - base / 4 && j <= base + base / 4,
                "jitter {j} out of bounds for base {base}"
            );
        }
        assert_eq!(jitter_ms(0), 0);
    }

    fn retry_entry(url: &str, attempts: u32, next_ms: u64) -> RetryEntry {
        RetryEntry {
            target: target(url, &["*"]),
            topic: "t".to_string(),
            body: b"{}".to_vec(),
            attempts,
            next_attempt_ms: next_ms,
        }
    }

    #[test]
    fn retry_queue_enqueue_rejects_when_full() {
        let q = RetryQueue::new(2);
        assert!(q.enqueue(retry_entry("https://h/a", 1, 0)).is_ok());
        assert!(q.enqueue(retry_entry("https://h/b", 1, 0)).is_ok());
        // Third enqueue over cap=2 is a loud, counted drop.
        assert!(q.enqueue(retry_entry("https://h/c", 1, 0)).is_err());
        assert_eq!(q.depth(), 2);
        assert_eq!(q.enqueued_total(), 2);
        assert_eq!(q.dropped_full_total(), 1);
    }

    #[test]
    fn retry_queue_drain_due_selects_only_ready_entries() {
        let q = RetryQueue::new(10);
        q.enqueue(retry_entry("https://h/now", 1, 100)).unwrap();
        q.enqueue(retry_entry("https://h/later", 1, 5000)).unwrap();
        // now_ms = 1000: the first entry is due, the second is not.
        let due = q.drain_due(1000);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].target.url, "https://h/now");
        // The not-yet-due entry remains queued.
        assert_eq!(q.depth(), 1);
    }

    #[test]
    fn dead_letter_records_and_counts() {
        let q = RetryQueue::new(10);
        let e = retry_entry("https://h/dead", WEBHOOK_MAX_ATTEMPTS, 0);
        q.dead_letter(&e, "max attempts exhausted");
        assert_eq!(q.dead_letter_total(), 1);
        let dl = q.dead_letters();
        assert_eq!(dl.len(), 1);
        assert_eq!(dl[0].url, "https://h/dead");
        assert_eq!(dl[0].attempts, WEBHOOK_MAX_ATTEMPTS);
        assert_eq!(dl[0].reason, "max attempts exhausted");
    }

    #[test]
    fn dead_letter_ring_is_bounded() {
        let q = RetryQueue::new(10);
        for i in 0..(DEAD_LETTER_RING_CAP + 20) {
            let e = retry_entry(&format!("https://h/{i}"), WEBHOOK_MAX_ATTEMPTS, 0);
            q.dead_letter(&e, "x");
        }
        // Ring is capped; total counter still reflects every dead-letter.
        assert_eq!(q.dead_letters().len(), DEAD_LETTER_RING_CAP);
        assert_eq!(q.dead_letter_total() as usize, DEAD_LETTER_RING_CAP + 20);
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
